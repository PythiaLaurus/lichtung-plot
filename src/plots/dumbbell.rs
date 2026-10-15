//! Horizontal categorical dumbbells with explicit paired endpoints.

#![expect(
    clippy::cast_precision_loss,
    reason = "validated row counts are converted to bounded plotting coordinates"
)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::layout::AxisSpec;
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D};
use crate::spec::{CategoryOrder, PlotRenderContext, PlotRenderResult};
use crate::Plot;

#[derive(Debug)]
struct DumbbellRow {
    category_index: usize,
    group_index: usize,
    left: f64,
    right: f64,
}

#[derive(Debug)]
pub(crate) struct DumbbellData {
    pub(crate) categories: Vec<String>,
    group_count: usize,
    rows: Vec<DumbbellRow>,
    pub(crate) range: (f64, f64),
}

pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_dumbbell
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
        .ok_or_else(|| Error::Render("Dumbbell intrinsic requires root-prepared pairs".into()))?;
    if prepared.categories.is_empty() || prepared.group_count == 0 || prepared.rows.is_empty() {
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
        if row.category_index >= prepared.categories.len()
            || row.group_index >= prepared.group_count
        {
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
    for color in [endpoints.left_color(), endpoints.right_color()] {
        if let Some(ink) = crate::render::geometry::point_ink_bounds(&chart.point_style(
            theme,
            color,
            theme.marks.point_sizes.dumbbell,
        ))? {
            outward = outward.max(ink.top.max(-ink.bottom));
            numeric = numeric.max(ink.right - ink.left);
        }
    }
    let connector =
        crate::render::geometry::line_ink_width(&theme.marks.dumbbell_connector.backend_style())?;
    outward = outward.max(connector / 2.0);
    numeric = numeric.max(connector);
    let pitch = (outward / (0.5 - offset)).max(1.0 / 1024.0);
    let rows =
        f64::from(u32::try_from(prepared.categories.len()).map_err(|_| {
            Error::Render("Dumbbell category count exceeds coordinate range".into())
        })?);
    let height = pitch * rows;
    if !height.is_finite() {
        return Err(Error::Render(
            "Dumbbell category hard span overflowed".into(),
        ));
    }
    Ok((
        if chart.user_x_limits().is_none() {
            numeric
        } else {
            0.0
        },
        height,
    ))
}

impl DumbbellData {
    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.rows.len() as u128
    }

    pub(crate) fn guide_entries(
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<Vec<LegendEntry>, Error> {
        let endpoints = chart
            .dumbbell_endpoints_spec()
            .ok_or_else(|| Error::Render("Dumbbell guide requires endpoint metadata".into()))?;
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, 2)?;
        for (label, color) in [
            (endpoints.left_label(), endpoints.left_color()),
            (endpoints.right_label(), endpoints.right_color()),
        ] {
            entries.push(LegendEntry::point(
                chart.point_legend_identity(label)?,
                chart.point_style(theme, color, theme.marks.point_sizes.dumbbell),
            ));
        }
        Ok(entries)
    }
}

#[derive(Debug)]
pub(crate) struct PreparedDumbbellInputs {
    pub(crate) panels: Vec<(Option<String>, DumbbellData)>,
}

#[derive(Debug)]
pub(crate) struct DumbbellPayload {
    clip: crate::layout::PanelBox,
    pub(crate) pairs: Vec<[Point2D; 2]>,
    connector: LineStyle,
    points: [crate::render::backend::PointStyle; 2],
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

impl DumbbellPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(3);
        with_clip(backend, self.clip, |backend| {
            for pair in &self.pairs {
                backend.draw_line(pair, &self.connector)?;
                backend.draw_scatter(&pair[..1], &self.points[0])?;
                backend.draw_scatter(&pair[1..], &self.points[1])?;
            }
            Ok(())
        })
    }
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(1);
    let prepared = chart
        .prepared_dumbbell
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| Error::Render("Dumbbell requires root-prepared semantics".into()))?;

    let projection = layout.projection.clone();
    let mut result = project(chart, prepared, layout, context.theme)?;
    let crate::layout::interior::MarkPayload::Dumbbell(payload) = &result
        .marks
        .as_ref()
        .expect("selected Dumbbell marks")
        .payload
    else {
        unreachable!("Dumbbell selected its own mark payload")
    };
    let mut ink = crate::render::geometry::line_ink_bounds(&payload.connector)?;
    for point in &payload.points {
        let next = crate::render::geometry::point_ink_bounds(point)?;
        ink = match (ink, next) {
            (Some(current), Some(next)) => Some(current.union(next)),
            (current, next) => current.or(next),
        };
    }
    if let Some(ink) = ink {
        let count = u32::try_from(prepared.categories.len()).map_err(|_| {
            Error::Render("Dumbbell category count exceeds coordinate range".into())
        })?;
        let requests = crate::layout::interior::automatic_mark_ink_expansions(
            [prepared.range, (0.0, f64::from(count))],
            &projection,
            ink,
            [chart.user_x_limits().is_none(), false],
            context.fit_scope(),
        )?;
        let marks =
            std::sync::Arc::get_mut(result.marks.as_mut().expect("selected Dumbbell marks"))
                .expect("newly selected Dumbbell marks have one owner");
        let crate::layout::interior::MarkPayload::Dumbbell(payload) = &mut marks.payload else {
            unreachable!("Dumbbell selected its own mark payload")
        };
        payload.domain_expansions = requests;
    }
    Ok(result)
}

