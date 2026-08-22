#![allow(
    dead_code,
    reason = "fixtures are shared by independently compiled integration test targets"
)]

use std::collections::BTreeMap;

use lichtung_plot::{AxisValues, BooleanValues, CategoryValues, NumericValues, SourceRows};

#[derive(Clone)]
pub(crate) enum FixtureColumn {
    Numeric(Vec<Option<f64>>),
    Category(Vec<Option<String>>),
    Boolean(Vec<Option<bool>>),
}

impl FixtureColumn {
    fn len(&self) -> usize {
        match self {
            Self::Numeric(values) => values.len(),
            Self::Category(values) => values.len(),
            Self::Boolean(values) => values.len(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Fixture {
    columns: BTreeMap<String, FixtureColumn>,
    height: usize,
}

impl Fixture {
    pub(crate) fn from_columns(columns: Vec<(&str, FixtureColumn)>) -> Result<Self, String> {
        let height = columns.first().map_or(0, |(_, column)| column.len());
        let mut named = BTreeMap::new();
        for (name, column) in columns {
            if column.len() != height {
                return Err(format!(
                    "fixture column {name:?} has length {}, expected {height}",
                    column.len()
                ));
            }
            if named.insert(name.to_owned(), column).is_some() {
                return Err(format!("duplicate fixture column {name:?}"));
            }
        }
        Ok(Self {
            columns: named,
            height,
        })
    }

    pub(crate) fn insert_column(
        &mut self,
        name: &str,
        column: FixtureColumn,
    ) -> Result<(), String> {
        if column.len() != self.height {
            return Err(format!(
                "fixture column {name:?} has length {}, expected {}",
                column.len(),
                self.height
            ));
        }
        self.columns.insert(name.to_owned(), column);
        Ok(())
    }

    fn column(&self, name: &str) -> &FixtureColumn {
        self.columns
            .get(name)
            .unwrap_or_else(|| panic!("missing fixture column {name:?}"))
    }

    pub(crate) fn has_column(&self, name: &str) -> bool {
        self.columns.contains_key(name)
    }
}

pub(crate) type DataFrame = Fixture;

pub(crate) struct Series {
    name: String,
    column: FixtureColumn,
}

pub(crate) struct FixtureField(Series);

impl From<Series> for FixtureField {
    fn from(series: Series) -> Self {
        Self(series)
    }
}

impl Series {
    pub(crate) fn new(name: String, values: impl IntoFixtureColumn) -> Self {
        Self {
            name,
            column: values.into_fixture_column(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.column.len()
    }
}

impl Fixture {
    pub(crate) fn new(height: usize, columns: Vec<FixtureField>) -> Result<Self, String> {
        let mut fixture = Self {
            columns: BTreeMap::new(),
            height,
        };
        for FixtureField(series) in columns {
            fixture.insert_column(&series.name, series.column)?;
        }
        Ok(fixture)
    }

    pub(crate) fn with_column(&mut self, FixtureField(series): FixtureField) -> Result<(), String> {
        self.insert_column(&series.name, series.column)
    }
}

pub(crate) fn numeric(fixture: &Fixture, column: &str) -> NumericValues {
    match fixture.column(column) {
        FixtureColumn::Numeric(values) => NumericValues::nullable(column, values.iter().copied()),
        FixtureColumn::Category(_) | FixtureColumn::Boolean(_) => {
            panic!("fixture column {column:?} must be numeric")
        }
    }
}

pub(crate) fn category(fixture: &Fixture, column: &str) -> CategoryValues {
    let values = match fixture.column(column) {
        FixtureColumn::Category(values) => values.clone(),
        FixtureColumn::Numeric(values) => values
            .iter()
            .map(|value| value.map(|value| value.to_string()))
            .collect(),
        FixtureColumn::Boolean(values) => values
            .iter()
            .map(|value| value.map(|value| value.to_string()))
            .collect(),
    };
    CategoryValues::nullable(column, values).unwrap()
}

pub(crate) fn boolean(fixture: &Fixture, column: &str) -> BooleanValues {
    match fixture.column(column) {
        FixtureColumn::Boolean(values) => BooleanValues::nullable(column, values.iter().copied()),
        FixtureColumn::Numeric(_) | FixtureColumn::Category(_) => {
            panic!("fixture column {column:?} must be Boolean")
        }
    }
}

pub(crate) fn axis(fixture: &Fixture, column: &str) -> AxisValues {
    match fixture.column(column) {
        FixtureColumn::Numeric(_) => numeric(fixture, column).into(),
        FixtureColumn::Category(_) | FixtureColumn::Boolean(_) => category(fixture, column).into(),
    }
}

pub(crate) fn source_rows(fixture: &Fixture, column: &str) -> SourceRows {
    let FixtureColumn::Numeric(values) = fixture.column(column) else {
        panic!("fixture column {column:?} must be numeric")
    };
    SourceRows::new(values.iter().map(|value| {
        let value = value.unwrap_or_else(|| panic!("fixture column {column:?} contains null"));
        assert!(value >= 0.0 && value.fract() == 0.0);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the fixture value is validated as a nonnegative integer source row"
        )]
        let value = value as usize;
        value
    }))
    .unwrap()
}

pub(crate) trait FixtureNumber: Copy {
    fn to_f64(self) -> f64;
}

impl FixtureNumber for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

macro_rules! fixture_numbers_from {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FixtureNumber for $type {
                fn to_f64(self) -> f64 {
                    f64::from(self)
                }
            }
        )+
    };
}

fixture_numbers_from!(f32, i8, i16, i32, u8, u16, u32);

macro_rules! fixture_numbers_wide {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FixtureNumber for $type {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "test fixtures intentionally normalize numeric literals to f64 display values"
                )]
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )+
    };
}

