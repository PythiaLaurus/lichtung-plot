"""Canonical native numeric policies and existing Scatter callback consumers."""

import gc
from xml.etree import ElementTree

import pytest

import lichtung_plot as lp

SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ElementTree.parse(path).getroot().iter(f"{SVG}text")
    ]


def holders(policy):
    return [
        value
        for value in gc.get_referents(policy)
        if type(value).__name__ == "_CallbackHolder"
    ]


def test_compiled_coupled_copy_keeps_opaque_policy_and_one_callback_holder(tmp_path):
    reached = []

    def labels(view):
        result = view.format_builtin()
        if view.purpose == "actual_set":
            reached.append((result.labels, view.locale.decimal))
        return result

    original = lp.NumericFormat.from_specifier("+010.3g").copy_with(
        prefix="[",
        suffix="]",
        locale=lp.NumericLocale.named("fr-FR"),
        unit=lp.NumericUnit("W", placement="per_label"),
        set_function=labels,
    )
    derived = original.copy_with(
        notation="integer_hex_upper",
        precision=lp.NumericPrecision.fractional(0),
        symbol="radix_prefix",
    )
    assert len(holders(original)) == len(holders(derived)) == 1
    assert holders(derived)[0] is holders(original)[0]
    with pytest.raises(lp.DataError):
        derived.copy_with(
            notation="general", precision=lp.NumericPrecision.significant(3)
        )
    builtin = derived.copy_with(
        notation="general",
        precision=lp.NumericPrecision.significant(3),
        symbol="none",
        set_function=None,
    )
    assert not holders(builtin) and len(holders(derived)) == 1
    assert builtin.prefix == "[" and builtin.suffix == "]"
    assert builtin.precision.digits == 3
    with pytest.raises(TypeError):
        derived.copy_with(prefix=None)
    with pytest.raises(lp.DataError):
        derived.copy_with(suffix="")

    plot = (
        lp.scatter(x=[15, 16], y=[0, 1])
        .size(1000, 700)
        .x_limits(15, 16)
        # A finite singleton on n>=2 is retained but unused, even outside domain.
        .x_tick_generation(lp.TickGeneration.exact_count(2, singleton=100))
        .x_tick_format(derived)
        .y_ticks([(0, "low"), (1, "high")])
    )
    plot.svg_preserve_text(tmp_path / "compiled-copy")
    assert reached
    assert all(
        values == ("[+0x00F W]", "[+0x010 W]") and decimal == ","
        for values, decimal in reached
    )
    rendered = texts(tmp_path / "compiled-copy.svg")
    assert "[+0x00F W]" in rendered and "[+0x010 W]" in rendered
