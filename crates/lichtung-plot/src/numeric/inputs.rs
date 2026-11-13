//! Complete normalized policies retained by upstream child and height owners.
//! Prepared scientific sets are immutable within each root-local source owner;
//! domain-dependent selections additionally use that owner's range/feedback key.

use std::sync::Arc;

use super::format::NumericFormat;
use crate::{HeatmapAnnotations, Plot, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NumericInputs {
    scale: crate::AxisScale,
    axes: [Option<Arc<NumericFormat>>; 2],
    color: Option<Arc<NumericFormat>>,
    cells: Option<NumericFormat>,
    estimates: Option<NumericFormat>,
    totals: Option<Arc<NumericFormat>>,
    owner: Option<[usize; 6]>,
}

fn cells<K: crate::kind::Marker>(plot: &Plot<K>) -> Option<&NumericFormat> {
    match plot.description().heatmap_annotations_spec() {
        HeatmapAnnotations::Hidden => None,
        HeatmapAnnotations::Values(format) => Some(format),
    }
}

impl NumericInputs {
    fn owner(
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        canvas: Option<&crate::Canvas>,
    ) -> Option<[usize; 6]> {
        canvas?.owner_topology_at(registered?.scope.numeric_owner.canvas?)
    }

    pub(crate) fn new<K: crate::kind::Marker>(
        plot: &Plot<K>,
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        canvas: Option<&crate::Canvas>,
    ) -> Result<Self> {
        Ok(Self {
            scale: plot.description().axis_scale_spec(),
            axes: [
                plot.description().x_axis_spec().format.clone(),
                plot.description().y_axis_spec().format.clone(),
            ],
            color: plot.description().color_label_format_spec().cloned(),
            cells: cells(plot).map(NumericFormat::try_copy).transpose()?,
            estimates: plot
                .description()
                .estimate_labels_spec()
                .map(NumericFormat::try_copy)
                .transpose()?,
            totals: plot.description().total_format_spec().cloned(),
            owner: Self::owner(registered, canvas),
        })
    }

    pub(crate) fn matches<K: crate::kind::Marker>(
        &self,
        plot: &Plot<K>,
        registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
        canvas: Option<&crate::Canvas>,
    ) -> bool {
        self.scale == plot.description().axis_scale_spec()
            && self.owner == Self::owner(registered, canvas)
            && self.axes[0] == plot.description().x_axis_spec().format
            && self.axes[1] == plot.description().y_axis_spec().format
            && self.color.as_ref() == plot.description().color_label_format_spec()
            && self.cells.as_ref() == cells(plot)
            && self.estimates.as_ref() == plot.description().estimate_labels_spec()
            && self.totals.as_ref() == plot.description().total_format_spec()
    }
}
