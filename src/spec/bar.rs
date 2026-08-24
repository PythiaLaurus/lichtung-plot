//! Typed ordinary-bar inputs and generated error-bar specifications.

use super::estimate::Bounds;
use crate::{CategoryValues, Error, NumericValues};

const DEFAULT_LEVEL: f64 = 0.95;
const DEFAULT_RESAMPLES: usize = 1_000;
const MAX_RESAMPLES: usize = 1_000_000;

/// Closed owned input for an ordinary vertical or horizontal bar plot.
#[derive(Debug, Clone)]
pub struct BarInput {
    pub(crate) source: BarSource,
}

#[derive(Debug, Clone)]
pub(crate) enum BarSource {
    Samples {
        values: NumericValues,
        error_bar: ErrorBar,
    },
    Estimates {
        values: NumericValues,
        bounds: Option<Bounds>,
    },
}

impl BarInput {
    /// Use raw observations, rendering each cell's arithmetic mean.
    ///
    /// The default error bar is a deterministic two-sided 95% percentile
    /// bootstrap confidence interval with 1,000 resamples and seed zero.
    #[must_use]
    pub fn samples(values: NumericValues) -> BarSamples {
        BarSamples {
            values,
            error_bar: ErrorBar::confidence_interval().into(),
        }
    }

    /// Use one precomputed estimate per category, group, and facet cell.
    #[must_use]
    pub fn estimates(values: NumericValues) -> Self {
        Self {
            source: BarSource::Estimates {
                values,
                bounds: None,
            },
        }
    }

    /// Use one precomputed estimate and explicit drawable bounds per cell.
    #[must_use]
    pub fn estimates_with_error_bars(values: NumericValues, bounds: Bounds) -> Self {
        Self {
            source: BarSource::Estimates {
                values,
                bounds: Some(bounds),
            },
        }
    }

    pub(crate) const fn values(&self) -> &NumericValues {
        match &self.source {
            BarSource::Samples { values, .. } | BarSource::Estimates { values, .. } => values,
        }
    }

    pub(crate) const fn bounds(&self) -> Option<&Bounds> {
        match &self.source {
            BarSource::Samples { .. } | BarSource::Estimates { bounds: None, .. } => None,
            BarSource::Estimates {
                bounds: Some(bounds),
                ..
            } => Some(bounds),
        }
    }

    pub(crate) const fn error_bar(&self) -> Option<&ErrorBar> {
        match &self.source {
            BarSource::Samples { error_bar, .. } => Some(error_bar),
            BarSource::Estimates { .. } => None,
        }
    }

    pub(crate) const fn is_samples(&self) -> bool {
        matches!(&self.source, BarSource::Samples { .. })
    }
}

/// Raw bar samples with sample-only generated error-bar configuration.
#[derive(Debug, Clone)]
pub struct BarSamples {
    values: NumericValues,
    error_bar: ErrorBar,
}

impl BarSamples {
    /// Replace the generated error-bar specification while retaining raw samples.
    #[must_use]
    pub fn error_bar(mut self, error_bar: impl Into<ErrorBar>) -> Self {
        self.error_bar = error_bar.into();
        self
    }
}

impl From<BarSamples> for BarInput {
    fn from(samples: BarSamples) -> Self {
        Self {
            source: BarSource::Samples {
                values: samples.values,
                error_bar: samples.error_bar,
            },
        }
    }
}

/// Closed generated error-bar taxonomy for raw bar samples.
#[derive(Debug, Clone)]
pub struct ErrorBar(pub(crate) ErrorBarKind);

#[derive(Debug, Clone)]
pub(crate) enum ErrorBarKind {
    Hidden,
    StandardDeviation(StandardDeviation),
    PercentileInterval(PercentileInterval),
    StandardError(StandardError),
    ConfidenceInterval(ConfidenceInterval),
}

impl ErrorBar {
    /// Draw no generated error bar.
    #[must_use]
    pub const fn hidden() -> Self {
        Self(ErrorBarKind::Hidden)
    }

    /// Configure sample standard-deviation error bars with scale one.
    #[must_use]
    pub const fn standard_deviation() -> StandardDeviation {
        StandardDeviation { scale: 1.0 }
    }

    /// Configure a central percentile interval of the original observations.
    #[must_use]
    pub const fn percentile_interval() -> PercentileInterval {
        PercentileInterval {
            level: DEFAULT_LEVEL,
        }
    }

    /// Configure sample standard-error bars with scale one.
    #[must_use]
    pub const fn standard_error() -> StandardError {
        StandardError { scale: 1.0 }
    }