fn project(
    chart: &Plot,
    prepared: &DumbbellData,
    layout: crate::layout::Layout,
    theme: &crate::theme::ResolvedTheme,
) -> Result<PlotRenderResult, Error> {
    let endpoints = chart
        .dumbbell_endpoints_spec()
        .expect("dumbbell input owns endpoint metadata");
    let category_count = prepared.categories.len();
    let connector = theme.marks.dumbbell_connector.backend_style();
    let mut payload = DumbbellPayload {
        clip: layout.projection.panel_box(),
        pairs: Vec::new(),
        connector,
        points: [
            chart.point_style(
                theme,
                endpoints.left_color(),
                theme.marks.point_sizes.dumbbell,
            ),
            chart.point_style(
                theme,
                endpoints.right_color(),
                theme.marks.point_sizes.dumbbell,
            ),
        ],
        domain_expansions: [None; 2],
    };
    crate::data::source::reserve_source(&mut payload.pairs, prepared.rows.len())?;

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
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for pair in &payload.pairs {
        geometry.line(pair, &payload.connector, Some(clip))?;
        geometry.scatter(&pair[..1], &payload.points[0], clip)?;
        geometry.scatter(&pair[1..], &payload.points[1], clip)?;
    }
    let legend = DumbbellData::guide_entries(chart, theme)?;
    #[cfg(test)]
    count(2);
    let mut result = PlotRenderResult::new(layout, legend);
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Dumbbell(payload),
            geometry,
        },
    )?);
    Ok(result)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let left = crate::data::strict_finite_f64(
        data,
        chart
            .y_col()
            .ok_or_else(|| Error::Data("dumbbell: left value column not set".into()))?,
        "dumbbell",
    )?;
    let right = crate::data::strict_finite_f64(
        data,
        chart
            .dumbbell_right_col()
            .ok_or_else(|| Error::Data("dumbbell: right value column not set".into()))?,
        "dumbbell",
    )?;

    crate::layout::continuous_auto_range(
        crate::data::min_max(left.into_iter().chain(right), "dumbbell")?,
        chart.x_axis_spec(),
    )
}

pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
) -> Result<DumbbellData, Error> {
    let data = data.into();
    data.validate_row_source()?;
    #[cfg(test)]
    count(0);
    if chart.y_axis_spec() != &AxisSpec::default() {
        return Err(Error::Data(
            "dumbbell: categorical y-axis limits are not supported".into(),
        ));
    }
    if chart.hue_col().is_some() || chart.color_col().is_some() {
        return Err(Error::Data(
            "dumbbell: hue and continuous color channels are not supported; endpoint colors are explicit"
                .into(),
        ));
    }
    if !matches!(chart.category_order_spec(), CategoryOrder::Observed) {
        return Err(Error::Data(
            "dumbbell: category ordering is not supported; category order is first-observed".into(),
        ));
    }
    let category_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("dumbbell: category column not set".into()))?;
    let left_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("dumbbell: left value column not set".into()))?;
    let right_col = chart
        .dumbbell_right_col()
        .ok_or_else(|| Error::Data("dumbbell: right value column not set".into()))?;
    let categories =
        crate::data::strict_normalized_strings(data, category_col, "dumbbell", "category")?;
    let left = crate::data::strict_finite_f64(data, left_col, "dumbbell")?;
    let right = crate::data::strict_finite_f64(data, right_col, "dumbbell")?;
    let groups = chart
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
        });
    }

    let range = crate::data::min_max(left.into_iter().chain(right), "dumbbell")?;
    Ok(DumbbellData {
        categories: category_levels,
        group_count: group_levels.len().max(1),
        rows,
        range,
    })
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

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
}

#[cfg(test)]
fn count(stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[stage] += 1;
        counts.set(next);
    });
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
mod tests {
    use super::*;
    use crate::font::canonical::OperationContext;
    use crate::{CategoryValues, DumbbellEndpoints, FacetScales, FacetSpec, NumericValues};

