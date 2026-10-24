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
    cell: Option<Arc<(usize, usize)>>,
}

fn cells<K: crate::kind::Marker>(plot: &Plot<K>) -> Option<&NumericFormat> {
    match plot.heatmap_annotations_spec() {
        HeatmapAnnotations::Hidden => None,
        HeatmapAnnotations::Values(format) => Some(format),
    }
}

impl NumericInputs {
    pub(crate) fn new<K: crate::kind::Marker>(plot: &Plot<K>) -> Result<Self> {
        Ok(Self {
            scale: plot.axis_scale_spec(),
            axes: [
                plot.x_axis_spec().format.clone(),
                plot.y_axis_spec().format.clone(),
            ],
            color: plot.color_label_format_spec().cloned(),
            cells: cells(plot).map(NumericFormat::try_copy).transpose()?,
            estimates: plot
                .estimate_labels_spec()
                .map(NumericFormat::try_copy)
                .transpose()?,
            totals: plot.total_format_spec().cloned(),
            cell: plot.numeric_cell.clone(),
        })
    }

    pub(crate) fn matches<K: crate::kind::Marker>(&self, plot: &Plot<K>) -> bool {
        self.scale == plot.axis_scale_spec()
            && self.cell == plot.numeric_cell
            && self.axes[0] == plot.x_axis_spec().format
            && self.axes[1] == plot.y_axis_spec().format
            && self.color.as_ref() == plot.color_label_format_spec()
            && self.cells.as_ref() == cells(plot)
            && self.estimates.as_ref() == plot.estimate_labels_spec()
            && self.totals.as_ref() == plot.total_format_spec()
    }
}
