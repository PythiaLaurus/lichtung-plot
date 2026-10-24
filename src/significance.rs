//! Significance bracket planning and rendering for categorical plots.

mod spec;

pub(crate) use spec::SignificanceAnnotation;
pub use spec::{Attached, FacetCell, Pairwise, Significance, SignificanceMark, SignificanceTarget};

use crate::Error;

use crate::layout::kernel::SpatialRounding;
use crate::layout::{DeferredAnnotationTextPlan, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{
    Backend, BracketOrientation, BracketStyle, PlannedSignificanceCall, SignificanceBracket,
};
use crate::theme::annotation::EffectiveSignificance;
use crate::theme::spacing::ExactLength;
use crate::Plot;
#[cfg(test)]
use crate::SignificanceMarkStyle;

#[derive(Debug)]
pub(crate) struct SignificancePlan {
    pub(crate) y_range: (f64, f64),
    pub(crate) brackets: Vec<PlannedSignificanceBracket>,
}

#[derive(Debug)]
pub(crate) struct PlannedSignificanceBracket {
    pub(crate) source_index: usize,
    pub(crate) geometry: SignificanceBracket,
    pub(crate) label: DeferredAnnotationTextPlan,
    pub(crate) style: BracketStyle,
}

#[derive(Debug)]
struct BracketMetrics {
    tick_height: ExactLength,
    label_gap: ExactLength,
    data_gap: ExactLength,
    level_step: ExactLength,
    panel_padding: ExactLength,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "non-default constraints remain only for direct significance planner regression tests"
    )
)]
enum LegendAnnotationConstraint {
    NoObstacle,
    TopCeiling(f64),
    UnsupportedInside,
}

#[derive(Debug)]
struct ResolvedComparison {
    source_index: usize,
    left: usize,
    right: usize,
    level: usize,
    label: DeferredAnnotationTextPlan,
}