    /// Configure a two-sided confidence interval with the default bootstrap.
    #[must_use]
    pub const fn confidence_interval() -> ConfidenceInterval {
        ConfidenceInterval {
            level: DEFAULT_LEVEL,
            algorithm: ConfidenceIntervalAlgorithm::Bootstrap(Bootstrap::new()),
        }
    }

    pub(crate) const fn kind(&self) -> &ErrorBarKind {
        &self.0
    }
}

impl Default for ErrorBar {
    fn default() -> Self {
        Self::confidence_interval().into()
    }
}

/// Standard-deviation error-bar configuration.
#[derive(Debug, Clone, Copy)]
pub struct StandardDeviation {
    pub(crate) scale: f64,
}

impl StandardDeviation {
    /// Set the positive finite standard-deviation multiplier.
    pub fn scale(mut self, scale: f64) -> Result<Self, Error> {
        validate_positive_finite(scale, "standard-deviation scale")?;
        self.scale = scale;
        Ok(self)
    }
}

impl From<StandardDeviation> for ErrorBar {
    fn from(spec: StandardDeviation) -> Self {
        Self(ErrorBarKind::StandardDeviation(spec))
    }
}

/// Central percentile interval configuration for original observations.
#[derive(Debug, Clone, Copy)]
pub struct PercentileInterval {
    pub(crate) level: f64,
}

impl PercentileInterval {
    /// Set coverage as a fraction strictly between zero and one.
    pub fn level(mut self, level: f64) -> Result<Self, Error> {
        validate_level(level)?;
        self.level = level;
        Ok(self)
    }
}

impl From<PercentileInterval> for ErrorBar {
    fn from(spec: PercentileInterval) -> Self {
        Self(ErrorBarKind::PercentileInterval(spec))
    }
}

/// Standard-error error-bar configuration.
#[derive(Debug, Clone, Copy)]
pub struct StandardError {
    pub(crate) scale: f64,
}

impl StandardError {
    /// Set the positive finite standard-error multiplier.
    pub fn scale(mut self, scale: f64) -> Result<Self, Error> {
        validate_positive_finite(scale, "standard-error scale")?;
        self.scale = scale;
        Ok(self)
    }
}

impl From<StandardError> for ErrorBar {
    fn from(spec: StandardError) -> Self {
        Self(ErrorBarKind::StandardError(spec))
    }
}

/// Two-sided confidence-interval configuration.
///
/// Algorithm selection is available only through [`Self::bootstrap`] and
/// [`Self::student_t`].
///
/// ```compile_fail
/// use lichtung_plot::ConfidenceIntervalAlgorithm;
/// ```
#[derive(Debug, Clone)]
pub struct ConfidenceInterval {
    pub(crate) level: f64,
    pub(crate) algorithm: ConfidenceIntervalAlgorithm,
}

impl ConfidenceInterval {
    /// Set coverage as a fraction strictly between zero and one.
    pub fn level(mut self, level: f64) -> Result<Self, Error> {
        validate_level(level)?;
        self.level = level;
        Ok(self)
    }

    /// Select a percentile bootstrap with the supplied resampling specification.
    #[must_use]
    pub fn bootstrap(mut self, bootstrap: Bootstrap) -> Self {
        self.algorithm = ConfidenceIntervalAlgorithm::Bootstrap(bootstrap);
        self
    }

    /// Select an exact two-sided Student-t interval for the sample mean.
    #[must_use]
    pub fn student_t(mut self) -> Self {
        self.algorithm = ConfidenceIntervalAlgorithm::StudentT;
        self
    }
}

impl Default for ConfidenceInterval {
    fn default() -> Self {
        ErrorBar::confidence_interval()
    }
}

impl From<ConfidenceInterval> for ErrorBar {
    fn from(spec: ConfidenceInterval) -> Self {
        Self(ErrorBarKind::ConfidenceInterval(spec))
    }
}

/// Closed algorithm selection owned by a confidence interval.
#[derive(Debug, Clone)]
pub(crate) enum ConfidenceIntervalAlgorithm {
    /// Percentile bootstrap of resampled means.
    Bootstrap(Bootstrap),
    /// Exact Student-t interval around the sample mean.
    StudentT,
}

/// Percentile-bootstrap resampling specification.
#[derive(Debug, Clone)]
pub struct Bootstrap {
    pub(crate) resamples: usize,
    pub(crate) seed: u64,
    pub(crate) units: Option<CategoryValues>,
}

