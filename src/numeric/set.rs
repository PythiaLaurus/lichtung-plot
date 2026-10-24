//! Complete semantic sets select precision before geometric fitting. Label-index
//! heapsort checks the whole grammar without assuming monotonic text ordering.

use super::format::{NumericFormat, NumericPrecision};
use super::power::{certified::PowerFormatter, Log10Operand};
use super::prefixes::selection::SelectedScale;
use crate::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FormattedSet {
    pub(crate) members: Vec<u64>,
    pub(crate) labels: Vec<String>,
    pub(crate) precision: u8,
    pub(crate) scales: Vec<SelectedScale>,
    pub(crate) unit: Option<super::unit::NumericUnit>,
    pub(crate) explanation: Option<crate::Text>,
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
        round.observe_count(usize::from(self.set.precision))?;
        round.observe_count(self.set.members.len())?;
        for &value in &self.set.members {
            // Spatial feedback normalizes zero; scientific occurrences retain its sign.
            round.observe_count(usize::from(value >> 63 != 0))?;
            round.observe_number(f64::from_bits(value))?;
        }
        for label in &self.set.labels {
            round.observe_string(label)?;
        }
        self.set.observe_factors(round)?;
        Ok(())
    }
}

impl FormattedSet {
    pub(crate) fn observe_factors(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
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
        Ok(())
    }
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            members: crate::data::source::collect_source(self.members.iter().copied().map(Ok))?,
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
        })
    }
}

/// One whole-set pre-font label policy for color and alpha explanations.
/// Repeated scientific values may share text; distinct finite values may not.
pub(crate) fn guide_presentation(
    values: &[f64],
    format: Option<&NumericFormat>,
) -> Result<FormattedSet> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Data("numeric guide values must be finite".into()));
    }
    if let Some(format) = format {
        return format.format_set(values, false, true)?.ok_or_else(|| {
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
                    values.iter().map(|value| Ok(value.to_bits())),
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
    pub(crate) fn format_singleton(&self, value: f64, power: bool) -> Result<String> {
        self.format_selected(
            value,
            self.get_precision().bounds()?.0,
            power,
            &mut PowerFormatter::default(),
        )
    }

    pub(crate) fn format_selected(
        &self,
        value: f64,
        digits: u8,
        power: bool,
        formatter: &mut PowerFormatter,
    ) -> Result<String> {
        let resolved = self.resolve_at(digits)?;
        let scales = self
            .prefix_policy()
            .resolve(&[value], power, &resolved, formatter)?;
        let (scales, _) = self.decorate_scales(&scales)?;
        self.format_scaled(value, digits, power, formatter, &scales[0])
    }

    fn format_scaled(
        &self,
        value: f64,
        digits: u8,
        power: bool,
        formatter: &mut PowerFormatter,
        scale: &SelectedScale,
    ) -> Result<String> {
        let mut resolved = self.resolve_at(digits)?;
        resolved.prefix = None;
        resolved.suffix = None;
        resolved.divisor = Some(&scale.divisor);
        let ascii = if power {
            let operand = Log10Operand::new(value, self.get_decimal_scale())?;
            formatter.format(&operand, &resolved)?.text
        } else {
            super::decimal::format_finite(value, &resolved)?
        };
        self.present(&ascii, scale)
    }

    pub(crate) fn format_set(
        &self,
        values: &[f64],
        power: bool,
        require_distinct: bool,
    ) -> Result<Option<FormattedSet>> {
        if values.iter().any(|value| !value.is_finite()) {
            return Err(Error::Data(
                "numeric presentation members must be finite".into(),
            ));
        }
        let (min, max) = self.get_precision().bounds()?;
        let adaptive = matches!(
            self.get_precision(),
            NumericPrecision::AdaptiveFractional { .. }
                | NumericPrecision::AdaptiveSignificant { .. }
        );
        let mut formatter = PowerFormatter::default();
        let mut order = crate::data::source::collect_source((0..values.len()).map(Ok))?;
        crate::data::source::sort_source_by(&mut order, |a, b| values[*a].total_cmp(&values[*b]));
        let mut occurrences =
            crate::data::source::collect_source(values.iter().map(|_| Ok(0_usize)))?;
        let mut distinct: Vec<f64> = Vec::new();
        let distinct_count = order
            .windows(2)
            .filter(|pair| values[pair[0]].to_bits() != values[pair[1]].to_bits())
            .count()
            + usize::from(!values.is_empty());
        crate::data::source::reserve_source(&mut distinct, distinct_count)?;
        for &index in &order {
            if distinct
                .last()
                .is_none_or(|value| value.to_bits() != values[index].to_bits())
            {
                distinct.push(values[index]);
            }
            occurrences[index] = distinct.len() - 1;
        }
        order.truncate(distinct.len());
        for (index, slot) in order.iter_mut().enumerate() {
            *slot = index;
        }
        for digits in min..=max {
            let scales = self.prefix_policy().resolve(
                &distinct,
                power,
                &self.resolve_at(digits)?,
                &mut formatter,
            )?;
            let (emission_scales, explanation) = self.decorate_scales(&scales)?;
            let mut unique_labels = crate::data::source::collect_source(
                distinct.iter().zip(&emission_scales).map(|(value, scale)| {
                    self.format_scaled(*value, digits, power, &mut formatter, scale)
                }),
            )?;
            if adaptive || require_distinct {
                crate::data::source::sort_source_by(&mut order, |a, b| {
                    unique_labels[*a].cmp(&unique_labels[*b])
                });
                if order.windows(2).any(|pair| {
                    let [a, b] = [pair[0], pair[1]];
                    unique_labels[a] == unique_labels[b]
                        && !scientifically_equal(distinct[a], distinct[b])
                }) {
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
                members: crate::data::source::collect_source(
                    values.iter().map(|v| Ok(v.to_bits())),
                )?,
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
            }));
        }
        Ok(None)
    }
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
        assert_eq!(set.members, [second, first, second].map(f64::to_bits));
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
        assert_eq!(selected.members, values.map(f64::to_bits));
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
