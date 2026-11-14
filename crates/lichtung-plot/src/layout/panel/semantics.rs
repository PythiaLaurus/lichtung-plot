//! Closed axis inputs, separate from selected ticks and pixel geometry.

use crate::orchestrator::registration::AxisIntentRef;
use crate::spec::{AxisScale, PlotKind};

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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisSemanticInputs {
    pub(crate) text_layout: Option<crate::TextLayout>,
    pub(crate) provenance: RangeProvenance,
    pub(crate) scale: AxisScale,
    pub(crate) policy: RangePolicy,
    pub(crate) generation: crate::TickGeneration,
    pub(crate) category_labels: crate::CategoryLabelSelection,
    pub(crate) format: Option<std::sync::Arc<crate::numeric::format::NumericFormat>>,
}

impl AxisSemanticInputs {
    pub(crate) fn new(
        intent: AxisIntentRef<'_>,
        provenance: RangeProvenance,
        scale: AxisScale,
        policy: RangePolicy,
    ) -> Self {
        let (generation, category_labels, format) = match intent {
            AxisIntentRef::Authored(spec) => (
                spec.generation.clone(),
                spec.category_labels.clone(),
                if spec.exact_ticks.is_some() || policy == RangePolicy::CategorySlots {
                    None
                } else {
                    spec.format.clone()
                },
            ),
            AxisIntentRef::MatrixVariable | AxisIntentRef::NativeAmplitude => (
                crate::TickGeneration::automatic(),
                crate::CategoryLabelSelection::all(),
                None,
            ),
        };
        Self {
            text_layout: None,
            provenance,
            scale,
            policy,
            generation,
            category_labels,
            format,
        }
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), crate::Error> {
        round.observe_count(self.provenance as usize)?;
        round.observe_count(self.scale as usize)?;
        round.observe_count(self.policy as usize)?;
        round.observe_count(usize::from(self.format.is_some()))?;
        if let Some(format) = &self.format {
            format.observe_feedback(round)?;
        }
        self.category_labels.observe(round)?;
        round.observe_text_layout(self.text_layout.as_ref())?;
        self.generation.observe_feedback(round)
    }
}

pub(crate) fn range_policies(kind: PlotKind) -> [RangePolicy; 2] {
    match kind {
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
    }
}

/// T5a supplies typography and preferred aspect; mark/density inputs belong to T5c.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CoordinateLayoutInputs {
    tick_layouts: [Option<crate::TextLayout>; 2],
    pub(crate) axes: crate::theme::AxisTheme,
    pub(crate) tick_label: crate::theme::TextTheme,
    pub(crate) axis_title: crate::theme::TextTheme,
    preferred_aspect: [u8; 2],
}

impl CoordinateLayoutInputs {
    pub(crate) fn new(theme: &crate::theme::ResolvedTheme) -> Result<Self, crate::Error> {
        let retain = |source: &crate::theme::TextTheme| -> Result<_, crate::Error> {
            let stack = source
                .stack
                .try_copy()
                .map_err(|error| crate::Error::Render(error.to_string()))?;
            crate::theme::TextTheme::new(
                stack,
                source.get_color(),
                source.get_font_size(),
                source.get_weight(),
                source.get_line_height(),
            )
        };
        Ok(Self {
            axes: theme.axes.clone(),
            tick_layouts: theme.text_layouts.ticks.clone(),
            tick_label: retain(&theme.typography.axes.tick_label)?,
            axis_title: retain(&theme.typography.axes.axis_title)?,
            preferred_aspect: [4, 3],
        })
    }

