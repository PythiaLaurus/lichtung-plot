//! Publication-grade visual themes.
//!
//! Every theme is hardcoded so visual regressions remain reviewable and
//! re-renders are byte-stable under the same font environment.
//! Each built-in theme resolves privately to colors, fonts, and layout style.

#![deny(missing_docs)]

use crate::render::backend::{LineStyle, Rgba, StrokePattern, TextAnchor, TextStyle};
use crate::{SignificanceMarkStyle, TextWeight};

/// Built-in axis tick-label font size in points.
const TICK_FONT_SIZE: u32 = 16;

const MUTED_TEXT: (u8, u8, u8) = (80, 80, 80);
const SIGNIFICANCE_FONT_SIZE: u32 = 16;

#[cfg(test)]
thread_local! {
    pub(crate) static THEME_RESOLUTION_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! define_themes {
    ($($variant:ident => ($documentation:literal, $resolver:ident)),+ $(,)?) => {
        /// Predefined publication themes with hardcoded journal/venue palettes.
        ///
        /// Attach one complete selection with [`crate::Plot::theme`] or
        /// [`crate::Canvas::theme`].
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
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedTheme {
    /// Series colors in draw order. Index 0 is the first series.
    pub(crate) colors: Vec<(u8, u8, u8)>,
    /// Legacy stack source retained for later legend, mark, and annotation roles.
    pub(crate) font_family: String,
    /// Legacy paint source retained for later legend, mark, and annotation roles.
    pub(crate) foreground: (u8, u8, u8),
    /// Legacy interval-mark width retained until T5c mark migration.
    pub(crate) interval_width: f32,
    /// Width of plot series lines in pixels.
    pub(crate) line_width: f32,
    /// Width of significance bracket strokes in pixels.
    pub(crate) bracket_width: f32,
    /// Font size in points for annotations, status text, and color scales.
    pub(crate) detail_font_size: u32,
    /// Legacy label-size source retained for later body and annotation roles.
    pub(crate) label_font_size: u32,
    /// Legacy Canvas-composition title size retained until T5b.
    pub(crate) composition_title_font_size: u32,
    /// Legacy facet-header size source retained until T5b composition migration.
    pub(crate) facet_header_font_size: u32,
    /// Canvas-root and Plot-panel paints.
    pub(crate) surfaces: SurfaceAppearance,
    /// Complete foundational text roles.
    pub(crate) typography: FoundationalTypography,
    /// Nonsemantic axis chrome and its local spacing.
    pub(crate) axes: AxisAppearance,
    /// Root and panel composition relationships.
    pub(crate) composition: CompositionAppearance,
}

/// Paints applied to renderer-owned drawing surfaces.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SurfaceAppearance {
    /// Canvas root paint.
    pub(crate) canvas_root: Rgba,
    /// Plot panel paint.
    pub(crate) plot_panel: Rgba,
}

/// Complete text appearance for one semantic typography role.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextAppearance {
    /// CSS-compatible system-font family stack.
    pub(crate) stack: String,
    /// Text paint.
    pub(crate) paint: Rgba,
    /// Font size in points.
    pub(crate) font_size: u32,
    /// Closed renderer-owned font weight.
    pub(crate) weight: TextWeight,
    /// Finite positive multiple of the font size used for line pitch.
    pub(crate) line_height_multiplier: f64,
}

impl TextAppearance {
    /// Resolve this role's current line pitch in pixels.
    pub(crate) fn line_height(&self) -> f64 {
        f64::from(self.font_size) * self.line_height_multiplier
    }

    /// Build the private backend style for a fixed renderer-owned placement.
    pub(crate) fn backend_style(&self, anchor: TextAnchor, rotation: f64) -> TextStyle {
        TextStyle {
            color: self.paint,
            font_family: self.stack.clone(),
            font_size: self.font_size,
            weight: self.weight,
            anchor,
            rotation,
        }
    }
}

/// Foundational role-based typography resolved before measurement.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FoundationalTypography {
    /// Root and panel title role.
    pub(crate) title: TextAppearance,
    /// Root and panel subtitle role.
    pub(crate) subtitle: TextAppearance,
    /// Root caption role.
    pub(crate) caption: TextAppearance,
    /// X- and y-axis title role.
    pub(crate) axis_title: TextAppearance,
    /// X- and y-axis tick-label role.
    pub(crate) tick_label: TextAppearance,
    /// Significance-label default before an exact caller override.
    pub(crate) significance_label: TextAppearance,
}

