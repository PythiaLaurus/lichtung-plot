"""Native label values and their actual mutable-owner consumers."""

from pathlib import Path

import pytest

import lichtung_plot as lp


def test_labels_derive_complete_native_policy_without_a_shadow_alias_map() -> None:
    supplied = [("z", "First"), ("a", "a"), ("z", "Last")]
    original = lp.Labels(mode="raw", aliases=supplied)
    supplied.clear()
    assert original.mode == "raw"
    assert original.aliases == (("a", "a"), ("z", "Last"))
    humanized = original.copy_with(mode="humanized")
    assert humanized.aliases == original.aliases
    assert humanized.resolve("a") == "a"
    assert humanized.resolve("control_group") == "Control group"
    replaced = original.copy_with(aliases=[("new", "New")])
    assert replaced.aliases == (("new", "New"),)
    assert replaced.resolve("z") == "z"
    assert original.copy_with(aliases=None).aliases is None
    assert lp.Labels(aliases=[]).aliases is None
    with pytest.raises(AttributeError):
        original.mode = "humanized"  # type: ignore[misc]
    with pytest.raises(TypeError):
        original.copy_with(mode=None)  # type: ignore[arg-type]
    invalid = original.copy_with(aliases=[("bad", "invalid\talias")])
    assert invalid.aliases == (("bad", "invalid\talias"),)
    with pytest.raises(lp.DataError, match="unsupported control"):
        invalid.resolve("bad")


def test_labels_preserve_sequence_length_errors_instead_of_iterating_anyway() -> None:
    failure = LookupError("authored aliases length")

    class Aliases:
        def __len__(self) -> int:
            raise failure

        def __getitem__(self, index: int) -> tuple[str, str]:
            raise AssertionError("must not iterate after len failed")

    with pytest.raises(LookupError) as caught:
        lp.Labels(aliases=Aliases())  # type: ignore[arg-type]
    assert caught.value is failure


def test_plot_and_canvas_forward_labels_without_rewriting_captured_children(
    tmp_path: Path,
) -> None:
    plot = lp.scatter(x=[0.0, 1.0], y=[1.0, 2.0]).size(600, 400)
    assert plot.labels(lp.Labels(mode="raw", aliases=[("x", "Child X")])) is plot
    canvas = lp.Canvas.grid(1, 1).size(800, 500).panel(0, 0, plot)
    captured = canvas.copy()
    plot.labels(lp.Labels(mode="raw", aliases=[("x", "Later X")]))
    assert canvas.labels(lp.Labels(mode="raw", aliases=[("x", "Canvas X")])) is canvas
    for owner, name, expected in [
        (plot, "plot", "Later X"),
        (captured, "captured", "Child X"),
        (canvas, "canvas", "Canvas X"),
    ]:
        path = tmp_path / f"{name}.svg"
        owner.svg_preserve_text(path)
        assert f">{expected}</text>" in path.read_text()
    plot.labels(lp.Labels(aliases=[("x", "invalid\talias")]))
    with pytest.raises(lp.DataError, match="unsupported control"):
        plot.svg_preserve_text(tmp_path / "invalid.svg")