    fn source() -> Plot<crate::kind::Dumbbell> {
        Plot::dumbbell(
            CategoryValues::new("category", ["b", "a", "b", "a"]).unwrap(),
            NumericValues::new("left", [1.0, 2.0, 3.0, 4.0]),
            NumericValues::new("right", [4.0, 3.0, 2.0, 1.0]),
            DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40)).unwrap(),
        )
        .group(CategoryValues::new("replicate", ["one", "one", "two", "two"]).unwrap())
    }

    #[test]
    fn grouped_dumbbell_pitch_tracks_only_occupied_endpoint_slots() {
        let plot = source().erase();
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources, None).unwrap();
        let chart = inputs.attach(plot).unwrap();
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let base = chart.intrinsic_body_minimum(&theme).unwrap();
        theme.marks.point_sizes.dumbbell = 54;
        let styled = chart.intrinsic_body_minimum(&theme).unwrap();
        assert!(styled.0 > base.0);
        assert!(styled.1 > 300.0);

        let plot = source().x_limits(0.0, 5.0).erase();
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources, None).unwrap();
        let chart = inputs.attach(plot).unwrap();
        let exact = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(exact.0.to_bits(), base.0.to_bits());
        assert_eq!(exact.1.to_bits(), styled.1.to_bits());
    }

    #[test]
    fn semantic_limits_and_shared_facets_prepare_once() {
        let plot = source().erase();
        STAGES.with(|counts| counts.set([0; 4]));
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources, None).unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), [1, 0, 0, 0]);
        assert_eq!(inputs.axes[0].1.ranges, [(0.85, 4.15), (0.0, 2.0)]);

        crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None).unwrap();
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        for scales in [
            FacetScales::Fixed,
            FacetScales::FreeX,
            FacetScales::FreeY,
            FacetScales::Free,
        ] {
            STAGES.with(|counts| counts.set([0; 4]));
            source()
                .facet(
                    FacetSpec::column(
                        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
                    )
                    .scales(scales),
                )
                .size(960, 640)
                .preflight()
                .unwrap();
            let counts = STAGES.with(std::cell::Cell::get);
            assert_eq!(counts[0], 2);
            assert_eq!(counts[1], counts[2]);
            assert_eq!(counts[3], 2);
        }
        STAGES.with(|counts| counts.set([0; 4]));
        crate::Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 480)
            .panel(0, 0, source())
            .unwrap()
            .panel(
                0,
                1,
                Plot::scatter(
                    NumericValues::new("x", [-1.0, 8.0]),
                    NumericValues::new("y", [1.0, 2.0]),
                ),
            )
            .unwrap()
            .x_range_group([(0, 0), (0, 1)])
            .unwrap()
            .preflight()
            .unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts[0], 1);
        assert_eq!(counts[1], counts[2]);
        assert_eq!(counts[3], 1);
    }

    #[test]
    fn projected_limits_and_geometry_match_drawn_ink() {
        let source = source().x_limits(1.5, 3.5).erase();
        let theme = crate::Theme::Publication.resolve().unwrap();
        let resources = OperationContext::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources, None).unwrap();
        let attached = inputs.attach(source).unwrap();
        let selected = attached
            .measure_selected_interior(PlotRenderContext::new(
                attached.data(),
                crate::layout::PanelBox::new(10.25, 410.75, 20.5, 320.25),
                &theme,
            ))
            .unwrap();
        let project = || {
            super::project(
                &attached,
                &inputs.dumbbell.as_ref().unwrap().panels[0].1,
                selected.layout.clone(),
                &theme,
            )
        };
        let projected = project().unwrap();
        project().unwrap();
        let plan = projected.marks.unwrap();
        let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
        drawn.begin(640, 480).unwrap();
        plan.draw(&mut drawn).unwrap();
        drawn.end().unwrap();
        assert_eq!(plan.geometry.bounds(), drawn.bounds());
    }

    #[test]
    fn reconciled_facet_guides_are_validated_before_fonts() {
        for scales in [
            FacetScales::Fixed,
            FacetScales::FreeX,
            FacetScales::FreeY,
            FacetScales::Free,
        ] {
            for (value, inside_union) in [
                (5.0, true),
                (100.0, false),
                (f64::NAN, false),
                (f64::INFINITY, false),
            ] {
                let plot = Plot::dumbbell(
                    CategoryValues::new("category", ["a", "b", "a", "b"]).unwrap(),
                    NumericValues::new("left", [0.0, 1.0, 10.0, 11.0]),
                    NumericValues::new("right", [0.5, 1.5, 10.5, 11.5]),
                    DumbbellEndpoints::new("Before", (20, 80, 140), "After", (180, 60, 40))
                        .unwrap(),
                )
                .guide(crate::GuideSpec::vertical(
                    value,
                    crate::GuideStyle::new((0, 0, 0), 1.0, crate::LinePattern::Solid).unwrap(),
                ))
                .facet(
                    FacetSpec::column(
                        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
                    )
                    .scales(scales),
                )
                .erase();
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(|count| count.set(0));
                let prepared =
                    crate::plots::shared_x::prepare_plot(&plot, &OperationContext::new(), None);
                if inside_union && matches!(scales, FacetScales::Fixed | FacetScales::FreeY) {
                    let prepared = prepared.unwrap();
                    assert_eq!(prepared.axes[0].1.ranges[0], prepared.axes[1].1.ranges[0]);
                } else {
                    assert!(matches!(prepared, Err(Error::Data(_))));
                }
                assert_eq!(
                    crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                    0
                );
            }
        }
    }
}
