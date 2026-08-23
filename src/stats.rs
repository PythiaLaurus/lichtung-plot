//! Display-only statistics submodules.

use statrs::distribution::{ContinuousCDF, StudentsT};

/// KDE, quantiles, and other distribution primitives.
pub mod distribution;
/// OLS regression and related fit-line primitives.
pub mod regression;

pub(crate) fn student_t_critical_975(df: usize) -> Result<f64, &'static str> {
    student_t_critical(df, 0.95)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "an in-memory sample cannot approach the first usize not exactly representable as f64"
)]
pub(crate) fn student_t_critical(df: usize, level: f64) -> Result<f64, &'static str> {
    if df == 0 {
        return Err("Student-t critical value requires positive degrees of freedom");
    }
    if !level.is_finite() || level <= 0.0 || level >= 1.0 {
        return Err("Student-t confidence level must be strictly between zero and one");
    }
    let distribution = StudentsT::new(0.0, 1.0, df as f64)
        .map_err(|_| "could not construct Student-t distribution")?;
    let critical = distribution.inverse_cdf(0.5 + level / 2.0);
    if critical.is_finite() && critical > 0.0 {
        Ok(critical)
    } else {
        Err("Student-t critical value is not finite and positive")
    }
}

#[cfg(test)]
mod tests {
    use super::student_t_critical_975;

    #[test]
    fn student_t_inverse_cdf_matches_hard_coded_oracles() {
        let expected = [
            (1, 12.706_204_736_432_095),
            (2, 4.302_652_729_696_142),
            (10, 2.228_138_851_964_938_5),
        ];
        for (df, oracle) in expected {
            let actual = student_t_critical_975(df).unwrap();
            assert!(
                (actual - oracle).abs() < 1e-9,
                "df={df}: {actual} != {oracle}"
            );
        }
    }
}