impl FoundationalTypography {
    /// Resolve the effective significance-label typography while preserving an
    /// exact caller size and weight over the theme defaults.
    pub(crate) fn effective_significance_label(
        &self,
        exact: Option<SignificanceMarkStyle>,
    ) -> TextAppearance {
        let mut effective = self.significance_label.clone();
        if let Some(exact) = exact {
            effective.font_size = exact.font_size_spec();
            effective.weight = exact.weight_spec();
        }
        effective
    }
}

/// One complete axis stroke style.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StrokeAppearance {
    /// Stroke paint.
    pub(crate) paint: Rgba,
    /// Finite positive stroke width in pixels.
    pub(crate) width: f32,
    /// Closed stroke pattern.
    pub(crate) pattern: StrokePattern,
}

impl StrokeAppearance {
    /// Build the private backend line style for this complete stroke.
    pub(crate) fn backend_style(&self) -> LineStyle {
        LineStyle {
            color: self.paint,
            width: self.width,
            pattern: self.pattern,
        }
    }
}

/// Structurally present frame appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FrameAppearance {
    /// Complete frame stroke.
    pub(crate) stroke: StrokeAppearance,
    /// Original preset setting retained until T5a5 normalizes frame topology.
    pub(crate) source_show_top_right_spines: bool,
}

/// Structurally present tick-mark appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TickAppearance {
    /// Complete tick stroke.
    pub(crate) stroke: StrokeAppearance,
    /// Tick length in pixels.
    pub(crate) length: u32,
}

/// Role-local axis spacing in pixels.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisSpacing {
    /// X tick-mark to tick-label gap.
    pub(crate) x_tick_label_gap: f64,
    /// Y tick-mark to tick-label gap.
    pub(crate) y_tick_label_gap: f64,
    /// X tick-label to axis-title gap.
    pub(crate) x_title_tick_label_gap: f64,
    /// X tick-mark to axis-title gap when labels are absent.
    pub(crate) x_title_without_tick_labels_gap: f64,
    /// Y tick-label to axis-title gap.
    pub(crate) y_title_tick_label_gap: f64,
    /// Minimum y-axis title clearance from the canvas edge.
    pub(crate) y_title_edge_padding: f64,
}

/// Structurally resolved axis, frame, tick, and grid appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AxisAppearance {
    /// Optional complete frame style; `None` means no frame.
    pub(crate) frame: Option<FrameAppearance>,
    /// Optional complete tick style; `None` means no tick marks.
    pub(crate) ticks: Option<TickAppearance>,
    /// Optional complete grid style; `None` means no grid.
    pub(crate) grid: Option<StrokeAppearance>,
    /// Axis-local spacing relationships.
    pub(crate) spacing: AxisSpacing,
}

/// Horizontal alignment of every role in one root title block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootTitleBlockAlignment {
    /// Align to the physical left edge of the local span.
    #[allow(
        dead_code,
        reason = "private alignment is activated by future public themes"
    )]
    Start,
    /// Align to the physical midpoint of the local span.
    Center,
    /// Align to the physical right edge of the local span.
    #[allow(
        dead_code,
        reason = "private alignment is activated by future public themes"
    )]
    End,
}

