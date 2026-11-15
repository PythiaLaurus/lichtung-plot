"""Thin mutable handles; all scientific and style state lives in the extension."""

from __future__ import annotations

from os import PathLike
from typing import (
    TYPE_CHECKING,
    Callable,
    Literal,
    Mapping,
    NoReturn,
    Sequence,
    TypeAlias,
    TypeVar,
    cast,
    overload,
)

from . import _native
from ._native import (
    CategoricalColorScale,
    Color,
    ColumnSelector,
    ColumnsSelector,
    ConfidenceInterval,
    ContinuousColorScale,
    InteractionStyle,
    Labels,
    NumericFormat,
    Output,
    PercentileInterval,
    PngOptions,
    SignificanceMarkStyle,
    StandardDeviation,
    StandardError,
    Text,
    TextLayout,
    TextPosition,
    TextTheme,
    Theme,
    TickGeneration,
)

if TYPE_CHECKING:
    from typing import Any

    import numpy as _np
    import pandas as _pd
    import polars as _pl

    # Annotation-only aliases keep optional input providers out of runtime imports.
    # Rank, dtype, validity and source-dependent lookup remain native checks.
    _NativeNumericColumn: TypeAlias = (
        list[int]
        | list[float]
        | list[int | None]
        | list[float | None]
        | tuple[float | None, ...]
    )
    _NativeNumericRectangle: TypeAlias = (
        list[list[int]]
        | list[list[float]]
        | list[list[int | None]]
        | list[list[float | None]]
        | list[tuple[int, ...]]
        | list[tuple[float, ...]]
        | list[tuple[int | None, ...]]
        | list[tuple[float | None, ...]]
        | list[_NativeNumericColumn]
        | tuple[_NativeNumericColumn, ...]
    )
    _NativeCategoryColumn: TypeAlias = list[str] | list[str | None] | tuple[str | None, ...]
    _NativeBooleanColumn: TypeAlias = (
        list[bool] | list[bool | None] | tuple[bool | None, ...]
    )
    _NativeBooleanRectangle: TypeAlias = (
        list[list[bool]]
        | list[list[bool | None]]
        | list[tuple[bool, ...]]
        | list[tuple[bool | None, ...]]
        | list[_NativeBooleanColumn]
        | tuple[_NativeBooleanColumn, ...]
    )
    _NativeRowIds: TypeAlias = list[int] | tuple[int, ...]
    _NativeRowIdRectangle: TypeAlias = (
        list[list[int]]
        | list[tuple[int, ...]]
        | list[_NativeRowIds]
        | tuple[_NativeRowIds, ...]
    )
    _Array: TypeAlias = _np.ndarray[tuple[int, ...], _np.dtype[_np.generic]]
    _Table: TypeAlias = _pd.DataFrame | _pl.DataFrame
    _Series: TypeAlias = _pd.Series | _pl.Series
    _DataSource: TypeAlias = _Array | _Table
    # Bare str/int are shorthand. Complete keys use col[Hashable]; a structural
    # index protocol would also admit tables with dynamic __getattr__.
    _SingleColumn: TypeAlias = str | int | _np.integer[Any] | ColumnSelector
    _NumericInput: TypeAlias = _NativeNumericColumn | _Array | _Series | _SingleColumn
    _NumericRectangle: TypeAlias = (
        _NativeNumericRectangle | _Array | _Table | ColumnsSelector
    )
    _CategoryInput: TypeAlias = _NativeCategoryColumn | _Array | _Series | _SingleColumn
    _BooleanColumnInput: TypeAlias = (
        _NativeBooleanColumn | _Array | _Series | _SingleColumn
    )
    _BooleanInput: TypeAlias = (
        _BooleanColumnInput | _NativeBooleanRectangle | _Table | ColumnsSelector
    )
    _SourceRowsInput: TypeAlias = (
        _NativeRowIds
        | _NativeRowIdRectangle
        | _Array
        | _Series
        | _Table
        | _SingleColumn
        | ColumnsSelector
    )
    _AlphaInput: TypeAlias = (
        list[int] | tuple[int, ...] | _Array | _Series | _SingleColumn
    )
    _IdInput: TypeAlias = (
        list[int] | tuple[int, ...] | _Array | _Series | _SingleColumn
    )
    _K = TypeVar("_K", bound=tuple[str | ColumnSelector, _CategoryInput])
    _AlphaKeys: TypeAlias = list[_K] | tuple[_K, ...]
    _AlphaLinear: TypeAlias = tuple[tuple[float, float], tuple[int, int]]
    _AlphaThreshold: TypeAlias = tuple[
        list[int] | list[float] | tuple[float, ...], list[int] | tuple[int, ...]
    ]
    _AlphaContinuous: TypeAlias = tuple[tuple[float, float], Callable[[float], int]]

ExactTicks: TypeAlias = list[tuple[float, str]] | tuple[tuple[float, str], ...]
PathInput: TypeAlias = str | bytes | PathLike[str] | PathLike[bytes]
TextInput: TypeAlias = str | Text
LegendPosition: TypeAlias = (
    Literal["auto"]
    | Literal["inside_top_left"]
    | Literal["inside_top"]
    | Literal["inside_top_right"]
    | Literal["inside_right_top"]
    | Literal["inside_right"]
    | Literal["inside_right_bottom"]
    | Literal["inside_bottom_left"]
    | Literal["inside_bottom"]
    | Literal["inside_bottom_right"]
    | Literal["inside_left_top"]
    | Literal["inside_left"]
    | Literal["inside_left_bottom"]
    | Literal["outside_right_top"]
    | Literal["outside_right"]
    | Literal["outside_right_bottom"]
    | Literal["outside_top_left"]
    | Literal["outside_top"]
    | Literal["outside_top_right"]
    | Literal["outside_bottom_left"]
    | Literal["outside_bottom"]
    | Literal["outside_bottom_right"]
    | Literal["outside_left_top"]
    | Literal["outside_left"]
    | Literal["outside_left_bottom"]
)
_Order: TypeAlias = (
    Literal["observed", "value_ascending", "value_descending"]
    | list[str] | tuple[str, ...]
)
_HeatmapOrder: TypeAlias = Literal["observed"] | list[str] | tuple[str, ...]
_Alignment: TypeAlias = Literal["start"] | Literal["center"] | Literal["end"]
_GuideRole: TypeAlias = Literal["title"] | Literal["entry"] | Literal["number"]
_Composition: TypeAlias = Literal["combined"] | Literal["separate"]
_Endpoints: TypeAlias = (
    Literal["none"] | Literal["first"] | Literal["last"] | Literal["both"]
)
_ErrorBar: TypeAlias = (
    StandardDeviation | StandardError | PercentileInterval | ConfidenceInterval
    | Literal["hidden"]
)
_EstimateStatus: TypeAlias = Literal["estimated"] | Literal["not_estimable"]
_StatusMapping: TypeAlias = (
    Mapping[str, _EstimateStatus]
    | list[tuple[str, _EstimateStatus]]
    | tuple[tuple[str, _EstimateStatus], ...]
)


class _Handle:
    __slots__ = ()
    _owner: _native._PlotOwner


_P = TypeVar("_P", bound="Plot")
_H = TypeVar("_H", bound=_Handle)
_UNSET = object()


