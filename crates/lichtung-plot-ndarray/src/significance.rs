//! Caller-computed result variables become reusable native annotation collections.
use crate::error::{binding, length, BindingResult};
use crate::input::sealed;
use crate::{
    AdapterError, ArrayElement, ArraySource, Attached, CategoryElement, CategoryInput, FacetCell,
    NumericElement, NumericInput, Pairwise, Significance, SignificanceMark, SignificanceTarget,
};
use ndarray::ArrayView1;

type Sequence<T> = (Vec<T>, bool);
/// Category variables, category/series pairs or explicitly native target scalars/sequences.
pub trait SignificanceTargets: sealed::Sealed {
    #[doc(hidden)]
    fn bind_targets(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceTarget>>;
}
/// Supplied p-values, typed native marks or explicitly mapped category variables.
pub trait SignificanceMarks: sealed::Sealed {
    #[doc(hidden)]
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>>;
}
/// A category variable, crossed category variables or explicit native facet cells.
pub trait SignificanceFacets: sealed::Sealed {
    #[doc(hidden)]
    fn bind_facets(self, source: Option<&dyn ArraySource>) -> BindingResult<Sequence<FacetCell>>;
}
fn categories(
    input: impl CategoryInput,
    source: Option<&dyn ArraySource>,
    role: &'static str,
) -> BindingResult<Vec<String>> {
    input
        .raw_category(source, role)?
        .1
        .into_iter()
        .enumerate()
        .map(|(slot, value)| {
            value.ok_or_else(|| binding(role, format!("missing category at slot {slot}")))
        })
        .collect()
}
fn p_values(
    input: impl NumericInput,
    source: Option<&dyn ArraySource>,
) -> BindingResult<Sequence<SignificanceMark>> {
    let values = input
        .raw_numeric(source, "significance p-values")?
        .1
        .into_iter()
        .enumerate()
        .map(|(slot, value)| {
            value.map(SignificanceMark::PValue).ok_or_else(|| {
                binding(
                    "significance p-values",
                    format!("missing value at slot {slot}"),
                )
            })
        })
        .collect::<BindingResult<_>>()?;
    Ok((values, false))
}
impl SignificanceTargets for usize {
    fn bind_targets(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceTarget>> {
        Ok((
            categories(self, source, "significance target")?
                .into_iter()
                .map(SignificanceTarget::category)
                .collect(),
            false,
        ))
    }
}
impl SignificanceMarks for usize {
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        p_values(self, source)
    }
}
impl SignificanceFacets for usize {
    fn bind_facets(self, source: Option<&dyn ArraySource>) -> BindingResult<Sequence<FacetCell>> {
        Ok((
            categories(self, source, "significance facet")?
                .into_iter()
                .map(FacetCell::single)
                .collect(),
            false,
        ))
    }
}
macro_rules! foreign_targets {
    ($ty:ty) => {
        impl<T: ArrayElement + CategoryElement> SignificanceTargets for $ty {
            fn bind_targets(
                self,
                source: Option<&dyn ArraySource>,
            ) -> BindingResult<Sequence<SignificanceTarget>> {
                Ok((
                    categories(self, source, "significance target")?
                        .into_iter()
                        .map(SignificanceTarget::category)
                        .collect(),
                    false,
                ))
            }
        }
        impl<T: ArrayElement + CategoryElement> SignificanceFacets for $ty {
            fn bind_facets(
                self,
                source: Option<&dyn ArraySource>,
            ) -> BindingResult<Sequence<FacetCell>> {
                Ok((
                    categories(self, source, "significance facet")?
                        .into_iter()
                        .map(FacetCell::single)
                        .collect(),
                    false,
                ))
            }
        }
    };
}
foreign_targets!(ArrayView1<'_, T>);
foreign_targets!((ArrayView1<'_, T>, ArrayView1<'_, bool>));
impl<T: ArrayElement + NumericElement> SignificanceMarks for ArrayView1<'_, T> {
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        p_values(self, source)
    }
}
impl<T: ArrayElement + NumericElement> SignificanceMarks
    for (ArrayView1<'_, T>, ArrayView1<'_, bool>)
{
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        p_values(self, source)
    }
}

