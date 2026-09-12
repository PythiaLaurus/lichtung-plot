//! Display-only distribution primitives.
//!
//! KDE primitives support violin and standalone density plots. Type-7 sample
//! quantiles support box, violin, and Q-Q reference geometry; standard-normal
//! quantiles provide deterministic Q-Q theoretical positions.

/// Kernel density estimate using a Gaussian kernel and
/// Scott's rule bandwidth.
///
/// `kde(samples, grid_points)` evaluates the KDE at each
/// point in `grid_points` and returns the density values
/// (one per grid point). The bandwidth is computed from
/// `samples` via Scott's rule (`h = n^(-1/5) * std(samples)`);
/// all grid points share the same bandwidth.
///
/// # Errors
///
/// Returns `Error::Data` if samples, grid points, or the derived
/// bandwidth cannot produce finite density values.
///
/// # Examples
///
/// ```
/// use lichtung_plot::stats::distribution::kde;
///
/// let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0];
/// let grid = vec![0.0, 1.0, 2.0, 3.0, 4.0];
/// let densities = kde(&samples, &grid).unwrap();
/// // The density is highest at the center (2.0) and
/// // falls off symmetrically.
/// assert!(densities[2] > densities[0]);
/// assert!(densities[2] > densities[4]);
/// ```
pub fn kde(samples: &[f64], grid_points: &[f64]) -> Result<Vec<f64>, crate::Error> {
    kde_with_source(samples, grid_points, None)
}

pub(crate) fn kde_with_source(
    samples: &[f64],
    grid_points: &[f64],
    data: Option<crate::data::PlotData<'_>>,
) -> Result<Vec<f64>, crate::Error> {
    use crate::data::source::SourceAdmission;
    use crate::Error;
    if samples.is_empty() {
        return Err(Error::Data(
            "kde: samples is empty; bandwidth is undefined".into(),
        ));
    }
    let n = samples.len();
    // Cast is intentional: sample counts are small in
    // practice (typical: tens to thousands); they fit in
    // f64 without loss for the n^(-1/5) computation.
    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;

    // Bandwidth via Scott's rule.
    data.source_visits(samples.len())?;
    let mean = samples.iter().sum::<f64>() / n_f;
    // Sample variance (n - 1 in the denominator for the
    // unbiased estimator; for the KDE bandwidth the n
    // denominator is also valid, but (n - 1) is the
    // scipy convention).
    let var = if n > 1 {
        data.source_visits(samples.len())?;
        samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_f - 1.0)
    } else {
        0.0
    };
    let std = var.sqrt();
    // Scott's rule: h = n^(-1/5) * std.
    // n = 1 → h = std (or 0 if all values are equal). Use
    // a small positive floor (1e-6) so the density
    // doesn't blow up to a Dirac delta.
    let h = (n_f.powf(-0.2) * std).max(1e-6);

    gaussian_kde_with_source(samples, grid_points, h, data)
}

/// Evaluate a Gaussian KDE with an already validated bandwidth.
pub(crate) fn gaussian_kde_with_source(
    samples: &[f64],
    grid_points: &[f64],
    bandwidth: f64,
    data: Option<crate::data::PlotData<'_>>,
) -> Result<Vec<f64>, crate::Error> {
    use crate::data::source::SourceAdmission;
    use crate::Error;
    if samples.is_empty() || !bandwidth.is_finite() || bandwidth <= 0.0 {
        return Err(Error::Data(
            "kde: samples and a positive finite bandwidth are required".into(),
        ));
    }
    data.source_visits(samples.len())?;
    data.source_visits(grid_points.len())?;
    if samples
        .iter()
        .chain(grid_points)
        .any(|value| !value.is_finite())
    {
        return Err(Error::Data(
            "kde: samples and grid points must be finite".into(),
        ));
    }
    #[allow(clippy::cast_precision_loss)]
    let n = samples.len() as f64;
    let norm = (1.0 / n) / bandwidth / (2.0 * std::f64::consts::PI).sqrt();
    if !norm.is_finite() {
        return Err(Error::Data(
            "kde: bandwidth is too small for finite density values".into(),
        ));
    }
    let mut densities = Vec::new();
    data.source_reserve(&mut densities, grid_points.len())?;
    for &x in grid_points {
        data.source_visits(samples.len())?;
        let mut sum = 0.0;
        for &s in samples {
            let z = (x - s) / bandwidth;
            sum += (-0.5 * z * z).exp();
        }
        let density = norm * sum;
        if !density.is_finite() {
            return Err(Error::Data("kde: computed a non-finite density".into()));
        }
        densities.push(density);
    }
    Ok(densities)
}

