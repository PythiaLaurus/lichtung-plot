use super::PlotAdapter;
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, AlphaEncoding, AlphaInput, AlphaValues, AlphaValuesInput, ArraySource,
    Bounds, CategoryInput, CategoryValues, NumericInput, NumericValues, SourceRowsInput,
};

fn mapped_numeric(
    source: Option<&dyn ArraySource>,
    input: impl NumericInput,
    mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
    copies: usize,
    expected: Option<usize>,
) -> BindingResult<AlphaValues> {
    let copies = if input.native_slots() { 1 } else { copies };
    let raw = input.bind_numeric(source, "alpha map")?;
    let len = raw.len();
    let values = mapper(raw)?;
    length("alpha mapper output", values.name(), len, values.len())?;
    crate::alpha::expand(values, expected, copies)
}
fn mapped_categories(
    source: Option<&dyn ArraySource>,
    input: impl CategoryInput,
    mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
    copies: usize,
    expected: Option<usize>,
) -> BindingResult<AlphaValues> {
    let copies = if input.native_slots() { 1 } else { copies };
    let raw = input.bind_category(source, "alpha map")?;
    let len = raw.len();
    let values = mapper(raw)?;
    length("alpha mapper output", values.name(), len, values.len())?;
    crate::alpha::expand(values, expected, copies)
}

