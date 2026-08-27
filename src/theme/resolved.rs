//! Publication-grade visual themes.
//!
//! Every theme is hardcoded so visual regressions remain reviewable and
//! re-renders are byte-stable under the same font environment.
//! Each built-in theme resolves privately to colors, fonts, and layout style.

#![deny(missing_docs)]

/// Built-in axis tick-label font size in points.
pub(crate) const TICK_FONT_SIZE: u32 = 16;

#[cfg(test)]
thread_local! {
    pub(crate) static THEME_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! define_themes {
    ($($variant:ident => ($documentation:literal, $resolver:ident)),+ $(,)?) => {
        /// Predefined publication themes with hardcoded journal/venue palettes.
        ///
        /// ```compile_fail
        /// let _ = lichtung_plot::Theme::Publication.resolve();
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Theme {
            $(
                #[doc = $documentation]
                $variant,
            )+
        }

        impl Theme {
            pub(crate) fn resolve(self) -> ResolvedTheme {
                #[cfg(test)]
                THEME_RESOLUTION_CALLS.with(|calls| calls.set(calls.get() + 1));
                match self { $(Self::$variant => $resolver()),+ }
            }
        }
    };
}

define_themes! {
    Publication => ("Default clean theme with high contrast and generous whitespace.", publication_theme),
    Nature => ("Nature-style muted blues and greens without top/right axes.", nature_theme),
    Science => ("Science-style high-saturation palette with bold axis labels.", science_theme),
    CHI => ("ACM CHI/HCI pastel palette with accessible contrast.", chi_theme),
}

/// A theme's full resolved visual specification. Layout and renderers consume
/// this value to produce concrete backend styles and layout decisions.
///
/// Defaults are tuned so that every semantic `Plot` constructor produces a
/// publication-quality canvas without the caller having to manually request
/// decorations.
///
/// The flag-style fields (show_frame / show_grid / show_ticks / ...)
/// are intentionally grouped together as a configuration block:
/// they all gate the layout grammar on a per-theme basis. The
/// "excessive bools" lint is suppressed because separating them
/// into enums would add boilerplate without changing behavior.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedTheme {
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
    /// Whether to draw the canvas title above the plot area.
    /// Default `true` when the chart has a title set.
    pub show_title: bool,
    /// Whether to draw the canvas subtitle. Default `true`
    /// when set.
    pub show_subtitle: bool,
    /// Whether to draw the canvas caption below the plot area.
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
    /// Font size in points for the canvas title. Default `20`.
    pub title_font_size: u32,
    /// Font size in points for the canvas subtitle. Default `14`.
    pub subtitle_font_size: u32,
    /// Font size in points for the canvas caption. Default `14`.
    pub caption_font_size: u32,
    /// Color of the grid lines (RGB). Default light gray.
    pub grid_color: (u8, u8, u8),
    /// Width of grid lines in pixels (0 to skip). Default `1`.
    pub grid_width: f32,
}

/// Convert an (R, G, B) tuple to a CSS hex string for use in SVG.
#[must_use]
pub(crate) fn rgb_to_hex(c: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
}

fn publication_theme() -> ResolvedTheme {
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
    ResolvedTheme {
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
        tick_font_size: TICK_FONT_SIZE,
        detail_font_size: 14,
        label_font_size: 17,
        title_font_size: 20,
        subtitle_font_size: 15,
        caption_font_size: 15,
        grid_color: (220, 220, 220),
        grid_width: 0.5,
    }
}

fn nature_theme() -> ResolvedTheme {
    // Nature-style: muted, monochromatic-leaning. Cool blue accent.
    let colors = vec![
        (40, 86, 130),   // deep blue
        (130, 160, 188), // muted blue
        (200, 215, 230), // pale blue
        (120, 130, 90),  // olive
        (180, 175, 130), // tan
    ];
    ResolvedTheme {
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
        tick_font_size: TICK_FONT_SIZE,
        detail_font_size: 14,
        label_font_size: 16,
        title_font_size: 19,
        subtitle_font_size: 14,
        caption_font_size: 14,
        grid_color: (230, 230, 230),
        grid_width: 0.5,
    }
}

fn science_theme() -> ResolvedTheme {
    // Science / Science-family: high-saturation primary palette.
    let colors = vec![
        (255, 0, 0),    // red
        (0, 0, 255),    // blue
        (0, 175, 0),    // green
        (255, 175, 0),  // gold
        (160, 32, 240), // purple
    ];
    ResolvedTheme {
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
        tick_font_size: TICK_FONT_SIZE,
        detail_font_size: 14,
        label_font_size: 17,
        title_font_size: 21,
        subtitle_font_size: 16,
        caption_font_size: 15,
        grid_color: (200, 200, 200),
        grid_width: 0.5,
    }
}

