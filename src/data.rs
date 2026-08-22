//! Private semantic value access shared by plot renderers.

use crate::{BooleanValues, CategoryValues, Error, NumericValues};

pub(crate) const X: &str = "__x";
pub(crate) const Y: &str = "__y";
pub(crate) const HUE: &str = "__hue";
pub(crate) const GROUP: &str = "__group";
pub(crate) const COLOR: &str = "__color";
pub(crate) const INTERVAL_LOWER: &str = "__interval_lower";
pub(crate) const INTERVAL_UPPER: &str = "__interval_upper";
pub(crate) const STATUS: &str = "__status";
pub(crate) const HEATMAP_HIGHLIGHT: &str = "__heatmap_highlight";
pub(crate) const COEFFICIENT_SIGNIFICANCE: &str = "__coefficient_significance";
pub(crate) const BAR_SIGNIFICANCE: &str = "__bar_significance";
pub(crate) const INTERACTION_POINT_SHAPE: &str = "__interaction_point_shape";
pub(crate) const INTERACTION_LINE_PATTERN: &str = "__interaction_line_pattern";
pub(crate) const DUMBBELL_RIGHT: &str = "__dumbbell_right";
pub(crate) const FACET_ROW: &str = "__facet_row";
pub(crate) const FACET_COLUMN: &str = "__facet_column";
pub(crate) const FACET_WRAP: &str = "__facet_wrap";

#[derive(Debug, Clone, Copy)]
pub(crate) enum DataColumn<'a> {
    Numeric(&'a NumericValues),
    Category(&'a CategoryValues),
    Boolean(&'a BooleanValues),
}

impl<'a> DataColumn<'a> {
    pub(crate) const fn is_numeric(self) -> bool {
        matches!(self, Self::Numeric(_))
    }

    pub(crate) fn name(self) -> &'a str {
        match self {
            Self::Numeric(values) => values.name(),
            Self::Category(values) => values.name(),
            Self::Boolean(values) => values.name(),
        }
    }

    pub(crate) fn len(self) -> usize {
        match self {
            Self::Numeric(values) => values.len(),
            Self::Category(values) => values.len(),
            Self::Boolean(values) => values.len(),
        }
    }
}

/// A borrowed semantic plot view, optionally restricted to selected source rows.
#[derive(Clone, Copy)]
pub(crate) struct PlotData<'a> {
    plot: &'a crate::builder::Plot,
    rows: Option<&'a [usize]>,
    source_rows: Option<&'a [usize]>,
}

impl std::fmt::Debug for PlotData<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlotData")
            .field("height", &self.height())
            .field("is_full_plot", &self.is_full_plot())
            .finish_non_exhaustive()
    }
}

impl<'a> PlotData<'a> {
    pub(crate) const fn new(plot: &'a crate::builder::Plot) -> Self {
        Self {
            plot,
            rows: None,
            source_rows: None,
        }
    }

    pub(crate) fn map_source_rows(mut self, source_rows: &'a [usize]) -> Self {
        self.source_rows = Some(source_rows);
        self
    }

    pub(crate) const fn with_rows(
        plot: &'a crate::builder::Plot,
        rows: &'a [usize],
        source_rows: &'a [usize],
    ) -> Self {
        Self {
            plot,
            rows: Some(rows),
            source_rows: Some(source_rows),
        }
    }

    pub(crate) fn column(self, key: &str) -> Result<DataColumn<'a>, Error> {
        self.plot
            .data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))
    }

    pub(crate) fn height(self) -> usize {
        self.rows
            .map_or_else(|| self.plot.row_count(), <[usize]>::len)
    }

    pub(crate) const fn is_full_plot(self) -> bool {
        self.rows.is_none()
    }

    pub(crate) fn source_row(self, local_row: usize) -> usize {
        self.source_rows
            .and_then(|rows| rows.get(local_row))
            .copied()
            .unwrap_or_else(|| self.row_index(local_row))
    }

    fn row_index(self, local_row: usize) -> usize {
        self.rows
            .and_then(|rows| rows.get(local_row))
            .copied()
            .unwrap_or(local_row)
    }
}

impl<'a> From<&'a crate::builder::Plot> for PlotData<'a> {
    fn from(plot: &'a crate::builder::Plot) -> Self {
        Self::new(plot)
    }
}

pub(crate) fn strict_finite_f64<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<f64>, Error> {
    let data = data.into();
    let DataColumn::Numeric(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be numeric",
            data.column(column)?.name()
        )));
    };
    (0..data.height())
        .map(|row| match values.get(data.row_index(row)) {
            Some(Some(value)) if value.is_finite() => Ok(value),
            Some(Some(value)) => Err(Error::Data(format!(
                "{plot}: value {:?} contains non-finite value {value} at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            None => Err(length_error(plot, values.name(), row)),
        })
        .collect()
}

pub(crate) fn strict_numeric_values(values: &NumericValues, plot: &str) -> Result<Vec<f64>, Error> {
    (0..values.len())
        .map(|row| match values.get(row) {
            Some(Some(value)) if value.is_finite() => Ok(value),
            Some(Some(value)) => Err(Error::Data(format!(
                "{plot}: value {:?} contains non-finite value {value} at source row {}",
                values.name(),
                values.source_row(row)
            ))),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                values.source_row(row)
            ))),
            None => unreachable!("row is bounded by NumericValues::len"),
        })
        .collect()
}

