//! Continuous-color content inside the single sectioned guide envelope.

#[cfg(test)]
mod tests;

use super::descriptor::ContinuousDescriptor;
use super::{LegendCall, LegendMetrics};
use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::layout::kernel::{SpatialRounding, TrackSpec};
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{self, FillStyle, GradientStop, PlacedText, Point2D, TextAnchor};
use crate::theme::spacing::ExactLength;
use crate::theme::ResolvedTheme;
use crate::Error;

fn hard(value: &ExactLength) -> Result<i64, Error> {
    Ok(value.round(SpatialRounding::Ceil)?.quanta())
}

fn centered_text(plan: ResolvedTextPlan, x: f64, y: f64) -> PlacedText {
    PlacedText::middle(plan, Point2D::new(x, y), TextAnchor::Middle, 0.0)
}

fn projected_minimum(
    lengths: &[f64],
    fractions: &[f64],
    gap: &ExactLength,
    traced: &ExactLength,
) -> Result<ExactLength, Error> {
    let mut required = traced.times(1);
    if let [half_extent] = lengths {
        let candidate = gap.times(2).add_pixels(2.0 * half_extent)?;
        if required.compare(&candidate).is_lt() {
            required = candidate;
        }
        return Ok(required);
    }
    for (extent, fraction) in lengths.windows(2).zip(fractions.windows(2)) {
        let candidate = gap
            .add_pixels(extent[0])?
            .add_pixels(extent[1])?
            .divided_by(fraction[1] - fraction[0])?;
        if required.compare(&candidate).is_lt() {
            required = candidate;
        }
    }
    Ok(required)
}

pub(super) fn intrinsic_width(
    descriptor: &ContinuousDescriptor,
    theme: &ResolvedTheme,
    metrics: &LegendMetrics,
) -> Result<ExactLength, Error> {
    let mut width = ExactLength::pixels(0.0)?;
    for label in descriptor
        .numeric
        .labels
        .iter()
        .map(String::as_str)
        .chain(descriptor.missing.iter().map(|key| key.label.as_str()))
    {
        let plan = theme.measure_text(
            label,
            &theme.typography.guides.legend_label,
            PhysicalTextAlignment::Start,
            "continuous guide value width",
            false,
        )?;
        let column = metrics
            .exact
            .key_width
            .add_length(&metrics.exact.key_label_gap)
            .add_pixels(plan.width().max(plan.fit_width()))?;
        if width.compare(&column).is_lt() {
            width = column;
        }
    }
    if let Some(text) = &descriptor.numeric.explanation {
        let candidate = ExactLength::pixels(crate::numeric::slot::preferred_width(
            text,
            &theme.typography.guides.legend_label,
            theme,
        )?)?;
        if width.compare(&candidate).is_lt() {
            width = candidate;
        }
    }
    Ok(width)
}

/// Shaped body operands before the common finite section allocation. It owns no
/// title, frame, outer padding, placement trial or measurement cache.
pub(super) struct ColorContent {
    labels: Vec<ResolvedTextPlan>,
    missing: Option<ResolvedTextPlan>,
    along: Vec<f64>,
    fractions: Vec<f64>,
    required: f64,
    label_height: f64,
    label_width: f64,
    missing_height: ExactLength,
    missing_width: ExactLength,
    missing_inset: f64,
    horizontal: bool,
    explanation: Option<crate::numeric::slot::ExplanationPlan>,
    explanation_height: ExactLength,
}

