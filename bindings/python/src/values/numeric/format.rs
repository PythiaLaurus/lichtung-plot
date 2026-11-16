//! Complete native format edits; opaque policy and callback identity stay native.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::{PyTraverseError, PyVisit};

use super::locale::PyNumericLocale;
use super::precision::{PyNumericField, PyNumericPrecision};
use super::prefixes::PyNumericPrefixes;
use super::unit::{PyNumericExplanation, PyNumericUnit};
use super::{callbacks, core, is_omitted, native_error, omitted, PyNumericFormat};

macro_rules! choice {
    ($parse:ident, $ty:ty, $label:literal, {$($name:literal => $value:expr),+ $(,)?}) => {
        fn $parse(value: &str) -> PyResult<$ty> {
            match value {
                $($name => Ok($value),)+
                _ => Err(PyValueError::new_err(concat!("invalid numeric ", $label))),
            }
        }
    };
}

choice!(notation, core::NumericNotation, "notation", {
    "decimal" => core::NumericNotation::Decimal,
    "scientific" => core::NumericNotation::Scientific,
    "engineering" => core::NumericNotation::Engineering,
    "general" => core::NumericNotation::General,
    "integer_decimal" => core::NumericNotation::Integer(core::NumericRadix::Decimal),
    "integer_binary" => core::NumericNotation::Integer(core::NumericRadix::Binary),
    "integer_octal" => core::NumericNotation::Integer(core::NumericRadix::Octal),
    "integer_hex_lower" => core::NumericNotation::Integer(core::NumericRadix::HexLower),
    "integer_hex_upper" => core::NumericNotation::Integer(core::NumericRadix::HexUpper),
});
choice!(negative_zero, core::NegativeZero, "negative_zero", {
    "preserve" => core::NegativeZero::Preserve,
    "normalize" => core::NegativeZero::Normalize,
});
choice!(trailing_zeros, core::TrailingZeros, "trailing_zeros", {
    "keep" => core::TrailingZeros::Keep,
    "trim" => core::TrailingZeros::Trim,
});
choice!(rounding, core::NumericRounding, "rounding", {
    "nearest_even" => core::NumericRounding::NearestEven,
    "nearest_away" => core::NumericRounding::NearestAway,
    "nearest_toward_zero" => core::NumericRounding::NearestTowardZero,
    "toward_zero" => core::NumericRounding::TowardZero,
    "away_from_zero" => core::NumericRounding::AwayFromZero,
    "floor" => core::NumericRounding::Floor,
    "ceiling" => core::NumericRounding::Ceiling,
});
choice!(rate, core::NumericRate, "rate", {
    "value" => core::NumericRate::Value,
    "percent" => core::NumericRate::Percent,
    "per_mille" => core::NumericRate::PerMille,
});
choice!(sign, core::NumericSign, "sign", {
    "negative_only" => core::NumericSign::NegativeOnly,
    "always" => core::NumericSign::Always,
    "space" => core::NumericSign::Space,
    "accounting" => core::NumericSign::Accounting,
});
choice!(symbol, core::NumericSymbol, "symbol", {
    "none" => core::NumericSymbol::None,
    "currency" => core::NumericSymbol::Currency,
    "radix_prefix" => core::NumericSymbol::RadixPrefix,
});

fn notation_name(value: core::NumericNotation) -> &'static str {
    match value {
        core::NumericNotation::Decimal => "decimal",
        core::NumericNotation::Scientific => "scientific",
        core::NumericNotation::Engineering => "engineering",
        core::NumericNotation::General => "general",
        core::NumericNotation::Integer(core::NumericRadix::Decimal) => "integer_decimal",
        core::NumericNotation::Integer(core::NumericRadix::Binary) => "integer_binary",
        core::NumericNotation::Integer(core::NumericRadix::Octal) => "integer_octal",
        core::NumericNotation::Integer(core::NumericRadix::HexLower) => "integer_hex_lower",
        core::NumericNotation::Integer(core::NumericRadix::HexUpper) => "integer_hex_upper",
    }
}

fn negative_zero_name(value: core::NegativeZero) -> &'static str {
    match value {
        core::NegativeZero::Preserve => "preserve",
        core::NegativeZero::Normalize => "normalize",
    }
}

