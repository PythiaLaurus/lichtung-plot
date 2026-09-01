//! Private complete-candidate layout kernel and exact-root adapter.

use std::cmp::Ordering;
use std::fmt;

use crate::spec::PlotKind;
use crate::Error;

const QUANTA_PER_CSS_PIXEL: i64 = 1024;
const RESOURCE_LIMIT: u128 = 1_000_000;
const CACHE_ENTRY_LIMIT: usize = 1_000_000;
const MAX_COMPLETE_PASSES: usize = 8;

#[cfg(test)]
thread_local! {
    static FORCED_RESERVATION_FAILURE: Cell<Option<u8>> = const { Cell::new(None) };
}

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
pub(crate) fn force_reservation_failure(stage: u8) {
    assert!((1..=5).contains(&stage));
    FORCED_RESERVATION_FAILURE.with(|forced| forced.set(Some(stage)));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutScalar(i64);

#[allow(
    dead_code,
    reason = "the kernel owns all exact binary64-to-quantum rounding categories before later producers activate them"
)]
#[derive(Debug, Clone, Copy)]
enum SpatialRounding {
    Floor,
    Ceil,
    NearestEven,
}

impl LayoutScalar {
    fn exact_css_pixels(value: u32) -> Result<Self, Error> {
        i64::from(value)
            .checked_mul(QUANTA_PER_CSS_PIXEL)
            .map(Self)
            .ok_or_else(|| Error::Render("exact layout dimension overflowed layout quanta".into()))
    }

    fn is_nonnegative(self) -> bool {
        self.0 >= 0
    }

    fn canonical_dimension(self) -> Option<u32> {
        let value = u128::try_from(self.0).ok()?;
        let quantum = QUANTA_PER_CSS_PIXEL as u128;
        let rounded = value.checked_add(quantum - 1)? / quantum;
        u32::try_from(rounded).ok()
    }

