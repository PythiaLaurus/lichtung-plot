//! Internal rendering backend and native SVG implementation.

/// The `Backend` trait and shared types (points, styles, brackets).
pub mod backend;

pub(crate) mod coordinates;
pub(crate) mod geometry;

pub mod svg_backend;
