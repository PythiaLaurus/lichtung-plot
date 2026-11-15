"""Real native callbacks, scoped views, and operation-owned Python exceptions."""

from collections import UserDict
import gc
from pathlib import Path
import struct
import sys
import threading
import xml.etree.ElementTree as ET

import pytest

from lichtung_plot import (
    DataError,
    NumericField,
    NumericFormat,
    NumericLabels,
    NumericPrecision,
    RenderError,
    Text,
    TickGeneration,
    TickPositions,
    scatter,
)


SVG = "{http://www.w3.org/2000/svg}"


def figure():
    return (
        scatter(x=[0.0, 1.0, 2.0], y=[1.0, 2.0, 1.0])
        .size(1000, 700)
        .x_limits(0.0, 2.0)
        .y_ticks([(1.0, "low"), (2.0, "high")])
        .x_tick_generation(TickGeneration.exact_count(3))
    )


def text(path: Path) -> str:
    return "\n".join(
        "".join(node.itertext())
        for node in ET.parse(path).getroot().iter(f"{SVG}text")
    )


def test_callback_reentry_replaces_itself_without_changing_current_snapshot(
    tmp_path: Path,
) -> None:
    plot = figure().title("Before callback")
    replaced = False
    later = NumericFormat.set_function(
        lambda context: [f"NEW{index}" for index, _ in enumerate(context.values)]
    )

    def original(context):
        nonlocal replaced
        if not replaced:
            replaced = True
            assert plot.x_tick_format(later).title("After callback") is plot
            gc.collect()
        return NumericLabels(
            [f"OLD{index}" for index, _ in enumerate(context.values)],
            explanation=Text("Materialized explanation"),
        )

    plot.x_tick_format(NumericFormat.set_function(original))
    first = plot.svg_preserve_text(tmp_path / "current")
    assert replaced
    current = text(tmp_path / "current.svg")
    assert "Before callback" in current and "OLD0" in current
    assert "After callback" not in current and "NEW0" not in current
    plot.svg_preserve_text(tmp_path / "later")
    following = text(tmp_path / "later.svg")
    assert "After callback" in following and "NEW0" in following
    assert "OLD0" not in following
    first.svg_preserve_text(tmp_path / "old-output")
    assert text(tmp_path / "old-output.svg") == current


def test_original_exception_identity_traceback_and_chains_survive(
    tmp_path: Path,
) -> None:
    class ApplicationFailure(Exception):
        pass

    failure = ApplicationFailure("authored failure")
    cause = ValueError("authored cause")
    context = LookupError("authored context")
    failure.__cause__ = cause
    failure.__context__ = context
    original_tracebacks = []
    escaped = []

    def application_callback(view):
        escaped.append(view.format_builtin)
        try:
            raise failure
        except ApplicationFailure:
            original_tracebacks.append(failure.__traceback__)
            raise

    plot = figure().x_tick_format(NumericFormat.value_function(application_callback))
    with pytest.raises(ApplicationFailure) as caught:
        plot.svg_preserve_text(tmp_path / "refused")
    assert caught.value is failure
    assert failure.__cause__ is cause
    assert failure.__context__ is context
    chain = []
    traceback = failure.__traceback__
    while traceback is not None:
        chain.append(traceback)
        traceback = traceback.tb_next
    assert original_tracebacks[0] in chain
    assert not (tmp_path / "refused.svg").exists()
    with pytest.raises(RuntimeError, match="no longer active"):
        escaped[0]()

    plot.x_tick_format(NumericFormat())
    plot.svg_preserve_text(tmp_path / "recovered")
    assert (tmp_path / "recovered.svg").is_file()


def test_result_extraction_preserves_its_python_exception(tmp_path: Path) -> None:
    failure = ArithmeticError("conversion refused")

    class Position:
        def __float__(self):
            raise failure

    plot = figure().x_tick_generation(TickGeneration.function(lambda _: [Position()]))
    with pytest.raises(ArithmeticError) as caught:
        plot.svg_preserve_text(tmp_path / "bad-conversion")
    assert caught.value is failure

    length_failure = LookupError("result length refused")

    class Result:
        def __len__(self):
            raise length_failure

        def __getitem__(self, index):
            raise IndexError(index)

    plot.x_tick_generation(TickGeneration.exact_count(3))
    plot.x_tick_format(NumericFormat.set_function(lambda _: Result()))
    with pytest.raises(LookupError) as caught_length:
        plot.svg_preserve_text(tmp_path / "bad-length")
    assert caught_length.value is length_failure

    # A wrong final string is a boundary extraction error, not RenderError.
    plot.x_tick_generation(TickGeneration.exact_count(3))
    plot.x_tick_format(NumericFormat.value_function(lambda _: object()))
    with pytest.raises(TypeError) as caught_type:
        plot.svg_preserve_text(tmp_path / "bad-string")
    assert not isinstance(caught_type.value, RenderError)
    plot.x_tick_format(NumericFormat())
    plot.svg_preserve_text(tmp_path / "after-conversion-errors")