pub(crate) fn plan(
    chart: &Plot,
    categories: &[String],
    rendered_maxima: &[f64],
    y_range: (f64, f64),
    panel: PanelBox,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    let effective = theme.effective_significance(chart.significance_mark_style_override())?;
    plan_comparisons_with_label(
        chart.significance_annotations(),
        categories,
        rendered_maxima,
        y_range,
        chart.user_x_limits(),
        chart.user_y_limits(),
        panel,
        LegendAnnotationConstraint::NoObstacle,
        &effective,
        theme,
        scope,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the wrapper preserves the existing validated significance planner inputs"
)]
#[cfg(test)]
fn plan_comparisons(
    annotations: &[SignificanceAnnotation],
    categories: &[String],
    rendered_maxima: &[f64],
    y_range: (f64, f64),
    categorical_x_override: Option<(f64, f64)>,
    y_limits: Option<(f64, f64)>,
    panel: PanelBox,
    legend_constraint: LegendAnnotationConstraint,
    mark_style: SignificanceMarkStyle,
) -> Result<SignificancePlan, Error> {
    let theme = crate::Theme::publication().resolve().unwrap();
    let effective = theme.effective_significance(Some(mark_style))?;
    plan_comparisons_with_label(
        annotations,
        categories,
        rendered_maxima,
        y_range,
        categorical_x_override,
        y_limits,
        panel,
        legend_constraint,
        &effective,
        &theme,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .map_err(crate::layout::fit::PlanError::into_error)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the wrapper preserves the existing validated significance planner inputs"
)]
fn plan_comparisons_with_label(
    annotations: &[SignificanceAnnotation],
    categories: &[String],
    rendered_maxima: &[f64],
    y_range: (f64, f64),
    categorical_x_override: Option<(f64, f64)>,
    y_limits: Option<(f64, f64)>,
    panel: PanelBox,
    legend_constraint: LegendAnnotationConstraint,
    effective: &EffectiveSignificance,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    plan_comparisons_at_with_label(
        annotations,
        categories,
        rendered_maxima,
        None,
        None,
        y_range,
        categorical_x_override,
        y_limits,
        panel,
        legend_constraint,
        false,
        effective,
        theme,
        scope,
    )
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "the private planner keeps validated anchors, ranges, and panel geometry together"
)]
fn plan_comparisons_at_with_label(
    annotations: &[SignificanceAnnotation],
    categories: &[String],
    rendered_maxima: &[f64],
    anchor_positions: Option<&[f64]>,
    anchor_targets: Option<&[SignificanceTarget]>,
    y_range: (f64, f64),
    categorical_x_override: Option<(f64, f64)>,
    y_limits: Option<(f64, f64)>,
    panel: PanelBox,
    legend_constraint: LegendAnnotationConstraint,
    horizontal: bool,
    effective: &EffectiveSignificance,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    validate_range(y_range, y_limits.is_some())?;
    if categories.len() != rendered_maxima.len()
        || anchor_positions.is_some_and(|positions| positions.len() != categories.len())
        || anchor_targets.is_some_and(|targets| targets.len() != categories.len())
        || categories.is_empty()
    {
        return Err(render_error(format!(
            "significance placement requires one rendered maximum per category ({} categories, {} maxima)",
            categories.len(),
            rendered_maxima.len()
        )).into());
    }
    if !panel.x0.is_finite()
        || !panel.x1.is_finite()
        || !panel.y0.is_finite()
        || !panel.y1.is_finite()
        || panel.width() <= 0.0
        || panel.height() <= 0.0
    {
        return Err(render_error(format!(
            "significance placement requires a finite positive plot panel; got {panel:?}"
        ))
        .into());
    }
    if matches!(legend_constraint, LegendAnnotationConstraint::TopCeiling(ceiling) if !ceiling.is_finite())
    {
        return Err(render_error(
            "significance placement received a non-finite annotation ceiling",
        )
        .into());
    }
    for (index, maximum) in rendered_maxima.iter().enumerate() {
        if !maximum.is_finite() {
            return Err(render_error(format!(
                "significance placement received a non-finite rendered maximum for category {:?} at index {index}",
                categories[index]
            )).into());
        }
    }
    for (index, category) in categories.iter().enumerate() {
        if categories[..index].iter().any(|seen| seen == category) {
            return Err(render_error(format!(
                "significance placement requires unique category labels; {category:?} appears more than once"
            )).into());
        }
    }

    let mut resolved = Vec::new();
    let mut comparison_pairs = Vec::new();
    let eligible = crate::data::source::collect_source(
        annotations
            .iter()
            .enumerate()
            .filter(|(_, annotation)| annotation.endpoints().is_some())
            .map(|(index, _)| Ok(index)),
    )?;
    for (source_index, annotation) in annotations.iter().enumerate() {
        let Some((endpoint_a, endpoint_b)) = annotation.endpoints() else {
            continue;
        };
        let (a, b) = if let Some(targets) = anchor_targets {
            let index = |endpoint: &SignificanceTarget| {
                targets
                    .iter()
                    .position(|target| target == endpoint)
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "significance target {endpoint:?} has no rendered anchor"
                        ))
                    })
            };
            if endpoint_a == endpoint_b {
                return Err(Error::Data(format!(
                    "significance comparison endpoints must be different; both were {:?}",
                    endpoint_a.category_name()
                ))
                .into());
            }
            (index(endpoint_a)?, index(endpoint_b)?)
        } else {
            validate_bracket(annotation, categories)?;
            let index = |endpoint: &SignificanceTarget| {
                categories
                    .iter()
                    .position(|category| category == endpoint.category_name())
                    .ok_or_else(|| {
                        render_error("validated significance category unexpectedly disappeared")
                    })
            };
            (index(endpoint_a)?, index(endpoint_b)?)
        };
        if anchor_positions
            .is_some_and(|positions| positions[a].to_bits() == positions[b].to_bits())
        {
            return Err(Error::Data(format!(
                "significance comparison targets {endpoint_a:?} and {endpoint_b:?} resolve to the same rendered anchor"
            )).into());
        }
        let pair = (a.min(b), a.max(b));
        if comparison_pairs.contains(&pair) {
            return Err(Error::Data(format!(
                "duplicate significance comparison for {:?} and {:?}; each unordered category pair may appear only once",
                categories[pair.0], categories[pair.1]
            )).into());
        }
        comparison_pairs.push(pair);
        let source = annotation.mark().glyph()?.to_owned();
        let text_plan =
            crate::layout::fit::measured(crate::layout::text::request::measure_occurrence(
                theme,
                &source,
                crate::text::TextAppearance {
                    theme: &effective.label,
                    exact_override: effective.exact_override,
                    layout: theme.text_layouts.significance.as_ref(),
                },
                i64::MAX,
                if horizontal {
                    PhysicalTextAlignment::Start
                } else {
                    PhysicalTextAlignment::Center
                },
                "significance bracket label",
                scope,
                (source_index, &eligible),
            ))?;
        resolved.push(ResolvedComparison {
            source_index,
            left: pair.0,
            right: pair.1,
            level: 0,
            label: DeferredAnnotationTextPlan::from_resolved(text_plan),
        });
    }

    if resolved.is_empty() {
        let planned_range = match (legend_constraint, y_limits) {
            (LegendAnnotationConstraint::TopCeiling(ceiling), None) => fit_data_below_ceiling(
                y_range,
                panel,
                rendered_maxima,
                ceiling,
                &bracket_metrics(effective)?,
            )?,
            _ => y_range,
        };
        return Ok(SignificancePlan {
            y_range: planned_range,
            brackets: Vec::new(),
        });
    }

    let metrics = bracket_metrics(effective)?;
    if categorical_x_override.is_some() {
        return Err(render_error(
            "categorical x_limits cannot be used with visible significance comparisons because bracket endpoints would not align; remove x_limits or remove the comparisons",
        ).into());
    }
    let annotation_ceiling = match legend_constraint {
        LegendAnnotationConstraint::NoObstacle => None,
        LegendAnnotationConstraint::TopCeiling(ceiling) => Some(ceiling),
        LegendAnnotationConstraint::UnsupportedInside => {
            return Err(render_error(
                "significance brackets cannot safely avoid a centered or bottom inside legend; use a top-aligned, outside, or hidden legend",
            ).into());
        }
    };
    let global_maximum = rendered_maxima
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    let mut labels = crate::data::source::collect_source(
        resolved
            .iter()
            .map(|comparison| Ok(comparison.label.plan.clone())),
    )?;
    crate::layout::text::font_fit::uniform(theme, &mut labels, scope)?;
    for (comparison, label) in resolved.iter_mut().zip(labels) {
        comparison.label = DeferredAnnotationTextPlan::from_resolved(label);
    }
    resolved.sort_by(|left, right| {
        left.left
            .cmp(&right.left)
            .then_with(|| left.right.cmp(&right.right))
            .then_with(|| left.label.plan.source.cmp(&right.label.plan.source))
    });
    // A label can overhang its bracket span. Occupancy therefore follows the
    // complete centered label and bracket ink, not just category indices.
    let mut occupied_levels = Vec::<Vec<(f64, f64)>>::new();

    for comparison in &mut resolved {
        let anchor = |index: usize| {
            panel.x0
                + anchor_positions.map_or_else(
                    || (index as f64 + 0.5) / categories.len() as f64,
                    |positions| positions[index],
                ) * panel.width()
        };
        let start = anchor(comparison.left).min(anchor(comparison.right));
        let end = anchor(comparison.left).max(anchor(comparison.right));
        let center = f64::midpoint(start, end);
        let footprint = label_footprint(&comparison.label.plan, horizontal)?;
        let (lower, upper) = if horizontal {
            (footprint.y0, footprint.y1)
        } else {
            (footprint.x0, footprint.x1)
        };
        let clearance = f64::from(
            theme
                .annotations
                .get_significance()
                .get_bracket_stroke()
                .get_width(),
        ) / 2.0
            + 1.0 / 1024.0;
        let interval = (
            start.min(center + lower) - clearance,
            end.max(center + upper) + clearance,
        );
        let mut first_free = None;
        for (level, occupied) in occupied_levels.iter().enumerate() {
            let mut free = true;
            for previous in occupied {
                if interval.0 < previous.1 && previous.0 < interval.1 {
                    free = false;
                    break;
                }
            }
            if free {
                first_free = Some(level);
                break;
            }
        }
        let level = first_free.unwrap_or(occupied_levels.len());
        if level == occupied_levels.len() {
            occupied_levels.push(Vec::new());
        }
        occupied_levels[level].push(interval);
        comparison.level = level;
    }

    let mut level_maxima = vec![f64::NEG_INFINITY; occupied_levels.len()];
    let mut level_label_extents = vec![0.0_f64; occupied_levels.len()];
    for comparison in &resolved {
        let span_maximum = rendered_maxima[comparison.left..=comparison.right]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        level_maxima[comparison.level] = level_maxima[comparison.level].max(span_maximum);
        let footprint = label_footprint(&comparison.label.plan, horizontal)?;
        let label_extent = if horizontal {
            footprint.x1.max(comparison.label.plan.occupied_size().0)
        } else {
            footprint.y1.max(comparison.label.plan.occupied_size().1)
        };
        level_label_extents[comparison.level] =
            level_label_extents[comparison.level].max(label_extent);
    }

    let available_height =
        annotation_capacity(panel, annotation_ceiling)?.pixels_rounded(SpatialRounding::Floor)?;
    let mut minimum_height = metrics.data_gap.add_length(&metrics.tick_height);
    for extent in level_label_extents
        .iter()
        .take(level_label_extents.len().saturating_sub(1))
    {
        minimum_height = minimum_height.add_length(&significance_level_spacing(*extent, &metrics)?);
    }
    let minimum_height = minimum_height
        .add_pixels(level_label_extents.last().copied().unwrap_or_default())?
        .add_length(&metrics.label_gap)
        .add_length(&metrics.panel_padding.times(3))
        .pixels_rounded(SpatialRounding::Ceil)?;
    if available_height <= minimum_height {
        return Err(scope.failure(crate::layout::fit::FitKind::TrackOrResidual, format!(
            "plot panel space below the inside legend is too short for {} significance level(s): {:.1}px available, more than {:.1}px required; increase chart height, move/hide the legend, or reduce comparisons",
            level_maxima.len(),
            available_height,
            minimum_height
        )).into());
    }

    let (planned_range, line_positions) = if let Some(limits) = y_limits {
        let Some(lines) = place_lines(
            limits,
            panel,
            &level_maxima,
            &level_label_extents,
            annotation_ceiling,
            global_maximum,
            &metrics,
        )?
        else {
            let suggested = expand_upper_limit(
                limits,
                panel,
                &level_maxima,
                &level_label_extents,
                annotation_ceiling,
                global_maximum,
                &metrics,
            )?
            .0;
            let remedy = if annotation_ceiling.is_some() {
                "increase the upper limit, move/hide the legend, or use automatic y limits"
            } else {
                "increase the upper limit or use automatic y limits"
            };
            return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, format!(
                "explicit y_limits ({}, {}) leave insufficient room for significance brackets; y_max must be at least {}; {remedy}",
                limits.0, limits.1, suggested.1
            )).into());
        };
        (limits, lines)
    } else if let Some(lines) = place_lines(
        y_range,
        panel,
        &level_maxima,
        &level_label_extents,
        annotation_ceiling,
        global_maximum,
        &metrics,
    )? {
        (y_range, lines)
    } else {
        expand_upper_limit(
            y_range,
            panel,
            &level_maxima,
            &level_label_extents,
            annotation_ceiling,
            global_maximum,
            &metrics,
        )?
    };

    resolved.sort_by(|left, right| {
        left.level
            .cmp(&right.level)
            .then_with(|| left.left.cmp(&right.left))
            .then_with(|| left.right.cmp(&right.right))
            .then_with(|| left.label.plan.source.cmp(&right.label.plan.source))
    });
    let category_width = panel.width() / categories.len() as f64;
    let mut brackets = crate::layout::fit::collect(
        "significance brackets",
        resolved.into_iter().map(|comparison| {
            let anchor = |index: usize| {
                anchor_positions.map_or_else(
                    || panel.x0 + (index as f64 + 0.5) * category_width,
                    |positions| panel.x0 + positions[index] * panel.width(),
                )
            };
            let x_left = anchor(comparison.left);
            let x_right = anchor(comparison.right);
            Ok(PlannedSignificanceBracket {
                source_index: comparison.source_index,
                geometry: SignificanceBracket {
                    x_left,
                    x_right,
                    y: line_positions[comparison.level],
                    label: comparison.label.plan.source.clone(),
                    orientation: BracketOrientation::Vertical,
                },
                style: effective.bracket_style(&comparison.label.plan)?,
                label: comparison.label,
            })
        }),
    )?;

    let mut labels = Vec::new();
    crate::data::source::reserve_source(&mut labels, brackets.len())?;
    let containing = if horizontal {
        PanelBox::new(panel.y0, panel.y1, panel.x0, panel.x1)
    } else {
        panel
    };
    for bracket in &mut brackets {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let mut painted = bracket.geometry.clone();
        if horizontal {
            painted.orientation = BracketOrientation::Horizontal;
        }
        let canonical = crate::render::backend::validate_bracket(&painted, &bracket.style)?;
        let placement =
            crate::render::backend::planned_bracket_placement(canonical, &bracket.label.plan)?;
        let label = crate::layout::text::placement::contain(
            bracket.label.plan.clone(),
            placement,
            containing,
            &scope.child(crate::layout::AddressRole::Annotation, bracket.source_index)?,
        )?;
        labels.push(crate::render::backend::PlacedText {
            plan: label,
            placement,
        });
        let path_only =
            DeferredAnnotationTextPlan::from_resolved(bracket.label.plan.clone().hide()?);
        geometry.bracket(PlannedSignificanceCall {
            bracket: &painted,
            style: &bracket.style,
            label: &path_only,
        })?;
        if geometry.bounds().iter().any(|ink| {
            ink.left < containing.x0
                || ink.right > containing.x1
                || ink.bottom < containing.y0
                || ink.top > containing.y1
        }) {
            return Err(scope
                .child(crate::layout::AddressRole::Annotation, bracket.source_index)?
                .failure(
                    crate::layout::fit::FitKind::OverlayCollision,
                    "significance bracket path exceeds its assigned panel",
                )
                .into());
        }
    }
    let indices = crate::data::source::collect_source(
        brackets.iter().map(|bracket| Ok(bracket.source_index)),
    )?;
    crate::layout::text::visibility::apply(&mut labels, &indices, containing, horizontal, scope)?;
    crate::layout::text::visibility::require_nonoverlap(&mut labels, scope)?;
    for (bracket, label) in brackets.iter_mut().zip(labels) {
        bracket.label = DeferredAnnotationTextPlan::from_resolved(label.plan);
    }

    Ok(SignificancePlan {
        y_range: planned_range,
        brackets,
    })
}

