use crate::{Error, Result};

/// Validated first-tried spacing for generated numeric ticks, not a tick count.
///
/// If the requested complete source cannot fit, the renderer tries the bounded
/// coarser sources and final midpoint. Exact caller ticks take precedence.
/// ```
/// use lichtung_plot::TickInterval;
/// let linear = TickInterval::additive(0.25)?;
/// let logarithmic = TickInterval::log10_decades(2)?;
/// assert_eq!(linear.additive_step(), Some(0.25));
/// assert_eq!(logarithmic.decade_step(), Some(2));
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// The choice and payload are private:
/// ```compile_fail
/// let _ = lichtung_plot::TickInterval::Additive(0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickInterval {
    spacing: Spacing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Spacing {
    Additive(f64),
    Log10Decades(u32),
}

impl TickInterval {
    /// Prefer a finite strictly positive additive step on a linear numeric axis.
    pub fn additive(step: f64) -> Result<Self> {
        if !step.is_finite() || step <= 0.0 {
            return Err(Error::Data(
                "tick interval must be finite and strictly positive".into(),
            ));
        }
        Ok(Self {
            spacing: Spacing::Additive(step),
        })
    }

    /// Prefer a nonzero integral exponent step on a base-ten logarithmic axis.
    pub fn log10_decades(decades: u32) -> Result<Self> {
        if decades == 0 {
            return Err(Error::Data(
                "tick interval decades must be strictly positive".into(),
            ));
        }
        Ok(Self {
            spacing: Spacing::Log10Decades(decades),
        })
    }

    /// Inspect the positive additive step, or None for logarithmic decades.
    pub const fn additive_step(&self) -> Option<f64> {
        match self.spacing {
            Spacing::Additive(step) => Some(step),
            Spacing::Log10Decades(_) => None,
        }
    }

    /// Inspect the positive integral decade step, or None for additive spacing.
    pub const fn decade_step(&self) -> Option<u32> {
        match self.spacing {
            Spacing::Additive(_) => None,
            Spacing::Log10Decades(step) => Some(step),
        }
    }
}
