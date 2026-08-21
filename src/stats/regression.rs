//! Stable display-only simple OLS and mean-response interval geometry.

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MeanResponseInterval {
    pub(crate) estimate: f64,
    pub(crate) lower: f64,
    pub(crate) upper: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct OlsFit {
    n: usize,
    x_scale: f64,
    y_scale: f64,
    x_mean_scaled: f64,
    y_mean_scaled: f64,
    slope_scaled: f64,
    sxx_scaled: f64,
    residual_sum_squares_scaled: f64,
    intercept: f64,
    slope: f64,
    r_squared: f64,
}

impl OlsFit {
    #[expect(
        clippy::cast_precision_loss,
        reason = "an in-memory sample cannot approach the first usize not exactly representable as f64"
    )]
    pub(crate) fn new(x: &[f64], y: &[f64]) -> Result<Self, Error> {
        if x.len() != y.len() {
            return Err(Error::Data(format!(
                "ols: x.len() ({}) != y.len() ({})",
                x.len(),
                y.len()
            )));
        }
        if x.len() < 2 {
            return Err(Error::Data(format!(
                "ols: need at least 2 points, got {}",
                x.len()
            )));
        }
        if x.iter().chain(y).any(|value| !value.is_finite()) {
            return Err(Error::Data(
                "ols: x and y must contain only finite values".into(),
            ));
        }

        let x_scale = nonzero_scale(x);
        let y_scale = nonzero_scale(y);
        let n = x.len();
        let n_f = n as f64;
        let x_mean_scaled = compensated_sum(x.iter().map(|value| value / x_scale))? / n_f;
        let y_mean_scaled = compensated_sum(y.iter().map(|value| value / y_scale))? / n_f;
        let mut predictor_variation = CompensatedSum::default();
        let mut cross_product = CompensatedSum::default();
        let mut response_variation = CompensatedSum::default();
        for (&x_value, &y_value) in x.iter().zip(y) {
            let dx = x_value / x_scale - x_mean_scaled;
            let dy = y_value / y_scale - y_mean_scaled;
            predictor_variation.add(dx * dx)?;
            cross_product.add(dx * dy)?;
            response_variation.add(dy * dy)?;
        }
        let denominator = predictor_variation.finish()?;
        if denominator == 0.0 {
            return Err(Error::Data(
                "ols: x has zero variance; slope is undefined".into(),
            ));
        }
        let covariance_numerator = cross_product.finish()?;
        let total_variation = response_variation.finish()?;
        let slope_scaled = covariance_numerator / denominator;
        let normalized_intercept = y_mean_scaled - slope_scaled * x_mean_scaled;
        let intercept = normalized_intercept * y_scale;
        let slope = (slope_scaled * y_scale) / x_scale;
        if !intercept.is_finite() || !slope.is_finite() {
            return Err(Error::Data(
                "ols: fitted coefficients are not numerically representable".into(),
            ));
        }
        let r_squared = if total_variation == 0.0 {
            1.0
        } else {
            let correlation = (covariance_numerator / denominator.sqrt()) / total_variation.sqrt();
            correlation.mul_add(correlation, 0.0).clamp(0.0, 1.0)
        };

        let mut residual_sum_squares_scaled = CompensatedSum::default();
        for (&x_value, &y_value) in x.iter().zip(y) {
            let estimate_scaled =
                y_mean_scaled + slope_scaled * (x_value / x_scale - x_mean_scaled);
            let residual_scaled = y_value / y_scale - estimate_scaled;
            residual_sum_squares_scaled.add(residual_scaled * residual_scaled)?;
        }

        Ok(Self {
            n,
            x_scale,
            y_scale,
            x_mean_scaled,
            y_mean_scaled,
            slope_scaled,
            sxx_scaled: denominator,
            residual_sum_squares_scaled: residual_sum_squares_scaled.finish()?,
            intercept,
            slope,
            r_squared,
        })
    }

    pub(crate) const fn coefficients(&self) -> (f64, f64, f64) {
        (self.intercept, self.slope, self.r_squared)
    }

    pub(crate) fn predict(&self, x: f64) -> Result<f64, Error> {
        if !x.is_finite() {
            return Err(Error::Data(
                "ols: prediction coordinate must be finite".into(),
            ));
        }
        let estimate = self.prediction_scaled(x) * self.y_scale;
        if estimate.is_finite() {
            Ok(estimate)
        } else {
            Err(Error::Data(
                "ols: fitted value is not numerically representable".into(),
            ))
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "an in-memory sample cannot approach the first usize not exactly representable as f64"
    )]
    pub(crate) fn pointwise_mean_response_95(&self, x: f64) -> Result<MeanResponseInterval, Error> {
        if self.n < 3 {
            return Err(Error::Data(format!(
                "regression: a pointwise mean-response Student-t band requires at least 3 points, got {}",
                self.n
            )));
        }
        if !x.is_finite() {
            return Err(Error::Data(
                "regression: band coordinate must be finite".into(),
            ));
        }
        let residual_df = self.n - 2;
        let critical = super::student_t_critical_975(residual_df)
            .map_err(|reason| Error::Data(format!("regression: {reason}")))?;
        let x_scaled = x / self.x_scale;
        let leverage =
            1.0 / self.n as f64 + (x_scaled - self.x_mean_scaled).powi(2) / self.sxx_scaled;
        let mse_scaled = self.residual_sum_squares_scaled / residual_df as f64;
        let half_width_scaled = critical * (mse_scaled * leverage).sqrt();
        let estimate_scaled = self.prediction_scaled(x);
        let interval = MeanResponseInterval {
            estimate: estimate_scaled * self.y_scale,
            lower: (estimate_scaled - half_width_scaled) * self.y_scale,
            upper: (estimate_scaled + half_width_scaled) * self.y_scale,
        };
        if [interval.estimate, interval.lower, interval.upper]
            .into_iter()
            .all(f64::is_finite)
            && interval.lower <= interval.estimate
            && interval.estimate <= interval.upper
        {
            Ok(interval)
        } else {
            Err(Error::Data(
                "regression: confidence-band geometry is not numerically representable".into(),
            ))
        }
    }

    fn prediction_scaled(&self, x: f64) -> f64 {
        self.y_mean_scaled + self.slope_scaled * (x / self.x_scale - self.x_mean_scaled)
    }
}

