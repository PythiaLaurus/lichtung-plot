//! Exact-family adapter ownership and data-bearing operations.

use polars_core::prelude::DataFrame;

use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, CategoryInput, FacetScales, FacetSpec, Labels, Plot, PlotKind,
    SourceRowsInput,
};

mod constructors;
mod forwarding;
mod grid;

/// A configured native plot plus an optional borrowed Polars lookup context.
///
/// The family aliases provide `new` and `from_data` on this actual type.
/// Direct variables are snapshotted immediately; only `from_data` retains a
/// DataFrame borrow. Conversion to native [`Plot`] releases that lookup context.
#[derive(Clone)]
pub struct PlotAdapter<'source, K: kind::Marker> {
    plot: Plot<K>,
    source: Option<&'source DataFrame>,
    len: Option<usize>,
    labels: Labels,
    facet: Option<FacetSpec>,
    grid: Option<grid::Grid>,
}

macro_rules! families {
    ($($family:ident),+ $(,)?) => {$ (
        #[doc = concat!("Polars plotting frontend for the native ", stringify!($family), " family.")]
        pub type $family<'source> = PlotAdapter<'source, kind::$family>;
    )+};
}
families!(
    Scatter,
    Histogram,
    Density,
    Qq,
    PointInterval,
    Line,
    Trajectory,
    LinePath,
    Interaction,
    Coefficient,
    Forest,
    Dumbbell,
    Heatmap,
    Bar,
    HorizontalBar,
    StackedBar,
    StackedHorizontalBar,
    Boxplot,
    Violin,
    Raincloud,
    Regression,
    Marginal,
    Matrix,
    Beeswarm,
    Waterfall
);

impl<'source, K: kind::Marker> PlotAdapter<'source, K> {
    fn bound(plot: Plot<K>, source: Option<&'source DataFrame>, len: Option<usize>) -> Self {
        Self {
            plot,
            source,
            len,
            labels: Labels::default(),
            facet: None,
            grid: None,
        }
    }

    fn aligned(&self, role: &'static str, name: &str, actual: usize) -> BindingResult<()> {
        if let Some(expected) = self.len {
            length(role, name, expected, actual)?;
        }
        Ok(())
    }

    /// Inspect the native plot family without erasure.
    pub fn kind(&self) -> PlotKind {
        self.plot.kind()
    }

    /// Replace the complete native label policy, including earlier grid aliases.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels.clone();
        self.plot = self.plot.labels(labels);
        self
    }

    /// Bind exact unique IDs, aligned with the original plot slots.
    pub fn source_rows(
        mut self,
        values: impl SourceRowsInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::SourceRows,
    {
        let values = values.bind_source_rows(self.source, "source rows")?;
        self.aligned("source rows", "source rows", values.len())?;
        self.plot = self.plot.source_rows(values);
        Ok(self)
    }

    /// Apply one complete native facet specification without reinterpreting its slots.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: kind::Facet,
    {
        self.facet = Some(facet.clone());
        self.plot = self.plot.facet(facet);
        self
    }

    /// Split rows by an explicitly categorical field or variable.
    pub fn facet_row(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = values.bind_category(self.source, "facet row")?;
        self.aligned("facet row", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::row(values)))
    }

    /// Split columns by an explicitly categorical field or variable.
    pub fn facet_column(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = values.bind_category(self.source, "facet column")?;
        self.aligned("facet column", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::column(values)))
    }

    /// Cross two categorical dimensions; this replaces the previous complete facet.
    pub fn facet_grid(
        self,
        row: impl CategoryInput,
        column: impl CategoryInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let row = row.bind_category(self.source, "facet row")?;
        let column = column.bind_category(self.source, "facet column")?;
        self.aligned("facet row", row.name(), row.len())?;
        self.aligned("facet column", column.name(), column.len())?;
        length("facet grid", column.name(), row.len(), column.len())?;
        Ok(self.facet(FacetSpec::grid(row, column)?))
    }

    /// Wrap a categorical dimension into a positive number of columns.
    pub fn facet_wrap(
        self,
        values: impl CategoryInput,
        columns: usize,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = values.bind_category(self.source, "facet wrap")?;
        self.aligned("facet wrap", values.name(), values.len())?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }

    /// Replace scale sharing on an existing facet; fails if no facet was selected.
    pub fn facet_scales(self, scales: FacetScales) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let facet = self
            .facet
            .clone()
            .ok_or_else(|| binding("facet scales", "select a facet first"))?;
        Ok(self.facet(facet.scales(scales)))
    }
}

macro_rules! category_binding {
    ($name:ident, $capability:ident) => {
        impl<K: kind::$capability> PlotAdapter<'_, K> {
            #[doc = concat!("Bind the native ", stringify!($name), " role from a field or categorical variable.")]
            pub fn $name(mut self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
                let values = values.bind_category(self.source, stringify!($name))?;
                self.aligned(stringify!($name), values.name(), values.len())?;
                self.plot = self.plot.$name(values);
                Ok(self)
            }
        }
    };
}
category_binding!(hue, Hue);
category_binding!(group, Group);
category_binding!(series, Series);

impl<K: kind::ExactFamily> PlotAdapter<'_, K> {
    /// Consume the adapter and return the native erased plot, ending source lookup.
    pub fn erase(self) -> Plot<kind::Any> {
        self.plot.erase()
    }
}

impl<K: kind::Marker> From<PlotAdapter<'_, K>> for Plot<K> {
    fn from(adapter: PlotAdapter<'_, K>) -> Self {
        adapter.plot
    }
}
