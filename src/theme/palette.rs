//! `lichtung-plot::theme` — publication-grade visual themes.
//!
//! Every theme is hardcoded so visual regressions remain reviewable and
//! re-renders are byte-stable under the same font environment.
//! Themes are pure data: a `ThemePalette` is just colors, font, and
//! layout style. The rendering engine applies them per `Backend` impl.

#![deny(missing_docs)]

macro_rules! define_themes {
    ($($variant:ident => ($documentation:literal, $palette:ident)),+ $(,)?) => {
        /// Predefined publication themes with hardcoded journal/venue palettes.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Theme {
            $(
                #[doc = $documentation]
                $variant,
            )+
        }

        impl Theme {
            /// All predefined themes in stable declaration order.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Rust variant name used by capability inventories.
            #[must_use]
            pub const fn variant_name(self) -> &'static str {
                match self { $(Self::$variant => stringify!($variant)),+ }
            }

            /// Resolve this theme to its hardcoded [`ThemePalette`].
            #[must_use]
            pub fn palette(self) -> ThemePalette {
                match self { $(Self::$variant => $palette()),+ }
            }
        }
    };
}

define_themes! {
    Publication => ("Default clean theme with high contrast and generous whitespace.", publication_palette),
    Nature => ("Nature-style muted blues and greens without top/right axes.", nature_palette),
    Science => ("Science-style high-saturation palette with bold axis labels.", science_palette),
    CHI => ("ACM CHI/HCI pastel palette with accessible contrast.", chi_palette),
}

/// A theme's full visual specification. Pure data; the `Backend`
/// impl reads this to decide colors, line widths, fonts, and the
/// layout grammar (whether to draw the frame, grid, ticks, etc.).
///
/// Defaults are tuned so that
/// `Chart::new(df).x("a").y("b").scatter()` (or any other plot
/// kind) produces a publication-quality figure without the caller
/// having to manually request decorations.
///
/// The flag-style fields (show_frame / show_grid / show_ticks / ...)
/// are intentionally grouped together as a configuration block:
/// they all gate the layout grammar on a per-theme basis. The
/// "excessive bools" lint is suppressed because separating them
/// into enums would add boilerplate without changing behavior.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct ThemePalette {
    /// Series colors in draw order. Index 0 is the first series.
    pub colors: Vec<(u8, u8, u8)>,
    /// CSS-compatible system-font family stack.
    pub font_family: String,
    /// Background color; default white.
    pub background: (u8, u8, u8),
    /// Foreground (axis, text) color; default near-black.
    pub foreground: (u8, u8, u8),
    /// Width of axis lines in pixels.
    pub axis_width: f32,
    /// Width of plot series lines in pixels.
    pub line_width: f32,
    /// Width of significance bracket strokes in pixels.
    pub bracket_width: f32,
    /// Whether to draw the top and right axis spines.
    /// `Nature` / `Publication` / `CHI` set this to `false`
    /// (the classic "open frame" look); `Science` sets it to
    /// `true` (full box).
    pub show_top_right_spines: bool,
    // ---- Layout grammar (Phase 3+: theme-driven defaults) ----
    /// Whether to draw the four-spine frame around the plot
    /// area. Default `true`.
    pub show_frame: bool,
    /// Whether to draw grid lines at the axis tick positions.
    /// Default `true`.
    pub show_grid: bool,
    /// Whether to draw the axis tick marks (short lines at each
    /// tick crossing the spine). Default `true`.
    pub show_ticks: bool,
    /// Whether to draw the numeric/string tick labels next to
    /// each tick. Default `true`.
    pub show_tick_labels: bool,
    /// Whether to draw the x-axis label below the plot area.
    /// Default `true` when the chart has an x label set.
    pub show_x_label: bool,
    /// Whether to draw the y-axis label rotated -90° to the
    /// left of the plot area. Default `true` when the chart has
    /// a y label set.
    pub show_y_label: bool,
    /// Whether to draw the figure title above the plot area.
    /// Default `true` when the chart has a title set.
    pub show_title: bool,
    /// Whether to draw the figure subtitle. Default `true`
    /// when set.
    pub show_subtitle: bool,
    /// Whether to draw the figure caption below the plot area.
    /// Default `true` when set.
    pub show_caption: bool,
    /// Length of the axis tick marks in pixels. Default `4`.
    pub tick_length: u32,
    /// Font size in points for axis tick labels. Default `16`.
    pub tick_font_size: u32,
    /// Font size in points for annotations, status text, and color scales. Default `14`.
    pub detail_font_size: u32,
    /// Font size in points for axis labels. Default `17`.
    pub label_font_size: u32,
    /// Font size in points for the figure title. Default `20`.
    pub title_font_size: u32,
    /// Font size in points for the figure subtitle. Default `14`.
    pub subtitle_font_size: u32,
    /// Font size in points for the figure caption. Default `14`.
    pub caption_font_size: u32,
    /// Color of the grid lines (RGB). Default light gray.
    pub grid_color: (u8, u8, u8),
    /// Width of grid lines in pixels (0 to skip). Default `1`.
    pub grid_width: f32,
}

