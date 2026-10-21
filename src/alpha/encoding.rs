//! Explicit source units, exact selectors and immutable prepared lookup.

use std::sync::Arc;

use super::{collect_input, numeric_key, validated_name, AlphaValues};
use crate::data::source::{collect_source, copy_source_string, sort_source_by, source_arc};
use crate::{Bounds, CategoryValues, Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AlphaEncoding {
    binding: Binding,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Binding {
    #[default]
    Theme,
    Observations(AlphaValues),
    Single(AlphaValues),
    Keyed(Arc<SelectorTable>, AlphaValues),
    Bins(Arc<BinTable>, AlphaValues),
}

#[derive(Debug, PartialEq, Eq)]
struct SelectorTable {
    fields: Vec<String>,
    rows: Vec<Vec<String>>,
    order: Vec<usize>,
}

#[derive(Debug, PartialEq, Eq)]
struct BinTable {
    facets: SelectorTable,
    bounds: Vec<[u64; 2]>,
    order: Vec<usize>,
}

/// The actual producer supplies its semantic class and complete required schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlphaUnitKind {
    Observation,
    Series,
    Aggregate,
    Bin,
    Single,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedAlpha {
    encoding: AlphaEncoding,
    unit: AlphaUnitKind,
    original_len: usize,
}

/// Retain the selected declaration member independently of its possibly zero byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedAlpha {
    pub(crate) index: usize,
    pub(crate) alpha: u8,
}

impl AlphaEncoding {
    pub(crate) const fn theme() -> Self {
        Self {
            binding: Binding::Theme,
        }
    }

    pub(crate) fn observations(values: AlphaValues) -> Self {
        Self {
            binding: Binding::Observations(values),
        }
    }

    pub(crate) fn single(values: AlphaValues) -> Result<Self> {
        if values.len() != 1 {
            return Err(Error::Data(
                "single alpha requires exactly one value".into(),
            ));
        }
        Ok(Self {
            binding: Binding::Single(values),
        })
    }

    pub(crate) fn keyed(
        keys: impl IntoIterator<Item = CategoryValues>,
        values: AlphaValues,
    ) -> Result<Self> {
        let table = SelectorTable::new(keys, values.len())?;
        if table.fields.is_empty() {
            return Err(Error::Data(
                "keyed alpha requires at least one selector field".into(),
            ));
        }
        if table
            .order
            .windows(2)
            .any(|pair| table.rows[pair[0]] == table.rows[pair[1]])
        {
            return Err(Error::Data(
                "keyed alpha contains duplicate selector tuples".into(),
            ));
        }
        Ok(Self {
            binding: Binding::Keyed(source_arc(table)?, values),
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "the consuming final constructor is staged privately before public cutover"
    )]
    pub(crate) fn bins(
        bounds: Bounds,
        facets: impl IntoIterator<Item = CategoryValues>,
        values: AlphaValues,
    ) -> Result<Self> {
        if bounds.lower().len() != values.len() || bounds.upper().len() != values.len() {
            return Err(Error::Data(
                "alpha bin bounds must align with the alpha values".into(),
            ));
        }
        let facets = SelectorTable::new(facets, values.len())?;
        let bounds = collect_source((0..values.len()).map(|index| {
            let low = bounds.lower().get(index).flatten();
            let high = bounds.upper().get(index).flatten();
            match (low, high) {
                (Some(low), Some(high)) if low.is_finite() && high.is_finite() && low < high => {
                    Ok([numeric_key(low), numeric_key(high)])
                }
                _ => Err(Error::Data(format!(
                    "alpha bin at index {index} requires finite nonmissing lower < upper bounds"
                ))),
            }
        }))?;
        let mut order = collect_source((0..values.len()).map(Ok))?;
        sort_source_by(&mut order, |a, b| {
            facets.rows[*a]
                .cmp(&facets.rows[*b])
                .then(bounds[*a].cmp(&bounds[*b]))
        });
        if order.windows(2).any(|pair| {
            facets.rows[pair[0]] == facets.rows[pair[1]] && bounds[pair[0]] == bounds[pair[1]]
        }) {
            return Err(Error::Data(
                "alpha bins contain duplicate exact interval/facet keys".into(),
            ));
        }
        Ok(Self {
            binding: Binding::Bins(
                source_arc(BinTable {
                    facets,
                    bounds,
                    order,
                })?,
                values,
            ),
        })
    }

    pub(crate) const fn is_theme(&self) -> bool {
        matches!(self.binding, Binding::Theme)
    }

    pub(crate) const fn values(&self) -> Option<&AlphaValues> {
        match &self.binding {
            Binding::Theme => None,
            Binding::Observations(values)
            | Binding::Single(values)
            | Binding::Keyed(_, values)
            | Binding::Bins(_, values) => Some(values),
        }
    }

    /// Validate even absent optional components' binding kind/schema before fonts.
    /// Actual producers resolve every consumed unit through the returned value.
    pub(crate) fn prepare(
        &self,
        unit: AlphaUnitKind,
        original_len: usize,
        required_fields: &[&str],
        facet_fields: &[&str],
    ) -> Result<PreparedAlpha> {
        match &self.binding {
            Binding::Theme | Binding::Single(_) => {}
            Binding::Observations(values) if unit == AlphaUnitKind::Observation => {
                if values.len() != original_len {
                    return Err(Error::Data(format!(
                        "observation alpha {:?} has length {}, expected all {original_len} original input slots",
                        values.name(), values.len(),
                    )));
                }
            }
            Binding::Keyed(table, _)
                if matches!(unit, AlphaUnitKind::Series | AlphaUnitKind::Aggregate) =>
            {
                table.validate_schema(required_fields, facet_fields)?;
            }
            Binding::Bins(table, _) if unit == AlphaUnitKind::Bin => {
                table.facets.validate_schema(&[], facet_fields)?;
            }
            _ => {
                return Err(Error::Data(format!(
                    "alpha binding does not support {unit:?} units"
                )))
            }
        }
        Ok(PreparedAlpha {
            encoding: self.clone(),
            unit,
            original_len,
        })
    }
}

impl SelectorTable {
    fn new(keys: impl IntoIterator<Item = CategoryValues>, len: usize) -> Result<Self> {
        let mut keys = collect_input(keys)?;
        sort_source_by(&mut keys, |a, b| a.name().cmp(b.name()));
        let fields = collect_source(keys.iter().map(|column| {
            if column.len() != len {
                return Err(Error::Data(format!(
                    "alpha selector {:?} has length {}, expected {len}",
                    column.name(),
                    column.len()
                )));
            }
            validated_name(copy_source_string(column.name())?)
        }))?;
        if fields.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(Error::Data(
                "alpha selector field names must be unique".into(),
            ));
        }
        let rows = collect_source((0..len).map(|index| {
            collect_source(keys.iter().map(|column| {
                let raw = column.get(index).flatten().ok_or_else(|| {
                    Error::Data(format!(
                        "alpha selector {:?} is missing at index {index}",
                        column.name()
                    ))
                })?;
                copy_source_string(raw)
            }))
        }))?;
        let mut order = collect_source((0..len).map(Ok))?;
        sort_source_by(&mut order, |a, b| rows[*a].cmp(&rows[*b]));
        Ok(Self {
            fields,
            rows,
            order,
        })
    }

    fn validate_schema(&self, required: &[&str], facets: &[&str]) -> Result<()> {
        for field in &self.fields {
            if !required.contains(&field.as_str()) && !facets.contains(&field.as_str()) {
                return Err(Error::Data(format!(
                    "alpha selector {field:?} is not an actual unit or owning facet field"
                )));
            }
        }
        for field in required {
            if !self.fields.iter().any(|name| name == field) {
                return Err(Error::Data(format!(
                    "alpha binding is missing required unit field {field:?}"
                )));
            }
        }
        Ok(())
    }

    fn tuple<'a>(&self, keys: &[(&str, &'a str)]) -> Result<Vec<&'a str>> {
        for (index, &(field, value)) in keys.iter().enumerate() {
            if keys[..index]
                .iter()
                .any(|&(other, raw)| other == field && raw != value)
            {
                return Err(Error::Data(format!(
                    "actual alpha unit has conflicting raw values for field {field:?}"
                )));
            }
        }
        collect_source(self.fields.iter().map(|field| {
            keys.iter()
                .find(|(name, _)| *name == field)
                .map(|(_, raw)| *raw)
                .ok_or_else(|| {
                    Error::Data(format!(
                        "actual alpha unit has no value for selector field {field:?}"
                    ))
                })
        }))
    }

    fn compare_row(&self, index: usize, tuple: &[&str]) -> std::cmp::Ordering {
        self.rows[index]
            .iter()
            .map(String::as_str)
            .cmp(tuple.iter().copied())
    }
}

