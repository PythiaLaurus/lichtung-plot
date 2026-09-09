use super::{AxisSide, AxisSpec, AxisTick, Projection};
use crate::layout::PhysicalTextAlignment;
use crate::theme::ResolvedTheme;
use crate::Error;

const DEFAULT_RANGE_MARGIN_FRACTION: f64 = 0.05;
const NICE_STEPS: [f64; 5] = [10.0, 5.0, 2.5, 2.0, 1.0];
pub(crate) const AUTOMATIC_TICK_TARGET: usize = 5;

pub(super) fn resolve_range(range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    if let Some(limits) = spec.limits {
        limits
    } else {
        continuous_auto_range(range, spec)
    }
}

/// Compute an auto display range for continuous marks.
pub(crate) fn continuous_auto_range(range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    if let Some(limits) = spec.limits {
        limits
    } else {
        broaden_nice_range(range, AUTOMATIC_TICK_TARGET, DEFAULT_RANGE_MARGIN_FRACTION)
    }
}

/// Compute an auto display range anchored at a meaningful baseline.
pub(crate) fn anchored_auto_range(range: (f64, f64), spec: &AxisSpec, anchor: f64) -> (f64, f64) {
    if let Some(limits) = spec.limits {
        return limits;
    }
    if !(range.0.is_finite() && range.1.is_finite() && anchor.is_finite()) {
        return range;
    }
    let lo = range.0.min(range.1);
    let hi = range.0.max(range.1);
    if lo.total_cmp(&hi).is_eq() {
        if hi.total_cmp(&anchor).is_eq() {
            return if anchor == 0.0 {
                (0.0, 1.0)
            } else if hi >= anchor {
                (anchor, pad_up(hi, relative_padding(anchor, hi)))
            } else {
                (pad_down(lo, relative_padding(lo, anchor)), anchor)
            };
        }
        return if hi >= anchor {
            (anchor, pad_up(hi, relative_padding(anchor, hi)))
        } else {
            (pad_down(lo, relative_padding(lo, anchor)), anchor)
        };
    }

    let (nice_min, nice_max) = nice_range(lo, hi, AUTOMATIC_TICK_TARGET);
    let pad = relative_padding(nice_min, nice_max);
    if lo >= anchor {
        (anchor, pad_up(nice_max.max(hi), pad))
    } else if hi <= anchor {
        (pad_down(nice_min.min(lo), pad), anchor)
    } else {
        (
            pad_down(nice_min.min(lo), pad),
            pad_up(nice_max.max(hi), pad),
        )
    }
}

fn relative_padding(minimum: f64, maximum: f64) -> f64 {
    let direct = maximum - minimum;
    if direct.is_finite() && direct > 0.0 {
        return direct * DEFAULT_RANGE_MARGIN_FRACTION;
    }
    let scale = minimum.abs().max(maximum.abs()).max(f64::MIN_POSITIVE);
    ((maximum / scale - minimum / scale).abs() * DEFAULT_RANGE_MARGIN_FRACTION) * scale
}

fn pad_up(value: f64, padding: f64) -> f64 {
    let padded = value + padding;
    if padded.is_finite() && padded > value {
        padded
    } else {
        let adjacent = value.next_up();
        if adjacent.is_finite() {
            adjacent
        } else {
            value
        }
    }
}

fn pad_down(value: f64, padding: f64) -> f64 {
    let padded = value - padding;
    if padded.is_finite() && padded < value {
        padded
    } else {
        let adjacent = value.next_down();
        if adjacent.is_finite() {
            adjacent
        } else {
            value
        }
    }
}

/// Return a spec that preserves an already-computed display range.
pub(crate) fn display_axis_spec(spec: &AxisSpec, range: (f64, f64)) -> AxisSpec {
    if spec.limits.is_some() {
        spec.clone()
    } else {
        spec.clone().limits(range.0, range.1)
    }
}

