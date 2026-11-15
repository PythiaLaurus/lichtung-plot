"""Render synthetic observations, then reuse their frozen Canvas output."""

from pathlib import Path
import sys

from lichtung_plot import Canvas, Text, Theme, scatter


def main() -> None:
    directory = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()
    directory.mkdir(parents=True, exist_ok=True)
    plot = (
        scatter(
            x=[0.0, 1.0, 2.0, 3.0],
            y=[0.6, 1.5, 1.1, 2.2],
            hue=["control", "treated", "control", "treated"],
        )
        .title(Text("Synthetic observations"))
        .x_label("Time (hours)")
        .y_label("Response (arbitrary units)")
    )
    canvas = Canvas.grid(1, 1).theme(Theme.publication()).panel(0, 0, plot)
    output = canvas.svg(directory / "reusable-canvas")
    output.svg_preserve_text(directory / "reusable-canvas-text")
    output.png_preview(directory / "reusable-canvas-preview")
    plot.title("A later edit does not change the captured Canvas")
    canvas.copy().title("Independent composition").svg(directory / "canvas-copy")


if __name__ == "__main__":
    main()