impl ColorContent {
    #[expect(
        clippy::too_many_lines,
        reason = "one color body retains truthful numeric positions and actual shaped extents before allocation"
    )]
    pub(super) fn prepare(
        descriptor: &ContinuousDescriptor,
        width: i64,
        horizontal: bool,
        theme: &ResolvedTheme,
        metrics: &LegendMetrics,
        scope: &FitScope,
    ) -> Result<Self, PlanError> {
        let mut labels = Vec::new();
        fit::reserve(
            &mut labels,
            descriptor.numeric.labels.len(),
            "continuous guide labels",
        )?;
        for (index, label) in descriptor.numeric.labels.iter().enumerate() {
            let plan = theme.measure_legacy_appearance_text(
                label,
                &theme.typography.guides.legend_label,
                PhysicalTextAlignment::Center,
                "continuous guide value",
                false,
            )?;
            if plan.width().max(plan.fit_width()) > super::measurement::pixels(width) {
                return Err(scope
                    .child(AddressRole::GuideLabel, index)?
                    .failure(
                        FitKind::Text,
                        "continuous guide number exceeds its assigned width",
                    )
                    .into());
            }
            labels.push(plan);
        }
        if !matches!(labels.len(), 1..=3) {
            return Err(Error::Render(
                "continuous guide must have one to three prepared labels".into(),
            )
            .into());
        }
        let missing = descriptor
            .missing
            .as_ref()
            .map(|key| {
                theme.measure_legacy_appearance_text(
                    &key.label,
                    &theme.typography.guides.legend_label,
                    PhysicalTextAlignment::Start,
                    "continuous guide missing key",
                    false,
                )
            })
            .transpose()?;
        let missing_inset = descriptor
            .missing
            .as_ref()
            .and_then(|key| key.outline.as_ref())
            .map_or(0.0, |outline| f64::from(outline.get_width()) / 2.0);
        let missing_height = if let Some(plan) = &missing {
            let key = metrics.exact.key_width.add_pixels(missing_inset)?;
            let text = ExactLength::pixels(plan.height())?;
            if key.compare(&text).is_lt() {
                text
            } else {
                key
            }
            .add_length(&metrics.exact.row_gap)
        } else {
            ExactLength::pixels(0.0)?
        };
        let missing_width = if let Some(plan) = &missing {
            metrics
                .exact
                .key_width
                .add_length(&metrics.exact.key_label_gap)
                .add_pixels(plan.width().max(plan.fit_width()))?
        } else {
            ExactLength::pixels(0.0)?
        };
        let mut along = Vec::new();
        fit::reserve(
            &mut along,
            labels.len(),
            "continuous guide projected extents",
        )?;
        for plan in &labels {
            let placed = centered_text(plan.clone(), 0.0, 0.0);
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
            ink.text(placed.placement, &placed.plan, None)?;
            let painted = ink
                .bounds()
                .iter()
                .map(|bounds| {
                    if horizontal {
                        (-bounds.left).max(bounds.right)
                    } else {
                        (-bounds.bottom).max(bounds.top)
                    }
                })
                .fold(0.0_f64, f64::max);
            let half_box = if horizontal {
                plan.width().max(plan.fit_width())
            } else {
                plan.height()
            } / 2.0;
            along.push(half_box.max(painted));
        }
        let mut fractions = Vec::new();
        fit::reserve(
            &mut fractions,
            labels.len(),
            "continuous guide numeric positions",
        )?;
        match labels.len() {
            1 => fractions.push(0.5),
            2 => fractions.extend_from_slice(&[0.0, 1.0]),
            _ => {
                let center = descriptor.center.ok_or_else(|| {
                    Error::Render("three-label continuous guide has no center".into())
                })?;
                fractions.extend_from_slice(&[
                    0.0,
                    crate::numeric::mapping::normalized_fraction(descriptor.domain, center)?,
                    1.0,
                ]);
            }
        }
        if fractions.windows(2).any(|pair| pair[1] <= pair[0]) {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "continuous guide labels have no positive projected separation",
                )
                .into());
        }
        let required = projected_minimum(
            &along,
            &fractions,
            &metrics.exact.row_gap.add_pixels(2.0 / 1024.0)?,
            &metrics.exact.minimum,
        )?
        .pixels_rounded(SpatialRounding::Ceil)?
        .max(metrics.minimum);
        let label_height = labels
            .iter()
            .map(ResolvedTextPlan::height)
            .fold(0.0_f64, f64::max);
        let label_width = labels
            .iter()
            .map(|plan| plan.width().max(plan.fit_width()))
            .fold(0.0_f64, f64::max);
        let explanation = descriptor
            .numeric
            .explanation
            .as_ref()
            .map(|text| {
                crate::numeric::slot::ExplanationPlan::measure(
                    text,
                    &theme.typography.guides.legend_label,
                    super::measurement::pixels(width),
                    crate::TextAlignment::Start,
                    theme,
                    scope,
                )
            })
            .transpose()?;
        let explanation_height = if let Some(plan) = &explanation {
            metrics.exact.row_gap.add_pixels(plan.height)?
        } else {
            ExactLength::pixels(0.0)?
        };
        Ok(Self {
            labels,
            missing,
            along,
            fractions,
            required,
            label_height,
            label_width,
            missing_height,
            missing_width,
            missing_inset,
            horizontal,
            explanation,
            explanation_height,
        })
    }

    fn body_height(&self, length: f64, metrics: &LegendMetrics) -> Result<ExactLength, Error> {
        let body = if self.horizontal {
            metrics
                .exact
                .key_width
                .add_length(&metrics.exact.row_gap)
                .add_pixels(self.label_height)?
        } else {
            ExactLength::pixels(length)?
                .add_pixels(self.along[0])?
                .add_pixels(self.along[self.along.len() - 1])?
        };
        Ok(body
            .add_length(&self.missing_height)
            .add_length(&self.explanation_height))
    }

    pub(super) fn track(
        &self,
        title: &ExactLength,
        metrics: &LegendMetrics,
    ) -> Result<TrackSpec, Error> {
        let minimum = hard(&title.add_length(&self.body_height(self.required, metrics)?))?;
        let preferred = hard(
            &title.add_length(&self.body_height(self.required.max(metrics.preferred), metrics)?),
        )?;
        Ok(TrackSpec {
            min: minimum,
            preferred,
            max: Some(preferred),
            flexible: true,
        })
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one assigned color body consumes shared metrics and finite section capacity without an outer owner"
    )]
    pub(super) fn assign(
        self,
        descriptor: &ContinuousDescriptor,
        width: i64,
        height: Option<&ExactLength>,
        compact: bool,
        metrics: &LegendMetrics,
        compact_changes: &mut bool,
        scope: &FitScope,
    ) -> Result<MeasuredColorBody, PlanError> {
        let pixels = super::measurement::pixels;
        let first = self.along[0];
        let last = self.along[self.along.len() - 1];
        let maximum = if self.horizontal {
            pixels(width) - first - last
        } else if let Some(height) = height {
            height
                .subtract(
                    &self
                        .missing_height
                        .add_length(&self.explanation_height)
                        .add_pixels(first)?
                        .add_pixels(last)?,
                )
                .map(|length| length.pixels_rounded(SpatialRounding::Floor))
                .transpose()?
                .unwrap_or(0.0)
        } else {
            metrics.preferred.max(self.required)
        };
        if maximum < self.required {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "continuous guide gradient is shorter than its projected label minimum",
                )
                .into());
        }
        let preferred = metrics.preferred.min(maximum).max(self.required);
        if !compact {
            *compact_changes |= preferred > self.required;
        }
        let length = if compact { self.required } else { preferred };
        let along_width = if self.horizontal {
            ExactLength::pixels(length)?
                .add_pixels(first)?
                .add_pixels(last)?
        } else {
            metrics
                .exact
                .key_width
                .add_length(&metrics.exact.key_label_gap)
                .add_pixels(self.label_width)?
        };
        let content_width = if along_width.compare(&self.missing_width).is_lt() {
            &self.missing_width
        } else {
            &along_width
        };
        let content_width = if self.explanation.is_some() {
            ExactLength::pixels(pixels(width))?
        } else {
            content_width.times(1)
        };
        let occupied_width = hard(&content_width)?;
        let occupied_height = self.body_height(length, metrics)?;
        if occupied_width > width
            || height.is_some_and(|height| height.compare(&occupied_height).is_lt())
        {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    format!("continuous color body exceeds its assigned section: width {width} quanta requires {occupied_width}, height {:?} quanta requires {}",
                        height.map(hard).transpose()?, hard(&occupied_height)?),
                )
                .into());
        }
        let thickness = metrics.handle_w;
        let (gradient, start, end) = if self.horizontal {
            (
                PanelBox::new(first, first + length, -thickness, 0.0),
                Point2D::new(first, -thickness / 2.0),
                Point2D::new(first + length, -thickness / 2.0),
            )
        } else {
            (
                PanelBox::new(0.0, thickness, -last - length, -last),
                Point2D::new(thickness / 2.0, -last - length),
                Point2D::new(thickness / 2.0, -last),
            )
        };
        let stops = crate::data::source::collect_source(descriptor.stops.iter().copied().map(Ok))?;
        backend::validate_linear_gradient(gradient, start, end, &stops)?;
        let label_x = metrics
            .exact
            .key_width
            .add_length(&metrics.exact.key_label_gap)
            .pixels_rounded(SpatialRounding::NearestEven)?;
        let row_gap = metrics
            .exact
            .row_gap
            .pixels_rounded(SpatialRounding::NearestEven)?;
        let mut labels = Vec::new();
        fit::reserve(
            &mut labels,
            self.labels.len(),
            "placed continuous guide labels",
        )?;
        for (plan, fraction) in self.labels.into_iter().zip(self.fractions) {
            let (x, y) = if self.horizontal {
                (
                    gradient.x0 + length * fraction,
                    -thickness - row_gap - self.label_height / 2.0,
                )
            } else {
                (
                    label_x + plan.width() / 2.0,
                    gradient.y0 + length * fraction,
                )
            };
            labels.push(centered_text(plan, x, y));
        }
        let missing = self
            .missing
            .zip(descriptor.missing.as_ref())
            .map(|(plan, key)| {
                let bottom = -occupied_height.pixels_rounded(SpatialRounding::NearestEven)?
                    + self
                        .explanation_height
                        .pixels_rounded(SpatialRounding::NearestEven)?;
                let row_height = plan.height().max(thickness + self.missing_inset);
                Ok::<_, Error>((
                    PanelBox::new(0.0, thickness, bottom, bottom + thickness),
                    crate::theme::FixedFillTheme::new(key.fill, key.outline.clone())?
                        .backend_style(),
                    PlacedText::middle(
                        plan,
                        Point2D::new(label_x, bottom + row_height / 2.0),
                        TextAnchor::Start,
                        0.0,
                    ),
                ))
            })
            .transpose()?;
        let explanation = self
            .explanation
            .map(|plan| {
                let top =
                    -occupied_height.pixels_rounded(SpatialRounding::NearestEven)? + plan.height;
                Ok::<_, Error>(plan.at(0.0, top))
            })
            .transpose()?;
        let measured = MeasuredColorBody {
            width: content_width,
            height: occupied_height,
            gradient,
            start,
            end,
            stops,
            labels,
            missing,
            explanation,
        };
        if !measured.labels_have_clearance(self.horizontal, metrics.row_gap)? {
            return Err(scope
                .failure(
                    FitKind::LegendOrGuide,
                    "continuous guide projected label ink violates its traced adjacent gap",
                )
                .into());
        }
        Ok(measured)
    }
}

