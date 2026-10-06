//! Bounded root requests, parent feedback and operation-scoped preparation resources.

use std::cmp::Ordering;

use crate::Error;

const QUANTA_PER_CSS_PIXEL: i64 = 1024;

pub(crate) fn nearest_spatial_quanta(value: f64) -> Result<i64, Error> {
    LayoutScalar::from_css_pixels(value, SpatialRounding::NearestEven).map(|value| value.0)
}

pub(crate) fn ceil_spatial_quanta(value: f64) -> Result<i64, Error> {
    LayoutScalar::from_css_pixels(value, SpatialRounding::Ceil).map(|value| value.0)
}
#[cfg(test)]
const TOPOLOGY_SLOT_LIMIT: u128 = 1_000_000;

#[cfg(test)]
thread_local! {
    static FORCED_RESERVATION_FAILURE: std::cell::Cell<Option<ReservationStage>> = const { std::cell::Cell::new(None) };
    pub(crate) static ROOT_FEEDBACK_ROUNDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn force_reservation_failure(stage: ReservationStage) {
    FORCED_RESERVATION_FAILURE.with(|forced| forced.set(Some(stage)));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LayoutScalar(i64);

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpatialRounding {
    Floor,
    Ceil,
    NearestEven,
}

impl LayoutScalar {
    pub(crate) fn exact_css_pixels(value: u32) -> Result<Self, Error> {
        i64::from(value)
            .checked_mul(QUANTA_PER_CSS_PIXEL)
            .map(Self)
            .ok_or_else(|| Error::Render("exact layout dimension overflowed layout quanta".into()))
    }

    pub(crate) const fn from_quanta(value: i64) -> Self {
        Self(value)
    }

    pub(crate) const fn quanta(self) -> i64 {
        self.0
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

    pub(crate) fn from_css_pixels(value: f64, rounding: SpatialRounding) -> Result<Self, Error> {
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
            (significand << shift, None)
        } else {
            let denominator_shift = shift.unsigned_abs();
            if denominator_shift >= 128 {
                (0, Some(Ordering::Less))
            } else {
                let denominator = 1_u128 << denominator_shift;
                let quotient = significand / denominator;
                let remainder = significand % denominator;
                (
                    quotient,
                    (remainder != 0).then(|| remainder.cmp(&(denominator / 2))),
                )
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
}

#[allow(
    dead_code,
    reason = "the closed role order is part of canonical structural identity"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub(crate) enum AddressRole {
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
    PreferenceMetric,
    Text,
    Glyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AddressSegment {
    pub(crate) role: AddressRole,
    pub(crate) slot_index: u32,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentAddress {
    root: AddressSegment,
    descendants: Option<std::sync::Arc<Vec<AddressSegment>>>,
}

impl std::fmt::Debug for ComponentAddress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ComponentAddress")
            .field("root", &self.root)
            .field("descendants", &self.descendants())
            .finish()
    }
}

impl ComponentAddress {
    fn descendants(&self) -> &[AddressSegment] {
        self.descendants.as_deref().map_or(&[], Vec::as_slice)
    }

    pub(crate) fn leaf_role(&self) -> AddressRole {
        self.descendants()
            .last()
            .map_or(self.root.role, |segment| segment.role)
    }

    pub(crate) fn leaf_slot_index(&self) -> u32 {
        self.descendants().last().unwrap_or(&self.root).slot_index
    }

    pub(crate) fn under_grid(
        &self,
        grid: AddressRole,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Self, Error> {
        let [leaf] = self.descendants() else {
            return Err(Error::Render(
                "child owner must be one root-local cell".into(),
            ));
        };
        Self::root()
            .child(grid, 0, resources)?
            .child(leaf.role, leaf.slot_index, resources)
    }
    fn is_within(&self, owner: &Self) -> bool {
        self.root == owner.root && self.descendants().starts_with(owner.descendants())
    }
    pub(crate) fn root() -> Self {
        Self {
            root: AddressSegment {
                role: AddressRole::Root,
                slot_index: 0,
            },
            descendants: None,
        }
    }

    pub(crate) fn child(
        &self,
        role: AddressRole,
        slot_index: u32,
        resources: &crate::font::canonical::OperationGenerationResources,
    ) -> Result<Self, Error> {
        let count = self
            .descendants()
            .len()
            .checked_add(1)
            .ok_or_else(|| Error::Render("component address length overflowed".into()))?;
        let bytes = count
            .checked_mul(std::mem::size_of::<AddressSegment>())
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<Vec<AddressSegment>>() + 2 * std::mem::size_of::<usize>(),
                )
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("component address storage overflowed".into()))?;
        // The immutable path owns its Vec header and each actual segment. A
        // scope, failure, or feedback clone only shares that admitted payload.
        resources
            .claim_slots(count as u128 + 1)
            .map_err(|error| Error::Render(format!("component address records: {error:?}")))?;
        resources
            .claim_bytes(bytes)
            .map_err(|error| Error::Render(format!("component address bytes: {error:?}")))?;
        let mut descendants = Vec::new();
        descendants.try_reserve_exact(count).map_err(|error| {
            Error::Render(format!(
                "layout could not reserve canonical component address: {error}"
            ))
        })?;
        descendants.extend_from_slice(self.descendants());
        descendants.push(AddressSegment { role, slot_index });
        Ok(Self {
            root: self.root,
            descendants: Some(std::sync::Arc::new(descendants)),
        })
    }

    pub(crate) fn canonical_text_cmp(&self, other: &Self) -> Ordering {
        let root = (
            self.root.role.text_traversal_ordinal(),
            self.root.slot_index,
        )
            .cmp(&(
                other.root.role.text_traversal_ordinal(),
                other.root.slot_index,
            ));
        if root != Ordering::Equal {
            return root;
        }
        self.descendants()
            .iter()
            .map(|segment| (segment.role.text_traversal_ordinal(), segment.slot_index))
            .cmp(
                other
                    .descendants()
                    .iter()
                    .map(|segment| (segment.role.text_traversal_ordinal(), segment.slot_index)),
            )
    }
}

impl AddressRole {
    const fn text_traversal_ordinal(self) -> u16 {
        match self {
            // Plot and Canvas preorder visit their frame/body before the caption.
            Self::RootCaption => u16::MAX,
            _ => self as u16,
        }
    }
}

struct ResourceState {
    generation_resources: crate::font::canonical::OperationGenerationResources,
    font_resources: crate::font::canonical::OperationFontResources,
}

impl ResourceState {
    fn new() -> Self {
        let font_resources = crate::font::canonical::OperationFontResources::new();
        #[cfg(test)]
        crate::orchestrator::measurements::resources(font_resources.clone());
        Self {
            generation_resources: font_resources.generation_resources(),
            font_resources,
        }
    }
}

pub(crate) struct PreparedStage<'a> {
    resources: &'a mut ResourceState,
    reservations: &'a mut ReservationDriver,
}

impl PreparedStage<'_> {
    pub(crate) fn claim_topology_slots(&mut self, amount: u128) -> Result<(), Error> {
        self.resources
            .generation_resources
            .claim_topology_slots(amount)
            .map_err(|error| Error::Render(format!("layout topology admission: {error:?}")))
    }

    pub(crate) fn reserve_scalar_presence_map(&mut self) -> Result<Vec<u64>, Error> {
        const SCALAR_MAP_WORDS: usize = 0x11_0000 / u64::BITS as usize;
        let mut presence = Vec::new();
        self.reserve_prepared(
            &mut presence,
            SCALAR_MAP_WORDS,
            "Unicode scalar presence map",
        )?;
        presence.resize(SCALAR_MAP_WORDS, 0);
        Ok(presence)
    }

    pub(crate) fn reserve_prepared<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        self.reservations.reserve_additional(
            ReservationStage::Prepared,
            values,
            additional,
            payload,
        )
    }

    pub(crate) fn reserve_prepared_string(
        &mut self,
        value: &mut String,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        self.reservations
            .enter(ReservationStage::Prepared, payload)?;
        value
            .try_reserve_exact(additional)
            .map_err(|error| reservation_error(ReservationStage::Prepared, payload, error))
    }

    pub(crate) fn font_resources(&self) -> crate::font::canonical::OperationFontResources {
        self.resources.font_resources.clone()
    }

    pub(crate) fn generation_resources(
        &self,
    ) -> crate::font::canonical::OperationGenerationResources {
        self.resources.generation_resources.clone()
    }
}

pub(crate) struct FinalStage<'a> {
    reservations: &'a mut ReservationDriver,
    resources: &'a ResourceState,
}

