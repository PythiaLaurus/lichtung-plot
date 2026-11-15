"""Owned numeric policy contracts; opaque format/slot integration is separate."""

import math

import pytest

from lichtung_plot import (
    DataError,
    NumericCompactRule,
    NumericCurrency,
    NumericExplanation,
    NumericFactor,
    NumericField,
    NumericFormat,
    NumericGrouping,
    NumericLocale,
    NumericPrecision,
    NumericPrefixes,
    NumericSymbols,
    NumericUnit,
    SiPrefix,
    Text,
    TextLayout,
    Theme,
)


def test_locale_snapshots_tokens_and_only_grouping_is_a_nullable_component():
    sizes = [3, 2]
    grouping = NumericGrouping("_", sizes, repeat="last")
    tokens = tuple(f"d{i}" for i in range(10))
    supplied = list(tokens)
    symbols = NumericSymbols("+", "-", "%", "‰")
    currency = NumericCurrency("", " credits")
    locale = NumericLocale.custom(
        "::", supplied, grouping=grouping, symbols=symbols, currency=currency,
    )
    alias = locale
    sizes[:] = [99]
    supplied.clear()
    assert grouping.sizes == (3, 2) and grouping.repeat == "last"
    assert locale.digits == tokens and locale.decimal == "::"
    assert locale.grouping == grouping
    changed = locale.copy_with(
        grouping=None,
        symbols=locale.symbols.copy_with(positive="plus"),
        currency=locale.currency.copy_with(suffix=""),
    )
    assert changed == NumericLocale.custom(
        "::", tokens, grouping=None,
        symbols=NumericSymbols("plus", "-", "%", "‰"),
        currency=NumericCurrency("", ""),
    )
    assert alias.grouping == grouping and alias.symbols == symbols
    assert alias.currency == currency and currency.suffix == " credits"
    named = NumericLocale.named("fr-FR")
    assert "fr-FR" in NumericLocale.named_ids()
    assert named.copy_with() == NumericLocale.custom(
        named.decimal, named.digits, grouping=named.grouping,
        symbols=named.symbols, currency=named.currency,
    )
    assert NumericLocale.invariant().copy_with(grouping=None).grouping is None
    with pytest.raises(TypeError):
        NumericLocale.custom(".", tokens, symbols=symbols, currency=currency)
    with pytest.raises(TypeError):
        locale.copy_with(symbols=None)
    with pytest.raises(ValueError, match="exactly ten"):
        locale.copy_with(digits=tokens[:-1])
    with pytest.raises(DataError):
        locale.copy_with(digits=["same"] * 10)
    with pytest.raises(DataError):
        grouping.copy_with(sizes=[3, 0])
    with pytest.raises(DataError):
        symbols.copy_with(positive="")
    with pytest.raises(AttributeError):
        locale.symbols.negative = "changed"
    assert locale == alias and locale.digits == tokens


def test_precision_copies_keep_branch_and_delay_bounds_until_format_attachment():
    original = NumericPrecision.fractional(2)
    unchecked = original.copy_with(digits=255)
    assert unchecked.kind == "fractional" and unchecked.digits == 255
    assert unchecked.minimum is None and unchecked.maximum is None
    with pytest.raises(DataError):
        NumericFormat(precision=unchecked)

    adaptive = NumericPrecision.adaptive_significant(0, 5)
    degenerate = adaptive.copy_with(minimum=3, maximum=3)
    assert degenerate.kind == "adaptive_significant" and degenerate.digits is None
    assert (degenerate.minimum, degenerate.maximum) == (3, 3)
    # The opaque field delegates its single-scalar/token checks to native new.
    field = NumericField(0, fill="λ", alignment="after_sign")
    policy = NumericFormat(notation="general", precision=degenerate, field=field)
    assert policy.precision == degenerate
    assert adaptive.minimum == 0 and original.digits == 2
    with pytest.raises(TypeError, match="branch"):
        original.copy_with(minimum=None)
    with pytest.raises(TypeError, match="branch"):
        adaptive.copy_with(digits=None)
    with pytest.raises(TypeError):
        original.copy_with(digits=None)
    with pytest.raises(OverflowError):
        NumericPrecision.fractional(256)
    with pytest.raises(DataError):
        NumericField(4, fill="\n")


