//! One assigned inline guide from a font-independent continuous descriptor.

use super::descriptor::ContinuousDescriptor;
use super::{LegendCall, LegendDrawPlan, LegendMetrics};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{
    self, FillStyle, GradientStop, PlacedText, PlannedTextPlacement, Point2D, TextAnchor,
};
use crate::theme::spacing::ExactLength;
use crate::theme::ResolvedTheme;
use crate::Error;

#[derive(Debug)]
pub(crate) struct MeasuredContinuousGuide {
    pub(crate) width: f64,
    pub(crate) height: f64,
    metrics: LegendMetrics,
    gradient: PanelBox,
    start: Point2D,
    end: Point2D,
    stops: Vec<GradientStop>,
    title: PlacedText,
    labels: Vec<PlacedText>,
    missing: Option<(PanelBox, FillStyle, PlacedText)>,
    format: Option<std::sync::Arc<crate::numeric::format::NumericFormat>>,
    members: Vec<u64>,
    precision: u8,
}

/// Canonical consequences of complete exact vertical-guide width requirements.
pub(crate) struct ContinuousGuideWidths {
    pub(crate) title_minimum: f64,
    pub(crate) minimum: f64,
    pub(crate) preferred: f64,
    pub(crate) outside_gap: f64,
}

fn centered_text(plan: crate::layout::ResolvedTextPlan, x: f64, y: f64) -> PlacedText {
    PlacedText::middle(plan, Point2D::new(x, y), TextAnchor::Middle, 0.0)
}

fn projected_minimum(
    lengths: &[f64],
    fractions: &[f64],
    gap: &crate::theme::spacing::ExactLength,
    traced: &crate::theme::spacing::ExactLength,
) -> Result<crate::theme::spacing::ExactLength, Error> {
    let mut required = traced.times(1);
    if let [half_extent] = lengths {
        let candidate = gap.times(2).add_pixels(2.0 * half_extent)?;
        if required.compare(&candidate) == std::cmp::Ordering::Less {
            required = candidate;
        }
        return Ok(required);
    }
    for (extent, fraction) in lengths.windows(2).zip(fractions.windows(2)) {
        let candidate = gap
            .add_pixels(extent[0])?
            .add_pixels(extent[1])?
            .divided_by(fraction[1] - fraction[0])?;
        if required.compare(&candidate) == std::cmp::Ordering::Less {
            required = candidate;
        }
    }
    Ok(required)
}

