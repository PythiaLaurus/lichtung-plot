//! Ordered, bounded numeric sources. A rejected source never materializes a subset.

use super::{AxisSide, AxisTick, Projection};
use crate::layout::PhysicalTextAlignment;
use crate::theme::ResolvedTheme;
use crate::{Error, TickGeneration};

// Only the automatic candidate program is bounded by this private seed grammar.
pub(super) const MAX_TICKS: usize = 10_000;
pub(crate) mod count;
mod geometric;
pub(super) mod interval;
const MAX_INDEX: f64 = 9_007_199_254_740_992.0;
const FACTORS: [f64; 4] = [1.0, 2.0, 3.0, 5.0];

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
    ExactCount,
    Multiplicative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TickSource {
    pub(crate) kind: SourceKind,
    pub(crate) ordinal: usize,
    pub(crate) cap: usize,
    pub(crate) precision: Option<u8>,
}

impl TickSource {
    pub(crate) const SUPPLIED: Self = Self {
        kind: SourceKind::Supplied,
        ordinal: 0,
        cap: 1,
        precision: None,
    };
    pub(crate) const EXACT: Self = Self {
        kind: SourceKind::Exact,
        ordinal: 0,
        cap: 1,
        precision: None,
    };
}

#[derive(Debug)]
pub(crate) struct NumericTickPlan {
    pub(crate) ticks: Vec<AxisTick>,
    pub(crate) source: TickSource,
    pub(crate) numeric: Option<std::sync::Arc<crate::numeric::set::FormattedSet>>,
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
    (count > 0).then_some((first, count))
}

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), Error> {
    crate::render::backend::reserve_text_records(values, count)
}

fn linear_positions(
    range: (f64, f64),
    step: f64,
    maximum: usize,
) -> Result<Option<Vec<f64>>, Error> {
    Ok(interval::positions(
        range,
        None,
        false,
        (
            step,
            0.0,
            crate::TickSpace::Data,
            crate::TickRequirement::Preferred,
        ),
        maximum,
    )?
    .and_then(|positions| (!positions.coordinates.is_empty()).then_some(positions.coordinates)))
}

fn endpoints(range: (f64, f64), maximum: usize) -> Result<Option<Vec<f64>>, Error> {
    if maximum < 2 {
        return Ok(None);
    }
    let mut positions = Vec::new();
    reserve(&mut positions, 2)?;
    positions.extend([range.0, range.1]);
    Ok(Some(positions))
}

