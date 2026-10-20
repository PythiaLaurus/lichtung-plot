# Numeric Tick Selection Design

This is the source-backed design gate for the admitted target/maximum numeric
tick controls. Baseline: `63a4730`. It fixes the public replacement, source order,
validation, transport, feedback and focused acceptance before implementation.
The implementation remains pending; no new export is current in this commit.

## One Final Public Value

Replace the opaque `TickInterval` with opaque crate-root `TickGeneration`, retaining
`Clone + Copy + Debug + PartialEq` and private discriminants/payloads. Its single
choice is Automatic, Additive, Log10Decades, Target or Maximum:

| Constructor / inspection | Contract |
| --- | --- |
| `automatic() -> Self`, `Default` | Automatic generation; also the explicit reset choice |
| `additive(f64) -> Result<Self>` | Existing finite strictly positive linear preferred interval |
| `log10_decades(u32) -> Result<Self>` | Existing nonzero integral Log10 exponent preference |
| `target(u32) -> Result<Self>` | Desired generated position count, validated in `1..=10_000` |
| `maximum(u32) -> Result<Self>` | Hard generated in-domain position maximum, same validation |
| `is_automatic() -> bool` | Inspect Automatic |
| `additive_step() -> Option<f64>`, `decade_step() -> Option<u32>` | Inspect the existing interval payloads |
| `target_count() -> Option<u32>`, `maximum_count() -> Option<u32>` | Inspect the count payloads |

Use consuming `Plot<K>::x_tick_generation(TickGeneration) -> Self` under NumericX
and the corresponding NumericY y setter. Each replaces the axis's complete
generated-source selection, so a later interval replaces a count, a later count
replaces an interval, and Automatic resets either. No separate optional target,
maximum, density, or interval chain is retained. Final `AxisSpec.generation` and
`AxisSemanticInputs.generation` hold the concrete complete value, with Automatic
as their default. Keep exact ticks, range provenance and NumericFormat independent.

Remove `TickInterval`, its root re-export and `x_tick_interval`/`y_tick_interval` at
the public cutover; migrate repository code, live examples and current API specs.
Do not preserve aliases, deprecated methods or both field representations. The
behavioral leaf may extend the existing opaque carrier privately before the
mechanical cutover, but it must use that same source/value route and retain no
second count field or alternate planner.

## Capabilities And Validation

The sealed sets in `src/kind.rs` remain unchanged:

- Both axes: Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath,
  Regression, Marginal.
- X only: PointInterval, Coefficient, Forest, Dumbbell, HorizontalBar,
  StackedHorizontalBar, Beeswarm, Waterfall.
- Y only: Interaction, Bar, StackedBar, Boxplot, Violin, Raincloud.
- Neither: Heatmap, Matrix, erased Any.

Capability is not proof of actual numeric applicability. In particular,
Trajectory can have categorical x; Waterfall's categorical source remains
categorical. Preparation rejects every non-Automatic generation choice on an
actual categorical axis before fonts. Additive/Log10Decades retain scale-kind
validation; Target/Maximum admit either existing numeric scale. Constructor
failures are Data errors before attachment or font discovery. No zero-as-hide
convention is introduced; byte-zero alpha is unrelated to tick count.

Exact caller ticks remain authoritative in either setter order, including empty
labels that retain positions. Generated count limits do not truncate or constrain
an exact list. Supplied categorical sources retain every category. Preserve
existing range/scale semantic validation rather than using exact precedence to
manufacture a categorical numeric source.

## Deterministic Source Order

`layout/panel/sources.rs` remains the sole generated-source owner. All modes
consume complete sources through the existing formatter, assigned cells, actual
text fit and one terminal original-domain midpoint. Numeric domains, nice-range
snapping, display policy, root width, scientific source rows and guide trials do
not change to achieve a count.

### Automatic And Intervals

Keep the existing Automatic behavior exactly: measured `6ch` upper seed,
descending linear adaptive seeds, or Forest full integral exponents followed by
its existing evenly indexed subsets, then midpoint. Keep interval complete-vector
ordering `(1,2,3,5)` by decades, deduplication/termination and original midpoint.
There is no new universal five-tick default or new automatic root search.

### Target

For target one, select the original-domain midpoint source. For a larger linear
target `n`, try adaptive seeds `n,n-1,...,1`, then midpoint. Use the existing nice
step grammar and larger-step tie break, not a new evenly spaced exact-count
generator. The requested target supplies the initial seed instead of the measured
6ch upper; actual label fit still decides acceptance.

For Forest with at least two integral exponents in its existing `[-323,308]`
intersection, start with `d=min(n, integral_count)`. At `d=integral_count`, use the
complete integral source; otherwise use the existing evenly indexed subset.
Then try `d-1,...,2`, then original-domain midpoint. Do not first try the full
integral source when it exceeds the declared target. If fewer than two integral
exponents exist, use the adaptive exponent-space program above. Do not invert to
binary64 raw powers or change certified raw-value display.

The target is approximate: independent linear domain `[0,1]`, seed four selects
`[0,0.5,1]` under the existing nice-step tie break; target five selects
`[0,0.25,0.5,0.75,1]`. These are semantic witnesses, not renderer snapshots.

