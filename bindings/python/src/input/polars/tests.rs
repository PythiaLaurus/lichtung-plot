//! Minimal producer-fidelity and C-data ownership guards.

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use polars_arrow::array::{Array, PrimitiveArray};
use polars_arrow::bitmap::Bitmap;
use polars_arrow::ffi::export_iterator;
use polars_core::prelude::PolarsResult;

use super::*;

#[test]
fn actual_chunked_slice_keeps_null_nan_order_and_both_zero_signs() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let fixture = PyModule::from_code(
            py,
            cr"import polars as pl
source = pl.concat([
    pl.Series('', [-7.0, None, float('nan'), -0.0]),
    pl.Series('', [0.0, 2.5, None, 9.0]),
], rechunk=False).slice(1, 6)
",
            c"polars_slice_contract.py",
            c"polars_slice_contract",
        )?;
        let source = fixture.getattr("source")?;
        assert!(source.call_method0("n_chunks")?.extract::<usize>()? >= 2);
        let imported = import_series(&source, "x")?;
        assert!(imported.name().is_empty());
        assert_eq!(imported.dtype(), &DataType::Float64);
        let values = imported.f64().unwrap().iter().collect::<Vec<_>>();
        assert_eq!(values.len(), 6);
        assert!(values[0].is_none() && values[5].is_none());
        assert!(values[1].unwrap().is_nan());
        assert_eq!(values[2].unwrap().to_bits(), (-0.0_f64).to_bits());
        assert_eq!(values[3].unwrap().to_bits(), 0.0_f64.to_bits());
        assert_eq!(values[4], Some(2.5));
        Ok(())
    })
    .unwrap();
}

#[derive(Default)]
struct Releases {
    capsule: AtomicUsize,
    stream: AtomicUsize,
    arrays: AtomicUsize,
    polls: AtomicUsize,
}

struct ArrayRelease(Arc<Releases>);

impl Drop for ArrayRelease {
    fn drop(&mut self) {
        self.0.arrays.fetch_add(1, Ordering::SeqCst);
    }
}

/// A specific owned Int64 fixture, not a C layout/callback implementation. Its
/// downcast is the real PrimitiveArray required by the upstream safe exporter.
/// The guard lives in that export's owned Array until its C-data release.
#[derive(Clone)]
struct CountedInt64 {
    inner: PrimitiveArray<i64>,
    release: Arc<ArrayRelease>,
}

impl CountedInt64 {
    fn boxed(values: Vec<i64>, releases: &Arc<Releases>) -> ArrayRef {
        Box::new(Self {
            inner: PrimitiveArray::from_vec(values),
            release: Arc::new(ArrayRelease(Arc::clone(releases))),
        })
    }
}

impl Array for CountedInt64 {
    fn as_any(&self) -> &dyn Any {
        &self.inner
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.inner
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn dtype(&self) -> &ArrowDataType {
        self.inner.dtype()
    }

    fn dtype_mut(&mut self) -> &mut ArrowDataType {
        self.inner.dtype_mut()
    }

    fn validity(&self) -> Option<&Bitmap> {
        self.inner.validity()
    }

    fn split_at_boxed(&self, offset: usize) -> (ArrayRef, ArrayRef) {
        assert!(offset <= self.len());
        let mut left = self.clone();
        let mut right = self.clone();
        left.slice(0, offset);
        right.slice(offset, self.len() - offset);
        (Box::new(left), Box::new(right))
    }

    #[allow(unsafe_code)]
    unsafe fn split_at_boxed_unchecked(&self, offset: usize) -> (ArrayRef, ArrayRef) {
        // SAFETY: the trait requires an in-bounds offset; the checked native
        // implementation satisfies it without any foreign-pointer dereference.
        self.split_at_boxed(offset)
    }

    fn slice(&mut self, offset: usize, length: usize) {
        self.inner.slice(offset, length);
    }

    #[allow(unsafe_code)]
    unsafe fn slice_unchecked(&mut self, offset: usize, length: usize) {
        // SAFETY: a checked native slice satisfies the trait's in-bounds contract.
        self.slice(offset, length);
    }

    fn with_validity(&self, validity: Option<Bitmap>) -> ArrayRef {
        Box::new(Self {
            inner: self.inner.clone().with_validity(validity),
            release: Arc::clone(&self.release),
        })
    }

    fn to_boxed(&self) -> ArrayRef {
        Box::new(self.clone())
    }
}

struct OwnedChunks {
    chunks: std::vec::IntoIter<PolarsResult<ArrayRef>>,
    releases: Arc<Releases>,
}

impl Iterator for OwnedChunks {
    type Item = PolarsResult<ArrayRef>;

