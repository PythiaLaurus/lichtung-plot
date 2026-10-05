//! Root-prepared estimate semantics and selected point/interval/label geometry.

#[cfg(test)]
mod extraction;

use crate::Error;

#[cfg(test)]
use crate::layout::AxisTick;
use crate::layout::{AxisSpec, Layout, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, TextAnchor};
use crate::spec::{AxisScale, EstimateStatus, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::Plot;

#[derive(Debug)]
pub(crate) struct PreparedEstimate {
    pub(crate) kind: crate::PlotKind,
    pub(crate) labels: Vec<String>,
    estimates: Vec<Option<EstimateTuple>>,
    adjacent: Vec<Option<(String, bool)>>,
    reference: Option<f64>,
    pub(crate) range: Option<(f64, f64)>,
    family: &'static str,
    point_interval: bool,
}

#[derive(Debug)]
pub(crate) struct PreparedEstimateInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedEstimate)>,
}

impl PreparedEstimate {
    pub(crate) fn primary_mark_count(
        &self,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<u128, Error> {
        resources
            .claim_visits(self.estimates.len() as u128)
            .map_err(|error| {
                Error::Render(format!("estimate primary-mark count admission: {error:?}"))
            })?;
        Ok(self
            .estimates
            .iter()
            .filter(|estimate| estimate.is_some())
            .count() as u128)
    }

    /// The category axis retains unavailable rows but does not fabricate a
    /// numeric estimate or painted interval for any such slot.
    pub(crate) fn mark_intrinsic(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
    ) -> Result<(f64, f64), Error> {
        if self.labels.len() != self.estimates.len() {
            return Err(Error::Render(
                "estimate categories and prepared rows differ".into(),
            ));
        }
        let resources = theme.prepared_font_environment()?.generation_resources();
        resources
            .claim_visits(self.estimates.len() as u128)
            .map_err(|error| {
                Error::Render(format!("estimate slot intrinsic admission: {error:?}"))
            })?;
        let mut available = false;
        let mut unavailable = false;
        let mut interval = false;
        for row in &self.estimates {
            if let Some(tuple) = row {
                available = true;
                interval |= tuple.lower.zip(tuple.upper).is_some();
            } else {
                unavailable = true;
            }
        }
        let mut numeric = 0.0_f64;
        let mut pitch = 1.0_f64 / 1024.0;
        if available {
            let point =
                chart.point_style(theme, (0, 0, 0), theme.marks.point_sizes.interval_estimate);
            let (width, height) = crate::render::geometry::point_ink_dimensions(theme, &point)?;
            numeric = numeric.max(width);
            pitch = pitch.max(height);
        }
        if interval {
            let stroke = crate::render::geometry::line_ink_width(
                theme,
                &theme.marks.estimate_interval.backend_style(),
            )?;
            numeric = numeric.max(stroke);
            pitch = pitch.max(stroke);
            if self.point_interval && stroke > 0.0 {
                pitch = pitch.max(8.0 + stroke);
            }
        }
        if unavailable {
            let text = theme.measure_text(
                NOT_ESTIMABLE_TEXT,
                &theme.typography.estimate_detail,
                PhysicalTextAlignment::Center,
                "estimate unavailable category intrinsic",
                false,
            )?;
            pitch = pitch.max(text.height());
        }
        if !self.adjacent.is_empty() && self.adjacent.len() != self.estimates.len() {
            return Err(Error::Render(
                "estimate adjacent labels and rows differ".into(),
            ));
        }
        resources
            .claim_visits(self.adjacent.len() as u128)
            .map_err(|error| {
                Error::Render(format!("estimate adjacent-height admission: {error:?}"))
            })?;
        let significance = theme
            .typography
            .effective_significance_label(chart.significance_mark_style_override(), &resources)?;
        for adjacent in self.adjacent.iter().flatten() {
            let appearance = if adjacent.1 {
                &significance
            } else {
                &theme.typography.estimate_detail
            };
            pitch = pitch.max(
                theme
                    .measure_text(
                        &adjacent.0,
                        appearance,
                        PhysicalTextAlignment::Start,
                        "estimate adjacent category height",
                        adjacent.1 && chart.significance_mark_style_override().is_some(),
                    )?
                    .height(),
            );
        }
        let count = u32::try_from(self.labels.len()).map_err(|_| {
            Error::Render("estimate category count exceeds coordinate range".into())
        })?;
        let height = pitch * f64::from(count);
        if !height.is_finite() {
            return Err(Error::Render(
                "estimate category hard span overflowed".into(),
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

    pub(crate) fn new<'a>(
        chart: &Plot,
        data: crate::data::PlotData<'_>,
        annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
    ) -> Result<Self, Error> {
        data.claim_mark_rows()?;
        #[cfg(test)]
        count(chart.kind(), 0);
        if chart.kind() != crate::PlotKind::PointInterval {
            return prepare_model(chart, data, annotations)
                .map_err(crate::layout::fit::PlanError::into_error);
        }
        let rows = crate::plots::point_interval::prepare(chart, data)?;
        data.claim_source_visits(rows.len())?;
        let range = estimate_data_range(rows.iter().map(|row| &row.estimate));
        let range = match (chart.x_axis_spec().limits, range) {
            (Some(limits), _) => Some(limits),
            (_, Some(range)) => Some(crate::plots::point_interval::point_interval_display_range(
                range,
                chart.x_axis_spec(),
                data.source_resources()?,
            )?),
            _ => None,
        };
        let mut labels = Vec::new();
        let mut estimates = Vec::new();
        data.reserve_source(&mut labels, rows.len())?;
        data.reserve_source(&mut estimates, rows.len())?;
        for row in rows {
            labels.push(row.label);
            estimates.push(row.estimate);
        }
        Ok(Self {
            kind: chart.kind(),
            labels,
            estimates,
            adjacent: Vec::new(),
            reference: chart.zero_line_visible().then_some(0.0),
            range,
            family: "point_interval",
            point_interval: true,
        })
    }

    pub(crate) fn x_range(&self) -> Result<(f64, f64), Error> {
        self.range.ok_or_else(|| {
            Error::Data(format!(
                "{}: all rows are not_estimable; set explicit x-axis limits",
                self.family
            ))
        })
    }
}

pub(crate) const NOT_ESTIMABLE_TEXT: &str = "Not estimable";

struct AdjacentTextPlan {
    plan: ResolvedTextPlan,
    position: Point2D,
    anchor: TextAnchor,
    deferred_annotation: bool,
}

#[derive(Debug)]
struct EstimateRowPayload {
    interval: Option<[Point2D; 2]>,
    caps: [Option<[Point2D; 2]>; 2],
    point: Option<Point2D>,
    text: Option<crate::render::backend::PlacedText>,
}

#[derive(Debug)]
pub(crate) struct EstimatePayload {
    clip: crate::layout::PanelBox,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
    reference: Option<[Point2D; 2]>,
    reference_style: LineStyle,
    interval_style: LineStyle,
    point_style: crate::render::backend::PointStyle,
    rows: Vec<EstimateRowPayload>,
    #[cfg(test)]
    kind: crate::PlotKind,
}

impl EstimatePayload {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(self.kind, 3);
        with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.reference {
                backend.draw_line(line, &self.reference_style)?;
            }
            for row in &self.rows {
                if let Some(line) = &row.interval {
                    backend.draw_line(line, &self.interval_style)?;
                }
                for cap in row.caps.iter().flatten() {
                    backend.draw_line(cap, &self.interval_style)?;
                }
                if let Some(point) = &row.point {
                    backend.draw_scatter(std::slice::from_ref(point), &self.point_style)?;
                }
                if let Some(text) = &row.text {
                    text.draw(backend)?;
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 3]> = const { std::cell::Cell::new([[0; 4]; 3]) };
}

#[cfg(test)]
fn count(kind: crate::PlotKind, stage: usize) {
    let family = match kind {
        crate::PlotKind::PointInterval => 0,
        crate::PlotKind::Coefficient => 1,
        crate::PlotKind::Forest => 2,
        _ => unreachable!(),
    };
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EstimateTuple {
    pub(crate) estimate: f64,
    pub(crate) lower: Option<f64>,
    pub(crate) upper: Option<f64>,
    /// Preserve raw bound distinction through correctly rounded scale transforms.
    pub(crate) nonzero_interval: bool,
}

pub(crate) fn prepare_estimate_tuples(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    family: &str,
    require_intervals: bool,
) -> Result<Vec<Option<EstimateTuple>>, Error> {
    let estimate_col = chart
        .x_col()
        .ok_or_else(|| Error::Data(format!("{family}: estimate column not set")))?;
    let interval_cols = match chart.interval_bound_cols() {
        Some(columns) => Some(columns),
        None if require_intervals => {
            return Err(Error::Data(format!(
                "{family}: complete interval columns are required"
            )))
        }
        None => None,
    };
    let Some(statuses) = data.estimate_statuses()? else {
        let estimates = crate::data::strict_finite_f64(data, estimate_col, family)?;
        let intervals = interval_cols
            .map(|(lower_col, upper_col)| {
                Ok::<_, Error>((
                    crate::data::strict_finite_f64(data, lower_col, family)?,
                    crate::data::strict_finite_f64(data, upper_col, family)?,
                ))
            })
            .transpose()?;
        if intervals.as_ref().is_some_and(|(lower, upper)| {
            lower.len() != estimates.len() || upper.len() != estimates.len()
        }) {
            return Err(Error::Data(format!(
                "{family}: estimate and interval columns must have equal lengths"
            )));
        }
        return data.collect_source(estimates.into_iter().enumerate().map(|(index, estimate)| {
            let (lower, upper) = intervals.as_ref().map_or((None, None), |(lower, upper)| {
                (lower.get(index).copied(), upper.get(index).copied())
            });
            Ok(Some(EstimateTuple {
                estimate,
                lower,
                upper,
                nonzero_interval: lower.zip(upper).is_some_and(|(lower, upper)| lower < upper),
            }))
        }));
    };

    let estimates = crate::data::nullable_finite_f64(data, estimate_col, family)?;
    let intervals = interval_cols
        .map(|(lower_col, upper_col)| {
            Ok::<_, Error>((
                crate::data::nullable_finite_f64(data, lower_col, family)?,
                crate::data::nullable_finite_f64(data, upper_col, family)?,
            ))
        })
        .transpose()?;
    if statuses.len() != estimates.len()
        || intervals.as_ref().is_some_and(|(lower, upper)| {
            lower.len() != estimates.len() || upper.len() != estimates.len()
        })
    {
        return Err(Error::Data(format!(
            "{family}: status, estimate, and interval columns must have equal lengths"
        )));
    }

    data.collect_source(statuses
        .into_iter()
        .enumerate()
        .map(|(index, status)| {
            data.claim_source_visits(1)?;
            let estimate = estimates[index];
            let (lower, upper) = intervals.as_ref().map_or((None, None), |(lower, upper)| {
                (lower[index], upper[index])
            });
            match status {
                EstimateStatus::Estimated => {
                    let Some(estimate) = estimate else {
                        return Err(Error::Data(format!(
                            "{family}: EstimateStatus::Estimated requires a complete non-null finite estimate tuple at source row {}",
                            data.source_row(index)
                        )));
                    };
                    if interval_cols.is_some() && (lower.is_none() || upper.is_none()) {
                        return Err(Error::Data(format!(
                            "{family}: EstimateStatus::Estimated requires a complete non-null finite estimate tuple at source row {}",
                            data.source_row(index)
                        )));
                    }
                    Ok(Some(EstimateTuple {
                        estimate,
                        lower,
                        upper,
                        nonzero_interval: lower.zip(upper).is_some_and(|(lower, upper)| lower < upper),
                    }))
                }
                EstimateStatus::NotEstimable
                    if estimate.is_none() && lower.is_none() && upper.is_none() =>
                {
                    Ok(None)
                }
                EstimateStatus::NotEstimable => Err(Error::Data(format!(
                    "{family}: EstimateStatus::NotEstimable requires the complete estimate tuple to be null at source row {}",
                    data.source_row(index)
                ))),
            }
        }))
}

pub(crate) fn estimate_data_range<'a>(
    estimates: impl IntoIterator<Item = &'a Option<EstimateTuple>>,
) -> Option<(f64, f64)> {
    estimates
        .into_iter()
        .filter_map(|estimate| {
            estimate.map(|tuple| {
                tuple
                    .lower
                    .zip(tuple.upper)
                    .map_or((tuple.estimate, tuple.estimate), |(lower, upper)| {
                        (lower, upper)
                    })
            })
        })
        .fold(None, |range, (lower, upper)| {
            Some(
                range.map_or((lower, upper), |(minimum, maximum): (f64, f64)| {
                    (minimum.min(lower), maximum.max(upper))
                }),
            )
        })
}

pub(crate) fn plan_not_estimable(
    theme: &ResolvedTheme,
    appearance: &crate::theme::TextAppearance,
    layout: &Layout,
    vertical_guide_x: Option<f64>,
    scope: &crate::layout::fit::FitScope,
) -> Result<(ResolvedTextPlan, f64), crate::layout::fit::PlanError> {
    let text_plan = theme.measure_legacy_appearance_text(
        NOT_ESTIMABLE_TEXT,
        appearance,
        PhysicalTextAlignment::Center,
        "not estimable status",
        false,
    )?;
    let x = not_estimable_text_x(
        layout.projection.panel_box(),
        text_plan.width(),
        vertical_guide_x,
        scope,
    )?;
    Ok((text_plan, x))
}

fn not_estimable_text_x(
    panel: crate::layout::PanelBox,
    text_width: f64,
    vertical_guide_x: Option<f64>,
    scope: &crate::layout::fit::FitScope,
) -> Result<f64, crate::layout::fit::PlanError> {
    const PANEL_PADDING: f64 = 6.0;
    const GUIDE_CLEARANCE: f64 = 6.0;

    let left_edge = panel.x0 + PANEL_PADDING;
    let right_edge = panel.x1 - PANEL_PADDING;
    if right_edge - left_edge < text_width {
        return Err(scope.failure(crate::layout::fit::FitKind::Text, format!(
            "Not estimable row text requires {text_width:.1}px but the chart panel provides {:.1}px after padding; increase chart width",
            (right_edge - left_edge).max(0.0)
        )).into());
    }
    let Some(guide_x) = vertical_guide_x else {
        return Ok(f64::midpoint(left_edge, right_edge));
    };
    let left_right = guide_x - GUIDE_CLEARANCE;
    let right_left = guide_x + GUIDE_CLEARANCE;
    let left_width = left_right - left_edge;
    let right_width = right_edge - right_left;
    let left_fits = left_width >= text_width;
    let right_fits = right_width >= text_width;
    match (left_fits, right_fits) {
        (true, true) if right_width > left_width => Ok(f64::midpoint(right_left, right_edge)),
        (true, _) => Ok(f64::midpoint(left_edge, left_right)),
        (false, true) => Ok(f64::midpoint(right_left, right_edge)),
        (false, false) => Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision,
            "Not estimable row text does not fit on either side of the vertical reference or zero line; increase chart width or move the line with exact explicit limits"
        ).into()),
    }
}

pub(crate) fn axis_categories(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    family: &str,
) -> Result<Vec<String>, Error> {
    let column = chart
        .y_col()
        .ok_or_else(|| Error::Data(format!("{family}: term/label column not set")))?;
    let labels = crate::data::strict_strings(data, column, family)?;
    for (index, label) in labels.iter().enumerate() {
        data.claim_source_visits(index)?;
        if labels[..index].contains(label) {
            return Err(Error::Data(format!(
                "{family}: term/label values must be unique"
            )));
        }
    }
    Ok(labels)
}

pub(crate) fn shared_x_domain(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
) -> Result<(f64, f64), Error> {
    let forest = chart.kind() == crate::spec::PlotKind::Forest;
    let family = if forest { "forest" } else { "coefficient" };
    let reference = if forest {
        chart.forest_reference_value()
    } else {
        chart.coefficient_reference_line_value().unwrap_or(0.0)
    };
    let estimates = prepare_estimate_tuples(chart, data, family, forest)?;
    let transform = |value: f64| -> Result<f64, Error> {
        if !value.is_finite() {
            return Err(Error::Data(format!("{family}: axis values must be finite")));
        }
        match chart.axis_scale_spec() {
            AxisScale::Linear => Ok(value),
            AxisScale::Log10 if value > 0.0 => crate::numeric::log10(value, data.source_resources()?),
            AxisScale::Log10 => Err(Error::Data(format!("{family}: logarithmic scale requires strictly positive estimates, bounds, and reference values"))),
        }
    };
    let reference = transform(reference)?;
    let mut range: Option<(f64, f64)> = None;
    for estimate in estimates.into_iter().flatten() {
        for value in [Some(estimate.estimate), estimate.lower, estimate.upper]
            .into_iter()
            .flatten()
        {
            let value = transform(value)?;
            range = Some(range.map_or((value, value), |range| {
                (range.0.min(value), range.1.max(value))
            }));
        }
    }
    if let Some(limits) = chart.x_axis_spec().limits {
        let range = (transform(limits.0)?, transform(limits.1)?);
        if range.0 >= range.1 {
            return Err(Error::Data(format!("{family}: logarithmic x-axis limits must remain distinguishable after transformation")));
        }
        return Ok(range);
    }
    let range = range.ok_or_else(|| {
        Error::Data(format!(
            "{family}: all rows are not_estimable; set explicit x-axis limits"
        ))
    })?;
    crate::layout::continuous_auto_range(
        (range.0.min(reference), range.1.max(reference)),
        &AxisSpec::default(),
        data.source_resources()?,
    )
}

#[expect(
    clippy::too_many_lines,
    reason = "model semantics validate tuples, attached labels, and transformed domains before fonts"
)]
fn prepare_model<'a>(
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
) -> Result<PreparedEstimate, crate::layout::fit::PlanError> {
    let require_intervals = chart.kind() == crate::PlotKind::Forest;
    let family = if require_intervals {
        "forest"
    } else {
        "coefficient"
    };
    let reference = if require_intervals {
        chart.forest_reference_value()
    } else {
        chart.coefficient_reference_line_value().unwrap_or(0.0)
    };
    if chart.hue_col().is_some() || chart.color_col().is_some() {
        return Err(Error::Data(format!(
            "{family}: hue and continuous color channels are not supported"
        ))
        .into());
    }
    let labels = axis_categories(chart, data, family)?;
    let estimates = prepare_estimate_tuples(chart, data, family, require_intervals)?;
    let has_intervals = chart.interval_bound_cols().is_some();
    if chart.estimate_labels_spec().is_some() && has_intervals {
        return Err(Error::Data(format!(
            "{family}: numeric estimate labels are supported only for point-only displays without intervals"
        )).into());
    }
    let mut coefficient_significance = data.collect_source((0..labels.len()).map(|_| Ok(None)))?;
    for annotation in annotations {
        data.claim_source_visits(labels.len())?;
        let Some(target) = annotation.at_target() else {
            return Err(Error::Data(
                "coefficient: only attached significance annotations are supported".into(),
            )
            .into());
        };
        if target.series_name().is_some() {
            return Err(Error::Data(
                "coefficient: series-qualified significance targets are not supported".into(),
            )
            .into());
        }
        let index = labels
            .iter()
            .position(|label| label == target.category_name())
            .ok_or_else(|| {
                Error::Data(format!(
                    "coefficient significance references unknown term {:?}; available terms: {}",
                    target.category_name(),
                    labels.join(", ")
                ))
            })?;
        if coefficient_significance[index].is_some() {
            return Err(Error::Data(format!(
                "coefficient significance target {:?} is annotated more than once",
                target.category_name()
            ))
            .into());
        }
        coefficient_significance[index] =
            Some(data.copy_source_string(annotation.mark().glyph()?)?);
    }
    if !reference.is_finite() {
        return Err(Error::Data(format!(
            "{family}: reference value must be finite, got {reference}"
        ))
        .into());
    }
    if labels.is_empty() || labels.len() != estimates.len() {
        return Err(Error::Data(format!(
            "{family}: label, estimate, and interval columns must have equal non-zero lengths"
        ))
        .into());
    }
    for (index, estimate) in estimates.iter().enumerate() {
        data.claim_source_visits(1)?;
        if estimate.is_some_and(|tuple| {
            tuple
                .lower
                .zip(tuple.upper)
                .is_some_and(|(lower, upper)| lower > tuple.estimate || tuple.estimate > upper)
        }) {
            let source_row = data.source_row(index);
            return Err(Error::Data(format!(
                "{family}: source row {source_row} must satisfy lower <= estimate <= upper"
            ))
            .into());
        }
        if estimate.is_none() && coefficient_significance[index].is_some() {
            return Err(Error::Data(format!(
                "coefficient: not_estimable source row {} cannot carry a significance mark",
                data.source_row(index)
            ))
            .into());
        }
    }
    let scale = chart.axis_scale_spec();
    let transform = |value: f64| -> Result<f64, Error> {
        match scale {
            AxisScale::Linear => Ok(value),
            AxisScale::Log10 if value > 0.0 => crate::numeric::log10(value, data.source_resources()?),
            AxisScale::Log10 => Err(Error::Data(format!(
                "{family}: logarithmic scale requires strictly positive estimates, bounds, and reference values"
            ))),
        }
    };
    let transformed_estimates = data.collect_source(estimates.iter().map(|estimate| {
        estimate
            .map(|tuple| {
                Ok::<_, Error>(EstimateTuple {
                    estimate: transform(tuple.estimate)?,
                    lower: tuple.lower.map(transform).transpose()?,
                    upper: tuple.upper.map(transform).transpose()?,
                    nonzero_interval: tuple.nonzero_interval,
                })
            })
            .transpose()
    }))?;
    let transformed_reference = transform(reference)?;
    data.claim_source_visits(transformed_estimates.len())?;
    let range = if let Some((lower, upper)) = chart.x_axis_spec().limits {
        let limits = (transform(lower)?, transform(upper)?);
        if limits.0 >= limits.1 {
            return Err(Error::Data(format!("{family}: logarithmic x-axis limits must remain distinguishable after transformation")).into());
        }
        Some(limits)
    } else {
        estimate_data_range(&transformed_estimates)
            .map(|(lower, upper)| {
                crate::layout::continuous_auto_range(
                    (
                        lower.min(transformed_reference),
                        upper.max(transformed_reference),
                    ),
                    &AxisSpec::default(),
                    data.source_resources()?,
                )
            })
            .transpose()?
    };
    let adjacent = data.collect_source(estimates.iter().zip(coefficient_significance).map(
        |(estimate, mark)| {
            if let (Some(tuple), Some(format)) = (estimate, chart.estimate_labels_spec()) {
                Ok(Some((format.format_source(tuple.estimate, data)?, false)))
            } else {
                Ok(mark.map(|mark| (mark, true)))
            }
        },
    ))?;
    Ok(PreparedEstimate {
        kind: chart.kind(),
        labels,
        estimates: transformed_estimates,
        adjacent,
        reference: Some(transformed_reference),
        range,
        family,
        point_interval: false,
    })
}

