"""Installed public input contracts, not executable rendering tests.

Use strict checking with unused-ignore errors. Each targeted ignore must still
reject its call: an Any-poisoned provider/union makes these guards fail. The
adjacent configuration follows real pandas source when no pandas-stubs provider
is installed; it does not supply replacement pandas declarations.
"""

from __future__ import annotations

import pandas as pd
import polars as pl

from lichtung_plot import Canvas, col, cols, scatter


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
