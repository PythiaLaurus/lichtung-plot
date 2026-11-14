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

use crate::alpha::{
    marks::{point_ink, push_point, ObservationAlpha},
    AlphaTarget, AlphaUnitKind,
};
use crate::Error;

use crate::decorations::legend::source::PointRole;
use crate::decorations::legend::GuideSourceKey;
use crate::render::backend::{with_clip, Backend, Point2D, PointStyle};
use crate::spec::{PlotRenderContext, PlotRenderResult};
use crate::theme::{AccentColors, CategoryColors};
use crate::Plot;

#[cfg(test)]
mod tests;

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[usize; 4]> = const { std::cell::Cell::new([0; 4]) };
}

#[derive(Debug)]
pub(crate) struct PreparedScatter {
    groups: Vec<ScatterGroup>,
}

#[derive(Debug)]
struct ScatterGroup {
    label: Option<String>,
    points: Vec<ScatterPoint>,
}

#[derive(Debug)]
struct ScatterPoint {
    x: f64,
    y: f64,
    alpha: ObservationAlpha,
}

#[derive(Debug)]
pub(crate) struct ScatterPayload {
    clip: crate::layout::PixelRect,
    pub(crate) groups: Vec<(Vec<Point2D>, PointStyle)>,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
}

/// Pre-selection hard size for a complete primary point. Exact data windows
/// may clip a point at their boundary; only automatic axes require full ink.
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = registered
        .map(|registered| registered.family.as_ref())
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Scatter(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("Scatter intrinsic requires root-prepared marks".into()))?;
    // This identity is independent of visibility, coincidence, and clipping.

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
    let style = chart.description().point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_scatter(),
    );
    crate::render::backend::validate_point_style(&style)?;
    // Reuse the canonical painted-ink owner on one origin-relative point; no
    // source point is projected or clipped and scratch storage is reserved.
    let (width, height) = point_ink(
        &style,
        prepared.groups.iter().flat_map(|group| {
            group
                .points
                .iter()
                .map(|point| (point.x, point.y, point.alpha))
        }),
    )?
    .map_or((0.0, 0.0), crate::alpha::marks::PointInk::dimensions);
    let raw_windows = registered.expect("prepared occurrence").raw_windows();
    Ok((
        if raw_windows[0].is_none() { width } else { 0.0 },
        if raw_windows[1].is_none() {
            height
        } else {
            0.0
        },
    ))
}

