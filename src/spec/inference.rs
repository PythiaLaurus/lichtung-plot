//! Validated interval and model-adjusted effect-band semantics.

/// Statistical interpretation of caller-provided interval bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalKind {
    /// Frequentist confidence interval.
    Confidence,
    /// Bayesian credible interval.
    Credible,
}

impl IntervalKind {
    /// Stable report-visible interval name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Confidence => "confidence interval",
            Self::Credible => "credible interval",
        }
    }
}

/// Inferential scale on which interval bounds were calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferentialScale {
    /// Original outcome or response scale.
    Response,
    /// Model link or linear-predictor scale.
    Link,
    /// Additive model-coefficient scale.
    Coefficient,
    /// Multiplicative ratio scale.
    Ratio,
}

impl InferentialScale {
    /// Stable report-visible scale name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Response => "response scale",
            Self::Link => "link scale",
            Self::Coefficient => "coefficient scale",
            Self::Ratio => "ratio scale",
        }
    }
}

/// Typed provenance for genuine caller-provided interval bounds.
///
/// Fields are private so invalid interval claims cannot be constructed through
/// the public API. Use [`Self::new`] and the accessors below.
///
/// ```compile_fail
/// use lichtung_plot::{InferentialScale, IntervalKind, IntervalSemantics};
///
/// let _ = IntervalSemantics {
///     kind: IntervalKind::Confidence,
///     level: 1.5,
///     scale: InferentialScale::Response,
///     provenance: String::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalSemantics {
    pub(super) kind: IntervalKind,
    pub(super) level: f64,
    pub(super) scale: InferentialScale,
    pub(super) provenance: String,
}

impl IntervalSemantics {
    /// Construct validated interval metadata.
    pub fn new(
        kind: IntervalKind,
        level: f64,
        scale: InferentialScale,
        provenance: impl Into<String>,
    ) -> Result<Self, crate::Error> {
        let semantics = Self {
            kind,
            level,
            scale,
            provenance: provenance.into(),
        };
        semantics.validate()?;
        Ok(semantics)
    }

    /// Revalidate interval invariants at a rendering or reporting boundary.
    pub fn validate(&self) -> Result<(), crate::Error> {
        if !self.level.is_finite() || self.level <= 0.0 || self.level >= 1.0 {
            return Err(crate::Error::Data(format!(
                "interval level must be finite and strictly between 0 and 1, got {}",
                self.level
            )));
        }
        if self.provenance.trim().is_empty() {
            return Err(crate::Error::Data(
                "interval provenance must not be empty".into(),
            ));
        }
        Ok(())
    }

    /// Confidence or credible interpretation.
    #[must_use]
    pub const fn kind(&self) -> IntervalKind {
        self.kind
    }

    /// Coverage level as a fraction strictly between zero and one.
    #[must_use]
    pub const fn level(&self) -> f64 {
        self.level
    }

    /// Scale on which the bounds were calculated.
    #[must_use]
    pub const fn scale(&self) -> InferentialScale {
        self.scale
    }

    /// Non-empty source or method that produced the bounds.
    #[must_use]
    pub fn provenance(&self) -> &str {
        &self.provenance
    }

    /// Complete report-visible description.
    #[must_use]
    pub fn label(&self) -> String {
        format!(
            "{}% {} · {} · {}",
            self.level * 100.0,
            self.kind.label(),
            self.scale.label(),
            self.provenance
        )
    }
}

/// Coverage claim carried by a model-adjusted effect band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectBandCoverage {
    /// Each x coordinate has a marginal interval; the band has no joint claim.
    Pointwise,
    /// The producing procedure controls coverage jointly over a named family.
    Simultaneous,
}

/// Validated semantics for a precomputed model-adjusted effect curve and band.
///
/// The contained [`IntervalSemantics`] owns interval kind, level, scale, and
/// interval-producing provenance. Simultaneous coverage can only be constructed
/// with [`Self::simultaneous`], which requires a named method and family.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectBandSemantics {
    interval: IntervalSemantics,
    coverage: EffectBandCoverage,
    simultaneous_method: Option<String>,
    simultaneous_family: Option<String>,
    model_provenance: String,
    grid_provenance: String,
}

