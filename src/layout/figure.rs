//! Figure-level layout: canvas reservations and panel placement.
//!
//! `Layout` owns a plot panel's data range and projection. This module
//! owns the figure geometry around that panel: title, subtitle, caption,
//! axis labels, tick labels, and future legend reservations.

use crate::Error;

use crate::layout::Layout;
use crate::theme::ThemePalette;

const MIN_PANEL_WIDTH: f64 = 80.0;
const MIN_PANEL_HEIGHT: f64 = 60.0;
const COMPOSED_PANEL_GAP: f64 = 12.0;
pub(crate) const PANEL_HEADER_GAP: f64 = 4.0;

/// A plot panel rectangle in math-style pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelBox {
    /// Left edge of the plot panel.
    pub x0: f64,
    /// Right edge of the plot panel.
    pub x1: f64,
    /// Bottom edge of the plot panel.
    pub y0: f64,
    /// Top edge of the plot panel.
    pub y1: f64,
}

impl PanelBox {
    /// Construct a panel box from math-style pixel coordinates.
    #[must_use]
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64) -> Self {
        Self { x0, x1, y0, y1 }
    }

    /// Return the tuple form expected by [`crate::layout::Projection`].
    #[must_use]
    pub fn as_tuple(self) -> (f64, f64, f64, f64) {
        (self.x0, self.x1, self.y0, self.y1)
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(self) -> f64 {
        self.x1 - self.x0
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(self) -> f64 {
        self.y1 - self.y0
    }
}

/// Space used by one panel's axes and compact text outside its data box.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct PanelRequirements {
    pub(crate) left: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
    pub(crate) top: f64,
}

