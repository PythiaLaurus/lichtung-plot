"""Complete authored text programs through immutable values and real title slots."""

from xml.etree import ElementTree

import pytest

from lichtung_plot import (
    Color,
    DataCoordinate,
    DataError,
    DataPosition,
    Text,
    TextAnchor,
    TextBounds,
    TextEllipsis,
    TextFit,
    TextLayout,
    TextOffset,
    TextOrientation,
    TextOverflow,
    TextRotation,
    TextVisibility,
    Theme,
    scatter,
)


SVG_TEXT = "{http://www.w3.org/2000/svg}text"


def test_instruction_clear_preserves_complete_layout_and_original_aliases():
    fields = {
        "alignment": "end",
        "fit_priority": "preferred_size_first",
        "direction": "right_to_left",
        "rotation_mode": "align_after_rotation",
        "offset": TextOffset(3, -2, frame="text_local"),
        "bounds": TextBounds(max_width=180, max_height=90, max_lines=3),
        "wrapping": "no_soft_wrap",
        "overlap": "allow",
        "fit": TextFit.range(8, 13, grouping="uniform"),
        "orientation": TextOrientation.fixed(TextRotation(30)),
        "anchor": TextAnchor.fractions(1.25, -0.5),
        "overflow": TextOverflow(
            failure="clip",
            ellipsis=TextEllipsis("middle", marker="..", max_lines=2, distinct=False),
        ),
        "visibility": TextVisibility.parity(
            endpoints="both", separation=3, boundary_tolerance=-1,
        ),
    }
    original = TextLayout(**fields)
    alias = original
    authored = Text("Unchanged source", layout=original)
    assert {name: getattr(original, name) for name in fields} == fields
    for field in ("fit", "orientation", "anchor", "overflow", "visibility"):
        remaining = dict(fields)
        del remaining[field]
        cleared = original.copy_with(**{field: None})
        assert getattr(cleared, field) is None
        assert {name: getattr(cleared, name) for name in remaining} == remaining
        restored = cleared.copy_with(**{field: fields[field]})
        assert {name: getattr(restored, name) for name in fields} == fields
    assert {name: getattr(original, name) for name in fields} == fields
    assert original == alias == authored.layout
    assert original.copy_with() == original
    with pytest.raises(AttributeError):
        original.fit = None
    with pytest.raises(TypeError):
        original.copy_with(fit=None, anchor="not an anchor")
    assert {name: getattr(original, name) for name in fields} == fields
    assert original == alias


def test_fit_rotation_and_anchor_copies_retain_native_closed_programs():
    fixed = TextFit.fixed()
    shrink = TextFit.shrink_to(1_000_000)
    exact = TextFit.range(11, 11)
    assert (fixed.minimum, fixed.maximum) == (None, None)
    assert (shrink.minimum, shrink.maximum) == (1_000_000, None)
    uniform = exact.copy_with(grouping="uniform")
    assert (uniform.minimum, uniform.maximum, uniform.grouping) == (11, 11, "uniform")
    assert exact == TextFit.range(11, 11, grouping="independent")
    assert shrink.copy_with(grouping="uniform") == TextFit.shrink_to(
        1_000_000, grouping="uniform",
    )
    rotation = TextRotation(540)
    assert rotation.degrees == -180
    assert rotation.copy_with(degrees=720) == TextRotation.horizontal()
    assert TextRotation.vertical_up() == TextRotation(-90)
    assert rotation.degrees == -180

    fraction = TextAnchor.fractions(-0.25, 1.5)
    first = TextAnchor.first_baseline(0.25)
    center = TextAnchor.center_baseline(0.25)
    assert fraction.copy_with(horizontal=2) == TextAnchor.fractions(2, 1.5)
    assert fraction.horizontal == -0.25 and fraction.baseline is None
    assert first.copy_with(horizontal=-1) == TextAnchor.first_baseline(-1)
    assert center.copy_with(horizontal=2) == TextAnchor.center_baseline(2)
    assert first.vertical_fraction is None and first.baseline == "first"
    assert center.vertical_fraction is None and center.baseline == "center"
    assert TextAnchor.bottom_right().copy_with(horizontal=0.5) == TextAnchor.bottom_center()
    with pytest.raises(DataError):
        TextFit.range(12, 8)
    with pytest.raises(DataError):
        TextRotation(float("inf"))
    with pytest.raises(TypeError):
        fixed.copy_with(grouping=None)
    with pytest.raises(TypeError):
        first.copy_with(horizontal=None)
    with pytest.raises(TypeError):
        TextFit()
    with pytest.raises(AttributeError):
        rotation.degrees = 0