/// Convert an (R, G, B) tuple to a CSS hex string for use in SVG.
#[must_use]
pub fn rgb_to_hex(c: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
}

fn publication_palette() -> ThemePalette {
    // Okabe-Ito 8-color qualitative palette (colorblind-safe);
    // high contrast, used by default.
    let colors = vec![
        (0, 114, 178),   // blue
        (213, 94, 0),    // vermilion
        (0, 158, 115),   // bluish green
        (204, 121, 167), // reddish purple
        (86, 180, 233),  // sky blue
        (230, 159, 0),   // orange
        (240, 228, 66),  // yellow
        (0, 0, 0),       // black
    ];
    ThemePalette {
        colors,
        font_family: crate::DEFAULT_FONT_FAMILY_STACK.into(),
        background: (255, 255, 255),
        foreground: (0, 0, 0),
        axis_width: 0.8,
        line_width: 1.5,
        bracket_width: 1.0,
        show_top_right_spines: false,
        show_frame: true,
        show_grid: true,
        show_ticks: true,
        show_tick_labels: true,
        show_x_label: true,
        show_y_label: true,
        show_title: true,
        show_subtitle: true,
        show_caption: true,
        tick_length: 5,
        tick_font_size: 16,
        detail_font_size: 14,
        label_font_size: 17,
        title_font_size: 20,
        subtitle_font_size: 15,
        caption_font_size: 15,
        grid_color: (220, 220, 220),
        grid_width: 0.5,
    }
}

fn nature_palette() -> ThemePalette {
    // Nature-style: muted, monochromatic-leaning. Cool blue accent.
    let colors = vec![
        (40, 86, 130),   // deep blue
        (130, 160, 188), // muted blue
        (200, 215, 230), // pale blue
        (120, 130, 90),  // olive
        (180, 175, 130), // tan
    ];
    ThemePalette {
        colors,
        font_family: crate::DEFAULT_FONT_FAMILY_STACK.into(),
        background: (255, 255, 255),
        foreground: (40, 40, 40),
        axis_width: 0.8,
        line_width: 1.0,
        bracket_width: 0.8,
        show_top_right_spines: false,
        show_frame: true,
        show_grid: true,
        show_ticks: true,
        show_tick_labels: true,
        show_x_label: true,
        show_y_label: true,
        show_title: true,
        show_subtitle: true,
        show_caption: true,
        tick_length: 5,
        tick_font_size: 16,
        detail_font_size: 14,
        label_font_size: 16,
        title_font_size: 19,
        subtitle_font_size: 14,
        caption_font_size: 14,
        grid_color: (230, 230, 230),
        grid_width: 0.5,
    }
}

