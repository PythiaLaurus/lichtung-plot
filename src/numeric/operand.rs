//! One retained representation for ordinary and mathematical numeric operands.

use super::{
    decimal::ResolvedFormat,
    lattice::LatticeOperand,
    power::{certified::PowerFormatter, Log10Operand},
};
use crate::{Error, Result};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Operand {
    Binary64(u64),
    PowerOfTen(u64),
    Multiplicative(Arc<LatticeOperand>),
}

impl Operand {
    pub(crate) fn binary64(&self) -> Result<f64> {
        if let Self::Binary64(bits) = self {
            Ok(f64::from_bits(*bits))
        } else {
            Err(Error::Render(
                "ordinary numeric consumer received a mathematical operand".into(),
            ))
        }
    }
    pub(crate) fn number(value: f64, power: bool) -> Self {
        if power {
            Self::PowerOfTen(value.to_bits())
        } else {
            Self::Binary64(value.to_bits())
        }
    }
    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Self::Binary64(bits) | Self::PowerOfTen(bits) if !f64::from_bits(*bits).is_finite() => {
                Err(Error::Data(
                    "numeric presentation members must be finite operands".into(),
                ))
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn negative(&self) -> bool {
        matches!(self,Self::Binary64(bits) if bits>>63!=0)
    }
    pub(crate) fn scientifically_equal(&self, other: &Self) -> bool {
        self == other
            || match (self, other) {
                (Self::Binary64(a), Self::Binary64(b))
                | (Self::PowerOfTen(a), Self::PowerOfTen(b)) => {
                    a & !(1_u64 << 63) == 0 && b & !(1_u64 << 63) == 0
                }
                _ => false,
            }
    }
    pub(crate) fn format(
        &self,
        format: &ResolvedFormat<'_>,
        formatter: &mut PowerFormatter,
    ) -> Result<String> {
        match self {
            Self::Binary64(bits) => super::decimal::format_finite(f64::from_bits(*bits), format),
            Self::PowerOfTen(bits) => Ok(formatter
                .format(
                    &Log10Operand::new(f64::from_bits(*bits), format.decimal_scale)?,
                    format,
                )?
                .text),
            Self::Multiplicative(value) => formatter.format_lattice(value, format),
        }
    }
    pub(crate) fn observe(&self, round: &mut crate::layout::kernel::FeedbackRound) -> Result<()> {
        match self {
            Self::Binary64(bits) | Self::PowerOfTen(bits) => {
                round.observe_count(usize::from(matches!(self, Self::PowerOfTen(_))))?;
                round.observe_count(usize::from(bits >> 63 != 0))?;
                round.observe_number(f64::from_bits(*bits))
            }
            Self::Multiplicative(value) => {
                round.observe_count(2)?;
                round.observe_number(f64::from_bits(value.anchor_bits))?;
                round.observe_number(f64::from_bits(value.ratio_bits))?;
                round.observe_string(&value.index_text)
            }
        }
    }
}