impl PanelRequirements {
    /// Measure decorations emitted by [`crate::layout::Axis`] around a panel.
    pub(crate) fn measure(
        layout: &Layout,
        palette: &ThemePalette,
        top_text: Option<&str>,
        available_panel: PanelBox,
    ) -> Result<Self, Error> {
        Self::measure_header(layout, palette, None, top_text, available_panel)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped line counts are bounded by the finite panel dimensions"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "one measurement pass keeps interdependent axis decoration geometry together"
    )]
    pub(crate) fn measure_header(
        layout: &Layout,
        palette: &ThemePalette,
        top_title: Option<&str>,
        top_subtitle: Option<&str>,
        available_panel: PanelBox,
    ) -> Result<Self, Error> {
        let tick_length = f64::from(palette.tick_length);
        let x_tick_width = available_panel.width() / layout.x_axis.ticks.len().max(1) as f64 * 0.9;
        let x_tick_blocks = layout
            .x_axis
            .ticks
            .iter()
            .filter(|tick| !tick.label.is_empty())
            .map(|tick| {
                crate::layout::TextBlock::layout(
                    &tick.label,
                    palette.tick_font_size,
                    x_tick_width.max(1.0),
                    "x-axis tick label",
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let y_tick_maximum_width = (available_panel.width() * 0.4).max(1.0);
        let y_tick_blocks = layout
            .y_axis
            .ticks
            .iter()
            .filter(|tick| !tick.label.is_empty())
            .map(|tick| {
                crate::layout::TextBlock::layout(
                    &tick.label,
                    palette.tick_font_size,
                    y_tick_maximum_width,
                    "y-axis tick label",
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let x_tick_half_width = if palette.show_tick_labels {
            x_tick_blocks
                .iter()
                .map(|block| block.width() / 2.0)
                .fold(0.0_f64, f64::max)
        } else {
            0.0
        };
        let y_tick_width = if palette.show_tick_labels {
            y_tick_blocks
                .iter()
                .map(crate::layout::TextBlock::width)
                .fold(0.0_f64, f64::max)
        } else {
            0.0
        };
        let x_label_block = (!layout.x_axis.label.is_empty())
            .then(|| {
                crate::layout::TextBlock::layout(
                    &layout.x_axis.label,
                    palette.label_font_size,
                    available_panel.width().max(1.0),
                    "x-axis title",
                )
            })
            .transpose()?;
        let y_label_block = (!layout.y_axis.label.is_empty())
            .then(|| {
                crate::layout::TextBlock::layout(
                    &layout.y_axis.label,
                    palette.label_font_size,
                    available_panel.height().max(1.0),
                    "y-axis title",
                )
            })
            .transpose()?;
        let x_label_overhang = if palette.show_x_label {
            ((x_label_block
                .as_ref()
                .map_or(0.0, crate::layout::TextBlock::width)
                - available_panel.width())
                / 2.0)
                .max(0.0)
        } else {
            0.0
        };
        let y_label_overhang = if palette.show_y_label {
            ((y_label_block
                .as_ref()
                .map_or(0.0, crate::layout::TextBlock::width)
                - available_panel.height())
                / 2.0)
                .max(0.0)
        } else {
            0.0
        };
        let y_tick_vertical_overhang = if palette.show_tick_labels {
            y_tick_blocks
                .iter()
                .map(|block| block.height() / 2.0)
                .fold(0.0_f64, f64::max)
        } else {
            0.0
        };

        let mut left = if palette.show_tick_labels {
            tick_length + 6.0 + y_tick_width
        } else {
            tick_length
        };
        if palette.show_y_label && !layout.y_axis.label.is_empty() {
            left += 16.0
                + y_label_block.as_ref().map_or(
                    f64::from(palette.label_font_size),
                    crate::layout::TextBlock::height,
                );
        }
        left = left.max(x_tick_half_width).max(x_label_overhang) + 4.0;

        let mut bottom = if palette.show_tick_labels {
            tick_length
                + 8.0
                + x_tick_blocks
                    .iter()
                    .map(crate::layout::TextBlock::height)
                    .fold(f64::from(palette.tick_font_size), f64::max)
        } else {
            tick_length + 4.0
        };
        if palette.show_x_label && !layout.x_axis.label.is_empty() {
            let extra_tick_height = (x_tick_blocks
                .iter()
                .map(crate::layout::TextBlock::height)
                .fold(0.0_f64, f64::max)
                - f64::from(palette.tick_font_size) * 1.2)
                .max(0.0);
            bottom = bottom.max(
                tick_length
                    + 30.0
                    + x_label_block.as_ref().map_or(
                        f64::from(palette.label_font_size),
                        crate::layout::TextBlock::height,
                    )
                    + extra_tick_height,
            );
        }
        bottom = bottom.max(y_label_overhang).max(y_tick_vertical_overhang) + 4.0;

        let maximum_width = (available_panel.width() - 16.0).max(1.0);
        let title_lines = top_title.map_or(Ok(0), |text| {
            Ok::<_, Error>(
                try_wrap_text(text, palette.title_font_size, maximum_width, "panel title")?.len(),
            )
        })?;
        let subtitle_lines = top_subtitle.map_or(Ok(0), |text| {
            Ok::<_, Error>(
                try_wrap_text(
                    text,
                    palette.subtitle_font_size,
                    maximum_width,
                    "panel subtitle or facet strip",
                )?
                .len(),
            )
        })?;
        let header_gap = if title_lines > 0 && subtitle_lines > 0 {
            PANEL_HEADER_GAP
        } else {
            0.0
        };
        let top = if title_lines == 0 && subtitle_lines == 0 {
            4.0
        } else {
            title_lines as f64 * (f64::from(palette.title_font_size) + 6.0)
                + subtitle_lines as f64 * (f64::from(palette.subtitle_font_size) + 6.0)
                + header_gap
                + 8.0
        };

        Ok(Self {
            left,
            right: x_tick_half_width.max(x_label_overhang) + 4.0,
            bottom,
            top: top
                .max(y_label_overhang + 4.0)
                .max(y_tick_vertical_overhang + 4.0),
        })
    }
}

/// Allocate equal data panels with measured decoration gutters and one safety gap.
#[expect(
    clippy::cast_precision_loss,
    reason = "validated grid dimensions are projected into finite canvas coordinates"
)]
pub(crate) fn allocate_composed_grid(
    outer: PanelBox,
    rows: usize,
    columns: usize,
    requirements: &[(usize, usize, PanelRequirements)],
    description: &str,
) -> Result<Vec<PanelBox>, Error> {
    if rows == 0 || columns == 0 {
        return Err(Error::Render(format!(
            "{description} requires at least one row and column"
        )));
    }
    let mut column_left = vec![0.0_f64; columns];
    let mut column_right = vec![0.0_f64; columns];
    let mut row_bottom = vec![0.0_f64; rows];
    let mut row_top = vec![0.0_f64; rows];
    for &(row, column, requirement) in requirements {
        if row >= rows || column >= columns {
            return Err(Error::Render(format!(
                "{description} requirement ({row}, {column}) is outside {rows}x{columns} grid"
            )));
        }
        column_left[column] = column_left[column].max(requirement.left);
        column_right[column] = column_right[column].max(requirement.right);
        row_bottom[row] = row_bottom[row].max(requirement.bottom);
        row_top[row] = row_top[row].max(requirement.top);
    }

    let horizontal_reservation = column_left.iter().sum::<f64>()
        + column_right.iter().sum::<f64>()
        + columns.saturating_sub(1) as f64 * COMPOSED_PANEL_GAP;
    let vertical_reservation = row_bottom.iter().sum::<f64>()
        + row_top.iter().sum::<f64>()
        + rows.saturating_sub(1) as f64 * COMPOSED_PANEL_GAP;
    let panel_width = (outer.width() - horizontal_reservation) / columns as f64;
    let panel_height = (outer.height() - vertical_reservation) / rows as f64;
    if panel_width < MIN_PANEL_WIDTH || panel_height < MIN_PANEL_HEIGHT {
        return Err(Error::Render(format!(
            "{description} is too dense for {rows}x{columns} panels after reserving measured decorations; available panel size is {panel_width:.1}x{panel_height:.1}"
        )));
    }

    let mut x_positions = Vec::with_capacity(columns);
    let mut cursor = outer.x0;
    for column in 0..columns {
        cursor += column_left[column];
        x_positions.push(cursor);
        cursor += panel_width + column_right[column];
        if column + 1 < columns {
            cursor += COMPOSED_PANEL_GAP;
        }
    }
    let mut y_positions = Vec::with_capacity(rows);
    let mut cursor = outer.y1;
    for row in 0..rows {
        cursor -= row_top[row];
        y_positions.push(cursor);
        cursor -= panel_height + row_bottom[row];
        if row + 1 < rows {
            cursor -= COMPOSED_PANEL_GAP;
        }
    }

    Ok((0..rows)
        .flat_map(|row| {
            let x_positions = &x_positions;
            let y_positions = &y_positions;
            (0..columns).map(move |column| {
                PanelBox::new(
                    x_positions[column],
                    x_positions[column] + panel_width,
                    y_positions[row] - panel_height,
                    y_positions[row],
                )
            })
        })
        .collect())
}

/// Reserved figure margins around the plot panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FigureInsets {
    /// Space reserved at the left of the canvas.
    pub left: f64,
    /// Space reserved at the right of the canvas.
    pub right: f64,
    /// Space reserved at the bottom of the canvas.
    pub bottom: f64,
    /// Space reserved at the top of the canvas.
    pub top: f64,
}

/// Caller-provided figure content flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FigureLayoutSpec {
    /// Reserve space for a title.
    pub has_title: bool,
    /// Reserve space for a subtitle.
    pub has_subtitle: bool,
    /// Reserve space for a caption.
    pub has_caption: bool,
    /// Reserve space for the x-axis label.
    pub has_x_label: bool,
    /// Reserve space for the y-axis label.
    pub has_y_label: bool,
}

/// A resolved figure layout for one chart canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct FigureLayout {
    /// Plot panel available for data marks, axes, grid, and frame.
    pub panel_box: PanelBox,
    /// Resolved reserved margins.
    pub insets: FigureInsets,
    /// Title baseline in math-style pixel coordinates.
    pub title_y: Option<f64>,
    /// Subtitle baseline in math-style pixel coordinates.
    pub subtitle_y: Option<f64>,
    /// Caption baseline in math-style pixel coordinates.
    pub caption_y: Option<f64>,
    pub(crate) title_lines_y: Vec<f64>,
    pub(crate) subtitle_lines_y: Vec<f64>,
    pub(crate) caption_lines_y: Vec<f64>,
}

