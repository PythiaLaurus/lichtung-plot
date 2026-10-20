use super::*;
use crate::decorations::legend::discrete::scoped::{
    ConsumedGuideDependency, DependencyUse, GuideAllocation, GuideOrientation,
};
use crate::decorations::legend::discrete::{LegendComposition, PreparedDiscreteGuide};
use crate::layout::fit::{self, FitScope};
use crate::layout::kernel::{FitKind, TrialOutcome};
use crate::theme::ResolvedTheme;

fn parent_guide(field: &str, theme: &ResolvedTheme) -> Arc<PreparedDiscreteGuide> {
    let plot = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .hue(crate::CategoryValues::new(field, ["control", "treated"]).unwrap())
    .labels(crate::Labels::raw().alias(field, "Cohort"))
    .erase();
    PreparedDiscreteGuide::prepare_for_test(&plot, theme, LegendComposition::Combined)
        .unwrap()
        .unwrap()
}

fn measure(
    children: &mut ChildPlans<'_>,
    source: &Plot,
    context: PlotRenderContext<'_>,
    owner: &ComponentAddress,
    round: &FeedbackRound,
    produced: &std::cell::Cell<usize>,
) -> Result<Arc<PlotRenderResult>, Error> {
    fit::measured(children.measure(
        source,
        context,
        owner,
        MeasurementPhase::Legend,
        round,
        || {
            produced.set(produced.get() + 1);
            Ok(FitOutcome::Measured(PlotRenderResult::new(
                crate::layout::Layout::envelope(context.panel),
                None,
            )))
        },
    ))
    .map_err(fit::PlanError::into_error)
}