fn label_footprint(
    plan: &crate::layout::ResolvedTextPlan,
    horizontal: bool,
) -> Result<PanelBox, Error> {
    use crate::render::backend::{PlannedTextPlacement, Point2D, TextAnchor};
    let default = if horizontal {
        PlannedTextPlacement::middle(Point2D::new(0.0, 0.0), TextAnchor::Start, 0.0)
    } else {
        PlannedTextPlacement::baseline(Point2D::new(0.0, 0.0), TextAnchor::Middle, 0.0)
    };
    let placement = crate::layout::text::placement::resolve(default, plan, None)?;
    crate::layout::text::placement::footprint(placement, plan)
}

/// Plan exact grouped dodge anchors for a vertical categorical plot.
#[expect(
    clippy::too_many_arguments,
    reason = "grouped anchors require explicit data, geometry, and legend constraints"
)]
pub(crate) fn plan_grouped(
    chart: &Plot,
    categories: &[String],
    groups: &[String],
    rendered_maxima: &[Vec<f64>],
    group_width: f64,
    y_range: (f64, f64),
    panel: PanelBox,
    stacked: bool,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    plan_grouped_oriented(
        chart,
        categories,
        groups,
        rendered_maxima,
        group_width,
        y_range,
        panel,
        LegendAnnotationConstraint::NoObstacle,
        stacked,
        false,
        theme,
        scope,
    )
}

