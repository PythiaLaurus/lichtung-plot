//! Private fidelity/release checks complement the installed-wheel Scatter users.
//! No Python debug exporter/importer or arbitrary raw-pointer producer is exposed.

use std::any::Any;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use polars_arrow::array::{Array, DictionaryArray, PrimitiveArray, Utf8Array, Utf8ViewArray};
use polars_arrow::bitmap::Bitmap;
use polars_arrow::datatypes::Metadata;
use polars_arrow::ffi::export_iterator;
use polars_core::prelude::{AnyValue, PolarsResult};
use pyo3::types::PyList;

use super::*;

fn python_series<'py>(
    py: Python<'py>,
    name: &str,
    dtype: &Bound<'py, PyAny>,
    values: &Bound<'py, PyList>,
) -> PyResult<Bound<'py, PyAny>> {
    let keywords = PyDict::new(py);
    keywords.set_item("dtype", dtype)?;
    PyModule::import(py, "polars")?
        .getattr("Series")?
        .call((name, values), Some(&keywords))
}

#[test]
fn concrete_primitive_identity_is_not_logical_or_subclass_inference() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        PyModule::import(py, "polars")?;
        let classes = PyModule::import(py, "polars.datatypes.classes")?;
        for (name, native, _) in PRIMITIVES {
            let class = classes.getattr(name)?;
            assert_eq!(
                primitive_dtype(&class)?,
                Some(native.clone()),
                "{name} class"
            );
            assert_eq!(
                primitive_dtype(&class.call0()?)?,
                Some(native),
                "{name} instance"
            );
        }
        let fixture = PyModule::from_code(
            py,
            cr"import polars as pl
class Derived(pl.Int64):
    pass
excluded = [pl.Boolean, pl.Null, pl.String, pl.Date, pl.Datetime('ns'),
            pl.Duration('us'), pl.Decimal(20, 2), pl.Binary, pl.Object,
            pl.List(pl.Int64), pl.Array(pl.Int64, 2),
            pl.Struct({'a': pl.Int64}), pl.Categorical(),
            pl.Enum(['unused', 'seen']), pl.Extension,
            pl.Extension('binding.example', pl.Int64, 'logical metadata'),
            Derived, Derived()]
",
            c"polars_dtype_contract.py",
            c"polars_dtype_contract",
        )?;
        for dtype in fixture.getattr("excluded")?.try_iter()? {
            assert_eq!(primitive_dtype(&dtype?)?, None);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn actual_thirteen_primitives_keep_dtype_values_and_typed_missing_shapes() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        PyModule::import(py, "polars")?;
        let classes = PyModule::import(py, "polars.datatypes.classes")?;
        for (name, native, _) in PRIMITIVES {
            let class = classes.getattr(name)?;
            let values = PyList::new(py, [Some(1_i64), None, Some(3)])?;
            let original_name = format!("raw_{name};populated");
            let source = python_series(py, &original_name, &class, &values)?;
            let imported = import_series(&source, "x")?;
            assert_eq!(imported.name().as_str(), original_name);
            assert_eq!(imported.dtype(), &native, "{name} populated");
            assert_eq!(imported.len(), values.len());
            assert_eq!(imported.get(0).unwrap().extract::<f64>(), Some(1.0));
            assert!(matches!(imported.get(1).unwrap(), AnyValue::Null));
            assert_eq!(imported.get(2).unwrap().extract::<f64>(), Some(3.0));
        }
        let empty = PyList::empty(py);
        let all_null = PyList::new(py, [None::<i64>, None])?;
        for (name, native, shape, values) in [
            ("Int128", DataType::Int128, "empty", &empty),
            ("Float16", DataType::Float16, "all-null", &all_null),
        ] {
            let class = classes.getattr(name)?;
            let original_name = format!("raw_{name};{shape}");
            let source = python_series(py, &original_name, &class, values)?;
            let imported = import_series(&source, "x")?;
            assert_eq!(imported.name().as_str(), original_name);
            assert_eq!(imported.dtype(), &native, "{name} {shape}");
            assert_eq!(imported.len(), values.len());
            assert_eq!(imported.null_count(), imported.len());
        }
        Ok(())
    })
    .unwrap();
}

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

