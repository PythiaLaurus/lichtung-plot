//! Private semantic value access shared by plot renderers.

use crate::{BooleanValues, CategoryValues, Error, NumericValues};

pub(crate) mod category;
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
pub(crate) struct PlotData<'src, 'op> {
    source: &'src crate::builder::PlotDescription,
    rows: Option<&'op [usize]>,
    diagnostic_rows: DiagnosticRows<'src, 'op>,
    numeric_facet: Option<&'op crate::FacetCell>,
    operation: &'op crate::font::canonical::OperationContext,
    row_source: &'op source::RowSourceContext<'src>,
}

#[derive(Clone, Copy)]
pub(crate) enum DiagnosticRows<'src, 'op> {
    Native(Option<&'src crate::SourceRows>),
    Selected(&'op [usize]),
}

impl std::fmt::Debug for PlotData<'_, '_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlotData")
            .field("height", &self.height())
            .field("is_full_plot", &self.is_full_plot())
            .finish_non_exhaustive()
    }
}

impl<'src, 'op> PlotData<'src, 'op> {
    pub(crate) fn checked(
        source: &'src crate::builder::PlotDescription,
        scope: &'op crate::orchestrator::registration::Scope,
        row_source: &'op source::RowSourceContext<'src>,
        operation: &'op crate::font::canonical::OperationContext,
    ) -> Result<Self, Error> {
        row_source.check(source, scope, operation)?;
        let (rows, diagnostic_rows) = scope.rows.selected().map_or_else(
            || (None, DiagnosticRows::Native(source.source_row_map())),
            |(rows, diagnostic_rows)| (Some(rows), DiagnosticRows::Selected(diagnostic_rows)),
        );
        Ok(Self {
            source,
            rows,
            diagnostic_rows,
            numeric_facet: scope.numeric_owner.facet.as_ref(),
            operation,
            row_source,
        })
    }

    pub(crate) const fn numeric_facet(self) -> Option<&'op crate::FacetCell> {
        self.numeric_facet
    }

    pub(crate) fn facet_selector(
        self,
    ) -> Result<Option<crate::orchestrator::registration::FacetSelector<'src, 'op>>, Error> {
        if self.height() == 0 {
            return Ok(None);
        }
        self.source
            .facet_spec()
            .zip(self.numeric_facet())
            .map(|(schema, levels)| {
                crate::orchestrator::registration::FacetSelector::checked(schema, levels)
            })
            .transpose()
    }

    pub(crate) const fn source_resources(self) -> &'op crate::font::canonical::OperationContext {
        self.operation
    }

    pub(crate) const fn plot_id(self) -> &'op crate::orchestrator::registration::PlotOccurrenceId {
        self.row_source.owner()
    }

    pub(crate) fn column(self, key: &str) -> Result<DataColumn<'src>, Error> {
        #[cfg(test)]
        SOURCE_COLUMN_READS.with(|calls| calls.set(calls.get() + 1));
        self.source
            .data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))
    }

    pub(crate) fn height(self) -> usize {
        self.rows
            .map_or_else(|| self.source.row_count(), <[usize]>::len)
    }

    pub(crate) const fn is_full_plot(self) -> bool {
        self.rows.is_none()
    }

    pub(crate) fn source_row(self, local_row: usize) -> usize {
        let diagnostic = match self.diagnostic_rows {
            DiagnosticRows::Native(rows) => rows.and_then(|rows| rows.as_slice().get(local_row)),
            DiagnosticRows::Selected(rows) => rows.get(local_row),
        };
        diagnostic
            .copied()
            .unwrap_or_else(|| self.row_index(local_row))
    }

    pub(crate) fn estimate_statuses(self) -> Result<Option<Vec<crate::EstimateStatus>>, Error> {
        let Some(statuses) = self.source.estimate_status_values() else {
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

    /// Alpha arrays align with complete original input slots, not public row IDs.
    pub(crate) fn observation_alpha(
        self,
        alpha: &crate::alpha::PreparedAlpha,
        local_row: usize,
    ) -> Result<Option<crate::alpha::ResolvedAlpha>, Error> {
        if local_row >= self.height() {
            return Err(Error::Data(format!(
                "alpha local row {local_row} is outside the selected input"
            )));
        }
        alpha.observation(self.row_index(local_row), self.source_row(local_row))
    }
}

/// Isolated scientific fixtures borrow the same build binding as production.
/// No prepared owner or operation-backed view can escape the callback.
#[cfg(test)]
pub(crate) fn with_source_data_for_test<'src, T>(
    source: &'src crate::Plot,
    operation: &crate::font::canonical::OperationContext,
    rows: crate::orchestrator::registration::RowSelection,
    visit: impl FnOnce(PlotData<'src, '_>) -> T,
) -> Result<T, Error> {
    let scope = crate::orchestrator::registration::Scope::new(
        source.description(),
        operation,
        0,
        crate::layout::ComponentAddress::root().child(crate::layout::AddressRole::PlotBody, 0)?,
        rows,
        None,
        None,
    )?;
    let mut registry =
        crate::orchestrator::registration::PlotRegistryBuilder::new(operation.root_identity());
    let policy = crate::orchestrator::registration::PlotScopePolicy::for_source(
        source.description(),
        scope.numeric_owner.facet.as_ref(),
    );
    let id = registry.reserve(
        source.description(),
        scope.address,
        scope.numeric_owner,
        scope.axis_numeric_owner,
        scope.rows,
        scope.render_name,
        policy,
        source.description().resolve_theme()?,
        operation,
    )?;
    Ok(visit(registry.data(&id, operation)?))
}