/// Plan transposed brackets to the right of horizontal categorical marks.
#[expect(
    clippy::too_many_arguments,
    reason = "horizontal anchors require explicit data, geometry, and stacking policy"
)]
pub(crate) fn plan_horizontal(
    chart: &Plot,
    categories: &[String],
    groups: &[String],
    rendered_maxima: &[Vec<f64>],
    group_width: f64,
    x_range: (f64, f64),
    panel: PanelBox,
    stacked: bool,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    plan_grouped_oriented(
        chart,
        categories,
        groups,
        rendered_maxima,
        group_width,
        x_range,
        PanelBox::new(panel.y0, panel.y1, panel.x0, panel.x1),
        LegendAnnotationConstraint::NoObstacle,
        stacked,
        true,
        theme,
        scope,
    )
    .map_err(|error| {
        let orient = |message: String| {
            message
                .replace("y_limits", "x_limits")
                .replace("y_max", "x_max")
                .replace("automatic y", "automatic x")
                .replace(" y range", " x range")
                .replace("categorical x_limits", "categorical y_limits")
                .replace("top-aligned", "right-aligned")
        };
        match error {
            crate::layout::fit::PlanError::Fatal(Error::Render(message)) => {
                Error::Render(orient(message)).into()
            }
            crate::layout::fit::PlanError::Fit(mut failure) => {
                failure.message = orient(failure.message);
                failure.into()
            }
            other @ crate::layout::fit::PlanError::Fatal(_) => other,
        }
    })
}