#[test]
fn actual_string_boolean_and_null_keep_non_numeric_logical_fidelity() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let fixture = PyModule::from_code(
            py,
            cr"import polars as pl
strings = pl.Series('raw_text', ['drop', 'a;b', None, '', 'tail']).slice(1, 3)
booleans = pl.concat([
    pl.Series('flag', [False, True, None]),
    pl.Series('flag', [False, True]),
], rechunk=False).slice(1, 3)
nulls = pl.Series('missing', [None, None], dtype=pl.Null)
",
            c"polars_non_numeric_contract.py",
            c"polars_non_numeric_contract",
        )?;
        let strings = import_series(&fixture.getattr("strings")?, "hue")?;
        assert_eq!(strings.dtype(), &DataType::String);
        assert_eq!(
            strings.str().unwrap().iter().collect::<Vec<_>>(),
            [Some("a;b"), None, Some("")]
        );
        // Import fidelity only: no claim of a public Python Boolean/Heatmap API.
        let booleans = import_series(&fixture.getattr("booleans")?, "Boolean")?;
        assert_eq!(booleans.dtype(), &DataType::Boolean);
        assert_eq!(
            booleans.bool().unwrap().iter().collect::<Vec<_>>(),
            [Some(true), None, Some(false)]
        );
        let nulls = import_series(&fixture.getattr("nulls")?, "x")?;
        assert_eq!(nulls.dtype(), &DataType::Null);
        assert_eq!((nulls.len(), nulls.null_count()), (2, 2));
        Ok(())
    })
    .unwrap();
}

fn decoded_categories(column: &Column) -> Vec<Option<String>> {
    (0..column.len())
        .map(|index| match column.get(index).unwrap() {
            AnyValue::Null => None,
            value => Some(value.str_value().into_owned()),
        })
        .collect()
}

#[test]
fn actual_categories_keep_named_widths_and_all_enum_levels_not_just_observed_labels() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let fixture = PyModule::from_code(
            py,
            cr"import polars as pl
cat8 = pl.Categorical(pl.Categories('name;8', 'binding;ffi', physical=pl.UInt8))
cat16 = pl.Categorical(pl.Categories('name;16', 'binding;ffi', physical=pl.UInt16))
enum = pl.Enum(['unused', 'beta', 'alpha', ''])
def source(dtype):
    return pl.concat([
        pl.Series('group', ['alpha', None, 'beta'], dtype=dtype),
        pl.Series('group', ['beta', 'alpha'], dtype=dtype),
    ], rechunk=False).slice(1, 4)
sources = [source(cat8), source(cat16), source(enum)]
",
            c"polars_category_contract.py",
            c"polars_category_contract",
        )?;
        let sources = fixture.getattr("sources")?;
        for (index, name, physical) in [
            (0, "name;8", CategoricalPhysical::U8),
            (1, "name;16", CategoricalPhysical::U16),
        ] {
            let source = sources.get_item(index)?;
            assert!(source.call_method0("n_chunks")?.extract::<usize>()? >= 2);
            let imported = import_series(&source, "hue")?;
            let DataType::Categorical(categories, _) = imported.dtype() else {
                panic!("named categorical was demoted");
            };
            assert_eq!(categories.name().as_str(), name);
            assert_eq!(categories.namespace().as_str(), "binding;ffi");
            assert_eq!(categories.physical(), physical);
            assert_eq!(
                decoded_categories(&imported),
                [
                    None,
                    Some("beta".to_owned()),
                    Some("beta".to_owned()),
                    Some("alpha".to_owned())
                ]
            );
        }
        let imported = import_series(&sources.get_item(2)?, "hue")?;
        let DataType::Enum(categories, _) = imported.dtype() else {
            panic!("Enum was demoted");
        };
        assert_eq!(
            categories.categories().values_iter().collect::<Vec<_>>(),
            ["unused", "beta", "alpha", ""]
        );
        assert_eq!(categories.physical(), CategoricalPhysical::U8);
        assert_eq!(
            decoded_categories(&imported),
            [
                None,
                Some("beta".to_owned()),
                Some("beta".to_owned()),
                Some("alpha".to_owned())
            ]
        );
        Ok(())
    })
    .unwrap();
}

