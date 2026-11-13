//! Owned native inputs and authored configuration, separate from an operation.

use super::{
    Arc, AxisScale, AxisSpec, BarPointLayer, BarSpacing, BooleanValues, Bounds, CategoryOrders,
    CategoryValues, FacetSpec, GuideSpec, HeatmapAnnotations, HeatmapLayout,
    HeatmapNullRowAnnotation, HistogramBins, HistogramComposite, HistogramStat, Labels,
    LegendPolicy, LinePathStyle, NumericFormat, PanelTextAnnotation, PlotInput, PlotKind,
    PointLayer, QqLine, RawOverlay, RegressionBand, SourceRows, Theme, WaterfallLabels,
};

#[derive(Clone)]
pub(crate) struct PlotDescription {
    pub(super) kind: PlotKind,
    pub(super) root_size: crate::layout::RootRequest,
    pub(super) title: crate::Text,
    pub(super) subtitle: crate::Text,
    pub(super) significance_mark_style: Option<crate::SignificanceMarkStyle>,
    pub(super) legend_composition: crate::LegendComposition,
    pub(super) raw_overlay: RawOverlay,
    pub(super) regression_band: RegressionBand,
    pub(super) line_path_style: LinePathStyle,
    pub(super) interaction_connectors: Option<bool>,
    pub(super) payload: Arc<SharedPayload>,
}

#[derive(Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent authored chart switches"
)]
pub(super) struct SharedPayload {
    pub(super) input: Arc<PlotInput>,
    pub(super) alpha: crate::alpha::AlphaInputs,
    pub(super) source_rows: Option<SourceRows>,
    pub(super) series: Option<CategoryValues>,
    pub(super) hue: Option<CategoryValues>,
    pub(super) group: Option<CategoryValues>,
    pub(super) caption: crate::Text,
    pub(super) x_label: crate::Text,
    pub(super) y_label: crate::Text,
    pub(super) labels: Labels,
    pub(super) x_axis_spec: AxisSpec,
    pub(super) y_axis_spec: AxisSpec,
    pub(super) text_layouts: crate::text::roles::TextRoles,
    pub(super) theme: Option<Arc<Theme>>,
    pub(super) significance: Vec<crate::significance::SignificanceAnnotation>,
    pub(super) bar_spacing: BarSpacing,
    pub(super) histogram_bins: HistogramBins,
    pub(super) histogram_stat: HistogramStat,
    pub(super) histogram_composite: Arc<HistogramComposite>,
    pub(super) qq_line: QqLine,
    pub(super) point_outline_visible: bool,
    pub(super) legend_policy: LegendPolicy,
    pub(super) color_label: crate::Text,
    pub(super) color_label_format: Option<Arc<NumericFormat>>,
    pub(super) continuous_color_scale: Option<crate::theme::ContinuousColorScale>,
    pub(super) heatmap_layout: HeatmapLayout,
    pub(super) heatmap_annotations: HeatmapAnnotations,
    pub(super) heatmap_null_row_annotations: Vec<HeatmapNullRowAnnotation>,
    pub(super) heatmap_highlight: Option<BooleanValues>,
    pub(super) estimate_labels: Option<NumericFormat>,
    pub(super) category_orders: CategoryOrders,
    pub(super) color_by_sign: bool,
    pub(super) zero_line: bool,
    pub(super) line_band: Option<Bounds>,
    pub(super) estimate_status: Option<Arc<[crate::EstimateStatus]>>,
    pub(super) markers_visible: bool,
    pub(super) coefficient_reference_line: Option<f64>,
    pub(super) axis_scale: AxisScale,
    pub(super) facet: Option<FacetSpec>,
    pub(super) interaction_styles: Option<Vec<crate::InteractionStyle>>,
    pub(super) categorical_color_scale: Option<crate::theme::CategoricalColorScale>,
    pub(super) waterfall_labels: Option<WaterfallLabels>,
    pub(super) total_format: Option<Arc<NumericFormat>>,
    pub(super) guides: Vec<GuideSpec>,
    pub(super) point_layers: Vec<PointLayer>,
    pub(super) bar_point_layers: Vec<BarPointLayer>,
    pub(super) panel_text_annotations: Vec<PanelTextAnnotation>,
}