pub(crate) fn measure_marks(
    chart: &Plot,
    context: PlotRenderContext<'_>,
    layout: Layout,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    #[cfg(test)]
    count(chart.kind(), 1);
    let prepared = chart
        .prepared_estimate
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, prepared)| prepared)
        .ok_or_else(|| {
            Error::Render("estimate measurement requires root-prepared semantics".into())
        })?;
    let theme = context.theme;
    let resources = theme.prepared_font_environment()?.generation_resources();
    let data = context.data.with_axis_resources(&resources);
    project(chart, prepared, layout, context, data)
}

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    reason = "project rows and their selected text in scientific order before capture"
)]
fn project(
    chart: &Plot,
    prepared: &PreparedEstimate,
    layout: Layout,
    context: PlotRenderContext<'_>,
    data: crate::data::PlotData<'_>,
) -> Result<PlotRenderResult, crate::layout::fit::PlanError> {
    let theme = context.theme;
    let labels = &prepared.labels;
    let estimates = &prepared.estimates;
    let family = prepared.family;
    let row_count = labels.len() as f64;
    let x_range = layout.projection.x_range();
    let reference_line_x = prepared
        .reference
        .filter(|value| *value >= x_range.0 && *value <= x_range.1)
        .map(|value| layout.projection.project(value, 0.0).0);
    let base = &theme.typography.significance_label;
    let exact = chart.significance_mark_style_override();
    let significance_label = crate::theme::TextAppearance {
        stack: data.copy_font_stack(&base.stack)?,
        paint: base.paint,
        font_size: exact.map_or(base.font_size, crate::SignificanceMarkStyle::font_size_spec),
        weight: exact.map_or(base.weight, crate::SignificanceMarkStyle::weight_spec),
        line_height_multiplier: base.line_height_multiplier,
    };
    let detail_role = &theme.typography.estimate_detail;
    let detail_label = crate::theme::TextAppearance {
        stack: data.copy_font_stack(&detail_role.stack)?,
        paint: detail_role.paint,
        font_size: detail_role.font_size,
        weight: detail_role.weight,
        line_height_multiplier: detail_role.line_height_multiplier,
    };
    data.claim_source_visits(estimates.len())?;
    let not_estimable_plan = estimates
        .iter()
        .position(Option::is_none)
        .map(|index| {
            plan_not_estimable(
                theme,
                &detail_label,
                &layout,
                reference_line_x,
                &context.fit_scope().child(
                    crate::layout::AddressRole::CategoricalValueLabel,
                    index,
                    &theme.prepared_font_environment()?.generation_resources(),
                )?,
            )
        })
        .transpose()?;
    let adjacent_label_plans = crate::layout::fit::collect(theme, "estimate adjacent labels", (0..labels.len())
        .map(|index| {
            let Some(tuple) = estimates[index] else {
                return Ok(None);
            };
            let Some((source, is_significance)) = prepared.adjacent.get(index).and_then(Option::as_ref) else {
                return Ok(None);
            };
            let is_significance = *is_significance;
            let scope = context.fit_scope().child(if is_significance { crate::layout::AddressRole::Annotation } else { crate::layout::AddressRole::CategoricalValueLabel }, index, &theme.prepared_font_environment()?.generation_resources())?;
            let (x, y) = layout
                .projection
                .project(tuple.estimate, row_count - index as f64 - 0.5);
            let (appearance, gap, owner, exact_override) = if is_significance {
                (
                    &significance_label,
                    chart.significance_mark_style_spec().gap_spec(),
                    "coefficient attached significance label",
                    chart.significance_mark_style_override().is_some(),
                )
            } else {
                (&detail_label, 7.0, "coefficient estimate label", false)
            };
            let panel = layout.projection.panel_box();
            let (position, anchor, plan) = {
                let measure = |alignment| {
                    if is_significance {
                        theme.measure_text(source, appearance, alignment, owner, exact_override)
                    } else {
                        theme.measure_legacy_appearance_text(
                            source,
                            appearance,
                            alignment,
                            owner,
                            exact_override,
                        )
                    }
                };
                let start_plan = measure(PhysicalTextAlignment::Start)?;
                let width = start_plan.width();
                if x + gap + width <= panel.x1 {
                    (
                        Point2D::new((x + gap).round(), y.round()),
                        TextAnchor::Start,
                        start_plan,
                    )
                } else if x - gap - width >= panel.x0 {
                    let end_plan = measure(PhysicalTextAlignment::End)?;
                    (
                        Point2D::new((x - gap).round(), y.round()),
                        TextAnchor::End,
                        end_plan,
                    )
                } else if chart.user_x_limits().is_none() {
                    // Keep the first legal anchor order. The selected required
                    // glyph requests numeric headroom from the root instead of
                    // failing a feasible automatic domain at this initial fit.
                    (
                        Point2D::new((x + gap).round(), y.round()),
                        TextAnchor::Start,
                        start_plan,
                    )
                } else {
                    let message = if is_significance {
                        format!(
                            "{family}: significance mark {source:?} does not fit beside its point; increase chart width or widen the x-axis range"
                        )
                    } else {
                        format!(
                            "{family}: formatted estimate label {source:?} does not fit beside its point; increase chart width, shorten the suffix, or reduce precision"
                        )
                    };
                    return Err(scope.failure(crate::layout::fit::FitKind::OverlayCollision, message).into());
                }
            };
            Ok(Some(AdjacentTextPlan {
                plan,
                position,
                anchor,
                deferred_annotation: is_significance,
            }))
        })
        )?;
    let reference = prepared
        .reference
        .filter(|_| !prepared.point_interval || reference_line_x.is_some())
        .map(|value| {
            let (x, y0) = layout.projection.project(value, 0.0);
            let (_, y1) = layout.projection.project(value, row_count);
            [Point2D::new(x, y0), Point2D::new(x, y1)]
        });
    let mut payload = EstimatePayload {
        clip: layout.projection.panel_box(),
        domain_expansions: [None; 2],
        reference,
        reference_style: if prepared.point_interval {
            theme.marks.point_interval_reference.backend_style()
        } else {
            theme.marks.estimate_reference.backend_style()
        },
        interval_style: theme.marks.estimate_interval.backend_style(),
        point_style: chart.point_style(
            theme,
            crate::theme::AccentColors::resolve(&theme.colors, 1, data.source_resources()?)?
                .color(0)?,
            theme.marks.point_sizes.interval_estimate,
        ),
        rows: Vec::new(),
        #[cfg(test)]
        kind: chart.kind(),
    };
    data.reserve_source(&mut payload.rows, estimates.len())?;
    let mut deferred_annotation_text = Vec::new();
    for (index, adjacent) in adjacent_label_plans.into_iter().enumerate() {
        data.claim_source_visits(1)?;
        let y = row_count - index as f64 - 0.5;
        let mut row = EstimateRowPayload {
            interval: None,
            caps: [None, None],
            point: None,
            text: None,
        };
        if let Some(tuple) = estimates[index] {
            if let Some((lower, upper)) = tuple.lower.zip(tuple.upper) {
                let (visible_lower, visible_upper) = if prepared.point_interval {
                    (lower.max(x_range.0), upper.min(x_range.1))
                } else {
                    (lower, upper)
                };
                if visible_lower <= visible_upper {
                    let (left, py) = layout.projection.project(visible_lower, y);
                    let (right, _) = layout.projection.project(visible_upper, y);
                    layout.projection.validate_span(
                        crate::layout::AxisSide::Bottom,
                        (visible_lower, visible_upper),
                        (left, right),
                        if prepared.point_interval {
                            visible_lower < visible_upper
                        } else {
                            tuple.nonzero_interval
                        },
                        "estimate interval",
                    )?;
                    row.interval = Some([Point2D::new(left, py), Point2D::new(right, py)]);
                }
                if prepared.point_interval {
                    for (index, bound) in [lower, upper].into_iter().enumerate() {
                        if bound >= x_range.0 && bound <= x_range.1 {
                            let (x, py) = layout.projection.project(bound, y);
                            row.caps[index] =
                                Some([Point2D::new(x, py - 4.0), Point2D::new(x, py + 4.0)]);
                        }
                    }
                }
            }
            if !prepared.point_interval
                || (tuple.estimate >= x_range.0 && tuple.estimate <= x_range.1)
            {
                let (x, py) = layout.projection.project(tuple.estimate, y);
                row.point = Some(Point2D::new(x, py));
            }
            if let Some(adjacent) = adjacent {
                if adjacent.deferred_annotation {
                    data.reserve_source(&mut deferred_annotation_text, 1)?;
                    deferred_annotation_text.push(
                        crate::layout::DeferredAnnotationTextPlan::from_resolved(
                            adjacent.plan.clone(),
                        ),
                    );
                }
                row.text = Some(crate::render::backend::PlacedText::middle(
                    adjacent.plan,
                    adjacent.position,
                    adjacent.anchor,
                    0.0,
                ));
            }
        } else {
            let (plan, x) = not_estimable_plan
                .as_ref()
                .ok_or_else(|| Error::Render("not estimable text plan is missing".into()))?;
            let (_, py) = layout.projection.project(x_range.0, y);
            row.text = Some(crate::render::backend::PlacedText::middle(
                plan.clone(),
                Point2D::new(x.round(), py.round()),
                TextAnchor::Middle,
                0.0,
            ));
        }
        payload.rows.push(row);
    }
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    let mut label_ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
    let mut label_source: Option<(f64, f64)> = None;
    data.claim_source_visits(payload.rows.len())?;
    for (index, row) in payload.rows.iter().enumerate() {
        let Some(text) = &row.text else { continue };
        let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
        ink.text(text.placement, &text.plan, None)?;
        for bounds in ink.bounds() {
            if bounds.bottom < clip.y0 || bounds.top > clip.y1 {
                return Err(context
                    .fit_scope()
                    .child(
                        crate::layout::AddressRole::CategoricalValueLabel,
                        index,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        crate::layout::fit::FitKind::Text,
                        "required Estimate text exceeds its categorical row",
                    )
                    .into());
            }
            if (bounds.left < clip.x0 || bounds.right > clip.x1)
                && (chart.user_x_limits().is_some() || estimates[index].is_none())
            {
                return Err(context
                    .fit_scope()
                    .child(
                        crate::layout::AddressRole::CategoricalValueLabel,
                        index,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        crate::layout::fit::FitKind::OverlayCollision,
                        "required Estimate text exceeds its numeric panel",
                    )
                    .into());
            }
        }
        if let Some(tuple) = estimates[index] {
            let source = label_source.get_or_insert((tuple.estimate, tuple.estimate));
            source.0 = source.0.min(tuple.estimate);
            source.1 = source.1.max(tuple.estimate);
            label_ink.append(&ink)?;
        }
    }
    if chart.user_x_limits().is_none() {
        if let (Some(numeric), Some(bounds)) = (
            label_source,
            label_ink
                .bounds()
                .iter()
                .copied()
                .reduce(crate::render::geometry::GeometryBox::union),
        ) {
            payload.domain_expansions = crate::layout::interior::automatic_absolute_ink_expansions(
                [numeric, (0.0, row_count)],
                &layout.projection,
                bounds,
                [true, false],
                data,
                context.fit_scope(),
            )?;
        }
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
    if let Some(line) = &payload.reference {
        geometry.line(line, &payload.reference_style, Some(clip))?;
    }
    for row in &payload.rows {
        if let Some(line) = &row.interval {
            geometry.line(line, &payload.interval_style, Some(clip))?;
        }
        for cap in row.caps.iter().flatten() {
            geometry.line(cap, &payload.interval_style, Some(clip))?;
        }
        if let Some(point) = &row.point {
            geometry.scatter(std::slice::from_ref(point), &payload.point_style, clip)?;
        }
        if let Some(text) = &row.text {
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
    }
    #[cfg(test)]
    count(chart.kind(), 2);
    let mut result = PlotRenderResult::new(layout, Vec::new())
        .with_deferred_annotation_text(deferred_annotation_text);
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.marks = Some(data.source_arc(crate::layout::interior::SelectedMarkPlan {
        payload: crate::layout::interior::MarkPayload::Estimate(payload),
        geometry,
    })?);
    Ok(result)
}

pub(crate) fn log_tick_plan(
    range: (f64, f64),
    interval: Option<crate::layout::TickInterval>,
    panel: crate::layout::PanelBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::NumericTickPlan, crate::layout::fit::PlanError> {
    let projection = crate::layout::Projection::new(range, (0.0, 1.0), panel.as_tuple());
    crate::layout::numeric_tick_plan(
        range,
        interval,
        true,
        &projection,
        crate::layout::AxisSide::Bottom,
        theme,
        false,
        scope,
    )
}

#[cfg(test)]
fn log_ticks(
    range: (f64, f64),
    panel: crate::layout::PanelBox,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    Ok(
        log_tick_plan(range, None, panel, theme, &crate::layout::fit::ROOT_SCOPE)
            .map_err(crate::layout::fit::PlanError::into_error)?
            .ticks,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Bounds, CategoryValues, NumericValues};

    #[test]
    fn categorical_estimate_hard_pitch_uses_available_marks_and_unavailable_text() {
        let source = crate::Plot::point_interval(
            CategoryValues::new("term", ["available", "unavailable"]).unwrap(),
            NumericValues::nullable("estimate", [Some(1.0), None]),
            Bounds::new(
                NumericValues::nullable("lower", [Some(0.5), None]),
                NumericValues::nullable("upper", [Some(1.5), None]),
            ),
        )
        .estimate_status([
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::NotEstimable,
        ])
        .x_limits(0.0, 2.0)
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let base = chart.intrinsic_body_minimum(&theme).unwrap();
        theme.typography.estimate_detail.font_size = 48;
        let text = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(text.0.to_bits(), base.0.to_bits());
        assert!(text.1 > base.1);
        theme.marks.point_sizes.interval_estimate = 54;
        let points = chart.intrinsic_body_minimum(&theme).unwrap();
        assert_eq!(points.0.to_bits(), base.0.to_bits());
        assert!(points.1 > text.1);
    }

    #[test]
    fn attached_estimate_labels_contribute_measured_categorical_row_height() {
        let source = |labels: bool| {
            let plot = crate::Plot::coefficient(
                CategoryValues::new("term", ["first", "second"]).unwrap(),
                crate::CoefficientInput::estimates(NumericValues::new("estimate", [1.0, 2.0])),
            );
            if labels {
                plot.estimate_labels(crate::NumericLabelFormat::default())
                    .erase()
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
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.typography.estimate_detail.font_size = 48;
        let without = intrinsic(source(false), &theme);
        let with = intrinsic(source(true), &theme);
        assert!(with.1 > without.1);
        assert_eq!(with.0.to_bits(), without.0.to_bits());
    }

    #[test]
    fn custom_estimate_detail_role_changes_required_unavailable_text_before_capture() {
        let source = crate::Plot::point_interval(
            CategoryValues::new("term", ["available", "unavailable"]).unwrap(),
            NumericValues::nullable("estimate", [Some(1.0), None]),
            Bounds::new(
                NumericValues::nullable("lower", [Some(0.5), None]),
                NumericValues::nullable("upper", [Some(1.5), None]),
            ),
        )
        .estimate_status([
            crate::EstimateStatus::Estimated,
            crate::EstimateStatus::NotEstimable,
        ])
        .size(1200, 800)
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let panel = crate::layout::PanelBox::new(100.0, 900.0, 60.0, 600.0);
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let old = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        theme.typography.estimate_detail.font_size = 18;
        theme.typography.estimate_detail.paint = crate::theme::Color::rgb(5, 7, 9);
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
            .unwrap();
        let text = |result: &PlotRenderResult| {
            let crate::layout::interior::MarkPayload::Estimate(payload) =
                &result.marks.as_ref().unwrap().payload
            else {
                panic!("selected estimate display")
            };
            let row = &payload.rows[1];
            (row.interval, row.text.as_ref().unwrap().plan.style.clone())
        };
        let (old_interval, old_text) = text(&old);
        let (new_interval, new_text) = text(&selected);
        assert_eq!(old_interval, new_interval);
        assert_eq!(old_text.font_size, 14);
        assert_eq!(new_text.font_size, 18);
        assert_eq!(new_text.paint, crate::theme::Color::rgb(5, 7, 9));
    }

    fn tick_theme() -> ResolvedTheme {
        crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap()
    }

    #[test]
    fn not_estimable_text_clears_vertical_guide_by_measured_half_width() {
        let panel = crate::layout::PanelBox::new(0.0, 200.0, 0.0, 100.0);
        let width = 80.0;
        let guide = 100.0;
        let center = not_estimable_text_x(
            panel,
            width,
            Some(guide),
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap();

        assert!(guide - (center + width / 2.0) >= 6.0);
    }

    #[test]
    fn wide_log_ticks_cover_the_full_domain_without_exceeding_the_maximum() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (-100.0, 100.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.len() > 5);
        assert!(ticks.first().is_some_and(|tick| tick.position <= -90.0));
        assert!(ticks.last().is_some_and(|tick| tick.position >= 90.0));
    }

    #[test]
    fn extreme_log_ticks_keep_every_label_finite_and_nonzero() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (-400.0, 400.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().all(|tick| {
            tick.label != "0"
                && tick.label != "inf"
                && tick
                    .label
                    .trim_end_matches(['k', 'M'])
                    .parse::<f64>()
                    .is_ok_and(f64::is_finite)
        }));
    }

    #[test]
    fn upper_extreme_log_ticks_have_unique_expression_labels() {
        let theme = tick_theme();
        let ticks = log_ticks(
            (
                308.0,
                crate::numeric::log10(
                    f64::MAX,
                    &theme
                        .prepared_font_environment()
                        .unwrap()
                        .generation_resources(),
                )
                .unwrap(),
            ),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().enumerate().all(|(index, tick)| {
            !ticks[..index]
                .iter()
                .any(|previous| previous.label == tick.label)
                && tick
                    .label
                    .strip_prefix("10^")
                    .is_some_and(|exponent| exponent.parse::<f64>().is_ok_and(f64::is_finite))
        }));
    }

    #[test]
    fn lower_subnormal_log_ticks_omit_duplicate_rounded_values() {
        let lower = crate::numeric::log10(
            f64::from_bits(1),
            &crate::font::canonical::OperationGenerationResources::new(),
        )
        .unwrap();
        let theme = tick_theme();
        let ticks = log_ticks(
            (lower, -323.0),
            crate::layout::PanelBox::new(0.0, 2_000.0, 0.0, 100.0),
            &theme,
        )
        .unwrap();

        assert!(ticks.iter().enumerate().all(|(index, tick)| {
            !ticks[..index]
                .iter()
                .any(|previous| previous.label == tick.label)
        }));
    }
}
