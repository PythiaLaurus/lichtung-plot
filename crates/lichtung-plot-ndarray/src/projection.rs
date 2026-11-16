//! Sparse borrowed columns in the original source coordinate system.

use std::collections::HashSet;

use crate::error::{binding, BindingResult};
use crate::input::sealed::{self, AxisKind};
use crate::{AdapterError, ArraySource, AxisValues, CategoryValues, NumericValues};

/// Borrowed one-column sources addressed by their original logical indices.
///
/// The shape is the actual source's `(rows, logical_column_count)`, not the
/// number of supplied columns. Each entry supplies a two-dimensional view or
/// values/present pair with shape `(rows, 1)`. Different entries may have
/// different element types, strides and validity. Construction validates only
/// descriptor structure; selecting a role snapshots only its supplied values.
///
/// Existing scalar and ordered index selectors retain their meaning. An
/// in-bounds column without a value entry fails explicitly; `..` still means
/// every original column, not every supplied entry. Grid positions and source
/// row IDs remain independent of field identities.
/// [`Self::from_identities`] supplies the same original namespace without any
/// value entries when only field-identity resolution is needed.
///
/// ```
/// use lichtung_plot_ndarray::{ArraySource, ColumnProjection, Line};
/// use ndarray::array;
///
/// let x = array![[0.], [5.], [20.]];
/// let y7 = array![[4.], [5.], [6.]];
/// let y2 = array![[1.], [2.], [3.]];
/// let columns: [(usize, &dyn ArraySource); 3] =
///     [(4, &x.view()), (7, &y7.view()), (2, &y2.view())];
/// let identities = [(4, "time"), (7, "treated"), (2, "control")];
/// let source = ColumnProjection::new((3, 10), &columns)?
///     .with_identities(&identities)?;
/// let adapted = Line::from_data(source, 4usize, [7usize, 2usize])?;
/// let (plot, metadata) = adapted.into_parts(); // releases all source borrows
/// let _restored = Line::from_parts(plot, metadata);
/// # Ok::<(), lichtung_plot_ndarray::AdapterError>(())
/// ```
pub struct ColumnProjection<'columns> {
    source_shape: (usize, usize),
    columns: &'columns [(usize, &'columns dyn ArraySource)],
    identities: Option<&'columns [(usize, &'columns str)]>,
}

impl<'columns> ColumnProjection<'columns> {
    /// Validate and borrow nonempty, unique, in-bounds one-column descriptors.
    ///
    /// Retains original coordinates and positional `Column N` identities without
    /// reading observations. Each source must have shape `(source_shape.0, 1)`
    /// and pass its existing validity-shape validation.
    pub fn new(
        source_shape: (usize, usize),
        columns: &'columns [(usize, &'columns dyn ArraySource)],
    ) -> std::result::Result<Self, AdapterError> {
        let projection = Self {
            source_shape,
            columns,
            identities: None,
        };
        projection.validate()?;
        Ok(projection)
    }

    /// Borrow an explicit original namespace without projecting observations.
    ///
    /// The shape is the actual source shape, not the number of identities or
    /// selector rows. Indices and raw identities follow [`Self::with_identities`]
    /// validation. An empty identity table remains explicit: it does not enable
    /// positional-name fallback. Every value lookup still fails as unprojected.
    ///
    /// This uses the same descriptor representation; [`Self::new`] continues to
    /// require at least one value entry.
    pub fn from_identities(
        source_shape: (usize, usize),
        identities: &'columns [(usize, &'columns str)],
    ) -> std::result::Result<Self, AdapterError> {
        let projection = Self {
            source_shape,
            columns: &[],
            identities: Some(identities),
        };
        projection.validate()?;
        Ok(projection)
    }

    /// Supply exact raw field identities separately from coordinates and labels.
    ///
    /// Indices and raw strings must each be unique, all indices must be in the
    /// original bounds, and every value entry needs an identity. Additional
    /// in-bounds identities may describe unprojected columns, for example targets
    /// named by a generated-column alpha key variable. They do not grant value
    /// lookup. An explicit table never falls back to positional identities.
    ///
    /// Replaces any previous table completely. Strings are borrowed unchanged:
    /// empty/whitespace names and display collisions retain native validation.
    /// No observation is read, and neither this table nor its borrow enters the
    /// owned native Plot or binding metadata.
    pub fn with_identities(
        mut self,
        identities: &'columns [(usize, &'columns str)],
    ) -> std::result::Result<Self, AdapterError> {
        self.validate_identities(identities)?;
        self.identities = Some(identities);
        Ok(self)
    }

    fn validate_identities(&self, identities: &[(usize, &str)]) -> BindingResult<()> {
        let mut indices = HashSet::new();
        let mut names = HashSet::new();
        indices
            .try_reserve(identities.len())
            .map_err(|source| AdapterError::Allocation {
                operation: "column identities",
                source,
            })?;
        names
            .try_reserve(identities.len())
            .map_err(|source| AdapterError::Allocation {
                operation: "column identities",
                source,
            })?;
        for &(index, identity) in identities {
            crate::source::lookup(Some(self), index, "column identities")?;
            if !indices.insert(index) {
                return Err(binding(
                    "column identities",
                    format!("duplicate logical column {index}"),
                ));
            }
            if !names.insert(identity) {
                return Err(binding(
                    "column identities",
                    format!("duplicate raw identity {identity:?}"),
                ));
            }
        }
        for &(index, _) in self.columns {
            if !indices.contains(&index) {
                return Err(binding(
                    "column identities",
                    format!("no raw identity supplied for logical column {index}"),
                ));
            }
        }
        Ok(())
    }

    fn column(&self, index: usize, role: &'static str) -> BindingResult<&dyn ArraySource> {
        crate::source::lookup(Some(self), index, role)?;
        self.columns
            .iter()
            .find(|(column, _)| *column == index)
            .map(|(_, source)| *source)
            .ok_or_else(|| binding(role, format!("logical column {index} was not projected")))
    }
}