impl PlotDescription {
    pub(super) fn new(input: PlotInput, kind: PlotKind) -> Self {
        Self {
            kind,
            root_size: crate::layout::RootRequest::Natural,
            title: crate::Text::UNSET,
            subtitle: crate::Text::UNSET,
            significance_mark_style: None,
            legend_composition: crate::LegendComposition::Combined,
            raw_overlay: RawOverlay::default(),
            regression_band: RegressionBand::default(),
            line_path_style: LinePathStyle::default(),
            interaction_connectors: None,
            payload: Arc::new(SharedPayload {
                input: Arc::new(input),
                alpha: crate::alpha::AlphaInputs::default(),
                source_rows: None,
                series: None,
                hue: None,
                group: None,
                caption: crate::Text::UNSET,
                x_label: crate::Text::UNSET,
                y_label: crate::Text::UNSET,
                labels: Labels::default(),
                x_axis_spec: AxisSpec::default(),
                y_axis_spec: AxisSpec::default(),
                text_layouts: crate::text::roles::TextRoles::default(),
                theme: None,
                significance: Vec::new(),
                bar_spacing: BarSpacing::default(),
                histogram_bins: HistogramBins::default(),
                histogram_stat: HistogramStat::default(),
                histogram_composite: Arc::new(HistogramComposite::default()),
                qq_line: QqLine::default(),
                point_outline_visible: true,
                legend_policy: LegendPolicy::Automatic,
                color_label: crate::Text::UNSET,
                color_label_format: None,
                continuous_color_scale: None,
                heatmap_layout: HeatmapLayout::default(),
                heatmap_annotations: HeatmapAnnotations::default(),
                heatmap_null_row_annotations: Vec::new(),
                heatmap_highlight: None,
                estimate_labels: None,
                category_orders: CategoryOrders::default(),
                color_by_sign: false,
                zero_line: false,
                line_band: None,
                estimate_status: None,
                markers_visible: true,
                coefficient_reference_line: None,
                axis_scale: AxisScale::Linear,
                facet: None,
                interaction_styles: None,
                categorical_color_scale: None,
                waterfall_labels: None,
                total_format: None,
                guides: Vec::new(),
                point_layers: Vec::new(),
                bar_point_layers: Vec::new(),
                panel_text_annotations: Vec::new(),
            }),
        }
    }

    pub(super) fn payload_mut(&mut self) -> &mut SharedPayload {
        Arc::make_mut(&mut self.payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NumericValues, Plot};

    #[test]
    fn authored_clones_share_one_payload_and_edits_keep_sibling_intent() {
        let original = Plot::scatter(
            NumericValues::new("x", [0.0, 1.0]),
            NumericValues::new("y", [1.0, 0.0]),
        )
        .title("Title")
        .x_label("Horizontal");
        let sibling = original.clone();
        assert!(Arc::ptr_eq(
            &original.description.payload,
            &sibling.description.payload
        ));
        let inline = sibling
            .width(640)
            .title_alignment(crate::TextAlignment::End);
        assert!(Arc::ptr_eq(
            &original.description.payload,
            &inline.description.payload
        ));
        assert_eq!(
            original.sizing_request(),
            crate::layout::RootRequest::Natural
        );
        assert_eq!(original.prose_alignments()[0], None);
        assert_eq!(
            inline.prose_alignments()[0],
            Some(crate::TextAlignment::End)
        );
        let edited = inline
            .x_limits(-1.0, 2.0)
            .clear_x_label_layout()
            .x_label("Changed");
        assert!(!Arc::ptr_eq(
            &original.description.payload,
            &edited.description.payload
        ));
        assert_eq!(original.x_axis_spec().limits, None);
        assert_eq!(edited.x_axis_spec().limits, Some((-1.0, 2.0)));
        assert_eq!(
            original.copy_axis_titles().unwrap()[0].as_str(),
            "Horizontal"
        );
        assert_eq!(edited.copy_axis_titles().unwrap()[0].as_str(), "Changed");
        let erased = edited.erase();
        assert_eq!(erased.kind(), PlotKind::Scatter);
        assert_eq!(
            erased.sizing_request(),
            crate::layout::RootRequest::ExactWidth(640)
        );
    }
}