fn broaden_nice_range(range: (f64, f64), target_ticks: usize, margin_fraction: f64) -> (f64, f64) {
    if !(range.0.is_finite() && range.1.is_finite()) {
        return range;
    }
    let lo = range.0.min(range.1);
    let hi = range.0.max(range.1);
    let (nice_min, nice_max) = nice_range(lo, hi, target_ticks);
    let broadened = if lo.total_cmp(&hi).is_eq() {
        let pad = (nice_max - nice_min).abs() * margin_fraction;
        (nice_min - pad, nice_max + pad)
    } else {
        let margin =
            ((nice_max - nice_min).abs() * margin_fraction).max((hi - lo) * margin_fraction);
        (nice_min - margin, nice_max + margin)
    };
    if broadened.0.is_finite() && broadened.1.is_finite() && broadened.0 < broadened.1 {
        broadened
    } else {
        (lo, hi)
    }
}

pub(crate) fn automatic_numeric_ticks(
    range: (f64, f64),
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    automatic_numeric_ticks_with_policy(range, projection, side, theme, false)
}

pub(crate) fn automatic_numeric_ticks_legacy(
    range: (f64, f64),
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    automatic_numeric_ticks_with_policy(range, projection, side, theme, true)
}

fn automatic_numeric_ticks_with_policy(
    range: (f64, f64),
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<Vec<AxisTick>, Error> {
    let mut best: Option<Vec<AxisTick>> = None;

    for target in (1..=AUTOMATIC_TICK_TARGET).rev() {
        let positions = nice_ticks(range.0, range.1, target);
        if positions.len() < 2 {
            continue;
        }
        let labels = distinct_tick_labels(&positions);
        let ticks = positions
            .into_iter()
            .zip(labels)
            .map(|(position, label)| AxisTick::new(position, label))
            .collect::<Vec<_>>();
        if ticks.len() > AUTOMATIC_TICK_TARGET {
            continue;
        }
        if !automatic_tick_labels_fit(&ticks, projection, side, theme, legacy)? {
            continue;
        }
        if best
            .as_ref()
            .is_none_or(|current| ticks.len() > current.len())
        {
            best = Some(ticks);
        }
    }

    Ok(best.unwrap_or_else(|| {
        let midpoint = finite_midpoint(range);
        vec![AxisTick::new(midpoint, format_tick(midpoint))]
    }))
}

pub(crate) fn fit_automatic_tick_candidates(
    ticks: Vec<AxisTick>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    fallback: AxisTick,
) -> Result<Vec<AxisTick>, Error> {
    if !ticks.is_empty()
        && ticks.len() <= AUTOMATIC_TICK_TARGET
        && automatic_tick_labels_fit(&ticks, projection, side, theme, false)?
    {
        return Ok(ticks);
    }

    for desired in (2..=ticks.len().min(AUTOMATIC_TICK_TARGET)).rev() {
        let last = ticks.len() - 1;
        let denominator = desired - 1;
        let candidate = (0..desired)
            .map(|index| {
                let numerator = index * last;
                let source = (numerator + denominator / 2) / denominator;
                ticks[source].clone()
            })
            .collect::<Vec<_>>();
        if automatic_tick_labels_fit(&candidate, projection, side, theme, false)? {
            return Ok(candidate);
        }
    }

    Ok(vec![fallback])
}

#[cfg(test)]
pub(crate) fn retain_automatic_ticks_inside_panel(
    ticks: Vec<AxisTick>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    retain_automatic_ticks_inside_panel_with_policy(ticks, projection, side, theme, false)
}

pub(crate) fn retain_automatic_ticks_inside_panel_legacy(
    ticks: Vec<AxisTick>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    retain_automatic_ticks_inside_panel_with_policy(ticks, projection, side, theme, true)
}

fn retain_automatic_ticks_inside_panel_with_policy(
    ticks: Vec<AxisTick>,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<Vec<AxisTick>, Error> {
    let (x0, x1, y0, y1) = projection.pixel_box();
    let bounds = match side {
        AxisSide::Bottom => (x0, x1),
        AxisSide::Left => (y0, y1),
    };
    let mut retained = Vec::new();
    for tick in ticks {
        let (start, end) = projected_tick_extent(&tick, projection, side, theme, legacy)?;
        if start >= bounds.0 && end <= bounds.1 {
            retained.push(tick);
        }
    }
    if !retained.is_empty() {
        return Ok(retained);
    }

    let range = match side {
        AxisSide::Bottom => projection.x_range(),
        AxisSide::Left => projection.y_range(),
    };
    let midpoint = finite_midpoint(range);
    let fallback = AxisTick::new(midpoint, format_tick(midpoint));
    let (start, end) = projected_tick_extent(&fallback, projection, side, theme, legacy)?;
    Ok((start >= bounds.0 && end <= bounds.1)
        .then_some(fallback)
        .into_iter()
        .collect())
}

fn automatic_tick_labels_fit(
    ticks: &[AxisTick],
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<bool, Error> {
    let mut previous_end = None;
    let (x0, x1, _, _) = projection.pixel_box();
    let maximum_width = match side {
        AxisSide::Bottom => Some((x1 - x0) / ticks.len().max(1) as f64 * 0.9),
        AxisSide::Left => None,
    };

    for tick in ticks {
        let (_, _, fit_width) = tick_text_size(theme, &tick.label, legacy)?;
        if maximum_width.is_some_and(|width| fit_width > width) {
            return Ok(false);
        }
        let (start, end) = projected_tick_extent(tick, projection, side, theme, legacy)?;
        let fits = previous_end.is_none_or(|previous| start >= previous);
        previous_end = Some(end);
        if !fits {
            return Ok(false);
        }
    }
    Ok(true)
}

fn projected_tick_extent(
    tick: &AxisTick,
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
    legacy: bool,
) -> Result<(f64, f64), Error> {
    let center = match side {
        AxisSide::Bottom => projection.project(tick.position, projection.y_range().0).0,
        AxisSide::Left => projection.project(projection.x_range().0, tick.position).1,
    }
    .round();
    let (width, height, _) = tick_text_size(theme, &tick.label, legacy)?;
    let half_extent = match side {
        AxisSide::Bottom => width / 2.0,
        AxisSide::Left => height / 2.0,
    };
    let start = center - half_extent;
    let end = center + half_extent;
    if start.is_finite() && end.is_finite() {
        Ok((start, end))
    } else {
        Err(Error::Render(
            "tick extent projection arithmetic overflowed".into(),
        ))
    }
}

fn tick_text_size(
    theme: &ResolvedTheme,
    label: &str,
    legacy: bool,
) -> Result<(f64, f64, f64), Error> {
    let plan = if legacy {
        theme.measure_legacy_appearance_text(
            label,
            &theme.typography.tick_label,
            PhysicalTextAlignment::Center,
            "legacy automatic tick label",
            false,
        )
    } else {
        theme.measure_text(
            label,
            &theme.typography.tick_label,
            PhysicalTextAlignment::Center,
            "automatic tick label",
            false,
        )
    }?;
    Ok((plan.width(), plan.height(), plan.fit_width()))
}

fn finite_midpoint(range: (f64, f64)) -> f64 {
    let midpoint = f64::midpoint(range.0, range.1);
    if midpoint.is_finite() {
        midpoint
    } else if range.0.is_finite() {
        range.0
    } else if range.1.is_finite() {
        range.1
    } else {
        0.0
    }
}

pub(super) fn distinct_tick_labels(positions: &[f64]) -> Vec<String> {
    let mut labels = positions
        .iter()
        .copied()
        .map(format_tick)
        .collect::<Vec<_>>();
    if labels_are_unique(&labels) {
        return labels;
    }
    for precision in 2..=17 {
        labels = positions
            .iter()
            .map(|value| format!("{value:.precision$e}"))
            .collect();
        if labels_are_unique(&labels) {
            break;
        }
    }
    labels
}

pub(super) fn labels_are_unique(labels: &[String]) -> bool {
    labels
        .iter()
        .enumerate()
        .all(|(index, label)| !labels[..index].contains(label))
}

/// Compute a "nice" axis range enclosing `[lo, hi]` by snapping
/// outward to multiples of the best tick step.
///
/// The step is chosen from the canonical set {1, 2, 2.5, 5, 10} × 10^k
/// to produce a tick count closest to `target_ticks` intervals.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn nice_range(lo: f64, hi: f64, target_ticks: usize) -> (f64, f64) {
    if !(lo.is_finite() && hi.is_finite()) || lo > hi || target_ticks == 0 {
        return (lo, hi);
    }
    if lo.total_cmp(&hi).is_eq() {
        let broad = (lo - 1.0, hi + 1.0);
        if broad.0.is_finite() && broad.1.is_finite() && broad.0 < broad.1 {
            return broad;
        }
        return finite_adjacent_range(lo);
    }
    let step = best_tick_step(lo, hi, target_ticks);
    if !step.is_finite() || step <= 0.0 {
        return (lo, hi);
    }
    let nice_lo = (lo / step).floor() * step;
    let nice_hi = (hi / step).ceil() * step;
    if nice_lo.is_finite()
        && nice_hi.is_finite()
        && nice_lo <= lo
        && nice_hi >= hi
        && nice_lo < nice_hi
    {
        (nice_lo, nice_hi)
    } else {
        (lo, hi)
    }
}

fn finite_adjacent_range(value: f64) -> (f64, f64) {
    let lower = value.next_down();
    let upper = value.next_up();
    if lower.is_finite() && lower < value {
        if upper.is_finite() && value < upper {
            (lower, upper)
        } else {
            (lower, value)
        }
    } else if upper.is_finite() && value < upper {
        (value, upper)
    } else {
        (value, value)
    }
}

/// Find the best tick step from the canonical nice-number set
/// {1, 2, 2.5, 5, 10} × 10^mag that produces tick count closest to
/// `target_ticks` intervals (i.e. ≈ target_ticks+1 tick marks).
///
/// Mirrors matplotlib `MaxNLocator` logic: among all candidate steps,
/// choose the one whose resulting tick count is closest to the target,
/// with ties broken in favor of larger steps (fewer, cleaner ticks).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn best_tick_step(lo: f64, hi: f64, target_ticks: usize) -> f64 {
    let range = finite_span(lo, hi);
    if !range.is_finite() || range <= 0.0 || target_ticks == 0 {
        return f64::NAN;
    }
    let raw_step = range / target_ticks as f64;
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return f64::NAN;
    }
    let mag = 10f64.powf(raw_step.log10().floor());
    if !mag.is_finite() || mag <= 0.0 {
        return raw_step;
    }
    let target_n = target_ticks.saturating_add(1); // desired number of tick marks

    let mut best_step = raw_step; // fallback
    let mut best_dist = usize::MAX;

    for &n in &NICE_STEPS {
        let step = n * mag;
        let Some(nticks) = snapped_tick_count(lo, hi, step) else {
            continue;
        };
        let dist = nticks.abs_diff(target_n);
        // Prefer smaller distance; on tie prefer larger step (scanned first)
        if dist < best_dist {
            best_dist = dist;
            best_step = step;
        }
    }
    best_step
}

