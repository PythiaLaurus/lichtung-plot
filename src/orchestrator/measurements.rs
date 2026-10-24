//! Opt-in, thread-local observations for the ignored release baseline.
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(crate) enum Counter {
    GuideTrial,
    GuideBodyCheck,
    InsideLegendCandidate,
    GeometryOverlapScan,
    SignificanceLevelCheck,
    SignificanceRangeGrowth,
    SignificanceBisection,
    EllipsisViewTrial,
    FontScalarProbe,
    FontUnitProbe,
    MatrixLayoutPass,
    TickSource,
    TickFitMiss,
    TickDistinctList,
    TickMetricLookup,
    TextProposalVisit,
    TextExactTrial,
    ChildCacheHit,
    ChildCacheMiss,
    ChildFeedbackReplay,
    AnnotationOverlapComparison,
    AxisRangePreparation,
    AxisSpecPreparation,
    AxisSourcePreparation,
    StandardPanelIntrinsic,
    RootTitleIntrinsic,
    HeatmapIntrinsic,
    LegendSelection,
    StandardCoordinateMeasure,
    LegacyPanelConstructor,
    OneCategoricalAxisMeasure,
    ForestRawTickProjection,
}

impl Counter {
    pub(crate) const ALL: [Self; 32] = [
        Self::GuideTrial,
        Self::GuideBodyCheck,
        Self::InsideLegendCandidate,
        Self::GeometryOverlapScan,
        Self::SignificanceLevelCheck,
        Self::SignificanceRangeGrowth,
        Self::SignificanceBisection,
        Self::EllipsisViewTrial,
        Self::FontScalarProbe,
        Self::FontUnitProbe,
        Self::MatrixLayoutPass,
        Self::TickSource,
        Self::TickFitMiss,
        Self::TickDistinctList,
        Self::TickMetricLookup,
        Self::TextProposalVisit,
        Self::TextExactTrial,
        Self::ChildCacheHit,
        Self::ChildCacheMiss,
        Self::ChildFeedbackReplay,
        Self::AnnotationOverlapComparison,
        Self::AxisRangePreparation,
        Self::AxisSpecPreparation,
        Self::AxisSourcePreparation,
        Self::StandardPanelIntrinsic,
        Self::RootTitleIntrinsic,
        Self::HeatmapIntrinsic,
        Self::LegendSelection,
        Self::StandardCoordinateMeasure,
        Self::LegacyPanelConstructor,
        Self::OneCategoricalAxisMeasure,
        Self::ForestRawTickProjection,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::GuideTrial => "GuideTrial",
            Self::GuideBodyCheck => "GuideBodyCheck",
            Self::InsideLegendCandidate => "InsideLegendCandidate",
            Self::GeometryOverlapScan => "GeometryOverlapScan",
            Self::SignificanceLevelCheck => "SignificanceLevelCheck",
            Self::SignificanceRangeGrowth => "SignificanceRangeGrowth",
            Self::SignificanceBisection => "SignificanceBisection",
            Self::EllipsisViewTrial => "EllipsisViewTrial",
            Self::FontScalarProbe => "FontScalarProbe",
            Self::FontUnitProbe => "FontUnitProbe",
            Self::MatrixLayoutPass => "MatrixLayoutPass",
            Self::TickSource => "TickSource",
            Self::TickFitMiss => "TickFitMiss",
            Self::TickDistinctList => "TickDistinctList",
            Self::TickMetricLookup => "TickMetricLookup",
            Self::TextProposalVisit => "TextProposalVisit",
            Self::TextExactTrial => "TextExactTrial",
            Self::ChildCacheHit => "ChildCacheHit",
            Self::ChildCacheMiss => "ChildCacheMiss",
            Self::ChildFeedbackReplay => "ChildFeedbackReplay",
            Self::AnnotationOverlapComparison => "AnnotationOverlapComparison",
            Self::AxisRangePreparation => "AxisRangePreparation",
            Self::AxisSpecPreparation => "AxisSpecPreparation",
            Self::AxisSourcePreparation => "AxisSourcePreparation",
            Self::StandardPanelIntrinsic => "StandardPanelIntrinsic",
            Self::RootTitleIntrinsic => "RootTitleIntrinsic",
            Self::HeatmapIntrinsic => "HeatmapIntrinsic",
            Self::LegendSelection => "LegendSelection",
            Self::StandardCoordinateMeasure => "StandardCoordinateMeasure",
            Self::LegacyPanelConstructor => "LegacyPanelConstructor",
            Self::OneCategoricalAxisMeasure => "OneCategoricalAxisMeasure",
            Self::ForestRawTickProjection => "ForestRawTickProjection",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GuideOutcome {
    Rejected,
    /// The local trial requests feedback; this is not a final placement.
    Provisional,
    /// The local trial fits; an enclosing trial or root can still reject it.
    Accepted,
    Aborted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GuidePhase {
    GuideOnly,
    /// The guide measured successfully; allocation/body checks are now running.
    Body,
}

#[derive(Clone, Debug)]
pub(crate) struct GuideAttempt {
    pub owner: crate::layout::ComponentAddress,
    pub root_round: usize,
    pub edge: crate::LegendPosition,
    pub compact: bool,
    pub outcome: GuideOutcome,
    pub phase: GuidePhase,
    pub elapsed: Duration,
}

/// The guide slice belongs to this round, not necessarily to the final selection.
#[derive(Clone, Debug)]
pub(crate) struct RootRound {
    pub ordinal: usize,
    pub ellipsis_view_trials: u64,
    pub guides: std::ops::Range<usize>,
    /// Set only for the terminal round after the exact root freezes successfully.
    pub successful: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ChildCacheEvent {
    pub owner: crate::layout::ComponentAddress,
    pub phase: &'static str,
    pub root_round: usize,
    pub counter: Counter,
}

#[derive(Clone, Debug)]
pub(crate) struct IntrinsicEvent {
    pub scope: Option<String>,
    pub counter: Counter,
    pub root_round: Option<usize>,
}

pub(crate) fn intrinsic(counter: Counter, scope: Option<&str>) {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.counters[counter as usize] += 1;
            observation.intrinsics.push(IntrinsicEvent {
                scope: scope.map(str::to_owned),
                counter,
                root_round: observation.roots.last().map(|root| root.ordinal),
            });
        }
    });
}

#[derive(Clone, Debug)]
pub(crate) struct GutterChange {
    pub owner: crate::layout::ComponentAddress,
    pub consumed: [f64; 4],
    pub requested: [f64; 4],
}

pub(crate) fn gutter_change(
    owner: &crate::layout::ComponentAddress,
    consumed: [f64; 4],
    requested: [f64; 4],
) {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.gutters.push(GutterChange {
                owner: owner.clone(),
                consumed,
                requested,
            });
        }
    });
}

