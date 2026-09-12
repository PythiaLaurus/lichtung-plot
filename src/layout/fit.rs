//! Typed local fit outcomes. Fatal producer errors never select another layout.

use crate::layout::{AddressRole, ComponentAddress};
use crate::layout::{
    PhysicalTextAlignment, ResolvedTextPlan, TextLineHeight, TextPlanInput, TextPlanProvenance,
};
use crate::theme::{ResolvedTheme, TextAppearance};
use crate::Error;

#[allow(
    dead_code,
    reason = "the complete canonical diagnostic order is shared by migrated and native owners"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FitKind {
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
pub(crate) struct FitReason {
    pub(crate) kind: FitKind,
    pub(crate) address: ComponentAddress,
    pub(crate) ordinal: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FitFailure {
    pub(crate) reason: FitReason,
    pub(crate) message: String,
}

impl std::fmt::Display for FitFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "layout fit {:?}: {}", self.reason, self.message)
    }
}

impl FitFailure {
    pub(crate) fn into_error(self) -> Error {
        Error::Render(self.to_string())
    }

    pub(crate) fn retain_least(slot: &mut Option<Self>, failure: Self) {
        if slot
            .as_ref()
            .is_none_or(|current| failure.reason < current.reason)
        {
            *slot = Some(failure);
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FitScope {
    pub(crate) address: ComponentAddress,
    pub(crate) ordinal: u32,
}

impl FitScope {
    pub(crate) fn new(address: &ComponentAddress, ordinal: u32) -> Self {
        Self {
            address: address.clone(),
            ordinal,
        }
    }
    pub(crate) fn root() -> Self {
        Self::new(&ComponentAddress::root(), 0)
    }
    pub(crate) fn child(&self, role: AddressRole, index: usize) -> Result<Self, Error> {
        let index = u32::try_from(index)
            .map_err(|_| Error::Render("fit address index overflowed".into()))?;
        Ok(Self {
            address: self.address.child(role, index)?,
            ordinal: 0,
        })
    }
    pub(crate) fn failure(&self, kind: FitKind, message: impl Into<String>) -> FitFailure {
        FitFailure {
            reason: FitReason {
                kind,
                address: self.address.clone(),
                ordinal: self.ordinal,
            },
            message: message.into(),
        }
    }
}

pub(crate) enum FitOutcome<T> {
    Measured(T),
    Miss(FitFailure),
}

pub(crate) static ROOT_SCOPE: std::sync::LazyLock<FitScope> =
    std::sync::LazyLock::new(FitScope::root);

#[derive(Debug)]
pub(crate) enum PlanError {
    Fatal(Error),
    Fit(FitFailure),
}

impl From<Error> for PlanError {
    fn from(error: Error) -> Self {
        Self::Fatal(error)
    }
}

impl From<FitFailure> for PlanError {
    fn from(failure: FitFailure) -> Self {
        Self::Fit(failure)
    }
}

impl PlanError {
    pub(crate) fn into_error(self) -> Error {
        match self {
            Self::Fatal(error) => error,
            Self::Fit(failure) => failure.into_error(),
        }
    }
}

pub(crate) fn outcome<T>(result: Result<T, PlanError>) -> Result<FitOutcome<T>, Error> {
    match result {
        Ok(value) => Ok(FitOutcome::Measured(value)),
        Err(PlanError::Fit(failure)) => Ok(FitOutcome::Miss(failure)),
        Err(PlanError::Fatal(error)) => Err(error),
    }
}

pub(crate) fn measured<T>(result: Result<FitOutcome<T>, Error>) -> Result<T, PlanError> {
    match result? {
        FitOutcome::Measured(value) => Ok(value),
        FitOutcome::Miss(failure) => Err(PlanError::Fit(failure)),
    }
}

pub(crate) fn text(
    theme: &ResolvedTheme,
    source: &str,
    appearance: &TextAppearance,
    width: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    theme.prepare_font_environment()?;
    if width.is_finite() && width < 0.0 {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::Constraint,
            format!("{owner}: negative assigned text width"),
        )));
    }
    let width = crate::layout::maximum_width_quanta(width)
        .map_err(|error| Error::Render(format!("{owner}: {error}")))?;
    outcome(theme.plan_text_quanta_with_policy(
        source, appearance, width, alignment, owner, false, true, scope,
    ))
}

