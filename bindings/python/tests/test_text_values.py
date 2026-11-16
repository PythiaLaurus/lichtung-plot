"""Complete authored text programs through immutable values and real title slots."""

from xml.etree import ElementTree

import pytest

from lichtung_plot import (
    Color,
    DataError,
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