impl FigureLayout {
    /// Resolve a single-panel chart layout from canvas size, theme, and text flags.
    #[must_use]
    pub fn single_panel(
        canvas_width: u32,
        canvas_height: u32,
        palette: &ThemePalette,
        spec: FigureLayoutSpec,
    ) -> Self {
        Self::resolve(
            canvas_width,
            canvas_height,
            palette,
            spec,
            (1, 1, 1),
            false,
            true,
        )
        .expect("unchecked figure layout always clamps to the minimum panel")
    }

    pub(crate) fn measured_single_panel(
        canvas_width: u32,
        canvas_height: u32,
        palette: &ThemePalette,
        spec: FigureLayoutSpec,
        text: (Option<&str>, Option<&str>, Option<&str>),
    ) -> Result<Self, Error> {
        let maximum_width = (f64::from(canvas_width) - 32.0).max(1.0);
        let counts = measured_line_counts(text, palette, maximum_width)?;
        Self::resolve(
            canvas_width,
            canvas_height,
            palette,
            spec,
            counts,
            true,
            true,
        )
    }

    pub(crate) fn measured_composed_canvas(
        canvas_width: u32,
        canvas_height: u32,
        palette: &ThemePalette,
        spec: FigureLayoutSpec,
        text: (Option<&str>, Option<&str>, Option<&str>),
    ) -> Result<Self, Error> {
        let maximum_width = (f64::from(canvas_width) - 32.0).max(1.0);
        let counts = measured_line_counts(text, palette, maximum_width)?;
        Self::resolve(
            canvas_width,
            canvas_height,
            palette,
            spec,
            counts,
            true,
            false,
        )
    }

