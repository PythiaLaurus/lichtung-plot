//! Independent numerical checks through the public display-statistics API.

use lichtung_plot::stats::{distribution::kde, regression::ols};

#[test]
fn ols_perfect_linear_match() {
    let x: Vec<f64> = (0..50).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|x| 3.0 + 2.0 * x).collect();
    let (intercept, slope, r2) = ols(&x, &y).unwrap();
    assert!((intercept - 3.0).abs() < 1e-12);
    assert!((slope - 2.0).abs() < 1e-12);
    assert!((r2 - 1.0).abs() < 1e-12);
}

#[test]
fn kde_integrates_to_one_over_wide_grid() {
    use statrs::distribution::{ContinuousCDF, Normal};
    let normal = Normal::new(0.0, 1.0).unwrap();
    let samples: Vec<f64> = (0..1000)
        .map(|index| normal.inverse_cdf((f64::from(index) + 0.5) / 1000.0))
        .collect();
    let grid: Vec<f64> = (0..401)
        .map(|index| -4.0 + f64::from(index) * 0.02)
        .collect();
    let densities = kde(&samples, &grid).unwrap();
    let integral: f64 = densities
        .windows(2)
        .map(|pair| (pair[0] + pair[1]) * 0.5 * 0.02)
        .sum();
    assert!((integral - 1.0).abs() < 0.02);
}
