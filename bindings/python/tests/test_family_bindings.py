"""Shape/provenance-sensitive base edits through actual public exact owners."""

import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp


SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ET.parse(path.with_suffix(".svg")).getroot().iter(f"{SVG}text")
    ]


def test_wide_roles_ids_and_bands_keep_constructor_authority(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({
        "time": [10.0, 30.0, 90.0], "a": [1.0, 2.0, 3.0], "b": [2.0, 3.0, 4.0],
        "lo_a": [0.0, 1.0, 2.0], "lo_b": [1.0, 2.0, 3.0],
        "fresh": ["old"] * 3,
    })
    plot = lp.line(frame, x="time", y=lp.cols["a", "b"], hue=None, group=None)
    # Equal total lengths do not erase single/many or per-dimension authority.
    with pytest.raises(lp.DataError):
        plot.band([0.0] * 6, [5.0] * 6)
    with pytest.raises(lp.DataError):
        plot.source_rows([10, 11, 12])
    with pytest.raises(lp.DataError):
        plot.source_rows_cells(np.arange(6, dtype=np.uint64).reshape(2, 3))
    plot.source_rows_cells(np.arange(6, dtype=np.uint64).reshape(3, 2))
    plot.band(lp.cols["lo_a", "lo_b"], [[2, 3], [3, 4], [4, 5]])
    # A later argument's container cannot turn this ND-origin plot into PL-origin.
    pl = pytest.importorskip("polars")
    plot.hue_cells(pl.Series("h", ["a-hue", "b-hue"] * 3))
    plot.group_columns().labels(lp.Labels(mode="raw")).column_labels(["First", "Second"])
    frame["fresh"] = ["live"] * 3
    plot.hue(lp.col["fresh"]).title("Replaced hue")
    canvas = lp.Canvas.grid(1, 2).size(2400, 1000).panel(0, 0, plot)
    plot.hue_columns().title("Explicit generated hue")
    canvas.panel(0, 1, plot)
    canvas.svg_preserve_text(tmp_path / "wide-role-state")
    rendered = set(texts(tmp_path / "wide-role-state"))
    assert {"First", "Second", "Replaced hue", "Explicit generated hue"} <= rendered
    assert all(
        any(group in text and "live" in text for text in rendered)
        for group in ("First", "Second")
    )
    assert not any("old" in text for text in rendered)


def test_core_and_adapted_facet_scales_reuse_bound_observations(tmp_path):
    pd = pytest.importorskip("pandas")
    core = lp.scatter(x=[0, 1, 2, 3], y=[1, 2, 3, 4]).labels(lp.Labels(mode="raw")).facet_row(
        ["Bound A", "Bound A", "Bound B", "Bound B"],
    ).facet_scales("free_y").title("Core facet")
    frame = pd.DataFrame({
        "x": [0.0, 1.0, 2.0, 3.0], "a": [1.0, 2.0, 3.0, 4.0],
        "b": [2.0, 3.0, 4.0, 5.0], "panel": ["Saved A"] * 2 + ["Saved B"] * 2,
    })
    adapted = lp.line(frame, x="x", y=lp.cols["a", "b"]).facet_row("panel")
    frame["panel"] = ["Must not be reread"] * 4
    adapted.labels(lp.Labels(mode="raw")).facet_scales("free_y").title("Cached facet")
    # Native generic facets are standalone plots, not composed-Canvas children.
    core.size(1600, 1400).svg_preserve_text(tmp_path / "facet-core")
    adapted.size(1600, 1400).svg_preserve_text(tmp_path / "facet-adapted")
    rendered = texts(tmp_path / "facet-core") + texts(tmp_path / "facet-adapted")
    assert all(
        any(f" = {level}" in text for text in rendered)
        for level in ("Bound A", "Bound B", "Saved A", "Saved B")
    )
    assert not any("Must not be reread" in text for text in rendered)
    with pytest.raises(lp.DataError):
        lp.scatter(x=[0, 1], y=[1, 2]).facet_scales("free")
    with pytest.raises(lp.DataError):
        lp.line(x=[0, 1], y=[1, 2]).facet_columns_wrap(2)


def test_explicit_mixed_cell_and_generated_facet_scopes_are_real_consumers(tmp_path):
    base = lp.line(x=[0, 1], y=[[1, 2], [2, 3]]).legend("hidden").labels(lp.Labels(mode="raw"))
    cells = ["Cell A", "Cell B", "Cell A", "Cell B"]
    plots = [
        base.copy().facet_grid(
            ["Rows upper", "Rows lower"], ["Rows left", "Rows right"],
        ).title("Rows"),
        base.copy().facet_grid_row_cells(
            ["Rowcell first", "Rowcell second"] * 2, ["Rowcell left", "Rowcell right"],
        ).title("Row cells"),
        base.copy().facet_grid_column_cells(
            ["Columncell upper", "Columncell lower"], ["Columncell first", "Columncell second"] * 2,
        ).title("Column cells"),
        base.copy().facet_grid_cells(
            ["Cell first", "Cell second"] * 2, ["Cell left"] * 2 + ["Cell right"] * 2,
        ).title("Cells"),
        base.copy().column_labels(["Grid variable first", "Grid variable second"]).facet_grid_columns_row(
            ["Grid external first", "Grid external second"],
        ).title("Generated row"),
        base.copy().column_labels(["Wrap variable first", "Wrap variable second"]).facet_columns_wrap(
            2,
        ).title("Generated wrap"),
    ]
    expected_facets = [
        {"Rows upper", "Rows lower", "Rows left", "Rows right"},
        {"Rowcell first", "Rowcell second", "Rowcell left", "Rowcell right"},
        {"Columncell upper", "Columncell lower", "Columncell first", "Columncell second"},
        {"Cell first", "Cell second", "Cell left", "Cell right"},
        {"Grid variable first", "Grid variable second", "Grid external first", "Grid external second"},
        {"Wrap variable first", "Wrap variable second"},
    ]
    # Scope-specific values cannot come from titles or another dispatch. Hidden
    # legends also prevent generated-column aliases from masking a missing facet.
    for index, (plot, expected) in enumerate(zip(plots, expected_facets)):
        path = tmp_path / f"facet-scope-{index}"
        plot.size(1800, 1400).svg_preserve_text(path)
        rendered = texts(path)
        assert all(
            any(f" = {level}" in text for text in rendered)
            for level in expected
        )
    with pytest.raises(lp.DataError):
        base.copy().facet_row(cells)  # row scope repeats; length does not pick cells.


def test_distribution_series_cells_and_generated_reset_do_not_change_sample_origin(tmp_path):
    np = pytest.importorskip("numpy")
    samples = np.array([[1.0, 2.0], [2.0, 3.0], [1.5, 2.5], [3.0, 4.0]])
    plot = lp.boxplot(category=["Group"] * 4, value=samples, series=None)
    plot.series(["replacement"] * 4).title("Row series")
    canvas = lp.Canvas.grid(1, 2).size(2400, 1000).panel(0, 0, plot)
    plot.series_cells(["first", "second"] * 4).series_columns().column_labels(
        ["Variable A", "Variable B"],
    ).title("Generated series")
    canvas.panel(0, 1, plot)
    canvas.svg_preserve_text(tmp_path / "positioned-series")
    assert {"Replacement", "Variable A", "Variable B"} <= set(texts(tmp_path / "positioned-series"))
    with pytest.raises(lp.DataError):
        plot.series(["not one per row"])


def test_heatmap_grid_labels_boolean_borders_and_cell_ids_keep_shape(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"a": [1.0, 2.0], "b": [3.0, 4.0],
                          "row": ["Upper row", "Lower row"], "one_flag": [True, False]})
    plot = lp.heatmap(frame, grid=lp.cols["a", "b"]).labels(lp.Labels(mode="raw"))
    plot.x_labels(["A", "B"]).row_labels("row")
    plot.heatmap_highlight_cells(np.array([[True, False], [False, True]]))
    plot.source_rows_cells([[10, 11], [12, 13]])
    with pytest.raises(lp.DataError):
        plot.heatmap_highlight(lp.cols["one_flag"])
    with pytest.raises(lp.DataError):
        plot.heatmap_highlight(np.ones((1, 4), dtype=bool))
    with pytest.raises(lp.DataError):
        plot.heatmap_highlight(np.empty((2, 0), dtype=bool))
    with pytest.raises(TypeError):
        plot.heatmap_highlight([0, 1, 0, 1])
    with pytest.raises(lp.DataError):
        plot.source_rows_cells([[10, 10], [12, 13]])
    plot.svg_preserve_text(tmp_path / "grid-metadata")
    rendered = texts(tmp_path / "grid-metadata")
    assert {"Upper row", "Lower row", "A", "B"} <= set(rendered)


