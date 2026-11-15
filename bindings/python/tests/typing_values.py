"""Complete appearance/Text contracts for installed strict checking, not execution."""

from typing import Literal

from typing_extensions import assert_type

import lichtung_plot as lp


def theme_graph(theme: lp.Theme, plot: lp.Scatter, canvas: lp.Canvas) -> None:
    assert_type(theme.axes, lp.AxisTheme)
    assert_type(theme.legend, lp.LegendTheme)
    assert_type(theme.composition, lp.CompositionTheme)
    assert_type(theme.marks, lp.MarkTheme)
    assert_type(theme.annotations, lp.AnnotationTheme)
    assert_type(theme.color_scales, lp.ColorScales)

    stroke = lp.StrokeTheme(
        color=lp.Color.hex("#194A75"), width=1.5, pattern="short_dash"
    )
    frame = lp.FrameTheme(
        sides=lp.FrameSides(top=False, right=False, bottom=True, left=True),
        stroke=stroke,
    )
    axes = theme.axes.copy_with(frame=frame, grid=None)
    primary = theme.marks.points.primary.copy_with(
        shape="triangle_up", outline=None
    )
    points = theme.marks.points.copy_with(primary=primary)
    cells = theme.marks.cells.copy_with(
        missing_fill=lp.Color(250, 250, 250), cell_outline=None
    )
    marks = theme.marks.copy_with(points=points, cells=cells)
    composition = theme.composition.copy_with(
        root_subtitle_alignment=None,
        root_padding=theme.composition.root_padding.copy_with(
            top=lp.RelativeSpacing.ratio(3, 2)
        ),
    )
    derived = lp.Theme(
        typography=theme.typography,
        surfaces=theme.surfaces,
        axes=axes,
        legend=theme.legend,
        composition=composition,
        marks=marks,
        annotations=theme.annotations,
        color_scales=theme.color_scales,
    )
    assert_type(derived.copy_with(axes=axes, marks=marks), lp.Theme)
    assert_type(plot.theme(derived), lp.Scatter)
    assert_type(canvas.theme(derived), lp.Canvas)
    assert_type(primary.shape, Literal["circle", "triangle_up"])

    theme.axes = axes  # type: ignore[misc]
    theme.axes.copy_with(spacing=1.0)  # type: ignore[arg-type]
    theme.copy_with(marks=None)  # type: ignore[arg-type]
    lp.Theme(axes=None)  # type: ignore[arg-type]
    lp.FrameSides(True, False, False, False)  # type: ignore[misc]
    primary.copy_with(shape="square")  # type: ignore[arg-type]
    stroke.copy_with(width=None)  # type: ignore[arg-type]


def exact_spacing(theme: lp.Theme) -> None:
    spacing = lp.RelativeSpacing.ratio(1, 3)
    assert_type(
        theme.legend.gradient_lengths.copy_with(
            minimum=spacing, preferred=lp.RelativeSpacing.ratio(2, 3)
        ),
        lp.GradientLengths,
    )
    assert_type(
        theme.annotations.copy_with(
            point_label_gap=spacing,
            significance=theme.annotations.significance.copy_with(
                label_gap=spacing
            ),
        ),
        lp.AnnotationTheme,
    )
    assert_type(
        theme.marks.extents.copy_with(
            categories=theme.marks.extents.categories.copy_with(
                vertical_body_breadth=spacing
            )
        ),
        lp.MarkExtentTheme,
    )
    theme.legend.gradient_lengths.copy_with(minimum=0.5)  # type: ignore[arg-type]
    theme.composition.copy_with(root_padding=None)  # type: ignore[arg-type]


def palette_and_closed_paint(theme: lp.Theme) -> None:
    colors = [lp.OpaqueColor(10, 40, 70), lp.OpaqueColor(170, 100, 20)]
    palette = lp.QualitativePalette.extendable(colors)
    stops = [lp.ColorStop(0.0, colors[0]), lp.ColorStop(1.0, colors[1])]
    gradient = lp.SequentialGradient(stops)
    scales = theme.color_scales.copy_with(
        qualitative=palette, sequential=gradient
    )
    assert_type(scales, lp.ColorScales)
    assert_type(palette.colors, tuple[lp.OpaqueColor, ...])
    assert_type(palette.capacity, Literal["fixed", "extendable"])
    assert_type(gradient.stops, tuple[lp.ColorStop, ...])
    assert_type(
        lp.AccentPalette(colors).copy_with(seeds=tuple(colors)),
        lp.AccentPalette,
    )
    outline = lp.DataOutlineTheme.mapped(1.5, 128)
    assert_type(outline.fixed_stroke, lp.OutlineTheme | None)
    assert_type(outline.mapped_width, float | None)
    assert_type(outline.copy_with(mapped_alpha=64), lp.DataOutlineTheme)
    lp.QualitativePalette()  # type: ignore[call-arg]
    lp.DataOutlineTheme()  # type: ignore[call-arg]
    lp.QualitativePalette.fixed([lp.Color(0, 0, 0)])  # type: ignore[list-item]
    lp.ColorStop("zero", colors[0])  # type: ignore[arg-type]
    palette.copy_with(capacity="cycle")  # type: ignore[arg-type]
    outline.copy_with(mapped_width=None)  # type: ignore[arg-type]


