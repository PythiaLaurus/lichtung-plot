"""Complete native Theme graphs: sibling validation, ownership and real consumers."""

from xml.etree import ElementTree

import pytest

import lichtung_plot as lp


def test_nested_leaves_rebuild_complete_theme_and_reach_plot_and_canvas(tmp_path):
    original = lp.Theme.nature()
    saved = original.copy_with()
    alias = original
    old_points = original.marks.points
    primary = old_points.primary.copy_with(
        shape="triangle_up", fill_alpha=173, outline=None
    )
    marks = original.marks.copy_with(
        points=old_points.copy_with(primary=primary),
    )
    paint = lp.Color.hex("#194A75")
    stroke = lp.StrokeTheme(color=paint, width=1.5, pattern="short_dash")
    axes = original.axes.copy_with(grid=stroke)
    legend = original.legend.copy_with(frame=None)
    composition = original.composition.copy_with(
        root_padding=original.composition.root_padding.copy_with(
            top=lp.RelativeSpacing.ratio(3, 2)
        ),
        root_subtitle_alignment=None,
    )
    annotations = original.annotations.copy_with(
        point_label_gap=lp.RelativeSpacing.ratio(1, 3)
    )
    scales = original.color_scales.copy_with(
        qualitative=lp.QualitativePalette.fixed(
            [lp.OpaqueColor(25, 74, 117), lp.OpaqueColor(186, 91, 35)]
        )
    )
    title = original.typography.root.title.copy_with(color=paint)
    typography = original.typography.copy_with(
        root=original.typography.root.copy_with(title=title)
    )
    complete = lp.Theme(
        typography=typography,
        surfaces=original.surfaces,
        axes=axes,
        legend=legend,
        composition=composition,
        marks=marks,
        annotations=annotations,
        color_scales=scales,
    )
    assert complete == original.copy_with(
        typography=typography,
        axes=axes,
        legend=legend,
        composition=composition,
        marks=marks,
        annotations=annotations,
        color_scales=scales,
    )
    assert alias == saved
    assert original.marks.points == old_points
    assert complete.marks.points.primary == primary
    assert complete.marks.fills == original.marks.fills
    assert complete.color_scales.diverging == original.color_scales.diverging
    assert lp.Theme(axes=axes) == lp.Theme.publication().copy_with(axes=axes)

    with pytest.raises(AttributeError):
        primary.shape = "circle"
    with pytest.raises(AttributeError):
        primary.unrelated = "no instance dictionary"
    with pytest.raises(TypeError):
        original.copy_with(axes=axes, marks=object())
    with pytest.raises(TypeError):
        complete.copy_with(color_scales=None)
    assert alias == saved

    plot = (
        lp.scatter(x=[0, 1, 2], y=[2, 1, 3], hue=["First", "Second", "First"])
        .theme(complete)
        .title("Plot appearance")
    )
    canvas = (
        lp.Canvas.grid(1, 1)
        .theme(complete)
        .size(800, 600)
        .title("Canvas appearance")
        .panel(0, 0, plot)
    )
    output = canvas.svg_preserve_text(tmp_path / "complete-theme")
    assert (output.width, output.height) == (800, 600)
    root = ElementTree.parse(tmp_path / "complete-theme.svg").getroot()
    titles = {
        "".join(node.itertext()): node.attrib
        for node in root.iter("{http://www.w3.org/2000/svg}text")
        if "".join(node.itertext()) in {"Plot appearance", "Canvas appearance"}
    }
    assert set(titles) == {"Plot appearance", "Canvas appearance"}
    assert all(attributes["fill"] == "#194A75" for attributes in titles.values())
    colors = {"#194A75", "#BA5B23"}
    legend_nodes = {
        node
        for group in root.iter("{http://www.w3.org/2000/svg}g")
        if group.get("id", "").endswith("legend")
        for node in group.iter()
    }
    triangles = [
        node
        for node in root.iter("{http://www.w3.org/2000/svg}polygon")
        if node not in legend_nodes and node.attrib.get("fill") in colors
    ]
    assert len(triangles) == 3
    assert {node.attrib["fill"] for node in triangles} == colors
    assert all(len(node.attrib["points"].split()) == 3 for node in triangles)


