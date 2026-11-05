mod sealed {
    pub trait Sealed {}
    macro_rules! seal {
        ($($ty:ty),+ $(,)?) => {$(
            impl Sealed for $ty {}
            impl Sealed for Option<$ty> {}
        )+};
    }
    seal!(
        f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool, String,
        &str
    );
}

/// Sealed primitive numeric input, including its nullable `Option` form.
pub trait NumericElement: sealed::Sealed {
    /// Normalize a present value to f64 with ordinary floating-point rounding.
    fn to_f64(&self) -> Option<f64>;
}

macro_rules! numeric {
    ($($ty:ty),+ $(,)?) => {$(
        impl NumericElement for $ty {
            #[allow(clippy::cast_precision_loss, clippy::cast_lossless,
                reason = "all primitive widths share ordinary f64 normalization, including rounding")]
            fn to_f64(&self) -> Option<f64> { Some(*self as f64) }
        }
        impl NumericElement for Option<$ty> {
            fn to_f64(&self) -> Option<f64> { self.as_ref().and_then(NumericElement::to_f64) }
        }
    )+};
}
numeric!(f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

/// Sealed raw string input, including nullable strings. No display conversion.
pub trait CategoryElement: sealed::Sealed {
    /// Borrow the exact present key without normalization.
    fn as_key(&self) -> Option<&str>;
}

macro_rules! category {
    ($($ty:ty),+ $(,)?) => {$(
        impl CategoryElement for $ty {
            fn as_key(&self) -> Option<&str> { Some(self) }
        }
        impl CategoryElement for Option<$ty> {
            fn as_key(&self) -> Option<&str> { self.as_ref().and_then(CategoryElement::as_key) }
        }
    )+};
}
category!(String, &str);

/// Sealed Boolean input. Booleans are not implicit numeric 0/1.
pub trait BooleanElement: sealed::Sealed {
    /// Read a Boolean independently of its validity.
    fn as_boolean(&self) -> Option<bool>;
}

impl BooleanElement for bool {
    fn as_boolean(&self) -> Option<bool> {
        Some(*self)
    }
}
impl BooleanElement for Option<bool> {
    fn as_boolean(&self) -> Option<bool> {
        *self
    }
}

/// Sealed integer provenance input; IDs never pass through f64.
pub trait SourceRowElement: sealed::Sealed + std::fmt::Debug {
    /// Return an exact present nonnegative ID that fits usize.
    fn to_usize(&self) -> Option<usize>;
}

macro_rules! source_row {
    ($($ty:ty),+ $(,)?) => {$(
        impl SourceRowElement for $ty {
            #[allow(clippy::useless_conversion)]
            fn to_usize(&self) -> Option<usize> { usize::try_from(*self).ok() }
        }
        impl SourceRowElement for Option<$ty> {
            fn to_usize(&self) -> Option<usize> { self.as_ref().and_then(SourceRowElement::to_usize) }
        }
    )+};
}
source_row!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

/// Sealed primitive source elements. Role support is checked before reading slots.
pub trait ArrayElement: sealed::Sealed + std::fmt::Debug {
    #[doc(hidden)]
    const NUMERIC: bool = false;
    #[doc(hidden)]
    const CATEGORY: bool = false;
    #[doc(hidden)]
    const BOOLEAN: bool = false;
    #[doc(hidden)]
    const ID: bool = false;
    #[doc(hidden)]
    const BYTE: bool = false;
    #[doc(hidden)]
    fn numeric(&self) -> Option<f64> {
        None
    }
    #[doc(hidden)]
    fn category(&self) -> Option<&str> {
        None
    }
    #[doc(hidden)]
    fn boolean(&self) -> Option<bool> {
        None
    }
    #[doc(hidden)]
    fn id(&self) -> Option<usize> {
        None
    }
    #[doc(hidden)]
    fn byte(&self) -> Option<u8> {
        None
    }
}

macro_rules! source_numeric {
    ($($ty:ty),+) => {$ (
        impl ArrayElement for $ty {
            const NUMERIC: bool = true;
            const ID: bool = true;
            fn numeric(&self) -> Option<f64> { self.to_f64() }
            fn id(&self) -> Option<usize> { self.to_usize() }
        }
    )+};
}
source_numeric!(i8, i16, i32, i64, i128, isize, u16, u32, u64, u128, usize);
impl ArrayElement for u8 {
    const NUMERIC: bool = true;
    const ID: bool = true;
    const BYTE: bool = true;
    fn numeric(&self) -> Option<f64> {
        self.to_f64()
    }
    fn id(&self) -> Option<usize> {
        self.to_usize()
    }
    fn byte(&self) -> Option<u8> {
        Some(*self)
    }
}
macro_rules! source_float {
    ($($ty:ty),+) => {$ (
        impl ArrayElement for $ty {
            const NUMERIC: bool = true;
            fn numeric(&self) -> Option<f64> { self.to_f64() }
        }
    )+};
}
source_float!(f32, f64);
impl ArrayElement for bool {
    const BOOLEAN: bool = true;
    fn boolean(&self) -> Option<bool> {
        Some(*self)
    }
}
impl ArrayElement for String {
    const CATEGORY: bool = true;
    fn category(&self) -> Option<&str> {
        Some(self)
    }
}
impl ArrayElement for &str {
    const CATEGORY: bool = true;
    fn category(&self) -> Option<&str> {
        Some(self)
    }
}
impl<T: ArrayElement> ArrayElement for Option<T>
where
    Option<T>: sealed::Sealed,
{
    const NUMERIC: bool = T::NUMERIC;
    const CATEGORY: bool = T::CATEGORY;
    const BOOLEAN: bool = T::BOOLEAN;
    const ID: bool = T::ID;
    const BYTE: bool = T::BYTE;
    fn numeric(&self) -> Option<f64> {
        self.as_ref().and_then(ArrayElement::numeric)
    }
    fn category(&self) -> Option<&str> {
        self.as_ref().and_then(ArrayElement::category)
    }
    fn boolean(&self) -> Option<bool> {
        self.as_ref().and_then(ArrayElement::boolean)
    }
    fn id(&self) -> Option<usize> {
        self.as_ref().and_then(ArrayElement::id)
    }
    fn byte(&self) -> Option<u8> {
        self.as_ref().and_then(ArrayElement::byte)
    }
}