impl MeasuredContinuousGuide {
    /// The measured guide and all containing width owners use the same complete
    /// padding+key+gap+label requirements, before a single hard boundary.
    pub(crate) fn vertical_widths(
        descriptor: &ContinuousDescriptor,
        theme: &ResolvedTheme,
    ) -> Result<ContinuousGuideWidths, Error> {
        let (metrics, frame_minimum) = Self::allocation_metrics(descriptor, theme)?;

        let (hard_title, _) = fit::intrinsic_widths(
            theme,
            &descriptor.title,
            &theme.typography.guides.legend_title,
            "continuous guide title",
        )?;
        let title = theme.measure_text(
            &descriptor.title,
            &theme.typography.guides.legend_title,
            PhysicalTextAlignment::Start,
            "continuous guide title width",
            false,
        )?;
        let mut hard = ExactLength::pixels(hard_title)?;
        let mut preferred =
            ExactLength::pixels(hard_title.max(title.width().max(title.fit_width())))?;
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
            if hard.compare(&column) == std::cmp::Ordering::Less {
                hard = column.times(1);
            }
            if preferred.compare(&column) == std::cmp::Ordering::Less {
                preferred = column;
            }
        }
        let minimum = metrics.outer_width(&hard)?.max(frame_minimum);
        Ok(ContinuousGuideWidths {
            title_minimum: metrics.outer_width(&ExactLength::pixels(hard_title)?)?,
            minimum,
            preferred: metrics.outer_width(&preferred)?.max(minimum),
            outside_gap: metrics.outside_gap,
        })
    }
    /// Effective content chrome and the frame's minimum square extent, shared
    /// with containing owners' intrinsic width proposals.
    pub(crate) fn allocation_metrics(
        descriptor: &ContinuousDescriptor,
        theme: &ResolvedTheme,
    ) -> Result<(LegendMetrics, f64), Error> {
        let mut metrics = LegendMetrics::from_theme(theme)?;
        let missing_inset = descriptor
            .missing
            .as_ref()
            .and_then(|key| key.outline.as_ref())
            .map_or(0.0, |outline| f64::from(outline.width()) / 2.0);
        let frame_minimum = metrics.allocation_insets(missing_inset)?;
        metrics.ensure_key_gap(missing_inset)?;
        Ok((metrics, frame_minimum))
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one assigned orientation and flow retains exact guide text and gradient operands"
    )]
    pub(crate) fn measure(
        descriptor: &ContinuousDescriptor,
        available: (f64, impl Into<Option<f64>>),
        horizontal: bool,
        compact: bool,
        theme: &ResolvedTheme,
        compact_changes: &mut bool,
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        theme.prepare_font_environment()?;
        let available = (available.0, available.1.into());
        if !available.0.is_finite() || available.1.is_some_and(|height| !height.is_finite()) {
            return Err(Error::Render(
                "nonfinite continuous guide assignment".into(),
            ));
        }
        let (metrics, frame_minimum) = Self::allocation_metrics(descriptor, theme)?;

        let missing_inset = descriptor
            .missing
            .as_ref()
            .and_then(|key| key.outline.as_ref())
            .map_or(0.0, |outline| f64::from(outline.width()) / 2.0);
        let inset = metrics.edge_reserve(false);
        let inner_width = available.0 - LegendMetrics::hard(&inset)?;
        let inner_height = available
            .1
            .map(|height| Ok::<_, Error>(height - LegendMetrics::hard(&inset)?))
            .transpose()?;
        if inner_width <= 0.0 || inner_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide has no positive content extent",
            )));
        }
        let title = match fit::text(
            theme,
            &descriptor.title,
            &theme.typography.guides.legend_title,
            inner_width,
            PhysicalTextAlignment::Start,
            "continuous guide title",
            &scope.child(AddressRole::GuideTitle, 0)?,
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_height = title.height();
        let mut label_plans = Vec::new();
        fit::reserve(
            &mut label_plans,
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
            if plan.width().max(plan.fit_width()) > inner_width {
                return Ok(FitOutcome::Miss(
                    scope.child(AddressRole::GuideLabel, index)?.failure(
                        FitKind::Text,
                        "continuous guide number exceeds its assigned width",
                    ),
                ));
            }
            label_plans.push(plan);
        }
        if !matches!(label_plans.len(), 1..=3) {
            return Err(Error::Render(
                "continuous guide must have one to three prepared labels".into(),
            ));
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
        let missing_height = if let Some(plan) = missing.as_ref() {
            let key = metrics.exact.key_width.add_pixels(missing_inset)?;
            let text = ExactLength::pixels(plan.height())?;
            let occupied = if key.compare(&text) == std::cmp::Ordering::Less {
                text
            } else {
                key
            };
            occupied.add_length(&metrics.exact.row_gap)
        } else {
            ExactLength::pixels(0.0)?
        };
        let label_height = label_plans
            .iter()
            .map(crate::layout::ResolvedTextPlan::height)
            .fold(0.0_f64, f64::max);
        let label_width = label_plans
            .iter()
            .map(|plan| plan.width().max(plan.fit_width()))
            .fold(0.0_f64, f64::max);
        let mut along = [0.0_f64; 3];
        for (extent, plan) in along.iter_mut().zip(&label_plans) {
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
            *extent = half_box.max(painted);
        }
        let along = &along[..label_plans.len()];
        let fractions = match label_plans.len() {
            1 => [0.5, 0.0, 0.0],
            2 => [0.0, 1.0, 0.0],
            _ => [
                0.0,
                crate::theme::ContinuousColors::normalized_fraction(
                    descriptor.domain,
                    descriptor.center.ok_or_else(|| {
                        Error::Render("three-label continuous guide has no center".into())
                    })?,
                )?,
                1.0,
            ],
        };
        let fractions = &fractions[..label_plans.len()];
        if fractions
            .windows(2)
            .any(|fraction| fraction[1] <= fraction[0])
        {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide labels have no positive projected separation",
            )));
        }
        let projected = projected_minimum(
            along,
            fractions,
            &metrics.exact.row_gap.add_pixels(2.0 / 1024.0)?,
            &metrics.exact.minimum,
        )?;
        let required = LegendMetrics::hard(&projected)?.max(metrics.minimum);
        let first = along[0];
        let last = along[along.len() - 1];
        let missing_width = if let Some(plan) = missing.as_ref() {
            metrics
                .exact
                .key_width
                .add_length(&metrics.exact.key_label_gap)
                .add_pixels(plan.width().max(plan.fit_width()))?
        } else {
            ExactLength::pixels(0.0)?
        };
        if metrics.outer_width(&missing_width)? > available.0 {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::GuideMissingBlock, 0)?.failure(
                    FitKind::Text,
                    "continuous guide missing label exceeds its assigned width",
                ),
            ));
        }
        let label_column = metrics
            .exact
            .key_width
            .add_length(&metrics.exact.key_label_gap)
            .add_pixels(label_width)?;
        if !horizontal && metrics.outer_width(&label_column)? > available.0 {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::GuideLabel, 0)?.failure(
                    FitKind::Text,
                    "continuous guide value column exceeds its assigned width",
                ),
            ));
        }
        let maximum = if horizontal {
            available.0 - LegendMetrics::hard(&inset.add_pixels(first)?.add_pixels(last)?)?
        } else if let Some(height) = available.1 {
            let mut reserve = inset
                .add_pixels(title_height)?
                .add_length(&missing_height)
                .add_pixels(first)?
                .add_pixels(last)?;
            if !descriptor.title.is_empty() {
                reserve = reserve.add_length(&metrics.exact.title_gap);
            }
            height - LegendMetrics::hard(&reserve)?
        } else {
            metrics.preferred.max(required)
        };
        if maximum < required {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide gradient is shorter than its projected label minimum",
            )));
        }
        let preferred = metrics.preferred.min(maximum);
        if !compact {
            *compact_changes = preferred > required;
        }
        let length = if compact {
            required
        } else {
            preferred.max(required)
        };
        let thickness = metrics.handle_w;
        let body_height = if horizontal {
            metrics
                .exact
                .key_width
                .add_length(&metrics.exact.row_gap)
                .add_pixels(label_height)?
        } else {
            ExactLength::pixels(length)?
                .add_pixels(first)?
                .add_pixels(last)?
        };
        let mut content_width = ExactLength::pixels(title.width().max(title.fit_width()))?;
        let along_width = if horizontal {
            ExactLength::pixels(length)?
                .add_pixels(first)?
                .add_pixels(last)?
        } else {
            label_column
        };
        for candidate in [&missing_width, &along_width] {
            if content_width.compare(candidate) == std::cmp::Ordering::Less {
                content_width = candidate.times(1);
            }
        }
        let width = metrics.outer_width(&content_width)?.max(frame_minimum);
        let mut content_height = body_height
            .add_length(&missing_height)
            .add_pixels(title_height)?;
        if !descriptor.title.is_empty() {
            content_height = content_height.add_length(&metrics.exact.title_gap);
        }
        let height = metrics
            .outer_height(&content_height, false)?
            .max(frame_minimum);
        if width > available.0 || available.1.is_some_and(|available| height > available) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide exceeds its assigned track",
            )));
        }
        let mut stops = Vec::new();
        fit::reserve(
            &mut stops,
            descriptor.stops.len(),
            "continuous guide gradient stops",
        )?;
        stops.extend_from_slice(&descriptor.stops);
        let base_exact = metrics.exact.padding.add_length(&missing_height);
        let (gradient, start, end) = if horizontal {
            let x0 = metrics.exact.padding.after_pixels(first)?;
            let y0 = LegendMetrics::position(
                &base_exact
                    .add_pixels(label_height)?
                    .add_length(&metrics.exact.row_gap),
            )?;
            (
                PanelBox::new(x0, x0 + length, y0, y0 + thickness),
                Point2D::new(x0, y0 + thickness / 2.0),
                Point2D::new(x0 + length, y0 + thickness / 2.0),
            )
        } else {
            let y0 = base_exact.after_pixels(first)?;
            (
                PanelBox::new(metrics.pad, metrics.pad + thickness, y0, y0 + length),
                Point2D::new(metrics.pad + thickness / 2.0, y0),
                Point2D::new(metrics.pad + thickness / 2.0, y0 + length),
            )
        };
        backend::validate_linear_gradient(gradient, start, end, &stops)?;
        let title_baseline = title.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
        let title = PlacedText {
            placement: PlannedTextPlacement::baseline(
                Point2D::new(
                    metrics.pad,
                    metrics
                        .exact
                        .padding
                        .before_pixels(height - title_baseline)?,
                ),
                TextAnchor::Start,
                0.0,
            ),
            plan: title,
        };
        let mut labels = Vec::new();
        fit::reserve(
            &mut labels,
            label_plans.len(),
            "placed continuous guide labels",
        )?;
        let label_x_exact = metrics
            .exact
            .padding
            .add_length(&metrics.exact.key_width)
            .add_length(&metrics.exact.key_label_gap);
        for (index, plan) in label_plans.into_iter().enumerate() {
            let fraction = fractions[index];
            labels.push(if horizontal {
                centered_text(
                    plan,
                    gradient.x0 + length * fraction,
                    base_exact.after_pixels(label_height / 2.0)?,
                )
            } else {
                let x = label_x_exact.after_pixels(plan.width() / 2.0)?;
                centered_text(plan, x, gradient.y0 + length * fraction)
            });
        }
        let missing = missing
            .zip(descriptor.missing.as_ref())
            .map(|(plan, key)| -> Result<_, Error> {
                let y0 = metrics.pad;
                let row_height = plan.height().max(thickness + missing_inset);
                Ok((
                    PanelBox::new(metrics.pad, metrics.pad + thickness, y0, y0 + thickness),
                    crate::theme::FixedFillTheme::new(key.fill, key.outline.clone())?
                        .backend_style(),
                    PlacedText::middle(
                        plan,
                        Point2D::new(
                            LegendMetrics::position(&label_x_exact)?,
                            y0 + row_height / 2.0,
                        ),
                        TextAnchor::Start,
                        0.0,
                    ),
                ))
            })
            .transpose()?;
        let measured = Self {
            width,
            height,
            metrics,
            gradient,
            start,
            end,
            stops,
            title,
            labels,
            missing,
            format: descriptor.format.clone(),
            members: crate::data::source::collect_source(
                descriptor.numeric.members.iter().copied().map(Ok),
            )?,
            precision: descriptor.numeric.precision,
        };
        if !measured.labels_have_clearance(horizontal)? {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide projected label ink violates its traced adjacent gap",
            )));
        }
        if !measured.ink_fits_allocation()? {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "complete continuous guide ink exceeds its assigned allocation",
            )));
        }
        Ok(FitOutcome::Measured(measured))
    }

    fn ink_fits_allocation(&self) -> Result<bool, Error> {
        let mut ink = crate::render::geometry::GeometrySnapshot::new();
        if let Some((frame, style)) = self.frame(0.0, 0.0) {
            ink.rounded_rect(frame, &style, None)?;
        }
        ink.linear_gradient(self.gradient, self.start, self.end, &self.stops, None)?;
        ink.text(self.title.placement, &self.title.plan, None)?;
        for text in &self.labels {
            ink.text(text.placement, &text.plan, None)?;
        }
        if let Some((bounds, fill, text)) = &self.missing {
            ink.polygon(&Self::corners(*bounds, 0.0, 0.0), fill, None)?;
            ink.text(text.placement, &text.plan, None)?;
        }
        Ok(ink.bounds().iter().all(|bounds| {
            bounds.left >= 0.0
                && bounds.bottom >= 0.0
                && bounds.right <= self.width
                && bounds.top <= self.height
        }))
    }

    fn labels_have_clearance(&self, horizontal: bool) -> Result<bool, Error> {
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
                if previous.is_some_and(|previous| start - previous < self.metrics.row_gap) {
                    return Ok(false);
                }
                previous = Some(end);
            }
        }
        Ok(true)
    }

    fn frame(&self, left: f64, bottom: f64) -> Option<([f64; 5], FillStyle)> {
        self.metrics
            .background(left, bottom, self.width, self.height)
    }

    fn corners(bounds: PanelBox, left: f64, bottom: f64) -> [Point2D; 4] {
        [
            Point2D::new(left + bounds.x0, bottom + bounds.y0),
            Point2D::new(left + bounds.x1, bottom + bounds.y0),
            Point2D::new(left + bounds.x1, bottom + bounds.y1),
            Point2D::new(left + bounds.x0, bottom + bounds.y1),
        ]
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        self.metrics.source.observe_feedback(round)?;
        round.observe_count(usize::from(self.format.is_some()))?;
        if let Some(format) = &self.format {
            format.observe_feedback(round)?;
        }
        round.observe_count(usize::from(self.precision))?;
        round.observe_count(self.members.len())?;
        for &value in &self.members {
            round.observe_number(f64::from_bits(value))?;
        }
        for value in [
            self.width,
            self.height,
            self.gradient.x0,
            self.gradient.y0,
            self.gradient.width(),
            self.gradient.height(),
            self.start.x,
            self.start.y,
            self.end.x,
            self.end.y,
        ] {
            round.observe_number(value)?;
        }
        round.observe_count(usize::from(self.metrics.frame.is_some()))?;
        if let Some((frame, style)) = self.frame(0.0, 0.0) {
            for value in frame {
                round.observe_number(value)?;
            }
            for paint in [style.color, style.stroke] {
                for channel in [paint.red(), paint.green(), paint.blue(), paint.alpha()] {
                    round.observe_count(usize::from(channel))?;
                }
            }
            round.observe_number(f64::from(style.stroke_width))?;
        }
        round.observe_text(&self.title.plan)?;
        round.observe_placement(self.title.placement)?;
        for label in &self.labels {
            round.observe_text(&label.plan)?;
            round.observe_placement(label.placement)?;
        }
        for stop in &self.stops {
            round.observe_number(stop.offset)?;
            for channel in [stop.color.0, stop.color.1, stop.color.2] {
                round.observe_count(usize::from(channel))?;
            }
        }
        round.observe_count(usize::from(self.missing.is_some()))?;
        if let Some((bounds, fill, text)) = &self.missing {
            round.observe_box(*bounds)?;
            for paint in [fill.color, fill.stroke] {
                for channel in [paint.red(), paint.green(), paint.blue(), paint.alpha()] {
                    round.observe_count(usize::from(channel))?;
                }
            }
            round.observe_number(f64::from(fill.stroke_width))?;
            round.observe_text(&text.plan)?;
            round.observe_placement(text.placement)?;
        }
        Ok(())
    }

    pub(crate) fn select(&self, left: f64, bottom: f64) -> Result<LegendDrawPlan, Error> {
        let translate = |text: &PlacedText| {
            let mut text = text.clone();
            text.placement.position.x += left;
            text.placement.position.y += bottom;
            text
        };
        let mut calls = Vec::new();
        crate::data::source::reserve_source(
            &mut calls,
            self.labels.len()
                + 2
                + usize::from(self.metrics.frame.is_some())
                + usize::from(self.missing.is_some()) * 2,
        )?;
        if let Some((frame, style)) = self.frame(left, bottom) {
            calls.push(LegendCall::Background(frame, style));
        }
        let bounds = self.gradient;
        let gradient = PanelBox::new(
            bounds.x0 + left,
            bounds.x1 + left,
            bounds.y0 + bottom,
            bounds.y1 + bottom,
        );
        let start = Point2D::new(self.start.x + left, self.start.y + bottom);
        let end = Point2D::new(self.end.x + left, self.end.y + bottom);

        let stops = crate::data::source::collect_source(self.stops.iter().copied().map(Ok))?;
        calls.push(LegendCall::Gradient(gradient, start, end, stops));
        calls.push(LegendCall::Text(translate(&self.title)));
        for label in &self.labels {
            calls.push(LegendCall::Text(translate(label)));
        }
        if let Some((bounds, fill, text)) = &self.missing {
            let corners = Self::corners(*bounds, left, bottom);
            calls.push(LegendCall::Fill(corners, fill.clone()));
            calls.push(LegendCall::Text(translate(text)));
        }
        Ok(LegendDrawPlan { calls })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decorations::legend::descriptor::MissingKey;

    fn descriptor() -> ContinuousDescriptor {
        ContinuousDescriptor {
            selection: crate::theme::ContinuousColorScale::diverging(
                0.0,
                Some(
                    crate::theme::scale_values::test_defaults()
                        .diverging()
                        .clone(),
                ),
                None,
            )
            .unwrap(),
            field: "signal".into(),
            title: "Signal".into(),
            stops: vec![
                GradientStop::new(0.0, (49, 130, 189)),
                GradientStop::new(0.4, (247, 247, 247)),
                GradientStop::new(1.0, (222, 45, 38)),
            ],
            center: Some(0.0),
            missing: Some(MissingKey {
                fill: crate::theme::Color::rgb(224, 224, 224),
                outline: Some(crate::theme::OutlineTheme {
                    paint: crate::theme::Color::rgb(120, 120, 120),
                    width: 1.0,
                }),
                label: "Missing".into(),
            }),
            domain: (-2.0, 3.0),
            exact: false,
            format: None,
            numeric: super::super::descriptor::domain_presentation((-2.0, 3.0), Some(0.0), None)
                .unwrap(),
        }
    }

    #[test]
    fn unusable_guide_misses_without_shaping() {
        {
            let theme = crate::Theme::publication().resolve().unwrap();
            theme.prepare_font_environment().unwrap();

            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
            let mut compact_changes = true;
            let result = MeasuredContinuousGuide::measure(
                &descriptor(),
                (0.0, 0.0),
                false,
                false,
                &theme,
                &mut compact_changes,
                &FitScope::root(),
            );
            assert!(matches!(result.unwrap(), FitOutcome::Miss(_)));

            assert_eq!(
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                shapes
            );
        }
    }

    #[test]
    fn repeated_continuous_guide_reuses_cached_text() {
        let theme = crate::Theme::publication().resolve().unwrap();

        let guide = descriptor();
        let measure = || {
            fit::measured(MeasuredContinuousGuide::measure(
                &guide,
                (300.0, 300.0),
                false,
                false,
                &theme,
                &mut false,
                &FitScope::root(),
            ))
            .unwrap()
        };
        let original = measure();

        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let repeated = measure();

        assert_eq!(
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
            shapes
        );
        assert!(repeated.title.plan.shares_payload(&original.title.plan));
        assert_eq!(repeated.gradient, original.gradient);
        assert_eq!(repeated.width.to_bits(), original.width.to_bits());
        assert_eq!(repeated.height.to_bits(), original.height.to_bits());
    }

    #[test]
    fn projected_extents_require_the_exact_center_and_endpoint_clearance() {
        let gap = crate::theme::spacing::ExactLength::pixels(5.0).unwrap();
        let minimum = crate::theme::spacing::ExactLength::pixels(40.0).unwrap();
        let projected = |lengths: &[f64], fractions: &[f64]| {
            projected_minimum(lengths, fractions, &gap, &minimum)
                .unwrap()
                .pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)
                .unwrap()
                .to_bits()
        };
        assert_eq!(
            projected(&[4.0, 6.0, 10.0], &[0.0, 0.5, 1.0]),
            42.0_f64.to_bits()
        );
        assert_eq!(projected(&[3.0], &[0.5]), 40.0_f64.to_bits());
        assert_eq!(projected(&[30.0], &[0.5]), 70.0_f64.to_bits());
        assert_eq!(
            projected(&[4.0, 6.0, 10.0], &[0.0, 0.1, 1.0]),
            150.0_f64.to_bits()
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "both orientations compare preferred/compact ink and exact singleton allocation boundaries"
    )]
    fn constant_domain_large_typography_reserves_singleton_length_in_both_flows() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.guides.legend_label.font_size = 96;
        theme.typography.guides.legend_label.weight = crate::TextWeight::Bold;
        let spacing = theme
            .legend
            .spacing()
            .clone()
            .with_row_gap(crate::theme::RelativeSpacing::from_pixels(5.0, 96).unwrap())
            .unwrap();
        let lengths = theme
            .legend
            .gradient_lengths()
            .clone()
            .with_minimum(crate::theme::RelativeSpacing::from_pixels(40.0, 96).unwrap())
            .unwrap()
            .with_preferred(crate::theme::RelativeSpacing::from_pixels(512.0, 96).unwrap())
            .unwrap();
        theme.legend = theme
            .legend
            .clone()
            .with_spacing(spacing)
            .unwrap()
            .with_gradient_lengths(lengths)
            .unwrap();

        let descriptor = ContinuousDescriptor {
            selection: crate::theme::ContinuousColorScale::sequential(
                Some(
                    crate::theme::scale_values::test_defaults()
                        .sequential()
                        .clone(),
                ),
                Some(crate::theme::ColorDomain::exact(12_345.0, 12_345.0).unwrap()),
            )
            .unwrap(),
            field: "signal".into(),
            title: "Signal".into(),
            domain: (12_345.0, 12_345.0),
            center: None,
            stops: vec![
                GradientStop::new(0.0, (49, 130, 189)),
                GradientStop::new(1.0, (49, 130, 189)),
            ],
            format: None,
            numeric: super::super::descriptor::domain_presentation(
                (12_345.0, 12_345.0),
                None,
                None,
            )
            .unwrap(),
            missing: None,
            exact: true,
        };
        for horizontal in [false, true] {
            let measure = |available, compact| {
                MeasuredContinuousGuide::measure(
                    &descriptor,
                    available,
                    horizontal,
                    compact,
                    &theme,
                    &mut false,
                    &FitScope::root(),
                )
                .unwrap()
            };
            let preferred = fit::measured(Ok(measure((1200.0, 1200.0), false))).unwrap();
            let compact = fit::measured(Ok(measure((1200.0, 1200.0), true))).unwrap();
            let length = |guide: &MeasuredContinuousGuide| {
                if horizontal {
                    guide.gradient.width()
                } else {
                    guide.gradient.height()
                }
            };
            assert!(length(&compact) > LegendMetrics::from_theme(&theme).unwrap().minimum);
            assert!(length(&preferred) > length(&compact));
            for guide in [&preferred, &compact] {
                assert_eq!(guide.labels.len(), 1);
                assert_eq!(guide.labels[0].plan.source, descriptor.numeric.labels[0]);
                assert_eq!(guide.labels[0].plan.style.font_size, 96);
                assert_eq!(guide.stops, descriptor.stops);
                let label = &guide.labels[0];
                let midpoint = if horizontal {
                    f64::midpoint(guide.gradient.x0, guide.gradient.x1)
                } else {
                    f64::midpoint(guide.gradient.y0, guide.gradient.y1)
                };
                let anchor = if horizontal {
                    label.placement.position.x
                } else {
                    label.placement.position.y
                };
                assert!((anchor - midpoint).abs() <= 1.0 / 1024.0);
                let mut ink = crate::render::geometry::GeometrySnapshot::new();
                ink.text(label.placement, &label.plan, None).unwrap();
                let bounds = ink.bounds()[0];
                let half = if horizontal {
                    (anchor - bounds.left).max(bounds.right - anchor)
                } else {
                    (anchor - bounds.bottom).max(bounds.top - anchor)
                };
                assert!(
                    length(guide)
                        >= 2.0 * half + 2.0 * LegendMetrics::from_theme(&theme).unwrap().row_gap
                );
                let selected = guide.select(20.0, 30.0).unwrap();
                let mut complete = crate::render::geometry::GeometrySnapshot::new();
                selected.geometry(&mut complete).unwrap();
                assert!(complete.bounds().iter().all(|bounds| {
                    bounds.left >= 20.0
                        && bounds.right <= 20.0 + guide.width
                        && bounds.bottom >= 30.0
                        && bounds.top <= 30.0 + guide.height
                }));
            }
            let minimum = (compact.width, compact.height);
            let refused = if horizontal {
                (compact.width - 1.0 / 1024.0, compact.height)
            } else {
                (compact.width, compact.height - 1.0 / 1024.0)
            };
            for compact in [false, true] {
                assert!(matches!(measure(minimum, compact), FitOutcome::Measured(_)));
                let FitOutcome::Miss(failure) = measure(refused, compact) else {
                    panic!("one quantum below singleton minimum must refuse either flow variant");
                };
                assert_eq!(failure.reason.kind, FitKind::LegendOrGuide);
                assert_eq!(failure.reason.address, FitScope::root().address);
            }
        }
    }

    #[test]
    fn asymmetric_numeric_centers_keep_their_fraction_or_return_typed_fit() {
        let theme = crate::Theme::publication().resolve().unwrap();
        let mut descriptor = descriptor();
        descriptor.missing = None;
        descriptor.domain = (-0.01, 100.0);
        descriptor.center = Some(0.0);
        descriptor.numeric = super::super::descriptor::domain_presentation(
            descriptor.domain,
            descriptor.center,
            None,
        )
        .unwrap();
        // Sampled gradient stop positions deliberately do not identify the
        // center. Even a stop at one half cannot move scientific value zero.
        descriptor.stops = vec![
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.000_025, (80, 150, 200)),
            GradientStop::new(
                crate::theme::ContinuousColors::normalized_fraction(descriptor.domain, 0.0)
                    .unwrap(),
                (247, 247, 247),
            ),
            GradientStop::new(0.5, (239, 125, 112)),
            GradientStop::new(1.0, (222, 45, 38)),
        ];
        for horizontal in [false, true] {
            let scope = FitScope::root()
                .child(AddressRole::OutsideGuideRight, 0)
                .unwrap();
            let FitOutcome::Miss(failure) = MeasuredContinuousGuide::measure(
                &descriptor,
                (300.0, 300.0),
                horizontal,
                false,
                &theme,
                &mut false,
                &scope,
            )
            .unwrap() else {
                panic!("near-endpoint center cannot have overlapping numbers")
            };
            assert_eq!(failure.reason.kind, FitKind::LegendOrGuide);
            assert_eq!(failure.reason.address, scope.address);
        }
        descriptor.domain = (-1.0, 9.0);
        descriptor.numeric = super::super::descriptor::domain_presentation(
            descriptor.domain,
            descriptor.center,
            None,
        )
        .unwrap();
        descriptor.stops[1].offset = 0.05;
        descriptor.stops[2].offset = 0.1;
        for horizontal in [false, true] {
            let measured = fit::measured(MeasuredContinuousGuide::measure(
                &descriptor,
                (600.0, 600.0),
                horizontal,
                true,
                &theme,
                &mut false,
                &FitScope::root(),
            ))
            .unwrap();
            let expected = if horizontal {
                measured.gradient.x0 + measured.gradient.width() * 0.1
            } else {
                measured.gradient.y0 + measured.gradient.height() * 0.1
            };
            let position = measured.labels[1].placement.position;
            assert_eq!(
                (if horizontal { position.x } else { position.y }).to_bits(),
                expected.to_bits()
            );
            assert!(measured.labels_have_clearance(horizontal).unwrap());
            assert_eq!(measured.stops, descriptor.stops);
            assert_eq!(measured.labels.len(), 3);
        }
    }

    #[test]
    fn sampled_stops_do_not_move_the_numeric_center_label() {
        let mut descriptor = descriptor();
        descriptor
            .stops
            .insert(1, GradientStop::new(0.1, (70, 140, 200)));
        let theme = crate::Theme::publication().resolve().unwrap();
        let guide = fit::measured(MeasuredContinuousGuide::measure(
            &descriptor,
            (200.0, 400.0),
            false,
            false,
            &theme,
            &mut false,
            &FitScope::root(),
        ))
        .unwrap();
        let expected = guide.gradient.y0 + 0.4 * guide.gradient.height();
        assert_eq!(
            guide.labels[1].placement.position.y.to_bits(),
            expected.to_bits()
        );
    }

    #[test]
    fn horizontal_and_vertical_guides_freeze_one_gradient_and_missing_block() {
        let theme = crate::Theme::publication().resolve().unwrap();
        let guide = descriptor();
        let scope = FitScope::root();
        for horizontal in [false, true] {
            let FitOutcome::Measured(measured) = MeasuredContinuousGuide::measure(
                &guide,
                (300.0, 300.0),
                horizontal,
                false,
                &theme,
                &mut false,
                &scope,
            )
            .unwrap() else {
                panic!("the preferred shared guide must fit");
            };
            assert_eq!(measured.stops.len(), 3);
            assert_eq!(measured.stops[1].offset.to_bits(), 0.4_f64.to_bits());
            assert_eq!(measured.labels.len(), 3);
            assert!(measured.missing.is_some());
            assert!(measured.width <= 300.0 && measured.height <= 300.0);
            if horizontal {
                assert_eq!(
                    measured.gradient.height().to_bits(),
                    LegendMetrics::from_theme(&theme)
                        .unwrap()
                        .handle_w
                        .to_bits()
                );
                assert!(
                    measured.labels[0].placement.position.x
                        < measured.labels[2].placement.position.x
                );
            } else {
                assert_eq!(
                    measured.gradient.width().to_bits(),
                    LegendMetrics::from_theme(&theme)
                        .unwrap()
                        .handle_w
                        .to_bits()
                );
                assert!(
                    measured.labels[0].placement.position.y
                        < measured.labels[2].placement.position.y
                );
            }
        }
    }

    #[test]
    fn insufficient_projected_length_is_a_guide_fit_miss() {
        let theme = crate::Theme::publication().resolve().unwrap();
        let scope = FitScope::root();
        assert!(matches!(
            MeasuredContinuousGuide::measure(
                &descriptor(),
                (80.0, 70.0),
                false,
                false,
                &theme,
                &mut false,
                &scope,
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));
    }

    #[test]
    fn custom_frame_and_owner_missing_strokes_are_allocated_and_retained_unclipped() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let frame = crate::theme::LegendFrameTheme::new(
            crate::theme::Color::rgba(1, 2, 3, 127),
            Some(
                crate::theme::OutlineTheme::new(crate::theme::Color::rgba(4, 5, 6, 210), 30.0)
                    .unwrap(),
            ),
            crate::theme::RelativeSpacing::from_pixels(18.0, 11).unwrap(),
        )
        .unwrap();
        theme.legend = theme.legend.clone().with_frame(Some(frame)).unwrap();
        let mut descriptor = descriptor();
        descriptor
            .missing
            .as_mut()
            .unwrap()
            .outline
            .as_mut()
            .unwrap()
            .width = 40.0;
        for horizontal in [false, true] {
            let measured = fit::measured(MeasuredContinuousGuide::measure(
                &descriptor,
                (400.0, 400.0),
                horizontal,
                true,
                &theme,
                &mut false,
                &FitScope::root(),
            ))
            .unwrap();

            let selected = measured.select(20.0, 30.0).unwrap();
            let LegendCall::Background(rect, fill) = &selected.calls[0] else {
                panic!("complete frame must be retained first");
            };
            assert_eq!(rect[4].to_bits(), 18.0_f64.to_bits());
            assert_eq!(fill.color, theme.legend.frame().unwrap().fill());
            assert_eq!(
                fill.stroke,
                theme.legend.frame().unwrap().outline().unwrap().color()
            );
            assert_eq!(fill.stroke_width.to_bits(), 30.0_f32.to_bits());
            let missing = selected
                .calls
                .iter()
                .find_map(|call| match call {
                    LegendCall::Fill(_, fill) => Some(fill),
                    _ => None,
                })
                .unwrap();
            assert_eq!(missing.stroke_width.to_bits(), 40.0_f32.to_bits());
            assert_eq!(missing.color, descriptor.missing.as_ref().unwrap().fill);
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
            selected.geometry(&mut ink).unwrap();
            assert!(ink.bounds().iter().all(|bounds| {
                bounds.left >= 20.0
                    && bounds.bottom >= 30.0
                    && bounds.right <= 20.0 + measured.width
                    && bounds.top <= 30.0 + measured.height
            }));
            assert!(matches!(
                MeasuredContinuousGuide::measure(
                    &descriptor,
                    (measured.width - 1.0 / 1024.0, measured.height),
                    horizontal,
                    true,
                    &theme,
                    &mut false,
                    &FitScope::root(),
                )
                .unwrap(),
                FitOutcome::Miss(_)
            ));
        }
    }

    #[test]
    fn custom_text_ink_overhang_cannot_be_hidden_by_the_guide_frame() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.guides.legend_title.font_size = 40;
        theme.typography.guides.legend_title.weight = crate::TextWeight::Bold;
        let mut measured = fit::measured(MeasuredContinuousGuide::measure(
            &descriptor(),
            (300.0, 300.0),
            false,
            false,
            &theme,
            &mut false,
            &FitScope::root(),
        ))
        .unwrap();
        assert!(measured.ink_fits_allocation().unwrap());
        // The anchor is still inside the frame, but the required glyph ink is not.
        measured.title.placement.position.x = measured.width - 1.0;
        assert!(!measured.ink_fits_allocation().unwrap());
    }

    #[test]
    fn absent_continuous_frame_retains_gradient_labels_and_missing_key() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.legend = theme.legend.clone().with_frame(None).unwrap();
        let mut descriptor = descriptor();
        descriptor.missing.as_mut().unwrap().outline = None;

        let measured = fit::measured(MeasuredContinuousGuide::measure(
            &descriptor,
            (500.0, 500.0),
            false,
            false,
            &theme,
            &mut false,
            &FitScope::root(),
        ))
        .unwrap();
        let selected = measured.select(20.0, 30.0).unwrap();
        assert!(!selected
            .calls
            .iter()
            .any(|call| matches!(call, LegendCall::Background(_, _))));
        assert!(selected
            .calls
            .iter()
            .any(|call| matches!(call, LegendCall::Gradient(_, _, _, _))));
        assert!(selected
            .calls
            .iter()
            .any(|call| matches!(call, LegendCall::Fill(_, fill)
                if fill.stroke_width.to_bits() == 0 && fill.stroke.alpha() == 0)));
        let labels = selected
            .calls
            .iter()
            .filter_map(|call| {
                if let LegendCall::Text(text) = call {
                    Some(text.plan.source.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(labels.len(), descriptor.numeric.labels.len() + 2);
        assert_eq!(labels[0], descriptor.title);
        assert_eq!(labels.last().copied(), Some("Missing"));
        assert_eq!(
            &labels[1..=descriptor.numeric.labels.len()],
            descriptor
                .numeric
                .labels
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn compact_gradient_keeps_all_labels_and_skips_duplicate_allocations() {
        let mut theme = crate::Theme::publication().resolve().unwrap();
        let guide = descriptor();
        let scope = FitScope::root();
        let mut changes = false;
        let FitOutcome::Measured(preferred) = MeasuredContinuousGuide::measure(
            &guide,
            (300.0, 300.0),
            false,
            false,
            &theme,
            &mut changes,
            &scope,
        )
        .unwrap() else {
            panic!("preferred guide must fit");
        };
        assert!(changes);
        let FitOutcome::Measured(compact) = MeasuredContinuousGuide::measure(
            &guide,
            (300.0, 300.0),
            false,
            true,
            &theme,
            &mut changes,
            &scope,
        )
        .unwrap() else {
            panic!("compact guide must fit");
        };
        assert!(compact.gradient.height() < preferred.gradient.height());
        assert_eq!(compact.labels.len(), preferred.labels.len());
        assert_eq!(compact.stops, preferred.stops);

        let minimum = theme.legend.gradient_lengths().minimum().clone();
        let lengths = theme
            .legend
            .gradient_lengths()
            .clone()
            .with_preferred(minimum)
            .unwrap();
        theme.legend = theme.legend.clone().with_gradient_lengths(lengths).unwrap();
        MeasuredContinuousGuide::measure(
            &guide,
            (300.0, 300.0),
            false,
            false,
            &theme,
            &mut changes,
            &scope,
        )
        .unwrap();
        assert!(
            !changes,
            "identical compact geometry must not consume a second trial"
        );
    }
}