#[expect(
    clippy::too_many_arguments,
    clippy::cast_precision_loss,
    reason = "grouped vertical and horizontal planners share one exact-anchor implementation"
)]
fn plan_grouped_oriented(
    chart: &Plot,
    categories: &[String],
    groups: &[String],
    rendered_maxima: &[Vec<f64>],
    group_width: f64,
    numeric_range: (f64, f64),
    panel: PanelBox,
    legend_constraint: LegendAnnotationConstraint,
    stacked: bool,
    horizontal: bool,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    if categories.is_empty()
        || rendered_maxima.len() != categories.len()
        || rendered_maxima
            .iter()
            .any(|maxima| maxima.len() != groups.len().max(1))
    {
        return Err(render_error(
            "grouped significance placement requires one maximum per category and series anchor",
        )
        .into());
    }
    let group_count = groups.len().max(1);
    let mut labels = Vec::new();
    let mut targets = Vec::new();
    let mut maxima = Vec::new();
    let mut positions = Vec::new();
    for (category_index, category) in categories.iter().enumerate() {
        labels.push(format!("{category_index}-cluster"));
        targets.push(SignificanceTarget::category(category));
        maxima.push(
            rendered_maxima[category_index]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
        );
        positions.push((category_index as f64 + 0.5) / categories.len() as f64);
        for (group_index, group) in groups.iter().enumerate() {
            if !rendered_maxima[category_index][group_index].is_finite() {
                continue;
            }
            labels.push(format!("{category_index}-series-{group_index}"));
            targets.push(SignificanceTarget::category_series(category, group));
            maxima.push(rendered_maxima[category_index][group_index]);
            let center = category_index as f64
                + 0.5
                + ((group_index as f64 + 0.5) / group_count as f64 - 0.5) * group_width;
            positions.push(center / categories.len() as f64);
        }
    }
    let mut order = (0..labels.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| positions[*left].total_cmp(&positions[*right]));
    let labels = order
        .iter()
        .map(|index| labels[*index].clone())
        .collect::<Vec<_>>();
    let targets = order
        .iter()
        .map(|index| targets[*index].clone())
        .collect::<Vec<_>>();
    let maxima = order.iter().map(|index| maxima[*index]).collect::<Vec<_>>();
    let positions = order
        .iter()
        .map(|index| positions[*index])
        .collect::<Vec<_>>();
    for annotation in chart.significance_annotations() {
        if let Some((endpoint_a, endpoint_b)) = annotation.endpoints() {
            if stacked && (endpoint_a.series_name().is_some() || endpoint_b.series_name().is_some())
            {
                return Err(Error::Data(
                    "series-qualified significance targets are ambiguous for stacked bars".into(),
                )
                .into());
            }
            resolve_grouped_endpoint(endpoint_a, categories, groups, rendered_maxima)?;
            resolve_grouped_endpoint(endpoint_b, categories, groups, rendered_maxima)?;
        }
    }
    let effective = theme.effective_significance(chart.significance_mark_style_override())?;
    let mut result = plan_comparisons_at_with_label(
        chart.significance_annotations(),
        &labels,
        &maxima,
        Some(&positions),
        Some(&targets),
        numeric_range,
        if horizontal {
            chart.user_y_limits()
        } else {
            chart.user_x_limits()
        },
        if horizontal {
            chart.user_x_limits()
        } else {
            chart.user_y_limits()
        },
        panel,
        legend_constraint,
        horizontal,
        &effective,
        theme,
        scope,
    )?;
    if horizontal {
        for bracket in &mut result.brackets {
            bracket.geometry.orientation = BracketOrientation::Horizontal;
        }
    }
    Ok(result)
}

