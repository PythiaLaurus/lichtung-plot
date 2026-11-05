use std::error::Error as _;

use polars_core::prelude::*;

use crate::{boolean, category, numeric, provenance, AdapterError as Error};

#[test]
fn numeric_chunks_and_sliced_validity_preserve_null_nan_and_signed_zero() {
    let mut series = Series::new("measured".into(), [Some(99.0), Some(-0.0), None]);
    series
        .append(&Series::new(
            "measured".into(),
            [Some(f64::NAN), Some(f64::INFINITY), Some(5.0)],
        ))
        .unwrap();
    let column = series.slice(1, 4).into_column();
    assert_eq!(column.as_materialized_series().n_chunks(), 2);
    let values = numeric::values(&column).unwrap();
    assert_eq!(values.len(), 4);
    assert_eq!(values[0].unwrap().to_bits(), (-0.0_f64).to_bits());
    assert!(values[1].is_none());
    assert!(values[2].unwrap().is_nan());
    assert_eq!(values[3].unwrap().to_bits(), f64::INFINITY.to_bits());
}

#[test]
fn numeric_dispatch_rounds_wide_primitives_without_a_precision_gate() {
    let integers = Column::new(
        "counts".into(),
        [Some(9_007_199_254_740_993_u64), None, Some(u64::MAX)],
    );
    assert_eq!(
        numeric::values(&integers)
            .unwrap()
            .iter()
            .map(|value| value.map(f64::to_bits))
            .collect::<Vec<_>>(),
        [
            Some(9_007_199_254_740_992.0_f64.to_bits()),
            None,
            Some(18_446_744_073_709_551_616.0_f64.to_bits())
        ]
    );
    let wide = Column::new("wide".into(), [u128::MAX]);
    assert_eq!(
        numeric::values(&wide).unwrap()[0].unwrap().to_bits(),
        2.0_f64.powi(128).to_bits()
    );
    let negative = Column::new("signed".into(), [i128::MIN]);
    assert_eq!(
        numeric::values(&negative).unwrap()[0].unwrap().to_bits(),
        (-2.0_f64.powi(127)).to_bits()
    );
    let half = Series::new("half".into(), [Some(1.5), None, Some(-0.0)])
        .strict_cast(&DataType::Float16)
        .unwrap()
        .into_column();
    assert_eq!(
        numeric::values(&half)
            .unwrap()
            .iter()
            .map(|value| value.map(f64::to_bits))
            .collect::<Vec<_>>(),
        [Some(1.5_f64.to_bits()), None, Some((-0.0_f64).to_bits())]
    );
}

#[test]
fn scalar_and_untyped_null_columns_use_logical_length_including_zero() {
    let scalar = Column::new_scalar(
        "repeat".into(),
        Scalar::new(DataType::Float64, AnyValue::Float64(1.5)),
        5,
    );
    let values = numeric::values(&scalar).unwrap();
    assert_eq!(values.len(), 5);
    assert!(values
        .iter()
        .all(|value| value.unwrap().to_bits() == 1.5_f64.to_bits()));
    let null = Column::new_scalar(
        "missing".into(),
        Scalar::new(DataType::Null, AnyValue::Null),
        3,
    );
    assert!(numeric::values(&null).unwrap().iter().all(Option::is_none));
    assert_eq!(category::values(&null).unwrap(), [None, None, None]);
    assert_eq!(boolean::values(&null).unwrap(), [None, None, None]);
    assert_eq!(numeric::numeric(&null).unwrap().len(), 3);
    let empty = Column::new_empty("empty".into(), &DataType::Float64);
    assert!(numeric::values(&empty).unwrap().is_empty());
    let typed_null = Column::new_scalar(
        "typed missing".into(),
        Scalar::new(DataType::Float64, AnyValue::Null),
        4,
    );
    assert_eq!(numeric::values(&typed_null).unwrap().len(), 4);
    assert!(numeric::values(&typed_null)
        .unwrap()
        .iter()
        .all(Option::is_none));
}

#[test]
fn categorical_and_enum_decode_observations_not_codes_or_unobserved_levels() {
    let strings = Series::new(
        "raw group".into(),
        [Some(" B "), None, Some("é"), Some(" B ")],
    );
    let categories = Categories::new(
        "adapter identities".into(),
        "lichtung-plot tests".into(),
        CategoricalPhysical::U8,
    );
    let categorical = strings
        .strict_cast(&DataType::from_categories(categories))
        .unwrap()
        .into_column();
    let levels = FrozenCategories::new(["unused", "é", " B "]).unwrap();
    let enumeration = strings
        .strict_cast(&DataType::from_frozen_categories(levels))
        .unwrap()
        .into_column();
    let expected = vec![
        Some(" B ".into()),
        None,
        Some("é".into()),
        Some(" B ".into()),
    ];
    assert_eq!(category::values(&categorical).unwrap(), expected);
    assert_eq!(category::values(&enumeration).unwrap(), expected);
    let owned = category::values(&strings.into_column()).unwrap();
    drop(categorical);
    drop(enumeration);
    assert_eq!(owned, expected);
}

