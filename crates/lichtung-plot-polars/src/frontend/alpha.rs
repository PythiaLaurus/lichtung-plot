use super::PlotAdapter;
use crate::error::{binding, length, BindingResult};
use crate::{
    kind, AdapterError, AlphaEncoding, AlphaInput, AlphaValues, AlphaValuesInput, Bounds,
    CategoryInput, CategoryValues, ColumnInput, NumericInput, NumericValues,
};
use polars_core::prelude::DataFrame;

impl<K: kind::Marker> PlotAdapter<'_, K> {
    fn mapped_numeric(
        &self,
        input: impl NumericInput,
        mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
        copies: usize,
        expected: Option<usize>,
    ) -> BindingResult<AlphaValues> {
        let copies = if input.native_slots() { 1 } else { copies };
        let values = input.bind_numeric(self.source, "alpha map")?;
        let len = values.len();
        let values = mapper(values)?;
        length("alpha mapper output", values.name(), len, values.len())?;
        crate::alpha::expand(values, expected, copies)
    }
    fn mapped_categories(
        &self,
        input: impl CategoryInput,
        mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
        copies: usize,
        expected: Option<usize>,
    ) -> BindingResult<AlphaValues> {
        let copies = if input.native_slots() { 1 } else { copies };
        let values = input.bind_category(self.source, "alpha map")?;
        let len = values.len();
        let values = mapper(values)?;
        length("alpha mapper output", values.name(), len, values.len())?;
        crate::alpha::expand(values, expected, copies)
    }
}