impl PreparedAlpha {
    fn resolved(&self, index: usize) -> Result<Option<ResolvedAlpha>> {
        self.encoding
            .values()
            .map(|values| {
                values
                    .get(index)
                    .map(|alpha| ResolvedAlpha { index, alpha })
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "alpha source {:?} has no original slot {index}",
                            values.name()
                        ))
                    })
            })
            .transpose()
    }

    fn constant(&self) -> Option<Result<Option<ResolvedAlpha>>> {
        match &self.encoding.binding {
            Binding::Theme => Some(Ok(None)),
            Binding::Single(_) => Some(self.resolved(0)),
            _ => None,
        }
    }

    pub(crate) fn observation(
        &self,
        index: usize,
        source_row: usize,
    ) -> Result<Option<ResolvedAlpha>> {
        if self.unit != AlphaUnitKind::Observation {
            return Err(Error::Data(
                "alpha source requires an observation unit".into(),
            ));
        }
        if index >= self.original_len {
            return Err(Error::Data(format!(
                "alpha original slot {index} is outside length {} at diagnostic source row {source_row}",
                self.original_len,
            )));
        }
        self.constant()
            .unwrap_or_else(|| self.resolved(index))
            .map_err(|error| Error::Data(format!("{error} at diagnostic source row {source_row}")))
    }

    pub(crate) fn single(&self) -> Result<Option<ResolvedAlpha>> {
        if self.unit != AlphaUnitKind::Single {
            return Err(Error::Data("alpha source requires a single unit".into()));
        }
        self.resolved(0)
    }

    pub(crate) fn keyed(&self, keys: &[(&str, &str)]) -> Result<Option<ResolvedAlpha>> {
        if !matches!(self.unit, AlphaUnitKind::Series | AlphaUnitKind::Aggregate) {
            return Err(Error::Data(
                "alpha source requires a series or aggregate unit".into(),
            ));
        }
        if let Some(value) = self.constant() {
            return value;
        }
        let Binding::Keyed(table, _) = &self.encoding.binding else {
            unreachable!("prepared keyed binding")
        };
        let tuple = table.tuple(keys)?;
        let position = table
            .order
            .binary_search_by(|&index| table.compare_row(index, &tuple))
            .map_err(|_| {
                Error::Data(format!(
                    "alpha binding has no value for actual raw unit {keys:?}"
                ))
            })?;
        self.resolved(table.order[position])
    }

    pub(crate) fn bin(
        &self,
        bounds: (f64, f64),
        facets: &[(&str, &str)],
    ) -> Result<Option<ResolvedAlpha>> {
        if self.unit != AlphaUnitKind::Bin {
            return Err(Error::Data("alpha source requires a bin unit".into()));
        }
        if !bounds.0.is_finite() || !bounds.1.is_finite() || bounds.0 >= bounds.1 {
            return Err(Error::Data(
                "actual alpha bin must have finite lower < upper bounds".into(),
            ));
        }
        if let Some(value) = self.constant() {
            return value;
        }
        let Binding::Bins(table, _) = &self.encoding.binding else {
            unreachable!("prepared bin binding")
        };
        let tuple = table.facets.tuple(facets)?;
        let key = [numeric_key(bounds.0), numeric_key(bounds.1)];
        let position = table
            .order
            .binary_search_by(|&index| {
                table
                    .facets
                    .compare_row(index, &tuple)
                    .then(table.bounds[index].cmp(&key))
            })
            .map_err(|_| {
                Error::Data(format!(
                    "alpha binding has no value for exact actual bin {bounds:?} in {facets:?}"
                ))
            })?;
        self.resolved(table.order[position])
    }
}

#[cfg(test)]
mod tests;
