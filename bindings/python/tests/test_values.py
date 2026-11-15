"""Immutable native values and their shared mutable-owner Text entry path."""

from xml.etree import ElementTree

import pytest

from lichtung_plot import (
    Color,
    DataError,
    FontFamily,
    FontStack,
    PngOptions,
    PngSizing,
    RelativeSpacing,
    RenderError,
    RootTextRoles,
    SurfaceTheme,
    Text,
    TextBounds,
    TextLayout,
    TextOffset,
    TextTheme,
    Theme,
    TypographyTheme,
    scatter,
)


def test_nested_theme_derivation_replaces_complete_children_without_editing_aliases():
    theme = Theme.nature()
    alias = theme
    typography = theme.typography
    root = typography.root
    title = root.title
    paint = Color.hex("#194A75")
    large = title.copy_with(font_size=24, color=paint)
    replaced_root = root.copy_with(title=large)
    replaced_typography = typography.copy_with(root=replaced_root)
    customized = theme.copy_with(typography=replaced_typography)

    assert alias == theme
    assert theme.typography == typography
    assert typography.root == root
    assert root.title == title
    assert title != large
    assert large.font_size == 24
    assert large.color == paint
    assert customized.typography.root.title == large
    assert customized.typography.root.subtitle == root.subtitle
    assert customized.typography.annotations == typography.annotations
    assert customized.surfaces == theme.surfaces
    assert customized.copy_with() == customized

    # Complete public constructors produce the same native groups as derivation.
    rebuilt_root = RootTextRoles(
        title=large, subtitle=root.subtitle, caption=root.caption
    )
    rebuilt = TypographyTheme(
        root=rebuilt_root,
        axes=typography.axes,
        guides=typography.guides,
        composition=typography.composition,
        annotations=typography.annotations,
    )
    assert rebuilt == replaced_typography
    assert Theme(typography=rebuilt).typography == rebuilt

    with pytest.raises(AttributeError):
        theme.typography.root.title.font_size = 99
    with pytest.raises(AttributeError):
        customized.typography = typography
    with pytest.raises(AttributeError):
        title.unrelated = "no instance dictionary"
    with pytest.raises(TypeError):
        title.copy_with(24)
    with pytest.raises(TypeError):
        theme.copy_with(typography=None)


def test_complete_text_appearance_uses_native_validation_without_font_discovery():
    title = Theme.publication().typography.root.title
    # An unavailable family is a valid configuration identity, not an eager lookup.
    stack = FontStack([FontFamily("Lichtung binding test absent family")])
    style = TextTheme(
        stack=stack,
        color=Color(10, 20, 30, alpha=127),
        font_size=23,
        weight="semibold",
        line_height=1.5,
    )
    assert style == title.copy_with(
        stack=stack,
        color=style.color,
        font_size=23,
        weight="semibold",
        line_height=1.5,
    )
    saved = style.copy_with()
    with pytest.raises(DataError):
        style.copy_with(font_size=-1)
    with pytest.raises(DataError):
        style.copy_with(line_height=float("nan"))
    with pytest.raises(TypeError):
        style.copy_with(color=None)
    with pytest.raises(TypeError):
        style.copy_with(font_size=24, stack={"family": "not a native value"})
    assert style == saved

    contrast = Theme.publication().typography.annotations.contrast
    light = contrast.light
    translucent = light.copy_with(color=light.color.copy_with(alpha=128))
    with pytest.raises(DataError):
        contrast.copy_with(light=translucent)
    assert contrast.light == light

    surfaces = SurfaceTheme(
        canvas_root=Color(1, 2, 3, alpha=0), plot_panel=Color(4, 5, 6)
    )
    derived = surfaces.copy_with(plot_panel=Color(7, 8, 9))
    assert derived.canvas_root == surfaces.canvas_root
    assert derived.plot_panel != surfaces.plot_panel
    assert Theme(surfaces=surfaces).surfaces == surfaces


def test_font_preferences_snapshot_the_sequence_and_return_immutable_children():
    literal = FontFamily("sans-serif")
    generic = FontFamily(generic_family="sans_serif")
    supplied = [literal, generic, literal]
    stack = FontStack(supplied)
    supplied.clear()
    assert stack.families == (literal, generic, literal)
    assert literal != generic
    assert literal.copy_with(name=None, generic_family="serif") == FontFamily(
        generic_family="serif"
    )
    with pytest.raises(TypeError):
        stack.families[0] = generic
    with pytest.raises(AttributeError):
        stack.families[0].name = "changed"
    with pytest.raises(DataError):
        FontStack([])
    with pytest.raises(DataError):
        literal.copy_with(name=" not already trimmed ")
    with pytest.raises(TypeError):
        literal.copy_with(name=None)
    assert literal.name == "sans-serif"


