//! Exact-family ownership and source-aware data binding.
use std::marker::PhantomData;
use std::rc::Rc;

use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, ArraySource, CategoryInput, CategoryValues, FacetScales, FacetSpec, Labels,
    Plot, PlotKind, SourceRowsInput,
};

mod alpha;
mod constructors;
mod forwarding;
mod geometry;
mod grid;
#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod projection_tests;
mod status;
mod wide;

pub(crate) type Context<'source> = Option<Rc<dyn ArraySource + 'source>>;
pub(crate) fn context<'source>(
    source: impl ArraySource + 'source,
) -> BindingResult<Context<'source>> {
    source.validate()?;
    Ok(Some(Rc::new(source)))
}

/// Source-free binding state paired with one native exact-family [`Plot`].
///
/// [`PlotAdapter::into_parts`] moves this state out without retaining the array
/// lookup context. It preserves alignment, generated origins and the adapter's
/// current native label/facet policies for later restoration.
///
/// Keep this metadata paired with the Plot configuration that produced it.
/// The family marker prevents cross-family pairing, not mismatched versions of
/// the same family. Metadata contains neither a Plot nor a lookup source.
#[derive(Clone)]
pub struct BindingMetadata<K: kind::Marker> {
    len: Option<usize>,
    geometry_points: Option<usize>,
    labels: Labels,
    facet: Option<FacetSpec>,
    grid: Option<grid::Grid>,
    expansion: Option<wide::Expansion>,
    matrix_keys: Option<Vec<String>>,
    marker: PhantomData<K>,
}

/// A native exact-family plot and an optional safely borrowed array lookup context.
/// Direct views are snapshotted and released at each binding; `.into()` releases
/// the retained source and moves the owned native plot without rendering.
/// [`Self::into_parts`] also preserves the source-free binding metadata.
#[derive(Clone)]
pub struct PlotAdapter<'source, K: kind::Marker> {
    plot: Plot<K>,
    pub(crate) source: Context<'source>,
    bindings: BindingMetadata<K>,
}

