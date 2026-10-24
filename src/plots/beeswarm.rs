//! Categorical beeswarm plot with a continuous color scale.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::alpha::{marks::ObservationAlpha, AlphaTarget, AlphaUnitKind};
use crate::layout::{Layout, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{with_clip, Backend, GradientStop, LineStyle, Point2D};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Error;
use crate::Plot;

#[derive(Debug)]
pub(crate) struct PreparedBeeswarmInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedBeeswarm)>,
}

#[derive(Debug)]
pub(crate) struct PreparedBeeswarm {
    pub(crate) levels: Vec<String>,
    pub(crate) x_data: (f64, f64),
    rows: Vec<(usize, f64, f64, ObservationAlpha)>,
    colors: Vec<crate::theme::OpaqueColor>,
    selection: crate::theme::ContinuousColorScale,
    scale: crate::theme::ContinuousColors,
    colorbar_stops: Vec<GradientStop>,
    values: Option<crate::numeric::set::FormattedSet>,
    format: Option<std::sync::Arc<crate::numeric::format::NumericFormat>>,
    pub(crate) color_label: crate::Text,
}

fn retained(chart: &Plot) -> Result<&PreparedBeeswarm, Error> {
    let inputs = chart
        .prepared_beeswarm
        .as_ref()
        .ok_or_else(|| Error::Render("Beeswarm requires root-prepared source rows".into()))?;
    crate::layout::interior::preference::panel(&inputs.panels, chart.axis_scope(), "Beeswarm")
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = retained(chart)?;
    if prepared.levels.is_empty()
        || prepared.rows.is_empty()
        || prepared.rows.len() != prepared.colors.len()
    {
        return Err(Error::Render(
            "Beeswarm source rows and prepared points differ".into(),
        ));
    }
    let point = chart.point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_beeswarm(),
    );
    let ink = crate::alpha::marks::point_ink(
        &point,
        prepared
            .rows
            .iter()
            .map(|&(_, x, _, alpha)| (x, 0.0, alpha)),
    )?;

    let pitch = 2.0 * quanta_pixels(packing_radius(chart, theme)?);
    for &(category, _, _, _) in &prepared.rows {
        if category >= prepared.levels.len() {
            return Err(Error::Render(
                "Beeswarm point category exceeds prepared levels".into(),
            ));
        }
    }
    let count = u32::try_from(prepared.levels.len())
        .map_err(|_| Error::Render("Beeswarm category count exceeds spatial range".into()))?;
    let height = pitch * f64::from(count);
    if !height.is_finite() {
        return Err(Error::Render(
            "Beeswarm category hard span overflowed".into(),
        ));
    }
    let mut numeric = ink.map_or(0.0, |ink| ink.dimensions().0);
    if chart.zero_line_visible() && prepared.x_data.0 <= 0.0 && prepared.x_data.1 >= 0.0 {
        numeric = numeric.max(crate::render::geometry::line_ink_width(
            &theme
                .marks
                .get_strokes()
                .get_references()
                .get_beeswarm_reference()
                .backend_style(),
        )?);
    }
    let axes =
        crate::layout::interior::coordinate::Intrinsics::prepared(chart, theme)?.minimum(chart);
    Ok((
        axes.0.max(if chart.user_x_limits().is_none() {
            numeric
        } else {
            0.0
        }),
        axes.1.max(height),
    ))
}

fn quanta_pixels(value: i64) -> f64 {
    value as f64 / 1024.0
}

