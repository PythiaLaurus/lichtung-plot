//! Admission for data-access temporaries activated by font-independent axis sources.

use super::PlotData;
use crate::Error;

/// Callers outside axis-source generation (public statistics and existing
/// configuration, mark and census paths) can omit this gate. Axis activation
/// always threads the operation-bearing PlotData through its source helpers.
pub(crate) trait SourceAdmission: Copy {
    fn source_visits(self, count: usize) -> Result<(), Error>;
    fn source_reserve<T>(self, values: &mut Vec<T>, count: usize) -> Result<(), Error>;
}

impl SourceAdmission for Option<PlotData<'_>> {
    fn source_visits(self, count: usize) -> Result<(), Error> {
        self.map_or(Ok(()), |data| data.claim_source_visits(count))
    }

    fn source_reserve<T>(self, values: &mut Vec<T>, count: usize) -> Result<(), Error> {
        if let Some(data) = self {
            data.reserve_source(values, count)
        } else {
            values
                .try_reserve_exact(count)
                .map_err(|error| Error::Render(format!("source reservation: {error}")))
        }
    }
}

pub(crate) fn sort_numeric_source(
    values: &mut [f64],
    data: Option<PlotData<'_>>,
) -> Result<(), Error> {
    sort_source_by(values, data, f64::total_cmp)
}