impl EffectBandSemantics {
    /// Construct a band whose intervals are marginal at each x coordinate.
    pub fn pointwise(
        interval: IntervalSemantics,
        model_provenance: impl Into<String>,
        grid_provenance: impl Into<String>,
    ) -> Result<Self, crate::Error> {
        Self::new(
            interval,
            EffectBandCoverage::Pointwise,
            None,
            None,
            model_provenance.into(),
            grid_provenance.into(),
        )
    }

    /// Construct a genuine simultaneous band over a named multiplicity family.
    pub fn simultaneous(
        interval: IntervalSemantics,
        method: impl Into<String>,
        family: impl Into<String>,
        model_provenance: impl Into<String>,
        grid_provenance: impl Into<String>,
    ) -> Result<Self, crate::Error> {
        Self::new(
            interval,
            EffectBandCoverage::Simultaneous,
            Some(method.into()),
            Some(family.into()),
            model_provenance.into(),
            grid_provenance.into(),
        )
    }

    fn new(
        interval: IntervalSemantics,
        coverage: EffectBandCoverage,
        simultaneous_method: Option<String>,
        simultaneous_family: Option<String>,
        model_provenance: String,
        grid_provenance: String,
    ) -> Result<Self, crate::Error> {
        let semantics = Self {
            interval,
            coverage,
            simultaneous_method,
            simultaneous_family,
            model_provenance,
            grid_provenance,
        };
        semantics.validate()?;
        Ok(semantics)
    }

    /// Revalidate effect-band invariants at rendering and reporting boundaries.
    pub fn validate(&self) -> Result<(), crate::Error> {
        self.interval.validate()?;
        if self.model_provenance.trim().is_empty() {
            return Err(crate::Error::Data(
                "effect-band model provenance must not be empty".into(),
            ));
        }
        if self.grid_provenance.trim().is_empty() {
            return Err(crate::Error::Data(
                "effect-band grid provenance must not be empty".into(),
            ));
        }
        match self.coverage {
            EffectBandCoverage::Pointwise => {
                if self.simultaneous_method.is_some() || self.simultaneous_family.is_some() {
                    return Err(crate::Error::Data(
                        "pointwise effect bands cannot carry simultaneous method or family metadata"
                            .into(),
                    ));
                }
            }
            EffectBandCoverage::Simultaneous => {
                if self
                    .simultaneous_method
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(crate::Error::Data(
                        "simultaneous effect-band method must not be empty".into(),
                    ));
                }
                if self
                    .simultaneous_family
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                {
                    return Err(crate::Error::Data(
                        "simultaneous effect-band family must not be empty".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Interval kind, level, scale, and producing provenance.
    #[must_use]
    pub const fn interval(&self) -> &IntervalSemantics {
        &self.interval
    }

    /// Pointwise or simultaneous coverage.
    #[must_use]
    pub const fn coverage(&self) -> EffectBandCoverage {
        self.coverage
    }

    /// Producing method for simultaneous coverage, absent for pointwise bands.
    #[must_use]
    pub fn simultaneous_method(&self) -> Option<&str> {
        self.simultaneous_method.as_deref()
    }

    /// Multiplicity family covered simultaneously, absent for pointwise bands.
    #[must_use]
    pub fn simultaneous_family(&self) -> Option<&str> {
        self.simultaneous_family.as_deref()
    }

    /// Non-empty provenance of the fitted model that produced the estimates.
    #[must_use]
    pub fn model_provenance(&self) -> &str {
        &self.model_provenance
    }

    /// Non-empty provenance of the adjusted prediction grid.
    #[must_use]
    pub fn grid_provenance(&self) -> &str {
        &self.grid_provenance
    }

    /// Concise on-canvas description that never implies unclaimed coverage.
    #[must_use]
    pub fn label(&self) -> String {
        match self.coverage {
            EffectBandCoverage::Pointwise => {
                format!(
                    "model-adjusted effect · pointwise {}",
                    self.interval.label()
                )
            }
            EffectBandCoverage::Simultaneous => format!(
                "model-adjusted effect · simultaneous {} · method {} · family {}",
                self.interval.label(),
                self.simultaneous_method.as_deref().unwrap_or_default(),
                self.simultaneous_family.as_deref().unwrap_or_default()
            ),
        }
    }
}
