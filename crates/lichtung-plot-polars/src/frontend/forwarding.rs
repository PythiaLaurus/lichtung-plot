//! One local inventory of native configuration forwarding, with original bounds.

use std::path::Path;

use super::PlotAdapter;
use crate::{
    kind, AxisScale, AxisTick, BarPointLayer, BarSpacing, CategoricalColorScale,
    CategoryLabelSelection, CategoryOrder, ContinuousColorScale, EstimateStatus, GuideSpec,
    GuideTextRole, HeatmapAnnotations, HeatmapLayout, HeatmapNullRowAnnotation, HistogramBins,
    HistogramComposite, HistogramStat, InteractionStyle, LegendComposition, LegendPolicy,
    LinePathStyle, NumericFormat, Output, PanelTextAnnotation, PngOptions, PointLayer, QqLine,
    RawOverlay, RegressionBand, Significance, SignificanceMarkStyle, Text, TextAlignment,
    TextLayout, Theme, TickGeneration, WaterfallLabels,
};

macro_rules! forward {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) $(=> $bound:ident)?) => {
        impl<K: kind::Marker> PlotAdapter<'_, K> {
            #[doc = concat!("Forward native [`crate::Plot::", stringify!($name), "`] configuration.")]
            #[must_use]
            pub fn $name(mut self, $($arg: $ty),*) -> Self $(where K: kind::$bound)? {
                self.plot = self.plot.$name($($arg),*);
                self
            }
        }
    };
}
forward!(title(text: impl Into<Text>));
forward!(subtitle(text: impl Into<Text>));
forward!(caption(text: impl Into<Text>));
forward!(x_label(text: impl Into<Text>));
forward!(y_label(text: impl Into<Text>));
forward!(title_alignment(alignment: TextAlignment));
forward!(subtitle_alignment(alignment: TextAlignment));
forward!(theme(theme: Theme));
forward!(width(width: u32));
forward!(max_width(maximum: u32));
forward!(size(width: u32, height: u32));
forward!(color_label(label: impl Into<Text>) => ContinuousColor);
forward!(color_label_format(format: NumericFormat) => ContinuousColor);
forward!(heatmap_null_row_annotation(annotation: HeatmapNullRowAnnotation) => HeatmapOptions);
forward!(estimate_labels(format: NumericFormat) => CoefficientOptions);
forward!(significance_mark_style(style: SignificanceMarkStyle) => SignificanceStyle);
forward!(panel_text_annotation(annotation: PanelTextAnnotation) => PanelText);
forward!(x_category_labels(selection: CategoryLabelSelection) => CategoricalX);
forward!(y_category_labels(selection: CategoryLabelSelection) => CategoricalY);
forward!(category_order(order: impl Into<CategoryOrder>) => CategoryOrder);
forward!(x_category_order(order: impl Into<CategoryOrder>) => HeatmapOptions);
forward!(y_category_order(order: impl Into<CategoryOrder>) => HeatmapOptions);
forward!(x_limits(min: f64, max: f64) => NumericX);
forward!(y_limits(min: f64, max: f64) => NumericY);
forward!(x_ticks(ticks: Vec<AxisTick>) => NumericX);
forward!(y_ticks(ticks: Vec<AxisTick>) => NumericY);
forward!(x_tick_generation(generation: TickGeneration) => NumericX);
forward!(y_tick_generation(generation: TickGeneration) => NumericY);
forward!(x_tick_format(format: NumericFormat) => NumericX);
forward!(y_tick_format(format: NumericFormat) => NumericY);
forward!(guide(guide: GuideSpec) => Guides);
forward!(categorical_color_scale(scale: CategoricalColorScale) => CategoricalScale);
forward!(continuous_color_scale(scale: ContinuousColorScale) => ContinuousColor);
forward!(color_by_sign() => SignedBars);
forward!(zero_line() => ZeroLine);
forward!(hide_point_outline() => PointOutline);
forward!(markers(visible: bool) => Markers);
forward!(legend(policy: LegendPolicy) => Legend);
forward!(legend_composition(composition: LegendComposition) => Legend);
forward!(raw_overlay(overlay: RawOverlay) => RawOverlay);
forward!(estimate_status(statuses: impl IntoIterator<Item = EstimateStatus>) => EstimateStatus);
forward!(x_tick_text_layout(layout: TextLayout) => XAxisText);
forward!(y_tick_text_layout(layout: TextLayout) => YAxisText);
forward!(facet_row_text_layout(layout: TextLayout) => Facet);
forward!(facet_column_text_layout(layout: TextLayout) => Facet);
forward!(estimate_detail_text_layout(layout: TextLayout) => EstimateDetailText);
forward!(significance_text_layout(layout: TextLayout) => SignificanceStyle);
forward!(guide_text_layout(role: GuideTextRole, layout: TextLayout) => Legend);