impl PreparedScatter {
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        let style = chart.description().point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_scatter(),
        );
        for point in self.groups.iter().flat_map(|group| &group.points) {
            guide.point(
                crate::decorations::legend::alpha::AlphaGlyphRole::Primary,
                point.alpha,
                style.clone(),
                false,
            )?;
        }
        Ok(())
    }

    pub(crate) fn checked_primary_mark_count(&self) -> Result<u128, Error> {
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

    pub(crate) fn new(
        chart: &crate::builder::PlotDescription,
        data: crate::data::PlotData<'_, '_>,
    ) -> Result<(Self, [(f64, f64); 2]), Error> {
        data.validate_row_source()?;
        let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
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

        let ranges = [
            crate::data::min_max(rows.iter().map(|(_, x, _)| *x), "scatter")?,
            crate::data::min_max(rows.iter().map(|(_, _, y)| *y), "scatter")?,
        ];
        let mut groups = Vec::new();
        if let Some(hue_col) = chart.hue_col() {
            let hues = crate::data::strict_strings(data, hue_col, "scatter")?;
            let retained_hues = crate::data::source::collect_source(
                rows.iter()
                    .map(|(row, _, _)| crate::data::source::copy_source_string(&hues[*row])),
            )?;
            let keys = crate::data::source::source_levels(&retained_hues)?;
            crate::data::source::reserve_source(&mut groups, keys.len())?;
            for label in keys {
                let count = rows
                    .iter()
                    .filter(|(row, _, _)| hues[*row] == label)
                    .count();
                let mut points = Vec::new();
                crate::data::source::reserve_source(&mut points, count)?;

                for &(row, x, y) in rows.iter().filter(|(row, _, _)| hues[*row] == label) {
                    points.push(ScatterPoint {
                        x,
                        y,
                        alpha: ObservationAlpha::resolve(data, &alpha, row)?,
                    });
                }
                groups.push(ScatterGroup {
                    label: Some(label),
                    points,
                });
            }
        } else {
            crate::data::source::reserve_source(&mut groups, 1)?;
            groups.push(ScatterGroup {
                label: None,
                points: crate::data::source::collect_source(rows.iter().map(|&(row, x, y)| {
                    Ok(ScatterPoint {
                        x,
                        y,
                        alpha: ObservationAlpha::resolve(data, &alpha, row)?,
                    })
                }))?,
            });
        }
        if let Some(exact) = chart.categorical_map_spec() {
            for group in &groups {
                if let Some(label) = &group.label {
                    if exact.color_in(label).is_none() {
                        return Err(Error::Data(format!(
                            "scatter: category {label:?} has no caller-owned color"
                        )));
                    }
                }
            }
        }
        Ok((Self { groups }, ranges))
    }

    fn resolved_colors(
        &self,
        binding: &crate::orchestrator::registration::CategoricalBinding,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: crate::data::PlotData<'_, '_>,
    ) -> Result<CategoryColors, Error> {
        let resources = data.source_resources();
        let keys = if let Some(global) = binding.parent.as_deref() {
            crate::data::source::collect_source(
                global.values().map(crate::data::source::copy_source_string),
            )?
        } else {
            let mut keys = Vec::new();
            crate::data::source::reserve_source(&mut keys, self.groups.len())?;

            for group in &self.groups {
                if let Some(label) = &group.label {
                    keys.push(crate::data::source::copy_source_string(label)?);
                }
            }
            keys
        };
        binding.resolve(keys.iter().map(String::as_str), theme, resources)
    }

    pub(crate) fn guide_entries_in_order(
        &self,
        chart: &Plot,
        binding: &crate::orchestrator::registration::CategoricalBinding,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: crate::data::PlotData<'_, '_>,
        order: &[usize],
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let map = self.resolved_colors(binding, theme, data)?;
        self.guide_entries_with_colors(chart, theme, order, &map)
    }

    fn guide_entries_with_colors(
        &self,
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,

        order: &[usize],
        map: &CategoryColors,
    ) -> Result<Vec<GuideSourceKey>, Error> {
        let mut entries = Vec::new();
        crate::data::source::reserve_source(
            &mut entries,
            self.groups
                .iter()
                .filter(|group| group.label.is_some())
                .count(),
        )?;
        for &group_index in order {
            let Some(label) = &self.groups[group_index].label else {
                continue;
            };
            let color = map.color(label)?;
            entries.push(
                GuideSourceKey::hue_only(chart.description().point_legend_identity(label)?)?
                    .with_point(
                        chart.description().point_style(
                            theme,
                            color,
                            theme.marks.get_points().get_radii().get_scatter(),
                        ),
                        PointRole::Primary,
                    ),
            );
        }
        Ok(entries)
    }

    /// Semantic order is independent of panel assignment and font discovery.
    pub(crate) fn group_order(
        &self,
        binding: &crate::orchestrator::registration::CategoricalBinding,
    ) -> Result<Vec<usize>, Error> {
        let mut order = crate::data::source::collect_source((0..self.groups.len()).map(Ok))?;
        let ranks = crate::data::source::collect_source(self.groups.iter().enumerate().map(
            |(index, group)| {
                if let Some(label) = &group.label {
                    if let Some(parent) = binding.parent.as_deref() {
                        return parent.values().position(|key| key == label).ok_or_else(|| {
                            Error::Data(format!(
                                "scatter hue {label:?} is missing from the prepared parent domain"
                            ))
                        });
                    }
                }
                Ok(index)
            },
        ))?;
        crate::data::source::sort_source_by(&mut order, |a, b| {
            (ranks[*a], *a).cmp(&(ranks[*b], *b))
        });
        Ok(order)
    }

    fn project(
        &self,
        chart: &Plot,
        binding: &crate::orchestrator::registration::CategoricalBinding,
        projection: &crate::layout::Projection,
        theme: &crate::text::TextRequestContext<'_, '_>,
        data: crate::data::PlotData<'_, '_>,
    ) -> Result<ScatterPayload, Error> {
        #[cfg(test)]
        STAGES.with(|stages| {
            let mut counts = stages.get();
            counts[2] += 1;
            stages.set(counts);
        });
        let mut groups = Vec::new();
        crate::data::source::reserve_source(&mut groups, self.groups.len())?;
        let order = self.group_order(binding)?;
        let map = self.resolved_colors(binding, theme, data)?;

        let accent = self
            .groups
            .iter()
            .any(|group| group.label.is_none())
            .then(|| AccentColors::resolve(theme.marks.get_accents(), 1))
            .transpose()?;
        for group_index in order {
            let group = &self.groups[group_index];
            let color = if let Some(label) = &group.label {
                map.color(label)?
            } else {
                accent
                    .as_ref()
                    .ok_or_else(|| Error::Render("Scatter accent owner is missing".into()))?
                    .color(0)?
            };
            let style = chart.description().point_style(
                theme,
                color,
                theme.marks.get_points().get_radii().get_scatter(),
            );
            for point in &group.points {
                let (x, y) = projection.project(point.x, point.y);
                push_point(
                    &mut groups,
                    Point2D::new(x, y),
                    point.alpha.point(style.clone()),
                )?;
            }
        }
        Ok(ScatterPayload {
            clip: projection.panel_box(),
            groups,
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
    context: PlotRenderContext<'_, '_>,
    layout: crate::layout::Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    STAGES.with(|stages| {
        let mut counts = stages.get();
        counts[1] += 1;
        stages.set(counts);
    });
    let theme = context.theme;
    let prepared = context
        .prepared_family()
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Scatter(marks) => Some(marks),
            _ => None,
        })
        .ok_or_else(|| {
            Error::Render(
                "Scatter measurement requires root-prepared semantics for its component".into(),
            )
        })?;
    theme.text.prepared_font_environment()?;
    let data = context.data;
    let mut payload = prepared.project(
        chart,
        context
            .registered
            .expect("prepared Scatter occurrence")
            .categorical
            .as_ref()
            .expect("prepared color binding"),
        &layout.projection,
        theme,
        data,
    )?;
    let point_style = chart.description().point_style(
        theme,
        (0, 0, 0),
        theme.marks.get_points().get_radii().get_scatter(),
    );
    if let Some(ink) = point_ink(
        &point_style,
        prepared.groups.iter().flat_map(|group| {
            group
                .points
                .iter()
                .map(|point| (point.x, point.y, point.alpha))
        }),
    )? {
        let raw_windows = context
            .registered
            .expect("prepared occurrence")
            .raw_windows();
        payload.domain_expansions = crate::layout::interior::automatic_mark_ink_expansions(
            ink.ranges,
            &layout.projection,
            ink.bounds,
            [raw_windows[0].is_none(), raw_windows[1].is_none()],
            context.fit_scope(),
        )?;
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for (points, style) in &payload.groups {
        geometry.scatter(points, style, clip)?;
    }
    let mut result = PlotRenderResult::new(
        layout,
        crate::orchestrator::registration::guide_descriptor(context.registered)?,
    );
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Scatter(payload),
            geometry,
        },
    )?);
    Ok(result)
}
