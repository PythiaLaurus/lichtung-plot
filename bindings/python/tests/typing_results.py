"""Installed independent-layer/result contracts; this module is not executed."""

from typing import Literal, Mapping

import numpy as np
import pandas as pd
import polars as pl
from numpy.typing import NDArray
from typing_extensions import assert_type

import lichtung_plot as lp


class _CustomBar(lp.Bar):
    pass


def independent_layers(
    line: lp.Line, bar: lp.Bar, horizontal: lp.HorizontalBar, custom: _CustomBar,
    frame: pd.DataFrame, polars_frame: pl.DataFrame,
    array: NDArray[np.float64], values: pl.Series,
) -> None:
    color = lp.Color(20, 40, 60, alpha=128)
    layout = lp.TextLayout(alignment="start")
    style = lp.Theme.publication().typography.root.title
    assert_type(
        line.point_layer(
            x=[0.0], y=[1.0], labels=None, color=None, radius=None, alpha=None,
            legend_label=None, label_text_layout=None,
        ),
        lp.Line,
    )
    assert_type(
        line.point_layer(
            frame, x=lp.col[("x", 0)], y="y", labels=lp.col["label"],
            color=color, radius=3, alpha=128,
            legend_label=lp.Text("Layer", style=style, layout=layout),
            legend_label_style=None, legend_label_layout=None,
            label_text_layout=layout,
        ),
        lp.Line,
    )
    assert_type(
        line.point_layer(
            array, x=0, y=lp.col[1], legend_label="",
            legend_label_style=style, legend_label_layout=layout,
        ),
        lp.Line,
    )
    assert_type(
        bar.point_layer(
            None, category=["a"], value=(1.0,), color=color, radius=2, series=None,
        ),
        lp.Bar,
    )
    assert_type(
        horizontal.point_layer(
            polars_frame, category=lp.col["category"], value=values,
            color=color, radius=2, series="series",
        ),
        lp.HorizontalBar,
    )
    assert_type(
        custom.point_layer(
            category=["a"], value=[1], color=color, radius=2,
        ).significance_attached(target_value="a", mark="*"),
        _CustomBar,
    )


def attached_forms(
    bar: lp.Bar, horizontal: lp.HorizontalBar, coefficient: lp.Coefficient,
    frame: pd.DataFrame, polars_frame: pl.DataFrame, array: NDArray[np.float64],
) -> None:
    numeric_pairs: list[tuple[str, float]] = [("low", 0.01), ("low", 0.02)]
    glyph_pairs: list[tuple[str, Literal["*"]]] = [("low", "*")]
    mapping: Mapping[str, float] = {"low": 0.01}
    assert_type(
        bar.significance_attached(
            target=("a",), marks=[0.01], target_series=["s"],
        ),
        lp.Bar,
    )
    assert_type(
        horizontal.significance_attached_from_data(
            frame, target=lp.col["target"], marks=lp.col["marks"],
            mapping=numeric_pairs, target_series=None,
        ),
        lp.HorizontalBar,
    )
    assert_type(
        coefficient.significance_attached(target_value="term", mark=1),
        lp.Coefficient,
    )
    assert_type(
        bar.significance_attached_in_facet(
            target_value=("a", "s"), mark="*", facet=["panel"],
            target_series=None,
        ),
        lp.Bar,
    )
    assert_type(
        coefficient.significance_attached_in_facet_from_data(
            array, target=lp.col[0], marks=lp.col[1], facet_value="",
        ),
        lp.Coefficient,
    )
    assert_type(
        horizontal.significance_attached_in_grid(
            target=["a"], marks=["low"], mapping=glyph_pairs,
            facet_row=["r"], facet_column=["c"],
        ),
        lp.HorizontalBar,
    )
    assert_type(
        bar.significance_attached_in_grid_from_data(
            polars_frame, target_value="a", marks="marks", mapping=mapping,
            facet_value=("r", "c"),
        ),
        lp.Bar,
    )


