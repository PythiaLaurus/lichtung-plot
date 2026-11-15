"""Synthetic observations through four routes and one native publication path."""

from __future__ import annotations

import argparse
from decimal import Decimal
from pathlib import Path

import numpy as np
import pandas as pd
import polars as pl

from lichtung_plot import Canvas, col, scatter


def main(destination: Path) -> None:
    time = [0.0, 5.0, 10.0, 20.0]
    response = [1.0, 2.2, 1.8, 3.4]
    cohorts = ["Control", "Treated", "Control", "Treated"]
    arrays = np.array([time, response]).T
    pandas_frame = pd.DataFrame(
        {
            "Time": time,
            "Response": response,
            "Hue": cohorts,
            # This unbound column must not be imported as numeric data.
            "Unused": [Decimal("0.1")] * len(time),
        }
    )
    polars_frame = pl.DataFrame(
        {
            "Time": time,
            "Response": response,
            "Hue": cohorts,
            # Complete names are retained without importing this Struct column.
            "Unused": [{"nested": 1}] * len(time),
        }
    )
    plots = [
        scatter(x=time, y=response, hue=cohorts),
        scatter(arrays, x=col[0], y=col[1], hue=np.array(cohorts)),
        scatter(
            pandas_frame, x=col["Time"], y=col["Response"], hue=col["Hue"]
        ),
        scatter(
            polars_frame, x=col["Time"], y=col["Response"], hue=col["Hue"]
        ),
    ]
    canvas = Canvas.grid(2, 2).size(1400, 1000)
    for index, (name, plot) in enumerate(
        zip(("Native Python", "NumPy", "pandas", "Eager Polars"), plots)
    ):
        plot.title(name).x_label("Time (h)").y_label("Response")
        canvas.panel(index // 2, index % 2, plot)
    destination.mkdir(parents=True, exist_ok=True)
    canvas.svg_preserve_text(destination / "four_inputs.svg")
    canvas.png_preview(destination / "four_inputs.png")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path, nargs="?", default=Path.cwd())
    main(parser.parse_args().destination)
