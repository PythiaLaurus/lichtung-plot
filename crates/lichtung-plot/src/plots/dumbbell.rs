//! Horizontal categorical dumbbells with explicit paired endpoints.

#![expect(
    clippy::cast_precision_loss,
    reason = "validated row counts are converted to bounded plotting coordinates"
)]

use crate::alpha::{
    marks::{line_style, ObservationAlpha},
    AlphaTarget, AlphaUnitKind,
};
use crate::Error;

use crate::builder::PlotDescription;
use crate::decorations::legend::source::{LiteralDomain, PointRole};
use crate::decorations::legend::GuideSourceKey;
use crate::layout::interior::MeasuredCoordinateMarks;
use crate::layout::AuthoredAxisSpec;
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{CategoryOrder, PlotMeasureContext};
#[cfg(test)]
use crate::Plot;

#[derive(Debug)]
struct DumbbellRow {
    category_index: usize,
    group_index: usize,
    left: f64,
    right: f64,
    alpha: ObservationAlpha,
    line_alpha: ObservationAlpha,
}

#[derive(Debug)]
pub(crate) struct DumbbellData {
    group_count: usize,
    rows: Vec<DumbbellRow>,
    pub(crate) range: (f64, f64),
}

pub(crate) fn intrinsic_body_minimum(
    chart: &PlotDescription,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Dumbbell(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Dumbbell intrinsic requires root-prepared pairs".into()))?;
    let categories = &registered
        .expect("prepared Dumbbell occurrence")
        .axes
        .categories(1)
        .expect("prepared Dumbbell categories")
        .raw;
    if categories.is_empty() || prepared.group_count == 0 || prepared.rows.is_empty() {
        return Err(Error::Render(
            "Dumbbell has no occupied category/pair slots".into(),
        ));
    }

    let slot = 0.72
        / f64::from(
            u32::try_from(prepared.group_count).map_err(|_| {
                Error::Render("Dumbbell series count exceeds coordinate range".into())
            })?,
        );
    let mut offset = 0.0_f64;
    for row in &prepared.rows {
        if row.category_index >= categories.len() || row.group_index >= prepared.group_count {
            return Err(Error::Render(
                "Dumbbell occupied slot exceeds prepared categories".into(),
            ));
        }
        offset = offset.max((0.36 - slot * (row.group_index as f64 + 0.5)).abs());
    }
    let endpoints = chart
        .dumbbell_endpoints_spec()
        .ok_or_else(|| Error::Render("Dumbbell endpoint paints are missing".into()))?;
    let mut outward = 0.0_f64;
    let mut numeric = 0.0_f64;
    for row in &prepared.rows {
        for color in [endpoints.left_color(), endpoints.right_color()] {
            if let Some(ink) = crate::render::geometry::point_ink_bounds(
                &row.alpha.point(endpoint_style(chart, theme, color)),
            )? {
                outward = outward.max(ink.top.max(-ink.bottom));
                numeric = numeric.max(ink.right - ink.left);
            }
        }
    }
    for row in &prepared.rows {
        let connector = crate::render::geometry::line_ink_width(&line_style(
            row.line_alpha.value,
            theme
                .marks
                .get_strokes()
                .get_connectors()
                .get_dumbbell_connector()
                .backend_style(),
        ))?;
        outward = outward.max(connector / 2.0);
        numeric = numeric.max(connector);
    }
    let pitch = (outward / (0.5 - offset)).max(1.0 / 1024.0);
    let rows =
        f64::from(u32::try_from(categories.len()).map_err(|_| {
            Error::Render("Dumbbell category count exceeds coordinate range".into())
        })?);
    let height = pitch * rows;
    if !height.is_finite() {
        return Err(Error::Render(
            "Dumbbell category hard span overflowed".into(),
        ));
    }
    Ok((
        if registered.expect("prepared occurrence").raw_windows()[0].is_none() {
            numeric
        } else {
            0.0
        },
        height,
    ))
}

impl DumbbellData {
    pub(crate) fn alpha_guide(
        &self,
        chart: &crate::builder::PlotDescription,
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let endpoints = chart
            .dumbbell_endpoints_spec()
            .ok_or_else(|| Error::Render("Dumbbell guide endpoint paints are missing".into()))?;
        for row in &self.rows {
            guide.point(
                AlphaGlyphRole::LeftEndpoint,
                row.alpha,
                endpoint_style(chart, theme, endpoints.left_color()),
                false,
            )?;
            guide.point(
                AlphaGlyphRole::RightEndpoint,
                row.alpha,
                endpoint_style(chart, theme, endpoints.right_color()),
                false,
            )?;
            guide.line(
                AlphaGlyphRole::Line,
                AlphaUnitKind::Observation,
                AlphaUnit::observation(row.line_alpha),
                row.line_alpha.value,
                theme
                    .marks
                    .get_strokes()
                    .get_connectors()
                    .get_dumbbell_connector()
                    .backend_style(),
                false,
            )?;
        }
        Ok(())
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.rows.len() as u128
    }

    pub(crate) fn guide_entries(
        chart: &crate::builder::PlotDescription,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let endpoints = chart
            .dumbbell_endpoints_spec()
            .ok_or_else(|| Error::Render("Dumbbell guide requires endpoint metadata".into()))?;
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, 2)?;
        for (label, color) in [
            (endpoints.left_label(), endpoints.left_color()),
            (endpoints.right_label(), endpoints.right_color()),
        ] {
            entries.push(
                GuideSourceKey::literal(
                    crate::decorations::legend::GuideEntryIdentity::endpoint(label)?,
                    LiteralDomain::DumbbellEndpoints,
                )
                .with_exact_point(endpoint_style(chart, theme, color), PointRole::Primary),
            );
        }
        Ok(entries)
    }
}