fn chi_theme() -> ResolvedTheme {
    // CHI / HCI: pastel, accessible contrast.
    let colors = vec![
        (171, 199, 232), // pastel blue
        (243, 169, 175), // pastel red
        (174, 224, 174), // pastel green
        (244, 213, 152), // pastel yellow
        (197, 176, 213), // pastel purple
    ];
    ResolvedTheme {
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
        tick_font_size: TICK_FONT_SIZE,
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

    const BUILT_IN_THEMES: [Theme; 4] = [
        Theme::Publication,
        Theme::Nature,
        Theme::Science,
        Theme::CHI,
    ];

    fn expected_publication_theme() -> ResolvedTheme {
        ResolvedTheme {
            colors: vec![
                (0, 114, 178),
                (213, 94, 0),
                (0, 158, 115),
                (204, 121, 167),
                (86, 180, 233),
                (230, 159, 0),
                (240, 228, 66),
                (0, 0, 0),
            ],
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
            tick_font_size: TICK_FONT_SIZE,
            detail_font_size: 14,
            label_font_size: 17,
            title_font_size: 20,
            subtitle_font_size: 15,
            caption_font_size: 15,
            grid_color: (220, 220, 220),
            grid_width: 0.5,
        }
    }

    fn expected_nature_theme() -> ResolvedTheme {
        ResolvedTheme {
            colors: vec![
                (40, 86, 130),
                (130, 160, 188),
                (200, 215, 230),
                (120, 130, 90),
                (180, 175, 130),
            ],
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
            tick_font_size: TICK_FONT_SIZE,
            detail_font_size: 14,
            label_font_size: 16,
            title_font_size: 19,
            subtitle_font_size: 14,
            caption_font_size: 14,
            grid_color: (230, 230, 230),
            grid_width: 0.5,
        }
    }

    fn expected_science_theme() -> ResolvedTheme {
        ResolvedTheme {
            colors: vec![
                (255, 0, 0),
                (0, 0, 255),
                (0, 175, 0),
                (255, 175, 0),
                (160, 32, 240),
            ],
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
            tick_font_size: TICK_FONT_SIZE,
            detail_font_size: 14,
            label_font_size: 17,
            title_font_size: 21,
            subtitle_font_size: 16,
            caption_font_size: 15,
            grid_color: (200, 200, 200),
            grid_width: 0.5,
        }
    }

    fn expected_chi_theme() -> ResolvedTheme {
        ResolvedTheme {
            colors: vec![
                (171, 199, 232),
                (243, 169, 175),
                (174, 224, 174),
                (244, 213, 152),
                (197, 176, 213),
            ],
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
            tick_font_size: TICK_FONT_SIZE,
            detail_font_size: 14,
            label_font_size: 17,
            title_font_size: 19,
            subtitle_font_size: 15,
            caption_font_size: 15,
            grid_color: (220, 220, 220),
            grid_width: 0.5,
        }
    }

    fn expected_theme(theme: Theme) -> ResolvedTheme {
        match theme {
            Theme::Publication => expected_publication_theme(),
            Theme::Nature => expected_nature_theme(),
            Theme::Science => expected_science_theme(),
            Theme::CHI => expected_chi_theme(),
        }
    }

    #[test]
    fn built_in_themes_preserve_every_resolved_value() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(theme.resolve(), expected_theme(theme), "{theme:?}");
        }
    }

    #[test]
    fn built_in_themes_are_structurally_complete() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let widths = [
                resolved.axis_width,
                resolved.line_width,
                resolved.bracket_width,
                resolved.grid_width,
            ];
            let font_sizes = [
                resolved.tick_font_size,
                resolved.detail_font_size,
                resolved.label_font_size,
                resolved.title_font_size,
                resolved.subtitle_font_size,
                resolved.caption_font_size,
            ];

            assert!(!resolved.colors.is_empty(), "{theme:?} has no colors");
            assert!(
                !resolved.font_family.trim().is_empty(),
                "{theme:?} has no font family"
            );
            assert!(
                widths
                    .into_iter()
                    .all(|width| width.is_finite() && width > 0.0),
                "{theme:?} has a nonpositive or nonfinite width"
            );
            assert!(resolved.tick_length > 0, "{theme:?} has no tick length");
            assert!(
                font_sizes.into_iter().all(|size| size > 0),
                "{theme:?} has a zero font size"
            );
        }
    }

    #[test]
    fn rgb_to_hex_format() {
        assert_eq!(rgb_to_hex((0, 114, 178)), "#0072B2");
        assert_eq!(rgb_to_hex((255, 255, 255)), "#FFFFFF");
        assert_eq!(rgb_to_hex((0, 0, 0)), "#000000");
    }

    #[test]
    fn all_themes_default_to_publication_grade_layout() {
        // Every current built-in opts in to the complete layout grammar.
        for t in BUILT_IN_THEMES {
            let p = t.resolve();
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
        assert!([
            Theme::Publication,
            Theme::Nature,
            Theme::Science,
            Theme::CHI,
        ]
        .into_iter()
        .all(|theme| theme.resolve().tick_font_size == TICK_FONT_SIZE));
    }

    #[test]
    fn all_themes_keep_non_axis_detail_typography_compact() {
        assert!([
            Theme::Publication,
            Theme::Nature,
            Theme::Science,
            Theme::CHI,
        ]
        .into_iter()
        .all(|theme| theme.resolve().detail_font_size == 14));
    }
}