@pytest.mark.parametrize("role", ["set", "tick"], ids=["set-result", "tick-result"])
def test_context_stays_active_through_deferred_result_extraction(
    tmp_path: Path, role
) -> None:
    """Two module-integration contracts: set and tick result extraction scopes."""
    results = []
    bound_reads = []

    class Position:
        def __init__(self, result, index):
            self.result = result
            self.index = index

        def __float__(self):
            lower, upper = self.result.context.domain.scale_limits
            assert lower <= self.index <= upper
            self.result.phases.add("float")
            return float(self.index)

    class Result:
        def __init__(self, context):
            self.context = context
            self.phases = set()

        def __len__(self):
            if role == "set":
                values = self.context.values
                assert self.context.owner.plot_kind == "scatter"
                count = len(values)
            else:
                assert self.context.axis == "x"
                lower, upper = self.context.domain.scale_limits
                assert lower <= 0.0 <= 2.0 <= upper
                count = 3
            self.phases.add("len")
            return count

        def __getitem__(self, index):
            if role == "set":
                values = self.context.values
                if index >= len(values):
                    raise IndexError(index)
                assert values[index].identity.kind in ("tick", "probe")
                self.phases.add("getitem")
                return f"label{index}"
            assert self.context.axis == "x"
            lower, upper = self.context.domain.scale_limits
            assert lower <= 0.0 <= 2.0 <= upper
            if index >= 3:
                raise IndexError(index)
            self.phases.add("getitem")
            position = Position(self, index)
            bound_reads.append((position.__float__, ()))
            return position

    def deferred(context):
        # No context reads or result extraction here: return the custom sequence
        # for the binding to consume only after this callback body has returned.
        result = Result(context)
        results.append(result)
        bound_reads.extend([(result.__len__, ()), (result.__getitem__, (0,))])
        return result

    plot = figure()
    if role == "set":
        plot.x_tick_format(NumericFormat.set_function(deferred))
    else:
        plot.x_tick_generation(TickGeneration.function(deferred))
    plot.svg_preserve_text(tmp_path / f"deferred-{role}")
    assert (tmp_path / f"deferred-{role}.svg").is_file()
    assert results
    required_phases = {"len", "getitem", "float"} if role == "tick" else {"len", "getitem"}
    for result in results:
        assert required_phases <= result.phases
    # Keep and check every reached context and every saved element method, not
    # merely the final context or a newly manufactured post-render wrapper.
    for method, arguments in bound_reads:
        with pytest.raises(RuntimeError, match="no longer active"):
            method(*arguments)


def test_nested_failure_does_not_poison_outer_and_outer_context_stays_active(
    tmp_path: Path,
) -> None:
    failure = LookupError("inner operation")
    inner = figure()
    entered = False
    outer_views = []

    def inner_callback(view):
        # The outer token is still active although it is not the stack top.
        assert outer_views[-1].index >= 0
        assert isinstance(outer_views[-1].format_builtin(), str)
        assert view.set.owner.plot_kind == "scatter"
        raise failure

    inner.x_tick_format(NumericFormat.value_function(inner_callback))

    def outer_callback(view):
        nonlocal entered
        outer_views.append(view)
        before = view.format_builtin()
        if not entered:
            entered = True
            with pytest.raises(LookupError) as caught:
                inner.svg_preserve_text(tmp_path / "inner")
            assert caught.value is failure
            assert view.format_builtin() == before
        return before

    outer = figure().x_tick_format(NumericFormat.value_function(outer_callback))
    outer.svg_preserve_text(tmp_path / "outer")
    assert entered and (tmp_path / "outer.svg").is_file()
    with pytest.raises(RuntimeError, match="no longer active"):
        outer_views[0].format_builtin()
    outer.svg_preserve_text(tmp_path / "outer-again")


