//! Canvas-level layout: canvas reservations and panel placement.
//!
//! `Layout` owns a plot panel's data range and projection. This module
//! owns the canvas geometry around that panel: title, subtitle, caption,
//! axis labels, tick labels, and future legend reservations.

use crate::Error;

#[cfg(test)]
use crate::layout::Layout;
use crate::layout::{PanelAxisPlan, PhysicalTextAlignment};
use crate::theme::ResolvedTheme;
#[cfg(test)]
use crate::theme::TextAppearance;

const MIN_PANEL_WIDTH: f64 = super::interior::DeferredLegacyPanelMinimum::SIZE.0;
const MIN_PANEL_HEIGHT: f64 = super::interior::DeferredLegacyPanelMinimum::SIZE.1;
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
        axes: &PanelAxisPlan,
        theme: &ResolvedTheme,
        top_text: Option<&str>,
        available_panel: PanelBox,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        let mut requirements =
            Self::measure_header(axes, theme, None, None, available_panel, scope)?;
        if let Some(text) = top_text {
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                text,
                &theme.legacy_text_appearance(
                    theme.facet_header_font_size,
                    crate::TextWeight::Normal,
                ),
                (available_panel.width() - 16.0).max(1.0),
                PhysicalTextAlignment::Center,
                "facet strip",
                &scope.child(crate::layout::AddressRole::FacetHeader, 0)?,
            ))?;
            let line_height = f64::from(theme.facet_header_font_size) + 6.0;
            let facet_top = plan
                .lines
                .iter()
                .fold(8.0, |height, _| height + line_height);
            requirements.top = requirements.top.max(facet_top);
        }
        Ok(requirements)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "wrapped line counts are bounded by the finite panel dimensions"
    )]
    pub(crate) fn measure_header(
        axes: &PanelAxisPlan,
        theme: &ResolvedTheme,
        top_title: Option<&str>,
        top_subtitle: Option<&str>,
        available_panel: PanelBox,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        use crate::layout::fit;
        let required = axes.requirements();

        let maximum_width = (available_panel.width() - 16.0).max(1.0);
        let title_lines = top_title.map_or(Ok(0), |text| {
            Ok::<_, fit::PlanError>(
                fit::measured(fit::text(
                    theme,
                    text,
                    &theme.typography.title,
                    maximum_width,
                    PhysicalTextAlignment::Start,
                    "panel title",
                    &scope.child(crate::layout::AddressRole::RootTitle, 0)?,
                ))?
                .lines
                .len(),
            )
        })?;
        let subtitle_lines = top_subtitle.map_or(Ok(0), |text| {
            Ok::<_, fit::PlanError>(
                fit::measured(fit::text(
                    theme,
                    text,
                    &theme.typography.subtitle,
                    maximum_width,
                    PhysicalTextAlignment::Start,
                    "panel subtitle",
                    &scope.child(crate::layout::AddressRole::RootSubtitle, 0)?,
                ))?
                .lines
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
            title_lines as f64 * theme.typography.title.line_height()
                + subtitle_lines as f64 * theme.typography.subtitle.line_height()
                + header_gap
                + 8.0
        };

        Ok(Self {
            left: required.left + 4.0,
            right: required.right + 4.0,
            bottom: required.bottom + 4.0,
            top: top.max(required.top + 4.0),
        })
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validated declared tracks fit the exact root coordinate domain"
)]
// Canvas parent adapter preservation boundary: declared equal legacy tracks,
// including sparse cells. T5b owns any measured-track replacement.
pub(crate) fn allocate_composed_grid_fit(
    outer: PanelBox,
    rows: usize,
    columns: usize,
    requirements: &[(usize, usize, PanelRequirements)],
    description: &str,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::fit::FitOutcome<Vec<PanelBox>>, Error> {
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
        return Ok(crate::layout::fit::FitOutcome::Miss(scope.failure(crate::layout::kernel::FitKind::TrackOrResidual, format!(
            "{description} is too dense for {rows}x{columns} panels after reserving measured decorations; available panel size is {panel_width:.1}x{panel_height:.1}"
        ))));
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

    Ok(crate::layout::fit::FitOutcome::Measured(
        (0..rows)
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
            .collect(),
    ))
}

/// Reserved canvas margins around the plot panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasInsets {
    /// Space reserved at the left of the canvas.
    pub left: f64,
    /// Space reserved at the right of the canvas.
    pub right: f64,
    /// Space reserved at the bottom of the canvas.
    pub bottom: f64,
    /// Space reserved at the top of the canvas.
    pub top: f64,
}

/// Caller-provided canvas content flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasLayoutSpec {
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

/// A resolved canvas layout for one chart canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasLayout {
    pub(crate) root_text: CanvasRootText,
    /// Plot panel available for data marks, axes, grid, and frame.
    pub panel_box: PanelBox,
    /// Resolved reserved margins.
    pub insets: CanvasInsets,
    /// Title baseline in math-style pixel coordinates.
    pub title_y: Option<f64>,
    /// Subtitle baseline in math-style pixel coordinates.
    pub subtitle_y: Option<f64>,
    /// Caption baseline in math-style pixel coordinates.
    pub caption_y: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "unmeasured geometry must not allocate a dummy text plan"
)]
pub(crate) enum CanvasRootText {
    Planned(crate::decorations::title_block::plan::RootTitleBlockPlan),
    Unmeasured,
}