fixture_numbers_wide!(i64, i128, isize, u64, u128, usize);

pub(crate) trait IntoFixtureColumn {
    fn into_fixture_column(self) -> FixtureColumn;
}

impl<T: FixtureNumber, const N: usize> IntoFixtureColumn for [T; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Numeric(self.into_iter().map(|value| Some(value.to_f64())).collect())
    }
}

impl<T: FixtureNumber> IntoFixtureColumn for Vec<T> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Numeric(self.into_iter().map(|value| Some(value.to_f64())).collect())
    }
}

impl<T: FixtureNumber, const N: usize> IntoFixtureColumn for [Option<T>; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Numeric(
            self.into_iter()
                .map(|value| value.map(FixtureNumber::to_f64))
                .collect(),
        )
    }
}

impl<T: FixtureNumber> IntoFixtureColumn for Vec<Option<T>> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Numeric(
            self.into_iter()
                .map(|value| value.map(FixtureNumber::to_f64))
                .collect(),
        )
    }
}

impl<const N: usize> IntoFixtureColumn for [&str; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(
            self.into_iter()
                .map(|value| Some(value.to_owned()))
                .collect(),
        )
    }
}

impl IntoFixtureColumn for Vec<&str> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(
            self.into_iter()
                .map(|value| Some(value.to_owned()))
                .collect(),
        )
    }
}

impl<const N: usize> IntoFixtureColumn for [String; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(self.into_iter().map(Some).collect())
    }
}

impl IntoFixtureColumn for Vec<String> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(self.into_iter().map(Some).collect())
    }
}

impl<const N: usize> IntoFixtureColumn for [Option<&str>; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(
            self.into_iter()
                .map(|value| value.map(str::to_owned))
                .collect(),
        )
    }
}

impl IntoFixtureColumn for Vec<Option<&str>> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(
            self.into_iter()
                .map(|value| value.map(str::to_owned))
                .collect(),
        )
    }
}

impl<const N: usize> IntoFixtureColumn for [Option<String>; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(self.into_iter().collect())
    }
}

impl IntoFixtureColumn for Vec<Option<String>> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Category(self)
    }
}

impl<const N: usize> IntoFixtureColumn for [bool; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Boolean(self.into_iter().map(Some).collect())
    }
}

impl IntoFixtureColumn for Vec<bool> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Boolean(self.into_iter().map(Some).collect())
    }
}

impl<const N: usize> IntoFixtureColumn for [Option<bool>; N] {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Boolean(self.into_iter().collect())
    }
}

impl IntoFixtureColumn for Vec<Option<bool>> {
    fn into_fixture_column(self) -> FixtureColumn {
        FixtureColumn::Boolean(self)
    }
}

pub(crate) fn into_column(values: impl IntoFixtureColumn) -> FixtureColumn {
    values.into_fixture_column()
}

macro_rules! test_data {
    ($($name:literal => $values:expr),+ $(,)?) => {
        $crate::data::Fixture::from_columns(vec![
            $(($name, $crate::data::into_column($values))),+
        ])
    };
}

pub(crate) use test_data;

macro_rules! df {
    ($($name:literal => $values:expr),+ $(,)?) => {
        $crate::data::test_data!($($name => $values),+)
    };
}

pub(crate) use df;
