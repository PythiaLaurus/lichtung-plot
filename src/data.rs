//! Private semantic value access shared by plot renderers.

use crate::{BooleanValues, CategoryValues, Error, NumericValues};

pub(crate) mod source;

pub(crate) const X: &str = "__x";
pub(crate) const Y: &str = "__y";
pub(crate) const HUE: &str = "__hue";
pub(crate) const GROUP: &str = "__group";
pub(crate) const SERIES: &str = "__series";
pub(crate) const COLOR: &str = "__color";
pub(crate) const INTERVAL_LOWER: &str = "__interval_lower";
pub(crate) const INTERVAL_UPPER: &str = "__interval_upper";
pub(crate) const BAR_INTERVAL_LOWER: &str = "__bar_interval_lower";
pub(crate) const BAR_INTERVAL_UPPER: &str = "__bar_interval_upper";
pub(crate) const BAR_UNITS: &str = "__bar_units";
pub(crate) const HEATMAP_HIGHLIGHT: &str = "__heatmap_highlight";
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

/// Global axis inputs normally installed on a generic facet child Plot.
#[derive(Clone, Copy)]
pub(crate) struct FacetAxisSources<'a> {
    pub(crate) hues: Option<&'a [String]>,
    pub(crate) groups: Option<&'a [String]>,
    pub(crate) conditions: Option<&'a [String]>,
}

/// A borrowed semantic plot view, optionally restricted to selected source rows.
#[derive(Clone, Copy)]
pub(crate) struct PlotData<'a> {
    plot: &'a crate::builder::Plot,
    rows: Option<&'a [usize]>,
    source_rows: Option<&'a [usize]>,
    operation: Option<&'a crate::font::canonical::OperationContext>,
    row_topology: Option<&'a source::RowSourceContext<'a>>,
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
            operation: None,
            row_topology: None,
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
            operation: None,
            row_topology: None,
        }
    }

    /// Axis source work uses the root counters; ordinary mark preparation remains separate.
    pub(crate) fn with_operation(
        mut self,
        resources: &'a crate::font::canonical::OperationContext,
    ) -> Self {
        self.operation = Some(resources);
        self
    }

    pub(crate) fn source_resources(
        self,
    ) -> Result<&'a crate::font::canonical::OperationContext, Error> {
        self.operation.ok_or_else(|| {
            Error::Render("numeric source requires the current root operation".into())
        })
    }

    pub(crate) fn column(self, key: &str) -> Result<DataColumn<'a>, Error> {
        #[cfg(test)]
        SOURCE_COLUMN_READS.with(|calls| calls.set(calls.get() + 1));
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

    pub(crate) fn estimate_statuses(self) -> Result<Option<Vec<crate::EstimateStatus>>, Error> {
        let Some(statuses) = self.plot.estimate_status_values() else {
            return Ok(None);
        };
        crate::data::source::collect_source((0..self.height())
            .map(|row| {

                statuses.get(self.row_index(row)).copied().ok_or_else(|| {
                    Error::Data(format!(
                        "estimate status values are shorter than the aligned plot input at source row {}",
                        self.source_row(row)
                    ))
                })
            }))
            .map(Some)
    }

    pub(crate) fn row_index(self, local_row: usize) -> usize {
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
    crate::data::source::collect_source((0..data.height()).map(|row| {
        match values.get(data.row_index(row)) {
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
        }
    }))
}

