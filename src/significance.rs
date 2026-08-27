//! Significance bracket planning and rendering for categorical plots.

mod spec;

pub(crate) use spec::SignificanceAnnotation;
pub use spec::{Attached, FacetCell, Pairwise, Significance, SignificanceMark, SignificanceTarget};

use crate::Error;

use crate::layout::{deterministic_text_width, PanelBox};
use crate::render::backend::{
    Backend, BracketMetrics, BracketOrientation, BracketStyle, SignificanceBracket,
    SIGNIFICANCE_BRACKET_METRICS,
};
use crate::{Plot, SignificanceMarkStyle};

#[derive(Debug)]
pub(crate) struct SignificancePlan {
    pub(crate) y_range: (f64, f64),
    pub(crate) brackets: Vec<SignificanceBracket>,
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
    left: usize,
    right: usize,
    level: usize,
    label: String,
}

pub(crate) fn plan(
    chart: &Plot,
    categories: &[String],
    rendered_maxima: &[f64],
    y_range: (f64, f64),
    panel: PanelBox,
) -> Result<SignificancePlan, Error> {
    plan_comparisons(
        chart.significance_annotations(),
        categories,
        rendered_maxima,
        y_range,
        chart.user_x_limits(),
        chart.user_y_limits(),
        panel,
        LegendAnnotationConstraint::NoObstacle,
        chart.significance_mark_style_spec(),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the wrapper preserves the existing validated significance planner inputs"
)]
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
    plan_comparisons_at(
        annotations,
        categories,
        rendered_maxima,
        None,
        y_range,
        categorical_x_override,
        y_limits,
        panel,
        legend_constraint,
        false,
        mark_style,
    )
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "the private planner keeps validated anchors, ranges, and panel geometry together"
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
    let metrics = bracket_metrics(mark_style);
    validate_range(y_range, y_limits.is_some())?;
    if categories.len() != rendered_maxima.len()
        || anchor_positions.is_some_and(|positions| positions.len() != categories.len())
        || categories.is_empty()
    {
        return Err(render_error(format!(
            "significance placement requires one rendered maximum per category ({} categories, {} maxima)",
            categories.len(),
            rendered_maxima.len()
        )));
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
        )));
    }
    if matches!(legend_constraint, LegendAnnotationConstraint::TopCeiling(ceiling) if !ceiling.is_finite())
    {
        return Err(render_error(
            "significance placement received a non-finite annotation ceiling",
        ));
    }
    for (index, maximum) in rendered_maxima.iter().enumerate() {
        if !maximum.is_finite() {
            return Err(render_error(format!(
                "significance placement received a non-finite rendered maximum for category {:?} at index {index}",
                categories[index]
            )));
        }
    }
    for (index, category) in categories.iter().enumerate() {
        if categories[..index].iter().any(|seen| seen == category) {
            return Err(render_error(format!(
                "significance placement requires unique category labels; {category:?} appears more than once"
            )));
        }
    }

    let mut resolved = Vec::new();
    let mut comparison_pairs = Vec::new();
    for annotation in annotations {
        let Some((endpoint_a, endpoint_b)) = annotation.endpoints() else {
            continue;
        };
        validate_bracket(annotation, categories)?;
        let a = categories
            .iter()
            .position(|category| category == endpoint_a.category_name())
            .ok_or_else(|| {
                render_error("validated significance category unexpectedly disappeared")
            })?;
        let b = categories
            .iter()
            .position(|category| category == endpoint_b.category_name())
            .ok_or_else(|| {
                render_error("validated significance category unexpectedly disappeared")
            })?;
        if anchor_positions
            .is_some_and(|positions| positions[a].to_bits() == positions[b].to_bits())
        {
            return Err(Error::Data(format!(
                "significance comparison targets {endpoint_a:?} and {endpoint_b:?} resolve to the same rendered anchor"
            )));
        }
        let pair = (a.min(b), a.max(b));
        if comparison_pairs.contains(&pair) {
            return Err(Error::Data(format!(
                "duplicate significance comparison for {:?} and {:?}; each unordered category pair may appear only once",
                categories[pair.0], categories[pair.1]
            )));
        }
        comparison_pairs.push(pair);
        resolved.push(ResolvedComparison {
            left: pair.0,
            right: pair.1,
            level: 0,
            label: annotation.mark().glyph()?.to_owned(),
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
        ));
    }
    let annotation_ceiling = match legend_constraint {
        LegendAnnotationConstraint::NoObstacle => None,
        LegendAnnotationConstraint::TopCeiling(ceiling) => Some(ceiling),
        LegendAnnotationConstraint::UnsupportedInside => {
            return Err(render_error(
                "significance brackets cannot safely avoid a centered or bottom inside legend; use a top-aligned, outside, or hidden legend",
            ));
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
            .then_with(|| left.label.cmp(&right.label))
    });
    let mut level_ends = Vec::<usize>::new();
    for comparison in &mut resolved {
        if let Some((level, end)) = level_ends
            .iter_mut()
            .enumerate()
            .find(|(_, end)| comparison.left > **end)
        {
            comparison.level = level;
            *end = comparison.right;
        } else {
            comparison.level = level_ends.len();
            level_ends.push(comparison.right);
        }
    }

    let mut level_maxima = vec![f64::NEG_INFINITY; level_ends.len()];
    let mut level_label_extents = vec![0.0_f64; level_ends.len()];
    for comparison in &resolved {
        let span_maximum = rendered_maxima[comparison.left..=comparison.right]
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        level_maxima[comparison.level] = level_maxima[comparison.level].max(span_maximum);
        let label_extent = if horizontal {
            deterministic_text_width(&comparison.label, f64::from(metrics.label_font_size))
        } else {
            f64::from(metrics.label_font_size)
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
        return Err(render_error(format!(
            "plot panel space below the inside legend is too short for {} significance level(s): {:.1}px available, more than {:.1}px required; increase chart height, move/hide the legend, or reduce comparisons",
            level_maxima.len(),
            available_height,
            minimum_height
        )));
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
        ) else {
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
            return Err(render_error(format!(
                "explicit y_limits ({}, {}) leave insufficient room for significance brackets; y_max must be at least {}; {remedy}",
                limits.0, limits.1, suggested.1
            )));
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
    ) {
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
            .then_with(|| left.label.cmp(&right.label))
    });
    let category_width = panel.width() / categories.len() as f64;
    let brackets = resolved
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
                f64::from(metrics.label_font_size) / 2.0
            } else {
                deterministic_text_width(
                    &comparison.label,
                    f64::from(metrics.label_font_size),
                ) / 2.0
            };
            if label_center - label_half_extent < panel.x0
                || label_center + label_half_extent > panel.x1
            {
                return Err(render_error(
                    "significance bracket label does not fit inside the plot panel; enlarge the chart or reduce mark typography",
                ));
            }
            Ok(SignificanceBracket {
                x_left,
                x_right,
                y: line_positions[comparison.level],
                label: comparison.label,
                orientation: BracketOrientation::Vertical,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

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
) -> Result<SignificancePlan, Error> {
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
) -> Result<SignificancePlan, Error> {
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
    )
    .map_err(|error| match error {
        Error::Render(message) => Error::Render(
            message
                .replace("y_limits", "x_limits")
                .replace("y_max", "x_max")
                .replace("automatic y", "automatic x")
                .replace(" y range", " x range")
                .replace("categorical x_limits", "categorical y_limits")
                .replace("top-aligned", "right-aligned"),
        ),
        other => other,
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
) -> Result<SignificancePlan, Error> {
    if categories.is_empty()
        || rendered_maxima.len() != categories.len()
        || rendered_maxima
            .iter()
            .any(|maxima| maxima.len() != groups.len().max(1))
    {
        return Err(render_error(
            "grouped significance placement requires one maximum per category and series anchor",
        ));
    }
    let group_count = groups.len().max(1);
    let mut labels = Vec::new();
    let mut maxima = Vec::new();
    let mut positions = Vec::new();
    for (category_index, category) in categories.iter().enumerate() {
        labels.push(format!("{category}\u{1f}"));
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
            labels.push(format!("{category}\u{1f}{group}"));
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
    let maxima = order.iter().map(|index| maxima[*index]).collect::<Vec<_>>();
    let positions = order
        .iter()
        .map(|index| positions[*index])
        .collect::<Vec<_>>();
    let transformed = chart
        .significance_annotations()
        .iter()
        .filter_map(|annotation| {
            annotation
                .endpoints()
                .map(|endpoints| (annotation, endpoints))
        })
        .map(|(annotation, (endpoint_a, endpoint_b))| {
            if stacked && (endpoint_a.series_name().is_some() || endpoint_b.series_name().is_some())
            {
                return Err(Error::Data(
                    "series-qualified significance targets are ambiguous for stacked bars".into(),
                ));
            }
            let endpoint_a =
                resolve_grouped_endpoint(endpoint_a, categories, groups, rendered_maxima)?;
            let endpoint_b =
                resolve_grouped_endpoint(endpoint_b, categories, groups, rendered_maxima)?;
            Ok(SignificanceAnnotation::pairwise(
                endpoint_a,
                endpoint_b,
                annotation.mark(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut result = plan_comparisons_at(
        &transformed,
        &labels,
        &maxima,
        Some(&positions),
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
    )?;
    if horizontal {
        for bracket in &mut result.brackets {
            bracket.orientation = BracketOrientation::Horizontal;
        }
    }
    Ok(result)
}

fn resolve_grouped_endpoint(
    endpoint: &SignificanceTarget,
    categories: &[String],
    groups: &[String],
    rendered_maxima: &[Vec<f64>],
) -> Result<String, Error> {
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
        Ok(format!(
            "{}\u{1f}{}",
            endpoint.category_name(),
            groups[index]
        ))
    } else {
        Ok(format!("{}\u{1f}", endpoint.category_name()))
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
) -> Option<Vec<f64>> {
    if annotation_ceiling.is_some_and(|ceiling| {
        !data_fits_below_ceiling(range, panel, global_maximum, ceiling, metrics)
    }) {
        return None;
    }
    if level_maxima.len() != level_label_extents.len() {
        return None;
    }
    let annotation_edge =
        annotation_ceiling.unwrap_or(panel.y1).min(panel.y1) - metrics.panel_padding;
    let mut lines = Vec::with_capacity(level_maxima.len());
    for (level, maximum) in level_maxima.iter().enumerate() {
        let fraction = ((*maximum - range.0) / (range.1 - range.0)).clamp(0.0, 1.0);
        let data_y = panel.y0 + fraction * panel.height();
        let clearance_y = data_y + metrics.tick_height + metrics.data_gap;
        let line = lines.last().map_or(clearance_y, |previous| {
            clearance_y
                .max(previous + significance_level_spacing(level_label_extents[level - 1], metrics))
        });
        let label_edge = line + metrics.label_gap + level_label_extents[level];
        if !line.is_finite() || !label_edge.is_finite() || label_edge > annotation_edge {
            return None;
        }
        lines.push(line);
    }
    Some(lines)
}

fn significance_level_spacing(label_extent: f64, metrics: BracketMetrics) -> f64 {
    metrics
        .level_step
        .max(metrics.label_gap + label_extent + metrics.panel_padding)
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
        ) {
            break lines;
        }
        failing_upper = passing_upper;
        span *= 2.0;
    };

    let mut lines = passing_lines;
    for _ in 0..64 {
        let midpoint = f64::midpoint(failing_upper, passing_upper);
        if let Some(candidate) = place_lines(
            (lower, midpoint),
            panel,
            level_maxima,
            level_label_extents,
            annotation_ceiling,
            global_maximum,
            metrics,
        ) {
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
        label_gap: style.gap_spec(),
        label_font_size: style.font_size_spec(),
        ..SIGNIFICANCE_BRACKET_METRICS
    }
}

pub(crate) fn render(
    chart: &Plot,
    brackets: &[SignificanceBracket],
    theme: &crate::theme::ResolvedTheme,
    backend: &mut dyn Backend,
) -> Result<(), Error> {
    let mark_style = chart.significance_mark_style_spec();
    let style = BracketStyle {
        color: theme.foreground,
        width: theme.bracket_width,
        font_family: theme.font_family.clone(),
        font_size: mark_style.font_size_spec(),
        weight: mark_style.weight_spec(),
        label_gap: mark_style.gap_spec(),
    };
    for bracket in brackets {
        backend.draw_bracket(bracket, &style)?;
    }
    Ok(())
}

fn render_error(message: impl Into<String>) -> Error {
    Error::Render(message.into())
}

#[cfg(test)]
mod tests;
