"""Independent supplemental observations and caller-supplied display results."""

from collections import Counter
import gc
import weakref
import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp

SVG = "{http://www.w3.org/2000/svg}"


def scene(path):
    return ET.parse(path.with_suffix(".svg")).getroot()


def text_values(root):
    return ["".join(node.itertext()) for node in root.iter(f"{SVG}text")]


def panel_marks(root, levels):
    # Actual panel surfaces, in the native first-observed row-major facet order.
    # No fixed coordinates or generated field/clip IDs enter the expectation.
    frames = sorted(
        (
            tuple(float(node.get(key)) for key in ("x", "y", "width", "height"))
            for node in root.findall(f"{SVG}rect")
            if all(node.get(key) is not None for key in ("x", "y", "width", "height"))
        ),
        key=lambda frame: (frame[1], frame[0]),
    )
    if len(frames) != len(levels):
        raise ValueError("expected one actual panel surface per supplied facet cell")
    result = {level: Counter() for level in levels}
    for node in root.iter(f"{SVG}text"):
        glyph = "".join(node.itertext())
        if glyph not in ("ns", "*", "**", "***", "****"):
            continue
        x, y = float(node.get("x")), float(node.get("y"))
        cell = next((
            level for level, (left, top, width, height) in zip(levels, frames)
            if left <= x <= left + width and top <= y <= top + height
        ), None)
        result.setdefault(cell, Counter())[glyph] += 1
    return result


def test_attached_results_keep_independent_count_and_native_pvalue_timing(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    plot = lp.coefficient(
        term=["A", "B", "C"], estimate=[1.0, 2.0, 3.0],
    ).legend("hidden")
    supplied = pd.DataFrame({"term": ["A", "B"]})
    supplied_ref = weakref.ref(supplied)
    plot.significance_attached_from_data(
        supplied, target="term", marks=np.array([0.049, 0.05]),
    ).significance_attached(target_value="C", mark="**")
    supplied["term"] = ["unknown", "unknown"]
    del supplied
    gc.collect()
    assert supplied_ref() is None
    plot.svg_preserve_text(tmp_path / "attached-results")
    glyphs = Counter(text_values(scene(tmp_path / "attached-results")))
    assert {glyph: glyphs[glyph] for glyph in ("*", "ns", "**")} == {"*": 1, "ns": 1, "**": 1}
    with pytest.raises(lp.DataError):
        plot.significance_attached(target=["A"], marks=[0.01, 0.2])
    with pytest.raises(lp.DataError, match="duplicate map key"):
        plot.significance_attached(
            target=["A"], marks=["hit"], mapping=[("hit", "*"), ("hit", "**")],
        )
    invalid = plot.copy().significance_attached(target_value="A", mark=1.2)
    with pytest.raises(lp.DataError, match="p-value"):
        invalid.svg(tmp_path / "invalid-display-p")


def test_grid_results_use_vector_cells_and_literal_grid_scalar_without_manual_broadcast(tmp_path):
    pd = pytest.importorskip("pandas")
    rows = [
        {"phase": phase, "site": site, "group": category, "estimate": value}
        for phase in ("Before", "After")
        for site in ("One", "Two")
        for category, value in (("A", 2.0), ("B", 3.0))
    ]
    plot = (
        lp.bar(pd.DataFrame(rows), category="group", estimate="estimate")
        .facet_grid("phase", "site").labels(lp.Labels(mode="raw")).legend("hidden")
        .size(1500, 1100)
    )
    results = pd.DataFrame({
        "r": ["Before", "Before", "After", "After"],
        "c": ["One", "Two", "One", "Two"], "p": [0.03, 0.04, 0.07, 0.1],
    })
    plot.significance_attached_in_grid_from_data(
        results, target_value="B", marks="p", facet_row="r", facet_column="c",
    ).significance_attached_in_grid(
        target_value="A", mark="****", facet_value=("Before", "One"),
    )
    plot.svg_preserve_text(tmp_path / "grid-results")
    assert panel_marks(scene(tmp_path / "grid-results"), [
        ("Before", "One"), ("Before", "Two"), ("After", "One"), ("After", "Two"),
    ]) == {
        ("Before", "One"): {"*": 1, "****": 1}, ("Before", "Two"): {"*": 1},
        ("After", "One"): {"ns": 1}, ("After", "Two"): {"ns": 1},
    }
