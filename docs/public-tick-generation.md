# Numeric Tick Generation

TickGeneration is one opaque immutable crate-root choice. Attach it with
Plot::x_tick_generation or Plot::y_tick_generation on numeric-axis families.
Each call replaces the axis's complete choice independently of format/limits:

| Constructor | Meaning |
| --- | --- |
| automatic(), also Default | Deterministic range/span/measured-label policy; explicit reset |
| additive(step) | First-tried positive finite linear spacing |
| log10_decades(step) | Positive integral exponent spacing for Forest Log10 |
| target(count) | Approximately count positions, allowing nice-step deviation and coarsening |
| maximum(count) | At most count complete generated in-domain positions, including endpoints |
| exact_count(count, TickSpace) | Exactly count endpoint-including affine positions; zero is empty |
| interval(step, origin, TickSpace, TickRequirement) | One origin-aligned fixed or preferred lattice |
| multiplicative(anchor, ratio, TickRequirement) | Positive anchored geometric lattice in either ratio direction |

Counts/caps are usize, with no 10000 application ceiling. Target/Maximum are
positive; ExactCount(0) is explicitly empty. `maximum_count(Some(n))` edits the one
positive cap without replacing the primary locator; None clears it. A cap smaller
than an ExactCount rejects. Reader methods is_automatic, additive_step, decade_step,
target_count and get_maximum_count inspect the choice without exposing raw variants.
Actual categorical axes reject non-Automatic instructions before font discovery;
intervals must match scale kind. Any/Heatmap/Matrix have no public axis attachment.

The renderer selects the first complete fitting candidate. Preferred intervals
can coarsen locally; generated maximum filters complete sources, not labels after
selection. For domain [0,1], target(4) can produce [0,0.5,1], target(5)
[0,0.25,0.5,0.75,1] when they fit. Count 1 selects original-domain midpoint, whose
label still must fit. These choices never change scientific values/domains.

ExactCount differs from both Target and Maximum: on [0,10], count 4 requests
[0,10/3,20/3,10], each exact affine combination independently rounded to binary64.
It never coarsens or substitutes a midpoint when labels cannot fit. A zero count
leaves marks/frame/title in place. Count 1 defaults to Lower; `.singleton(...)`
selects Lower/Midpoint/Upper/Position in the declared space. The edit is only valid
on ExactCount. For n>=2 singleton intent is retained but endpoints cannot move.

TickSpace::Data uses raw units; Scale uses scientific scale coordinates. For a
Log10 domain [0.1,10], count 3 in Scale gives exponent operands [-1,0,1] and values
[0.1,1,10]; Data instead gives [0.1,5.05,10] before projection. Endpoint bits are
retained. Physical mathematical endpoints without retained raw bounds are evaluated
through certified rational enclosures, not powf or an invented raw-domain inverse.
Arithmetic/storage failures and unrepresentably collapsed positions reject. Private
Automatic seed bounds remain finite; they do not cap explicit counts or lists.

## Fixed Data Intervals

```rust
use lichtung_plot::{TickGeneration, TickRequirement, TickSpace};
let ticks = TickGeneration::interval(20.0, 5.0, TickSpace::Data, TickRequirement::Fixed)?;
// On [5,95]: [5,25,45,65,85]. Attach with x_tick_generation or y_tick_generation.
# let _ = ticks;
# Ok::<(), lichtung_plot::Error>(())
```

Origin 0 gives [0,20,...,100] on [0,100], eleven positions on [0,200],
[40,60,80,100] on [40,100] and [20,40,60,80] on [5,95]. Neither width, fonts nor
cache settings rescale the data interval. Fixed retains the entire list, including
valid empty lists; cap violations or strict text nonfit refuse instead of dropping
positions. Preferred tries complete coarser steps and then the original scale-space
midpoint. Existing additive/log10_decades constructors remain preferred and retain
their original scale binding. `interval_parameters()` exposes the configured tuple.

Each origin+k*step uses exact signed integer arithmetic and one binary64 rounding.
Membership uses exact endpoint rounding cells, including tie parity; 0.1 steps on
[0,1] include the represented endpoint 1 without an epsilon. Large integer indices
are never cast to f64. Data intervals on Log10 still use physical units; Scale
intervals use exponents. Mathematical domain endpoints without retained raw bounds
are certified into the source's binary representation, not computed with powf or
advertised as invented raw domain data.

## Multiplicative Intervals

```rust
use lichtung_plot::{TickGeneration, TickRequirement};
let ticks = TickGeneration::multiplicative(1.0, 2.0, TickRequirement::Fixed)?;
// On [1,64]: [1,2,4,8,16,32,64]. Ratio 1.5 starts [1,1.5,2.25,3.375,...].
# let _ = ticks;
# Ok::<(), lichtung_plot::Error>(())
```

The anchor and ratio are finite and positive, with ratio other than one. A ratio
below one retains the supplied direction and negative indices rather than rounding
its reciprocal. Each member comes from the original anchor and exact integer
index, never the previous rounded value. Linear coordinates are finite binary64;
Log10 coordinates and labels are certified directly from the mathematical lattice,
including physical powers outside binary64 range. Numeric callbacks receive
`NumericOperand::Multiplicative { anchor, ratio, index }`; `index` is borrowed
canonical signed decimal text, with no machine-integer interchange ceiling.

Fixed keeps the complete represented in-domain lattice or refuses a caller cap,
collapsed representation or text nonfit. A wholly negative linear view or a gap
between powers is valid empty output, without numeric callbacks or explanation.
A linear view containing zero and positive values has an infinite near-zero tail
and refuses rather than applying an invented exponent cutoff.

Preferred first tries the same full lattice. Its bounded recovery then tries
zero-index-aligned sublattices at integer strides, seeded by the assigned span's
measured 6ch count and descending desired counts. Duplicate strides are skipped.
Finally it tries the original member at the floor of the middle index. It never
invents an off-lattice domain midpoint. Every candidate still obeys the caller cap;
actual arithmetic/font/storage errors remain fatal.

Exact x_ticks/y_ticks remain indivisible and authoritative in either setter
order, including intentionally empty labels. A generated maximum cannot truncate
an exact list. Exact positions are caller data-space values; collisions reject
rather than drop/move/rotate supplied text. NumericFormat separately controls
generated number spelling.

Facets/Canvas retain complete per-axis instructions alongside independent range
and numeric format policies. Shared ranges alone do not make panel tick choices
identical. Frozen Output reuses selected ticks/text without generation or layout.

See `examples/07_tick_generation.rs` and the
[numeric specification](../specs/06_NUMERIC_PRESENTATION.md) for complete source
ordering and bounds.
