//! Fallible source storage and original-row operation identity.

use super::PlotData;
use crate::Error;

pub(crate) struct RowSourceContext<'a> {
    source: &'a crate::Plot,
    operation: &'a crate::font::canonical::OperationContext,
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

impl<'a> PlotData<'a> {
    pub(crate) fn bind_row_source(self) -> Result<RowSourceContext<'a>, Error> {
        if !self.is_full_plot() {
            return Err(Error::Render(
                "row topology census requires the original source".into(),
            ));
        }
        let operation = self
            .operation
            .ok_or_else(|| Error::Render("row topology census requires an operation".into()))?;
        Ok(RowSourceContext {
            source: self.plot,
            operation,
        })
    }

    pub(crate) fn with_row_source(mut self, source: Option<&'a RowSourceContext<'a>>) -> Self {
        self.row_topology = source;
        self
    }

    pub(crate) fn validate_row_source(self) -> Result<(), Error> {
        if let Some(source) = self.row_topology {
            if !std::ptr::eq(self.plot, source.source)
                || self
                    .operation
                    .is_none_or(|operation| !operation.same_operation(source.operation))
            {
                return Err(Error::Render(
                    "row source belongs to a different source or operation".into(),
                ));
            }
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
    values: impl ExactSizeIterator<Item = Result<T, Error>>,
) -> Result<Vec<T>, Error> {
    let mut retained = Vec::new();
    reserve_source(&mut retained, values.len())?;
    for value in values {
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
        let binding = source
            .data()
            .with_operation(&operation)
            .bind_row_source()
            .unwrap();
        let clone = operation.clone();
        let rows = [2, 0];
        let ids = [302, 300];
        let data = PlotData::with_rows(&source, &rows, &ids)
            .with_operation(&clone)
            .with_row_source(Some(&binding));
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
        assert!(data.with_operation(&other).validate_row_source().is_err());
        assert!(source
            .clone()
            .data()
            .with_operation(&operation)
            .with_row_source(Some(&binding))
            .validate_row_source()
            .is_err());
    }
}
