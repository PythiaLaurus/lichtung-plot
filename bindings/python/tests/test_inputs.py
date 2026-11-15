"""Shared four-route contracts through actual public owners and Scatter consumers."""

import gc
from pathlib import Path
import weakref
import xml.etree.ElementTree as ET

import pytest

from lichtung_plot import Canvas, DataError, Labels, TickGeneration, TickPositions, col, cols, scatter


SVG = "{http://www.w3.org/2000/svg}"


def scene(plot, path: Path):
    plot.svg_preserve_text(path)
    root = ET.parse(path.with_suffix(".svg")).getroot()
    text = ["".join(node.itertext()) for node in root.iter(f"{SVG}text")]
    legend = {
        node for group in root.iter(f"{SVG}g")
        if group.get("id", "").startswith("legend")
        for node in group.iter(f"{SVG}circle")
    }
    points = [(node.get("cx"), node.get("cy")) for node in root.iter(f"{SVG}circle")
              if node not in legend]
    return text, points


def registry(plot):
    dictionaries = [value for value in gc.get_referents(plot._owner) if isinstance(value, dict)]
    assert len(dictionaries) == 1
    return dictionaries[0]


def raw_for(plot, key):
    return next(raw for raw, original in registry(plot).items() if original == key)


def test_scientific_combinations_are_checked_before_foreign_resolution():
    class Poison:
        def __getattribute__(self, _):
            raise AssertionError("argument combination must be rejected first")

    with pytest.raises(TypeError):
        scatter(Poison(), x=col[Poison()], y=cols[Poison()], by_row_position=True)
    with pytest.raises(TypeError):
        scatter(Poison(), x=col[Poison()], numeric_only=True)
    with pytest.raises(TypeError):
        scatter(Poison(), y=cols[Poison()], by_row_position=True, numeric_only=True)
    with pytest.raises(TypeError):
        scatter(by_row_position=True, numeric_only=True)
    with pytest.raises(TypeError):
        scatter(x=None, y=[0, 1])


def test_core_native_and_later_foreign_roles_keep_delayed_alignment(tmp_path):
    np = pytest.importorskip("numpy")
    native = scatter(x=[0, 1, 2], y=(2, 3)).hue(["a"])
    with pytest.raises(DataError):
        native.svg_preserve_text(tmp_path / "native-mismatch")
    core = scatter(x=[0, 1, 2], y=[2, 3, 1])
    core.hue(np.array(["a"], dtype="U1"))
    with pytest.raises(DataError):
        core.svg_preserve_text(tmp_path / "core-foreign-mismatch")
    with pytest.raises(DataError):
        scatter(x=np.array([0, 1, 2]), y=np.array([2, 3]))
    # ND binds y before even resolving the invalid x position.
    with pytest.raises(TypeError, match="y"):
        scatter(np.ones((2, 2), dtype=bool), x=99, y=1)
    # Adapter alignment precedes optional hue; P1's hue extraction precedes
    # native construction and its alignment remains delayed.
    with pytest.raises(DataError):
        scatter(x=np.array([0, 1]), y=np.array([2]), hue=object())
    with pytest.raises(TypeError):
        scatter(x=[0, 1], y=[2], hue=object())


@pytest.mark.parametrize("layout", ["reverse", "transpose", "fortran", "broadcast"])
def test_numpy_logical_strides_and_snapshot(layout, tmp_path):
    np = pytest.importorskip("numpy")
    base = np.array([[0, 2], [5, 4], [20, 1]], dtype=np.float64)
    if layout == "reverse":
        table = base[::-1]
    elif layout == "transpose":
        table = base.T
    elif layout == "fortran":
        table = np.asfortranarray(base)
    else:
        table = np.broadcast_to(np.array([[0.0, 2.0]]), (3, 2))
    x, y = [float(value) for value in table[:, 0]], [float(value) for value in table[:, 1]]
    plot = scatter(table, x=np.int64(0), y=1).x_label("Time").y_label("Value")
    if table.flags.writeable:
        table[:] = 999
    actual = scene(plot, tmp_path / layout)[1]
    expected = scene(scatter(x=x, y=y).x_label("Time").y_label("Value"),
                     tmp_path / f"{layout}-reference")[1]
    assert actual == expected


