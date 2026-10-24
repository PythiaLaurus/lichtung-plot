//! Permitted CSS font-size intervals and actual-role grouping.

use super::layout::Finite;
use crate::{Error, Result};

/// Whether eligible members select separate sizes or one common size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFitGrouping {
    /// Fit each member within its own permitted interval.
    #[default]
    Independent,
    /// Fit the actual owner/role group at one size in its common interval.
    Uniform,
}

/// Order complete-content fitting and explicitly permitted abbreviation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFitPriority {
    /// Try complete content throughout the permitted interval first.
    #[default]
    ContentFirst,
    /// Try abbreviation at the preferred permitted size before shrinking.
    PreferredSizeFirst,
}

/// A font-size program, independent of terminal overflow handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextFit {
    interval: Interval,
    pub(crate) grouping: TextFitGrouping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Interval {
    Preferred,
    At(Finite),
    Shrink(Finite),
    Range(Finite, Finite),
}

impl TextFit {
    /// Keep the complete appearance's preferred size.
    pub const fn fixed() -> Self {
        Self {
            interval: Interval::Preferred,
            grouping: TextFitGrouping::Independent,
        }
    }
    /// Permit shrinking to a positive finite floor.
    pub fn shrink_to(minimum: f64) -> Result<Self> {
        Ok(Self {
            interval: Interval::Shrink(positive(minimum)?),
            ..Self::fixed()
        })
    }
    /// Permit growth and shrink within one closed positive finite interval.
    pub fn range(minimum: f64, maximum: f64) -> Result<Self> {
        let lower = positive(minimum)?;
        let upper = positive(maximum)?;
        if minimum > maximum {
            return Err(Error::Data("text fit minimum exceeds maximum".into()));
        }
        Ok(Self {
            interval: if lower == upper {
                Interval::At(lower)
            } else {
                Interval::Range(lower, upper)
            },
            ..Self::fixed()
        })
    }
    /// Replace the size grouping, without changing the permitted interval.
    #[must_use]
    pub const fn grouping(mut self, value: TextFitGrouping) -> Self {
        self.grouping = value;
        self
    }

    pub(crate) fn bounds(self, preferred: f64) -> Result<(f64, f64)> {
        let (lower, upper) = match self.interval {
            Interval::Preferred => (preferred, preferred),
            Interval::At(value) => (value.get(), value.get()),
            Interval::Shrink(lower) => (lower.get(), preferred),
            Interval::Range(lower, upper) => (lower.get(), upper.get()),
        };
        if lower > upper {
            return Err(Error::Data(
                "text fit floor exceeds the preferred size".into(),
            ));
        }
        Ok((lower, upper))
    }
}

fn positive(value: f64) -> Result<Finite> {
    if value <= 0.0 {
        return Err(Error::Data(
            "text fit size must be positive and finite".into(),
        ));
    }
    Finite::new(value)
}
