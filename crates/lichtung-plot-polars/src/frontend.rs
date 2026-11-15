//! Exact-family adapter ownership and data-bearing operations.

use std::marker::PhantomData;

use polars_core::prelude::DataFrame;

use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, CategoryInput, FacetScales, FacetSpec, Labels, Plot, PlotKind,
    SourceRowsInput,
};

mod alpha;
mod constructors;
mod forwarding;
mod geometry;
mod grid;
mod status;
#[cfg(test)]
mod tests;
mod wide;

/// Source-free binding state paired with one exact-family native [`Plot`].
///
/// This opaque value retains the adapter's alignment, display and expansion
/// metadata, not a DataFrame, schema or lookup borrow. Keep it paired with the
/// native configuration returned by [`PlotAdapter::into_parts`]. The family
/// marker prevents cross-family pairing, but cannot detect mismatched versions
/// of the same family; callers must commit each Plot/metadata candidate together.
///
/// ```compile_fail
/// use lichtung_plot_polars::{kind, BindingMetadata, Line, Plot};
/// fn wrong_family(plot: Plot<kind::Line>, metadata: BindingMetadata<kind::Scatter>) {
///     let _ = Line::from_parts(plot, metadata);
/// }
/// ```
#[derive(Clone)]
pub struct BindingMetadata<K: kind::Marker> {
    len: Option<usize>,
    geometry_points: Option<usize>,
    labels: Labels,
    facet: Option<FacetSpec>,
    grid: Option<grid::Grid>,
    expansion: Option<wide::Expansion>,
    matrix_keys: Option<Vec<String>>,
    _kind: PhantomData<K>,
}

/// A configured native plot plus an optional borrowed Polars lookup context.
///
/// The family aliases provide `new` and `from_data` on this actual type.
/// Direct variables are snapshotted immediately; only `from_data` retains a
/// DataFrame borrow. Conversion to native [`Plot`] releases that lookup context.
/// Use [`Self::into_parts`] to retain source-free binding metadata as well.
#[derive(Clone)]
pub struct PlotAdapter<'source, K: kind::Marker> {
    plot: Plot<K>,
    pub(crate) source: Option<&'source DataFrame>,
    bindings: BindingMetadata<K>,
}

macro_rules! families {
    ($($family:ident),+ $(,)?) => {$ (
        #[doc = concat!("Polars plotting frontend for the native ", stringify!($family), " family.")]
        pub type $family<'source> = PlotAdapter<'source, kind::$family>;
    )+
        #[cfg(test)]
        const _: fn() = || {
            fn owned<T: Clone + Send + Sync + 'static>() {}
            $(owned::<BindingMetadata<kind::$family>>();)+
        };
    };
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
            bindings: BindingMetadata {
                len,
                geometry_points: None,
                labels: Labels::default(),
                facet: None,
                grid: None,
                expansion: None,
                matrix_keys: None,
                _kind: PhantomData,
            },
        }
    }

    fn aligned(&self, role: &'static str, name: &str, actual: usize) -> BindingResult<()> {
        if let Some(expected) = self.bindings.len {
            length(role, name, expected, actual)?;
        }
        Ok(())
    }

    fn row_copies(&self) -> usize {
        self.bindings.expansion.as_ref().map_or_else(
            || self.bindings.grid.as_ref().map_or(1, grid::Grid::columns),
            |wide| wide.keys.len(),
        )
    }
    fn alpha_len(&self) -> Option<usize> {
        self.bindings.geometry_points.or(self.bindings.len)
    }

    fn category_rows(
        &self,
        values: impl CategoryInput,
        role: &'static str,
    ) -> BindingResult<crate::CategoryValues> {
        let values = values.bind_category_repeated(self.source, role, self.row_copies())?;
        self.aligned(role, values.name(), values.len())?;
        Ok(values)
    }

    /// Inspect the native plot family without erasure.
    pub fn kind(&self) -> PlotKind {
        self.plot.kind()
    }

    /// Move the native Plot and its metadata out, releasing source lookup.
    ///
    /// Both returned values are source-free and must remain paired at the same
    /// configuration version. No binding or scientific validation is performed.
    #[must_use]
    pub fn into_parts(self) -> (Plot<K>, BindingMetadata<K>) {
        (self.plot, self.bindings)
    }

    /// Replace the complete native label policy, including earlier grid aliases.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.bindings.labels = labels.clone();
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
        self.cell_shape(&values, "source rows")?;
        let values =
            values.bind_source_rows_repeated(self.source, "source rows", self.row_copies())?;
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
        self.bindings.facet = Some(facet.clone());
        self.plot = self.plot.facet(facet);
        self
    }

    /// Split rows by an explicitly categorical field or variable.
    pub fn facet_row(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = self.category_rows(values, "facet row")?;
        Ok(self.facet(FacetSpec::row(values)))
    }

    /// Split columns by an explicitly categorical field or variable.
    pub fn facet_column(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = self.category_rows(values, "facet column")?;
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
        let row = self.category_rows(row, "facet row")?;
        let column = self.category_rows(column, "facet column")?;
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
        let values = self.category_rows(values, "facet wrap")?;
        Ok(self.facet(FacetSpec::wrap(values, columns)?))
    }

    /// Replace scale sharing on an existing facet; fails if no facet was selected.
    pub fn facet_scales(self, scales: FacetScales) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let facet = self
            .bindings
            .facet
            .clone()
            .ok_or_else(|| binding("facet scales", "select a facet first"))?;
        Ok(self.facet(facet.scales(scales)))
    }
}

impl<K: kind::Marker> PlotAdapter<'static, K> {
    /// Restore a paired native configuration and metadata without source lookup.
    ///
    /// The caller is responsible for pairing the same configuration version.
    /// Restoration performs no binding or scientific validation.
    #[must_use]
    pub fn from_parts(plot: Plot<K>, metadata: BindingMetadata<K>) -> Self {
        Self {
            plot,
            source: None,
            bindings: metadata,
        }
    }

    /// Restore a pair with a borrowed DataFrame for subsequent field lookups.
    ///
    /// This only installs the lookup context; it does not refresh bound values,
    /// validate the pair or compare source height with existing plot slots.
    /// Subsequent data-bearing setters retain their own alignment checks.
    #[must_use]
    pub fn from_parts_with_data(
        plot: Plot<K>,
        metadata: BindingMetadata<K>,
        source: &DataFrame,
    ) -> PlotAdapter<'_, K> {
        PlotAdapter {
            plot,
            source: Some(source),
            bindings: metadata,
        }
    }
}

macro_rules! category_binding {
    ($name:ident, $capability:ident) => {
        impl<K: kind::$capability> PlotAdapter<'_, K> {
            #[doc = concat!("Bind the native ", stringify!($name), " role from a field or categorical variable.")]
            pub fn $name(mut self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
                let values = self.category_rows(values, stringify!($name))?;
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
