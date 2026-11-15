"""Native scientific/policy snapshots; publication belongs to their consumers."""

import gc
import weakref

import pytest

import lichtung_plot as lp


def test_intervals_preserve_current_algorithm_and_use_native_scalar_validation():
    deviation = lp.StandardDeviation(scale=2)
    error = lp.StandardError(scale=0.5)
    percentile = lp.PercentileInterval(level=0.8)
    assert deviation.copy_with(scale=3).scale == 3 and deviation.scale == 2
    assert error.copy_with().scale == 0.5
    assert percentile.copy_with(level=0.9).level == 0.9 and percentile.level == 0.8

    default = lp.ConfidenceInterval()
    assert (default.bootstrap.resamples, default.bootstrap.seed) == (1000, 0)
    original = lp.ConfidenceInterval(
        level=0.9, bootstrap=lp.Bootstrap(resamples=37, seed=11),
    )
    # A property returns a complete independent policy, usable in the same copy.
    changed = original.copy_with(
        level=0.8, bootstrap=original.bootstrap.copy_with(seed=13),
    )
    assert (changed.level, changed.bootstrap.resamples, changed.bootstrap.seed) == (
        0.8, 37, 13,
    )
    assert (original.level, original.bootstrap.seed) == (0.9, 11)
    kept = original.copy_with(level=0.7)
    assert (kept.bootstrap.resamples, kept.bootstrap.seed) == (37, 11)
    student = original.copy_with(bootstrap=None)
    assert student.bootstrap is None
    assert student.copy_with(level=0.8).bootstrap is None
    assert lp.ConfidenceInterval(bootstrap=None).bootstrap is None

    with pytest.raises(lp.DataError, match="standard-deviation scale"):
        deviation.copy_with(scale=0)
    with pytest.raises(lp.DataError, match="standard-error scale"):
        error.copy_with(scale=float("inf"))
    with pytest.raises(lp.DataError, match="strictly between"):
        percentile.copy_with(level=1)
    with pytest.raises(TypeError):
        original.copy_with(level=None)
    with pytest.raises(TypeError):
        original.copy_with(bootstrap="student_t")
    assert (original.level, original.bootstrap.seed) == (0.9, 11)


def test_bootstrap_units_use_only_explicit_context_and_release_python_inputs():
    pd = pytest.importorskip("pandas")
    source = pd.DataFrame({"unit": ["first", "second", "first"]})
    source_ref = weakref.ref(source)
    original = lp.Bootstrap(
        resamples=41, seed=2**64 - 1, units=lp.col["unit"], data=source,
    )
    source["unit"] = ["later"] * 3
    # No new units means no lookup, even if this call's context is unusable.
    kept = original.copy_with(data=object())
    cleared = kept.copy_with(units=None)
    edited = kept.copy_with(units=None, resamples=43, seed=0)
    assert (cleared.resamples, cleared.seed) == (41, 2**64 - 1)
    assert (edited.resamples, edited.seed) == (43, 0)
    with pytest.raises(TypeError):
        original.copy_with(units=lp.col["unit"])
    with pytest.raises(KeyError):
        original.copy_with(resamples=47, units=lp.col["missing"], data=source)
    with pytest.raises(lp.DataError, match="bootstrap resamples"):
        original.copy_with(resamples=0)
    with pytest.raises(TypeError):
        original.copy_with(seed=None)
    assert (original.resamples, original.seed) == (41, 2**64 - 1)

    class Units(list):
        pass

    units = Units(["first", "first", "second"])
    units_ref = weakref.ref(units)
    replacement = kept.copy_with(units=units)
    interval = lp.ConfidenceInterval(bootstrap=replacement)
    del source, units, replacement
    gc.collect()
    assert source_ref() is None and units_ref() is None
    assert (interval.bootstrap.resamples, interval.bootstrap.seed) == (
        41, 2**64 - 1,
    )
    # Actual resampling/alignment of the opaque units is covered by Bar consumers.


def test_categorical_maps_keep_order_and_raw_identity_without_domain_preflight():
    dark = lp.OpaqueColor(25, 74, 117)
    warm = lp.OpaqueColor(186, 91, 35)
    supplied = [("A B", warm), ("A  B", dark)]
    mapping = lp.CategoricalColorMap(supplied)
    supplied.clear()
    assert mapping.entries == (("A B", warm), ("A  B", dark))
    assert mapping.color("A  B") == dark and mapping.color("A B") == warm
    assert mapping.color(" A B ") is None
    # Missing active-domain coverage is not a standalone factory check.
    assert mapping.color("not covered") is None
    selected = lp.CategoricalColorScale.from_map(mapping)
    changed = selected.copy_with(map=mapping.copy_with(entries=[("A B", dark)]))
    assert changed.map.entries == (("A B", dark),)
    assert selected.map == mapping and selected.kind == "map"
    assert selected.palette is None and selected.copy_with() == selected
    palette = lp.QualitativePalette.fixed([warm])
    palette_scale = lp.CategoricalColorScale.from_palette(palette)
    assert palette_scale.copy_with(
        palette=palette.copy_with(colors=[dark]),
    ).palette.colors == (dark,)

    with pytest.raises(lp.DataError, match="duplicated"):
        mapping.copy_with(entries=[("A B", dark), ("A B", warm)])
    with pytest.raises(TypeError):
        lp.CategoricalColorMap({"A B": dark})
    with pytest.raises(TypeError):
        lp.CategoricalColorMap([("A B", lp.Color(25, 74, 117))])
    with pytest.raises(TypeError, match="branch"):
        selected.copy_with(palette=None)
    with pytest.raises(TypeError):
        selected.copy_with(map=None)
    with pytest.raises(TypeError):
        lp.CategoricalColorScale()
    assert selected.map.entries == mapping.entries