/// Strict, scale-robust Scott bandwidth for standalone density plots.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn strict_scott_bandwidth(samples: &[f64]) -> Result<f64, &'static str> {
    if samples.len() < 2 {
        return Err("requires at least 2 observations for Scott bandwidth");
    }
    if samples.iter().any(|value| !value.is_finite()) {
        return Err("contains non-finite values");
    }
    let minimum = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if minimum.total_cmp(&maximum).is_eq() {
        return Err("has constant values; Scott bandwidth requires nonzero scale");
    }
    let scale = samples.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() || scale <= 0.0 {
        return Err("has nonconstant values but no finite variance scale");
    }

    let mut mean = 0.0;
    let mut sum_squared_deviations = 0.0;
    for (index, value) in samples.iter().copied().enumerate() {
        let normalized = value / scale;
        let count = (index + 1) as f64;
        let delta = normalized - mean;
        mean += delta / count;
        sum_squared_deviations += delta * (normalized - mean);
    }
    let variance = sum_squared_deviations / (samples.len() - 1) as f64;
    if !variance.is_finite() || variance <= 0.0 {
        return Err("has nonconstant values but variance is not numerically representable");
    }
    let standard_deviation = variance.sqrt() * scale;
    if !standard_deviation.is_finite() || standard_deviation <= 0.0 {
        return Err("has nonconstant values but standard deviation is not representable");
    }
    let bandwidth = (samples.len() as f64).powf(-0.2) * standard_deviation;
    if !bandwidth.is_finite() || bandwidth <= 0.0 {
        return Err("has nonzero scale but Scott bandwidth is not representable");
    }
    Ok(bandwidth)
}

/// Scott's-rule KDE bandwidth used by display-only density plots.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn kde_bandwidth(samples: &[f64]) -> f64 {
    if samples.len() < 2 {
        return 1e-6;
    }
    let n = samples.len() as f64;
    let mean = samples.iter().sum::<f64>() / n;
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (n.powf(-0.2) * variance.sqrt()).max(1e-6)
}

/// Union KDE support for one or more sample groups.
pub(crate) fn kde_support_range<'a>(
    groups: impl IntoIterator<Item = &'a [f64]>,
    cut: f64,
) -> Option<(f64, f64)> {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for samples in groups {
        let mut group_lo = f64::INFINITY;
        let mut group_hi = f64::NEG_INFINITY;
        for value in samples.iter().copied().filter(|value| value.is_finite()) {
            group_lo = group_lo.min(value);
            group_hi = group_hi.max(value);
        }
        if !(group_lo.is_finite() && group_hi.is_finite()) {
            continue;
        }
        let bandwidth = kde_bandwidth(samples);
        lo = lo.min(group_lo - cut * bandwidth);
        hi = hi.max(group_hi + cut * bandwidth);
    }
    if !(lo.is_finite() && hi.is_finite()) {
        return None;
    }
    Some(if (hi - lo).abs() < f64::EPSILON {
        (lo - 1.0, hi + 1.0)
    } else {
        (lo, hi)
    })
}

/// Quantile (q-th percentile) of `samples`.
///
/// Returns the value below which `q * 100%` of the
/// `samples` lie, using linear interpolation between
/// order statistics (the "Type 7" quantile method in
/// Hyndman & Fan, the same convention as R's default
/// `quantile(type=7)` and NumPy's default
/// `numpy.quantile(method='linear')`).
///
/// `q` must be in `[0.0, 1.0]`.
///
/// # Errors
///
/// Returns `Error::Data` if `samples` is empty, `q` is non-finite or
/// outside `[0, 1]`, or any sample is non-finite.
///
/// # Examples
///
/// ```
/// use lichtung_plot::stats::distribution::quantile;
///
/// let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// assert!((quantile(&samples, 0.5).unwrap() - 3.0).abs() < 1e-9);
/// assert!((quantile(&samples, 0.0).unwrap() - 1.0).abs() < 1e-9);
/// assert!((quantile(&samples, 1.0).unwrap() - 5.0).abs() < 1e-9);
/// ```
pub fn quantile(samples: &[f64], q: f64) -> Result<f64, crate::Error> {
    use crate::Error;
    if samples.is_empty() {
        return Err(Error::Data("quantile: samples is empty".into()));
    }
    if !q.is_finite() || !(0.0..=1.0).contains(&q) {
        return Err(Error::Data(format!(
            "quantile: q must be finite and in [0, 1], got {q}"
        )));
    }
    if let Some((index, value)) = samples
        .iter()
        .copied()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(Error::Data(format!(
            "quantile: sample at index {index} must be finite, got {value}"
        )));
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(sorted_quantile(&sorted, q))
}

