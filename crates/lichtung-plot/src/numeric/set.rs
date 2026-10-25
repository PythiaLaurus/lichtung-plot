//! Complete semantic sets select precision before geometric fitting. Label-index
//! heapsort checks the whole grammar without assuming monotonic text ordering.

use super::format::{NumericFormat, NumericPrecision};
use super::operand::Operand;
use super::power::certified::PowerFormatter;
use super::prefixes::selection::SelectedScale;
use crate::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FormattedSet {
    pub(crate) members: Vec<Operand>,
    pub(crate) labels: Vec<String>,
    pub(crate) precision: u8,
    pub(crate) scales: Vec<SelectedScale>,
    pub(crate) unit: Option<super::unit::NumericUnit>,
    pub(crate) explanation: Option<crate::Text>,
    pub(crate) context: Option<std::sync::Arc<super::context::ContextInfo>>,
}

/// Immutable policy, scientific occurrences and selected text for a value owner.
/// None denotes the owning consumer's prescribed omitted-policy profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NumericPresentation {
    pub(crate) format: Option<std::sync::Arc<NumericFormat>>,
    pub(crate) set: FormattedSet,
}

impl NumericPresentation {
    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        round.observe_count(usize::from(self.format.is_some()))?;
        if let Some(format) = &self.format {
            format.observe_feedback(round)?;
        }
        self.set.observe_feedback(round)
    }
}

impl FormattedSet {
    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        round.observe_count(usize::from(self.precision))?;
        round.observe_count(self.members.len())?;
        for member in &self.members {
            member.observe(round)?;
        }
        round.observe_count(self.labels.len())?;
        for label in &self.labels {
            round.observe_string(label)?;
        }
        round.observe_count(self.scales.len())?;
        for scale in &self.scales {
            scale.observe(round)?;
        }
        round.observe_count(usize::from(self.unit.is_some()))?;
        if let Some(unit) = &self.unit {
            round.observe_string(unit.text())?;
            round.observe_string(unit.get_separator())?;
            round.observe_count(unit.placement() as usize)?;
        }
        round.observe_count(usize::from(self.explanation.is_some()))?;
        if let Some(text) = &self.explanation {
            round.observe_authored_text(text)?;
        }
        round.observe_count(usize::from(self.context.is_some()))?;
        if let Some(context) = &self.context {
            round.observe_numeric_context(context)?;
        }
        Ok(())
    }
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            members: crate::data::source::collect_source(self.members.iter().cloned().map(Ok))?,
            labels: crate::data::source::collect_source(
                self.labels
                    .iter()
                    .map(|text| crate::data::source::copy_source_string(text)),
            )?,
            precision: self.precision,
            scales: crate::data::source::collect_source(
                self.scales.iter().map(SelectedScale::try_copy),
            )?,
            unit: self
                .unit
                .as_ref()
                .map(super::unit::NumericUnit::try_copy)
                .transpose()?,
            explanation: self
                .explanation
                .as_ref()
                .map(crate::Text::try_copy)
                .transpose()?,
            context: self.context.clone(),
        })
    }
}

/// One whole-set pre-font label policy for color and alpha explanations.
/// Repeated scientific values may share text; distinct finite values may not.
pub(crate) fn guide_presentation(
    values: &[f64],
    format: Option<(
        &NumericFormat,
        &super::source::NumericSource,
        crate::NumericRangeOrigin,
    )>,
) -> Result<FormattedSet> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Data("numeric guide values must be finite".into()));
    }
    if let Some((format, source, origin)) = format {
        let domain =
            values
                .first()
                .zip(values.last())
                .map(|(lower, upper)| super::context::DomainInfo {
                    limits: (lower.to_bits(), upper.to_bits()),
                    data: Some((lower.to_bits(), upper.to_bits())),
                    scale: crate::AxisScale::Linear,
                    origin,
                });
        let info = source.indexed(
            &crate::data::source::collect_source(
                values
                    .iter()
                    .map(|value| Ok(Operand::number(*value, false))),
            )?,
            crate::NumericRole::ContinuousColor,
            crate::NumericPurpose::ActualSet,
            domain,
        )?;
        return format
            .materialize(&source.operation, info, true)?
            .ok_or_else(|| {
                Error::Data("numeric guide labels are not unique at configured precision".into())
            });
    }
    let mut order = crate::data::source::collect_source((0..values.len()).map(Ok))?;
    for precision in std::iter::once(0).chain(2..=17) {
        let profile =
            super::profiles::BuiltinProfile::Automatic((precision != 0).then_some(precision));
        let labels =
            crate::data::source::collect_source(values.iter().map(|value| profile.format(*value)))?;
        crate::data::source::sort_source_by(&mut order, |a, b| labels[*a].cmp(&labels[*b]));
        if order.windows(2).all(|pair| {
            labels[pair[0]] != labels[pair[1]]
                || scientifically_equal(values[pair[0]], values[pair[1]])
        }) {
            return Ok(FormattedSet {
                members: crate::data::source::collect_source(
                    values
                        .iter()
                        .map(|value| Ok(Operand::number(*value, false))),
                )?,
                labels,
                precision,
                ..FormattedSet::default()
            });
        }
    }
    Err(Error::Data(
        "numeric guide labels are not unique at supported numeric precisions".into(),
    ))
}