pub(crate) fn strict_category_values(
    values: &CategoryValues,
    plot: &str,
) -> Result<Vec<String>, Error> {
    (0..values.len())
        .map(|row| match values.get(row) {
            Some(Some(value)) => Ok(value.to_owned()),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                values.source_row(row)
            ))),
            None => unreachable!("row is bounded by CategoryValues::len"),
        })
        .collect()
}

pub(crate) fn nullable_finite_f64<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<Option<f64>>, Error> {
    let data = data.into();
    let DataColumn::Numeric(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be numeric",
            data.column(column)?.name()
        )));
    };
    (0..data.height())
        .map(|row| match values.get(data.row_index(row)) {
            Some(Some(value)) if value.is_finite() => Ok(Some(value)),
            Some(Some(value)) => Err(Error::Data(format!(
                "{plot}: value {:?} contains non-finite value {value} at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            Some(None) => Ok(None),
            None => Err(length_error(plot, values.name(), row)),
        })
        .collect()
}

pub(crate) fn strict_bool<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<bool>, Error> {
    let data = data.into();
    let DataColumn::Boolean(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be Boolean",
            data.column(column)?.name()
        )));
    };
    (0..data.height())
        .map(|row| match values.get(data.row_index(row)) {
            Some(Some(value)) => Ok(value),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            None => Err(length_error(plot, values.name(), row)),
        })
        .collect()
}

pub(crate) fn aligned_finite_f64<'a>(
    data: impl Into<PlotData<'a>>,
    x_column: &str,
    y_column: &str,
    plot: &str,
) -> Result<Vec<(f64, f64)>, Error> {
    let data = data.into();
    let x = optional_finite_f64(data, x_column, plot)?;
    let y = optional_finite_f64(data, y_column, plot)?;
    if x.len() != y.len() {
        return Err(Error::Data(format!(
            "{plot}: aligned value length mismatch {} vs {}",
            x.len(),
            y.len()
        )));
    }
    Ok(x.into_iter().zip(y).filter_map(|(x, y)| x.zip(y)).collect())
}

pub(crate) fn aligned_finite_rows<'a>(
    data: impl Into<PlotData<'a>>,
    x_column: &str,
    y_column: &str,
    plot: &str,
) -> Result<Vec<(usize, f64, f64)>, Error> {
    let data = data.into();
    let x = optional_finite_f64(data, x_column, plot)?;
    let y = optional_finite_f64(data, y_column, plot)?;
    if x.len() != y.len() {
        return Err(Error::Data(format!(
            "{plot}: aligned value length mismatch {} vs {}",
            x.len(),
            y.len()
        )));
    }
    Ok(x.into_iter()
        .zip(y)
        .enumerate()
        .filter_map(|(row, (x, y))| Some((row, x?, y?)))
        .collect())
}