#[cfg(test)]
thread_local! {
    pub(crate) static SOURCE_COLUMN_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) fn strict_numeric_value(
    values: &NumericValues,
    row: usize,
    plot: &str,
) -> Result<f64, Error> {
    match values.get(row) {
        Some(Some(value)) if value.is_finite() => Ok(value),
        Some(Some(value)) => Err(Error::Data(format!(
            "{plot}: value {:?} contains non-finite value {value} at source row {}",
            values.name(),
            row
        ))),
        Some(None) => Err(Error::Data(format!(
            "{plot}: value {:?} contains null at source row {}",
            values.name(),
            row
        ))),
        None => unreachable!("row is bounded by NumericValues::len"),
    }
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
    crate::data::source::collect_source((0..data.height()).map(|row| {
        match values.get(data.row_index(row)) {
            Some(Some(value)) if value.is_finite() => Ok(Some(value)),
            Some(Some(value)) => Err(Error::Data(format!(
                "{plot}: value {:?} contains non-finite value {value} at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            Some(None) => Ok(None),
            None => Err(length_error(plot, values.name(), row)),
        }
    }))
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
    crate::data::source::collect_source((0..data.height()).map(|row| {
        match values.get(data.row_index(row)) {
            Some(Some(value)) => Ok(value),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            None => Err(length_error(plot, values.name(), row)),
        }
    }))
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
    let mut pairs = Vec::new();
    crate::data::source::reserve_source(&mut pairs, x.len())?;

    pairs.extend(x.into_iter().zip(y).filter_map(|(x, y)| x.zip(y)));
    Ok(pairs)
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
    let mut rows = Vec::new();
    crate::data::source::reserve_source(&mut rows, x.len())?;

    rows.extend(
        x.into_iter()
            .zip(y)
            .enumerate()
            .filter_map(|(row, (x, y))| Some((row, x?, y?))),
    );
    Ok(rows)
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
    crate::data::source::collect_source((0..data.height()).map(|row| {
        match values.get(data.row_index(row)) {
            Some(Some(value)) if value.is_infinite() => Err(Error::Data(format!(
                "{plot}: value {:?} contains infinite value {value} at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            Some(Some(value)) if value.is_nan() => Ok(None),
            Some(value) => Ok(value),
            None => Err(length_error(plot, values.name(), row)),
        }
    }))
}

#[derive(Debug)]
pub(crate) struct CollectiveChannels {
    pub(crate) groups: Option<Vec<String>>,
    pub(crate) hues: Option<Vec<String>>,
    pub(crate) group_levels: Vec<String>,
    pub(crate) hue_levels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CollectivePair {
    pub(crate) group: Option<String>,
    pub(crate) hue: Option<String>,
}

impl CollectivePair {
    pub(crate) fn description(&self, collapsed: bool) -> String {
        match (&self.group, &self.hue, collapsed) {
            (Some(group), _, true) | (Some(group), None, false) => {
                format!("graphical group {group:?}")
            }
            (Some(group), Some(hue), false) => {
                format!("graphical group {group:?} with hue {hue:?}")
            }
            (None, Some(hue), _) => format!("hue {hue:?}"),
            (None, None, _) => "ungrouped series".to_string(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ObservedCollectives {
    pub(crate) rows: Vec<CollectivePair>,
    pub(crate) identities: Vec<CollectivePair>,
    pub(crate) group_levels: Vec<String>,
    pub(crate) hue_levels: Vec<String>,
    pub(crate) collapsed: bool,
}

pub(crate) fn observed_collectives<'a>(
    data: impl Into<PlotData<'a>>,
    group_column: Option<&str>,
    group_name: Option<&str>,
    hue_column: Option<&str>,
    hue_name: Option<&str>,
    retained_rows: Option<&[usize]>,
    plot: &str,
) -> Result<ObservedCollectives, Error> {
    let data = data.into();
    let mut row_indices = Vec::new();
    crate::data::source::reserve_source(
        &mut row_indices,
        retained_rows.map_or(data.height(), <[usize]>::len),
    )?;
    if let Some(rows) = retained_rows {
        row_indices.extend_from_slice(rows);
    } else {
        row_indices.extend(0..data.height());
    }
    let groups = group_column
        .map(|column| strict_strings_for_rows(data, column, &row_indices, plot))
        .transpose()?;
    let hues = hue_column
        .map(|column| strict_strings_for_rows(data, column, &row_indices, plot))
        .transpose()?;
    let collapsed = group_name
        .zip(hue_name)
        .is_some_and(|(group, hue)| group == hue);
    if collapsed {
        let (Some(groups), Some(hues)) = (groups.as_ref(), hues.as_ref()) else {
            return Err(Error::Data(format!(
                "{plot}: equal graphical group and hue names require both categorical channels"
            )));
        };
        if let Some(index) = groups
            .iter()
            .zip(hues)
            .position(|(group, hue)| group != hue)
        {
            return Err(Error::Data(format!(
                "{plot}: graphical group and hue share semantic name {:?} but differ at source row {}",
                group_name.unwrap_or_default(),
                data.source_row(row_indices[index])
            )));
        }
    }

    let rows = crate::data::source::collect_source((0..row_indices.len()).map(|index| {
        Ok(CollectivePair {
            group: groups
                .as_ref()
                .map(|values| crate::data::source::copy_source_string(&values[index]))
                .transpose()?,
            hue: hues
                .as_ref()
                .map(|values| crate::data::source::copy_source_string(&values[index]))
                .transpose()?,
        })
    }))?;
    let group_levels = groups.as_ref().map_or_else(
        || Ok(Vec::new()),
        |values| crate::data::source::source_levels(values),
    )?;
    let hue_levels = hues.as_ref().map_or_else(
        || Ok(Vec::new()),
        |values| crate::data::source::source_levels(values),
    )?;
    let mut identities = Vec::new();
    crate::data::source::reserve_source(&mut identities, rows.len())?;
    if groups.is_some() {
        for group in &group_levels {
            for identity in &rows {
                if identity.group.as_ref() != Some(group) {
                    continue;
                }
                if crate::data::source::source_position(&identities, |seen| seen == identity)
                    .is_none()
                {
                    identities.push(crate::data::source::copy_source_pair(identity)?);
                }
            }
        }
    } else {
        for identity in &rows {
            if crate::data::source::source_position(&identities, |seen| seen == identity).is_none()
            {
                identities.push(crate::data::source::copy_source_pair(identity)?);
            }
        }
    }

    Ok(ObservedCollectives {
        rows,
        identities,
        group_levels,
        hue_levels,
        collapsed,
    })
}

fn strict_strings_for_rows(
    data: PlotData<'_>,
    column: &str,
    rows: &[usize],
    plot: &str,
) -> Result<Vec<String>, Error> {
    let DataColumn::Category(values) = data.column(column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be categorical",
            data.column(column)?.name()
        )));
    };
    crate::data::source::collect_source(rows.iter().map(
        |row| match values.get(data.row_index(*row)) {
            Some(Some(value)) => crate::data::source::copy_source_string(value),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(*row)
            ))),
            None => Err(length_error(plot, values.name(), *row)),
        },
    ))
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
        crate::data::source::reserve_source(&mut marks, row_count)?;
        for row in 0..row_count {
            let cell = cells.map(|values| values[row].as_str());
            let group = groups.as_ref().map(|values| values[row].as_str());
            let hue = hues_ref[row].as_str();
            if let Some(index) =
                crate::data::source::source_position(&marks, |(seen_cell, seen_group, _)| {
                    *seen_cell == cell && *seen_group == group
                })
            {
                let (_, _, seen_hue) = &marks[index];
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
        group_levels: groups.as_ref().map_or_else(
            || Ok(Vec::new()),
            |values| crate::data::source::source_levels(values),
        )?,
        hue_levels: hues.as_ref().map_or_else(
            || Ok(Vec::new()),
            |values| crate::data::source::source_levels(values),
        )?,
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
    let mut seen = Vec::new();
    crate::data::source::reserve_source(&mut seen, requested.len())?;
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
        seen.push(crate::data::source::copy_source_string(level)?);
    }
    for level in observed {
        if !seen.contains(level) {
            return Err(Error::Data(format!(
                "{context}: explicit category order omits observed level {level:?}"
            )));
        }
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
    crate::data::source::collect_source((0..data.height()).map(|row| {
        match values.get(data.row_index(row)) {
            Some(Some(value)) => crate::data::source::copy_source_string(value),
            Some(None) => Err(Error::Data(format!(
                "{plot}: value {:?} contains null at source row {}",
                values.name(),
                data.source_row(row)
            ))),
            None => Err(length_error(plot, values.name(), row)),
        }
    }))
}

pub(crate) fn strict_normalized_strings<'a>(
    data: impl Into<PlotData<'a>>,
    column: &str,
    plot: &str,
    role: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    crate::data::source::collect_source(
        strict_strings(data, column, plot)?
            .into_iter()
            .enumerate()
            .map(|(row, value)| {
                let normalized = crate::data::source::normalize_source_string(&value)?;
                if normalized.is_empty() {
                    Err(Error::Data(format!(
                    "{plot}: {role} label is empty after whitespace normalization at source row {}",
                    data.source_row(row)
                )))
                } else {
                    Ok(normalized)
                }
            }),
    )
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

pub(crate) fn observed_levels_for_retained_finite_pairs<'a>(
    data: impl Into<PlotData<'a>>,
    x_column: &str,
    y_column: &str,
    hue_column: &str,
    plot: &str,
) -> Result<Vec<String>, Error> {
    let data = data.into();
    let x = optional_finite_f64(data, x_column, plot)?;
    let y = optional_finite_f64(data, y_column, plot)?;
    let DataColumn::Category(hue) = data.column(hue_column)? else {
        return Err(Error::Data(format!(
            "{plot}: value {:?} must be categorical",
            data.column(hue_column)?.name()
        )));
    };
    let mut levels = Vec::new();
    for row in 0..data.height() {
        if x[row].is_none() || y[row].is_none() {
            continue;
        }
        let value = match hue.get(data.row_index(row)) {
            Some(Some(value)) => value,
            Some(None) => {
                return Err(Error::Data(format!(
                    "{plot}: value {:?} contains null at source row {}",
                    hue.name(),
                    data.source_row(row)
                )));
            }
            None => return Err(length_error(plot, hue.name(), row)),
        };
        if !levels.iter().any(|level| level == value) {
            levels.push(value.to_owned());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Plot;

    #[test]
    fn observed_collectives_order_groups_then_hues_within_each_group() {
        let plot = Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 2.0, 3.0, 4.0]),
        )
        .group(crate::CategoryValues::new("group", ["g1", "g2", "g2", "g1", "g1"]).unwrap())
        .hue(crate::CategoryValues::new("hue", ["h2", "h1", "h2", "h1", "h2"]).unwrap())
        .erase();

        let channels = observed_collectives(
            &plot,
            plot.group_col(),
            plot.group_name(),
            plot.hue_col(),
            plot.hue_name(),
            None,
            "test",
        )
        .unwrap();

        assert_eq!(
            channels.identities,
            [
                CollectivePair {
                    group: Some("g1".into()),
                    hue: Some("h2".into()),
                },
                CollectivePair {
                    group: Some("g1".into()),
                    hue: Some("h1".into()),
                },
                CollectivePair {
                    group: Some("g2".into()),
                    hue: Some("h1".into()),
                },
                CollectivePair {
                    group: Some("g2".into()),
                    hue: Some("h2".into()),
                },
            ]
        );
    }
}
