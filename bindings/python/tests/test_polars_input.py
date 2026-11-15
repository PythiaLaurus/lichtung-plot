"""Real installed-wheel Scatter consumers of eager Polars public C streams.

No import shadowing, optional skips, debug FFI API or PyArrow/NumPy transport.
Private Rust tests additionally inspect logical dtypes and unused Enum levels:
native plot legends intentionally use observed labels, not the foreign schema.
"""

from decimal import Decimal
import gc
from pathlib import Path
import weakref
import xml.etree.ElementTree as ET

import polars as pl
import pytest

from lichtung_plot import DataError, Scatter, TickGeneration, TickPositions, scatter


SVG = "{http://www.w3.org/2000/svg}"
PRIMITIVES = [
    pl.Int8,
    pl.Int16,
    pl.Int32,
    pl.Int64,
    pl.Int128,
    pl.UInt8,
    pl.UInt16,
    pl.UInt32,
    pl.UInt64,
    pl.UInt128,
    pl.Float16,
    pl.Float32,
    pl.Float64,
]


def svg_text(path: Path) -> str:
    return "\n".join(
        "".join(node.itertext())
        for node in ET.parse(path).getroot().iter(f"{SVG}text")
    )


@pytest.mark.parametrize("dtype", PRIMITIVES, ids=[str(dtype) for dtype in PRIMITIVES])
def test_thirteen_primitives_really_export_into_public_scatter(dtype, monkeypatch):
    exported = []
    original = pl.Series.__arrow_c_stream__

    def recorded(series, *args, **kwargs):
        exported.append((series.name, series.dtype))
        # Still the genuine public native producer, returning a fresh capsule.
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", recorded)
    plot = scatter(
        x=pl.Series("Exposure", [0, None, 2], dtype=dtype),
        y=pl.Series("Response", [2, 3, 1], dtype=dtype),
    )
    assert isinstance(plot, Scatter)
    assert sorted(name for name, _ in exported) == ["Exposure", "Response"]
    assert all(logical == dtype for _, logical in exported)
    # Do not multiply full publications by dtype. Binding performs the actual
    # import; the Rust fidelity test inspects every dtype/value/missing shape.


@pytest.mark.parametrize(
    ("dtype", "values"),
    [(pl.Int128, []), (pl.Float16, [None, None])],
    ids=["typed-empty-i128", "typed-all-null-f16"],
)
def test_typed_missing_series_bind_before_native_no_rows_failure(
    dtype, values, tmp_path: Path
):
    plot = scatter(
        x=pl.Series("Exposure", values, dtype=dtype),
        y=pl.Series("Response", [1.0] * len(values), dtype=pl.Float64),
    )
    with pytest.raises(DataError, match="no rows"):
        plot.svg(tmp_path / "no-observations")
    assert not (tmp_path / "no-observations.svg").exists()