impl Bootstrap {
    /// Construct the default 1,000-resample, seed-zero bootstrap.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            resamples: DEFAULT_RESAMPLES,
            seed: 0,
            units: None,
        }
    }

    /// Set the positive resample count, up to one million.
    pub fn resamples(mut self, resamples: usize) -> Result<Self, Error> {
        if !(1..=MAX_RESAMPLES).contains(&resamples) {
            return Err(Error::Data(format!(
                "bootstrap resamples must be between 1 and {MAX_RESAMPLES}, got {resamples}"
            )));
        }
        self.resamples = resamples;
        Ok(self)
    }

    /// Set the deterministic root seed.
    #[must_use]
    pub const fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set aligned sampling-unit identities for multilevel bootstrap resampling.
    #[must_use]
    pub fn units(mut self, units: CategoryValues) -> Self {
        self.units = Some(units);
        self
    }
}

impl Default for Bootstrap {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_level(level: f64) -> Result<(), Error> {
    if !level.is_finite() || level <= 0.0 || level >= 1.0 {
        return Err(Error::Data(format!(
            "error-bar level must be finite and strictly between 0 and 1, got {level}"
        )));
    }
    Ok(())
}

fn validate_positive_finite(value: f64, role: &str) -> Result<(), Error> {
    if !value.is_finite() || value <= 0.0 {
        return Err(Error::Data(format!(
            "{role} must be finite and greater than zero, got {value}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BarInput, BarSource, Bootstrap, ConfidenceIntervalAlgorithm, ErrorBar, ErrorBarKind,
        NumericValues,
    };
    use crate::CategoryValues;

    #[test]
    fn sample_defaults_are_percentile_bootstrap_95_with_1000_resamples_and_seed_zero() {
        let input: BarInput = BarInput::samples(NumericValues::new("value", [1.0])).into();
        let BarSource::Samples { error_bar, .. } = input.source else {
            panic!("samples constructor must retain sample provenance");
        };
        let ErrorBarKind::ConfidenceInterval(spec) = error_bar.0 else {
            panic!("sample default must be a confidence interval");
        };
        let ConfidenceIntervalAlgorithm::Bootstrap(bootstrap) = spec.algorithm else {
            panic!("sample default must use bootstrap");
        };

        assert_eq!(spec.level.to_bits(), 0.95_f64.to_bits());
        assert_eq!((bootstrap.resamples, bootstrap.seed), (1_000, 0));
        assert!(bootstrap.units.is_none());
    }

    #[test]
    fn error_bar_factories_cover_the_closed_taxonomy() {
        let variants = [
            ErrorBar::hidden(),
            ErrorBar::standard_deviation().into(),
            ErrorBar::percentile_interval().into(),
            ErrorBar::standard_error().into(),
            ErrorBar::confidence_interval().into(),
        ];

        assert!(matches!(variants[0].kind(), ErrorBarKind::Hidden));
        assert!(matches!(
            variants[1].kind(),
            ErrorBarKind::StandardDeviation(_)
        ));
        assert!(matches!(
            variants[2].kind(),
            ErrorBarKind::PercentileInterval(_)
        ));
        assert!(matches!(variants[3].kind(), ErrorBarKind::StandardError(_)));
        assert!(matches!(
            variants[4].kind(),
            ErrorBarKind::ConfidenceInterval(_)
        ));
    }

    #[test]
    fn confidence_interval_modifiers_preserve_unspecified_defaults() {
        let units = CategoryValues::new("participant", ["a", "b"]).unwrap();
        let bootstrap = Bootstrap::new().resamples(37).unwrap().seed(9).units(units);
        let interval = ErrorBar::confidence_interval()
            .level(0.9)
            .unwrap()
            .bootstrap(bootstrap);
        let ConfidenceIntervalAlgorithm::Bootstrap(bootstrap) = interval.algorithm else {
            panic!("bootstrap modifier must retain the bootstrap algorithm");
        };

        assert_eq!(interval.level.to_bits(), 0.9_f64.to_bits());
        assert_eq!((bootstrap.resamples, bootstrap.seed), (37, 9));
        assert!(bootstrap.units.is_some());
    }

    #[test]
    fn numeric_modifiers_reject_invalid_values() {
        assert!(ErrorBar::confidence_interval().level(1.0).is_err());
        assert!(ErrorBar::standard_deviation().scale(0.0).is_err());
        assert!(ErrorBar::percentile_interval().level(f64::NAN).is_err());
        assert!(ErrorBar::standard_error().scale(f64::INFINITY).is_err());
        assert!(Bootstrap::new().resamples(0).is_err());
    }
}
