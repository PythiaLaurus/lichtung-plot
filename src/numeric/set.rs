//! Complete semantic sets select precision before geometric fitting. Label-index
//! heapsort checks the whole grammar without assuming monotonic text ordering.

use super::format::{NumericFormat, NumericPrecision};
use super::power::{certified::PowerFormatter, Log10Operand};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormattedSet {
    pub(crate) members: Vec<u64>,
    pub(crate) labels: Vec<String>,
    pub(crate) precision: u8,
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
            round.observe_number(f64::from_bits(value))?;
        }
        for label in &self.set.labels {
            round.observe_string(label)?;
        }
        Ok(())
    }
}

impl FormattedSet {
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            members: crate::data::source::collect_source(self.members.iter().copied().map(Ok))?,
            labels: crate::data::source::collect_source(
                self.labels
                    .iter()
                    .map(|text| crate::data::source::copy_source_string(text)),
            )?,
            precision: self.precision,
        })
    }
}

impl NumericFormat {
    pub(crate) fn format_singleton(&self, value: f64, power: bool) -> Result<String> {
        self.format_selected(
            value,
            self.precision().bounds()?.0,
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
        if power {
            let operand = Log10Operand::new(value, self.decimal_scale())?;
            Ok(formatter.format(&operand, &resolved)?.text)
        } else {
            super::decimal::format_finite(value, &resolved)
        }
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
        let (min, max) = self.precision().bounds()?;
        let adaptive = matches!(
            self.precision(),
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
            let mut unique_labels = crate::data::source::collect_source(
                distinct
                    .iter()
                    .map(|value| self.format_selected(*value, digits, power, &mut formatter)),
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
            return Ok(Some(FormattedSet {
                members: crate::data::source::collect_source(
                    values.iter().map(|v| Ok(v.to_bits())),
                )?,
                labels,
                precision: digits,
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
            .with_precision(NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
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
