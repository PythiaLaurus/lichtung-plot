"""Alpha declarations, original scopes and borrow-free binding-time mappers."""

from collections import Counter
import gc
import weakref
import xml.etree.ElementTree as ET

import pytest

import lichtung_plot as lp


SVG = "{http://www.w3.org/2000/svg}"


def texts(path):
    return [
        "".join(node.itertext())
        for node in ET.parse(path.with_suffix(".svg")).getroot().iter(f"{SVG}text")
    ]


def alphas(path, paint, tag=None):
    root = ET.parse(path.with_suffix(".svg")).getroot()
    nodes = root.iter(f"{SVG}{tag}") if tag else root.iter()
    return [
        round(float(node.get(f"{paint}-opacity", "1")) * 255)
        for node in nodes
        if node.get(paint) not in (None, "none")
    ]


def test_byte_inputs_explicit_single_and_theme_reset_keep_their_own_semantics(tmp_path):
    np = pytest.importorskip("numpy")
    theme = lp.Theme.publication()
    theme = theme.copy_with(marks=theme.marks.copy_with(
        points=theme.marks.points.copy_with(
            primary=theme.marks.points.primary.copy_with(fill_alpha=113),
        ),
    ))
    plot = lp.scatter(x=[0, 1, 2], y=[1, 2, 3]).theme(theme).legend("hidden")
    plot.point_alpha([17, 129, 241]).svg_preserve_text(tmp_path / "bytes")
    assert alphas(tmp_path / "bytes", "fill", "circle") == [17, 129, 241]

    with pytest.raises(TypeError):
        plot.point_alpha([True, 0, 255])
    with pytest.raises(TypeError):
        plot.point_alpha([0.0, 1.0, 2.0])
    with pytest.raises(TypeError):
        plot.point_alpha(np.array([17, 129, 241], dtype=np.int64))
    with pytest.raises(lp.DataError):
        plot.point_alpha(np.ma.array(
            np.array([17, 129, 241], dtype=np.uint8), mask=[False, True, False],
        ))
    # One ordinary member is not an implicit scalar broadcast.
    with pytest.raises(lp.DataError):
        plot.copy().point_alpha([71]).svg(tmp_path / "not-single")

    plot.point_alpha_single(input=np.array([129], dtype=np.uint8))
    plot.svg_preserve_text(tmp_path / "explicit-single")
    assert alphas(tmp_path / "explicit-single", "fill", "circle") == [129] * 3
    plot.point_alpha_theme().svg_preserve_text(tmp_path / "theme-reset")
    assert alphas(tmp_path / "theme-reset", "fill", "circle") == [113] * 3