def test_continuous_copy_validates_final_center_domain_and_preserves_auto_none():
    scales = lp.Theme.publication().color_scales
    domain = lp.ColorDomain(-1, 1)
    original = lp.ContinuousColorScale.diverging(
        0, gradient=scales.diverging, domain=domain,
    )
    moved_domain = domain.copy_with(minimum=2, maximum=4)
    moved = original.copy_with(center=3, domain=moved_domain)
    assert moved.center == 3 and moved.domain == moved_domain
    assert moved.diverging_gradient == original.diverging_gradient
    assert original.center == 0 and original.domain == domain
    automatic = moved.copy_with(domain=None, diverging_gradient=None)
    assert automatic == lp.ContinuousColorScale.diverging(3)
    assert automatic.domain is None and automatic.diverging_gradient is None
    constant = lp.ColorDomain(2, 2)
    sequential = lp.ContinuousColorScale.sequential(
        gradient=scales.sequential, domain=constant,
    )
    assert sequential.copy_with(
        domain=None, sequential_gradient=None,
    ) == lp.ContinuousColorScale.sequential()

    with pytest.raises(lp.DataError, match="contain center"):
        original.copy_with(center=3)
    with pytest.raises(lp.DataError, match="finite"):
        domain.copy_with(maximum=float("inf"))
    with pytest.raises(TypeError, match="branch"):
        original.copy_with(sequential_gradient=None)
    with pytest.raises(TypeError, match="branch"):
        sequential.copy_with(diverging_gradient=None)
    with pytest.raises(TypeError):
        original.copy_with(center=None)
    assert original.domain == domain and original.center == 0


def test_explicit_mark_style_is_css_pixels_and_interaction_selectors_are_exact():
    explicit = lp.SignificanceMarkStyle()
    assert (explicit.font_size, explicit.weight, explicit.gap) == (16, "normal", 8)
    changed = explicit.copy_with(font_size=18, weight="semibold", gap=-0.0)
    assert (changed.font_size, changed.weight, changed.gap) == (18, "semibold", 0)
    assert (explicit.font_size, explicit.weight, explicit.gap) == (16, "normal", 8)
    with pytest.raises(TypeError):
        explicit.copy_with(gap=lp.RelativeSpacing(1))
    with pytest.raises(lp.DataError, match="finite nonnegative gap"):
        explicit.copy_with(gap=float("nan"))
    with pytest.raises(AttributeError):
        explicit.gap = 12

    raw = lp.InteractionStyle(
        "A  B", point_shape="triangle_up", line_pattern="dashed",
    )
    renamed = raw.copy_with(group="A B")
    assert (renamed.group, renamed.point_shape, renamed.line_pattern) == (
        "A B", "triangle_up", "dashed",
    )
    assert (raw.group, raw.point_shape, raw.line_pattern) == (
        "A  B", "triangle_up", "dashed",
    )
    with pytest.raises(lp.DataError, match="already trimmed"):
        raw.copy_with(group=" A  B ")
    with pytest.raises(ValueError, match="line_pattern"):
        raw.copy_with(line_pattern="long_dash")


def test_text_positions_keep_native_coordinate_domains_without_fake_field_copies():
    raw = lp.DataCoordinate.category_series("A  B", "raw series")
    data = lp.TextCoordinate.data(raw)
    panel = lp.TextCoordinate.data_area_fraction(1.25)
    figure = lp.TextCoordinate.figure_fraction(1.25)
    mixed = lp.TextPosition.coordinates(x=data, y=figure)
    assert mixed != lp.TextPosition.coordinates(
        x=data, y=lp.TextCoordinate.figure_fraction(1),
    )
    assert mixed != lp.TextPosition.coordinates(x=data, y=panel)
    assert lp.TextPosition.coordinates(
        x=lp.TextCoordinate.data_area_fraction(-0.25), y=data,
    ) != lp.TextPosition.coordinates(
        x=lp.TextCoordinate.data_area_fraction(0), y=data,
    )
    assert lp.TextPosition.corner("top_left") != lp.TextPosition.coordinates(
        x=lp.TextCoordinate.data_area_fraction(0),
        y=lp.TextCoordinate.data_area_fraction(1),
    )
    with pytest.raises(lp.DataError):
        lp.TextCoordinate.figure_fraction(float("nan"))
    with pytest.raises(TypeError):
        lp.TextPosition.coordinates(x=None, y=figure)
    with pytest.raises(ValueError, match="corner"):
        lp.TextPosition.corner("center")
