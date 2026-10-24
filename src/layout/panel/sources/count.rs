//! One exact affine source, never a density seed or a coarsening candidate.

use super::{AxisSide, NumericTickPlan, Projection, SourceKind, TickSource};
use crate::layout::fit::{FitKind, FitScope, PlanError};
use crate::theme::ResolvedTheme;
use crate::{Error, TickSingleton, TickSpace};

pub(crate) struct CountPositions {
    pub(crate) coordinates: Vec<f64>,
    data: Option<Vec<f64>>,
    power: bool,
}

pub(crate) fn positions(
    range: (f64, f64),
    raw: Option<(f64, f64)>,
    parameters: (usize, TickSpace, TickSingleton),
    log: bool,
) -> Result<CountPositions, Error> {
    if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        return Err(Error::Data(
            "exact-count domain must be finite and strictly ascending".into(),
        ));
    }
    let (count, space, singleton) = parameters;
    let raw_mode = log && space == TickSpace::Data;
    let mut coordinates = Vec::new();
    super::reserve(&mut coordinates, count)?;
    let mut data = raw_mode.then(Vec::new);
    if let Some(data) = &mut data {
        super::reserve(data, count)?;
    }
    let mut formatter = crate::numeric::power::certified::PowerFormatter::default();
    for index in 0..count {
        let value = if count == 1 {
            singleton_value(range, raw, singleton, raw_mode, &mut formatter)?
        } else if raw_mode {
            if let Some((lower, upper)) = raw {
                crate::numeric::binary::affine(lower, upper, index, count - 1)?
            } else {
                formatter.affine_binary64(range, index, count - 1)?
            }
        } else {
            crate::numeric::binary::affine(range.0, range.1, index, count - 1)?
        };
        let coordinate = if raw_mode {
            if value <= 0.0 {
                return Err(Error::Render(
                    "exact data-space logarithmic position is not positive binary64".into(),
                ));
            }
            if (count >= 2 && index == 0) || (count == 1 && singleton == TickSingleton::Lower) {
                range.0
            } else if (count >= 2 && index == count - 1)
                || (count == 1 && singleton == TickSingleton::Upper)
            {
                range.1
            } else {
                crate::numeric::log10(value)?
            }
        } else {
            value
        };
        if !coordinate.is_finite()
            || coordinate < range.0
            || coordinate > range.1
            || coordinates
                .last()
                .is_some_and(|previous| *previous >= coordinate)
        {
            return Err(Error::Render(format!("exact-count position {index} is outside its domain or collapses at binary64 precision")));
        }
        coordinates.push(coordinate);
        if let Some(data) = &mut data {
            data.push(value);
        }
    }
    Ok(CountPositions {
        coordinates,
        data,
        power: log && !raw_mode,
    })
}

#[cfg(test)]
mod tests;

fn singleton_value(
    range: (f64, f64),
    raw: Option<(f64, f64)>,
    singleton: TickSingleton,
    raw_mode: bool,
    formatter: &mut crate::numeric::power::certified::PowerFormatter,
) -> Result<f64, Error> {
    let (index, denominator) = match singleton {
        TickSingleton::Lower => (0, 1),
        TickSingleton::Midpoint => (1, 2),
        TickSingleton::Upper => (1, 1),
        TickSingleton::Position(value) => {
            let inside = if raw_mode {
                value > 0.0
                    && if let Some((lower, upper)) = raw {
                        value >= lower && value <= upper
                    } else {
                        let exponent = crate::numeric::log10(value)?;
                        exponent >= range.0 && exponent <= range.1
                    }
            } else {
                value >= range.0 && value <= range.1
            };
            return if inside {
                Ok(value)
            } else {
                Err(Error::Data(
                    "exact singleton is outside its resolved domain".into(),
                ))
            };
        }
    };
    if raw_mode {
        if let Some((lower, upper)) = raw {
            crate::numeric::binary::affine(lower, upper, index, denominator)
        } else {
            formatter.affine_binary64(range, index, denominator)
        }
    } else {
        crate::numeric::binary::affine(range.0, range.1, index, denominator)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one complete exact source consumes the same assigned axis operands as automatic sources"
)]
pub(crate) fn plan(
    range: (f64, f64),
    parameters: (usize, TickSpace, TickSingleton),
    log: bool,
    raw: Option<(f64, f64)>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    format: Option<&crate::numeric::source::AxisRequest<'_>>,
    scope: &FitScope,
) -> Result<NumericTickPlan, PlanError> {
    let positions = positions(range, raw, parameters, log)?;
    let source = TickSource {
        kind: SourceKind::ExactCount,
        ordinal: 0,
        cap: 1,
        precision: None,
    };
    if positions.coordinates.is_empty() {
        return Ok(NumericTickPlan {
            ticks: Vec::new(),
            source,
            numeric: None,
        });
    }
    let operands = crate::data::source::collect_source(
        positions
            .data
            .as_deref()
            .unwrap_or(&positions.coordinates)
            .iter()
            .map(|value| {
                Ok(crate::numeric::operand::Operand::number(
                    *value,
                    positions.power,
                ))
            }),
    )?;
    let selected = if let Some(format) = format {
        NumericTickPlan::formatted(
            &positions.coordinates,
            &operands,
            log,
            format,
            theme,
            source,
            projection,
            side,
        )?
    } else {
        NumericTickPlan::omitted(&positions.coordinates, &operands, false, source)?
    };
    if let Some(selected) = selected {
        if super::labels_fit(&selected.ticks, projection, side, theme)? {
            return Ok(selected);
        }
    }
    Err(scope
        .failure(
            FitKind::Tick,
            "complete exact-count tick source cannot fit its selected labels",
        )
        .into())
}