fn optional_finite_f64(
    data: PlotData<'_>,
    column: &str,
    plot: &str,
) -> Result<Vec<Option<f64>>, Error> {
    let DataColumn::Numeric(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be numeric",
            data.column(column)?.name()
        )));
    };
    (0..data.height())
        .map(|row| match values.get(data.row_index(row)) {
            Some(Some(value)) if value.is_infinite() => Err(Error::Data(format!(
                "{plot}: value {:?} contains infinite value {value} at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            Some(Some(value)) if value.is_nan() => Ok(None),
            Some(value) => Ok(value),
            None => Err(length_error(plot, values.name(), row)),
        })
        .collect()
}

#[derive(Debug)]
pub(crate) struct CollectiveChannels {
    pub(crate) groups: Option<Vec<String>>,
    pub(crate) hues: Option<Vec<String>>,
    pub(crate) group_levels: Vec<String>,
    pub(crate) hue_levels: Vec<String>,
}

pub(crate) fn collective_channels<'a>(
    data: impl Into<PlotData<'a>>,
    group_column: Option<&str>,
    hue_column: Option<&str>,
    cells: Option<&[String]>,
    plot: &str,
) -> Result<CollectiveChannels, Error> {
    let data = data.into();
    let groups = group_column
        .map(|column| strict_strings(data, column, plot))
        .transpose()?;
    let hues = hue_column
        .map(|column| strict_strings(data, column, plot))
        .transpose()?;
    let row_count = data.height();
    if groups
        .as_ref()
        .is_some_and(|values| values.len() != row_count)
        || hues
            .as_ref()
            .is_some_and(|values| values.len() != row_count)
        || cells.is_some_and(|values| values.len() != row_count)
    {
        return Err(Error::Data(format!(
            "{plot}: graphical group, hue, and mark-cell values must align with the plot input"
        )));
    }

    if let Some(hues_ref) = hues.as_ref() {
        let mut marks: Vec<(Option<&str>, Option<&str>, &str)> = Vec::new();
        for row in 0..row_count {
            let cell = cells.map(|values| values[row].as_str());
            let group = groups.as_ref().map(|values| values[row].as_str());
            let hue = hues_ref[row].as_str();
            if let Some((_, _, seen_hue)) = marks
                .iter()
                .find(|(seen_cell, seen_group, _)| *seen_cell == cell && *seen_group == group)
            {
                if *seen_hue != hue {
                    return Err(Error::Data(format!(
                        "{plot}: each collective mark must have exactly one hue; found {seen_hue:?} and {hue:?} at source row {}",
                        data.source_row(row)
                    )));
                }
            } else {
                marks.push((cell, group, hue));
            }
        }
    }

    Ok(CollectiveChannels {
        group_levels: groups
            .as_ref()
            .map_or_else(Vec::new, |values| stable_levels(values)),
        hue_levels: hues
            .as_ref()
            .map_or_else(Vec::new, |values| stable_levels(values)),
        groups,
        hues,
    })
}

pub(crate) fn stable_levels(values: &[String]) -> Vec<String> {
    let mut levels = Vec::new();
    for value in values {
        if !levels.contains(value) {
            levels.push(value.clone());
        }
    }
    levels
}

pub(crate) fn explicit_category_order(
    observed: &[String],
    requested: &[String],
    context: &str,
    allow_unobserved: bool,
) -> Result<Vec<String>, Error> {
    let mut seen = Vec::with_capacity(requested.len());
    for level in requested {
        if seen.contains(level) {
            return Err(Error::Data(format!(
                "{context}: explicit category order contains duplicate level {level:?}"
            )));
        }
        if !allow_unobserved && !observed.contains(level) {
            return Err(Error::Data(format!(
                "{context}: explicit category order contains unknown level {level:?}"
            )));
        }
        seen.push(level.clone());
    }
    if let Some(level) = observed.iter().find(|level| !seen.contains(level)) {
        return Err(Error::Data(format!(
            "{context}: explicit category order omits observed level {level:?}"
        )));
    }
    if allow_unobserved {
        seen.retain(|level| observed.contains(level));
    }
    Ok(seen)
}

pub(crate) fn strict_strings<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    let DataColumn::Category(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be categorical",
            data.column(column)?.name()
        )));
    };
    (0..data.height())
        .map(|row| match values.get(data.row_index(row)) {
            Some(Some(value)) => Ok(value.to_owned()),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            None => Err(length_error(plot, values.name(), row)),
        })
        .collect()
}

pub(crate) fn strict_normalized_strings<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
    role: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    strict_strings(data, column, plot)?
        .into_iter()
        .enumerate()
        .map(|(row, value)| {
            let normalized = collapse_whitespace(&value);
            if normalized.is_empty() {
                Err(Error::Data(format!(
                    "{plot}: {role} label is empty after whitespace normalization at source row {}",
                    data.source_row(row)
                )))
            } else {
                Ok(normalized)
            }
        })
        .collect()
}

pub(crate) fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn observed_levels_for_finite_pairs<'a>(
    data: impl Into<PlotData<'a>>,
    x_column: &str,
    y_column: &str,
    hue_column: &str,
    plot: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    let x = optional_finite_f64(data, x_column, plot)?;
    let y = optional_finite_f64(data, y_column, plot)?;
    let hue = strict_strings(data, hue_column, plot)?;
    let mut levels = Vec::new();
    for ((x, y), value) in x.into_iter().zip(y).zip(hue) {
        if x.is_some() && y.is_some() && !levels.contains(&value) {
            levels.push(value);
        }
    }
    Ok(levels)
}

pub(crate) fn min_max(
    values: impl IntoIterator<Item = f64>,
    plot: &str,
) -> Result<(f64, f64), Error> {
    let mut range: Option<(f64, f64)> = None;
    for value in values {
        if !value.is_finite() {
            return Err(Error::Data(format!(
                "{plot}: cannot compute range from non-finite value {value}"
            )));
        }
        range = Some(range.map_or((value, value), |(lo, hi)| (lo.min(value), hi.max(value))));
    }
    range.ok_or_else(|| Error::Data(format!("{plot}: cannot compute range from no values")))
}

fn length_error(plot: &str, name: &str, row: usize) -> Error {
    Error::Data(format!(
        "{plot}: value {name:?} is shorter than the aligned plot input at row {row}"
    ))
}