    #[expect(
        clippy::cast_precision_loss,
        clippy::too_many_lines,
        reason = "measured line counts are bounded by the finite canvas dimensions"
    )]
    fn resolve(
        canvas_width: u32,
        canvas_height: u32,
        palette: &ThemePalette,
        spec: FigureLayoutSpec,
        line_counts: (usize, usize, usize),
        checked: bool,
        reserve_axes: bool,
    ) -> Result<Self, Error> {
        let w = f64::from(canvas_width);
        let h = f64::from(canvas_height);

        let title_gap = if spec.has_title || spec.has_subtitle {
            14.0
        } else {
            0.0
        };
        let title_line_h = f64::from(palette.title_font_size) + 6.0;
        let subtitle_line_h = f64::from(palette.subtitle_font_size) + 6.0;
        let caption_line_h = f64::from(palette.caption_font_size) + 6.0;
        let title_h = if spec.has_title {
            title_line_h * line_counts.0.max(1) as f64
        } else {
            0.0
        };
        let subtitle_h = if spec.has_subtitle {
            subtitle_line_h * line_counts.1.max(1) as f64
        } else {
            0.0
        };
        let caption_h = if spec.has_caption {
            caption_line_h * line_counts.2.max(1) as f64 + 12.0
        } else {
            0.0
        };

        let tick_h = if reserve_axes && palette.show_tick_labels {
            f64::from(palette.tick_font_size) + f64::from(palette.tick_length) + 8.0
        } else if reserve_axes {
            f64::from(palette.tick_length) + 4.0
        } else {
            0.0
        };
        let x_label_h = if spec.has_x_label && palette.show_x_label {
            f64::from(palette.label_font_size) + 14.0
        } else {
            0.0
        };
        let y_tick_w = if reserve_axes && palette.show_tick_labels {
            f64::from(palette.tick_font_size) * 2.2 + f64::from(palette.tick_length) + 6.0
        } else if reserve_axes {
            f64::from(palette.tick_length) + 4.0
        } else {
            0.0
        };
        let y_label_w = if spec.has_y_label && palette.show_y_label {
            f64::from(palette.label_font_size) + 12.0
        } else {
            0.0
        };

        let insets = FigureInsets {
            left: if reserve_axes {
                (y_tick_w + y_label_w + 4.0).max(62.0)
            } else {
                16.0 + y_label_w
            },
            right: if reserve_axes { 30.0 } else { 16.0 },
            bottom: tick_h + x_label_h + caption_h + 8.0,
            top: (title_gap + title_h + subtitle_h + 12.0).max(30.0),
        };

        if checked
            && (w - insets.left - insets.right < MIN_PANEL_WIDTH
                || h - insets.bottom - insets.top < MIN_PANEL_HEIGHT)
        {
            return Err(Error::Render(format!(
                "measured figure text does not fit on {canvas_width}x{canvas_height} canvas while preserving the minimum 80x60 plot panel"
            )));
        }
        let x0 = insets.left.min((w - MIN_PANEL_WIDTH).max(0.0));
        let x1 = (w - insets.right).max(x0 + MIN_PANEL_WIDTH).min(w);
        let y0 = insets.bottom.min((h - MIN_PANEL_HEIGHT).max(0.0));
        let y1 = (h - insets.top).max(y0 + MIN_PANEL_HEIGHT).min(h);
        let panel_box = PanelBox::new(x0, x1, y0, y1);

        let title_y = spec.has_title.then_some(h - 20.0);
        let subtitle_y = spec
            .has_subtitle
            .then_some(h - 20.0 - if spec.has_title { title_h } else { 0.0 });
        let caption_y = spec
            .has_caption
            .then_some(f64::from(palette.caption_font_size) + 6.0);

        let title_lines_y = title_y.map_or_else(Vec::new, |first| {
            (0..line_counts.0.max(1))
                .map(|index| first - index as f64 * title_line_h)
                .collect()
        });
        let subtitle_lines_y = subtitle_y.map_or_else(Vec::new, |first| {
            (0..line_counts.1.max(1))
                .map(|index| first - index as f64 * subtitle_line_h)
                .collect()
        });
        let caption_lines_y = caption_y.map_or_else(Vec::new, |last| {
            (0..line_counts.2.max(1))
                .map(|index| last + (line_counts.2.max(1) - 1 - index) as f64 * caption_line_h)
                .collect()
        });

        Ok(Self {
            panel_box,
            insets,
            title_y,
            subtitle_y,
            caption_y,
            title_lines_y,
            subtitle_lines_y,
            caption_lines_y,
        })
    }
}