impl<C: CategoryInput, S: CategoryInput> sealed::Sealed for (C, S) {}
impl<C: CategoryInput, S: CategoryInput> SignificanceTargets for (C, S) {
    fn bind_targets(
        self,
        source: Option<&dyn ArraySource>,
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
impl<C: CategoryInput, S: CategoryInput> SignificanceFacets for (C, S) {
    fn bind_facets(self, source: Option<&dyn ArraySource>) -> BindingResult<Sequence<FacetCell>> {
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
    source: Option<&dyn ArraySource>,
) -> BindingResult<Sequence<SignificanceMark>>
where
    B: CategoryInput,
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
impl<B: CategoryInput, S: AsRef<str>, const N: usize> sealed::Sealed
    for (B, [(S, SignificanceMark); N])
{
}
impl<B: CategoryInput, S: AsRef<str>, const N: usize> SignificanceMarks
    for (B, [(S, SignificanceMark); N])
{
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        mapped_marks(self.0, self.1, source)
    }
}
impl<B: CategoryInput, S: AsRef<str>> sealed::Sealed for (B, Vec<(S, SignificanceMark)>) {}
impl<B: CategoryInput, S: AsRef<str>> SignificanceMarks for (B, Vec<(S, SignificanceMark)>) {
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        mapped_marks(self.0, self.1, source)
    }
}
impl<B: CategoryInput, S: AsRef<str>> sealed::Sealed for (B, &[(S, SignificanceMark)]) {}
impl<B: CategoryInput, S: AsRef<str>> SignificanceMarks for (B, &[(S, SignificanceMark)]) {
    fn bind_marks(
        self,
        source: Option<&dyn ArraySource>,
    ) -> BindingResult<Sequence<SignificanceMark>> {
        mapped_marks(
            self.0,
            self.1.iter().map(|(key, mark)| (key, *mark)),
            source,
        )
    }
}

macro_rules! native_sequence {
    ($ty:ty,$trait:ident,$method:ident) => {
        impl sealed::Sealed for $ty {}
        impl $trait for $ty {
            fn $method(self, _: Option<&dyn ArraySource>) -> BindingResult<Sequence<$ty>> {
                Ok((vec![self], true))
            }
        }
        impl sealed::Sealed for Vec<$ty> {}
        impl $trait for Vec<$ty> {
            fn $method(self, _: Option<&dyn ArraySource>) -> BindingResult<Sequence<$ty>> {
                Ok((self, false))
            }
        }
        impl sealed::Sealed for &[$ty] {}
        impl $trait for &[$ty] {
            fn $method(self, _: Option<&dyn ArraySource>) -> BindingResult<Sequence<$ty>> {
                Ok((self.to_vec(), false))
            }
        }
        impl<const N: usize> sealed::Sealed for [$ty; N] {}
        impl<const N: usize> $trait for [$ty; N] {
            fn $method(self, _: Option<&dyn ArraySource>) -> BindingResult<Sequence<$ty>> {
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
fn pairwise(
    source: Option<&dyn ArraySource>,
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
    if let Some(facets) = facets {
        sizes.push((facets.0.len(), facets.1));
    }
    let len = count(&sizes)?;
    let mut result = crate::snapshot::allocate(len, "significance")?;
    for row in 0..len {
        let mut value =
            Significance::between(member(&left, row), member(&right, row), member(&marks, row));
        if let Some(facets) = facets {
            value = value.facet(member(facets, row));
        }
        result.push(value);
    }
    Ok(result)
}
fn attached(
    source: Option<&dyn ArraySource>,
    target: impl SignificanceTargets,
    marks: impl SignificanceMarks,
    facets: Option<&Sequence<FacetCell>>,
) -> BindingResult<Vec<Significance<Attached>>> {
    let target = target.bind_targets(source)?;
    let marks = marks.bind_marks(source)?;
    let mut sizes = vec![(target.0.len(), target.1), (marks.0.len(), marks.1)];
    if let Some(facets) = facets {
        sizes.push((facets.0.len(), facets.1));
    }
    let len = count(&sizes)?;
    let mut result = crate::snapshot::allocate(len, "significance")?;
    for row in 0..len {
        let mut value = Significance::at(member(&target, row), member(&marks, row));
        if let Some(facets) = facets {
            value = value.facet(member(facets, row));
        }
        result.push(value);
    }
    Ok(result)
}
#[allow(
    clippy::unnecessary_wraps,
    reason = "uniform fallible facet binding in API macros"
)]
fn no_facet(_: Option<&dyn ArraySource>) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(None)
}
fn one_facet(
    source: Option<&dyn ArraySource>,
    facet: impl SignificanceFacets,
) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(Some(facet.bind_facets(source)?))
}
fn grid_facet(
    source: Option<&dyn ArraySource>,
    row: impl CategoryInput,
    column: impl CategoryInput,
) -> BindingResult<Option<Sequence<FacetCell>>> {
    Ok(Some((row, column).bind_facets(source)?))
}
macro_rules! pairwise_api {
    ($name:ident,$from:ident,($($arg:ident:$ty:ident),*),$facets:expr) => {
        /// Prepare reusable native pairwise annotations from direct result variables.
        pub fn $name(left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Vec<Significance<Pairwise>>,AdapterError> {let source=None;pairwise(source,left,right,marks,($facets)(source,$($arg),*)?.as_ref())}
        /// Prepare reusable native pairwise annotations using an independent array source.
        pub fn $from(source:impl ArraySource,left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Vec<Significance<Pairwise>>,AdapterError> {source.validate()?;let source=Some(&source as &dyn ArraySource);pairwise(source,left,right,marks,($facets)(source,$($arg),*)?.as_ref())}
        impl<K:crate::kind::Significance<Pairwise>> crate::PlotAdapter<'_,K> {
            /// Append pairwise results in supplied order, independent of primary observation count.
            pub fn $name(self,left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Self,AdapterError> {let source=self.source.as_deref();let values=pairwise(source,left,right,marks,($facets)(source,$($arg),*)?.as_ref())?;Ok(self.significance(values))}
            /// Append pairwise results from an independently bound array source.
            pub fn $from(self,source:impl ArraySource,left:impl SignificanceTargets,right:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Self,AdapterError> {Ok(self.significance($from(source,left,right,marks,$($arg),*)?))}
        }
    };
}
pairwise_api!(
    significance_pairwise,
    significance_pairwise_from_data,
    (),
    no_facet
);
pairwise_api!(significance_pairwise_in_facet,significance_pairwise_in_facet_from_data,(facet:SignificanceFacets),one_facet);
pairwise_api!(significance_pairwise_in_grid,significance_pairwise_in_grid_from_data,(row:CategoryInput,column:CategoryInput),grid_facet);
macro_rules! attached_api {
    ($name:ident,$from:ident,($($arg:ident:$ty:ident),*),$facets:expr) => {
        /// Prepare reusable native attached annotations from direct result variables.
        pub fn $name(target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Vec<Significance<Attached>>,AdapterError> {let source=None;attached(source,target,marks,($facets)(source,$($arg),*)?.as_ref())}
        /// Prepare reusable native attached annotations using an independent array source.
        pub fn $from(source:impl ArraySource,target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Vec<Significance<Attached>>,AdapterError> {source.validate()?;let source=Some(&source as &dyn ArraySource);attached(source,target,marks,($facets)(source,$($arg),*)?.as_ref())}
        impl<K:crate::kind::Significance<Attached>> crate::PlotAdapter<'_,K> {
            /// Append attached results in supplied order with their native placement type.
            pub fn $name(self,target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Self,AdapterError> {let source=self.source.as_deref();let values=attached(source,target,marks,($facets)(source,$($arg),*)?.as_ref())?;Ok(self.significance(values))}
            /// Append attached results from an independently bound array source.
            pub fn $from(self,source:impl ArraySource,target:impl SignificanceTargets,marks:impl SignificanceMarks,$($arg:impl $ty),*)->std::result::Result<Self,AdapterError> {Ok(self.significance($from(source,target,marks,$($arg),*)?))}
        }
    };
}
attached_api!(
    significance_attached,
    significance_attached_from_data,
    (),
    no_facet
);
attached_api!(significance_attached_in_facet,significance_attached_in_facet_from_data,(facet:SignificanceFacets),one_facet);
attached_api!(significance_attached_in_grid,significance_attached_in_grid_from_data,(row:CategoryInput,column:CategoryInput),grid_facet);
