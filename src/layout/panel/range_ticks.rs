use super::AxisSpec;
#[cfg(test)]
use super::{AxisSide, AxisTick, Projection};
#[cfg(test)]
use crate::theme::ResolvedTheme;
#[cfg(test)]
use crate::Error;

const DEFAULT_RANGE_MARGIN_FRACTION: f64 = 0.05;
const NICE_RANGE_SEED_COUNT: usize = 5;
const NICE_STEPS: [f64; 5] = [10.0, 5.0, 2.5, 2.0, 1.0];

pub(super) fn resolve_range(range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    continuous_auto_range(range, spec)
}

pub(crate) fn continuous_auto_range(range: (f64, f64), spec: &AxisSpec) -> (f64, f64) {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisRangePreparation,
    );
    if let Some(limits) = spec.limits {
        limits
    } else {
        broaden_nice_range(range, NICE_RANGE_SEED_COUNT, DEFAULT_RANGE_MARGIN_FRACTION)
    }
}

pub(crate) fn anchored_auto_range(range: (f64, f64), spec: &AxisSpec, anchor: f64) -> (f64, f64) {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisRangePreparation,
    );
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
    let (nice_min, nice_max) = nice_range(lo, hi, NICE_RANGE_SEED_COUNT);
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

pub(crate) fn display_axis_spec(spec: &AxisSpec, range: (f64, f64)) -> AxisSpec {
    #[cfg(test)]
    crate::orchestrator::measurements::count(
        crate::orchestrator::measurements::Counter::AxisSpecPreparation,
    );
    if spec.limits.is_some() {
        spec.clone()
    } else {
        let mut display = spec.clone();
        display.limits = Some(range);
        display
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

#[cfg(test)]
pub(crate) fn automatic_numeric_ticks(
    range: (f64, f64),
    projection: &Projection,
    side: AxisSide,
    theme: &ResolvedTheme,
) -> Result<Vec<AxisTick>, Error> {
    Ok(super::sources::numeric_tick_plan(
        range,
        None,
        false,
        projection,
        side,
        theme,
        false,
        &crate::layout::fit::ROOT_SCOPE,
    )
    .map_err(crate::layout::fit::PlanError::into_error)?
    .ticks)
}

#[cfg(test)]
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

#[cfg(test)]
pub(super) fn labels_are_unique(labels: &[String]) -> bool {
    labels
        .iter()
        .enumerate()
        .all(|(index, label)| !labels[..index].contains(label))
}

/// The range owner's snapping grammar is deliberately independent of tick sources.
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

fn best_tick_step(lo: f64, hi: f64, target_ticks: usize) -> f64 {
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
    let target_n = target_ticks.saturating_add(1);
    let mut best_step = raw_step;
    let mut best_dist = usize::MAX;
    for &n in &NICE_STEPS {
        let step = n * mag;
        let Some(nticks) = snapped_tick_count(lo, hi, step) else {
            continue;
        };
        let dist = nticks.abs_diff(target_n);
        if dist < best_dist {
            best_dist = dist;
            best_step = step;
        }
    }
    best_step
}

#[cfg(test)]
pub(crate) fn nice_ticks(lo: f64, hi: f64, seed: usize) -> Vec<f64> {
    if !(lo.is_finite() && hi.is_finite()) || lo >= hi || seed == 0 {
        return Vec::new();
    }
    super::sources::adaptive_positions((lo, hi), seed, None)
        .unwrap()
        .unwrap_or_default()
}

fn finite_span(lo: f64, hi: f64) -> f64 {
    let direct = hi - lo;
    if direct.is_finite() {
        return direct;
    }
    let scale = lo.abs().max(hi.abs());
    (hi / scale - lo / scale) * scale
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

#[cfg(test)]
pub(crate) fn format_tick(value: f64) -> String {
    super::sources::format_label(value, false, false, 0, None).unwrap()
}
