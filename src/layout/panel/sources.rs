//! Ordered, bounded numeric sources. A rejected source never materializes a subset.

use super::{AxisSide, AxisTick, Projection};
use crate::font::canonical::PreparedFontEnvironment;
use crate::layout::PhysicalTextAlignment;
use crate::theme::ResolvedTheme;
use crate::Error;

pub(super) const MAX_TICKS: usize = 10_000;
pub(crate) const NUMERIC_ALPHABET: &[char] = &[
    '-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'M', '^', 'e', 'k',
];
const MAX_INDEX: f64 = 9_007_199_254_740_992.0;
const FACTORS: [f64; 4] = [1.0, 2.0, 3.0, 5.0];

// The input stays private until T6 exposes interval setters.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TickInterval {
    Additive(f64),
    Log10Decades(u64),
}

impl TickInterval {
    pub(crate) fn validate(self) -> Result<(), Error> {
        if match self {
            Self::Additive(step) => step.is_finite() && step > 0.0,
            Self::Log10Decades(step) => step > 0,
        } {
            Ok(())
        } else {
            Err(Error::Data(
                "tick interval must be finite and strictly positive".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SourceKind {
    Supplied,
    Exact,
    Adaptive(usize),
    Additive(f64),
    LogIntegral,
    LogSubset(usize),
    LogInterval(u64),
    Midpoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TickSource {
    pub(crate) kind: SourceKind,
    pub(crate) ordinal: usize,
    pub(crate) cap: usize,
}

impl TickSource {
    pub(crate) const SUPPLIED: Self = Self {
        kind: SourceKind::Supplied,
        ordinal: 0,
        cap: 1,
    };
    pub(crate) const EXACT: Self = Self {
        kind: SourceKind::Exact,
        ordinal: 0,
        cap: 1,
    };
}

#[derive(Debug)]
pub(crate) struct NumericTickPlan {
    pub(crate) ticks: Vec<AxisTick>,
    pub(crate) source: TickSource,
}

/// Decimal literals are parsed by Rust's correctly rounded decimal conversion,
/// rather than libm pow/log at a decimal boundary.
pub(super) fn decimal_power(exponent: i32) -> f64 {
    // A bounded stack formatter avoids allocating a source string.
    let mut bytes = [0_u8; 16];
    let mut cursor = std::io::Cursor::new(bytes.as_mut_slice());
    std::io::Write::write_fmt(&mut cursor, format_args!("1e{exponent}")).expect("bounded exponent");
    let length = cursor.position() as usize;
    std::str::from_utf8(&bytes[..length])
        .expect("ASCII decimal")
        .parse()
        .expect("decimal power")
}

fn indices(range: (f64, f64), step: f64) -> Option<(i64, usize)> {
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    let first = (range.0 / step).ceil();
    let last = (range.1 / step).floor();
    if !first.is_finite()
        || !last.is_finite()
        || first.abs() > MAX_INDEX
        || last.abs() > MAX_INDEX
        || last < first
    {
        return None;
    }
    let first = first as i64;
    let count = usize::try_from(
        i128::from(last as i64)
            .checked_sub(i128::from(first))?
            .checked_add(1)?,
    )
    .ok()?;
    (1..=MAX_TICKS).contains(&count).then_some((first, count))
}

fn reserve<T>(
    values: &mut Vec<T>,
    count: usize,
    environment: Option<&PreparedFontEnvironment>,
) -> Result<(), Error> {
    if let Some(environment) = environment {
        environment
            .claim_processing_visits(count as u128)
            .map_err(|error| Error::Render(format!("numeric source work admission: {error:?}")))?;
        crate::render::backend::reserve_text_records(values, count, environment)?;
    } else {
        values
            .try_reserve_exact(count)
            .map_err(|error| Error::Render(error.to_string()))?;
    }
    Ok(())
}

fn linear_positions(
    range: (f64, f64),
    step: f64,
    environment: Option<&PreparedFontEnvironment>,
) -> Result<Option<Vec<f64>>, Error> {
    let Some((first, count)) = indices(range, step) else {
        return Ok(None);
    };
    let mut positions = Vec::new();
    reserve(&mut positions, count, environment)?;
    for offset in 0..count {
        let position = ((first + i64::try_from(offset).expect("bounded tick count")) as f64) * step;
        if !position.is_finite()
            || position < range.0
            || position > range.1
            || positions
                .last()
                .is_some_and(|previous| *previous >= position)
        {
            return Ok(None);
        }
        positions.push(position);
    }
    Ok(Some(positions))
}

fn endpoints(
    range: (f64, f64),
    environment: Option<&PreparedFontEnvironment>,
) -> Result<Option<Vec<f64>>, Error> {
    let mut positions = Vec::new();
    reserve(&mut positions, 2, environment)?;
    positions.extend([range.0, range.1]);
    Ok(Some(positions))
}

pub(super) fn adaptive_positions(
    range: (f64, f64),
    seed: usize,
    environment: Option<&PreparedFontEnvironment>,
) -> Result<Option<Vec<f64>>, Error> {
    if range.0.next_up() >= range.1 {
        return endpoints(range, environment);
    }
    let direct = range.1 - range.0;
    let raw = if direct.is_finite() {
        direct / seed as f64
    } else {
        let scale = range.0.abs().max(range.1.abs());
        ((range.1 / scale) - (range.0 / scale)) * (scale / seed as f64)
    };
    if !raw.is_finite() || raw <= 0.0 {
        return Ok(None);
    }
    let mut magnitude = None;
    for exponent in (-323..=308).rev() {
        if let Some(environment) = environment {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("decimal table admission: {error:?}")))?;
        }
        let power = decimal_power(exponent);
        if power <= raw {
            magnitude = Some(power);
            break;
        }
    }
    let mut best = None;
    for factor in [10.0, 5.0, 2.5, 2.0, 1.0] {
        if let Some(environment) = environment {
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("decimal step admission: {error:?}")))?;
        }
        let step = magnitude.map_or(raw, |magnitude| factor * magnitude);
        if let Some((_, count)) = indices(range, step) {
            let distance = count.abs_diff(seed);
            if best.is_none_or(|(_, previous)| distance < previous) {
                best = Some((step, distance));
            }
        }
        if magnitude.is_none() {
            break;
        }
    }
    let Some((step, _)) = best else {
        return Ok(None);
    };
    let Some(positions) = linear_positions(range, step, environment)? else {
        return Ok(None);
    };
    if positions.len() < 2 {
        endpoints(range, environment)
    } else {
        Ok(Some(positions))
    }
}

fn upper_seed_count(span: f64, ch_quanta: i64) -> Result<usize, Error> {
    let span = span_quanta(span)?;
    let pitch = ch_quanta
        .checked_mul(6)
        .ok_or_else(|| Error::Render("6ch pitch overflow".into()))?
        .max(1);
    Ok(1 + usize::try_from((span / pitch).clamp(1, 9_999))
        .map_err(|error| Error::Render(error.to_string()))?)
}

pub(super) fn span_quanta(span: f64) -> Result<i64, Error> {
    if !span.is_finite() || span < 0.0 {
        return Err(Error::Render(
            "numeric axis span must be finite and nonnegative".into(),
        ));
    }
    if span < 1.0 / 1024.0 {
        return Ok(0);
    }
    crate::layout::maximum_width_quanta(span).map_err(|error| Error::Render(error.to_string()))
}

struct Sources {
    range: (f64, f64),
    log: bool,
    interval: Option<TickInterval>,
    upper: usize,
    ordinal: usize,
    cap: usize,
    base: f64,
    log_base: u64,
    factor: usize,
    previous_step: Option<f64>,
    intervals_done: bool,
    terminal_emitted: bool,
}

impl Sources {
    fn new(
        range: (f64, f64),
        log: bool,
        interval: Option<TickInterval>,
        upper: usize,
    ) -> Result<Self, Error> {
        if !(1..=MAX_TICKS).contains(&upper) {
            return Err(Error::Render(
                "numeric axis seed cap must be in 1..=10000".into(),
            ));
        }
        if !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
            return Err(Error::Data(
                "numeric tick domain must be finite and strictly ascending".into(),
            ));
        }
        if let Some(interval) = interval {
            interval.validate()?;
            if !matches!(
                (log, interval),
                (false, TickInterval::Additive(_)) | (true, TickInterval::Log10Decades(_))
            ) {
                return Err(Error::Data(
                    "tick interval does not match the axis scale".into(),
                ));
            }
        }
        // Binary64's full decimal span has at most 633 decade bases. Integer
        // intervals have at most 20. Include the one terminal midpoint source.
        let integral_count =
            (range.1.min(308.0).floor() - range.0.max(-323.0).ceil() + 1.0).max(0.0) as usize;
        let cap = match interval {
            Some(TickInterval::Additive(_)) => 4 * 633 + 1,
            Some(TickInterval::Log10Decades(_)) => 4 * 20 + 1,
            None if log && integral_count >= 2 => upper.min(integral_count) + 1,
            None => upper + 1,
        };
        Ok(Self {
            range,
            log,
            interval,
            upper,
            ordinal: 0,
            cap,
            base: if let Some(TickInterval::Additive(step)) = interval {
                step
            } else {
                0.0
            },
            log_base: if let Some(TickInterval::Log10Decades(step)) = interval {
                step
            } else {
                0
            },
            factor: 0,
            previous_step: None,
            intervals_done: false,
            terminal_emitted: false,
        })
    }

    fn next_kind(&mut self) -> Option<SourceKind> {
        if self.interval.is_some() && !self.intervals_done {
            loop {
                let maximum = if self.log {
                    self.range
                        .0
                        .max(-323.0)
                        .abs()
                        .max(self.range.1.min(308.0).abs())
                } else {
                    self.range.0.abs().max(self.range.1.abs())
                };
                let factor = self.factor;
                self.factor += 1;
                let kind = match self.interval? {
                    TickInterval::Additive(_) => {
                        let step = self.base * FACTORS[factor];
                        let kind = (step.is_finite() && self.previous_step != Some(step))
                            .then_some(SourceKind::Additive(step));
                        if kind.is_some() {
                            self.previous_step = Some(step);
                        }
                        kind
                    }
                    TickInterval::Log10Decades(_) => self
                        .log_base
                        .checked_mul([1, 2, 3, 5][factor])
                        .map(SourceKind::LogInterval),
                };
                if self.factor == 4 {
                    self.factor = 0;
                    if self.log {
                        let next = self.log_base.checked_mul(10);
                        self.intervals_done = self.log_base as f64 > maximum || next.is_none();
                        if let Some(next) = next {
                            self.log_base = next;
                        }
                    } else {
                        let next = self.base * 10.0;
                        self.intervals_done = self.base > maximum || !next.is_finite();
                        self.base = next;
                    }
                }
                if let Some(kind) = kind {
                    return Some(kind);
                }
                if self.intervals_done {
                    break;
                }
            }
        }
        if self.interval.is_none() {
            let count = (self.range.1.min(308.0).floor() - self.range.0.max(-323.0).ceil() + 1.0)
                .max(0.0) as usize;
            if self.log && count >= 2 {
                if self.ordinal == 0 {
                    return Some(SourceKind::LogIntegral);
                }
                let desired = self.upper.min(count).saturating_sub(self.ordinal - 1);
                if desired >= 2 {
                    return Some(SourceKind::LogSubset(desired));
                }
            } else if self.ordinal < self.upper {
                return Some(SourceKind::Adaptive(self.upper - self.ordinal));
            }
        }
        Some(SourceKind::Midpoint)
    }

    fn next(
        &mut self,
        environment: &PreparedFontEnvironment,
    ) -> Result<(TickSource, Option<Vec<f64>>), Error> {
        if self.ordinal >= self.cap {
            return Err(Error::Render("numeric axis source cap exhausted".into()));
        }
        if self.terminal_emitted {
            return Err(Error::Render(
                "numeric axis source order exhausted after midpoint".into(),
            ));
        }
        environment
            .claim_processing_visits(1)
            .map_err(|error| Error::Render(format!("axis source admission: {error:?}")))?;
        environment
            .claim_text_generation_slots(1)
            .map_err(|error| Error::Render(format!("axis source admission: {error:?}")))?;
        environment
            .claim_unicode_bytes(std::mem::size_of::<TickSource>() as u64)
            .map_err(|error| Error::Render(error.to_string()))?;
        let kind = self
            .next_kind()
            .ok_or_else(|| Error::Render("numeric source order exhausted".into()))?;
        self.terminal_emitted = kind == SourceKind::Midpoint;
        let source = TickSource {
            kind,
            ordinal: self.ordinal,
            cap: self.cap,
        };
        self.ordinal += 1;
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::TickSource,
        );
        let positions = match kind {
            SourceKind::Adaptive(seed) => adaptive_positions(self.range, seed, Some(environment))?,
            SourceKind::Additive(step) => linear_positions(self.range, step, Some(environment))?,
            SourceKind::LogInterval(step) => {
                let clamped = (self.range.0.max(-323.0), self.range.1.min(308.0));
                if clamped.0 > clamped.1 {
                    self.intervals_done = true;
                    None
                } else {
                    linear_positions(clamped, step as f64, Some(environment))?
                }
            }
            SourceKind::LogIntegral | SourceKind::LogSubset(_) => {
                let first = self.range.0.max(-323.0).ceil() as i32;
                let count = (self.range.1.min(308.0).floor() as i32 - first + 1) as usize;
                let desired = if let SourceKind::LogSubset(desired) = kind {
                    desired
                } else {
                    count
                };
                let mut positions = Vec::new();
                reserve(&mut positions, desired, Some(environment))?;
                for index in 0..desired {
                    positions.push(
                        f64::from(first)
                            + ((index * (count - 1) + (desired - 1) / 2) / (desired - 1)) as f64,
                    );
                }
                Some(positions)
            }
            SourceKind::Midpoint => {
                let mut positions = Vec::new();
                reserve(&mut positions, 1, Some(environment))?;
                positions.push(f64::midpoint(self.range.0, self.range.1));
                Some(positions)
            }
            SourceKind::Supplied | SourceKind::Exact => unreachable!("not generated sources"),
        };
        Ok((source, positions))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn numeric_tick_plan(
    range: (f64, f64),
    interval: Option<TickInterval>,
    log: bool,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
    scope: &crate::layout::fit::FitScope,
) -> Result<NumericTickPlan, crate::layout::fit::PlanError> {
    // Validate semantic inputs before asking for the metric probe or environment.
    Sources::new(range, log, interval, 1)?;
    let probe = if legacy {
        theme.measure_legacy_appearance_text(
            "0",
            &theme.typography.tick_label,
            PhysicalTextAlignment::Center,
            "numeric tick pitch",
            false,
        )
    } else {
        theme.measure_text(
            "0",
            &theme.typography.tick_label,
            PhysicalTextAlignment::Center,
            "numeric tick pitch",
            false,
        )
    }?;
    let (x0, x1, y0, y1) = projection.pixel_box();
    let span = match side {
        AxisSide::Bottom => x1 - x0,
        AxisSide::Left => y1 - y0,
    };
    let upper = upper_seed_count(span, probe.lines[0].logical_advance_quanta)?;
    let environment = theme.prepared_font_environment()?;
    let mut sources = Sources::new(range, log, interval, upper)?;
    loop {
        let (source, positions) = sources.next(environment)?;
        if let Some(positions) = positions {
            // Each formatting attempt is admitted before strings or record storage.
            let mut ticks = Vec::new();
            reserve(&mut ticks, positions.len(), Some(environment))?;
            let mut unique = false;
            for precision in 0..=17 {
                if precision == 1 {
                    continue;
                }
                ticks.clear();
                environment
                    .claim_processing_visits(positions.len() as u128)
                    .map_err(|error| Error::Render(format!("tick label admission: {error:?}")))?;
                for &position in &positions {
                    let integral = log
                        && matches!(
                            source.kind,
                            SourceKind::LogIntegral
                                | SourceKind::LogSubset(_)
                                | SourceKind::LogInterval(_)
                        );
                    let label =
                        format_label(position, log, integral, precision, Some(environment))?;
                    ticks.push(AxisTick::new(position, label));
                }
                // Formatting of an ordered numeric source is monotone. Only
                // adjacent labels can coincide, including SI boundary rounding.
                unique = ticks.windows(2).all(|pair| pair[0].label != pair[1].label);
                if unique {
                    break;
                }
            }
            #[cfg(test)]
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::TickDistinctList,
            );
            environment
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("axis fit admission: {error:?}")))?;
            if unique && labels_fit(&ticks, projection, side, theme, legacy)? {
                return Ok(NumericTickPlan { ticks, source });
            }
        }
        if source.kind == SourceKind::Midpoint {
            #[cfg(test)]
            crate::orchestrator::measurements::count(
                crate::orchestrator::measurements::Counter::TickFitMiss,
            );
            return Err(scope
                .child(
                    match side {
                        AxisSide::Bottom => crate::layout::AddressRole::AxisBottom,
                        AxisSide::Left => crate::layout::AddressRole::AxisLeft,
                    },
                    0,
                )?
                .failure(
                    crate::layout::fit::FitKind::Tick,
                    "numeric axis terminal midpoint label does not fit assigned span",
                )
                .into());
        }
    }
}

