"""Synthetic scientific views with native snapshots and illustrative supplied bounds."""

# ANCHOR: example
from pathlib import Path
import sys

from lichtung_plot import (
    Canvas,
    ColorDomain,
    ContinuousColorScale,
    NumericFormat,
    NumericPrecision,
    bar,
    forest,
    heatmap,
    line,
)


def main(directory: Path) -> None:
    trends = (
        line(
            x=[0.0, 1.0, 2.0, 3.0],
            y=[[1.0, 1.2], [1.3, 1.8], [1.6, 2.0], [1.8, 2.5]],
        )
        .column_labels(["Control", "Stimulus"])
        .title("Paired synthetic trends")
        .x_label("Time (hours)")
        .y_label("Response (arbitrary units)")
    )
    summaries = (
        bar(
            category=["Control", "Dose 1", "Dose 2"],
            estimate=[2.0, 3.1, 3.7],
            lower=[1.5, 2.6, 3.1],
            upper=[2.5, 3.6, 4.3],
        )
        .title("Supplied illustrative intervals")
        .x_label("Condition")
        .y_label("Response (arbitrary units)")
    )
    ratios = (
        forest(
            term=["Outcome A", "Outcome B", "Outcome C"],
            estimate=[0.8, 1.4, None],
            lower=[0.6, 1.1, None],
            upper=[1.0, 1.8, None],
            reference=1.0,
        )
        .estimate_status(["estimated", "estimated", "not_estimable"])
        .scale("log10")
        .title("Explicitly unavailable estimate")
        .x_label("Ratio (logarithmic scale)")
        .y_label("Outcome")
    )
    cells = (
        heatmap(grid=[[0.1, 0.6, -0.2], [-0.4, 0.3, 0.8]])
        .x_labels(["Start", "Week 1", "Week 2"])
        .y_labels(["Process A", "Process B"])
        .continuous_color_scale(
            ContinuousColorScale.diverging(0.0, domain=ColorDomain(-1.0, 1.0))
        )
        .annotations(NumericFormat(precision=NumericPrecision.fractional(1)))
        .heatmap_highlight_cells([[True, False, False], [False, False, True]])
        .title("Supplied cell values")
        .x_label("Time point")
        .y_label("Process")
    )
    canvas = (
        Canvas.grid(2, 2)
        .size(1280, 960)
        .title("Scientific views from native snapshots")
        .caption(
            "Synthetic values; supplied bounds are illustrative, not inferential results."
        )
        .panel(0, 0, trends)
        .panel(0, 1, summaries)
        .panel(1, 0, ratios)
        .panel(1, 1, cells)
    )
    directory.mkdir(parents=True, exist_ok=True)
    output = canvas.svg(directory / "scientific-composition")
    output.svg_preserve_text(directory / "scientific-composition-text")
    output.png_preview(directory / "scientific-composition-preview")


if __name__ == "__main__":
    main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
# ANCHOR_END: example
