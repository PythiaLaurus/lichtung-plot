"""Installed complete numeric-policy typing; these contracts are not executed."""

from typing import Literal

from typing_extensions import assert_type

import lichtung_plot as lp


def _builtin_value(view: lp.NumericValueContext) -> str:
    return view.format_builtin()


def complete_policy(plot: lp.Scatter) -> None:
    original = lp.NumericLocale.invariant()
    locale = lp.NumericLocale.custom(
        ",",
        original.digits,
        grouping=lp.NumericGrouping(".", [3], repeat="last"),
        symbols=original.symbols,
        currency=lp.NumericCurrency("", " €"),
    )
    policy = lp.NumericFormat(
        notation="engineering",
        precision=lp.NumericPrecision.significant(4),
        decimal_scale=0,
        prefix="≈ ",
        suffix=" W",
        negative_zero="normalize",
        trailing_zeros="trim",
        rounding="nearest_away",
        multiplier=1.0,
        rate="value",
        locale=locale,
        grouping=True,
        sign="negative_only",
        symbol="none",
        field=lp.NumericField(6, fill=" ", alignment="right"),
        prefixes=lp.NumericPrefixes.none(),
        unit=None,
        explanation=lp.NumericExplanation.automatic(),
    )
    assert_type(policy.precision, lp.NumericPrecision)
    assert_type(policy.decimal_scale, int)
    assert_type(policy.prefix, str | None)
    assert_type(policy.copy_with(field=None, unit=None), lp.NumericFormat)
    callback = policy.copy_with(value_function=_builtin_value)
    assert_type(callback, lp.NumericFormat)
    assert_type(callback.copy_with(value_function=None), lp.NumericFormat)
    assert_type(
        lp.NumericFormat.from_specifier(",.2f").copy_with(
            precision=lp.NumericPrecision.fractional(1)
        ),
        lp.NumericFormat,
    )
    assert_type(plot.y_tick_format(callback), lp.Scatter)
    policy.precision = lp.NumericPrecision.fractional(0)  # type: ignore[misc]
    policy.fractional_digits  # type: ignore[attr-defined]
    lp.NumericFormat(fractional_digits=2)  # type: ignore[call-overload]
    policy.copy_with(field_width=6)  # type: ignore[call-overload]
    policy.copy_with(prefix=None)  # type: ignore[call-overload]
    policy.copy_with(value_function=None, set_function=None)  # type: ignore[call-overload]
    lp.NumericFormat(rounding="half_up")  # type: ignore[call-overload]
    lp.NumericField(6).width  # type: ignore[attr-defined]


def native_value_branches() -> None:
    precision = lp.NumericPrecision.adaptive_significant(1, 5)
    assert_type(precision.copy_with(minimum=2), lp.NumericPrecision)
    assert_type(precision.minimum, int | None)
    assert_type(precision.digits, int | None)
    # Standalone enum construction is not format-attachment validation.
    assert_type(lp.NumericPrecision.significant(0), lp.NumericPrecision)
    factor = lp.NumericFactor.power_of_ten("-" + "9" * 80)
    assert_type(factor.exponent, str | None)
    assert_type(factor.copy_with(exponent="123"), lp.NumericFactor)
    assert_type(lp.NumericPrefixes.factor(factor), lp.NumericPrefixes)
    rule = lp.NumericCompactRule(-1.0, 1000.0, prefix="", suffix="k")
    assert_type(rule.copy_with(threshold=0.0), lp.NumericCompactRule)
    assert_type(
        lp.NumericPrefixes.compact(
            [rule], selection=0, promotion="keep_threshold"
        ),
        lp.NumericPrefixes,
    )
    assert_type(
        lp.NumericPrefixes.si(
            lp.SiPrefix.from_exponent(30), promotion="keep_threshold"
        ),
        lp.NumericPrefixes,
    )
    lp.NumericPrecision()  # type: ignore[call-arg]
    precision.copy_with(minimum=None)  # type: ignore[arg-type]
    factor.copy_with(exponent=123)  # type: ignore[arg-type]
    lp.NumericPrefixes.si(0, promotion="keep_threshold")  # type: ignore[arg-type]
    lp.NumericPrefixes.none().mode  # type: ignore[attr-defined]


def locale_and_explanation(view: lp.NumericSetContext) -> None:
    locale = lp.NumericLocale.named("de-DE")
    assert_type(locale.copy_with(grouping=None), lp.NumericLocale)
    assert_type(locale.digits, tuple[str, ...])
    assert_type(locale.currency, lp.NumericCurrency)
    assert_type(
        lp.NumericLocale.custom(
            ".",
            locale.digits,
            grouping=None,
            symbols=locale.symbols,
            currency=locale.currency,
        ),
        lp.NumericLocale,
    )
    unit = lp.NumericUnit("mV", placement="shared", separator="")
    assert_type(unit.copy_with(text="V"), lp.NumericUnit)
    assert_type(unit.placement, Literal["per_label", "shared"])
    authored = lp.NumericExplanation.text(
        lp.Text("Authored explanation", layout=lp.TextLayout()),
        layout=None,
    )
    assert_type(authored.copy_with(style=None), lp.NumericExplanation)
    assert_type(
        lp.NumericLabels(
            ["first", "second"], explanation="Details", layout=lp.TextLayout()
        ),
        lp.NumericLabels,
    )
    lp.NumericLocale()  # type: ignore[call-arg]
    lp.NumericLocale.custom(  # type: ignore[call-arg]
        ".", locale.digits, symbols=locale.symbols, currency=locale.currency
    )
    locale.copy_with(symbols=None)  # type: ignore[arg-type]
    lp.NumericFormat(locale=view.locale)  # type: ignore[call-overload]
    unit.copy_with(placement="automatic")  # type: ignore[arg-type]
    authored.copy_with(text="Replacement")  # type: ignore[call-arg]


def locator_programs(plot: lp.Scatter) -> None:
    exact = lp.TickGeneration.exact_count(0, space="data", singleton="upper")
    assert_type(
        exact.copy_with(singleton=1.0, maximum_count=None), lp.TickGeneration
    )
    interval = lp.TickGeneration.interval(
        0.5, -1.0, space="scale", requirement="fixed"
    )
    assert_type(
        interval.interval_parameters,
        tuple[
            float,
            float,
            Literal["data", "scale"],
            Literal["fixed", "preferred"],
        ]
        | None,
    )
    assert_type(lp.TickGeneration.log10_decades(2).decade_step, int | None)
    assert_type(
        lp.TickGeneration.multiplicative(1.0, 0.5, requirement="preferred"),
        lp.TickGeneration,
    )
    assert_type(plot.x_tick_generation(interval), lp.Scatter)
    exact.copy_with(singleton=None)  # type: ignore[arg-type]
    lp.TickGeneration.interval(
        1, 0, space="screen", requirement="fixed"  # type: ignore[arg-type]
    )
