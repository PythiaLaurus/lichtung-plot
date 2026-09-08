//! Single measured exact-root layout and operation-scoped preparation resources.

use std::cmp::Ordering;

use crate::Error;

const QUANTA_PER_CSS_PIXEL: i64 = 1024;
const TOPOLOGY_SLOT_LIMIT: u128 = 1_000_000;

#[cfg(test)]
thread_local! {
    static FORCED_RESERVATION_FAILURE: std::cell::Cell<Option<ReservationStage>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn force_reservation_failure(stage: ReservationStage) {
    FORCED_RESERVATION_FAILURE.with(|forced| forced.set(Some(stage)));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LayoutScalar(i64);

#[cfg(test)]
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

    #[cfg(test)]
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
    Text,
    Glyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AddressSegment {
    pub(crate) role: AddressRole,
    pub(crate) slot_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ComponentAddress {
    root: AddressSegment,
    descendants: Box<[AddressSegment]>,
}

impl ComponentAddress {
    pub(crate) fn root() -> Self {
        Self {
            root: AddressSegment {
                role: AddressRole::Root,
                slot_index: 0,
            },
            descendants: Box::new([]),
        }
    }

    pub(crate) fn child(&self, role: AddressRole, slot_index: u32) -> Result<Self, Error> {
        let mut descendants = Vec::new();
        descendants
            .try_reserve_exact(self.descendants.len().saturating_add(1))
            .map_err(|error| {
                Error::Render(format!(
                    "layout could not reserve canonical component address: {error}"
                ))
            })?;
        descendants.extend_from_slice(&self.descendants);
        descendants.push(AddressSegment { role, slot_index });
        Ok(Self {
            root: self.root,
            descendants: descendants.into_boxed_slice(),
        })
    }

    pub(crate) fn prepared_child(
        &self,
        role: AddressRole,
        slot_index: u32,
        stage: &mut PreparedStage<'_>,
    ) -> Result<Self, Error> {
        let mut descendants = Vec::new();
        stage.reserve_prepared(
            &mut descendants,
            self.descendants.len().saturating_add(1),
            "prepared component address",
        )?;
        descendants.extend_from_slice(&self.descendants);
        descendants.push(AddressSegment { role, slot_index });
        Ok(Self {
            root: self.root,
            descendants: descendants.into_boxed_slice(),
        })
    }

    pub(crate) fn prepared_clone(&self, stage: &mut PreparedStage<'_>) -> Result<Self, Error> {
        let mut descendants = Vec::new();
        stage.reserve_prepared(
            &mut descendants,
            self.descendants.len(),
            "prepared component address",
        )?;
        descendants.extend_from_slice(&self.descendants);
        Ok(Self {
            root: self.root,
            descendants: descendants.into_boxed_slice(),
        })
    }

    pub(crate) fn prepared_copy_shell(&self, stage: &mut PreparedStage<'_>) -> Result<Self, Error> {
        let mut descendants = Vec::new();
        stage.reserve_prepared(
            &mut descendants,
            self.descendants.len(),
            "prepared component address",
        )?;
        descendants.resize(
            self.descendants.len(),
            AddressSegment {
                role: AddressRole::Root,
                slot_index: 0,
            },
        );
        Ok(Self {
            root: self.root,
            descendants: descendants.into_boxed_slice(),
        })
    }

    pub(crate) fn fill_prepared_copy(&mut self, source: &Self) -> Result<(), Error> {
        if self.descendants.len() != source.descendants.len() {
            return Err(Error::Render(
                "prepared component address reservation mismatch".into(),
            ));
        }
        self.root = source.root;
        self.descendants.copy_from_slice(&source.descendants);
        Ok(())
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
        self.descendants
            .iter()
            .map(|segment| (segment.role.text_traversal_ordinal(), segment.slot_index))
            .cmp(
                other
                    .descendants
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
    topology_slots: u128,
    generation_resources: crate::font::canonical::OperationGenerationResources,
    font_resources: crate::font::canonical::OperationFontResources,
}

impl ResourceState {
    fn new() -> Self {
        Self {
            topology_slots: 0,
            generation_resources: crate::font::canonical::OperationGenerationResources::new(),
            font_resources: crate::font::canonical::OperationFontResources::new(),
        }
    }
}

pub(crate) struct PreparedStage<'a> {
    resources: &'a mut ResourceState,
    reservations: &'a mut ReservationDriver,
}

impl PreparedStage<'_> {
    pub(crate) fn claim_topology_slots(&mut self, amount: u128) -> Result<(), Error> {
        let next = self
            .resources
            .topology_slots
            .checked_add(amount)
            .ok_or_else(|| {
                Error::Render("layout prepared topology slots counter overflowed".into())
            })?;
        if next > TOPOLOGY_SLOT_LIMIT {
            return Err(Error::Render(format!("layout prepared topology slots limit of {TOPOLOGY_SLOT_LIMIT} exceeded by cumulative claim {next}")));
        }
        self.resources.topology_slots = next;
        Ok(())
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
}

impl FinalStage<'_> {
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

/// Exact assigned CSS-pixel dimensions, not an intrinsic-size proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RootSize {
    width: u32,
    height: u32,
}

impl RootSize {
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    pub(crate) const fn dimensions(self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn validate_dimensions(self, width: u32, height: u32) -> Result<(), Error> {
        if self.dimensions() != (width, height) {
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

/// One frozen result, with no convergence or final-admission claim.
pub(crate) struct FrozenRoot<Plan, Prepared = ()> {
    allocation: Box<[RootAllocation<Plan, Prepared>; 1]>,
}

impl<Plan, Prepared> FrozenRoot<Plan, Prepared> {
    pub(crate) fn dimensions(&self) -> (u32, u32) {
        self.allocation[0].size.dimensions()
    }
    pub(crate) fn plan(&self) -> &Plan {
        &self.allocation[0].plan
    }
    pub(crate) fn prepared(&self) -> &Prepared {
        &self.allocation[0].prepared
    }
}

/// Prepare once, measure the exact assignment once, then freeze once.
pub(crate) fn plan_exact_root<Plan, FinalPlan, Prepared>(
    width: u32,
    height: u32,
    prepare: impl FnOnce(&mut PreparedStage<'_>) -> Result<Prepared, Error>,
    measure: impl FnOnce(&Prepared, RootSize) -> Result<MeasuredLayout<Plan>, Error>,
    finalize: impl FnOnce(&Prepared, RootSize, Plan, &mut FinalStage<'_>) -> Result<FinalPlan, Error>,
) -> Result<FrozenRoot<FinalPlan, Prepared>, Error> {
    let assigned = RootSize::new(width, height);
    let constraints = LayoutConstraints::exact_css_pixels(width, height)?;
    let mut resources = ResourceState::new();
    let mut reservations = ReservationDriver::new();
    let mut prepared_stage = PreparedStage {
        resources: &mut resources,
        reservations: &mut reservations,
    };
    prepared_stage.claim_topology_slots(1)?;
    let prepared = prepare(&mut prepared_stage)?;

    reservations.phase = ReservationStage::Measured;
    let mut measured = Vec::new();
    reservations.reserve_additional(
        ReservationStage::Measured,
        &mut measured,
        1,
        "measured exact-root plan",
    )?;
    measured.push(measure(&prepared, assigned)?);
    let measured = measured.pop().expect("one exact-root measurement");
    let size = LayoutSize::exact_css_pixels(measured.size.width, measured.size.height)?;
    if !constraints.contains(size) {
        return Err(Error::Render(
            "measured root dimensions do not match the exact assignment".into(),
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
    assigned.validate_dimensions(dimensions.0, dimensions.1)?;

    reservations.phase = ReservationStage::Final;
    let mut allocation = Vec::new();
    let mut final_stage = FinalStage {
        reservations: &mut reservations,
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
    Ok(FrozenRoot { allocation })
}

#[cfg(test)]
#[path = "kernel/tests.rs"]
mod tests;