pub(super) fn format_label(
    value: f64,
    log: bool,
    integral: bool,
    precision: usize,
    environment: Option<&PreparedFontEnvironment>,
) -> Result<String, Error> {
    use std::io::Write;
    let mut buffer = [0_u8; 64];
    let mut cursor = std::io::Cursor::new(buffer.as_mut_slice());
    let value = if value == 0.0 { 0.0 } else { value };
    let mut fixed = false;
    let mut suffix = "";
    let write = (|| -> std::io::Result<()> {
        if integral {
            let exponent = value as i32;
            let literal = match exponent {
                -2 => Some("0.01"),
                -1 => Some("0.1"),
                0 => Some("1"),
                1 => Some("10"),
                2 => Some("100"),
                3 => Some("1000"),
                4 => Some("10k"),
                5 => Some("100k"),
                6 => Some("1M"),
                7 => Some("10M"),
                8 => Some("100M"),
                9 => Some("1000M"),
                10 => Some("10000M"),
                11 => Some("100000M"),
                _ => None,
            };
            return if let Some(literal) = literal {
                cursor.write_all(literal.as_bytes())
            } else {
                write!(cursor, "1.00e{exponent}")
            };
        }
        if log {
            cursor.write_all(b"10^")?;
        }
        let absolute = value.abs();
        if precision > 0 {
            write!(cursor, "{value:.precision$e}")
        } else if absolute >= 1e12 || (absolute > 0.0 && absolute < 0.01) {
            write!(cursor, "{value:.2e}")
        } else {
            fixed = true;
            if absolute >= 1_000_000.0 {
                suffix = "M";
                write!(cursor, "{:.1}", value / 1_000_000.0)
            } else if absolute >= 10_000.0 {
                suffix = "k";
                write!(cursor, "{:.1}", value / 1000.0)
            } else if absolute >= 100.0 {
                fixed = false;
                write!(cursor, "{value:.0}")
            } else if absolute >= 1.0 {
                write!(cursor, "{value:.1}")
            } else {
                write!(cursor, "{value:.2}")
            }
        }
    })();
    write.map_err(|error| Error::Render(format!("bounded numeric formatter: {error}")))?;
    let length = cursor.position() as usize;
    let text = std::str::from_utf8(&buffer[..length]).expect("ASCII numeric formatter");
    let text = if fixed {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text
    };
    let length = text
        .len()
        .checked_add(suffix.len())
        .ok_or_else(|| Error::Render("numeric label bytes overflow".into()))?;
    if let Some(environment) = environment {
        environment
            .claim_unicode_bytes(length as u64)
            .map_err(|error| Error::Render(error.to_string()))?;
    }
    let mut label = String::new();
    label
        .try_reserve_exact(length)
        .map_err(|error| Error::Render(error.to_string()))?;
    label.push_str(text);
    label.push_str(suffix);
    Ok(label)
}