def test_sibling_edits_validate_the_complete_final_value_not_intermediate_states():
    sides = lp.FrameSides(top=True, right=False, bottom=False, left=False)
    switched = sides.copy_with(top=False, left=True)
    assert switched == lp.FrameSides(top=False, right=False, bottom=False, left=True)
    with pytest.raises(lp.DataError):
        sides.copy_with(top=False)
    with pytest.raises(TypeError):
        lp.FrameSides(True, False, False, False)
    assert sides.top and not sides.left

    lengths = lp.GradientLengths(
        minimum=lp.RelativeSpacing.ratio(1, 3),
        preferred=lp.RelativeSpacing.ratio(2, 3),
    )
    moved = lengths.copy_with(
        minimum=lp.RelativeSpacing(3), preferred=lp.RelativeSpacing(4)
    )
    assert moved == lp.GradientLengths(
        minimum=lp.RelativeSpacing(3), preferred=lp.RelativeSpacing(4)
    )
    with pytest.raises(lp.DataError):
        lengths.copy_with(minimum=lp.RelativeSpacing(3))
    with pytest.raises(lp.DataError):
        lengths.copy_with(minimum=lp.RelativeSpacing(0))
    # The binary64 approximation is below exact 1/3; do not compare via .value.
    with pytest.raises(lp.DataError):
        lengths.copy_with(preferred=lp.RelativeSpacing(1 / 3))
    assert lengths.preferred == lp.RelativeSpacing.ratio(2, 3)


def test_native_sequences_keep_order_snapshot_and_factory_validation_timing(tmp_path):
    dark = lp.OpaqueColor(25, 74, 117)
    neutral = lp.OpaqueColor(240, 240, 240)
    warm = lp.OpaqueColor(186, 91, 35)
    supplied = [dark, warm, dark]
    palette = lp.QualitativePalette.fixed(supplied)
    accents = lp.AccentPalette(supplied)
    supplied.clear()
    assert palette.colors == accents.seeds == (dark, warm, dark)
    replacement = [warm, dark]
    extended = palette.copy_with(colors=replacement, capacity="extendable")
    replacement.clear()
    assert extended.colors == (warm, dark)
    assert extended.capacity == "extendable" and palette.capacity == "fixed"
    assert palette.colors == (dark, warm, dark)

    supplied_stops = [
        lp.ColorStop(-0.0, dark),
        lp.ColorStop(0.5, neutral),
        lp.ColorStop(1.0, warm),
    ]
    sequential = lp.SequentialGradient(supplied_stops)
    diverging = lp.DivergingGradient(supplied_stops)
    supplied_stops.reverse()
    assert tuple(stop.offset for stop in sequential.stops) == (0.0, 0.5, 1.0)
    assert sequential.stops == diverging.stops
    assert diverging.neutral == neutral
    changed = diverging.copy_with(
        stops=[
            diverging.stops[0],
            diverging.stops[1].copy_with(color=warm),
            diverging.stops[2],
        ]
    )
    assert changed.neutral == warm and diverging.neutral == neutral
    with pytest.raises(lp.DataError):
        lp.QualitativePalette.fixed([])
    with pytest.raises(TypeError):
        lp.AccentPalette([lp.Color(25, 74, 117)])
    with pytest.raises(lp.DataError):
        lp.ColorStop(float("nan"), dark)
    with pytest.raises(lp.DataError):
        lp.SequentialGradient(supplied_stops)
    with pytest.raises(lp.DataError):
        lp.SequentialGradient([lp.ColorStop(0.25, dark), lp.ColorStop(1, warm)])
    with pytest.raises(lp.DataError):
        lp.DivergingGradient(
            [lp.ColorStop(0, dark), lp.ColorStop(0.25, neutral), lp.ColorStop(1, warm)]
        )
    with pytest.raises(TypeError):
        diverging.copy_with(neutral=warm)
    with pytest.raises(ValueError):
        palette.copy_with(capacity="cycle")

    # Fixed capacity is domain-dependent: complete values may be constructed,
    # but two active raw hue levels cannot consume a one-color fixed supply.
    theme = lp.Theme(
        color_scales=lp.Theme().color_scales.copy_with(
            qualitative=lp.QualitativePalette.fixed([dark])
        )
    )
    plot = lp.scatter(x=[0, 1], y=[1, 0], hue=["First", "Second"]).theme(theme)
    with pytest.raises(lp.DataError):
        plot.svg(tmp_path / "insufficient-palette")
    assert not (tmp_path / "insufficient-palette.svg").exists()


