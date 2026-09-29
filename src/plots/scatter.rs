//! `lichtung-plot::plots::scatter` — scatter plot implementation.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]

use crate::Error;

use crate::decorations::legend::LegendEntry;
use crate::render::backend::{with_clip, Backend, Point2D, PointStyle};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::Plot;

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
}

#[derive(Debug)]
pub(crate) struct PreparedScatterInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedScatter)>,
}

#[derive(Debug)]
pub(crate) struct PreparedScatter {
    pub(crate) ranges: [(f64, f64); 2],
    groups: Vec<ScatterGroup>,
}

#[derive(Debug)]
struct ScatterGroup {
    label: Option<String>,
    points: Vec<(f64, f64)>,
}

#[derive(Debug)]
pub(crate) struct ScatterPayload {
    clip: crate::layout::PanelBox,
    pub(crate) groups: Vec<(Vec<Point2D>, PointStyle)>,
    legends: Vec<LegendEntry>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

/// Pre-selection hard size for a complete primary point. Exact data windows
/// may clip a point at their boundary; only automatic axes require full ink.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    theme: &crate::theme::ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = chart
        .prepared_scatter
        .as_ref()
        .and_then(|panels| {
            panels
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| Error::Render("Scatter intrinsic requires root-prepared marks".into()))?;
    // This identity is independent of visibility, coincidence, and clipping.
    // Group cardinality is charged before checking each group's source length.
    let resources = theme.prepared_font_environment()?.generation_resources();
    resources
        .claim_visits(prepared.groups.len() as u128)
        .map_err(|error| {
            Error::Render(format!(
                "Scatter intrinsic group-count admission: {error:?}"
            ))
        })?;
    let count = prepared.groups.iter().try_fold(0_u128, |total, group| {
        total
            .checked_add(group.points.len() as u128)
            .ok_or_else(|| Error::Render("Scatter primary mark cardinality overflowed".into()))
    })?;
    if count == 0 {
        return Err(Error::Render(
            "Scatter has no retained primary marks".into(),
        ));
    }
    let style = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.scatter);
    crate::render::backend::validate_point_style(&style)?;
    // Reuse the canonical painted-ink owner on one origin-relative point; no
    // source point is projected or clipped and scratch/record growth is admitted.
    let (width, height) = crate::render::geometry::point_ink_dimensions(theme, &style)?;
    Ok((
        if chart.user_x_limits().is_none() {
            width
        } else {
            0.0
        },
        if chart.user_y_limits().is_none() {
            height
        } else {
            0.0
        },
    ))
}