def pairwise_forms(
    bar: lp.Bar, stacked: lp.StackedBar, horizontal: lp.StackedHorizontalBar,
    boxplot: lp.Boxplot, violin: lp.Violin, raincloud: lp.Raincloud,
    frame: pd.DataFrame,
) -> None:
    integer_pairs: list[tuple[str, int]] = [("none", 1)]
    tuple_pairs: tuple[tuple[str, Literal["ns"]], ...] = (("none", "ns"),)
    assert_type(
        stacked.significance_pairwise(
            left_value=("a", "s"), right_value=("b", "s"), marks=(0.01,),
            left_series=None, right_series=None,
        ),
        lp.StackedBar,
    )
    assert_type(
        horizontal.significance_pairwise_from_data(
            None, left=["a"], left_series=["s"], right_value=("b", "s"),
            marks=["none"], mapping=integer_pairs,
        ),
        lp.StackedHorizontalBar,
    )
    assert_type(
        boxplot.significance_pairwise_in_facet(
            left_value="a", right=["b"], right_series=["s"], mark="**",
            facet_value="panel",
        ),
        lp.Boxplot,
    )
    assert_type(
        violin.significance_pairwise_in_facet_from_data(
            frame, left=lp.col["left"], right=lp.col["right"],
            marks=lp.col["marks"], mapping=tuple_pairs, facet="panel",
        ),
        lp.Violin,
    )
    assert_type(
        raincloud.significance_pairwise_in_grid(
            left=["a"], right_value="b", mark=0.01,
            facet_row=["r"], facet_column=["c"],
        ),
        lp.Raincloud,
    )
    assert_type(
        bar.significance_pairwise_in_grid_from_data(
            left_value="a", right_value="b", mark="***", facet_value=("r", "c"),
        ),
        lp.Bar,
    )


def rejected_forms(
    scatter: lp.Scatter, line: lp.Line, bar: lp.Bar,
    coefficient: lp.Coefficient, boxplot: lp.Boxplot, frame: pd.DataFrame,
) -> None:
    color = lp.Color(20, 40, 60)
    numeric_marks: list[float] = [0.01]
    scatter.point_layer(x=[0], y=[1])  # type: ignore[attr-defined]
    coefficient.significance_pairwise(left_value="a", right_value="b", mark="*")  # type: ignore[attr-defined]
    boxplot.significance_attached(target_value="a", mark="*")  # type: ignore[attr-defined]
    bar.point_layer(category=["a"], value=[1])  # type: ignore[call-arg]
    bar.point_layer(category=["a"], value=[1], color=color, radius=2, alpha=128)  # type: ignore[call-arg]
    line.point_layer(x=[0], y=[1], legend_label=1)  # type: ignore[call-overload]
    line.point_layer(x=[0], y=[1], legend_label_style=None)  # type: ignore[call-overload]
    line.point_layer(x=[0], y=[1], alpha=0.5)  # type: ignore[call-overload]
    bar.significance_attached(["a"], marks=[0.01])  # type: ignore[call-overload]
    bar.significance_attached_from_data(None, ["a"], mark="*")  # type: ignore[call-overload]
    bar.significance_attached(mark="*")  # type: ignore[call-overload]
    bar.significance_attached(target=["a"], target_value="a", mark="*")  # type: ignore[call-overload]
    bar.significance_pairwise(left_value="a", mark="*")  # type: ignore[call-overload]
    bar.significance_pairwise(left_value="a", left_series=["s"], right_value="b", mark="*")  # type: ignore[call-overload]
    bar.significance_attached(target_value=None, mark="*")  # type: ignore[call-overload]
    bar.significance_attached(target=["a"], marks=numeric_marks, mapping={"a": "*"})  # type: ignore[arg-type]
    bar.significance_attached(target=["a"], marks=["low"])  # type: ignore[arg-type]
    bar.significance_attached(target=["a"], marks=[0.01], mark="*")  # type: ignore[call-overload]
    bar.significance_attached(target=["a"], marks=["low"], mapping=None)  # type: ignore[call-overload]
    bar.significance_attached(target=frame, mark="*")  # type: ignore[call-overload]
    bar.significance_attached(target_value="a", mark="*", facet_value="panel")  # type: ignore[call-overload]
    bar.significance_attached_in_facet(target_value="a", mark="*", facet=["p"], facet_value="p")  # type: ignore[call-overload]
    bar.significance_pairwise_in_grid(left_value="a", right_value="b", mark="*", facet_row=["r"])  # type: ignore[call-overload]
    bar.significance_pairwise_in_grid(left_value="a", right_value="b", mark="*", facet_row=["r"], facet_column=["c"], facet_value=("r", "c"))  # type: ignore[call-overload]