class Plot(_Handle):
    """Common configuration and output operations of exact plot families."""

    __slots__ = ("_owner", "__weakref__")
    _owner: _native._PlotOwner

    def __init__(self, _unavailable: NoReturn, /) -> None:
        raise TypeError("use a plot-family constructor such as scatter(...)")

    @classmethod
    def _from_owner(cls: type[_P], owner: _native._PlotOwner) -> _P:
        instance = object.__new__(cls)
        instance._owner = owner
        return instance

    def copy(self: _P) -> _P:
        return type(self)._from_owner(self._owner.copy())

    def source_rows(self: _P, values: _SourceRowsInput) -> _P:
        self._owner.source_rows(values)
        return self

    def source_rows_cells(self: _P, values: _SourceRowsInput) -> _P:
        self._owner.source_rows_cells(values)
        return self

    def column_labels(self: _P, labels: list[str] | tuple[str, ...]) -> _P:
        self._owner.column_labels(labels)
        return self

    def labels(self: _P, labels: Labels) -> _P:
        self._owner.labels(labels)
        return self

    def title(
        self: _P,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _P:
        self._owner.title(text, style, layout, _UNSET)
        return self

    def subtitle(
        self: _P,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _P:
        self._owner.subtitle(text, style, layout, _UNSET)
        return self

    def caption(
        self: _P,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _P:
        self._owner.caption(text, style, layout, _UNSET)
        return self

    def x_label(
        self: _P,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _P:
        self._owner.x_label(text, style, layout, _UNSET)
        return self

    def y_label(
        self: _P,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _P:
        self._owner.y_label(text, style, layout, _UNSET)
        return self

    def theme(self: _P, theme: Theme) -> _P:
        self._owner.theme(theme)
        return self

    def width(self: _P, width: int) -> _P:
        self._owner.width(width)
        return self

    def max_width(self: _P, maximum: int) -> _P:
        self._owner.max_width(maximum)
        return self

    def size(self: _P, width: int, height: int) -> _P:
        self._owner.size(width, height)
        return self

    def title_alignment(self: _P, alignment: _Alignment) -> _P:
        self._owner.title_alignment(alignment)
        return self

    def subtitle_alignment(self: _P, alignment: _Alignment) -> _P:
        self._owner.subtitle_alignment(alignment)
        return self

    def clear_title_style(self: _P) -> _P:
        self._owner.clear_title_style()
        return self

    def clear_title_layout(self: _P) -> _P:
        self._owner.clear_title_layout()
        return self

    def clear_subtitle_style(self: _P) -> _P:
        self._owner.clear_subtitle_style()
        return self

    def clear_subtitle_layout(self: _P) -> _P:
        self._owner.clear_subtitle_layout()
        return self

    def clear_caption_style(self: _P) -> _P:
        self._owner.clear_caption_style()
        return self

    def clear_caption_layout(self: _P) -> _P:
        self._owner.clear_caption_layout()
        return self

    def clear_x_label_style(self: _P) -> _P:
        self._owner.clear_x_label_style()
        return self

    def clear_x_label_layout(self: _P) -> _P:
        self._owner.clear_x_label_layout()
        return self

    def clear_y_label_style(self: _P) -> _P:
        self._owner.clear_y_label_style()
        return self

    def clear_y_label_layout(self: _P) -> _P:
        self._owner.clear_y_label_layout()
        return self

    def x_tick_text_layout(self: _P, layout: TextLayout) -> _P:
        self._owner.x_tick_text_layout(layout)
        return self

    def y_tick_text_layout(self: _P, layout: TextLayout) -> _P:
        self._owner.y_tick_text_layout(layout)
        return self

    def guide_text_layout(self: _P, role: _GuideRole, layout: TextLayout) -> _P:
        self._owner.guide_text_layout(role, layout)
        return self

    @overload
    def legend(self: _P, policy: Literal["automatic", "hidden"]) -> _P: ...

    @overload
    def legend(
        self: _P, policy: Literal["visible"], *, position: LegendPosition = "auto"
    ) -> _P: ...

    def legend(
        self: _P,
        policy: Literal["automatic", "hidden", "visible"],
        *,
        position: LegendPosition = cast(LegendPosition, _UNSET),
    ) -> _P:
        self._owner.legend(policy, position, _UNSET)
        return self

    def legend_composition(self: _P, composition: _Composition) -> _P:
        self._owner.legend_composition(composition)
        return self

    def svg(self, path: PathInput) -> Output:
        return self._owner.svg(path)

    def svg_preserve_text(self, path: PathInput) -> Output:
        return self._owner.svg_preserve_text(path)

    def png(self, path: PathInput) -> Output:
        return self._owner.png(path)

    def png_preview(self, path: PathInput) -> Output:
        return self._owner.png_preview(path)

    def png_with_options(self, path: PathInput, options: PngOptions) -> Output:
        return self._owner.png_with_options(path, options)


class _NumericX(_Handle):
    __slots__ = ()

    def x_limits(self: _H, minimum: float, maximum: float) -> _H:
        self._owner.x_limits(minimum, maximum)
        return self

    def x_ticks(self: _H, ticks: ExactTicks) -> _H:
        self._owner.x_ticks(ticks)
        return self

    def x_tick_format(self: _H, format: NumericFormat) -> _H:
        self._owner.x_tick_format(format)
        return self

    def x_tick_generation(self: _H, generation: TickGeneration) -> _H:
        self._owner.x_tick_generation(generation)
        return self


class _NumericY(_Handle):
    __slots__ = ()

    def y_limits(self: _H, minimum: float, maximum: float) -> _H:
        self._owner.y_limits(minimum, maximum)
        return self

    def y_ticks(self: _H, ticks: ExactTicks) -> _H:
        self._owner.y_ticks(ticks)
        return self

    def y_tick_format(self: _H, format: NumericFormat) -> _H:
        self._owner.y_tick_format(format)
        return self

    def y_tick_generation(self: _H, generation: TickGeneration) -> _H:
        self._owner.y_tick_generation(generation)
        return self


class _Hue(_Handle):
    __slots__ = ()

    def hue(self: _H, values: _CategoryInput) -> _H:
        self._owner.hue(values)
        return self

    def hue_cells(self: _H, values: _CategoryInput) -> _H:
        self._owner.hue_cells(values)
        return self

    def hue_columns(self: _H) -> _H:
        self._owner.hue_columns()
        return self


class _Group(_Handle):
    __slots__ = ()

    def group(self: _H, values: _CategoryInput) -> _H:
        self._owner.group(values)
        return self

    def group_cells(self: _H, values: _CategoryInput) -> _H:
        self._owner.group_cells(values)
        return self

    def group_columns(self: _H) -> _H:
        self._owner.group_columns()
        return self


class _SeriesRole(_Handle):
    __slots__ = ()

    def series(self: _H, values: _CategoryInput) -> _H:
        self._owner.series(values)
        return self

    def series_cells(self: _H, values: _CategoryInput) -> _H:
        self._owner.series_cells(values)
        return self

    def series_columns(self: _H) -> _H:
        self._owner.series_columns()
        return self


class _Facet(_Handle):
    __slots__ = ()

    def facet_row(self: _H, values: _CategoryInput) -> _H:
        self._owner.facet_row(values)
        return self

    def facet_column(self: _H, values: _CategoryInput) -> _H:
        self._owner.facet_column(values)
        return self

    def facet_grid(self: _H, row: _CategoryInput, column: _CategoryInput) -> _H:
        self._owner.facet_grid(row, column)
        return self

    def facet_wrap(self: _H, values: _CategoryInput, columns: int) -> _H:
        self._owner.facet_wrap(values, columns)
        return self

    def facet_row_cells(self: _H, values: _CategoryInput) -> _H:
        self._owner.facet_row_cells(values)
        return self

    def facet_column_cells(self: _H, values: _CategoryInput) -> _H:
        self._owner.facet_column_cells(values)
        return self

    def facet_wrap_cells(self: _H, values: _CategoryInput, columns: int) -> _H:
        self._owner.facet_wrap_cells(values, columns)
        return self

    def facet_grid_cells(
        self: _H, row: _CategoryInput, column: _CategoryInput
    ) -> _H:
        self._owner.facet_grid_cells(row, column)
        return self

    def facet_grid_row_cells(
        self: _H, row: _CategoryInput, column: _CategoryInput
    ) -> _H:
        self._owner.facet_grid_row_cells(row, column)
        return self

    def facet_grid_column_cells(
        self: _H, row: _CategoryInput, column: _CategoryInput
    ) -> _H:
        self._owner.facet_grid_column_cells(row, column)
        return self

    def facet_columns_row(self: _H) -> _H:
        self._owner.facet_columns_row()
        return self

    def facet_columns_column(self: _H) -> _H:
        self._owner.facet_columns_column()
        return self

    def facet_columns_wrap(self: _H, columns: int) -> _H:
        self._owner.facet_columns_wrap(columns)
        return self

    def facet_grid_columns_row(self: _H, column: _CategoryInput) -> _H:
        self._owner.facet_grid_columns_row(column)
        return self

    def facet_grid_columns_column(self: _H, row: _CategoryInput) -> _H:
        self._owner.facet_grid_columns_column(row)
        return self

    def facet_grid_columns_row_cells(self: _H, column: _CategoryInput) -> _H:
        self._owner.facet_grid_columns_row_cells(column)
        return self

    def facet_grid_columns_column_cells(self: _H, row: _CategoryInput) -> _H:
        self._owner.facet_grid_columns_column_cells(row)
        return self

    def facet_scales(self: _H, scales: Literal["fixed", "free_x", "free_y", "free"]) -> _H:
        self._owner.facet_scales(scales)
        return self

    def facet_row_text_layout(self: _H, layout: TextLayout) -> _H:
        self._owner.facet_row_text_layout(layout)
        return self

    def facet_column_text_layout(self: _H, layout: TextLayout) -> _H:
        self._owner.facet_column_text_layout(layout)
        return self


class _Guides(_Handle):
    __slots__ = ()

    def guide_horizontal(
        self: _H, y: float, *, color: Color, width: float,
        pattern: Literal["solid", "dashed"],
    ) -> _H:
        self._owner.guide_horizontal(y, color, width, pattern)
        return self

    def guide_vertical(
        self: _H, x: float, *, color: Color, width: float,
        pattern: Literal["solid", "dashed"],
    ) -> _H:
        self._owner.guide_vertical(x, color, width, pattern)
        return self

    def guide_segment(
        self: _H, x_start: float, y_start: float, x_end: float, y_end: float,
        *, color: Color, width: float, pattern: Literal["solid", "dashed"],
    ) -> _H:
        self._owner.guide_segment(
            x_start, y_start, x_end, y_end, color, width, pattern
        )
        return self


class _Annotations(_Handle):
    __slots__ = ()

    def text_annotation(
        self: _H, position: TextPosition, text: TextInput, *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _H:
        self._owner.text_annotation(position, text, style, layout, _UNSET)
        return self


class _Outline(_Handle):
    __slots__ = ()

    def hide_point_outline(self: _H) -> _H:
        self._owner.hide_point_outline()
        return self


class _CategoryOrder(_Handle):
    __slots__ = ()

    def category_order(self: _H, order: _Order) -> _H:
        self._owner.category_order(order)
        return self


class _ZeroLine(_Handle):
    __slots__ = ()

    def zero_line(self: _H) -> _H:
        self._owner.zero_line()
        return self


class _Markers(_Handle):
    __slots__ = ()

    def markers(self: _H, visible: bool) -> _H:
        self._owner.markers(visible)
        return self


class _CategoricalScale(_Handle):
    __slots__ = ()

    def categorical_color_scale(self: _H, scale: CategoricalColorScale) -> _H:
        self._owner.categorical_color_scale(scale)
        return self


class _ContinuousColor(_Handle):
    __slots__ = ()

    def continuous_color_scale(self: _H, scale: ContinuousColorScale) -> _H:
        self._owner.continuous_color_scale(scale)
        return self

    def color_label_format(self: _H, format: NumericFormat) -> _H:
        self._owner.color_label_format(format)
        return self

    def color_label(
        self: _H, text: TextInput, *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> _H:
        self._owner.color_label(text, style, layout, _UNSET)
        return self

    def clear_color_label_style(self: _H) -> _H:
        self._owner.clear_color_label_style()
        return self

    def clear_color_label_layout(self: _H) -> _H:
        self._owner.clear_color_label_layout()
        return self


class _RawOverlay(_Handle):
    __slots__ = ()

    def raw_overlay(self: _H, overlay: Literal["hidden", "points", "jittered"]) -> _H:
        self._owner.raw_overlay(overlay)
        return self


class _SignificanceStyle(_Handle):
    __slots__ = ()

    def significance_mark_style(self: _H, style: SignificanceMarkStyle) -> _H:
        self._owner.significance_mark_style(style)
        return self

    def significance_text_layout(self: _H, layout: TextLayout) -> _H:
        self._owner.significance_text_layout(layout)
        return self


class _EstimateDetails(_Handle):
    __slots__ = ()

    def estimate_detail_text_layout(self: _H, layout: TextLayout) -> _H:
        self._owner.estimate_detail_text_layout(layout)
        return self


class _CategoricalX(_Handle):
    __slots__ = ()

    def x_category_labels(
        self: _H, *, every: int = 1, offset: int = 0, endpoints: _Endpoints = "none"
    ) -> _H:
        self._owner.x_category_labels(every=every, offset=offset, endpoints=endpoints)
        return self


class _CategoricalY(_Handle):
    __slots__ = ()

    def y_category_labels(
        self: _H, *, every: int = 1, offset: int = 0, endpoints: _Endpoints = "none"
    ) -> _H:
        self._owner.y_category_labels(every=every, offset=offset, endpoints=endpoints)
        return self


class _EstimateStatusMethods(_Handle):
    __slots__ = ()

    def estimate_status(
        self: _H, values: list[_EstimateStatus] | tuple[_EstimateStatus, ...],
    ) -> _H:
        self._owner.estimate_status(values)
        return self

    def estimate_status_boolean(self: _H, values: _BooleanColumnInput) -> _H:
        self._owner.estimate_status_boolean(values)
        return self

    def estimate_status_categorical(
        self: _H, values: _CategoryInput, mapping: _StatusMapping,
    ) -> _H:
        self._owner.estimate_status_categorical(values, mapping)
        return self


class _PointAlpha(_Handle):
    __slots__ = ()

    def point_alpha(self: _H, input: _AlphaInput) -> _H:
        self._owner.point_alpha(input)
        return self

    def point_alpha_cells(self: _H, input: _AlphaInput) -> _H:
        self._owner.point_alpha_cells(input)
        return self

    def point_alpha_linear(
        self: _H, input: _NumericInput,
        domain: tuple[float, float], range: tuple[int, int],
    ) -> _H:
        self._owner.point_alpha_linear(input, domain, range)
        return self

    @overload
    def point_alpha_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_map(input, linear, log10, threshold, continuous, _UNSET)
        return self

    def point_alpha_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_map_categories(input, mapper)
        return self

    @overload
    def point_alpha_cells_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_cells_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_cells_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_cells_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_cells_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_cells_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def point_alpha_cells_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_cells_map_categories(input, mapper)
        return self

    @overload
    def point_alpha_single(self: _H, *, value: int) -> _H: ...

    @overload
    def point_alpha_single(self: _H, *, input: _AlphaInput) -> _H: ...

    def point_alpha_single(
        self: _H, *, value: int = cast(int, _UNSET),
        input: _AlphaInput = cast("_AlphaInput", _UNSET),
    ) -> _H:
        self._owner.point_alpha_single(value, input, _UNSET)
        return self

    @overload
    def point_alpha_single_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_single_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_single_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_single_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_single_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_single_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def point_alpha_single_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_single_map_categories(input, mapper)
        return self

    def point_alpha_keyed(
        self: _H, input: _AlphaInput, *, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.point_alpha_keyed(input, keys)
        return self

    @overload
    def point_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_keyed_map(
        self: _H, input: _NumericInput, *, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_keyed_map(
            input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def point_alpha_keyed_map_categories(
        self: _H, input: _CategoryInput, *,
        keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_keyed_map_categories(input, keys, mapper)
        return self

    def point_alpha_keyed_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _AlphaInput, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.point_alpha_keyed_from_data(data, input, keys)
        return self

    @overload
    def point_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_keyed_map_from_data(
            data, input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def point_alpha_keyed_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _CategoryInput, keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_keyed_map_categories_from_data(data, input, keys, mapper)
        return self

    def point_alpha_columns(
        self: _H, input: _AlphaInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.point_alpha_columns(input, column_key, other_keys)
        return self

    @overload
    def point_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_columns_map(
            input, column_key, other_keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def point_alpha_columns_map_categories(
        self: _H, input: _CategoryInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_columns_map_categories(
            input, column_key, other_keys, mapper
        )
        return self

    def point_alpha_columns_from_data(
        self: _H, data: _DataSource | None = None, *, input: _AlphaInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.point_alpha_columns_from_data(data, input, column_key, other_keys)
        return self

    @overload
    def point_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def point_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def point_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.point_alpha_columns_map_from_data(
            data, input, column_key, other_keys,
            linear, log10, threshold, continuous, _UNSET,
        )
        return self

    def point_alpha_columns_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *, input: _CategoryInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.point_alpha_columns_map_categories_from_data(
            data, input, column_key, other_keys, mapper
        )
        return self

    def point_alpha_theme(self: _H) -> _H:
        self._owner.point_alpha_theme()
        return self


class _LineAlpha(_Handle):
    __slots__ = ()

    def line_alpha(self: _H, input: _AlphaInput) -> _H:
        self._owner.line_alpha(input)
        return self

    def line_alpha_cells(self: _H, input: _AlphaInput) -> _H:
        self._owner.line_alpha_cells(input)
        return self

    def line_alpha_linear(
        self: _H, input: _NumericInput,
        domain: tuple[float, float], range: tuple[int, int],
    ) -> _H:
        self._owner.line_alpha_linear(input, domain, range)
        return self

    @overload
    def line_alpha_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_map(input, linear, log10, threshold, continuous, _UNSET)
        return self

    def line_alpha_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_map_categories(input, mapper)
        return self

    @overload
    def line_alpha_cells_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_cells_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_cells_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_cells_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_cells_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_cells_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def line_alpha_cells_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_cells_map_categories(input, mapper)
        return self

    @overload
    def line_alpha_single(self: _H, *, value: int) -> _H: ...

    @overload
    def line_alpha_single(self: _H, *, input: _AlphaInput) -> _H: ...

    def line_alpha_single(
        self: _H, *, value: int = cast(int, _UNSET),
        input: _AlphaInput = cast("_AlphaInput", _UNSET),
    ) -> _H:
        self._owner.line_alpha_single(value, input, _UNSET)
        return self

    @overload
    def line_alpha_single_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_single_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_single_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_single_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_single_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_single_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def line_alpha_single_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_single_map_categories(input, mapper)
        return self

    def line_alpha_keyed(
        self: _H, input: _AlphaInput, *, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.line_alpha_keyed(input, keys)
        return self

    @overload
    def line_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_keyed_map(
        self: _H, input: _NumericInput, *, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_keyed_map(
            input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def line_alpha_keyed_map_categories(
        self: _H, input: _CategoryInput, *,
        keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_keyed_map_categories(input, keys, mapper)
        return self

    def line_alpha_keyed_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _AlphaInput, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.line_alpha_keyed_from_data(data, input, keys)
        return self

    @overload
    def line_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_keyed_map_from_data(
            data, input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def line_alpha_keyed_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _CategoryInput, keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_keyed_map_categories_from_data(data, input, keys, mapper)
        return self

    def line_alpha_columns(
        self: _H, input: _AlphaInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.line_alpha_columns(input, column_key, other_keys)
        return self

    @overload
    def line_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_columns_map(
            input, column_key, other_keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def line_alpha_columns_map_categories(
        self: _H, input: _CategoryInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_columns_map_categories(
            input, column_key, other_keys, mapper
        )
        return self

    def line_alpha_columns_from_data(
        self: _H, data: _DataSource | None = None, *, input: _AlphaInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.line_alpha_columns_from_data(data, input, column_key, other_keys)
        return self

    @overload
    def line_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def line_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def line_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.line_alpha_columns_map_from_data(
            data, input, column_key, other_keys,
            linear, log10, threshold, continuous, _UNSET,
        )
        return self

    def line_alpha_columns_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *, input: _CategoryInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.line_alpha_columns_map_categories_from_data(
            data, input, column_key, other_keys, mapper
        )
        return self

    def line_alpha_theme(self: _H) -> _H:
        self._owner.line_alpha_theme()
        return self


class _FillAlpha(_Handle):
    __slots__ = ()

    def fill_alpha(self: _H, input: _AlphaInput) -> _H:
        self._owner.fill_alpha(input)
        return self

    def fill_alpha_cells(self: _H, input: _AlphaInput) -> _H:
        self._owner.fill_alpha_cells(input)
        return self

    def fill_alpha_linear(
        self: _H, input: _NumericInput,
        domain: tuple[float, float], range: tuple[int, int],
    ) -> _H:
        self._owner.fill_alpha_linear(input, domain, range)
        return self

    @overload
    def fill_alpha_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_map(input, linear, log10, threshold, continuous, _UNSET)
        return self

    def fill_alpha_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_map_categories(input, mapper)
        return self

    @overload
    def fill_alpha_cells_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_cells_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_cells_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_cells_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_cells_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_cells_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_cells_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_cells_map_categories(input, mapper)
        return self

    @overload
    def fill_alpha_single(self: _H, *, value: int) -> _H: ...

    @overload
    def fill_alpha_single(self: _H, *, input: _AlphaInput) -> _H: ...

    def fill_alpha_single(
        self: _H, *, value: int = cast(int, _UNSET),
        input: _AlphaInput = cast("_AlphaInput", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_single(value, input, _UNSET)
        return self

    @overload
    def fill_alpha_single_map(
        self: _H, input: _NumericInput, *, linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_single_map(
        self: _H, input: _NumericInput, *, log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_single_map(
        self: _H, input: _NumericInput, *, threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_single_map(
        self: _H, input: _NumericInput, *, continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_single_map(
        self: _H, input: _NumericInput, *,
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_single_map(
            input, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_single_map_categories(
        self: _H, input: _CategoryInput, *, mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_single_map_categories(input, mapper)
        return self

    def fill_alpha_keyed(
        self: _H, input: _AlphaInput, *, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.fill_alpha_keyed(input, keys)
        return self

    @overload
    def fill_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map(
        self: _H, input: _NumericInput, *,
        keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_keyed_map(
        self: _H, input: _NumericInput, *, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_keyed_map(
            input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_keyed_map_categories(
        self: _H, input: _CategoryInput, *,
        keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_keyed_map_categories(input, keys, mapper)
        return self

    def fill_alpha_keyed_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _AlphaInput, keys: _AlphaKeys[_K],
    ) -> _H:
        self._owner.fill_alpha_keyed_from_data(data, input, keys)
        return self

    @overload
    def fill_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K], continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_keyed_map_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _NumericInput, keys: _AlphaKeys[_K],
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_keyed_map_from_data(
            data, input, keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_keyed_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *,
        input: _CategoryInput, keys: _AlphaKeys[_K], mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_keyed_map_categories_from_data(data, input, keys, mapper)
        return self

    def fill_alpha_columns(
        self: _H, input: _AlphaInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.fill_alpha_columns(input, column_key, other_keys)
        return self

    @overload
    def fill_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_columns_map(
        self: _H, input: _NumericInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_columns_map(
            input, column_key, other_keys, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_columns_map_categories(
        self: _H, input: _CategoryInput, *,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_columns_map_categories(
            input, column_key, other_keys, mapper
        )
        return self

    def fill_alpha_columns_from_data(
        self: _H, data: _DataSource | None = None, *, input: _AlphaInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
    ) -> _H:
        self._owner.fill_alpha_columns_from_data(data, input, column_key, other_keys)
        return self

    @overload
    def fill_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> _H: ...

    @overload
    def fill_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> _H: ...

    def fill_alpha_columns_map_from_data(
        self: _H, data: _DataSource | None = None, *, input: _NumericInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> _H:
        self._owner.fill_alpha_columns_map_from_data(
            data, input, column_key, other_keys,
            linear, log10, threshold, continuous, _UNSET,
        )
        return self

    def fill_alpha_columns_map_categories_from_data(
        self: _H, data: _DataSource | None = None, *, input: _CategoryInput,
        column_key: _CategoryInput | _IdInput, other_keys: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> _H:
        self._owner.fill_alpha_columns_map_categories_from_data(
            data, input, column_key, other_keys, mapper
        )
        return self

    def fill_alpha_theme(self: _H) -> _H:
        self._owner.fill_alpha_theme()
        return self


class Scatter(
    _NumericX, _NumericY, _Hue, _Facet, _Guides, _Annotations, _Outline,
    _CategoricalScale, _PointAlpha, Plot,
):
    """Numeric Scatter with exact native capabilities."""
    __slots__ = ()


class Histogram(
    _NumericX, _NumericY, _Facet, _Guides, _Annotations, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    @overload
    def bins(self) -> Histogram: ...

    @overload
    def bins(self, *, count: int) -> Histogram: ...

    @overload
    def bins(self, *, width: float) -> Histogram: ...

    def bins(
        self, *, count: int = cast(int, _UNSET),
        width: float = cast(float, _UNSET),
    ) -> Histogram:
        self._owner.bins(count, width, _UNSET)
        return self

    def stat(self, stat: Literal["count", "probability", "density"]) -> Histogram:
        self._owner.stat(stat)
        return self

    def composite(
        self, composite: Literal["histogram_only", "kde_and_rug"]
    ) -> Histogram:
        self._owner.composite(composite)
        return self

    def composite_precomputed_kde(
        self, *, x: _NumericInput, y: _NumericInput
    ) -> Histogram:
        self._owner.composite_precomputed_kde(x, y)
        return self

    def composite_precomputed_kde_from_data(
        self, data: _DataSource | None, *, x: _NumericInput, y: _NumericInput
    ) -> Histogram:
        self._owner.composite_precomputed_kde_from_data(data, x, y)
        return self

    def fill_alpha_bins(
        self, input: _AlphaInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (),
    ) -> Histogram:
        self._owner.fill_alpha_bins(input, lower, upper, facets)
        return self

    @overload
    def fill_alpha_bins_map(
        self, input: _NumericInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (), linear: _AlphaLinear,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map(
        self, input: _NumericInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (), log10: _AlphaLinear,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map(
        self, input: _NumericInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (), threshold: _AlphaThreshold,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map(
        self, input: _NumericInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (), continuous: _AlphaContinuous,
    ) -> Histogram: ...

    def fill_alpha_bins_map(
        self, input: _NumericInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> Histogram:
        self._owner.fill_alpha_bins_map(
            input, lower, upper, facets, linear, log10, threshold, continuous, _UNSET
        )
        return self

    def fill_alpha_bins_map_categories(
        self, input: _CategoryInput, *, lower: _NumericInput, upper: _NumericInput,
        facets: _AlphaKeys[_K] = (), mapper: Callable[[str], int],
    ) -> Histogram:
        self._owner.fill_alpha_bins_map_categories(input, lower, upper, facets, mapper)
        return self

    def fill_alpha_bins_from_data(
        self, data: _DataSource | None = None, *, input: _AlphaInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
    ) -> Histogram:
        self._owner.fill_alpha_bins_from_data(data, input, lower, upper, facets)
        return self

    @overload
    def fill_alpha_bins_map_from_data(
        self, data: _DataSource | None = None, *, input: _NumericInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        linear: _AlphaLinear,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map_from_data(
        self, data: _DataSource | None = None, *, input: _NumericInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        log10: _AlphaLinear,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map_from_data(
        self, data: _DataSource | None = None, *, input: _NumericInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        threshold: _AlphaThreshold,
    ) -> Histogram: ...

    @overload
    def fill_alpha_bins_map_from_data(
        self, data: _DataSource | None = None, *, input: _NumericInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        continuous: _AlphaContinuous,
    ) -> Histogram: ...

    def fill_alpha_bins_map_from_data(
        self, data: _DataSource | None = None, *, input: _NumericInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        linear: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        log10: _AlphaLinear = cast("_AlphaLinear", _UNSET),
        threshold: _AlphaThreshold = cast("_AlphaThreshold", _UNSET),
        continuous: _AlphaContinuous = cast("_AlphaContinuous", _UNSET),
    ) -> Histogram:
        self._owner.fill_alpha_bins_map_from_data(
            data, input, lower, upper, facets,
            linear, log10, threshold, continuous, _UNSET,
        )
        return self

    def fill_alpha_bins_map_categories_from_data(
        self, data: _DataSource | None = None, *, input: _CategoryInput,
        lower: _NumericInput, upper: _NumericInput, facets: _AlphaKeys[_K] = (),
        mapper: Callable[[str], int],
    ) -> Histogram:
        self._owner.fill_alpha_bins_map_categories_from_data(
            data, input, lower, upper, facets, mapper
        )
        return self


class Density(
    _NumericX, _NumericY, _Hue, _Group, _Facet, _Guides, _Annotations,
    _Outline, _CategoricalScale, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()


class Qq(
    _NumericX, _NumericY, _Facet, _Guides, _Annotations, _Outline,
    _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()

    def reference_line(self, line: Literal["quartiles", "identity", "hidden"]) -> Qq:
        self._owner.qq_reference_line(line)
        return self


class PointInterval(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _Outline,
    _CategoryOrder, _ZeroLine, _EstimateDetails, _EstimateStatusMethods,
    _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()


class Line(
    _NumericX, _NumericY, _Hue, _Group, _Facet, _Guides, _Annotations, _Outline,
    _CategoricalScale, _Markers, _PointAlpha, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    def band(
        self, lower: _NumericInput | _NumericRectangle,
        upper: _NumericInput | _NumericRectangle,
    ) -> Line:
        self._owner.line_band(lower, upper)
        return self


class Trajectory(
    _NumericX, _NumericY, _CategoricalX, _Hue, _Facet, _Guides, _Annotations,
    _Outline, _CategoricalScale, _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()


class LinePath(
    _NumericX, _NumericY, _Hue, _Facet, _Guides, _Annotations, _Outline,
    _CategoricalScale, _Markers, _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()

    def style(self, style: Literal["neutral", "hue"]) -> LinePath:
        self._owner.style(style)
        return self


class Interaction(
    _NumericY, _CategoricalX, _Hue, _Facet, _Guides, _Annotations, _Outline,
    _CategoricalScale, _Markers, _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()

    def connectors(self, visible: bool) -> Interaction:
        self._owner.connectors(visible)
        return self

    def styles(self, entries: Sequence[InteractionStyle]) -> Interaction:
        self._owner.styles(entries)
        return self


class Coefficient(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _Outline,
    _EstimateDetails, _EstimateStatusMethods, _SignificanceStyle,
    _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()

    def reference_line(self, value: float) -> Coefficient:
        self._owner.coefficient_reference_line(value)
        return self

    def estimate_labels(self, format: NumericFormat) -> Coefficient:
        self._owner.estimate_labels(format)
        return self


class Forest(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _Outline,
    _EstimateDetails, _EstimateStatusMethods, _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()

    def scale(self, scale: Literal["linear", "log10"]) -> Forest:
        self._owner.scale(scale)
        return self


class Dumbbell(
    _NumericX, _CategoricalY, _Group, _Facet, _Guides, _Annotations, _Outline,
    _PointAlpha, _LineAlpha, Plot,
):
    __slots__ = ()


class Heatmap(
    _CategoricalX, _CategoricalY, _Facet, _Annotations, _ContinuousColor, _FillAlpha, Plot,
):
    __slots__ = ()

    @overload
    def layout(self, layout: Literal["cartesian"]) -> Heatmap: ...

    @overload
    def layout(
        self, layout: Literal["lower_triangle", "upper_triangle"], *, diagonal: bool
    ) -> Heatmap: ...

    def layout(
        self, layout: Literal["cartesian", "lower_triangle", "upper_triangle"],
        *, diagonal: bool = cast(bool, _UNSET),
    ) -> Heatmap:
        self._owner.layout(layout, diagonal, _UNSET)
        return self

    def annotations(self, annotations: NumericFormat | Literal["hidden"]) -> Heatmap:
        self._owner.annotations(annotations)
        return self

    def x_category_order(self, order: _HeatmapOrder) -> Heatmap:
        self._owner.x_category_order(order)
        return self

    def y_category_order(self, order: _HeatmapOrder) -> Heatmap:
        self._owner.y_category_order(order)
        return self

    def x_labels(self, labels: list[str] | tuple[str, ...]) -> Heatmap:
        self._owner.x_labels(labels)
        return self

    def y_labels(self, labels: list[str] | tuple[str, ...]) -> Heatmap:
        self._owner.y_labels(labels)
        return self

    def row_labels(self, values: _CategoryInput) -> Heatmap:
        self._owner.row_labels(values)
        return self

    def heatmap_highlight(self, values: _BooleanInput) -> Heatmap:
        self._owner.heatmap_highlight(values)
        return self

    def heatmap_highlight_cells(self, values: _BooleanInput) -> Heatmap:
        self._owner.heatmap_highlight_cells(values)
        return self

    def value_text_layout(self, layout: TextLayout) -> Heatmap:
        self._owner.value_text_layout(layout)
        return self

    def null_row_text_layout(self, layout: TextLayout) -> Heatmap:
        self._owner.null_row_text_layout(layout)
        return self

    def heatmap_null_row_annotation(
        self, row_label: str, text: TextInput, *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Heatmap:
        self._owner.heatmap_null_row_annotation(row_label, text, style, layout, _UNSET)
        return self


class Bar(
    _NumericY, _CategoricalX, _SeriesRole, _Facet, _Guides, _Annotations,
    _CategoryOrder, _CategoricalScale, _SignificanceStyle, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    def spacing(self, spacing: Literal["comfort", "compact"]) -> Bar:
        self._owner.spacing(spacing)
        return self


class HorizontalBar(
    _NumericX, _CategoricalY, _SeriesRole, _Facet, _Guides, _Annotations,
    _CategoryOrder, _CategoricalScale, _SignificanceStyle, _ZeroLine,
    _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    def spacing(self, spacing: Literal["comfort", "compact"]) -> HorizontalBar:
        self._owner.spacing(spacing)
        return self

    def color_by_sign(self) -> HorizontalBar:
        self._owner.color_by_sign()
        return self


class StackedBar(
    _NumericY, _CategoricalX, _Facet, _Guides, _Annotations, _CategoryOrder,
    _CategoricalScale, _SignificanceStyle, _FillAlpha, Plot,
):
    __slots__ = ()


class StackedHorizontalBar(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _CategoryOrder,
    _CategoricalScale, _SignificanceStyle, _ZeroLine, _FillAlpha, Plot,
):
    __slots__ = ()


class Boxplot(
    _NumericY, _CategoricalX, _SeriesRole, _Facet, _Guides, _Annotations,
    _CategoricalScale, _SignificanceStyle, _Outline, _RawOverlay,
    _PointAlpha, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()


class Violin(
    _NumericY, _CategoricalX, _SeriesRole, _Facet, _Guides, _Annotations,
    _CategoricalScale, _SignificanceStyle, _Outline, _RawOverlay,
    _PointAlpha, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()


class Raincloud(
    _NumericY, _CategoricalX, _SeriesRole, _Facet, _Guides, _Annotations,
    _CategoricalScale, _SignificanceStyle, _Outline,
    _PointAlpha, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    def hue_label(
        self, text: TextInput, *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Raincloud:
        self._owner.hue_label(text, style, layout, _UNSET)
        return self

    def clear_hue_label_style(self) -> Raincloud:
        self._owner.clear_hue_label_style()
        return self

    def clear_hue_label_layout(self) -> Raincloud:
        self._owner.clear_hue_label_layout()
        return self


class Regression(
    _NumericX, _NumericY, _Facet, _Guides, _Annotations, _Outline,
    _PointAlpha, _LineAlpha, _FillAlpha, Plot,
):
    __slots__ = ()

    def band(self, band: Literal["pointwise_mean_95", "hidden"]) -> Regression:
        self._owner.regression_band(band)
        return self


class Marginal(
    _NumericX, _NumericY, _Hue, _Group, _Guides, _Annotations, _Outline,
    _CategoricalScale, _PointAlpha, _FillAlpha, Plot,
):
    __slots__ = ()


class Matrix(_Hue, _Group, _Outline, _CategoricalScale, _PointAlpha, _FillAlpha, Plot):
    __slots__ = ()

    def matrix_row_text_layout(self, layout: TextLayout) -> Matrix:
        self._owner.matrix_row_text_layout(layout)
        return self

    def matrix_column_text_layout(self, layout: TextLayout) -> Matrix:
        self._owner.matrix_column_text_layout(layout)
        return self


class Beeswarm(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _Outline,
    _ContinuousColor, _ZeroLine, _PointAlpha, Plot,
):
    __slots__ = ()


class Waterfall(
    _NumericX, _CategoricalY, _Facet, _Guides, _Annotations, _FillAlpha, Plot,
):
    __slots__ = ()

    def total_labels(self, base: TextInput, terminal: TextInput) -> Waterfall:
        self._owner.total_labels(base, terminal, _UNSET)
        return self

    def total_format(self, format: NumericFormat) -> Waterfall:
        self._owner.total_format(format)
        return self

    def total_text_layout(self, layout: TextLayout) -> Waterfall:
        self._owner.total_text_layout(layout)
        return self


@overload
def scatter(
    data: _DataSource | None = None,
    *,
    x: _NumericInput,
    y: _NumericInput | _NumericRectangle,
    hue: _CategoryInput | None = None,
    by_row_position: Literal[False] = False,
    numeric_only: Literal[False] = False,
) -> Scatter: ...


@overload
def scatter(
    data: _DataSource | None = None,
    *,
    y: _NumericRectangle,
    by_row_position: Literal[True],
    hue: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Scatter: ...


@overload
def scatter(
    data: _DataSource,
    *,
    by_row_position: Literal[True],
    numeric_only: Literal[True],
    hue: _CategoryInput | None = None,
) -> Scatter: ...


def scatter(
    data: _DataSource | None = None,
    *,
    x: _NumericInput = cast("_NumericInput", _UNSET),
    y: _NumericInput | _NumericRectangle = cast(
        "_NumericInput | _NumericRectangle", _UNSET
    ),
    hue: _CategoryInput | None = None,
    by_row_position: bool = False,
    numeric_only: bool = False,
) -> Scatter:
    """Bind direct values or source columns; native code validates the call form."""
    return Scatter._from_owner(
        _native._PlotOwner.scatter(
            data,
            x=x,
            y=y,
            hue=hue,
            by_row_position=by_row_position,
            numeric_only=numeric_only,
            unset=_UNSET,
        )
    )


@overload
def histogram(
    data: _DataSource | None = None, *, samples: _NumericInput,
) -> Histogram: ...


@overload
def histogram(
    data: _DataSource | None = None, *,
    bin_lower: _NumericInput, bin_upper: _NumericInput, bin_density: _NumericInput,
    kde_x: _NumericInput, kde_y: _NumericInput,
    rug_x: _NumericInput, rug_baseline: _NumericInput, rug_top: _NumericInput,
    kde_data: _DataSource | None = None, rug_data: _DataSource | None = None,
    value_label: TextInput = ..., density_label: TextInput = ...,
) -> Histogram: ...


def histogram(
    data: _DataSource | None = None, *,
    samples: _NumericInput = cast("_NumericInput", _UNSET),
    bin_lower: _NumericInput = cast("_NumericInput", _UNSET),
    bin_upper: _NumericInput = cast("_NumericInput", _UNSET),
    bin_density: _NumericInput = cast("_NumericInput", _UNSET),
    kde_x: _NumericInput = cast("_NumericInput", _UNSET),
    kde_y: _NumericInput = cast("_NumericInput", _UNSET),
    rug_x: _NumericInput = cast("_NumericInput", _UNSET),
    rug_baseline: _NumericInput = cast("_NumericInput", _UNSET),
    rug_top: _NumericInput = cast("_NumericInput", _UNSET),
    kde_data: _DataSource | None = None, rug_data: _DataSource | None = None,
    value_label: TextInput = cast(TextInput, _UNSET),
    density_label: TextInput = cast(TextInput, _UNSET),
) -> Histogram:
    return Histogram._from_owner(_native._PlotOwner.histogram(
        data, samples=samples, bin_lower=bin_lower, bin_upper=bin_upper,
        bin_density=bin_density, kde_x=kde_x, kde_y=kde_y, rug_x=rug_x,
        rug_baseline=rug_baseline, rug_top=rug_top, kde_data=kde_data,
        rug_data=rug_data, value_label=value_label, density_label=density_label,
        unset=_UNSET,
    ))


def density(
    data: _DataSource | None = None, *, value: _NumericInput,
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
) -> Density:
    return Density._from_owner(_native._PlotOwner.density(
        data, value=value, hue=hue, group=group,
    ))


def qq(data: _DataSource | None = None, *, value: _NumericInput) -> Qq:
    return Qq._from_owner(_native._PlotOwner.qq(data, value=value))


def point_interval(
    data: _DataSource | None = None, *, category: _CategoryInput,
    estimate: _NumericInput, lower: _NumericInput, upper: _NumericInput,
) -> PointInterval:
    return PointInterval._from_owner(_native._PlotOwner.point_interval(
        data, category=category, estimate=estimate, lower=lower, upper=upper,
    ))


@overload
def line(
    data: _DataSource | None = None, *, x: _NumericInput,
    y: _NumericInput | _NumericRectangle,
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
    by_row_position: Literal[False] = False,
    numeric_only: Literal[False] = False,
) -> Line: ...


@overload
def line(
    data: _DataSource | None = None, *, y: _NumericRectangle,
    by_row_position: Literal[True],
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Line: ...


@overload
def line(
    data: _DataSource, *, by_row_position: Literal[True],
    numeric_only: Literal[True],
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
) -> Line: ...


def line(
    data: _DataSource | None = None, *,
    x: _NumericInput = cast("_NumericInput", _UNSET),
    y: _NumericInput | _NumericRectangle = cast("_NumericInput | _NumericRectangle", _UNSET),
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
    by_row_position: bool = False, numeric_only: bool = False,
) -> Line:
    return Line._from_owner(_native._PlotOwner.line(
        data, x=x, y=y, hue=hue, group=group, by_row_position=by_row_position,
        numeric_only=numeric_only, unset=_UNSET,
    ))


def trajectory(
    data: _DataSource | None = None, *, x: _NumericInput | _CategoryInput,
    y: _NumericInput, group: _CategoryInput, hue: _CategoryInput | None = None,
) -> Trajectory:
    return Trajectory._from_owner(_native._PlotOwner.trajectory(
        data, x=x, y=y, group=group, hue=hue,
    ))


def line_path(
    data: _DataSource | None = None, *, x: _NumericInput, y: _NumericInput,
    group: _CategoryInput, hue: _CategoryInput | None = None,
) -> LinePath:
    return LinePath._from_owner(_native._PlotOwner.line_path(
        data, x=x, y=y, group=group, hue=hue,
    ))


def interaction(
    data: _DataSource | None = None, *, condition: _CategoryInput,
    estimate: _NumericInput, group: _CategoryInput,
    lower: _NumericInput, upper: _NumericInput, hue: _CategoryInput | None = None,
) -> Interaction:
    return Interaction._from_owner(_native._PlotOwner.interaction(
        data, condition=condition, estimate=estimate, group=group,
        lower=lower, upper=upper, hue=hue,
    ))


@overload
def coefficient(
    data: _DataSource | None = None, *, term: _CategoryInput,
    estimate: _NumericInput,
) -> Coefficient: ...


@overload
def coefficient(
    data: _DataSource | None = None, *, term: _CategoryInput,
    estimate: _NumericInput, lower: _NumericInput, upper: _NumericInput,
) -> Coefficient: ...


def coefficient(
    data: _DataSource | None = None, *, term: _CategoryInput,
    estimate: _NumericInput,
    lower: _NumericInput = cast("_NumericInput", _UNSET),
    upper: _NumericInput = cast("_NumericInput", _UNSET),
) -> Coefficient:
    return Coefficient._from_owner(_native._PlotOwner.coefficient(
        data, term=term, estimate=estimate, lower=lower, upper=upper, unset=_UNSET,
    ))


def forest(
    data: _DataSource | None = None, *, term: _CategoryInput,
    estimate: _NumericInput, lower: _NumericInput, upper: _NumericInput,
    reference: float,
) -> Forest:
    return Forest._from_owner(_native._PlotOwner.forest(
        data, term=term, estimate=estimate, lower=lower, upper=upper,
        reference=reference,
    ))


def dumbbell(
    data: _DataSource | None = None, *, category: _CategoryInput,
    left: _NumericInput, right: _NumericInput,
    left_label: str, left_color: Color, right_label: str, right_color: Color,
    group: _CategoryInput | None = None,
) -> Dumbbell:
    return Dumbbell._from_owner(_native._PlotOwner.dumbbell(
        data, category=category, left=left, right=right,
        left_label=left_label, left_color=left_color,
        right_label=right_label, right_color=right_color, group=group,
    ))


@overload
def heatmap(
    data: _DataSource | None = None, *, grid: _NumericRectangle,
) -> Heatmap: ...


@overload
def heatmap(
    data: _DataSource | None = None, *, x: _CategoryInput,
    y: _CategoryInput, fill: _NumericInput,
) -> Heatmap: ...


def heatmap(
    data: _DataSource | None = None, *,
    grid: _NumericRectangle = cast("_NumericRectangle", _UNSET),
    x: _CategoryInput = cast("_CategoryInput", _UNSET),
    y: _CategoryInput = cast("_CategoryInput", _UNSET),
    fill: _NumericInput = cast("_NumericInput", _UNSET),
) -> Heatmap:
    return Heatmap._from_owner(_native._PlotOwner.heatmap(
        data, grid=grid, x=x, y=y, fill=fill, unset=_UNSET,
    ))


@overload
def bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    samples: _NumericInput, error_bar: _ErrorBar = ...,
    series: _CategoryInput | None = None,
) -> Bar: ...


@overload
def bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    estimate: _NumericInput, series: _CategoryInput | None = None,
) -> Bar: ...


@overload
def bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    estimate: _NumericInput, lower: _NumericInput, upper: _NumericInput,
    series: _CategoryInput | None = None,
) -> Bar: ...


def bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    samples: _NumericInput = cast("_NumericInput", _UNSET),
    estimate: _NumericInput = cast("_NumericInput", _UNSET),
    lower: _NumericInput = cast("_NumericInput", _UNSET),
    upper: _NumericInput = cast("_NumericInput", _UNSET),
    error_bar: _ErrorBar = cast(_ErrorBar, _UNSET),
    series: _CategoryInput | None = None,
) -> Bar:
    return Bar._from_owner(_native._PlotOwner.bar(
        data, category=category, samples=samples, estimate=estimate, lower=lower,
        upper=upper, error_bar=error_bar, series=series, unset=_UNSET,
    ))


@overload
def horizontal_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    samples: _NumericInput, error_bar: _ErrorBar = ...,
    series: _CategoryInput | None = None,
) -> HorizontalBar: ...


@overload
def horizontal_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    estimate: _NumericInput, series: _CategoryInput | None = None,
) -> HorizontalBar: ...


@overload
def horizontal_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    estimate: _NumericInput, lower: _NumericInput, upper: _NumericInput,
    series: _CategoryInput | None = None,
) -> HorizontalBar: ...


def horizontal_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    samples: _NumericInput = cast("_NumericInput", _UNSET),
    estimate: _NumericInput = cast("_NumericInput", _UNSET),
    lower: _NumericInput = cast("_NumericInput", _UNSET),
    upper: _NumericInput = cast("_NumericInput", _UNSET),
    error_bar: _ErrorBar = cast(_ErrorBar, _UNSET),
    series: _CategoryInput | None = None,
) -> HorizontalBar:
    return HorizontalBar._from_owner(_native._PlotOwner.horizontal_bar(
        data, category=category, samples=samples, estimate=estimate, lower=lower,
        upper=upper, error_bar=error_bar, series=series, unset=_UNSET,
    ))


def stacked_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput, series: _CategoryInput,
) -> StackedBar:
    return StackedBar._from_owner(_native._PlotOwner.stacked_bar(
        data, category=category, value=value, series=series,
    ))


def stacked_horizontal_bar(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput, series: _CategoryInput,
) -> StackedHorizontalBar:
    return StackedHorizontalBar._from_owner(_native._PlotOwner.stacked_horizontal_bar(
        data, category=category, value=value, series=series,
    ))


@overload
def boxplot(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput | _NumericRectangle,
    series: _CategoryInput | None = None, by_column_name: Literal[False] = False,
    numeric_only: Literal[False] = False,
) -> Boxplot: ...


@overload
def boxplot(
    data: _DataSource | None = None, *, value: _NumericRectangle,
    by_column_name: Literal[True], series: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Boxplot: ...


@overload
def boxplot(
    data: _DataSource, *, by_column_name: Literal[True], numeric_only: Literal[True],
    series: _CategoryInput | None = None,
) -> Boxplot: ...


def boxplot(
    data: _DataSource | None = None, *,
    category: _CategoryInput = cast("_CategoryInput", _UNSET),
    value: _NumericInput | _NumericRectangle = cast("_NumericInput | _NumericRectangle", _UNSET),
    series: _CategoryInput | None = None, by_column_name: bool = False,
    numeric_only: bool = False,
) -> Boxplot:
    return Boxplot._from_owner(_native._PlotOwner.boxplot(
        data, category=category, value=value, series=series,
        by_column_name=by_column_name, numeric_only=numeric_only, unset=_UNSET,
    ))


@overload
def violin(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput | _NumericRectangle,
    series: _CategoryInput | None = None, by_column_name: Literal[False] = False,
    numeric_only: Literal[False] = False,
) -> Violin: ...


@overload
def violin(
    data: _DataSource | None = None, *, value: _NumericRectangle,
    by_column_name: Literal[True], series: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Violin: ...


@overload
def violin(
    data: _DataSource, *, by_column_name: Literal[True], numeric_only: Literal[True],
    series: _CategoryInput | None = None,
) -> Violin: ...


def violin(
    data: _DataSource | None = None, *,
    category: _CategoryInput = cast("_CategoryInput", _UNSET),
    value: _NumericInput | _NumericRectangle = cast("_NumericInput | _NumericRectangle", _UNSET),
    series: _CategoryInput | None = None, by_column_name: bool = False,
    numeric_only: bool = False,
) -> Violin:
    return Violin._from_owner(_native._PlotOwner.violin(
        data, category=category, value=value, series=series,
        by_column_name=by_column_name, numeric_only=numeric_only, unset=_UNSET,
    ))


@overload
def raincloud(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput | _NumericRectangle,
    series: _CategoryInput | None = None, by_column_name: Literal[False] = False,
    numeric_only: Literal[False] = False,
) -> Raincloud: ...


@overload
def raincloud(
    data: _DataSource | None = None, *, value: _NumericRectangle,
    by_column_name: Literal[True], series: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Raincloud: ...


@overload
def raincloud(
    data: _DataSource, *, by_column_name: Literal[True], numeric_only: Literal[True],
    series: _CategoryInput | None = None,
) -> Raincloud: ...


@overload
def raincloud(
    data: _DataSource | None = None, *, category: _CategoryInput,
    q1: _NumericInput, median: _NumericInput, q3: _NumericInput,
    density_category: _CategoryInput, density_y: _NumericInput,
    normalized_width: _NumericInput, point_category: _CategoryInput,
    point_y: _NumericInput, normalized_strip: _NumericInput,
    density_data: _DataSource | None = None, point_data: _DataSource | None = None,
    hue: None = None, density_hue: None = None, point_hue: None = None,
    category_label: TextInput = ..., value_label: TextInput = ...,
    hue_label: TextInput = ...,
) -> Raincloud: ...


@overload
def raincloud(
    data: _DataSource | None = None, *, category: _CategoryInput,
    q1: _NumericInput, median: _NumericInput, q3: _NumericInput,
    density_category: _CategoryInput, density_y: _NumericInput,
    normalized_width: _NumericInput, point_category: _CategoryInput,
    point_y: _NumericInput, normalized_strip: _NumericInput,
    hue: _CategoryInput, density_hue: _CategoryInput, point_hue: _CategoryInput,
    density_data: _DataSource | None = None, point_data: _DataSource | None = None,
    category_label: TextInput = ..., value_label: TextInput = ...,
    hue_label: TextInput = ...,
) -> Raincloud: ...


def raincloud(
    data: _DataSource | None = None, *,
    category: _CategoryInput = cast("_CategoryInput", _UNSET),
    value: _NumericInput | _NumericRectangle = cast("_NumericInput | _NumericRectangle", _UNSET),
    series: _CategoryInput | None = None, by_column_name: bool = False,
    numeric_only: bool = False,
    q1: _NumericInput = cast("_NumericInput", _UNSET),
    median: _NumericInput = cast("_NumericInput", _UNSET),
    q3: _NumericInput = cast("_NumericInput", _UNSET),
    density_category: _CategoryInput = cast("_CategoryInput", _UNSET),
    density_y: _NumericInput = cast("_NumericInput", _UNSET),
    normalized_width: _NumericInput = cast("_NumericInput", _UNSET),
    point_category: _CategoryInput = cast("_CategoryInput", _UNSET),
    point_y: _NumericInput = cast("_NumericInput", _UNSET),
    normalized_strip: _NumericInput = cast("_NumericInput", _UNSET),
    hue: _CategoryInput | None = None, density_hue: _CategoryInput | None = None,
    point_hue: _CategoryInput | None = None,
    density_data: _DataSource | None = None, point_data: _DataSource | None = None,
    category_label: TextInput = cast(TextInput, _UNSET),
    value_label: TextInput = cast(TextInput, _UNSET),
    hue_label: TextInput = cast(TextInput, _UNSET),
) -> Raincloud:
    return Raincloud._from_owner(_native._PlotOwner.raincloud(
        data, category=category, value=value, series=series,
        by_column_name=by_column_name, numeric_only=numeric_only, q1=q1,
        median=median, q3=q3, density_category=density_category,
        density_y=density_y, normalized_width=normalized_width,
        point_category=point_category, point_y=point_y, normalized_strip=normalized_strip,
        hue=hue, density_hue=density_hue, point_hue=point_hue,
        density_data=density_data, point_data=point_data,
        category_label=category_label, value_label=value_label, hue_label=hue_label,
        unset=_UNSET,
    ))


def regression(
    data: _DataSource | None = None, *, x: _NumericInput, y: _NumericInput,
) -> Regression:
    return Regression._from_owner(_native._PlotOwner.regression(data, x=x, y=y))


def marginal(
    data: _DataSource | None = None, *, x: _NumericInput, y: _NumericInput,
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
) -> Marginal:
    return Marginal._from_owner(_native._PlotOwner.marginal(
        data, x=x, y=y, hue=hue, group=group,
    ))


@overload
def matrix(
    data: _DataSource | None = None, *, variables: _NumericRectangle,
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
    numeric_only: Literal[False] = False,
) -> Matrix: ...


@overload
def matrix(
    data: _DataSource, *, numeric_only: Literal[True],
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
) -> Matrix: ...


def matrix(
    data: _DataSource | None = None, *,
    variables: _NumericRectangle = cast("_NumericRectangle", _UNSET),
    hue: _CategoryInput | None = None, group: _CategoryInput | None = None,
    numeric_only: bool = False,
) -> Matrix:
    return Matrix._from_owner(_native._PlotOwner.matrix(
        data, variables=variables, hue=hue, group=group,
        numeric_only=numeric_only, unset=_UNSET,
    ))


def beeswarm(
    data: _DataSource | None = None, *, category: _CategoryInput,
    value: _NumericInput, color: _NumericInput,
) -> Beeswarm:
    return Beeswarm._from_owner(_native._PlotOwner.beeswarm(
        data, category=category, value=value, color=color,
    ))


def waterfall(
    data: _DataSource | None = None, *, category: _CategoryInput,
    delta: _NumericInput, base: float,
) -> Waterfall:
    return Waterfall._from_owner(_native._PlotOwner.waterfall(
        data, category=category, delta=delta, base=base,
    ))


class Canvas:
    """A mutable native grid with insertion-time Plot captures."""

    __slots__ = ("_owner", "__weakref__")
    _owner: _native._CanvasOwner

    def __init__(self, _unavailable: NoReturn, /) -> None:
        raise TypeError("use Canvas.grid(rows, columns)")

    @staticmethod
    def _from_owner(owner: _native._CanvasOwner) -> Canvas:
        instance = object.__new__(Canvas)
        instance._owner = owner
        return instance

    @staticmethod
    def grid(rows: int, columns: int) -> Canvas:
        return Canvas._from_owner(_native._CanvasOwner.grid(rows, columns))

    def copy(self) -> Canvas:
        return Canvas._from_owner(self._owner.copy())

    def labels(self, labels: Labels) -> Canvas:
        self._owner.labels(labels)
        return self

    def panel(
        self,
        row: int,
        column: int,
        plot: Plot,
        *,
        row_span: int = 1,
        column_span: int = 1,
        overwrite: bool = False,
    ) -> Canvas:
        if not isinstance(plot, Plot):
            raise TypeError("panel requires a Plot")
        self._owner.panel(
            row,
            column,
            plot._owner,
            row_span=row_span,
            column_span=column_span,
            overwrite=overwrite,
        )
        return self

    def text_panel(
        self, row: int, column: int, *, title: TextInput, body: TextInput
    ) -> Canvas:
        self._owner.text_panel(row, column, title, body, _UNSET)
        return self

    def title(
        self,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.title(text, style, layout, _UNSET)
        return self

    def subtitle(
        self,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.subtitle(text, style, layout, _UNSET)
        return self

    def caption(
        self,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.caption(text, style, layout, _UNSET)
        return self

    def column_header(
        self,
        start_column: int,
        span: int,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.column_header(start_column, span, text, style, layout, _UNSET)
        return self

    def row_header(
        self,
        start_row: int,
        span: int,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.row_header(start_row, span, text, style, layout, _UNSET)
        return self

    def row_title(
        self,
        row: int,
        text: TextInput,
        *,
        style: TextTheme | None = cast(TextTheme | None, _UNSET),
        layout: TextLayout | None = cast(TextLayout | None, _UNSET),
    ) -> Canvas:
        self._owner.row_title(row, text, style, layout, _UNSET)
        return self

    def theme(self, theme: Theme) -> Canvas:
        self._owner.theme(theme)
        return self

    def width(self, width: int) -> Canvas:
        self._owner.width(width)
        return self

    def max_width(self, maximum: int) -> Canvas:
        self._owner.max_width(maximum)
        return self

    def size(self, width: int, height: int) -> Canvas:
        self._owner.size(width, height)
        return self

    def legend_policy(self, policy: Literal["per_plot", "shared", "hidden"]) -> Canvas:
        self._owner.legend_policy(policy)
        return self

    def legend_composition(self, composition: _Composition) -> Canvas:
        self._owner.legend_composition(composition)
        return self

    def guide_text_layout(self, role: _GuideRole, layout: TextLayout) -> Canvas:
        self._owner.guide_text_layout(role, layout)
        return self

    def panel_title_alignment(self, alignment: _Alignment) -> Canvas:
        self._owner.panel_title_alignment(alignment)
        return self

    def panel_subtitle_alignment(self, alignment: _Alignment) -> Canvas:
        self._owner.panel_subtitle_alignment(alignment)
        return self

    def category_labels(
        self, policy: Literal["every_plot", "first_column_per_row"]
    ) -> Canvas:
        self._owner.category_labels(policy)
        return self

    def x_range_group(self, cells: Sequence[tuple[int, int]]) -> Canvas:
        self._owner.x_range_group(cells)
        return self

    def clear_title_style(self) -> Canvas:
        self._owner.clear_title_style()
        return self

    def clear_title_layout(self) -> Canvas:
        self._owner.clear_title_layout()
        return self

    def clear_subtitle_style(self) -> Canvas:
        self._owner.clear_subtitle_style()
        return self

    def clear_subtitle_layout(self) -> Canvas:
        self._owner.clear_subtitle_layout()
        return self

    def clear_caption_style(self) -> Canvas:
        self._owner.clear_caption_style()
        return self

    def clear_caption_layout(self) -> Canvas:
        self._owner.clear_caption_layout()
        return self

    def svg(self, path: PathInput) -> Output:
        return self._owner.svg(path)

    def svg_preserve_text(self, path: PathInput) -> Output:
        return self._owner.svg_preserve_text(path)

    def png(self, path: PathInput) -> Output:
        return self._owner.png(path)

    def png_preview(self, path: PathInput) -> Output:
        return self._owner.png_preview(path)

    def png_with_options(self, path: PathInput, options: PngOptions) -> Output:
        return self._owner.png_with_options(path, options)
