//! Closed axis inputs, separate from selected ticks and pixel geometry.

use super::AxisSpec;
use crate::spec::{AxisDomain, AxisScale, PlotKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum RangeProvenance {
    #[default]
    Automatic,
    Configured,
    Shared,
}

/// Automatic range rule; configured/shared provenance suppresses local broadening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RangePolicy {
    ContinuousNice,
    AnchoredZero,
    HistogramEdges,
    QqFinite,
    PointIntervalFinite,
    MarginalSupport,
    CategorySlots,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AxisSemanticInputs {
    pub(crate) provenance: RangeProvenance,
    pub(crate) scale: AxisScale,
    pub(crate) policy: RangePolicy,
    pub(crate) interval: Option<super::TickInterval>,
}

impl AxisSemanticInputs {
    pub(crate) fn new(spec: &AxisSpec, scale: AxisScale, policy: RangePolicy) -> Self {
        Self {
            provenance: spec.provenance,
            scale,
            policy,
            interval: spec.interval,
        }
    }

    pub(crate) fn observe_feedback(
        self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), crate::Error> {
        round.observe_count(self.provenance as usize)?;
        round.observe_count(self.scale as usize)?;
        round.observe_count(self.policy as usize)?;
        match self.interval {
            None => round.observe_count(0),
            Some(super::TickInterval::Additive(step)) => {
                round.observe_count(1)?;
                round.observe_number(step)
            }
            Some(super::TickInterval::Log10Decades(step)) => {
                round.observe_count(2)?;
                round.observe_count((step >> 32) as usize)?;
                round.observe_count((step & 0xffff_ffff) as usize)
            }
        }
    }
}

impl crate::Plot {
    pub(crate) fn axis_semantic_inputs(&self, domains: [AxisDomain; 2]) -> [AxisSemanticInputs; 2] {
        let policies = match self.kind() {
            PlotKind::Bar | PlotKind::StackedBar | PlotKind::Density => {
                [RangePolicy::ContinuousNice, RangePolicy::AnchoredZero]
            }
            PlotKind::HorizontalBar | PlotKind::StackedHorizontalBar => {
                [RangePolicy::AnchoredZero, RangePolicy::CategorySlots]
            }
            PlotKind::Histogram => [RangePolicy::HistogramEdges, RangePolicy::AnchoredZero],
            PlotKind::Qq => [RangePolicy::QqFinite; 2],
            PlotKind::PointInterval => [RangePolicy::PointIntervalFinite; 2],
            PlotKind::Marginal => [RangePolicy::MarginalSupport; 2],
            _ => [RangePolicy::ContinuousNice; 2],
        };
        std::array::from_fn(|index| {
            AxisSemanticInputs::new(
                if index == 0 {
                    self.x_axis_spec()
                } else {
                    self.y_axis_spec()
                },
                if index == 0 && self.kind() == PlotKind::Forest {
                    self.axis_scale_spec()
                } else {
                    AxisScale::Linear
                },
                if domains[index] == AxisDomain::Categorical {
                    RangePolicy::CategorySlots
                } else {
                    policies[index]
                },
            )
        })
    }
}

/// T5a supplies typography and preferred aspect; mark/density inputs belong to T5c.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StandardPanelInputs {
    pub(crate) axes: crate::theme::AxisTheme,
    pub(crate) tick_label: crate::theme::TextTheme,
    pub(crate) axis_title: crate::theme::TextTheme,
    preferred_aspect: [u8; 2],
}

impl StandardPanelInputs {
    pub(crate) fn new(theme: &crate::theme::ResolvedTheme) -> Result<Self, crate::Error> {
        let retain = |source: &crate::theme::TextTheme| -> Result<_, crate::Error> {
            let stack = source
                .stack
                .copy_with_generation(&theme.prepared_font_environment()?.generation_resources())
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            crate::theme::TextTheme::new(
                stack,
                source.color(),
                source.font_size(),
                source.weight(),
                source.line_height(),
            )
        };
        Ok(Self {
            axes: theme
                .axes
                .share(&theme.prepared_font_environment()?.generation_resources())?,
            tick_label: retain(&theme.typography.axes.tick_label)?,
            axis_title: retain(&theme.typography.axes.axis_title)?,
            preferred_aspect: [4, 3],
        })
    }

