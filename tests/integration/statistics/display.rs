//! `lichtung-plot::tests::test_stats_display` — display-stat correctness.
//!
//! The plot layer computes **display** statistics (KDE, OLS, quantiles)
//! as geometric prerequisites for rendering, not as
//! inferential statistics. These tests confirm the
//! display stats are correct against well-known
//! reference values:
//!
//! 1. **OLS** matches the closed-form regression on a
//!    hand-constructed `(x, y)` pair where the slope and
//!    intercept are known exactly. (We cross-check
//!    against the analytical formula rather than sklearn
//!    to keep the test free of Python dependencies.)
//! 2. **KDE** integrates to ≈ 1.0 over a wide grid — a
//!    properly normalized PDF property.
//! 3. **Quantile** returns the expected percentile for
//!    a sorted input.
//!
//! These are *display*-level invariants. The actual
//! statistical inference (p-values, F-statistic, etc.)
//! lives in `lichtung-analysis` and is tested there.

#![allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::unreadable_literal
)]

use lichtung_plot::stats::distribution::{kde, quantile};
use lichtung_plot::stats::regression::ols;

/// Closed-form OLS for `(x, y)` with known slope and
/// intercept. Useful as a reference for `ols()` tests.
fn closed_form_ols(x: &[f64], y: &[f64]) -> (f64, f64, f64) {
    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
    let sum_x2: f64 = x.iter().map(|v| v * v).sum();
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;
    let sxx = sum_x2 - n * mean_x * mean_x;
    let sxy = sum_xy - n * mean_x * mean_y;
    let slope = sxy / sxx;
    let intercept = mean_y - slope * mean_x;
    let syy: f64 = y.iter().map(|v| (v - mean_y).powi(2)).sum();
    let r2 = (sxy * sxy) / (sxx * syy);
    (intercept, slope, r2)
}

#[test]
fn ols_perfect_linear_match() {
    // y = 3.0 + 2.0*x, no noise. R² must be exactly 1.0.
    let x: Vec<f64> = (0..50).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|xi| 3.0 + 2.0 * xi).collect();

    let (intercept, slope, r2) = ols(&x, &y).expect("ols on 50 points");
    let (cf_i, cf_s, cf_r2) = closed_form_ols(&x, &y);

    assert!(
        (intercept - cf_i).abs() < 1e-9,
        "intercept {intercept} != closed-form {cf_i}"
    );
    assert!(
        (slope - cf_s).abs() < 1e-9,
        "slope {slope} != closed-form {cf_s}"
    );
    assert!(
        (r2 - 1.0).abs() < 1e-12,
        "R² should be exactly 1.0 for y = a + b*x (got {r2})"
    );
    let _ = cf_r2;
}

#[test]
fn ols_noisy_linear_approximates_truth() {
    // y = 1.0 + 0.5*x + Gaussian noise (mean 0, std 0.1).
    // R² should be very high (≥ 0.99) and slope close to 0.5.
    let x: Vec<f64> = (0..200).map(|i| f64::from(i) * 0.1).collect();
    // Use a deterministic pseudo-random sequence for
    // reproducibility (no `rand` dep).
    let y: Vec<f64> = x
        .iter()
        .enumerate()
        .map(|(i, &xi)| {
            // LCG-ish: 6-sigma-clamped noise.
            let r = (((i * 1103515245 + 12345) % 1000) as f64 / 1000.0) - 0.5;
            1.0 + 0.5 * xi + r * 0.2
        })
        .collect();

    let (intercept, slope, r2) = ols(&x, &y).expect("ols on 200 points");
    assert!(
        (slope - 0.5).abs() < 0.05,
        "estimated slope {slope} should be close to 0.5 (tolerance 0.05)"
    );
    assert!(
        (intercept - 1.0).abs() < 0.1,
        "estimated intercept {intercept} should be close to 1.0"
    );
    assert!(
        r2 > 0.99,
        "R² should be ≥ 0.99 for nearly-perfect linear data (got {r2})"
    );
}

#[test]
fn ols_rejects_mismatched_lengths() {
    let x = vec![1.0, 2.0, 3.0];
    let y = vec![4.0, 5.0];
    let err = ols(&x, &y).expect_err("mismatched lengths must error");
    let msg = err.to_string();
    assert!(
        msg.contains("x.len()") && msg.contains("y.len()"),
        "error should describe the length mismatch, got: {msg}"
    );
}

