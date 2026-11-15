"""GC edges follow physical Python references, including native captures."""

import ctypes
import gc
from pathlib import Path
import weakref

import pytest

from lichtung_plot import Canvas, DataError, NumericFormat, TickGeneration, scatter


def clear_gc_object(value) -> None:
    """Invoke the actual CPython tp_clear, retaining the GIL throughout.

    PEP 442 normally calls Python finalizers *before* cycle clearing. Invoking
    this stable-ABI slot explicitly exercises DECREF finalizer reentry during
    tp_clear rather than relying on incidental collector finalizer order.
    """
    get_slot = ctypes.pythonapi.PyType_GetSlot
    get_slot.argtypes = (ctypes.py_object, ctypes.c_int)
    get_slot.restype = ctypes.c_void_p
    pointer = get_slot(type(value), 51)  # Stable-ABI Py_tp_clear.
    assert pointer
    clear = ctypes.PYFUNCTYPE(ctypes.c_int, ctypes.py_object)(pointer)
    assert clear(value) == 0


def figure():
    return scatter(x=[0.0, 1.0, 2.0], y=[1.0, 2.0, 1.0]).size(1000, 700)


class Formatter:
    def __init__(self, plot):
        self.plot = plot

    def __call__(self, context):
        return context.format_builtin()


def test_cycle_sharing_copy_capture_and_overwrite_retained_branch(
    tmp_path: Path,
) -> None:
    plot = figure()
    callback = Formatter(plot)
    format = NumericFormat.value_function(callback)
    same_emitter = format.copy_with()
    plot.x_tick_format(format)
    branch = plot.copy()
    canvas = Canvas.grid(1, 1).size(1000, 700).panel(0, 0, plot)
    retained_canvas = canvas.copy()
    plot_ref, callback_ref = weakref.ref(plot), weakref.ref(callback)
    format_ref = weakref.ref(format)

    # Each value has one real Py<holder>, both pointing to the unique holder.
    first_holders = [
        value
        for value in gc.get_referents(format)
        if type(value).__name__ == "_CallbackHolder"
    ]
    second_holders = [
        value
        for value in gc.get_referents(same_emitter)
        if type(value).__name__ == "_CallbackHolder"
    ]
    assert len(first_holders) == len(second_holders) == 1
    assert first_holders[0] is second_holders[0]
    # A holder visits the callable exactly once, independent of native Arc clones.
    assert sum(value is callback for value in gc.get_referents(first_holders[0])) == 1
    del first_holders, second_holders
    del plot, callback, format, same_emitter
    gc.collect()
    assert format_ref() is None
    assert plot_ref() is not None and callback_ref() is not None
    frozen = branch.svg_preserve_text(tmp_path / "copy-keeps-callback")
    del branch

    canvas.panel(0, 0, figure(), overwrite=True)
    gc.collect()
    assert plot_ref() is not None and callback_ref() is not None
    retained_canvas.svg_preserve_text(tmp_path / "retained-capture")
    del retained_canvas
    gc.collect()
    assert plot_ref() is None and callback_ref() is None
    frozen.svg_preserve_text(tmp_path / "output-without-callback-roots")
    canvas.svg_preserve_text(tmp_path / "overwritten-capture")


def test_failed_placement_drops_incoming_roots_without_losing_owner(
    tmp_path: Path,
) -> None:
    canvas = Canvas.grid(1, 1).panel(0, 0, figure())

    def rejected():
        plot = figure()
        callback = Formatter(plot)
        plot.x_tick_format(NumericFormat.value_function(callback))
        references = weakref.ref(plot), weakref.ref(callback)
        with pytest.raises(DataError):
            canvas.panel(0, 0, plot)
        return references

    references = rejected()
    gc.collect()
    assert all(reference() is None for reference in references)
    canvas.svg_preserve_text(tmp_path / "unchanged-after-rejection")


def test_tick_policy_cycles_use_the_same_gc_topology() -> None:
    def cycle():
        plot = figure()

        class Locator:
            def __init__(self):
                self.plot = plot

            def __call__(self, _):
                return [0.0, 1.0, 2.0]

        callback = Locator()
        generation = TickGeneration.function(callback)
        plot.x_tick_generation(generation)
        return weakref.ref(plot), weakref.ref(callback), weakref.ref(generation)

    references = cycle()
    gc.collect()
    assert all(reference() is None for reference in references)


def test_holder_clear_releases_mutex_and_pyborrow_before_callable_finalizer() -> None:
    events = []

    class Finalizer:
        target = None

        def __call__(self, _):
            return "label"

        def __del__(self):
            try:
                clear_gc_object(self.target)
                events.append("holder reentered")
            except BaseException as error:
                events.append(error)

    callback = Finalizer()
    format = NumericFormat.value_function(callback)
    holder = next(
        value
        for value in gc.get_referents(format)
        if type(value).__name__ == "_CallbackHolder"
    )
    callback.target = holder
    callback_ref = weakref.ref(callback)
    del callback
    clear_gc_object(holder)
    assert events == ["holder reentered"]
    assert callback_ref() is None
    clear_gc_object(holder)
    with pytest.raises(RuntimeError, match="cleared"):
        format.copy_with()


