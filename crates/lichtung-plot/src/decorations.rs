//! Non-data chart decorations: legends, titles, captions, and decoration orchestration.

/// Apply canvas-level decorations after plot data is rendered.
pub(crate) mod apply;
/// Plot legend rendering and placement helpers.
pub(crate) mod legend;
pub(crate) mod panel_text;
pub(crate) mod plan;
/// Title, subtitle, and caption rendering.
pub(crate) mod title_block;