    #[allow(
        dead_code,
        reason = "later T5 producers consume the already-tested canonical quantum conversion"
    )]
    fn from_css_pixels(value: f64, rounding: SpatialRounding) -> Result<Self, Error> {
        let bits = value.to_bits();
        let exponent_bits = ((bits >> 52) & 0x7ff) as u16;
        if exponent_bits == 0x7ff {
            return Err(Error::Render("layout spatial input must be finite".into()));
        }
        let fraction = bits & 0x000f_ffff_ffff_ffff;
        if exponent_bits == 0 && fraction == 0 {
            return Ok(Self(0));
        }
        let negative = bits >> 63 != 0;
        let significand = if exponent_bits == 0 {
            u128::from(fraction)
        } else {
            u128::from((1_u64 << 52) | fraction)
        };
        let shift = if exponent_bits == 0 {
            -1064
        } else {
            i32::from(exponent_bits) - 1065
        };
        let (quotient, remainder_order) = if shift >= 0 {
            let shift = u32::try_from(shift)
                .map_err(|_| Error::Render("layout quantum shift overflowed".into()))?;
            if shift >= u128::BITS || significand > (u128::MAX >> shift) {
                return Err(Error::Render("layout quantum conversion overflowed".into()));
            }
            let quotient = significand << shift;
            (quotient, None)
        } else {
            let denominator_shift = shift.unsigned_abs();
            if denominator_shift >= 128 {
                (0, Some(Ordering::Less))
            } else {
                let denominator = 1_u128 << denominator_shift;
                let quotient = significand / denominator;
                let remainder = significand % denominator;
                let order = (remainder != 0).then(|| remainder.cmp(&(denominator / 2)));
                (quotient, order)
            }
        };
        let increment = match remainder_order {
            None => false,
            Some(order) => match rounding {
                SpatialRounding::Floor => negative,
                SpatialRounding::Ceil => !negative,
                SpatialRounding::NearestEven => {
                    order == Ordering::Greater || (order == Ordering::Equal && quotient % 2 != 0)
                }
            },
        };
        let magnitude = quotient
            .checked_add(u128::from(increment))
            .ok_or_else(|| Error::Render("layout quantum rounding overflowed".into()))?;
        if negative {
            if magnitude == 1_u128 << 63 {
                Ok(Self(i64::MIN))
            } else {
                let magnitude = i64::try_from(magnitude)
                    .map_err(|_| Error::Render("layout quantum conversion overflowed".into()))?;
                Ok(Self(-magnitude))
            }
        } else {
            i64::try_from(magnitude)
                .map(Self)
                .map_err(|_| Error::Render("layout quantum conversion overflowed".into()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LengthConstraint {
    minimum: LayoutScalar,
    maximum: Option<LayoutScalar>,
}

impl LengthConstraint {
    fn new(minimum: LayoutScalar, maximum: Option<LayoutScalar>) -> Result<Self, Error> {
        if !minimum.is_nonnegative() {
            return Err(Error::Render(
                "layout constraint minimum must be nonnegative".into(),
            ));
        }
        if maximum.is_some_and(|maximum| maximum < minimum) {
            return Err(Error::Render(
                "layout constraint maximum must be at least its minimum".into(),
            ));
        }
        Ok(Self { minimum, maximum })
    }

    fn exact_css_pixels(value: u32) -> Result<Self, Error> {
        let value = LayoutScalar::exact_css_pixels(value)?;
        Self::new(value, Some(value))
    }

    fn contains(self, value: LayoutScalar) -> bool {
        value >= self.minimum && self.maximum.is_none_or(|maximum| value <= maximum)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutConstraints {
    width: LengthConstraint,
    height: LengthConstraint,
}

impl LayoutConstraints {
    fn exact_css_pixels(width: u32, height: u32) -> Result<Self, Error> {
        Ok(Self {
            width: LengthConstraint::exact_css_pixels(width)?,
            height: LengthConstraint::exact_css_pixels(height)?,
        })
    }

    fn contains(self, size: LayoutSize) -> bool {
        self.width.contains(size.width) && self.height.contains(size.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutSize {
    width: LayoutScalar,
    height: LayoutScalar,
}

impl LayoutSize {
    fn exact_css_pixels(width: u32, height: u32) -> Result<Self, Error> {
        Self::new(
            LayoutScalar::exact_css_pixels(width)?,
            LayoutScalar::exact_css_pixels(height)?,
        )
    }

    fn new(width: LayoutScalar, height: LayoutScalar) -> Result<Self, Error> {
        if !(width.is_nonnegative() && height.is_nonnegative()) {
            return Err(Error::Render(
                "layout size dimensions must be nonnegative".into(),
            ));
        }
        Ok(Self { width, height })
    }

    fn canonical_dimensions(self) -> Option<(u32, u32)> {
        Some((
            self.width.canonical_dimension()?,
            self.height.canonical_dimension()?,
        ))
    }

    fn checked_area(self) -> Result<u128, Error> {
        let width = u128::try_from(self.width.0).map_err(|_| {
            Error::Render("layout width was negative during area calculation".into())
        })?;
        let height = u128::try_from(self.height.0).map_err(|_| {
            Error::Render("layout height was negative during area calculation".into())
        })?;
        width
            .checked_mul(height)
            .ok_or_else(|| Error::Render("layout area overflowed".into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutBox {
    left: LayoutScalar,
    top: LayoutScalar,
    right: LayoutScalar,
    bottom: LayoutScalar,
}

impl LayoutBox {
    fn root(size: LayoutSize) -> Self {
        Self {
            left: LayoutScalar(0),
            top: LayoutScalar(0),
            right: size.width,
            bottom: size.height,
        }
    }
}

#[allow(
    dead_code,
    reason = "the closed role order is part of canonical structural identity"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AddressRole {
    Root,
    RootTitle,
    RootSubtitle,
    RootBody,
    RootCaption,
    PlotBody,
    CanvasBody,
    CanvasGrid,
    FacetGrid,
    CanvasSharedLegend,
    CanvasHeader,
    CanvasRowTitle,
    CanvasRangeGroup,
    CanvasCell,
    FacetHeader,
    FacetRowTitle,
    FacetRangeGroup,
    FacetCell,
    DataPanel,
    PanelInterior,
    SpecializedInterior,
    MarkLayer,
    MarkItem,
    HeatmapCell,
    HeatmapMergedRow,
    HeatmapValueLabel,
    HeatmapHighlight,
    BeeswarmCategory,
    BeeswarmPoint,
    MarginalCentral,
    MarginalTop,
    MarginalRight,
    MatrixCell,
    MatrixMark,
    CategoricalCategory,
    CategoricalSeriesSlot,
    CategoricalBodyMark,
    CategoricalInterval,
    CategoricalValueLabel,
    WaterfallBar,
    WaterfallConnector,
    WaterfallBaseLabel,
    WaterfallTerminalLabel,
    LegendTitle,
    LegendEntry,
    LegendKey,
    LegendLabel,
    GuideTitle,
    GuideGradient,
    GuideLabel,
    GuideMissingBlock,
    Surface,
    TextPanelContent,
    TextPanelTitle,
    TextPanelBody,
    InsideGuide,
    Annotation,
    AnnotationTarget,
    OtherOverlay,
    AxisTop,
    AxisRight,
    AxisBottom,
    AxisLeft,
    AxisTitle,
    OutsideGuideTop,
    OutsideGuideRight,
    OutsideGuideBottom,
    OutsideGuideLeft,
    OutsideGuideAuto,
    Text,
    Glyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct AddressSegment {
    role: AddressRole,
    slot_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ComponentAddress {
    root: AddressSegment,
    descendants: Box<[AddressSegment]>,
}

impl ComponentAddress {
    fn root() -> Self {
        Self {
            root: AddressSegment {
                role: AddressRole::Root,
                slot_index: 0,
            },
            descendants: Box::new([]),
        }
    }
}

#[allow(
    dead_code,
    reason = "the closed component order is part of canonical structural identity"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ComponentType {
    EmptySlot,
    Canvas,
    Plot,
    PlotBody,
    PlotFrame,
    CanvasBody,
    CanvasGrid,
    FacetGrid,
    TrackSet,
    DataPanel,
    StandardInterior,
    Axis,
    DiscreteLegend,
    ContinuousGuide,
    CompositionText,
    TextPanel,
    TextPanelContent,
    LegendEntry,
    AnnotationLayer,
    OverlayLayer,
    HeatmapInterior,
    BeeswarmInterior,
    MarginalInterior,
    MarginalSubpanel,
    MatrixInterior,
    CategoricalInterior,
    WaterfallInterior,
    MarkLayer,
    Text,
    Glyph,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StructuralRecord {
    address: ComponentAddress,
    component_type: ComponentType,
    plot_kind: Option<u8>,
    constraints: LayoutConstraints,
    candidate_size: LayoutSize,
    track_vectors: Box<[Box<[LayoutScalar]>]>,
    arranged_root_box: LayoutBox,
    baselines: Box<[LayoutScalar]>,
    root_ink_edges: LayoutBox,
    text_decisions: Box<[u8]>,
    tick_decisions: Box<[u8]>,
    legend_and_guide_decisions: Box<[u8]>,
    domain_order_keys: Box<[u64]>,
    aspect_inputs: Box<[(u128, u128)]>,
    preference_inputs: Box<[u128]>,
    raw_rank_inputs: Box<[u128]>,
    arrangement_decisions: Box<[u8]>,
    overlay_decisions: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StructuralSignature {
    root: StructuralRecord,
    descendants: Box<[StructuralRecord]>,
}

impl StructuralSignature {
    fn exact_plot_root(kind: PlotKind, constraints: LayoutConstraints, size: LayoutSize) -> Self {
        Self {
            root: StructuralRecord {
                address: ComponentAddress::root(),
                component_type: ComponentType::Plot,
                plot_kind: Some(plot_kind_discriminant(kind)),
                constraints,
                candidate_size: size,
                track_vectors: Box::new([]),
                arranged_root_box: LayoutBox::root(size),
                baselines: Box::new([]),
                root_ink_edges: LayoutBox::root(size),
                text_decisions: Box::new([]),
                tick_decisions: Box::new([]),
                legend_and_guide_decisions: Box::new([]),
                domain_order_keys: Box::new([]),
                aspect_inputs: Box::new([]),
                preference_inputs: Box::new([]),
                raw_rank_inputs: Box::new([]),
                arrangement_decisions: Box::new([]),
                overlay_decisions: Box::new([]),
            },
            descendants: Box::new([]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AggregateRank {
    maximum: u128,
    sum: u128,
    preorder: Box<[u128]>,
}

impl AggregateRank {
    fn empty() -> Self {
        Self {
            maximum: 0,
            sum: 0,
            preorder: Box::new([]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RootQualityKey {
    semantic_gate: u8,
    hard_fit_gate: u8,
    exact_aspect_gate: u8,
    exact_aspect_error_vector: Box<[u128]>,
    tick_interval_rank: AggregateRank,
    preferred_aspect_error_vector: Box<[u128]>,
    alignment_error_vector: Box<[u128]>,
    line_rank: AggregateRank,
    tick_rank: AggregateRank,
    legend_rank: AggregateRank,
    density_rank: AggregateRank,
    panel_rank: AggregateRank,
    root_area: u128,
    normal_flow_whitespace: u128,
    root_width: LayoutScalar,
    root_height: LayoutScalar,
    structural_signature: StructuralSignature,
}

impl RootQualityKey {
    fn exact(size: LayoutSize, structural_signature: StructuralSignature) -> Result<Self, Error> {
        Ok(Self {
            semantic_gate: 0,
            hard_fit_gate: 0,
            exact_aspect_gate: 0,
            exact_aspect_error_vector: Box::new([]),
            tick_interval_rank: AggregateRank::empty(),
            preferred_aspect_error_vector: Box::new([]),
            alignment_error_vector: Box::new([]),
            line_rank: AggregateRank::empty(),
            tick_rank: AggregateRank::empty(),
            legend_rank: AggregateRank::empty(),
            density_rank: AggregateRank::empty(),
            panel_rank: AggregateRank::empty(),
            root_area: size.checked_area()?,
            normal_flow_whitespace: 0,
            root_width: size.width,
            root_height: size.height,
            structural_signature,
        })
    }
}

#[allow(
    dead_code,
    reason = "the closed diagnostic order is part of the active kernel representation"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FitKind {
    Constraint,
    Text,
    Tick,
    LegendOrGuide,
    TrackOrResidual,
    OverlayCollision,
    SpecializedGeometry,
    AssignedChild,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FitReason {
    kind: FitKind,
    component_address: ComponentAddress,
    raw_descriptor_ordinal: u128,
}

impl fmt::Display for FitReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} at {:?}, descriptor {}",
            self.kind, self.component_address, self.raw_descriptor_ordinal
        )
    }
}

#[allow(
    dead_code,
    reason = "the closed gate-reason order is part of the active selector representation"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EligibilityViolation {
    MissingSemantic,
    DuplicateSemantic,
    ClippedOrOmittedSemantic,
    TransformedSemantic,
    Constraint,
    HardMinimum,
    OutwardInk,
    Collision,
    RepresentableGeometry,
    ExactAspect,
    FinitePositiveRootSize,
    CanonicalDimension,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EligibilityReason {
    filter_ordinal: u8,
    component_address: ComponentAddress,
    axis: u8,
    violation: EligibilityViolation,
    local_ordinal: u128,
}

impl EligibilityReason {
    fn root(filter_ordinal: u8, violation: EligibilityViolation) -> Self {
        Self {
            filter_ordinal,
            component_address: ComponentAddress::root(),
            axis: 0,
            violation,
            local_ordinal: 0,
        }
    }
}

struct MeasuredLayout<LayoutPlan> {
    size: LayoutSize,
    plan: LayoutPlan,
    quality: RootQualityKey,
    eligibility_reasons: Box<[EligibilityReason]>,
}

struct MeasuredCandidates<LayoutPlan> {
    candidates: Box<[MeasuredLayout<LayoutPlan>]>,
}

impl<LayoutPlan> MeasuredCandidates<LayoutPlan> {
    fn new(mut candidates: Vec<MeasuredLayout<LayoutPlan>>) -> Result<Self, Error> {
        if candidates.is_empty() {
            return Err(Error::Render(
                "successful layout measurement returned an empty candidate collection".into(),
            ));
        }
        candidates.sort_by(|left, right| {
            left.quality
                .structural_signature
                .cmp(&right.quality.structural_signature)
        });
        candidates.dedup_by(|left, right| {
            left.quality.structural_signature == right.quality.structural_signature
        });
        Ok(Self {
            candidates: candidates.into_boxed_slice(),
        })
    }

    fn get(&self, index: usize) -> &MeasuredLayout<LayoutPlan> {
        &self.candidates[index]
    }

    fn into_selected(self, index: usize) -> Result<MeasuredLayout<LayoutPlan>, Error> {
        self.candidates
            .into_vec()
            .into_iter()
            .nth(index)
            .ok_or_else(|| Error::Render("selected layout candidate was not retained".into()))
    }
}

#[allow(
    dead_code,
    reason = "exact-root misses currently succeed, but cache entries must retain typed fit misses"
)]
enum MeasurementOutcome<LayoutPlan> {
    Candidates(MeasuredCandidates<LayoutPlan>),
    Unsatisfied(FitReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(
    clippy::struct_field_names,
    reason = "the cache accounting field names are fixed by the normative tuple"
)]
struct MeasurementAccounting {
    descriptor_count: u128,
    evaluation_count: u128,
    rejected_count: u128,
    admitted_before_dedup_count: u128,
    returned_count: u128,
}

impl MeasurementAccounting {
    fn exact_root() -> Self {
        Self {
            descriptor_count: 1,
            evaluation_count: 1,
            rejected_count: 0,
            admitted_before_dedup_count: 1,
            returned_count: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AssignedGeometryContext {
    containing_size: LayoutSize,
    projection_identity: u128,
    target_geometry: LayoutBox,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MeasurementFrontier {
    address: ComponentAddress,
    component_type: ComponentType,
    plot_kind: u8,
    root_policy: u8,
    prepared_semantic_identity: u128,
    transferred_feedback: Box<[u8]>,
}

impl MeasurementFrontier {
    fn exact_plot(kind: PlotKind) -> Self {
        Self {
            address: ComponentAddress::root(),
            component_type: ComponentType::Plot,
            plot_kind: plot_kind_discriminant(kind),
            root_policy: 0,
            prepared_semantic_identity: 0,
            transferred_feedback: Box::new([]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MeasurementCacheKey {
    constraints: LayoutConstraints,
    assigned: Option<AssignedGeometryContext>,
    frontier: MeasurementFrontier,
}

struct MeasurementContext<'a> {
    constraints: LayoutConstraints,
    assigned: Option<&'a AssignedGeometryContext>,
    frontier: &'a MeasurementFrontier,
}

impl MeasurementContext<'_> {
    fn cache_key(&self) -> MeasurementCacheKey {
        MeasurementCacheKey {
            constraints: self.constraints,
            assigned: self.assigned.cloned(),
            frontier: self.frontier.clone(),
        }
    }
}

struct CachedMeasurement<LayoutPlan> {
    outcome: MeasurementOutcome<LayoutPlan>,
    accounting: MeasurementAccounting,
}

struct MeasurementCache<LayoutPlan> {
    entries: Vec<(MeasurementCacheKey, CachedMeasurement<LayoutPlan>)>,
}

impl<LayoutPlan> MeasurementCache<LayoutPlan> {
    fn position(&self, key: &MeasurementCacheKey) -> Option<usize> {
        self.entries
            .binary_search_by(|(candidate, _)| candidate.cmp(key))
            .ok()
    }

    fn insert(
        &mut self,
        key: MeasurementCacheKey,
        measurement: CachedMeasurement<LayoutPlan>,
    ) -> usize {
        let index = self
            .entries
            .binary_search_by(|(candidate, _)| candidate.cmp(&key))
            .unwrap_or_else(std::convert::identity);
        self.entries.insert(index, (key, measurement));
        index
    }

    fn remove(&mut self, index: usize) -> CachedMeasurement<LayoutPlan> {
        self.entries.remove(index).1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceKind {
    CandidateEvaluations,
    GenerationSlots,
    GenerationWork,
    MeasurementCalls,
    CacheEntries,
    TopologySlots,
}

impl ResourceKind {
    fn name(self) -> &'static str {
        match self {
            Self::CandidateEvaluations => "candidate evaluations",
            Self::GenerationSlots => "candidate-generation slots",
            Self::GenerationWork => "candidate-generation work units",
            Self::MeasurementCalls => "measurement-call attempts",
            Self::CacheEntries => "measurement-cache entries",
            Self::TopologySlots => "prepared topology slots",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResourceLimits {
    candidate_evaluations: u128,
    generation_slots: u128,
    generation_work: u128,
    measurement_calls: u128,
    cache_entries: u128,
    topology_slots: u128,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            candidate_evaluations: RESOURCE_LIMIT,
            generation_slots: RESOURCE_LIMIT,
            generation_work: RESOURCE_LIMIT,
            measurement_calls: RESOURCE_LIMIT,
            cache_entries: RESOURCE_LIMIT,
            topology_slots: RESOURCE_LIMIT,
        }
    }
}

#[derive(Debug)]
struct ResourceState {
    limits: ResourceLimits,
    candidate_evaluations: u128,
    generation_slots: u128,
    generation_work: u128,
    measurement_calls: u128,
    cache_entries: u128,
    topology_slots: u128,
}

impl ResourceState {
    fn new() -> Self {
        Self {
            limits: ResourceLimits::default(),
            candidate_evaluations: 0,
            generation_slots: 0,
            generation_work: 0,
            measurement_calls: 0,
            cache_entries: 0,
            topology_slots: 0,
        }
    }

    fn claim(&mut self, kind: ResourceKind, amount: u128) -> Result<(), Error> {
        let (consumed, limit) = match kind {
            ResourceKind::CandidateEvaluations => (
                &mut self.candidate_evaluations,
                self.limits.candidate_evaluations,
            ),
            ResourceKind::GenerationSlots => {
                (&mut self.generation_slots, self.limits.generation_slots)
            }
            ResourceKind::GenerationWork => {
                (&mut self.generation_work, self.limits.generation_work)
            }
            ResourceKind::MeasurementCalls => {
                (&mut self.measurement_calls, self.limits.measurement_calls)
            }
            ResourceKind::CacheEntries => (&mut self.cache_entries, self.limits.cache_entries),
            ResourceKind::TopologySlots => (&mut self.topology_slots, self.limits.topology_slots),
        };
        let next = consumed
            .checked_add(amount)
            .ok_or_else(|| Error::Render(format!("layout {} counter overflowed", kind.name())))?;
        if next > limit {
            return Err(Error::Render(format!(
                "layout {} limit of {limit} exceeded by cumulative claim {next}",
                kind.name()
            )));
        }
        *consumed = next;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReservationStage {
    Prepared,
    Generation,
    Candidates,
    Convergence,
    Final,
}

impl ReservationStage {
    fn number(self) -> u8 {
        match self {
            Self::Prepared => 1,
            Self::Generation => 2,
            Self::Candidates => 3,
            Self::Convergence => 4,
            Self::Final => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReservationPhase {
    Preparing,
    Ready,
    Measuring(Option<ReservationStage>),
    Stable,
    Final,
}

struct ReservationDriver {
    phase: ReservationPhase,
}

impl ReservationDriver {
    fn new() -> Self {
        Self {
            phase: ReservationPhase::Preparing,
        }
    }

    fn finish_prepared(&mut self) {
        debug_assert_eq!(self.phase, ReservationPhase::Preparing);
        self.phase = ReservationPhase::Ready;
    }

    fn start_pass(&mut self) -> Result<(), Error> {
        match self.phase {
            ReservationPhase::Ready | ReservationPhase::Measuring(_) => {
                self.phase = ReservationPhase::Measuring(None);
                Ok(())
            }
            _ => Err(Error::Render(
                "layout reservation pass began outside measurement".into(),
            )),
        }
    }

    fn mark_stable(&mut self) -> Result<(), Error> {
        if self.phase == ReservationPhase::Measuring(Some(ReservationStage::Convergence)) {
            self.phase = ReservationPhase::Stable;
            Ok(())
        } else {
            Err(Error::Render(
                "layout stabilized before convergence-state reservation".into(),
            ))
        }
    }

    fn reserve_additional<T>(
        &mut self,
        stage: ReservationStage,
        values: &mut Vec<T>,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        self.enter(stage)?;
        #[cfg(test)]
        if FORCED_RESERVATION_FAILURE.with(|forced| {
            if forced.get() == Some(stage.number()) {
                forced.set(None);
                true
            } else {
                false
            }
        }) {
            return Err(Error::Render(format!(
                "layout reservation stage {} could not reserve {payload}: forced test refusal",
                stage.number()
            )));
        }
        values.try_reserve_exact(additional).map_err(|error| {
            Error::Render(format!(
                "layout reservation stage {} could not reserve {payload}: {error}",
                stage.number()
            ))
        })
    }

    fn enter(&mut self, stage: ReservationStage) -> Result<(), Error> {
        let allowed = matches!(
            (self.phase, stage),
            (ReservationPhase::Preparing, ReservationStage::Prepared)
                | (ReservationPhase::Stable, ReservationStage::Final)
                | (
                    ReservationPhase::Measuring(None),
                    ReservationStage::Generation | ReservationStage::Convergence
                )
                | (
                    ReservationPhase::Measuring(Some(ReservationStage::Generation)),
                    ReservationStage::Generation | ReservationStage::Candidates
                )
                | (
                    ReservationPhase::Measuring(Some(ReservationStage::Candidates)),
                    ReservationStage::Candidates | ReservationStage::Convergence
                )
                | (
                    ReservationPhase::Measuring(Some(ReservationStage::Convergence)),
                    ReservationStage::Convergence
                )
        );
        if !allowed {
            return Err(Error::Render(format!(
                "layout reservation stage {} was requested out of order",
                stage.number()
            )));
        }
        self.phase = match stage {
            ReservationStage::Prepared => ReservationPhase::Preparing,
            ReservationStage::Generation
            | ReservationStage::Candidates
            | ReservationStage::Convergence => ReservationPhase::Measuring(Some(stage)),
            ReservationStage::Final => ReservationPhase::Final,
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConvergenceState {
    selected_size: LayoutSize,
    selected_quality: RootQualityKey,
    selected_candidate_index: usize,
    selected_child_identities: Box<[StructuralSignature]>,
    selected_call_constraints: LayoutConstraints,
    assigned_contexts: Box<[AssignedGeometryContext]>,
    provisional_root_box: LayoutBox,
    accounting: MeasurementAccounting,
    next_frontier: MeasurementFrontier,
    painted_ink_inside_root: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConvergenceStatus {
    Continue,
    Stable,
}

struct ConvergenceTracker {
    states: Vec<ConvergenceState>,
}

impl ConvergenceTracker {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn observe(
        &mut self,
        state: ConvergenceState,
        reservations: &mut ReservationDriver,
    ) -> Result<ConvergenceStatus, Error> {
        reservations.reserve_additional(
            ReservationStage::Convergence,
            &mut self.states,
            1,
            "complete convergence state",
        )?;
        if let Some(previous) = self.states.last() {
            if previous == &state {
                if !state.painted_ink_inside_root {
                    return Err(Error::Render(
                        "layout fit failed after convergence: painted ink leaves the root box"
                            .into(),
                    ));
                }
                self.states.push(state);
                return Ok(ConvergenceStatus::Stable);
            }
            if self.states.iter().any(|earlier| earlier == &state) {
                return Err(Error::Render(
                    "layout convergence repeated a nonfinal state".into(),
                ));
            }
        }
        if self.states.len() + 1 == MAX_COMPLETE_PASSES {
            return Err(Error::Render(
                "layout did not converge within eight complete passes".into(),
            ));
        }
        self.states.push(state);
        Ok(ConvergenceStatus::Continue)
    }
}

fn run_convergence(
    initial_frontier: MeasurementFrontier,
    resources: &mut ResourceState,
    reservations: &mut ReservationDriver,
    mut complete_pass: impl FnMut(
        &MeasurementFrontier,
        &mut ResourceState,
        &mut ReservationDriver,
    ) -> Result<ConvergenceState, Error>,
) -> Result<ConvergenceState, Error> {
    let mut frontier = initial_frontier;
    let mut tracker = ConvergenceTracker::new();
    loop {
        reservations.start_pass()?;
        resources.claim(ResourceKind::MeasurementCalls, 1)?;
        let state = complete_pass(&frontier, resources, reservations)?;
        let next_frontier = state.next_frontier.clone();
        if tracker.observe(state, reservations)? == ConvergenceStatus::Stable {
            reservations.mark_stable()?;
            return tracker
                .states
                .pop()
                .ok_or_else(|| Error::Render("layout converged without a selected state".into()));
        }
        frontier = next_frontier;
    }
}

fn complete_cartesian_product(
    radices: &[usize],
    resources: &mut ResourceState,
    reservations: &mut ReservationDriver,
) -> Result<Vec<Box<[usize]>>, Error> {
    if radices.contains(&0) {
        return Err(Error::Render(
            "layout parent received an empty successful child collection".into(),
        ));
    }
    let count = radices.iter().try_fold(1u128, |count, radix| {
        count
            .checked_mul(u128::try_from(*radix).map_err(|_| {
                Error::Render("layout Cartesian radix overflowed the accounting domain".into())
            })?)
            .ok_or_else(|| Error::Render("layout Cartesian cardinality overflowed".into()))
    })?;
    let width = u128::try_from(radices.len())
        .map_err(|_| Error::Render("layout Cartesian width overflowed".into()))?;
    let slots = count
        .checked_mul(width.checked_add(1).ok_or_else(|| {
            Error::Render("layout Cartesian generation-slot count overflowed".into())
        })?)
        .ok_or_else(|| Error::Render("layout Cartesian generation-slot count overflowed".into()))?;
    resources.claim(ResourceKind::GenerationSlots, slots)?;
    resources.claim(
        ResourceKind::GenerationWork,
        slots
            .checked_mul(2)
            .ok_or_else(|| Error::Render("layout Cartesian work count overflowed".into()))?,
    )?;
    let count = usize::try_from(count).map_err(|_| {
        Error::Render("layout Cartesian cardinality exceeded addressable storage".into())
    })?;

    let mut combinations = Vec::new();
    reservations.reserve_additional(
        ReservationStage::Generation,
        &mut combinations,
        count,
        "complete child Cartesian combinations",
    )?;
    for ordinal in 0..count {
        let mut remainder = ordinal;
        let mut combination = Vec::new();
        reservations.reserve_additional(
            ReservationStage::Generation,
            &mut combination,
            radices.len(),
            "one child Cartesian combination",
        )?;
        combination.resize(radices.len(), 0);
        for (index, radix) in radices.iter().enumerate().rev() {
            combination[index] = remainder % radix;
            remainder /= radix;
        }
        combinations.push(combination.into_boxed_slice());
    }
    Ok(combinations)
}

fn select_root<LayoutPlan>(candidates: &MeasuredCandidates<LayoutPlan>) -> Result<usize, Error> {
    let mut survivors = (0..candidates.candidates.len()).collect::<Vec<_>>();
    for filter in 1..=3 {
        let previous = survivors.clone();
        survivors.retain(|index| {
            let quality = &candidates.get(*index).quality;
            (match filter {
                1 => quality.semantic_gate,
                2 => quality.hard_fit_gate,
                _ => quality.exact_aspect_gate,
            }) == 0
        });
        if survivors.is_empty() {
            return Err(first_empty_frontier_error(candidates, &previous, filter));
        }
    }

    let previous = survivors.clone();
    survivors.retain(|index| {
        let size = candidates.get(*index).size;
        size.width.0 > 0 && size.height.0 > 0
    });
    if survivors.is_empty() {
        return Err(first_empty_frontier_error(candidates, &previous, 4));
    }

    let previous = survivors.clone();
    survivors.retain(|index| candidates.get(*index).size.canonical_dimensions().is_some());
    if survivors.is_empty() {
        return Err(first_empty_frontier_error(candidates, &previous, 5));
    }

    survivors
        .into_iter()
        .min_by(|left, right| {
            candidates
                .get(*left)
                .quality
                .cmp(&candidates.get(*right).quality)
        })
        .ok_or_else(|| Error::Render("layout selector had no surviving candidate".into()))
}

fn first_empty_frontier_error<LayoutPlan>(
    candidates: &MeasuredCandidates<LayoutPlan>,
    removed: &[usize],
    filter: u8,
) -> Error {
    let fallback = match filter {
        1 => EligibilityViolation::MissingSemantic,
        2 => EligibilityViolation::HardMinimum,
        3 => EligibilityViolation::ExactAspect,
        4 => EligibilityViolation::FinitePositiveRootSize,
        _ => EligibilityViolation::CanonicalDimension,
    };
    let reason = removed
        .iter()
        .flat_map(|index| candidates.get(*index).eligibility_reasons.iter())
        .filter(|reason| reason.filter_ordinal == filter)
        .min()
        .cloned()
        .unwrap_or_else(|| EligibilityReason::root(filter, fallback));
    Error::Render(format!(
        "layout fit failed at filter {}: {:?}",
        reason.filter_ordinal, reason.violation
    ))
}

/// One promoted root plan whose dimensions are final for every backend route.
pub(crate) struct SelectedRoot<LayoutPlan> {
    size: LayoutSize,
    signature: StructuralSignature,
    plan: LayoutPlan,
}

impl<LayoutPlan> SelectedRoot<LayoutPlan> {
    pub(crate) fn dimensions(&self) -> (u32, u32) {
        debug_assert_eq!(
            self.signature.root.candidate_size, self.size,
            "promoted root must retain its selected structural identity"
        );
        self.size
            .canonical_dimensions()
            .expect("selected exact-root dimensions were validated before promotion")
    }

    pub(crate) fn plan(&self) -> &LayoutPlan {
        &self.plan
    }
}

/// Plan one active exact-size root through complete measurement and convergence.
#[expect(
    clippy::too_many_lines,
    reason = "the exact-root operation keeps the five ordered reservation stages explicit"
)]
pub(crate) fn plan_exact_root<LayoutPlan>(
    width: u32,
    height: u32,
    kind: PlotKind,
    build_plan: impl FnOnce() -> Result<LayoutPlan, Error>,
) -> Result<SelectedRoot<LayoutPlan>, Error> {
    let constraints = LayoutConstraints::exact_css_pixels(width, height)?;
    let size = LayoutSize::exact_css_pixels(width, height)?;
    debug_assert!(constraints.contains(size));
    let frontier = MeasurementFrontier::exact_plot(kind);

    let mut resources = ResourceState::new();
    resources.claim(ResourceKind::TopologySlots, 1)?;
    let mut reservations = ReservationDriver::new();
    let mut cache_entries = Vec::new();
    reservations.reserve_additional(
        ReservationStage::Prepared,
        &mut cache_entries,
        CACHE_ENTRY_LIMIT,
        "fixed measurement-cache entries",
    )?;
    reservations.finish_prepared();
    let mut cache = MeasurementCache {
        entries: cache_entries,
    };
    let mut build_plan = Some(build_plan);
    let selected_state = run_convergence(
        frontier,
        &mut resources,
        &mut reservations,
        |consumed_frontier, resources, reservations| {
            let key = MeasurementContext {
                constraints,
                assigned: None,
                frontier: consumed_frontier,
            }
            .cache_key();
            let cache_index = if let Some(index) = cache.position(&key) {
                index
            } else {
                resources.claim(ResourceKind::CacheEntries, 1)?;
                let combinations = complete_cartesian_product(&[1], resources, reservations)?;
                debug_assert_eq!(&*combinations[0], &[0]);

                let mut measured = Vec::new();
                reservations.reserve_additional(
                    ReservationStage::Candidates,
                    &mut measured,
                    1,
                    "exact-root candidate and immutable plan",
                )?;
                resources.claim(ResourceKind::CandidateEvaluations, 1)?;
                let plan = build_plan.take().ok_or_else(|| {
                    Error::Render("exact-root adapter was evaluated twice".into())
                })?()?;
                let signature = StructuralSignature::exact_plot_root(kind, constraints, size);
                let quality = RootQualityKey::exact(size, signature)?;
                measured.push(MeasuredLayout {
                    size,
                    plan,
                    quality,
                    eligibility_reasons: Box::new([]),
                });
                let candidates = MeasuredCandidates::new(measured)?;
                cache.insert(
                    key,
                    CachedMeasurement {
                        outcome: MeasurementOutcome::Candidates(candidates),
                        accounting: MeasurementAccounting::exact_root(),
                    },
                )
            };

            let measurement = &cache.entries[cache_index].1;
            let (selected_candidate_index, candidate) = match &measurement.outcome {
                MeasurementOutcome::Candidates(candidates) => {
                    let index = select_root(candidates)?;
                    (index, candidates.get(index))
                }
                MeasurementOutcome::Unsatisfied(reason) => {
                    return Err(Error::Render(format!(
                        "layout constraints were unsatisfied: {reason}"
                    )));
                }
            };
            Ok(ConvergenceState {
                selected_size: candidate.size,
                selected_quality: candidate.quality.clone(),
                selected_candidate_index,
                selected_child_identities: Box::new([]),
                selected_call_constraints: constraints,
                assigned_contexts: Box::new([]),
                provisional_root_box: LayoutBox::root(candidate.size),
                accounting: measurement.accounting,
                next_frontier: consumed_frontier.clone(),
                painted_ink_inside_root: true,
            })
        },
    )?;

    let key = MeasurementContext {
        constraints,
        assigned: None,
        frontier: &selected_state.next_frontier,
    }
    .cache_key();
    let cache_index = cache.position(&key).ok_or_else(|| {
        Error::Render("converged exact-root measurement was missing from its cache".into())
    })?;
    let measurement = cache.remove(cache_index);
    let candidates = match measurement.outcome {
        MeasurementOutcome::Candidates(candidates) => candidates,
        MeasurementOutcome::Unsatisfied(reason) => {
            return Err(Error::Render(format!(
                "converged layout unexpectedly became unsatisfied: {reason}"
            )));
        }
    };
    let selected = candidates.into_selected(selected_state.selected_candidate_index)?;
    if selected.quality != selected_state.selected_quality {
        return Err(Error::Render(
            "promoted layout candidate did not match the converged selection".into(),
        ));
    }
    let dimensions = selected.size.canonical_dimensions().ok_or_else(|| {
        Error::Render("selected exact-root dimensions are not canonically representable".into())
    })?;
    if dimensions != (width, height) {
        return Err(Error::Render(format!(
            "selected exact-root dimensions {}x{} do not match requested dimensions {width}x{height}",
            dimensions.0, dimensions.1
        )));
    }
    let mut promoted = Vec::new();
    reservations.reserve_additional(
        ReservationStage::Final,
        &mut promoted,
        1,
        "promoted exact-root plan",
    )?;
    promoted.push(SelectedRoot {
        size: selected.size,
        signature: selected.quality.structural_signature,
        plan: selected.plan,
    });
    promoted
        .pop()
        .ok_or_else(|| Error::Render("exact-root promotion produced no plan".into()))
}

fn plot_kind_discriminant(kind: PlotKind) -> u8 {
    match kind {
        PlotKind::Scatter => 0,
        PlotKind::Histogram => 1,
        PlotKind::Density => 2,
        PlotKind::Qq => 3,
        PlotKind::PointInterval => 4,
        PlotKind::Line => 5,
        PlotKind::Trajectory => 6,
        PlotKind::LinePath => 7,
        PlotKind::Interaction => 8,
        PlotKind::Coefficient => 9,
        PlotKind::Forest => 10,
        PlotKind::Dumbbell => 11,
        PlotKind::Heatmap => 12,
        PlotKind::Bar => 13,
        PlotKind::HorizontalBar => 14,
        PlotKind::StackedBar => 15,
        PlotKind::StackedHorizontalBar => 16,
        PlotKind::Boxplot => 17,
        PlotKind::Violin => 18,
        PlotKind::Raincloud => 19,
        PlotKind::Regression => 20,
        PlotKind::Marginal => 21,
        PlotKind::Matrix => 22,
        PlotKind::Beeswarm => 23,
        PlotKind::Waterfall => 24,
    }
}

#[cfg(test)]
#[path = "kernel/tests.rs"]
mod tests;