def test_prefix_policies_keep_fixed_zero_order_and_native_empty_normalization():
    first = NumericCompactRule(-2, 1, prefix="", suffix="")
    later = NumericCompactRule(1000, 1000, prefix="", suffix="k")
    supplied = [first, later]
    fixed = NumericPrefixes.compact(
        supplied, selection=0, promotion="keep_threshold",
    )
    supplied.clear()
    assert fixed == NumericPrefixes.compact(
        [first, later], selection=0, promotion="keep_threshold",
    )
    assert fixed != NumericPrefixes.compact(
        [first, later], selection="whole_set", promotion="keep_threshold",
    )
    assert first.copy_with(threshold=-3, divisor=2, suffix=" ") == NumericCompactRule(
        -3, 2, prefix="", suffix=" ",
    )
    assert first.threshold == -2 and first.divisor == 1 and first.suffix == ""
    assert NumericPrefixes.compact(
        [], selection="whole_set", promotion="promote_on_rounding",
    ) == NumericPrefixes.none()
    neutral = SiPrefix.from_exponent(0)
    assert neutral.exponent == 0 and neutral.symbol == ""
    assert NumericPrefixes.si(
        neutral, promotion="keep_threshold",
    ) != NumericPrefixes.none()
    assert SiPrefix.from_exponent(-2).symbol == "c"
    assert SiPrefix.from_exponent(30).symbol == "Q"
    with pytest.raises(DataError):
        NumericPrefixes.compact(
            [later, first], selection="whole_set", promotion="keep_threshold",
        )
    with pytest.raises(DataError):
        NumericPrefixes.compact([], selection=0, promotion="keep_threshold")
    with pytest.raises(DataError):
        NumericPrefixes.si(neutral, promotion="promote_on_rounding")
    with pytest.raises(DataError):
        SiPrefix.from_exponent(4)
    with pytest.raises(TypeError):
        NumericPrefixes.si(0, promotion="keep_threshold")


def test_factor_copy_retains_raw_payload_until_native_prefix_normalization():
    source = "+0001000000000000000000000000000000000000000"
    raw = NumericFactor.power_of_ten(source)
    assert raw.kind == "power_of_ten" and raw.value is None and raw.exponent == source
    canonical = NumericFactor.power_of_ten("1000000000000000000000000000000000000000")
    assert NumericPrefixes.factor(raw) == NumericPrefixes.factor(canonical)
    assert raw.exponent == source
    assert raw.copy_with(exponent="-0002").exponent == "-0002"
    zero = NumericFactor.from_value(0)
    infinite = zero.copy_with(value=float("inf"))
    assert zero.kind == "value" and zero.value == 0 and zero.exponent is None
    assert math.isinf(infinite.value)
    assert NumericPrefixes.factor(NumericFactor.from_value(-2)) != NumericPrefixes.none()
    malformed = NumericFactor.power_of_ten("2x")
    assert malformed.exponent == "2x"
    automatic = NumericFactor.automatic_decimal()
    assert automatic.copy_with() == automatic and automatic.kind == "automatic_decimal"
    with pytest.raises(DataError):
        NumericPrefixes.factor(zero)
    with pytest.raises(DataError):
        NumericPrefixes.factor(infinite)
    with pytest.raises(DataError):
        NumericPrefixes.factor(malformed)
    with pytest.raises(TypeError, match="branch"):
        zero.copy_with(exponent=None)
    with pytest.raises(TypeError, match="branch"):
        automatic.copy_with(value=None)
    with pytest.raises(TypeError):
        raw.copy_with(exponent=None)
    assert raw.exponent == source and zero.value == 0


def test_units_and_explanations_keep_content_presence_and_text_instruction_clears():
    unit = NumericUnit("kg", placement="shared", separator="_")
    unchecked = unit.copy_with(separator="\n")
    assert unchecked.separator == "\n" and unit.separator == "_"
    assert unit.copy_with(text="", separator="") == NumericUnit(
        "", placement="shared", separator="",
    )
    with pytest.raises(DataError):
        NumericFormat(unit=unchecked)
    with pytest.raises(TypeError):
        unit.copy_with(text=None)

    style = Theme.publication().typography.root.caption
    layout = TextLayout(alignment="end")
    incoming = Text("Readout", style=style, layout=layout)
    authored = NumericExplanation.text(incoming)
    plain = NumericExplanation.text("Readout")
    without_style = NumericExplanation.text(incoming, style=None)
    without_layout = NumericExplanation.text(incoming, layout=None)
    assert authored.copy_with() == authored
    assert authored != without_style != plain
    assert authored != without_layout != plain
    assert authored.copy_with(style=None) == without_style
    assert authored.copy_with(layout=None) == without_layout
    assert authored.copy_with(style=None, layout=None) == plain
    generated = NumericExplanation.automatic(style=style, layout=layout)
    cleared = generated.copy_with(style=None, layout=None)
    assert cleared == NumericExplanation.automatic()
    assert cleared != NumericExplanation.text("")
    assert NumericExplanation.automatic(layout=TextLayout()) != cleared
    assert generated != generated.copy_with(style=None) != cleared
    assert generated == NumericExplanation.automatic(style=style, layout=layout)
    assert incoming.style == style and incoming.layout == layout
    with pytest.raises(TypeError):
        NumericExplanation.text(None)
    with pytest.raises(TypeError):
        NumericExplanation.automatic(style=object())
