"""Exact owners, four additional physical formatter slots and Canvas composition."""

import gc
import weakref
from xml.etree import ElementTree

import pytest

import lichtung_plot as lp


SVG = "{http://www.w3.org/2000/svg}"


def _text_nodes(path):
    return list(ElementTree.parse(path).getroot().iter(f"{SVG}text"))


def _texts(path):
    return ["".join(node.itertext()) for node in _text_nodes(path)]


def _holder_ids(value):
    return [
        id(root) for root in gc.get_referents(value)
        if type(root).__name__ == "_CallbackHolder"
    ]


def test_heatmap_values_and_continuous_color_keep_distinct_same_holder_edges(tmp_path):
    reached = []

    def labels(view):
        if view.purpose == "actual_set":
            reached.append((view.role, view.owner.canvas_cell))
        return view.format_builtin()

    callback_ref = weakref.ref(labels)
    format = lp.NumericFormat.set_function(labels)
    holder = _holder_ids(format)[0]
    plot = (
        lp.heatmap(grid=[[1.0, 2.0], [3.0, 4.0]])
        .annotations(format)
        .color_label_format(format)
        .continuous_color_scale(
            lp.ContinuousColorScale.sequential(domain=lp.ColorDomain(0.0, 4.0))
        )
        .color_label("Color")
        .size(1000, 700)
    )
    assert _holder_ids(plot._owner) == [holder, holder]
    copied = plot.copy()
    assert type(copied) is lp.Heatmap and _holder_ids(copied._owner) == [holder, holder]
    canvas = (
        lp.Canvas.grid(1, 2).size(1600, 700)
        .panel(0, 0, copied).panel(0, 1, copied)
    )
    assert _holder_ids(canvas._owner) == [holder] * 4
    fresh = lp.NumericFormat.set_function(labels)
    fresh_holder = _holder_ids(fresh)[0]
    plot.color_label_format(fresh)
    assert fresh_holder != holder and _holder_ids(plot._owner) == [fresh_holder, holder]
    with pytest.raises(TypeError):
        copied.annotations(None)
    with pytest.raises(lp.DataError):
        canvas.panel(0, 0, copied)
    assert _holder_ids(copied._owner) == [holder, holder]
    assert _holder_ids(canvas._owner) == [holder] * 4
    assert reached == []

    # Hiding values retires only that slot. Continuous color still really runs.
    plot.annotations("hidden")
    assert _holder_ids(plot._owner) == [fresh_holder]
    plot.svg_preserve_text(tmp_path / "hidden-values-keeps-color")
    assert reached and {role for role, _ in reached} == {"continuous_color"}
    reached.clear()
    del copied, plot, labels, format, fresh
    gc.collect()
    assert callback_ref() is not None
    output = canvas.svg_preserve_text(tmp_path / "captured-two-slot-occurrences")
    assert set(reached) == {
        ("continuous_color", (0, 0)), ("heatmap_values", (0, 0)),
        ("continuous_color", (0, 1)), ("heatmap_values", (0, 1)),
    }
    canvas.panel(0, 0, lp.heatmap(grid=[[1.0]]), overwrite=True)
    assert _holder_ids(canvas._owner) == [holder, holder]
    canvas.panel(0, 1, lp.heatmap(grid=[[2.0]]), overwrite=True)
    gc.collect()
    assert (
        _holder_ids(canvas._owner) == [] and callback_ref() is None
        and _holder_ids(output) == []
    )