def test_numeric_clear_finalizer_reentry_observes_cleared_value_not_pyborrow() -> None:
    events = []

    class Finalizer:
        target = None

        def __call__(self, _):
            return "label"

        def __del__(self):
            try:
                self.target.copy_with()
            except BaseException as error:
                events.append(error)
            clear_gc_object(self.target)

    callback = Finalizer()
    format = NumericFormat.value_function(callback)
    callback.target = format
    del callback
    clear_gc_object(format)
    assert len(events) == 1
    assert isinstance(events[0], RuntimeError) and "cleared" in str(events[0])
    clear_gc_object(format)
    with pytest.raises(RuntimeError, match="cleared"):
        _ = format.fractional_digits


def test_owner_clear_finalizers_reenter_after_native_and_roots_are_taken() -> None:
    events = []

    class Finalizer:
        def __init__(self, target):
            self.target = target

        def __call__(self, _):
            return "label"

        def __del__(self):
            try:
                self.target.title("must not replace a cleared owner")
            except BaseException as error:
                events.append(error)
            clear_gc_object(self.target._owner)

    plot = figure()
    plot.x_tick_format(NumericFormat.value_function(Finalizer(plot)))
    clear_gc_object(plot._owner)
    assert len(events) == 1
    assert isinstance(events[0], RuntimeError) and "cleared" in str(events[0])
    with pytest.raises(RuntimeError, match="cleared"):
        plot.copy()
    with pytest.raises(TypeError):
        plot.hue([True])
    with pytest.raises(RuntimeError, match="cleared"):
        plot.hue(["a", "a", "b"])

    canvas = Canvas.grid(1, 1)
    temporary = figure().x_tick_format(NumericFormat.value_function(Finalizer(canvas)))
    canvas.panel(0, 0, temporary)
    del temporary
    clear_gc_object(canvas._owner)
    assert len(events) == 2
    assert isinstance(events[1], RuntimeError) and "cleared" in str(events[1])
    with pytest.raises(RuntimeError, match="cleared"):
        canvas.copy()


def test_operation_releases_last_pin_before_returning_original_extraction_error(
    tmp_path: Path,
) -> None:
    events = []
    plot = figure()

    class ReplaceThenFinalize:
        def __init__(self, target):
            self.target = target

        def __call__(self, context):
            self.target.x_tick_format(NumericFormat())
            gc.collect()
            # Failure is in binding result extraction, so its traceback does
            # not retain this Python __call__ frame/self and postpone __del__.
            return object()

        def __del__(self):
            try:
                self.target.svg_preserve_text(tmp_path / "pin-finalizer-render")
                events.append("fresh operation succeeded")
            except BaseException as error:
                events.append(error)

    plot.x_tick_format(NumericFormat.value_function(ReplaceThenFinalize(plot)))
    with pytest.raises(TypeError):
        plot.svg_preserve_text(tmp_path / "extraction-refused")
    assert events == ["fresh operation succeeded"]
    assert (tmp_path / "pin-finalizer-render.svg").is_file()
    assert not (tmp_path / "extraction-refused.svg").exists()
    plot.svg_preserve_text(tmp_path / "after-pin-finalizer")


@pytest.mark.parametrize(
    ("setter_name", "policy_type", "callback_policy"),
    [
        pytest.param(
            "x_tick_format",
            NumericFormat,
            NumericFormat.value_function,
            id="numeric-format-input",
        ),
        pytest.param(
            "y_tick_generation",
            TickGeneration,
            TickGeneration.function,
            id="tick-generation-input",
        ),
    ],
)
def test_replacement_finalizer_can_clear_new_policy_after_input_borrow_ends(
    tmp_path: Path, setter_name, policy_type, callback_policy
) -> None:
    """Two module-integration contracts: formatter and locator input borrows."""
    events = []
    incoming = policy_type()

    class ClearIncoming:
        def __init__(self, target):
            self.target = target

        def __call__(self, _):
            return [0.0, 1.0, 2.0] if policy_type is TickGeneration else "label"

        def __del__(self):
            try:
                clear_gc_object(self.target)
                events.append("incoming policy cleared")
            except BaseException as error:
                events.append(error)

    plot = figure()
    setter = getattr(plot, setter_name)
    # Do not retain the old policy/callable: only the owner's holder reference
    # keeps it alive after this call, until replacement retires that reference.
    assert setter(callback_policy(ClearIncoming(incoming))) is plot
    assert events == []
    assert setter(incoming) is plot
    assert events == ["incoming policy cleared"]
    with pytest.raises(RuntimeError, match="cleared") as caught:
        incoming.copy_with()
    assert "borrow" not in str(caught.value).lower()

    # Clearing the input cannot revoke the owned native snapshot already
    # captured by the setter, even when clear happens before the setter returns.
    output = plot.svg_preserve_text(tmp_path / setter_name)
    assert (tmp_path / f"{setter_name}.svg").is_file()
    assert (output.width, output.height) == (1000, 700)
