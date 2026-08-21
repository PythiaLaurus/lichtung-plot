//! `lichtung-plot::render::layout` — FacetGrid subplot math.
//!
//! A `FacetGrid` divides the canvas into an `n_rows × n_cols`
//! grid of panels (subplots). Each panel has a fixed
//! `(x_offset, y_offset, width, height)` in pixel
//! coordinates, computed from the grid's total canvas
//! size and a per-panel margin.
//!
//! The current commit implements the layout math only
//! (no actual multi-panel rendering). The full
//! multi-panel rendering (where each panel renders
//! one plot kind) is deferred to a future commit;
//! for now, callers compute per-panel rectangles and
//! pass them to individual plot renderers (which
//! currently use a single-panel layout).
//!
//! # Example
//!
//! ```
//! use lichtung_plot::render::layout::FacetGrid;
//!
//! let canvas_w = 800.0;
//! let canvas_h = 600.0;
//! let grid = FacetGrid::new(canvas_w, canvas_h, 2, 3);
//! assert_eq!(grid.n_panels(), 6);
//! let panel = grid.panel(0, 0).unwrap();
//! // The first panel (top-left) starts at the top-left
//! // margin and has the standard 60/40/20/20 padding.
//! assert!(panel.x_offset > 0.0);
//! ```

/// `FacetGrid` — n_rows × n_cols panel layout.
///
/// All units are in pixels; the panel rectangles are
/// computed once at construction (they don't change
/// after).
#[derive(Debug, Clone)]
pub struct FacetGrid {
    /// Total canvas width in pixels.
    canvas_w: f64,
    /// Total canvas height in pixels.
    canvas_h: f64,
    /// Number of rows in the grid.
    n_rows: usize,
    /// Number of columns in the grid.
    n_cols: usize,
    /// Per-panel margins in pixels (left, right, top, bottom).
    margins: Margins,
    /// Per-panel rectangles, indexed by `(row, col)`.
    panels: Vec<Panel>,
}

/// Per-panel margins in pixels.
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    /// Left margin in pixels.
    pub left: f64,
    /// Right margin in pixels.
    pub right: f64,
    /// Top margin in pixels.
    pub top: f64,
    /// Bottom margin in pixels.
    pub bottom: f64,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            left: 60.0,
            right: 20.0,
            top: 20.0,
            bottom: 40.0,
        }
    }
}

/// A single panel's pixel rectangle.
#[derive(Debug, Clone, Copy)]
pub struct Panel {
    /// X offset from the canvas top-left (pixels).
    pub x_offset: f64,
    /// Y offset from the canvas top-left (pixels).
    pub y_offset: f64,
    /// Panel width in pixels.
    pub width: f64,
    /// Panel height in pixels.
    pub height: f64,
}

impl FacetGrid {
    /// Construct a new `FacetGrid` for the given canvas
    /// size and `n_rows × n_cols` layout. Uses the default
    /// per-panel margins (60/20/20/40 pixels).
    #[must_use]
    pub fn new(canvas_w: f64, canvas_h: f64, n_rows: usize, n_cols: usize) -> Self {
        Self::from_margins(canvas_w, canvas_h, n_rows, n_cols, Margins::default())
    }

    /// Construct a new `FacetGrid` with custom margins.
    ///
    /// The `col as f64` and `row as f64` casts are
    /// intentional: panel indices fit in f64 without
    /// loss (canvas dimensions are well below 2^53
    /// pixels). The lint allow on the function covers
    /// both the construction loop and the test-only
    /// assertions below.
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_margins(
        canvas_w: f64,
        canvas_h: f64,
        n_rows: usize,
        n_cols: usize,
        margins: Margins,
    ) -> Self {
        assert!(n_rows > 0, "n_rows must be > 0, got {n_rows}");
        assert!(n_cols > 0, "n_cols must be > 0, got {n_cols}");
        assert!(canvas_w > 0.0, "canvas_w must be > 0, got {canvas_w}");
        assert!(canvas_h > 0.0, "canvas_h must be > 0, got {canvas_h}");

        // Compute the available area after margins.
        let available_w = canvas_w - margins.left - margins.right;
        let available_h = canvas_h - margins.top - margins.bottom;
        assert!(
            available_w > 0.0 && available_h > 0.0,
            "canvas {canvas_w}x{canvas_h} is too small for the given margins"
        );
        let panel_w = available_w / n_cols as f64;
        let panel_h = available_h / n_rows as f64;

        // The `col as f64` and `row as f64` casts are
        // intentional: panel indices fit in f64
        // without loss (canvas dimensions are well
        // below 2^53 pixels).
        #[allow(clippy::cast_precision_loss)]
        let mut panels: Vec<Panel> = Vec::with_capacity(n_rows * n_cols);
        for row in 0..n_rows {
            for col in 0..n_cols {
                let x_offset = margins.left + col as f64 * panel_w;
                let y_offset = margins.top + row as f64 * panel_h;
                panels.push(Panel {
                    x_offset,
                    y_offset,
                    width: panel_w,
                    height: panel_h,
                });
            }
        }

        Self {
            canvas_w,
            canvas_h,
            n_rows,
            n_cols,
            margins,
            panels,
        }
    }