def test_native_helper_is_truly_lazy_and_caught_error_is_not_pending(
    tmp_path: Path,
) -> None:
    # usize::MAX cannot be a real output field. Native callback metadata itself
    # is valid: only explicitly requesting the built-in helper allocates a field.
    enormous_width = sys.maxsize * 2 + 1
    ignored = NumericFormat.value_function(
        lambda view: str(view.index), field=NumericField(enormous_width)
    )
    plot = figure().x_tick_format(ignored)
    plot.svg_preserve_text(tmp_path / "ignored-helper")

    caught_helpers = []

    def catch_helper(view):
        try:
            view.format_builtin()
        except RenderError as error:
            caught_helpers.append(error)
        return str(view.index)

    plot.x_tick_format(ignored.copy_with(value_function=catch_helper))
    plot.svg_preserve_text(tmp_path / "caught-helper")
    assert caught_helpers

    escaped_errors = []

    def request_helper(view):
        try:
            return view.format_builtin()
        except RenderError as error:
            escaped_errors.append(error)
            raise

    plot.x_tick_format(ignored.copy_with(value_function=request_helper))
    with pytest.raises(RenderError) as caught:
        plot.svg_preserve_text(tmp_path / "requested-helper")
    assert caught.value is escaped_errors[0]
    plot.x_tick_format(NumericFormat())
    plot.svg_preserve_text(tmp_path / "after-helper-error")


def test_scoped_nested_views_bound_methods_threads_and_nonreused_keys(
    tmp_path: Path,
) -> None:
    saved = []
    methods = []
    thread_errors = []
    old_checked_in_new_scope = False

    def inspect(view):
        nonlocal old_checked_in_new_scope
        if saved:
            # An old key must not become valid when a later context occupies the
            # same stack depth/address, including within the same operation.
            with pytest.raises(RuntimeError, match="no longer active"):
                getattr(saved[0][0], saved[0][1])
            old_checked_in_new_scope = True
        enclosing = view.set
        assert enclosing.role == "x_axis"
        assert enclosing.purpose in ("intrinsic_probe", "actual_set")
        assert enclosing.owner.plot_kind == "scatter"
        assert enclosing.owner.canvas_cell is None
        assert not enclosing.owner.is_shared
        assert enclosing.owner.facet_cell is None
        assert view.index < len(enclosing.values)
        assert view.value.operand.kind == "binary64"
        assert view.value.operand.binary64 is not None
        assert view.value.identity.kind in ("tick", "probe")
        assert view.factors.multiplier == 1.0
        assert view.factors.decimal_scale == 0
        assert view.factors.rate == "value"
        divisor = view.factors.divisor
        assert divisor.kind in ("value", "power_of_ten")
        if divisor.kind == "value":
            assert divisor.value == 1.0
        else:
            assert divisor.exponent == "0"
        assert view.factors.unit is None
        assert enclosing.locale.decimal == "."
        assert enclosing.locale.digits == tuple("0123456789")
        assert enclosing.locale.grouping.sizes == (3,)
        assert enclosing.locale.symbols.negative == "-"
        assert enclosing.locale.currency.prefix == "$"
        complete_builtin = enclosing.format_builtin()
        if enclosing.purpose == "actual_set":
            assert complete_builtin.labels == ("0.00", "1.00", "2.00")
        if enclosing.domain is not None:
            assert enclosing.domain.scale == "linear"
            assert enclosing.domain.lower.kind == "binary64"
            assert enclosing.domain.scale_limits[0] <= enclosing.domain.scale_limits[1]
        if enclosing.common_factors is not None:
            assert enclosing.common_factors.rate == "value"

        if not saved:
            saved.extend(
                [
                    (view, "index"),
                    (enclosing, "purpose"),
                    (enclosing.owner, "plot_kind"),
                    (view.value, "operand"),
                    (view.value.operand, "binary64"),
                    (view.value.identity, "kind"),
                    (view.factors, "rate"),
                    (view.factors.divisor, "kind"),
                    (enclosing.locale, "decimal"),
                    (enclosing.locale.grouping, "sizes"),
                    (enclosing.locale.symbols, "positive"),
                    (enclosing.locale.currency, "prefix"),
                ]
            )
            if enclosing.domain is not None:
                saved.append((enclosing.domain, "scale_limits"))
            methods.extend([view.format_builtin, enclosing.format_builtin])

            def other_thread():
                try:
                    methods[0]()
                except BaseException as error:
                    thread_errors.append(error)

            worker = threading.Thread(target=other_thread)
            worker.start()
            worker.join()
            assert len(thread_errors) == 1
            assert isinstance(thread_errors[0], RuntimeError)
        return view.format_builtin()

    plot = figure().x_tick_format(NumericFormat.value_function(inspect))
    plot.svg_preserve_text(tmp_path / "view-scope")
    for value, attribute in saved:
        with pytest.raises(RuntimeError, match="no longer active"):
            getattr(value, attribute)
    for method in methods:
        with pytest.raises(RuntimeError, match="no longer active"):
            method()
    plot.svg_preserve_text(tmp_path / "fresh-view-scope")
    assert old_checked_in_new_scope


