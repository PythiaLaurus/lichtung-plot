use super::{context, Context, PlotAdapter};
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, ArraySource, BooleanInput, CategoryInput, CategoryValues, ColumnsInput,
    NumericValues, Plot,
};

#[derive(Clone)]
pub(super) struct Grid {
    x: Vec<String>,
    y: Vec<String>,
    origins: Vec<usize>,
}
impl Grid {
    pub(super) fn columns(&self) -> usize {
        self.x.len()
    }
}
impl<K: kind::Marker> PlotAdapter<'_, K> {
    pub(super) fn cell_shape(
        &self,
        input: &impl crate::input::sealed::Sealed,
        role: &'static str,
    ) -> BindingResult<()> {
        let shape = if let Some(indices) = input.selected_columns() {
            let first = *indices
                .first()
                .ok_or_else(|| binding(role, "select at least one column"))?;
            let source = crate::source::lookup(self.source.as_deref(), first, role)?;
            Some((source.shape().0, indices.len()))
        } else {
            input.cell_shape()
        };
        if let Some(shape) = shape {
            let expected = self
                .grid
                .as_ref()
                .map(|grid| (grid.y.len(), grid.x.len()))
                .or_else(|| {
                    self.expansion
                        .as_ref()
                        .map(|wide| (wide.rows, wide.keys.len()))
                });
            if expected != Some(shape) {
                return Err(binding(
                    role,
                    format!("expected cell shape {expected:?}, got {shape:?}; selected source columns {:?}", self.grid.as_ref().map(|grid| &grid.origins)),
                ));
            }
        }
        Ok(())
    }
}
impl<'source> PlotAdapter<'source, kind::Heatmap> {
    /// Bind a numeric matrix with logical rows top-to-bottom and columns left-to-right.
    pub fn new(grid: impl ColumnsInput) -> std::result::Result<Self, AdapterError> {
        Self::grid_input(None, grid)
    }
    /// Select ordered source columns, or `..` for every column.
    pub fn from_data(
        source: impl ArraySource + 'source,
        columns: impl ColumnsInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::grid_input(context(source)?, columns)
    }
    fn grid_input(source: Context<'source>, columns: impl ColumnsInput) -> BindingResult<Self> {
        let (rows, columns) = columns.bind_columns(source.as_deref())?;
        let cells = rows
            .checked_mul(columns.len())
            .ok_or_else(|| binding("heatmap", "cell count exceeds usize"))?;
        let xkeys = (0..columns.len())
            .map(|index| format!("__lichtung_grid_x_{index}"))
            .collect::<Vec<_>>();
        let ykeys = (0..rows)
            .map(|index| format!("__lichtung_grid_y_{index}"))
            .collect::<Vec<_>>();
        let mut xs = crate::snapshot::allocate(cells, "heatmap x")?;
        let mut ys = crate::snapshot::allocate(cells, "heatmap y")?;
        let mut fill = crate::snapshot::allocate(cells, "heatmap fill")?;
        for (row, y) in ykeys.iter().enumerate() {
            for (column, x) in xkeys.iter().enumerate() {
                xs.push(x.as_str());
                ys.push(y.as_str());
                fill.push(columns[column].1[row]);
            }
        }
        let plot = Plot::heatmap(
            CategoryValues::new("Column", xs)?,
            CategoryValues::new("Row", ys)?,
            NumericValues::nullable("Value", fill),
        );
        let mut adapter = Self::bound(plot, source, Some(cells));
        adapter.grid = Some(Grid {
            x: xkeys,
            y: ykeys,
            origins: columns.iter().map(|(index, _)| *index).collect(),
        });
        adapter
            .x_labels(columns.iter().map(|(index, _)| crate::source::name(*index)))?
            .y_labels((0..rows).map(|row| format!("Row {}", row + 1)))
    }
    fn axis_labels<I, S>(mut self, axis: &'static str, values: I) -> BindingResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let grid = self
            .grid
            .as_ref()
            .ok_or_else(|| binding(axis, "axis metadata requires a rectangular heatmap"))?;
        let keys = if axis == "x labels" { &grid.x } else { &grid.y };
        let mut aliases = crate::snapshot::allocate(keys.len(), axis)?;
        for value in values {
            if aliases.len() == keys.len() {
                return Err(binding(axis, "more labels than grid positions"));
            }
            aliases.push(value.into());
        }
        length(axis, axis, keys.len(), aliases.len())?;
        self.labels = self.labels.aliases(keys.iter().cloned().zip(aliases));
        self.plot = self.plot.labels(self.labels.clone());
        Ok(self)
    }
    /// Assign literal column display labels without merging positional identities.
    pub fn x_labels<I, S>(self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.axis_labels("x labels", labels)
    }
    /// Assign literal row display labels without changing source membership.
    pub fn y_labels<I, S>(self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.axis_labels("y labels", labels)
    }
    /// Bind row display metadata from a categorical source column or direct vector.
    pub fn row_labels(self, values: impl CategoryInput) -> std::result::Result<Self, AdapterError> {
        let (_, values) = values.raw_category(self.source.as_deref(), "row labels")?;
        let values = values
            .into_iter()
            .enumerate()
            .map(|(slot, value)| {
                value.ok_or_else(|| binding("row labels", format!("missing label at slot {slot}")))
            })
            .collect::<BindingResult<Vec<_>>>()?;
        self.y_labels(values)
    }
    /// Bind Boolean records or a same-shape cell grid; true highlights, not masks.
    pub fn heatmap_highlight(
        mut self,
        values: impl BooleanInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.cell_shape(&values, "heatmap highlights")?;
        let values = values.bind_boolean(self.source.as_deref(), "heatmap highlights")?;
        self.aligned("heatmap highlights", values.name(), values.len())?;
        self.plot = self.plot.heatmap_highlight(values);
        Ok(self)
    }
    /// Bind explicitly expanded Boolean slots without row repetition.
    pub fn heatmap_highlight_cells(
        self,
        values: impl BooleanInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.heatmap_highlight(values)
    }
}
