//! Shared DataFrame extraction for plot renderers.

use std::collections::BTreeSet;

use crate::Error;
#[cfg(test)]
use polars::prelude::Series;
use polars::prelude::{AnyValue, DataFrame, DataType};

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotData<'a> {
    frame: &'a DataFrame,
    source_rows: Option<&'a [usize]>,
}

impl<'a> PlotData<'a> {
    pub(crate) const fn new(frame: &'a DataFrame) -> Self {
        Self {
            frame,
            source_rows: None,
        }
    }

    pub(crate) const fn with_source_rows(frame: &'a DataFrame, source_rows: &'a [usize]) -> Self {
        Self {
            frame,
            source_rows: Some(source_rows),
        }
    }

    pub(crate) fn source_row(self, local_row: usize) -> usize {
        self.source_rows
            .and_then(|rows| rows.get(local_row))
            .copied()
            .unwrap_or(local_row)
    }

    pub(crate) fn is_frame(self, frame: &DataFrame) -> bool {
        std::ptr::eq(self.frame, frame)
    }
}

impl std::ops::Deref for PlotData<'_> {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        self.frame
    }
}

impl<'a> From<&'a DataFrame> for PlotData<'a> {
    fn from(frame: &'a DataFrame) -> Self {
        Self::new(frame)
    }
}

pub(crate) fn strict_source_rows(
    data: &DataFrame,
    column: &str,
    plot: &str,
) -> Result<Vec<usize>, Error> {
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: source-row column {column:?}: {error}")))?;
    if !source.dtype().is_integer() {
        return Err(Error::Data(format!(
            "{plot}: source-row column {column:?} must have an integer dtype, got {:?}",
            source.dtype()
        )));
    }
    let values = source.cast(&DataType::UInt64).map_err(|error| {
        Error::Data(format!(
            "{plot}: source-row column {column:?} must contain non-negative integers: {error}"
        ))
    })?;
    let mut seen = BTreeSet::new();
    values
        .u64()
        .map_err(|error| {
            Error::Data(format!(
                "{plot}: source-row column {column:?} cannot be read as unsigned integers: {error}"
            ))
        })?
        .iter()
        .enumerate()
        .map(|(row, value)| {
            let value = value.ok_or_else(|| {
                Error::Data(format!(
                    "{plot}: source-row column {column:?} contains null at local row {row}"
                ))
            })?;
            let value = usize::try_from(value).map_err(|_| {
                Error::Data(format!(
                    "{plot}: source-row identity {value} at local row {row} exceeds platform limits"
                ))
            })?;
            if !seen.insert(value) {
                return Err(Error::Data(format!(
                    "{plot}: source-row column {column:?} contains duplicate identity {value}"
                )));
            }
            Ok(value)
        })
        .collect()
}

pub(crate) fn strict_finite_f64<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<f64>, Error> {
    let data = data.into();
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: column {column:?}: {error}")))?;
    if !source.dtype().is_numeric() {
        return Err(Error::Data(format!(
            "{plot}: column {column:?} must have a numeric dtype, got {:?}",
            source.dtype()
        )));
    }
    let values = source
        .cast(&DataType::Float64)
        .map_err(|error| Error::Data(format!("{plot}: cast {column:?}: {error}")))?;
    values
        .f64()
        .map_err(|error| Error::Data(format!("{plot}: as f64 {column:?}: {error}")))?
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Some(value) if value.is_finite() => Ok(value),
            Some(value) => Err(Error::Data(format!(
                "{plot}: column {column:?} contains non-finite value {value} at source row {}",
                data.source_row(index)
            ))),
            None => Err(Error::Data(format!(
                "{plot}: column {column:?} contains null at source row {}",
                data.source_row(index)
            ))),
        })
        .collect()
}

pub(crate) fn nullable_finite_f64<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<Option<f64>>, Error> {
    let data = data.into();
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: column {column:?}: {error}")))?;
    if !source.dtype().is_numeric() {
        return Err(Error::Data(format!(
            "{plot}: column {column:?} must have a numeric dtype, got {:?}",
            source.dtype()
        )));
    }
    let values = source
        .cast(&DataType::Float64)
        .map_err(|error| Error::Data(format!("{plot}: cast {column:?}: {error}")))?;
    values
        .f64()
        .map_err(|error| Error::Data(format!("{plot}: as f64 {column:?}: {error}")))?
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Some(value) if value.is_finite() => Ok(Some(value)),
            Some(value) => Err(Error::Data(format!(
                "{plot}: column {column:?} contains non-finite value {value} at source row {}",
                data.source_row(index)
            ))),
            None => Ok(None),
        })
        .collect()
}