def test_exact_relative_spacing_identity_survives_unchanged_derivation():
    exact = RelativeSpacing.ratio(3, 5)
    approximate = RelativeSpacing(0.6)
    assert exact != approximate
    assert exact.copy_with() == exact
    assert exact.copy_with(value=0.6) == approximate
    with pytest.raises(RenderError):
        exact.copy_with(value=-1)
    with pytest.raises(TypeError):
        exact.copy_with(value=None)
    assert exact == RelativeSpacing.ratio(6, 10)


def test_text_constructor_and_copy_distinguish_omission_none_and_default_layout():
    style = Theme.science().typography.root.title
    layout = TextLayout(alignment="end", direction="right_to_left")
    plain = Text("Verbatim\r\nsource ")
    assert plain.style is None
    assert plain.layout is None
    assert Text(plain.source, style=None, layout=None) == plain
    authored = Text(plain.source, style=style, layout=layout)
    assert authored.copy_with() == authored
    assert authored.copy_with(source="Changed") == Text(
        "Changed", style=style, layout=layout
    )
    assert authored.copy_with(style=None) == Text(plain.source, layout=layout)
    assert authored.copy_with(layout=None) == Text(plain.source, style=style)
    assert authored.copy_with(style=None, layout=None) == plain
    reset = authored.copy_with(layout=TextLayout())
    assert reset.layout == TextLayout()
    assert reset != authored.copy_with(layout=None)
    assert authored.style == style
    assert authored.layout == layout
    with pytest.raises(TypeError):
        authored.copy_with(source=None)
    with pytest.raises(TypeError):
        authored.copy_with(style=None, layout=object())
    with pytest.raises(AttributeError):
        authored.style = None
    assert authored == Text(plain.source, style=style, layout=layout)


def test_layout_copy_clears_one_native_instruction_without_losing_siblings():
    offset = TextOffset(3, -2, frame="text_local")
    bounds = TextBounds(max_width=170, max_height=60, max_lines=3)
    layout = TextLayout(
        alignment="end",
        fit_priority="preferred_size_first",
        direction="right_to_left",
        rotation_mode="align_after_rotation",
        offset=offset,
        bounds=bounds,
        wrapping="no_soft_wrap",
        overlap="allow",
    )
    cleared = layout.copy_with(alignment=None)
    expected = TextLayout(
        fit_priority="preferred_size_first",
        direction="right_to_left",
        rotation_mode="align_after_rotation",
        offset=offset,
        bounds=bounds,
        wrapping="no_soft_wrap",
        overlap="allow",
    )
    assert cleared == expected
    assert layout.alignment == "end"
    assert cleared.alignment is None
    assert layout.copy_with() == layout
    assert cleared.copy_with(alignment="end") == layout
    edited_offset = layout.offset.copy_with(dx=8)
    assert edited_offset == TextOffset(8, -2, frame="text_local")
    assert layout.offset == offset
    edited_bounds = layout.bounds.copy_with(max_height=None)
    assert edited_bounds == TextBounds(max_width=170, max_lines=3)
    assert layout.bounds == bounds
    assert layout.copy_with(bounds=None).offset == offset
    with pytest.raises(AttributeError):
        layout.offset.dx = 90
    with pytest.raises(AttributeError):
        cleared.direction = "auto"


def test_layout_type_and_native_constraint_failures_leave_original_values_intact():
    offset = TextOffset(1, 2)
    bounds = TextBounds(max_width=40, max_lines=0)
    layout = TextLayout(offset=offset, bounds=bounds, alignment="start")
    with pytest.raises(DataError):
        offset.copy_with(dy=float("inf"))
    with pytest.raises(DataError):
        bounds.copy_with(max_width=-1)
    with pytest.raises(TypeError):
        offset.copy_with(dx=None)
    with pytest.raises(TypeError):
        layout.copy_with(offset=None, bounds="not a native bounds value")
    with pytest.raises(ValueError):
        layout.copy_with(alignment="automatic")
    assert offset == TextOffset(1, 2)
    assert bounds == TextBounds(max_width=40, max_lines=0)
    assert layout == TextLayout(offset=offset, bounds=bounds, alignment="start")
    assert TextLayout() == TextLayout(alignment=None, bounds=None)


def test_png_options_keep_native_closed_sizing_and_deferred_validation():
    original = PngOptions(sizing=PngSizing("width", 320), compression="fast")
    derived = original.copy_with(compression="high")
    assert derived.sizing == original.sizing
    assert original.compression == "fast"
    assert derived.sizing.value == 320
    assert PngSizing("source").value is None
    assert PngSizing("scale", 2).copy_with(value=3) == PngSizing("scale", 3)
    # Native options validate actual sizing at publication, not value construction.
    invalid_at_use = PngOptions(sizing=PngSizing("dpi", -1))
    assert invalid_at_use.sizing.value == -1
    with pytest.raises(TypeError):
        original.copy_with(sizing=None)
    with pytest.raises(AttributeError):
        original.sizing.value = 99
    assert original.sizing == PngSizing("width", 320)