def test_numpy_endian_unicode_and_role_admission(tmp_path):
    np = pytest.importorskip("numpy")
    values = np.array([2, 3, 1], dtype=">i8")
    hue = np.array(["control", "treated", "control"], dtype="U8")[::-1]
    plot = scatter(x=np.array([0, 1, 2]), y=values, hue=hue)
    values[:] = 100
    hue[:] = "changed"
    text, _ = scene(plot, tmp_path / "endian-unicode")
    assert "Control" in text and "Treated" in text and "Changed" not in text
    with pytest.raises(TypeError):
        scatter(x=np.array([True, False]), y=[1, 2])
    with pytest.raises(TypeError):
        plot.hue(np.array([1, 2, 3]))


def test_numpy_final_carrier_and_bitmap_shape_drift_are_rejected(monkeypatch):
    np = pytest.importorskip("numpy")
    calls = []
    swapped = np.dtype("i8").newbyteorder("S")
    base = np.array([[2, 3], [4, 1]], dtype=swapped)

    class ReshapeCarrier(np.ndarray):
        def astype(self, *args, **kwargs):
            calls.append("payload")
            return super().astype(*args, **kwargs).view(np.ndarray).reshape(4, 1)

    with pytest.raises(DataError, match="shape"):
        scatter(x=[0, 5], y=base.view(ReshapeCarrier))
    assert calls == ["payload"]

    mask = np.zeros((2, 2), dtype=bool)
    masked = np.ma.array(base, mask=False)

    class ChangeBitmap(np.ndarray):
        def astype(self, *args, **kwargs):
            calls.append("bitmap")
            mask.shape = (4, 1)
            return super().astype(*args, **kwargs).view(np.ndarray)

    carrier = base.view(ChangeBitmap)
    monkeypatch.setattr(np.ma, "getdata", lambda value, **kwargs: carrier)
    monkeypatch.setattr(np.ma, "getmaskarray", lambda value: mask)
    with pytest.raises(DataError, match="shape"):
        scatter(x=[0, 5], y=masked)
    assert calls == ["payload", "bitmap"]


def test_pandas_numpy_carriers_keep_original_rank_and_logical_dtype():
    pd = pytest.importorskip("pandas")
    np = pytest.importorskip("numpy")

    class RankDrift(pd.Series):
        def to_numpy(self, *args, **kwargs):
            return super().to_numpy(*args, **kwargs).reshape(2, 2)

    # A four-value Series must not become a two-row rectangle whose second
    # column is silently discarded by a scalar y consumer.
    with pytest.raises(DataError, match="shape"):
        scatter(x=np.array([0, 5]), y=RankDrift([2, 3, 4, 1], dtype="int64"))

    class DtypeDrift(pd.Series):
        def to_numpy(self, *args, **kwargs):
            return super().to_numpy(*args, **kwargs).astype(np.float64)

    with pytest.raises(TypeError, match="declared logical dtype"):
        scatter(x=np.array([0, 5]), y=DtypeDrift([2, 3], dtype="int64"))


def test_masked_source_snapshots_independent_data_and_mask_strides(tmp_path):
    np = pytest.importorskip("numpy")
    data = np.array([[0, 2, 5], [5, 3, 6], [20, 1, 4]], dtype=np.float64)
    mask = np.zeros((3, 3), dtype=bool, order="F")
    mask[1, 0] = True
    mask[0, 1] = True
    source = np.ma.MaskedArray(data[::-1], mask=mask[::-1], copy=False)
    plot = scatter(source, x=0, y=cols[2, 1])
    source.data[:] = 999
    source.mask[:] = True
    _, points = scene(plot, tmp_path / "masked-snapshot")
    assert len(points) == 3


