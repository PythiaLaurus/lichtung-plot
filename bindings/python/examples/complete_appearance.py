"""Synthetic observations with native nested appearance and authored Text values."""

from pathlib import Path
import sys

from lichtung_plot import (
    Canvas,
    Color,
    OpaqueColor,
    PointOutlineTheme,
    QualitativePalette,
    RelativeSpacing,
    StrokeTheme,
    Text,
    TextAnchor,
    TextEllipsis,
    TextFit,
    TextLayout,
    TextOrientation,
    TextOverflow,
    TextRotation,
    TextVisibility,
    Theme,
    scatter,
)


def main(directory: Path) -> None:
    original = Theme.publication()
    primary = original.marks.points.primary.copy_with(
        shape="triangle_up",
        outline=PointOutlineTheme(
            color=Color.hex("#26364A"),
            width_ratio=RelativeSpacing.ratio(1, 8),
        ),
    )
    points = original.marks.points.copy_with(
        primary=primary,
        radii=original.marks.points.radii.copy_with(scatter=6),
    )
    theme = original.copy_with(
        axes=original.axes.copy_with(
            grid=StrokeTheme(
                color=Color(220, 225, 230), width=1.0, pattern="short_dash"
            )
        ),
        marks=original.marks.copy_with(points=points),
        color_scales=original.color_scales.copy_with(
            qualitative=QualitativePalette.fixed(
                [OpaqueColor(25, 74, 117), OpaqueColor(186, 91, 35)]
            )
        ),
    )
    title_layout = TextLayout(
        fit=TextFit.range(16.0, 24.0),
        orientation=TextOrientation.fixed(TextRotation.horizontal()),
        anchor=TextAnchor.center(),
        overflow=TextOverflow(
            failure="error", ellipsis=TextEllipsis("middle", max_lines=1)
        ),
        visibility=TextVisibility.all(),
    )
    plot = (
        scatter(
            x=[0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            y=[0.7, 1.2, 1.0, 1.9, 1.6, 2.5],
            hue=["Control", "Treated"] * 3,
        )
        .x_label("Time (hours)")
        .y_label("Response (arbitrary units)")
    )
    canvas = (
        Canvas.grid(1, 1)
        .size(960, 680)
        .theme(theme)
        .title(Text("Synthetic observations", layout=title_layout))
        .caption("Illustrative values, not a study or an inferential analysis.")
        .panel(0, 0, plot)
    )
    directory.mkdir(parents=True, exist_ok=True)
    output = canvas.svg(directory / "complete-appearance")
    output.svg_preserve_text(directory / "complete-appearance-text")
    output.png_preview(directory / "complete-appearance-preview")


if __name__ == "__main__":
    main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