pub(crate) fn sort_source_by<T>(
    values: &mut [T],
    data: Option<PlotData<'_>>,
    compare: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Result<(), Error> {
    // In-place heapsort can stop at the first refused comparison without an
    // allocator-owned scratch buffer or an infallible comparator fallback.
    for root in (0..values.len() / 2).rev() {
        sift(values, root, data, &compare)?;
    }
    for end in (1..values.len()).rev() {
        data.source_visits(1)?;
        values.swap(0, end);
        sift(&mut values[..end], 0, data, &compare)?;
    }
    Ok(())
}

fn sift<T>(
    values: &mut [T],
    mut root: usize,
    data: Option<PlotData<'_>>,
    compare: &impl Fn(&T, &T) -> std::cmp::Ordering,
) -> Result<(), Error> {
    while root < values.len() / 2 {
        data.source_visits(1)?;
        let mut child = 2 * root + 1;
        if child + 1 < values.len() {
            data.source_visits(1)?;
            if compare(&values[child], &values[child + 1]).is_lt() {
                child += 1;
            }
        }
        data.source_visits(1)?;
        if !compare(&values[root], &values[child]).is_lt() {
            break;
        }
        values.swap(root, child);
        root = child;
    }
    Ok(())
}

impl PlotData<'_> {
    pub(crate) fn reserve_source_string(
        self,
        value: &mut String,
        additional: usize,
    ) -> Result<(), Error> {
        let needed = value
            .len()
            .checked_add(additional)
            .ok_or_else(|| Error::Render("axis source string length overflow".into()))?;
        self.claim_source_visits(additional)?;
        self.claim_source_bytes(needed.saturating_sub(value.capacity()))?;
        value
            .try_reserve_exact(additional)
            .map_err(|error| Error::Render(format!("axis source string reservation: {error}")))
    }

    pub(crate) fn claim_source_bytes(self, bytes: usize) -> Result<(), Error> {
        if let Some(resources) = self.axis_resources {
            resources
                .claim_bytes(
                    u64::try_from(bytes)
                        .map_err(|_| Error::Render("axis source bytes overflow".into()))?,
                )
                .map_err(|error| {
                    Error::Render(format!("axis source storage admission: {error}"))
                })?;
        }
        Ok(())
    }

    pub(crate) fn source_arc<T>(self, value: T) -> Result<std::sync::Arc<T>, Error> {
        // Stable std has no fallible Arc constructor. Admit its allocation layout
        // before construction; allocator exhaustion still aborts in std.
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .map_err(|_| Error::Render("axis source Arc layout overflow".into()))?
            .0
            .pad_to_align();
        self.claim_source_visits(1)?;
        self.claim_source_bytes(layout.size())?;
        Ok(std::sync::Arc::new(value))
    }
    pub(crate) fn copy_source_string(self, source: &str) -> Result<String, Error> {
        self.claim_source_visits(source.len())?;
        if let Some(resources) = self.axis_resources {
            let bytes = u64::try_from(source.len())
                .map_err(|_| Error::Render("axis source string byte count overflow".into()))?;
            resources.claim_bytes(bytes).map_err(|error| {
                Error::Render(format!("axis source storage admission: {error}"))
            })?;
        }
        let mut value = String::new();
        value
            .try_reserve_exact(source.len())
            .map_err(|error| Error::Render(format!("axis source string reservation: {error}")))?;
        value.push_str(source);
        Ok(value)
    }

    pub(crate) fn source_levels(self, values: &[String]) -> Result<Vec<String>, Error> {
        let mut levels = Vec::new();
        self.reserve_source(&mut levels, values.len())?;
        for value in values {
            self.claim_source_visits(levels.len())?;
            if !levels.contains(value) {
                levels.push(self.copy_source_string(value)?);
            }
        }
        Ok(levels)
    }

    pub(crate) fn copy_source_pair(
        self,
        pair: &super::CollectivePair,
    ) -> Result<super::CollectivePair, Error> {
        Ok(super::CollectivePair {
            group: pair
                .group
                .as_deref()
                .map(|s| self.copy_source_string(s))
                .transpose()?,
            hue: pair
                .hue
                .as_deref()
                .map(|s| self.copy_source_string(s))
                .transpose()?,
        })
    }

    pub(crate) fn claim_source_visits(self, count: usize) -> Result<(), Error> {
        if let Some(resources) = self.axis_resources {
            resources
                .claim_visits(count as u128)
                .map_err(|error| Error::Render(format!("axis source work admission: {error:?}")))?;
        }
        Ok(())
    }

    pub(crate) fn reserve_source<T>(
        self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), Error> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(|| Error::Render("axis source length overflow".into()))?;
        let bytes = needed
            .saturating_sub(values.capacity())
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| Error::Render("axis source byte count overflow".into()))?;
        self.claim_source_visits(additional)?;
        if let Some(resources) = self.axis_resources {
            resources.claim_bytes(bytes).map_err(|error| {
                Error::Render(format!("axis source storage admission: {error}"))
            })?;
        }
        values
            .try_reserve_exact(additional)
            .map_err(|error| Error::Render(format!("axis source reservation: {error}")))
    }

    pub(crate) fn collect_source<T>(
        self,
        values: impl ExactSizeIterator<Item = Result<T, Error>>,
    ) -> Result<Vec<T>, Error> {
        let mut retained = Vec::new();
        self.reserve_source(&mut retained, values.len())?;
        for value in values {
            retained.push(value?);
        }
        Ok(retained)
    }

    pub(crate) fn normalize_source_string(self, source: &str) -> Result<String, Error> {
        // Whitespace collapse cannot grow the UTF-8 source. Reuse the admitted
        // copy's capacity rather than allocating a temporary word vector.
        let mut normalized = self.copy_source_string(source)?;
        normalized.clear();
        self.claim_source_visits(source.len())?;
        for word in source.split_whitespace() {
            if !normalized.is_empty() {
                normalized.push(' ');
            }
            normalized.push_str(word);
        }
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::canonical::OperationGenerationResources;

    #[test]
    fn arc_retention_admits_header_and_value_without_recopying_vector_storage() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        for (visits, bytes) in [(0, u64::MAX), (u128::MAX, 0)] {
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            assert!(plot
                .data()
                .with_axis_resources(&resources)
                .source_arc(Vec::<u64>::new())
                .is_err());
        }
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        let mut values = Vec::new();
        data.reserve_source(&mut values, 4).unwrap();
        values.extend([1_u64, 2, 3, 4]);
        let pointer = values.as_ptr();
        let before = (
            resources.byte_usage().unwrap(),
            resources.usage().unwrap().1,
        );
        let retained = data.source_arc(values).unwrap();
        assert_eq!(retained.as_ptr(), pointer);
        let after = (
            resources.byte_usage().unwrap(),
            resources.usage().unwrap().1,
        );
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1 + 1);
        let bytes = after.0 - before.0;
        for bytes in [bytes - 1, bytes] {
            let resources = OperationGenerationResources::with_test_limits(1, bytes);
            let result = plot
                .data()
                .with_axis_resources(&resources)
                .source_arc(Vec::<u64>::new());
            assert_eq!(result.is_ok(), bytes == after.0 - before.0);
        }
    }

    #[test]
    fn numeric_sort_refuses_before_mutation_and_preserves_total_order() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let resources = OperationGenerationResources::with_test_limits(0, 0);
        let mut values = [3.0, -0.0, 0.0, -2.0, 3.0];
        let original = values.map(f64::to_bits);
        assert!(sort_numeric_source(
            &mut values,
            Some(plot.data().with_axis_resources(&resources))
        )
        .is_err());
        assert_eq!(values.map(f64::to_bits), original);
        for length in 0..40 {
            let mut values = (0..length)
                .map(|index| f64::from((index * 17) % 11 - 5))
                .collect::<Vec<_>>();
            let mut expected = values.clone();
            expected.sort_by(f64::total_cmp);
            sort_numeric_source(&mut values, None).unwrap();
            assert_eq!(values, expected);
        }
        sort_numeric_source(&mut values, None).unwrap();
        assert_eq!(
            values.map(f64::to_bits),
            [-2.0, -0.0, 0.0, 3.0, 3.0].map(f64::to_bits)
        );
    }

    #[test]
    fn kde_refuses_before_validation_storage_and_kernel_evaluation() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let evaluate = |samples: &[f64], resources: &OperationGenerationResources| {
            crate::stats::distribution::gaussian_kde_with_source(
                samples,
                &[0.0, 1.0],
                1.0,
                Some(plot.data().with_axis_resources(resources)),
            )
        };
        let resources = OperationGenerationResources::with_test_limits(0, 100);
        assert!(evaluate(&[f64::INFINITY], &resources)
            .unwrap_err()
            .to_string()
            .contains("work admission"));
        let resources = OperationGenerationResources::with_test_limits(100, 0);
        assert!(evaluate(&[1.0, 2.0, 3.0], &resources)
            .unwrap_err()
            .to_string()
            .contains("storage admission"));
        // Three sample validations, two grid validations, two output slots;
        // no kernel visit is permitted after that admission.
        let resources = OperationGenerationResources::with_test_limits(7, 16);
        assert!(evaluate(&[1.0, 2.0, 3.0], &resources)
            .unwrap_err()
            .to_string()
            .contains("work admission"));
        assert_eq!(resources.usage().unwrap().1, 7);
    }

    #[test]
    fn source_collection_and_normalization_refuse_before_producer_work() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let calls = std::cell::Cell::new(0);
        for (visits, bytes) in [(0, 1024), (100, 0)] {
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            let data = plot.data().with_axis_resources(&resources);
            let error = data
                .collect_source((0..3).map(|value| {
                    calls.set(calls.get() + 1);
                    Ok(value)
                }))
                .unwrap_err();
            assert!(matches!(error, Error::Render(_)));
            assert_eq!(calls.get(), 0);
            assert!(data.normalize_source_string(" a\t b ").is_err());
        }
        for source in ["", "   ", " a\t b ", "a\nb\r\nc", "  alpha  beta"] {
            assert_eq!(
                plot.data().normalize_source_string(source).unwrap(),
                crate::data::collapse_whitespace(source)
            );
        }
    }

    #[test]
    fn numeric_refusal_precedes_storage_and_value_validation() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [f64::INFINITY]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        for (visits, bytes, expected) in
            [(0, 1024, "work admission"), (100, 0, "storage admission")]
        {
            let resources = OperationGenerationResources::with_test_limits(visits, bytes);
            let data = plot.data().with_axis_resources(&resources);
            let error = crate::data::strict_finite_f64(data, crate::data::X, "source").unwrap_err();
            assert!(matches!(error, Error::Render(_)), "{error}");
            assert!(error.to_string().contains(expected), "{error}");
            let mut records: Vec<u64> = Vec::new();
            assert!(data.reserve_source(&mut records, 1).is_err());
            assert_eq!(records.capacity(), 0);
        }
    }

    #[test]
    fn source_reservation_checks_arithmetic_and_cumulative_operation_budget() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let resources = OperationGenerationResources::with_test_limits(100, 8);
        let data = plot.data().with_axis_resources(&resources);
        let mut values: Vec<u64> = Vec::new();
        assert!(data
            .reserve_source(&mut values, usize::MAX)
            .unwrap_err()
            .to_string()
            .contains("overflow"));
        assert_eq!(values.capacity(), 0);
        data.reserve_source(&mut values, 1).unwrap();
        let mut sibling: Vec<u64> = Vec::new();
        let cloned_resources = resources.clone();
        assert!(plot
            .data()
            .with_axis_resources(&cloned_resources)
            .reserve_source(&mut sibling, 1)
            .is_err());
        assert_eq!(sibling.capacity(), 0);
    }

    #[test]
    fn admitted_pair_sources_preserve_filtering_source_rows_and_errors() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::nullable("x", [Some(1.0), None, Some(f64::NAN), Some(4.0)]),
            crate::NumericValues::nullable("y", [Some(2.0), Some(3.0), Some(4.0), None]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let data = plot.data().with_axis_resources(&resources);
        assert_eq!(
            crate::data::aligned_finite_f64(data, crate::data::X, crate::data::Y, "source")
                .unwrap(),
            [(1.0, 2.0)],
        );
        assert_eq!(
            crate::data::aligned_finite_rows(data, crate::data::X, crate::data::Y, "source")
                .unwrap(),
            [(0, 1.0, 2.0)],
        );
        assert_eq!(
            crate::data::strict_finite_f64(data, crate::data::X, "source")
                .unwrap_err()
                .to_string(),
            crate::data::strict_finite_f64(plot.data(), crate::data::X, "source")
                .unwrap_err()
                .to_string(),
        );
        assert!(resources.usage().unwrap().1 > 0);
    }
}