macro_rules! families {
    ($($family:ident),+) => {$ (
        #[doc = concat!("ndarray frontend for native ", stringify!($family), ".")]
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
    fn bound(plot: Plot<K>, source: Context<'source>, len: Option<usize>) -> Self {
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
                marker: PhantomData,
            },
        }
    }
    /// End source lookup and move out the native Plot with its binding metadata.
    ///
    /// This does not render, prepare scientific data or retain the source borrow.
    /// Keep both returned values from the same configuration together.
    ///
    /// ```
    /// use lichtung_plot_ndarray::{Scatter, PlotKind};
    /// use ndarray::array;
    ///
    /// let (plot, metadata) = {
    ///     let source = array![[0., 1.], [1., 0.]];
    ///     let adapted = Scatter::from_data(source.view(), 0, 1)?;
    ///     adapted.into_parts()
    /// };
    /// let restored = Scatter::from_parts(plot, metadata);
    /// assert_eq!(restored.kind(), PlotKind::Scatter);
    /// # Ok::<(), lichtung_plot_ndarray::AdapterError>(())
    /// ```
    pub fn into_parts(self) -> (Plot<K>, BindingMetadata<K>) {
        (self.plot, self.bindings)
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
    ) -> BindingResult<CategoryValues> {
        let values =
            values.bind_category_repeated(self.source.as_deref(), role, self.row_copies())?;
        self.aligned(role, values.name(), values.len())?;
        Ok(values)
    }
    /// Inspect the native exact family without erasing it.
    pub fn kind(&self) -> PlotKind {
        self.plot.kind()
    }
    /// Replace the entire native label policy, including generated aliases.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.bindings.labels = labels.clone();
        self.plot = self.plot.labels(labels);
        self
    }
    /// Bind unique exact IDs, never repeating row IDs across expanded cells.
    pub fn source_rows(
        mut self,
        values: impl SourceRowsInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        K: kind::SourceRows,
    {
        self.cell_shape(&values, "source rows")?;
        let values = values.bind_source_rows_repeated(
            self.source.as_deref(),
            "source rows",
            self.row_copies(),
        )?;
        self.aligned("source rows", "source rows", values.len())?;
        self.plot = self.plot.source_rows(values);
        Ok(self)
    }
    /// Apply a complete native facet without reinterpreting its slots.
    #[must_use]
    pub fn facet(mut self, facet: FacetSpec) -> Self
    where
        K: kind::Facet,
    {
        self.bindings.facet = Some(facet.clone());
        self.plot = self.plot.facet(facet);
        self
    }
    /// Split panel rows by a row-scoped categorical variable.
    pub fn facet_row(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = self.category_rows(values, "facet row")?;
        Ok(self.facet(FacetSpec::row(values)))
    }
    /// Split panel columns by a row-scoped categorical variable.
    pub fn facet_column(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError>
    where
        K: kind::Facet,
    {
        let values = self.category_rows(values, "facet column")?;
        Ok(self.facet(FacetSpec::column(values)))
    }
    /// Cross two row-scoped categorical variables, replacing the complete facet.
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
        Ok(self.facet(FacetSpec::grid(row, column)?))
    }
    /// Wrap a row-scoped categorical variable into a positive number of columns.
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
    /// Change scale sharing on an existing facet; each new facet resets this policy.
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
    /// Restore a native Plot and its metadata without a lookup source.
    ///
    /// No scientific validation or version check runs here. The caller must
    /// supply the matching Plot/metadata configuration; the family marker alone
    /// does not establish that they came from the same version.
    /// Direct variable bindings remain available; selectors need an explicit
    /// source through [`Self::from_parts_with_data`].
    ///
    /// Metadata from another family cannot be paired with this Plot:
    ///
    /// ```compile_fail
    /// use lichtung_plot_ndarray::{kind, BindingMetadata, Plot, Scatter};
    ///
    /// fn wrong_family(plot: Plot<kind::Scatter>, metadata: BindingMetadata<kind::Line>) {
    ///     let _ = Scatter::from_parts(plot, metadata);
    /// }
    /// ```
    #[must_use]
    pub fn from_parts(plot: Plot<K>, metadata: BindingMetadata<K>) -> Self {
        Self {
            plot,
            source: None,
            bindings: metadata,
        }
    }

    /// Restore a matching Plot/metadata pair with a new array lookup context.
    ///
    /// Validates the source's structural validity shape exactly as `from_data`
    /// does, without rebinding existing values, rendering or scientific preflight.
    /// The source height need not equal the old Plot's slots; subsequent bindings
    /// retain their existing domain-specific alignment checks.
    ///
    /// The caller must keep the Plot and metadata from the same configuration.
    pub fn from_parts_with_data<'source>(
        plot: Plot<K>,
        metadata: BindingMetadata<K>,
        source: impl ArraySource + 'source,
    ) -> std::result::Result<PlotAdapter<'source, K>, AdapterError> {
        Ok(PlotAdapter {
            plot,
            source: context(source)?,
            bindings: metadata,
        })
    }
}
macro_rules! category_binding {
    ($name:ident,$cap:ident) => {
        impl<K:kind::$cap> PlotAdapter<'_,K> {
            #[doc = concat!("Bind row-scoped native ",stringify!($name)," from a column or categorical vector.")]
            pub fn $name(mut self,values:impl CategoryInput) -> std::result::Result<Self,AdapterError> {
                let values=self.category_rows(values,stringify!($name))?; self.plot=self.plot.$name(values); Ok(self)
            }
        }
    };
}
category_binding!(hue, Hue);
category_binding!(group, Group);
category_binding!(series, Series);
impl<K: kind::ExactFamily> PlotAdapter<'_, K> {
    /// End source lookup and return the native erased plot.
    pub fn erase(self) -> Plot<kind::Any> {
        self.plot.erase()
    }
}
impl<K: kind::Marker> From<PlotAdapter<'_, K>> for Plot<K> {
    fn from(adapter: PlotAdapter<'_, K>) -> Self {
        adapter.plot
    }
}