def test_masked_nomask_all_missing_and_unsupported_empty_are_not_inferred():
    np = pytest.importorskip("numpy")
    scatter(x=np.ma.array([0, 1], mask=np.ma.nomask), y=np.array([2, 3]))
    scatter(x=np.ma.array([0, 1], mask=True), y=np.array([2, 3]))
    empty = scatter(np.ma.array(np.empty((0, 4), dtype=np.int64), mask=False),
                    x=3, y=cols[2, 0])
    empty.source_rows(np.empty((0, 2), dtype=np.uint64))
    with pytest.raises(DataError):
        empty.source_rows(np.empty((0, 3), dtype=np.uint64))
    for values in (
        np.empty((0, 2), dtype="datetime64[ns]"),
        np.ma.array(np.empty((0, 2), dtype=object), mask=True),
        np.ma.array(np.ones((2, 2), dtype=np.float16), mask=True),
    ):
        with pytest.raises(TypeError):
            scatter(values, x=0, y=1)
    with pytest.raises(TypeError):
        scatter(x=np.array(1), y=[2])
    with pytest.raises(TypeError):
        scatter(x=np.ones((1, 1, 1)), y=[2])


def test_explicit_wide_and_row_position_keep_shapes_and_actual_x(tmp_path):
    np = pytest.importorskip("numpy")
    table = np.array([[0, 2, 5], [5, 3, 6], [20, 1, 4]], dtype=np.int64)
    calls, domains = [], []

    class ChangeAfterY:
        def __index__(self):
            calls.append("x")
            table[:, 1:] = 999
            return 0

    def positions(view):
        domains.append(view.domain.data_limits)
        return TickPositions.data([1, 3, 6])

    actual = scatter(table, x=col[ChangeAfterY()], y=cols[2, 1]).hue(["a", "a", "b"])
    actual.x_label("Time").y_label("Response").size(640, 480)
    actual.y_tick_generation(TickGeneration.function(positions))
    _, points = scene(actual, tmp_path / "actual-x")
    assert len(points) == 6
    assert len({x for x, _ in points}) == 3
    assert calls == ["x"]
    actual_domains = domains[:]
    domains.clear()
    reference = scatter(
        x=[0, 0, 5, 5, 20, 20], y=[5, 2, 6, 3, 4, 1],
        hue=["a", "a", "a", "a", "b", "b"],
    ).x_label("Time").y_label("Response").size(640, 480)
    reference.y_tick_generation(TickGeneration.function(positions))
    _, reference_points = scene(reference, tmp_path / "actual-x-native-reference")
    assert points == reference_points
    assert actual_domains and domains
    assert all(domain == domains[0] for domain in actual_domains + domains)
    indexed = scatter(table, y=cols[1:], by_row_position=True)
    assert len(scene(indexed, tmp_path / "indexed")[1]) == 6
    direct = scatter(y=[[2, 5], [3, 6], [1, 4]], by_row_position=True)
    assert len(scene(direct, tmp_path / "direct-indexed")[1]) == 6
    with pytest.raises(TypeError):
        scatter(table, y=col[1], by_row_position=True)


def test_bound_only_pandas_and_later_role_never_reimports_old_xy(monkeypatch, tmp_path):
    pd = pytest.importorskip("pandas")
    from decimal import Decimal

    imported = []
    original = pd.Series.to_numpy

    def observed(self, *args, **kwargs):
        imported.append(self.name)
        return original(self, *args, **kwargs)

    monkeypatch.setattr(pd.Series, "to_numpy", observed)
    frame = pd.DataFrame({"time": [0, 5, 20], "value": [2, 3, 1],
                          "unused": [Decimal("1.1")] * 3})
    plot = scatter(frame, x="time", y="value")
    assert set(imported) == {"time", "value"}
    imported.clear()
    frame["time"] = [object()] * 3
    frame["value"] = [object()] * 3
    frame["group"] = pd.array(["control", "treated", "control"], dtype="string")
    retained = frame
    frame = pd.DataFrame({"group": ["wrong source"]})
    plot.hue("group")
    assert "time" not in imported and "value" not in imported and "unused" not in imported
    retained["group"] = "changed"
    text, points = scene(plot, tmp_path / "fresh-role")
    assert len(points) == 3 and "Control" in text and "Changed" not in text


def test_pandas_discovery_is_explicit_ordered_and_not_a_filter(tmp_path):
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"b": pd.array([2, 3], dtype="Int64"),
                          "group": ["a", "b"], "id": [10, 11], "a": [4, 5],
                          "flag": [True, False]})
    discovered = scatter(frame, by_row_position=True, numeric_only=True)
    text, points = scene(discovered, tmp_path / "discovery")
    assert len(points) == 6
    assert text.index("b") < text.index("id") < text.index("a")
    with pytest.raises(TypeError):
        scatter(frame, y=cols["b", "group"], by_row_position=True)
    with pytest.raises(TypeError):
        scatter(frame, y=cols["b"], by_row_position=True, numeric_only=True)
    with pytest.raises(TypeError):
        scatter(frame, by_row_position=True)