pub(crate) fn reserve<T>(
    theme: &ResolvedTheme,
    values: &mut Vec<T>,
    count: usize,
    owner: &str,
) -> Result<(), Error> {
    let environment = theme.prepared_font_environment()?;
    environment
        .claim_text_generation_slots(count as u128)
        .map_err(|error| Error::Render(format!("{owner}: {error:?}")))?;
    if count > values.capacity() - values.len() {
        let bytes = values
            .len()
            .checked_add(count)
            .and_then(|length| length.checked_mul(std::mem::size_of::<T>()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render(format!("{owner}: storage overflow")))?;
        environment
            .claim_unicode_bytes(bytes)
            .map_err(|error| Error::Render(format!("{owner}: {error}")))?;
    }
    values
        .try_reserve_exact(count)
        .map_err(|error| Error::Render(format!("{owner} allocation: {error}")))
}

/// Complete constructible siblings before choosing their least recoverable miss.
pub(crate) fn collect<T>(
    theme: &ResolvedTheme,
    owner: &str,
    values: impl Iterator<Item = Result<T, PlanError>>,
) -> Result<Vec<T>, PlanError> {
    let mut output = Vec::new();
    let count = values.size_hint().1.ok_or_else(|| {
        Error::Render(format!(
            "{owner}: sibling collection has no finite admission bound"
        ))
    })?;
    reserve(theme, &mut output, count, owner)?;
    let mut miss = None;
    for value in values {
        match value {
            Ok(value) => output.push(value),
            Err(PlanError::Fit(failure)) => FitFailure::retain_least(&mut miss, failure),
            Err(error) => return Err(error),
        }
    }
    match miss {
        Some(failure) => Err(failure.into()),
        None => Ok(output),
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "intrinsic quantum widths are compared with root-bounded pixel assignments"
)]
pub(crate) fn intrinsic_widths(
    theme: &ResolvedTheme,
    source: &str,
    appearance: &TextAppearance,
    owner: &str,
) -> Result<(f64, f64), Error> {
    theme.prepare_font_environment()?;
    let requirements = theme
        .prepared_font_environment()?
        .text_intrinsic_requirements(
            TextPlanInput {
                source,
                stack: &appearance.stack,
                weight: appearance.weight,
                font_size: appearance.font_size,
                line_height: TextLineHeight::Multiplier(appearance.line_height_multiplier),
                maximum_width_quanta: i64::MAX,
                alignment: PhysicalTextAlignment::Start,
                paint: appearance.paint,
                provenance: TextPlanProvenance {
                    pair_ordinal: 0,
                    owner: owner.into(),
                },
                exact_override: false,
            },
            1,
        )
        .map_err(|error| Error::Render(format!("{owner}: {error}")))?;
    Ok((
        requirements.hard_width_quanta as f64 / 1024.0,
        requirements
            .natural_width_quanta
            .max(requirements.preferred_width_quanta) as f64
            / 1024.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sibling_collection_preserves_later_fatal_and_canonical_child_identity() {
        let theme = crate::Theme::Publication.resolve();
        theme.prepare_font_environment().unwrap();
        let first = FitScope::root().child(AddressRole::CanvasCell, 0).unwrap();
        let result = collect::<()>(
            &theme,
            "fit siblings",
            [
                Err(first.failure(FitKind::Text, "earlier fit").into()),
                Err(Error::Data("later fatal".into()).into()),
            ]
            .into_iter(),
        );
        let Err(PlanError::Fatal(Error::Data(message))) = result else {
            panic!("later fatal was hidden")
        };
        assert_eq!(message, "later fatal");
        let expected = first.failure(FitKind::Text, "first child");
        let result = collect::<()>(
            &theme,
            "fit siblings",
            [
                Err(FitScope::root()
                    .child(AddressRole::CanvasCell, 1)
                    .unwrap()
                    .failure(FitKind::Text, "second child")
                    .into()),
                Err(first.failure(FitKind::Text, "first child").into()),
            ]
            .into_iter(),
        );
        let Err(PlanError::Fit(failure)) = result else {
            panic!("fit reason was lost")
        };
        assert_eq!(failure, expected);
    }

    #[test]
    fn inline_child_calls_do_not_inherit_the_parent_guide_attempt() {
        let parent = FitScope::new(&ComponentAddress::root(), 7);
        let child = parent.child(AddressRole::AxisLeft, 0).unwrap();
        assert_eq!(child.ordinal, 0);
        assert_eq!(
            child.address,
            parent.address.child(AddressRole::AxisLeft, 0).unwrap()
        );
    }

    #[test]
    fn exhausted_policy_selects_kind_then_address_then_local_attempt() {
        let kinds = [
            FitKind::Constraint,
            FitKind::Text,
            FitKind::Tick,
            FitKind::LegendOrGuide,
            FitKind::TrackOrResidual,
            FitKind::OverlayCollision,
            FitKind::SpecializedGeometry,
            FitKind::AssignedChild,
        ];
        assert!(kinds.windows(2).all(|pair| pair[0] < pair[1]));
        let first = FitScope::root().child(AddressRole::CanvasCell, 0).unwrap();
        let second = FitScope::root().child(AddressRole::CanvasCell, 1).unwrap();
        let later_attempt = FitScope::new(&first.address, 1);
        let expected = first.failure(FitKind::Text, "last diagnostic alphabetically");
        let mut least = None;
        for failure in [
            first.failure(FitKind::Tick, "a tick failure"),
            second.failure(FitKind::Text, "a later sibling"),
            later_attempt.failure(FitKind::Text, "a later attempt"),
            first.failure(FitKind::Text, "last diagnostic alphabetically"),
        ] {
            FitFailure::retain_least(&mut least, failure);
        }
        assert_eq!(least, Some(expected));
    }

    #[test]
    fn typed_propagation_preserves_the_original_child_reason() {
        let child = FitScope::root().child(AddressRole::FacetCell, 2).unwrap();
        let scope = FitScope::new(&child.address, 3);
        let failure = scope.failure(FitKind::LegendOrGuide, "guide does not fit");
        let expected = failure.reason.clone();
        let result: Result<(), PlanError> = Err(failure.into());
        let Err(PlanError::Fit(propagated)) = measured(outcome(result)) else {
            panic!("fit failure lost its typed channel");
        };
        assert_eq!(propagated.reason, expected);
        assert_eq!(propagated.message, "guide does not fit");
    }

    #[test]
    fn render_error_messages_never_select_recovery() {
        let result: Result<(), PlanError> = Err(Error::Render("text does not fit".into()).into());
        let Err(Error::Render(message)) = outcome(result) else {
            panic!("fatal producer error was reclassified as fit");
        };
        assert_eq!(message, "text does not fit");
    }
}