pub(crate) fn child_cache(
    owner: &crate::layout::ComponentAddress,
    phase: &'static str,
    counter: Counter,
) {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.counters[counter as usize] += 1;
            observation.children.push(ChildCacheEvent {
                owner: owner.clone(),
                phase,
                root_round: observation.roots.last().map_or(0, |root| root.ordinal),
                counter,
            });
        }
    });
}

pub(crate) const STAGES: [&str; 8] = [
    "prepare_including_fonts",
    "root_measure",
    "capture",
    "null_validation",
    "publication",
    "svg_outline",
    "png_raster_encode",
    "svg_replay_serialization",
];

#[derive(Clone, Default)]
pub(crate) struct Observations {
    pub times: [Duration; 8],
    pub calls: [usize; 8],
    pub producers: usize,
    pub counters: [u64; Counter::ALL.len()],
    pub capture_counters: [u64; Counter::ALL.len()],
    pub guides: Vec<GuideAttempt>,
    pub roots: Vec<RootRound>,
    pub children: Vec<ChildCacheEvent>,
    pub intrinsics: Vec<IntrinsicEvent>,
    pub gutters: Vec<GutterChange>,
    session: Rc<()>,
}

pub(crate) fn count(counter: Counter) {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.counters[counter as usize] += 1;
            if counter == Counter::EllipsisViewTrial {
                if let Some(root) = observation.roots.last_mut() {
                    root.ellipsis_view_trials += 1;
                }
            }
        }
    });
}

pub(crate) fn root_round(ordinal: usize) -> Timer {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            if let Some(previous) = observation.roots.last_mut() {
                previous.guides.end = observation.guides.len();
            }
            observation.roots.push(RootRound {
                ordinal,
                ellipsis_view_trials: 0,
                guides: observation.guides.len()..observation.guides.len(),
                successful: false,
            });
        }
    });
    Timer::new(1)
}

pub(crate) fn root_success() {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            if let Some(root) = observation.roots.last_mut() {
                root.successful = true;
                root.guides.end = observation.guides.len();
            }
        }
    });
}

