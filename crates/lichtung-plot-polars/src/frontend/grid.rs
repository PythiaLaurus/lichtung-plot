//! Rectangles retain private axis identities independently of display aliases.

use polars_core::prelude::DataFrame;

use super::PlotAdapter;
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, BooleanInput, CategoryValues, ColumnInput, ColumnsInput, NumericValues,
    Plot,
};

#[derive(Clone)]
pub(super) struct Grid {
    x: Vec<String>,
    y: Vec<String>,
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
        if let Some((rows, columns)) = input.cell_shape(self.source)? {
            let expected = self
                .grid
                .as_ref()
                .map(|grid| (grid.y.len(), grid.x.len()))
                .or_else(|| {
                    self.expansion
                        .as_ref()
                        .map(|wide| (wide.rows, wide.keys.len()))
                });
            if expected != Some((rows, columns)) {
                return Err(binding(
                    role,
                    format!("expected cell shape {expected:?}, got ({rows}, {columns})"),
                ));
            }
        }
        Ok(())
    }
}

fn cell_values(columns: &[Vec<Option<f64>>], rows: usize) -> BindingResult<Vec<Option<f64>>> {
    let cells = rows
        .checked_mul(columns.len())
        .ok_or_else(|| binding("heatmap", "cell count exceeds usize"))?;
    let mut values = crate::snapshot::allocate(cells, "heatmap fill")?;
    for row in 0..rows {
        for column in columns {
            values.push(column[row]);
        }
    }
    Ok(values)
}

impl<'source> PlotAdapter<'source, kind::Heatmap> {
    /// Bind a complete numeric DataFrame rectangle or ordered borrowed columns.
    /// This direct form does not install a field-lookup source.
    pub fn new(grid: impl ColumnsInput) -> std::result::Result<Self, AdapterError> {
        Self::grid_input(None, grid)
    }

    /// Bind explicitly selected grid columns, or `..` for every logical column.
    pub fn from_data(
        source: &'source DataFrame,
        columns: impl ColumnsInput,
    ) -> std::result::Result<Self, AdapterError> {
        Self::grid_input(Some(source), columns)
    }

    fn grid_input(
        source: Option<&'source DataFrame>,
        columns: impl ColumnsInput,
    ) -> BindingResult<Self> {
        let columns = columns.bind_columns(source)?;
        let rows = columns[0].len();
        let cells = rows
            .checked_mul(columns.len())
            .ok_or_else(|| binding("heatmap", "cell count exceeds usize"))?;
        let mut numeric = crate::snapshot::allocate(columns.len(), "heatmap columns")?;
        for column in &columns {
            length("heatmap", column.name().as_str(), rows, column.len())?;
            numeric.push(crate::numeric::values(column)?);
        }
        let mut xkeys = crate::snapshot::allocate(columns.len(), "heatmap x identities")?;
        let mut ykeys = crate::snapshot::allocate(rows, "heatmap y identities")?;
        xkeys.extend((0..columns.len()).map(|index| format!("__lichtung_grid_x_{index}")));
        ykeys.extend((0..rows).map(|index| format!("__lichtung_grid_y_{index}")));
        let mut xs = crate::snapshot::allocate(cells, "heatmap x")?;
        let mut ys = crate::snapshot::allocate(cells, "heatmap y")?;
        let fill = cell_values(&numeric, rows)?;
        for y in &ykeys {
            for x in &xkeys {
                xs.push(x.as_str());
                ys.push(y.as_str());
            }
        }
        let plot = Plot::heatmap(
            CategoryValues::new("Column", xs)?,
            CategoryValues::new("Row", ys)?,
            NumericValues::nullable("Value", fill),
        );
        let mut adapter = Self::bound(plot, source, Some(cells));
        adapter.grid = Some(Grid { x: xkeys, y: ykeys });
        let xlabels = columns.iter().enumerate().map(|(index, column)| {
            if column.name().is_empty() {
                format!("Column {}", index + 1)
            } else {
                column.name().to_string()
            }
        });
        let ylabels = (0..rows).map(|index| format!("Row {}", index + 1));
        adapter.x_labels(xlabels)?.y_labels(ylabels)
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

    /// Assign literal display labels to grid columns without changing their identities.
    pub fn x_labels<I, S>(self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.axis_labels("x labels", labels)
    }

    /// Assign literal display labels to grid rows without merging positions.
    pub fn y_labels<I, S>(self, labels: I) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.axis_labels("y labels", labels)
    }

    /// Bind grid-row display labels without changing positional row identity.
    pub fn row_labels(self, values: impl ColumnInput) -> std::result::Result<Self, AdapterError> {
        let column = values.bind_column(self.source, "row labels")?;
        let values = crate::category::values(&column)?
            .into_iter()
            .enumerate()
            .map(|(slot, value)| {
                value.ok_or_else(|| binding("row labels", format!("missing label at slot {slot}")))
            })
            .collect::<BindingResult<Vec<_>>>()?;
        self.y_labels(values)
    }

    /// Bind explicitly flattened cell highlights, independently of source-row scope.
    pub fn heatmap_highlight_cells(
        self,
        values: impl BooleanInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.heatmap_highlight(values)
    }

    /// Bind Boolean record highlights; true adds a border, not a validity mask.
    pub fn heatmap_highlight(
        mut self,
        values: impl BooleanInput,
    ) -> std::result::Result<Self, AdapterError> {
        self.cell_shape(&values, "heatmap highlights")?;
        let values = values.bind_boolean(self.source, "heatmap highlights")?;
        self.aligned("heatmap highlights", values.name(), values.len())?;
        self.plot = self.plot.heatmap_highlight(values);
        Ok(self)
    }
}
