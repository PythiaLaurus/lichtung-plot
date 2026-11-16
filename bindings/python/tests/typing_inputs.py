"""Installed public input contracts, not executable rendering tests.

Use strict checking with unused-ignore errors. Each targeted ignore must still
reject its call: an Any-poisoned provider/union makes these guards fail. The
adjacent configuration follows real pandas source when no pandas-stubs provider
is installed; it does not supply replacement pandas declarations.
"""

from __future__ import annotations

from collections.abc import Hashable

import numpy as np
import pandas as pd
import polars as pl
from numpy.typing import NDArray
from typing_extensions import assert_type

from lichtung_plot import (
    Canvas,
    ColumnSelector,
    ColumnsSelector,
    Labels,
    Scatter,
    col,
    cols,
    scatter,
)


class _HashableLabel:
    def __hash__(self) -> int:
        return 17


def selector_contract(
    frame: pd.DataFrame, key: Hashable, custom_key: _HashableLabel
) -> None:
    assert_type(col[key], ColumnSelector)
    assert_type(cols[key], ColumnsSelector)
    assert_type(cols[::-1], ColumnsSelector)
    assert_type(scatter(frame, x=col[custom_key], y=cols["y"]), Scatter)
    # Missing lookup context is a native check, not a container-overload matrix.
    assert_type(scatter(None, x=col["x"], y=col["y"]), Scatter)

    col[["not hashable"]]  # type: ignore[index]
    cols[["not selector syntax"]]  # type: ignore[index]
    ColumnSelector()  # type: ignore[call-arg]
    ColumnsSelector()  # type: ignore[call-arg]


def native_contract(
    nullable_integers: list[int | None],
    nullable_floats: list[float | None],
    nullable_rows: list[list[float | None]],
) -> None:
    assert_type(scatter(x=nullable_integers, y=nullable_floats), Scatter)
    assert_type(scatter(y=nullable_rows, by_row_position=True), Scatter)


def numpy_contract(
    table: NDArray[np.float64],
    values: NDArray[np.float64],
    rectangle: NDArray[np.float64],
    masked: np.ma.MaskedArray[tuple[int, ...], np.dtype[np.float64]],
    categories: NDArray[np.str_],
) -> None:
    # NumPy rank/dtype/validity remain runtime checks, including MaskedArray rank.
    assert_type(scatter(table, x=0, y=cols[1, 2]), Scatter)
    assert_type(scatter(table, x=np.int64(0), y=1), Scatter)
    assert_type(scatter(x=masked, y=values, hue=categories), Scatter)
    assert_type(scatter(y=rectangle, by_row_position=True), Scatter)


def pandas_contract(
    frame: pd.DataFrame, values: pd.Series, categories: pd.Series, ids: pd.Series
) -> None:
    assert_type(scatter(frame, x="x", y=cols["a", "b"], hue="group"), Scatter)
    assert_type(scatter(x=values, y=values, hue=categories), Scatter)
    plot = scatter(frame, x=col["x"], y=cols["y"])
    assert_type(plot.hue(categories).source_rows(ids), Scatter)


def polars_contract(
    frame: pl.DataFrame, values: pl.Series, categories: pl.Series, ids: pl.Series
) -> None:
    assert_type(scatter(frame, x="x", y=cols["a", "b"]), Scatter)
    assert_type(scatter(x=values, y=values, hue=categories), Scatter)
    plot = scatter(frame, x=col["x"], y=col["y"])
    assert_type(plot.hue(categories).source_rows(ids), Scatter)


def role_boundaries(
    frame: pd.DataFrame,
    series: pd.Series,
    polars_frame: pl.DataFrame,
    polars_series: pl.Series,
    expression: pl.Expr,
    lazy: pl.LazyFrame,
    unrelated: object,
    float_ids: list[float],
) -> None:
    # Both pandas directions are essential: source-follow must retain nominal
    # DataFrame/Series boundaries, not merely reject unrelated builtin values.
    scatter(series, x=[0], y=[1])  # type: ignore[call-overload]
    scatter(x=frame, y=[1])  # type: ignore[call-overload]
    scatter(polars_series, x=[0], y=[1])  # type: ignore[call-overload]
    scatter(x=polars_frame, y=[1])  # type: ignore[call-overload]
    scatter([0, 1], x=[0], y=[1])  # type: ignore[call-overload]
    scatter([[0, 1]], x=[0], y=[1])  # type: ignore[call-overload]
    scatter(expression, x=[0], y=[1])  # type: ignore[call-overload]
    scatter(lazy, x=[0], y=[1])  # type: ignore[call-overload]
    scatter(unrelated, x=[0], y=[1])  # type: ignore[call-overload]
    scatter(x=unrelated, y=[1])  # type: ignore[call-overload]
    scatter(x=[0], y=[1], unset=unrelated)  # type: ignore[call-overload]
    scatter(x=expression, y=[1])  # type: ignore[call-overload]
    scatter(x=[0], y=lazy)  # type: ignore[call-overload]
    scatter(frame, x=cols["x"], y=col["y"])  # type: ignore[call-overload]
    scatter(frame, x=col["x"], y=col["y"], hue=cols["group"])  # type: ignore[call-overload]
    scatter(x=None, y=[1])  # type: ignore[call-overload]
    scatter(x=[0], y=None)  # type: ignore[call-overload]

    plot = scatter(frame, x=col["x"], y=col["y"])
    plot.hue(cols["group"])  # type: ignore[arg-type]
    plot.hue(frame)  # type: ignore[arg-type]
    plot.hue(polars_frame)  # type: ignore[arg-type]
    plot.hue(expression)  # type: ignore[arg-type]
    plot.hue(None)  # type: ignore[arg-type]
    plot.source_rows(float_ids)  # type: ignore[arg-type]
    plot.source_rows(None)  # type: ignore[arg-type]
    plot.labels(None)  # type: ignore[arg-type]
    Canvas.grid(1, 1).labels(None)  # type: ignore[arg-type]


def mode_boundaries(frame: pd.DataFrame) -> None:
    scatter(x=[0])  # type: ignore[call-overload]
    scatter(y=[[1]])  # type: ignore[call-overload]
    scatter(frame, y=cols["y"])  # type: ignore[call-overload]
    scatter(frame, by_row_position=True)  # type: ignore[call-overload]
    scatter(frame, x=col["x"], y=cols["y"], by_row_position=True)  # type: ignore[call-overload]
    scatter(frame, y=col["y"], by_row_position=True)  # type: ignore[call-overload]
    scatter(frame, y=None, by_row_position=True)  # type: ignore[call-overload]
    scatter(frame, x=None, y=cols["y"], by_row_position=True)  # type: ignore[call-overload]
    scatter(frame, numeric_only=True)  # type: ignore[call-overload]
    scatter(by_row_position=True, numeric_only=True)  # type: ignore[call-overload]
    scatter(None, by_row_position=True, numeric_only=True)  # type: ignore[call-overload]
    scatter(frame, y=cols["y"], by_row_position=True, numeric_only=True)  # type: ignore[call-overload]
    scatter(frame, y=None, by_row_position=True, numeric_only=True)  # type: ignore[call-overload]
    scatter(frame, x=col["x"], by_row_position=True, numeric_only=True)  # type: ignore[call-overload]


class _CustomScatter(Scatter):
    pass


def labels_and_owner_contract(custom: _CustomScatter) -> None:
    labels = Labels(aliases=[("raw_key", "Displayed")])
    assert_type(custom.labels(labels).source_rows([0, 1]).copy(), _CustomScatter)