def test_coefficient_labels_failure_copy_capture_and_retired_finalizer_reentry(
    tmp_path,
):
    reached = []
    retired = []
    plot = lp.coefficient(term=["a", "b"], estimate=[1.0, 2.0]).size(1000, 700)
    plot_ref = weakref.ref(plot)

    class Labels:
        def __call__(self, view):
            result = view.format_builtin()
            if view.purpose == "actual_set":
                reached.append((
                    view.role, view.owner.plot_kind, view.owner.canvas_cell,
                    tuple(value.identity.category for value in view.values),
                ))
            return result.copy_with(labels=["E" + value for value in result.labels])

        def __del__(self):
            owner = plot_ref()
            if owner is not None:
                # This must not run under the old Plot owner's mutable borrow.
                owner.title("Retired formatter").copy()
                retired.append("borrow-free")

    callback = Labels()
    callback_ref = weakref.ref(callback)
    format = lp.NumericFormat.set_function(
        callback, precision=lp.NumericPrecision.fractional(1)
    )
    holder = _holder_ids(format)[0]
    plot.estimate_labels(format)
    copied = plot.copy()
    canvas = lp.Canvas.grid(1, 1).size(1000, 700).panel(0, 0, copied)
    copied.estimate_labels(lp.NumericFormat())
    assert _holder_ids(plot._owner) == [holder]
    assert _holder_ids(copied._owner) == []
    assert _holder_ids(canvas._owner) == [holder]
    with pytest.raises(TypeError):
        plot.estimate_labels(None)
    assert _holder_ids(plot._owner) == [holder]
    del callback, format
    canvas.svg_preserve_text(tmp_path / "captured-coefficient-labels")
    assert reached and all(
        entry == ("coefficient_labels", "coefficient", (0, 0), ("a", "b"))
        for entry in reached
    )
    assert {"E1.0", "E2.0"} <= set(_texts(tmp_path / "captured-coefficient-labels.svg"))
    del canvas, copied
    assert callback_ref() is not None and retired == []
    plot.estimate_labels(lp.NumericFormat())
    gc.collect()
    assert callback_ref() is None and retired == ["borrow-free"]
    assert _holder_ids(plot._owner) == []


def test_waterfall_totals_capture_complete_policy_and_independent_label_text(tmp_path):
    reached = []
    style = lp.Theme.publication().typography.annotations.waterfall_total
    blue = style.copy_with(color=lp.Color.hex("#194A75"))
    red = style.copy_with(color=lp.Color.hex("#A53131"))

    def labels(view):
        result = view.format_builtin()
        if view.purpose == "actual_set":
            reached.append((
                view.role, view.owner.canvas_cell,
                tuple(value.identity.kind for value in view.values),
                tuple(value.operand.binary64 for value in view.values),
            ))
        return result.copy_with(labels=["T" + value for value in result.labels])

    callback_ref = weakref.ref(labels)
    format = lp.NumericFormat.from_specifier(".1f").copy_with(set_function=labels)
    holder = _holder_ids(format)[0]
    original = (
        lp.waterfall(category=["Up", "Down"], delta=[2.0, -1.0], base=10.0)
        .x_limits(9.0, 13.0)
        .total_labels(lp.Text("Base", style=blue), lp.Text("Terminal", style=red))
        .total_text_layout(lp.TextLayout(alignment="start"))
        .total_format(format)
    )
    copied = original.copy()
    canvas = lp.Canvas.grid(1, 1).size(1200, 800).panel(0, 0, copied)
    original.total_format(lp.NumericFormat())
    copied.total_format(lp.NumericFormat())
    assert _holder_ids(original._owner) == _holder_ids(copied._owner) == []
    assert _holder_ids(canvas._owner) == [holder]
    del labels, format, copied
    assert callback_ref() is not None
    canvas.svg_preserve_text(tmp_path / "captured-waterfall-totals")
    assert reached and all(
        entry == (
            "waterfall_totals", (0, 0),
            ("waterfall_base", "waterfall_terminal"), (10.0, 11.0),
        )
        for entry in reached
    )
    nodes = {
        "".join(node.itertext()): node
        for node in _text_nodes(tmp_path / "captured-waterfall-totals.svg")
    }
    assert nodes["Base T10.0"].attrib["fill"] == "#194A75"
    assert nodes["Terminal T11.0"].attrib["fill"] == "#A53131"
    canvas.panel(0, 0, original, overwrite=True)
    gc.collect()
    assert callback_ref() is None and _holder_ids(canvas._owner) == []