def test_nullable_ids_stay_exact_and_missing_ids_fail_transactionally(tmp_path):
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"x": [0, 1], "y": [2, 3],
                          "ids": pd.array([2**53 + 1, 2**64 - 1], dtype="UInt64")})
    plot = scatter(frame, x="x", y="y")
    assert plot.source_rows("ids") is plot
    frame["ids"] = pd.array([2**53 + 1, pd.NA], dtype="UInt64")
    with pytest.raises(DataError):
        plot.source_rows("ids")
    assert len(scene(plot, tmp_path / "exact-ids")[1]) == 2
    np = pytest.importorskip("numpy")
    scatter(x=[0, 1], y=[2, 3]).source_rows(np.array([2**53, 2**53 + 1], dtype=np.uint64))
    with pytest.raises(DataError):
        scatter(x=[0, 1], y=[2, 3]).source_rows(
            np.ma.array([10, 20], mask=[False, True], dtype=np.uint64)
        )


def test_ids_keep_expanded_cell_shape_and_do_not_repeat_rows():
    np = pytest.importorskip("numpy")
    table = np.array([[0, 2, 5, 100, 101], [5, 3, 6, 200, 201]], dtype=np.uint64)
    plot = scatter(table, x=0, y=cols[1, 2])
    with pytest.raises(DataError):
        plot.source_rows(col[3])
    with pytest.raises(DataError):
        plot.source_rows(np.array([100, 200], dtype=np.uint64))
    with pytest.raises(DataError):
        plot.source_rows(np.array([[100, 101, 200, 201]], dtype=np.uint64))
    plot.source_rows(cols[3, 4])
    plot.source_rows(np.array([[100, 101], [200, 201]], dtype=np.uint64))
    plot.source_rows([[100, 101], [200, 201]])
    with pytest.raises(TypeError):
        plot.source_rows(np.array([[True, False], [False, True]]))


def test_numpy_backed_nan_never_uses_a_pandas_null_bitmap(monkeypatch, tmp_path):
    pd = pytest.importorskip("pandas")
    np = pytest.importorskip("numpy")
    values = pd.Series([0.0, np.nan, 2.0], name="time")

    def forbidden(*args, **kwargs):
        raise AssertionError("a NumPy-backed float has no independent null bitmap")

    monkeypatch.setattr(type(values.array), "isna", forbidden)
    plot = scatter(x=values, y=np.array([2, 3, 1]))
    assert len(scene(plot, tmp_path / "numpy-nan")[1]) == 2


def test_nullable_float_bitmap_string_and_categorical_decoding(tmp_path):
    pd = pytest.importorskip("pandas")
    np = pytest.importorskip("numpy")
    floating = pd.arrays.FloatingArray(
        np.array([0.0, np.nan, 2.0, 3.0]), np.array([False, False, True, False])
    )
    frame = pd.DataFrame({"x": floating, "y": [2, 3, 1, 4],
                          "group": pd.Categorical(["b", "a", "b", "a"],
                                                  categories=["unused", "a", "b"])})
    text, points = scene(scatter(frame, x="x", y="y", hue="group"),
                         tmp_path / "nullable-categorical")
    assert len(points) == 2 and "Unused" not in text
    with pytest.raises(TypeError):
        scatter(frame.assign(group=pd.Categorical([1, 2, 1, 2])), x="x", y="y", hue="group")

    class NotString:
        def __str__(self):
            raise AssertionError("arbitrary categorical data must not be stringified")

    with pytest.raises(TypeError):
        scatter(x=[0, 1], y=[2, 3]).hue(pd.Series(["a", NotString()], dtype=object))