impl NumericFormat {
    #[cfg(test)]
    pub(crate) fn format_singleton(&self, value: f64, power: bool) -> Result<String> {
        let mut set = self.function_builtin(&[Operand::number(value, power)])?;
        Ok(set.labels.remove(0))
    }

    fn format_scaled(
        &self,
        value: &Operand,
        digits: u8,
        formatter: &mut PowerFormatter,
        scale: &SelectedScale,
    ) -> Result<String> {
        let mut resolved = self.resolve_at(digits)?;
        resolved.prefix = None;
        resolved.suffix = None;
        resolved.divisor = Some(&scale.divisor);
        let ascii = value.format(&resolved, formatter)?;
        self.present(&ascii, scale)
    }

    #[cfg(test)]
    pub(crate) fn format_set(
        &self,
        values: &[f64],
        power: bool,
        require_distinct: bool,
    ) -> Result<Option<FormattedSet>> {
        let values = crate::data::source::collect_source(
            values
                .iter()
                .map(|value| Ok(Operand::number(*value, power))),
        )?;
        self.resolve_builtin(&values, require_distinct, false, None)
    }

    pub(super) fn function_builtin(&self, values: &[Operand]) -> Result<FormattedSet> {
        self.resolve_builtin(values, false, true, None)?
            .ok_or_else(|| Error::Render("complete built-in helper result was not retained".into()))
    }

    pub(super) fn function_metadata(&self, values: &[Operand]) -> Result<FormattedSet> {
        let precision = self.get_precision().bounds()?.0;
        if values.is_empty() {
            return Ok(FormattedSet {
                precision,
                ..FormattedSet::default()
            });
        }
        let scales = self.prefix_policy().resolve(
            values,
            &self.resolve_at(precision)?,
            &mut PowerFormatter::default(),
        )?;
        let explanation = self.numeric_explanation(&scales)?;
        Ok(FormattedSet {
            members: crate::data::source::collect_source(values.iter().cloned().map(Ok))?,
            precision,
            scales,
            unit: self
                .units
                .unit
                .as_ref()
                .map(super::unit::NumericUnit::try_copy)
                .transpose()?,
            explanation,
            ..FormattedSet::default()
        })
    }

    pub(super) fn function_helper(
        &self,
        values: &[Operand],
        scales: &[SelectedScale],
        required: bool,
        probe: bool,
    ) -> Result<FormattedSet> {
        self.resolve_builtin(values, required && !probe, probe, Some(scales))?
            .ok_or_else(|| {
                Error::Data(
                    "built-in numeric precision does not distinguish this complete set".into(),
                )
            })
    }

    fn resolve_builtin(
        &self,
        values: &[Operand],
        require_distinct: bool,
        retain_collision: bool,
        frozen: Option<&[SelectedScale]>,
    ) -> Result<Option<FormattedSet>> {
        for value in values {
            value.validate()?;
        }
        let (min, max) = self.get_precision().bounds()?;
        if values.is_empty() {
            return Ok(Some(FormattedSet {
                precision: min,
                ..FormattedSet::default()
            }));
        }
        let adaptive = matches!(
            self.get_precision(),
            NumericPrecision::AdaptiveFractional { .. }
                | NumericPrecision::AdaptiveSignificant { .. }
        );
        let mut formatter = PowerFormatter::default();
        let mut order = crate::data::source::collect_source((0..values.len()).map(Ok))?;
        let MemberOrder {
            distinct,
            occurrences,
            representatives,
        } = distinct_members(values, &mut order)?;
        for digits in min..=max {
            let scales = if let Some(frozen) = frozen {
                crate::data::source::collect_source(
                    representatives
                        .iter()
                        .map(|&index| frozen[index].try_copy()),
                )?
            } else {
                self.prefix_policy().resolve(
                    &distinct,
                    &self.resolve_at(digits)?,
                    &mut formatter,
                )?
            };
            let (emission_scales, explanation) = self.decorate_scales(&scales)?;
            let mut unique_labels =
                crate::data::source::collect_source(distinct.iter().zip(&emission_scales).map(
                    |(value, scale)| self.format_scaled(value, digits, &mut formatter, scale),
                ))?;
            if adaptive || require_distinct {
                crate::data::source::sort_source_by(&mut order, |a, b| {
                    unique_labels[*a].cmp(&unique_labels[*b])
                });
                if order.windows(2).any(|pair| {
                    let [a, b] = [pair[0], pair[1]];
                    unique_labels[a] == unique_labels[b]
                        && !distinct[a].scientifically_equal(&distinct[b])
                }) && (!retain_collision || digits < max)
                {
                    continue;
                }
            }
            order.fill(0);
            for &index in &occurrences {
                order[index] += 1;
            }
            let labels = crate::data::source::collect_source(occurrences.iter().map(|&index| {
                order[index] -= 1;
                if order[index] == 0 {
                    Ok(std::mem::take(&mut unique_labels[index]))
                } else {
                    crate::data::source::copy_source_string(&unique_labels[index])
                }
            }))?;
            let scales = crate::data::source::collect_source(
                occurrences.iter().map(|&index| scales[index].try_copy()),
            )?;
            return Ok(Some(FormattedSet {
                members: crate::data::source::collect_source(values.iter().cloned().map(Ok))?,
                labels,
                precision: digits,
                scales,
                unit: self
                    .units
                    .unit
                    .as_ref()
                    .map(super::unit::NumericUnit::try_copy)
                    .transpose()?,
                explanation,
                context: None,
            }));
        }
        Ok(None)
    }
}

