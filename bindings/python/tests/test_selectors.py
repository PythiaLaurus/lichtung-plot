"""Real selector descriptions and public Scatter binding, not a selector emulator."""

import gc
import weakref

import pytest

from lichtung_plot import DataError, Scatter, col, cols, scatter
from lichtung_plot import _native


def test_factories_have_distinct_immutable_runtime_types_and_no_public_constructor():
    single, many = col["a"], cols["a"]
    assert type(single) is _native.ColumnSelector
    assert type(many) is _native.ColumnsSelector
    assert type(col) is _native._ColumnEntry
    assert type(cols) is _native._ColumnsEntry
    for value in (single, many, col, cols):
        with pytest.raises(TypeError):
            type(value)()
        with pytest.raises(AttributeError):
            value.key = "changed"
        assert not hasattr(value, "__dict__")


def test_creation_neither_hashes_formats_invokes_nor_captures_a_source():
    calls = []

    class Key:
        def __hash__(self):
            calls.append("hash")
            return 123

        def __str__(self):
            calls.append("str")
            return "key"

        def __call__(self, *args):
            raise AssertionError("literal key was invoked")

    key = Key()
    single, many = col[key], cols[key, key]
    assert calls == []
    assert sum(value is key for value in gc.get_referents(single)) == 1
    assert sum(value is key for value in gc.get_referents(many)) == 2
    with pytest.raises(TypeError):
        scatter(x=[0, 1], y=single)
    assert calls == []


def test_selector_key_cycle_is_visible_to_gc():
    class Key:
        pass

    key = Key()
    selector = cols[key]
    key.selector = selector
    reference = weakref.ref(key)
    del key, selector
    gc.collect()
    assert reference() is None


@pytest.mark.parametrize(
    "selection",
    [lambda: cols[-3:], lambda: cols[::-1], lambda: cols[3:5],
     lambda: cols[::10**100], lambda: cols[::-10**100]],
)
def test_signed_slices_are_admitted_as_collections(selection):
    np = pytest.importorskip("numpy")
    table = np.arange(15, dtype=np.int64).reshape(3, 5)
    assert isinstance(scatter(table, y=selection(), by_row_position=True), Scatter)


@pytest.mark.parametrize(
    "selection",
    [lambda: cols[3:8], lambda: cols[-6:], lambda: cols[::0],
     lambda: cols[-1:-1:-1], lambda: cols[0, -5],
     lambda: cols[True:], lambda: cols[::False], lambda: cols["a":"d"],
     lambda: cols[()], lambda: col[True]],
)
def test_strict_bounds_empty_duplicate_and_boolean_positions_fail(selection):
    np = pytest.importorskip("numpy")
    table = np.arange(15, dtype=np.int64).reshape(3, 5)
    with pytest.raises((TypeError, ValueError, IndexError, DataError)):
        scatter(table, x=0, y=selection())


def test_index_protocol_and_one_column_collection_do_not_collapse(tmp_path):
    np = pytest.importorskip("numpy")
    table = np.array([[0, 2], [5, 3], [20, 1]], dtype=np.int64)

    class Last:
        def __index__(self):
            return -1

    single = scatter(table, x=np.int64(0), y=col[Last()])
    collection = scatter(table, x=0, y=cols[np.int64(-1)])
    for key in (True, np.bool_(False)):
        with pytest.raises(TypeError):
            scatter(table, x=key, y=1)
    with pytest.raises((TypeError, DataError)):
        scatter(table, x=cols[0], y=1)
    with pytest.raises((TypeError, DataError)):
        collection.hue(cols[1])
    # A one-column collection still has an expanded-cell origin.
    with pytest.raises(DataError):
        single.source_rows(np.array([[10], [20], [30]], dtype=np.uint64))
    collection.source_rows(np.array([[10], [20], [30]], dtype=np.uint64))
    # Container indexing keeps its ordinary clipping, separate from cols.
    scatter(x=table[:, 0], y=table[:, 1:100]).svg_preserve_text(tmp_path / "clipped")