def test_exact_pure_policies_text_coordinates_and_nonuniversal_capabilities(tmp_path):
    paint = lp.Color.hex("#194A75")
    categorical = lp.CategoricalColorScale.from_map(
        lp.CategoricalColorMap([("a", lp.OpaqueColor(25, 74, 117))])
    )
    path = (
        lp.line_path(
            x=[1.0, 0.0, 1.0], y=[1.0, 2.0, 3.0],
            group=["g"] * 3, hue=["a"] * 3,
        )
        .style("hue").markers(False).categorical_color_scale(categorical)
        .guide_segment(0.0, 1.0, 1.0, 3.0, color=paint, width=1.0, pattern="dashed")
        .text_annotation(
            lp.TextPosition.coordinates(
                x=lp.TextCoordinate.data(lp.DataCoordinate.numeric(0.5)),
                y=lp.TextCoordinate.data_area_fraction(0.9),
            ),
            "Mixed reference",
        )
        .title("Source path")
    )
    interaction = (
        lp.interaction(
            condition=["before", "after"], estimate=[1.0, 2.0], group=["g", "g"],
            lower=[0.5, 1.5], upper=[1.5, 2.5], hue=["a", "a"],
        )
        .connectors(True).markers(True)
        .styles([lp.InteractionStyle(
            "g", point_shape="triangle_up", line_pattern="dashed"
        )])
        .legend("visible").legend_composition("separate").title("Styled interaction")
    )
    histogram = (
        lp.histogram(samples=[0.0, 0.5, 1.0, 1.5, 2.0])
        .bins(width=0.5).stat("density").composite("kde_and_rug")
        .title("Sample density")
    )
    forest = (
        lp.forest(
            term=["a", "b"], estimate=[0.5, 2.0], lower=[0.25, 1.0],
            upper=[1.0, 4.0], reference=1.0,
        )
        .scale("log10").x_tick_generation(lp.TickGeneration.log10_decades(1))
        .estimate_detail_text_layout(lp.TextLayout(alignment="start"))
        .title("Log reference")
    )
    matrix = (
        lp.matrix(variables=[[0.0, 0.0], [1.0, 2.0], [2.0, 1.0], [3.0, 3.0]])
        .matrix_row_text_layout(lp.TextLayout(alignment="end"))
        .matrix_column_text_layout(lp.TextLayout(alignment="start"))
        .labels(lp.Labels(mode="raw")).title("Matrix title")
    )
    bars = (
        lp.horizontal_bar(category=["Positive", "Negative"], estimate=[2.0, -1.0])
        .spacing("compact").category_order("value_ascending")
        .color_by_sign().zero_line()
        .significance_mark_style(lp.SignificanceMarkStyle())
        .title("Signed estimates")
    )
    assert not hasattr(matrix, "x_limits") and not hasattr(matrix, "facet_row")
    assert not hasattr(matrix, "text_annotation") and not hasattr(
        matrix, "guide_horizontal"
    )
    assert not hasattr(bars, "hue") and not hasattr(forest, "estimate_labels")
    with pytest.raises(TypeError):
        histogram.bins(count=4, width=0.5)
    with pytest.raises(TypeError):
        interaction.legend("automatic", position="auto")
    canvas = (
        lp.Canvas.grid(2, 3).size(3000, 1800)
        .panel(0, 0, path).panel(0, 1, interaction).panel(0, 2, histogram)
        .panel(1, 0, forest).panel(1, 2, bars)
    )
    # Matrix owns its interior layout; rejection stays at native render preflight.
    nested = canvas.copy().panel(1, 1, matrix)
    with pytest.raises(lp.RenderError, match="Matrix manages its own canvas"):
        nested.svg(tmp_path / "nested-matrix-rejected")
    assert not (tmp_path / "nested-matrix-rejected.svg").exists()
    matrix.size(1200, 1000).svg_preserve_text(tmp_path / "standalone-matrix-policy")
    assert "Matrix title" in _texts(tmp_path / "standalone-matrix-policy.svg")
    canvas.svg_preserve_text(tmp_path / "exact-pure-policies")
    assert {
        "Mixed reference", "Source path", "Styled interaction", "Sample density",
        "Log reference", "Signed estimates",
    } <= set(_texts(tmp_path / "exact-pure-policies.svg"))


