//! Canonical system-font preparation and current rendering defaults.

pub(crate) mod canonical;
mod unicode16;

/// Preferred system font family.
pub const DEFAULT_FONT_FAMILY: &str = "Noto Sans";

/// Ordered CSS system-font fallback stack used by built-in themes.
pub const DEFAULT_FONT_FAMILY_STACK: &str =
    "Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif";