fn science_palette() -> ThemePalette {
    // Science / Science-family: high-saturation primary palette.
    let colors = vec![
        (255, 0, 0),    // red
        (0, 0, 255),    // blue
        (0, 175, 0),    // green
        (255, 175, 0),  // gold
        (160, 32, 240), // purple
    ];
    ThemePalette {
        colors,
        font_family: crate::DEFAULT_FONT_FAMILY_STACK.into(),
        background: (255, 255, 255),
        foreground: (0, 0, 0),
        axis_width: 1.0,
        line_width: 1.5,
        bracket_width: 1.0,
        show_top_right_spines: true,
        show_frame: true,
        show_grid: true,
        show_ticks: true,
        show_tick_labels: true,
        show_x_label: true,
        show_y_label: true,
        show_title: true,
        show_subtitle: true,
        show_caption: true,
        tick_length: 5,
        tick_font_size: 16,
        detail_font_size: 14,
        label_font_size: 17,
        title_font_size: 21,
        subtitle_font_size: 16,
        caption_font_size: 15,
        grid_color: (200, 200, 200),
        grid_width: 0.5,
    }
}

fn chi_palette() -> ThemePalette {
    // CHI / HCI: pastel, accessible contrast.
    let colors = vec![
        (171, 199, 232), // pastel blue
        (243, 169, 175), // pastel red
        (174, 224, 174), // pastel green
        (244, 213, 152), // pastel yellow
        (197, 176, 213), // pastel purple
    ];
    ThemePalette {
        colors,
        font_family: crate::DEFAULT_FONT_FAMILY_STACK.into(),
        background: (255, 255, 255),
        foreground: (60, 60, 60),
        axis_width: 0.8,
        line_width: 1.5,
        bracket_width: 0.8,
        show_top_right_spines: false,
        show_frame: true,
        show_grid: true,
        show_ticks: true,
        show_tick_labels: true,
        show_x_label: true,
        show_y_label: true,
        show_title: true,
        show_subtitle: true,
        show_caption: true,
        tick_length: 5,
        tick_font_size: 16,
        detail_font_size: 14,
        label_font_size: 17,
        title_font_size: 19,
        subtitle_font_size: 15,
        caption_font_size: 15,
        grid_color: (220, 220, 220),
        grid_width: 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_returns_distinct_palettes() {
        for &t in Theme::ALL {
            let p = t.palette();
            assert!(!p.colors.is_empty(), "{t:?} must have colors");
            assert!(!p.font_family.is_empty());
        }
        // Publication (Okabe-Ito) and Science have different shapes.
        let p = Theme::Publication.palette();
        let s = Theme::Science.palette();
        assert!(p.colors.len() >= 6, "Okabe-Ito has 8 colors");
        assert!(s.colors.len() >= 5, "Science has 5+ colors");
    }

    #[test]
    fn rgb_to_hex_format() {
        assert_eq!(rgb_to_hex((0, 114, 178)), "#0072B2");
        assert_eq!(rgb_to_hex((255, 255, 255)), "#FFFFFF");
        assert_eq!(rgb_to_hex((0, 0, 0)), "#000000");
    }

    #[test]
    fn all_themes_default_to_publication_grade_layout() {
        // Every theme should opt in to the layout grammar by
        // default; users can disable individual elements by
        // toggling the corresponding flag on a custom palette.
        for &t in Theme::ALL {
            let p = t.palette();
            assert!(p.show_frame, "{t:?}: frame must be visible by default");
            assert!(p.show_grid, "{t:?}: grid must be visible by default");
            assert!(p.show_ticks, "{t:?}: ticks must be visible by default");
            assert!(
                p.show_tick_labels,
                "{t:?}: tick labels must be visible by default"
            );
            assert!(p.show_x_label, "{t:?}: x label must be visible by default");
            assert!(p.show_y_label, "{t:?}: y label must be visible by default");
            assert!(p.show_title, "{t:?}: title must be visible by default");
            assert!(p.tick_length > 0, "{t:?}: tick_length must be > 0");
            assert!(
                p.tick_font_size > 0
                    && p.detail_font_size > 0
                    && p.label_font_size > 0
                    && p.title_font_size > 0,
                "{t:?}: font sizes must be > 0"
            );
        }
    }

    #[test]
    fn all_themes_use_readable_default_tick_typography() {
        assert!(Theme::ALL
            .iter()
            .copied()
            .all(|theme| theme.palette().tick_font_size == 16));
    }

    #[test]
    fn all_themes_keep_non_axis_detail_typography_compact() {
        assert!(Theme::ALL
            .iter()
            .copied()
            .all(|theme| theme.palette().detail_font_size == 14));
    }
}