impl RootTitleBlockAlignment {
    /// Resolve this alignment into one physical x coordinate and backend anchor.
    #[allow(
        dead_code,
        reason = "private root alignment is resolved now and applied by T5a4 title plans"
    )]
    pub(crate) fn resolve_physical(self, span_start: f64, span_end: f64) -> (f64, TextAnchor) {
        match self {
            Self::Start => (span_start, TextAnchor::Start),
            Self::Center => (
                span_start + (span_end - span_start) / 2.0,
                TextAnchor::Middle,
            ),
            Self::End => (span_end, TextAnchor::End),
        }
    }
}

/// Root title-block appearance and local spacing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootTitleBlockAppearance {
    /// Shared title, subtitle, and caption alignment.
    pub(crate) alignment: RootTitleBlockAlignment,
    /// Gap between adjacent title and subtitle line boxes.
    pub(crate) title_subtitle_gap: f64,
    /// Clearance between the top title block and root content.
    pub(crate) title_block_to_content_clearance: f64,
    /// Clearance between root content and the caption.
    pub(crate) caption_to_content_clearance: f64,
}

/// Root composition appearance.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompositionAppearance {
    /// Plot or Canvas root title block.
    pub(crate) root_title_block: RootTitleBlockAppearance,
}

#[allow(
    clippy::too_many_arguments,
    reason = "the four closed presets provide one compact table of preserved scalar values"
)]
fn resolved_theme(
    colors: Vec<(u8, u8, u8)>,
    foreground: (u8, u8, u8),
    axis_width: f32,
    line_width: f32,
    bracket_width: f32,
    title_font_size: u32,
    subtitle_font_size: u32,
    caption_font_size: u32,
    axis_title_font_size: u32,
    grid_color: (u8, u8, u8),
    source_show_top_right_spines: bool,
) -> ResolvedTheme {
    let stack = crate::DEFAULT_FONT_FAMILY_STACK.to_owned();
    let foreground_paint = Rgba::from(foreground);
    let muted_paint = Rgba::from(MUTED_TEXT);

    ResolvedTheme {
        colors,
        font_family: stack.clone(),
        foreground,
        interval_width: axis_width,
        line_width,
        bracket_width,
        detail_font_size: 14,
        label_font_size: axis_title_font_size,
        composition_title_font_size: title_font_size,
        facet_header_font_size: subtitle_font_size,
        surfaces: SurfaceAppearance {
            canvas_root: Rgba::rgb(255, 255, 255),
            plot_panel: Rgba::rgb(255, 255, 255),
        },
        typography: FoundationalTypography {
            title: text_appearance(
                &stack,
                foreground_paint,
                title_font_size,
                plus_six_line_height(title_font_size),
            ),
            subtitle: text_appearance(
                &stack,
                muted_paint,
                subtitle_font_size,
                plus_six_line_height(subtitle_font_size),
            ),
            caption: text_appearance(
                &stack,
                muted_paint,
                caption_font_size,
                plus_six_line_height(caption_font_size),
            ),
            axis_title: text_appearance(&stack, foreground_paint, axis_title_font_size, 1.2),
            tick_label: text_appearance(&stack, foreground_paint, TICK_FONT_SIZE, 1.2),
            significance_label: text_appearance(
                &stack,
                foreground_paint,
                SIGNIFICANCE_FONT_SIZE,
                1.2,
            ),
        },
        axes: AxisAppearance {
            frame: Some(FrameAppearance {
                stroke: stroke_appearance(foreground_paint, axis_width, StrokePattern::Solid),
                source_show_top_right_spines,
            }),
            ticks: Some(TickAppearance {
                stroke: stroke_appearance(foreground_paint, axis_width, StrokePattern::Solid),
                length: 5,
            }),
            grid: Some(stroke_appearance(
                Rgba::from(grid_color),
                0.5,
                StrokePattern::MediumDash,
            )),
            spacing: AxisSpacing {
                x_tick_label_gap: 8.0,
                y_tick_label_gap: 6.0,
                x_title_tick_label_gap: 8.0,
                x_title_without_tick_labels_gap: 16.0,
                y_title_tick_label_gap: 16.0,
                y_title_edge_padding: 12.0,
            },
        },
        composition: CompositionAppearance {
            root_title_block: RootTitleBlockAppearance {
                alignment: RootTitleBlockAlignment::Center,
                title_subtitle_gap: 0.0,
                title_block_to_content_clearance: 14.0,
                caption_to_content_clearance: 12.0,
            },
        },
    }
}