/// Type-7 interpolation for a nonempty sorted finite sample and a validated q.
pub(crate) fn sorted_quantile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 1 || q == 0.0 {
        return sorted[0];
    }
    if q.to_bits() == 1.0_f64.to_bits() {
        return sorted[n - 1];
    }
    // Type 7 quantile: h = (n - 1) * q; index = floor(h);
    // fraction = h - index; result = sorted[index] +
    // fraction * (sorted[index + 1] - sorted[index]).
    // Casts are intentional: n is small (≤ thousands)
    // and fits in f64 without loss; h is non-negative
    // and < n - 1 (because q < 1), so the index cast
    // is in range.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let h = (n - 1) as f64 * q;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let index = h.floor() as usize;
    #[allow(clippy::cast_precision_loss)]
    let fraction = h - index as f64;
    if index >= n - 1 {
        return sorted[n - 1];
    }
    if fraction == 0.0 {
        return sorted[index];
    }
    let lower = sorted[index];
    let upper = sorted[index + 1];
    let difference = upper - lower;
    if difference.is_finite() {
        lower + fraction * difference
    } else {
        // Center/half-span form is algebraically equivalent when the direct
        // difference overflows for finite opposite-sign endpoints.
        let midpoint = f64::midpoint(lower, upper);
        let half_span = upper / 2.0 - lower / 2.0;
        midpoint + (2.0 * fraction - 1.0) * half_span
    }
}