fn packing_radius(chart: &Plot, theme: &crate::theme::ResolvedTheme) -> Result<i64, Error> {
    let mut style = chart.point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_beeswarm(),
    );
    // A data-alpha binding retains every observation's full glyph in the
    // scientific packing input, independent of which bytes are transparent.
    if !chart.alpha_inputs().get(AlphaTarget::Point).is_theme() {
        style.color = style.color.alpha(255);
    }
    packing::radius(&style)
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    selection: crate::theme::ContinuousColorScale,
) -> Result<PreparedBeeswarm, Error> {
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
    let category_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("beeswarm: categorical row column not set".into()))?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("beeswarm: x column not set".into()))?;
    let color_col = chart
        .color_col()
        .ok_or_else(|| Error::Data("beeswarm: continuous color column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_col, "beeswarm")?;
    let xs = crate::data::strict_finite_f64(data, x_col, "beeswarm")?;
    let colors = crate::data::strict_finite_f64(data, color_col, "beeswarm")?;
    if categories.len() != xs.len() || xs.len() != colors.len() || xs.is_empty() {
        return Err(Error::Data(
            "beeswarm: category/x/color columns must have equal non-zero lengths".into(),
        ));
    }
    let levels = crate::data::source::source_levels(&categories)?;
    data.validate_row_source()?;

    let x_data = crate::data::min_max(xs.iter().copied(), "beeswarm")?;
    let color_range = selection.prepared_domain(&colors, "beeswarm")?.as_pair();
    let mut indices = crate::data::source::collect_source(
        levels.iter().enumerate().map(|(i, s)| Ok((s.as_str(), i))),
    )?;
    crate::data::source::sort_source_by(&mut indices, |a, b| a.0.cmp(b.0));
    let mut rows =
        crate::data::source::collect_source(
            categories.iter().zip(&xs).zip(&colors).enumerate().map(
                |(row, ((category, x), color))| {
                    let index = indices
                        .binary_search_by_key(&category.as_str(), |entry| entry.0)
                        .map_err(|_| Error::Data("beeswarm: category identity missing".into()))?;
                    Ok((
                        indices[index].1,
                        *x,
                        *color,
                        ObservationAlpha::resolve(data, &alpha, row)?,
                    ))
                },
            ),
        )?;
    crate::data::source::sort_source_by(&mut rows, |a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.3.source_row.cmp(&b.3.source_row))
    });
    let format = chart.color_label_format_spec().cloned();
    let values = if chart.continuous_guide_suppressed() {
        None
    } else {
        Some(crate::decorations::legend::descriptor::domain_presentation(
            color_range,
            selection.get_center(),
            format.as_deref(),
        )?)
    };
    let scale = crate::theme::ContinuousColors::selected(color_range, &selection)?;
    let samples = crate::data::source::collect_source(rows.iter().map(|row| Ok(row.2)))?;
    let colorbar_stops = scale.guide_stops(&samples)?;
    let colors = crate::data::source::collect_source(rows.iter().map(|row| scale.color(row.2)))?;
    Ok(PreparedBeeswarm {
        levels,
        x_data,
        rows,
        colors,
        values,
        format,
        scale,
        selection,
        colorbar_stops,
        color_label: chart.prepare_color_label(None)?,
    })
}

