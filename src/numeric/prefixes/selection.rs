//! Complete-set scale resolution. No selected label can change the frozen factor.

use std::cmp::Ordering;

use num_bigint::{BigInt, BigUint};

use super::{
    NumericFactor, NumericPrefixes, NumericPromotion, NumericSelection, PrefixMode, SiPrefix,
};
use crate::numeric::decimal::{decimal_quantum, ExactDecimal, NumericRounding, ResolvedFormat};
use crate::numeric::power::certified::PowerFormatter;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectedScale {
    pub(crate) divisor: NumericFactor,
    pub(crate) prefix: String,
    pub(crate) suffix: String,
    pub(crate) si: Option<SiPrefix>,
}

impl SelectedScale {
    pub(crate) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        self.divisor.observe(round)?;
        round.observe_string(&self.prefix)?;
        round.observe_string(&self.suffix)?;
        round.observe_count(self.si.map_or(0, |prefix| {
            usize::from(prefix.exponent().unsigned_abs()) * 2
                + usize::from(prefix.exponent() < 0)
                + 1
        }))
    }
    fn neutral() -> Self {
        Self {
            divisor: NumericFactor::Value(1.0),
            prefix: String::new(),
            suffix: String::new(),
            si: None,
        }
    }
    pub(crate) fn try_copy(&self) -> Result<Self> {
        Ok(Self {
            divisor: self.divisor.try_copy()?,
            prefix: crate::data::source::copy_source_string(&self.prefix)?,
            suffix: crate::data::source::copy_source_string(&self.suffix)?,
            si: self.si,
        })
    }
}

impl NumericPrefixes {
    pub(crate) fn resolve(
        &self,
        values: &[f64],
        power: bool,
        format: &ResolvedFormat<'_>,
        formatter: &mut PowerFormatter,
    ) -> Result<Vec<SelectedScale>> {
        match self.0.as_ref() {
            PrefixMode::None => crate::data::source::collect_source(
                values.iter().map(|_| Ok(SelectedScale::neutral())),
            ),
            PrefixMode::Factor(factor) => {
                let divisor = if matches!(factor, NumericFactor::AutomaticDecimal) {
                    let mut decade = None;
                    for value in values {
                        if let Some(next) = formatter.operand_decade(*value, power, format)? {
                            decade = Some(decade.map_or(next.clone(), |old: BigInt| old.max(next)));
                        }
                    }
                    NumericFactor::PowerOfTen(decade.unwrap_or_default().to_string())
                } else {
                    factor.try_copy()?
                };
                crate::data::source::collect_source(values.iter().map(|_| {
                    Ok(SelectedScale {
                        divisor: divisor.try_copy()?,
                        ..SelectedScale::neutral()
                    })
                }))
            }
            PrefixMode::Si(NumericSelection::Fixed(prefix), _) => {
                crate::data::source::collect_source(values.iter().map(|_| si_scale(*prefix)))
            }
            PrefixMode::Compact(_, NumericSelection::Fixed(index), _) => {
                crate::data::source::collect_source(values.iter().map(|_| self.scale(*index + 1)))
            }
            PrefixMode::Si(selection, promotion) => self.resolve_automatic(
                values,
                power,
                format,
                formatter,
                matches!(selection, NumericSelection::WholeSet),
                *promotion,
            ),
            PrefixMode::Compact(_, selection, promotion) => self.resolve_automatic(
                values,
                power,
                format,
                formatter,
                matches!(selection, NumericSelection::WholeSet),
                *promotion,
            ),
        }
    }

    fn resolve_automatic(
        &self,
        values: &[f64],
        power: bool,
        format: &ResolvedFormat<'_>,
        formatter: &mut PowerFormatter,
        whole_set: bool,
        promotion: NumericPromotion,
    ) -> Result<Vec<SelectedScale>> {
        let indices = crate::data::source::collect_source(
            values
                .iter()
                .map(|value| self.initial(*value, power, format, formatter)),
        )?;
        if whole_set {
            let mut index = indices.iter().copied().max().unwrap_or(0);
            if matches!(self.0.as_ref(), PrefixMode::Si(..)) {
                let mut nonzero = None;
                for (value, index) in values.iter().zip(&indices) {
                    if formatter.operand_decade(*value, power, format)?.is_some() {
                        nonzero = Some(nonzero.map_or(*index, |old: usize| old.max(*index)));
                    }
                }
                index = nonzero.unwrap_or(10);
            }
            if promotion == NumericPromotion::PromoteOnRounding {
                while let Some(threshold) = self.next_threshold(index)? {
                    let scale = self.scale(index)?;
                    let mut promote = false;
                    for value in values {
                        promote |= carries(*value, power, format, formatter, &scale, &threshold)?;
                    }
                    if !promote {
                        break;
                    }
                    index += 1;
                }
            }
            let scale = self.scale(index)?;
            crate::data::source::collect_source(values.iter().map(|_| scale.try_copy()))
        } else {
            crate::data::source::collect_source(values.iter().zip(indices).map(
                |(value, mut index)| {
                    if promotion == NumericPromotion::PromoteOnRounding {
                        while let Some(threshold) = self.next_threshold(index)? {
                            if !carries(
                                *value,
                                power,
                                format,
                                formatter,
                                &self.scale(index)?,
                                &threshold,
                            )? {
                                break;
                            }
                            index += 1;
                        }
                    }
                    self.scale(index)
                },
            ))
        }
    }