@pytest.mark.parametrize("origin", ["ndarray", "polars"])
def test_row_cell_and_generated_column_scopes_restore_the_matching_origin(origin, tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    pl = pytest.importorskip("polars")
    columns = {
        "time": [0.0, 1.0, 2.0], "a": [1.0, 2.0, 3.0], "b": [2.0, 3.0, 4.0],
        "q": [17, 129, 241],
    }
    if origin == "ndarray":
        class NamedSource(pd.DataFrame):
            def __getattribute__(self, name):
                if name == "iloc" and self.__dict__.get("_forbid_value_reads", False):
                    raise AssertionError("direct column alpha needs identities, not source values")
                return super().__getattribute__(name)

        source = NamedSource(columns)
        source["q"] = source["q"].astype("uint8")
        column_keys = [1, 2]
    else:
        source = pl.DataFrame(columns).with_columns(pl.col("q").cast(pl.UInt8))
        column_keys = pd.Series(["a", "b"], dtype=object)
    plot = lp.line(source, x="time", y=lp.cols["a", "b"]).legend("hidden")
    plot.band(
        [[0.5, 1.5], [1.5, 2.5], [2.5, 3.5]],
        [[1.5, 2.5], [2.5, 3.5], [3.5, 4.5]],
    )
    plot.point_alpha("q").svg_preserve_text(tmp_path / "row-alpha")
    assert Counter(alphas(tmp_path / "row-alpha", "fill", "circle")) == {
        17: 2, 129: 2, 241: 2,
    }
    plot.point_alpha_cells(np.array([31, 61, 91, 121, 151, 181], dtype=np.uint8))
    if origin == "ndarray":
        source.__dict__["_forbid_value_reads"] = True
        plot.line_alpha_columns([55, 173], column_key=column_keys)
        plot.fill_alpha_single_map([10.0], log10=((1.0, 100.0), (17, 241)))
    else:
        selector = pd.DataFrame({"series_name": pd.Series(["a", "b"], dtype=object)})
        plot.line_alpha_columns_from_data(
            selector, input=[117, 117], column_key="series_name",
        )
        plot.fill_alpha_columns_map_categories(
            ["light", "dark"], column_key=column_keys,
            mapper=lambda raw: {"light": 55, "dark": 173}[raw],
        )
    plot.svg_preserve_text(tmp_path / "cell-and-column-alpha")
    assert Counter(alphas(tmp_path / "cell-and-column-alpha", "fill", "circle")) == {
        value: 1 for value in (31, 61, 91, 121, 151, 181)
    }
    if origin == "ndarray":
        assert {55, 173} <= set(alphas(tmp_path / "cell-and-column-alpha", "stroke"))
        assert 129 in alphas(tmp_path / "cell-and-column-alpha", "fill")
    else:
        assert 117 in alphas(tmp_path / "cell-and-column-alpha", "stroke")
        assert {55, 173} <= set(alphas(tmp_path / "cell-and-column-alpha", "fill"))
    # The parent's real q field must not become this independent call's context.
    with pytest.raises(TypeError):
        plot.line_alpha_columns_from_data(None, input="q", column_key=column_keys)


def test_keyed_fields_distinguish_raw_role_from_same_named_pandas_identity(tmp_path):
    pd = pytest.importorskip("pandas")
    plot = lp.line(
        x=[0.0, 1.0], y=[1.0, 2.0], hue=["h", "h"],
        group=pd.Series(["g", "g"], name="hue"),
    ).legend("hidden")
    source = pd.DataFrame({
        "hk": ["h"], "gk": ["g"], "byte": pd.Series([101], dtype="UInt8"),
    })
    source_ref = weakref.ref(source)
    plot.line_alpha_keyed_from_data(
        source, input="byte", keys=[("hue", "hk"), (lp.col["hue"], "gk")],
    )
    del source
    gc.collect()
    assert source_ref() is None
    plot.svg_preserve_text(tmp_path / "two-semantic-fields")
    assert 101 in alphas(tmp_path / "two-semantic-fields", "stroke")
    # Repeated raw fields must reach native duplicate checking, not dict collapse.
    with pytest.raises(lp.DataError):
        plot.line_alpha_keyed([101], keys=[("hue", ["h"]), ("hue", ["g"])])


def test_histogram_bin_maps_use_independent_selector_rows_and_native_thresholds(tmp_path):
    pd = pytest.importorskip("pandas")
    histogram = lp.histogram(
        bin_lower=[0.0, 1.0], bin_upper=[1.0, 2.0], bin_density=[0.5, 0.5],
        kde_x=[0.0, 1.0, 2.0], kde_y=[0.0, 1.0, 0.0],
        rug_x=[0.25, 0.5, 1.25, 1.5], rug_baseline=[0.0] * 4, rug_top=[0.1] * 4,
    ).legend("hidden")
    table = pd.DataFrame({"lo": [0.0, 1.0], "hi": [1.0, 2.0], "q": [0.49, 0.5]})
    table_ref = weakref.ref(table)
    histogram.fill_alpha_bins_map_from_data(
        table, input="q", lower="lo", upper="hi", threshold=((0.5,), (55, 173)),
    )
    del table
    gc.collect()
    assert table_ref() is None
    histogram.svg_preserve_text(tmp_path / "bin-map")
    assert {55, 173} <= set(alphas(tmp_path / "bin-map", "fill"))


def test_mappers_see_owned_inputs_and_original_errors_precede_native_and_stale_errors(tmp_path):
    np = pytest.importorskip("numpy")
    pd = pytest.importorskip("pandas")
    plot = lp.line(
        x=[0.0, 1.0], y=[1.0, 2.0], hue=["h", "h"], group=["g", "g"],
    ).legend("hidden")
    selector = pd.DataFrame({"hk": ["h"], "gk": ["g"]})
    q = np.array([0.5])
    def mutate_sources(value):
        q[:] = 0.9
        selector["hk"] = ["changed"]
        selector["gk"] = ["changed"]
        return 83 if value <= 0.5 else 171

    plot.line_alpha_keyed_map_from_data(
        selector, input=q, keys=[("hue", "hk"), ("group", "gk")],
        continuous=((0.0, 1.0), mutate_sources),
    )
    plot.svg_preserve_text(tmp_path / "owned-before-mapper")
    assert 83 in alphas(tmp_path / "owned-before-mapper", "stroke")

    owner = lp.scatter(x=[0.0, 1.0], y=[1.0, 2.0]).legend("hidden")
    owner.point_alpha_single(value=71)
    original = LookupError("original mapper exception")
    failed_calls = []

    def fail(value):
        failed_calls.append(value)
        owner.title("Inner edit survives failure")
        raise original

    with pytest.raises(LookupError) as raised:
        # Missing expansion is a later producer error; it must not mask the PyErr.
        owner.point_alpha_columns_map(
            [0.5], column_key=[0], continuous=((0.0, 1.0), fail),
        )
    assert raised.value is original and len(failed_calls) == 1

    def reenter(value):
        owner.title("Inner edit survives stale candidate")
        return 200

    with pytest.raises(RuntimeError, match="stale"):
        owner.point_alpha_map([0.5, 0.5], continuous=((0.0, 1.0), reenter))
    owner.svg_preserve_text(tmp_path / "stale-safe-alpha")
    assert "Inner edit survives stale candidate" in texts(tmp_path / "stale-safe-alpha")
    assert alphas(tmp_path / "stale-safe-alpha", "fill", "circle") == [71, 71]


def test_categorical_declaration_survives_and_mapper_is_not_a_persistent_slot(tmp_path):
    reached = []

    class Mapper:
        def __call__(self, raw):
            reached.append(raw)
            return {"low": 55, "high": 173}[raw]

    mapper = Mapper()
    mapper_ref = weakref.ref(mapper)
    plot = lp.scatter(x=[0.0, 1.0, 2.0], y=[1.0, 2.0, 3.0]).labels(lp.Labels(mode="raw"))
    plot.point_alpha_map_categories(["low", "high", "low"], mapper=mapper)
    del mapper
    gc.collect()
    assert mapper_ref() is None and reached == ["low", "high"]
    plot.size(1400, 1000).svg_preserve_text(tmp_path / "alpha-declaration")
    assert {"low", "high"} <= set(texts(tmp_path / "alpha-declaration"))
    assert reached == ["low", "high"]


def test_mapping_completeness_is_checked_before_fresh_source_lookup():
    pd = pytest.importorskip("pandas")

    class Guarded(pd.DataFrame):
        def __getattribute__(self, name):
            if name == "columns" and self.__dict__.get("_armed", False):
                raise AssertionError("source lookup must not precede complete-form checks")
            return super().__getattribute__(name)

    source = Guarded({"x": [0.0, 1.0], "y": [1.0, 2.0], "q": [0.2, 0.8]})
    plot = lp.scatter(source, x="x", y="y")
    source.__dict__["_armed"] = True
    with pytest.raises(TypeError):
        plot.point_alpha_map(
            "q", linear=((0.0, 1.0), (17, 241)), log10=((0.1, 1.0), (17, 241)),
        )
    with pytest.raises(TypeError):
        plot.point_alpha_single(value=71, input="q")