fn resolve_grouped_endpoint(
    endpoint: &SignificanceTarget,
    categories: &[String],
    groups: &[String],
    rendered_maxima: &[Vec<f64>],
) -> Result<(), Error> {
    let category_index = categories
        .iter()
        .position(|category| category == endpoint.category_name())
        .ok_or_else(|| {
            Error::Data(format!(
                "significance comparison references unknown category {:?}; available categories: {}",
                endpoint.category_name(),
                categories.join(", ")
            ))
        })?;
    let group_index = if let Some(group) = endpoint.series_name() {
        let index = groups
            .iter()
            .position(|candidate| candidate == group)
            .ok_or_else(|| {
                Error::Data(format!(
                "significance comparison references unknown series {group:?}; available series: {}",
                groups.join(", ")
            ))
            })?;
        Some(index)
    } else {
        None
    };
    if let Some(index) = group_index {
        if !rendered_maxima[category_index][index].is_finite() {
            return Err(Error::Data(format!(
                "significance target category {:?}, series {:?} has no rendered mark",
                endpoint.category_name(),
                groups[index]
            )));
        }
        Ok(())
    } else {
        Ok(())
    }
}

fn validate_range(range: (f64, f64), explicit: bool) -> Result<(), Error> {
    if range.0.is_finite() && range.1.is_finite() && range.0 < range.1 {
        return Ok(());
    }
    let source = if explicit {
        "explicit y_limits"
    } else {
        "y range"
    };
    Err(render_error(format!(
        "significance placement requires finite ascending {source}; got ({}, {})",
        range.0, range.1
    )))
}