impl PreparedBeeswarm {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        let point = chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_beeswarm(),
        );
        for &(_, _, _, alpha) in &self.rows {
            guide.point(
                crate::decorations::legend::alpha::AlphaGlyphRole::Primary,
                alpha,
                point.clone(),
                false,
            )?;
        }
        Ok(())
    }

    pub(crate) fn guide_samples(&self) -> Result<Vec<f64>, Error> {
        crate::data::source::collect_source(self.rows.iter().map(|row| Ok(row.2)))
    }

    fn guide_stops(&self) -> Result<Vec<GradientStop>, Error> {
        crate::data::source::collect_source(self.colorbar_stops.iter().copied().map(Ok))
    }

    pub(crate) fn install_shared_domain(
        &mut self,
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
    ) -> Result<(), Error> {
        let values = Some(descriptor.numeric.try_copy()?);
        let scale =
            crate::theme::ContinuousColors::selected(descriptor.domain, &descriptor.selection)?;
        let colorbar_stops =
            crate::data::source::collect_source(descriptor.stops.iter().copied().map(Ok))?;
        self.values = values;
        self.selection = descriptor.selection.clone();
        self.scale = scale;
        self.colorbar_stops = colorbar_stops;
        for (color, &(_, _, value, _)) in self.colors.iter_mut().zip(&self.rows) {
            *color = self.scale.color(value)?;
        }
        Ok(())
    }

    pub(crate) fn continuous_semantics(
        &self,
    ) -> crate::decorations::legend::descriptor::ContinuousSemantics {
        crate::decorations::legend::descriptor::ContinuousSemantics {
            domain: self.scale.domain(),
            selection: self.selection.clone(),
            missing: None,
            format: self.format.clone(),
        }
    }

    pub(crate) fn guide_descriptor(
        &self,
        field: &str,
    ) -> Result<Option<crate::decorations::legend::descriptor::ContinuousDescriptor>, Error> {
        let Some(values) = &self.values else {
            return Ok(None);
        };
        Ok(Some(
            crate::decorations::legend::descriptor::ContinuousDescriptor {
                field: crate::data::source::copy_source_string(field)?,
                title: crate::data::source::source_arc(self.color_label.try_copy()?)?,
                stops: self.guide_stops()?,
                selection: self.selection.clone(),
                center: self.selection.get_center(),
                missing: None,
                domain: self.scale.domain(),
                exact: self.selection.get_domain().is_some(),
                format: self.format.clone(),
                numeric: values.try_copy()?,
            },
        ))
    }
}

