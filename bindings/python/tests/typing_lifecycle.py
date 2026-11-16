"""Static contracts for the installed lifecycle package; these functions do not run."""

from typing_extensions import assert_type

from lichtung_plot import (
    Canvas,
    NumericValueContext,
    Output,
    Scatter,
    TickPositions,
    scatter,
)


def owner_contract(destination: str) -> None:
    plot = scatter(x=[0.0, 1.0], y=[1.0, 2.0])
    assert_type(plot, Scatter)
    assert_type(plot.title("Results").copy(), Scatter)

    canvas = Canvas.grid(1, 2)
    assert_type(
        canvas.panel(0, 0, plot, row_span=1, column_span=1, overwrite=True),
        Canvas,
    )
    assert_type(canvas.copy(), Canvas)

    output = plot.svg(destination)
    assert_type(output, Output)
    assert_type(output.svg(destination), Output)

    # Scatter must not inherit setters for another scientific capability.
    plot.line_alpha(None)  # type: ignore[attr-defined]


def factory_only_contract() -> None:
    assert_type(TickPositions.data([0.0, 1.0]), TickPositions)
    TickPositions()  # type: ignore[call-arg]
    NumericValueContext()  # type: ignore[call-arg]
    Output()  # type: ignore[call-arg]