@pytest.mark.parametrize("origin", ["ndarray", "polars"])
def test_later_container_does_not_switch_expansion_authority(origin, tmp_path):
    np = pytest.importorskip("numpy")
    pl = pytest.importorskip("polars")
    if origin == "ndarray":
        plot = scatter(x=np.array([0, 5]), y=np.array([[2, 3], [4, 1]]))
        values = pl.Series("group", ["a", "b"])
    else:
        plot = scatter(x=pl.Series("time", [0, 5]),
                       y=pl.DataFrame({"a": [2, 4], "b": [3, 1]}))
        values = np.array(["a", "b"], dtype="U1")
    assert plot.hue(values) is plot
    assert len(scene(plot, tmp_path / origin)[1]) == 4
    with pytest.raises(DataError):
        plot.hue(["too short"])


def test_polars_selected_namespace_and_discovery_are_bound_only(monkeypatch, tmp_path):
    pl = pytest.importorskip("polars")
    from decimal import Decimal

    source = pl.DataFrame({
        "x": [0, 5], "b": [2, 4], "a": [3, 1],
        "Variable": [{"unused": 1}, {"unused": 2}],
        "decimal": pl.Series([Decimal("1.25")] * 2, dtype=pl.Decimal(10, 2)),
        "group": ["a", "b"],
    })
    exported = []
    original = pl.Series.__arrow_c_stream__

    def bound_only(series, *args, **kwargs):
        assert series.name not in {"Variable", "decimal"}
        exported.append(series.name)
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", bound_only)
    selected = scatter(source, x="x", y=cols["b", "a"]).labels(Labels(mode="raw"))
    text, points = scene(selected, tmp_path / "pl-namespace")
    assert len(points) == 4 and "Variable_" in text
    assert text.index("b") < text.index("a")
    assert sorted(exported) == ["a", "b", "x"]
    exported.clear()
    scatter(source, by_row_position=True, numeric_only=True)
    assert exported == ["x", "b", "a"]
    # String is transportable, but the original PL numeric y consumer rejects
    # it before the missing x key is resolved.
    with pytest.raises(TypeError, match="y"):
        scatter(source, x="missing", y="group")


def test_polars_direct_local_namespace_and_ordinals_are_bound_only(monkeypatch, tmp_path):
    np = pytest.importorskip("numpy")
    pl = pytest.importorskip("polars")
    source = pl.DataFrame({
        "x": [0, 5],
        "Variable": [{"unused": 1}, {"unused": 2}],
        "Variable_": [{"unused": 3}, {"unused": 4}],
    })
    exported = []
    original = pl.Series.__arrow_c_stream__

    def bound_only(series, *args, **kwargs):
        assert series.name == "x"
        exported.append(series.name)
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", bound_only)
    local = scatter(source, x="x", y=np.array([[2, 3], [4, 1]]))
    text, points = scene(local, tmp_path / "pl-local-defaults")
    assert len(points) == 4 and "Column 1" in text and "Column 2" in text
    assert exported == ["x"]
    # Native Expansion owns the generated field and private ordinal keys;
    # the default display names must not become raw collection identities.
    labels = Labels(mode="raw", aliases=[
        ("__lichtung_variable_0", "First local"),
        ("__lichtung_variable_1", "Second local"),
        ("Column 1", "Wrong raw-name key"),
    ])
    text, _ = scene(local.labels(labels), tmp_path / "pl-local-ordinals")
    assert "Variable__" in text and "First local" in text and "Second local" in text
    assert "Wrong raw-name key" not in text

    exported.clear()
    native = scatter(source, x=[0, 5], y=[[2, 3], [4, 1]]).labels(labels)
    text, points = scene(native, tmp_path / "pl-local-native")
    assert len(points) == 4 and "Variable__" in text
    assert "First local" in text and "Second local" in text
    assert "Wrong raw-name key" not in text and exported == []


def test_polars_cols_one_remains_a_collection_but_series_is_scalar(tmp_path):
    pl = pytest.importorskip("polars")
    source = pl.DataFrame({
        "x": [0, 5], "only": [2, 4],
        "Variable": [{"unused": 1}, {"unused": 2}],
        "Variable_": [{"unused": 3}, {"unused": 4}],
    })
    collection = scatter(source, x="x", y=cols["only"]).labels(Labels(mode="raw"))
    text, points = scene(collection, tmp_path / "pl-cols-one")
    assert len(points) == 2 and "Variable__" in text and "only" in text
    scalar = scatter(source, x="x", y=source["only"]).labels(Labels(mode="raw"))
    text, points = scene(scalar, tmp_path / "pl-series-scalar")
    assert len(points) == 2 and "only" in text
    assert not any(label.startswith("Variable") for label in text)


