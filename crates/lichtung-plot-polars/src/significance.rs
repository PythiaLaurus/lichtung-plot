//! Independent caller-computed results become ordinary native annotation collections.
use crate::error::{binding, length, BindingResult};
use crate::input::sealed;
use crate::{
    AdapterError, Attached, ColumnInput, FacetCell, Pairwise, Significance, SignificanceMark,
    SignificanceTarget,
};
use polars_core::prelude::{Column, DataFrame, Series};

type Sequence<T> = (Vec<T>, bool);

/// Category, category/series, explicitly native targets or an explicit native scalar.
pub trait SignificanceTargets: sealed::Sealed {
    #[doc(hidden)]
    fn bind_targets(
        self,
        source: Option<&DataFrame>,
    ) -> BindingResult<Sequence<SignificanceTarget>>;
}
/// Supplied p-values, typed native marks or an explicitly mapped categorical variable.
pub trait SignificanceMarks: sealed::Sealed {
    #[doc(hidden)]
    fn bind_marks(self, source: Option<&DataFrame>) -> BindingResult<Sequence<SignificanceMark>>;
}
/// A single-dimensional field, crossed fields, or an explicit native facet cell.
pub trait SignificanceFacets: sealed::Sealed {
    #[doc(hidden)]
    fn bind_facets(self, source: Option<&DataFrame>) -> BindingResult<Sequence<FacetCell>>;
}

pub(crate) fn categories(
    input: impl ColumnInput,
    source: Option<&DataFrame>,
    role: &'static str,
) -> BindingResult<Vec<String>> {
    let column = input.bind_column(source, role)?;
    crate::category::values(&column)?
        .into_iter()
        .enumerate()
        .map(|(slot, value)| {
            value.ok_or_else(|| binding(role, format!("missing category at slot {slot}")))
        })
        .collect()
}

macro_rules! foreign_inputs {
    ($($ty:ty),+) => {$ (
        impl SignificanceTargets for $ty {
            fn bind_targets(self, source: Option<&DataFrame>) -> BindingResult<Sequence<SignificanceTarget>> { Ok((categories(self, source, "significance target")?.into_iter().map(SignificanceTarget::category).collect(), false)) }
        }
        impl SignificanceMarks for $ty {
            fn bind_marks(self, source: Option<&DataFrame>) -> BindingResult<Sequence<SignificanceMark>> {
                let column = self.bind_column(source, "significance p-values")?;
                let values = crate::numeric::values(&column)?.into_iter().enumerate().map(|(slot,value)| value.map(SignificanceMark::PValue).ok_or_else(|| binding("significance p-values", format!("missing value at slot {slot}")))).collect::<BindingResult<_>>()?;
                Ok((values, false))
            }
        }
        impl SignificanceFacets for $ty {
            fn bind_facets(self, source: Option<&DataFrame>) -> BindingResult<Sequence<FacetCell>> { Ok((categories(self, source, "significance facet")?.into_iter().map(FacetCell::single).collect(), false)) }
        }
    )+};
}
foreign_inputs!(&str, &String, &Column, &Series);

impl<C: ColumnInput, S: ColumnInput> sealed::Sealed for (C, S) {}
impl<C: ColumnInput, S: ColumnInput> SignificanceTargets for (C, S) {
    fn bind_targets(
        self,
        source: Option<&DataFrame>,
    ) -> BindingResult<Sequence<SignificanceTarget>> {
        let category = categories(self.0, source, "significance category")?;
        let series = categories(self.1, source, "significance series")?;
        length(
            "significance series",
            "series",
            category.len(),
            series.len(),
        )?;
        Ok((
            category
                .into_iter()
                .zip(series)
                .map(|(category, series)| SignificanceTarget::category_series(category, series))
                .collect(),
            false,
        ))
    }
}
impl<C: ColumnInput, S: ColumnInput> SignificanceFacets for (C, S) {
    fn bind_facets(self, source: Option<&DataFrame>) -> BindingResult<Sequence<FacetCell>> {
        let row = categories(self.0, source, "significance facet row")?;
        let column = categories(self.1, source, "significance facet column")?;
        length(
            "significance facet column",
            "column",
            row.len(),
            column.len(),
        )?;
        Ok((
            row.into_iter()
                .zip(column)
                .map(|(row, column)| FacetCell::grid(row, column))
                .collect(),
            false,
        ))
    }
}

