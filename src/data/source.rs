//! Admission for data-access temporaries activated by font-independent axis sources.

use super::PlotData;
use crate::Error;

/// The original row slots, admitted before a facet partitions the source.
pub(crate) struct RowTopologyAdmission<'a> {
    source: &'a crate::Plot,
    resources: &'a crate::font::canonical::OperationGenerationResources,
}

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
    // PF4 admits sort input records once, not comparisons or storage moves.
    data.source_visits(values.len())?;
    for root in (0..values.len() / 2).rev() {
        sift(values, root, &compare);
    }
    for end in (1..values.len()).rev() {
        values.swap(0, end);
        sift(&mut values[..end], 0, &compare);
    }
    Ok(())
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
    pub(crate) fn admit_row_topology(self) -> Result<RowTopologyAdmission<'a>, Error> {
        if !self.is_full_plot() {
            return Err(Error::Render(
                "row topology census requires the original source".into(),
            ));
        }
        let resources = self.axis_resources.ok_or_else(|| {
            Error::Render("row topology census requires operation resources".into())
        })?;
        self.claim_topology_slots(self.height())?;
        Ok(RowTopologyAdmission {
            source: self.plot,
            resources,
        })
    }

    pub(crate) fn with_row_topology(
        mut self,
        admission: Option<&'a RowTopologyAdmission<'a>>,
    ) -> Self {
        self.row_topology = admission;
        self
    }

    pub(crate) fn claim_mark_rows(self) -> Result<(), Error> {
        if let Some(admission) = self.row_topology {
            if !std::ptr::eq(self.plot, admission.source)
                || self
                    .axis_resources
                    .is_none_or(|resources| !resources.same_operation(admission.resources))
            {
                return Err(Error::Render(
                    "row topology admission belongs to a different source or operation".into(),
                ));
            }
            return Ok(());
        }
        self.claim_topology_slots(self.height())
    }

    pub(crate) fn source_position<T>(
        self,
        values: &[T],
        matches: impl Fn(&T) -> bool,
    ) -> Result<Option<usize>, Error> {
        for (index, value) in values.iter().enumerate() {
            self.claim_source_visits(1)?;
            if matches(value) {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    pub(crate) fn claim_topology_slots(self, count: usize) -> Result<(), Error> {
        if let Some(resources) = self.axis_resources {
            resources
                .claim_topology_slots(count as u128)
                .map_err(|error| Error::Render(format!("source topology admission: {error:?}")))?;
        }
        Ok(())
    }

    /// Admit an infallible standard-library clone before it copies any storage.
    pub(crate) fn admit_clone_slice<T>(self, values: &[T]) -> Result<(), Error> {
        self.claim_source_records(values.len())?;
        self.claim_source_bytes(std::mem::size_of_val(values))
    }

    pub(crate) fn admit_clone_str(self, value: &str) -> Result<(), Error> {
        self.claim_source_records(1)?;
        self.claim_source_bytes(value.len())
    }

    pub(crate) fn reserve_source_string(
        self,
        value: &mut String,
        additional: usize,
    ) -> Result<(), Error> {
        let needed = value
            .len()
            .checked_add(additional)
            .ok_or_else(|| Error::Render("axis source string length overflow".into()))?;
        if value.capacity() == 0 && needed > 0 {
            self.claim_source_records(1)?;
        }
        if needed > value.capacity() {
            self.claim_source_bytes(needed)?;
        }
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
        self.claim_source_records(1)?;
        self.claim_source_bytes(layout.size())?;
        Ok(std::sync::Arc::new(value))
    }
    pub(crate) fn copy_source_string(self, source: &str) -> Result<String, Error> {
        self.admit_clone_str(source)?;
        let mut value = String::new();
        value
            .try_reserve_exact(source.len())
            .map_err(|error| Error::Render(format!("axis source string reservation: {error}")))?;
        value.push_str(source);
        Ok(value)
    }

    pub(crate) fn source_levels(self, values: &[String]) -> Result<Vec<String>, Error> {
        let mut levels = Vec::new();
        for value in values {
            self.claim_source_visits(1)?;
            let mut present = false;
            for level in &levels {
                self.claim_source_visits(1)?;
                if level == value {
                    present = true;
                    break;
                }
            }
            if !present {
                self.reserve_source(&mut levels, 1)?;
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

    pub(crate) fn claim_source_records(self, count: usize) -> Result<(), Error> {
        if let Some(resources) = self.axis_resources {
            resources.claim_slots(count as u128).map_err(|error| {
                Error::Render(format!("axis source record admission: {error:?}"))
            })?;
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
        let bytes = (if needed > values.capacity() {
            needed
        } else {
            0
        })
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Render("axis source byte count overflow".into()))?;
        self.claim_source_records(additional)?;
        if let Some(resources) = self.axis_resources {
            resources.claim_bytes(bytes).map_err(|error| {
                Error::Render(format!("axis source storage admission: {error}"))
            })?;
        }
        values
            .try_reserve_exact(additional)
            .map_err(|error| Error::Render(format!("axis source reservation: {error}")))
    }

    /// Storage admission only; semantic producers admit their work before doing it.
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
        self.claim_source_visits(source.chars().count())?;
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
        for (records, bytes) in [(0, u64::MAX), (1, 0)] {
            let resources = OperationGenerationResources::with_test_limits(0, bytes);
            resources.claim_slots(1_000_000 - records).unwrap();
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
            resources.usage().unwrap().0,
        );
        let retained = data.source_arc(values).unwrap();
        assert_eq!(retained.as_ptr(), pointer);
        let after = (
            resources.byte_usage().unwrap(),
            resources.usage().unwrap().0,
        );
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1 + 1);
        assert_eq!(resources.usage().unwrap().1, 0);
        let bytes = after.0 - before.0;
        for bytes in [bytes - 1, bytes] {
            let resources = OperationGenerationResources::with_test_limits(0, bytes);
            let result = plot
                .data()
                .with_axis_resources(&resources)
                .source_arc(Vec::<u64>::new());
            assert_eq!(result.is_ok(), bytes == after.0 - before.0);
        }
    }

    #[test]
    fn source_storage_uses_records_and_bytes_without_processing_visits() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        for allowance in [2, 3, 4] {
            let resources = OperationGenerationResources::with_test_limits(0, 24);
            resources.claim_slots(1_000_000 - allowance).unwrap();
            let data = plot.data().with_axis_resources(&resources);
            let mut values = Vec::<u64>::new();
            assert_eq!(data.reserve_source(&mut values, 3).is_ok(), allowance >= 3);
            assert_eq!(resources.usage().unwrap().1, 0);
            if allowance < 3 {
                assert_eq!(values.capacity(), 0);
                assert_eq!(resources.byte_usage().unwrap(), 0);
            } else {
                assert!(values.capacity() >= 3);
                assert_eq!(resources.byte_usage().unwrap(), 24);
            }
        }
        let resources = OperationGenerationResources::with_test_limits(0, u64::MAX);
        let data = plot.data().with_axis_resources(&resources);
        let text = "A".repeat(10_000);
        assert_eq!(data.copy_source_string(&text).unwrap(), text);
        data.admit_clone_slice(&[1_u32, 2, 3]).unwrap();
        assert_eq!(resources.usage().unwrap(), (4, 0));
        assert_eq!(resources.byte_usage().unwrap(), 10_012);
        resources.claim_slots(999_996).unwrap();
        let before = resources.byte_usage().unwrap();
        assert!(data.copy_source_string("A").is_err());
        assert_eq!(resources.byte_usage().unwrap(), before);
    }

    #[test]
    fn source_levels_charge_only_examined_entries_and_sort_charges_input_once() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let values = ["a", "b", "c", "a"].map(str::to_owned);
        // Four source records plus 0 + 1 + 2 + 1 examined existing levels.
        for visits in [7, 8, 9] {
            let resources = OperationGenerationResources::with_test_limits(visits, 1024);
            let result = plot
                .data()
                .with_axis_resources(&resources)
                .source_levels(&values);
            assert_eq!(result.is_ok(), visits >= 8);
            if let Ok(levels) = result {
                assert_eq!(levels, ["a", "b", "c"]);
                assert_eq!(resources.usage().unwrap().1, 8);
            }
        }
        for visits in [3, 4, 5] {
            let resources = OperationGenerationResources::with_test_limits(visits, 0);
            let mut values = [4.0, 3.0, 2.0, 1.0];
            let result = sort_numeric_source(
                &mut values,
                Some(plot.data().with_axis_resources(&resources)),
            );
            assert_eq!(result.is_ok(), visits >= 4);
            assert_eq!(
                values.map(f64::to_bits),
                if visits >= 4 {
                    [1.0, 2.0, 3.0, 4.0]
                } else {
                    [4.0, 3.0, 2.0, 1.0]
                }
                .map(f64::to_bits)
            );
            assert_eq!(
                resources.usage().unwrap(),
                (0, if visits >= 4 { 4 } else { 0 })
            );
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
        // Three sample validations and two grid validations. Output storage is
        // admitted separately; no kernel visit is permitted after validation.
        let resources = OperationGenerationResources::with_test_limits(5, 16);
        assert!(evaluate(&[1.0, 2.0, 3.0], &resources)
            .unwrap_err()
            .to_string()
            .contains("work admission"));
        assert_eq!(resources.usage().unwrap(), (2, 5));
    }

    #[test]
    fn source_collection_has_only_storage_cost_and_normalization_admits_its_work() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        let calls = std::cell::Cell::new(0);
        let resources = OperationGenerationResources::with_test_limits(0, 0);
        let data = plot.data().with_axis_resources(&resources);
        assert!(data
            .collect_source((0..3).map(|value| {
                calls.set(calls.get() + 1);
                Ok(value)
            }))
            .is_err());
        assert_eq!(calls.get(), 0);
        let resources = OperationGenerationResources::with_test_limits(0, 1024);
        let data = plot.data().with_axis_resources(&resources);
        assert_eq!(
            data.collect_source([1_u64, 2, 3].into_iter().map(Ok))
                .unwrap(),
            [1, 2, 3]
        );
        assert_eq!(resources.usage().unwrap(), (3, 0));
        assert_eq!(resources.byte_usage().unwrap(), 24);
        assert!(data
            .normalize_source_string(" a\t b ")
            .unwrap_err()
            .to_string()
            .contains("work admission"));
        for source in ["", "   ", " a\t b ", "a\nb\r\nc", "  alpha  beta"] {
            assert_eq!(
                plot.data().normalize_source_string(source).unwrap(),
                crate::data::collapse_whitespace(source)
            );
        }
    }

    #[test]
    fn p2_list_searches_charge_only_examined_entries_before_comparison() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .erase();
        for (needle, examined) in [(1, 1), (3, 3), (4, 3)] {
            for available in [examined - 1, examined, examined + 1] {
                let resources = OperationGenerationResources::with_test_limits(available, 0);
                let comparisons = std::cell::Cell::new(0);
                let found = source
                    .data()
                    .with_axis_resources(&resources)
                    .source_position(&[1, 2, 3], |value| {
                        comparisons.set(comparisons.get() + 1);
                        *value == needle
                    });
                assert_eq!(found.is_ok(), available >= examined);
                assert_eq!(comparisons.get(), available.min(examined));
                assert_eq!(resources.usage().unwrap(), (0, available.min(examined)));
                if let Ok(index) = found {
                    assert_eq!(index, if needle <= 3 { Some(needle - 1) } else { None });
                }
            }
        }
    }

    #[test]
    fn p2_row_topology_ticket_is_bound_to_its_source_and_shared_operation() {
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0, 3.0]),
            crate::NumericValues::new("y", [1.0, 2.0, 3.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let admission = source
            .data()
            .with_axis_resources(&resources)
            .admit_row_topology()
            .unwrap();
        let clone = resources.clone();
        let rows = [2, 0];
        let ids = [302, 300];
        let data = PlotData::with_rows(&source, &rows, &ids)
            .with_axis_resources(&clone)
            .with_row_topology(Some(&admission));
        data.claim_mark_rows().unwrap();
        assert_eq!(
            (
                data.row_index(0),
                data.source_row(0),
                data.row_index(1),
                data.source_row(1)
            ),
            (2, 302, 0, 300)
        );
        assert_eq!(resources.topology_usage(), 3);
        assert_eq!(resources.usage().unwrap(), (0, 0));
        assert_eq!(resources.byte_usage().unwrap(), 0);
        let other = OperationGenerationResources::new();
        assert!(data.with_axis_resources(&other).claim_mark_rows().is_err());
        assert!(source
            .clone()
            .data()
            .with_axis_resources(&resources)
            .with_row_topology(Some(&admission))
            .claim_mark_rows()
            .is_err());
        assert_eq!(other.topology_usage(), 0);
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
            resources
                .claim_slots(1_000_000 - resources.usage().unwrap().0)
                .unwrap();
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