/// Generate ticks within `[lo, hi]`.
///
/// Selects the nice step that places the closest number of ticks
/// to `k` **inside** the given range (no boundary snap — just
/// ceil(lo/step) to floor(hi/step)).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn nice_ticks(lo: f64, hi: f64, k: usize) -> Vec<f64> {
    if !(lo.is_finite() && hi.is_finite()) || lo >= hi || k == 0 {
        return Vec::new();
    }
    if lo.next_up() >= hi {
        return vec![lo, hi];
    }
    let range = finite_span(lo, hi);
    if !range.is_finite() || range <= 0.0 {
        return vec![lo, hi];
    }
    let raw_step = range / k as f64;
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return vec![lo, hi];
    }
    let mag = 10f64.powf(raw_step.log10().floor());
    if !mag.is_finite() || mag <= 0.0 {
        return vec![lo, hi];
    }
    let target_n = k; // desired number of tick marks inside range

    let mut best_step = raw_step;
    let mut best_dist = usize::MAX;

    for &n in &NICE_STEPS {
        let step = n * mag;
        let Some(nticks) = tick_count(lo, hi, step) else {
            continue;
        };
        let dist = nticks.abs_diff(target_n);
        if dist < best_dist {
            best_dist = dist;
            best_step = step;
        }
    }

    let start = (lo / best_step).ceil() * best_step;
    let Some(count) = tick_count(lo, hi, best_step) else {
        return vec![lo, hi];
    };
    let mut out = Vec::with_capacity(count);
    let mut tick = start;
    let mut stalled = false;
    for _ in 0..count {
        if tick.is_finite()
            && tick >= lo
            && tick <= hi + best_step * 1e-9
            && out.last().is_none_or(|previous| *previous < tick)
        {
            out.push(tick);
        }
        let next = tick + best_step;
        if next <= tick {
            stalled = true;
            break;
        }
        tick = next;
    }
    if stalled || out.len() < 2 {
        vec![lo, hi]
    } else {
        out
    }
}

