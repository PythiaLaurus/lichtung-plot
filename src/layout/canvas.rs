//! Canvas-level layout: canvas reservations and panel placement.
//!
//! `Layout` owns a plot panel's data range and projection. This module
//! owns the canvas geometry around that panel: title, subtitle, caption,
//! axis labels, tick labels, and future legend reservations.

#[cfg(test)]
use crate::layout::Layout;
use crate::layout::{PanelAxisPlan, PhysicalTextAlignment};
use crate::theme::ResolvedTheme;
#[cfg(test)]
use crate::theme::TextAppearance;
#[cfg(test)]
use crate::Error;

/// The smallest positive canonical span when a real child supplies no larger
/// intrinsic. It represents one spatial quantum, never a compatibility floor.
pub(crate) const MINIMUM_PANEL_SPAN: f64 = 1.0 / 1024.0;
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

/// Axis reservations retained by the specialized and composition adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LegacyRootAxes {
    /// Reserve space for the x-axis label.
    pub has_x_label: bool,
    /// Reserve space for the y-axis label.
    pub has_y_label: bool,
}

/// A resolved canvas layout for one chart canvas.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootFramePlan {
    pub(crate) root_text: crate::decorations::title_block::plan::RootTitleBlockPlan,
    /// Plot panel available for data marks, axes, grid, and frame.
    pub panel_box: PanelBox,
    /// Resolved reserved margins.
    pub insets: CanvasInsets,
    pub(crate) native_preference:
        Option<crate::layout::interior::preference::NativePanelPreference>,
}

impl RootFramePlan {
    pub(crate) fn measured_plot_root(
        canvas_size: (u32, u32),
        theme: &ResolvedTheme,
        spec: LegacyRootAxes,
        text: (Option<&str>, Option<&str>, Option<&str>),
        alignments: crate::decorations::title_block::plan::RootAlignments,
        reserve_axes: bool,
        minimum_body: (f64, f64),
    ) -> Result<Self, crate::layout::fit::PlanError> {
        let (canvas_width, canvas_height) = canvas_size;
        let mut insets = spec.insets(theme, reserve_axes);
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
            minimum_body.1 + insets.bottom - 8.0,
            &crate::layout::fit::FitScope::root(),
        )?;
        insets.top = (12.0 + plan.top_reserve).max(30.0);
        insets.bottom += plan.bottom_reserve;
        let panel_box = PanelBox::new(
            insets.left,
            f64::from(canvas_width) - insets.right,
            insets.bottom,
            f64::from(canvas_height) - insets.top,
        );
        if panel_box.width() < minimum_body.0 || panel_box.height() < minimum_body.1 {
            return Err(crate::layout::fit::FitScope::root()
                .failure(
                    crate::layout::fit::FitKind::TrackOrResidual,
                    "root title block leaves less than the measured child body minimum",
                )
                .into());
        }
        Ok(Self {
            panel_box,
            insets,
            root_text: plan,
            native_preference: None,
        })
    }

    #[cfg(test)]
    pub(crate) fn measured_composed_canvas(
        canvas_width: u32,
        canvas_height: u32,
        theme: &ResolvedTheme,
        spec: LegacyRootAxes,
        text: (Option<&str>, Option<&str>, Option<&str>),
    ) -> Result<Self, Error> {
        Self::measured_plot_root(
            (canvas_width, canvas_height),
            theme,
            spec,
            text,
            crate::decorations::title_block::plan::RootAlignments::resolve(
                theme, [None; 2], [None; 2],
            ),
            false,
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .map_err(crate::layout::fit::PlanError::into_error)
    }
}

impl LegacyRootAxes {
    // T5b/T5e retain this axis/packing estimate, not an alternate root layout.
    // Root text is always measured separately and never estimated here.
    pub(crate) fn insets(self, theme: &ResolvedTheme, reserve_axes: bool) -> CanvasInsets {
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
        let x_label_h = if self.has_x_label {
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
        let y_label_w = if self.has_y_label {
            f64::from(theme.typography.axis_title.font_size)
                + theme.axes.spacing.y_title_edge_padding
        } else {
            0.0
        };

        CanvasInsets {
            left: if reserve_axes {
                (y_tick_w + y_label_w + 4.0).max(62.0)
            } else {
                16.0 + y_label_w
            },
            right: if reserve_axes { 30.0 } else { 16.0 },
            bottom: tick_h + x_label_h + 8.0,
            top: 30.0,
        }
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
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "positive built-in insets plus the measured child minimum fit u32 after ceiling"
    )]
    fn measured_child_root_frame_rejects_residual_fit_without_clamping_or_text_fallback() {
        let theme = Theme::Publication.resolve();
        let spec = LegacyRootAxes {
            has_x_label: true,
            has_y_label: true,
        };
        let insets = spec.insets(&theme, true);
        let child_minimum = (13.0, 9.0);
        let width = (insets.left + insets.right + child_minimum.0).ceil() as u32;
        let height = (insets.bottom + insets.top + child_minimum.1).ceil() as u32;
        let measure = |width, height| {
            RootFramePlan::measured_plot_root(
                (width, height),
                &theme,
                spec,
                (None, None, None),
                crate::decorations::title_block::plan::RootAlignments::resolve(
                    &theme, [None; 2], [None; 2],
                ),
                true,
                child_minimum,
            )
        };
        let frame = measure(width, height).unwrap();
        assert_eq!(frame.panel_box.x0.to_bits(), insets.left.to_bits());
        assert_eq!(
            frame.panel_box.x1.to_bits(),
            (f64::from(width) - insets.right).to_bits()
        );
        for result in [measure(width - 1, height), measure(width, height - 1)] {
            assert!(matches!(result, Err(crate::layout::fit::PlanError::Fit(_))));
        }
    }

    #[test]
    fn measured_caption_wraps_without_shortening_and_rejects_impossible_fit() {
        let theme = Theme::Publication.resolve();
        let caption =
            "diagnostic provenance with a deliberately long model and adjustment description";
        let spec = LegacyRootAxes {
            has_x_label: true,
            has_y_label: true,
        };
        let layout = RootFramePlan::measured_composed_canvas(
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
        let plan = &layout.root_text;
        assert!(plan.bottom_reserve > theme.typography.caption.line_height());
        assert!(RootFramePlan::measured_composed_canvas(
            160,
            120,
            &theme,
            spec,
            (None, None, Some(&caption.repeat(8))),
        )
        .is_err());
    }

    #[test]
    fn root_title_subtitle_gap_affects_reservation_and_position() {
        let base = Theme::Publication.resolve();
        let mut spaced = base.clone();
        spaced.composition.root_title_block.title_subtitle_gap = 9.0;
        let spec = LegacyRootAxes {
            has_x_label: false,
            has_y_label: false,
        };
        let content = (Some("title"), Some("subtitle"), None);
        let base_layout = RootFramePlan::measured_composed_canvas(640, 480, &base, spec, content)
            .expect("base title block fits");
        let spaced_layout =
            RootFramePlan::measured_composed_canvas(640, 480, &spaced, spec, content)
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
                assigned_cells: None,
                source: crate::layout::TickSource::SUPPLIED,
            },
            y_axis: crate::layout::Axis {
                label: String::new(),
                ticks: Vec::new(),
                explicit_ticks: false,
                assigned_tick_width: None,
                assigned_cells: None,
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