#[test]
fn ols_rejects_too_few_points() {
    let x = vec![1.0];
    let y = vec![2.0];
    let err = ols(&x, &y).expect_err("n=1 must error");
    assert!(
        err.to_string().contains("at least 2"),
        "error should mention the n=2 minimum, got: {err}"
    );
}

#[test]
fn kde_integrates_to_one_over_wide_grid() {
    // 1000 samples from a standard normal, KDE evaluated
    // on a wide grid. Trapezoidal-rule integration should
    // give ≈ 1.0 (the PDF integrates to 1 over ℝ).
    use statrs::distribution::{ContinuousCDF, Normal};
    let normal = Normal::new(0.0, 1.0).unwrap();
    let samples: Vec<f64> = (0..1000)
        .map(|i| {
            // Inverse-CDF sampling: u = i/999 spans (0, 1),
            // inverse_cdf maps to a standard-normal quantile.
            // Use clamping to avoid the ±∞ at u=0/1.
            #[allow(clippy::cast_precision_loss)]
            let u = (i as f64 + 0.5) / 1000.0;
            normal.inverse_cdf(u)
        })
        .collect();

    // Wide grid: ±4σ covers 99.99% of the density.
    let grid: Vec<f64> = (0..401).map(|i| -4.0 + f64::from(i) * 0.02).collect();
    let densities = kde(&samples, &grid).expect("kde on 1000 points");

    // Trapezoidal integration.
    let dx = 0.02;
    let mut integral = 0.0;
    for window in densities.windows(2) {
        integral += (window[0] + window[1]) * 0.5 * dx;
    }
    println!("KDE integral: {integral:.6} (expected ≈ 1.0)");
    assert!(
        (integral - 1.0).abs() < 0.02,
        "KDE integral {integral} should be close to 1.0 (tolerance 0.02)"
    );
}

#[test]
fn kde_peaks_near_mean() {
    // KDE on a known unimodal sample should peak near the
    // sample mean.
    let samples: Vec<f64> = (0..500).map(|i| 5.0 + f64::from(i) * 0.01).collect();
    let grid: Vec<f64> = (0..101).map(|i| f64::from(i) * 0.1).collect();
    let densities = kde(&samples, &grid).expect("kde");

    // Find the argmax.
    let (argmax_idx, &max_density) = densities
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap();
    let argmax_x = grid[argmax_idx];
    println!("KDE peaks at x = {argmax_x:.2} with density {max_density:.4}");
    // Sample mean = 5.0 + 249.5 * 0.01 = 7.495. The KDE
    // should peak in a small neighborhood of that.
    assert!(
        (argmax_x - 7.5).abs() < 1.0,
        "KDE peak at {argmax_x} should be near the sample mean 7.5"
    );
}

#[test]
fn kde_rejects_empty_samples() {
    let grid = vec![0.0, 1.0, 2.0];
    let err = kde(&[], &grid).expect_err("empty samples must error");
    assert!(err.to_string().contains("empty"), "got: {err}");
}

#[test]
fn quantile_linear_ascending() {
    let samples: Vec<f64> = (1..=100).map(f64::from).collect();
    let q50 = quantile(&samples, 0.5).expect("median");
    let q25 = quantile(&samples, 0.25).expect("q1");
    let q75 = quantile(&samples, 0.75).expect("q3");

    assert_eq!(
        [q25, q50, q75].map(f64::to_bits),
        [25.75_f64, 50.5, 75.25].map(f64::to_bits)
    );
}

#[test]
fn quantile_rejects_out_of_range_p() {
    let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        quantile(&samples, -0.1).unwrap_err().to_string(),
        "data error: quantile: q must be finite and in [0, 1], got -0.1"
    );
    assert_eq!(
        quantile(&samples, 1.1).unwrap_err().to_string(),
        "data error: quantile: q must be finite and in [0, 1], got 1.1"
    );
    assert_eq!(
        quantile(&samples, 0.0).unwrap().to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!(
        quantile(&samples, 1.0).unwrap().to_bits(),
        5.0_f64.to_bits()
    );
}

#[test]
fn quantile_rejects_empty() {
    let err = quantile(&[], 0.5).expect_err("empty must error");
    assert_eq!(err.to_string(), "data error: quantile: samples is empty");
}
