"""Exact owners, four additional physical formatter slots and Canvas composition."""

import gc
import weakref

import pytest

import lichtung_plot as lp


def _holder_ids(value):
    return [
        id(root) for root in gc.get_referents(value)
        if type(root).__name__ == "_CallbackHolder"
    ]


def test_heatmap_values_and_continuous_color_keep_distinct_same_holder_edges(tmp_path):
    reached = []

    def labels(view):
        if view.purpose == "actual_set":
            reached.append((view.role, view.owner.canvas_cell))
        return view.format_builtin()

    callback_ref = weakref.ref(labels)
    format = lp.NumericFormat.set_function(labels)
    holder = _holder_ids(format)[0]
    plot = (
        lp.heatmap(grid=[[1.0, 2.0], [3.0, 4.0]])
        .annotations(format)
        .color_label_format(format)
        .continuous_color_scale(
            lp.ContinuousColorScale.sequential(domain=lp.ColorDomain(0.0, 4.0))
        )
        .color_label("Color")
        .size(1000, 700)
    )
    assert _holder_ids(plot._owner) == [holder, holder]
    copied = plot.copy()
    assert type(copied) is lp.Heatmap and _holder_ids(copied._owner) == [holder, holder]
    canvas = (
        lp.Canvas.grid(1, 2).size(1600, 700)
        .panel(0, 0, copied).panel(0, 1, copied)
    )
    assert _holder_ids(canvas._owner) == [holder] * 4
    fresh = lp.NumericFormat.set_function(labels)
    fresh_holder = _holder_ids(fresh)[0]
    plot.color_label_format(fresh)
    assert fresh_holder != holder and _holder_ids(plot._owner) == [fresh_holder, holder]
    with pytest.raises(TypeError):
        copied.annotations(None)
    with pytest.raises(lp.DataError):
        canvas.panel(0, 0, copied)
    assert _holder_ids(copied._owner) == [holder, holder]
    assert _holder_ids(canvas._owner) == [holder] * 4
    assert reached == []

    # Hiding values retires only that slot. Continuous color still really runs.
    plot.annotations("hidden")
    assert _holder_ids(plot._owner) == [fresh_holder]
    plot.svg_preserve_text(tmp_path / "hidden-values-keeps-color")
    assert reached and {role for role, _ in reached} == {"continuous_color"}
    reached.clear()
    del copied, plot, labels, format, fresh
    gc.collect()
    assert callback_ref() is not None
    output = canvas.svg_preserve_text(tmp_path / "captured-two-slot-occurrences")
    assert set(reached) == {
        ("continuous_color", (0, 0)), ("heatmap_values", (0, 0)),
        ("continuous_color", (0, 1)), ("heatmap_values", (0, 1)),
    }
    canvas.panel(0, 0, lp.heatmap(grid=[[1.0]]), overwrite=True)
    assert _holder_ids(canvas._owner) == [holder, holder]
    canvas.panel(0, 1, lp.heatmap(grid=[[2.0]]), overwrite=True)
    gc.collect()
    assert (
        _holder_ids(canvas._owner) == [] and callback_ref() is None
        and _holder_ids(output) == []
    )
