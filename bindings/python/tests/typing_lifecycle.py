"""Static contracts for the installed lifecycle package; these functions do not run."""

from typing_extensions import assert_type

from lichtung_plot import (
    Canvas,
    Color,
    NumericValueContext,
    Output,
    Scatter,
    Text,
    TextLayout,
    TextTheme,
    Theme,
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


def immutable_text_contract(plot: Scatter) -> None:
    theme = Theme.nature()
    title = theme.typography.root.title
    assert_type(title, TextTheme)
    larger = title.copy_with(font_size=24, color=Color.hex("#194A75"))
    root = theme.typography.root.copy_with(title=larger)
    derived = theme.copy_with(
        typography=theme.typography.copy_with(root=root),
    )
    assert_type(derived, Theme)
    assert_type(derived.typography.root.title.font_size, float)

    text = Text("Results", style=larger, layout=TextLayout(alignment="end"))
    assert_type(text.copy_with(style=None), Text)
    assert_type(text.style, TextTheme | None)
    assert_type(text.layout, TextLayout | None)
    assert_type(plot.title(text, style=None, layout=TextLayout()), Scatter)

    # Targeted ignores must remain necessary under --warn-unused-ignores.
    title.font_size = 30  # type: ignore[misc]
    text.style = None  # type: ignore[misc]
    title.copy_with(font_size=None)  # type: ignore[arg-type]
    Text("Results", style=42)  # type: ignore[arg-type]


def factory_only_contract() -> None:
    assert_type(TickPositions.data([0.0, 1.0]), TickPositions)
    TickPositions()  # type: ignore[call-arg]
    NumericValueContext()  # type: ignore[call-arg]
    Output()  # type: ignore[call-arg]