pub(crate) fn strict_finite_f64(
    data: PlotData<'_, '_>,
    column: &str,
    plot: &str,
) -> Result<Vec<f64>, Error> {
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

pub(crate) fn strict_category_at<'a>(
    values: &'a CategoryValues,
    input_slot: usize,
    diagnostic_row: usize,
    context: &str,
) -> Result<&'a str, Error> {
    match values.get(input_slot) {
        Some(Some(value)) => Ok(value),
        Some(None) => Err(Error::Data(format!(
            "{context}: value {:?} contains null at source row {diagnostic_row}",
            values.name()
        ))),
        None => Err(length_error(context, values.name(), input_slot)),
    }
}

pub(crate) fn strict_numeric_at(
    values: &NumericValues,
    input_slot: usize,
    diagnostic_row: usize,
    context: &str,
) -> Result<f64, Error> {
    match values.get(input_slot) {
        Some(Some(value)) if value.is_finite() => Ok(value),
        Some(Some(value)) => Err(Error::Data(format!(
            "{context}: value {:?} contains non-finite value {value} at source row {diagnostic_row}",
            values.name()
        ))),
        Some(None) => Err(Error::Data(format!(
            "{context}: value {:?} contains null at source row {diagnostic_row}",
            values.name()
        ))),
        None => Err(length_error(context, values.name(), input_slot)),
    }
}

pub(crate) fn strict_source_strings(
    values: &CategoryValues,
    rows: Option<&crate::SourceRows>,
    context: &str,
) -> Result<Vec<String>, Error> {
    source::collect_source((0..values.len()).map(|slot| {
        let diagnostic = rows
            .and_then(|rows| rows.as_slice().get(slot))
            .copied()
            .unwrap_or(slot);
        source::copy_source_string(strict_category_at(values, slot, diagnostic, context)?)
    }))
}

pub(crate) fn strict_source_f64(
    values: &NumericValues,
    rows: Option<&crate::SourceRows>,
    context: &str,
) -> Result<Vec<f64>, Error> {
    source::collect_source((0..values.len()).map(|slot| {
        let diagnostic = rows
            .and_then(|rows| rows.as_slice().get(slot))
            .copied()
            .unwrap_or(slot);
        strict_numeric_at(values, slot, diagnostic, context)
    }))
}

pub(crate) fn strict_source_normalized_strings(
    values: &CategoryValues,
    rows: Option<&crate::SourceRows>,
    context: &str,
    role: &str,
) -> Result<Vec<String>, Error> {
    source::collect_source(strict_source_strings(values, rows, context)?.into_iter().enumerate().map(|(slot, value)| {
        let normalized = source::normalize_source_string(&value)?;
        if normalized.is_empty() {
            let diagnostic = rows.and_then(|rows| rows.as_slice().get(slot)).copied().unwrap_or(slot);
            Err(Error::Data(format!("{context}: {role} label is empty after whitespace normalization at source row {diagnostic}")))
        } else {
            Ok(normalized)
        }
    }))
}