pub(super) fn adaptive_positions(
    range: (f64, f64),
    seed: usize,
    maximum: usize,
) -> Result<Option<Vec<f64>>, Error> {
    if range.0.next_up() >= range.1 {
        return endpoints(range, maximum);
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
        let power = decimal_power(exponent);
        if power <= raw {
            magnitude = Some(power);
            break;
        }
    }
    let mut best = None;
    for factor in [10.0, 5.0, 2.5, 2.0, 1.0] {
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
    let Some(positions) = linear_positions(range, step, maximum)? else {
        return Ok(None);
    };
    if positions.len() < 2 {
        endpoints(range, maximum)
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

fn measured_seed(
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<usize, Error> {
    let probe = if legacy {
        theme.measure_legacy_appearance_text(
            "0",
            &theme.typography.axes.tick_label,
            PhysicalTextAlignment::Center,
            "numeric tick pitch",
            false,
        )
    } else {
        theme.measure_text(
            "0",
            &theme.typography.axes.tick_label,
            PhysicalTextAlignment::Center,
            "numeric tick pitch",
            false,
        )
    }?;
    let (x0, x1, y0, y1) = projection.pixel_box();
    let (_, span) = match side {
        AxisSide::Bottom => super::projection::axis_quanta(x0, x1)?,
        AxisSide::Left => super::projection::axis_quanta(y0, y1)?,
    };
    upper_seed_count(span as f64 / 1024.0, probe.lines[0].logical_advance_quanta)
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
    generation: TickGeneration,
    upper: usize,
    ordinal: usize,
    cap: usize,
    base: f64,
    log_base: u64,
    factor: usize,
    previous_step: Option<f64>,
    intervals_done: bool,
    terminal_emitted: bool,
    raw: Option<(f64, f64)>,
    operands: Option<Vec<crate::numeric::operand::Operand>>,
}

impl Sources {
    fn new(
        range: (f64, f64),
        log: bool,
        generation: TickGeneration,
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
        generation.validate_scale(log)?;
        // Binary64's full decimal span has at most 633 decade bases. Integer
        // intervals have at most 20. Include the one terminal midpoint source.
        let integral_count =
            (range.1.min(308.0).floor() - range.0.max(-323.0).ceil() + 1.0).max(0.0) as usize;
        let target = generation.target_count();
        let upper = target.unwrap_or(upper);
        let singleton = target == Some(1)
            || (!generation.is_interval() && generation.get_maximum_count() == Some(1));
        let cap = match &generation {
            value
                if value
                    .interval_parameters()
                    .is_some_and(|parameters| parameters.3 == crate::TickRequirement::Fixed) =>
            {
                1
            }
            _ if singleton => 1,
            value if value.decade_step().is_some() => 4 * 20 + 1,
            value if value.is_interval() => 4 * 633 + 1,
            _ if log && integral_count >= 2 => {
                upper.min(integral_count) + usize::from(target.is_none())
            }
            _ => upper
                .checked_add(1)
                .ok_or_else(|| Error::Render("tick source program length overflow".into()))?,
        };
        let base = generation
            .interval_parameters()
            .map_or(0.0, |parameters| parameters.0);
        let log_base = generation.decade_step().map_or(0, u64::from);
        Ok(Self {
            range,
            log,
            generation,
            upper,
            ordinal: 0,
            cap,
            base,
            log_base,
            factor: 0,
            previous_step: None,
            intervals_done: false,
            terminal_emitted: false,
            raw: None,
            operands: None,
        })
    }

    fn next_kind(&mut self) -> SourceKind {
        let interval = self.generation.is_interval();
        let target = self.generation.target_count();
        if self.fixed() {
            return SourceKind::Additive(self.base);
        }
        if target == Some(1) || (!interval && self.generation.get_maximum_count() == Some(1)) {
            return SourceKind::Midpoint;
        }
        if interval && !self.intervals_done {
            loop {
                let decades = self.generation.decade_step().is_some();
                let maximum = if decades {
                    self.range
                        .0
                        .max(-323.0)
                        .abs()
                        .max(self.range.1.min(308.0).abs())
                } else {
                    let view = if self.log
                        && self
                            .generation
                            .interval_parameters()
                            .is_some_and(|parameters| parameters.2 == crate::TickSpace::Data)
                    {
                        self.raw.unwrap_or(self.range)
                    } else {
                        self.range
                    };
                    let origin = self
                        .generation
                        .interval_parameters()
                        .map_or(0.0, |parameters| parameters.1);
                    (view.0 - origin).abs().max((view.1 - origin).abs())
                };
                let factor = self.factor;
                self.factor += 1;
                let kind = if decades {
                    self.log_base
                        .checked_mul([1, 2, 3, 5][factor])
                        .map(SourceKind::LogInterval)
                } else {
                    let step = self.base * FACTORS[factor];
                    let kind = (step.is_finite() && self.previous_step != Some(step))
                        .then_some(SourceKind::Additive(step));
                    if kind.is_some() {
                        self.previous_step = Some(step);
                    }
                    kind
                };
                if self.factor == 4 {
                    self.factor = 0;
                    if decades {
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
                    return kind;
                }
                if self.intervals_done {
                    break;
                }
            }
        }
        if !interval {
            let count = (self.range.1.min(308.0).floor() - self.range.0.max(-323.0).ceil() + 1.0)
                .max(0.0) as usize;
            if self.log && count >= 2 {
                if target.is_some() {
                    let desired = self.upper.min(count).saturating_sub(self.ordinal);
                    return if desired < 2 {
                        SourceKind::Midpoint
                    } else if desired == count {
                        SourceKind::LogIntegral
                    } else {
                        SourceKind::LogSubset(desired)
                    };
                }
                if self.ordinal == 0 {
                    return SourceKind::LogIntegral;
                }
                let desired = self.upper.min(count).saturating_sub(self.ordinal - 1);
                if desired >= 2 {
                    return SourceKind::LogSubset(desired);
                }
            } else if self.ordinal < self.upper {
                return SourceKind::Adaptive(self.upper - self.ordinal);
            }
        }
        SourceKind::Midpoint
    }

    fn next(&mut self) -> Result<(TickSource, Option<Vec<f64>>), Error> {
        if self.ordinal >= self.cap {
            return Err(Error::Render("numeric axis source cap exhausted".into()));
        }
        if self.terminal_emitted {
            return Err(Error::Render(
                "numeric axis source order exhausted after midpoint".into(),
            ));
        }

        if self.log
            && self
                .generation
                .interval_parameters()
                .is_some_and(|parameters| parameters.2 == crate::TickSpace::Data)
        {
            self.raw = Some(interval::data_range(self.range, self.raw)?);
        }
        self.operands = None;
        let kind = self.next_kind();
        self.terminal_emitted = kind == SourceKind::Midpoint || self.fixed();
        let source = TickSource {
            kind,
            ordinal: self.ordinal,
            cap: self.cap,
            precision: None,
        };
        self.ordinal += 1;
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::TickSource,
        );
        let mut maximum = self.generation.get_maximum_count().unwrap_or(
            if self.generation.target_count().is_some() || self.generation.is_interval() {
                usize::MAX
            } else {
                MAX_TICKS
            },
        );
        if !self.generation.is_interval() && self.generation.target_count().is_none() {
            maximum = maximum.min(MAX_TICKS);
        }
        let positions = match kind {
            SourceKind::Adaptive(seed) => adaptive_positions(self.range, seed, maximum)?,
            SourceKind::Additive(step) => self.interval_positions(self.range, step, maximum)?,
            SourceKind::LogInterval(step) => {
                let clamped = (self.range.0.max(-323.0), self.range.1.min(308.0));
                if clamped.0 > clamped.1 {
                    self.intervals_done = true;
                    None
                } else {
                    self.interval_positions(clamped, step as f64, maximum)?
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
                if desired > maximum {
                    return Ok((source, None));
                }
                let mut positions = Vec::new();
                reserve(&mut positions, desired)?;
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
                reserve(&mut positions, 1)?;
                positions.push(f64::midpoint(self.range.0, self.range.1));
                Some(positions)
            }
            SourceKind::Supplied
            | SourceKind::Exact
            | SourceKind::ExactCount
            | SourceKind::Multiplicative => {
                unreachable!("not adaptive sources")
            }
        };
        Ok((source, positions))
    }

    fn fixed(&self) -> bool {
        self.generation
            .interval_parameters()
            .is_some_and(|parameters| parameters.3 == crate::TickRequirement::Fixed)
    }

    fn interval_positions(
        &mut self,
        range: (f64, f64),
        step: f64,
        maximum: usize,
    ) -> Result<Option<Vec<f64>>, Error> {
        let (_, origin, space, requirement) = self
            .generation
            .interval_parameters()
            .ok_or_else(|| Error::Render("interval source lacks its normalized policy".into()))?;
        let positions = interval::positions(
            range,
            self.raw,
            self.log,
            (step, origin, space, requirement),
            maximum,
        )?;
        let Some(positions) = positions else {
            return if self.fixed() {
                Err(Error::Render(
                    "fixed interval count exceeds its configured maximum or storage representation"
                        .into(),
                ))
            } else {
                Ok(None)
            };
        };
        if positions.coordinates.is_empty() && !self.fixed() {
            return Ok(None);
        }
        self.operands = positions
            .data
            .map(|data| {
                crate::data::source::collect_source(
                    data.into_iter()
                        .map(|value| Ok(crate::numeric::operand::Operand::number(value, false))),
                )
            })
            .transpose()?;
        Ok(Some(positions.coordinates))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn numeric_tick_plan(
    range: (f64, f64),
    generation: TickGeneration,
    log: bool,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
    format: Option<&crate::numeric::source::AxisRequest<'_>>,
    scope: &crate::layout::fit::FitScope,
) -> Result<NumericTickPlan, crate::layout::fit::PlanError> {
    NumericTickPlan::measure_selected_source(
        range, generation, log, projection, side, theme, legacy, format, scope, None,
    )
}

impl NumericTickPlan {
    fn omitted(
        positions: &[f64],
        operands: &[crate::numeric::operand::Operand],
        integral: bool,
        mut source: TickSource,
    ) -> Result<Option<Self>, Error> {
        let mut formatter = crate::numeric::power::certified::PowerFormatter::default();
        for precision in std::iter::once(0).chain(2..=17) {
            let ticks =
                crate::data::source::collect_source(positions.iter().copied().zip(operands).map(
                    |(position, value)| {
                        Ok(AxisTick::new(
                            position,
                            crate::numeric::profiles::format_operand(
                                value,
                                integral,
                                precision,
                                &mut formatter,
                            )?,
                        ))
                    },
                ))?;
            if ticks.windows(2).all(|pair| pair[0].label != pair[1].label) {
                source.precision = Some(u8::try_from(precision).expect("bounded precision"));
                let set = crate::numeric::set::FormattedSet {
                    members: crate::data::source::collect_source(operands.iter().cloned().map(Ok))?,
                    labels: crate::data::source::collect_source(
                        ticks
                            .iter()
                            .map(|tick| crate::data::source::copy_source_string(&tick.label)),
                    )?,
                    precision: source.precision.expect("selected precision"),
                    ..crate::numeric::set::FormattedSet::default()
                };
                return Ok(Some(Self {
                    ticks,
                    source,
                    numeric: Some(crate::data::source::source_arc(set)?),
                }));
            }
        }
        Ok(None)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "scientific coordinates and retained operands may differ on a logarithmic data-space source"
    )]
    fn formatted(
        positions: &[f64],
        operands: &[crate::numeric::operand::Operand],
        log: bool,
        format: &crate::numeric::source::AxisRequest<'_>,
        theme: &ResolvedTheme,
        mut source: TickSource,
        projection: &Projection,
        side: AxisSide,
    ) -> Result<Option<Self>, Error> {
        let range = match side {
            AxisSide::Bottom => projection.x_range(),
            AxisSide::Left => projection.y_range(),
        };
        let Some(set) =
            format.materialize(operands, log, side, range, crate::NumericPurpose::ActualSet)?
        else {
            return Ok(None);
        };
        source.precision = Some(set.precision);
        let ticks =
            crate::data::source::collect_source(positions.iter().copied().zip(&set.labels).map(
                |(position, label)| {
                    Ok(AxisTick::new(
                        position,
                        crate::data::source::copy_source_string(label)?,
                    ))
                },
            ))?;
        if let Some(explanation) = &set.explanation {
            let appearance = explanation.appearance(&theme.typography.axes.tick_label);
            theme
                .prepared_font_environment()?
                .admit_text(
                    &appearance.theme.stack,
                    appearance.theme.weight.into(),
                    explanation.as_str(),
                    None,
                )
                .map_err(|error| {
                    Error::Render(format!("numeric explanation admission: {error}"))
                })?;
        }
        Ok(Some(Self {
            ticks,
            source,
            numeric: Some(crate::data::source::source_arc(set)?),
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn measure_selected_source(
        range: (f64, f64),
        generation: TickGeneration,
        log: bool,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
        legacy: bool,
        format: Option<&crate::numeric::source::AxisRequest<'_>>,
        scope: &crate::layout::fit::FitScope,
        raw: Option<(f64, f64)>,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        if let Some(parameters) = generation.exact_parameters() {
            return count::plan(
                range, parameters, log, raw, projection, side, theme, format, scope,
            );
        }
        if let Some(parameters) = generation.multiplicative_parameters() {
            return geometric::plan(
                range,
                log,
                parameters,
                generation.get_maximum_count(),
                projection,
                side,
                theme,
                format,
                scope,
            );
        }
        // Validate semantic inputs before asking for the metric probe or environment.
        Sources::new(range, log, generation.clone(), 1)?;
        let upper = if generation.is_interval() || generation.target_count().is_some() {
            1
        } else {
            measured_seed(projection, side, theme, legacy)?
        };

        let mut sources = Sources::new(range, log, generation, upper)?;
        sources.raw = raw;
        loop {
            let (source, positions) = sources.next()?;
            if let Some(positions) = positions {
                if positions.is_empty() {
                    return Ok(Self {
                        ticks: Vec::new(),
                        source,
                        numeric: None,
                    });
                }
                let operands =
                    if let Some(operands) = sources.operands.take() {
                        operands
                    } else {
                        crate::data::source::collect_source(positions.iter().map(|value| {
                            Ok(crate::numeric::operand::Operand::number(*value, log))
                        }))?
                    };
                let plan = if let Some(format) = format {
                    Self::formatted(
                        &positions, &operands, log, format, theme, source, projection, side,
                    )?
                } else {
                    let integral = log
                        && matches!(
                            source.kind,
                            SourceKind::LogIntegral
                                | SourceKind::LogSubset(_)
                                | SourceKind::LogInterval(_)
                        );
                    Self::omitted(&positions, &operands, integral, source)?
                };
                #[cfg(test)]
                crate::orchestrator::measurements::count(
                    crate::orchestrator::measurements::Counter::TickDistinctList,
                );

                if let Some(plan) = plan {
                    if labels_fit(&plan.ticks, projection, side, theme)? {
                        return Ok(plan);
                    }
                }
            }
            if source.kind == SourceKind::Midpoint || sources.fixed() {
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
                        if sources.fixed() {
                            "complete fixed interval source does not fit assigned span"
                        } else {
                            "numeric axis terminal midpoint label does not fit assigned span"
                        },
                    )
                    .into());
            }
        }
    }
}

pub(crate) fn format_label(
    value: f64,
    log: bool,
    integral: bool,
    precision: usize,
) -> Result<String, Error> {
    use crate::numeric::profiles::BuiltinProfile;
    let precision = if precision == 0 {
        None
    } else {
        Some(
            u8::try_from(precision)
                .map_err(|_| Error::Data("numeric source precision is not representable".into()))?,
        )
    };
    let profile = if integral {
        BuiltinProfile::ForestIntegral
    } else if log {
        BuiltinProfile::ForestExpression(precision)
    } else {
        BuiltinProfile::Automatic(precision)
    };
    profile.format(value)
}

/// Prepare a guide source before fonts through the common automatic profile.
pub(crate) fn format_guide_label(value: f64, precision: usize) -> Result<String, Error> {
    format_label(value, false, false, precision)
}

pub(super) fn labels_fit(
    ticks: &[AxisTick],
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<bool, Error> {
    let appearance = &theme.typography.axes.tick_label;
    let environment = theme.prepared_font_environment()?;
    for tick in ticks.iter().filter(|tick| !tick.label.is_empty()) {
        environment
            .admit_text(
                &appearance.stack,
                appearance.weight.into(),
                &tick.label,
                None,
            )
            .map_err(|error| {
                Error::Render(format!("numeric axis candidate text admission: {error}"))
            })?;
    }
    let cells = match super::cells::assign(
        ticks,
        false,
        true,
        None,
        projection,
        side,
        &crate::layout::fit::FitScope::root(),
    ) {
        Ok(cells) => cells,
        Err(crate::layout::fit::PlanError::Fit(_)) => return Ok(false),
        Err(crate::layout::fit::PlanError::Fatal(error)) => return Err(error),
    };
    for (tick, cell) in ticks.iter().zip(cells) {
        if tick.label.is_empty() {
            continue;
        }
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::TickMetricLookup,
        );
        let (width, alignment, role) = match side {
            AxisSide::Bottom => (
                crate::layout::maximum_width_quanta(cell.capacity())
                    .map_err(|error| Error::Render(error.to_string()))?,
                PhysicalTextAlignment::Center,
                "x-axis tick label",
            ),
            AxisSide::Left => (i64::MAX, PhysicalTextAlignment::End, "y-axis tick label"),
        };
        let plan = match theme.plan_text_quanta_scoped(
            &tick.label,
            &theme.typography.axes.tick_label,
            width,
            alignment,
            role,
            false,
            &crate::layout::fit::ROOT_SCOPE,
        ) {
            Ok(plan) => plan,
            Err(crate::layout::fit::PlanError::Fit(_)) => return Ok(false),
            Err(crate::layout::fit::PlanError::Fatal(error)) => return Err(error),
        };
        if cell.placement(&plan, side, 0.0)?.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests;
