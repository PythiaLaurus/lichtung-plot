"""Shared four-route contracts through actual public owners and Scatter consumers."""

import gc
from pathlib import Path
import weakref
import xml.etree.ElementTree as ET

import pytest

from lichtung_plot import Canvas, DataError, TickGeneration, TickPositions, col, cols, scatter

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