struct MemberOrder {
    distinct: Vec<Operand>,
    occurrences: Vec<usize>,
    representatives: Vec<usize>,
}

fn distinct_members(values: &[Operand], order: &mut Vec<usize>) -> Result<MemberOrder> {
    crate::data::source::sort_source_by(order, |a, b| values[*a].cmp(&values[*b]));
    let mut occurrences = crate::data::source::collect_source(values.iter().map(|_| Ok(0_usize)))?;
    let mut distinct: Vec<Operand> = Vec::new();
    let count = order
        .windows(2)
        .filter(|pair| values[pair[0]] != values[pair[1]])
        .count()
        + usize::from(!values.is_empty());
    crate::data::source::reserve_source(&mut distinct, count)?;
    let mut representatives = Vec::new();
    crate::data::source::reserve_source(&mut representatives, count)?;
    for &index in order.iter() {
        if distinct.last().is_none_or(|value| *value != values[index]) {
            distinct.push(values[index].clone());
            representatives.push(index);
        }
        occurrences[index] = distinct.len() - 1;
    }
    order.truncate(distinct.len());
    for (index, slot) in order.iter_mut().enumerate() {
        *slot = index;
    }
    Ok(MemberOrder {
        distinct,
        occurrences,
        representatives,
    })
}

fn scientifically_equal(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.abs().to_bits() == 0 && b.abs().to_bits() == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::decimal::NumericNotation;

    #[test]
    fn extended_fractional_precision_distinguishes_small_exact_dyadic_sets() {
        let format = NumericFormat::new(
            NumericNotation::Decimal,
            NumericPrecision::AdaptiveFractional { min: 0, max: 20 },
        )
        .unwrap();
        let first = f64::from_bits((1023_u64 - 60) << 52);
        let second = f64::from_bits((1023_u64 - 59) << 52);
        let set = format
            .format_set(&[second, first, second], false, true)
            .unwrap()
            .unwrap();
        assert_eq!(set.precision, 18);
        assert_eq!(
            set.labels,
            [
                "0.000000000000000002",
                "0.000000000000000001",
                "0.000000000000000002"
            ]
        );
        assert_eq!(
            set.members,
            [second, first, second].map(|value| Operand::number(value, false))
        );
    }

    #[test]
    fn semantic_sets_resolve_uniform_precision_preserve_occurrences_and_exhaust_powers() {
        let exact =
            NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0)).unwrap();
        let values = [1.01, 1.02, 1.01];
        assert!(exact.format_set(&values, false, true).unwrap().is_none());
        assert_eq!(
            exact
                .format_set(&values, false, false)
                .unwrap()
                .unwrap()
                .labels,
            ["1", "1", "1"]
        );
        let adaptive = exact
            .precision(NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
            .unwrap();
        let selected = adaptive.format_set(&values, false, true).unwrap().unwrap();
        assert_eq!(selected.precision, 2);
        assert_eq!(selected.labels, ["1.01", "1.02", "1.01"]);
        assert_eq!(
            selected.members,
            values.map(|value| Operand::number(value, false))
        );
        let zeros = adaptive
            .format_set(&[0.0, -0.0], false, true)
            .unwrap()
            .unwrap();
        assert_eq!(zeros.precision, 0);
        assert_eq!(zeros.labels, ["0", "-0"]);
        assert_eq!(
            adaptive
                .format_set(&[], false, true)
                .unwrap()
                .unwrap()
                .precision,
            0
        );
        let powers = NumericFormat::new(
            NumericNotation::Scientific,
            NumericPrecision::AdaptiveSignificant { min: 1, max: 17 },
        )
        .unwrap();
        assert!(powers
            .format_set(&[f64::from_bits(1), f64::from_bits(2)], true, true)
            .unwrap()
            .is_none());
        assert_eq!(
            NumericFormat::default()
                .format_set(&[0.5], true, true)
                .unwrap()
                .unwrap()
                .labels,
            ["3.16"]
        );
    }
}