def test_nullable_estimates_are_completed_by_three_explicit_status_forms(tmp_path):
    pd = pytest.importorskip("pandas")
    pl = pytest.importorskip("polars")
    frame = pd.DataFrame({
        "term": ["available", "unavailable"],
        "estimate": pd.Series([1.0, None], dtype="Float64"),
        "lower": pd.Series([0.5, None], dtype="Float64"),
        "upper": pd.Series([1.5, None], dtype="Float64"),
        "available": pd.Series([True, False], dtype="boolean"),
        "status": ["fit", "missing"],
    })
    point = lp.point_interval(
        category=["available", "unavailable"], estimate=[1.0, None],
        lower=[0.5, None], upper=[1.5, None],
    ).estimate_status(["estimated", "not_estimable"]).title("Literal status")
    coefficient = lp.coefficient(
        frame, term="term", estimate="estimate", lower="lower", upper="upper",
    ).estimate_status_boolean("available").title("Boolean status")
    forest = lp.forest(
        frame, term="term", estimate="estimate", lower="lower", upper="upper", reference=1,
    ).estimate_status_categorical(
        "status", [("fit", "estimated"), ("missing", "not_estimable")],
    ).title("Mapped status")
    # Exercise the Core-origin mapping path without a public CategoryValues reader.
    core_mapped = lp.point_interval(
        category=["available", "unavailable"], estimate=[1.0, None],
        lower=[0.5, None], upper=[1.5, None],
    ).estimate_status_categorical(
        ["fit", "missing"], [("fit", "estimated"), ("missing", "not_estimable")],
    ).title("Core categorical status")
    with pytest.raises(lp.DataError):
        forest.estimate_status_categorical("status", [("fit", "estimated"), ("fit", "not_estimable")])
    with pytest.raises(lp.DataError):
        coefficient.estimate_status_boolean([True, None])
    with pytest.raises(lp.DataError):
        coefficient.estimate_status_boolean(pl.Series("availability", [None, None]))
    canvas = lp.Canvas.grid(2, 2).size(3200, 2400).panel(
        0, 0, point,
    ).panel(0, 1, coefficient).panel(1, 0, forest).panel(1, 1, core_mapped)
    canvas.svg_preserve_text(tmp_path / "explicit-status")
    assert {"Literal status", "Boolean status", "Mapped status", "Core categorical status"} <= set(
        texts(tmp_path / "explicit-status")
    )
    assert any("not estimable" in text.lower() for text in texts(tmp_path / "explicit-status"))


def test_new_channel_key_reentry_keeps_the_reentrant_edit_not_the_stale_pair(tmp_path):
    pd = pytest.importorskip("pandas")

    class Key:
        plot = None

        def __hash__(self):
            return 721

        def __str__(self):
            if self.plot is not None:
                self.plot.title("Reentrant title")
            return "Fresh grouping"

    key = Key()
    frame = pd.DataFrame({"x": [0.0, 1.0, 2.0], "y": [1.0, 2.0, 3.0]})
    plot = lp.line(frame, x="x", y="y")
    frame[key] = ["stale-group"] * 3
    key.plot = plot
    with pytest.raises(RuntimeError, match="stale"):
        plot.group(lp.col[key])
    key.plot = None
    plot.labels(lp.Labels(mode="raw")).svg_preserve_text(tmp_path / "channel-stale")
    rendered = set(texts(tmp_path / "channel-stale"))
    assert "Reentrant title" in rendered and "stale-group" not in rendered