macro_rules! family_forward {
    ($kind:ident, $name:ident($($arg:ident: $ty:ty),* $(,)?)) => {
        impl PlotAdapter<'_, kind::$kind> {
            #[doc = concat!("Forward the native ", stringify!($kind), " `", stringify!($name), "` instruction.")]
            #[must_use]
            pub fn $name(mut self, $($arg: $ty),*) -> Self {
                self.plot = self.plot.$name($($arg),*);
                self
            }
        }
    };
}
family_forward!(Heatmap, layout(layout: HeatmapLayout));
family_forward!(Heatmap, annotations(annotations: HeatmapAnnotations));
family_forward!(Heatmap, value_text_layout(layout: TextLayout));
family_forward!(Heatmap, null_row_text_layout(layout: TextLayout));
family_forward!(Forest, scale(scale: AxisScale));
family_forward!(Bar, spacing(spacing: BarSpacing));
family_forward!(HorizontalBar, spacing(spacing: BarSpacing));
family_forward!(Histogram, bins(bins: HistogramBins));
family_forward!(Histogram, stat(stat: HistogramStat));
family_forward!(Histogram, composite(composite: HistogramComposite));
family_forward!(Qq, reference_line(line: QqLine));
family_forward!(Regression, band(band: RegressionBand));
family_forward!(LinePath, style(style: LinePathStyle));
family_forward!(Coefficient, reference_line(value: f64));
family_forward!(Interaction, connectors(visible: bool));
family_forward!(Interaction, styles(styles: impl IntoIterator<Item = InteractionStyle>));
family_forward!(Waterfall, total_format(format: NumericFormat));
family_forward!(Waterfall, total_labels(labels: WaterfallLabels));
family_forward!(Waterfall, total_text_layout(layout: TextLayout));
family_forward!(Matrix, matrix_row_text_layout(layout: TextLayout));
family_forward!(Matrix, matrix_column_text_layout(layout: TextLayout));

macro_rules! layer_forward {
    ($family:ident, $layer:ty) => {
        impl PlotAdapter<'_, kind::$family> {
            /// Append a fully bound native or adapter layer without inheriting mappings.
            #[must_use]
            pub fn point_layer(mut self, layer: impl Into<$layer>) -> Self {
                self.plot = self.plot.point_layer(layer.into());
                self
            }
        }
    };
}
layer_forward!(Line, PointLayer);
layer_forward!(Bar, BarPointLayer);
layer_forward!(HorizontalBar, BarPointLayer);

impl<K: kind::Marker> PlotAdapter<'_, K> {
    /// Append caller-owned native significance annotations of an admitted placement.
    #[must_use]
    pub fn significance<P>(mut self, annotations: impl IntoIterator<Item = Significance<P>>) -> Self
    where
        K: kind::Significance<P>,
    {
        self.plot = self.plot.significance(annotations);
        self
    }

    /// Render and publish SVG; return the original native result and reusable Output.
    pub fn svg(self, path: impl AsRef<Path>) -> crate::Result<Output> {
        self.plot.svg(path)
    }
    /// Render and publish SVG retaining selectable text.
    pub fn svg_preserve_text(self, path: impl AsRef<Path>) -> crate::Result<Output> {
        self.plot.svg_preserve_text(path)
    }
    /// Render and publish a 300-DPI PNG.
    pub fn png(self, path: impl AsRef<Path>) -> crate::Result<Output> {
        self.plot.png(path)
    }
    /// Render and publish a source-sized PNG.
    pub fn png_preview(self, path: impl AsRef<Path>) -> crate::Result<Output> {
        self.plot.png_preview(path)
    }
    /// Render and publish a PNG using caller-owned native sizing/encoding options.
    pub fn png_with_options(
        self,
        path: impl AsRef<Path>,
        options: PngOptions,
    ) -> crate::Result<Output> {
        self.plot.png_with_options(path, options)
    }
}
