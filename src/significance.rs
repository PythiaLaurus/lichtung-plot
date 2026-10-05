//! Significance bracket planning and rendering for categorical plots.

mod spec;

pub(crate) use spec::SignificanceAnnotation;
pub use spec::{Attached, FacetCell, Pairwise, Significance, SignificanceMark, SignificanceTarget};

use crate::Error;

use crate::layout::{DeferredAnnotationTextPlan, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{
    Backend, BracketOrientation, BracketStyle, PlannedSignificanceCall, SignificanceBracket,
    SIGNIFICANCE_BRACKET_GEOMETRY,
};
use crate::theme::TextAppearance;
use crate::{Plot, SignificanceMarkStyle};

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
}

#[derive(Debug, Clone, Copy)]
struct BracketMetrics {
    tick_height: f64,
    label_gap: f64,
    data_gap: f64,
    level_step: f64,
    panel_padding: f64,
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
    let label = theme.typography.effective_significance_label(
        chart.significance_mark_style_override(),
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    plan_comparisons_with_label(
        chart.significance_annotations(),
        categories,
        rendered_maxima,
        y_range,
        chart.user_x_limits(),
        chart.user_y_limits(),
        panel,
        LegendAnnotationConstraint::NoObstacle,
        chart.significance_mark_style_spec(),
        &label,
        theme,
        chart.significance_mark_style_override().is_some(),
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
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let label = theme.typography.effective_significance_label(
        Some(mark_style),
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    plan_comparisons_with_label(
        annotations,
        categories,
        rendered_maxima,
        y_range,
        categorical_x_override,
        y_limits,
        panel,
        legend_constraint,
        mark_style,
        &label,
        &theme,
        false,
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
    mark_style: SignificanceMarkStyle,
    label: &TextAppearance,
    theme: &crate::theme::ResolvedTheme,
    exact_override: bool,
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
        mark_style,
        label,
        theme,
        exact_override,
        scope,
    )
}

#[cfg(test)]
#[expect(
    clippy::too_many_arguments,
    reason = "direct planner tests preserve exact anchor and orientation inputs"
)]
fn plan_comparisons_at(
    annotations: &[SignificanceAnnotation],
    categories: &[String],
    rendered_maxima: &[f64],
    anchor_positions: Option<&[f64]>,
    y_range: (f64, f64),
    categorical_x_override: Option<(f64, f64)>,
    y_limits: Option<(f64, f64)>,
    panel: PanelBox,
    legend_constraint: LegendAnnotationConstraint,
    horizontal: bool,
    mark_style: SignificanceMarkStyle,
) -> Result<SignificancePlan, Error> {
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let label = theme.typography.effective_significance_label(
        Some(mark_style),
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
    plan_comparisons_at_with_label(
        annotations,
        categories,
        rendered_maxima,
        anchor_positions,
        None,
        y_range,
        categorical_x_override,
        y_limits,
        panel,
        legend_constraint,
        horizontal,
        mark_style,
        &label,
        &theme,
        false,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .map_err(crate::layout::fit::PlanError::into_error)
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
    mark_style: SignificanceMarkStyle,
    label: &TextAppearance,
    theme: &crate::theme::ResolvedTheme,
    exact_override: bool,
    scope: &crate::layout::fit::FitScope,
) -> Result<SignificancePlan, crate::layout::fit::PlanError> {
    let mut metrics = bracket_metrics(mark_style);
    metrics.panel_padding = metrics
        .panel_padding
        .max(f64::from(theme.annotations.bracket_stroke.width) / 2.0 + 1.0 / 1024.0);
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
        let text_plan = theme.measure_text(
            &source,
            label,
            if horizontal {
                PhysicalTextAlignment::Start
            } else {
                PhysicalTextAlignment::Center
            },
            "significance bracket label",
            exact_override,
        )?;
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
            (LegendAnnotationConstraint::TopCeiling(ceiling), None) => {
                fit_data_below_ceiling(y_range, panel, rendered_maxima, ceiling, metrics)?
            }
            _ => y_range,
        };
        return Ok(SignificancePlan {
            y_range: planned_range,
            brackets: Vec::new(),
        });
    }

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

    resolved.sort_by(|left, right| {
        left.left
            .cmp(&right.left)
            .then_with(|| left.right.cmp(&right.right))
            .then_with(|| left.label.plan.source.cmp(&right.label.plan.source))
    });
    // A label can overhang its bracket span. Occupancy therefore follows the
    // complete centered label and bracket ink, not just category indices.
    let mut occupied_levels = Vec::<Vec<(f64, f64)>>::new();
    let resources = theme.prepared_font_environment()?.generation_resources();
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
        let label_span = if horizontal {
            comparison.label.plan.height()
        } else {
            comparison.label.plan.width()
        };
        let clearance = f64::from(theme.annotations.bracket_stroke.width) / 2.0 + 1.0 / 1024.0;
        let interval = (
            start.min(center - label_span / 2.0) - clearance,
            end.max(center + label_span / 2.0) + clearance,
        );
        let mut first_free = None;
        for (level, occupied) in occupied_levels.iter().enumerate() {
            resources.claim_visits(1).map_err(|error| {
                Error::Render(format!("significance shelf level admission: {error:?}"))
            })?;
            let mut free = true;
            for previous in occupied {
                resources.claim_visits(1).map_err(|error| {
                    Error::Render(format!("significance shelf collision admission: {error:?}"))
                })?;
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
        let label_extent = if horizontal {
            comparison.label.plan.width()
        } else {
            comparison.label.plan.height()
        };
        level_label_extents[comparison.level] =
            level_label_extents[comparison.level].max(label_extent);
    }

    let annotation_top = annotation_ceiling.unwrap_or(panel.y1) - metrics.panel_padding;
    let annotation_top = annotation_top.min(panel.y1 - metrics.panel_padding);
    let available_height = annotation_top - panel.y0;
    let minimum_height = metrics.data_gap
        + metrics.tick_height
        + level_label_extents
            .iter()
            .take(level_label_extents.len().saturating_sub(1))
            .map(|extent| significance_level_spacing(*extent, metrics))
            .sum::<f64>()
        + level_label_extents.last().copied().unwrap_or_default()
        + metrics.label_gap
        + 2.0 * metrics.panel_padding;
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
            metrics,
        )?
        else {
            let suggested = expand_upper_limit(
                limits,
                panel,
                &level_maxima,
                &level_label_extents,
                annotation_ceiling,
                global_maximum,
                metrics,
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
        metrics,
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
            metrics,
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
    let brackets = crate::layout::fit::collect(theme, "significance brackets", resolved
        .into_iter()
        .map(|comparison| {
            let anchor = |index: usize| {
                anchor_positions.map_or_else(
                    || panel.x0 + (index as f64 + 0.5) * category_width,
                    |positions| panel.x0 + positions[index] * panel.width(),
                )
            };
            let x_left = anchor(comparison.left);
            let x_right = anchor(comparison.right);
            let label_center = f64::midpoint(x_left, x_right);
            let label_half_extent = if horizontal {
                comparison.label.plan.height() / 2.0
            } else {
                comparison.label.plan.width() / 2.0
            };
            if label_center - label_half_extent < panel.x0
                || label_center + label_half_extent > panel.x1
            {
                return Err(scope.child(crate::layout::AddressRole::Annotation, comparison.source_index, &theme.prepared_font_environment()?.generation_resources())?.failure(crate::layout::fit::FitKind::Text,
                    "significance bracket label does not fit inside the plot panel; enlarge the chart or reduce mark typography",
                ).into());
            }
            Ok(PlannedSignificanceBracket {
                source_index: comparison.source_index,
                geometry: SignificanceBracket {
                    x_left,
                    x_right,
                    y: line_positions[comparison.level],
                    label: comparison.label.plan.source.clone(),
                    orientation: BracketOrientation::Vertical,
                },
                label: comparison.label,
            })
        })
        )?;

    for bracket in &brackets {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let label = &bracket.label.plan;
        let style = BracketStyle {
            color: theme.annotations.bracket_stroke.paint,
            width: theme.annotations.bracket_stroke.width,
            label_color: label.style.paint,
            font_family: label
                .copy_stack()
                .map_err(|error| Error::Render(error.to_string()))?,
            font_size: label.style.font_size,
            line_height: label.line_height(),
            weight: label.style.weight,
            label_gap: metrics.label_gap,
        };
        let mut painted = bracket.geometry.clone();
        let containing = if horizontal {
            painted.orientation = BracketOrientation::Horizontal;
            PanelBox::new(panel.y0, panel.y1, panel.x0, panel.x1)
        } else {
            panel
        };
        geometry.bracket(PlannedSignificanceCall {
            bracket: &painted,
            style: &style,
            label: &bracket.label,
        })?;
        if geometry.bounds().iter().any(|ink| {
            ink.left < containing.x0
                || ink.right > containing.x1
                || ink.bottom < containing.y0
                || ink.top > containing.y1
        }) {
            return Err(scope
                .child(
                    crate::layout::AddressRole::Annotation,
                    bracket.source_index,
                    &theme.prepared_font_environment()?.generation_resources(),
                )?
                .failure(
                    crate::layout::fit::FitKind::OverlayCollision,
                    "complete significance bracket and label ink exceeds its assigned panel",
                )
                .into());
        }
    }

    Ok(SignificancePlan {
        y_range: planned_range,
        brackets,
    })
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
    clippy::too_many_lines,
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
    let label = theme.typography.effective_significance_label(
        chart.significance_mark_style_override(),
        &theme.prepared_font_environment()?.generation_resources(),
    )?;
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
        chart.significance_mark_style_spec(),
        &label,
        theme,
        chart.significance_mark_style_override().is_some(),
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
    metrics: BracketMetrics,
) -> Result<Option<Vec<f64>>, Error> {
    if annotation_ceiling.is_some_and(|ceiling| {
        !data_fits_below_ceiling(range, panel, global_maximum, ceiling, metrics)
    }) {
        return Ok(None);
    }
    if level_maxima.len() != level_label_extents.len() {
        return Err(render_error("significance level/label counts differ"));
    }
    let annotation_edge =
        annotation_ceiling.unwrap_or(panel.y1).min(panel.y1) - metrics.panel_padding;
    let mut lines = Vec::with_capacity(level_maxima.len());
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
        let data_y = panel.y0 + fraction * panel.height();
        let clearance_y = data_y + metrics.tick_height + metrics.data_gap;
        let line = lines.last().map_or(clearance_y, |previous| {
            clearance_y
                .max(previous + significance_level_spacing(level_label_extents[level - 1], metrics))
        });
        let label_edge = line + metrics.label_gap + level_label_extents[level];
        if !line.is_finite() || !label_edge.is_finite() || !annotation_edge.is_finite() {
            return Err(render_error(
                "significance placement arithmetic is not finite",
            ));
        }
        if label_edge > annotation_edge {
            return Ok(None);
        }
        lines.push(line);
    }
    Ok(Some(lines))
}

fn significance_level_spacing(label_extent: f64, metrics: BracketMetrics) -> f64 {
    metrics
        .level_step
        .max(metrics.label_gap + label_extent + metrics.tick_height)
}

fn expand_upper_limit(
    range: (f64, f64),
    panel: PanelBox,
    level_maxima: &[f64],
    level_label_extents: &[f64],
    annotation_ceiling: Option<f64>,
    global_maximum: f64,
    metrics: BracketMetrics,
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
    metrics: BracketMetrics,
) -> Result<(f64, f64), Error> {
    let maximum = rendered_maxima
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let fits =
        |upper: f64| data_fits_below_ceiling((range.0, upper), panel, maximum, ceiling, metrics);
    if fits(range.1) {
        return Ok(range);
    }
    if ceiling - panel.y0 <= metrics.data_gap + metrics.panel_padding {
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
        if fits(passing_upper) {
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
        if fits(midpoint) {
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
    metrics: BracketMetrics,
) -> bool {
    let fraction = ((maximum - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
    let data_top = panel.y0 + fraction * panel.height();
    data_top + metrics.data_gap <= ceiling - metrics.panel_padding
}

fn bracket_metrics(style: SignificanceMarkStyle) -> BracketMetrics {
    BracketMetrics {
        tick_height: SIGNIFICANCE_BRACKET_GEOMETRY.tick_height,
        label_gap: style.gap_spec(),
        data_gap: SIGNIFICANCE_BRACKET_GEOMETRY.data_gap,
        level_step: SIGNIFICANCE_BRACKET_GEOMETRY.level_step,
        panel_padding: SIGNIFICANCE_BRACKET_GEOMETRY.panel_padding,
    }
}

pub(crate) fn render_selected(
    chart: &Plot,
    result: &crate::spec::PlotRenderResult,
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let Some(plan) = &result.significance else {
        return Ok(());
    };
    if let Some(styles) = result
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.significance_styles.as_ref())
    {
        if styles.len() != plan.brackets.len() {
            return Err(Error::Render(
                "selected significance styles do not match retained brackets".into(),
            ));
        }
        for (bracket, style) in plan.brackets.iter().zip(styles) {
            backend.draw_planned_bracket(PlannedSignificanceCall {
                bracket: &bracket.geometry,
                style,
                label: &bracket.label,
            })?;
        }
        return Ok(());
    }
    render(chart, plan, theme, backend)
}

pub(crate) fn render(
    chart: &Plot,
    plan: &SignificancePlan,
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let mark_style = chart.significance_mark_style_spec();
    for bracket in &plan.brackets {
        let label = &bracket.label.plan;
        let style = BracketStyle {
            color: theme.annotations.bracket_stroke.paint,
            width: theme.annotations.bracket_stroke.width,
            label_color: label.style.paint,
            font_family: label
                .copy_stack()
                .map_err(|error| Error::Render(error.to_string()))?,
            font_size: label.style.font_size,
            line_height: label.line_height(),
            weight: label.style.weight,
            label_gap: mark_style.gap_spec(),
        };
        backend.draw_planned_bracket(PlannedSignificanceCall {
            bracket: &bracket.geometry,
            style: &style,
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
