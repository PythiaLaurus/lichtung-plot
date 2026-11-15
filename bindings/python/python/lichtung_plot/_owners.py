"""Thin mutable handles; all scientific and style state lives in the extension."""

from __future__ import annotations

from os import PathLike
from typing import (
    TYPE_CHECKING,
    Literal,
    TypeAlias,
    TypeVar,
    cast,
    overload,
)

from . import _native
from ._native import (
    ColumnSelector,
    ColumnsSelector,
    Labels,
    NumericFormat,
    Output,
    PngOptions,
    Text,
    TextLayout,
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
    _SourceRowsInput: TypeAlias = (
        _NativeRowIds
        | _NativeRowIdRectangle
        | _Array
        | _Series
        | _Table
        | _SingleColumn
        | ColumnsSelector
    )

ExactTicks: TypeAlias = list[tuple[float, str]] | tuple[tuple[float, str], ...]
PathInput: TypeAlias = str | bytes | PathLike[str] | PathLike[bytes]
TextInput: TypeAlias = str | Text

_P = TypeVar("_P", bound="Plot")
_UNSET = object()


class Plot:
    """Common configuration and output operations of exact plot families."""

    __slots__ = ("_owner", "__weakref__")
    _owner: _native._PlotOwner

    def __init__(self) -> None:
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


class Scatter(Plot):
    """Numeric Scatter with its exact hue and numeric-axis capabilities."""

    __slots__ = ()

    def hue(self, values: _CategoryInput) -> Scatter:
        self._owner.hue(values)
        return self

    def x_limits(self, minimum: float, maximum: float) -> Scatter:
        self._owner.x_limits(minimum, maximum)
        return self

    def y_limits(self, minimum: float, maximum: float) -> Scatter:
        self._owner.y_limits(minimum, maximum)
        return self

    def x_ticks(self, ticks: ExactTicks) -> Scatter:
        self._owner.x_ticks(ticks)
        return self

    def y_ticks(self, ticks: ExactTicks) -> Scatter:
        self._owner.y_ticks(ticks)
        return self

    def x_tick_format(self, format: NumericFormat) -> Scatter:
        self._owner.x_tick_format(format)
        return self

    def y_tick_format(self, format: NumericFormat) -> Scatter:
        self._owner.y_tick_format(format)
        return self

    def x_tick_generation(self, generation: TickGeneration) -> Scatter:
        self._owner.x_tick_generation(generation)
        return self

    def y_tick_generation(self, generation: TickGeneration) -> Scatter:
        self._owner.y_tick_generation(generation)
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


class Canvas:
    """A mutable native grid with insertion-time Plot captures."""

    __slots__ = ("_owner", "__weakref__")
    _owner: _native._CanvasOwner

    def __init__(self) -> None:
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