fn validate_bracket(
    annotation: &SignificanceAnnotation,
    categories: &[String],
) -> Result<(), Error> {
    let Some((endpoint_a, endpoint_b)) = annotation.endpoints() else {
        return Ok(());
    };
    if endpoint_a == endpoint_b {
        return Err(Error::Data(format!(
            "significance comparison endpoints must be different; both were {:?}",
            endpoint_a.category_name()
        )));
    }
    if endpoint_a.series_name().is_some() || endpoint_b.series_name().is_some() {
        return Err(Error::Data(
            "series-qualified significance targets require rendered series anchors".into(),
        ));
    }
    for target in [endpoint_a, endpoint_b] {
        if !categories
            .iter()
            .any(|category| category == target.category_name())
        {
            return Err(Error::Data(format!(
                "significance comparison references unknown category {:?}; available categories: {}",
                target.category_name(),
                categories.join(", ")
            )));
        }
    }
    annotation.mark().glyph()?;
    Ok(())
}

fn place_lines(
    range: (f64, f64),
    panel: PanelBox,
    level_maxima: &[f64],
    level_label_extents: &[f64],
    annotation_ceiling: Option<f64>,
    global_maximum: f64,
    metrics: &BracketMetrics,
) -> Result<Option<Vec<f64>>, Error> {
    if let Some(ceiling) = annotation_ceiling {
        if !data_fits_below_ceiling(range, panel, global_maximum, ceiling, metrics)? {
            return Ok(None);
        }
    }
    if level_maxima.len() != level_label_extents.len() {
        return Err(render_error("significance level/label counts differ"));
    }
    let capacity =
        annotation_capacity(panel, annotation_ceiling)?.pixels_rounded(SpatialRounding::Floor)?;
    let mut lines = Vec::<ExactLength>::with_capacity(level_maxima.len());
    for (level, maximum) in level_maxima.iter().enumerate() {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::SignificanceLevelCheck,
        );
        let offset = *maximum - range.0;
        let span = range.1 - range.0;
        if !offset.is_finite() || !span.is_finite() || span <= 0.0 {
            return Err(render_error(
                "significance projection arithmetic is not finite",
            ));
        }
        let fraction = offset / span;
        if !fraction.is_finite() {
            return Err(render_error("significance projection ratio is not finite"));
        }
        let fraction = fraction.clamp(0.0, 1.0);
        let mut line = metrics
            .tick_height
            .add_length(&metrics.data_gap)
            .add_pixels(fraction * panel.height())?;
        if let Some(previous) = lines.last() {
            let next = previous.add_length(&significance_level_spacing(
                level_label_extents[level - 1],
                metrics,
            )?);
            if line.compare(&next).is_lt() {
                line = next;
            }
        }
        let required = line
            .add_length(&metrics.label_gap)
            .add_pixels(level_label_extents[level])?
            .add_length(&metrics.panel_padding)
            .pixels_rounded(SpatialRounding::Ceil)?;
        if required > capacity {
            return Ok(None);
        }
        lines.push(line);
    }
    // Freeze whole positions only after exact level packing and hard fit. The
    // backend still receives its independently selected canonical tick/gap.
    lines
        .iter()
        .map(|line| line.after_pixels(panel.y0))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn significance_level_spacing(
    label_extent: f64,
    metrics: &BracketMetrics,
) -> Result<ExactLength, Error> {
    let occupancy = metrics
        .label_gap
        .add_pixels(label_extent)?
        .add_length(&metrics.tick_height);
    Ok(if metrics.level_step.compare(&occupancy).is_lt() {
        occupancy
    } else {
        metrics.level_step.clone()
    })
}

fn annotation_capacity(panel: PanelBox, ceiling: Option<f64>) -> Result<ExactLength, Error> {
    ExactLength::pixels((ceiling.unwrap_or(panel.y1).min(panel.y1) - panel.y0).max(0.0))
}