#[derive(Debug)]
pub(crate) struct DumbbellPayload {
    clip: crate::layout::PixelRect,
    pub(crate) pairs: Vec<[Point2D; 2]>,
    connectors: Vec<LineStyle>,
    points: Vec<[crate::render::backend::PointStyle; 2]>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl DumbbellPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        with_clip(backend, self.clip, |backend| {
            for ((pair, points), connector) in
                self.pairs.iter().zip(&self.points).zip(&self.connectors)
            {
                backend.draw_line(pair, connector)?;
                backend.draw_scatter(&pair[..1], &points[0])?;
                backend.draw_scatter(&pair[1..], &points[1])?;
            }
            Ok(())
        })
    }
}

pub(crate) fn measure_marks(
    chart: &PlotDescription,
    context: &PlotMeasureContext<'_, '_>,
    request: &crate::text::TextRequestContext<'_, '_>,
    layout: &crate::layout::Layout,
) -> Result<MeasuredCoordinateMarks, crate::layout::fit::PlanError> {
    let prepared = (match context.family()? {
        crate::orchestrator::registration::FamilyRef::Dumbbell(marks) => Some(marks),
        _ => None,
    })
    .ok_or_else(|| Error::Render("Dumbbell requires root-prepared semantics".into()))?;

    let projection = layout.projection.clone();
    let categories = &context
        .registered
        .axes
        .categories(1)
        .expect("prepared Dumbbell categories")
        .raw;
    let mut result = project(chart, prepared, categories, layout, request)?;
    let crate::layout::interior::MarkPayload::Dumbbell(payload) = &result.marks.payload else {
        unreachable!("Dumbbell selected its own mark payload")
    };
    let mut full = crate::render::geometry::GeometrySnapshot::new();
    for ((pair, points), connector) in payload
        .pairs
        .iter()
        .zip(&payload.points)
        .zip(&payload.connectors)
    {
        full.line(pair, connector, None)?;
        full.scatter_with_clip(&pair[..1], &points[0], None)?;
        full.scatter_with_clip(&pair[1..], &points[1], None)?;
    }
    if let Some(ink) = full
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    {
        let count = u32::try_from(categories.len()).map_err(|_| {
            Error::Render("Dumbbell category count exceeds coordinate range".into())
        })?;
        let requests = crate::layout::interior::automatic_absolute_ink_expansions(
            [prepared.range, (0.0, f64::from(count))],
            &projection,
            ink,
            [context.registered.raw_windows()[0].is_none(), false],
            context.fit_scope(),
        )?;
        let marks = std::sync::Arc::get_mut(&mut result.marks)
            .expect("newly selected Dumbbell marks have one owner");
        let crate::layout::interior::MarkPayload::Dumbbell(payload) = &mut marks.payload else {
            unreachable!("Dumbbell selected its own mark payload")
        };
        payload.domain_expansions = requests;
    }
    Ok(result)
}