impl CanvasLayout {
    pub(crate) fn measured_plot_root(
        canvas_width: u32,
        canvas_height: u32,
        theme: &ResolvedTheme,
        mut spec: CanvasLayoutSpec,
        text: (Option<&str>, Option<&str>, Option<&str>),
        alignments: crate::decorations::title_block::plan::RootAlignments,
        reserve_axes: bool,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        spec.has_title = false;
        spec.has_subtitle = false;
        spec.has_caption = false;
        let mut canvas = Self::resolve(
            canvas_width,
            canvas_height,
            theme,
            spec,
            (0, 0, 0),
            false,
            reserve_axes,
        )?;
        let span = PanelBox::new(
            16.0,
            f64::from(canvas_width) - 16.0,
            8.0,
            f64::from(canvas_height) - 12.0,
        );
        let plan = crate::decorations::title_block::plan::RootTitleBlockPlan::measure(
            theme,
            [text.0, text.1, text.2],
            span,
            alignments,
            MIN_PANEL_HEIGHT + canvas.insets.bottom - 8.0,
            &crate::layout::fit::FitScope::root(),
        )?;
        canvas.insets.top = (12.0 + plan.top_reserve).max(30.0);
        canvas.insets.bottom += plan.bottom_reserve;
        canvas.panel_box = PanelBox::new(
            canvas.insets.left,
            f64::from(canvas_width) - canvas.insets.right,
            canvas.insets.bottom,
            f64::from(canvas_height) - canvas.insets.top,
        );
        if canvas.panel_box.width() < MIN_PANEL_WIDTH
            || canvas.panel_box.height() < MIN_PANEL_HEIGHT
        {
            return Err(crate::layout::fit::FitScope::root()
                .failure(
                    crate::layout::fit::FitKind::TrackOrResidual,
                    "root title block leaves less than the minimum 80x60 plot panel",
                )
                .into());
        }
        canvas.root_text = CanvasRootText::Planned(plan);
        Ok(canvas)
    }

    /// Resolve a single-panel chart layout from canvas size, theme, and text flags.
    #[must_use]
    pub(crate) fn single_panel(
        canvas_width: u32,
        canvas_height: u32,
        theme: &ResolvedTheme,
        spec: CanvasLayoutSpec,
    ) -> Self {
        Self::resolve(
            canvas_width,
            canvas_height,
            theme,
            spec,
            (1, 1, 1),
            false,
            true,
        )
        .expect("unchecked canvas layout always clamps to the minimum panel")
    }

    #[cfg(test)]
    pub(crate) fn measured_composed_canvas(
        canvas_width: u32,
        canvas_height: u32,
        theme: &ResolvedTheme,
        spec: CanvasLayoutSpec,
        text: (Option<&str>, Option<&str>, Option<&str>),
    ) -> Result<Self, Error> {
        Self::measured_plot_root(
            canvas_width,
            canvas_height,
            theme,
            spec,
            text,
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme, [None; 2], [None; 2],
            ),
            false,
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }

    #[expect(
        clippy::cast_precision_loss,
        clippy::too_many_lines,
        reason = "measured line counts are bounded by the finite canvas dimensions"
    )]
    fn resolve(
        canvas_width: u32,
        canvas_height: u32,
        theme: &ResolvedTheme,
        spec: CanvasLayoutSpec,
        line_counts: (usize, usize, usize),
        checked: bool,
        reserve_axes: bool,
    ) -> Result<Self, Error> {
        let w = f64::from(canvas_width);
        let h = f64::from(canvas_height);

        let title_gap = if spec.has_title || spec.has_subtitle {
            theme
                .composition
                .root_title_block
                .title_block_to_content_clearance
        } else {
            0.0
        };
        let title_line_h = theme.typography.title.line_height();
        let subtitle_line_h = theme.typography.subtitle.line_height();
        let caption_line_h = theme.typography.caption.line_height();
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
        let title_subtitle_gap = if spec.has_title && spec.has_subtitle {
            theme.composition.root_title_block.title_subtitle_gap
        } else {
            0.0
        };
        let caption_h = if spec.has_caption {
            caption_line_h * line_counts.2.max(1) as f64
                + theme
                    .composition
                    .root_title_block
                    .caption_to_content_clearance
        } else {
            0.0
        };

        let tick_length = theme
            .axes
            .ticks
            .as_ref()
            .map_or(0.0, |ticks| f64::from(ticks.length));
        let tick_h = if reserve_axes {
            tick_length
                + theme.axes.spacing.x_tick_label_gap
                + theme.typography.tick_label.line_height()
        } else {
            0.0
        };
        let x_label_h = if spec.has_x_label {
            let gap = if reserve_axes {
                theme.axes.spacing.x_title_tick_label_gap
            } else {
                theme.axes.spacing.x_title_without_tick_labels_gap
            };
            gap + theme.typography.axis_title.line_height()
        } else {
            0.0
        };
        let y_tick_w = if reserve_axes {
            f64::from(theme.typography.tick_label.font_size) * 2.2
                + tick_length
                + theme.axes.spacing.y_tick_label_gap
        } else {
            0.0
        };
        let y_label_w = if spec.has_y_label {
            f64::from(theme.typography.axis_title.font_size)
                + theme.axes.spacing.y_title_edge_padding
        } else {
            0.0
        };

        let insets = CanvasInsets {
            left: if reserve_axes {
                (y_tick_w + y_label_w + 4.0).max(62.0)
            } else {
                16.0 + y_label_w
            },
            right: if reserve_axes { 30.0 } else { 16.0 },
            bottom: tick_h + x_label_h + caption_h + 8.0,
            top: (title_gap + title_h + title_subtitle_gap + subtitle_h + 12.0).max(30.0),
        };

        if checked
            && (w - insets.left - insets.right < MIN_PANEL_WIDTH
                || h - insets.bottom - insets.top < MIN_PANEL_HEIGHT)
        {
            return Err(Error::Render(format!(
                "measured canvas text does not fit on {canvas_width}x{canvas_height} canvas while preserving the minimum 80x60 plot panel"
            )));
        }
        let x0 = insets.left.min((w - MIN_PANEL_WIDTH).max(0.0));
        let x1 = (w - insets.right).max(x0 + MIN_PANEL_WIDTH).min(w);
        let y0 = insets.bottom.min((h - MIN_PANEL_HEIGHT).max(0.0));
        let y1 = (h - insets.top).max(y0 + MIN_PANEL_HEIGHT).min(h);
        let panel_box = PanelBox::new(x0, x1, y0, y1);

        let title_y = spec.has_title.then_some(h - 20.0);
        let subtitle_y = spec.has_subtitle.then_some(
            h - 20.0
                - if spec.has_title {
                    title_h + title_subtitle_gap
                } else {
                    0.0
                },
        );
        let caption_y = spec
            .has_caption
            .then_some(theme.typography.caption.line_height());

        Ok(Self {
            panel_box,
            insets,
            root_text: CanvasRootText::Unmeasured,
            title_y,
            subtitle_y,
            caption_y,
        })
    }
}

#[cfg(test)]
fn try_wrap_text(
    theme: &ResolvedTheme,
    text: &str,
    appearance: &TextAppearance,
    maximum_width: f64,
    alignment: PhysicalTextAlignment,
    role: &str,
) -> Result<Vec<String>, Error> {
    theme
        .plan_text(text, appearance, maximum_width, alignment, role, false)
        .and_then(|plan| {
            plan.into_line_texts(theme.prepared_font_environment()?)
                .map_err(|error| Error::Render(error.to_string()))
        })
}