    pub(crate) fn matches(
        &self,
        theme: &crate::theme::ResolvedTheme,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<bool, crate::Error> {
        Ok(self.axes.same(&theme.axes, resources)?
            && self
                .tick_label
                .same(&theme.typography.axes.tick_label, resources)?
            && self
                .axis_title
                .same(&theme.typography.axes.axis_title, resources)?)
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), crate::Error> {
        self.axes.observe_feedback(round)?;
        for text in [&self.tick_label, &self.axis_title] {
            round.observe_font_stack(&text.stack)?;
            round.observe_count(text.font_size as usize)?;
            round.observe_count(text.weight as usize)?;
            round.observe_number(text.line_height_multiplier)?;
            for channel in [
                text.paint.red(),
                text.paint.green(),
                text.paint.blue(),
                text.paint.alpha(),
            ] {
                round.observe_count(usize::from(channel))?;
            }
        }
        for value in self.preferred_aspect {
            round.observe_count(usize::from(value))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_panel_owns_complete_typography_and_four_to_three_preference() {
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let standard = StandardPanelInputs::new(&theme).unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        assert_eq!(standard.preferred_aspect, [4, 3]);
        assert!(standard.matches(&theme, &resources).unwrap());
        theme.typography.axes.tick_label.line_height_multiplier += 0.25;
        assert!(!standard.matches(&theme, &resources).unwrap());
        theme.typography.axes.tick_label = standard.tick_label.clone();
        theme.typography.axes.axis_title.weight = crate::TextWeight::Bold;
        theme.typography.axes.axis_title.stack = crate::font::canonical::FontStack::new(
            theme
                .typography
                .axes
                .axis_title
                .stack
                .families()
                .iter()
                .cloned()
                .chain([crate::font::canonical::FontFamily::generic(
                    crate::font::canonical::GenericFontFamily::Serif,
                )]),
        )
        .unwrap();
        assert!(!standard.matches(&theme, &resources).unwrap());
        assert_ne!(
            standard.axis_title.stack,
            theme.typography.axes.axis_title.stack
        );
    }

    #[test]
    fn standard_panel_stack_mismatch_admits_its_terminating_byte() {
        use crate::font::canonical::{named_stack, OperationGenerationResources};
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.typography.axes.tick_label.stack = named_stack(&["αZ"]);
        let standard = StandardPanelInputs::new(&theme).unwrap();
        theme.typography.axes.tick_label.stack = named_stack(&["αY"]);
        for remaining in [0, 12, 13] {
            let resources = OperationGenerationResources::with_test_limits(remaining, 0);
            let result = standard.matches(&theme, &resources);
            if remaining == 13 {
                assert!(!result.unwrap());
            } else {
                assert!(result.is_err());
            }
            assert_eq!(resources.usage().unwrap().1, remaining);
            assert_eq!(resources.byte_usage().unwrap(), 0);
        }
    }

    #[test]
    fn automatic_display_and_planned_limits_preserve_semantics() {
        let automatic = AxisSpec::default();
        let displayed = super::super::display_axis_spec(&automatic, (-1.0, 11.0));
        let configured = AxisSpec::default().limits(-1.0, 11.0);
        assert_eq!(displayed.limits, configured.limits);
        assert_eq!(displayed.provenance, RangeProvenance::Automatic);
        assert_eq!(configured.provenance, RangeProvenance::Configured);
        let forest = crate::Plot::forest(
            crate::CategoryValues::new("term", ["a"]).unwrap(),
            crate::NumericValues::new("estimate", [1.0]),
            crate::Bounds::new(
                crate::NumericValues::new("lower", [0.1]),
                crate::NumericValues::new("upper", [10.0]),
            ),
            1.0,
        )
        .scale(AxisScale::Log10)
        .erase()
        .with_resolved_facet_ranges(PlotKind::Forest, Some((-2.0, 2.0)), None);
        let inputs = forest.axis_semantic_inputs([AxisDomain::Continuous, AxisDomain::Categorical]);
        assert_eq!(inputs[0].provenance, RangeProvenance::Shared);
        assert_eq!(inputs[0].scale, AxisScale::Log10);
        assert_eq!(inputs[0].policy, RangePolicy::ContinuousNice);
        assert_eq!(inputs[1].policy, RangePolicy::CategorySlots);
        let planned = forest.with_planned_x_limits((0.01, 100.0));
        assert_eq!(
            inputs,
            planned.axis_semantic_inputs([AxisDomain::Continuous, AxisDomain::Categorical])
        );
    }
}