def _scatter():
    return scatter(x=[0.0, 1.0, 2.0], y=[1.0, 3.0, 2.0])


def _title_attributes(plot, destination, source):
    plot.svg_preserve_text(destination)
    root = ElementTree.parse(destination).getroot()
    matches = [
        element
        for element in root.iter("{http://www.w3.org/2000/svg}text")
        if "".join(element.itertext()) == source
    ]
    assert len(matches) == 1
    return matches[0].attrib


def test_owner_text_keywords_equal_authored_text_and_fail_before_editing(tmp_path):
    style = Theme.publication().typography.root.title.copy_with(
        font_size=25, color=Color.hex("#194A75")
    )
    layout = TextLayout(alignment="end")
    incoming = Text("Shared entry", style=style, layout=layout)
    direct = _scatter().title("Shared entry", style=style, layout=layout)
    authored = _scatter().title(incoming)
    overridden = _scatter().title(
        incoming.copy_with(
            style=style.copy_with(font_size=29),
            layout=TextLayout(alignment="start"),
        ),
        style=style,
        layout=layout,
    )
    direct.svg_preserve_text(tmp_path / "direct.svg")
    authored.svg_preserve_text(tmp_path / "authored.svg")
    overridden.svg_preserve_text(tmp_path / "overridden.svg")
    assert (tmp_path / "direct.svg").read_bytes() == (
        tmp_path / "authored.svg"
    ).read_bytes()
    assert (tmp_path / "direct.svg").read_bytes() == (
        tmp_path / "overridden.svg"
    ).read_bytes()
    with pytest.raises(TypeError):
        direct.title("Must not commit", style=None, layout=object())
    direct.svg_preserve_text(tmp_path / "after-failure.svg")
    assert (tmp_path / "direct.svg").read_bytes() == (
        tmp_path / "after-failure.svg"
    ).read_bytes()
    assert incoming == Text("Shared entry", style=style, layout=layout)


def test_owner_none_clearing_restores_theme_inheritance_independently(tmp_path):
    theme = Theme.nature()
    root = theme.typography.root
    inherited_style = root.title.copy_with(
        font_size=26, color=Color.hex("#194A75")
    )
    custom = theme.copy_with(
        typography=theme.typography.copy_with(
            root=root.copy_with(title=inherited_style)
        )
    )
    local_style = inherited_style.copy_with(
        font_size=31, color=Color.hex("#711F3B")
    )
    incoming = Text(
        "Retained title", style=local_style, layout=TextLayout(alignment="end")
    )
    original = _scatter().theme(theme).title(incoming)
    plain = incoming.copy_with(style=None, layout=None)
    preserved = original.copy().title(plain).theme(custom)
    kept = _title_attributes(
        preserved, tmp_path / "preserved.svg", incoming.source
    )
    assert float(kept["font-size"]) == 31
    assert kept["fill"] == "#711F3B"
    _scatter().theme(custom).title(incoming).svg_preserve_text(
        tmp_path / "expected-preserved.svg"
    )
    assert (tmp_path / "preserved.svg").read_bytes() == (
        tmp_path / "expected-preserved.svg"
    ).read_bytes()

    cleared_style = original.copy().title(incoming, style=None).theme(custom)
    appearance = _title_attributes(
        cleared_style, tmp_path / "style-cleared.svg", incoming.source
    )
    assert float(appearance["font-size"]) == 26
    assert appearance["fill"] == "#194A75"
    _scatter().theme(custom).title(
        Text(incoming.source, layout=incoming.layout)
    ).svg_preserve_text(tmp_path / "expected-style-cleared.svg")
    assert (tmp_path / "style-cleared.svg").read_bytes() == (
        tmp_path / "expected-style-cleared.svg"
    ).read_bytes()

    cleared_layout = original.copy().title(plain, layout=None)
    layout = _title_attributes(
        cleared_layout, tmp_path / "layout-cleared.svg", incoming.source
    )
    assert float(layout["font-size"]) == 31
    assert layout["fill"] == "#711F3B"
    _scatter().theme(theme).title(
        Text(incoming.source, style=local_style)
    ).svg_preserve_text(tmp_path / "expected-layout-cleared.svg")
    assert (tmp_path / "layout-cleared.svg").read_bytes() == (
        tmp_path / "expected-layout-cleared.svg"
    ).read_bytes()

    cleared = original.copy().title(incoming, style=None, layout=None).theme(custom)
    cleared.svg_preserve_text(tmp_path / "both-cleared.svg")
    _scatter().theme(custom).title(incoming.source).svg_preserve_text(
        tmp_path / "inherited.svg"
    )
    assert (tmp_path / "both-cleared.svg").read_bytes() == (
        tmp_path / "inherited.svg"
    ).read_bytes()
    assert original is not cleared
    assert incoming.style == local_style
    assert incoming.layout == TextLayout(alignment="end")