def test_chunked_slice_snapshot_survives_series_release_and_keeps_aligned_domain(
    tmp_path: Path,
):
    x = pl.concat(
        [
            pl.Series("Exposure", [-7.0, None, float("nan"), -0.0]),
            pl.Series("Exposure", [0.0, 2.5, None, 9.0]),
        ],
        rechunk=False,
    ).slice(1, 6)
    y = pl.Series("Response", [0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
    assert x.n_chunks() >= 2
    x_ref, y_ref = weakref.ref(x), weakref.ref(y)
    plot = scatter(x=x, y=y)
    del x, y
    gc.collect()
    assert x_ref() is None and y_ref() is None

    native = scatter(
        x=[None, float("nan"), -0.0, +0.0, 2.5, None],
        y=[0, 1, 2, 3, 4, 5],
    ).x_label("Exposure").y_label("Response")

    def publish(current, name):
        metadata = []

        def positions(view):
            domain = view.domain
            metadata.append(
                (
                    view.axis,
                    view.owner.plot_kind,
                    view.owner.canvas_cell,
                    domain.scale,
                    domain.origin,
                    domain.data_limits,
                    domain.scale_limits,
                    view.span,
                )
            )
            return TickPositions.data(
                [0.0, 1.0, 2.5] if view.axis == "x" else [2.0, 3.0, 4.0]
            )

        current.size(640, 480)
        current.x_tick_generation(TickGeneration.function(positions))
        current.y_tick_generation(TickGeneration.function(positions))
        current.svg_preserve_text(tmp_path / name)
        return metadata

    domains = publish(plot, "slice")
    native_domains = publish(native, "slice-native-reference")
    assert domains
    assert {axis for axis, *_ in domains} == {"x", "y"}
    # Compare current padded domains, not raw extrema or callback repetition.
    assert set(domains) == set(native_domains)
    for axis, _, _, _, _, limits, _, _ in domains:
        assert limits is not None
        aligned = (-0.0, +0.0, 2.5) if axis == "x" else (2.0, 3.0, 4.0)
        assert all(limits[0] <= value <= limits[1] for value in aligned)
    text = svg_text(tmp_path / "slice.svg")
    assert "Exposure" in text and "Response" in text
    # Null-vs-NaN and signed-zero bits are checked privately before the native
    # Scatter policy intentionally drops missing/NaN coordinate observations.


@pytest.mark.parametrize("kind", ["categorical-u8", "categorical-u16", "enum"])
def test_chunked_categorical_and_enum_hue_reach_the_real_consumer(
    kind, tmp_path: Path
):
    if kind == "enum":
        dtype = pl.Enum(["unused level", "beta", "alpha"])
    else:
        physical = pl.UInt8 if kind == "categorical-u8" else pl.UInt16
        dtype = pl.Categorical(
            pl.Categories(f"named;{kind}", "binding;ffi", physical=physical)
        )
    hue = pl.concat(
        [
            pl.Series("Treatment", ["alpha", "beta"], dtype=dtype),
            pl.Series("Treatment", ["beta", "alpha"], dtype=dtype),
        ],
        rechunk=False,
    ).slice(1, 3)
    assert hue.n_chunks() >= 2
    plot = scatter(
        x=pl.Series("Exposure", [0.0, 1.0, 2.0]),
        y=pl.Series("Response", [2.0, 1.0, 3.0]),
        hue=hue,
    )
    plot.svg_preserve_text(tmp_path / "categories")
    # Display humanization is a native label policy, not transport fidelity.
    text = svg_text(tmp_path / "categories.svg").casefold()
    assert "alpha" in text and "beta" in text
    assert "unused level" not in text


def test_uint64_ids_are_not_rounded_and_uint128_overflow_is_atomic(tmp_path: Path):
    plot = scatter(
        x=pl.Series("Exposure", [0.0, float("inf")]),
        y=pl.Series("Response", [2.0, 3.0]),
    )
    # These become duplicates if an importer rounds through binary64 first.
    exact = pl.Series("identity", [2**53, 2**53 + 1], dtype=pl.UInt64)
    assert plot.source_rows(exact) is plot
    for invalid in [
        pl.Series("identity", [2**53 + 1, 2**53 + 1], dtype=pl.UInt64),
        pl.Series("identity", [2**100, 1], dtype=pl.UInt128),
    ]:
        with pytest.raises(DataError):
            plot.source_rows(invalid)
    # After both rejected replacements, the original native diagnostic identity
    # must still be the exact second ID, not the positional slot or a rounded ID.
    with pytest.raises(DataError, match=rf"source row {2**53 + 1}\b"):
        plot.svg(tmp_path / "exact-diagnostic")
    assert not (tmp_path / "exact-diagnostic.svg").exists()


def test_source_exports_only_bound_fields_and_later_lookup_does_not_refresh_xy(
    monkeypatch, tmp_path: Path
):
    from lichtung_plot import Labels

    source = pl.DataFrame(
        [
            pl.Series("Exposure", [0.0, 1.0, 2.0]),
            pl.Series("Response", [2.0, 1.0, 3.0]),
            pl.Series("Treatment", ["alpha", "beta", "alpha"]),
            pl.Series("unused_decimal", [Decimal("1.25")] * 3, dtype=pl.Decimal(10, 2)),
            pl.Series("unused_struct", [{"nested": 1}] * 3),
        ]
    )
    exported = []
    original = pl.Series.__arrow_c_stream__

    def recorded(series, *args, **kwargs):
        exported.append(series.name)
        assert series.name not in {"unused_decimal", "unused_struct"}
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", recorded)
    plot = scatter(source, x="Exposure", y="Response", hue="Treatment")
    assert sorted(exported) == ["Exposure", "Response", "Treatment"]
    source.replace_column(0, pl.Series("Exposure", [90.0, 91.0, 92.0]))
    source.replace_column(
        2, pl.Series("Treatment", ["later one", "later two", "later one"])
    )
    assert plot.hue("Treatment") is plot
    with pytest.raises(TypeError, match="unsupported Polars logical dtype"):
        plot.hue("unused_decimal")
    assert sorted(exported) == ["Exposure", "Response", "Treatment", "Treatment"]

    # The reference uses the old coordinate snapshot and the newly bound hue.
    native = (
        scatter(
            x=[0, 1, 2],
            y=[2, 1, 3],
            hue=["later one", "later two", "later one"],
        )
        .labels(Labels(aliases=[("hue", "Treatment")]))
        .x_label("Exposure")
        .y_label("Response")
    )

    def publish(current, name):
        metadata = []

        def positions(view):
            domain = view.domain
            metadata.append(
                (
                    view.axis,
                    view.owner.plot_kind,
                    view.owner.canvas_cell,
                    domain.scale,
                    domain.origin,
                    domain.data_limits,
                    domain.scale_limits,
                    view.span,
                )
            )
            return TickPositions.data(
                [0.0, 1.0, 2.0] if view.axis == "x" else [1.0, 2.0, 3.0]
            )

        current.size(640, 480)
        current.x_tick_generation(TickGeneration.function(positions))
        current.y_tick_generation(TickGeneration.function(positions))
        current.svg_preserve_text(tmp_path / name)
        return metadata

    domains = publish(plot, "bound-only")
    native_domains = publish(native, "bound-only-native-reference")
    assert domains
    assert {axis for axis, *_ in domains} == {"x", "y"}
    assert set(domains) == set(native_domains)
    for axis, _, _, _, _, limits, _, _ in domains:
        assert limits is not None
        aligned = (0, 1, 2) if axis == "x" else (2, 1, 3)
        assert all(limits[0] <= value <= limits[1] for value in aligned)
    assert sorted(exported) == ["Exposure", "Response", "Treatment", "Treatment"]
    text = svg_text(tmp_path / "bound-only.svg").casefold()
    assert "later" in text and "alpha" not in text and "beta" not in text


@pytest.mark.parametrize(
    "dtype",
    [pl.Date, pl.Decimal(20, 2), pl.List(pl.Int64), pl.Object],
    ids=["date", "decimal", "list", "object"],
)
def test_bound_unsupported_empty_logical_dtype_is_rejected_before_export(
    dtype, monkeypatch
):
    unsupported = pl.Series("Unsupported", [], dtype=dtype)
    y = pl.Series("Response", [], dtype=pl.Float64)
    original = pl.Series.__arrow_c_stream__

    def guarded(series, *args, **kwargs):
        if series.name == "Unsupported":
            raise AssertionError("unsupported logical dtype reached the C producer")
        # Do not prescribe the order of independent scalar role imports.
        return original(series, *args, **kwargs)

    monkeypatch.setattr(pl.Series, "__arrow_c_stream__", guarded)
    with pytest.raises(TypeError, match="unsupported Polars logical dtype"):
        scatter(x=unsupported, y=y)


@pytest.mark.parametrize("attribute", ["name", "dtype", "__len__", "__arrow_c_stream__"])
def test_original_metadata_and_export_errors_reenter_without_an_owner_borrow(
    attribute, monkeypatch, tmp_path: Path
):
    class ProducerFailure(Exception):
        pass

    failure = ProducerFailure("original producer error")
    source = pl.Series("Treatment", ["new one", "new two"])
    plot = scatter(x=[0.0, 1.0], y=[2.0, 3.0], hue=["old one", "old two"])

    def fail(*_args, **_kwargs):
        assert plot.title("Reentered safely") is plot
        raise failure

    replacement = property(fail) if attribute in {"name", "dtype"} else fail
    with monkeypatch.context() as patch:
        patch.setattr(pl.Series, attribute, replacement)
        with pytest.raises(ProducerFailure) as caught:
            plot.hue(source)
        assert caught.value is failure
    plot.svg_preserve_text(tmp_path / "preserved")
    text = svg_text(tmp_path / "preserved.svg").casefold()
    assert "reentered safely" in text and "old" in text and "new" not in text


def test_arrow_duck_is_not_a_fifth_input_family():
    class ArrowDuck:
        def __arrow_c_stream__(self, *_args, **_kwargs):
            raise AssertionError("a duck producer must never be called")

    with pytest.raises(TypeError):
        scatter(x=ArrowDuck(), y=[1.0])