### Maximum

Maximum is Automatic's existing ordered program with a complete-source cardinality
filter, not a target and not a post-selection label-dropping pass. Reject a source
whose actual position count exceeds the maximum before vector materialization,
formatting or label measurement. Preserve Automatic's source/tie order; skip an
over-limit full Forest integral source and continue its existing subsets. Maximum
one selects the original-domain midpoint. A candidate containing endpoint positions
counts them; an empty label never exempts its position from the bound.

Refactor common generation just enough to share checked cardinality admission
between Automatic/interval and maximum paths. Target does not acquire a hard cap
at `n`; all paths retain the actual 10,000-position source bound. Adjacent finite
domains can still yield both endpoints for adaptive target seeds, whereas Maximum
one must never admit that two-position vector. Do not choose a different nice
factor within a seed merely because the originally selected factor exceeds the
maximum: that complete source misses and the ordered program advances.

Source ordinals and finite program bounds remain local implementation metadata,
not fees or added quotas. Target's linear program has at most `n+1` candidates;
its integral Forest program has at most `d` candidates. Maximum uses Automatic's
existing bound. The midpoint is emitted at most once. Arithmetic/allocation/font
failures remain fatal; only real source/fit misses advance candidates.

## Source, Parent And Feedback Census

| Current owner | Required migration / retained relationship |
| --- | --- |
| `src/spec/ticks.rs`, `src/spec.rs`, `src/lib.rs` | One validated generation value and sole public export; remove interval-only type at cutover |
| `src/builder/configuration.rs` | Replace both interval attachments with complete generation setters and compile-pass/fail docs |
| `src/layout/panel.rs::AxisSpec` | One generation field, independent exact/format/limits/provenance |
| `src/layout/panel/semantics.rs::AxisSemanticInputs` | Complete generation equality and discriminant/payload feedback, not a two-case interval `expect` |
| `src/layout/panel/sources.rs::{Sources,numeric_tick_plan,NumericTickPlan}` | Actual candidate order, checked cardinality, first fitting complete formatted set and original midpoint |
| `src/layout/panel/geometry.rs::AxisSpec::layout_ticks` | Exact/supplied/generated precedence and ordinary numeric source attachment |
| `src/layout/interior/coordinate.rs` | Prepared per-axis generation, including the special no-exact Forest Log10 call |
| `src/plots/interval_estimate.rs::log_tick_plan` | Forward the same value into the common source owner; preserve exponent-space projection |
| `src/plots/shared_x/prepared.rs::prepare` | Explicit AxisSpec copy and actual categorical validation before font census |
| `src/spec.rs::PlotRenderContext::layout_with_ticks` | Explicit AxisSpec reconstruction retains the complete value |
| `src/builder/facets.rs::{same_composed_inputs,retain_composed_inputs}` | Both-axis exact generation comparison/copy, including Automatic reset and count-kind distinction at equal endpoints |
| Child and height reuse | The existing retained complete Plot/composed/axis inputs carry the new value; no extent-only hit or new sibling dependency scan |
| Numeric font requirements and selected replay | Existing format/default alphabet and exact suppression remain; selection freezes positions, labels and draw operands without recomputation |

Keep feedback tags stable for existing modes: Automatic 0, Additive 1 plus exact
binary64 step, Log10Decades 2 plus integer step. Add Target 3 and Maximum 4 plus
their complete validated integer counts. Equal final text or positions do not
erase the input-kind distinction in composed/feedback keys. Preserve range
provenance and independent NumericFormat equality.

Current executable callers additionally include
`src/builder/facets/interval_tests.rs`, `src/layout/panel/sources/tests.rs`,
`tests/lines/numeric_layers.rs`, and `tests/estimates/model_displays.rs`.
Historical evidence documents can describe their recorded interval API, but
current canonical contracts and compiled callers must use only the replacement.

## Focused Implementation Evidence Plan

Replace lower-value source-policy tests in the same logical change for meaningful
count coverage; the 671/17 library discovery baseline must not grow. Retain the
independent six-ch, index-product, extreme/subnormal, interval and certified Forest
witnesses. Required focused contracts are:

- Actual Target versus Maximum behavior, nice deviation, endpoint cardinality,
  singleton/adjacent domains and midpoint refusal under inadequate assigned space.
- Forest complete/subset ordering, empty/clamped integral intersection, exact raw
  ticks and configured certified formatting; no changed label grammar.
- Immediate constructor refusal and actual categorical refusal before fonts.
- Complete replacement/reset, independent axes, exact setter order, composed
  copy/equality, relevant cache invalidation at equal width/domain and cache-disabled
  selected SVG/PNG equality.
- Existing numeric source tests, six composed interval callers migrated in place,
  representative public Line/Forest contracts, constructor capability inventory
  and new API doctests/compile-fail boundaries.

Run focused tests, strict `just lint`, fmt/diff checks and actual discovery counts.
The public cutover includes doctests, warning-denied docs and package checks.
No implementation, compiler execution or new benchmark result is claimed by this
documentation gate. Integrated full/release verification remains at T6Z3.