fn mapped_marks<B, I, S>(
    input: B,
    mapping: I,
    source: Option<&DataFrame>,
) -> BindingResult<Sequence<SignificanceMark>>
where
    B: ColumnInput,
    I: IntoIterator<Item = (S, SignificanceMark)>,
    S: AsRef<str>,
{
    let mapping = mapping
        .into_iter()
        .map(|(key, mark)| (key.as_ref().to_owned(), mark))
        .collect::<Vec<_>>();
    let values = categories(input, source, "significance marks")?
        .into_iter()
        .map(|value| {
            let mut matches = mapping.iter().filter(|(key, _)| key == &value);
            let mark = matches
                .next()
                .ok_or_else(|| binding("significance marks", format!("unmapped key {value:?}")))?
                .1;
            if matches.next().is_some() {
                return Err(binding(
                    "significance marks",
                    format!("duplicate map key {value:?}"),
                ));
            }
            Ok(mark)
        })
        .collect::<BindingResult<_>>()?;
    Ok((values, false))
}

macro_rules! mapped_arrays {
    ($ty:ty) => {
        impl<B: ColumnInput, const N: usize> sealed::Sealed for (B, [($ty, SignificanceMark); N]) {}
        impl<B: ColumnInput, const N: usize> SignificanceMarks
            for (B, [($ty, SignificanceMark); N])
        {
            fn bind_marks(
                self,
                source: Option<&DataFrame>,
            ) -> BindingResult<Sequence<SignificanceMark>> {
                mapped_marks(self.0, self.1, source)
            }
        }
    };
}
mapped_arrays!(&str);
mapped_arrays!(String);

impl<B: ColumnInput, S: AsRef<str>> sealed::Sealed for (B, Vec<(S, SignificanceMark)>) {}
impl<B: ColumnInput, S: AsRef<str>> SignificanceMarks for (B, Vec<(S, SignificanceMark)>) {
    fn bind_marks(self, source: Option<&DataFrame>) -> BindingResult<Sequence<SignificanceMark>> {
        mapped_marks(self.0, self.1, source)
    }
}
impl<B: ColumnInput, S: AsRef<str>> sealed::Sealed for (B, &[(S, SignificanceMark)]) {}
impl<B: ColumnInput, S: AsRef<str>> SignificanceMarks for (B, &[(S, SignificanceMark)]) {
    fn bind_marks(self, source: Option<&DataFrame>) -> BindingResult<Sequence<SignificanceMark>> {
        mapped_marks(
            self.0,
            self.1.iter().map(|(key, mark)| (key, *mark)),
            source,
        )
    }
}

macro_rules! native_sequence {
    ($ty:ty, $trait:ident, $method:ident) => {
        impl sealed::Sealed for $ty {}
        impl $trait for $ty {
            fn $method(self, _: Option<&DataFrame>) -> BindingResult<Sequence<$ty>> {
                Ok((vec![self], true))
            }
        }
        impl sealed::Sealed for Vec<$ty> {}
        impl $trait for Vec<$ty> {
            fn $method(self, _: Option<&DataFrame>) -> BindingResult<Sequence<$ty>> {
                Ok((self, false))
            }
        }
        impl sealed::Sealed for &[$ty] {}
        impl $trait for &[$ty] {
            fn $method(self, _: Option<&DataFrame>) -> BindingResult<Sequence<$ty>> {
                Ok((self.to_vec(), false))
            }
        }
        impl<const N: usize> sealed::Sealed for [$ty; N] {}
        impl<const N: usize> $trait for [$ty; N] {
            fn $method(self, _: Option<&DataFrame>) -> BindingResult<Sequence<$ty>> {
                Ok((self.into(), false))
            }
        }
    };
}
native_sequence!(SignificanceTarget, SignificanceTargets, bind_targets);
native_sequence!(SignificanceMark, SignificanceMarks, bind_marks);
native_sequence!(FacetCell, SignificanceFacets, bind_facets);

fn count(parts: &[(usize, bool)]) -> BindingResult<usize> {
    let len = parts
        .iter()
        .find(|(_, constant)| !constant)
        .map_or(1, |(len, _)| *len);
    for &(actual, constant) in parts {
        if !constant {
            length("significance", "result variable", len, actual)?;
        }
    }
    Ok(len)
}
fn member<T: Clone>(values: &Sequence<T>, row: usize) -> T {
    values.0[if values.1 { 0 } else { row }].clone()
}