/// Frozen content relative to the section body's top-left; y decreases down.
#[derive(Debug)]
pub(super) struct MeasuredColorBody {
    pub(super) width: ExactLength,
    pub(super) height: ExactLength,
    pub(super) gradient: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: Vec<GradientStop>,
    labels: Vec<PlacedText>,
    missing: Option<(PanelBox, FillStyle, PlacedText)>,
    explanation: Option<PlacedText>,
}

impl MeasuredColorBody {
    fn labels_have_clearance(&self, horizontal: bool, gap: f64) -> Result<bool, Error> {
        let mut previous = None;
        for text in &self.labels {
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
            ink.text(text.placement, &text.plan, None)?;
            if let Some(bounds) = ink
                .bounds()
                .iter()
                .copied()
                .reduce(crate::render::geometry::GeometryBox::union)
            {
                let (start, end) = if horizontal {
                    (bounds.left, bounds.right)
                } else {
                    (bounds.bottom, bounds.top)
                };
                if previous.is_some_and(|previous| start - previous < gap) {
                    return Ok(false);
                }
                previous = Some(end);
            }
        }
        Ok(true)
    }

    pub(super) fn visit_calls(
        &self,
        x: f64,
        y: f64,
        mut emit: impl FnMut(LegendCall) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let translate = |point: Point2D| Point2D::new(point.x + x, point.y + y);
        let text = |text: &PlacedText| {
            let mut text = text.clone();
            text.placement.position = translate(text.placement.position);
            LegendCall::Text(text)
        };
        let bounds = self.gradient;
        let gradient = PanelBox::new(bounds.x0 + x, bounds.x1 + x, bounds.y0 + y, bounds.y1 + y);
        let stops = crate::data::source::collect_source(self.stops.iter().copied().map(Ok))?;
        emit(LegendCall::Gradient(
            gradient,
            translate(self.start),
            translate(self.end),
            stops,
        ))?;
        for label in &self.labels {
            emit(text(label))?;
        }
        if let Some(explanation) = &self.explanation {
            emit(text(explanation))?;
        }
        if let Some((bounds, fill, label)) = &self.missing {
            emit(LegendCall::Fill(
                [
                    translate(Point2D::new(bounds.x0, bounds.y0)),
                    translate(Point2D::new(bounds.x1, bounds.y0)),
                    translate(Point2D::new(bounds.x1, bounds.y1)),
                    translate(Point2D::new(bounds.x0, bounds.y1)),
                ],
                fill.clone(),
            ))?;
            emit(text(label))?;
        }
        Ok(())
    }

    pub(super) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        for value in [
            self.gradient.x0,
            self.gradient.y0,
            self.gradient.x1,
            self.gradient.y1,
            self.start.x,
            self.start.y,
            self.end.x,
            self.end.y,
        ] {
            round.observe_number(value)?;
        }
        for label in &self.labels {
            round.observe_text(&label.plan)?;
            round.observe_placement(label.placement)?;
        }
        round.observe_count(usize::from(self.explanation.is_some()))?;
        if let Some(explanation) = &self.explanation {
            round.observe_text(&explanation.plan)?;
            round.observe_placement(explanation.placement)?;
        }
        round.observe_count(self.stops.len())?;
        for stop in &self.stops {
            round.observe_number(stop.offset)?;
            for channel in [stop.color.0, stop.color.1, stop.color.2] {
                round.observe_count(usize::from(channel))?;
            }
        }
        round.observe_count(usize::from(self.missing.is_some()))?;
        if let Some((bounds, fill, label)) = &self.missing {
            round.observe_box(*bounds)?;
            round.observe_legend_glyph(None, None, Some(fill))?;
            round.observe_text(&label.plan)?;
            round.observe_placement(label.placement)?;
        }
        Ok(())
    }
}
