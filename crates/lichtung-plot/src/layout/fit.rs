//! Typed local fit outcomes. Fatal producer errors never select another layout.

use crate::layout::{AddressRole, ComponentAddress};
use crate::layout::{PhysicalTextAlignment, ResolvedTextPlan};
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

pub(crate) fn text<'a>(
    theme: &crate::text::TextRequestContext<'_, '_>,
    source: &str,
    appearance: impl Into<crate::text::TextAppearance<'a>>,
    width: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let appearance = appearance.into();
    if !appearance.hidden() {
        theme.text.prepare_font_environment()?;
    }
    if width.is_finite() && width < 0.0 {
        return Ok(FitOutcome::Miss(scope.failure(
            FitKind::Constraint,
            format!("{owner}: negative assigned text width"),
        )));
    }
    super::text::request::measure(theme, source, appearance, width, alignment, owner, scope)
}

/// Independent authored slots accept All/Hidden; reduction programs require a real label group.
pub(crate) fn single_text<'a>(
    theme: &crate::text::TextRequestContext<'_, '_>,
    source: &str,
    appearance: impl Into<crate::text::TextAppearance<'a>>,
    width: f64,
    alignment: PhysicalTextAlignment,
    owner: &str,
    scope: &FitScope,
) -> Result<FitOutcome<ResolvedTextPlan>, Error> {
    let appearance = appearance.into().require_single()?;
    text(theme, source, appearance, width, alignment, owner, scope)
}

pub(crate) fn reserve<T>(values: &mut Vec<T>, count: usize, owner: &str) -> Result<(), Error> {
    values
        .try_reserve_exact(count)
        .map_err(|error| Error::Render(format!("{owner} allocation: {error}")))
}

/// Complete constructible siblings before choosing their least recoverable miss.
pub(crate) fn collect<T>(
    owner: &str,
    values: impl Iterator<Item = Result<T, PlanError>>,
) -> Result<Vec<T>, PlanError> {
    let mut output = Vec::new();
    let count = values.size_hint().1.ok_or_else(|| {
        Error::Render(format!(
            "{owner}: sibling collection has no finite admission bound"
        ))
    })?;
    reserve(&mut output, count, owner)?;
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
pub(crate) fn intrinsic_widths<'a>(
    theme: &crate::text::TextRequestContext<'_, '_>,
    source: &str,
    appearance: impl Into<crate::text::TextAppearance<'a>>,
    owner: &str,
) -> Result<(f64, f64), Error> {
    let appearance = appearance.into();
    if appearance.hidden() {
        return Ok((0.0, 0.0));
    }
    let unwrapped = theme.measure_text(
        source,
        appearance.theme,
        PhysicalTextAlignment::Start,
        owner,
        appearance.exact_override,
    )?;
    let requirements = unwrapped
        .intrinsic_from_analysis(1)
        .map_err(|error| Error::Render(format!("{owner}: {error}")))?;
    let floor_appearance = appearance
        .layout
        .and_then(|request| request.instructions().fit)
        .map(|policy| {
            policy
                .bounds(appearance.theme.get_font_size())
                .and_then(|(minimum, _)| appearance.theme.clone().font_size(minimum))
        })
        .transpose()?;
    let floor_requirements = if let Some(style) = &floor_appearance {
        theme
            .measure_text(
                source,
                style,
                PhysicalTextAlignment::Start,
                owner,
                appearance.exact_override,
            )?
            .intrinsic_from_analysis(1)
            .map_err(|error| Error::Render(format!("{owner}: {error}")))?
    } else {
        requirements
    };
    let hard = if appearance.layout.is_some_and(|request| {
        request
            .instructions()
            .visibility
            .as_ref()
            .is_some_and(crate::TextVisibility::reduces)
            || request
                .instructions()
                .overflow
                .as_ref()
                .is_some_and(|overflow| {
                    overflow.failure != crate::TextFailure::Error || overflow.ellipsis.is_some()
                })
    }) {
        0.0
    } else if appearance.layout.is_some_and(|request| {
        request.instructions().wrapping == Some(crate::TextWrapping::NoSoftWrap)
    }) {
        floor_requirements.natural_width_quanta as f64 / 1024.0
    } else {
        floor_requirements.hard_width_quanta as f64 / 1024.0
    };
    let hard = appearance
        .layout
        .and_then(|request| request.instructions().bounds)
        .and_then(|bounds| bounds.width)
        .map_or(hard, |maximum| hard.min(maximum.get()));
    let natural = requirements
        .natural_width_quanta
        .max(requirements.preferred_width_quanta) as f64
        / 1024.0;
    if let Some(orientation) = appearance
        .layout
        .and_then(|request| request.instructions().orientation.as_ref())
    {
        let project = |angle: crate::TextRotation| {
            let (sine, cosine) = crate::text::orientation::components(angle.0.get());
            (
                hard * cosine.abs() + unwrapped.height() * sine.abs(),
                natural * cosine.abs() + unwrapped.height() * sine.abs(),
            )
        };
        return match &orientation.0 {
            crate::text::orientation::Orientation::Fixed(angle) => Ok(project(*angle)),
            crate::text::orientation::Orientation::Automatic(angles) => Ok(angles
                .iter()
                .map(|&angle| project(angle))
                .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1)))
                .unwrap_or((0.0, natural))),
            crate::text::orientation::Orientation::Data(from, to) => theme
                .reference
                .as_ref()
                .map_or(Ok((0.0, natural)), |frame| {
                    frame.rotation(from, to, theme).map(project)
                }),
        };
    }
    Ok((hard, natural))
}