def test_canvas_append_text_panel_ranges_and_plot_overwrite_are_native(tmp_path):
    typography = lp.Theme.publication().typography
    root_style = typography.root.title.copy_with(color=lp.Color.hex("#194A75"))
    title_style = typography.composition.text_panel_title.copy_with(
        color=lp.Color.hex("#A53131")
    )
    body_style = typography.composition.body.copy_with(color=lp.Color.hex("#28753C"))
    left = lp.scatter(x=[0.0, 1.0], y=[1.0, 2.0]).title("Left")
    right = lp.line(x=[2.0, 3.0], y=[2.0, 1.0]).title("Right")
    reached = []

    def labels(view):
        if view.purpose == "actual_set":
            reached.append(view.owner.canvas_cell)
        return view.format_builtin()

    coefficient = lp.coefficient(term=["a"], estimate=[1.0]).estimate_labels(
        lp.NumericFormat.set_function(labels)
    )
    canvas = (
        lp.Canvas.grid(2, 2).size(1800, 1300)
        .title(lp.Text("Canvas root", style=root_style))
        .subtitle(lp.Text(
            "Canvas subtitle", style=title_style, layout=lp.TextLayout(alignment="end")
        ))
        .panel_title_alignment("end").panel_subtitle_alignment("start")
        .panel(0, 0, left).panel(0, 1, right)
        .text_panel(
            1, 0, title=lp.Text("Panel title", style=title_style),
            body=lp.Text("Independent body", style=body_style),
        )
        .panel(1, 1, coefficient)
        .column_header(0, 2, "Column header", style=None, layout=None)
        .row_header(0, 2, "Row header", style=None)
        .row_title(1, "Lower row", layout=lp.TextLayout(alignment="end"))
        .category_labels("every_plot")
        .x_range_group([(0, 0), (0, 1)])
        .legend_policy("per_plot").legend_composition("combined")
        .guide_text_layout("entry", lp.TextLayout())
    )
    roots_before = _holder_ids(canvas._owner)
    with pytest.raises(lp.DataError):
        canvas.text_panel(1, 1, title="Collision", body="Not committed")
    with pytest.raises(lp.DataError):
        canvas.column_header(1, 1, "Overlap")
    assert _holder_ids(canvas._owner) == roots_before
    assert reached == []
    copied = canvas.copy().clear_subtitle_style().clear_subtitle_layout()
    copied.svg_preserve_text(tmp_path / "canvas-native-text-and-ranges")
    nodes = {
        "".join(node.itertext()): node
        for node in _text_nodes(tmp_path / "canvas-native-text-and-ranges.svg")
    }
    assert nodes["Canvas root"].attrib["fill"] == "#194A75"
    assert nodes["Canvas subtitle"].attrib["fill"] != "#A53131"
    assert nodes["Panel title"].attrib["fill"] == "#A53131"
    assert nodes["Independent body"].attrib["fill"] == "#28753C"
    assert {"Column header", "Row header", "Lower row"} <= nodes.keys()
    assert "Collision" not in nodes and "Overlap" not in nodes
    assert reached and all(cell == (1, 1) for cell in reached)
    canvas.panel(1, 0, coefficient, overwrite=True)
    assert len(_holder_ids(canvas._owner)) == len(roots_before) + 1
    reached.clear()
    canvas.svg_preserve_text(tmp_path / "plot-overwrites-text-panel")
    retained = {
        "".join(node.itertext()): node
        for node in _text_nodes(tmp_path / "plot-overwrites-text-panel.svg")
    }
    assert "Panel title" not in retained
    assert retained["Canvas subtitle"].attrib["fill"] == "#A53131"
    assert set(reached) == {(1, 0), (1, 1)}

    # First-column suppression has its real native horizontal-bar-domain rule;
    # it is not a universal layout toggle for the mixed prose scene above.
    bars = lp.horizontal_bar(
        category=["A", "B"], estimate=[1.0, 2.0]
    ).labels(lp.Labels(mode="raw"))
    shared = (
        lp.Canvas.grid(1, 2).size(1800, 800)
        .panel(0, 0, bars).panel(0, 1, bars.copy())
        .category_labels("first_column_per_row").x_range_group([(0, 0), (0, 1)])
    )
    shared.svg_preserve_text(tmp_path / "native-shared-category-domains")
    rendered = _texts(tmp_path / "native-shared-category-domains.svg")
    assert rendered.count("A") == rendered.count("B") == 1


