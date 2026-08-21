//! The `Backend` trait, native SVG backend, and the `FacetGrid`
//! layout math.

/// The `Backend` trait and shared types (points, styles, brackets).
pub mod backend;

pub(crate) mod geometry;

/// `FacetGrid` subplot grid math. Pure layout math; no actual multi-panel
/// rendering yet (the per-panel rendering is a future commit;
/// the `Plot` step in `lichtung-pipeline` can use the panel
/// rectangles to call individual plot renderers).
pub mod layout;

/// Native SVG backend used by `Chart::render_to_svg`. It writes
/// deterministic vector output directly, including axes, text, and
/// significance labels.
pub mod svg_backend;