fn project(
    chart: &PlotDescription,
    prepared: &DumbbellData,
    categories: &[String],
    layout: &crate::layout::Layout,
    request: &crate::text::TextRequestContext<'_, '_>,
) -> Result<MeasuredCoordinateMarks, Error> {
    let theme = request;
    let endpoints = chart
        .dumbbell_endpoints_spec()
        .expect("dumbbell input owns endpoint metadata");
    let category_count = categories.len();
    let connector = theme
        .marks
        .get_strokes()
        .get_connectors()
        .get_dumbbell_connector()
        .backend_style();
    let mut payload = DumbbellPayload {
        clip: layout.projection.panel_box(),
        pairs: Vec::new(),
        connectors: Vec::new(),
        points: Vec::new(),
        domain_expansions: [None; 2],
    };
    crate::data::source::reserve_source(&mut payload.pairs, prepared.rows.len())?;
    crate::data::source::reserve_source(&mut payload.points, prepared.rows.len())?;
    crate::data::source::reserve_source(&mut payload.connectors, prepared.rows.len())?;

    for row in &prepared.rows {
        let y = replicate_y(
            row.category_index,
            category_count,
            row.group_index,
            prepared.group_count,
        );
        let (left_x, left_y) = layout.projection.project(row.left, y);
        let (right_x, right_y) = layout.projection.project(row.right, y);
        layout.projection.validate_span(
            crate::layout::AxisSide::Bottom,
            (row.left, row.right),
            (left_x, right_x),
            row.left.min(row.right) < row.left.max(row.right),
            "dumbbell interval",
        )?;
        payload
            .pairs
            .push([Point2D::new(left_x, left_y), Point2D::new(right_x, right_y)]);
        payload.points.push([
            row.alpha
                .point(endpoint_style(chart, theme, endpoints.left_color())),
            row.alpha
                .point(endpoint_style(chart, theme, endpoints.right_color())),
        ]);
        payload
            .connectors
            .push(line_style(row.line_alpha.value, connector.clone()));
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for ((pair, points), connector) in payload
        .pairs
        .iter()
        .zip(&payload.points)
        .zip(&payload.connectors)
    {
        geometry.line(pair, connector, Some(clip))?;
        geometry.scatter(&pair[..1], &points[0], clip)?;
        geometry.scatter(&pair[1..], &points[1], clip)?;
    }
    Ok(MeasuredCoordinateMarks::new(
        crate::data::source::source_arc(crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Dumbbell(payload),
            geometry,
        })?,
    ))
}

