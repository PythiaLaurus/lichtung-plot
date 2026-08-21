//! Concrete plot implementations, one per `PlotKind` variant.
//! Each submodule exposes a `pub fn render(chart, backend)` free
//! function with the same signature, dispatched to by
//! `Chart::render_with_kind`.

// Phase 1: scatter, bar (with CI), boxplot.
pub mod bar;
pub(crate) mod bar_data;
pub mod boxplot;
pub mod density;
pub(crate) mod distribution_sample;
pub mod dumbbell;
pub mod heatmap;
pub mod histogram;
pub mod horizontal_bar;
pub mod scatter;
// Phase 3: violin (KDE + box overlay), regression, marginal, matrix.
pub mod coefficient;
pub mod forest;
pub mod interaction;
pub(crate) mod interval_estimate;
pub mod line;
pub mod marginal;
pub mod matrix;
pub mod point_interval;
pub mod qq;
pub mod regression;
pub mod violin;
// Post-Phase 4: mass_spectrum (line + peak markers + m/z labels).
pub mod beeswarm;
pub mod mass_spectrum;
pub mod waterfall;