fn original_column(error: AdapterError, index: usize) -> AdapterError {
    match error {
        AdapterError::UnsupportedElement { role, element, .. } => {
            AdapterError::UnsupportedElement {
                role,
                element,
                column: Some(index),
            }
        }
        AdapterError::SourceRow { slot, value, .. } => AdapterError::SourceRow {
            slot,
            value,
            column: Some(index),
        },
        other => other,
    }
}

impl sealed::Sealed for ColumnProjection<'_> {
    fn column_identities(&self) -> Option<&[(usize, &str)]> {
        self.identities
    }

    fn axis_kind(&self, index: usize, role: &'static str) -> BindingResult<AxisKind> {
        sealed::Sealed::axis_kind(self.column(index, role)?, 0, role)
            .map_err(|error| original_column(error, index))
    }
}

macro_rules! projected_column {
    ($method:ident, $ty:ty) => {
        fn $method(&self, index: usize, role: &'static str) -> BindingResult<Vec<$ty>> {
            self.column(index, role)?
                .$method(0, role)
                .map_err(|error| original_column(error, index))
        }
    };
}

impl ArraySource for ColumnProjection<'_> {
    fn validate(&self) -> BindingResult<()> {
        if self.columns.is_empty() && self.identities.is_none() {
            return Err(binding(
                "column projection",
                "supply at least one value column",
            ));
        }
        let mut indices = HashSet::new();
        indices
            .try_reserve(self.columns.len())
            .map_err(|source| AdapterError::Allocation {
                operation: "column projection",
                source,
            })?;
        for &(index, source) in self.columns {
            crate::source::lookup(Some(self), index, "column projection")?;
            if !indices.insert(index) {
                return Err(binding(
                    "column projection",
                    format!("duplicate logical column {index}"),
                ));
            }
            source.validate()?;
            let actual = source.shape();
            if actual != (self.source_shape.0, 1) {
                return Err(AdapterError::Shape {
                    operation: "column projection",
                    expected: vec![self.source_shape.0, 1],
                    actual: vec![actual.0, actual.1],
                });
            }
        }
        if let Some(identities) = self.identities {
            self.validate_identities(identities)?;
        }
        Ok(())
    }

    fn shape(&self) -> (usize, usize) {
        self.source_shape
    }

    projected_column!(numeric_column, Option<f64>);
    projected_column!(category_column, Option<String>);
    projected_column!(boolean_column, Option<bool>);
    projected_column!(id_column, usize);
    projected_column!(byte_column, u8);

    fn axis_column(&self, index: usize, role: &'static str) -> BindingResult<AxisValues> {
        match sealed::Sealed::axis_kind(self, index, role)? {
            AxisKind::Numeric => {
                let values = self.numeric_column(index, role)?;
                Ok(AxisValues::Numeric(NumericValues::nullable(
                    crate::source::identity(Some(self), index, role)?,
                    values,
                )))
            }
            AxisKind::Category => {
                let values = self.category_column(index, role)?;
                Ok(AxisValues::Category(CategoryValues::nullable(
                    crate::source::identity(Some(self), index, role)?,
                    values,
                )?))
            }
        }
    }
}