impl FinalStage<'_> {
    pub(crate) fn claim_capture_bytes(&self, bytes: usize, payload: &str) -> Result<(), Error> {
        let bytes = u64::try_from(bytes)
            .map_err(|_| Error::Render(format!("{payload}: capture byte count overflow")))?;
        self.resources
            .font_resources
            .claim_bytes(bytes)
            .map_err(|error| Error::Render(format!("{payload}: {error}")))
    }
    pub(crate) fn claim_capture_call(&self) -> Result<(), Error> {
        self.resources
            .generation_resources
            .claim_slots(1)
            .map_err(|error| Error::Render(format!("prepared scene call admission: {error:?}")))
    }

    pub(crate) fn reserve_capture<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| Error::Render(format!("{payload}: capture length overflow")))?;
        if needed <= values.capacity() {
            return Ok(());
        }
        let capacity = needed.max(values.capacity().saturating_mul(2));
        let bytes = (capacity - values.capacity())
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render(format!("{payload}: capture byte count overflow")))?;
        self.resources
            .font_resources
            .claim_bytes(bytes)
            .map_err(|error| Error::Render(format!("{payload}: {error}")))?;
        self.reserve_final(values, capacity - values.len(), payload)
    }

    pub(crate) fn reserve_final<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        self.reservations
            .reserve_additional(ReservationStage::Final, values, additional, payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReservationStage {
    Prepared,
    Measured,
    Final,
}

impl ReservationStage {
    pub(crate) fn number(self) -> u8 {
        match self {
            Self::Prepared => 1,
            Self::Measured => 2,
            Self::Final => 3,
        }
    }
}

struct ReservationDriver {
    phase: ReservationStage,
}

impl ReservationDriver {
    fn new() -> Self {
        Self {
            phase: ReservationStage::Prepared,
        }
    }

    fn reserve_additional<T>(
        &mut self,
        stage: ReservationStage,
        values: &mut Vec<T>,
        additional: usize,
        payload: &str,
    ) -> Result<(), Error> {
        self.enter(stage, payload)?;
        values
            .try_reserve_exact(additional)
            .map_err(|error| reservation_error(stage, payload, error))
    }

    fn enter(&mut self, stage: ReservationStage, payload: &str) -> Result<(), Error> {
        if self.phase != stage {
            return Err(Error::Render(format!(
                "layout reservation stage {} was requested out of order",
                stage.number()
            )));
        }
        #[cfg(test)]
        if FORCED_RESERVATION_FAILURE.with(|forced| {
            if forced.get() == Some(stage) {
                forced.set(None);
                true
            } else {
                false
            }
        }) {
            return Err(reservation_error(stage, payload, "forced test refusal"));
        }
        let _ = payload;
        Ok(())
    }
}

fn reservation_error(
    stage: ReservationStage,
    payload: &str,
    error: impl std::fmt::Display,
) -> Error {
    Error::Render(format!(
        "layout reservation stage {} could not reserve {payload}: {error}",
        stage.number()
    ))
}

/// One private root request; measurement resolves both dimensions before freeze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootSize {
    width: Option<u32>,
    height: Option<u32>,
}