fn expand_upper_limit(
    range: (f64, f64),
    panel: PanelBox,
    level_maxima: &[f64],
    level_label_extents: &[f64],
    annotation_ceiling: Option<f64>,
    global_maximum: f64,
    metrics: &BracketMetrics,
) -> Result<((f64, f64), Vec<f64>), Error> {
    let lower = range.0;
    let mut failing_upper = range.1;
    let mut passing_upper = range.1;
    let mut span = (range.1 - lower).max(f64::EPSILON);
    let passing_lines = loop {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::SignificanceRangeGrowth,
        );
        passing_upper = passing_upper
            .max(
                level_maxima
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max),
            )
            .max(global_maximum)
            + span;
        if !passing_upper.is_finite() {
            return Err(render_error(
                "could not find a finite automatic y range for significance brackets; check data magnitudes and chart height",
            ));
        }
        if let Some(lines) = place_lines(
            (lower, passing_upper),
            panel,
            level_maxima,
            level_label_extents,
            annotation_ceiling,
            global_maximum,
            metrics,
        )? {
            break lines;
        }
        failing_upper = passing_upper;
        span *= 2.0;
    };

    let mut lines = passing_lines;
    for _ in 0..64 {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::SignificanceBisection,
        );
        let midpoint = f64::midpoint(failing_upper, passing_upper);
        if let Some(candidate) = place_lines(
            (lower, midpoint),
            panel,
            level_maxima,
            level_label_extents,
            annotation_ceiling,
            global_maximum,
            metrics,
        )? {
            passing_upper = midpoint;
            lines = candidate;
        } else {
            failing_upper = midpoint;
        }
    }
    Ok(((lower, passing_upper), lines))
}

fn fit_data_below_ceiling(
    range: (f64, f64),
    panel: PanelBox,
    rendered_maxima: &[f64],
    ceiling: f64,
    metrics: &BracketMetrics,
) -> Result<(f64, f64), Error> {
    let maximum = rendered_maxima
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let fits =
        |upper: f64| data_fits_below_ceiling((range.0, upper), panel, maximum, ceiling, metrics);
    if fits(range.1)? {
        return Ok(range);
    }
    let available =
        annotation_capacity(panel, Some(ceiling))?.pixels_rounded(SpatialRounding::Floor)?;
    let required = metrics
        .data_gap
        .add_length(&metrics.panel_padding)
        .pixels_rounded(SpatialRounding::Ceil)?;
    if available <= required {
        return Err(render_error(
            "inside legend leaves no vertical room for categorical data; increase chart height or move/hide the legend",
        ));
    }

    let mut failing_upper = range.1;
    let mut passing_upper = range.1;
    let mut span = range.1 - range.0;
    loop {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::SignificanceRangeGrowth,
        );
        passing_upper = passing_upper.max(maximum) + span;
        if !passing_upper.is_finite() {
            return Err(render_error(
                "could not find a finite automatic y range below the inside legend; check data magnitudes and chart height",
            ));
        }
        if fits(passing_upper)? {
            break;
        }
        failing_upper = passing_upper;
        span *= 2.0;
    }
    for _ in 0..64 {
        let midpoint = f64::midpoint(failing_upper, passing_upper);
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::SignificanceBisection,
        );
        if fits(midpoint)? {
            passing_upper = midpoint;
        } else {
            failing_upper = midpoint;
        }
    }
    Ok((range.0, passing_upper))
}

fn data_fits_below_ceiling(
    range: (f64, f64),
    panel: PanelBox,
    maximum: f64,
    ceiling: f64,
    metrics: &BracketMetrics,
) -> Result<bool, Error> {
    let fraction = ((maximum - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
    let required = metrics
        .data_gap
        .add_length(&metrics.panel_padding)
        .add_pixels(fraction * panel.height())?
        .pixels_rounded(SpatialRounding::Ceil)?;
    let available =
        annotation_capacity(panel, Some(ceiling))?.pixels_rounded(SpatialRounding::Floor)?;
    Ok(required <= available)
}

fn bracket_metrics(effective: &EffectiveSignificance) -> Result<BracketMetrics, Error> {
    Ok(BracketMetrics {
        tick_height: effective.tick_height()?,
        label_gap: effective.gap.clone(),
        data_gap: effective.data_gap()?,
        level_step: effective.level_step()?,
        panel_padding: effective.panel_padding()?,
    })
}

pub(crate) fn render_selected(
    result: &crate::spec::PlotRenderResult,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let Some(plan) = &result.significance else {
        return Ok(());
    };
    for bracket in &plan.brackets {
        backend.draw_planned_bracket(PlannedSignificanceCall {
            bracket: &bracket.geometry,
            style: &bracket.style,
            label: &bracket.label,
        })?;
    }
    Ok(())
}

fn render_error(message: impl Into<String>) -> Error {
    Error::Render(message.into())
}

#[cfg(test)]
mod tests;