#[cfg(test)]
fn wrap_text(
    theme: &ResolvedTheme,
    text: &str,
    appearance: &TextAppearance,
    maximum_width: f64,
) -> Vec<String> {
    try_wrap_text(
        theme,
        text,
        appearance,
        maximum_width,
        PhysicalTextAlignment::Start,
        "test text",
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn measured_caption_wraps_without_shortening_and_rejects_impossible_fit() {
        let theme = Theme::Publication.resolve();
        let caption =
            "diagnostic provenance with a deliberately long model and adjustment description";
        let spec = CanvasLayoutSpec {
            has_title: false,
            has_subtitle: false,
            has_caption: true,
            has_x_label: true,
            has_y_label: true,
        };
        let layout = CanvasLayout::measured_composed_canvas(
            360,
            240,
            &theme,
            spec,
            (None, None, Some(caption)),
        )
        .unwrap();
        let lines = wrap_text(&theme, caption, &theme.typography.caption, 328.0);

        assert!(lines.len() > 1);
        assert_eq!(lines.concat(), caption);
        let CanvasRootText::Planned(plan) = &layout.root_text else {
            panic!("common composition layout");
        };
        assert!(plan.bottom_reserve > theme.typography.caption.line_height());
        assert!(CanvasLayout::measured_composed_canvas(
            160,
            120,
            &theme,
            spec,
            (None, None, Some(&caption.repeat(8))),
        )
        .is_err());
    }

    #[test]
    fn composed_grid_retains_fixed_gap_between_adjacent_panels() {
        let cells = crate::layout::fit::measured(allocate_composed_grid_fit(
            PanelBox::new(0.0, 400.0, 0.0, 300.0),
            2,
            2,
            &[],
            "test grid",
            &crate::layout::fit::FitScope::root(),
        ))
        .unwrap();

        assert_eq!(
            (cells[1].x0 - cells[0].x1, cells[0].y0 - cells[2].y1),
            (COMPOSED_PANEL_GAP, COMPOSED_PANEL_GAP)
        );
    }

    #[test]
    fn root_title_subtitle_gap_affects_reservation_and_position() {
        let base = Theme::Publication.resolve();
        let mut spaced = base.clone();
        spaced.composition.root_title_block.title_subtitle_gap = 9.0;
        let spec = CanvasLayoutSpec {
            has_title: true,
            has_subtitle: true,
            has_caption: false,
            has_x_label: false,
            has_y_label: false,
        };
        let content = (Some("title"), Some("subtitle"), None);
        let base_layout = CanvasLayout::measured_composed_canvas(640, 480, &base, spec, content)
            .expect("base title block fits");
        let spaced_layout =
            CanvasLayout::measured_composed_canvas(640, 480, &spaced, spec, content)
                .expect("spaced title block fits");

        assert!((spaced_layout.insets.top - base_layout.insets.top - 9.0).abs() < f64::EPSILON);
        assert_ne!(base_layout.root_text, spaced_layout.root_text);
    }

    #[test]
    fn facet_measurement_uses_the_same_legacy_metrics_as_facet_drawing() {
        let mut theme = Theme::Publication.resolve();
        theme.typography.subtitle.font_size = 7;
        theme.facet_header_font_size = 32;
        let panel = PanelBox::new(20.0, 420.0, 20.0, 320.0);
        let layout = Layout {
            projection: crate::layout::Projection::new((0.0, 1.0), (0.0, 1.0), panel.as_tuple()),
            x_axis: crate::layout::Axis {
                label: String::new(),
                ticks: Vec::new(),
                explicit_ticks: false,
                assigned_tick_width: None,
                source: crate::layout::TickSource::SUPPLIED,
            },
            y_axis: crate::layout::Axis {
                label: String::new(),
                ticks: Vec::new(),
                explicit_ticks: false,
                assigned_tick_width: None,
                source: crate::layout::TickSource::SUPPLIED,
            },
        };

        let requirements = PanelRequirements::measure(
            &layout.plan_axes(&theme).unwrap(),
            &theme,
            Some("facet"),
            panel,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap();

        assert!((requirements.top - 46.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wrapping_uses_shared_measurement_for_wide_combining_and_newline_text() {
        let theme = Theme::Publication.resolve();
        let appearance = theme.legacy_text_appearance(12, crate::TextWeight::Normal);

        let explicit = wrap_text(&theme, "first line\nsecond line\n", &appearance, 200.0);
        assert_eq!(explicit, ["first line", "second line", ""]);

        let measured = theme
            .measure_text(
                "alpha beta gamma",
                &appearance,
                PhysicalTextAlignment::Start,
                "test text",
                false,
            )
            .unwrap();
        assert!(theme
            .plan_text(
                "alpha beta gamma",
                &appearance,
                measured.width() / 3.0,
                PhysicalTextAlignment::Start,
                "test text",
                false,
            )
            .is_err());

        let combined = theme
            .measure_text(
                "e\u{301}",
                &appearance,
                PhysicalTextAlignment::Start,
                "test text",
                false,
            )
            .unwrap();
        assert_eq!(combined.lines.len(), 1);
        assert!(!combined.lines[0].runs.is_empty());
    }
}
