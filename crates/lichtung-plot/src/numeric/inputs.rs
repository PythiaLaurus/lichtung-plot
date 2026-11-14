//! Complete normalized policies retained by upstream child and height owners.
//! Prepared scientific sets are immutable within each root-local source owner;
//! domain-dependent selections additionally use that owner's range/feedback key.

use std::sync::Arc;

use super::format::NumericFormat;
use crate::builder::PlotDescription;
use crate::orchestrator::registration::PlotRegistration;
use crate::{HeatmapAnnotations, Result};

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

fn cells(source: &PlotDescription) -> Option<&NumericFormat> {
    match source.heatmap_annotations_spec() {
        HeatmapAnnotations::Hidden => None,
        HeatmapAnnotations::Values(format) => Some(format),
    }
}

impl NumericInputs {
    fn owner(
        registered: &PlotRegistration<'_>,
        canvas: Option<&crate::Canvas>,
    ) -> Option<[usize; 6]> {
        canvas?.owner_topology_at(registered.scope.numeric_owner.canvas?)
    }

    pub(crate) fn new(
        registered: &PlotRegistration<'_>,
        canvas: Option<&crate::Canvas>,
    ) -> Result<Self> {
        let source = registered.source;
        Ok(Self {
            scale: source.axis_scale_spec(),
            axes: [
                source.x_axis_spec().format.clone(),
                source.y_axis_spec().format.clone(),
            ],
            color: source.color_label_format_spec().cloned(),
            cells: cells(source).map(NumericFormat::try_copy).transpose()?,
            estimates: source
                .estimate_labels_spec()
                .map(NumericFormat::try_copy)
                .transpose()?,
            totals: source.total_format_spec().cloned(),
            owner: Self::owner(registered, canvas),
        })
    }

    pub(crate) fn matches(
        &self,
        registered: &PlotRegistration<'_>,
        canvas: Option<&crate::Canvas>,
    ) -> bool {
        let source = registered.source;
        self.scale == source.axis_scale_spec()
            && self.owner == Self::owner(registered, canvas)
            && self.axes[0] == source.x_axis_spec().format
            && self.axes[1] == source.y_axis_spec().format
            && self.color.as_ref() == source.color_label_format_spec()
            && self.cells.as_ref() == cells(source)
            && self.estimates.as_ref() == source.estimate_labels_spec()
            && self.totals.as_ref() == source.total_format_spec()
    }
}