pub(crate) fn natural_body_height(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
    width: f64,
) -> Result<f64, Error> {
    let inputs = chart
        .prepared_beeswarm
        .as_ref()
        .ok_or_else(|| Error::Render("natural Beeswarm rows missing".into()))?;
    let marks = crate::layout::interior::preference::panel(
        &inputs.panels,
        chart.axis_scope(),
        "natural Beeswarm",
    )?;
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("natural Beeswarm axes missing".into()))?;
    let range = chart.x_axis_spec().limits.unwrap_or(axes.ranges[0]);
    let projection = crate::layout::Projection::new(range, (0.0, 1.0), (0.0, width, 0.0, 1.0));
    let radius = packing_radius(chart, theme)?;
    let packed = packing::pack(
        marks
            .rows
            .iter()
            .map(|&(category, value, color, alpha)| (category, value, color, alpha.source_row)),
        &projection,
        radius,
    )?;
    let hard = intrinsic_body_minimum(chart, theme)?;
    let height = quanta_pixels(packed.hard_pitch)
        .max(2.0 * f64::from(theme.typography.axes.tick_label.font_size))
        * marks.levels.len() as f64;
    Ok(height.max(hard.1).max(3.0 * width / 4.0))
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let prepared = retained(chart)?;
    let theme = context.theme;

    let minimum = intrinsic_body_minimum(chart, theme)?;
    let (panel, colorbar) = if chart.continuous_guide_suppressed()
        || chart.legend_policy_spec() == crate::LegendPolicy::Hidden
    {
        (context.panel, None)
    } else {
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        let descriptor = chart
            .prepared_discrete_guide()?
            .ok_or_else(|| Error::Render("native Beeswarm guide was not prepared".into()))?;
        let (body, guide) = crate::layout::interior::guide::native_right(
            descriptor,
            context.panel,
            minimum.0,
            theme,
            &guide_scope,
            context.content_height.then_some(chart),
        )?;
        (body, Some(guide))
    };
    let axes = chart
        .prepared_axis_inputs()
        .ok_or_else(|| Error::Render("Beeswarm axes require preparation".into()))?;
    let range = context.axis_range(crate::layout::AxisSide::Bottom, || Ok(prepared.x_data))?;
    let projection = crate::layout::Projection::new(
        range,
        (0.0, prepared.levels.len() as f64),
        panel.as_tuple(),
    );
    let x = axes.specs[0].layout_ticks(
        Vec::new(),
        &projection,
        crate::layout::AxisSide::Bottom,
        theme,
        context.fit_scope(),
    )?;
    let radius = packing_radius(chart, theme)?;
    let packed = packing::pack(
        prepared
            .rows
            .iter()
            .map(|&(category, value, color, alpha)| (category, value, color, alpha.source_row)),
        &projection,
        radius,
    )?;
    let labels = axes.display_categories[1]
        .as_ref()
        .ok_or_else(|| Error::Render("Beeswarm category labels missing".into()))?;
    let mut hard_pitch = packed.hard_pitch;
    let width = context.axis_label_width.max(1.0 / 1024.0);
    if chart.y_tick_labels_visible() {
        for label in labels {
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.axes.tick_label,
                width,
                PhysicalTextAlignment::End,
                "y-axis tick label",
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::AxisLeft, 0)?,
            ))?;
            hard_pitch = hard_pitch.max(crate::layout::kernel::ceil_spatial_quanta(
                plan.height() + 2.0 * crate::layout::LABEL_CLEARANCE,
            )?);
        }
    }
    let preferred = hard_pitch.max(crate::layout::kernel::ceil_spatial_quanta(
        2.0 * f64::from(theme.typography.axes.tick_label.font_size),
    )?);
    let tracks = crate::data::source::collect_source(prepared.levels.iter().map(|_| {
        Ok(crate::layout::kernel::TrackSpec {
            min: hard_pitch,
            preferred,
            max: None,
            flexible: true,
        })
    }))?;
    let bands = crate::layout::interior::tracks::assign(
        &tracks,
        (panel.y0, panel.y1),
        0.0,
        false,
        context.fit_scope(),
    )?;
    let ticks =
        crate::data::source::collect_source(labels.iter().zip(&bands).map(|(label, band)| {
            let middle = f64::midpoint(band.0, band.1);
            Ok(crate::layout::AxisTick::new(
                (middle - panel.y0) / panel.height() * prepared.levels.len() as f64,
                crate::data::source::copy_source_string(label)?,
            ))
        }))?;
    let layout = Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: axes.copy_label(0)?,
            ticks: x.ticks,
            tick_labels_visible: true,
            source: x.source,
            numeric: x.numeric,
            explicit_ticks: axes.specs[0].exact_ticks.is_some(),
            assigned_tick_width: None,
            assigned_cells: None,
        },
        y_axis: crate::layout::Axis {
            label: axes.copy_label(1)?,
            ticks,
            tick_labels_visible: chart.y_tick_labels_visible(),
            source: crate::layout::TickSource::SUPPLIED,
            numeric: None,
            explicit_ticks: false,
            assigned_tick_width: Some(width),
            assigned_cells: Some(crate::data::source::collect_source(
                bands.iter().map(|band| Ok(*band)),
            )?),
        },
    };
    let x_range = layout.projection.x_range();
    let zero = if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
        let (x, y0) = layout.projection.project(0.0, 0.0);
        let (_, y1) = layout.projection.project(0.0, prepared.levels.len() as f64);
        Some((
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            theme
                .marks
                .get_strokes()
                .get_references()
                .get_beeswarm_reference()
                .backend_style(),
        ))
    } else {
        None
    };
    let points = crate::data::source::collect_source(
        packed
            .points
            .iter()
            .zip(&prepared.colors)
            .zip(&prepared.rows)
            .map(|((point, &color), row)| {
                let band = bands[point.category];
                let middle =
                    crate::layout::kernel::nearest_spatial_quanta(f64::midpoint(band.0, band.1))?;
                let y = middle
                    .checked_add(point.offset)
                    .ok_or_else(|| Error::Render("Beeswarm arranged point overflow".into()))?;
                Ok((
                    Point2D::new(quanta_pixels(point.x), quanta_pixels(y)),
                    row.3.point(chart.point_style(
                        theme,
                        color.as_rgb(),
                        theme.marks.get_points().get_radii().get_beeswarm(),
                    )),
                ))
            }),
    )?;
    let mut payload = BeeswarmPayload {
        clip: layout.projection.panel_box(),
        zero,
        points,
        source_rows: crate::data::source::collect_source(
            packed
                .points
                .iter()
                .map(|point| Ok((point.category, point.source_row))),
        )?,
        domain_expansions: [None; 2],
        colorbar: colorbar
            .as_ref()
            .map(|guide| select_colorbar(guide, panel, theme))
            .transpose()?,
    };
    let geometry = payload.geometry()?;
    let mut full = crate::render::geometry::GeometrySnapshot::new();

    for (point, style) in &payload.points {
        full.scatter_with_clip(std::slice::from_ref(point), style, None)?;
    }
    let panel = layout.projection.panel_box();
    if let Some(bounds) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        if bounds.bottom < panel.y0 || bounds.top > panel.y1 {
            return Err(context
                .fit_scope()
                .failure(
                    crate::layout::fit::FitKind::SpecializedGeometry,
                    "Beeswarm painted ink exceeds its assigned category panel",
                )
                .into());
        }
    }
    // A structural zero reference intentionally terminates at the frame;
    // its caps do not impose category-track overflow on data points.
    if let Some((line, style)) = &payload.zero {
        full.line(line, style, None)?;
    }
    if let Some(bounds) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        if chart.user_x_limits().is_none() {
            let levels = u32::try_from(prepared.levels.len()).map_err(|_| {
                Error::Render("Beeswarm category count exceeds spatial range".into())
            })?;
            payload.domain_expansions = crate::layout::interior::automatic_absolute_ink_expansions(
                [prepared.x_data, (0.0, f64::from(levels))],
                &layout.projection,
                bounds,
                [true, false],
                context.fit_scope(),
            )?;
        }
    }
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?);
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.y_tick_keys = Some(crate::data::source::collect_source(
        prepared
            .levels
            .iter()
            .map(|level| crate::data::source::copy_source_string(level)),
    )?);
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([result.x_domain, result.y_domain]),
        || crate::layout::StandardPanelInputs::new(theme),
    )?);
    if let Some(bar) = &payload.colorbar {
        for call in &bar.calls {
            if let crate::decorations::legend::LegendCall::Text(text) = call {
                crate::data::source::reserve_source(&mut result.producer_text, 1)?;
                result.producer_text.push(text.clone());
            }
        }
    }
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Beeswarm(payload),
            geometry,
        },
    )?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct BeeswarmPayload {
    clip: PanelBox,
    zero: Option<([Point2D; 2], LineStyle)>,
    points: Vec<(Point2D, crate::render::backend::PointStyle)>,
    source_rows: Vec<(usize, usize)>,
    colorbar: Option<crate::decorations::legend::LegendDrawPlan>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl BeeswarmPayload {
    pub(crate) fn observe_sources(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.observe_count(self.source_rows.len())?;
        for &(category, row) in &self.source_rows {
            round.observe_count(category)?;
            round.observe_count(row)?;
        }
        Ok(())
    }
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            if let Some((line, style)) = &self.zero {
                backend.draw_line(line, style)?;
            }
            for (point, style) in &self.points {
                backend.draw_scatter(&[*point], style)?;
            }
            Ok(())
        })?;
        if let Some(bar) = &self.colorbar {
            bar.draw(backend)?;
        }
        Ok(())
    }

    fn geometry(&self) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        if let Some((line, style)) = &self.zero {
            geometry.line(line, style, Some(clip))?;
        }
        for (point, style) in &self.points {
            geometry.scatter(&[*point], style, clip)?;
        }
        if let Some(bar) = &self.colorbar {
            bar.geometry(&mut geometry)?;
        }
        Ok(geometry)
    }
}

fn select_colorbar(
    guide: &crate::decorations::legend::Legend,
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
) -> Result<crate::decorations::legend::LegendDrawPlan, Error> {
    let bounds = guide.box_at(
        panel.x1 + theme.legend_outside_gap()?,
        panel.y0 + (panel.height() - guide.compute_box_height()) / 2.0,
    );
    let (x, y) = bounds.anchor();
    guide.select(x, y)
}

mod packing;

#[cfg(test)]
mod tests;