def test_polars_independent_named_frames_keep_rows_names_and_main_lookup(monkeypatch, tmp_path):
    pl = pytest.importorskip("polars")
    source = pl.DataFrame({
        "Variable": [{"unused": 1}, {"unused": 2}],
        "Variable_": [{"unused": 3}, {"unused": 4}],
    })
    frame = pl.DataFrame({"Variable_": [2, 4, 6], "Variable__": [3, 1, 5]})
    exported = []
    original = pl.Series.__arrow_c_stream__

    def bound_only(series, *args, **kwargs):
        assert series.name in {"Variable_", "Variable__"}
        assert series.dtype == pl.Int64
        exported.append(series.name)
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", bound_only)
    # The overlapping Variable_ name is deduplicated; Variable__ comes only
    # from y. Three actual y rows must not be checked against the two-row main.
    ordinary = scatter(source, x=[0, 5, 20], y=frame).labels(Labels(mode="raw", aliases=[
        ("Variable_", "First named"),
        ("Variable__", "Second named"),
        ("__lichtung_variable_0", "Wrong local key"),
    ]))
    text, points = scene(ordinary, tmp_path / "pl-independent-ordinary")
    assert len(points) == 6 and len({x for x, _ in points}) == 3
    assert "Variable___" in text and "First named" in text and "Second named" in text
    assert "Wrong local key" not in text
    assert exported == ["Variable_", "Variable__"]
    referents = gc.get_referents(ordinary._owner)
    assert sum(value is source for value in referents) == 1
    assert all(value is not frame for value in referents)
    with pytest.raises(KeyError):
        ordinary.hue("Variable__")

    exported.clear()
    single = frame.select("Variable__")
    indexed = scatter(y=single, by_row_position=True).labels(Labels(mode="raw", aliases=[
        ("Variable__", "Indexed named"),
        ("__lichtung_variable_0", "Wrong local key"),
    ]))
    text, points = scene(indexed, tmp_path / "pl-independent-indexed-one")
    assert len(points) == 3 and "Variable" in text and "Indexed named" in text
    assert "Wrong local key" not in text and exported == ["Variable__"]
    with pytest.raises(TypeError, match="retained data source"):
        indexed.hue(col["Variable__"])


def test_source_key_reordering_new_keys_and_copy_forks_use_stable_distinct_ids():
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"x": [0, 1], "y": [2, 3], "group": ["a", "b"]})
    plot = scatter(frame, x="x", y="y", hue="group")
    group = raw_for(plot, "group")
    frame.insert(0, "extra", [10, 11])
    frame.insert(0, "moved", frame.pop("x"))
    plot.hue("group")
    assert raw_for(plot, "group") == group
    frame.rename(columns={"group": "renamed"}, inplace=True)
    plot.hue("renamed")
    assert raw_for(plot, "renamed") != group
    with pytest.raises(KeyError):
        plot.hue("group")
    first, second = plot.copy(), plot.copy()
    frame["left"] = ["l", "l"]
    frame["right"] = ["r", "r"]
    before = registry(plot)
    first.hue("left")
    second.hue("right")
    assert raw_for(first, "left") != raw_for(second, "right")
    assert registry(first) is not registry(second)
    assert before is registry(plot)
    assert "left" not in before.values() and "right" not in before.values()

    direct = scatter(x=[0, 1], y=[2, 3], hue=pd.Series(["a", "b"], name="known direct"))
    first, second = direct.copy(), direct.copy()
    before = registry(direct)
    original_entries = dict(before)
    first.hue(pd.Series(["a", "b"], name="fresh direct"))
    second.hue(pd.Series(["a", "b"], name="fresh direct"))
    assert raw_for(first, "fresh direct") != raw_for(second, "fresh direct")
    assert registry(first) is not registry(second)
    assert registry(direct) is before and before == original_entries