fn text_appearance(
    stack: &str,
    paint: Rgba,
    font_size: u32,
    line_height_multiplier: f64,
) -> TextAppearance {
    TextAppearance {
        stack: stack.to_owned(),
        paint,
        font_size,
        weight: TextWeight::Normal,
        line_height_multiplier,
    }
}

fn stroke_appearance(paint: Rgba, width: f32, pattern: StrokePattern) -> StrokeAppearance {
    StrokeAppearance {
        paint,
        width,
        pattern,
    }
}

fn plus_six_line_height(font_size: u32) -> f64 {
    (f64::from(font_size) + 6.0) / f64::from(font_size)
}

fn publication_theme() -> ResolvedTheme {
    // Okabe-Ito 8-color qualitative palette (colorblind-safe).
    resolved_theme(
        vec![
            (0, 114, 178),
            (213, 94, 0),
            (0, 158, 115),
            (204, 121, 167),
            (86, 180, 233),
            (230, 159, 0),
            (240, 228, 66),
            (0, 0, 0),
        ],
        (0, 0, 0),
        0.8,
        1.5,
        1.0,
        20,
        15,
        15,
        17,
        (220, 220, 220),
        false,
    )
}

fn nature_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
            (40, 86, 130),
            (130, 160, 188),
            (200, 215, 230),
            (120, 130, 90),
            (180, 175, 130),
        ],
        (40, 40, 40),
        0.8,
        1.0,
        0.8,
        19,
        14,
        14,
        16,
        (230, 230, 230),
        false,
    )
}

fn science_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
            (255, 0, 0),
            (0, 0, 255),
            (0, 175, 0),
            (255, 175, 0),
            (160, 32, 240),
        ],
        (0, 0, 0),
        1.0,
        1.5,
        1.0,
        21,
        16,
        15,
        17,
        (200, 200, 200),
        true,
    )
}