#[test]
fn actual_wide_integers_reach_the_exact_native_id_authority() {
    use lichtung_plot_polars::{AdapterError, SourceRows, SourceRowsInput};

    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let fixture = PyModule::from_code(
            py,
            cr"import polars as pl
ids = pl.Series('identity', [2**53, 2**53 + 1], dtype=pl.UInt64)
overflow = pl.Series('identity', [2**100], dtype=pl.UInt128)
signed = pl.Series('signed', [-(2**100) + 1], dtype=pl.Int128)
",
            c"polars_id_contract.py",
            c"polars_id_contract",
        )?;
        let imported = import_series(&fixture.getattr("ids")?, "source rows")?;
        assert_eq!(
            imported.u64().unwrap().iter().collect::<Vec<_>>(),
            [Some(1_u64 << 53), Some((1_u64 << 53) + 1)]
        );
        assert_eq!(
            (&imported).bind_source_rows(None, "source rows").unwrap(),
            SourceRows::new([
                usize::try_from(1_u64 << 53).unwrap(),
                usize::try_from((1_u64 << 53) + 1).unwrap(),
            ])
            .unwrap()
        );
        let overflow = import_series(&fixture.getattr("overflow")?, "source rows")?;
        assert_eq!(overflow.u128().unwrap().get(0), Some(1_u128 << 100));
        assert!(matches!(
            (&overflow).bind_source_rows(None, "source rows"),
            Err(AdapterError::SourceRow { slot: 0, .. })
        ));
        let signed = import_series(&fixture.getattr("signed")?, "x")?;
        assert_eq!(signed.i128().unwrap().get(0), Some(-(1_i128 << 100) + 1));
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

#[test]
fn owned_empty_and_mismatched_streams_keep_length_contracts() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let field = Field::new("exported".into(), ArrowDataType::Int64, true);
        let releases = Arc::new(Releases::default());
        let capsule = owned_capsule(py, field.clone(), vec![], &releases)?;
        let empty = consume_capsule(capsule, &expected_int64(0), "x")?;
        assert_eq!(empty.dtype(), &DataType::Int64);
        assert_eq!((empty.len(), empty.n_chunks()), (0, 1));
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);

        // A zero-chunk Enum still needs its complete logical level metadata,
        // even though every level is unobserved and the fallback array is empty.
        let releases = Arc::new(Releases::default());
        let enum_field = dictionary_field(DTYPE_ENUM_VALUES_NEW, Some("6;unused1;b1;a"));
        let capsule = owned_capsule(py, enum_field, vec![], &releases)?;
        let expected = ExpectedSeries {
            name: "empty enum".into(),
            len: 0,
            logical: LogicalType::Enum(vec!["unused".into(), "b".into(), "a".into()]),
        };
        let empty_enum = consume_capsule(capsule, &expected, "hue")?;
        assert_eq!(empty_enum.name().as_str(), "empty enum");
        assert_eq!((empty_enum.len(), empty_enum.n_chunks()), (0, 1));
        let DataType::Enum(categories, _) = empty_enum.dtype() else {
            panic!("zero-chunk Enum metadata was lost");
        };
        assert_eq!(
            categories.categories().values_iter().collect::<Vec<_>>(),
            ["unused", "b", "a"]
        );
        assert_eq!(releases.polls.load(Ordering::SeqCst), 1);
        assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 0);

        let releases = Arc::new(Releases::default());
        let chunks = vec![
            Ok(CountedInt64::boxed(vec![], &releases)),
            Ok(CountedInt64::boxed(vec![42], &releases)),
        ];
        let capsule = owned_capsule(py, field.clone(), chunks, &releases)?;
        let with_empty = consume_capsule(capsule, &expected_int64(1), "x")?;
        assert_eq!(
            with_empty
                .as_materialized_series()
                .chunks()
                .iter()
                .map(|chunk| chunk.len())
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(with_empty.i64().unwrap().get(0), Some(42));
        drop(with_empty);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 2);

        let releases = Arc::new(Releases::default());
        let capsule = owned_capsule(py, field, vec![], &releases)?;
        let error = consume_capsule(capsule, &expected_int64(1), "x").unwrap_err();
        assert!(error.is_instance_of::<PyTypeError>(py));
        assert!(error.to_string().contains("stream length 0"));
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);

        let releases = Arc::new(Releases::default());
        let chunks = vec![Ok(CountedInt64::boxed(vec![10, 20], &releases))];
        let capsule = owned_capsule(
            py,
            Field::new("exported".into(), ArrowDataType::Int64, true),
            chunks,
            &releases,
        )?;
        let error = consume_capsule(capsule, &expected_int64(1), "x").unwrap_err();
        assert!(error.is_instance_of::<PyTypeError>(py));
        assert!(error.to_string().contains("stream is longer"));
        assert_eq!(releases.polls.load(Ordering::SeqCst), 1);
        assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        assert_eq!(releases.arrays.load(Ordering::SeqCst), 1);
        Ok(())
    })
    .unwrap();
}

fn dictionary_field(key: &str, encoded: Option<&str>) -> Field {
    let dtype = ArrowDataType::Dictionary(IntegerType::UInt8, Box::new(ArrowDataType::Utf8), false);
    let mut metadata = Metadata::new();
    if let Some(encoded) = encoded {
        metadata.insert(key.into(), encoded.into());
    }
    Field::new("dictionary".into(), dtype, true).with_metadata(metadata)
}

