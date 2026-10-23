//! One complete embedded Plot title/subtitle/caption block in its Canvas cell.

use crate::builder::Plot;
use crate::decorations::title_block::plan::{
    RootAlignments, RootTitleBlockIntrinsics, RootTitleBlockPlan,
};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
use crate::layout::{AddressRole, PanelBox};
use crate::theme::ResolvedTheme;
use crate::{Error, TextAlignment};

pub(super) struct PlotLocalTextPlan {
    pub(super) text: RootTitleBlockPlan,
    pub(super) body: PanelBox,
}

impl PlotLocalTextPlan {
    pub(super) fn measure(
        plot: &Plot,
        cell: PanelBox,
        theme: &ResolvedTheme,
        panel_alignments: [Option<TextAlignment>; 2],
        intrinsics: RootTitleBlockIntrinsics,
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        let sources = [plot.title_text(), plot.subtitle_text(), plot.caption_text()];
        let present = sources.iter().any(Option::is_some);

        let padding = if present {
            theme.root_padding_lengths()?
        } else {
            crate::theme::composition::RootPaddingLengths::zero()?
        };
        let span = padding.inset(cell)?;
        let maximum_width = if present {
            theme.root_inner_width(cell.width())?
        } else {
            cell.width()
        };
        if span.width() <= 0.0 || span.height() <= 0.0 {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::RootBody, 0)?.failure(
                    FitKind::TrackOrResidual,
                    "Canvas Plot local text leaves no content span",
                ),
            ));
        }
        if let FitOutcome::Miss(reason) =
            fit::outcome(intrinsics.check_width(maximum_width.min(span.width()), scope))?
        {
            return Ok(FitOutcome::Miss(reason));
        }
        let minimum = plot.intrinsic_body_minimum(theme)?;
        let alignments = RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            panel_alignments,
        );
        let text = match fit::outcome(RootTitleBlockPlan::measure(
            theme,
            sources,
            cell,
            &padding,
            maximum_width,
            alignments,
            minimum.1,
            scope,
        ))? {
            FitOutcome::Measured(text) => text,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let reserved = text.reserves.body(cell, &padding)?;
        let body = PanelBox::new(cell.x0, cell.x1, reserved.y0, reserved.y1);
        if body.width() < minimum.0 || body.height() < minimum.1 {
            return Ok(FitOutcome::Miss(
                scope.child(AddressRole::RootBody, 0)?.failure(
                    FitKind::TrackOrResidual,
                    "Canvas Plot local title block leaves insufficient body space",
                ),
            ));
        }
        Ok(FitOutcome::Measured(Self { text, body }))
    }
}