macro_rules! alpha_target {
    ($cap:ident,$base:ident,$cells:ident,$linear:ident,$map:ident,$categories:ident,$cell_map:ident,$cell_categories:ident,
        $single:ident,$single_map:ident,$single_categories:ident,$keyed:ident,$keyed_data:ident,$keyed_map:ident,$keyed_map_data:ident,
        $keyed_categories:ident,$keyed_categories_data:ident,$columns:ident,$columns_data:ident,$columns_map:ident,$columns_map_data:ident,$columns_categories:ident,$columns_categories_data:ident) => {
        impl<K: kind::$cap> PlotAdapter<'_, K> {
            /// Bind observation bytes or pass through a native encoding without reinterpretation.
            pub fn $base(
                mut self,
                input: impl AlphaInput,
            ) -> std::result::Result<Self, AdapterError> {
                let encoding =
                    input.bind_alpha(self.source.as_deref(), self.len, self.row_copies())?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Bind already-expanded bytes without repeating source rows.
            pub fn $cells(
                mut self,
                input: impl AlphaInput,
            ) -> std::result::Result<Self, AdapterError> {
                let encoding = input.bind_alpha(self.source.as_deref(), self.len, 1)?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Materialize the native linear declaration on the bound numeric variable.
            pub fn $linear(
                self,
                input: impl NumericInput,
                domain: (f64, f64),
                range: (u8, u8),
            ) -> std::result::Result<Self, AdapterError> {
                self.$map(input, |values| AlphaValues::linear(values, domain, range))
            }
            /// Map numeric row values once, preserving the native declaration through expansion.
            pub fn $map(
                self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_numeric(
                    self.source.as_deref(),
                    input,
                    mapper,
                    self.row_copies(),
                    self.len,
                )?;
                self.$base(AlphaEncoding::observations(values))
            }
            /// Map categorical row values once before expansion.
            pub fn $categories(
                self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_categories(
                    self.source.as_deref(),
                    input,
                    mapper,
                    self.row_copies(),
                    self.len,
                )?;
                self.$base(AlphaEncoding::observations(values))
            }
            /// Map explicitly expanded numeric slots without row repetition.
            pub fn $cell_map(
                self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_numeric(self.source.as_deref(), input, mapper, 1, self.len)?;
                self.$base(AlphaEncoding::observations(values))
            }
            /// Map explicitly expanded categorical slots without row repetition.
            pub fn $cell_categories(
                self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_categories(self.source.as_deref(), input, mapper, 1, self.len)?;
                self.$base(AlphaEncoding::observations(values))
            }
            /// Explicitly broadcast a u8 literal or exactly one bound alpha value.
            pub fn $single(
                self,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError> {
                let values = input.bind_alpha_values(self.source.as_deref())?;
                self.$base(AlphaEncoding::single(values)?)
            }
            /// Map exactly one numeric member and explicitly broadcast its declaration.
            pub fn $single_map(
                self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_numeric(self.source.as_deref(), input, mapper, 1, Some(1))?;
                self.$base(AlphaEncoding::single(values)?)
            }
            /// Map exactly one categorical member and explicitly broadcast its declaration.
            pub fn $single_categories(
                self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = mapped_categories(self.source.as_deref(), input, mapper, 1, Some(1))?;
                self.$base(AlphaEncoding::single(values)?)
            }
            /// Bind exact series/aggregate keys; observations are never implicitly reduced.
            pub fn $keyed<I, S, B>(
                self,
                keys: I,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let encoding = AlphaEncoding::keyed(
                    crate::alpha::keys(self.source.as_deref(), keys)?,
                    input.bind_alpha_values(self.source.as_deref())?,
                )?;
                self.$base(encoding)
            }
            /// Bind an independent selector source eagerly, retaining no borrow of it.
            pub fn $keyed_data<I, S, B>(
                self,
                source: impl ArraySource,
                keys: I,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let encoding = AlphaEncoding::keyed(
                    crate::alpha::keys(Some(&source), keys)?,
                    input.bind_alpha_values(Some(&source))?,
                )?;
                self.$base(encoding)
            }
            /// Map numeric alpha for exact actual series/aggregate selectors.
            pub fn $keyed_map<I, S, B>(
                self,
                keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let values = mapped_numeric(self.source.as_deref(), input, mapper, 1, None)?;
                self.$keyed(keys, values)
            }
            /// Map numeric alpha in an independent selector source.
            pub fn $keyed_map_data<I, S, B>(
                self,
                source: impl ArraySource,
                keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let values = mapped_numeric(Some(&source), input, mapper, 1, None)?;
                self.$keyed_data(source, keys, values)
            }
            /// Map categorical alpha for exact series/aggregate selectors.
            pub fn $keyed_categories<I, S, B>(
                self,
                keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let values = mapped_categories(self.source.as_deref(), input, mapper, 1, None)?;
                self.$keyed(keys, values)
            }
            /// Map categorical alpha in an independent selector source.
            pub fn $keyed_categories_data<I, S, B>(
                self,
                source: impl ArraySource,
                keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let values = mapped_categories(Some(&source), input, mapper, 1, None)?;
                self.$keyed_data(source, keys, values)
            }
            /// Key alpha by logical source-column indices, not display labels or private field strings.
            pub fn $columns<I, S, B>(
                self,
                column_key: impl SourceRowsInput,
                other_keys: I,
                values: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let expansion = self
                    .expansion
                    .as_ref()
                    .ok_or_else(|| binding("column alpha", "no generated variable dimension"))?;
                let mut keys = crate::alpha::keys(self.source.as_deref(), other_keys)?;
                keys.push(expansion.column_key(self.source.as_deref(), column_key)?);
                let encoding =
                    AlphaEncoding::keyed(keys, values.bind_alpha_values(self.source.as_deref())?)?;
                self.$base(encoding)
            }
            /// Bind generated-column alpha from an independent selector source.
            pub fn $columns_data<I, S, B>(
                self,
                source: impl ArraySource,
                column_key: impl SourceRowsInput,
                other_keys: I,
                values: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let expansion = self
                    .expansion
                    .as_ref()
                    .ok_or_else(|| binding("column alpha", "no generated variable dimension"))?;
                let mut keys = crate::alpha::keys(Some(&source), other_keys)?;
                keys.push(expansion.column_key(Some(&source), column_key)?);
                let encoding =
                    AlphaEncoding::keyed(keys, values.bind_alpha_values(Some(&source))?)?;
                self.$base(encoding)
            }
            /// Map numeric alpha on exact generated-column selectors.
            pub fn $columns_map<I, S, B>(
                self,
                column_key: impl SourceRowsInput,
                other_keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let values = mapped_numeric(self.source.as_deref(), input, mapper, 1, None)?;
                self.$columns(column_key, other_keys, values)
            }
            /// Map numeric generated-column alpha in an independent source.
            pub fn $columns_map_data<I, S, B>(
                self,
                source: impl ArraySource,
                column_key: impl SourceRowsInput,
                other_keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let values = mapped_numeric(Some(&source), input, mapper, 1, None)?;
                self.$columns_data(source, column_key, other_keys, values)
            }
            /// Map categorical alpha on exact generated-column selectors.
            pub fn $columns_categories<I, S, B>(
                self,
                column_key: impl SourceRowsInput,
                other_keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let values = mapped_categories(self.source.as_deref(), input, mapper, 1, None)?;
                self.$columns(column_key, other_keys, values)
            }
            /// Map categorical generated-column alpha in an independent source.
            pub fn $columns_categories_data<I, S, B>(
                self,
                source: impl ArraySource,
                column_key: impl SourceRowsInput,
                other_keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let values = mapped_categories(Some(&source), input, mapper, 1, None)?;
                self.$columns_data(source, column_key, other_keys, values)
            }
        }
    };
}
alpha_target!(
    PointAlpha,
    point_alpha,
    point_alpha_cells,
    point_alpha_linear,
    point_alpha_map,
    point_alpha_map_categories,
    point_alpha_cells_map,
    point_alpha_cells_map_categories,
    point_alpha_single,
    point_alpha_single_map,
    point_alpha_single_map_categories,
    point_alpha_keyed,
    point_alpha_keyed_from_data,
    point_alpha_keyed_map,
    point_alpha_keyed_map_from_data,
    point_alpha_keyed_map_categories,
    point_alpha_keyed_map_categories_from_data,
    point_alpha_columns,
    point_alpha_columns_from_data,
    point_alpha_columns_map,
    point_alpha_columns_map_from_data,
    point_alpha_columns_map_categories,
    point_alpha_columns_map_categories_from_data
);
alpha_target!(
    LineAlpha,
    line_alpha,
    line_alpha_cells,
    line_alpha_linear,
    line_alpha_map,
    line_alpha_map_categories,
    line_alpha_cells_map,
    line_alpha_cells_map_categories,
    line_alpha_single,
    line_alpha_single_map,
    line_alpha_single_map_categories,
    line_alpha_keyed,
    line_alpha_keyed_from_data,
    line_alpha_keyed_map,
    line_alpha_keyed_map_from_data,
    line_alpha_keyed_map_categories,
    line_alpha_keyed_map_categories_from_data,
    line_alpha_columns,
    line_alpha_columns_from_data,
    line_alpha_columns_map,
    line_alpha_columns_map_from_data,
    line_alpha_columns_map_categories,
    line_alpha_columns_map_categories_from_data
);
alpha_target!(
    FillAlpha,
    fill_alpha,
    fill_alpha_cells,
    fill_alpha_linear,
    fill_alpha_map,
    fill_alpha_map_categories,
    fill_alpha_cells_map,
    fill_alpha_cells_map_categories,
    fill_alpha_single,
    fill_alpha_single_map,
    fill_alpha_single_map_categories,
    fill_alpha_keyed,
    fill_alpha_keyed_from_data,
    fill_alpha_keyed_map,
    fill_alpha_keyed_map_from_data,
    fill_alpha_keyed_map_categories,
    fill_alpha_keyed_map_categories_from_data,
    fill_alpha_columns,
    fill_alpha_columns_from_data,
    fill_alpha_columns_map,
    fill_alpha_columns_map_from_data,
    fill_alpha_columns_map_categories,
    fill_alpha_columns_map_categories_from_data
);

impl PlotAdapter<'_, kind::Histogram> {
    /// Bind exact histogram bin intervals and optional facet keys, without sample reduction.
    pub fn fill_alpha_bins<I, S, B>(
        self,
        lower: impl NumericInput,
        upper: impl NumericInput,
        facets: I,
        values: impl AlphaValuesInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, B)>,
        S: AsRef<str>,
        B: CategoryInput,
    {
        let bounds = Bounds::new(
            lower.bind_numeric(self.source.as_deref(), "bin lower")?,
            upper.bind_numeric(self.source.as_deref(), "bin upper")?,
        );
        let encoding = AlphaEncoding::bins(
            bounds,
            crate::alpha::keys(self.source.as_deref(), facets)?,
            values.bind_alpha_values(self.source.as_deref())?,
        )?;
        self.fill_alpha(encoding)
    }
    /// Bind explicit bin alpha in an independent result source, retaining no borrow.
    pub fn fill_alpha_bins_from_data<I, S, B>(
        self,
        source: impl ArraySource,
        lower: impl NumericInput,
        upper: impl NumericInput,
        facets: I,
        values: impl AlphaValuesInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, B)>,
        S: AsRef<str>,
        B: CategoryInput,
    {
        source.validate()?;
        let bounds = Bounds::new(
            lower.bind_numeric(Some(&source), "bin lower")?,
            upper.bind_numeric(Some(&source), "bin upper")?,
        );
        let encoding = AlphaEncoding::bins(
            bounds,
            crate::alpha::keys(Some(&source), facets)?,
            values.bind_alpha_values(Some(&source))?,
        )?;
        self.fill_alpha(encoding)
    }
}
macro_rules! bin_mapping {
    ($name:ident,$from:ident,$input:ident,$value:ty,$map:ident) => {
        impl PlotAdapter<'_, kind::Histogram> {
            /// Map alpha for explicit native bin intervals, never sample rows.
            pub fn $name<I, S, B>(
                self,
                lower: impl NumericInput,
                upper: impl NumericInput,
                facets: I,
                input: impl $input,
                mapper: impl FnOnce($value) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                let values = $map(self.source.as_deref(), input, mapper, 1, None)?;
                self.fill_alpha_bins(lower, upper, facets, values)
            }
            /// Materialize an explicit bin mapping in an independent result source.
            pub fn $from<I, S, B>(
                self,
                source: impl ArraySource,
                lower: impl NumericInput,
                upper: impl NumericInput,
                facets: I,
                input: impl $input,
                mapper: impl FnOnce($value) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: CategoryInput,
            {
                source.validate()?;
                let values = $map(Some(&source), input, mapper, 1, None)?;
                self.fill_alpha_bins_from_data(source, lower, upper, facets, values)
            }
        }
    };
}
bin_mapping!(
    fill_alpha_bins_map,
    fill_alpha_bins_map_from_data,
    NumericInput,
    NumericValues,
    mapped_numeric
);
bin_mapping!(
    fill_alpha_bins_map_categories,
    fill_alpha_bins_map_categories_from_data,
    CategoryInput,
    CategoryValues,
    mapped_categories
);
