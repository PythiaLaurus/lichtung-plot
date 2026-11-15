"""Installed alpha call forms; this module is not executed."""

from typing import Callable

from typing_extensions import assert_type

import lichtung_plot as lp


def scoped_returns(
    scatter: lp.Scatter, line: lp.Line, histogram: lp.Histogram,
    heatmap: lp.Heatmap,
) -> None:
    assert_type(scatter.copy().point_alpha([0, 255]).point_alpha_theme(), lp.Scatter)
    assert_type(heatmap.fill_alpha_cells((64, 128, 192, 255)), lp.Heatmap)
    assert_type(
        line.line_alpha_keyed(
            [64, 192], keys=(("hue", ("a", "b")), (lp.col["hue"], ("c", "d"))),
        ),
        lp.Line,
    )
    assert_type(
        line.fill_alpha_columns([64, 192], column_key=[0, 1], other_keys=()),
        lp.Line,
    )
    assert_type(line.point_alpha_single(value=128), lp.Line)
    assert_type(line.point_alpha_single(input=lp.col["opacity"]), lp.Line)
    keys: list[tuple[str, list[str]]] = [("group", ["g"])]
    assert_type(
        line.line_alpha_keyed_from_data(None, input=(128,), keys=keys),
        lp.Line,
    )
    assert_type(
        line.fill_alpha_columns_from_data(
            input=[64, 192], column_key=[0, 1],
        ),
        lp.Line,
    )
    assert_type(
        histogram.fill_alpha_bins(
            [64, 192], lower=[0.0, 1.0], upper=[1.0, 2.0],
            facets=(("facet", ("a", "b")),),
        ),
        lp.Histogram,
    )


def mapped_returns(
    line: lp.Line, histogram: lp.Histogram,
    numeric_mapper: Callable[[float], int],
    categorical_mapper: Callable[[str], int],
) -> None:
    assert_type(
        line.point_alpha_linear([0.0, 1.0], (0.0, 1.0), (0, 255)), lp.Line,
    )
    assert_type(
        line.point_alpha_map([0.0, 1.0], linear=((0.0, 1.0), (32, 224))),
        lp.Line,
    )
    assert_type(
        line.line_alpha_single_map([2.0], log10=((1.0, 10.0), (32, 224))),
        lp.Line,
    )
    bounds: list[int] = [0]
    assert_type(
        line.point_alpha_cells_map(
            [0.0, 1.0], threshold=(bounds, [32, 224]),
        ),
        lp.Line,
    )
    assert_type(
        line.fill_alpha_keyed_map(
            [0.5], keys=(("group", ("g",)),),
            continuous=((0.0, 1.0), numeric_mapper),
        ),
        lp.Line,
    )
    assert_type(
        line.point_alpha_single_map_categories(["a"], mapper=categorical_mapper),
        lp.Line,
    )
    assert_type(
        line.line_alpha_columns_map_from_data(
            input=[1.0, 10.0], column_key=("a", "b"),
            log10=((1.0, 10.0), (32, 224)),
        ),
        lp.Line,
    )
    assert_type(
        histogram.fill_alpha_bins_map_categories_from_data(
            None, input=("a", "b"), lower=(0.0, 1.0), upper=(1.0, 2.0),
            mapper=categorical_mapper,
        ),
        lp.Histogram,
    )


def exact_capabilities_and_rejected_forms(
    scatter: lp.Scatter, line: lp.Line, heatmap: lp.Heatmap,
    histogram: lp.Histogram,
    numeric_float_mapper: Callable[[float], float],
    categorical_float_mapper: Callable[[str], float],
) -> None:
    float_opacity: list[float] = [0.5]
    nonsemantic_keys: list[tuple[float, list[str]]] = [(1.5, ["g"])]
    scatter.line_alpha([255])  # type: ignore[attr-defined]
    heatmap.point_alpha([255])  # type: ignore[attr-defined]
    scatter.fill_alpha([255])  # type: ignore[attr-defined]
    scatter.point_alpha(None)  # type: ignore[arg-type]
    scatter.point_alpha(float_opacity)  # type: ignore[arg-type]
    line.point_alpha_map([0.0], linear=((0.0, 1.0), (0, 255)), log10=((1.0, 10.0), (0, 255)))  # type: ignore[call-overload]
    line.point_alpha_map([0.0])  # type: ignore[call-overload]
    line.point_alpha_map([0.0], mapping=((0.0, 1.0), (0, 255)))  # type: ignore[call-overload]
    line.point_alpha_single()  # type: ignore[call-overload]
    line.point_alpha_single(value=128, input=[128])  # type: ignore[call-overload]
    line.point_alpha_single(value=None)  # type: ignore[call-overload]
    line.point_alpha_map([0.5], continuous=((0.0, 1.0), numeric_float_mapper))  # type: ignore[arg-type]
    line.point_alpha_map_categories(["a"], mapper=categorical_float_mapper)  # type: ignore[arg-type]
    line.line_alpha_keyed_from_data(None, [128], keys=())  # type: ignore[misc]
    line.line_alpha_keyed([128], keys=nonsemantic_keys)  # type: ignore[type-var]
    line.fill_alpha_bins([128], lower=[0.0], upper=[1.0])  # type: ignore[attr-defined]
    histogram.fill_alpha_bins_from_data(None, [128], lower=[0.0], upper=[1.0])  # type: ignore[misc]
    line.point_alpha_from_data(None, input=[128])  # type: ignore[attr-defined]