/// Inverse CDF of the standard normal distribution.
///
/// Uses Peter J. Acklam's rational approximation and is intended for
/// deterministic display statistics such as normal Q-Q plots. The tested
/// absolute error is below `1e-7` across central and representable lower-tail
/// probabilities; this API does not claim full-f64 inverse-CDF accuracy.
///
/// # Errors
///
/// Returns `Error::Data` unless `probability` is finite and strictly
/// between zero and one.
pub fn standard_normal_quantile(probability: f64) -> Result<f64, crate::Error> {
    use crate::Error;
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const LOWER: f64 = 0.024_25;
    const UPPER: f64 = 1.0 - LOWER;

    if !probability.is_finite() || probability <= 0.0 || probability >= 1.0 {
        return Err(Error::Data(format!(
            "standard normal quantile: probability must be finite and in (0, 1), got {probability}"
        )));
    }

    let quantile = if probability < LOWER {
        let q = (-2.0 * probability.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if probability <= UPPER {
        let q = probability - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - probability).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    if quantile.is_finite() {
        Ok(quantile)
    } else {
        Err(Error::Data(
            "standard normal quantile: approximation produced a non-finite value".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- kde ----

    #[test]
    fn kde_dense_at_center_for_uniform_samples() {
        // 5 samples uniformly spaced in [0, 4]; the
        // density is highest at the center (x = 2)
        // and falls off at the edges.
        let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let grid = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let d = kde(&samples, &grid).unwrap();
        assert!(d[2] > d[0], "center should have higher density than edge");
        assert!(d[2] > d[4], "center should have higher density than edge");
    }

    #[test]
    fn kde_integrates_to_approximately_one() {
        // The KDE integrates to 1 over the real line
        // (it's a probability density). With Scott's
        // rule bandwidth and a fine grid, the
        // trapezoidal sum should be ≈ 1.
        let samples: Vec<f64> = (0..100).map(f64::from).map(|i| i / 10.0).collect();
        let grid: Vec<f64> = (-50..=150).map(f64::from).map(|i| i / 10.0).collect();
        let d = kde(&samples, &grid).unwrap();
        let dx = grid[1] - grid[0];
        let integral: f64 = d.iter().sum::<f64>() * dx;
        // Looser tolerance because the trapezoidal
        // rule is approximate and Scott's bandwidth
        // is itself approximate.
        assert!(
            (integral - 1.0).abs() < 0.1,
            "KDE integral should be ≈ 1.0, got {integral}"
        );
    }

    #[test]
    fn kde_empty_samples_raises() {
        let err = kde(&[], &[0.0, 1.0]).unwrap_err();
        assert!(matches!(err, crate::Error::Data(_)));
    }

    #[test]
    fn strict_scott_bandwidth_rejects_small_and_constant_samples() {
        assert!(strict_scott_bandwidth(&[1.0]).is_err());
        assert!(strict_scott_bandwidth(&[2.0, 2.0]).is_err());
    }

    #[test]
    fn strict_scott_bandwidth_handles_large_offset_samples() {
        let bandwidth = strict_scott_bandwidth(&[
            1e300,
            1.000_000_000_000_000_2e300,
            1.000_000_000_000_000_4e300,
        ])
        .unwrap();

        assert!(bandwidth.is_finite() && bandwidth > 0.0);
    }

    #[test]
    fn strict_scott_bandwidth_handles_subnormal_scale() {
        let bandwidth = strict_scott_bandwidth(&[
            f64::from_bits(1_000),
            f64::from_bits(2_000),
            f64::from_bits(3_000),
        ])
        .unwrap();

        assert!(bandwidth.is_finite() && bandwidth > 0.0);
    }

    #[test]
    fn strict_scott_bandwidth_reports_unrepresentable_variance() {
        let mut samples = vec![0.0; 1_024];
        samples[0] = f64::from_bits(1);

        let error = strict_scott_bandwidth(&samples).unwrap_err();

        assert!(error.contains("not representable"));
    }

    // ---- quantile ----

    fn quartiles(samples: &[f64]) -> [f64; 3] {
        [
            quantile(samples, 0.25).unwrap(),
            quantile(samples, 0.5).unwrap(),
            quantile(samples, 0.75).unwrap(),
        ]
    }

    fn bits<const N: usize>(values: [f64; N]) -> [u64; N] {
        values.map(f64::to_bits)
    }

    #[test]
    fn quantile_rejects_empty_sample_exactly() {
        let error = quantile(&[], 0.5).unwrap_err();

        assert_eq!(error.to_string(), "data error: quantile: samples is empty");
    }

    #[test]
    fn quantile_singleton_has_identical_quartiles() {
        assert_eq!(bits(quartiles(&[7.0])), bits([7.0, 7.0, 7.0]));
    }

    #[test]
    fn quantile_type_7_quartiles_are_exact_for_two_samples() {
        assert_eq!(bits(quartiles(&[0.0, 4.0])), bits([1.0, 2.0, 3.0]));
    }

    #[test]
    fn quantile_type_7_quartiles_are_exact_for_three_samples() {
        assert_eq!(bits(quartiles(&[0.0, 2.0, 4.0])), bits([1.0, 2.0, 3.0]));
    }

    #[test]
    fn quantile_type_7_quartiles_are_exact_for_four_samples() {
        assert_eq!(
            bits(quartiles(&[0.0, 2.0, 4.0, 6.0])),
            bits([1.5, 3.0, 4.5])
        );
    }

    #[test]
    fn quantile_type_7_quartiles_are_exact_for_five_samples() {
        assert_eq!(
            bits(quartiles(&[0.0, 2.0, 4.0, 6.0, 8.0])),
            bits([2.0, 4.0, 6.0])
        );
    }

    #[test]
    fn quantile_type_7_quartiles_are_exact_for_six_samples() {
        assert_eq!(
            bits(quartiles(&[0.0, 2.0, 4.0, 6.0, 8.0, 10.0])),
            bits([2.5, 5.0, 7.5])
        );
    }

    #[test]
    fn quantile_type_7_handles_repeated_values_exactly() {
        assert_eq!(
            bits(quartiles(&[1.0, 1.0, 1.0, 9.0])),
            bits([1.0, 1.0, 3.0])
        );
    }

    #[test]
    fn quantile_sorts_unsorted_input_deterministically() {
        assert_eq!(
            bits(quartiles(&[6.0, 0.0, 4.0, 2.0])),
            bits([1.5, 3.0, 4.5])
        );
    }

    #[test]
    fn quantile_returns_exact_validated_endpoints() {
        let samples = [3.0, -2.0, 8.0, 1.0];

        assert_eq!(
            bits([
                quantile(&samples, 0.0).unwrap(),
                quantile(&samples, 1.0).unwrap()
            ]),
            bits([-2.0, 8.0])
        );
    }

    #[test]
    fn quantile_type_7_interpolates_opposite_extreme_finite_values_without_overflow() {
        let samples = [-f64::MAX, f64::MAX];

        assert_eq!(
            bits([
                quantile(&samples, 0.25).unwrap(),
                quantile(&samples, 0.5).unwrap(),
                quantile(&samples, 0.75).unwrap(),
            ]),
            bits([-f64::MAX / 2.0, 0.0, f64::MAX / 2.0])
        );
    }

    #[test]
    fn quantile_type_7_interpolates_same_sign_extremes_without_overflow() {
        let lower = f64::MAX / 2.0;

        assert_eq!(
            quantile(&[lower, f64::MAX], 0.5).unwrap().to_bits(),
            (lower + (f64::MAX - lower) * 0.5).to_bits()
        );
    }

    #[test]
    fn quantile_rejects_non_finite_and_out_of_range_q_exactly() {
        for q in [f64::NAN, f64::NEG_INFINITY, -0.1, 1.1, f64::INFINITY] {
            let error = quantile(&[1.0], q).unwrap_err();

            assert_eq!(
                error.to_string(),
                format!("data error: quantile: q must be finite and in [0, 1], got {q}")
            );
        }
    }

    #[test]
    fn quantile_rejects_each_non_finite_sample_exactly() {
        for value in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
            let error = quantile(&[1.0, value, 3.0], 0.5).unwrap_err();

            assert_eq!(
                error.to_string(),
                format!("data error: quantile: sample at index 1 must be finite, got {value}")
            );
        }
    }

    #[test]
    fn standard_normal_quantile_matches_known_values() {
        let lower = standard_normal_quantile(0.025).unwrap();
        let median = standard_normal_quantile(0.5).unwrap();
        let upper = standard_normal_quantile(0.975).unwrap();

        assert!((lower + 1.959_963_984_540_054).abs() < 1e-7);
        assert!(median.abs() < 1e-12);
        assert!((upper - 1.959_963_984_540_054).abs() < 1e-7);
    }

    #[test]
    fn standard_normal_quantile_handles_extreme_open_probabilities() {
        let lower = standard_normal_quantile(f64::MIN_POSITIVE).unwrap();
        let upper = standard_normal_quantile(1.0 - f64::EPSILON).unwrap();

        assert!(lower.is_finite() && lower < 0.0);
        assert!(upper.is_finite() && upper > 0.0);
    }

    #[test]
    fn standard_normal_quantile_matches_central_tail_and_subnormal_references() {
        let references = [
            (0.1, -1.281_551_565_544_600_4),
            (0.9, 1.281_551_565_544_600_4),
            (1e-10, -6.361_340_902_404_056),
            (1e-50, -14.933_337_534_788_487),
            (f64::MIN_POSITIVE, -37.519_379_347_144_5),
            (f64::from_bits(1), -38.467_405_617_144_344),
        ];

        for (probability, expected) in references {
            let actual = standard_normal_quantile(probability).unwrap();
            assert!(
                (actual - expected).abs() < 1e-7,
                "p={probability}: {actual}"
            );
        }
    }

    #[test]
    fn standard_normal_quantile_rejects_closed_boundaries() {
        for invalid in [
            -1.0,
            0.0,
            1.0,
            2.0,
            f64::NAN,
            f64::NEG_INFINITY,
            f64::INFINITY,
        ] {
            assert!(standard_normal_quantile(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn standard_normal_quantile_is_continuous_at_branch_transitions() {
        for (probability, expected) in [
            (0.024_25_f64, -1.972_961_051_311_885),
            (1.0 - 0.024_25, 1.972_961_051_311_885),
        ] {
            let below = standard_normal_quantile(probability.next_down()).unwrap();
            let at = standard_normal_quantile(probability).unwrap();
            let above = standard_normal_quantile(probability.next_up()).unwrap();

            assert!((above - below).abs() < 1e-7);
            assert!((at - expected).abs() < 1e-7);
        }
    }

    #[test]
    fn standard_normal_quantile_preserves_representable_symmetry() {
        for probability in [0.1, 0.024_25, 1e-10] {
            let lower = standard_normal_quantile(probability).unwrap();
            let upper = standard_normal_quantile(1.0 - probability).unwrap();

            assert!((lower + upper).abs() < 1e-7, "p={probability}");
        }
    }

    #[test]
    fn standard_normal_quantile_matches_extreme_representable_upper_tail() {
        let probability = 1.0_f64.next_down();
        let actual = standard_normal_quantile(probability).unwrap();

        assert!((actual - 8.209_536_151_601_387).abs() < 1e-7, "{actual}");
    }
}