pub(crate) fn pairwise(
    source: Option<&DataFrame>,
    left: impl SignificanceTargets,
    right: impl SignificanceTargets,
    marks: impl SignificanceMarks,
    facets: Option<&Sequence<FacetCell>>,
) -> BindingResult<Vec<Significance<Pairwise>>> {
    let left = left.bind_targets(source)?;
    let right = right.bind_targets(source)?;
    let marks = marks.bind_marks(source)?;
    let mut sizes = vec![
        (left.0.len(), left.1),
        (right.0.len(), right.1),
        (marks.0.len(), marks.1),
    ];
    if let Some(facets) = &facets {
        sizes.push((facets.0.len(), facets.1));
    }
    let len = count(&sizes)?;
    let mut result = crate::snapshot::allocate(len, "significance")?;
    for row in 0..len {
        let mut value =
            Significance::between(member(&left, row), member(&right, row), member(&marks, row));
        if let Some(facets) = &facets {
            value = value.facet(member(facets, row));
        }
        result.push(value);
    }
    Ok(result)
}
pub(crate) fn attached(
    source: Option<&DataFrame>,
    target: impl SignificanceTargets,
    marks: impl SignificanceMarks,
    facets: Option<&Sequence<FacetCell>>,
) -> BindingResult<Vec<Significance<Attached>>> {
    let target = target.bind_targets(source)?;
    let marks = marks.bind_marks(source)?;
    let mut sizes = vec![(target.0.len(), target.1), (marks.0.len(), marks.1)];
    if let Some(facets) = &facets {
        sizes.push((facets.0.len(), facets.1));
    }
    let len = count(&sizes)?;
    let mut result = crate::snapshot::allocate(len, "significance")?;
    for row in 0..len {
        let mut value = Significance::at(member(&target, row), member(&marks, row));
        if let Some(facets) = &facets {
            value = value.facet(member(facets, row));
        }
        result.push(value);
    }
    Ok(result)
}

macro_rules! pairwise_api {
    ($name:ident, $from:ident, ($($arg:ident:$ty:ident),*), $facets:expr) => {
        /// Prepare reusable native pairwise annotations from direct result variables.
        pub fn $name(left: impl SignificanceTargets, right: impl SignificanceTargets, marks: impl SignificanceMarks, $($arg: impl $ty),*) -> std::result::Result<Vec<Significance<Pairwise>>,AdapterError> { let source=None; pairwise(source,left,right,marks,($facets)(source,$($arg),*)?.as_ref()) }
        /// Prepare reusable native pairwise annotations using an independent result source.
        pub fn $from(source: &DataFrame, left: impl SignificanceTargets, right: impl SignificanceTargets, marks: impl SignificanceMarks, $($arg: impl $ty),*) -> std::result::Result<Vec<Significance<Pairwise>>,AdapterError> { let source=Some(source); pairwise(source,left,right,marks,($facets)(source,$($arg),*)?.as_ref()) }
        impl<K: crate::kind::Significance<Pairwise>> crate::PlotAdapter<'_,K> {
            /// Bind pairwise results and append their native annotations in supplied order.
            pub fn $name(self,left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*) -> std::result::Result<Self,AdapterError> { let values=pairwise(self.source,left,right,marks,($facets)(self.source,$($arg),*)?.as_ref())?; Ok(self.significance(values)) }
            /// Append pairwise annotations from an independently bound result table.
            pub fn $from(self,source:&DataFrame,left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*) -> std::result::Result<Self,AdapterError> { Ok(self.significance($from(source,left,right,marks,$($arg),*)?)) }
        }
    };
}
#[allow(
    clippy::unnecessary_wraps,
    reason = "uniform fallible facet binder used by the API macros"
)]
fn no_facet(_: Option<&DataFrame>) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(None)
}
fn one_facet(
    source: Option<&DataFrame>,
    facet: impl SignificanceFacets,
) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(Some(facet.bind_facets(source)?))
}
fn grid_facet(
    source: Option<&DataFrame>,
    row: impl ColumnInput,
    column: impl ColumnInput,
) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(Some((row, column).bind_facets(source)?))
}
pairwise_api!(
    significance_pairwise,
    significance_pairwise_from_data,
    (),
    no_facet
);
pairwise_api!(significance_pairwise_in_facet,significance_pairwise_in_facet_from_data,(facet:SignificanceFacets),one_facet);
pairwise_api!(significance_pairwise_in_grid,significance_pairwise_in_grid_from_data,(row:ColumnInput,column:ColumnInput),grid_facet);

