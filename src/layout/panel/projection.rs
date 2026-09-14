use super::{AxisSide, Projection};
use crate::layout::PanelBox;

#[cfg(test)]
thread_local! {
    static PROJECT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl Projection {
    #[cfg(test)]
    pub(crate) fn test_projection_calls() -> usize {
        PROJECT_CALLS.with(std::cell::Cell::get)
    }
    /// Construct a new `Projection` for arbitrary data ranges
    /// and a custom pixel box. Used by plot renderers (e.g.
    /// `marginal`) that need to control the panel's exact pixel
    /// placement on the canvas.
    #[must_use]
    pub fn new(x_range: (f64, f64), y_range: (f64, f64), pixel_box: (f64, f64, f64, f64)) -> Self {
        Self {
            x_range,
            y_range,
            pixel_box,
            category_counts: [None; 2],
        }
    }

    pub(crate) fn with_categories(mut self, counts: [Option<usize>; 2]) -> Self {
        self.category_counts = counts;
        self
    }

    pub(crate) fn category_counts(&self) -> [Option<usize>; 2] {
        self.category_counts
    }

    /// Project a single data point to pixel coordinates.
    ///
    /// Returns `(px, py)` in math-style canvas space: data `y_max` maps
    /// to `pixel_box.y1` (top of the plot area), and data `y_min` maps
    /// to `pixel_box.y0` (bottom). Backends perform any native coordinate
    /// conversion at the final drawing boundary.
    pub fn project(&self, x: f64, y: f64) -> (f64, f64) {
        #[cfg(test)]
        PROJECT_CALLS.with(|calls| calls.set(calls.get() + 1));
        let (x_min, x_max) = self.x_range;
        let (y_min, y_max) = self.y_range;
        let (x0, x1, y0, y1) = self.pixel_box;

        let px = project_axis(x, (x_min, x_max), x0, x1, self.category_counts[0]);
        let py = project_axis(y, (y_min, y_max), y0, y1, self.category_counts[1]);
        (px, py)
    }

    /// Preserve a required scientific extent before retaining quantized operands.
    /// `nonzero` comes from the scientific aggregate, before cumulative addition.
    #[expect(
        clippy::float_cmp,
        reason = "exact projected endpoint identity, not a tolerance"
    )]
    pub(crate) fn validate_span(
        &self,
        axis: AxisSide,
        source: (f64, f64),
        projected: (f64, f64),
        nonzero: bool,
        context: &str,
    ) -> Result<(), crate::Error> {
        let range = match axis {
            AxisSide::Bottom => self.x_range,
            AxisSide::Left => self.y_range,
        };
        // Wholly off-domain spans are not quantized by project_axis. Keep their
        // existing backend validation, without treating clipping as degeneracy.
        if nonzero
            && source.0.min(source.1) <= range.1
            && source.0.max(source.1) >= range.0
            && projected.0.is_finite()
            && projected.0 == projected.1
        {
            return Err(crate::Error::Render(format!(
                "{context}: nonzero span endpoints collapse at 1/1024 CSS-pixel coordinate precision"
            )));
        }
        Ok(())
    }

    /// The data range of the x-axis.
    #[must_use]
    pub fn x_range(&self) -> (f64, f64) {
        self.x_range
    }

    /// The data range of the y-axis.
    #[must_use]
    pub fn y_range(&self) -> (f64, f64) {
        self.y_range
    }

    /// The pixel bounding box of the plot area.
    #[must_use]
    pub fn pixel_box(&self) -> (f64, f64, f64, f64) {
        self.pixel_box
    }

    /// The resolved plot panel in math-style canvas coordinates.
    #[must_use]
    pub fn panel_box(&self) -> PanelBox {
        let (x0, x1, y0, y1) = self.pixel_box;
        PanelBox::new(x0, x1, y0, y1)
    }

    /// Whether a data-space point lies within both resolved axis ranges.
    #[must_use]
    pub fn contains_data(&self, x: f64, y: f64) -> bool {
        x.is_finite()
            && y.is_finite()
            && x >= self.x_range.0
            && x <= self.x_range.1
            && y >= self.y_range.0
            && y <= self.y_range.1
    }
}

