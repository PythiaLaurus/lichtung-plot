//! Supplemental layers bind their own data and never inherit parent fields.
use crate::error::{length, BindingResult};
use crate::frontend::{context, Context};
use crate::{
    AdapterError, ArraySource, BarPointLayer, CategoryInput, CategoryValues, Color, NumericInput,
    NumericValues, PointLayer, Text, TextLayout,
};

/// An independent native point layer with its own optional array label context.
#[derive(Clone)]
pub struct PointLayerAdapter<'source> {
    layer: PointLayer,
    source: Context<'source>,
    len: usize,
}
impl<'source> PointLayerAdapter<'source> {
    /// Snapshot coordinates independently of the parent line's record count.
    pub fn new(
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(None, x, y)
    }
    /// Bind coordinates and retain only this layer's source for later labels.
    pub fn from_data(
        source: impl ArraySource + 'source,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(context(source)?, x, y)
    }
    fn bind(
        source: Context<'source>,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> BindingResult<Self> {
        let x = x.bind_numeric(source.as_deref(), "point layer x")?;
        let y = y.bind_numeric(source.as_deref(), "point layer y")?;
        let len = x.len();
        length("point layer y", y.name(), len, y.len())?;
        Ok(Self {
            layer: PointLayer::new(x, y),
            source,
            len,
        })
    }
    /// Bind labels against the layer's own coordinates, not the parent source.
    pub fn labels(mut self, labels: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
        let labels = labels.bind_category(self.source.as_deref(), "point layer labels")?;
        length("point layer labels", labels.name(), self.len, labels.len())?;
        self.layer = self.layer.labels(labels);
        Ok(self)
    }
    /// Replace the native exact-paint override.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.layer = self.layer.color(color);
        self
    }
    /// Replace the radius, retaining the native result/error type.
    pub fn radius(mut self, radius: u32) -> crate::Result<Self> {
        self.layer = self.layer.radius(radius)?;
        Ok(self)
    }
    /// Override point/label alpha with an exact byte.
    #[must_use]
    pub fn alpha(mut self, alpha: u8) -> Self {
        self.layer = self.layer.alpha(alpha);
        self
    }
    /// Supply a complete-layer legend identity, independently of point labels.
    #[must_use]
    pub fn legend_label(mut self, label: impl Into<Text>) -> Self {
        self.layer = self.layer.legend_label(label);
        self
    }
    /// Replace native label layout.
    #[must_use]
    pub fn label_text_layout(mut self, layout: TextLayout) -> Self {
        self.layer = self.layer.label_text_layout(layout);
        self
    }
    /// Inspect the native paint override.
    pub fn get_color(&self) -> Option<Color> {
        self.layer.get_color()
    }
    /// Inspect the native radius override.
    pub fn get_radius(&self) -> Option<u32> {
        self.layer.get_radius()
    }
    /// Inspect the native alpha override.
    pub fn get_alpha(&self) -> Option<u8> {
        self.layer.get_alpha()
    }
}
impl From<PointLayerAdapter<'_>> for PointLayer {
    fn from(adapter: PointLayerAdapter<'_>) -> Self {
        adapter.layer
    }
}

/// Independent observations positioned at prepared native ordinary-bar centers.
#[derive(Clone)]
pub struct BarPointLayerAdapter<'source> {
    layer: BarPointLayer,
    source: Context<'source>,
}
impl<'source> BarPointLayerAdapter<'source> {
    /// Bind direct categories/values and native exact paint/radius.
    pub fn new(
        category: impl CategoryInput,
        value: impl NumericInput,
        color: Color,
        radius: u32,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(None, category, value, color, radius)
    }
    /// Bind this layer's own source, independently of parent bar observations.
    pub fn from_data(
        source: impl ArraySource + 'source,
        category: impl CategoryInput,
        value: impl NumericInput,
        color: Color,
        radius: u32,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(context(source)?, category, value, color, radius)
    }
    fn bind(
        source: Context<'source>,
        category: impl CategoryInput,
        value: impl NumericInput,
        color: Color,
        radius: u32,
    ) -> BindingResult<Self> {
        let category = category.bind_category(source.as_deref(), "bar point layer category")?;
        let value = value.bind_numeric(source.as_deref(), "bar point layer value")?;
        length(
            "bar point layer value",
            value.name(),
            category.len(),
            value.len(),
        )?;
        Ok(Self {
            layer: BarPointLayer::new(category, None, value, color, radius)?,
            source,
        })
    }
    /// Bind positioned series in this layer's source and reconstruct natively.
    pub fn series(mut self, series: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
        let series = series.bind_category(self.source.as_deref(), "bar point layer series")?;
        length(
            "bar point layer series",
            series.name(),
            self.layer.values().len(),
            series.len(),
        )?;
        self.layer = BarPointLayer::new(
            self.layer.category_values().clone(),
            Some(series),
            self.layer.values().clone(),
            self.layer.color(),
            self.layer.radius(),
        )?;
        Ok(self)
    }
    /// Inspect native categories.
    pub fn category_values(&self) -> &CategoryValues {
        self.layer.category_values()
    }
    /// Inspect optional native positioned-series values.
    pub fn series_values(&self) -> Option<&CategoryValues> {
        self.layer.series_values()
    }
    /// Inspect owned numeric values.
    pub fn values(&self) -> &NumericValues {
        self.layer.values()
    }
    /// Inspect native complete paint.
    pub fn color(&self) -> Color {
        self.layer.color()
    }
    /// Inspect the native radius.
    pub fn radius(&self) -> u32 {
        self.layer.radius()
    }
}
impl From<BarPointLayerAdapter<'_>> for BarPointLayer {
    fn from(adapter: BarPointLayerAdapter<'_>) -> Self {
        adapter.layer
    }
}
