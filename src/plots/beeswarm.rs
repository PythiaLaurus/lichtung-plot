//! Categorical beeswarm plot with a continuous color scale.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]

use crate::layout::{Layout, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{with_clip, Backend, GradientStop, LineStyle, PlacedText, Point2D};
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
    rows: Vec<(usize, f64, f64, usize)>,
    colors: Vec<(u8, u8, u8)>,
    scale: crate::theme::ContinuousColors,
    colorbar_stops: Vec<GradientStop>,
    values: [String; 2],
    pub(crate) color_label: String,
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_beeswarm
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| {
            Error::Render("Beeswarm intrinsic requires root-prepared source rows".into())
        })?;
    if prepared.levels.is_empty()
        || prepared.rows.is_empty()
        || prepared.rows.len() != prepared.colors.len()
    {
        return Err(Error::Render(
            "Beeswarm source rows and prepared points differ".into(),
        ));
    }
    let point = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.beeswarm);
    let ink = crate::render::geometry::point_ink_bounds(theme, &point)?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_visits(prepared.rows.len() as u128)
        .map_err(|error| {
            Error::Render(format!("Beeswarm occupied category admission: {error:?}"))
        })?;
    let pitch = 2.0 * quanta_pixels(packing::radius(theme, &point)?);
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
    let mut numeric = ink.map_or(0.0, |ink| ink.right - ink.left);
    if chart.zero_line_visible() && prepared.x_data.0 <= 0.0 && prepared.x_data.1 >= 0.0 {
        numeric = numeric.max(crate::render::geometry::line_ink_width(
            theme,
            &theme.marks.beeswarm_reference.backend_style(),
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

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 3]> = const { std::cell::Cell::new([0; 3]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn prepare(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<PreparedBeeswarm, Error> {
    #[cfg(test)]
    count(0);
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
    let levels = data.source_levels(&categories)?;
    data.claim_source_visits(xs.len())?;
    let x_data = crate::data::min_max(xs.iter().copied(), "beeswarm")?;
    let color_range = if let Some((minimum, maximum)) = chart.beeswarm_color_limits_spec() {
        if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
            return Err(Error::Data(format!(
                    "beeswarm: color limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )));
        }
        (minimum, maximum)
    } else {
        data.claim_source_visits(colors.len())?;
        crate::data::min_max(colors.iter().copied(), "beeswarm")?
    };
    let mut indices =
        data.collect_source(levels.iter().enumerate().map(|(i, s)| Ok((s.as_str(), i))))?;
    crate::data::source::sort_source_by(&mut indices, Some(data), |a, b| a.0.cmp(b.0))?;
    let mut rows = data.collect_source(categories.iter().zip(&xs).zip(&colors).enumerate().map(
        |(row, ((category, x), color))| {
            data.claim_source_visits(indices.len().ilog2() as usize + 2)?;
            let index = indices
                .binary_search_by_key(&category.as_str(), |entry| entry.0)
                .map_err(|_| Error::Data("beeswarm: category identity missing".into()))?;
            Ok((indices[index].1, *x, *color, data.source_row(row)))
        },
    ))?;
    crate::data::source::sort_source_by(&mut rows, Some(data), |a, b| {
        a.0.cmp(&b.0).then_with(|| a.3.cmp(&b.3))
    })?;
    let values = colorbar_values(color_range, data)?;
    let scale = crate::theme::ContinuousColors::resolve(
        color_range,
        chart
            .beeswarm_gradient_spec()
            .unwrap_or(&crate::theme::BEESWARM_SEQUENTIAL_STOPS),
        data.source_resources()?,
    )?;
    let samples = data.collect_source(rows.iter().map(|row| Ok(row.2)))?;
    let colorbar_stops = scale.guide_stops(&samples, data)?;
    let colors = data.collect_source(
        rows.iter()
            .map(|row| scale.color(row.2, data.source_resources()?)),
    )?;
    Ok(PreparedBeeswarm {
        levels,
        x_data,
        rows,
        colors,
        values,
        scale,
        colorbar_stops,
        color_label: chart.prepare_color_label(data, None)?,
    })
}

impl PreparedBeeswarm {
    pub(crate) fn guide_samples(&self, data: crate::data::PlotData<'_>) -> Result<Vec<f64>, Error> {
        data.collect_source(self.rows.iter().map(|row| Ok(row.2)))
    }

    fn guide_stops(&self, data: crate::data::PlotData<'_>) -> Result<Vec<GradientStop>, Error> {
        data.collect_source(self.colorbar_stops.iter().copied().map(Ok))
    }

    #[cfg(test)]
    pub(crate) fn color_for_source_value(&self, value: f64) -> Option<(u8, u8, u8)> {
        self.rows
            .iter()
            .zip(&self.colors)
            .find(|(row, _)| row.2.to_bits() == value.to_bits())
            .map(|(_, color)| *color)
    }

    pub(crate) fn install_shared_domain(
        &mut self,
        descriptor: &crate::decorations::legend::descriptor::ContinuousDescriptor,
        data: crate::data::PlotData<'_>,
    ) -> Result<(), Error> {
        let values = colorbar_values(descriptor.domain, data)?;
        let scale = self
            .scale
            .with_domain(descriptor.domain, data.source_resources()?)?;
        let colorbar_stops = scale.guide_stops(&self.guide_samples(data)?, data)?;
        self.values = values;
        self.scale = scale;
        self.colorbar_stops = colorbar_stops;
        for (color, &(_, _, value, _)) in self.colors.iter_mut().zip(&self.rows) {
            *color = self.scale.color(value, data.source_resources()?)?;
        }
        Ok(())
    }

    pub(crate) fn guide_semantics(
        &self,
        data: crate::data::PlotData<'_>,
    ) -> Result<crate::decorations::legend::descriptor::ContinuousSemantics, Error> {
        let stops = self.scale.stops();
        Ok(
            crate::decorations::legend::descriptor::ContinuousSemantics {
                domain: self.scale.domain(),
                center: None,
                stops: data.collect_source(stops.iter().copied().map(Ok))?,
                missing: None,
            },
        )
    }

    pub(crate) fn guide_labels(&self) -> impl Iterator<Item = &str> {
        self.values.iter().map(String::as_str)
    }
}

fn colorbar_values(
    range: (f64, f64),
    data: crate::data::PlotData<'_>,
) -> Result<[String; 2], Error> {
    let mut values = [String::new(), String::new()];
    for (label, value) in values.iter_mut().zip([range.1, range.0]) {
        use std::fmt::Write;
        data.reserve_source_string(label, 32)?;
        if value.abs() >= 1_000_000.0 || (value.abs().to_bits() != 0 && value.abs() < 0.0001) {
            write!(label, "{value:.3e}").expect("reserved colorbar label");
        } else {
            write!(label, "{value:.3}").expect("reserved colorbar label");
        }
    }
    Ok(values)
}

pub(crate) fn measure(
    chart: &Plot,
    context: PlotRenderContext<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = chart
        .prepared_beeswarm
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("beeswarm requires root-prepared semantics".into()))?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let minimum = intrinsic_body_minimum(chart, theme)?;
    let (panel, colorbar) = if chart.continuous_guide_suppressed() {
        (context.panel, None)
    } else {
        let [high_label, low_label] = &prepared.values;
        let guide_scope = context
            .fit_scope()
            .child(crate::layout::AddressRole::OutsideGuideRight, 0)?;
        let labels: &[&str] =
            if prepared.scale.domain().0.to_bits() == prepared.scale.domain().1.to_bits() {
                &[low_label]
            } else {
                &[low_label, high_label]
            };
        let descriptor = crate::decorations::legend::descriptor::ContinuousDescriptor {
            field: data.copy_source_string(chart.color_dimension_key().unwrap_or(""))?,
            title: data.copy_source_string(&prepared.color_label)?,
            stops: prepared.guide_stops(data)?,
            center: None,
            missing: None,
            domain: prepared.scale.domain(),
            exact: chart.beeswarm_color_limits_spec().is_some(),
            labels: data
                .collect_source(labels.iter().map(|label| data.copy_source_string(label)))?,
        };
        let (body, guide) = crate::layout::interior::guide::continuous_right(
            &descriptor,
            context.panel,
            minimum.0,
            theme,
            &guide_scope,
            data,
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
    let radius = packing::radius(
        theme,
        &chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.beeswarm),
    )?;
    #[cfg(test)]
    count(2);
    let packed = packing::pack(&prepared.rows, &projection, radius, data)?;
    let labels = axes.display_categories[1]
        .as_ref()
        .ok_or_else(|| Error::Render("Beeswarm category labels missing".into()))?;
    let mut hard_pitch = packed.hard_pitch;
    let width = context.axis_label_width.max(1.0 / 1024.0);
    if chart.y_tick_labels_visible() {
        for label in labels {
            data.claim_source_visits(1)?;
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                label,
                &theme.typography.tick_label,
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
        2.0 * f64::from(theme.typography.tick_label.font_size),
    )?);
    let tracks = data.collect_source(prepared.levels.iter().map(|_| {
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
        data,
        context.fit_scope(),
    )?;
    let ticks = data.collect_source(labels.iter().zip(&bands).map(|(label, band)| {
        data.claim_source_visits(1)?;
        let middle = f64::midpoint(band.0, band.1);
        Ok(crate::layout::AxisTick::new(
            (middle - panel.y0) / panel.height() * prepared.levels.len() as f64,
            if chart.y_tick_labels_visible() {
                data.copy_source_string(label)?
            } else {
                String::new()
            },
        ))
    }))?;
    let layout = Layout {
        projection,
        x_axis: crate::layout::Axis {
            label: data.copy_source_string(&axes.labels[0])?,
            ticks: x.ticks,
            source: x.source,
            explicit_ticks: axes.specs[0].exact_ticks.is_some(),
            assigned_tick_width: None,
            assigned_cells: None,
        },
        y_axis: crate::layout::Axis {
            label: data.copy_source_string(&axes.labels[1])?,
            ticks,
            source: crate::layout::TickSource::SUPPLIED,
            explicit_ticks: false,
            assigned_tick_width: Some(width),
            assigned_cells: Some(data.collect_source(bands.iter().map(|band| {
                data.claim_source_visits(1)?;
                Ok(*band)
            }))?),
        },
    };
    let x_range = layout.projection.x_range();
    let zero = if chart.zero_line_visible() && x_range.0 <= 0.0 && x_range.1 >= 0.0 {
        let (x, y0) = layout.projection.project(0.0, 0.0);
        let (_, y1) = layout.projection.project(0.0, prepared.levels.len() as f64);
        Some((
            [Point2D::new(x, y0), Point2D::new(x, y1)],
            theme.marks.beeswarm_reference.backend_style(),
        ))
    } else {
        None
    };
    let points = data.collect_source(packed.points.iter().zip(&prepared.colors).map(
        |(point, &color)| {
            data.claim_source_visits(1)?;
            let band = bands[point.category];
            let middle =
                crate::layout::kernel::nearest_spatial_quanta(f64::midpoint(band.0, band.1))?;
            let y = middle
                .checked_add(point.offset)
                .ok_or_else(|| Error::Render("Beeswarm arranged point overflow".into()))?;
            Ok((
                Point2D::new(quanta_pixels(point.x), quanta_pixels(y)),
                chart.point_style(theme, color, theme.marks.point_sizes.beeswarm),
            ))
        },
    ))?;
    let mut payload = BeeswarmPayload {
        clip: layout.projection.panel_box(),
        zero,
        points,
        source_rows: data.collect_source(packed.points.iter().map(|point| {
            data.claim_source_visits(1)?;
            Ok((point.category, point.source_row))
        }))?,
        domain_expansions: [None; 2],
        colorbar: colorbar
            .as_ref()
            .map(|guide| select_colorbar(guide, panel, theme, data))
            .transpose()?,
    };
    let geometry = payload.geometry(theme)?;
    let mut full = crate::render::geometry::GeometrySnapshot::new(theme)?;
    data.claim_source_visits(payload.points.len())?;
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
                data,
                context.fit_scope(),
            )?;
        }
    }
    let mut result = PlotRenderResult::new(layout, Vec::new());
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.y_tick_keys = Some(
        data.collect_source(
            prepared
                .levels
                .iter()
                .map(|level| data.copy_source_string(level)),
        )?,
    );
    result.axes = Some(result.layout.measure_axes_with_inputs(
        theme,
        context.fit_scope(),
        chart.axis_semantic_inputs([result.x_domain, result.y_domain]),
        || crate::layout::StandardPanelInputs::new(theme),
    )?);
    if let Some(bar) = &payload.colorbar {
        result.producer_text = data.collect_source(bar.text.iter().map(|text| Ok(text.clone())))?;
    }
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Beeswarm(payload),
        geometry,
    })?);
    Ok(result)
}

#[derive(Debug)]
pub(crate) struct BeeswarmPayload {
    clip: PanelBox,
    zero: Option<([Point2D; 2], LineStyle)>,
    points: Vec<(Point2D, crate::render::backend::PointStyle)>,
    source_rows: Vec<(usize, usize)>,
    colorbar: Option<ColorbarPayload>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

#[derive(Debug)]
struct ColorbarPayload {
    bounds: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: Vec<GradientStop>,
    text: Vec<PlacedText>,
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
            backend.draw_linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops)?;
            for text in &bar.text {
                text.draw(backend)?;
            }
        }
        Ok(())
    }

    fn geometry(
        &self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        if let Some((line, style)) = &self.zero {
            geometry.line(line, style, Some(clip))?;
        }
        for (point, style) in &self.points {
            geometry.scatter(&[*point], style, clip)?;
        }
        if let Some(bar) = &self.colorbar {
            geometry.linear_gradient(bar.bounds, bar.start, bar.end, &bar.stops, None)?;
            for text in &bar.text {
                geometry.text(text.placement, &text.plan, None)?;
            }
        }
        Ok(geometry)
    }
}

fn select_colorbar(
    guide: &crate::decorations::legend::MeasuredContinuousGuide,
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    data: crate::data::PlotData<'_>,
) -> Result<ColorbarPayload, Error> {
    let selected = guide.select(
        panel.x1 + theme.legend.outside_gap,
        panel.y0 + (panel.height() - guide.height) / 2.0,
        data,
    )?;
    let mut gradient = None;
    let mut text = Vec::new();
    data.reserve_source(&mut text, selected.calls.len())?;
    for call in selected.calls {
        data.claim_source_visits(1)?;
        match call {
            crate::decorations::legend::LegendCall::Gradient(bounds, start, end, stops) => {
                gradient = Some((bounds, start, end, stops));
            }
            crate::decorations::legend::LegendCall::Text(placed) => text.push(placed),
            crate::decorations::legend::LegendCall::Background(_, _) => {}
            _ => {
                return Err(Error::Render(
                    "unexpected Beeswarm continuous guide operand".into(),
                ))
            }
        }
    }
    if !text.is_empty() {
        text.rotate_left(1);
    }
    let (bounds, start, end, stops) =
        gradient.ok_or_else(|| Error::Render("Beeswarm guide has no gradient".into()))?;
    Ok(ColorbarPayload {
        bounds,
        start,
        end,
        stops,
        text,
    })
}

#[cfg(test)]
mod extraction;

mod packing;

#[cfg(test)]
mod fit_tests {
    use super::*;
    use crate::layout::fit::{FitKind, FitOutcome, FitScope};
    use crate::layout::{AddressRole, ComponentAddress};
    use crate::theme::ResolvedTheme;

    #[test]
    fn complete_marker_ink_raises_native_category_hard_pitch() {
        let source = |exact: bool| {
            let plot = Plot::beeswarm(
                crate::CategoryValues::new(
                    "category",
                    ["a", "a", "a", "a", "a", "a", "a", "a", "b"],
                )
                .unwrap(),
                crate::NumericValues::new("x", [0.0; 9]),
                crate::NumericValues::new("color", [0.5; 9]),
            );
            if exact {
                plot.x_limits(-1.0, 1.0).erase()
            } else {
                plot.erase()
            }
        };
        let intrinsic = |source: Plot, theme: &ResolvedTheme| {
            let resources = crate::font::canonical::OperationGenerationResources::new();
            let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
            inputs
                .attach(source)
                .unwrap()
                .intrinsic_body_minimum(theme)
                .unwrap()
        };
        let theme = crate::Theme::Publication.resolve();
        let baseline = intrinsic(source(false), &theme);
        let mut styled = theme.clone();
        styled.marks.point_sizes.beeswarm = 54;
        let raised = intrinsic(source(false), &styled);
        assert!(raised.0 > baseline.0);
        assert!(raised.1 > baseline.1);
        let exact = intrinsic(source(true), &styled);
        assert!(exact.0 >= 1.0 / 1024.0);
        assert!(exact.0 < raised.0);
        assert_eq!(exact.1.to_bits(), raised.1.to_bits());
    }

    #[test]
    fn tight_native_height_refuses_without_hiding_coincident_source_points() {
        let source = Plot::beeswarm(
            crate::CategoryValues::new("category", ["a"; 12]).unwrap(),
            crate::NumericValues::new("x", [0.0; 12]),
            crate::NumericValues::new("color", [0.5; 12]),
        )
        .x_label("")
        .y_label("")
        .erase()
        .with_suppressed_continuous_guide();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let chart = crate::plots::shared_x::prepare_plot(&source, &resources)
            .unwrap()
            .attach(source)
            .unwrap();
        let theme = crate::Theme::Publication.resolve();
        let result = chart.measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(50.0, 850.0, 50.0, 80.0),
            &theme,
        ));
        assert!(
            matches!(result, Err(crate::layout::fit::PlanError::Fit(failure)) if failure.reason.kind == FitKind::TrackOrResidual)
        );
        assert_eq!(
            chart.prepared_beeswarm.as_ref().unwrap().panels[0]
                .1
                .rows
                .len(),
            12
        );
    }

    #[test]
    fn selected_swarm_ink_clears_numeric_edges_without_repacking_points() {
        let source = Plot::beeswarm(
            crate::CategoryValues::new("category", ["a", "a", "a", "a", "a", "a", "a", "a", "b"])
                .unwrap(),
            crate::NumericValues::new("x", [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 100.0]),
            crate::NumericValues::new("color", [0.5; 9]),
        )
        .color_limits(0.0, 1.0)
        .size(1800, 30000)
        .erase();
        let mut theme = crate::Theme::Publication.resolve();
        theme.marks.point_sizes.beeswarm = 300;
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let cell = PanelBox::new(80.0, 1750.0, 80.0, 29900.0);
        let selected = crate::decorations::plan::plan_panel_for_test(
            &chart,
            crate::PlotKind::Beeswarm,
            cell,
            crate::decorations::plan::LegendRole::Suppressed,
            &theme,
        )
        .unwrap();
        assert!(selected.planned_x_limits().is_some());
        let crate::layout::interior::MarkPayload::Beeswarm(payload) =
            &selected.result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Beeswarm")
        };
        assert!(payload.domain_expansions.iter().all(Option::is_none));
        assert_eq!(payload.points.len(), 9);
        let panel = selected.result.layout.projection.panel_box();
        for (point, style) in &payload.points {
            let ink = crate::render::geometry::point_ink_bounds(&theme, style)
                .unwrap()
                .unwrap();
            assert!(point.x + ink.left >= panel.x0 && point.x + ink.right <= panel.x1);
            assert!(point.y + ink.bottom >= panel.y0 && point.y + ink.top <= panel.y1);
        }
    }

    #[test]
    fn colorbar_title_fit_preserves_its_child_address_and_zero_local_attempt() {
        let plot = Plot::beeswarm(
            crate::CategoryValues::new("category", ["one", "two"]).unwrap(),
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("color", [0.0, 1.0]),
        )
        .color_label("W".repeat(200))
        .erase();
        let inputs = crate::plots::shared_x::prepare_plot(
            &plot,
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let plot = inputs.attach(plot).unwrap();
        let owner = ComponentAddress::root()
            .child(AddressRole::CanvasCell, 2)
            .unwrap();
        let scope = FitScope::new(&owner, 7);
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let result = plot
            .preflight_measurement(
                crate::spec::PlotKind::Beeswarm,
                PanelBox::new(80.0, 500.0, 80.0, 350.0),
                &theme,
                &scope,
            )
            .unwrap();
        let FitOutcome::Miss(failure) = result else {
            panic!("unbreakable colorbar title unexpectedly fitted")
        };
        assert_eq!(failure.reason.kind, FitKind::Text);
        assert_eq!(failure.reason.ordinal, 0);
        assert_eq!(
            failure.reason.address,
            owner
                .child(AddressRole::OutsideGuideRight, 0)
                .unwrap()
                .child(AddressRole::GuideTitle, 0)
                .unwrap()
        );

        let mut invalid = crate::Theme::Publication.resolve();
        invalid.typography.legend_title.font_size = 0;
        invalid.prepare_font_environment().unwrap();
        assert!(plot
            .preflight_measurement(
                crate::spec::PlotKind::Beeswarm,
                PanelBox::new(80.0, 500.0, 80.0, 350.0),
                &invalid,
                &scope
            )
            .is_err());
    }
}