#[allow(
    clippy::float_cmp,
    reason = "axis endpoint identity is exact, not a tolerance test"
)]
pub(super) fn projection_key(value: f64, range: (f64, f64)) -> Option<u64> {
    let fraction = if value == range.0 {
        0.0
    } else if value == range.1 {
        1.0
    } else {
        let difference = range.1 - range.0;
        if difference.is_finite() {
            (value - range.0) / difference
        } else {
            let scale = range.0.abs().max(range.1.abs());
            ((value / scale) - (range.0 / scale)) / ((range.1 / scale) - (range.0 / scale))
        }
    };
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return None;
    }
    if fraction == 1.0 {
        return Some(u64::MAX);
    }
    let bits = fraction.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
    let shift = (1075 - exponent.max(1)) as u32;
    let numerator = u128::from(mantissa) * u128::from(u64::MAX);
    if shift >= 128 {
        return Some(0);
    }
    let quotient = numerator >> shift;
    let remainder = numerator & ((1_u128 << shift) - 1);
    let halfway = 1_u128 << (shift - 1);
    u64::try_from(
        quotient
            + u128::from(
                remainder > halfway || (remainder == halfway && !quotient.is_multiple_of(2)),
            ),
    )
    .ok()
}

pub(super) fn projected_center(key: u64, span: i64) -> Option<i64> {
    let numerator = u128::try_from(span).ok()? * u128::from(key);
    let denominator = u128::from(u64::MAX);
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    i64::try_from(
        quotient
            + u128::from(
                2 * remainder > denominator || (2 * remainder == denominator && quotient % 2 != 0),
            ),
    )
    .ok()
}

pub(super) fn labels_fit(
    ticks: &[AxisTick],
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<bool, Error> {
    let cells = match super::cells::assign(
        ticks,
        false,
        projection,
        side,
        theme,
        &crate::layout::fit::FitScope::root(),
    ) {
        Ok(cells) => cells,
        Err(crate::layout::fit::PlanError::Fit(_)) => return Ok(false),
        Err(crate::layout::fit::PlanError::Fatal(error)) => return Err(error),
    };
    for (tick, cell) in ticks.iter().zip(cells) {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::TickMetricLookup,
        );
        let plan = if legacy {
            theme.measure_legacy_appearance_text(
                &tick.label,
                &theme.typography.tick_label,
                PhysicalTextAlignment::Center,
                "automatic tick label",
                false,
            )
        } else {
            theme.measure_text(
                &tick.label,
                &theme.typography.tick_label,
                PhysicalTextAlignment::Center,
                "automatic tick label",
                false,
            )
        }?;
        let extent = match side {
            AxisSide::Bottom => plan.width().max(plan.fit_width()),
            AxisSide::Left => plan.height(),
        };
        if extent > cell.capacity() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