def test_complete_label_replacement_does_not_resurrect_known_key_defaults(tmp_path):
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"old_x": [0, 1], "old_y": [2, 3], "old_group": ["a", "b"]})
    plot = scatter(frame, x="old_x", y="old_y", hue="old_group")
    plot.labels(Labels(mode="raw", aliases=[("a", "Authored A")]))
    frame.insert(0, "extra", [3, 4])
    plot.hue("old_group")
    frame["fresh_group"] = ["a", "b"]
    plot.hue("fresh_group")
    text, _ = scene(plot, tmp_path / "label-replacement")
    assert "old_x" not in text and "old_y" not in text and "old_group" not in text
    assert "fresh_group" in text and "Authored A" in text

    named = pd.Series(["a", "b"], name="old direct group")
    core = scatter(x=[0, 1], y=[2, 3], hue=named)
    raw = raw_for(core, "old direct group")
    core.labels(Labels(mode="raw", aliases=[("a", "Authored A")]))
    core.hue(named)
    text, _ = scene(core, tmp_path / "direct-known-label-replacement")
    assert raw in text and "old direct group" not in text and "Authored A" in text
    core.hue(pd.Series(["a", "b"], name="fresh direct group"))
    text, _ = scene(core, tmp_path / "direct-fresh-label-default")
    assert "fresh direct group" in text and "Authored A" in text


def test_source_key_gc_edges_copy_and_canvas_capture(tmp_path):
    pd = pytest.importorskip("pandas")

    class Key:
        plot = None

        def __hash__(self):
            return 991

        def __str__(self):
            return "time"

    key = Key()
    frame = pd.DataFrame([[0, 2], [1, 3]], columns=[key, "value"])
    plot = scatter(frame, x=col[key], y="value")
    branch = plot.copy()
    # Remove the foreign object-Index's hidden edge to isolate binding-owner GC.
    frame.columns = ["renamed", "value"]
    key.plot = plot
    frame.attrs["plot"] = plot
    canvas = Canvas.grid(1, 1).panel(0, 0, plot)
    assert sum(value is frame for value in gc.get_referents(plot._owner)) == 1
    assert sum(value is frame for value in gc.get_referents(branch._owner)) == 1
    assert sum(value is key for value in registry(plot).values()) == 1
    references = weakref.ref(key), weakref.ref(frame), weakref.ref(plot)
    del key, frame, plot
    gc.collect()
    assert all(reference() is not None for reference in references)
    del branch
    gc.collect()
    assert all(reference() is None for reference in references)
    assert len(scene(canvas, tmp_path / "capture-without-source")[1]) == 2


def test_direct_series_do_not_install_an_implicit_parent_source():
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"x": [0, 1], "y": [2, 3], "group": ["a", "b"]}, index=[90, 40])
    reference = weakref.ref(frame)
    plot = scatter(x=frame["x"], y=frame["y"])
    with pytest.raises(TypeError):
        plot.hue("group")
    del frame
    gc.collect()
    assert reference() is None


def test_direct_series_names_are_scientific_keys_not_display_strings(tmp_path):
    pd = pytest.importorskip("pandas")
    integer = pd.Series(["a", "b"], name=1)
    string = pd.Series(["a", "b"], name="1")
    references = weakref.ref(integer), weakref.ref(string)
    core = scatter(x=[0, 1], y=[2, 3]).hue(integer).hue(string)
    integer_raw, string_raw = raw_for(core, 1), raw_for(core, "1")
    assert integer_raw != string_raw
    core.labels(Labels(mode="raw", aliases=[
        (integer_raw, "Integer name"), (string_raw, "String name"),
    ]))
    text, _ = scene(core, tmp_path / "direct-string-name")
    assert "String name" in text and "Integer name" not in text
    core.hue(integer)
    text, _ = scene(core, tmp_path / "direct-integer-name")
    assert "Integer name" in text and "String name" not in text

    numeric = scatter(
        x=pd.Series([0, 5], name=1), y=pd.Series([2, 3], name="1"),
    )
    x_raw, y_raw = raw_for(numeric, 1), raw_for(numeric, "1")
    assert x_raw != y_raw
    numeric.labels(Labels(mode="raw", aliases=[
        (x_raw, "Integer axis"), (y_raw, "String axis"),
    ]))
    text, points = scene(numeric, tmp_path / "direct-numeric-names")
    assert len(points) == 2 and "Integer axis" in text and "String axis" in text

    # pandas missing-key equivalence, not ordinary float/dict equality.
    first_nan = float("nan")
    core.hue(pd.Series(["a", "b"], name=first_nan))
    nan_raw = next(raw for raw, name in registry(core).items() if name is first_nan)
    before = set(registry(core))
    core.hue(pd.Series(["a", "b"], name=float("nan")))
    assert set(registry(core)) == before and registry(core)[nan_raw] is first_nan
    core.hue(pd.Series(["a", "b"], name=None))
    assert set(registry(core)) == before
    del integer, string
    gc.collect()
    assert all(reference() is None for reference in references)