#[test]
fn boolean_chunks_keep_nullable_observations_without_coercion() {
    let mut source = Series::new("flag".into(), [Some(true), None]);
    source
        .append(&Series::new("flag".into(), [Some(false), Some(true)]))
        .unwrap();
    let column = source.slice(1, 3).into_column();
    assert_eq!(
        boolean::values(&column).unwrap(),
        [None, Some(false), Some(true)]
    );
    assert_eq!(boolean::booleans(&column).unwrap().len(), 3);
    assert!(matches!(
        numeric::numeric(&column),
        Err(Error::UnsupportedDtype {
            channel: "numeric",
            ..
        })
    ));
}

#[test]
fn provenance_is_exact_and_never_accepts_numeric_coercion() {
    let column = Column::new("sample".into(), [9_007_199_254_740_993_u64, 42]);
    if usize::BITS > 53 {
        assert_eq!(
            provenance::ids(&column).unwrap()[0],
            usize::try_from(9_007_199_254_740_993_u64).unwrap()
        );
    }
    let negative = Column::new("sample".into(), [Some(10_i128), Some(-1), None]);
    assert!(matches!(
        provenance::source_rows(&negative),
        Err(Error::SourceRow { slot: 1, .. })
    ));
    assert!(matches!(
        provenance::source_rows(&negative.slice(2, 1)),
        Err(Error::SourceRow { slot: 0, .. })
    ));
    assert!(matches!(
        provenance::source_rows(&Column::new("too wide".into(), [u128::MAX])),
        Err(Error::SourceRow { slot: 0, .. })
    ));
    assert!(matches!(
        provenance::source_rows(&Column::new("float".into(), [1.0])),
        Err(Error::UnsupportedDtype { .. })
    ));
    let duplicate = provenance::source_rows(&Column::new("sample".into(), [4, 4])).unwrap_err();
    assert!(duplicate.source().is_some());
    assert!(duplicate.to_string().contains("duplicate 4"));
}

#[test]
fn source_and_direct_bindings_release_borrows_at_native_handoff() {
    let frame = DataFrame::new(
        2,
        vec![
            Column::new("a".into(), [1, 2]),
            Column::new("b".into(), [3.0, 4.0]),
            Column::new("text".into(), ["one", "two"]),
        ],
    )
    .unwrap();
    let external = Series::new("corrected".into(), [5.0, 7.0]);
    let plot = crate::Scatter::from_data(&frame, "a", &external)
        .unwrap()
        .hue("text")
        .unwrap()
        .facet_wrap("text", 2)
        .unwrap();
    drop(external); // Only the explicitly installed DataFrame remains borrowed.
    let native: crate::Plot<crate::kind::Scatter> = plot.into();
    let matrix = crate::Matrix::from_data(&frame, ["b", "a"]).unwrap();
    let matrix: crate::Plot<crate::kind::Matrix> = matrix.into();
    assert!(matches!(
        crate::Scatter::from_data(&frame, "a", "text"),
        Err(Error::UnsupportedDtype { channel: "y", .. })
    ));
    let missing = crate::Scatter::from_data(&frame, "absent", "b")
        .err()
        .unwrap();
    assert!(matches!(missing, Error::Lookup { role: "x", .. }));
    assert!(missing.source().is_some());
    assert!(matches!(
        crate::Scatter::new("a", "b"),
        Err(Error::MissingSource { .. })
    ));
    drop(frame);
    let owned: crate::Scatter<'static> = crate::Scatter::new(
        &Series::new("x".into(), [0.0, 2.0]),
        &Series::new("y".into(), [1.0, 3.0]),
    )
    .unwrap();
    let owned: crate::Plot<crate::kind::Scatter> = owned.into();
    let _canvas = crate::Canvas::grid(1, 3)
        .unwrap()
        .panel(0, 0, native)
        .unwrap()
        .panel(0, 1, matrix)
        .unwrap()
        .panel(0, 2, owned)
        .unwrap();
}

#[test]
fn empty_semantic_dtypes_are_rejected_before_physical_materialization() {
    let date = Column::new_empty("date".into(), &DataType::Date);
    assert!(matches!(
        numeric::numeric(&date),
        Err(Error::UnsupportedDtype {
            dtype: DataType::Date,
            ..
        })
    ));
    let nested = Column::new_empty("nested".into(), &DataType::List(Box::new(DataType::Int64)));
    assert!(matches!(
        category::categories(&nested),
        Err(Error::UnsupportedDtype { .. })
    ));
    let null_id = Column::new_empty("unknown ID".into(), &DataType::Null);
    assert!(matches!(
        provenance::source_rows(&null_id),
        Err(Error::UnsupportedDtype { .. })
    ));
}
