//! Supplemental layers own their lookup contexts independently of parent plots.

use polars_core::prelude::DataFrame;

use crate::error::{length, BindingResult};
use crate::{
    AdapterError, BarPointLayer, CategoryInput, CategoryValues, Color, NumericInput, NumericValues,
    PointLayer, Text, TextLayout,
};

/// An independent native point layer with optional borrowed Polars label lookup.
#[derive(Clone)]
pub struct PointLayerAdapter<'source> {
    layer: PointLayer,
    source: Option<&'source DataFrame>,
    len: usize,
}

impl<'source> PointLayerAdapter<'source> {
    /// Snapshot direct coordinates; they need not have the parent line's length.
    pub fn new(
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(None, x, y)
    }

    /// Snapshot coordinates and retain this layer's own lookup source for labels.
    pub fn from_data(
        source: &'source DataFrame,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(Some(source), x, y)
    }

    fn bind(
        source: Option<&'source DataFrame>,
        x: impl NumericInput,
        y: impl NumericInput,
    ) -> BindingResult<Self> {
        let x = x.bind_numeric(source, "point layer x")?;
        let y = y.bind_numeric(source, "point layer y")?;
        let len = x.len();
        length("point layer y", y.name(), len, y.len())?;
        Ok(Self {
            layer: PointLayer::new(x, y),
            source,
            len,
        })
    }

    /// Bind one label per layer point, never against the parent plot's source.
    pub fn labels(mut self, labels: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
        let labels = labels.bind_category(self.source, "point layer labels")?;
        length("point layer labels", labels.name(), self.len, labels.len())?;
        self.layer = self.layer.labels(labels);
        Ok(self)
    }

    /// Replace complete exact paint with the native precedence rules.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.layer = self.layer.color(color);
        self
    }
    /// Replace the native point radius, preserving its original error type.
    pub fn radius(mut self, radius: u32) -> crate::Result<Self> {
        self.layer = self.layer.radius(radius)?;
        Ok(self)
    }
    /// Override native point/label alpha with an exact byte.
    #[must_use]
    pub fn alpha(mut self, alpha: u8) -> Self {
        self.layer = self.layer.alpha(alpha);
        self
    }
    /// Supply one native legend identity for the complete independent layer.
    #[must_use]
    pub fn legend_label(mut self, label: impl Into<Text>) -> Self {
        self.layer = self.layer.legend_label(label);
        self
    }
    /// Replace the layer's column-backed label layout.
    #[must_use]
    pub fn label_text_layout(mut self, layout: TextLayout) -> Self {
        self.layer = self.layer.label_text_layout(layout);
        self
    }
    /// Inspect the native exact-paint override.
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
    source: Option<&'source DataFrame>,
}

impl<'source> BarPointLayerAdapter<'source> {
    /// Bind direct category/value variables and an exact native paint/radius.
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
        source: &'source DataFrame,
        category: impl CategoryInput,
        value: impl NumericInput,
        color: Color,
        radius: u32,
    ) -> std::result::Result<Self, AdapterError> {
        Self::bind(Some(source), category, value, color, radius)
    }
    fn bind(
        source: Option<&'source DataFrame>,
        category: impl CategoryInput,
        value: impl NumericInput,
        color: Color,
        radius: u32,
    ) -> BindingResult<Self> {
        let category = category.bind_category(source, "bar point layer category")?;
        let value = value.bind_numeric(source, "bar point layer value")?;
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
    /// Bind positioned-series identity from this layer's source and rebuild natively.
    pub fn series(mut self, series: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
        let series = series.bind_category(self.source, "bar point layer series")?;
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
    /// Inspect categories used to resolve prepared native bar cells.
    pub fn category_values(&self) -> &CategoryValues {
        self.layer.category_values()
    }
    /// Inspect optional native positioned-series values.
    pub fn series_values(&self) -> Option<&CategoryValues> {
        self.layer.series_values()
    }
    /// Inspect owned native numeric values.
    pub fn values(&self) -> &NumericValues {
        self.layer.values()
    }
    /// Inspect complete exact native paint.
    pub fn color(&self) -> Color {
        self.layer.color()
    }
    /// Inspect the fixed native radius.
    pub fn radius(&self) -> u32 {
        self.layer.radius()
    }
}

impl From<BarPointLayerAdapter<'_>> for BarPointLayer {
    fn from(adapter: BarPointLayerAdapter<'_>) -> Self {
        adapter.layer
    }
}
