"""Real selector descriptions and public Scatter binding, not a selector emulator."""

import pytest

from lichtung_plot import DataError, Scatter, col, scatter


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