fn trailing_zeros_name(value: core::TrailingZeros) -> &'static str {
    match value {
        core::TrailingZeros::Keep => "keep",
        core::TrailingZeros::Trim => "trim",
    }
}

fn notation_precision(
    native: core::NumericFormat,
    notation: core::NumericNotation,
    precision: core::NumericPrecision,
) -> core::Result<core::NumericFormat> {
    let precision_error = match native
        .clone()
        .precision(precision)
        .and_then(|value| value.notation(notation))
    {
        Ok(candidate) => return Ok(candidate),
        Err(error) => error,
    };
    if let Ok(candidate) = native
        .clone()
        .notation(notation)
        .and_then(|value| value.precision(precision))
    {
        return Ok(candidate);
    }
    match native.notation(core::NumericNotation::Decimal) {
        Ok(bridge) => bridge.precision(precision)?.notation(notation),
        Err(bridge_error) => {
            // A retained radix prefix can forbid the bridge. For a final
            // nondecimal integer, that prefix is still applicable: preserve
            // the direct native precision error, not an intermediate error.
            if matches!(notation, core::NumericNotation::Integer(radix) if radix != core::NumericRadix::Decimal)
            {
                Err(precision_error)
            } else {
                Err(bridge_error)
            }
        }
    }
}

/// All inputs are decoded before these native edits. No symbol reader or
/// speculative replacement of opaque state is needed. Direct orders precede
/// any bridge, including for every legal retained-radix integer-to-integer edit.
fn coupled(
    mut native: core::NumericFormat,
    notation: Option<core::NumericNotation>,
    precision: Option<core::NumericPrecision>,
    symbol: Option<core::NumericSymbol>,
) -> core::Result<core::NumericFormat> {
    if let Some(value) = symbol.filter(|value| *value != core::NumericSymbol::RadixPrefix) {
        native = native.symbol(value)?;
    }
    if notation.is_some() || precision.is_some() {
        let notation = notation.unwrap_or_else(|| native.get_notation());
        let precision = precision.unwrap_or_else(|| native.get_precision());
        native = notation_precision(native, notation, precision)?;
    }
    if symbol == Some(core::NumericSymbol::RadixPrefix) {
        native = native.symbol(core::NumericSymbol::RadixPrefix)?;
    }
    Ok(native)
}

/// Call-local keep/clear/set choice for the two nullable format policies.
enum ClearableFormatEdit<T> {
    Keep,
    Replace(Option<T>),
}

/// Call-local decoded replacements, not retained configuration or Python mirrors.
struct FormatEdits {
    notation: Option<core::NumericNotation>,
    precision: Option<core::NumericPrecision>,
    decimal_scale: Option<i16>,
    prefix: Option<String>,
    suffix: Option<String>,
    negative_zero: Option<core::NegativeZero>,
    trailing_zeros: Option<core::TrailingZeros>,
    rounding: Option<core::NumericRounding>,
    multiplier: Option<f64>,
    rate: Option<core::NumericRate>,
    locale: Option<core::NumericLocale>,
    grouping: Option<bool>,
    sign: Option<core::NumericSign>,
    symbol: Option<core::NumericSymbol>,
    field: ClearableFormatEdit<core::NumericField>,
    prefixes: Option<core::NumericPrefixes>,
    unit: ClearableFormatEdit<core::NumericUnit>,
    explanation: Option<core::NumericExplanation>,
}

impl FormatEdits {
    fn apply(self, native: core::NumericFormat) -> core::Result<core::NumericFormat> {
        let mut native = coupled(native, self.notation, self.precision, self.symbol)?;
        if let Some(value) = self.decimal_scale {
            native = native.decimal_scale(value)?;
        }
        if let Some(value) = self.prefix {
            native = native.prefix(value)?;
        }
        if let Some(value) = self.suffix {
            native = native.suffix(value)?;
        }
        if let Some(value) = self.negative_zero {
            native = native.negative_zero(value)?;
        }
        if let Some(value) = self.trailing_zeros {
            native = native.trailing_zeros(value)?;
        }
        if let Some(value) = self.rounding {
            native = native.rounding(value)?;
        }
        if let Some(value) = self.multiplier {
            native = native.multiplier(value)?;
        }
        if let Some(value) = self.rate {
            native = native.rate(value)?;
        }
        if let Some(value) = self.locale {
            native = native.locale(value)?;
        }
        if let Some(value) = self.grouping {
            native = native.grouping(value)?;
        }
        if let Some(value) = self.sign {
            native = native.sign(value)?;
        }
        if let ClearableFormatEdit::Replace(value) = self.field {
            native = native.field(value)?;
        }
        if let Some(value) = self.prefixes {
            native = native.prefixes(value)?;
        }
        if let ClearableFormatEdit::Replace(value) = self.unit {
            native = native.unit(value)?;
        }
        if let Some(value) = self.explanation {
            native = native.explanation(value);
        }
        Ok(native)
    }
}