def test_coordinate_and_orientation_copies_preserve_branch_and_empty_presence():
    numeric = DataCoordinate.numeric(0)
    category = DataCoordinate.category(" raw category ")
    series = DataCoordinate.category_series("", " raw series ")
    assert numeric.kind == "numeric" and numeric.copy_with(value=2).value == 2
    assert category.kind == "category" and category.value == " raw category "
    assert series.kind == "category_series" and series.value == ("", " raw series ")
    assert series.copy_with(value=("next", "")).value == ("next", "")
    origin = DataPosition(x=numeric, y=category)
    target = origin.copy_with(x=series)
    assert target.y == origin.y == category and target.x == series
    direction = TextOrientation.data_direction(origin, target)
    changed = direction.copy_with(data_to=origin)
    assert changed.data_from == changed.data_to == origin
    assert direction.data_to == target and direction.rotation is None
    assert direction.angles is None
    fixed = TextOrientation.fixed(TextRotation.tilt_up())
    assert fixed.rotation == TextRotation(-45) and fixed.data_from is None
    assert fixed.copy_with(rotation=TextRotation.tilt_down()).rotation == TextRotation(45)
    supplied = [TextRotation.horizontal(), TextRotation.vertical_down()]
    automatic = TextOrientation.automatic_ticks(supplied)
    supplied.clear()
    assert automatic.angles == (TextRotation(0), TextRotation(90))
    empty = automatic.copy_with(angles=[])
    assert empty.angles == () and empty.rotation is None and empty.data_to is None
    assert empty.copy_with() == TextOrientation.automatic_ticks(())
    assert automatic.angles != empty.angles
    with pytest.raises(DataError):
        DataCoordinate.numeric(float("nan"))
    with pytest.raises(TypeError):
        series.copy_with(value="not a category/series pair")
    with pytest.raises(TypeError):
        numeric.copy_with(value=None)
    with pytest.raises(TypeError):
        origin.copy_with(x=None)
    with pytest.raises(TypeError, match="branch"):
        fixed.copy_with(angles=None)
    with pytest.raises(TypeError):
        direction.copy_with(data_from=None)
    with pytest.raises(TypeError, match="branch"):
        automatic.copy_with(rotation=TextRotation.horizontal())


def test_ellipsis_optional_clear_preserves_false_zero_empty_and_overflow_failure():
    original = TextEllipsis("middle", marker="", max_lines=0, distinct=False)
    alias = original
    cleared = original.copy_with(max_lines=None, distinct=None)
    assert cleared == TextEllipsis("middle", marker="", max_lines=None)
    assert (original.marker, original.max_lines, original.distinct) == ("", 0, False)
    assert cleared.distinct is None and cleared.max_lines is None
    changed = cleared.copy_with(position="left", marker="..", distinct=False)
    assert changed.position == "left" and changed.distinct is False
    assert alias == original
    overflow = TextOverflow(failure="clip", ellipsis=original)
    assert overflow.copy_with(ellipsis=None) == TextOverflow(failure="clip")
    assert overflow.ellipsis == original
    assert overflow.copy_with(failure="visible", ellipsis=cleared) == TextOverflow(
        failure="visible", ellipsis=cleared,
    )
    # Marker admission belongs to a reached native abbreviation operation.
    assert original.copy_with(marker="\t").marker == "\t"
    with pytest.raises(TypeError):
        overflow.copy_with(failure=None)
    with pytest.raises(TypeError):
        original.copy_with(marker=None)