    /// Total number of panels (`n_rows * n_cols`).
    #[must_use]
    pub fn n_panels(&self) -> usize {
        self.panels.len()
    }

    /// The pixel rectangle for the panel at `(row, col)`.
    ///
    /// `row = 0` is the top row; `col = 0` is the left
    /// column. Returns `None` if the indices are out of
    /// range.
    #[must_use]
    pub fn panel(&self, row: usize, col: usize) -> Option<Panel> {
        if row >= self.n_rows || col >= self.n_cols {
            return None;
        }
        Some(self.panels[row * self.n_cols + col])
    }

    /// All panels in row-major order (top-to-bottom,
    /// left-to-right).
    #[must_use]
    pub fn panels(&self) -> &[Panel] {
        &self.panels
    }

    /// Canvas width in pixels.
    #[must_use]
    pub fn canvas_width(&self) -> f64 {
        self.canvas_w
    }

    /// Canvas height in pixels.
    #[must_use]
    pub fn canvas_height(&self) -> f64 {
        self.canvas_h
    }

    /// Number of rows in the grid.
    #[must_use]
    pub fn n_rows(&self) -> usize {
        self.n_rows
    }

    /// Number of columns in the grid.
    #[must_use]
    pub fn n_cols(&self) -> usize {
        self.n_cols
    }

    /// The per-panel margins.
    #[must_use]
    pub fn margins(&self) -> Margins {
        self.margins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} vs {b}");
    }

    #[test]
    fn single_panel_uses_full_canvas() {
        let grid = FacetGrid::new(800.0, 600.0, 1, 1);
        assert_eq!(grid.n_panels(), 1);
        let p = grid.panel(0, 0).unwrap();
        approx(p.x_offset, 60.0);
        approx(p.y_offset, 20.0);
        approx(p.width, 800.0 - 60.0 - 20.0);
        approx(p.height, 600.0 - 20.0 - 40.0);
    }

    #[test]
    fn two_by_two_grid() {
        let grid = FacetGrid::new(800.0, 600.0, 2, 2);
        assert_eq!(grid.n_panels(), 4);
        // Panel (0, 0): top-left.
        let p00 = grid.panel(0, 0).unwrap();
        approx(p00.x_offset, 60.0);
        approx(p00.y_offset, 20.0);
        // Panel (0, 1): top-right (col 1).
        let p01 = grid.panel(0, 1).unwrap();
        approx(p01.x_offset, 60.0 + (800.0 - 60.0 - 20.0) / 2.0);
        approx(p01.y_offset, 20.0);
        // Panel (1, 0): bottom-left.
        let p10 = grid.panel(1, 0).unwrap();
        approx(p10.x_offset, 60.0);
        approx(p10.y_offset, 20.0 + (600.0 - 20.0 - 40.0) / 2.0);
    }

    #[test]
    fn three_by_five_grid() {
        let grid = FacetGrid::new(1000.0, 600.0, 3, 5);
        assert_eq!(grid.n_panels(), 15);
        // Panel (2, 4) = bottom-right.
        let p = grid.panel(2, 4).unwrap();
        approx(p.x_offset, 60.0 + 4.0 * (1000.0 - 60.0 - 20.0) / 5.0);
        approx(p.y_offset, 20.0 + 2.0 * (600.0 - 20.0 - 40.0) / 3.0);
    }

    #[test]
    fn out_of_range_panel_returns_none() {
        let grid = FacetGrid::new(800.0, 600.0, 2, 2);
        assert!(grid.panel(2, 0).is_none());
        assert!(grid.panel(0, 2).is_none());
        assert!(grid.panel(99, 99).is_none());
    }

    #[test]
    #[should_panic(expected = "n_rows must be > 0")]
    fn zero_rows_panics() {
        let _ = FacetGrid::new(800.0, 600.0, 0, 2);
    }

    #[test]
    #[should_panic(expected = "canvas_w must be > 0")]
    fn zero_width_panics() {
        let _ = FacetGrid::new(0.0, 600.0, 2, 2);
    }

    #[test]
    fn panels_iter_yields_row_major_order() {
        let grid = FacetGrid::new(800.0, 600.0, 2, 3);
        let panels = grid.panels();
        // Row 0 panels have y_offset = 20 (top).
        // Row 1 panels have y_offset = 20 + (600 - 20 - 40) / 2.
        for col in 0..3 {
            assert!(panels[col].y_offset < panels[3 + col].y_offset);
        }
    }
}