impl RootSize {
    pub(crate) const fn natural() -> Self {
        Self {
            width: None,
            height: None,
        }
    }
    pub(crate) const fn width(width: u32) -> Self {
        Self {
            width: Some(width),
            height: None,
        }
    }
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
        }
    }
    pub(crate) const fn requested_dimensions(self) -> (Option<u32>, Option<u32>) {
        (self.width, self.height)
    }
    pub(crate) fn dimensions(self) -> Result<(u32, u32), Error> {
        match (self.width, self.height) {
            (Some(width), Some(height)) => Ok((width, height)),
            _ => Err(Error::Render(
                "root content measurement must provide both finite dimensions".into(),
            )),
        }
    }

    pub(crate) fn validate_dimensions(self, width: u32, height: u32) -> Result<(), Error> {
        if self.dimensions()? != (width, height) {
            return Err(Error::Render(
                "measured root dimensions do not match the exact assignment".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) struct MeasuredLayout<Plan> {
    size: RootSize,
    plan: Plan,
}

impl<Plan> MeasuredLayout<Plan> {
    pub(crate) fn new(size: RootSize, plan: Plan) -> Self {
        Self { size, plan }
    }
}

struct RootAllocation<Plan, Prepared> {
    size: RootSize,
    prepared: Prepared,
    plan: Plan,
}

/// One result frozen after parent feedback. Final admission remains a separate stage.
pub(crate) struct FrozenRoot<Plan, Prepared = ()> {
    allocation: Box<[RootAllocation<Plan, Prepared>; 1]>,
}

impl<Plan, Prepared> FrozenRoot<Plan, Prepared> {
    pub(crate) fn dimensions(&self) -> (u32, u32) {
        self.allocation[0]
            .size
            .dimensions()
            .expect("frozen roots have validated finite dimensions")
    }
    pub(crate) fn plan(&self) -> &Plan {
        &self.allocation[0].plan
    }
    pub(crate) fn prepared(&self) -> &Prepared {
        &self.allocation[0].prepared
    }
}

mod feedback;
mod tracks;
pub(crate) use feedback::TrialOutcome;
pub(crate) use feedback::{ChildFeedback, ChildFeedbackInputs};
pub(crate) use feedback::{
    FeedbackKind, FeedbackRequestId, FeedbackRound, FitKind, FrameRequirements,
};
pub(crate) use tracks::{allocate_grid_axis, GridRequirement, TrackSpec};

pub(crate) fn required_length(value: f64) -> Result<f64, Error> {
    let length = LayoutScalar::from_css_pixels(value, SpatialRounding::Ceil)?;
    if !length.is_nonnegative() {
        return Err(Error::Render(
            "required layout length must be nonnegative".into(),
        ));
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "required guide lengths are subsequently checked against exact u32 root constraints"
    )]
    let value = length.0 as f64 / 1024.0;
    Ok(value)
}

/// Prepare once, coordinate at most eight complete rounds, then freeze once.
#[expect(
    clippy::too_many_lines,
    reason = "preparation, cumulative round admission, convergence, and finalization stay ordered"
)]
pub(crate) fn plan_exact_root<Plan, FinalPlan, Prepared>(
    width: impl Into<Option<u32>>,
    height: impl Into<Option<u32>>,
    prepare: impl FnOnce(&mut PreparedStage<'_>) -> Result<Prepared, Error>,
    mut measure: impl FnMut(
        &Prepared,
        RootSize,
        &mut FeedbackRound,
    ) -> Result<MeasuredLayout<Plan>, Error>,
    finalize: impl FnOnce(&Prepared, RootSize, Plan, &mut FinalStage<'_>) -> Result<FinalPlan, Error>,
) -> Result<FrozenRoot<FinalPlan, Prepared>, Error> {
    let width = width.into();
    let height = height.into();
    if width.is_none() && height.is_some() {
        return Err(Error::Render(
            "an exact root height requires an exact width".into(),
        ));
    }
    if width == Some(0) || height == Some(0) {
        return Err(Error::Render(
            "layout root dimensions must be finite and positive".into(),
        ));
    }
    let mut assigned = match (width, height) {
        (Some(width), Some(height)) => RootSize::new(width, height),
        (Some(width), None) => RootSize::width(width),
        (None, None) => RootSize::natural(),
        (None, Some(_)) => unreachable!("height-only requests reject before preparation"),
    };
    let dimension = |value: Option<u32>| -> Result<LengthConstraint, Error> {
        let exact = value.map(LayoutScalar::exact_css_pixels).transpose()?;
        LengthConstraint::new(exact.unwrap_or(LayoutScalar(0)), exact)
    };
    let constraints = match (width, height) {
        (Some(width), Some(height)) => LayoutConstraints::exact_css_pixels(width, height)?,
        _ => LayoutConstraints {
            width: dimension(width)?,
            height: dimension(height)?,
        },
    };
    let mut resources = ResourceState::new();
    let mut reservations = ReservationDriver::new();
    let mut prepared_stage = PreparedStage {
        resources: &mut resources,
        reservations: &mut reservations,
    };
    prepared_stage.claim_topology_slots(1)?;
    #[cfg(test)]
    let preparation_timer = crate::orchestrator::measurements::Timer::new(0);
    let prepared = prepare(&mut prepared_stage)?;
    #[cfg(test)]
    drop(preparation_timer);

    reservations.phase = ReservationStage::Measured;
    #[cfg(test)]
    ROOT_FEEDBACK_ROUNDS.with(|rounds| rounds.set(0));
    let mut history = Vec::new();
    reservations.reserve_additional(
        ReservationStage::Measured,
        &mut history,
        1,
        "measured exact-root plan",
    )?;
    let measured = loop {
        resources
            .generation_resources
            .claim_calls(1)
            .map_err(|error| {
                Error::Render(format!("root measurement call admission: {error:?}"))
            })?;
        #[cfg(test)]
        let _measurement_timer = crate::orchestrator::measurements::root_round(history.len());
        #[cfg(test)]
        ROOT_FEEDBACK_ROUNDS.with(|rounds| rounds.set(rounds.get() + 1));
        let mut round = FeedbackRound::new(
            history.last(),
            resources.generation_resources.clone(),
            history.len(),
        )?;
        if let Some(width) = assigned.requested_dimensions().0 {
            round.observe_number(f64::from(width))?;
        }
        if let Some(height) = height {
            round.observe_number(f64::from(height))?;
        }
        let result = measure(&prepared, assigned, &mut round)?;
        let (measured_width, measured_height) = result.size.dimensions()?;
        if assigned.width.is_some_and(|width| width != measured_width)
            || height.is_some_and(|height| height != measured_height)
        {
            return Err(Error::Render(
                if height.is_some() {
                    "measured root dimensions do not match the exact assignment"
                } else {
                    "measured root dimensions do not match the sizing request"
                }
                .into(),
            ));
        }
        if measured_width == 0 || measured_height == 0 {
            return Err(Error::Render(
                "layout root dimensions must be finite and positive".into(),
            ));
        }
        // Automatic width is chosen by initial bottom-up composition exactly once.
        assigned.width = Some(measured_width);
        let state = round.finish()?;
        if let Some(error) = state.terminal_error(&resources.generation_resources)? {
            return Err(error);
        }
        // Measure and validate the current assignment before accepting its plan.
        // Observation equality would require an unconditional duplicate pass.
        if state.feedback_unchanged(&resources.generation_resources)? {
            if let Some(error) = state.unchanged_error(&resources.generation_resources)? {
                return Err(error);
            }
            break result;
        }
        if state.repeats_history(&history, &resources.generation_resources)? {
            return Err(Error::Render("layout root feedback cycle detected".into()));
        }
        if history.len() == 7 {
            return Err(Error::Render(
                "layout root feedback exhausted eight rounds".into(),
            ));
        }
        reservations.reserve_additional(
            ReservationStage::Measured,
            &mut history,
            1,
            "root feedback state",
        )?;
        history.push(state);
    };
    let (width, height) = measured.size.dimensions()?;
    let size = LayoutSize::exact_css_pixels(width, height)?;
    if !constraints.contains(size) {
        return Err(Error::Render(
            "measured root dimensions do not match the sizing request".into(),
        ));
    }
    if size.width.0 == 0 || size.height.0 == 0 {
        return Err(Error::Render(
            "layout root dimensions must be finite and positive".into(),
        ));
    }
    let dimensions = size.canonical_dimensions().ok_or_else(|| {
        Error::Render("measured root dimensions are not canonically representable".into())
    })?;
    measured
        .size
        .validate_dimensions(dimensions.0, dimensions.1)?;
    let assigned = RootSize::new(width, height);

    reservations.phase = ReservationStage::Final;
    let mut allocation = Vec::new();
    let mut final_stage = FinalStage {
        reservations: &mut reservations,
        resources: &resources,
    };
    final_stage.reserve_final(&mut allocation, 1, "frozen exact-root plan")?;
    let plan = finalize(&prepared, assigned, measured.plan, &mut final_stage)?;
    allocation.push(RootAllocation {
        size: assigned,
        prepared,
        plan,
    });
    let allocation = allocation
        .into_boxed_slice()
        .try_into()
        .unwrap_or_else(|_| unreachable!("one frozen exact-root allocation"));
    #[cfg(test)]
    crate::orchestrator::measurements::root_success();
    Ok(FrozenRoot { allocation })
}

#[cfg(test)]
#[path = "kernel/tests.rs"]
mod tests;
