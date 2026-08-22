use polars::prelude::{AnyValue, DataFrame, DataType};

use crate::{AxisValues, CategoryValues, NumericValues, SourceRows};

pub(crate) fn numeric(frame: &DataFrame, column: &str) -> NumericValues {
    let source = frame.column(column).unwrap();
    assert!(source.dtype().is_numeric(), "{column} must be numeric");
    let cast = source.cast(&DataType::Float64).unwrap();
    NumericValues::nullable(column, cast.f64().unwrap().iter())
}

pub(crate) fn category(frame: &DataFrame, column: &str) -> CategoryValues {
    let values = frame
        .column(column)
        .unwrap()
        .as_materialized_series()
        .iter()
        .map(|value| match value {
            AnyValue::Null => None,
            AnyValue::String(value) => Some(value.to_owned()),
            AnyValue::StringOwned(value) => Some(value.as_str().to_owned()),
            value => Some(value.to_string()),
        });
    CategoryValues::nullable(column, values).unwrap()
}

pub(crate) fn axis(frame: &DataFrame, column: &str) -> AxisValues {
    if frame.column(column).unwrap().dtype().is_numeric() {
        numeric(frame, column).into()
    } else {
        category(frame, column).into()
    }
}

pub(crate) fn source_rows(frame: &DataFrame, column: &str) -> SourceRows {
    let source = frame.column(column).unwrap();
    let cast = source.cast(&DataType::UInt64).unwrap();
    SourceRows::new(
        cast.u64()
            .unwrap()
            .iter()
            .map(|value| usize::try_from(value.unwrap()).unwrap()),
    )
    .unwrap()
}

pub(crate) fn constant_numeric(frame: &DataFrame, column: &str) -> f64 {
    let source = frame
        .column(column)
        .unwrap()
        .cast(&DataType::Float64)
        .unwrap();
    let values = source.f64().unwrap();
    let first = values.get(0).unwrap();
    assert_eq!(values.len(), frame.height());
    assert!(values
        .iter()
        .all(|value| value.is_some_and(|value| (value - first).abs() <= 1e-12)));
    first
}