def test_key_hash_exception_identity_and_original_state_are_preserved():
    pd = pytest.importorskip("pandas")
    failure = RuntimeError("application key failure")

    class Key:
        armed = False

        def __hash__(self):
            if self.armed:
                raise failure
            return 391

        def __str__(self):
            return "group"

    key = Key()
    frame = pd.DataFrame([[0, 2, "a"], [1, 3, "b"]], columns=["x", "y", key])
    plot = scatter(frame, x="x", y="y")
    before = registry(plot)
    original_entries = dict(before)
    key.armed = True
    with pytest.raises(RuntimeError) as caught:
        plot.hue(col[key])
    assert caught.value is failure
    assert registry(plot) is before
    assert before == original_entries


def test_reentrant_key_hash_rejects_stale_candidate_without_losing_inner_edit(tmp_path):
    pd = pytest.importorskip("pandas")

    class Key:
        plot = None
        armed = False

        def __hash__(self):
            if self.armed:
                self.armed = False
                self.plot.title("Inner edit survives")
            return 1991

        def __str__(self):
            return "group"

    key = Key()
    frame = pd.DataFrame([[0, 2, "a"], [1, 3, "b"]], columns=["x", "y", key])
    plot = scatter(frame, x="x", y="y")
    key.plot, key.armed = plot, True
    before = registry(plot)
    original_entries = dict(before)
    with pytest.raises(RuntimeError, match="stale"):
        plot.hue(col[key])
    assert registry(plot) is before
    assert before == original_entries
    text, _ = scene(plot, tmp_path / "reentrant-input")
    assert "Inner edit survives" in text

    direct = scatter(x=[0, 1], y=[2, 3], hue=pd.Series(["a", "b"], name="known direct"))
    direct_key = Key()
    named = pd.Series(["a", "b"], name=direct_key)
    direct_key.plot, direct_key.armed = direct, True
    before = registry(direct)
    original_entries = dict(before)
    with pytest.raises(RuntimeError, match="stale"):
        direct.hue(named)
    assert registry(direct) is before and before == original_entries
    text, _ = scene(direct, tmp_path / "reentrant-direct-name")
    assert "Inner edit survives" in text


def test_schema_reentry_cannot_attach_old_key_to_a_new_position():
    pd = pytest.importorskip("pandas")

    class Key:
        frame = None
        armed = False

        def __hash__(self):
            if self.armed:
                self.armed = False
                self.frame.columns = ["x", "y", "different"]
            return 881

        def __str__(self):
            return "group"

    key = Key()
    frame = pd.DataFrame([[0, 2, "a"], [1, 3, "b"]], columns=["x", "y", key])
    plot = scatter(frame, x="x", y="y")
    key.frame, key.armed = frame, True
    before = registry(plot)
    with pytest.raises(DataError, match="schema changed"):
        plot.hue(col[key])
    assert registry(plot) is before
    plot.hue("different")


def test_source_clear_finalizer_reenters_after_owner_borrow_ends():
    pd = pytest.importorskip("pandas")
    from .test_gc import clear_gc_object

    events = []

    class Source(pd.DataFrame):
        def __del__(self):
            target = self.attrs.get("target")
            if target is not None:
                try:
                    target.hue(["a", "b"])
                except BaseException as error:
                    # A retained traceback would keep this finalizer's self alive.
                    events.append((type(error), str(error)))

    source = Source({"x": [0, 1], "y": [2, 3]})
    plot = scatter(source, x="x", y="y")
    source.attrs["target"] = plot
    reference = weakref.ref(source)
    del source
    clear_gc_object(plot._owner)
    assert reference() is None
    assert len(events) == 1
    assert events[0][0] is RuntimeError and "cleared" in events[0][1]
