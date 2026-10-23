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
use crate::theme::TextTheme;
#[cfg(test)]
use crate::Error;

/// The smallest positive canonical span when a real child supplies no larger
/// intrinsic. It represents one spatial quantum, never a compatibility floor.
pub(crate) const MINIMUM_PANEL_SPAN: f64 = 1.0 / 1024.0;

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
        let mut requirements = Self::measure_header(axes, theme)?;
        if let Some(text) = top_text {
            let strip = theme.facet_strip_lengths()?;
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                text,
                &theme.typography.composition.facet_header,
                crate::theme::composition::capacity(
                    available_panel.width(),
                    &strip.horizontal_inset,
                )?,
                PhysicalTextAlignment::Center,
                "facet strip",
                &scope.child(crate::layout::AddressRole::FacetHeader, 0)?,
            ))?;
            let facet_top = crate::theme::composition::hard(
                &strip.content_clearance.add_pixels(plan.height())?,
            )?;
            requirements.top = requirements.top.max(facet_top);
        }
        Ok(requirements)
    }

    pub(crate) fn measure_header(
        axes: &PanelAxisPlan,
        theme: &ResolvedTheme,
    ) -> Result<Self, crate::layout::fit::PlanError> {
        let required = axes.requirements();

        let clearance = theme.panel_edge_length()?;
        let edge = |value| crate::theme::composition::hard(&clearance.add_pixels(value)?);
        Ok(Self {
            left: edge(required.left)?,
            right: edge(required.right)?,
            bottom: edge(required.bottom)?,
            top: edge(required.top)?,
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

/// A resolved canvas layout for one chart canvas.
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
    /// Recompose a selected scientific body with the same exact root chrome.
    /// Any added shared-axis bands are already canonical measured consequences.
    pub(crate) fn height_for_body(
        &self,
        body: f64,
        theme: &ResolvedTheme,
    ) -> Result<f64, crate::Error> {
        let padding = theme.root_padding_lengths()?;
        let bottom = self
            .root_text
            .reserves
            .bottom
            .add_length(&padding.bottom)
            .after_pixels(0.0)?;
        let top = self
            .root_text
            .reserves
            .top
            .add_length(&padding.top)
            .pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?;
        let additional = (self.insets.bottom - bottom) + (self.insets.top - top);
        crate::theme::composition::hard(
            &self
                .root_text
                .reserves
                .height_with_body(body + additional)?
                .add_length(&padding.vertical()),
        )
    }

    pub(crate) fn into_shared(self) -> std::sync::Arc<Self> {
        std::sync::Arc::new(self)
    }
    pub(crate) fn measure(
        canvas_size: (u32, u32),
        theme: &ResolvedTheme,
        text: [Option<&str>; 3],
        alignments: crate::decorations::title_block::plan::RootAlignments,
        minimum_body: (f64, f64),
    ) -> Result<Self, crate::layout::fit::PlanError> {
        let (canvas_width, canvas_height) = canvas_size;

        let outer = PanelBox::new(0.0, f64::from(canvas_width), 0.0, f64::from(canvas_height));
        let padding = theme.root_padding_lengths()?;
        let plan = crate::decorations::title_block::plan::RootTitleBlockPlan::measure(
            theme,
            text,
            outer,
            &padding,
            theme.root_inner_width(f64::from(canvas_width))?,
            alignments,
            minimum_body.1,
            &crate::layout::fit::FitScope::root(),
        )?;
        if theme.root_outer_width(minimum_body.0)? > f64::from(canvas_width) {
            return Err(crate::layout::fit::FitScope::root()
                .failure(
                    crate::layout::kernel::FitKind::TrackOrResidual,
                    "root padding and complete content exceed the assigned root",
                )
                .into());
        }
        let panel_box = plan.reserves.body(outer, &padding)?;
        let insets = CanvasInsets {
            left: panel_box.x0,
            right: outer.x1 - panel_box.x1,
            bottom: panel_box.y0,
            top: outer.y1 - panel_box.y1,
        };
        crate::layout::fit::measured(
            crate::layout::kernel::FrameRequirements {
                minimum_body,
                natural_insets: [0.0; 4],
            }
            .assign_fit(panel_box, [0.0; 4], &crate::layout::fit::FitScope::root()),
        )?;
        Ok(Self {
            panel_box,
            insets,
            root_text: plan,
            native_preference: None,
        })
    }
}

#[cfg(test)]
fn try_wrap_text(
    theme: &ResolvedTheme,
    text: &str,
    appearance: &TextTheme,
    maximum_width: f64,
    alignment: PhysicalTextAlignment,
    role: &str,
) -> Result<Vec<String>, Error> {
    theme
        .plan_text(text, appearance, maximum_width, alignment, role, false)
        .and_then(|plan| {
            plan.into_line_texts()
                .map_err(|error| Error::Render(error.to_string()))
        })
}

#[cfg(test)]
fn wrap_text(
    theme: &ResolvedTheme,
    text: &str,
    appearance: &TextTheme,
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
    fn root_padding_has_no_axis_estimate_or_unconditional_top_minimum() {
        let theme = Theme::publication().resolve().unwrap();
        let frame = RootFramePlan::measure(
            (100, 80),
            &theme,
            [None; 3],
            crate::decorations::title_block::plan::RootAlignments::resolve(
                &theme, [None; 2], [None; 2],
            ),
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .unwrap();
        assert_eq!(
            frame.insets,
            CanvasInsets {
                left: 16.0,
                right: 16.0,
                bottom: 8.0,
                top: 12.0
            }
        );
        assert_eq!(frame.panel_box, PanelBox::new(16.0, 84.0, 8.0, 68.0));
    }

    #[test]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "positive built-in insets plus the measured child minimum fit u32 after ceiling"
    )]
    fn measured_child_root_frame_rejects_residual_fit_without_clamping_or_text_fallback() {
        let theme = Theme::publication().resolve().unwrap();
        let insets = CanvasInsets {
            left: 16.0,
            right: 16.0,
            bottom: 8.0,
            top: 12.0,
        };
        let child_minimum = (13.0, 9.0);
        let width = (insets.left + insets.right + child_minimum.0).ceil() as u32;
        let height = (insets.bottom + insets.top + child_minimum.1).ceil() as u32;
        let measure = |width, height| {
            RootFramePlan::measure(
                (width, height),
                &theme,
                [None; 3],
                crate::decorations::title_block::plan::RootAlignments::resolve(
                    &theme, [None; 2], [None; 2],
                ),
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
        let theme = Theme::publication().resolve().unwrap();
        let caption =
            "diagnostic provenance with a deliberately long model and adjustment description";
        let alignments = crate::decorations::title_block::plan::RootAlignments::resolve(
            &theme, [None; 2], [None; 2],
        );
        let layout = RootFramePlan::measure(
            (360, 240),
            &theme,
            [None, None, Some(caption)],
            alignments,
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .unwrap();
        let lines = wrap_text(&theme, caption, &theme.typography.root.caption, 328.0);

        assert!(lines.len() > 1);
        assert_eq!(lines.concat(), caption);
        let plan = &layout.root_text;
        assert!(
            crate::theme::composition::hard(&plan.reserves.bottom).unwrap()
                > theme.typography.root.caption.line_pitch_pixels()
        );
        assert!(RootFramePlan::measure(
            (160, 120),
            &theme,
            [None, None, Some(&caption.repeat(8))],
            alignments,
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .is_err());
    }

    #[test]
    fn root_title_subtitle_gap_affects_reservation_and_position() {
        let base = Theme::publication().resolve().unwrap();
        let mut spaced = base.clone();
        spaced.composition = spaced
            .composition
            .clone()
            .root_title_block(
                spaced
                    .composition
                    .get_root_title_block()
                    .clone()
                    .title_subtitle_gap(crate::theme::RelativeSpacing::ratio(9, 20).unwrap())
                    .unwrap(),
            )
            .unwrap();
        let content = [Some("title"), Some("subtitle"), None];
        let alignments = crate::decorations::title_block::plan::RootAlignments::resolve(
            &base, [None; 2], [None; 2],
        );
        let base_layout = RootFramePlan::measure(
            (640, 480),
            &base,
            content,
            alignments,
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .expect("base title block fits");
        let spaced_layout = RootFramePlan::measure(
            (640, 480),
            &spaced,
            content,
            alignments,
            (MINIMUM_PANEL_SPAN, MINIMUM_PANEL_SPAN),
        )
        .expect("spaced title block fits");

        assert!((spaced_layout.insets.top - base_layout.insets.top - 9.0).abs() < f64::EPSILON);
        assert_ne!(base_layout.root_text, spaced_layout.root_text);
    }

    #[test]
    fn facet_measurement_uses_the_complete_selected_role_occupancy() {
        let mut theme = Theme::publication().resolve().unwrap();
        theme.typography.root.subtitle.font_size = 7;
        theme.typography.composition.facet_header = theme
            .typography
            .composition
            .facet_header
            .clone()
            .font_size(32)
            .unwrap();
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

        let strip = theme.facet_strip_lengths().unwrap();
        let plan = theme
            .plan_text(
                "facet",
                &theme.typography.composition.facet_header,
                crate::theme::composition::capacity(panel.width(), &strip.horizontal_inset)
                    .unwrap(),
                PhysicalTextAlignment::Center,
                "facet strip",
                false,
            )
            .unwrap();
        assert_eq!(
            requirements.top.to_bits(),
            crate::theme::composition::hard(
                &strip.content_clearance.add_pixels(plan.height()).unwrap()
            )
            .unwrap()
            .to_bits()
        );
    }

    #[test]
    fn wrapping_uses_shared_measurement_for_wide_combining_and_newline_text() {
        let theme = Theme::publication().resolve().unwrap();
        let mut appearance = theme.typography.annotations.panel_text.clone();
        appearance.font_size = 12;

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