pub(crate) fn prepare(
    chart: &crate::builder::PlotDescription,
    data: crate::data::PlotData<'_, '_>,
) -> Result<(DumbbellData, Vec<String>), Error> {
    data.validate_row_source()?;
    let source = chart;
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
    let line_alpha = chart.prepare_alpha(AlphaTarget::Line, AlphaUnitKind::Observation, &[])?;
    if source.y_axis_spec() != &AuthoredAxisSpec::default() {
        return Err(Error::Data(
            "dumbbell: categorical y-axis limits are not supported".into(),
        ));
    }
    if source.hue_col().is_some() || source.color_col().is_some() {
        return Err(Error::Data(
            "dumbbell: hue and continuous color channels are not supported; endpoint colors are explicit"
                .into(),
        ));
    }
    if !matches!(source.category_order_spec(), CategoryOrder::Observed) {
        return Err(Error::Data(
            "dumbbell: category ordering is not supported; category order is first-observed".into(),
        ));
    }
    let category_col = source
        .x_col()
        .ok_or_else(|| Error::Data("dumbbell: category column not set".into()))?;
    let left_col = source
        .y_col()
        .ok_or_else(|| Error::Data("dumbbell: left value column not set".into()))?;
    let right_col = source
        .dumbbell_right_col()
        .ok_or_else(|| Error::Data("dumbbell: right value column not set".into()))?;
    let categories =
        crate::data::strict_normalized_strings(data, category_col, "dumbbell", "category")?;
    let left = crate::data::strict_finite_f64(data, left_col, "dumbbell")?;
    let right = crate::data::strict_finite_f64(data, right_col, "dumbbell")?;
    let groups = source
        .group_col()
        .map(|column| {
            crate::data::strict_normalized_strings(data, column, "dumbbell", "replicate/group")
        })
        .transpose()?;
    if categories.is_empty()
        || categories.len() != left.len()
        || categories.len() != right.len()
        || groups
            .as_ref()
            .is_some_and(|values| values.len() != categories.len())
    {
        return Err(Error::Data(
            "dumbbell: category, left, right, and optional group columns must have equal non-zero lengths"
                .into(),
        ));
    }

    let category_levels = crate::data::source::source_levels(&categories)?;
    let group_levels = groups
        .as_ref()
        .map(|values| crate::data::source::source_levels(values))
        .transpose()?
        .unwrap_or_default();
    let mut rows = Vec::new();
    crate::data::source::reserve_source(&mut rows, categories.len())?;
    for index in 0..categories.len() {
        let source_row = data.source_row(index);
        let group = groups.as_ref().map(|values| values[index].as_str());

        if (0..index).any(|previous| {
            categories[previous] == categories[index]
                && groups.as_ref().map(|values| values[previous].as_str()) == group
        }) {
            return Err(Error::Data(format!(
                "dumbbell: duplicate normalized (category, group) key ({:?}, {:?}) at source row {}",
                categories[index],
                group,
                source_row
            )));
        }

        let category_index =
            resolved_level_index(&category_levels, &categories[index], "category", source_row)?;
        let group_index = group
            .map(|group| resolved_level_index(&group_levels, group, "group", source_row))
            .transpose()?
            .unwrap_or(0);
        rows.push(DumbbellRow {
            category_index,
            group_index,
            left: left[index],
            right: right[index],
            alpha: ObservationAlpha::resolve(data, &alpha, index)?,
            line_alpha: ObservationAlpha::resolve(data, &line_alpha, index)?,
        });
    }

    let range = crate::data::min_max(left.into_iter().chain(right), "dumbbell")?;
    Ok((
        DumbbellData {
            group_count: group_levels.len().max(1),
            rows,
            range,
        },
        category_levels,
    ))
}

fn endpoint_style(
    chart: &crate::builder::PlotDescription,
    theme: &crate::theme::ResolvedTheme,
    color: crate::Color,
) -> crate::render::backend::PointStyle {
    let mut point = chart.point_style(
        theme,
        (color.red(), color.green(), color.blue()),
        theme.marks.get_points().get_radii().get_dumbbell(),
    );
    // Exact endpoint paint replaces the entire fill. Theme geometry and the
    // independent outline policy still apply to both marks and literal keys.
    point.color = color;
    point
}

fn resolved_level_index(
    levels: &[String],
    value: &str,
    dimension: &str,
    source_row: usize,
) -> Result<usize, Error> {
    levels
        .iter()
        .position(|level| level == value)
        .ok_or_else(|| {
            Error::Data(format!(
                "dumbbell: normalized {dimension} {value:?} at source row {source_row} is absent from the resolved {dimension} order"
            ))
        })
}

fn row_y(category_index: usize, category_count: usize) -> f64 {
    category_count as f64 - category_index as f64 - 0.5
}

fn replicate_y(
    category_index: usize,
    category_count: usize,
    group_index: usize,
    group_count: usize,
) -> f64 {
    let center = row_y(category_index, category_count);
    let slot = 0.72 / group_count as f64;
    center + 0.36 - slot * (group_index as f64 + 0.5)
}

#[cfg(test)]
mod tests;