pub(crate) fn strict_bool<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<bool>, Error> {
    let data = data.into();
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: column {column:?}: {error}")))?;
    if source.dtype() != &DataType::Boolean {
        return Err(Error::Data(format!(
            "{plot}: column {column:?} must have Boolean dtype, got {:?}",
            source.dtype()
        )));
    }
    source
        .bool()
        .map_err(|error| Error::Data(format!("{plot}: as bool {column:?}: {error}")))?
        .iter()
        .enumerate()
        .map(|(row, value)| {
            value.ok_or_else(|| {
                Error::Data(format!(
                    "{plot}: column {column:?} contains null at source row {}",
                    data.source_row(row)
                ))
            })
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
            "{plot}: aligned column length mismatch {} vs {}",
            x.len(),
            y.len()
        )));
    }
    Ok(x.into_iter().zip(y).filter_map(|(x, y)| x.zip(y)).collect())
}

fn optional_finite_f64(
    data: PlotData<'_>,
    column: &str,
    plot: &str,
) -> Result<Vec<Option<f64>>, Error> {
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: column {column:?}: {error}")))?;
    if !source.dtype().is_numeric() {
        return Err(Error::Data(format!(
            "{plot}: column {column:?} must have a numeric dtype, got {:?}",
            source.dtype()
        )));
    }
    let values = source
        .cast(&DataType::Float64)
        .map_err(|error| Error::Data(format!("{plot}: cast {column:?}: {error}")))?;
    values
        .f64()
        .map_err(|error| Error::Data(format!("{plot}: as f64 {column:?}: {error}")))?
        .iter()
        .enumerate()
        .map(|(row, value)| match value {
            Some(value) if value.is_infinite() => Err(Error::Data(format!(
                "{plot}: column {column:?} contains infinite value {value} at source row {}",
                data.source_row(row)
            ))),
            Some(value) if value.is_nan() => Ok(None),
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        })
        .collect()
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
            "{plot}: aligned column length mismatch {} vs {}",
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
            "{plot}: graphical group, hue, and mark-cell columns must align with the data"
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
                    let cell_context =
                        cell.map_or_else(String::new, |cell| format!(" in cell {cell:?}"));
                    let group_context = group
                        .map_or_else(String::new, |group| format!(" graphical group {group:?}"));
                    return Err(Error::Data(format!(
                        "{plot}:{group_context} collective mark{cell_context} contains multiple hue values {seen_hue:?} and {hue:?}; each collective mark must have exactly one hue (source row {})",
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

pub(crate) fn categorical_value_text(value: &AnyValue<'_>) -> String {
    match value {
        AnyValue::String(value) => (*value).to_owned(),
        AnyValue::StringOwned(value) => value.as_str().to_owned(),
        _ => value.to_string(),
    }
}

pub(crate) fn strict_strings<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    let source = data
        .column(column)
        .map_err(|error| Error::Data(format!("{plot}: column {column:?}: {error}")))?;
    for (row, value) in source.as_materialized_series().iter().enumerate() {
        let reason = match value {
            AnyValue::Null => Some("null"),
            AnyValue::Float16(value) if value.is_nan() => Some("NaN"),
            AnyValue::Float32(value) if value.is_nan() => Some("NaN"),
            AnyValue::Float64(value) if value.is_nan() => Some("NaN"),
            AnyValue::Float16(value) if value.0.is_infinite() => Some("infinite value"),
            AnyValue::Float32(value) if value.is_infinite() => Some("infinite value"),
            AnyValue::Float64(value) if value.is_infinite() => Some("infinite value"),
            _ => None,
        };
        if let Some(reason) = reason {
            return Err(Error::Data(format!(
                "{plot}: column {column:?} contains {reason} at source row {}",
                data.source_row(row)
            )));
        }
    }
    source
        .cast(&DataType::String)
        .map_err(|error| Error::Data(format!("{plot}: cast {column:?}: {error}")))?
        .str()
        .map_err(|error| Error::Data(format!("{plot}: as string {column:?}: {error}")))?
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.map(str::to_owned).ok_or_else(|| {
                Error::Data(format!(
                    "{plot}: column {column:?} contains null at source row {}",
                    data.source_row(index)
                ))
            })
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

#[cfg(test)]
pub(crate) fn observed_levels(series: &Series) -> Vec<AnyValue<'static>> {
    let mut levels = Vec::new();
    for value in series.iter().map(AnyValue::into_static) {
        if !matches!(value, AnyValue::Null) && !levels.iter().any(|seen| seen == &value) {
            levels.push(value);
        }
    }
    levels
}

pub(crate) fn observed_levels_for_finite_pairs<'a>(
    data: impl Into<PlotData<'a>>,
    x_column: &str,
    y_column: &str,
    hue_column: &str,
    plot: &str,
) -> Result<Vec<AnyValue<'static>>, Error> {
    let data = data.into();
    let x = optional_finite_f64(data, x_column, plot)?;
    let y = optional_finite_f64(data, y_column, plot)?;
    let hue = data
        .column(hue_column)
        .map_err(|error| Error::Data(format!("{plot}: column {hue_column:?}: {error}")))?
        .as_materialized_series();
    let mut levels = Vec::new();
    for (row, ((x, y), value)) in x.into_iter().zip(y).zip(hue.iter()).enumerate() {
        validate_required_category_value(&value, hue_column, plot, data.source_row(row))?;
        if x.is_none() || y.is_none() {
            continue;
        }
        let value = value.into_static();
        if !levels.iter().any(|seen| seen == &value) {
            levels.push(value);
        }
    }
    Ok(levels)
}

fn validate_required_category_value(
    value: &AnyValue<'_>,
    column: &str,
    plot: &str,
    row: usize,
) -> Result<(), Error> {
    let invalid = match value {
        AnyValue::Null => Some("null"),
        AnyValue::Float16(value) if value.is_nan() => Some("NaN"),
        AnyValue::Float32(value) if value.is_nan() => Some("NaN"),
        AnyValue::Float64(value) if value.is_nan() => Some("NaN"),
        AnyValue::Float16(value) if value.0.is_infinite() => Some("infinite"),
        AnyValue::Float32(value) if value.is_infinite() => Some("infinite"),
        AnyValue::Float64(value) if value.is_infinite() => Some("infinite"),
        _ => None,
    };
    if let Some(reason) = invalid {
        return Err(Error::Data(format!(
            "{plot}: required categorical column {column:?} contains {reason} at source row {row}"
        )));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use polars::prelude::{DataFrame, NamedFrom, Series};

    use super::*;

    #[test]
    fn aligned_finite_f64_preserves_row_alignment_when_filtering() {
        let x = Series::new("x".into(), [Some(1.0_f64), None, Some(3.0)]);
        let y = Series::new("y".into(), [Some(10.0_f64), Some(20.0), Some(f64::NAN)]);
        let data = DataFrame::new(x.len(), vec![x.into(), y.into()]).unwrap();

        assert_eq!(
            aligned_finite_f64(&data, "x", "y", "test").unwrap(),
            [(1.0, 10.0)]
        );
    }

    #[test]
    fn aligned_finite_f64_rejects_x_infinity_with_source_row() {
        let x = Series::new("x".into(), [1.0_f64, f64::INFINITY]);
        let y = Series::new("y".into(), [10.0_f64, 20.0]);
        let data = DataFrame::new(x.len(), vec![x.into(), y.into()]).unwrap();

        let error = aligned_finite_f64(&data, "x", "y", "scatter").unwrap_err();

        assert_eq!(
            error.to_string(),
            "data error: scatter: column \"x\" contains infinite value inf at source row 1"
        );
    }

    #[test]
    fn aligned_finite_f64_rejects_y_infinity_with_source_row() {
        let x = Series::new("x".into(), [1.0_f64, 2.0]);
        let y = Series::new("y".into(), [10.0_f64, f64::NEG_INFINITY]);
        let data = DataFrame::new(x.len(), vec![x.into(), y.into()]).unwrap();

        let error = aligned_finite_f64(&data, "x", "y", "scatter").unwrap_err();

        assert_eq!(
            error.to_string(),
            "data error: scatter: column \"y\" contains infinite value -inf at source row 1"
        );
    }

    #[test]
    fn strict_finite_f64_rejects_non_finite_values() {
        let values = Series::new("x".into(), [1.0_f64, f64::INFINITY]);
        let data = DataFrame::new(values.len(), vec![values.into()]).unwrap();

        assert!(strict_finite_f64(&data, "x", "test").is_err());
    }

    #[test]
    fn strict_finite_f64_rejects_numeric_looking_strings() {
        let values = Series::new("x".into(), ["1.0", "2.0"]);
        let data = DataFrame::new(values.len(), vec![values.into()]).unwrap();

        let error = strict_finite_f64(&data, "x", "test").unwrap_err();

        assert!(error.to_string().contains("must have a numeric dtype"));
    }

    #[test]
    fn strict_finite_f64_rejects_booleans() {
        let values = Series::new("x".into(), [true, false]);
        let data = DataFrame::new(values.len(), vec![values.into()]).unwrap();

        let error = strict_finite_f64(&data, "x", "test").unwrap_err();

        assert!(error.to_string().contains("must have a numeric dtype"));
    }

    #[test]
    fn strict_strings_casts_numeric_categories_and_rejects_nulls() {
        let numeric = Series::new("group".into(), [1_i64, 2, 1]);
        let numeric_data = DataFrame::new(numeric.len(), vec![numeric.into()]).unwrap();
        assert_eq!(
            strict_strings(&numeric_data, "group", "test").unwrap(),
            ["1", "2", "1"]
        );

        let nullable = Series::new("group".into(), [Some(1_i64), None]);
        let nullable_data = DataFrame::new(nullable.len(), vec![nullable.into()]).unwrap();
        assert!(strict_strings(&nullable_data, "group", "test").is_err());
    }

    #[test]
    fn strict_finite_f64_rejects_null_exactly() {
        let values = Series::new("intensity".into(), [Some(2.0_f64), None, Some(4.0)]);
        let data = DataFrame::new(values.len(), vec![values.into()]).unwrap();

        assert_eq!(
            strict_finite_f64(&data, "intensity", "mass_spectrum")
                .unwrap_err()
                .to_string(),
            "data error: mass_spectrum: column \"intensity\" contains null at source row 1"
        );
    }

    #[test]
    fn collective_channels_require_one_hue_per_ungrouped_mark() {
        let hues = Series::new("hue".into(), ["first", "second"]);
        let data = DataFrame::new(hues.len(), vec![hues.into()]).unwrap();

        let error = collective_channels(&data, None, Some("hue"), None, "line").unwrap_err();

        assert!(error
            .to_string()
            .contains("each collective mark must have exactly one hue"));
    }

    #[test]
    fn collective_channels_scope_ungrouped_hue_consistency_by_cell() {
        let hues = Series::new("hue".into(), ["first", "second"]);
        let data = DataFrame::new(hues.len(), vec![hues.into()]).unwrap();
        let cells = ["left".to_owned(), "right".to_owned()];

        let channels = collective_channels(&data, None, Some("hue"), Some(&cells), "bar").unwrap();

        assert!(channels.groups.is_none());
        assert_eq!(channels.hue_levels, ["first", "second"]);
    }

    #[test]
    fn observed_levels_preserves_first_seen_order() {
        let values = Series::new("group".into(), ["b", "a", "b", "c"]);

        let labels = observed_levels(&values)
            .into_iter()
            .map(|value| categorical_value_text(&value))
            .collect::<Vec<_>>();
        assert_eq!(labels, ["b", "a", "c"]);
    }

    #[test]
    fn categorical_value_text_preserves_literal_quotes() {
        let values = Series::new("group".into(), ["\"control\"", "'active'"]);
        let labels = observed_levels(&values)
            .into_iter()
            .map(|value| categorical_value_text(&value))
            .collect::<Vec<_>>();

        assert_eq!(labels, ["\"control\"", "'active'"]);
    }
}
