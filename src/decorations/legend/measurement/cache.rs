//! Operation-local exact measurement reuse, scoped to the original guide owner.

use std::sync::Arc;

use super::{Legend, PreparedGuide};
use crate::layout::fit::{FitFailure, FitOutcome, FitScope};
use crate::layout::kernel::{LayoutScalar, SpatialRounding};
use crate::layout::ComponentAddress;
use crate::theme::{LegendTheme, ResolvedTheme, TextTheme};
use crate::Error;

#[derive(Debug, Default)]
pub(crate) struct GuideMeasurements {
    entries: Vec<CachedMeasurement>,
}

#[derive(Debug)]
struct CachedMeasurement {
    descriptor: Arc<PreparedGuide>,
    owner: ComponentAddress,
    ordinal: u32,
    width: i64,
    height: Option<i64>,
    horizontal: bool,
    compact: bool,
    compact_changes: bool,
    appearance: LegendTheme,
    title: TextTheme,
    label: TextTheme,
    outcome: CachedOutcome,
}

#[derive(Debug)]
enum CachedOutcome {
    Measured(Legend),
    Miss(FitFailure),
}

impl CachedOutcome {
    fn copy(&self) -> Result<FitOutcome<Legend>, Error> {
        match self {
            Self::Measured(legend) => Ok(FitOutcome::Measured(legend.clone())),
            Self::Miss(failure) => Ok(FitOutcome::Miss(copy_failure(failure)?)),
        }
    }
}

fn copy_failure(failure: &FitFailure) -> Result<FitFailure, Error> {
    let mut message = String::new();
    message
        .try_reserve_exact(failure.message.len())
        .map_err(|error| {
            Error::Render(format!("guide fit diagnostic allocation failed: {error}"))
        })?;
    message.push_str(&failure.message);
    Ok(FitFailure {
        reason: failure.reason.clone(),
        message,
    })
}

/// All measurement constraints are canonical before cache lookup. Assignment
/// validation still owns typed nonpositive-content misses under these values.
fn constraints(available: (f64, Option<f64>)) -> Result<(i64, Option<i64>), Error> {
    let scalar = |value| {
        let value = LayoutScalar::from_css_pixels(value, SpatialRounding::Floor)?.quanta();
        if value.unsigned_abs() > 1_u64 << 53 {
            return Err(Error::Render(
                "guide assignment cannot retain exact CSS quanta".into(),
            ));
        }
        Ok(value)
    };
    Ok((scalar(available.0)?, available.1.map(scalar).transpose()?))
}

pub(super) fn measure_assigned(
    descriptor: Arc<PreparedGuide>,
    available: (f64, Option<f64>),
    horizontal: bool,
    compact: bool,
    theme: &ResolvedTheme,
    compact_changes: &mut bool,
    scope: &FitScope,
) -> Result<FitOutcome<Legend>, Error> {
    let (width, height) = constraints(available)?;
    {
        let cache = theme
            .guide_measurements
            .lock()
            .map_err(|_| Error::Render("guide measurement cache lock poisoned".into()))?;
        for entry in &cache.entries {
            if entry.owner == scope.address
                && entry.ordinal == scope.ordinal
                && entry.width == width
                && entry.height == height
                && entry.horizontal == horizontal
                && (entry.compact == compact
                    || (compact && !entry.compact && !entry.compact_changes))
                && entry.descriptor.as_ref() == descriptor.as_ref()
                && entry.appearance.same(&theme.legend)
                && entry.title.same(&theme.typography.guides.legend_title)
                && entry.label.same(&theme.typography.guides.legend_label)
            {
                *compact_changes = entry.compact_changes;
                return entry.outcome.copy();
            }
        }
    }
    let outcome = super::measure_uncached(
        Arc::clone(&descriptor),
        available,
        horizontal,
        compact,
        theme,
        compact_changes,
        scope,
    )?;
    let cached = CachedMeasurement {
        descriptor,
        owner: scope.address.clone(),
        ordinal: scope.ordinal,
        width,
        height,
        horizontal,
        compact,
        compact_changes: *compact_changes,
        appearance: theme.legend.clone(),
        title: theme.typography.guides.legend_title.try_copy()?,
        label: theme.typography.guides.legend_label.try_copy()?,
        outcome: match &outcome {
            FitOutcome::Measured(legend) => CachedOutcome::Measured(legend.clone()),
            FitOutcome::Miss(failure) => CachedOutcome::Miss(copy_failure(failure)?),
        },
    };
    let mut cache = theme
        .guide_measurements
        .lock()
        .map_err(|_| Error::Render("guide measurement cache lock poisoned".into()))?;
    cache.entries.try_reserve(1).map_err(|error| {
        Error::Render(format!(
            "guide measurement cache allocation failed: {error}"
        ))
    })?;
    cache.entries.push(cached);
    Ok(outcome)
}