/// Called immediately after the Canvas guide itself fits, before any child trials.
pub(crate) fn guide_body() {
    ACTIVE.with(|active| {
        if let Some(observation) = active
            .borrow_mut()
            .as_mut()
            .and_then(|observation| observation.guides.last_mut())
        {
            observation.phase = GuidePhase::Body;
        }
    });
}

pub(crate) struct GuideTrial(Option<(usize, Instant, Rc<()>)>);

impl GuideTrial {
    pub(crate) fn new(
        owner: &crate::layout::ComponentAddress,
        edge: crate::LegendPosition,
        compact: bool,
    ) -> Self {
        Self(ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            let observation = active.as_mut()?;
            observation.counters[Counter::GuideTrial as usize] += 1;
            let index = observation.guides.len();
            observation.guides.push(GuideAttempt {
                owner: owner.clone(),
                root_round: observation.roots.last().map_or(0, |root| root.ordinal),
                edge,
                compact,
                outcome: GuideOutcome::Aborted,
                phase: GuidePhase::GuideOnly,
                elapsed: Duration::ZERO,
            });
            Some((index, Instant::now(), Rc::clone(&observation.session)))
        }))
    }

    pub(crate) fn body(&self) {
        self.update(|attempt| attempt.phase = GuidePhase::Body);
    }

    pub(crate) fn reject(&self) {
        self.update(|attempt| attempt.outcome = GuideOutcome::Rejected);
    }

    pub(crate) fn complete(&self, outcome: &crate::layout::kernel::TrialOutcome) {
        use crate::layout::kernel::TrialOutcome;
        self.update(|attempt| {
            attempt.outcome = match outcome {
                TrialOutcome::Fit => GuideOutcome::Accepted,
                TrialOutcome::Provisional => GuideOutcome::Provisional,
                TrialOutcome::Miss(_) => GuideOutcome::Rejected,
            }
        });
    }

    fn update(&self, update: impl FnOnce(&mut GuideAttempt)) {
        if let Some((index, _, session)) = &self.0 {
            ACTIVE.with(|active| {
                if let Some(attempt) = active
                    .borrow_mut()
                    .as_mut()
                    .filter(|observation| Rc::ptr_eq(&observation.session, session))
                    .and_then(|observation| observation.guides.get_mut(*index))
                {
                    update(attempt);
                }
            });
        }
    }
}

impl Drop for GuideTrial {
    fn drop(&mut self) {
        if let Some((_, start, _)) = &self.0 {
            self.update(|attempt| attempt.elapsed = start.elapsed());
        }
    }
}

thread_local! {
    static ACTIVE: RefCell<Option<Observations>> = const { RefCell::new(None) };
}

pub(crate) fn start() {
    ACTIVE.with(|active| *active.borrow_mut() = Some(Observations::default()));
}

pub(crate) fn finish() -> Observations {
    let mut observation = ACTIVE.with(|active| active.borrow_mut().take().unwrap());
    if let Some(root) = observation.roots.last_mut() {
        root.guides.end = observation.guides.len();
    }
    observation
}

pub(crate) fn producer() {
    ACTIVE.with(|active| {
        if let Some(observation) = active.borrow_mut().as_mut() {
            observation.producers += 1;
        }
    });
}

pub(crate) struct Timer(usize, Option<(Instant, Rc<()>, [u64; Counter::ALL.len()])>);

impl Timer {
    pub(crate) fn new(stage: usize) -> Self {
        Self(
            stage,
            ACTIVE.with(|active| {
                active.borrow().as_ref().map(|observation| {
                    (
                        Instant::now(),
                        Rc::clone(&observation.session),
                        observation.counters,
                    )
                })
            }),
        )
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if let Some((start, session, counters)) = &self.1 {
            ACTIVE.with(|active| {
                if let Some(observation) = active
                    .borrow_mut()
                    .as_mut()
                    .filter(|observation| Rc::ptr_eq(&observation.session, session))
                {
                    observation.times[self.0] += start.elapsed();
                    observation.calls[self.0] += 1;
                    if self.0 == 2 {
                        for (index, before) in counters.iter().enumerate() {
                            observation.capture_counters[index] +=
                                observation.counters[index] - before;
                        }
                    }
                }
            });
        }
    }
}

