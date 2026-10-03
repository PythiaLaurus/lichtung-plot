//! One complete embedded Plot title/subtitle/caption block in its Canvas cell.

use crate::builder::Plot;
use crate::decorations::title_block::plan::{
    RootAlignments, RootTitleBlockIntrinsics, RootTitleBlockPlan,
};
use crate::layout::fit::{self, FitOutcome, FitScope};
use crate::layout::kernel::FitKind;
use crate::layout::{AddressRole, PanelBox};
use crate::theme::{ResolvedTheme, TitleAlignment};
use crate::Error;

pub(super) struct PlotLocalTextPlan {
    pub(super) text: RootTitleBlockPlan,
    pub(super) body: PanelBox,
}

impl PlotLocalTextPlan {
    pub(super) fn measure(
        plot: &Plot,
        cell: PanelBox,
        theme: &ResolvedTheme,
        panel_alignments: [Option<TitleAlignment>; 2],
        intrinsics: RootTitleBlockIntrinsics,
        scope: &FitScope,
    ) -> Result<FitOutcome<Self>, Error> {
        let sources = [plot.title_text(), plot.subtitle_text(), plot.caption_text()];
        let present = sources
            .iter()
            .any(|source| source.is_some_and(|text| !text.is_empty()));
        let span = if present {
            PanelBox::new(
                cell.x0 + 16.0,
                cell.x1 - 16.0,
                cell.y0 + 8.0,
                cell.y1 - 12.0,
            )
        } else {
            cell
        };
        if span.width() <= 0.0 || span.height() <= 0.0 {
            return Ok(FitOutcome::Miss(
                scope
                    .child(
                        AddressRole::RootBody,
                        0,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        FitKind::TrackOrResidual,
                        "Canvas Plot local text leaves no content span",
                    ),
            ));
        }
        if let FitOutcome::Miss(reason) = fit::outcome(intrinsics.check_width(
            span.width(),
            scope,
            &theme.prepared_font_environment()?.generation_resources(),
        ))? {
            return Ok(FitOutcome::Miss(reason));
        }
        let minimum = plot.intrinsic_body_minimum(theme)?;
        let alignments = RootAlignments::resolve(
            theme,
            [plot.title_alignment, plot.subtitle_alignment],
            panel_alignments,
        );
        let text = match fit::outcome(RootTitleBlockPlan::measure(
            theme, sources, span, alignments, minimum.1, scope,
        ))? {
            FitOutcome::Measured(text) => text,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let body = PanelBox::new(
            cell.x0,
            cell.x1,
            span.y0 + text.bottom_reserve,
            span.y1 - text.top_reserve,
        );
        if body.width() < minimum.0 || body.height() < minimum.1 {
            return Ok(FitOutcome::Miss(
                scope
                    .child(
                        AddressRole::RootBody,
                        0,
                        &theme.prepared_font_environment()?.generation_resources(),
                    )?
                    .failure(
                        FitKind::TrackOrResidual,
                        "Canvas Plot local title block leaves insufficient body space",
                    ),
            ));
        }
        Ok(FitOutcome::Measured(Self { text, body }))
    }
}
