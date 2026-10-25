//! Fixed complete geometric lattices and bounded preferred index sublattices.

use super::{AxisSide, NumericTickPlan, Projection, SourceKind, TickSource};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::numeric::{lattice::generation::LatticeRange, logarithm, operand::Operand};
use crate::theme::ResolvedTheme;
use crate::{Error, Result, TickRequirement};
use num_bigint::BigInt;

struct Positions {
    coordinates: Vec<f64>,
    operands: Vec<Operand>,
}

#[cfg(test)]
mod tests;

fn positions(
    range: &LatticeRange,
    first: &BigInt,
    last: &BigInt,
    stride: &BigInt,
    log: bool,
    maximum: usize,
) -> Result<Option<Positions>> {
    let first = logarithm::ceil(first, stride) * stride;
    let last = logarithm::floor(last, stride) * stride;
    if last < first {
        return Ok(None);
    }
    let count = (&last - &first) / stride + 1_u8;
    if count > BigInt::from(maximum) {
        return Ok(None);
    }
    let count = usize::try_from(count)
        .map_err(|_| Error::Render("multiplicative source count is not representable".into()))?;
    let mut coordinates = Vec::new();
    let mut operands = Vec::new();
    super::reserve(&mut coordinates, count)?;
    super::reserve(&mut operands, count)?;
    let mut formatter = crate::numeric::power::certified::PowerFormatter::default();
    for offset in 0..count {
        let operand = range.operand(&(&first + stride * offset))?;
        let coordinate = if log {
            operand.coordinate()?
        } else {
            formatter.lattice_binary(&operand)?
        };
        if coordinates
            .last()
            .is_some_and(|previous| *previous >= coordinate)
        {
            return Err(Error::Render(format!(
                "multiplicative occurrence {offset} collapses at binary64 coordinate precision"
            )));
        }
        coordinates.push(coordinate);
        operands.push(Operand::Multiplicative(crate::data::source::source_arc(
            operand,
        )?));
    }
    Ok(Some(Positions {
        coordinates,
        operands,
    }))
}

fn selected(
    positions: &Positions,
    log: bool,
    source: TickSource,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    format: Option<&crate::numeric::source::AxisRequest<'_>>,
) -> Result<Option<NumericTickPlan>> {
    let plan = if let Some(format) = format {
        NumericTickPlan::formatted(
            &positions.coordinates,
            &positions.operands,
            log,
            format,
            theme,
            source,
            projection,
            side,
        )?
    } else {
        NumericTickPlan::omitted(&positions.coordinates, &positions.operands, false, source)?
    };
    if let Some(plan) = plan {
        if super::labels_fit(&plan.ticks, projection, side, theme)? {
            return Ok(Some(plan));
        }
    }
    Ok(None)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the normalized locator consumes complete scientific and assigned-axis inputs"
)]
pub(super) fn plan(
    view: (f64, f64),
    log: bool,
    parameters: (f64, f64, TickRequirement),
    maximum: Option<usize>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    format: Option<&crate::numeric::source::AxisRequest<'_>>,
    scope: &FitScope,
) -> std::result::Result<NumericTickPlan, PlanError> {
    let (anchor, ratio, requirement) = parameters;
    let range = LatticeRange::new(view, log, anchor, ratio)?;
    let maximum = maximum.unwrap_or(usize::MAX);
    let count = range.count();
    let fixed = requirement == TickRequirement::Fixed;
    let seed = if fixed || count == BigInt::ZERO {
        1
    } else {
        super::measured_seed(projection, side, theme)?
    };
    let cap = if fixed { 1 } else { seed + 1 };
    let mut source = TickSource {
        kind: SourceKind::Multiplicative,
        ordinal: 0,
        cap,
        precision: None,
    };
    if count == BigInt::ZERO {
        return Ok(NumericTickPlan {
            ticks: Vec::new(),
            source,
            numeric: None,
        });
    }
    if fixed && count > BigInt::from(maximum) {
        return Err(Error::Render(
            "fixed multiplicative count exceeds its configured maximum or platform representation"
                .into(),
        )
        .into());
    }
    if let Some(positions) = positions(
        &range,
        &range.first,
        &range.last,
        &BigInt::from(1_u8),
        log,
        maximum,
    )? {
        if let Some(plan) = selected(&positions, log, source, projection, side, theme, format)? {
            return Ok(plan);
        }
    }
    if !fixed {
        source.ordinal += 1;
        let mut previous_stride = Some(BigInt::from(1_u8));
        for desired in (2..=seed).rev() {
            let stride = logarithm::ceil(&(&range.last - &range.first), &BigInt::from(desired - 1))
                .max(BigInt::from(1_u8));
            if previous_stride.as_ref() == Some(&stride) {
                continue;
            }
            previous_stride = Some(stride.clone());
            if let Some(positions) =
                positions(&range, &range.first, &range.last, &stride, log, maximum)?
            {
                if let Some(plan) =
                    selected(&positions, log, source, projection, side, theme, format)?
                {
                    return Ok(plan);
                }
            }
            source.ordinal += 1;
        }
        // This singleton is still an original lattice member, not an invented midpoint.
        let midpoint = logarithm::floor(&(&range.first + &range.last), &BigInt::from(2_u8));
        if let Some(positions) = positions(
            &range,
            &midpoint,
            &midpoint,
            &BigInt::from(1_u8),
            log,
            maximum,
        )? {
            if let Some(plan) = selected(&positions, log, source, projection, side, theme, format)?
            {
                return Ok(plan);
            }
        }
    }
    Err(scope
        .child(
            match side {
                AxisSide::Bottom => crate::layout::AddressRole::AxisBottom,
                AxisSide::Left => crate::layout::AddressRole::AxisLeft,
            },
            0,
        )?
        .failure(
            FitKind::Tick,
            "complete multiplicative source cannot fit its selected labels",
        )
        .into())
}
