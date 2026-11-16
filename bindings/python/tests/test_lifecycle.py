"""Focused native-data and reusable-owner contracts, not scene snapshots."""

import gc
from pathlib import Path
import struct
import weakref
import xml.etree.ElementTree as ET

import pytest

from lichtung_plot import Canvas, Plot, Scatter, Text, Theme, scatter


SVG = "{http://www.w3.org/2000/svg}"


def svg_text(path: Path) -> str:
    root = ET.parse(path).getroot()
    return "\n".join("".join(node.itertext()) for node in root.iter(f"{SVG}text"))


def test_mutation_alias_copy_and_rejected_edits(tmp_path: Path) -> None:
    plot = scatter(x=[0.0, 1.0, 2.0], y=(1.0, 3.0, 2.0))
    assert isinstance(plot, Scatter)
    assert isinstance(plot, Plot)
    alias = plot
    assert plot.title(Text("Original")).theme(Theme.publication()) is plot
    assert alias.caption("Alias edit") is plot
    branch = plot.copy()
    assert isinstance(branch, Scatter)
    assert branch is not plot
    assert branch.title("Independent") is branch

    with pytest.raises(TypeError):
        plot.title("Must not be committed", style=object())
    with pytest.raises(TypeError):
        plot.hue(["valid", True, "valid"])
    with pytest.raises(AttributeError):
        plot.title = "No method shadowing"
    with pytest.raises(AttributeError):
        branch.bins
    assert not hasattr(plot, "__dict__")
    assert not hasattr(plot, "render")

    plot.svg_preserve_text(tmp_path / "original")
    branch.svg_preserve_text(tmp_path / "branch")
    original = svg_text(tmp_path / "original.svg")
    independent = svg_text(tmp_path / "branch.svg")
    assert "Original" in original and "Alias edit" in original
    assert "Independent" not in original
    assert "Must not be committed" not in original
    assert "Independent" in independent and "Alias edit" in independent
    root = ET.parse(tmp_path / "original.svg").getroot()
    assert len(list(root.iter(f"{SVG}circle"))) == 3


def test_bound_values_and_optional_hue_are_snapshots(tmp_path: Path) -> None:
    x = [0.0, None, 2.0, 3.0]
    y = [1.0, 100.0, None, 4.0]
    plot = scatter(x=x, y=y).title("Nullable observations")
    x[:] = [True]
    y[:] = [float("inf")]
    plot.svg_preserve_text(tmp_path / "snapshot")
    reference = scatter(
        x=[0.0, None, 2.0, 3.0], y=[1.0, 100.0, None, 4.0]
    ).title("Nullable observations")
    reference.svg_preserve_text(tmp_path / "reference")
    assert (tmp_path / "snapshot.svg").read_bytes() == (
        tmp_path / "reference.svg"
    ).read_bytes()
    root = ET.parse(tmp_path / "snapshot.svg").getroot()
    assert len(list(root.iter(f"{SVG}circle"))) == 2

    hue = ["control", "treated", "control"]
    constructor = scatter(x=[0.0, 1.0, 2.0], y=[2.0, 3.0, 1.0], hue=hue)
    chained = scatter(x=[0.0, 1.0, 2.0], y=[2.0, 3.0, 1.0])
    assert chained.hue(hue) is chained
    hue[:] = ["changed"]
    constructor.svg_preserve_text(tmp_path / "constructor-hue")
    chained.svg_preserve_text(tmp_path / "chained-hue")
    assert (tmp_path / "constructor-hue.svg").read_bytes() == (
        tmp_path / "chained-hue.svg"
    ).read_bytes()
    assert "Control" in svg_text(tmp_path / "chained-hue.svg")
    assert "Changed" not in svg_text(tmp_path / "chained-hue.svg")


def test_native_input_grammar_is_explicit() -> None:
    with pytest.raises(TypeError):
        scatter([0.0, 1.0], [1.0, 2.0])
    with pytest.raises(TypeError):
        scatter({"x": [0.0]}, x=[0.0], y=[1.0])
    with pytest.raises(TypeError):
        scatter(x=[True], y=[1.0])
    with pytest.raises(TypeError):
        scatter(x=[0.0], y=["1.0"])
    with pytest.raises(TypeError):
        scatter(x=[0.0])


def test_canvas_captures_native_configuration_without_python_children(
    tmp_path: Path,
) -> None:
    plot = scatter(x=[0.0, 1.0], y=[1.0, 2.0]).title("Captured")
    reference = weakref.ref(plot)
    canvas = Canvas.grid(1, 2)
    assert canvas.panel(0, 0, plot) is canvas
    plot.title("External later edit")
    canvas.panel(
        0, 1, scatter(x=(0.0, 1.0), y=(2.0, 1.0)).title("Temporary child")
    )
    del plot
    gc.collect()
    assert reference() is None
    with pytest.raises(AttributeError):
        canvas.title = "No method shadowing"
    assert not hasattr(canvas, "__dict__")

    branch = canvas.copy()
    assert branch is not canvas
    assert branch.title("Independent Canvas") is branch
    with pytest.raises(OSError):
        canvas.svg(tmp_path / "missing-canvas-parent" / "figure")
    canvas.svg_preserve_text(tmp_path / "capture")
    branch.svg_preserve_text(tmp_path / "canvas-branch")
    captured = svg_text(tmp_path / "capture.svg")
    assert "Captured" in captured and "Temporary child" in captured
    assert "External later edit" not in captured
    assert "Independent Canvas" not in captured
    assert "Independent Canvas" in svg_text(tmp_path / "canvas-branch.svg")


def test_new_terminals_frozen_output_and_io_failure_preserve_handles(
    tmp_path: Path,
) -> None:
    plot = scatter(x=[0.0, 1.0, 2.0], y=[1.0, 3.0, 2.0]).title("First scene")
    assert plot.width(640) is plot
    first = plot.svg_preserve_text(tmp_path / "first")
    first_bytes = (tmp_path / "first.svg").read_bytes()
    assert first.width == 640
    assert first.height > 0
    with pytest.raises(AttributeError):
        first.width = 1

    plot.title("Second scene")
    second = plot.svg_preserve_text(tmp_path / "second")
    assert "Second scene" in svg_text(tmp_path / "second.svg")
    assert "First scene" not in svg_text(tmp_path / "second.svg")
    assert second.width == first.width
    fresh_png = plot.png_preview(tmp_path / "second-preview")
    assert struct.unpack(
        ">II", (tmp_path / "second-preview.png").read_bytes()[16:24]
    ) == (fresh_png.width, fresh_png.height)

    replay = first.svg_preserve_text(tmp_path / "replay")
    assert replay is not first
    assert (tmp_path / "replay.svg").read_bytes() == first_bytes
    first.png_preview(tmp_path / "preview")
    png = (tmp_path / "preview.png").read_bytes()
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    assert struct.unpack(">II", png[16:24]) == (first.width, first.height)
    first.svg(tmp_path / "outlined")
    assert not list(ET.parse(tmp_path / "outlined.svg").getroot().iter(f"{SVG}text"))

    missing = tmp_path / "missing-parent" / "figure"
    with pytest.raises(OSError):
        plot.svg(missing)
    with pytest.raises(OSError):
        first.svg_preserve_text(missing)
    assert (tmp_path / "first.svg").read_bytes() == first_bytes
    first.copy().svg_preserve_text(tmp_path / "after-failure-replay")
    assert (tmp_path / "after-failure-replay.svg").read_bytes() == first_bytes
    plot.svg_preserve_text(tmp_path / "after-failure-render")
    assert "Second scene" in svg_text(tmp_path / "after-failure-render.svg")
