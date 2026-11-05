//! Exact bytes and native mapping declarations, snapshotted at each binding.
use crate::error::{binding, BindingResult};
use crate::input::sealed;
use crate::{AlphaEncoding, AlphaValues, ArrayElement, ArraySource, CategoryInput, CategoryValues};
use ndarray::ArrayView1;

impl sealed::Sealed for AlphaEncoding {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for AlphaValues {
    fn native_slots(&self) -> bool {
        true
    }
}
impl sealed::Sealed for u8 {}
/// Exact non-null u8 variables or already-materialized native alpha values.
pub trait AlphaValuesInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_alpha_values(self, source: Option<&dyn ArraySource>) -> BindingResult<AlphaValues>;
}
impl AlphaValuesInput for AlphaValues {
    fn bind_alpha_values(self, _: Option<&dyn ArraySource>) -> BindingResult<AlphaValues> {
        Ok(self)
    }
}
impl AlphaValuesInput for u8 {
    fn bind_alpha_values(self, _: Option<&dyn ArraySource>) -> BindingResult<AlphaValues> {
        Ok(AlphaValues::new("Alpha", [self])?)
    }
}
impl AlphaValuesInput for usize {
    fn bind_alpha_values(self, source: Option<&dyn ArraySource>) -> BindingResult<AlphaValues> {
        Ok(AlphaValues::new(
            crate::source::name(self),
            crate::source::lookup(source, self, "alpha bytes")?.byte_column(self, "alpha bytes")?,
        )?)
    }
}
impl<T: ArrayElement> AlphaValuesInput for ArrayView1<'_, T> {
    fn bind_alpha_values(self, _: Option<&dyn ArraySource>) -> BindingResult<AlphaValues> {
        Ok(AlphaValues::new(
            "Alpha",
            crate::input::bytes(self, None, "alpha bytes")?,
        )?)
    }
}
impl<T: ArrayElement> AlphaValuesInput for (ArrayView1<'_, T>, ArrayView1<'_, bool>) {
    fn bind_alpha_values(self, _: Option<&dyn ArraySource>) -> BindingResult<AlphaValues> {
        Ok(AlphaValues::new(
            "Alpha",
            crate::input::bytes(self.0, Some(self.1), "alpha bytes")?,
        )?)
    }
}
/// Observation bytes or an unchanged native encoding; units remain native contracts.
pub trait AlphaInput: sealed::Sealed {
    #[doc(hidden)]
    fn bind_alpha(
        self,
        source: Option<&dyn ArraySource>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding>;
}
impl AlphaInput for AlphaEncoding {
    fn bind_alpha(
        self,
        _: Option<&dyn ArraySource>,
        _: Option<usize>,
        _: usize,
    ) -> BindingResult<AlphaEncoding> {
        Ok(self)
    }
}
fn observations(
    input: impl AlphaValuesInput,
    source: Option<&dyn ArraySource>,
    expected: Option<usize>,
    copies: usize,
) -> BindingResult<AlphaEncoding> {
    let copies = if input.native_slots() { 1 } else { copies };
    Ok(AlphaEncoding::observations(expand(
        input.bind_alpha_values(source)?,
        expected,
        copies,
    )?))
}
impl AlphaInput for AlphaValues {
    fn bind_alpha(
        self,
        source: Option<&dyn ArraySource>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding> {
        observations(self, source, expected, copies)
    }
}
impl AlphaInput for usize {
    fn bind_alpha(
        self,
        source: Option<&dyn ArraySource>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding> {
        observations(self, source, expected, copies)
    }
}
impl<T: ArrayElement> AlphaInput for ArrayView1<'_, T> {
    fn bind_alpha(
        self,
        source: Option<&dyn ArraySource>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding> {
        observations(self, source, expected, copies)
    }
}
impl<T: ArrayElement> AlphaInput for (ArrayView1<'_, T>, ArrayView1<'_, bool>) {
    fn bind_alpha(
        self,
        source: Option<&dyn ArraySource>,
        expected: Option<usize>,
        copies: usize,
    ) -> BindingResult<AlphaEncoding> {
        observations(self, source, expected, copies)
    }
}

pub(crate) fn expand(
    values: AlphaValues,
    expected: Option<usize>,
    copies: usize,
) -> BindingResult<AlphaValues> {
    if copies == 0 {
        return Err(binding("alpha", "no generated columns"));
    }
    if let Some(expected) = expected {
        crate::error::length("alpha", values.name(), expected / copies, values.len())?;
    }
    if copies == 1 {
        Ok(values)
    } else {
        Ok(values.select((0..values.len()).flat_map(|row| std::iter::repeat_n(row, copies)))?)
    }
}
pub(crate) fn keys<I, S, B>(
    source: Option<&dyn ArraySource>,
    keys: I,
) -> BindingResult<Vec<CategoryValues>>
where
    I: IntoIterator<Item = (S, B)>,
    S: AsRef<str>,
    B: CategoryInput,
{
    keys.into_iter()
        .map(|(field, input)| {
            let (_, values) = input.raw_category(source, "alpha selector")?;
            Ok(CategoryValues::nullable(field.as_ref(), values)?)
        })
        .collect()
}