    fn next(&mut self) -> Option<Self::Item> {
        self.releases.polls.fetch_add(1, Ordering::SeqCst);
        self.chunks.next()
    }
}

impl Drop for OwnedChunks {
    fn drop(&mut self) {
        self.releases.stream.fetch_add(1, Ordering::SeqCst);
    }
}

fn owned_capsule<'py>(
    py: Python<'py>,
    field: Field,
    chunks: Vec<PolarsResult<ArrayRef>>,
    releases: &Arc<Releases>,
) -> PyResult<Bound<'py, PyCapsule>> {
    let iterator = OwnedChunks {
        chunks: chunks.into_iter(),
        releases: Arc::clone(releases),
    };
    let stream = export_iterator(Box::new(iterator), field);
    let releases = Arc::clone(releases);
    // The safe constructor stores the actual repr(C) ArrowArrayStream value at
    // the capsule pointer. No raw allocation/layout/release callbacks are made.
    PyCapsule::new_with_value_and_destructor(py, stream, c"arrow_array_stream", move |stream, _| {
        releases.capsule.fetch_add(1, Ordering::SeqCst);
        drop(stream);
    })
}

fn expected_int64(len: usize) -> ExpectedSeries {
    ExpectedSeries {
        name: "original raw name".into(),
        len,
        logical: LogicalType::Primitive(DataType::Int64),
    }
}

#[test]
fn owned_stream_release_is_distinct_from_successful_array_buffer_owners() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let releases = Arc::new(Releases::default());
        let chunks = vec![
            Ok(CountedInt64::boxed(vec![10], &releases)),
            Ok(CountedInt64::boxed(vec![20], &releases)),
        ];
        let capsule = owned_capsule(
            py,
            Field::new("exported".into(), ArrowDataType::Int64, true),
            chunks,
            &releases,
        )?;
        let imported = consume_capsule(capsule, &expected_int64(2), "x")?;
        assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        assert_eq!(releases.polls.load(Ordering::SeqCst), 3);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 0);
        assert_eq!(imported.name().as_str(), "original raw name");
        assert_eq!(
            imported.i64().unwrap().iter().collect::<Vec<_>>(),
            [Some(10), Some(20)]
        );
        let y = Column::new("response".into(), [1.0, 2.0]);
        let (native, _metadata) = lichtung_plot_polars::Scatter::new(&imported, &y)
            .unwrap()
            .into_parts();
        drop(imported);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 2);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        // Actual adapter/native consumer: keeping its Plot/metadata alive must
        // not retain an imported C-data owner after the binding snapshot.
        native.preflight().unwrap();
        Ok(())
    })
    .unwrap();
}

#[test]
fn owned_first_and_second_chunk_errors_release_consumed_and_queued_arrays() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        for fail_after in [0, 1] {
            let releases = Arc::new(Releases::default());
            let first = Ok(CountedInt64::boxed(vec![10], &releases));
            let queued = Ok(CountedInt64::boxed(vec![20], &releases));
            let error = Err(PolarsError::ComputeError("owned producer failure".into()));
            let chunks = if fail_after == 0 {
                vec![error, first, queued]
            } else {
                vec![first, error, queued]
            };
            let capsule = owned_capsule(
                py,
                Field::new("exported".into(), ArrowDataType::Int64, true),
                chunks,
                &releases,
            )?;
            let error = consume_capsule(capsule, &expected_int64(2), "x").unwrap_err();
            assert!(error.is_instance_of::<crate::DataError>(py));
            assert!(error.to_string().contains("owned producer failure"));
            assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
            assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
            assert_eq!(releases.polls.load(Ordering::SeqCst), fail_after + 1);
            assert_eq!(releases.arrays.load(Ordering::SeqCst), 2);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn owned_field_and_capsule_name_guards_release_before_any_value_callback() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let releases = Arc::new(Releases::default());
        let field = Field::new("exported".into(), ArrowDataType::Int64, true);
        let chunks = vec![Ok(CountedInt64::boxed(vec![10], &releases))];
        let capsule = owned_capsule(py, field.clone(), chunks, &releases)?;
        let expected = ExpectedSeries {
            name: "raw".into(),
            len: 1,
            logical: LogicalType::Primitive(DataType::Float64),
        };
        let error = consume_capsule(capsule, &expected, "x").unwrap_err();
        assert!(error.is_instance_of::<PyTypeError>(py));
        assert_eq!(releases.polls.load(Ordering::SeqCst), 0);
        assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 1);

        let releases = Arc::new(Releases::default());
        let iterator = OwnedChunks {
            chunks: vec![Ok(CountedInt64::boxed(vec![10], &releases))].into_iter(),
            releases: Arc::clone(&releases),
        };
        let stream = export_iterator(Box::new(iterator), field);
        let destructor_releases = Arc::clone(&releases);
        // Own a valid stream under a different name; no invalid raw pointer and
        // no mutation of a producer capsule's name/destructor is involved.
        let capsule = PyCapsule::new_with_value_and_destructor(
            py,
            stream,
            c"owned_not_arrow",
            move |stream, _| {
                destructor_releases.capsule.fetch_add(1, Ordering::SeqCst);
                drop(stream);
            },
        )?;
        let error = consume_capsule(capsule, &expected_int64(1), "x").unwrap_err();
        assert!(error.is_instance_of::<pyo3::exceptions::PyValueError>(py));
        assert_eq!(releases.polls.load(Ordering::SeqCst), 0);
        assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 1);
        Ok(())
    })
    .unwrap();
}