#[allow(clippy::float_cmp, reason = "exact domain endpoint identity")]
fn fraction(value: f64, range: (f64, f64)) -> f64 {
    if !value.is_finite() || !range.0.is_finite() || !range.1.is_finite() || range.0 >= range.1 {
        f64::NAN
    } else if value == range.0 {
        0.0
    } else if value == range.1 {
        1.0
    } else {
        let difference = range.1 - range.0;
        let offset = value - range.0;
        if difference.is_finite() && offset.is_finite() {
            offset / difference
        } else {
            let scale = value.abs().max(range.0.abs()).max(range.1.abs());
            ((value / scale) - (range.0 / scale)) / ((range.1 / scale) - (range.0 / scale))
        }
    }
}

#[allow(
    clippy::float_cmp,
    reason = "exact semantic band-edge and midpoint identity"
)]
fn project_axis(
    value: f64,
    range: (f64, f64),
    start: f64,
    end: f64,
    categories: Option<usize>,
) -> f64 {
    // Clipped marks still need extrapolated coordinates outside the data domain.
    if value < range.0 || value > range.1 {
        return start + fraction(value, range) * (end - start);
    }
    let position = (|| {
        let key = projection_key(value, range)?;
        let (origin, span) = axis_quanta(start, end).ok()?;
        if let Some(count) = categories {
            let denominator = (count as u128).checked_mul(2)?;
            let half_index = (fraction(value, range) * denominator as f64).round_ties_even();
            // Multiply before dividing: canonical category inputs i/2 in [0,n]
            // then reconstruct exactly. Jittered positions are not snapped to bands.
            let band_position = range.0 + half_index * (range.1 - range.0) / denominator as f64;
            if value == band_position {
                return ratio_position(origin, span, half_index as u128, denominator);
            }
        }
        ratio_position(origin, span, u128::from(key), u128::from(u64::MAX))
    })();
    // This public infallible API propagates invalid geometry to backend validation.
    position.map_or(f64::NAN, |position| position as f64 / 1024.0)
}

pub(super) fn axis_quanta(start: f64, end: f64) -> Result<(i64, i64), crate::Error> {
    let origin = crate::layout::kernel::nearest_spatial_quanta(start)?;
    let end = crate::layout::kernel::nearest_spatial_quanta(end)?;
    let span = end
        .checked_sub(origin)
        .filter(|span| *span >= 0)
        .ok_or_else(|| crate::Error::Render("axis span overflow or reversed edges".into()))?;
    Ok((origin, span))
}

#[allow(clippy::float_cmp, reason = "exact unit fraction endpoint")]
pub(super) fn projection_key(value: f64, range: (f64, f64)) -> Option<u64> {
    let fraction = fraction(value, range);
    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
        return None;
    }
    if fraction == 1.0 {
        return Some(u64::MAX);
    }
    let bits = fraction.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | if exponent == 0 { 0 } else { 1_u64 << 52 };
    let shift = (1075 - exponent.max(1)) as u32;
    let numerator = u128::from(mantissa) * u128::from(u64::MAX);
    if shift >= 128 {
        return Some(0);
    }
    let quotient = numerator >> shift;
    let remainder = numerator & ((1_u128 << shift) - 1);
    let halfway = 1_u128 << (shift - 1);
    u64::try_from(
        quotient
            + u128::from(
                remainder > halfway || (remainder == halfway && !quotient.is_multiple_of(2)),
            ),
    )
    .ok()
}

/// Round the complete position, including the origin's parity at exact ties.
pub(super) fn ratio_position(
    origin: i64,
    span: i64,
    numerator: u128,
    denominator: u128,
) -> Option<i64> {
    let product = u128::try_from(span).ok()?.checked_mul(numerator)?;
    let quotient = product.checked_div(denominator)?;
    let remainder = product % denominator;
    let position = i128::from(origin).checked_add(i128::try_from(quotient).ok()?)?;
    let complement = denominator - remainder;
    i64::try_from(
        position
            + i128::from(remainder > complement || (remainder == complement && position % 2 != 0)),
    )
    .ok()
}
