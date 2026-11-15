"""Synthetic signals with supplied coverage, not a statistical confidence model."""

# ANCHOR: example
from pathlib import Path
import sys

from lichtung_plot import (
    Canvas,
    ColorDomain,
    ContinuousColorScale,
    NumericFormat,
    NumericPrecision,
    heatmap,
    scatter,
)


def coverage_alpha(raw: str) -> int:
    return {"Complete": 240, "Partial": 150, "Sparse": 85}[raw]


def main(directory: Path) -> None:
    observations = (
        scatter(
            x=[0.0, 0.5, 1.0, 2.0, 4.0, 8.0],
            y=[0.1, 0.35, 0.6, 0.9, 1.1, 1.25],
        )
        .point_alpha_map_categories(
            ["Complete", "Partial", "Complete", "Sparse", "Partial", "Complete"],
            mapper=coverage_alpha,
        )
        .title("Response with supplied coverage classes")
        .x_label("Dose (micromolar)")
        .y_label("Response (arbitrary units)")
    )
    signals = (
        heatmap(grid=[[0.1, 0.6, -0.2], [-0.4, 0.3, 0.8]])
        .x_labels(["Start", "Week 1", "Week 2"])
        .y_labels(["Process A", "Process B"])
        .continuous_color_scale(
            ContinuousColorScale.diverging(0.0, domain=ColorDomain(-1.0, 1.0))
        )
        .annotations(NumericFormat(precision=NumericPrecision.fractional(1)))
        .fill_alpha_cells_map(
            [1.0, 0.8, 0.4, 0.6, 0.9, 1.0],
            linear=((0.0, 1.0), (85, 255)),
        )
        .title("Cell signals with supplied coverage fractions")
        .x_label("Time point")
        .y_label("Process")
    )
    canvas = (
        Canvas.grid(1, 2)
        .size(1600, 900)
        .title("Synthetic assays: alpha encodes supplied coverage")
        .caption(
            "Synthetic values; coverage is supplied display metadata, not confidence. "
            "Alpha does not filter observations or alter their values."
        )
        .panel(0, 0, observations)
        .panel(0, 1, signals)
    )
    directory.mkdir(parents=True, exist_ok=True)
    output = canvas.svg(directory / "alpha-mappings")
    output.svg_preserve_text(directory / "alpha-mappings-text")
    output.png_preview(directory / "alpha-mappings-preview")


if __name__ == "__main__":
    main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
# ANCHOR_END: example