def test_pandas_integer_and_boolean_labels_are_not_positions():
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame([[0, 2, 5], [1, 3, 6]], columns=[-1, True, "value"])
    assert isinstance(scatter(frame, x=col[-1], y=col[True]), Scatter)
    assert isinstance(scatter(frame, y=cols[-1:], by_row_position=True), Scatter)
    with pytest.raises((TypeError, ValueError)):
        scatter(frame, y=cols[False:], by_row_position=True)


def test_pandas_exact_nan_timestamp_tuple_custom_and_callable_labels():
    pd = pytest.importorskip("pandas")

    class Key:
        def __init__(self, value):
            self.value = value

        def __hash__(self):
            return hash(self.value)

        def __eq__(self, other):
            return isinstance(other, Key) and self.value == other.value

        def __str__(self):
            return self.value

        def __call__(self, *_):
            raise AssertionError("a literal callable label must not run")

    stamp = pd.Timestamp("2024-04-03", tz="UTC")
    key = Key("custom")
    labels = [float("nan"), stamp, key, ("full", float("nan")), 1, "1"]
    frame = pd.DataFrame(
        [[0, 2, 3, 4, 5, 6], [1, 3, 4, 5, 6, 7]],
        columns=pd.Index(labels, dtype=object, tupleize_cols=False),
    )
    for query in (float("nan"), stamp, Key("custom"), labels[3], 1.0, "1"):
        assert isinstance(scatter(frame, x=col[query], y=[2, 3]), Scatter)
    multi = pd.DataFrame([[0], [1]], columns=pd.MultiIndex.from_tuples([("full", float("nan"))]))
    assert isinstance(scatter(multi, x=col[("full", float("nan"))], y=[2, 3]), Scatter)
    # This tuple is direct data, not the identically named source field.
    with pytest.raises(TypeError):
        scatter(frame, x=[0, 1], y=("full", float("nan")))


def test_partial_date_interval_and_multiindex_queries_fail_even_if_unique():
    pd = pytest.importorskip("pandas")
    date = pd.DataFrame([[0], [1]], columns=pd.DatetimeIndex(["2024-04-03"]))
    interval = pd.Interval(0, 2)
    intervals = pd.DataFrame([[0], [1]], columns=pd.IntervalIndex([interval]))
    multi = pd.DataFrame([[0], [1]], columns=pd.MultiIndex.from_tuples([("a", "b")]))
    for frame, query in ((date, "2024"), (date, "2024-04-03"),
                         (intervals, 1), (multi, "a"), (multi, ("a",))):
        with pytest.raises((KeyError, DataError)):
            scatter(frame, x=col[query], y=[2, 3])
    assert isinstance(scatter(date, x=col[pd.Timestamp("2024-04-03")], y=[2, 3]), Scatter)
    assert isinstance(scatter(intervals, x=col[interval], y=[2, 3]), Scatter)
    assert isinstance(scatter(multi, x=col[("a", "b")], y=[2, 3]), Scatter)


def test_duplicate_pandas_keys_fail_only_when_selected():
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame([[0, 1, 2, 3], [1, 2, 3, 4]], columns=["x", "y", "dup", "dup"])
    assert isinstance(scatter(frame, x="x", y="y"), Scatter)
    with pytest.raises((KeyError, DataError)):
        scatter(frame, x="x", y=col["dup"])
    with pytest.raises((KeyError, DataError)):
        scatter(frame, x="x", y=cols[2:])


def test_selector_clear_releases_key_after_the_pyborrow():
    from .test_gc import clear_gc_object

    events = []

    class Key:
        selector = None

        def __del__(self):
            try:
                clear_gc_object(self.selector)
                events.append("reentered")
            except BaseException as error:
                events.append(error)

    key = Key()
    selector = col[key]
    key.selector = selector
    reference = weakref.ref(key)
    del key
    clear_gc_object(selector)
    assert reference() is None
    assert events == ["reentered"]
    with pytest.raises(ValueError, match="cleared"):
        scatter(x=[0, 1], y=selector)