impl PreparedScatter {
    pub(crate) fn primary_mark_count(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<u128, Error> {
        resources
            .claim_visits(self.groups.len() as u128)
            .map_err(|error| {
                Error::Render(format!("Scatter primary-mark count admission: {error:?}"))
            })?;
        self.groups.iter().try_fold(0_u128, |count, group| {
            count
                .checked_add(group.points.len() as u128)
                .ok_or_else(|| Error::Render("Scatter primary mark count overflowed".into()))
        })
    }

    pub(crate) fn hue_levels(&self) -> impl Iterator<Item = &str> {
        self.groups
            .iter()
            .filter_map(|group| group.label.as_deref())
    }

    pub(crate) fn new(chart: &Plot, data: crate::data::PlotData<'_>) -> Result<Self, Error> {
        data.claim_mark_rows()?;
        #[cfg(test)]
        STAGES.with(|stages| {
            let mut counts = stages.get();
            counts[0] += 1;
            stages.set(counts);
        });
        let x_col = chart
            .x_col()
            .ok_or_else(|| Error::Data("scatter: x column not set".into()))?;
        let y_col = chart
            .y_col()
            .ok_or_else(|| Error::Data("scatter: y column not set".into()))?;
        let rows = crate::data::aligned_finite_rows(data, x_col, y_col, "scatter")?;
        if rows.is_empty() {
            return Err(Error::Data("scatter: no rows".into()));
        }
        data.claim_source_visits(rows.len())?;
        data.claim_source_visits(rows.len())?;
        let ranges = [
            crate::data::min_max(rows.iter().map(|(_, x, _)| *x), "scatter")?,
            crate::data::min_max(rows.iter().map(|(_, _, y)| *y), "scatter")?,
        ];
        let mut groups = Vec::new();
        if let Some(hue_col) = chart.hue_col() {
            let hues = crate::data::strict_strings(data, hue_col, "scatter")?;
            let retained_hues = data.collect_source(
                rows.iter()
                    .map(|(row, _, _)| data.copy_source_string(&hues[*row])),
            )?;
            let keys = data.source_levels(&retained_hues)?;
            data.reserve_source(&mut groups, keys.len())?;
            for label in keys {
                data.claim_source_visits(rows.len())?;
                let count = rows
                    .iter()
                    .filter(|(row, _, _)| hues[*row] == label)
                    .count();
                let mut points = Vec::new();
                data.reserve_source(&mut points, count)?;
                data.claim_source_visits(rows.len())?;
                points.extend(
                    rows.iter()
                        .filter(|(row, _, _)| hues[*row] == label)
                        .map(|(_, x, y)| (*x, *y)),
                );
                groups.push(ScatterGroup {
                    label: Some(label),
                    points,
                });
            }
        } else {
            data.reserve_source(&mut groups, 1)?;
            groups.push(ScatterGroup {
                label: None,
                points: data.collect_source(rows.iter().map(|(_, x, y)| Ok((*x, *y))))?,
            });
        }
        Ok(Self { ranges, groups })
    }

    pub(crate) fn guide_entries_in_order(
        &self,
        chart: &Plot,
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
        order: &[usize],
    ) -> Result<Vec<LegendEntry>, Error> {
        let mut entries = Vec::new();
        data.reserve_source(
            &mut entries,
            self.groups
                .iter()
                .filter(|group| group.label.is_some())
                .count(),
        )?;
        for (slot, &group_index) in order.iter().enumerate() {
            let Some(label) = &self.groups[group_index].label else {
                continue;
            };
            let color = theme.colors[chart.hue_color_index(label, slot) % theme.colors.len()];
            entries.push(LegendEntry::point(
                chart.point_legend_identity(label, data)?,
                chart.point_style(theme, color, theme.marks.point_sizes.scatter),
            ));
        }
        Ok(entries)
    }

    /// Semantic order is independent of panel assignment and font discovery.
    pub(crate) fn group_order(
        &self,
        chart: &Plot,
        data: crate::data::PlotData<'_>,
    ) -> Result<Vec<usize>, Error> {
        let mut order = data.collect_source((0..self.groups.len()).map(Ok))?;
        crate::data::source::sort_source_by(&mut order, Some(data), |a, b| {
            let key = |index: usize| {
                self.groups[index]
                    .label
                    .as_ref()
                    .map_or(0, |label| chart.hue_color_index(label, usize::MAX))
            };
            (key(*a), *a).cmp(&(key(*b), *b))
        })?;
        Ok(order)
    }

    fn project(
        &self,
        chart: &Plot,
        projection: &crate::layout::Projection,
        theme: &crate::theme::ResolvedTheme,
        data: crate::data::PlotData<'_>,
    ) -> Result<ScatterPayload, Error> {
        #[cfg(test)]
        STAGES.with(|stages| {
            let mut counts = stages.get();
            counts[2] += 1;
            stages.set(counts);
        });
        let mut groups = Vec::new();
        data.reserve_source(&mut groups, self.groups.len())?;
        let order = self.group_order(chart, data)?;
        let legends = self.guide_entries_in_order(chart, theme, data, &order)?;
        for (index, group_index) in order.into_iter().enumerate() {
            let group = &self.groups[group_index];
            let color = group.label.as_ref().map_or_else(
                || theme.colors.first().copied().unwrap_or((0, 0, 0)),
                |label| theme.colors[chart.hue_color_index(label, index) % theme.colors.len()],
            );
            let style = chart.point_style(theme, color, theme.marks.point_sizes.scatter);
            let points = data.collect_source(group.points.iter().map(|(x, y)| {
                data.claim_source_visits(1)?;
                let (x, y) = projection.project(*x, *y);
                Ok(Point2D::new(x, y))
            }))?;
            groups.push((points, style));
        }
        Ok(ScatterPayload {
            clip: projection.panel_box(),
            groups,
            legends,
            domain_expansions: [None; 2],
        })
    }
}

impl ScatterPayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        STAGES.with(|stages| {
            let mut counts = stages.get();
            counts[3] += 1;
            stages.set(counts);
        });
        with_clip(backend, self.clip, |backend| {
            for (points, style) in &self.groups {
                backend.draw_scatter(points, style)?;
            }
            Ok(())
        })
    }
}