/// Fit simple OLS and return `(intercept, slope, r_squared)`.
///
/// # Errors
///
/// Returns an error for unequal lengths, fewer than two points, non-finite
/// values, zero x variance, or unrepresentable fitted coefficients.
pub fn ols(x: &[f64], y: &[f64]) -> Result<(f64, f64, f64), Error> {
    Ok(OlsFit::new(x, y)?.coefficients())
}

fn nonzero_scale(values: &[f64]) -> f64 {
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        1.0
    } else {
        scale
    }
}

fn compensated_sum(values: impl IntoIterator<Item = f64>) -> Result<f64, Error> {
    let mut sum = CompensatedSum::default();
    for value in values {
        sum.add(value)?;
    }
    sum.finish()
}

#[derive(Debug, Default)]
struct CompensatedSum {
    sum: f64,
    correction: f64,
}

impl CompensatedSum {
    fn add(&mut self, value: f64) -> Result<(), Error> {
        let next = self.sum + value;
        if !next.is_finite() {
            return Err(Error::Data(
                "ols: centered/scaled accumulation is not numerically representable".into(),
            ));
        }
        self.correction += if self.sum.abs() >= value.abs() {
            (self.sum - next) + value
        } else {
            (value - next) + self.sum
        };
        if !self.correction.is_finite() {
            return Err(Error::Data(
                "ols: centered/scaled accumulation is not numerically representable".into(),
            ));
        }
        self.sum = next;
        Ok(())
    }

    fn finish(self) -> Result<f64, Error> {
        let value = self.sum + self.correction;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(Error::Data(
                "ols: centered/scaled accumulation is not numerically representable".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ols, OlsFit};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected} within {tolerance}"
        );
    }

    #[test]
    fn pointwise_band_matches_hard_coded_student_t_oracle() {
        let fit = OlsFit::new(&[1.0, 2.0, 3.0], &[1.0, 2.0, 4.0]).unwrap();
        let interval = fit.pointwise_mean_response_95(2.0).unwrap();

        assert_close(interval.estimate, 2.333_333_333_333_333_5, 1e-14);
        assert_close(interval.lower, -0.661_547_844_031_249_9, 1e-12);
        assert_close(interval.upper, 5.328_214_510_697_917, 1e-12);
    }

    #[test]
    fn fit_and_band_are_affine_and_scale_invariant() {
        let x = [1.0, 2.0, 4.0, 7.0, 11.0];
        let y = [2.0, 2.5, 5.0, 6.5, 10.0];
        let transformed_x = x.map(|value| 1_073_741_824.0 + 1_024.0 * value);
        let transformed_y = y.map(|value| -268_435_456.0 + 4_096.0 * value);
        let original = OlsFit::new(&x, &y).unwrap();
        let transformed = OlsFit::new(&transformed_x, &transformed_y).unwrap();
        let original_interval = original.pointwise_mean_response_95(4.0).unwrap();
        let transformed_interval = transformed
            .pointwise_mean_response_95(1_073_741_824.0 + 4_096.0)
            .unwrap();

        assert_close(
            transformed_interval.estimate,
            -268_435_456.0 + 4_096.0 * original_interval.estimate,
            1e-7,
        );
        assert_close(
            transformed_interval.lower,
            -268_435_456.0 + 4_096.0 * original_interval.lower,
            1e-7,
        );
        assert_close(
            transformed_interval.upper,
            -268_435_456.0 + 4_096.0 * original_interval.upper,
            1e-7,
        );
    }

    #[test]
    fn tiny_nonzero_x_scale_is_not_rejected_by_an_absolute_floor() {
        let (_, slope, _) = ols(&[1.0e-200, 2.0e-200, 3.0e-200], &[1.0, 2.0, 3.0]).unwrap();

        assert_close(slope, 1.0e200, 1.0e185);
    }

    #[test]
    fn public_wrapper_preserves_two_point_descriptive_fit() {
        let result = ols(&[2.0, 4.0], &[5.0, 9.0]).unwrap();

        assert_close(result.0, 1.0, 1e-14);
        assert_close(result.1, 2.0, 1e-14);
        assert_close(result.2, 1.0, 1e-14);
    }

    #[test]
    fn pointwise_band_rejects_two_points() {
        let fit = OlsFit::new(&[2.0, 4.0], &[5.0, 9.0]).unwrap();

        assert!(fit
            .pointwise_mean_response_95(3.0)
            .unwrap_err()
            .to_string()
            .contains("at least 3 points"));
    }
}