def test_visibility_program_edits_and_owner_context_have_distinct_boundaries(tmp_path):
    every = TextVisibility.every(3, 1, endpoints="both", boundary_tolerance=-2)
    cleared = every.copy_with(boundary_tolerance=None)
    assert cleared.kind == "every" and (cleared.stride, cleared.offset) == (3, 1)
    assert cleared.endpoints == "both" and cleared.boundary_tolerance is None
    assert every.boundary_tolerance == -2
    greedy = TextVisibility.greedy(endpoints="first", separation=4)
    assert greedy.copy_with(boundary_tolerance=-3).kind == "greedy"
    assert greedy.separation == 4 and greedy.endpoints == "first"
    parity = TextVisibility.parity(endpoints="last", separation=2)
    assert parity.copy_with(separation=0) == TextVisibility.parity(endpoints="last")
    assert TextVisibility.hidden().kind == "hidden"
    with pytest.raises(DataError):
        TextVisibility.all().copy_with(separation=0)
    with pytest.raises(DataError):
        TextVisibility.every(0, 0, endpoints="none")
    with pytest.raises(DataError):
        TextVisibility.every(2, 2, endpoints="both")
    with pytest.raises(DataError):
        greedy.copy_with(separation=-1)
    with pytest.raises(TypeError):
        greedy.copy_with(separation=None)

    # All+None has no group reduction; present zero does request one.
    boundary = TextVisibility.all(boundary_tolerance=0)
    assert boundary.kind == "all" and boundary.boundary_tolerance == 0
    grouped = scatter(x=[0, 1], y=[2, 3]).title(
        Text("Independent title", layout=TextLayout(visibility=boundary)),
    )
    with pytest.raises(DataError, match="actual label group"):
        grouped.svg_preserve_text(tmp_path / "title-boundary-reduction.svg")
    independent = grouped.copy().title(
        "Independent title",
        layout=TextLayout(visibility=boundary.copy_with(boundary_tolerance=None)),
    )
    independent.svg_preserve_text(tmp_path / "title-no-boundary-reduction.svg")
    automatic = independent.copy().title(
        "Independent title",
        layout=TextLayout(orientation=TextOrientation.automatic_ticks([])),
    )
    with pytest.raises(DataError, match="actual tick group"):
        automatic.svg_preserve_text(tmp_path / "title-automatic-orientation.svg")


def _title_view(plot, path):
    plot.svg_preserve_text(path)
    root = ElementTree.parse(path).getroot()
    titles = [node for node in root.iter(SVG_TEXT) if node.get("fill") == "#194A75"]
    assert len(titles) == 1
    return "".join(titles[0].itertext()), float(titles[0].get("font-size"))


def test_full_layout_publication_uses_fit_overflow_and_preserves_authored_source(tmp_path):
    source = "Complete text program retains its unmodified authored source"
    style = Theme.publication().typography.root.title.copy_with(
        font_size=18, color=Color.hex("#194A75"),
    )
    ellipsis = TextEllipsis("end", marker="...", distinct=False)
    layout = TextLayout(
        alignment="center",
        fit_priority="content_first",
        direction="left_to_right",
        rotation_mode="rotate_about_anchor",
        offset=TextOffset(0, 0, frame="text_local"),
        bounds=TextBounds(max_width=100, max_lines=1),
        wrapping="no_soft_wrap",
        overlap="allow",
        fit=TextFit.range(10, 10),
        orientation=TextOrientation.fixed(TextRotation.horizontal()),
        anchor=TextAnchor.center(),
        overflow=TextOverflow(failure="error", ellipsis=ellipsis),
        visibility=TextVisibility.all(),
    )
    authored = Text(source, style=style, layout=layout)
    plot = scatter(x=[0, 1], y=[2, 3]).size(640, 480).title(authored)
    abbreviated, fitted_size = _title_view(plot, tmp_path / "complete-bounded.svg")
    assert abbreviated != source and abbreviated.endswith("...")
    assert fitted_size == layout.fit.minimum == layout.fit.maximum
    expanded = authored.copy_with(layout=layout.copy_with(bounds=None))
    complete, same_size = _title_view(
        plot.copy().title(expanded), tmp_path / "complete-unbounded.svg",
    )
    assert complete == source and same_size == fitted_size
    preferred = expanded.copy_with(layout=expanded.layout.copy_with(fit=None))
    complete, preferred_size = _title_view(
        plot.copy().title(preferred), tmp_path / "complete-preferred.svg",
    )
    assert complete == source and preferred_size == style.font_size > fitted_size
    assert authored.source == source and authored.layout == layout

    bad_marker = layout.overflow.copy_with(ellipsis=ellipsis.copy_with(marker="\t"))
    unused = expanded.copy_with(layout=expanded.layout.copy_with(overflow=bad_marker))
    assert _title_view(
        plot.copy().title(unused), tmp_path / "unused-marker.svg",
    )[0] == source
    reached = authored.copy_with(layout=layout.copy_with(overflow=bad_marker))
    with pytest.raises(DataError, match="ellipsis marker.*U\\+0009"):
        plot.copy().title(reached).svg_preserve_text(tmp_path / "reached-marker.svg")