def test_geometry_raincloud_title_uses_native_text_slot_and_explicit_clears(tmp_path):
    # Invalid display content must be rejected before even classifying a source.
    with pytest.raises(TypeError, match="text must be str or Text"):
        lp.histogram(
            object(), bin_lower=lp.col["lo"], bin_upper=lp.col["hi"],
            bin_density=lp.col["density"], kde_x=[0.0, 1.0], kde_y=[0.0, 0.0],
            rug_x=[0.5], rug_baseline=[0.0], rug_top=[0.1], value_label=None,
        )
    with pytest.raises(TypeError, match="text must be str or Text"):
        lp.raincloud(
            object(), category=lp.col["category"], q1=[1.0], median=[2.0], q3=[3.0],
            density_category=["a"] * 3, density_y=[0.0, 2.0, 4.0],
            normalized_width=[0.0, 1.0, 0.0], point_category=["a"],
            point_y=[2.0], normalized_strip=[0.5], hue_label=None,
        )
    red = lp.Theme.publication().typography.guides.legend_title.copy_with(
        color=lp.Color.hex("#A53131")
    )
    plot = lp.raincloud(
        category=["a"], q1=[1.0], median=[2.0], q3=[3.0], hue=["control"],
        density_category=["a", "a", "a"], density_y=[0.0, 2.0, 4.0],
        normalized_width=[0.0, 1.0, 0.0], density_hue=["control"] * 3,
        point_category=["a", "a"], point_y=[1.0, 3.0],
        normalized_strip=[0.25, 0.75], point_hue=["control"] * 2,
        category_label="Condition", value_label="Measurement",
        hue_label=lp.Text(
            "Treatment", style=red, layout=lp.TextLayout(alignment="end")
        ),
    ).legend("visible")
    cleared = plot.copy().hue_label("Treatment", style=None, layout=None)
    peer = plot.copy().clear_hue_label_style().clear_hue_label_layout()
    canvas = (
        lp.Canvas.grid(1, 2).size(1800, 900).legend_policy("shared")
        .panel(0, 0, cleared).panel(0, 1, peer)
    )
    canvas.svg_preserve_text(tmp_path / "raincloud-authored-guide-title")
    titles = [
        node for node in _text_nodes(tmp_path / "raincloud-authored-guide-title.svg")
        if "".join(node.itertext()) == "Treatment"
    ]
    assert titles and all(node.attrib["fill"] != "#A53131" for node in titles)
    assert {"Condition", "Measurement"} <= set(
        _texts(tmp_path / "raincloud-authored-guide-title.svg")
    )
    # Same content with a different complete Text request remains a native conflict.
    with pytest.raises(lp.RenderError, match="requires compatible semantic titles"):
        canvas.copy().panel(0, 1, plot, overwrite=True).svg(tmp_path / "title-conflict")
    assert not (tmp_path / "title-conflict.svg").exists()
