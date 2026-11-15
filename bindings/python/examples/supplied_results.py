"""Synthetic observations and illustrative supplied intervals/display p-values."""

# ANCHOR: example
from pathlib import Path
import sys

from lichtung_plot import Canvas, Color, bar, line


def main(directory: Path) -> None:
    trend = (
        line(x=[0.0, 1.0, 2.0, 3.0], y=[0.6, 1.1, 1.6, 2.0])
        .band([0.4, 0.8, 1.3, 1.7], [0.8, 1.4, 1.9, 2.3])
        .markers(False)
        .point_layer(
            x=[0.1, 0.6, 1.1, 1.8, 2.3, 2.9],
            y=[0.7, 0.9, 1.4, 1.5, 1.9, 2.1],
            color=Color(25, 130, 100), radius=4, alpha=195,
            legend_label="Supplied observations",
        )
        .title("Supplied trend and independent observations")
        .x_label("Time (hours)")
        .y_label("Response (arbitrary units)")
    )
    conditions = (
        bar(
            category=["Control", "Low", "High"], estimate=[1.0, 1.6, 2.1],
            lower=[0.8, 1.3, 1.8], upper=[1.2, 1.9, 2.4],
        )
        .point_layer(
            category=["Control", "Control", "Low", "Low", "High", "High"],
            value=[0.9, 1.1, 1.4, 1.8, 1.9, 2.3],
            color=Color(35, 35, 35, alpha=165), radius=4,
        )
        .significance_pairwise(
            left_value="Control", right=["Low", "High"], marks=[0.04, 0.004],
        )
        .legend("hidden")
        .title("Supplied intervals and display comparisons")
        .x_label("Condition")
        .y_label("Response (arbitrary units)")
    )
    canvas = (
        Canvas.grid(1, 2).size(1700, 1000)
        .title("Synthetic assays: independent observations and supplied results")
        .caption(
            "Synthetic data and illustrative supplied bounds/display p-values. "
            "The renderer does not fit trends, compute tests or adjust p-values."
        )
        .panel(0, 0, trend).panel(0, 1, conditions)
    )
    directory.mkdir(parents=True, exist_ok=True)
    output = canvas.svg(directory / "supplied-results")
    output.svg_preserve_text(directory / "supplied-results-text")
    output.png_preview(directory / "supplied-results-preview")


if __name__ == "__main__":
    main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
# ANCHOR_END: example