    pub(crate) fn matches(
        &self,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<bool, crate::Error> {
        Ok(self.axes.same_validated(&theme.axes)?
            && self.tick_layouts == theme.text_layouts.ticks
            && self.tick_label.same(&theme.typography.axes.tick_label)
            && self.axis_title.same(&theme.typography.axes.axis_title))
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), crate::Error> {
        self.axes.observe_feedback(round)?;
        for layout in &self.tick_layouts {
            round.observe_text_layout(layout.as_ref())?;
        }
        for text in [&self.tick_label, &self.axis_title] {
            round.observe_font_stack(&text.stack)?;
            round.observe_number(text.font_size)?;
            round.observe_count(text.weight as usize)?;
            round.observe_number(text.line_height_multiplier)?;
            for channel in [
                text.paint.red(),
                text.paint.green(),
                text.paint.blue(),
                text.paint.get_alpha(),
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
    fn standard_panel_stack_matching_includes_the_complete_utf8_name() {
        use crate::font::canonical::named_stack;
        let mut theme = crate::Theme::publication().resolve().unwrap();
        theme.typography.axes.tick_label.stack = named_stack(&["αZ"]);
        let standard = CoordinateLayoutInputs::new(&theme).unwrap();
        theme.typography.axes.tick_label.stack = named_stack(&["αY"]);
        assert!(!standard.matches(&theme).unwrap());
    }

    #[test]
    fn automatic_display_and_planned_limits_preserve_semantics() {
        let appearance = crate::Theme::publication().resolve().unwrap();
        let semantic = |source: &crate::builder::PlotDescription,
                        ranges: [(f64, f64); 2],
                        provenance: [RangeProvenance; 2]| {
            let operation = crate::font::canonical::OperationContext::new();
            let (registry, _) =
                crate::plots::shared_x::prepare_plot(source, &operation, &appearance).unwrap();
            let panel = &registry.panels[0];
            let axes = panel.axes.as_ref().unwrap();
            std::array::from_fn::<_, 2, _>(|side| {
                let domain = crate::orchestrator::assignment::AxisDomainAssignment {
                    extent: crate::orchestrator::registration::DisplayExtent {
                        lower: ranges[side].0,
                        upper: ranges[side].1,
                    },
                    provenance: provenance[side],
                };
                crate::orchestrator::assignment::AssignedAxisRef {
                    base: axes.axis(source, side),
                    domain: &domain,
                }
                .semantic_inputs()
            })
        };
        let automatic = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 10.0]),
            crate::NumericValues::new("y", [1.0, 2.0]),
        );
        let configured = automatic.clone().x_limits(-1.0, 11.0);
        let displayed = semantic(
            automatic.description(),
            [(-1.0, 11.0), (1.0, 2.0)],
            [RangeProvenance::Automatic; 2],
        );
        assert_eq!(displayed[0].provenance, RangeProvenance::Automatic);
        assert!(automatic.description().x_axis_spec().window.is_none());
        let configured_inputs = semantic(
            configured.description(),
            [(-1.0, 11.0), (1.0, 2.0)],
            [RangeProvenance::Configured, RangeProvenance::Automatic],
        );
        assert_eq!(configured_inputs[0].provenance, RangeProvenance::Configured);
        let forest = crate::Plot::forest(
            crate::CategoryValues::new("term", ["a"]).unwrap(),
            crate::NumericValues::new("estimate", [1.0]),
            crate::Bounds::new(
                crate::NumericValues::new("lower", [0.1]),
                crate::NumericValues::new("upper", [10.0]),
            ),
            1.0,
        )
        .scale(AxisScale::Log10);
        let inputs = semantic(
            forest.description(),
            [(-2.0, 2.0), (0.0, 1.0)],
            [RangeProvenance::Shared, RangeProvenance::Automatic],
        );
        assert_eq!(inputs[0].provenance, RangeProvenance::Shared);
        assert_eq!(inputs[0].scale, AxisScale::Log10);
        assert_eq!(inputs[0].policy, RangePolicy::ContinuousNice);
        assert_eq!(inputs[1].policy, RangePolicy::CategorySlots);
        let planned = semantic(
            forest.description(),
            [(-4.0, 4.0), (0.0, 1.0)],
            [RangeProvenance::Shared, RangeProvenance::Automatic],
        );
        assert_eq!(inputs, planned);
        assert!(forest.description().x_axis_spec().window.is_none());
    }
}
