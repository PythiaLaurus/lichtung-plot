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
    Target(u32),
    Maximum(u32),
}

impl TickInterval {
    pub(crate) const MAX_COUNT: u32 = 10_000;

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

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private target sources precede the generation API cutover"
        )
    )]
    pub(crate) fn target(count: u32) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            spacing: Spacing::Target(count),
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private maximum sources precede the generation API cutover"
        )
    )]
    pub(crate) fn maximum(count: u32) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            spacing: Spacing::Maximum(count),
        })
    }

    fn validate_count(count: u32) -> Result<()> {
        if !(1..=Self::MAX_COUNT).contains(&count) {
            return Err(Error::Data(
                "generated tick count must be in 1..=10000".into(),
            ));
        }
        Ok(())
    }

    pub(crate) const fn target_count(&self) -> Option<u32> {
        match self.spacing {
            Spacing::Target(count) => Some(count),
            _ => None,
        }
    }

    pub(crate) const fn maximum_count(&self) -> Option<u32> {
        match self.spacing {
            Spacing::Maximum(count) => Some(count),
            _ => None,
        }
    }

    pub(crate) const fn is_interval(&self) -> bool {
        matches!(
            self.spacing,
            Spacing::Additive(_) | Spacing::Log10Decades(_)
        )
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        match self.spacing {
            Spacing::Additive(step) => {
                round.observe_count(1)?;
                round.observe_number(step)
            }
            Spacing::Log10Decades(step) => {
                round.observe_count(2)?;
                round.observe_count(step as usize)
            }
            Spacing::Target(count) => {
                round.observe_count(3)?;
                round.observe_count(count as usize)
            }
            Spacing::Maximum(count) => {
                round.observe_count(4)?;
                round.observe_count(count as usize)
            }
        }
    }

    /// Inspect the positive additive step, or None for logarithmic decades.
    pub const fn additive_step(&self) -> Option<f64> {
        match self.spacing {
            Spacing::Additive(step) => Some(step),
            _ => None,
        }
    }

    /// Inspect the positive integral decade step, or None for additive spacing.
    pub const fn decade_step(&self) -> Option<u32> {
        match self.spacing {
            Spacing::Log10Decades(step) => Some(step),
            _ => None,
        }
    }
}