    fn initial(
        &self,
        value: f64,
        power: bool,
        format: &ResolvedFormat<'_>,
        formatter: &mut PowerFormatter,
    ) -> Result<usize> {
        match self.0.as_ref() {
            PrefixMode::Si(..) => {
                let Some(decade) = formatter.operand_decade(value, power, format)? else {
                    return Ok(10);
                };
                let decade = decade.clamp(BigInt::from(-30), BigInt::from(30));
                let decade = i8::try_from(decade).expect("clamped SI decade");
                Ok(usize::try_from(decade.div_euclid(3) + 10).expect("SI engineering index"))
            }
            PrefixMode::Compact(rules, ..) => {
                let mut index = 0;
                for rule in rules.iter() {
                    if rule.threshold() > 0.0
                        && formatter.compare_magnitude(
                            value,
                            power,
                            format,
                            &ExactDecimal::decode(rule.threshold(), 0)?,
                        )? == Ordering::Less
                    {
                        break;
                    }
                    index += 1;
                }
                Ok(index)
            }
            _ => Err(Error::Render(
                "automatic prefix selection requires a scale table".into(),
            )),
        }
    }

    fn scale(&self, index: usize) -> Result<SelectedScale> {
        if let PrefixMode::Compact(rules, ..) = self.0.as_ref() {
            if index == 0 {
                return Ok(SelectedScale::neutral());
            }
            let rule = &rules[index - 1];
            Ok(SelectedScale {
                divisor: NumericFactor::Value(rule.divisor()),
                prefix: crate::data::source::copy_source_string(rule.prefix())?,
                suffix: crate::data::source::copy_source_string(rule.suffix())?,
                si: None,
            })
        } else {
            si_scale(SiPrefix::from_exponent(
                i8::try_from(index).expect("SI index") * 3 - 30,
            )?)
        }
    }

    fn next_threshold(&self, index: usize) -> Result<Option<ExactDecimal>> {
        if let PrefixMode::Compact(rules, ..) = self.0.as_ref() {
            rules
                .get(index)
                .map(|rule| ExactDecimal::decode(rule.threshold(), 0))
                .transpose()
        } else if index < 20 {
            Ok(Some(ExactDecimal::new(
                false,
                BigUint::from(1_u8),
                (BigInt::from(index) * 3_u8) - 27_u8,
            )?))
        } else {
            Ok(None)
        }
    }
}

fn si_scale(prefix: SiPrefix) -> Result<SelectedScale> {
    Ok(SelectedScale {
        divisor: NumericFactor::PowerOfTen(prefix.exponent().to_string()),
        prefix: String::new(),
        suffix: crate::data::source::copy_source_string(prefix.symbol())?,
        si: Some(prefix),
    })
}

fn carries(
    value: f64,
    power: bool,
    format: &ResolvedFormat<'_>,
    formatter: &mut PowerFormatter,
    scale: &SelectedScale,
    threshold: &ExactDecimal,
) -> Result<bool> {
    let mut resolved = *format;
    resolved.divisor = Some(&scale.divisor);
    let Some(decade) = formatter.operand_decade(value, power, &resolved)? else {
        return Ok(false);
    };
    let quantum = decimal_quantum(&decade, &resolved);
    let target = ExactDecimal::ratio(
        false,
        threshold.coefficient.clone(),
        threshold.denominator.clone(),
        threshold.exponent.clone(),
    )?
    .divide(Some(&scale.divisor))?;
    if target.decade > (&decade + 1_u8).max(quantum.clone()) {
        return Ok(false);
    }
    let coefficient = target.ceiling_at(&quantum)?;
    let negative = (!power && value.is_sign_negative()) ^ format.multiplier.is_sign_negative();
    let (numerator, denominator, inclusive) = match format.rounding {
        NumericRounding::NearestEven
        | NumericRounding::NearestAway
        | NumericRounding::NearestTowardZero => {
            let inclusive = match format.rounding {
                NumericRounding::NearestEven => !coefficient.bit(0),
                NumericRounding::NearestAway => true,
                _ => false,
            };
            (coefficient * 2_u8 - 1_u8, 2_u8, inclusive)
        }
        NumericRounding::AwayFromZero | NumericRounding::Floor
            if format.rounding == NumericRounding::AwayFromZero || negative =>
        {
            (coefficient - 1_u8, 1_u8, false)
        }
        NumericRounding::Ceiling if !negative => (coefficient - 1_u8, 1_u8, false),
        _ => (coefficient, 1_u8, true),
    };
    let boundary = ExactDecimal::ratio(false, numerator, BigUint::from(denominator), quantum)?;
    let comparison = formatter.compare_magnitude(value, power, &resolved, &boundary)?;
    Ok(comparison == Ordering::Greater || (inclusive && comparison == Ordering::Equal))
}