fn finite_span(lo: f64, hi: f64) -> f64 {
    let direct = hi - lo;
    if direct.is_finite() {
        return direct;
    }
    let scale = lo.abs().max(hi.abs());
    (hi / scale - lo / scale) * scale
}

fn tick_count(lo: f64, hi: f64, step: f64) -> Option<usize> {
    const MAX_TICKS: usize = 10_000;
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    let first = (lo / step).ceil();
    let last = (hi / step).floor();
    if !first.is_finite() || !last.is_finite() || last < first {
        return Some(0);
    }
    let count = last - first + 1.0;
    if !count.is_finite() || count > MAX_TICKS as f64 {
        None
    } else {
        Some(count.max(0.0) as usize)
    }
}

fn snapped_tick_count(lo: f64, hi: f64, step: f64) -> Option<usize> {
    let snapped_lo = (lo / step).floor() * step;
    let snapped_hi = (hi / step).ceil() * step;
    let intervals = (snapped_hi - snapped_lo) / step;
    if snapped_lo.is_finite()
        && snapped_hi.is_finite()
        && intervals.is_finite()
        && intervals >= 0.0
        && intervals < usize::MAX as f64
    {
        Some(intervals.round() as usize + 1)
    } else {
        None
    }
}

/// Format a tick value smartly: no trailing zeros, no -0.0, SI prefixes for large numbers.
pub(crate) fn format_tick(value: f64) -> String {
    // Eliminate -0.0
    let v = if value == 0.0 { 0.0 } else { value };

    let abs = v.abs();
    if abs >= 1.0e12 {
        format!("{v:.2e}")
    } else if abs >= 1_000_000.0 {
        let m = v / 1_000_000.0;
        format_clean(m, "M")
    } else if abs >= 10_000.0 {
        let k = v / 1_000.0;
        format_clean(k, "k")
    } else if abs < 0.01 && abs > 0.0 {
        // Scientific-ish for very small values
        format!("{v:.2e}")
    } else {
        // Auto-select decimal places based on magnitude
        let s = if abs >= 100.0 {
            format!("{v:.0}")
        } else if abs >= 1.0 {
            format!("{v:.1}")
        } else {
            format!("{v:.2}")
        };
        // Strip trailing zeros after decimal point
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            s
        }
    }
}

fn format_clean(v: f64, suffix: &str) -> String {
    let s = format!("{v:.1}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}{suffix}")
}