def test_native_domains_are_not_reclassified_as_python_callback_failures(
    tmp_path: Path,
) -> None:
    plot = figure().x_tick_format(NumericFormat.set_function(lambda _: []))
    with pytest.raises(DataError):
        plot.svg_preserve_text(tmp_path / "cardinality")
    plot.x_tick_format(NumericFormat.value_function(lambda _: "tab\ttext"))
    with pytest.raises(DataError):
        plot.svg_preserve_text(tmp_path / "native-text")
    plot.x_tick_format(NumericFormat())
    plot.x_tick_generation(
        TickGeneration.function(lambda _: TickPositions.log10_exponents([0.0, 1.0]))
    )
    with pytest.raises(DataError):
        plot.svg_preserve_text(tmp_path / "linear-exponents")
    plot.x_tick_generation(TickGeneration.function(lambda _: [float("nan")]))
    with pytest.raises(RenderError):
        plot.svg_preserve_text(tmp_path / "unprojectable")
    plot.x_tick_generation(TickGeneration.exact_count(3))
    plot.svg_preserve_text(tmp_path / "after-domain-errors")


def test_exact_ticks_suppress_callbacks_in_both_setter_orders(tmp_path: Path) -> None:
    def forbidden(_):
        raise AssertionError("exact ticks must suppress generated callback work")

    locator = TickGeneration.function(forbidden)
    formatter = NumericFormat.value_function(forbidden)
    exact = [(0.0, "zero"), (1.0, "one"), (2.0, "two")]
    first = figure().x_ticks(exact).x_tick_generation(locator).x_tick_format(formatter)
    second = figure().x_tick_format(formatter).x_tick_generation(locator).x_ticks(exact)
    first.svg_preserve_text(tmp_path / "exact-first")
    second.svg_preserve_text(tmp_path / "exact-last")
    assert text(tmp_path / "exact-first.svg") == text(tmp_path / "exact-last.svg")


def test_reached_contexts_new_terminals_frozen_replay_and_io_failure(
    tmp_path: Path,
) -> None:
    prefix = "FIRST"
    tick_views = []
    numeric_views = []
    escaped_tick_views = []

    def positions(view):
        key = (
            view.axis,
            view.owner.plot_kind,
            view.owner.canvas_cell,
            view.domain.scale,
            view.domain.origin,
            view.domain.data_limits,
            view.domain.scale_limits,
            view.span,
        )
        assert key not in tick_views
        tick_views.append(key)
        escaped_tick_views.append(view)
        return TickPositions.data([0.0, 1.0, 2.0])

    def labels(view):
        key = (
            view.purpose,
            view.owner.canvas_cell,
            view.domain.scale_limits if view.domain is not None else None,
            tuple(
                (
                    value.operand.binary64,
                    value.identity.kind,
                    value.identity.source_index,
                    value.identity.index,
                )
                for value in view.values
            ),
        )
        assert key not in numeric_views
        numeric_views.append(key)
        return [f"{prefix}{index}" for index, _ in enumerate(view.values)]

    plot = (
        figure()
        .x_tick_generation(TickGeneration.function(positions))
        .x_tick_format(NumericFormat.set_function(labels))
    )
    output = plot.svg_preserve_text(tmp_path / "first")
    assert tick_views and numeric_views
    with pytest.raises(RuntimeError, match="no longer active"):
        _ = escaped_tick_views[0].span
    assert "FIRST0" in text(tmp_path / "first.svg")
    prior_count = (len(tick_views), len(numeric_views))
    with pytest.raises(OSError):
        output.svg_preserve_text(tmp_path / "missing-parent" / "output")
    output.svg_preserve_text(tmp_path / "replay")
    output.png_preview(tmp_path / "replay-preview")
    assert (len(tick_views), len(numeric_views)) == prior_count
    png = (tmp_path / "replay-preview.png").read_bytes()
    assert png[:8] == b"\x89PNG\r\n\x1a\n"
    assert struct.unpack(">II", png[16:24]) == (output.width, output.height)

    prefix = "SECOND"
    tick_views.clear()
    numeric_views.clear()
    plot.svg_preserve_text(tmp_path / "second")
    assert tick_views and numeric_views
    assert "SECOND0" in text(tmp_path / "second.svg")
    assert "FIRST0" in text(tmp_path / "replay.svg")

    # A new terminal does callback work even if publication subsequently fails.
    tick_views.clear()
    numeric_views.clear()
    with pytest.raises(OSError):
        plot.svg_preserve_text(tmp_path / "missing-parent" / "fresh")
    assert tick_views and numeric_views
    tick_views.clear()
    numeric_views.clear()
    plot.svg_preserve_text(tmp_path / "after-io-failure")
    assert tick_views and numeric_views


