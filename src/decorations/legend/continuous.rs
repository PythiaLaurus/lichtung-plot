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

fn projected_minimum(lengths: &[f64], gap: f64, traced: f64) -> f64 {
    if lengths.len() == 1 {
        return traced.max(2.0 * lengths[0] + 2.0 * gap);
    }
    let start = lengths[0];
    let end = lengths[lengths.len() - 1];
    let mut required = traced.max(start + gap + end);
    if lengths.len() == 3 {
        required = required
            .max(2.0 * (start + gap + lengths[1]))
            .max(2.0 * (lengths[1] + gap + end));
    }
    required
}

fn fraction(value: f64, domain: (f64, f64)) -> f64 {
    let span = domain.1 - domain.0;
    if span.is_finite() {
        ((value - domain.0) / span).clamp(0.0, 1.0)
    } else {
        let scale = domain.0.abs().max(domain.1.abs());
        ((value / scale - domain.0 / scale) / (domain.1 / scale - domain.0 / scale)).clamp(0.0, 1.0)
    }
}

impl MeasuredContinuousGuide {
    #[expect(
        clippy::too_many_lines,
        reason = "one assigned orientation and flow retains exact guide text and gradient operands"
    )]
    pub(crate) fn measure(
        descriptor: &ContinuousDescriptor,
        available: (f64, f64),
        horizontal: bool,
        compact: bool,
        theme: &ResolvedTheme,
        compact_changes: &mut bool,
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        theme.prepare_font_environment()?;
        if !available.0.is_finite() || !available.1.is_finite() {
            return Err(Error::Render(
                "nonfinite continuous guide assignment".into(),
            ));
        }
        let metrics = LegendMetrics::from_theme(theme);
        let inner_width = available.0 - 2.0 * metrics.pad;
        let inner_height = available.1 - 2.0 * metrics.pad;
        if inner_width <= 0.0 || inner_height <= 0.0 {
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
            &scope.child(AddressRole::GuideTitle, 0)?,
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
                    &theme.typography.legend_label,
                    PhysicalTextAlignment::Start,
                    "continuous guide missing key",
                    false,
                )
            })
            .transpose()?;
        let missing_height = missing.as_ref().map_or(0.0, |plan| {
            plan.height().max(metrics.handle_w) + metrics.row_gap
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
            *extent = if horizontal {
                plan.width().max(plan.fit_width())
            } else {
                plan.height()
            } / 2.0;
        }
        let along = &along[..label_plans.len()];
        let required = crate::layout::kernel::required_length(projected_minimum(
            along,
            metrics.row_gap,
            theme.legend.gradient_min_length,
        ))?;
        let first = along[0];
        let last = along[along.len() - 1];
        let missing_width = missing
            .as_ref()
            .map_or(0.0, |plan| metrics.handle_w + key_gap + plan.width());
        if missing_width > inner_width {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::GuideMissingBlock, 0)?.failure(
                    FitKind::Text,
                    "continuous guide missing label exceeds its assigned width",
                ),
            ));
        }
        if !horizontal && metrics.handle_w + key_gap + label_width > inner_width {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::GuideLabel, 0)?.failure(
                    FitKind::Text,
                    "continuous guide value column exceeds its assigned width",
                ),
            ));
        }
        let maximum = if horizontal {
            inner_width - first - last
        } else {
            inner_height - title_height - title_gap - missing_height - first - last
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
        let width = 2.0 * metrics.pad
            + title.width().max(missing_width).max(if horizontal {
                length + first + last
            } else {
                thickness + key_gap + label_width
            });
        let height = 2.0 * metrics.pad + title_height + title_gap + body_height + missing_height;
        if width > available.0 || height > available.1 {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "continuous guide exceeds its assigned track",
            )));
        }
        let mut stops = Vec::new();
        fit::reserve(theme, &mut stops, 3, "continuous guide gradient stops")?;
        stops.push(GradientStop::new(0.0, descriptor.palette.low));
        if let (Some(center), Some(color)) = (descriptor.center, descriptor.palette.center) {
            if center > descriptor.domain.0 && center < descriptor.domain.1 {
                stops.push(GradientStop::new(
                    fraction(center, descriptor.domain),
                    color,
                ));
            }
        }
        stops.push(GradientStop::new(1.0, descriptor.palette.high));
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
            let fraction = if descriptor.labels.len() == 1 {
                0.5
            } else if index == 0 {
                0.0
            } else if index + 1 == descriptor.labels.len() {
                1.0
            } else {
                fraction(
                    descriptor.center.expect("prepared center label"),
                    descriptor.domain,
                )
            };
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
            let row_height = plan.height().max(thickness);
            (
                PanelBox::new(metrics.pad, metrics.pad + thickness, y0, y0 + thickness),
                FillStyle::new(
                    key.fill.into(),
                    key.outline.into(),
                    metrics.frame_stroke_width,
                ),
                PlacedText::middle(
                    plan,
                    Point2D::new(metrics.pad + thickness + key_gap, y0 + row_height / 2.0),
                    TextAnchor::Start,
                    0.0,
                ),
            )
        });
        Ok(FitOutcome::Measured(Self {
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
        }))
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
        ] {
            round.observe_number(value)?;
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
            for channel in [
                fill.color.red(),
                fill.color.green(),
                fill.color.blue(),
                fill.color.alpha(),
            ] {
                round.observe_count(usize::from(channel))?;
            }
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
        let m = &self.metrics;
        calls.push(LegendCall::Background(
            [
                left + 0.5,
                bottom + 0.5,
                self.width - 1.0,
                self.height - 1.0,
                m.frame_radius,
            ],
            FillStyle::new(m.frame_fill, m.frame_outline, m.frame_stroke_width),
        ));
        let bounds = self.gradient;
        let gradient = PanelBox::new(
            bounds.x0 + left,
            bounds.x1 + left,
            bounds.y0 + bottom,
            bounds.y1 + bottom,
        );
        let start = Point2D::new(self.start.x + left, self.start.y + bottom);
        let end = Point2D::new(self.end.x + left, self.end.y + bottom);
        let stops = data.collect_source(self.stops.iter().copied().map(Ok))?;
        calls.push(LegendCall::Gradient(gradient, start, end, stops));
        calls.push(LegendCall::Text(translate(&self.title)));
        for label in &self.labels {
            calls.push(LegendCall::Text(translate(label)));
        }
        if let Some((bounds, fill, text)) = &self.missing {
            let corners = [
                Point2D::new(left + bounds.x0, bottom + bounds.y0),
                Point2D::new(left + bounds.x1, bottom + bounds.y0),
                Point2D::new(left + bounds.x1, bottom + bounds.y1),
                Point2D::new(left + bounds.x0, bottom + bounds.y1),
            ];
            calls.push(LegendCall::Fill(corners, fill.clone()));
            calls.push(LegendCall::Text(translate(text)));
        }
        Ok(LegendDrawPlan { calls })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decorations::legend::descriptor::{MissingKey, Palette};

    fn descriptor() -> ContinuousDescriptor {
        ContinuousDescriptor {
            field: "signal".into(),
            title: "Signal".into(),
            palette: Palette {
                low: (49, 130, 189),
                center: Some((247, 247, 247)),
                high: (222, 45, 38),
            },
            center: Some(0.0),
            missing: Some(MissingKey {
                fill: (224, 224, 224),
                outline: (120, 120, 120),
                label: "Missing".into(),
            }),
            domain: (-2.0, 3.0),
            exact: false,
            labels: vec!["-2".into(), "0".into(), "3".into()],
        }
    }

    #[test]
    fn projected_extents_require_the_exact_center_and_endpoint_clearance() {
        assert_eq!(
            projected_minimum(&[4.0, 6.0, 10.0], 5.0, 40.0).to_bits(),
            42.0_f64.to_bits()
        );
        assert_eq!(
            projected_minimum(&[3.0], 5.0, 40.0).to_bits(),
            40.0_f64.to_bits()
        );
    }

    #[test]
    fn horizontal_and_vertical_guides_freeze_one_gradient_and_missing_block() {
        let theme = crate::Theme::Publication.resolve();
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
        let theme = crate::Theme::Publication.resolve();
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
    fn compact_gradient_keeps_all_labels_and_skips_duplicate_allocations() {
        let mut theme = crate::Theme::Publication.resolve();
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