impl PyNumericFormat {
    #[allow(
        clippy::too_many_arguments,
        reason = "one constructor prepares the complete native policy and exclusive callback selection"
    )]
    fn construct(
        py: Python<'_>,
        notation: &str,
        precision: Py<PyAny>,
        decimal_scale: i16,
        prefix: Option<String>,
        suffix: Option<String>,
        negative_zero: &str,
        trailing_zeros: &str,
        rounding: &str,
        multiplier: f64,
        rate: &str,
        locale: Py<PyAny>,
        grouping: bool,
        sign: &str,
        symbol: &str,
        field: Option<PyRef<'_, PyNumericField>>,
        prefixes: Py<PyAny>,
        unit: Option<PyRef<'_, PyNumericUnit>>,
        explanation: Py<PyAny>,
        value_function: Option<&Bound<'_, PyAny>>,
        set_function: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        if value_function.is_some() && set_function.is_some() {
            return Err(PyTypeError::new_err(
                "value_function and set_function are mutually exclusive",
            ));
        }
        let precision = precision.into_bound(py);
        let locale = locale.into_bound(py);
        let prefixes = prefixes.into_bound(py);
        let explanation = explanation.into_bound(py);
        let edits = FormatEdits {
            notation: Some(self::notation(notation)?),
            precision: Some(if is_omitted(&precision) {
                core::NumericPrecision::Fractional(2)
            } else {
                precision.extract::<PyRef<'_, PyNumericPrecision>>()?.native
            }),
            decimal_scale: Some(decimal_scale),
            prefix,
            suffix,
            negative_zero: Some(self::negative_zero(negative_zero)?),
            trailing_zeros: Some(self::trailing_zeros(trailing_zeros)?),
            rounding: Some(self::rounding(rounding)?),
            multiplier: Some(multiplier),
            rate: Some(self::rate(rate)?),
            locale: Some(if is_omitted(&locale) {
                core::NumericLocale::invariant()
            } else {
                locale
                    .extract::<PyRef<'_, PyNumericLocale>>()?
                    .native
                    .clone()
            }),
            grouping: Some(grouping),
            sign: Some(self::sign(sign)?),
            symbol: Some(self::symbol(symbol)?),
            field: ClearableFormatEdit::Replace(field.map(|value| value.native.clone())),
            prefixes: Some(if is_omitted(&prefixes) {
                core::NumericPrefixes::none()
            } else {
                prefixes
                    .extract::<PyRef<'_, PyNumericPrefixes>>()?
                    .native
                    .clone()
            }),
            unit: ClearableFormatEdit::Replace(unit.map(|value| value.native.clone())),
            explanation: Some(if is_omitted(&explanation) {
                core::NumericExplanation::automatic()
            } else {
                explanation
                    .extract::<PyRef<'_, PyNumericExplanation>>()?
                    .native
                    .clone()
            }),
        };
        let native = edits
            .apply(core::NumericFormat::default())
            .map_err(native_error)?;
        let (native, roots) = if let Some(callable) = value_function {
            callbacks::value_function(py, native, callable)?
        } else if let Some(callable) = set_function {
            callbacks::set_function(py, native, callable)?
        } else {
            (native, Vec::new())
        };
        Ok(Self {
            native: Some(native),
            roots,
        })
    }
}

