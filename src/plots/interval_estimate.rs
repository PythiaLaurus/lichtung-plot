//! Root-prepared estimate semantics and selected point/interval/label geometry.

use crate::alpha::{
    marks::{line_style, point_ink, ObservationAlpha},
    AlphaTarget, AlphaUnitKind,
};
use crate::Error;

use crate::layout::{AxisSpec, Layout, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{with_clip, Backend, LineStyle, Point2D, TextAnchor};
use crate::spec::{AxisScale, EstimateStatus, PlotRenderContext, PlotRenderResult};
use crate::theme::ResolvedTheme;
use crate::Plot;

#[cfg(test)]
mod alpha_tests;

#[derive(Debug)]
pub(crate) struct PreparedEstimate {
    pub(crate) kind: crate::PlotKind,
    pub(crate) labels: Vec<String>,
    estimates: Vec<Option<EstimateTuple>>,
    alphas: Vec<ObservationAlpha>,
    line_alphas: Vec<ObservationAlpha>,
    adjacent: Vec<Option<(String, bool)>>,
    reference: Option<f64>,
    pub(crate) range: Option<(f64, f64)>,
    family: &'static str,
    point_interval: bool,
    value_presentation: Option<std::sync::Arc<crate::numeric::set::NumericPresentation>>,
}

#[derive(Debug)]
pub(crate) struct PreparedEstimateInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedEstimate)>,
}

impl PreparedEstimate {
    pub(crate) fn numeric_explanation(&self) -> Option<&crate::Text> {
        self.value_presentation
            .as_ref()
            .and_then(|numeric| numeric.set.explanation.as_ref())
    }
    pub(crate) fn alpha_guide(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let point = chart.point_style(
            theme,
            (0, 0, 0),
            theme.marks.get_points().get_radii().get_interval_estimate(),
        );
        let line = theme
            .marks
            .get_strokes()
            .get_summaries()
            .get_estimate_interval()
            .backend_style();
        for ((row, &alpha), &interval) in self
            .estimates
            .iter()
            .zip(&self.alphas)
            .zip(&self.line_alphas)
        {
            let Some(row) = row else { continue };
            guide.point(AlphaGlyphRole::Primary, alpha, point.clone(), false)?;
            if row.lower.zip(row.upper).is_some() {
                guide.line(
                    AlphaGlyphRole::Line,
                    crate::alpha::AlphaUnitKind::Observation,
                    AlphaUnit::observation(interval),
                    interval.value,
                    line.clone(),
                    false,
                )?;
            }
        }
        Ok(())
    }

    fn interval_ink(&self, theme: &ResolvedTheme) -> Result<f64, Error> {
        crate::alpha::marks::require_aligned_points(self.estimates.len(), self.line_alphas.len())?;
        let style = theme
            .marks
            .get_strokes()
            .get_summaries()
            .get_estimate_interval()
            .backend_style();
        let mut width = 0.0_f64;
        for (row, alpha) in self.estimates.iter().zip(&self.line_alphas) {
            if row.and_then(|tuple| tuple.lower.zip(tuple.upper)).is_some() {
                width = width.max(crate::render::geometry::line_ink_width(&line_style(
                    alpha.value,
                    style.clone(),
                ))?);
            }
        }
        Ok(width)
    }

    fn point_ink(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
    ) -> Result<Option<crate::alpha::marks::PointInk>, Error> {
        point_ink(
            &chart.point_style(
                theme,
                (0, 0, 0),
                theme.marks.get_points().get_radii().get_interval_estimate(),
            ),
            self.estimates
                .iter()
                .zip(&self.alphas)
                .filter_map(|(row, &alpha)| row.map(|tuple| (tuple.estimate, 0.0, alpha))),
        )
    }
    pub(crate) fn text_occurrences(&self) -> impl Iterator<Item = (usize, &str, bool)> {
        self.estimates
            .iter()
            .enumerate()
            .filter_map(|(index, estimate)| {
                if estimate.is_none() {
                    Some((index, NOT_ESTIMABLE_TEXT, false))
                } else {
                    self.adjacent
                        .get(index)
                        .and_then(Option::as_ref)
                        .map(|(source, significance)| (index, source.as_str(), *significance))
                }
            })
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.estimates
            .iter()
            .filter(|estimate| estimate.is_some())
            .count() as u128
    }

