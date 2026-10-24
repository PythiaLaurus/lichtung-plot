use crate::{Error, Result};

/// One validated selection for generated numeric tick positions.
///
/// Targets allow nice-step deviation; maxima bound the complete in-domain list,
/// including endpoint positions. Preferred intervals are tried before bounded
/// coarser sources. Exact caller ticks take precedence over every selection.
/// ```
/// use lichtung_plot::TickGeneration;
/// let linear = TickGeneration::additive(0.25)?;
/// let logarithmic = TickGeneration::log10_decades(2)?;
/// assert_eq!(linear.additive_step(), Some(0.25));
/// assert_eq!(logarithmic.decade_step(), Some(2));
/// assert_eq!(TickGeneration::target(5)?.target_count(), Some(5));
/// assert_eq!(TickGeneration::maximum(6)?.maximum_count(), Some(6));
/// assert!(TickGeneration::default().is_automatic());
/// assert!(TickGeneration::target(0).is_err());
/// assert!(TickGeneration::maximum(10_001).is_err());
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// The choice and payload are private:
/// ```compile_fail
/// let _ = lichtung_plot::TickGeneration::Target(0);
/// ```
/// Requests are owned and cloneable, not Copy:
/// ```compile_fail
/// use lichtung_plot::TickGeneration;
/// let request = TickGeneration::automatic();
/// let moved = request;
/// let _ = (moved, request);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TickGeneration {
    primary: Primary,
    maximum: Option<u32>,
}

// All admitted floating steps are finite and strictly positive.
impl Eq for TickGeneration {}

#[derive(Debug, Clone, Default, PartialEq)]
enum Primary {
    #[default]
    Automatic,
    Interval {
        step: f64,
        binding: IntervalBinding,
    },
    Target(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalBinding {
    Linear,
    Log10,
}

impl TickGeneration {
    pub(crate) const MAX_COUNT: u32 = 10_000;

    /// Use renderer-owned automatic density and nice-step selection.
    pub const fn automatic() -> Self {
        Self {
            primary: Primary::Automatic,
            maximum: None,
        }
    }

    /// Inspect whether the complete request is Automatic without an explicit cap.
    pub const fn is_automatic(&self) -> bool {
        matches!(self.primary, Primary::Automatic) && self.maximum.is_none()
    }

    /// Prefer a finite strictly positive additive step on a linear numeric axis.
    pub fn additive(step: f64) -> Result<Self> {
        if !step.is_finite() || step <= 0.0 {
            return Err(Error::Data(
                "tick interval must be finite and strictly positive".into(),
            ));
        }
        Ok(Self {
            primary: Primary::Interval {
                step,
                binding: IntervalBinding::Linear,
            },
            maximum: None,
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
            primary: Primary::Interval {
                step: f64::from(decades),
                binding: IntervalBinding::Log10,
            },
            maximum: None,
        })
    }

    /// Request approximately `count` numeric positions, allowing nice-step deviation.
    ///
    /// Counts must be in `1..=10_000`; one requests the original-domain midpoint.
    /// Actual label fit can select a coarser complete source.
    pub fn target(count: u32) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            primary: Primary::Target(count),
            maximum: None,
        })
    }

    /// Bound generated in-domain positions, including endpoints, by `count`.
    ///
    /// Counts must be in `1..=10_000`. The bound applies to complete generated
    /// sources, not exact caller ticks; it never drops individual labels.
    pub fn maximum(count: u32) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            primary: Primary::Automatic,
            maximum: Some(count),
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

    /// Inspect the target count, or None for another selection.
    pub const fn target_count(&self) -> Option<u32> {
        match self.primary {
            Primary::Target(count) => Some(count),
            _ => None,
        }
    }

    /// Inspect the hard maximum count, or None for another selection.
    pub const fn maximum_count(&self) -> Option<u32> {
        self.maximum
    }

    pub(crate) const fn is_interval(&self) -> bool {
        matches!(self.primary, Primary::Interval { .. })
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        round.observe_count(self.maximum.map_or(0, |count| count as usize))?;
        match self.primary {
            Primary::Automatic => round.observe_count(0),
            Primary::Interval { step, binding } => {
                round.observe_count(1)?;
                round.observe_count(binding as usize)?;
                round.observe_number(step)
            }
            Primary::Target(count) => {
                round.observe_count(3)?;
                round.observe_count(count as usize)
            }
        }
    }

    /// Inspect the positive additive step, or None for another selection.
    pub const fn additive_step(&self) -> Option<f64> {
        match self.primary {
            Primary::Interval {
                step,
                binding: IntervalBinding::Linear,
            } => Some(step),
            _ => None,
        }
    }

    /// Inspect the positive integral decade step, or None for another selection.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "Log10 binding is constructed only from a nonzero exactly represented u32"
    )]
    pub const fn decade_step(&self) -> Option<u32> {
        match self.primary {
            Primary::Interval {
                step,
                binding: IntervalBinding::Log10,
            } => Some(step as u32),
            _ => None,
        }
    }
}
