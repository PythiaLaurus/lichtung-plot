//! Per-call dimensions, distinct from the root's completed output size.

use crate::layout::PixelRect;
use crate::layout::RootSize;
use crate::orchestrator::registration::{BaseAxisRef, DisplayExtent};

/// A selected domain owns scale coordinates, never authored raw endpoints.
#[derive(Debug)]
pub(crate) struct AxisDomainAssignment {
    pub(crate) extent: DisplayExtent,
    pub(crate) provenance: crate::layout::RangeProvenance,
}

#[derive(Clone, Copy)]
pub(crate) struct AssignedAxisRef<'v> {
    pub(crate) base: BaseAxisRef<'v>,
    pub(crate) domain: &'v AxisDomainAssignment,
}

/// The selected allocation remains distinct from its completed root frame.
pub(crate) struct PlotMeasureAssignment {
    pub(crate) dimensions: PlotMeasureDimensions,
    pub(crate) frame: PixelRect,
    pub(crate) content_height: bool,
    pub(crate) fill_content_height: bool,
    pub(crate) content_preference: Option<f64>,
    pub(crate) insets: ParentInsets,
}

pub(crate) struct ParentInsets {
    pub(crate) axis_gutter: [i64; 4],
    pub(crate) owned_ink: [i64; 4],
}

impl ParentInsets {
    pub(crate) fn consumed(
        round: &crate::layout::kernel::FeedbackRound,
        address: &crate::layout::ComponentAddress,
    ) -> Self {
        use crate::layout::kernel::FeedbackKind;
        Self {
            axis_gutter: round.insets_quanta(address, FeedbackKind::AxisGutter),
            owned_ink: round.insets_quanta(address, FeedbackKind::OwnedInk),
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "consumed insets retain the original separate exact-root-bounded pixel conversions before addition"
    )]
    pub(crate) fn assigned_gutters(&self) -> [f64; 4] {
        let ink = self.owned_ink_pixels();
        std::array::from_fn(|side| self.axis_gutter[side] as f64 / 1024.0 + ink[side])
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "consumed owned ink is exact-root-bounded layout quanta"
    )]
    pub(crate) fn owned_ink_pixels(&self) -> [f64; 4] {
        self.owned_ink.map(|value| value as f64 / 1024.0)
    }
}

pub(crate) struct PlotFrameHeightPreference {
    pub(crate) height: f64,
}

pub(crate) fn completion_frame(
    kind: crate::PlotKind,
    selected: &PlotMeasureAssignment,
    result: &crate::spec::PlotRenderResult,
) -> PixelRect {
    let frame = selected.frame;
    if selected.fill_content_height {
        PixelRect::new(
            frame.x0,
            frame.x1,
            frame.y0,
            frame.y0 + selected.content_preference.unwrap_or(frame.height()),
        )
    } else if selected.content_height
        && matches!(
            kind,
            crate::PlotKind::Heatmap | crate::PlotKind::Beeswarm | crate::PlotKind::Matrix
        )
    {
        PixelRect::new(
            frame.x0,
            frame.x1,
            frame.y0,
            result.layout.projection.panel_box().y1 + selected.insets.assigned_gutters()[3],
        )
    } else {
        frame
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotMeasureDimensions {
    pub(crate) canvas: RootSize,
    pub(crate) output_override: Option<RootSize>,
}

impl PlotMeasureDimensions {
    pub(crate) const fn canvas_size(self) -> RootSize {
        self.canvas
    }

    pub(crate) fn figure_size(self) -> RootSize {
        self.output_override.unwrap_or(self.canvas_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_keeps_the_selected_origin_and_original_fill_and_native_top_branches() {
        let frame = PixelRect::new(5.0, 105.0, 20.0, 220.0);
        let result = crate::spec::PlotRenderResult::new(
            crate::layout::Layout::envelope(PixelRect::new(10.0, 100.0, 30.0, 80.0)),
            None,
        );
        let mut selected = PlotMeasureAssignment {
            dimensions: PlotMeasureDimensions {
                canvas: RootSize::new(800, 600),
                output_override: None,
            },
            frame,
            content_height: true,
            fill_content_height: true,
            content_preference: Some(25.5),
            insets: ParentInsets {
                axis_gutter: [0, 0, 0, 3328],
                owned_ink: [0, 0, 0, 1536],
            },
        };
        assert_eq!(
            completion_frame(crate::PlotKind::Scatter, &selected, &result),
            PixelRect::new(5.0, 105.0, 20.0, 45.5)
        );
        selected.content_preference = None;
        assert_eq!(
            completion_frame(crate::PlotKind::Heatmap, &selected, &result),
            frame
        );
        selected.fill_content_height = false;
        for kind in [
            crate::PlotKind::Heatmap,
            crate::PlotKind::Beeswarm,
            crate::PlotKind::Matrix,
        ] {
            assert_eq!(
                completion_frame(kind, &selected, &result),
                PixelRect::new(5.0, 105.0, 20.0, 84.75)
            );
        }
        assert_eq!(
            completion_frame(crate::PlotKind::Scatter, &selected, &result),
            frame
        );
        selected.content_height = false;
        assert_eq!(
            completion_frame(crate::PlotKind::Heatmap, &selected, &result),
            frame
        );
    }
}