#[test]
fn counters_are_opt_in_reset_and_thread_local() {
    count(Counter::GuideTrial);
    assert!(ACTIVE.with(|active| active.borrow().is_none()));
    start();
    for (index, counter) in Counter::ALL.into_iter().enumerate() {
        assert!(!counter.name().is_empty());
        assert_eq!(counter as usize, index);
        count(counter);
    }
    std::thread::spawn(|| {
        count(Counter::GuideTrial);
        assert!(ACTIVE.with(|active| active.borrow().is_none()));
    })
    .join()
    .unwrap();
    assert_eq!(finish().counters, [1; Counter::ALL.len()]);
    start();
    count(Counter::TextExactTrial);
    let old_timer = Timer::new(0);
    let old_trial = GuideTrial::new(
        &crate::layout::ComponentAddress::root(),
        crate::LegendPosition::OutsideRight,
        false,
    );
    start();
    drop(old_timer);
    drop(old_trial);
    let observation = finish();
    assert_eq!(observation.counters, [0; Counter::ALL.len()]);
    assert_eq!(observation.calls, [0; 8]);
    assert!(observation.guides.is_empty());
    assert!(observation.roots.is_empty());
    count(Counter::GuideTrial);
    assert!(ACTIVE.with(|active| active.borrow().is_none()));
}

#[test]
fn guide_guards_preserve_outcomes_and_root_boundaries() {
    use crate::layout::kernel::TrialOutcome;
    let started = Instant::now();
    let owner = crate::layout::ComponentAddress::root();
    let edge = crate::LegendPosition::OutsideRight;
    drop(GuideTrial::new(&owner, edge, false));
    start();
    root_round(0);
    count(Counter::EllipsisViewTrial);
    {
        let trial = GuideTrial::new(&owner, edge, false);
        trial.reject();
    }
    {
        let trial = GuideTrial::new(&owner, edge, true);
        guide_body();
        trial.complete(&TrialOutcome::Provisional);
    }
    root_round(1);
    count(Counter::EllipsisViewTrial);
    count(Counter::EllipsisViewTrial);
    {
        let trial = GuideTrial::new(&owner, edge, false);
        trial.body();
        trial.complete(&TrialOutcome::Fit);
    }
    root_success();
    drop(GuideTrial::new(&owner, edge, true));
    let observation = finish();
    assert_eq!(observation.counters[Counter::GuideTrial as usize], 4);
    assert_eq!(observation.roots[0].guides, 0..2);
    assert_eq!(observation.roots[0].ellipsis_view_trials, 1);
    assert_eq!(observation.roots[1].ellipsis_view_trials, 2);
    assert_eq!(observation.counters[Counter::EllipsisViewTrial as usize], 3);
    assert!(!observation.roots[0].successful);
    assert!(observation.roots[1].successful);
    let attempts = &observation.guides;
    assert_eq!(attempts[0].owner, owner);
    assert_eq!(attempts[0].edge, edge);
    assert!(!attempts[0].compact);
    assert_eq!(attempts[0].root_round, 0);
    assert_eq!(attempts[0].phase, GuidePhase::GuideOnly);
    assert_eq!(attempts[0].outcome, GuideOutcome::Rejected);
    assert_eq!(attempts[1].phase, GuidePhase::Body);
    assert_eq!(attempts[1].outcome, GuideOutcome::Provisional);
    assert_eq!(attempts[2].root_round, 1);
    assert_eq!(attempts[2].outcome, GuideOutcome::Accepted);
    assert_eq!(attempts[3].outcome, GuideOutcome::Aborted);
    assert!(attempts
        .iter()
        .all(|attempt| attempt.elapsed <= started.elapsed()));
}

#[test]
fn child_cache_events_are_opt_in_attributed_and_reset() {
    let owner = crate::layout::ComponentAddress::root();
    child_cache(&owner, "panel", Counter::ChildCacheMiss);
    assert!(ACTIVE.with(|active| active.borrow().is_none()));
    start();
    root_round(3);
    for counter in [
        Counter::ChildCacheMiss,
        Counter::ChildCacheHit,
        Counter::ChildFeedbackReplay,
    ] {
        child_cache(&owner, "panel", counter);
    }
    std::thread::spawn(|| {
        child_cache(
            &crate::layout::ComponentAddress::root(),
            "panel",
            Counter::ChildCacheHit,
        );
        assert!(ACTIVE.with(|active| active.borrow().is_none()));
    })
    .join()
    .unwrap();
    let observation = finish();
    assert_eq!(observation.children.len(), 3);
    for event in observation.children {
        assert_eq!(event.owner, owner);
        assert_eq!(event.phase, "panel");
        assert_eq!(event.root_round, 3);
        assert_eq!(observation.counters[event.counter as usize], 1);
    }
    start();
    assert!(finish().children.is_empty());
}