macro_rules! alpha_target {
    ($cap:ident, $base:ident, $cells:ident, $linear:ident, $map:ident, $categories:ident,
        $cell_map:ident, $cell_categories:ident, $single:ident, $single_map:ident, $single_categories:ident,
        $keyed:ident, $keyed_data:ident, $keyed_map:ident, $keyed_map_data:ident, $keyed_categories:ident, $keyed_categories_data:ident,
        $columns:ident, $columns_data:ident, $columns_map:ident, $columns_map_data:ident, $columns_categories:ident, $columns_categories_data:ident) => {
        impl<K: kind::$cap> PlotAdapter<'_, K> {
            /// Bind observation bytes or forward an existing native encoding without reinterpreting it.
            pub fn $base(
                mut self,
                input: impl AlphaInput,
            ) -> std::result::Result<Self, AdapterError> {
                let encoding = input.bind_alpha(self.source, self.len, self.row_copies())?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Bind already-expanded observation bytes, never repeating source rows.
            pub fn $cells(
                mut self,
                input: impl AlphaInput,
            ) -> std::result::Result<Self, AdapterError> {
                let encoding = input.bind_alpha(self.source, self.len, 1)?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Apply the native linear declaration to a numeric observation variable.
            pub fn $linear(
                self,
                input: impl NumericInput,
                domain: (f64, f64),
                range: (u8, u8),
            ) -> std::result::Result<Self, AdapterError> {
                self.$map(input, |values| AlphaValues::linear(values, domain, range))
            }
            /// Resolve a numeric input and materialize its native alpha map once.
            pub fn $map(
                mut self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_numeric(input, mapper, self.row_copies(), self.len)?;
                self.plot = self.plot.$base(AlphaEncoding::observations(values));
                Ok(self)
            }
            /// Materialize a native categorical map once before row expansion.
            pub fn $categories(
                mut self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_categories(input, mapper, self.row_copies(), self.len)?;
                self.plot = self.plot.$base(AlphaEncoding::observations(values));
                Ok(self)
            }
            /// Map explicitly expanded numeric slots with the native factory.
            pub fn $cell_map(
                mut self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_numeric(input, mapper, 1, self.len)?;
                self.plot = self.plot.$base(AlphaEncoding::observations(values));
                Ok(self)
            }
            /// Map explicitly expanded categorical slots with the native factory.
            pub fn $cell_categories(
                mut self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_categories(input, mapper, 1, self.len)?;
                self.plot = self.plot.$base(AlphaEncoding::observations(values));
                Ok(self)
            }
            /// Explicitly broadcast one exact byte or one bound native alpha value.
            pub fn $single(
                mut self,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError> {
                self.plot = self.plot.$base(AlphaEncoding::single(
                    input.bind_alpha_values(self.source)?,
                )?);
                Ok(self)
            }
            /// Map exactly one numeric source member and broadcast its native declaration.
            pub fn $single_map(
                mut self,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_numeric(input, mapper, 1, Some(1))?;
                self.plot = self.plot.$base(AlphaEncoding::single(values)?);
                Ok(self)
            }
            /// Map exactly one categorical source member and broadcast its declaration.
            pub fn $single_categories(
                mut self,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError> {
                let values = self.mapped_categories(input, mapper, 1, Some(1))?;
                self.plot = self.plot.$base(AlphaEncoding::single(values)?);
                Ok(self)
            }
            /// Bind exact series/aggregate selectors in the plot's source; never reduce observations.
            pub fn $keyed<I, S, B>(
                mut self,
                keys: I,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let encoding = AlphaEncoding::keyed(
                    crate::alpha::keys(self.source, keys)?,
                    input.bind_alpha_values(self.source)?,
                )?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Bind an independent selector table eagerly, retaining no borrow of that table.
            pub fn $keyed_data<I, S, B>(
                mut self,
                source: &DataFrame,
                keys: I,
                input: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let encoding = AlphaEncoding::keyed(
                    crate::alpha::keys(Some(source), keys)?,
                    input.bind_alpha_values(Some(source))?,
                )?;
                self.plot = self.plot.$base(encoding);
                Ok(self)
            }
            /// Bind numeric mapped alpha to exact actual series/aggregate keys.
            pub fn $keyed_map<I, S, B>(
                mut self,
                keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let values = self.mapped_numeric(input, mapper, 1, None)?;
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    crate::alpha::keys(self.source, keys)?,
                    values,
                )?);
                Ok(self)
            }
            /// Map numeric alpha in an independent selector source, without retaining it.
            pub fn $keyed_map_data<I, S, B>(
                mut self,
                source: &DataFrame,
                keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let raw = input.bind_numeric(Some(source), "alpha map")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    crate::alpha::keys(Some(source), keys)?,
                    values,
                )?);
                Ok(self)
            }
            /// Bind categorical mapped alpha to exact series/aggregate keys.
            pub fn $keyed_categories<I, S, B>(
                mut self,
                keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let values = self.mapped_categories(input, mapper, 1, None)?;
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    crate::alpha::keys(self.source, keys)?,
                    values,
                )?);
                Ok(self)
            }
            /// Materialize categorical alpha in an independent selector source.
            pub fn $keyed_categories_data<I, S, B>(
                mut self,
                source: &DataFrame,
                keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let raw = input.bind_category(Some(source), "alpha map")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    crate::alpha::keys(Some(source), keys)?,
                    values,
                )?);
                Ok(self)
            }
            /// Key alpha to the generated variable dimension and any actual additional keys.
            pub fn $columns<I, S, B>(
                mut self,
                column_key: impl ColumnInput,
                other_keys: I,
                values: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let expansion = self
                    .expansion
                    .as_ref()
                    .ok_or_else(|| binding("column alpha", "no generated variable dimension"))?;
                let mut keys = crate::alpha::keys(self.source, other_keys)?;
                keys.push(expansion.column_key(self.source, column_key)?);
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    keys,
                    values.bind_alpha_values(self.source)?,
                )?);
                Ok(self)
            }
            /// Resolve generated-variable alpha from an independent table at this call.
            pub fn $columns_data<I, S, B>(
                mut self,
                source: &DataFrame,
                column_key: impl ColumnInput,
                other_keys: I,
                values: impl AlphaValuesInput,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let expansion = self
                    .expansion
                    .as_ref()
                    .ok_or_else(|| binding("column alpha", "no generated variable dimension"))?;
                let mut keys = crate::alpha::keys(Some(source), other_keys)?;
                keys.push(expansion.column_key(Some(source), column_key)?);
                self.plot = self.plot.$base(AlphaEncoding::keyed(
                    keys,
                    values.bind_alpha_values(Some(source))?,
                )?);
                Ok(self)
            }
            /// Materialize numeric alpha for generated-column selectors.
            pub fn $columns_map<I, S, B>(
                self,
                column_key: impl ColumnInput,
                other_keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let values = self.mapped_numeric(input, mapper, 1, None)?;
                self.$columns(column_key, other_keys, values)
            }
            /// Map generated-column alpha in an independent selector source.
            pub fn $columns_map_data<I, S, B>(
                self,
                source: &DataFrame,
                column_key: impl ColumnInput,
                other_keys: I,
                input: impl NumericInput,
                mapper: impl FnOnce(NumericValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let raw = input.bind_numeric(Some(source), "alpha map")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
                self.$columns_data(source, column_key, other_keys, values)
            }
            /// Materialize categorical alpha for generated-column selectors.
            pub fn $columns_categories<I, S, B>(
                self,
                column_key: impl ColumnInput,
                other_keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let values = self.mapped_categories(input, mapper, 1, None)?;
                self.$columns(column_key, other_keys, values)
            }
            /// Map categorical generated-column alpha in an independent source.
            pub fn $columns_categories_data<I, S, B>(
                self,
                source: &DataFrame,
                column_key: impl ColumnInput,
                other_keys: I,
                input: impl CategoryInput,
                mapper: impl FnOnce(CategoryValues) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let raw = input.bind_category(Some(source), "alpha map")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
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
    /// Bind exact bin bounds, optional facet keys and u8 alpha without sample reduction.
    pub fn fill_alpha_bins<I, S, B>(
        mut self,
        lower: impl NumericInput,
        upper: impl NumericInput,
        facets: I,
        values: impl AlphaValuesInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, B)>,
        S: AsRef<str>,
        B: ColumnInput,
    {
        let bounds = Bounds::new(
            lower.bind_numeric(self.source, "bin lower")?,
            upper.bind_numeric(self.source, "bin upper")?,
        );
        self.plot = self.plot.fill_alpha(AlphaEncoding::bins(
            bounds,
            crate::alpha::keys(self.source, facets)?,
            values.bind_alpha_values(self.source)?,
        )?);
        Ok(self)
    }
    /// Bind exact bin alpha from an independent result table, retaining no foreign table.
    pub fn fill_alpha_bins_from_data<I, S, B>(
        mut self,
        source: &DataFrame,
        lower: impl NumericInput,
        upper: impl NumericInput,
        facets: I,
        values: impl AlphaValuesInput,
    ) -> std::result::Result<Self, AdapterError>
    where
        I: IntoIterator<Item = (S, B)>,
        S: AsRef<str>,
        B: ColumnInput,
    {
        let bounds = Bounds::new(
            lower.bind_numeric(Some(source), "bin lower")?,
            upper.bind_numeric(Some(source), "bin upper")?,
        );
        self.plot = self.plot.fill_alpha(AlphaEncoding::bins(
            bounds,
            crate::alpha::keys(Some(source), facets)?,
            values.bind_alpha_values(Some(source))?,
        )?);
        Ok(self)
    }
}

macro_rules! bin_mapping {
    ($name:ident, $from:ident, $input:ident, $value:ty, $bind:ident) => {
        impl PlotAdapter<'_, kind::Histogram> {
            /// Map alpha to explicit native bins without reducing sample rows.
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
                B: ColumnInput,
            {
                let raw = input.$bind(self.source, "bin alpha")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
                self.fill_alpha_bins(lower, upper, facets, values)
            }
            /// Materialize a bin mapping in an independent result source.
            pub fn $from<I, S, B>(
                self,
                source: &DataFrame,
                lower: impl NumericInput,
                upper: impl NumericInput,
                facets: I,
                input: impl $input,
                mapper: impl FnOnce($value) -> crate::Result<AlphaValues>,
            ) -> std::result::Result<Self, AdapterError>
            where
                I: IntoIterator<Item = (S, B)>,
                S: AsRef<str>,
                B: ColumnInput,
            {
                let raw = input.$bind(Some(source), "bin alpha")?;
                let len = raw.len();
                let values = mapper(raw)?;
                length("alpha mapper output", values.name(), len, values.len())?;
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
    bind_numeric
);
bin_mapping!(
    fill_alpha_bins_map_categories,
    fill_alpha_bins_map_categories_from_data,
    CategoryInput,
    CategoryValues,
    bind_category
);