macro_rules! attached_api {
    ($name:ident, $from:ident, ($($arg:ident:$ty:ident),*), $facets:expr) => {
        /// Prepare reusable native attached annotations from direct result variables.
        pub fn $name(target: impl SignificanceTargets, marks: impl SignificanceMarks, $($arg: impl $ty),*) -> std::result::Result<Vec<Significance<Attached>>,AdapterError> { let source=None; attached(source,target,marks,($facets)(source,$($arg),*)?.as_ref()) }
        /// Prepare reusable native attached annotations from an independent result source.
        pub fn $from(source: &DataFrame, target: impl SignificanceTargets, marks: impl SignificanceMarks, $($arg: impl $ty),*) -> std::result::Result<Vec<Significance<Attached>>,AdapterError> { let source=Some(source); attached(source,target,marks,($facets)(source,$($arg),*)?.as_ref()) }
        impl<K: crate::kind::Significance<Attached>> crate::PlotAdapter<'_,K> {
            /// Bind attached results and append their native annotations in supplied order.
            pub fn $name(self,target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*) -> std::result::Result<Self,AdapterError> { let values=attached(self.source,target,marks,($facets)(self.source,$($arg),*)?.as_ref())?; Ok(self.significance(values)) }
            /// Append attached annotations from an independently bound result table.
            pub fn $from(self,source:&DataFrame,target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*) -> std::result::Result<Self,AdapterError> { Ok(self.significance($from(source,target,marks,$($arg),*)?)) }
        }
    };
}
attached_api!(
    significance_attached,
    significance_attached_from_data,
    (),
    no_facet
);

#[cfg(test)]
mod tests {
    use crate::{FacetCell, Significance, SignificanceMark, SignificanceTarget};
    use polars_core::prelude::*;

    #[test]
    fn independent_results_preserve_typed_series_facets_scalars_order_and_native_validation_timing()
    {
        let results = df!("left" => ["A", "B"], "right" => ["B", "C"], "series" => ["drug", "drug"], "p" => [0.01, 2.0], "row" => ["first", "second"], "column" => ["east", "west"], "mark" => ["yes", "no"]).unwrap();
        let bound = crate::significance_pairwise_in_grid_from_data(
            &results,
            ("left", "series"),
            "right",
            "p",
            "row",
            "column",
        )
        .unwrap();
        assert_eq!(
            bound,
            vec![
                Significance::between(
                    SignificanceTarget::category_series("A", "drug"),
                    "B",
                    SignificanceMark::PValue(0.01)
                )
                .facet(FacetCell::grid("first", "east")),
                Significance::between(
                    SignificanceTarget::category_series("B", "drug"),
                    "C",
                    SignificanceMark::PValue(2.0)
                )
                .facet(FacetCell::grid("second", "west")),
            ]
        );
        let attached = crate::significance_attached_in_facet_from_data(
            &results,
            SignificanceTarget::category("A"),
            (
                "mark",
                [
                    ("yes", SignificanceMark::PValue(0.01)),
                    ("no", SignificanceMark::PValue(0.5)),
                ],
            ),
            FacetCell::single("pooled"),
        )
        .unwrap();
        assert_eq!(attached.len(), 2);
        assert_eq!(
            attached[1],
            Significance::at("A", SignificanceMark::PValue(0.5)).facet(FacetCell::single("pooled"))
        );
        assert!(
            crate::significance_pairwise("left", "right", SignificanceMark::PValue(0.1)).is_err()
        );
        let short = Series::new("short".into(), [0.1]);
        assert!(crate::significance_pairwise_from_data(&results, "left", "right", &short).is_err());
        drop(results);
        assert_eq!(bound.len(), 2); // No independent result-table borrow survives binding.
    }
}
attached_api!(significance_attached_in_facet,significance_attached_in_facet_from_data,(facet:SignificanceFacets),one_facet);
attached_api!(significance_attached_in_grid,significance_attached_in_grid_from_data,(row:ColumnInput,column:ColumnInput),grid_facet);