    /// The category axis retains unavailable rows but does not fabricate a
    /// numeric estimate or painted interval for any such slot.
    #[expect(
        clippy::too_many_lines,
        reason = "scientific ink and eligible adjacent/status roles jointly determine the native row minimum"
    )]
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

        let mut available = false;
        let mut unavailable = false;
        for row in &self.estimates {
            if row.is_some() {
                available = true;
            } else {
                unavailable = true;
            }
        }
        let mut numeric = 0.0_f64;
        let mut pitch = 1.0_f64 / 1024.0;
        if available {
            let (width, height) = self
                .point_ink(chart, theme)?
                .map_or((0.0, 0.0), crate::alpha::marks::PointInk::dimensions);
            numeric = numeric.max(width);
            pitch = pitch.max(height);
        }
        let stroke = self.interval_ink(theme)?;
        numeric = numeric.max(stroke);
        pitch = pitch.max(stroke);
        if self.point_interval && stroke > 0.0 {
            pitch = pitch.max(
                2.0 * f64::from(
                    theme
                        .marks
                        .get_extents()
                        .get_point_interval_cap_half_height(),
                ) + stroke,
            );
        }
        let detail_indices = crate::data::source::collect_source(
            self.text_occurrences()
                .filter(|(_, _, significance)| !*significance)
                .map(|(index, _, _)| Ok(index)),
        )?;
        if unavailable
            && self.estimates.iter().enumerate().any(|(index, row)| {
                row.is_none()
                    && crate::text::TextAppearance::from(
                        &theme.typography.annotations.estimate_detail,
                    )
                    .layout(theme.text_layouts.estimate.as_ref())
                    .includes(index, &detail_indices)
            })
        {
            let text = theme.measure_appearance_text(
                NOT_ESTIMABLE_TEXT,
                crate::text::TextAppearance::from(&theme.typography.annotations.estimate_detail)
                    .layout(theme.text_layouts.estimate.as_ref()),
                PhysicalTextAlignment::Center,
                "estimate unavailable category intrinsic",
            )?;
            pitch = pitch.max(text.occupied_size().1);
        }
        if !self.adjacent.is_empty() && self.adjacent.len() != self.estimates.len() {
            return Err(Error::Render(
                "estimate adjacent labels and rows differ".into(),
            ));
        }

        let significance =
            theme.effective_significance(chart.significance_mark_style_override())?;
        let significance_indices = crate::data::source::collect_source(
            self.text_occurrences()
                .filter(|(_, _, significance)| *significance)
                .map(|(index, _, _)| Ok(index)),
        )?;
        for (index, adjacent) in self
            .adjacent
            .iter()
            .enumerate()
            .filter_map(|(index, adjacent)| Some((index, adjacent.as_ref()?)))
        {
            let appearance = if adjacent.1 {
                &significance.label
            } else {
                &theme.typography.annotations.estimate_detail
            };
            let selected = crate::text::TextAppearance {
                theme: appearance,
                exact_override: adjacent.1 && chart.significance_mark_style_override().is_some(),
                layout: if adjacent.1 {
                    theme.text_layouts.significance.as_ref()
                } else {
                    theme.text_layouts.estimate.as_ref()
                },
            };
            if !selected.includes(
                index,
                if adjacent.1 {
                    &significance_indices
                } else {
                    &detail_indices
                },
            ) {
                continue;
            }
            pitch = pitch.max(
                theme
                    .measure_appearance_text(
                        &adjacent.0,
                        selected,
                        PhysicalTextAlignment::Start,
                        "estimate adjacent category height",
                    )?
                    .occupied_size()
                    .1,
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
        data.validate_row_source()?;
        #[cfg(test)]
        count(chart.kind(), 0);
        if chart.kind() != crate::PlotKind::PointInterval {
            return prepare_model(chart, data, annotations)
                .map_err(crate::layout::fit::PlanError::into_error);
        }
        let rows = crate::plots::point_interval::prepare(chart, data)?;

        let range = estimate_data_range(rows.iter().map(|row| &row.estimate));
        let range = match (chart.x_axis_spec().limits, range) {
            (Some(limits), _) => Some(limits),
            (_, Some(range)) => Some(crate::plots::point_interval::point_interval_display_range(
                range,
                chart.x_axis_spec(),
            )?),
            _ => None,
        };
        let mut labels = Vec::new();
        let mut estimates = Vec::new();
        let mut alphas = Vec::new();
        let mut line_alphas = Vec::new();
        crate::data::source::reserve_source(&mut labels, rows.len())?;
        crate::data::source::reserve_source(&mut estimates, rows.len())?;
        crate::data::source::reserve_source(&mut alphas, rows.len())?;
        crate::data::source::reserve_source(&mut line_alphas, rows.len())?;
        for row in rows {
            labels.push(row.label);
            estimates.push(row.estimate);
            alphas.push(row.alpha);
            line_alphas.push(row.line_alpha);
        }
        Ok(Self {
            kind: chart.kind(),
            labels,
            estimates,
            alphas,
            line_alphas,
            adjacent: Vec::new(),
            reference: chart.zero_line_visible().then_some(0.0),
            range,
            family: "point_interval",
            point_interval: true,
            value_presentation: None,
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
    point_style: crate::render::backend::PointStyle,
    interval_style: LineStyle,
    text: Option<crate::render::backend::PlacedText>,
}

#[derive(Debug)]
pub(crate) struct EstimatePayload {
    clip: crate::layout::PanelBox,
    pub(crate) domain_expansions: [Option<crate::layout::interior::DomainExpansion>; 2],
    reference: Option<[Point2D; 2]>,
    reference_style: LineStyle,
    rows: Vec<EstimateRowPayload>,
    #[cfg(test)]
    kind: crate::PlotKind,
}

impl EstimatePayload {
    fn scientific_ink(&self) -> Result<Option<crate::render::geometry::GeometryBox>, Error> {
        let mut full = crate::render::geometry::GeometrySnapshot::new();
        for row in &self.rows {
            if let Some(line) = &row.interval {
                full.line(line, &row.interval_style, None)?;
            }
            for cap in row.caps.iter().flatten() {
                full.line(cap, &row.interval_style, None)?;
            }
            if let Some(point) = &row.point {
                full.scatter_with_clip(std::slice::from_ref(point), &row.point_style, None)?;
            }
        }
        Ok(full
            .bounds()
            .iter()
            .copied()
            .reduce(crate::render::geometry::GeometryBox::union))
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        #[cfg(test)]
        count(self.kind, 3);
        with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.reference {
                backend.draw_line(line, &self.reference_style)?;
            }
            for row in &self.rows {
                if let Some(line) = &row.interval {
                    backend.draw_line(line, &row.interval_style)?;
                }
                for cap in row.caps.iter().flatten() {
                    backend.draw_line(cap, &row.interval_style)?;
                }
                if let Some(point) = &row.point {
                    backend.draw_scatter(std::slice::from_ref(point), &row.point_style)?;
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
    use crate::data::source::collect_source;
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
        return collect_source(estimates.into_iter().enumerate().map(|(index, estimate)| {
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

    collect_source(statuses
        .into_iter()
        .enumerate()
        .map(|(index, status)| {

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

fn plan_not_estimable(
    theme: &ResolvedTheme,
    appearance: &crate::theme::TextTheme,
    layout: &Layout,
    vertical_guide_x: Option<f64>,
    reference_style: &LineStyle,
    scope: &crate::layout::fit::FitScope,
    occurrence: (usize, &[usize]),
) -> Result<(ResolvedTextPlan, f64), crate::layout::fit::PlanError> {
    let text_plan =
        crate::layout::fit::measured(crate::layout::text::request::measure_occurrence(
            theme,
            NOT_ESTIMABLE_TEXT,
            crate::text::TextAppearance::from(appearance)
                .layout(theme.text_layouts.estimate.as_ref()),
            i64::MAX,
            PhysicalTextAlignment::Center,
            "not estimable status",
            scope,
            occurrence,
        ))?;
    if text_plan.lines.is_empty()
        || text_plan.request.as_ref().is_some_and(|request| {
            request.instructions().overlap == Some(crate::TextOverlap::Allow)
                || request
                    .instructions()
                    .overflow
                    .as_ref()
                    .is_some_and(|overflow| overflow.failure != crate::TextFailure::Error)
        })
    {
        return Ok((
            text_plan,
            f64::midpoint(
                layout.projection.panel_box().x0,
                layout.projection.panel_box().x1,
            ),
        ));
    }
    let x = not_estimable_text_x(
        layout.projection.panel_box(),
        text_plan.occupied_size().0,
        vertical_guide_x,
        &theme
            .annotations
            .get_estimates()
            .lengths(appearance.get_font_size())?,
        if vertical_guide_x.is_some() {
            crate::render::geometry::line_ink_width(reference_style)?
        } else {
            0.0
        },
        scope,
    )?;
    Ok((text_plan, x))
}

fn not_estimable_text_x(
    panel: crate::layout::PanelBox,
    text_width: f64,
    vertical_guide_x: Option<f64>,
    spacing: &crate::theme::annotation::EstimateAnnotationLengths,
    guide_ink: f64,
    scope: &crate::layout::fit::FitScope,
) -> Result<f64, crate::layout::fit::PlanError> {
    use crate::layout::kernel::SpatialRounding;
    let left_edge = spacing.panel_inset.after_pixels(panel.x0)?;
    let right_edge = spacing.panel_inset.before_pixels(panel.x1)?;
    let required = spacing
        .panel_inset
        .times(2)
        .add_pixels(text_width)?
        .pixels_rounded(SpatialRounding::Ceil)?;
    if required > panel.width() {
        return Err(scope.failure(crate::layout::fit::FitKind::Text, format!(
            "Not estimable row text requires {text_width:.1}px but the chart panel provides {:.1}px after padding; increase chart width",
            (right_edge - left_edge).max(0.0)
        )).into());
    }
    let Some(guide_x) = vertical_guide_x.filter(|_| guide_ink > 0.0) else {
        return Ok(f64::midpoint(left_edge, right_edge));
    };
    let clearance = spacing.guide_clearance.add_pixels(guide_ink / 2.0)?;
    let left_right = clearance.before_pixels(guide_x)?;
    let right_left = clearance.after_pixels(guide_x)?;
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
            AxisScale::Log10 if value > 0.0 => crate::numeric::log10(value),
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
    let alpha = chart.prepare_alpha(AlphaTarget::Point, AlphaUnitKind::Observation, &[])?;
    let alphas = crate::data::source::collect_source(
        (0..data.height()).map(|row| ObservationAlpha::resolve(data, &alpha, row)),
    )?;
    let line_alpha = chart.prepare_alpha(AlphaTarget::Line, AlphaUnitKind::Observation, &[])?;
    let line_alphas = crate::data::source::collect_source(
        (0..data.height()).map(|row| ObservationAlpha::resolve(data, &line_alpha, row)),
    )?;
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
    let mut coefficient_significance =
        crate::data::source::collect_source((0..labels.len()).map(|_| Ok(None)))?;
    for annotation in annotations {
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
        coefficient_significance[index] = Some(crate::data::source::copy_source_string(
            annotation.mark().glyph()?,
        )?);
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
            AxisScale::Log10 if value > 0.0 => crate::numeric::log10(value),
            AxisScale::Log10 => Err(Error::Data(format!(
                "{family}: logarithmic scale requires strictly positive estimates, bounds, and reference values"
            ))),
        }
    };
    let transformed_estimates =
        crate::data::source::collect_source(estimates.iter().map(|estimate| {
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
                )
            })
            .transpose()?
    };
    let value_presentation = chart
        .estimate_labels_spec()
        .map(|format| -> Result<_, Error> {
            let format = crate::data::source::source_arc(format.try_copy()?)?;
            let source = crate::numeric::source::NumericSource::for_data(chart, data)?;
            let values = crate::numeric::source::collect_values(
                estimates.iter().flatten().count(),
                estimates
                    .iter()
                    .zip(&labels)
                    .filter_map(|(estimate, label)| {
                        estimate.as_ref().map(|estimate| (estimate, label))
                    })
                    .map(|(estimate, label)| {
                        Ok(crate::numeric::context::ContextValue {
                            operand: crate::numeric::operand::Operand::number(
                                estimate.estimate,
                                false,
                            ),
                            identity: crate::numeric::context::Identity::Estimate(
                                crate::data::source::copy_source_string(label)?,
                            ),
                        })
                    }),
            )?;
            let info = source.context(
                crate::NumericRole::CoefficientLabels,
                crate::NumericPurpose::ActualSet,
                values,
                None,
            )?;
            let set = format
                .materialize(&source.operation, info, false)?
                .ok_or_else(|| {
                    Error::Data(
                        "Coefficient estimate labels are not unique at configured precision".into(),
                    )
                })?;
            crate::data::source::source_arc(crate::numeric::set::NumericPresentation {
                format: Some(format),
                set,
            })
        })
        .transpose()?;
    let mut selected = value_presentation
        .as_ref()
        .map(|presentation| presentation.set.labels.iter());
    let adjacent =
        crate::data::source::collect_source(estimates.iter().zip(coefficient_significance).map(
            |(estimate, mark)| {
                if estimate.is_some() && value_presentation.is_some() {
                    let label = selected.as_mut().and_then(Iterator::next).ok_or_else(|| {
                        Error::Render("prepared estimate numeric source missing".into())
                    })?;
                    Ok(Some((
                        crate::data::source::copy_source_string(label)?,
                        false,
                    )))
                } else {
                    Ok(mark.map(|mark| (mark, true)))
                }
            },
        ))?;
    Ok(PreparedEstimate {
        kind: chart.kind(),
        labels,
        estimates: transformed_estimates,
        alphas,
        line_alphas,
        adjacent,
        reference: Some(transformed_reference),
        range,
        family,
        point_interval: false,
        value_presentation,
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

    project(chart, prepared, layout, context)
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
    let significance = theme.effective_significance(chart.significance_mark_style_override())?;
    let reference_style = if prepared.point_interval {
        theme
            .marks
            .get_strokes()
            .get_references()
            .get_point_interval_reference()
            .backend_style()
    } else {
        theme
            .marks
            .get_strokes()
            .get_references()
            .get_estimate_reference()
            .backend_style()
    };
    let detail_role = &theme.typography.annotations.estimate_detail;
    let detail_label = crate::theme::TextTheme::new(
        crate::data::source::copy_font_stack(detail_role.get_stack())?,
        detail_role.get_color(),
        detail_role.get_font_size(),
        detail_role.get_weight(),
        detail_role.get_line_height(),
    )?;

    let detail_indices = crate::data::source::collect_source(
        prepared
            .text_occurrences()
            .filter(|(_, _, significance)| !*significance)
            .map(|(index, _, _)| Ok(index)),
    )?;
    let significance_indices = crate::data::source::collect_source(
        prepared
            .text_occurrences()
            .filter(|(_, _, significance)| *significance)
            .map(|(index, _, _)| Ok(index)),
    )?;
    let adjacent_label_plans = crate::layout::fit::collect( "estimate adjacent labels", (0..labels.len())
        .map(|index| {
            let Some(tuple) = estimates[index] else {
                return Ok(None);
            };
            let Some((source, is_significance)) = prepared.adjacent.get(index).and_then(Option::as_ref) else {
                return Ok(None);
            };
            let is_significance = *is_significance;
            let scope = context.fit_scope().child(if is_significance { crate::layout::AddressRole::Annotation } else { crate::layout::AddressRole::CategoricalValueLabel }, index)?;
            let (x, y) = layout
                .projection
                .project(tuple.estimate, row_count - index as f64 - 0.5);
            let (appearance, gap, owner, exact_override) = if is_significance {
                (
                    &significance.label,
                    significance.gap_pixels()?,
                    "coefficient attached significance label",
                    significance.exact_override,
                )
            } else {
                (&detail_label, theme.annotations.get_estimates().get_numeric_label_gap()
                    .product(crate::theme::LocalMetric::css_pixels(detail_label.get_font_size())?)
                    .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?,
                    "coefficient estimate label", false)
            };
            let panel = layout.projection.panel_box();
            let (position, anchor, plan) = {
                let measure = |alignment| {
                    crate::layout::fit::measured(crate::layout::text::request::measure_occurrence(theme,source,crate::text::TextAppearance {theme:appearance,exact_override,layout:if is_significance {theme.text_layouts.significance.as_ref()} else {theme.text_layouts.estimate.as_ref()}},i64::MAX,alignment,owner,&scope,(index,if is_significance {&significance_indices} else {&detail_indices})))
                };
                let start_plan = measure(PhysicalTextAlignment::Start)?;
                let width = start_plan.occupied_size().0;
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
                } else if chart.user_x_limits().is_none() || start_plan.request.as_ref().and_then(|request|request.instructions().overflow.as_ref()).is_some_and(|overflow|overflow.failure!=crate::TextFailure::Error) {
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
        reference_style,
        rows: Vec::new(),
        #[cfg(test)]
        kind: chart.kind(),
    };
    let point_style = chart.point_style(
        theme,
        crate::theme::AccentColors::resolve(theme.marks.get_accents(), 1)?.color(0)?,
        theme.marks.get_points().get_radii().get_interval_estimate(),
    );
    crate::alpha::marks::require_aligned_points(estimates.len(), prepared.alphas.len())?;
    crate::alpha::marks::require_aligned_points(estimates.len(), prepared.line_alphas.len())?;
    crate::data::source::reserve_source(&mut payload.rows, estimates.len())?;
    let mut deferred_annotation_text = Vec::new();
    for (index, adjacent) in adjacent_label_plans.into_iter().enumerate() {
        let y = row_count - index as f64 - 0.5;
        let mut row = EstimateRowPayload {
            interval: None,
            caps: [None, None],
            point: None,
            point_style: prepared.alphas[index].point(point_style.clone()),
            interval_style: line_style(
                prepared.line_alphas[index].value,
                theme
                    .marks
                    .get_strokes()
                    .get_summaries()
                    .get_estimate_interval()
                    .backend_style(),
            ),
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
                            let half = f64::from(
                                theme
                                    .marks
                                    .get_extents()
                                    .get_point_interval_cap_half_height(),
                            );
                            row.caps[index] =
                                Some([Point2D::new(x, py - half), Point2D::new(x, py + half)]);
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
                    crate::data::source::reserve_source(&mut deferred_annotation_text, 1)?;
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
                )?);
            }
        } else {
            let (plan, x) = plan_not_estimable(
                theme,
                &detail_label,
                &layout,
                reference_line_x,
                &payload.reference_style,
                &context
                    .fit_scope()
                    .child(crate::layout::AddressRole::CategoricalValueLabel, index)?,
                (index, &detail_indices),
            )?;
            let (_, py) = layout.projection.project(x_range.0, y);
            row.text = Some(crate::render::backend::PlacedText::middle(
                plan,
                Point2D::new(x.round(), py.round()),
                TextAnchor::Middle,
                0.0,
            )?);
        }
        payload.rows.push(row);
    }
    let clip = crate::render::backend::validate_clip_region(payload.clip)?.panel();
    for is_significance in [false, true] {
        let indices = crate::data::source::collect_source(
            prepared
                .text_occurrences()
                .filter(|(_, _, significance)| *significance == is_significance)
                .map(|(index, _, _)| Ok(index)),
        )?;
        let mut labels = crate::data::source::collect_source(indices.iter().map(|&index| {
            Ok(payload.rows[index]
                .text
                .as_ref()
                .expect("prepared text occurrence")
                .clone())
        }))?;
        crate::layout::text::font_fit::uniform_placed(theme, &mut labels, context.fit_scope())?;
        crate::layout::text::visibility::apply(
            &mut labels,
            &indices,
            clip,
            true,
            context.fit_scope(),
        )?;
        crate::layout::text::visibility::require_nonoverlap(&mut labels, context.fit_scope())?;
        for (&index, text) in indices.iter().zip(labels) {
            payload.rows[index].text = Some(text);
        }
    }
    let mut label_ink = crate::render::geometry::GeometrySnapshot::new();
    let mut label_source: Option<(f64, f64)> = None;

    for (index, row) in payload.rows.iter_mut().enumerate() {
        let Some(text) = &mut row.text else { continue };
        let bounds = crate::layout::text::placement::footprint(text.placement, &text.plan)?;
        let region = crate::layout::PanelBox::new(
            if chart.user_x_limits().is_some() || estimates[index].is_none() {
                clip.x0
            } else {
                clip.x0.min(bounds.x0)
            },
            if chart.user_x_limits().is_some() || estimates[index].is_none() {
                clip.x1
            } else {
                clip.x1.max(bounds.x1)
            },
            clip.y0,
            clip.y1,
        );
        text.plan = crate::layout::text::placement::contain(
            text.plan.clone(),
            text.placement,
            region,
            &context
                .fit_scope()
                .child(crate::layout::AddressRole::CategoricalValueLabel, index)?,
        )?;
        let mut ink = crate::render::geometry::GeometrySnapshot::new();
        ink.text(text.placement, &text.plan, None)?;
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
                context.fit_scope(),
            )?;
        }
        if let (Some(numeric), Some(bounds)) =
            (estimate_data_range(estimates), payload.scientific_ink()?)
        {
            let requests = crate::layout::interior::automatic_absolute_ink_expansions(
                [numeric, (0.0, row_count)],
                &layout.projection,
                bounds,
                [true, false],
                context.fit_scope(),
            )?;
            payload.domain_expansions = crate::layout::interior::merge_mark_domain_expansions(
                payload.domain_expansions,
                requests,
            );
        }
    }
    let mut geometry = crate::render::geometry::GeometrySnapshot::new();
    if let Some(line) = &payload.reference {
        geometry.line(line, &payload.reference_style, Some(clip))?;
    }
    for row in &payload.rows {
        if let Some(line) = &row.interval {
            geometry.line(line, &row.interval_style, Some(clip))?;
        }
        for cap in row.caps.iter().flatten() {
            geometry.line(cap, &row.interval_style, Some(clip))?;
        }
        if let Some(point) = &row.point {
            geometry.scatter(std::slice::from_ref(point), &row.point_style, clip)?;
        }
        if let Some(text) = &row.text {
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
    }
    #[cfg(test)]
    count(chart.kind(), 2);
    let mut result = PlotRenderResult::new(layout, chart.prepared_discrete_guide()?)
        .with_deferred_annotation_text(deferred_annotation_text);
    result
        .numeric_presentation
        .clone_from(&prepared.value_presentation);
    result.y_domain = crate::spec::AxisDomain::Categorical;
    result.marks = Some(crate::data::source::source_arc(
        crate::layout::interior::SelectedMarkPlan {
            payload: crate::layout::interior::MarkPayload::Estimate(payload),
            geometry,
        },
    )?);
    Ok(result)
}

pub(crate) fn log_tick_plan(
    range: (f64, f64),
    spec: &crate::layout::AxisSpec,
    panel: crate::layout::PanelBox,
    theme: &ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::NumericTickPlan, crate::layout::fit::PlanError> {
    let projection = crate::layout::Projection::new(range, (0.0, 1.0), panel.as_tuple());
    let format = spec.numeric_request()?;
    let raw = spec
        .numeric_source
        .as_deref()
        .and_then(|source| source.raw_limits(crate::layout::AxisSide::Bottom));
    crate::layout::NumericTickPlan::measure_selected_source(
        range,
        spec.generation.clone(),
        true,
        &projection,
        crate::layout::AxisSide::Bottom,
        theme,
        format.as_ref(),
        scope,
        raw,
    )
}