/// Measure the assigned Scatter payload from root-prepared scientific inputs.
/// Projection may change with the assignment; capture only draws this payload.
pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    STAGES.with(|stages| {
        let mut counts = stages.get();
        counts[1] += 1;
        stages.set(counts);
    });
    let theme = context.theme;
    let prepared = chart
        .prepared_scatter
        .as_ref()
        .and_then(|panels| {
            panels
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| {
            Error::Render(
                "Scatter measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    let mut payload = prepared.project(chart, &layout.projection, theme, data)?;
    let point_style = chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.scatter);
    if let Some(ink) = crate::render::geometry::point_ink_bounds(theme, &point_style)? {
        payload.domain_expansions = crate::layout::interior::automatic_mark_ink_expansions(
            prepared.ranges,
            &layout.projection,
            ink,
            [
                chart.user_x_limits().is_none(),
                chart.user_y_limits().is_none(),
            ],
            data,
            context.fit_scope(),
        )?;
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for (points, style) in &payload.groups {
        geometry.scatter(points, style, clip)?;
    }
    let mut result = PlotRenderResult::new(layout, std::mem::take(&mut payload.legends));
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Scatter(payload),
        geometry,
    })?);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;
    use crate::{CategoryValues, NumericValues};

    fn source() -> Plot {
        Plot::scatter(
            NumericValues::new("x", [2.0, f64::NAN, 1.0, 3.0]),
            NumericValues::new("y", [4.0, 999.0, 2.0, 6.0]),
        )
        .hue(CategoryValues::new("hue", ["b", "omitted", "a", "b"]).unwrap())
        .erase()
    }

    #[test]
    fn complete_scatter_point_ink_sets_only_automatic_axis_hard_size() {
        let source = source();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        let original = chart.intrinsic_body_minimum(&theme).unwrap();
        theme.marks.point_sizes.scatter = 54;
        let styled = chart.intrinsic_body_minimum(&theme).unwrap();
        // 54px radius with a 5.4px outline rounds that outline to 5530
        // quanta before the canonical bounds are formed.
        let diameter = 108.0 + 5530.0 / 1024.0;
        assert_eq!(styled.0.to_bits(), original.0.max(diameter).to_bits());
        assert_eq!(styled.1.to_bits(), original.1.max(diameter).to_bits());
        theme.marks.primary_point.shape = crate::PointShape::TriangleUp;
        let triangle = chart.intrinsic_body_minimum(&theme).unwrap();
        assert!(triangle.0 < styled.0);
        theme.marks.primary_point.shape = crate::PointShape::Circle;
        assert!(matches!(
            chart.measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0),
                &theme,
            )),
            Err(crate::layout::fit::PlanError::Fit(_))
        ));

        theme.marks.primary_point.fill_alpha = 0;
        theme.marks.primary_point.outline.color =
            crate::render::backend::Rgba::new(255, 255, 255, 0);
        assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), original);

        let source = Plot::scatter(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [3.0, 4.0]),
        )
        .x_limits(0.0, 3.0)
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        theme.marks.primary_point.fill_alpha = 255;
        theme.marks.primary_point.outline.color = crate::render::backend::Rgba::rgb(255, 255, 255);
        let exact = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(exact.0.to_bits(), original.0.to_bits());
        assert!(exact.1 >= diameter);
    }

    #[test]
    fn automatic_scatter_edges_request_both_domains_from_complete_painted_ink() {
        let source = Plot::scatter(
            NumericValues::new("x", [0.0, 10.0]),
            NumericValues::new("y", [0.0, 10.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        theme.marks.point_sizes.scatter = 18;
        let panel = crate::layout::PanelBox::new(30.0, 190.0, 40.0, 180.0);
        let result = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let crate::layout::interior::MarkPayload::Scatter(selected) =
            &result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Scatter")
        };
        let [x, y] = selected
            .domain_expansions
            .map(|expansion| expansion.unwrap().limits);
        assert!(x.0 < result.layout.projection.x_range().0);
        assert!(x.1 > result.layout.projection.x_range().1);
        assert!(y.0 < result.layout.projection.y_range().0);
        assert!(y.1 > result.layout.projection.y_range().1);
        let projection = crate::layout::Projection::new(x, y, panel.as_tuple());
        let point = chart.point_style(&theme, (0, 0, 0), 18);
        let ink = crate::render::geometry::point_ink_bounds(&theme, &point)
            .unwrap()
            .unwrap();
        let low = projection.project(0.0, 0.0);
        let high = projection.project(10.0, 10.0);
        assert!(low.0 + ink.left >= panel.x0 && low.1 + ink.bottom >= panel.y0);
        assert!(high.0 + ink.right <= panel.x1 && high.1 + ink.top <= panel.y1);

        let source = Plot::scatter(
            NumericValues::new("x", [0.0, 10.0]),
            NumericValues::new("y", [0.0, 10.0]),
        )
        .x_limits(0.0, 10.0)
        .erase();
        let resources = OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let exact = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let crate::layout::interior::MarkPayload::Scatter(selected) =
            &exact.marks.as_ref().unwrap().payload
        else {
            panic!("selected Scatter")
        };
        assert!(selected.domain_expansions[0].is_none());
        assert!(selected.domain_expansions[1].is_some());
    }

    #[test]
    fn prepared_guide_glyphs_match_selected_marks_without_fonts_or_a_panel_search() {
        let plot = source();
        let theme = crate::Theme::Publication.resolve();
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        let prepared = PreparedScatter::new(&plot, data).unwrap();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let order = prepared.group_order(&plot, data).unwrap();
        let entries = prepared
            .guide_entries_in_order(&plot, &theme, data, &order)
            .unwrap();
        let selected = prepared
            .project(
                &plot,
                &crate::layout::Projection::new(
                    prepared.ranges[0],
                    prepared.ranges[1],
                    (30.0, 600.0, 30.0, 400.0),
                ),
                &theme,
                data,
            )
            .unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .zip(&selected.legends)
            .all(|(prepared, selected)| prepared.same_child_input(selected)));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn custom_primary_point_paints_match_prepared_guides_and_hidden_outline_stays_hidden() {
        let plot = source();
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        let prepared = PreparedScatter::new(&plot, data).unwrap();
        let order = prepared.group_order(&plot, data).unwrap();
        let mut theme = crate::Theme::Publication.resolve();
        let original = prepared
            .project(
                &plot,
                &crate::layout::Projection::new(
                    prepared.ranges[0],
                    prepared.ranges[1],
                    (30.0, 600.0, 30.0, 400.0),
                ),
                &theme,
                data,
            )
            .unwrap();
        theme.marks.primary_point.fill_alpha = 128;
        theme.marks.primary_point.shape = crate::PointShape::TriangleUp;
        theme.marks.primary_point.outline.color = crate::render::backend::Rgba::new(3, 5, 7, 64);
        theme.marks.point_sizes.scatter = 5;
        let selected = prepared
            .project(
                &plot,
                &crate::layout::Projection::new(
                    prepared.ranges[0],
                    prepared.ranges[1],
                    (30.0, 600.0, 30.0, 400.0),
                ),
                &theme,
                data,
            )
            .unwrap();
        assert_eq!(original.groups[0].0, selected.groups[0].0);
        let style = &selected.groups[0].1;
        assert_eq!(style.color.alpha(), 128);
        assert_eq!(style.shape, crate::PointShape::TriangleUp);
        assert_eq!(style.radius, 5);
        assert_eq!(style.outline.unwrap().color.alpha(), 64);
        assert!(selected
            .legends
            .iter()
            .zip(
                prepared
                    .guide_entries_in_order(&plot, &theme, data, &order)
                    .unwrap()
            )
            .all(|(mark, guide)| mark.same_child_input(&guide)));
        assert!(crate::decorations::legend::merge_exact_entries(
            [original.legends[0].clone(), selected.legends[0].clone()],
            "custom primary point",
            &OperationGenerationResources::new(),
        )
        .unwrap_err()
        .to_string()
        .contains("glyphs must match exactly"));
        let hidden = Plot::scatter(
            NumericValues::new("x", [1.0, 2.0]),
            NumericValues::new("y", [2.0, 3.0]),
        )
        .hide_point_outline()
        .erase();
        assert!(hidden.point_style(&theme, (1, 2, 3), 3).outline.is_none());
    }

    #[test]
    fn root_prepares_scatter_once_without_geometry_backend_layout() {
        STAGES.with(|stages| stages.set([0; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        let plot = source().size(640, 480);
        plot.preflight().unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts[0], 1);
        assert!(counts[1] > 0);
        assert_eq!(counts[1], counts[2]);
        assert_eq!(counts[3], 1);
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn missing_preparation_and_invalid_guides_reject_before_fonts_or_backend_work() {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        let plot = source();
        let theme = crate::Theme::Publication.resolve();
        let context = PlotRenderContext::new(
            plot.data(),
            crate::layout::PanelBox::new(0.0, 100.0, 0.0, 100.0),
            &theme,
        );
        let error = plot.measure_selected_interior(context).unwrap_err();
        assert!(error.into_error().to_string().contains("root-prepared"));
        for coordinate in [f64::INFINITY, 100.0] {
            let plot = Plot::scatter(
                NumericValues::new("x", [0.0, 1.0]),
                NumericValues::new("y", [0.0, 1.0]),
            )
            .guide(crate::GuideSpec::horizontal(
                coordinate,
                crate::GuideStyle::new((0, 0, 0), 1.0, crate::LinePattern::Solid).unwrap(),
            ))
            .size(1, 1);
            assert!(matches!(plot.preflight(), Err(Error::Data(_))));
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn facet_root_prepares_each_scatter_component_once_and_captures_only_selected_payloads() {
        let plot = Plot::scatter(
            NumericValues::new("x", [0.0, 1.0, 10.0, 20.0]),
            NumericValues::new("y", [0.0, 1.0, 1.0, 0.0]),
        )
        .facet(crate::FacetSpec::column(
            CategoryValues::new("panel", ["a", "a", "b", "b"]).unwrap(),
        ))
        .size(960, 640);
        STAGES.with(|stages| stages.set([0; 4]));
        plot.preflight().unwrap();
        let counts = STAGES.with(std::cell::Cell::get);
        assert_eq!(counts[0], 2);
        assert_eq!(counts[1], counts[2]);
        assert_eq!(counts[3], 2);
    }

    #[test]
    fn selected_scatter_capture_does_not_read_sources_project_generate_ticks_or_shape() {
        use crate::orchestrator::measurements::{self, Counter};
        let plot = Plot::scatter(
            NumericValues::new("x", [2.0, f64::NAN, 1.0, 3.0]),
            NumericValues::new("y", [4.0, 999.0, 2.0, 6.0]),
        )
        .hue(CategoryValues::new("hue", ["b", "omitted", "a", "b"]).unwrap())
        .guide(crate::GuideSpec::horizontal(
            3.0,
            crate::GuideStyle::new((10, 20, 30), 1.0, crate::LinePattern::Dashed).unwrap(),
        ))
        .panel_text_annotation(
            crate::PanelTextAnnotation::new(crate::PanelCorner::TopLeft, "retained annotation")
                .unwrap(),
        )
        .legend(crate::LegendPolicy::Hidden)
        .size(640, 480)
        .erase();
        let theme = crate::Theme::Publication.resolve();
        STAGES.with(|stages| stages.set([0; 4]));
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
        let root = crate::layout::plan_exact_root(
            640,
            480,
            |stage| {
                let inputs =
                    crate::plots::shared_x::prepare_plot(&plot, &stage.generation_resources())?;
                let requirements = crate::layout::foundational_plot_requirements(
                    None,
                    &plot,
                    crate::PlotKind::Scatter,
                    &theme,
                    &crate::layout::ComponentAddress::root(),
                    true,
                    true,
                    true,
                    Some(stage),
                )?;
                let mut presence = stage.reserve_scalar_presence_map()?;
                let mut census = crate::layout::PreparedTextCensus::materialize(
                    requirements,
                    Some(stage),
                    Some(&mut presence),
                )
                .map_err(|error| Error::Render(error.to_string()))?;
                theme.prepare_font_environment_from_census(&mut census, presence, stage)?;
                Ok(inputs)
            },
            |inputs, assigned, _| {
                let attached = inputs.attach(plot.clone())?;
                assert!(std::sync::Arc::ptr_eq(
                    attached.prepared_scatter.as_ref().unwrap(),
                    inputs.scatter.as_ref().unwrap()
                ));
                let context = PlotRenderContext::new(
                    attached.data(),
                    crate::layout::PanelBox::new(80.25, 550.75, 70.5, 410.25),
                    &theme,
                );
                let result = attached
                    .measure_selected_interior(context)
                    .map_err(crate::layout::fit::PlanError::into_error)?;
                assert!(!result.marks.as_ref().unwrap().geometry.bounds().is_empty());
                assert!(!result
                    .annotations
                    .as_ref()
                    .unwrap()
                    .geometry
                    .bounds()
                    .is_empty());
                Ok(crate::layout::MeasuredLayout::new(assigned, result))
            },
            |_, _, selected, stage| {
                let counts = STAGES.with(std::cell::Cell::get);
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
                let projections = crate::layout::Projection::test_projection_calls();
                measurements::start();
                let poison = Plot::scatter(
                    crate::NumericValues::new("x", [f64::INFINITY]),
                    crate::NumericValues::new("y", [f64::NAN]),
                )
                .erase();
                let captured = crate::orchestrator::capture_text_replay_plan(
                    640,
                    480,
                    "selected Scatter test",
                    &theme,
                    stage,
                    |backend| {
                        let context = PlotRenderContext::new(
                            poison.data(),
                            selected.layout.projection.panel_box(),
                            &theme,
                        )
                        .with_retained_body(&selected);
                        poison.render_panel(crate::PlotKind::Scatter, context, backend)?;
                        selected.axes.as_ref().unwrap().draw(backend)
                    },
                )?;
                let work = measurements::finish();
                assert_eq!(
                    crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                    reads
                );
                assert_eq!(
                    crate::layout::Projection::test_projection_calls(),
                    projections
                );
                let after = STAGES.with(std::cell::Cell::get);
                assert_eq!(&after[..3], &counts[..3]);
                assert_eq!(after[3], counts[3] + 1);
                assert_eq!(
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    shapes
                );
                for counter in [
                    Counter::StandardPanelIntrinsic,
                    Counter::StandardCoordinateMeasure,
                    Counter::LegacyPanelConstructor,
                    Counter::AxisRangePreparation,
                    Counter::AxisSpecPreparation,
                    Counter::AxisSourcePreparation,
                    Counter::TickSource,
                    Counter::TickMetricLookup,
                    Counter::TextProposalVisit,
                    Counter::TextExactTrial,
                ] {
                    assert_eq!(work.counters[counter as usize], 0, "{counter:?}");
                }
                assert!(captured
                    .retained_text()
                    .iter()
                    .any(|(_, plan)| plan.source == "retained annotation"));
                Ok(captured)
            },
        )
        .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get)[0], 1);
        assert_eq!(
            crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
            0
        );
        let counts = STAGES.with(std::cell::Cell::get);
        crate::orchestrator::validate_text_replay_plan(
            640,
            480,
            root.plan(),
            "selected Scatter Null replay",
        )
        .unwrap();
        let mut backend = root.plan().take_svg_backend().unwrap();
        root.plan()
            .replay(640, 480, &mut backend, "selected Scatter SVG replay")
            .unwrap();
        assert_eq!(STAGES.with(std::cell::Cell::get), counts);
    }

    #[test]
    fn selected_extents_match_clipped_legacy_ink_for_every_point_shape() {
        use crate::render::geometry::{GeometryBackend, GeometrySnapshot};
        let theme = crate::Theme::Publication.resolve();
        let clip = crate::layout::PanelBox::new(10.25, 100.75, 20.5, 90.25);
        let points = [
            Point2D::new(10.25, 20.5),
            Point2D::new(60.0, 60.0),
            Point2D::new(200.0, 200.0),
        ];
        for shape in [crate::PointShape::Circle, crate::PointShape::TriangleUp] {
            for outline in [true, false] {
                let mut style = source().point_style(&theme, (10, 20, 30), 3);
                style.shape = shape;
                if !outline {
                    style.outline = None;
                }
                for opacity in [0.0, 0.5, 1.0] {
                    style = style.with_opacity(opacity).unwrap();
                    let mut selected = GeometrySnapshot::new(&theme).unwrap();
                    selected.scatter(&points, &style, clip).unwrap();
                    let mut legacy = GeometryBackend::new(&theme).unwrap();
                    legacy.begin(640, 480).unwrap();
                    with_clip(&mut legacy, clip, |backend| {
                        backend.draw_scatter(&points, &style)
                    })
                    .unwrap();
                    legacy.end().unwrap();
                    assert_eq!(selected.bounds(), legacy.bounds());
                    assert!(selected.bounds().iter().all(|ink| ink.left >= clip.x0
                        && ink.right <= clip.x1
                        && ink.bottom >= clip.y0
                        && ink.top <= clip.y1));
                }
            }
        }
    }

    #[test]
    fn semantic_preparation_preserves_aligned_rows_and_observed_hue_order_without_fonts() {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let plot = source();
        let resources = OperationGenerationResources::new();
        let prepared =
            PreparedScatter::new(&plot, plot.data().with_axis_resources(&resources)).unwrap();
        assert_eq!(prepared.ranges, [(1.0, 3.0), (2.0, 6.0)]);
        assert_eq!(prepared.groups.len(), 2);
        assert_eq!(prepared.groups[0].label.as_deref(), Some("b"));
        assert_eq!(prepared.groups[0].points, [(2.0, 4.0), (3.0, 6.0)]);
        assert_eq!(prepared.groups[1].label.as_deref(), Some("a"));
        assert_eq!(prepared.groups[1].points, [(1.0, 2.0)]);
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn semantic_and_projected_buffers_use_the_callers_exact_admission() {
        let plot = source();
        let theme = crate::Theme::Publication.resolve();
        let projection =
            crate::layout::Projection::new((1.0, 3.0), (2.0, 6.0), (10.25, 210.75, 20.5, 120.25));
        let resources = OperationGenerationResources::new();
        let prepared =
            PreparedScatter::new(&plot, plot.data().with_axis_resources(&resources)).unwrap();
        let semantic = (
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        );
        for (visits, bytes) in [(semantic.0 - 1, semantic.1), (semantic.0, semantic.1 - 1)] {
            let limited = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(
                PreparedScatter::new(&plot, plot.data().with_axis_resources(&limited)).is_err()
            );
        }
        let exact = OperationGenerationResources::with_test_limits(semantic.0, semantic.1);
        PreparedScatter::new(&plot, plot.data().with_axis_resources(&exact)).unwrap();
        let resources = OperationGenerationResources::new();
        let payload = prepared
            .project(
                &plot,
                &projection,
                &theme,
                plot.data().with_axis_resources(&resources),
            )
            .unwrap();
        assert_eq!(payload.clip, projection.panel_box());
        assert!((payload.groups[0].0[0].x - 110.5).abs() < 1e-12);
        assert!((payload.groups[0].0[0].y - 70.375).abs() < 1e-12);
        assert_eq!(payload.groups[0].0[1], Point2D::new(210.75, 120.25));
        assert_eq!(payload.groups[1].0, [Point2D::new(10.25, 20.5)]);
        assert_eq!(payload.legends.len(), 2);
        let projected = (
            resources.usage().unwrap().1,
            resources.byte_usage().unwrap(),
        );
        for (visits, bytes) in [
            (projected.0 - 1, projected.1),
            (projected.0, projected.1 - 1),
        ] {
            let limited = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(prepared
                .project(
                    &plot,
                    &projection,
                    &theme,
                    plot.data().with_axis_resources(&limited)
                )
                .is_err());
        }
        let exact = OperationGenerationResources::with_test_limits(projected.0, projected.1);
        prepared
            .project(
                &plot,
                &projection,
                &theme,
                plot.data().with_axis_resources(&exact),
            )
            .unwrap();
    }

    #[test]
    fn invalid_scatter_semantics_reject_without_font_discovery() {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let plot = Plot::scatter(
            NumericValues::new("x", [f64::NAN]),
            NumericValues::new("y", [1.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        assert!(matches!(
            PreparedScatter::new(&plot, plot.data().with_axis_resources(&resources)),
            Err(Error::Data(_))
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn projected_legend_fields_and_values_are_admitted_for_every_candidate() {
        let field = "caller_owned_hue_dimension_".repeat(128);
        let plot = |name: &str| {
            Plot::scatter(
                NumericValues::new("x", [0.0, 1.0]),
                NumericValues::new("y", [1.0, 0.0]),
            )
            .hue(CategoryValues::new(name, ["a", "b"]).unwrap())
            .erase()
        };
        let short = plot("hue");
        let long = plot(&field);
        let resources = OperationGenerationResources::new();
        let prepared =
            PreparedScatter::new(&short, short.data().with_axis_resources(&resources)).unwrap();
        let theme = crate::Theme::Publication.resolve();
        let projection =
            crate::layout::Projection::new((0.0, 1.0), (0.0, 1.0), (10.25, 210.75, 20.5, 120.25));
        let mut usage = Vec::new();
        for plot in [&short, &long] {
            let resources = OperationGenerationResources::new();
            let payload = prepared
                .project(
                    plot,
                    &projection,
                    &theme,
                    plot.data().with_axis_resources(&resources),
                )
                .unwrap();
            for (entry, value) in payload.legends.iter().zip(["a", "b"]) {
                assert_eq!(
                    entry.identity(),
                    &crate::decorations::legend::LegendIdentity::single(
                        plot.hue_name().unwrap(),
                        value
                    )
                );
            }
            let visits = resources.usage().unwrap().1;
            let bytes = resources.byte_usage().unwrap();
            let records = resources.usage().unwrap().0;
            usage.push((visits, bytes, records));
            let limited = OperationGenerationResources::with_test_limits(visits, bytes);
            limited.claim_slots(1_000_000 - records + 1).unwrap();
            assert!(prepared
                .project(
                    plot,
                    &projection,
                    &theme,
                    plot.data().with_axis_resources(&limited)
                )
                .is_err());
            for (visits, bytes) in [(visits - 1, bytes), (visits, bytes - 1)] {
                let limited = OperationGenerationResources::with_test_limits(visits, bytes);
                assert!(prepared
                    .project(
                        plot,
                        &projection,
                        &theme,
                        plot.data().with_axis_resources(&limited)
                    )
                    .is_err());
            }
            let exact = OperationGenerationResources::with_test_limits(2 * visits, 2 * bytes);
            for _ in 0..2 {
                prepared
                    .project(
                        plot,
                        &projection,
                        &theme,
                        plot.data().with_axis_resources(&exact),
                    )
                    .unwrap();
            }
            assert_eq!(exact.usage().unwrap().1, 2 * visits);
            assert_eq!(exact.byte_usage().unwrap(), 2 * bytes);
            assert!(prepared
                .project(
                    plot,
                    &projection,
                    &theme,
                    plot.data().with_axis_resources(&exact)
                )
                .is_err());
        }
        let extra = 2 * (field.len() - "hue".len());
        assert_eq!(
            usage[1].0, usage[0].0,
            "identity byte copying is not processing"
        );
        assert_eq!(
            usage[1].2, usage[0].2,
            "string payload bytes are not records"
        );
        assert_eq!(usage[1].1 - usage[0].1, u64::try_from(extra).unwrap());
        let short_budget = OperationGenerationResources::with_test_limits(usage[0].0, usage[0].1);
        assert!(prepared
            .project(
                &long,
                &projection,
                &theme,
                long.data().with_axis_resources(&short_budget)
            )
            .is_err());
    }
}
