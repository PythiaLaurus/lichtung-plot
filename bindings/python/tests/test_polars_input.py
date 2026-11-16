"""Installed-wheel consumers of genuine Polars C streams and exact row IDs."""

import gc
from pathlib import Path
import weakref
import xml.etree.ElementTree as ET

import polars as pl
import pytest

from lichtung_plot import DataError, TickGeneration, TickPositions, scatter

SVG = "{http://www.w3.org/2000/svg}"


def svg_text(path: Path) -> str:
    return "\n".join(
        "".join(node.itertext())
        for node in ET.parse(path).getroot().iter(f"{SVG}text")
    )


def test_chunked_slice_snapshot_survives_series_release_and_keeps_aligned_domain(
    tmp_path: Path,
):
    x = pl.concat(
        [
            pl.Series("Exposure", [-7.0, None, float("nan"), -0.0]),
            pl.Series("Exposure", [0.0, 2.5, None, 9.0]),
        ],
        rechunk=False,
    ).slice(1, 6)
    y = pl.Series("Response", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    assert x.n_chunks() >= 2
    x_ref, y_ref = weakref.ref(x), weakref.ref(y)
    plot = scatter(x=x, y=y)
    del x, y
    gc.collect()
    assert x_ref() is None and y_ref() is None

    native = scatter(
        x=[None, float("nan"), -0.0, +0.0, 2.5, None],
        y=[0, 1, 2, 3, 4, 5],
    ).x_label("Exposure").y_label("Response")

    def publish(current, name):
        metadata = []

        def positions(view):
            domain = view.domain
            metadata.append(
                (
                    view.axis,
                    view.owner.plot_kind,
                    view.owner.canvas_cell,
                    domain.scale,
                    domain.origin,
                    domain.data_limits,
                    domain.scale_limits,
                    view.span,
                )
            )
            return TickPositions.data(
                [0.0, 1.0, 2.5] if view.axis == "x" else [2.0, 3.0, 4.0]
            )

        current.size(640, 480)
        current.x_tick_generation(TickGeneration.function(positions))
        current.y_tick_generation(TickGeneration.function(positions))
        current.svg_preserve_text(tmp_path / name)
        return metadata

    domains = publish(plot, "slice")
    native_domains = publish(native, "slice-native-reference")
    assert domains
    assert {axis for axis, *_ in domains} == {"x", "y"}
    # Compare current padded domains, not raw extrema or callback repetition.
    assert set(domains) == set(native_domains)
    for axis, _, _, _, _, limits, _, _ in domains:
        assert limits is not None
        aligned = (-0.0, +0.0, 2.5) if axis == "x" else (2.0, 3.0, 4.0)
        assert all(limits[0] <= value <= limits[1] for value in aligned)
    text = svg_text(tmp_path / "slice.svg")
    assert "Exposure" in text and "Response" in text
    # Null-vs-NaN and signed-zero bits are checked privately before the native
    # Scatter policy intentionally drops missing/NaN coordinate observations.


def test_uint64_ids_are_not_rounded_and_uint128_overflow_is_atomic(tmp_path: Path):
    plot = scatter(
        x=pl.Series("Exposure", [0.0, float("inf")]),
        y=pl.Series("Response", [2.0, 3.0]),
    )
    # These become duplicates if an importer rounds through binary64 first.
    exact = pl.Series("identity", [2**53, 2**53 + 1], dtype=pl.UInt64)
    assert plot.source_rows(exact) is plot
    for invalid in [
        pl.Series("identity", [2**53 + 1, 2**53 + 1], dtype=pl.UInt64),
        pl.Series("identity", [2**100, 1], dtype=pl.UInt128),
    ]:
        with pytest.raises(DataError):
            plot.source_rows(invalid)
    # After both rejected replacements, the original native diagnostic identity
    # must still be the exact second ID, not the positional slot or a rounded ID.
    with pytest.raises(DataError, match=rf"source row {2**53 + 1}\b"):
        plot.svg(tmp_path / "exact-diagnostic")
    assert not (tmp_path / "exact-diagnostic.svg").exists()