pub(crate) fn nullable_finite_f64(
    data: PlotData<'_, '_>,
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

pub(crate) fn strict_bool(
    data: PlotData<'_, '_>,
    column: &str,
    plot: &str,
) -> Result<Vec<bool>, Error> {
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

pub(crate) fn aligned_finite_rows(
    data: PlotData<'_, '_>,
    x_column: &str,
    y_column: &str,
    plot: &str,
) -> Result<Vec<(usize, f64, f64)>, Error> {
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
    data: PlotData<'_, '_>,
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

pub(crate) fn observed_collectives(
    data: PlotData<'_, '_>,
    group_column: Option<&str>,
    group_name: Option<&str>,
    hue_column: Option<&str>,
    hue_name: Option<&str>,
    retained_rows: Option<&[usize]>,
    plot: &str,
) -> Result<ObservedCollectives, Error> {
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
    finish_observed_collectives(
        groups,
        group_name,
        hues,
        hue_name,
        row_indices.len(),
        |index| data.source_row(row_indices[index]),
        plot,
    )
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the decoded source arrays belong to this reduction and are not retained by its callers"
)]
pub(crate) fn finish_observed_collectives(
    groups: Option<Vec<String>>,
    group_name: Option<&str>,
    hues: Option<Vec<String>>,
    hue_name: Option<&str>,
    row_count: usize,
    diagnostic_row: impl Fn(usize) -> usize,
    plot: &str,
) -> Result<ObservedCollectives, Error> {
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
                diagnostic_row(index)
            )));
        }
    }

    let rows = crate::data::source::collect_source((0..row_count).map(|index| {
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
    data: PlotData<'_, '_>,
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

pub(crate) fn collective_channels(
    data: PlotData<'_, '_>,
    group_column: Option<&str>,
    hue_column: Option<&str>,
    cells: Option<&[String]>,
    plot: &str,
) -> Result<CollectiveChannels, Error> {
    let groups = group_column
        .map(|column| strict_strings(data, column, plot))
        .transpose()?;
    let hues = hue_column
        .map(|column| strict_strings(data, column, plot))
        .transpose()?;
    let row_count = data.height();
    finish_collective_channels(
        groups,
        hues,
        cells,
        row_count,
        |row| data.source_row(row),
        plot,
    )
}

pub(crate) fn finish_collective_channels(
    groups: Option<Vec<String>>,
    hues: Option<Vec<String>>,
    cells: Option<&[String]>,
    row_count: usize,
    diagnostic_row: impl Fn(usize) -> usize,
    plot: &str,
) -> Result<CollectiveChannels, Error> {
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
                        diagnostic_row(row)
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

pub(crate) fn source_collective_channels(
    source: &crate::builder::PlotDescription,
    cells: Option<&[String]>,
    context: &str,
) -> Result<CollectiveChannels, Error> {
    let strings = |key: &str| {
        let column = source
            .data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))?;
        let DataColumn::Category(values) = column else {
            return Err(Error::Data(format!(
                "{context}: value {:?} must be categorical",
                column.name()
            )));
        };
        strict_source_strings(values, None, context)
    };
    let groups = source.group_col().map(strings).transpose()?;
    let hues = source.hue_col().map(strings).transpose()?;
    finish_collective_channels(groups, hues, cells, source.row_count(), |row| row, context)
}

pub(crate) fn observed_source_collectives(
    source: &crate::builder::PlotDescription,
    context: &str,
) -> Result<ObservedCollectives, Error> {
    let rows = source.source_row_map();
    let mut row_indices = Vec::new();
    self::source::reserve_source(&mut row_indices, source.row_count())?;
    row_indices.extend(0..source.row_count());
    let strings = |key: &str| {
        let column = source
            .data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))?;
        let DataColumn::Category(values) = column else {
            return Err(Error::Data(format!(
                "{context}: value {:?} must be categorical",
                column.name()
            )));
        };
        strict_source_strings(values, rows, context)
    };
    let groups = source.group_col().map(strings).transpose()?;
    let hues = source.hue_col().map(strings).transpose()?;
    finish_observed_collectives(
        groups,
        source.group_name(),
        hues,
        source.hue_name(),
        row_indices.len(),
        |slot| {
            let slot = row_indices[slot];
            rows.and_then(|rows| rows.as_slice().get(slot))
                .copied()
                .unwrap_or(slot)
        },
        context,
    )
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

pub(crate) fn strict_strings(
    data: PlotData<'_, '_>,
    column: &str,
    plot: &str,
) -> Result<Vec<String>, Error> {
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

pub(crate) fn strict_normalized_strings(
    data: PlotData<'_, '_>,
    column: &str,
    plot: &str,
    role: &str,
) -> Result<Vec<String>, Error> {
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
    fn typed_source_reads_keep_full_read_precedence_and_diagnostic_rows() {
        let values = CategoryValues::nullable("category", [Some("   "), None]).unwrap();
        let rows = crate::SourceRows::new([70, 901]).unwrap();
        let error = strict_source_normalized_strings(&values, Some(&rows), "source", "category")
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "data error: source: value \"category\" contains null at source row 901"
        );
        let values = CategoryValues::new("category", ["  ", "valid"]).unwrap();
        let error = strict_source_normalized_strings(&values, Some(&rows), "source", "category")
            .unwrap_err();
        assert_eq!(error.to_string(), "data error: source: category label is empty after whitespace normalization at source row 70");
        let numbers = NumericValues::new("value", [3.0, f64::INFINITY]);
        assert_eq!(
            strict_numeric_at(&numbers, 0, 901, "source")
                .unwrap()
                .to_bits(),
            3.0_f64.to_bits()
        );
        assert_eq!(
            strict_source_f64(&numbers, Some(&rows), "source")
                .unwrap_err()
                .to_string(),
            "data error: source: value \"value\" contains non-finite value inf at source row 901"
        );
    }

    #[test]
    fn source_configuration_keeps_original_bar_and_trajectory_ordinal_errors() {
        let rows = crate::SourceRows::new([70, 901]).unwrap();
        let bar = Plot::bar(
            CategoryValues::new("category", ["a", "b"]).unwrap(),
            crate::BarInput::estimates(NumericValues::nullable("value", [Some(1.0), None])),
        )
        .source_rows(rows.clone());
        assert_eq!(
            bar.description()
                .validate_configuration(bar.kind())
                .unwrap_err()
                .to_string(),
            "data error: bar: value \"value\" contains null at source row 1"
        );
        let trajectory = Plot::trajectory(
            CategoryValues::new("condition", ["valid", "   "]).unwrap(),
            NumericValues::new("value", [1.0, 2.0]),
            CategoryValues::new("group", ["g", "g"]).unwrap(),
        )
        .source_rows(rows);
        assert_eq!(trajectory.description().validate_configuration(trajectory.kind()).unwrap_err().to_string(), "data error: trajectory: condition label is empty after whitespace normalization at source row 1");
    }

    #[test]
    fn observed_collectives_order_groups_then_hues_within_each_group() {
        let plot = Plot::line(
            crate::NumericValues::new("x", [0.0, 1.0, 2.0, 3.0, 4.0]),
            crate::NumericValues::new("y", [0.0, 1.0, 2.0, 3.0, 4.0]),
        )
        .group(crate::CategoryValues::new("group", ["g1", "g2", "g2", "g1", "g1"]).unwrap())
        .hue(crate::CategoryValues::new("hue", ["h2", "h1", "h2", "h1", "h2"]).unwrap())
        .erase();

        let channels = with_source_data_for_test(
            &plot,
            &crate::font::canonical::OperationContext::new(),
            crate::orchestrator::registration::RowSelection::Full,
            |data| {
                observed_collectives(
                    data,
                    plot.description().group_col(),
                    plot.description().group_name(),
                    plot.description().hue_col(),
                    plot.description().hue_name(),
                    None,
                    "test",
                )
                .unwrap()
            },
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
