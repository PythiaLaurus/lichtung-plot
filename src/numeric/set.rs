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
        for digits in min..=max {
            let labels = crate::data::source::collect_source(
                values
                    .iter()
                    .map(|value| self.format_selected(*value, digits, power, &mut formatter)),
            )?;
            if adaptive || require_distinct {
                crate::data::source::sort_source_by(&mut order, |a, b| labels[*a].cmp(&labels[*b]));
                if order.windows(2).any(|pair| {
                    let [a, b] = [pair[0], pair[1]];
                    labels[a] == labels[b] && !scientifically_equal(values[a], values[b])
                }) {
                    continue;
                }
            }
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