fn descriptor_use(
    descriptor: &Arc<PreparedDiscreteGuide>,
    orientation: GuideOrientation,
) -> DependencyUse {
    DependencyUse::DescriptorAndAllocation {
        descriptor: Arc::clone(descriptor),
        orientation,
        compact: false,
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one actual scoped child cache distinguishes consumed descriptors/allocations, selective ancestor stability and literal font inputs"
)]
fn actual_child_hits_require_exact_consumed_guides_and_stable_declared_ancestors() {
    let source = Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .erase();
    let theme = crate::Theme::publication().resolve().unwrap();
    let first_guide = parent_guide("first field", &theme);
    let second_guide = parent_guide("second field", &theme);
    let guide_owner = ComponentAddress::root()
        .child(AddressRole::CanvasSharedLegend, 0)
        .unwrap();
    let owner = ComponentAddress::root()
        .child(AddressRole::FacetCell, 0)
        .unwrap();
    crate::layout::plan_exact_root(
        800,
        600,
        |_| Ok(()),
        |(), assigned, round| {
            let mut children = ChildPlans::facets(&source);
            let produced = std::cell::Cell::new(0);
            let panel = PanelBox::new(20.0, 400.0, 20.0, 500.0);
            let allocation = GuideAllocation::bounds(panel)?;
            let first = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                descriptor_use(&first_guide, GuideOrientation::Vertical),
                allocation,
                round,
            )?];
            let first_view = GuideConsumption {
                use_kind: GuideUse::SharedSuppressed,
                local_guide: None,
                dependencies: &first,
            };
            let context =
                PlotRenderContext::new(source.data(), panel, &theme).with_guides(&first_view);
            let original = measure(&mut children, &source, context, &owner, round, &produced)?;
            let hit = measure(&mut children, &source, context, &owner, round, &produced)?;
            assert!(Arc::ptr_eq(&original, &hit));

            // Aliased titles/labels and all glyphs agree; only complete raw/source
            // identities differ. An equal containing box cannot preserve this hit.
            let second = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                descriptor_use(&second_guide, GuideOrientation::Vertical),
                allocation,
                round,
            )?];
            let second_view = GuideConsumption {
                dependencies: &second,
                ..first_view
            };
            let changed = measure(
                &mut children,
                &source,
                context.with_guides(&second_view),
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&original, &changed));
            assert_eq!(
                original.layout.projection.pixel_box(),
                changed.layout.projection.pixel_box()
            );
            let horizontal = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                descriptor_use(&second_guide, GuideOrientation::Horizontal),
                allocation,
                round,
            )?];
            let horizontal_view = GuideConsumption {
                dependencies: &horizontal,
                ..first_view
            };
            let rotated = measure(
                &mut children,
                &source,
                context.with_guides(&horizontal_view),
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&changed, &rotated));
            let compact = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                DependencyUse::DescriptorAndAllocation {
                    descriptor: Arc::clone(&second_guide),
                    orientation: GuideOrientation::Horizontal,
                    compact: true,
                },
                allocation,
                round,
            )?];
            let compact_view = GuideConsumption {
                dependencies: &compact,
                ..first_view
            };
            let compacted = measure(
                &mut children,
                &source,
                context.with_guides(&compact_view),
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&rotated, &compacted));

            let only = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                DependencyUse::AllocationOnly,
                allocation,
                round,
            )?];
            let only_view = GuideConsumption {
                dependencies: &only,
                ..first_view
            };
            let only_context = context.with_guides(&only_view);
            let allocated = measure(
                &mut children,
                &source,
                only_context,
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&compacted, &allocated));
            let equal = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                DependencyUse::AllocationOnly,
                allocation,
                round,
            )?];
            let equal_view = GuideConsumption {
                dependencies: &equal,
                ..first_view
            };
            assert!(Arc::ptr_eq(
                &allocated,
                &measure(
                    &mut children,
                    &source,
                    context.with_guides(&equal_view),
                    &owner,
                    round,
                    &produced
                )?
            ));
            let width = [ConsumedGuideDependency::new(
                guide_owner.clone(),
                DependencyUse::AllocationOnly,
                GuideAllocation::width(panel.width())?,
                round,
            )?];
            let width_view = GuideConsumption {
                dependencies: &width,
                ..first_view
            };
            let width_only = measure(
                &mut children,
                &source,
                context.with_guides(&width_view),
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&allocated, &width_only));
            let baseline = measure(
                &mut children,
                &source,
                only_context,
                &owner,
                round,
                &produced,
            )?;

            let unrelated = ComponentAddress::root().child(AddressRole::FacetCell, 1)?;
            let trial = round.begin_trial()?;
            round.begin_owner(&unrelated)?;
            round.observe_number(1.0)?;
            round.request_insets(
                &unrelated,
                &unrelated,
                crate::layout::kernel::FeedbackKind::AxisGutter,
                [1.0, 0.0, 0.0, 0.0],
            )?;
            let hit = measure(
                &mut children,
                &source,
                only_context,
                &owner,
                round,
                &produced,
            )?;
            assert!(Arc::ptr_eq(&baseline, &hit));
            round.record_failure(
                FitScope::new(&unrelated, 0)
                    .failure(FitKind::TrackOrResidual, "reject unrelated trial"),
            )?;
            assert!(matches!(round.finish_trial(trial), TrialOutcome::Miss(_)));

            let trial = round.begin_trial()?;
            round.begin_owner(&guide_owner)?;
            round.observe_number(2.0)?;
            round.request_insets(
                &guide_owner,
                &guide_owner,
                crate::layout::kernel::FeedbackKind::AxisGutter,
                [1.0, 0.0, 0.0, 0.0],
            )?;
            let unstable = measure(
                &mut children,
                &source,
                only_context,
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&baseline, &unstable));
            round.record_failure(
                FitScope::new(&guide_owner, 0)
                    .failure(FitKind::TrackOrResidual, "reject consumed ancestor trial"),
            )?;
            assert!(matches!(round.finish_trial(trial), TrialOutcome::Miss(_)));

            let mut different_font = theme.clone();
            different_font.typography.axes.tick_label.stack =
                crate::font::canonical::FontStack::new(
                    [crate::font::canonical::FontFamily::named("Noto Sand")?]
                        .into_iter()
                        .chain(
                            theme.typography.axes.tick_label.stack.families()[1..]
                                .iter()
                                .cloned(),
                        ),
                )?;
            let changed_font = PlotRenderContext::new(source.data(), panel, &different_font)
                .with_guides(&only_view);
            let changed = measure(
                &mut children,
                &source,
                changed_font,
                &owner,
                round,
                &produced,
            )?;
            assert!(!Arc::ptr_eq(&unstable, &changed));
            assert_eq!(produced.get(), 9);
            Ok(crate::layout::MeasuredLayout::new(assigned, ()))
        },
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}
