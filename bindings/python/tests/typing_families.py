"""Installed exact-family typing contracts; this module is not executed."""

from typing_extensions import assert_type

import lichtung_plot as lp


def curve_family_returns() -> None:
    assert_type(lp.histogram(samples=[1.0, 2.0]), lp.Histogram)
    assert_type(lp.density(value=[1.0, 2.0]), lp.Density)
    assert_type(lp.qq(value=[1.0, 2.0]), lp.Qq)
    assert_type(lp.line(x=[0.0, 1.0], y=[1.0, 2.0]), lp.Line)
    assert_type(
        lp.trajectory(x=["before", "after"], y=[1.0, 2.0], group=["a", "a"]),
        lp.Trajectory,
    )
    assert_type(
        lp.line_path(x=[1.0, 0.0], y=[1.0, 2.0], group=["a", "a"]),
        lp.LinePath,
    )
    assert_type(lp.regression(x=[0.0, 1.0], y=[1.0, 2.0]), lp.Regression)
    assert_type(lp.marginal(x=[0.0, 1.0], y=[1.0, 2.0]), lp.Marginal)


def estimate_and_bar_returns() -> None:
    assert_type(
        lp.point_interval(category=["a"], estimate=[1.0], lower=[0.0], upper=[2.0]),
        lp.PointInterval,
    )
    assert_type(
        lp.interaction(
            condition=["a"], estimate=[1.0], group=["g"],
            lower=[0.0], upper=[2.0],
        ),
        lp.Interaction,
    )
    assert_type(lp.coefficient(term=["a"], estimate=[1.0]), lp.Coefficient)
    assert_type(
        lp.forest(
            term=["a"], estimate=[1.0], lower=[0.0], upper=[2.0], reference=0.0,
        ),
        lp.Forest,
    )
    assert_type(
        lp.dumbbell(
            category=["a"], left=[0.0], right=[1.0],
            left_label="Before", left_color=lp.Color(20, 40, 60),
            right_label="After", right_color=lp.Color(80, 100, 120),
        ),
        lp.Dumbbell,
    )
    assert_type(lp.bar(category=["a"], samples=[1.0]), lp.Bar)
    assert_type(
        lp.horizontal_bar(category=["a"], estimate=[1.0]), lp.HorizontalBar
    )
    assert_type(
        lp.stacked_bar(category=["a"], value=[1.0], series=["s"]), lp.StackedBar
    )
    assert_type(
        lp.stacked_horizontal_bar(category=["a"], value=[1.0], series=["s"]),
        lp.StackedHorizontalBar,
    )
    assert_type(
        lp.waterfall(category=["a"], delta=[1.0], base=10.0), lp.Waterfall
    )


def distribution_and_grid_returns() -> None:
    assert_type(lp.heatmap(grid=[[1.0, 2.0], [3.0, 4.0]]), lp.Heatmap)
    assert_type(lp.boxplot(category=["a", "a"], value=[1.0, 2.0]), lp.Boxplot)
    assert_type(lp.violin(category=["a", "a"], value=[1.0, 2.0]), lp.Violin)
    assert_type(lp.raincloud(category=["a", "a"], value=[1.0, 2.0]), lp.Raincloud)
    assert_type(lp.matrix(variables=[[1.0, 2.0], [3.0, 4.0]]), lp.Matrix)
    assert_type(
        lp.beeswarm(category=["a"], value=[1.0], color=[0.5]), lp.Beeswarm
    )


def rejected_family_capabilities_and_constructor_forms(
    heatmap: lp.Heatmap, coefficient: lp.Coefficient, waterfall: lp.Waterfall,
    matrix: lp.Matrix, marginal: lp.Marginal, regression: lp.Regression,
    bars: lp.Bar,
) -> None:
    format = lp.NumericFormat()
    heatmap.y_limits(0, 1)  # type: ignore[attr-defined]
    heatmap.annotations(None)  # type: ignore[arg-type]
    heatmap.layout("lower_triangle")  # type: ignore[call-overload]
    heatmap.layout("cartesian", diagonal=True)  # type: ignore[call-overload]
    heatmap.x_category_order("value_ascending")  # type: ignore[arg-type]
    matrix.x_tick_format(format)  # type: ignore[attr-defined]
    matrix.facet_row(["a"])  # type: ignore[attr-defined]
    matrix.guide_horizontal(0, color=lp.Color(0, 0, 0), width=1, pattern="solid")  # type: ignore[attr-defined]
    matrix.text_annotation(lp.TextPosition.corner("top_left"), "x")  # type: ignore[attr-defined]
    marginal.facet_column(["a"])  # type: ignore[attr-defined]
    marginal.bins(count=4)  # type: ignore[attr-defined]
    regression.hue(["a"])  # type: ignore[attr-defined]
    regression.band([0], [1])  # type: ignore[call-arg, arg-type]
    bars.x_limits(0, 1)  # type: ignore[attr-defined]
    bars.hue(["a"])  # type: ignore[attr-defined]
    bars.estimate_labels(format)  # type: ignore[attr-defined]
    bars.category_order("bogus")  # type: ignore[arg-type]
    coefficient.series(["a"])  # type: ignore[attr-defined]
    coefficient.estimate_status([True])  # type: ignore[list-item]
    waterfall.y_tick_generation(lp.TickGeneration.automatic())  # type: ignore[attr-defined]
    lp.forest(term=["a"], estimate=[1], lower=[0], upper=[2])  # type: ignore[call-arg]
    lp.waterfall(category=["a"], delta=[1])  # type: ignore[call-arg]
    lp.dumbbell(category=["a"], left=[0], right=[1])  # type: ignore[call-arg]
    lp.bar(category=["a"], samples=[1], estimate=[1])  # type: ignore[call-overload]
    lp.bar(category=["a"], estimate=[1], error_bar="hidden")  # type: ignore[call-overload]
    lp.bar(category=["a"], samples=[1], error_bar=None)  # type: ignore[call-overload]
    lp.coefficient(term=["a"], estimate=[1], lower=[0])  # type: ignore[call-overload]
    lp.heatmap(grid=[[1]], x=["a"], y=["b"], fill=[1])  # type: ignore[call-overload]
    lp.line(y=[[1, 2]])  # type: ignore[call-overload]
    lp.line(x=[0], y=[1]).column_labels("AB")  # type: ignore[arg-type]
    lp.Line()  # type: ignore[call-arg]
    lp.raincloud(category=["a"], value=[1], hue=["g"])  # type: ignore[call-overload]
    lp.matrix()  # type: ignore[call-overload]
    lp.histogram(samples=[1, 2]).bins(count=4, width=0.5)  # type: ignore[call-overload]
    lp.forest(term=["a"], estimate=[1], lower=[0], upper=[2], reference=None)  # type: ignore[arg-type]