def test_nested_native_boundaries_and_structural_optional_chrome():
    original = lp.Theme.publication()
    transparent = lp.StrokeTheme(
        color=lp.Color(25, 74, 117, alpha=0), width=1, pattern="long_dash"
    )
    present = original.axes.copy_with(grid=transparent)
    absent = present.copy_with(grid=None)
    assert present.grid == transparent and absent.grid is None
    assert absent.spacing == original.axes.spacing
    assert lp.AxisTheme(spacing=original.axes.spacing).frame is None
    composition = original.composition.copy_with(
        root_title_alignment="end", root_subtitle_alignment="start"
    )
    following = composition.copy_with(root_subtitle_alignment=None)
    assert following.root_subtitle_alignment is None
    assert following.root_title_alignment == "end"
    assert composition.root_subtitle_alignment == "start"

    with pytest.raises(lp.DataError):
        transparent.copy_with(width=0)
    with pytest.raises(lp.DataError):
        lp.PointOutlineTheme(
            color=transparent.color, width_ratio=lp.RelativeSpacing(0)
        )
    with pytest.raises(lp.DataError):
        original.marks.points.radii.copy_with(scatter=0)
    with pytest.raises(lp.DataError):
        original.marks.cells.copy_with(
            missing_cross_inset=lp.RelativeSpacing.ratio(1, 2)
        )
    categories = original.marks.extents.categories
    assert categories.copy_with(
        uncertainty_cap_ratio=lp.RelativeSpacing.ratio(1, 2)
    ).uncertainty_cap_ratio == lp.RelativeSpacing.ratio(1, 2)
    with pytest.raises(lp.DataError):
        categories.copy_with(uncertainty_cap_ratio=lp.RelativeSpacing.ratio(51, 100))
    with pytest.raises(TypeError):
        original.marks.cells.copy_with(cell_outline=None, missing_cross=object())
    with pytest.raises(TypeError):
        original.annotations.copy_with(planning_backdrop=lp.Color(0, 0, 0))
    assert original == lp.Theme.publication()


def test_closed_outline_branches_are_not_nullable_optional_fields():
    stroke = lp.OutlineTheme(color=lp.Color(25, 74, 117), width=1)
    fixed = lp.DataOutlineTheme.fixed(stroke)
    mapped = lp.DataOutlineTheme.mapped(1.5, 128)
    edited_fixed = fixed.copy_with(fixed_stroke=stroke.copy_with(width=2))
    edited_mapped = mapped.copy_with(mapped_width=2, mapped_alpha=0)
    assert edited_fixed.fixed_stroke.width == 2
    assert edited_fixed.kind == "fixed" and edited_fixed.mapped_width is None
    assert edited_mapped.kind == "mapped" and edited_mapped.fixed_stroke is None
    assert edited_mapped.mapped_width == 2 and edited_mapped.mapped_alpha == 0
    assert fixed.fixed_stroke == stroke and mapped.mapped_alpha == 128
    assert fixed.copy_with() == fixed and mapped.copy_with() == mapped
    with pytest.raises(ValueError):
        fixed.copy_with(mapped_width=None)
    with pytest.raises(ValueError):
        mapped.copy_with(fixed_stroke=None)
    with pytest.raises(TypeError):
        fixed.copy_with(fixed_stroke=None)
    with pytest.raises(TypeError):
        mapped.copy_with(mapped_width=None)
    with pytest.raises(TypeError):
        mapped.copy_with(mapped_alpha=None)
    with pytest.raises(TypeError):
        lp.DataOutlineTheme()
    with pytest.raises(ValueError):
        lp.StrokeTheme(color=stroke.color, width=1, pattern="dashed")
    with pytest.raises(ValueError):
        lp.PointTheme(fill_alpha=128, shape="square")
    fill = lp.DataFillTheme(fill_alpha=0, outline=mapped)
    assert fill.copy_with(outline=None) == lp.DataFillTheme(fill_alpha=0)
    assert fill.outline == mapped