def complete_text_layout(style: lp.TextTheme, plot: lp.Scatter) -> None:
    fit = lp.TextFit.range(10.0, 18.0, grouping="independent")
    rotation = lp.TextRotation(15)
    orientation = lp.TextOrientation.fixed(rotation)
    anchor = lp.TextAnchor.first_baseline(0.5)
    ellipsis = lp.TextEllipsis(
        "middle", marker="…", max_lines=None, distinct=False
    )
    overflow = lp.TextOverflow(failure="error", ellipsis=ellipsis)
    visibility = lp.TextVisibility.all()
    layout = lp.TextLayout(
        alignment="center",
        fit_priority="content_first",
        fit=fit,
        orientation=orientation,
        direction="auto",
        anchor=anchor,
        offset=lp.TextOffset(0.0, 0.0, frame="text_local"),
        bounds=lp.TextBounds(max_width=240),
        overflow=overflow,
        visibility=visibility,
        wrapping="legal",
        overlap="allow",
        rotation_mode="rotate_about_anchor",
    )
    assert_type(layout.fit, lp.TextFit | None)
    assert_type(layout.orientation, lp.TextOrientation | None)
    assert_type(layout.anchor, lp.TextAnchor | None)
    assert_type(layout.overflow, lp.TextOverflow | None)
    assert_type(layout.visibility, lp.TextVisibility | None)
    assert_type(layout.copy_with(fit=None, visibility=None), lp.TextLayout)
    assert_type(plot.title(lp.Text("Results", style=style, layout=layout)), lp.Scatter)
    assert_type(fit.minimum, float | None)
    assert_type(rotation.degrees, float)
    assert_type(anchor.baseline, Literal["first", "center"] | None)
    layout.fit = None  # type: ignore[misc]
    lp.TextFit()  # type: ignore[call-arg]
    lp.TextOrientation()  # type: ignore[call-arg]
    lp.TextAnchor()  # type: ignore[call-arg]
    lp.TextFit.fixed().copy_with(minimum=10.0)  # type: ignore[call-arg]
    layout.copy_with(orientation=15)  # type: ignore[arg-type]
    ellipsis.copy_with(marker=None)  # type: ignore[arg-type]


def authored_locations_and_visibility() -> None:
    start = lp.DataPosition(
        x=lp.DataCoordinate.category_series("Group", "Series"),
        y=lp.DataCoordinate.numeric(1.0),
    )
    end = start.copy_with(y=lp.DataCoordinate.numeric(2.0))
    orientation = lp.TextOrientation.data_direction(start, end)
    assert_type(start.x, lp.DataCoordinate)
    assert_type(start.x.value, float | str | tuple[str, str])
    assert_type(orientation.data_from, lp.DataPosition | None)
    assert_type(orientation.copy_with(data_to=end), lp.TextOrientation)
    assert_type(
        lp.TextOrientation.automatic_ticks([]).angles,
        tuple[lp.TextRotation, ...] | None,
    )
    visibility = lp.TextVisibility.every(3, 1, endpoints="both")
    assert_type(visibility.stride, int | None)
    assert_type(
        visibility.copy_with(boundary_tolerance=None),
        lp.TextVisibility,
    )
    assert_type(
        lp.TextVisibility.greedy(
            endpoints="first", separation=2.0, boundary_tolerance=-1.0
        ),
        lp.TextVisibility,
    )
    lp.DataCoordinate()  # type: ignore[call-arg]
    lp.DataCoordinate.numeric([1.0])  # type: ignore[arg-type]
    lp.TextVisibility.every(3, 1)  # type: ignore[call-arg]
    visibility.copy_with(endpoints="none")  # type: ignore[call-arg]
    lp.TextOverflow(failure="ignore")  # type: ignore[arg-type]
    lp.scatter(x=start.x, y=[1.0])  # type: ignore[call-overload]
