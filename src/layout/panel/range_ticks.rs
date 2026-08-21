use super::{AxisSpec, AxisTick};

const DEFAULT_RANGE_MARGIN_FRACTION: f64 = 0.05;
const NICE_STEPS: [f64; 5] = [10.0, 5.0, 2.5, 2.0, 1.0];

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
        broaden_nice_range(range, spec.tick_count, DEFAULT_RANGE_MARGIN_FRACTION)
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

    let (nice_min, nice_max) = nice_range(lo, hi, spec.tick_count);
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

pub(super) fn numeric_ticks(range: (f64, f64), spec: &AxisSpec) -> Vec<AxisTick> {
    let positions = nice_ticks(range.0, range.1, spec.tick_count);
    let labels = distinct_tick_labels(&positions);
    positions
        .into_iter()
        .zip(labels)
        .map(|(position, label)| AxisTick::new(position, label))
        .collect()
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