#[test]
fn malformed_or_missing_logical_metadata_fails_before_any_value_callback() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        assert!(validate_expected_levels(&["a".to_owned(), "a".to_owned()]).is_err());
        // These are valid owned C schemas with invalid Polars metadata, not
        // malicious pointers. The public exporter owns/releases their schemas.
        let cases = [
            (false, None),
            (false, Some("4;ab")),
            (false, Some("1;\u{e9}0;u8;")),
            (false, Some("184467440737095516160;name0;u8;")),
            (false, Some("0;0;u64;")),
            (false, Some("0;0;u8;trailing")),
            (true, None),
            (true, Some("1;a")),
            (true, Some("1;a1;a")),
            (true, Some("6;unused1;a")),
            (true, Some("1;a6;unusedtrailing")),
        ];
        for (is_enum, encoded) in cases {
            let logical = if is_enum {
                LogicalType::Enum(vec!["a".into(), "unused".into()])
            } else {
                LogicalType::Categorical {
                    name: String::new(),
                    namespace: String::new(),
                    physical: CategoricalPhysical::U8,
                }
            };
            let key = if is_enum {
                DTYPE_ENUM_VALUES_NEW
            } else {
                DTYPE_CATEGORICAL_NEW
            };
            let releases = Arc::new(Releases::default());
            let capsule = owned_capsule(py, dictionary_field(key, encoded), vec![], &releases)?;
            let expected = ExpectedSeries {
                name: "raw".into(),
                len: 0,
                logical,
            };
            let error = consume_capsule(capsule, &expected, "hue").unwrap_err();
            assert!(error.is_instance_of::<PyTypeError>(py), "{encoded:?}");
            assert_eq!(releases.polls.load(Ordering::SeqCst), 0);
            assert_eq!(releases.capsule.load(Ordering::SeqCst), 1);
            assert_eq!(releases.stream.load(Ordering::SeqCst), 1);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn owned_string_layouts_keep_logical_values() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let values = [Some("longer than an inline view"), None, Some("a;b")];
        let arrays: [ArrayRef; 3] = [
            Box::new(Utf8Array::<i32>::from(values)),
            Box::new(Utf8Array::<i64>::from(values)),
            Box::new(Utf8ViewArray::from_slice(values)),
        ];
        for array in arrays {
            let releases = Arc::new(Releases::default());
            let field = Field::new("exported".into(), array.dtype().clone(), true);
            let capsule = owned_capsule(py, field, vec![Ok(array)], &releases)?;
            let expected = ExpectedSeries {
                name: "text".into(),
                len: 3,
                logical: LogicalType::String,
            };
            let imported = consume_capsule(capsule, &expected, "hue")?;
            assert_eq!(imported.dtype(), &DataType::String);
            assert_eq!(imported.str().unwrap().iter().collect::<Vec<_>>(), values);
        }

        Ok(())
    })
    .unwrap();
}

#[test]
fn owned_dictionary_codebooks_remap_without_losing_unused_enum_levels() {
    Python::initialize();
    Python::attach(|py| -> PyResult<()> {
        let first = DictionaryArray::<u8>::try_from_keys(
            PrimitiveArray::from([Some(0_u8), None, Some(1)]),
            Box::new(Utf8Array::<i32>::from_slice(["a", "b"])),
            false,
        )
        .unwrap();
        let second = DictionaryArray::<u8>::try_from_keys(
            PrimitiveArray::from_vec(vec![0_u8, 1]),
            Box::new(Utf8Array::<i32>::from_slice(["b", "a"])),
            false,
        )
        .unwrap();
        let field = dictionary_field(DTYPE_ENUM_VALUES_NEW, Some("6;unused1;b1;a"));
        let releases = Arc::new(Releases::default());
        let capsule = owned_capsule(
            py,
            field,
            vec![Ok(Box::new(first)), Ok(Box::new(second))],
            &releases,
        )?;
        let expected = ExpectedSeries {
            name: "group".into(),
            len: 5,
            logical: LogicalType::Enum(vec!["unused".into(), "b".into(), "a".into()]),
        };
        let imported = consume_capsule(capsule, &expected, "hue")?;
        assert_eq!(
            decoded_categories(&imported),
            [
                Some("a".to_owned()),
                None,
                Some("b".to_owned()),
                Some("b".to_owned()),
                Some("a".to_owned())
            ]
        );
        let DataType::Enum(categories, _) = imported.dtype() else {
            panic!("dictionary metadata was lost");
        };
        assert_eq!(
            categories.categories().values_iter().collect::<Vec<_>>(),
            ["unused", "b", "a"]
        );
        Ok(())
    })
    .unwrap();
}
