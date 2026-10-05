//! One assigned inline guide from a font-independent continuous descriptor.

use super::descriptor::ContinuousDescriptor;
use super::{LegendCall, LegendDrawPlan, LegendMetrics};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
use crate::layout::{AddressRole, PanelBox, PhysicalTextAlignment};
use crate::render::backend::{
    self, FillStyle, GradientStop, PlacedText, PlannedTextPlacement, Point2D, TextAnchor,
};
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
}

fn centered_text(plan: crate::layout::ResolvedTextPlan, x: f64, y: f64) -> PlacedText {
    PlacedText::middle(plan, Point2D::new(x, y), TextAnchor::Middle, 0.0)
}

fn projected_minimum(lengths: &[f64], fractions: &[f64], gap: f64, traced: f64) -> f64 {
    if let [half_extent] = lengths {
        return traced.max(2.0 * half_extent + 2.0 * gap);
    }
    lengths
        .windows(2)
        .zip(fractions.windows(2))
        .fold(traced, |required, (extent, fraction)| {
            required.max((extent[0] + gap + extent[1]) / (fraction[1] - fraction[0]))
        })
}

impl MeasuredContinuousGuide {
    /// Effective content chrome and the frame's minimum square extent, shared
    /// with containing owners' intrinsic width proposals.
    pub(crate) fn allocation_metrics(
        descriptor: &ContinuousDescriptor,
        theme: &ResolvedTheme,
    ) -> Result<(LegendMetrics, f64), Error> {
        let mut metrics = LegendMetrics::from_theme(theme);
        let frame_inset =
            crate::layout::kernel::required_length(f64::from(metrics.frame_stroke_width) / 2.0)?
                .max(0.5);
        let missing_inset = descriptor
            .missing
            .as_ref()
            .map_or(0.0, |key| f64::from(key.outline.width) / 2.0);
        metrics.pad = metrics.pad.max(frame_inset).max(missing_inset);
        metrics.handle_text = metrics.handle_text.max(missing_inset);
        let frame_minimum = 2.0 * (frame_inset + metrics.frame_radius);
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
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| {
                Error::Render(format!(
                    "continuous guide measurement call admission: {error:?}"
                ))
            })?;
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
            .map_or(0.0, |key| f64::from(key.outline.width) / 2.0);
        let inner_width = available.0 - 2.0 * metrics.pad;
        let inner_height = available.1.map(|height| height - 2.0 * metrics.pad);
        if inner_width <= 0.0 || inner_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide has no positive content extent",
            )));
        }
        let title = match fit::text(
            theme,
            &descriptor.title,
            &theme.typography.legend_title,
            inner_width,
            PhysicalTextAlignment::Start,
            "continuous guide title",
            &scope.child(
                AddressRole::GuideTitle,
                0,
                &theme.prepared_font_environment()?.generation_resources(),
            )?,
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_height = title.height();
        let title_gap = if descriptor.title.is_empty() {
            0.0
        } else {
            metrics.title_gap
        };
        let mut label_plans = Vec::new();
        fit::reserve(
            theme,
            &mut label_plans,
            descriptor.labels.len(),
            "continuous guide labels",
        )?;
        for (index, label) in descriptor.labels.iter().enumerate() {
            let plan = theme.measure_legacy_appearance_text(
                label,
                &theme.typography.legend_label,
                PhysicalTextAlignment::Center,
                "continuous guide value",
                false,
            )?;
            if plan.width().max(plan.fit_width()) > inner_width {
                return Ok(FitOutcome::Miss(
                    scope
                        .child(
                            AddressRole::GuideLabel,
                            index,
                            &theme.prepared_font_environment()?.generation_resources(),
                        )?
                        .failure(
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
                    &theme.typography.legend_label,
                    PhysicalTextAlignment::Start,
                    "continuous guide missing key",
                    false,
                )
            })
            .transpose()?;
        let missing_height = missing.as_ref().map_or(0.0, |plan| {
            plan.height().max(metrics.handle_w + missing_inset) + metrics.row_gap
        });
        let key_gap = metrics.handle_text;
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
            let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
            ink.text(placed.placement, &placed.plan, None)?;
            theme
                .prepared_font_environment()?
                .claim_processing_visits(1 + ink.bounds().len() as u128)
                .map_err(|error| {
                    Error::Render(format!("continuous label extent visits: {error:?}"))
                })?;
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
        let projected = projected_minimum(
            along,
            fractions,
            metrics.row_gap + 2.0 / 1024.0,
            theme.legend.gradient_min_length,
        );
        let required = match crate::layout::kernel::required_length(projected) {
            Ok(length) if projected.is_finite() => length,
            _ => {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::LegendOrGuide,
                    "continuous guide labels have no representable projected separation",
                )))
            }
        };
        let first = along[0];
        let last = along[along.len() - 1];
        let missing_width = missing.as_ref().map_or(0.0, |plan| {
            metrics.handle_w + key_gap + plan.width().max(plan.fit_width())
        });
        if missing_width > inner_width {
            return Ok(FitOutcome::Miss(
                scope
                    .child(
                        AddressRole::GuideMissingBlock,
                        0,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        FitKind::Text,
                        "continuous guide missing label exceeds its assigned width",
                    ),
            ));
        }
        if !horizontal && metrics.handle_w + key_gap + label_width > inner_width {
            return Ok(FitOutcome::Miss(
                scope
                    .child(
                        AddressRole::GuideLabel,
                        0,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        FitKind::Text,
                        "continuous guide value column exceeds its assigned width",
                    ),
            ));
        }
        let maximum = if horizontal {
            inner_width - first - last
        } else {
            inner_height.map_or(
                theme.legend.gradient_preferred_length.max(required),
                |height| height - title_height - title_gap - missing_height - first - last,
            )
        };
        if maximum < required {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide gradient is shorter than its projected label minimum",
            )));
        }
        let preferred = theme.legend.gradient_preferred_length.min(maximum);
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
            thickness + metrics.row_gap + label_height
        } else {
            length + first + last
        };
        let width = crate::layout::kernel::required_length(
            (2.0 * metrics.pad
                + title
                    .width()
                    .max(title.fit_width())
                    .max(missing_width)
                    .max(if horizontal {
                        length + first + last
                    } else {
                        thickness + key_gap + label_width
                    }))
            .max(frame_minimum),
        )?;
        let height = crate::layout::kernel::required_length(
            (2.0 * metrics.pad + title_height + title_gap + body_height + missing_height)
                .max(frame_minimum),
        )?;
        if width > available.0 || available.1.is_some_and(|available| height > available) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide exceeds its assigned track",
            )));
        }
        let mut stops = Vec::new();
        fit::reserve(
            theme,
            &mut stops,
            descriptor.stops.len(),
            "continuous guide gradient stops",
        )?;
        stops.extend_from_slice(&descriptor.stops);
        let base = metrics.pad + missing_height;
        let (gradient, start, end) = if horizontal {
            let x0 = metrics.pad + first;
            let y0 = base + label_height + metrics.row_gap;
            (
                PanelBox::new(x0, x0 + length, y0, y0 + thickness),
                Point2D::new(x0, y0 + thickness / 2.0),
                Point2D::new(x0 + length, y0 + thickness / 2.0),
            )
        } else {
            let y0 = base + first;
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
                Point2D::new(metrics.pad, height - metrics.pad - title_baseline),
                TextAnchor::Start,
                0.0,
            ),
            plan: title,
        };
        let mut labels = Vec::new();
        fit::reserve(
            theme,
            &mut labels,
            label_plans.len(),
            "placed continuous guide labels",
        )?;
        let label_x = metrics.pad + thickness + key_gap;
        for (index, plan) in label_plans.into_iter().enumerate() {
            let fraction = fractions[index];
            labels.push(if horizontal {
                centered_text(
                    plan,
                    gradient.x0 + length * fraction,
                    base + label_height / 2.0,
                )
            } else {
                let x = label_x + plan.width() / 2.0;
                centered_text(plan, x, gradient.y0 + length * fraction)
            });
        }
        let missing = missing.zip(descriptor.missing.as_ref()).map(|(plan, key)| {
            let y0 = metrics.pad;
            let row_height = plan.height().max(thickness + missing_inset);
            (
                PanelBox::new(metrics.pad, metrics.pad + thickness, y0, y0 + thickness),
                key.outline.backend_style(key.fill),
                PlacedText::middle(
                    plan,
                    Point2D::new(metrics.pad + thickness + key_gap, y0 + row_height / 2.0),
                    TextAnchor::Start,
                    0.0,
                ),
            )
        });
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
        };
        if !measured.labels_have_clearance(theme, horizontal)? {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide projected label ink violates its traced adjacent gap",
            )));
        }
        if !measured.ink_fits_allocation(theme)? {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "complete continuous guide ink exceeds its assigned allocation",
            )));
        }
        Ok(FitOutcome::Measured(measured))
    }

    fn ink_fits_allocation(&self, theme: &ResolvedTheme) -> Result<bool, Error> {
        let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let (frame, style) = self.frame(0.0, 0.0)?;
        ink.rounded_rect(frame, &style, None)?;
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

    fn labels_have_clearance(
        &self,
        theme: &ResolvedTheme,
        horizontal: bool,
    ) -> Result<bool, Error> {
        let mut previous = None;
        for text in &self.labels {
            theme
                .prepared_font_environment()?
                .claim_processing_visits(1)
                .map_err(|error| {
                    Error::Render(format!("continuous label clearance visits: {error:?}"))
                })?;
            let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
            ink.text(text.placement, &text.plan, None)?;
            theme
                .prepared_font_environment()?
                .claim_processing_visits(ink.bounds().len() as u128)
                .map_err(|error| {
                    Error::Render(format!("continuous label ink visits: {error:?}"))
                })?;
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

    fn frame(&self, left: f64, bottom: f64) -> Result<([f64; 5], FillStyle), Error> {
        let m = &self.metrics;
        let inset =
            crate::layout::kernel::required_length(f64::from(m.frame_stroke_width) / 2.0)?.max(0.5);
        Ok((
            [
                left + inset,
                bottom + inset,
                self.width - 2.0 * inset,
                self.height - 2.0 * inset,
                m.frame_radius,
            ],
            FillStyle::new(m.frame_fill, m.frame_outline, m.frame_stroke_width),
        ))
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
        let (frame, style) = self.frame(0.0, 0.0)?;
        for value in frame {
            round.observe_number(value)?;
        }
        for paint in [style.color, style.stroke] {
            for channel in [paint.red(), paint.green(), paint.blue(), paint.alpha()] {
                round.observe_count(usize::from(channel))?;
            }
        }
        round.observe_number(f64::from(style.stroke_width))?;
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

    pub(crate) fn select(
        &self,
        left: f64,
        bottom: f64,
        data: crate::data::PlotData<'_>,
    ) -> Result<LegendDrawPlan, Error> {
        let translate = |text: &PlacedText| {
            let mut text = text.clone();
            text.placement.position.x += left;
            text.placement.position.y += bottom;
            text
        };
        let mut calls = Vec::new();
        data.reserve_source(
            &mut calls,
            self.labels.len() + 3 + usize::from(self.missing.is_some()) * 2,
        )?;
        let (frame, style) = self.frame(left, bottom)?;
        calls.push(LegendCall::Background(frame, style));
        let bounds = self.gradient;
        let gradient = PanelBox::new(
            bounds.x0 + left,
            bounds.x1 + left,
            bounds.y0 + bottom,
            bounds.y1 + bottom,
        );
        let start = Point2D::new(self.start.x + left, self.start.y + bottom);
        let end = Point2D::new(self.end.x + left, self.end.y + bottom);
        data.claim_source_visits(self.stops.len())?;
        let stops = data.collect_source(self.stops.iter().copied().map(Ok))?;
        calls.push(LegendCall::Gradient(gradient, start, end, stops));
        calls.push(LegendCall::Text(translate(&self.title)));
        for label in &self.labels {
            data.claim_source_visits(1)?;
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
                outline: crate::theme::OutlineTheme {
                    paint: crate::theme::Color::rgb(120, 120, 120),
                    width: 1.0,
                },
                label: "Missing".into(),
            }),
            domain: (-2.0, 3.0),
            exact: false,
            labels: vec!["-2".into(), "0".into(), "3".into()],
        }
    }

    #[test]
    fn unusable_guide_admits_a_call_before_any_measurement_work() {
        for remaining in [0, 1, 2] {
            let theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            theme.prepare_font_environment().unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            resources
                .claim_calls(1_000_000 - resources.call_cache_usage().0 - remaining)
                .unwrap();
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
            if remaining == 0 {
                assert!(result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("measurement/intrinsic call attempts"));
            } else {
                assert!(matches!(result.unwrap(), FitOutcome::Miss(_)));
            }
            assert_eq!(
                resources.call_cache_usage().0,
                1_000_000 - remaining + u128::from(remaining > 0)
            );
            assert_eq!(
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                shapes
            );
        }
    }

    #[test]
    fn repeated_continuous_guide_admits_calls_for_cached_text() {
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
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
        let before = resources.call_cache_usage().0;
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let repeated = measure();
        // One guide call, title, three endpoint/center labels, and Missing.
        assert_eq!(resources.call_cache_usage().0 - before, 6);
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
        assert_eq!(
            projected_minimum(&[4.0, 6.0, 10.0], &[0.0, 0.5, 1.0], 5.0, 40.0).to_bits(),
            42.0_f64.to_bits()
        );
        assert_eq!(
            projected_minimum(&[3.0], &[0.5], 5.0, 40.0).to_bits(),
            40.0_f64.to_bits()
        );
        assert_eq!(
            projected_minimum(&[30.0], &[0.5], 5.0, 40.0).to_bits(),
            70.0_f64.to_bits()
        );
        assert_eq!(
            projected_minimum(&[4.0, 6.0, 10.0], &[0.0, 0.1, 1.0], 5.0, 40.0).to_bits(),
            150.0_f64.to_bits()
        );
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "both orientations compare preferred/compact ink and exact singleton allocation boundaries"
    )]
    fn constant_domain_large_typography_reserves_singleton_length_in_both_flows() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.typography.legend_label.font_size = 96;
        theme.typography.legend_label.weight = crate::TextWeight::Bold;
        theme.legend.row_gap = 5.0;
        theme.legend.gradient_preferred_length = 512.0;
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let data = source.data().with_axis_resources(&resources);
        let descriptor = ContinuousDescriptor {
            field: "signal".into(),
            title: "Signal".into(),
            domain: (12_345.0, 12_345.0),
            center: None,
            stops: vec![
                GradientStop::new(0.0, (49, 130, 189)),
                GradientStop::new(1.0, (49, 130, 189)),
            ],
            labels: super::super::descriptor::numeric_labels(&[12_345.0], data).unwrap(),
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
            assert!(length(&compact) > theme.legend.gradient_min_length);
            assert!(length(&preferred) > length(&compact));
            for guide in [&preferred, &compact] {
                assert_eq!(guide.labels.len(), 1);
                assert_eq!(guide.labels[0].plan.source, descriptor.labels[0]);
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
                let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
                ink.text(label.placement, &label.plan, None).unwrap();
                let bounds = ink.bounds()[0];
                let half = if horizontal {
                    (anchor - bounds.left).max(bounds.right - anchor)
                } else {
                    (anchor - bounds.bottom).max(bounds.top - anchor)
                };
                assert!(length(guide) >= 2.0 * half + 2.0 * theme.legend.row_gap);
                let selected = guide.select(20.0, 30.0, data).unwrap();
                let mut complete = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
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
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let mut descriptor = descriptor();
        descriptor.missing = None;
        descriptor.domain = (-0.01, 100.0);
        descriptor.center = Some(0.0);
        descriptor.labels = vec!["-0.01".into(), "0".into(), "100".into()];
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
                .child(
                    AddressRole::OutsideGuideRight,
                    0,
                    &theme
                        .prepared_font_environment()
                        .unwrap()
                        .generation_resources(),
                )
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
        descriptor.labels = vec!["-1".into(), "0".into(), "9".into()];
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
            assert!(measured.labels_have_clearance(&theme, horizontal).unwrap());
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
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
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
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
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
                    theme.legend.key_width.to_bits()
                );
                assert!(
                    measured.labels[0].placement.position.x
                        < measured.labels[2].placement.position.x
                );
            } else {
                assert_eq!(
                    measured.gradient.width().to_bits(),
                    theme.legend.key_width.to_bits()
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
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
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
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .erase();
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.legend.frame_fill = crate::theme::Color::rgba(1, 2, 3, 127);
        theme.legend.frame_outline = crate::theme::Color::rgba(4, 5, 6, 210);
        theme.legend.frame_stroke_width = 30.0;
        theme.legend.frame_radius = 18.0;
        let mut descriptor = descriptor();
        descriptor.missing.as_mut().unwrap().outline.width = 40.0;
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
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let selected = measured
                .select(20.0, 30.0, source.data().with_axis_resources(&resources))
                .unwrap();
            let LegendCall::Background(rect, fill) = &selected.calls[0] else {
                panic!("complete frame must be retained first");
            };
            assert_eq!(rect[4].to_bits(), 18.0_f64.to_bits());
            assert_eq!(fill.color, theme.legend.frame_fill);
            assert_eq!(fill.stroke, theme.legend.frame_outline);
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
            let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
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
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.typography.legend_title.font_size = 40;
        theme.typography.legend_title.weight = crate::TextWeight::Bold;
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
        assert!(measured.ink_fits_allocation(&theme).unwrap());
        // The anchor is still inside the frame, but the required glyph ink is not.
        measured.title.placement.position.x = measured.width - 1.0;
        assert!(!measured.ink_fits_allocation(&theme).unwrap());
    }

    #[test]
    fn compact_gradient_keeps_all_labels_and_skips_duplicate_allocations() {
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
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

        theme.legend.gradient_preferred_length = theme.legend.gradient_min_length;
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
