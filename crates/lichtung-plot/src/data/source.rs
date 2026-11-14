//! Fallible source storage and original-row operation identity.

use super::PlotData;
use crate::Error;

pub(crate) struct RowSourceContext<'src> {
    source: &'src crate::builder::PlotDescription,
    owner: crate::orchestrator::registration::PlotOccurrenceId,
    root_identity: crate::orchestrator::registration::RootIdentity,
}

impl<'src> RowSourceContext<'src> {
    #[cfg(test)]
    pub(super) const fn owner(&self) -> &crate::orchestrator::registration::PlotOccurrenceId {
        &self.owner
    }

    pub(crate) fn new(
        source: &'src crate::builder::PlotDescription,
        scope: &crate::orchestrator::registration::Scope,
        operation: &crate::font::canonical::OperationContext,
    ) -> Self {
        Self {
            source,
            owner: scope.id.clone(),
            root_identity: operation.root_identity(),
        }
    }

    pub(crate) fn check(
        &self,
        source: &crate::builder::PlotDescription,
        scope: &crate::orchestrator::registration::Scope,
        operation: &crate::font::canonical::OperationContext,
    ) -> Result<(), Error> {
        if !std::ptr::eq(self.source, source)
            || self.owner != scope.id
            || self.root_identity != operation.root_identity()
            || scope.id.root != self.root_identity
        {
            return Err(Error::Render(
                "row source belongs to a different source or operation".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn sort_numeric_source(values: &mut [f64]) {
    sort_source_by(values, f64::total_cmp);
}

pub(crate) fn sort_source_by<T>(values: &mut [T], compare: impl Fn(&T, &T) -> std::cmp::Ordering) {
    for root in (0..values.len() / 2).rev() {
        sift(values, root, &compare);
    }
    for end in (1..values.len()).rev() {
        values.swap(0, end);
        sift(&mut values[..end], 0, &compare);
    }
}

fn sift<T>(values: &mut [T], mut root: usize, compare: &impl Fn(&T, &T) -> std::cmp::Ordering) {
    while root < values.len() / 2 {
        let mut child = 2 * root + 1;
        if child + 1 < values.len() && compare(&values[child], &values[child + 1]).is_lt() {
            child += 1;
        }
        if !compare(&values[root], &values[child]).is_lt() {
            break;
        }
        values.swap(root, child);
        root = child;
    }
}

impl PlotData<'_, '_> {
    pub(crate) fn validate_row_source(self) -> Result<(), Error> {
        let source = self.row_source;
        if !std::ptr::eq(self.source, source.source)
            || self.operation.root_identity() != source.root_identity
        {
            return Err(Error::Render(
                "row source belongs to a different source or operation".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn check_numeric_binding(
        self,
        binding: crate::numeric::source::NumericBinding<'_>,
    ) -> Result<(), Error> {
        self.validate_row_source()?;
        if !self.operation.same_operation(binding.operation)
            || binding
                .plot_id
                .is_some_and(|id| &self.row_source.owner != id)
        {
            return Err(Error::Render(
                "row source belongs to a different source or operation".into(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn source_position<T>(values: &[T], matches: impl Fn(&T) -> bool) -> Option<usize> {
    values.iter().position(matches)
}

pub(crate) fn reserve_source_string(value: &mut String, additional: usize) -> Result<(), Error> {
    value
        .len()
        .checked_add(additional)
        .ok_or_else(|| Error::Render("source string length overflow".into()))?;
    value
        .try_reserve_exact(additional)
        .map_err(|error| Error::Render(format!("source string reservation: {error}")))
}

pub(crate) fn source_arc<T>(value: T) -> Result<std::sync::Arc<T>, Error> {
    // Stable Arc construction is infallible, but its represented layout is checked.
    std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Error::Render("source Arc layout overflow".into()))?;
    Ok(std::sync::Arc::new(value))
}

pub(crate) fn copy_source_string(source: &str) -> Result<String, Error> {
    let mut value = String::new();
    reserve_source_string(&mut value, source.len())?;
    value.push_str(source);
    Ok(value)
}

pub(crate) fn copy_font_stack(
    source: &crate::font::canonical::FontStack,
) -> Result<crate::font::canonical::FontStack, Error> {
    source
        .try_copy()
        .map_err(|error| Error::Render(format!("font stack retention: {error}")))
}

pub(crate) fn source_levels(values: &[String]) -> Result<Vec<String>, Error> {
    let mut levels = Vec::new();
    for value in values {
        if !levels.contains(value) {
            reserve_source(&mut levels, 1)?;
            levels.push(copy_source_string(value)?);
        }
    }
    Ok(levels)
}

pub(crate) fn copy_source_pair(
    pair: &super::CollectivePair,
) -> Result<super::CollectivePair, Error> {
    Ok(super::CollectivePair {
        group: pair.group.as_deref().map(copy_source_string).transpose()?,
        hue: pair.hue.as_deref().map(copy_source_string).transpose()?,
    })
}

pub(crate) fn reserve_source<T>(values: &mut Vec<T>, additional: usize) -> Result<(), Error> {
    values
        .len()
        .checked_add(additional)
        .and_then(|n| n.checked_mul(std::mem::size_of::<T>()))
        .ok_or_else(|| Error::Render("source storage length overflow".into()))?;
    values
        .try_reserve_exact(additional)
        .map_err(|error| Error::Render(format!("source reservation: {error}")))
}

pub(crate) fn collect_source<T>(
    values: impl Iterator<Item = Result<T, Error>>,
) -> Result<Vec<T>, Error> {
    let mut retained = Vec::new();
    let (minimum, maximum) = values.size_hint();
    reserve_source(&mut retained, maximum.unwrap_or(minimum))?;
    for value in values {
        if retained.len() == retained.capacity() {
            let additional = retained.len().max(1);
            reserve_source(&mut retained, additional)?;
        }
        retained.push(value?);
    }
    Ok(retained)
}

pub(crate) fn normalize_source_string(source: &str) -> Result<String, Error> {
    let mut normalized = copy_source_string(source)?;
    normalized.clear();
    for word in source.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.push_str(word);
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationContext;
    use crate::orchestrator::registration::{RowSelection, Scope};

    #[test]
    fn checked_full_rows_and_numeric_bindings_keep_occurrence_and_owner_identity() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [10.0, 20.0]),
            crate::NumericValues::new("y", [2.0, 1.0]),
        )
        .source_rows(crate::SourceRows::new([9001, 70]).unwrap())
        .erase();
        let operation = OperationContext::new();
        let first = Scope::new(
            source.description(),
            &operation,
            0,
            crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)
                .unwrap(),
            RowSelection::Full,
            None,
            None,
        )
        .unwrap();
        let second = Scope::new(
            source.description(),
            &operation,
            1,
            crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::CanvasCell, 1)
                .unwrap(),
            RowSelection::Full,
            None,
            Some("panel-1".into()),
        )
        .unwrap();
        let rows = RowSourceContext::new(source.description(), &first, &operation);
        let other_rows = RowSourceContext::new(source.description(), &second, &operation);
        let data = PlotData::checked(source.description(), &first, &rows, &operation).unwrap();
        assert!(data.is_full_plot());
        assert_eq!(
            (data.height(), data.row_index(1), data.source_row(1)),
            (2, 1, 70)
        );
        assert!(
            data.rows.is_none(),
            "Full does not allocate selected input slots"
        );
        let numeric = first
            .numeric_binding(source.description(), &rows, &operation)
            .unwrap();
        numeric.check_data(data).unwrap();
        assert!(PlotData::checked(source.description(), &second, &rows, &operation).is_err());
        assert!(second
            .numeric_binding(source.description(), &other_rows, &operation)
            .unwrap()
            .check_data(data)
            .is_err());
        assert!(
            PlotData::checked(source.clone().description(), &first, &rows, &operation).is_err()
        );
        let foreign = OperationContext::new();
        assert!(PlotData::checked(source.description(), &first, &rows, &foreign).is_err());
        let ordinary = second
            .numeric_binding(source.description(), &other_rows, &operation)
            .unwrap();
        let axis = second
            .axis_numeric_binding(source.description(), &other_rows, &operation)
            .unwrap();
        assert!(ordinary.owner.scope.is_none());
        assert_eq!(axis.owner.scope.as_deref(), Some("panel-1"));
    }

    #[test]
    fn native_columns_outlive_selected_views_without_retaining_operation_resources() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [10.0, 20.0, 30.0]),
            crate::NumericValues::new("y", [3.0, 2.0, 1.0]),
        )
        .erase();
        let (column, binding, alive) = {
            let operation = OperationContext::new();
            let scope = Scope::new(
                source.description(),
                &operation,
                0,
                crate::layout::ComponentAddress::root()
                    .child(crate::layout::AddressRole::PlotBody, 0)
                    .unwrap(),
                RowSelection::Selected {
                    input_slots: vec![2, 0],
                    diagnostic_rows: vec![9001, 700],
                },
                None,
                None,
            )
            .unwrap();
            let binding = RowSourceContext::new(source.description(), &scope, &operation);
            let selected =
                PlotData::checked(source.description(), &scope, &binding, &operation).unwrap();
            selected.validate_row_source().unwrap();
            assert_eq!((selected.row_index(0), selected.source_row(0)), (2, 9001));
            (
                selected.column(crate::data::X).unwrap(),
                binding,
                operation.liveness_probe(),
            )
        };
        assert!(!alive());
        let crate::data::DataColumn::Numeric(values) = column else {
            panic!("native numeric source")
        };
        assert_eq!(
            values.get(2).flatten().unwrap().to_bits(),
            30.0_f64.to_bits()
        );
        let foreign = OperationContext::new();
        let scope = Scope::new(
            source.description(),
            &foreign,
            0,
            crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)
                .unwrap(),
            RowSelection::Full,
            None,
            None,
        )
        .unwrap();
        assert!(PlotData::checked(source.description(), &scope, &binding, &foreign).is_err());
        assert!(!alive());
    }

    #[test]
    fn row_source_is_bound_to_its_source_and_shared_operation() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
        )
        .with_alpha(
            crate::alpha::AlphaTarget::Point,
            crate::alpha::AlphaEncoding::observations(
                crate::alpha::AlphaValues::new("confidence", [0, 127, 230]).unwrap(),
            ),
        )
        .erase();
        let operation = OperationContext::new();
        let scope = Scope::new(
            source.description(),
            &operation,
            0,
            crate::layout::ComponentAddress::root()
                .child(crate::layout::AddressRole::PlotBody, 0)
                .unwrap(),
            RowSelection::Selected {
                input_slots: vec![2, 0],
                diagnostic_rows: vec![302, 300],
            },
            None,
            None,
        )
        .unwrap();
        let binding = RowSourceContext::new(source.description(), &scope, &operation);
        let clone = operation.clone();
        let data = PlotData::checked(source.description(), &scope, &binding, &clone).unwrap();
        data.validate_row_source().unwrap();
        assert_eq!(
            (
                data.row_index(0),
                data.source_row(0),
                data.row_index(1),
                data.source_row(1)
            ),
            (2, 302, 0, 300)
        );
        let alpha = source
            .description()
            .prepare_alpha(
                crate::alpha::AlphaTarget::Point,
                crate::alpha::AlphaUnitKind::Observation,
                &[],
            )
            .unwrap();
        assert_eq!(
            data.observation_alpha(&alpha, 0).unwrap(),
            Some(crate::alpha::ResolvedAlpha {
                index: 2,
                alpha: 230
            })
        );
        assert_eq!(
            data.observation_alpha(&alpha, 1).unwrap(),
            Some(crate::alpha::ResolvedAlpha { index: 0, alpha: 0 })
        );
        assert!(data.observation_alpha(&alpha, 2).is_err());
        let invalid_slot = alpha.observation(3, 999).unwrap_err();
        assert!(invalid_slot.to_string().contains("original slot 3"));
        assert!(invalid_slot.to_string().contains("source row 999"));
        let other = OperationContext::new();
        assert!(PlotData {
            operation: &other,
            ..data
        }
        .validate_row_source()
        .is_err());
        assert!(
            PlotData::checked(source.clone().description(), &scope, &binding, &operation).is_err()
        );
    }
}
