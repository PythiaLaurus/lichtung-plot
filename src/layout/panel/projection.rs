use super::Projection;
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
        }
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

        let tx = scaled_fraction(x, x_min, x_max);
        let ty = scaled_fraction(y, y_min, y_max);

        let px = x0 + tx * (x1 - x0);
        let py = y0 + ty * (y1 - y0);
        (px, py)
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

fn scaled_fraction(value: f64, minimum: f64, maximum: f64) -> f64 {
    let scale = value
        .abs()
        .max(minimum.abs())
        .max(maximum.abs())
        .max(f64::MIN_POSITIVE);
    let scaled_value = value / scale;
    let scaled_minimum = minimum / scale;
    let scaled_maximum = maximum / scale;
    (scaled_value - scaled_minimum) / (scaled_maximum - scaled_minimum)
}
