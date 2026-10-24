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
/// assert_eq!(TickGeneration::maximum(6)?.get_maximum_count(), Some(6));
/// assert!(TickGeneration::default().is_automatic());
/// assert!(TickGeneration::target(0).is_err());
/// assert!(TickGeneration::maximum(10_001).is_ok());
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
    maximum: Option<usize>,
}

// All admitted floating steps are finite and strictly positive.
impl Eq for TickGeneration {}

#[derive(Debug, Clone, Default, PartialEq)]
enum Primary {
    #[default]
    Automatic,
    Interval {
        step: f64,
        origin: f64,
        space: TickSpace,
        requirement: TickRequirement,
        binding: IntervalBinding,
    },
    Target(usize),
    ExactCount {
        count: usize,
        space: TickSpace,
        singleton: TickSingleton,
    },
}

/// Coordinate space for generated positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickSpace {
    /// Raw scientific data units.
    Data,
    /// Linear coordinates or logarithmic exponents.
    Scale,
}

/// Whether a lattice is fixed or may try complete coarser candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickRequirement {
    /// Retain the complete requested lattice, including a valid empty result.
    Fixed,
    /// Try the requested lattice before the documented coarser source program.
    Preferred,
}

/// Position of an exact singleton, in the selected coordinate space.
#[derive(Debug, Clone, Copy)]
pub enum TickSingleton {
    /// Lower domain endpoint (default).
    Lower,
    /// Exact midpoint, rounded once.
    Midpoint,
    /// Upper domain endpoint.
    Upper,
    /// A finite in-domain position.
    Position(f64),
}
impl PartialEq for TickSingleton {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Lower, Self::Lower)
            | (Self::Midpoint, Self::Midpoint)
            | (Self::Upper, Self::Upper) => true,
            (Self::Position(a), Self::Position(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}
impl Eq for TickSingleton {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalBinding {
    Linear,
    Log10,
    Context,
}

impl TickGeneration {
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
                origin: 0.0,
                space: TickSpace::Data,
                requirement: TickRequirement::Preferred,
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
                origin: 0.0,
                space: TickSpace::Scale,
                requirement: TickRequirement::Preferred,
                binding: IntervalBinding::Log10,
            },
            maximum: None,
        })
    }

    /// Select a finite positive interval and finite origin in the declared space.
    pub fn interval(
        step: f64,
        origin: f64,
        space: TickSpace,
        requirement: TickRequirement,
    ) -> Result<Self> {
        if !step.is_finite() || step <= 0.0 || !origin.is_finite() {
            return Err(Error::Data(
                "tick interval needs a finite positive step and finite origin".into(),
            ));
        }
        Ok(Self {
            primary: Primary::Interval {
                step,
                origin: if origin == 0.0 { 0.0 } else { origin },
                space,
                requirement,
                binding: IntervalBinding::Context,
            },
            maximum: None,
        })
    }

    /// Inspect the retained step, origin, coordinate space and source requirement.
    pub const fn interval_parameters(&self) -> Option<(f64, f64, TickSpace, TickRequirement)> {
        if let Primary::Interval {
            step,
            origin,
            space,
            requirement,
            ..
        } = self.primary
        {
            Some((step, origin, space, requirement))
        } else {
            None
        }
    }

    pub(crate) fn validate_scale(&self, log: bool) -> Result<()> {
        if let Primary::Interval { binding, .. } = self.primary {
            if (binding == IntervalBinding::Linear && log)
                || (binding == IntervalBinding::Log10 && !log)
            {
                return Err(Error::Data(
                    "tick interval does not match the axis scale".into(),
                ));
            }
        }
        Ok(())
    }

    /// Request approximately `count` numeric positions, allowing nice-step deviation.
    ///
    /// Counts must be positive; one requests the original-domain midpoint.
    /// Actual label fit can select a coarser complete source.
    pub fn target(count: usize) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            primary: Primary::Target(count),
            maximum: None,
        })
    }

    /// Bound generated in-domain positions, including endpoints, by `count`.
    ///
    /// Counts must be positive. The bound applies to complete generated
    /// sources, not exact caller ticks; it never drops individual labels.
    pub fn maximum(count: usize) -> Result<Self> {
        Self::validate_count(count)?;
        Ok(Self {
            primary: Primary::Automatic,
            maximum: Some(count),
        })
    }

    fn validate_count(count: usize) -> Result<()> {
        if count == 0 {
            return Err(Error::Data("generated tick count must be positive".into()));
        }
        Ok(())
    }

    /// Inspect the target count, or None for another selection.
    pub const fn target_count(&self) -> Option<usize> {
        match self.primary {
            Primary::Target(count) => Some(count),
            _ => None,
        }
    }

    /// Inspect the hard maximum count, or None for another selection.
    pub const fn get_maximum_count(&self) -> Option<usize> {
        self.maximum
    }

    /// Set or clear the positive generated cap, without replacing the locator.
    pub fn maximum_count(mut self, maximum: Option<usize>) -> Result<Self> {
        if let Some(maximum) = maximum {
            Self::validate_count(maximum)?;
            if self
                .exact_parameters()
                .is_some_and(|(count, _, _)| count > maximum)
            {
                return Err(Error::Data(
                    "exact tick count exceeds the generated maximum".into(),
                ));
            }
        }
        self.maximum = maximum;
        Ok(self)
    }

    /// Generate exactly count positions; zero is empty and one initially uses Lower.
    pub const fn exact_count(count: usize, space: TickSpace) -> Self {
        Self {
            primary: Primary::ExactCount {
                count,
                space,
                singleton: TickSingleton::Lower,
            },
            maximum: None,
        }
    }

    /// Set exact-count singleton placement, retaining the complete request for n>=2.
    pub fn singleton(mut self, position: TickSingleton) -> Result<Self> {
        if let TickSingleton::Position(value) = position {
            if !value.is_finite() {
                return Err(Error::Data("tick singleton must be finite".into()));
            }
        }
        let Primary::ExactCount { singleton, .. } = &mut self.primary else {
            return Err(Error::Data(
                "singleton placement requires an exact-count locator".into(),
            ));
        };
        *singleton = position;
        Ok(self)
    }

    pub(crate) const fn exact_parameters(&self) -> Option<(usize, TickSpace, TickSingleton)> {
        if let Primary::ExactCount {
            count,
            space,
            singleton,
        } = self.primary
        {
            Some((count, space, singleton))
        } else {
            None
        }
    }

    pub(crate) const fn is_interval(&self) -> bool {
        matches!(self.primary, Primary::Interval { .. })
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<()> {
        round.observe_count(self.maximum.unwrap_or(0))?;
        match self.primary {
            Primary::Automatic => round.observe_count(0),
            Primary::Interval {
                step,
                origin,
                space,
                requirement,
                binding,
            } => {
                round.observe_count(1)?;
                round.observe_count(binding as usize)?;
                round.observe_number(origin)?;
                round.observe_count(space as usize)?;
                round.observe_count(requirement as usize)?;
                round.observe_number(step)
            }
            Primary::Target(count) => {
                round.observe_count(3)?;
                round.observe_count(count)
            }
            Primary::ExactCount {
                count,
                space,
                singleton,
            } => {
                round.observe_count(4)?;
                round.observe_count(count)?;
                round.observe_count(space as usize)?;
                match singleton {
                    TickSingleton::Lower => round.observe_count(0),
                    TickSingleton::Midpoint => round.observe_count(1),
                    TickSingleton::Upper => round.observe_count(2),
                    TickSingleton::Position(value) => {
                        round.observe_count(3)?;
                        round.observe_count(usize::from(value.is_sign_negative()))?;
                        round.observe_number(value)
                    }
                }
            }
        }
    }

    /// Inspect the positive additive step, or None for another selection.
    pub const fn additive_step(&self) -> Option<f64> {
        match self.primary {
            Primary::Interval {
                step,
                binding: IntervalBinding::Linear,
                ..
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
                ..
            } => Some(step as u32),
            _ => None,
        }
    }
}