#[pymethods]
impl PyNumericFormat {
    #[new]
    #[pyo3(signature = (*, notation="decimal", precision=omitted(), decimal_scale=0,
        prefix=None, suffix=None, negative_zero="preserve", trailing_zeros="keep",
        rounding="nearest_even", multiplier=1.0, rate="value", locale=omitted(),
        grouping=false, sign="negative_only", symbol="none", field=None,
        prefixes=omitted(), unit=None, explanation=omitted(), value_function=None, set_function=None))]
    #[allow(
        clippy::too_many_arguments,
        reason = "all native numeric policy inputs are independent keywords"
    )]
    fn new(
        py: Python<'_>,
        notation: &str,
        precision: Py<PyAny>,
        decimal_scale: i16,
        prefix: Option<String>,
        suffix: Option<String>,
        negative_zero: &str,
        trailing_zeros: &str,
        rounding: &str,
        multiplier: f64,
        rate: &str,
        locale: Py<PyAny>,
        grouping: bool,
        sign: &str,
        symbol: &str,
        field: Option<PyRef<'_, PyNumericField>>,
        prefixes: Py<PyAny>,
        unit: Option<PyRef<'_, PyNumericUnit>>,
        explanation: Py<PyAny>,
        value_function: Option<&Bound<'_, PyAny>>,
        set_function: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        Self::construct(
            py,
            notation,
            precision,
            decimal_scale,
            prefix,
            suffix,
            negative_zero,
            trailing_zeros,
            rounding,
            multiplier,
            rate,
            locale,
            grouping,
            sign,
            symbol,
            field,
            prefixes,
            unit,
            explanation,
            value_function,
            set_function,
        )
    }

    #[staticmethod]
    fn from_specifier(specifier: &str) -> PyResult<Self> {
        Ok(Self {
            native: Some(core::NumericFormat::from_specifier(specifier).map_err(native_error)?),
            roots: Vec::new(),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (callback, *, notation="decimal", precision=omitted(), decimal_scale=0,
        prefix=None, suffix=None, negative_zero="preserve", trailing_zeros="keep",
        rounding="nearest_even", multiplier=1.0, rate="value", locale=omitted(),
        grouping=false, sign="negative_only", symbol="none", field=None,
        prefixes=omitted(), unit=None, explanation=omitted()))]
    #[allow(
        clippy::too_many_arguments,
        reason = "callback factories accept the same complete native numeric policy"
    )]
    fn value_function(
        py: Python<'_>,
        callback: &Bound<'_, PyAny>,
        notation: &str,
        precision: Py<PyAny>,
        decimal_scale: i16,
        prefix: Option<String>,
        suffix: Option<String>,
        negative_zero: &str,
        trailing_zeros: &str,
        rounding: &str,
        multiplier: f64,
        rate: &str,
        locale: Py<PyAny>,
        grouping: bool,
        sign: &str,
        symbol: &str,
        field: Option<PyRef<'_, PyNumericField>>,
        prefixes: Py<PyAny>,
        unit: Option<PyRef<'_, PyNumericUnit>>,
        explanation: Py<PyAny>,
    ) -> PyResult<Self> {
        Self::construct(
            py,
            notation,
            precision,
            decimal_scale,
            prefix,
            suffix,
            negative_zero,
            trailing_zeros,
            rounding,
            multiplier,
            rate,
            locale,
            grouping,
            sign,
            symbol,
            field,
            prefixes,
            unit,
            explanation,
            Some(callback),
            None,
        )
    }

    #[staticmethod]
    #[pyo3(signature = (callback, *, notation="decimal", precision=omitted(), decimal_scale=0,
        prefix=None, suffix=None, negative_zero="preserve", trailing_zeros="keep",
        rounding="nearest_even", multiplier=1.0, rate="value", locale=omitted(),
        grouping=false, sign="negative_only", symbol="none", field=None,
        prefixes=omitted(), unit=None, explanation=omitted()))]
    #[allow(
        clippy::too_many_arguments,
        reason = "callback factories accept the same complete native numeric policy"
    )]
    fn set_function(
        py: Python<'_>,
        callback: &Bound<'_, PyAny>,
        notation: &str,
        precision: Py<PyAny>,
        decimal_scale: i16,
        prefix: Option<String>,
        suffix: Option<String>,
        negative_zero: &str,
        trailing_zeros: &str,
        rounding: &str,
        multiplier: f64,
        rate: &str,
        locale: Py<PyAny>,
        grouping: bool,
        sign: &str,
        symbol: &str,
        field: Option<PyRef<'_, PyNumericField>>,
        prefixes: Py<PyAny>,
        unit: Option<PyRef<'_, PyNumericUnit>>,
        explanation: Py<PyAny>,
    ) -> PyResult<Self> {
        Self::construct(
            py,
            notation,
            precision,
            decimal_scale,
            prefix,
            suffix,
            negative_zero,
            trailing_zeros,
            rounding,
            multiplier,
            rate,
            locale,
            grouping,
            sign,
            symbol,
            field,
            prefixes,
            unit,
            explanation,
            None,
            Some(callback),
        )
    }

    #[getter]
    fn notation(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(notation_name(self.live_native(py)?.get_notation()))
    }

    #[getter]
    fn precision(&self, py: Python<'_>) -> PyResult<PyNumericPrecision> {
        Ok(PyNumericPrecision {
            native: self.live_native(py)?.get_precision(),
        })
    }

    #[getter]
    fn decimal_scale(&self, py: Python<'_>) -> PyResult<i16> {
        Ok(self.live_native(py)?.get_decimal_scale())
    }

    #[getter]
    fn prefix(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.live_native(py)?.get_prefix().map(str::to_owned))
    }

    #[getter]
    fn suffix(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.live_native(py)?.get_suffix().map(str::to_owned))
    }

    #[getter]
    fn negative_zero(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(negative_zero_name(
            self.live_native(py)?.get_negative_zero(),
        ))
    }

    #[getter]
    fn trailing_zeros(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(trailing_zeros_name(
            self.live_native(py)?.get_trailing_zeros(),
        ))
    }

    #[pyo3(signature = (*, notation=omitted(), precision=omitted(), decimal_scale=omitted(),
        prefix=omitted(), suffix=omitted(), negative_zero=omitted(), trailing_zeros=omitted(),
        rounding=omitted(), multiplier=omitted(), rate=omitted(), locale=omitted(),
        grouping=omitted(), sign=omitted(), symbol=omitted(), field=omitted(),
        prefixes=omitted(), unit=omitted(), explanation=omitted(),
        value_function=omitted(), set_function=omitted()),
        text_signature = "($self, *, notation=..., precision=..., decimal_scale=..., prefix=..., suffix=..., negative_zero=..., trailing_zeros=..., rounding=..., multiplier=..., rate=..., locale=..., grouping=..., sign=..., symbol=..., field=..., prefixes=..., unit=..., explanation=..., value_function=..., set_function=...)")]
    #[allow(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "decode all independently omissible policy and callback edits before one native clone/edit plan"
    )]
    fn copy_with(
        slf: &Bound<'_, Self>,
        py: Python<'_>,
        notation: Py<PyAny>,
        precision: Py<PyAny>,
        decimal_scale: Py<PyAny>,
        prefix: Py<PyAny>,
        suffix: Py<PyAny>,
        negative_zero: Py<PyAny>,
        trailing_zeros: Py<PyAny>,
        rounding: Py<PyAny>,
        multiplier: Py<PyAny>,
        rate: Py<PyAny>,
        locale: Py<PyAny>,
        grouping: Py<PyAny>,
        sign: Py<PyAny>,
        symbol: Py<PyAny>,
        field: Py<PyAny>,
        prefixes: Py<PyAny>,
        unit: Py<PyAny>,
        explanation: Py<PyAny>,
        value_function: Py<PyAny>,
        set_function: Py<PyAny>,
    ) -> PyResult<Self> {
        let (native, mut roots) = {
            let borrowed = slf.try_borrow()?;
            borrowed.snapshot(py)?
        };
        // The slf PyRef has ended before conversion can run foreign Python code,
        // and before failure cleanup or callback replacement can DECREF roots.
        let notation = &notation.into_bound(py);
        let precision = &precision.into_bound(py);
        let decimal_scale = &decimal_scale.into_bound(py);
        let prefix = &prefix.into_bound(py);
        let suffix = &suffix.into_bound(py);
        let negative_zero = &negative_zero.into_bound(py);
        let trailing_zeros = &trailing_zeros.into_bound(py);
        let rounding = &rounding.into_bound(py);
        let multiplier = &multiplier.into_bound(py);
        let rate = &rate.into_bound(py);
        let locale = &locale.into_bound(py);
        let grouping = &grouping.into_bound(py);
        let sign = &sign.into_bound(py);
        let symbol = &symbol.into_bound(py);
        let field = &field.into_bound(py);
        let prefixes = &prefixes.into_bound(py);
        let unit = &unit.into_bound(py);
        let explanation = &explanation.into_bound(py);
        let value_function = &value_function.into_bound(py);
        let set_function = &set_function.into_bound(py);
        let value_present = !is_omitted(value_function);
        let set_present = !is_omitted(set_function);
        if value_present && set_present {
            return Err(PyTypeError::new_err(
                "replace only one of value_function and set_function",
            ));
        }
        let edits = FormatEdits {
            notation: (!is_omitted(notation))
                .then(|| self::notation(notation.extract()?))
                .transpose()?,
            precision: (!is_omitted(precision))
                .then(|| {
                    precision
                        .extract::<PyRef<'_, PyNumericPrecision>>()
                        .map(|value| value.native)
                })
                .transpose()?,
            decimal_scale: (!is_omitted(decimal_scale))
                .then(|| decimal_scale.extract())
                .transpose()?,
            prefix: (!is_omitted(prefix))
                .then(|| prefix.extract())
                .transpose()?,
            suffix: (!is_omitted(suffix))
                .then(|| suffix.extract())
                .transpose()?,
            negative_zero: (!is_omitted(negative_zero))
                .then(|| self::negative_zero(negative_zero.extract()?))
                .transpose()?,
            trailing_zeros: (!is_omitted(trailing_zeros))
                .then(|| self::trailing_zeros(trailing_zeros.extract()?))
                .transpose()?,
            rounding: (!is_omitted(rounding))
                .then(|| self::rounding(rounding.extract()?))
                .transpose()?,
            multiplier: (!is_omitted(multiplier))
                .then(|| multiplier.extract())
                .transpose()?,
            rate: (!is_omitted(rate))
                .then(|| self::rate(rate.extract()?))
                .transpose()?,
            locale: (!is_omitted(locale))
                .then(|| {
                    locale
                        .extract::<PyRef<'_, PyNumericLocale>>()
                        .map(|value| value.native.clone())
                })
                .transpose()?,
            grouping: (!is_omitted(grouping))
                .then(|| grouping.extract())
                .transpose()?,
            sign: (!is_omitted(sign))
                .then(|| self::sign(sign.extract()?))
                .transpose()?,
            symbol: (!is_omitted(symbol))
                .then(|| self::symbol(symbol.extract()?))
                .transpose()?,
            field: if is_omitted(field) {
                ClearableFormatEdit::Keep
            } else {
                ClearableFormatEdit::Replace(
                    field
                        .extract::<Option<PyRef<'_, PyNumericField>>>()?
                        .map(|value| value.native.clone()),
                )
            },
            prefixes: (!is_omitted(prefixes))
                .then(|| {
                    prefixes
                        .extract::<PyRef<'_, PyNumericPrefixes>>()
                        .map(|value| value.native.clone())
                })
                .transpose()?,
            unit: if is_omitted(unit) {
                ClearableFormatEdit::Keep
            } else {
                ClearableFormatEdit::Replace(
                    unit.extract::<Option<PyRef<'_, PyNumericUnit>>>()?
                        .map(|value| value.native.clone()),
                )
            },
            explanation: (!is_omitted(explanation))
                .then(|| {
                    explanation
                        .extract::<PyRef<'_, PyNumericExplanation>>()
                        .map(|value| value.native.clone())
                })
                .transpose()?,
        };
        let mut native = edits.apply(native).map_err(native_error)?;
        if value_present || set_present {
            let callable = if value_present {
                value_function
            } else {
                set_function
            };
            if callable.is_none() {
                native = native.builtin();
                roots.clear();
            } else {
                let replaced = if value_present {
                    callbacks::value_function(py, native, callable)?
                } else {
                    callbacks::set_function(py, native, callable)?
                };
                native = replaced.0;
                roots = replaced.1;
            }
        }
        Ok(Self {
            native: Some(native),
            roots,
        })
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "PyO3 GC traversal requires an owned PyVisit argument"
    )]
    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        for root in &self.roots {
            visit.call(root)?;
        }
        Ok(())
    }

    fn __clear__(slf: &Bound<'_, Self>) -> PyResult<()> {
        let retired = {
            let mut value = slf.try_borrow_mut()?;
            (value.native.take(), std::mem::take(&mut value.roots))
        };
        drop(retired);
        Ok(())
    }
}
