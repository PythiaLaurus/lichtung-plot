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


def circles(root, color):
    return [
        node for node in root.iter(f"{SVG}circle")
        if node.get("fill", "").lower() == color.lower()
    ]


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


def test_line_layer_snapshots_independent_points_and_clears_legend_text_instructions(tmp_path):
    pd = pytest.importorskip("pandas")
    parent = pd.DataFrame({"x": [0.0, 3.0], "y": [0.0, 3.0]})
    observed = pd.DataFrame({
        "obs_x": [0.5, 1.5, 2.5], "obs_y": [0.7, 1.7, 2.7],
        "label": ["first", "middle", "last"],
    })
    observed_ref = weakref.ref(observed)
    theme = lp.Theme.publication()
    legend_style = theme.typography.guides.legend_label.copy_with(color=lp.Color(11, 33, 55))
    theme = theme.copy_with(typography=theme.typography.copy_with(
        guides=theme.typography.guides.copy_with(legend_label=legend_style),
    ))
    label = lp.Text(
        "Observed", style=legend_style.copy_with(color=lp.Color(200, 10, 10)),
        layout=lp.TextLayout(visibility=lp.TextVisibility.hidden()),
    )
    plot = lp.line(parent, x="x", y="y").markers(False).theme(theme).labels(lp.Labels(mode="raw"))
    plot.point_layer(
        observed, x="obs_x", y="obs_y", labels="label",
        color=lp.Color(20, 150, 90, alpha=73), radius=5, alpha=211,
        legend_label=label, legend_label_style=None, legend_label_layout=None,
        label_text_layout=lp.TextLayout(alignment="start"),
    )
    observed["obs_x"] = [100.0, 100.0, 100.0]
    observed["label"] = ["changed"] * 3
    del observed
    gc.collect()
    assert observed_ref() is None
    plot.legend("hidden").svg_preserve_text(tmp_path / "layer-points")
    root = scene(tmp_path / "layer-points")
    points = circles(root, "#14965a")
    assert len(points) == 3 and len({node.get("cx") for node in points}) == 3
    assert [round(float(node.get("fill-opacity", "1")) * 255) for node in points] == [211] * 3
    assert {"first", "middle", "last"} <= set(text_values(root))
    # Only this new layer's legend Text instructions are cleared. Theme is live.
    plot.legend("visible").svg_preserve_text(tmp_path / "layer-guide")
    labels = [
        node for node in scene(tmp_path / "layer-guide").iter(f"{SVG}text")
        if "".join(node.itertext()) == "Observed"
    ]
    assert len(labels) == 1 and labels[0].get("fill", "").lower() == "#0b2137"
    with pytest.raises(TypeError):
        plot.point_layer(None, x="x", y="y")


def test_bar_layer_positions_its_own_series_and_preserves_complete_rgba(tmp_path):
    pl = pytest.importorskip("polars")
    plot = lp.bar(
        category=["A", "A", "B", "B"], estimate=[2.0, 2.4, 3.0, 3.5],
        series=["control", "treated", "control", "treated"],
    ).legend("hidden")
    supplied = pl.DataFrame({
        "group": ["A", "B", "A"], "arm": ["control", "treated", "treated"],
        "observed": [1.8, 3.3, 2.2],
    })
    plot.point_layer(
        supplied, category="group", value="observed", series="arm",
        color=lp.Color(80, 90, 100, alpha=149), radius=5,
    )
    plot.svg_preserve_text(tmp_path / "positioned-observations")
    points = circles(scene(tmp_path / "positioned-observations"), "#505a64")
    assert len(points) == 3 and len({node.get("cx") for node in points}) == 3
    assert [round(float(node.get("fill-opacity", "1")) * 255) for node in points] == [149] * 3
    # Membership is a terminal native check, not a binding-time filtering pass.
    invalid = plot.copy().point_layer(
        category=["C"], value=[2.0], series=["control"],
        color=lp.Color(80, 90, 100), radius=5,
    )
    with pytest.raises(lp.DataError, match="unknown category"):
        invalid.svg(tmp_path / "unknown-layer-category")


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


def test_pairwise_facet_results_broadcast_only_explicit_literals(tmp_path):
    pl = pytest.importorskip("polars")
    source = pl.DataFrame({
        "condition": ["A", "A", "B", "B", "C", "C"] * 2,
        "response": [1.0, 1.2, 2.0, 2.2, 3.0, 3.2, 1.4, 1.6, 2.4, 2.6, 3.4, 3.6],
        "site": ["North"] * 6 + ["South"] * 6,
    })
    plot = (
        lp.boxplot(source, category="condition", value="response")
        .facet_wrap("site", 2).labels(lp.Labels(mode="raw")).legend("hidden").size(1400, 900)
    )
    results = pl.DataFrame({
        "other": ["B", "C"], "where": ["North", "South"], "code": ["weak", "strong"],
    })
    plot.significance_pairwise_in_facet_from_data(
        results, left_value="A", right="other", marks="code",
        mapping=[("weak", "*"), ("strong", "***")], facet="where",
    ).facet_scales("fixed")
    plot.svg_preserve_text(tmp_path / "facet-comparisons")
    assert panel_marks(scene(tmp_path / "facet-comparisons"), ["North", "South"]) == {
        "North": {"*": 1}, "South": {"***": 1},
    }
    with pytest.raises(lp.DataError):
        plot.significance_pairwise_in_facet_from_data(
            None, left=["A"], right=["B", "C"], mark="*",
            facet=["North", "South"],
        )
    with pytest.raises(TypeError):
        plot.significance_pairwise_from_data(None, left="condition", right_value="B", mark="*")


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


def test_complete_result_forms_precede_lookup_and_stale_append_is_not_committed(tmp_path):
    pd = pytest.importorskip("pandas")

    class Guarded(pd.DataFrame):
        def __getattribute__(self, name):
            if name == "columns" and self.__dict__.get("_forbid_lookup", False):
                raise AssertionError("complete forms must be checked before source lookup")
            if name == "iloc" and self.__dict__.get("_reenter", False):
                self.__dict__["_reenter"] = False
                owner.title("Inner result edit survives")
            return super().__getattribute__(name)

    source = Guarded({"category": ["A", "B"], "estimate": [2.0, 3.0]})
    owner = lp.bar(source, category="category", estimate="estimate").legend("hidden")
    owner.significance_attached(target_value="B", mark="ns")
    source.__dict__["_forbid_lookup"] = True
    with pytest.raises(TypeError):
        owner.significance_attached(target="category", target_value="A", mark="*")
    with pytest.raises(TypeError):
        owner.significance_attached(target="category", marks=[0.1], mark="*")
    with pytest.raises(TypeError):
        owner.significance_attached(target="category", marks=[0.1], mapping=None)
    with pytest.raises(TypeError):
        owner.point_layer(source, category="category", value="estimate", color=lp.Color(1, 2, 3))
    source.__dict__["_forbid_lookup"] = False
    source.__dict__["_reenter"] = True
    with pytest.raises(RuntimeError, match="stale"):
        owner.significance_attached(target="category", mark="*")
    owner.svg_preserve_text(tmp_path / "stale-result")
    values = text_values(scene(tmp_path / "stale-result"))
    assert "Inner result edit survives" in values and values.count("ns") == 1 and "*" not in values