def test_callback_operation_is_fresh_on_each_python_render_thread(tmp_path: Path) -> None:
    caller_ids = set()
    callback_ids = set()
    errors = []
    barrier = threading.Barrier(2)

    def labels(view):
        callback_ids.add(threading.get_ident())
        return view.format_builtin()

    plot = figure().x_tick_format(NumericFormat.value_function(labels))

    def render(name):
        caller_ids.add(threading.get_ident())
        barrier.wait()
        try:
            plot.svg_preserve_text(tmp_path / name)
        except BaseException as error:
            errors.append(error)

    first = threading.Thread(target=render, args=("thread-one",))
    second = threading.Thread(target=render, args=("thread-two",))
    first.start()
    second.start()
    first.join()
    second.join()
    assert not errors
    assert len(caller_ids) == 2 and callback_ids == caller_ids
    assert (tmp_path / "thread-one.svg").is_file()
    assert (tmp_path / "thread-two.svg").is_file()


def test_numeric_value_derivation_keeps_native_policy_and_callback_identity(
    tmp_path: Path,
) -> None:
    original = NumericFormat.value_function(lambda view: view.format_builtin())
    derived = original.copy_with(precision=NumericPrecision.fractional(1))
    assert original.precision.digits == 2 and derived.precision.digits == 1
    with pytest.raises(AttributeError):
        derived.precision = NumericPrecision.fractional(3)
    with pytest.raises(DataError):
        original.copy_with(precision=NumericPrecision.fractional(255))
    with pytest.raises(TypeError):
        original.copy_with(value_function=None, set_function=None)
    figure().x_tick_format(derived).svg_preserve_text(tmp_path / "derived-format")
    figure().x_tick_format(original.copy_with(value_function=None)).svg_preserve_text(
        tmp_path / "native-emitter"
    )

    locator = TickGeneration.function(lambda _: [0.0, 1.0, 2.0])
    capped = locator.copy_with(maximum_count=3)
    assert locator.maximum_count is None and capped.maximum_count == 3
    figure().x_tick_generation(capped).svg_preserve_text(tmp_path / "derived-locator")
    positions = TickPositions.log10_exponents([0.0, 1.0])
    changed = positions.copy_with(positions=[2.0])
    assert positions.positions == (0.0, 1.0)
    assert changed.kind == "log10_exponents" and changed.positions == (2.0,)
    labels = NumericLabels(["a"], explanation=Text("Authored"))
    plain = labels.copy_with(explanation=None)
    assert labels.explanation.source == "Authored"
    assert plain.labels == ("a",) and plain.explanation is None

    # Mapping keys, unordered sets, and generator-only inputs are not sequence data.
    with pytest.raises(TypeError):
        NumericLabels({"key": "not a label sequence"})

    class DictSubclass(dict):
        pass

    with pytest.raises(TypeError):
        NumericLabels(DictSubclass({"key": "not a label sequence"}))
    with pytest.raises(TypeError):
        NumericLabels(UserDict({"key": "not a label sequence"}))
    with pytest.raises(TypeError):
        TickPositions.data({0.0, 1.0})
    with pytest.raises(TypeError):
        TickPositions.data(value for value in [0.0, 1.0])

    protocol_failure = LookupError("sequence protocol lookup refused")

    class ProtocolLookup(type):
        def __getattribute__(cls, name):
            if name == "__getitem__":
                raise protocol_failure
            return super().__getattribute__(name)

    class ProtocolResult(metaclass=ProtocolLookup):
        def __len__(self):
            return 1

        def __getitem__(self, index):
            if index == 0:
                return "label"
            raise IndexError(index)

    with pytest.raises(LookupError) as caught_protocol:
        NumericLabels(ProtocolResult())
    assert caught_protocol.value is protocol_failure