fn measured_line_counts(
    text: (Option<&str>, Option<&str>, Option<&str>),
    palette: &ThemePalette,
    maximum_width: f64,
) -> Result<(usize, usize, usize), Error> {
    Ok((
        text.0.map_or(Ok(0), |value| {
            Ok::<_, Error>(
                try_wrap_text(
                    value,
                    palette.title_font_size,
                    maximum_width,
                    "figure title",
                )?
                .len(),
            )
        })?,
        text.1.map_or(Ok(0), |value| {
            Ok::<_, Error>(
                try_wrap_text(
                    value,
                    palette.subtitle_font_size,
                    maximum_width,
                    "figure subtitle",
                )?
                .len(),
            )
        })?,
        text.2.map_or(Ok(0), |value| {
            Ok::<_, Error>(
                try_wrap_text(
                    value,
                    palette.caption_font_size,
                    maximum_width,
                    "figure caption",
                )?
                .len(),
            )
        })?,
    ))
}

pub(crate) fn try_wrap_text(
    text: &str,
    font_size: u32,
    maximum_width: f64,
    role: &str,
) -> Result<Vec<String>, Error> {
    crate::layout::TextBlock::layout(text, font_size, maximum_width, role)
        .map(crate::layout::TextBlock::into_lines)
}

#[cfg(test)]
fn wrap_text(text: &str, font_size: u32, maximum_width: f64) -> Vec<String> {
    try_wrap_text(text, font_size, maximum_width, "test text").unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn measured_caption_wraps_without_shortening_and_rejects_impossible_fit() {
        let palette = Theme::Publication.palette();
        let caption =
            "diagnostic provenance with a deliberately long model and adjustment description";
        let spec = FigureLayoutSpec {
            has_title: false,
            has_subtitle: false,
            has_caption: true,
            has_x_label: true,
            has_y_label: true,
        };
        let layout = FigureLayout::measured_single_panel(
            360,
            240,
            &palette,
            spec,
            (None, None, Some(caption)),
        )
        .unwrap();
        let lines = wrap_text(caption, palette.caption_font_size, 328.0);

        assert!(lines.len() > 1);
        assert_eq!(lines.concat(), caption);
        assert_eq!(layout.caption_lines_y.len(), lines.len());
        assert!(FigureLayout::measured_single_panel(
            160,
            120,
            &palette,
            spec,
            (None, None, Some(&caption.repeat(8))),
        )
        .is_err());
    }

    #[test]
    fn composed_grid_retains_fixed_gap_between_adjacent_panels() {
        let cells = allocate_composed_grid(
            PanelBox::new(0.0, 400.0, 0.0, 300.0),
            2,
            2,
            &[],
            "test grid",
        )
        .unwrap();

        assert_eq!(
            (cells[1].x0 - cells[0].x1, cells[0].y0 - cells[2].y1),
            (COMPOSED_PANEL_GAP, COMPOSED_PANEL_GAP)
        );
    }

    #[test]
    fn wrapping_uses_shared_measurement_for_wide_combining_and_newline_text() {
        let cases = [
            ("中文字符边界", 24.0),
            ("wide🙂glyph🙂bounds", 42.0),
            ("e\u{301}e\u{301}e\u{301}e\u{301}", 14.0),
        ];
        for (text, width) in cases {
            let lines = wrap_text(text, 12, width);
            assert!(crate::layout::deterministic_text_width(text, 12.0) > width);
            assert_eq!(lines, [text]);
        }

        let explicit = wrap_text("first line\n第二行\n", 12, 200.0);
        assert_eq!(explicit, ["first line", "第二行", ""]);
        assert!(explicit
            .iter()
            .all(|line| { crate::layout::deterministic_text_width(line, 12.0) <= 200.0 }));

        let word_width = crate::layout::deterministic_text_width("alpha beta ", 12.0);
        assert_eq!(
            wrap_text("alpha beta gamma", 12, word_width),
            ["alpha beta ", "gamma"]
        );
        assert!(
            (crate::layout::deterministic_text_width("e\u{301}", 12.0)
                - crate::layout::deterministic_text_width("e", 12.0))
            .abs()
                < f64::EPSILON
        );
        assert!(
            crate::layout::deterministic_text_width("中", 12.0)
                > crate::layout::deterministic_text_width("a", 12.0)
        );
    }
}
