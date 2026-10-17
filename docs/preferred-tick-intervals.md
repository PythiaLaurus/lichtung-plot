# Preferred Numeric Tick Intervals

## One Validated Public Input

Crate-root `TickInterval` is an opaque immutable Clone/Copy/Debug/PartialEq value.
`additive(f64)` requires a finite strictly positive step; `log10_decades(u32)`
requires a nonzero exponent step. Invalid construction returns Data immediately,
before attachment or font discovery. `additive_step()` and `decade_step()` inspect
the selected normalized input; raw choice variants and mutable fields are not
public. The old layout-owned raw enum and its delayed scalar validation are gone.

`Plot::x_tick_interval` and `Plot::y_tick_interval` use the existing sealed
NumericX/Y capabilities. Repeated calls replace only their own axis instruction.
Exact `x_ticks`/`y_ticks` override generated spacing in either setter order, while
actual axis-kind and scientific scale compatibility still validate. The complete
input survives erasure, Canvas/facet copies, measured keys and root feedback.

```rust
use lichtung_plot::{NumericValues, Plot, TickInterval};

let plot = Plot::scatter(
    NumericValues::new("x", [0.0, 10.0]),
    NumericValues::new("y", [100.0, 105.0]),
)
.x_tick_interval(TickInterval::additive(2.0)?)
.y_tick_interval(TickInterval::additive(1.0)?);
# let _ = plot;
# Ok::<(), lichtung_plot::Error>(())
```

## Existing Scientific Source Order

The selected spacing is a first-tried preference, not a count or a promise to
paint every requested tick under inadequate space. A linear source uses complete
integral index products and the existing checked `(1,2,3,5)` decade coarsenings.
Logarithmic spacing selects divisible integral exponents in the representable
exponent intersection; the public u32 input widens to u64 for checked coarsening.
No raw-value logarithm inversion, interval subsequence or changed midpoint domain
is introduced. Actual fit uses the same measured label cells and quantum boundary
as frozen axes.

Source caps, dense/empty misses, fatal font/arithmetic failures, exact-tick
provenance and terminal midpoint rules remain with the existing numeric source
owner. Since valid intervals cannot encode a missing choice or invalid scalar,
the internal next-kind method now returns its selected kind directly; the outer
source iterator still enforces exhaustion and real candidate failures.

Scientific preparation rejects intervals on actual categorical axes before label
resolution or fonts. This matters for Trajectory's admitted NumericX capability,
whose constructor may instead carry categorical conditions. The same prepared
boundary serves standalone, facets and Canvas. Static categorical neighbors and
erased Any do not expose the setters.

## Focused Acceptance Evidence

One public-path regression,
`categorical_trajectory_intervals_refuse_before_fonts_in_standalone_and_canvas`,
replaces `standard_panel_owns_complete_typography_and_four_to_three_preference`.
It supplies an unavailable font and proves the actual categorical-axis Data error
wins with unchanged discovery count in both scopes. The retired test was a
default-aspect/field-match inventory; actual complete-text/cache/ink tests remain.

Existing interval fixtures now construct valid public values and use public
setters. Invalid scalar tests move to immediate constructor refusal rather than
fabricating a delayed invalid state. The former private raw-u64 request
fixture now tests the maximum admitted u32 decade input and all four widened
coarsenings before termination. It retains checked integer behavior without a
test-only public-argument escape route.

An external public contract observes first-tried x spacing 2 and y spacing 1,
including replacement of an earlier x preference. Opposite exact-list/interval
orders publish identical canonical SVG with only the supplied labels. The
existing Forest exact-tick integration supplies a later decade preference and
still projects all exact raw-data positions. Existing Canvas reuse-on/off,
closed font-alphabet, first/coarser/midpoint and independent quantum-fit tests
exercise the same complete prepared source.

Validation passed:

- **31 focused library tests**: eight interval/parent-input tests, twenty-two
  numeric-source tests and the automatic/frozen quantum-boundary contract.
- **Six external contracts**: public first-preference/replacement/exact-order
  behavior, three Forest scientific/exact/raw-domain contracts and two
  numeric/categorical Trajectory callers.
- **92 doctests**, including public construction/attachment examples and opaque
  choice, unsupported-axis and erased-Any compile-fail boundaries.
- Strict all-target Clippy, formatting, whitespace checks, warning-denied API docs
  and successful locked/offline package staging compilation.
- Library discovery remains **671**, including the same **17 ignored** tests.
  One meaningful unit replacement offsets one deletion; no unit cases move to
  integration suites and no complete library/integration suite is needed.

Execution logs are `/tmp/opencode/tick-interval-*.log`. Public root sizing remains
the following owning cutover; TickInterval does not expose a layout extension.
