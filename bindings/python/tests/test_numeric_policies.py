"""Canonical native numeric policies and existing Scatter callback consumers."""

import ctypes
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


def test_foreign_copy_extraction_can_clear_source_after_its_pyref_ends():
    def clear(policy):
        # The same real CPython tp_clear boundary used by the GC contracts:
        # no runtime test hook or synthetic mutable policy entry point.
        get_slot = ctypes.pythonapi.PyType_GetSlot
        get_slot.argtypes = (ctypes.py_object, ctypes.c_int)
        get_slot.restype = ctypes.c_void_p
        pointer = get_slot(type(policy), 51)
        assert pointer
        assert ctypes.PYFUNCTYPE(ctypes.c_int, ctypes.py_object)(pointer)(policy) == 0

    source = lp.NumericFormat.value_function(lambda view: view.format_builtin())
    retained = holders(source)[0]

    class Multiplier:
        def __float__(self):
            clear(source)
            return 2.0

    copied = source.copy_with(multiplier=Multiplier())
    assert holders(copied)[0] is retained
    with pytest.raises(RuntimeError, match="cleared"):
        source.copy_with()

    locator = lp.TickGeneration.function(lambda _: [0, 1, 2])
    retained_locator = holders(locator)[0]

    class Maximum:
        def __index__(self):
            clear(locator)
            return 3

    capped = locator.copy_with(maximum_count=Maximum())
    assert capped.maximum_count == 3
    assert holders(capped)[0] is retained_locator
    with pytest.raises(RuntimeError, match="cleared"):
        locator.copy_with()


def test_result_explanation_uses_text_intent_without_clearing_format_content(tmp_path):
    style = lp.Theme.publication().typography.axes.tick_label.copy_with(
        color=lp.Color.hex("#194A75")
    )
    layout = lp.TextLayout(alignment="end")
    authored = lp.Text("Authored", style=style, layout=layout)
    original = lp.NumericLabels(["a"], explanation=authored)
    cleared_style = original.copy_with(style=None)
    assert (
        cleared_style.explanation.source,
        cleared_style.explanation.style,
        cleared_style.explanation.layout,
    ) == ("Authored", None, layout)
    assert original.explanation == authored
    cleared_layout = original.copy_with(layout=None).explanation
    assert (
        cleared_layout.source, cleared_layout.style, cleared_layout.layout,
    ) == ("Authored", style, None)
    empty = original.copy_with(explanation="", style=style, layout=layout)
    assert (
        empty.explanation.source, empty.explanation.style, empty.explanation.layout,
    ) == ("", style, layout)
    absent = original.copy_with(explanation=None)
    with pytest.raises(TypeError):
        lp.NumericLabels(["a"], style=None)
    with pytest.raises(TypeError):
        absent.copy_with(layout=layout)

    def labels(view):
        return lp.NumericLabels(
            [f"result{index}" for index, _ in enumerate(view.values)],
            explanation=None,
        )

    configured = lp.NumericFormat.set_function(
        labels,
        unit=lp.NumericUnit("W", placement="shared"),
        explanation=lp.NumericExplanation.text(
            "Configured units", style=style, layout=layout
        ),
    )
    (
        lp.scatter(x=[0, 1, 2], y=[1, 2, 1])
        .size(1000, 700)
        .x_limits(0, 2)
        .x_tick_generation(lp.TickGeneration.exact_count(2))
        .x_tick_format(configured)
        .svg_preserve_text(tmp_path / "retained-explanation")
    )
    root = ElementTree.parse(tmp_path / "retained-explanation.svg").getroot()
    explanations = [
        node for node in root.iter(f"{SVG}text")
        if "".join(node.itertext()) == "Configured units"
    ]
    assert explanations
    assert all(node.attrib["fill"] == "#194A75" for node in explanations)


def test_locator_clones_keep_binding_caps_and_native_singleton_timing(tmp_path):
    interval = lp.TickGeneration.interval(
        0.5, 0.25, space="data", requirement="fixed"
    )
    capped = interval.copy_with(maximum_count=5)
    assert capped.interval_parameters == (0.5, 0.25, "data", "fixed")
    assert capped.copy_with(maximum_count=None).interval_parameters == (
        0.5, 0.25, "data", "fixed"
    )
    geometric = lp.TickGeneration.multiplicative(2, 0.5, requirement="fixed")
    assert geometric.copy_with(maximum_count=None).multiplicative_parameters == (
        2.0, 0.5, "fixed"
    )
    maximum = lp.TickGeneration.maximum(5)
    assert maximum.copy_with(maximum_count=None).is_automatic
    with pytest.raises(lp.DataError):
        lp.TickGeneration.target(3).copy_with(singleton="lower")
    with pytest.raises(lp.DataError):
        lp.TickGeneration.exact_count(2).copy_with(maximum_count=1)
    with pytest.raises(lp.DataError):
        geometric.copy_with(maximum_count=0)
    with pytest.raises(lp.DataError):
        lp.TickGeneration.multiplicative(2, 1, requirement="fixed")
    with pytest.raises(lp.DataError):
        lp.TickGeneration.exact_count(0, singleton=float("nan"))
    decade = lp.TickGeneration.log10_decades(2).copy_with(maximum_count=5)

    reached = {"x": [], "y": []}

    def labels(axis, prefix):
        def record(view):
            result = view.format_builtin()
            if view.purpose == "actual_set":
                reached[axis].append(result.labels)
            return result.copy_with(labels=[prefix + value for value in result.labels])
        return record

    plot = (
        lp.scatter(x=[0, 1, 2], y=[0.5, 1, 2])
        .size(1000, 700)
        .x_limits(0, 2)
        .y_limits(0.5, 2)
        .x_tick_generation(capped)
        .y_tick_generation(geometric)
        .x_tick_format(lp.NumericFormat.set_function(labels("x", "INTERVAL")))
        .y_tick_format(lp.NumericFormat.set_function(labels("y", "GEOMETRIC")))
    )
    plot.svg_preserve_text(tmp_path / "lattices")
    assert reached["x"] and reached["y"]
    assert all(values == ("0.25", "0.75", "1.25", "1.75") for values in reached["x"])
    assert all(
        len(values) == 3 and set(values) == {"0.50", "1.00", "2.00"}
        for values in reached["y"]
    )
    reached["x"].clear()
    reached["y"].clear()
    (
        plot.copy()
        .x_tick_generation(lp.TickGeneration.exact_count(0, singleton="upper"))
        .y_tick_generation(lp.TickGeneration.exact_count(1, singleton="midpoint"))
        .svg_preserve_text(tmp_path / "empty-and-singleton")
    )
    rendered = texts(tmp_path / "empty-and-singleton.svg")
    assert not any(value.startswith("INTERVAL") for value in rendered)
    assert "GEOMETRIC1.25" in rendered
    assert reached["y"] and all(values == ("1.25",) for values in reached["y"])

    # A cloned Log10-bound interval must not silently become Context-bound.
    with pytest.raises(lp.DataError):
        plot.copy().x_tick_generation(decade).svg(tmp_path / "wrong-scale")
