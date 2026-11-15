"""Synthetic power readings with native numeric policies and frozen publication."""

from pathlib import Path
import sys

from lichtung_plot import (
    NumericFormat,
    NumericLocale,
    NumericPrecision,
    NumericPrefixes,
    NumericUnit,
    TickGeneration,
    scatter,
)


def main(directory: Path) -> None:
    # All observation values remain in seconds and watts. Only display spelling
    # uses the selected locale, precision, common SI prefix and unit.
    power_format = NumericFormat.from_specifier(",.2f").copy_with(
        precision=NumericPrecision.significant(3),
        locale=NumericLocale.named("de-DE"),
        trailing_zeros="trim",
        prefixes=NumericPrefixes.si(
            "whole_set", promotion="keep_threshold"
        ),
        unit=NumericUnit("W", placement="per_label"),
    )
    plot = (
        scatter(
            x=[0.0, 1.0, 2.0, 3.0, 4.0, 5.0],
            y=[1250.0, 1750.0, 1500.0, 2250.0, 2000.0, 2500.0],
        )
        .size(980, 640)
        .title("Synthetic power readings")
        .x_label("Time (seconds)")
        .y_label("Power")
        .caption(
            "Illustrative inputs in W; display prefixes do not change bound observations."
        )
        .x_tick_format(NumericFormat(precision=NumericPrecision.fractional(0)))
        .x_tick_generation(
            TickGeneration.interval(1.0, 0.0, space="data", requirement="fixed")
        )
        .y_tick_format(power_format)
        .y_tick_generation(
            TickGeneration.interval(
                250.0, 0.0, space="data", requirement="preferred"
            )
        )
    )
    directory.mkdir(parents=True, exist_ok=True)
    output = plot.svg(directory / "complete-numeric")
    output.svg_preserve_text(directory / "complete-numeric-text")
    output.png_preview(directory / "complete-numeric-preview")


if __name__ == "__main__":
    main(Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd())