fn chi_theme() -> ResolvedTheme {
    resolved_theme(
        vec![
            (171, 199, 232),
            (243, 169, 175),
            (174, 224, 174),
            (244, 213, 152),
            (197, 176, 213),
        ],
        (60, 60, 60),
        0.8,
        1.5,
        0.8,
        19,
        15,
        15,
        17,
        (220, 220, 220),
        false,
    )
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

    #[derive(Clone, Copy)]
    struct ExpectedPreset {
        foreground: (u8, u8, u8),
        axis_width: f32,
        line_width: f32,
        bracket_width: f32,
        title_size: u32,
        subtitle_size: u32,
        caption_size: u32,
        axis_title_size: u32,
        grid_color: (u8, u8, u8),
        top_right_spines: bool,
    }

    fn expected_preset(theme: Theme) -> (Vec<(u8, u8, u8)>, ExpectedPreset) {
        match theme {
            Theme::Publication => (
                vec![
                    (0, 114, 178),
                    (213, 94, 0),
                    (0, 158, 115),
                    (204, 121, 167),
                    (86, 180, 233),
                    (230, 159, 0),
                    (240, 228, 66),
                    (0, 0, 0),
                ],
                ExpectedPreset {
                    foreground: (0, 0, 0),
                    axis_width: 0.8,
                    line_width: 1.5,
                    bracket_width: 1.0,
                    title_size: 20,
                    subtitle_size: 15,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (220, 220, 220),
                    top_right_spines: false,
                },
            ),
            Theme::Nature => (
                vec![
                    (40, 86, 130),
                    (130, 160, 188),
                    (200, 215, 230),
                    (120, 130, 90),
                    (180, 175, 130),
                ],
                ExpectedPreset {
                    foreground: (40, 40, 40),
                    axis_width: 0.8,
                    line_width: 1.0,
                    bracket_width: 0.8,
                    title_size: 19,
                    subtitle_size: 14,
                    caption_size: 14,
                    axis_title_size: 16,
                    grid_color: (230, 230, 230),
                    top_right_spines: false,
                },
            ),
            Theme::Science => (
                vec![
                    (255, 0, 0),
                    (0, 0, 255),
                    (0, 175, 0),
                    (255, 175, 0),
                    (160, 32, 240),
                ],
                ExpectedPreset {
                    foreground: (0, 0, 0),
                    axis_width: 1.0,
                    line_width: 1.5,
                    bracket_width: 1.0,
                    title_size: 21,
                    subtitle_size: 16,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (200, 200, 200),
                    top_right_spines: true,
                },
            ),
            Theme::CHI => (
                vec![
                    (171, 199, 232),
                    (243, 169, 175),
                    (174, 224, 174),
                    (244, 213, 152),
                    (197, 176, 213),
                ],
                ExpectedPreset {
                    foreground: (60, 60, 60),
                    axis_width: 0.8,
                    line_width: 1.5,
                    bracket_width: 0.8,
                    title_size: 19,
                    subtitle_size: 15,
                    caption_size: 15,
                    axis_title_size: 17,
                    grid_color: (220, 220, 220),
                    top_right_spines: false,
                },
            ),
        }
    }

    fn expected_theme(theme: Theme) -> ResolvedTheme {
        let (colors, expected) = expected_preset(theme);
        let stack = crate::DEFAULT_FONT_FAMILY_STACK.to_owned();
        let foreground = Rgba::from(expected.foreground);
        let text = |paint, font_size, line_height_multiplier| TextAppearance {
            stack: stack.clone(),
            paint,
            font_size,
            weight: TextWeight::Normal,
            line_height_multiplier,
        };

        ResolvedTheme {
            colors,
            font_family: stack.clone(),
            foreground: expected.foreground,
            interval_width: expected.axis_width,
            line_width: expected.line_width,
            bracket_width: expected.bracket_width,
            detail_font_size: 14,
            label_font_size: expected.axis_title_size,
            composition_title_font_size: expected.title_size,
            facet_header_font_size: expected.subtitle_size,
            surfaces: SurfaceAppearance {
                canvas_root: Rgba::new(255, 255, 255, 255),
                plot_panel: Rgba::new(255, 255, 255, 255),
            },
            typography: FoundationalTypography {
                title: text(
                    foreground,
                    expected.title_size,
                    (f64::from(expected.title_size) + 6.0) / f64::from(expected.title_size),
                ),
                subtitle: text(
                    Rgba::rgb(80, 80, 80),
                    expected.subtitle_size,
                    (f64::from(expected.subtitle_size) + 6.0) / f64::from(expected.subtitle_size),
                ),
                caption: text(
                    Rgba::rgb(80, 80, 80),
                    expected.caption_size,
                    (f64::from(expected.caption_size) + 6.0) / f64::from(expected.caption_size),
                ),
                axis_title: text(foreground, expected.axis_title_size, 1.2),
                tick_label: text(foreground, 16, 1.2),
                significance_label: text(foreground, 16, 1.2),
            },
            axes: AxisAppearance {
                frame: Some(FrameAppearance {
                    stroke: StrokeAppearance {
                        paint: foreground,
                        width: expected.axis_width,
                        pattern: StrokePattern::Solid,
                    },
                    source_show_top_right_spines: expected.top_right_spines,
                }),
                ticks: Some(TickAppearance {
                    stroke: StrokeAppearance {
                        paint: foreground,
                        width: expected.axis_width,
                        pattern: StrokePattern::Solid,
                    },
                    length: 5,
                }),
                grid: Some(StrokeAppearance {
                    paint: Rgba::from(expected.grid_color),
                    width: 0.5,
                    pattern: StrokePattern::MediumDash,
                }),
                spacing: AxisSpacing {
                    x_tick_label_gap: 8.0,
                    y_tick_label_gap: 6.0,
                    x_title_tick_label_gap: 8.0,
                    x_title_without_tick_labels_gap: 16.0,
                    y_title_tick_label_gap: 16.0,
                    y_title_edge_padding: 12.0,
                },
            },
            composition: CompositionAppearance {
                root_title_block: RootTitleBlockAppearance {
                    alignment: RootTitleBlockAlignment::Center,
                    title_subtitle_gap: 0.0,
                    title_block_to_content_clearance: 14.0,
                    caption_to_content_clearance: 12.0,
                },
            },
        }
    }

    #[test]
    fn built_in_themes_preserve_every_resolved_value() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(theme.resolve(), expected_theme(theme), "{theme:?}");
        }
    }

    #[test]
    fn built_in_themes_have_complete_foundational_records() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let text_roles = [
                &resolved.typography.title,
                &resolved.typography.subtitle,
                &resolved.typography.caption,
                &resolved.typography.axis_title,
                &resolved.typography.tick_label,
                &resolved.typography.significance_label,
            ];

            assert!(!resolved.colors.is_empty(), "{theme:?} has no colors");
            assert_eq!(resolved.font_family, crate::DEFAULT_FONT_FAMILY_STACK);
            assert!(resolved.interval_width.is_finite() && resolved.interval_width > 0.0);
            assert!(resolved.line_width.is_finite() && resolved.line_width > 0.0);
            assert!(resolved.bracket_width.is_finite() && resolved.bracket_width > 0.0);
            assert_eq!(resolved.detail_font_size, 14);
            let (_, expected) = expected_preset(theme);
            assert_eq!(resolved.label_font_size, expected.axis_title_size);
            assert_eq!(resolved.composition_title_font_size, expected.title_size);
            assert_eq!(resolved.facet_header_font_size, expected.subtitle_size);
            assert!(text_roles.iter().all(|role| {
                !role.stack.trim().is_empty()
                    && role.font_size > 0
                    && role.line_height_multiplier.is_finite()
                    && role.line_height_multiplier > 0.0
            }));

            let frame = resolved.axes.frame.as_ref().expect("built-in frame");
            let ticks = resolved.axes.ticks.as_ref().expect("built-in ticks");
            let grid = resolved.axes.grid.as_ref().expect("built-in grid");
            assert!(frame.stroke.width.is_finite() && frame.stroke.width > 0.0);
            assert!(ticks.stroke.width.is_finite() && ticks.stroke.width > 0.0);
            assert!(grid.width.is_finite() && grid.width > 0.0);
            assert!(ticks.length > 0);
        }
    }

    #[test]
    fn built_in_foundational_paints_are_complete_rgba() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let foreground = Rgba::from(resolved.foreground);
            assert_eq!(resolved.surfaces.canvas_root, Rgba::new(255, 255, 255, 255));
            assert_eq!(resolved.surfaces.plot_panel, Rgba::new(255, 255, 255, 255));
            assert_eq!(resolved.typography.title.paint, foreground);
            assert_eq!(resolved.typography.subtitle.paint, Rgba::rgb(80, 80, 80));
            assert_eq!(resolved.typography.caption.paint, Rgba::rgb(80, 80, 80));
            assert_eq!(resolved.typography.axis_title.paint, foreground);
            assert_eq!(resolved.typography.tick_label.paint, foreground);
            assert_eq!(resolved.typography.significance_label.paint, foreground);
        }
    }

    #[test]
    fn built_in_line_heights_preserve_current_geometry() {
        for theme in BUILT_IN_THEMES {
            let typography = theme.resolve().typography;
            for role in [&typography.title, &typography.subtitle, &typography.caption] {
                let pitch = role.line_height_multiplier * f64::from(role.font_size);
                let expected = f64::from(role.font_size) + 6.0;
                assert!((pitch - expected).abs() < 1.0e-12, "{theme:?}");
            }
            for role in [
                &typography.axis_title,
                &typography.tick_label,
                &typography.significance_label,
            ] {
                assert!(
                    (role.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                    "{theme:?}"
                );
            }
        }
    }

    #[test]
    fn all_built_ins_center_the_root_title_block() {
        for theme in BUILT_IN_THEMES {
            assert_eq!(
                theme.resolve().composition.root_title_block.alignment,
                RootTitleBlockAlignment::Center,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn root_alignment_resolves_physical_x_and_anchor() {
        let span = (12.0, 92.0);
        assert_eq!(
            RootTitleBlockAlignment::Start.resolve_physical(span.0, span.1),
            (12.0, TextAnchor::Start)
        );
        assert_eq!(
            RootTitleBlockAlignment::Center.resolve_physical(span.0, span.1),
            (52.0, TextAnchor::Middle)
        );
        assert_eq!(
            RootTitleBlockAlignment::End.resolve_physical(span.0, span.1),
            (92.0, TextAnchor::End)
        );
    }

    #[test]
    fn built_in_axis_chrome_is_structural_and_preserves_sources() {
        for theme in BUILT_IN_THEMES {
            let resolved = theme.resolve();
            let (_, expected) = expected_preset(theme);
            let frame = resolved.axes.frame.expect("built-in frame");
            let ticks = resolved.axes.ticks.expect("built-in ticks");
            let grid = resolved.axes.grid.expect("built-in grid");

            assert_eq!(frame.stroke.paint, Rgba::from(expected.foreground));
            assert!((frame.stroke.width - expected.axis_width).abs() < f32::EPSILON);
            assert_eq!(frame.stroke.pattern, StrokePattern::Solid);
            assert_eq!(
                frame.source_show_top_right_spines,
                expected.top_right_spines
            );
            assert_eq!(ticks.stroke, frame.stroke);
            assert_eq!(ticks.length, 5);
            assert_eq!(grid.paint, Rgba::from(expected.grid_color));
            assert!((grid.width - 0.5).abs() < f32::EPSILON);
            assert_eq!(grid.pattern, StrokePattern::MediumDash);
        }
    }

    #[test]
    fn significance_label_defaults_and_exact_precedence_are_complete() {
        for theme in BUILT_IN_THEMES {
            let typography = theme.resolve().typography;
            let default = typography.effective_significance_label(None);
            assert_eq!(default.font_size, 16, "{theme:?}");
            assert_eq!(default.weight, TextWeight::Normal, "{theme:?}");
            assert!(
                (default.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                "{theme:?}"
            );

            let exact = SignificanceMarkStyle::new(23, TextWeight::Bold, 9.0)
                .expect("valid exact significance style");
            let effective = typography.effective_significance_label(Some(exact));
            assert_eq!(effective.stack, default.stack, "{theme:?}");
            assert_eq!(effective.paint, default.paint, "{theme:?}");
            assert_eq!(effective.font_size, 23, "{theme:?}");
            assert_eq!(effective.weight, TextWeight::Bold, "{theme:?}");
            assert!(
                (effective.line_height_multiplier - 1.2).abs() < f64::EPSILON,
                "{theme:?}"
            );
        }
    }

    #[test]
    fn composition_spacing_preserves_active_values() {
        for theme in BUILT_IN_THEMES {
            let composition = theme.resolve().composition;
            assert!(composition.root_title_block.title_subtitle_gap.abs() < f64::EPSILON);
            assert!(
                (composition
                    .root_title_block
                    .title_block_to_content_clearance
                    - 14.0)
                    .abs()
                    < f64::EPSILON
            );
            assert!(
                (composition.root_title_block.caption_to_content_clearance - 12.0).abs()
                    < f64::EPSILON
            );
        }
    }
}
