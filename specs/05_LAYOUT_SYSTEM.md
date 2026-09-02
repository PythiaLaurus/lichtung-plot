# Layout System

**Status**: Approved implementation target; non-current until Phase T6

## Purpose

This specification defines the approved constraint, measurement, arrangement,
and automatic-sizing architecture for `lichtung-plot`.
The implemented public contract in `01_ARCHITECTURE.md` and
`02_RENDERING_CONTRACT.md` remains current
until Phase T6 performs the atomic migration in `91_IMPLEMENTATION_PLAN.md`.
The names and behaviors below are not a second current API and must not be added
as aliases, dormant public types, or fallback routes.

The target replaces unconditional Plot and Canvas dimensions with a scientific
layout protocol. Parents pass spatial constraints to children, children return
measured sizes and immutable local plans, and parents assign final positions.
An unconstrained root chooses one finite publishable layout from the measured
requirements of its complete content. It does not begin from a hidden fixed
canvas and enlarge it until validation happens to pass.

## Terminology

The private layout vocabulary is explicit about spatial meaning and does not
reuse data-axis terminology:

- `LayoutScalar` is one signed integer number of layout quanta.
- A `LengthConstraint` is the admitted interval for one spatial length. It is
  used for width or height, not for a numeric plot axis.
- `LayoutConstraints` contains one named width constraint and one named height
  constraint.
- `AssignedGeometryContext`, present only for a size/anchor-dependent assigned
  child call, contains the exact containing size, selected projection identity,
  and attachment/shelf target geometry needed to finalize that child plan. Sibling
  obstacle records are deliberately absent. It is not a public constraint or rectangle API.
- `LayoutSize` is one finite nonnegative logical width and height before final
  canonical CSS-pixel conversion.
- `MeasuredLayout<LayoutPlan>` is one candidate: one `LayoutSize` paired with
  the immutable component-specific local plan that produced it.
- `MeasuredCandidates<LayoutPlan>` is the complete nonempty collection returned
  by one successful measurement call.
- A layout candidate is one finite component-specific arrangement satisfying
  its supplied constraints.
- Candidate-local arrangement assigns provisional positions to the child
  candidates named by one parent candidate. A child selects neither its
  parent-relative position nor one globally preferred candidate. Root selection
  later promotes exactly one provisionally arranged tree to final positions.
- Ink extent is painted geometry outside a component's flow box, such as point
  outline, stroke overhang, bracket, or label ink. Ink extent is not silently
  converted into ordinary padding.

The conceptual private kernel has this shape:

```rust,ignore
type LayoutScalar = i64;

struct LengthConstraint {
    minimum: LayoutScalar,
    maximum: Option<LayoutScalar>,
}

struct LayoutConstraints {
    width: LengthConstraint,
    height: LengthConstraint,
}

struct LayoutSize {
    width: LayoutScalar,
    height: LayoutScalar,
}

struct MeasuredLayout<LayoutPlan> {
    size: LayoutSize,
    plan: LayoutPlan,
}

struct MeasuredCandidates<LayoutPlan> {
    candidates: Box<[MeasuredLayout<LayoutPlan>]>,
}

enum MeasurementOutcome<LayoutPlan> {
    Candidates(MeasuredCandidates<LayoutPlan>),
    Unsatisfied(FitReason),
}
```

A `Candidates` outcome contains at least one candidate. `Unsatisfied` is a typed
recoverable fit miss, not a successful empty collection and not a render error.
A parent rejects the enclosing raw descriptor when an assigned child is
unsatisfied and continues with its remaining descriptors. A component whose
complete descriptor set yields no candidate returns `Unsatisfied` upward; only
the root converts final `Unsatisfied` into the ordered fit error. Numeric,
reservation, internal-invariant, and other root errors remain fatal immediately.
A parent performs every initial-sizing child call in canonical address order.
After an initial child returns `Unsatisfied`, later siblings are still called so
a later fatal error wins; if no fatal error occurs, the parent returns the least
child reason without constructing descriptors or charging a parent evaluation.
A parent consumes each child's complete candidate collection and
emits one parent candidate for every admitted combination of child candidates,
parent alternatives, breakpoints, and residual assignments. A parent must not
retain only a locally preferred child candidate, stop at the first fitting
candidate, or prune a candidate because another local candidate has a better
quality tuple.

`FitReason` is the exact tuple `(kind,component_address,raw_descriptor_ordinal)`.
Ordinal zero is also the sole sentinel for a failure that returns before an
ordinary descriptor stream exists; that path and ordinary ordinal zero are
mutually exclusive within one component call.
Its closed `kind` order is:

```text
constraint, text, tick, legend_or_guide, track_or_residual,
overlay_collision, specialized_geometry, assigned_child
```

Predicate ownership maps exhaustively to `kind`:

- `constraint`: a normalized component size outside its call interval, an
  oversized inset before child-constraint subtraction, or an assigned box that
  cannot be constructed for a non-track reason;
- `text`: mandatory-break, unbreakable-sequence, wrap, or fit failure owned by
  an independently measured `text`/`composition_text` component or an inline
  annotation/overlay text record;
- `tick`: numeric/categorical tick source, inline tick-label alternative,
  uniqueness, projection, interval, or axis-fit failure;
- `legend_or_guide`: legend flow, guide gradient/label generation, edge routing,
  or empty routed-view failure, excluding a reason returned by an independently
  measured text child;
- `track_or_residual`: Grid/sequence track minima, gap/inset total, residual
  derivation, receiver, balanced-vector, or track-containment failure;
- `overlay_collision`: only the generic typed anchor/obstacle intersection
  producer;
- `specialized_geometry`: every Heatmap, Beeswarm, Marginal, Matrix, horizontal-
  categorical, or Waterfall producer rejection, including Waterfall shelf-label
  overlap;
- `assigned_child`: only an assigned child miss for which no returned or mapped
  producer reason exists.

Every counted raw rejection records every applicable recoverable reason at its
canonical descriptor ordinal after evaluating all recoverable predicates needed
by that complete raw attempt. Reasons sort by the closed tuple order and exact
duplicates deduplicate; the least reason is that attempt's diagnostic. A child's reason retains its original kind/address and participates in
the parent's set; `assigned_child` is used only when no more specific child
reason exists. If a component admits no candidate, its `Unsatisfied` stores the
lexicographically least recorded reason. The root uses that reason when it has no
complete candidates. If complete root candidates exist, ordered gate/viability
first-empty-frontier precedence applies instead. Cache storage preserves the
same reason tuple, so diagnostics never depend on traversal or cache hits.

Before a collection is returned, every candidate has its complete local
provisional arrangement, ink, collision, overlay, and serialization inputs.
Candidates then sort by complete structural signature and exact duplicate
signatures deduplicate. Every candidate evaluated before deduplication still
consumes one candidate evaluation. A parent candidate structurally names one
candidate at every child address for that Cartesian combination, but does not
make a local quality choice. Only the root selector globally chooses one
complete candidate. The selected root candidate recursively names exactly one
candidate at every child address; its already-complete provisional positions are
then promoted to final positions without recomputation.

These names describe private responsibilities. Phase T6 does not expose a
public layout module, constraint type, candidate collection, measurement
operation, plan type, callback, or plugin.

## Canonical Numeric Domain

`LayoutScalar` is exactly a signed 64-bit two's-complement integer number of
layout quanta. One quantum is exactly `1/1024` CSS pixel. Spatial lengths occupy
`0..=i64::MAX`; signed positions and ink edges occupy
`i64::MIN..=i64::MAX`. A box is valid only when its edges are ordered and each
edge difference is nonnegative and no greater than `i64::MAX`.

Conversion from finite binary64 input is defined over the input's exact binary
rational value after mathematical multiplication by `1024`; it does not use an
intermediate rounded binary64 product. The exhaustive rounding categories are:

1. a lower length constraint rounds toward positive infinity;
2. an upper length constraint rounds toward negative infinity;
3. a nonnegative hard required length or reserve rounds toward positive
   infinity;
4. a signed lower ink, collision, or containing-box edge rounds toward negative
   infinity;
5. a signed upper ink, collision, or containing-box edge rounds toward positive
   infinity;
6. a font size, glyph advance, baseline, preferred length, alignment guide, or
   unconstrained derived position rounds to nearest with ties to even;
7. an exact public integral CSS-pixel dimension converts by checked
   multiplication by `1024` and is not rounded.

No spatial value may use another rounding category. Stroke centers and widths
use category 6; their painted edges use categories 4 and 5. Theme spacing uses
category 6; the containing hard extent that includes the spacing uses category
3. Negative zero becomes zero before conversion.

One `em` is the category-6 normalized resolved font size of the text role named
by the formula. One `ch` is the category-6 normalized shaped advance of U+0030
DIGIT ZERO using that role's resolved font stack, selected face, weight, size,
features, and fallback rules. Text readability formulas use the measured text
role; tick formulas use the tick-label role. Failure to resolve or shape U+0030
is a font-resolution error before candidate generation. Multiples including
`1em`, `6ch`, `66ch`, and `72ch` use checked integer multiplication.

Spatial addition and subtraction use checked `i128` intermediates and must
convert back to the required `LayoutScalar` range. Counts, products, areas,
squared distances, rank sums, byte counts, and ratio cross-products use checked
`u128`. Failure of any checked operation aborts the root operation with a render
error. It never saturates, wraps, changes rank, drops only the overflowing
candidate, or retries with a smaller value.

A mathematically derived optional breakpoint outside its constraints or the
`LayoutScalar` range is not admitted. That range test occurs before integer
conversion and is not an overflow-recovery path.

Every nonspatial `f64` entering convergence, a candidate descriptor, a quality
input, or a structural signature must be finite. NaN and infinity reject.
Negative zero first becomes positive zero. For the remaining value with bits
`b`, its total-order key is:

```text
f64_order_key(b) =
    !b                         when bit 63 of b is one
    b ^ 0x8000000000000000     otherwise
```

These keys compare as unsigned `u64`. Equality requires equal normalized bits.
Nonspatial binary64 calculations use round-to-nearest, ties-to-even, without
fused-multiply-add substitution. Strings compare by UTF-8 bytes.

Any logarithmic scale transform that supplies a layout position uses the
correctly rounded ties-to-even binary64 result of the mathematical `log10` over
the input's exact binary rational value through a vendored safe Rust port of
CORE-MATH binary64 `cr_log10` at external revision
`e0a3599597503f2af90311c82e2ca92dc06c789a`. The port must preserve that source's
integer tables, operation order, exceptional cases, and rounding proof bit for
bit, with exhaustive hard-to-round vectors compared against the pinned C source;
the revision is provenance for this one current routine, not a selector.
Platform libm, target CPU instructions, network-fetched source, and approximate
fallback results are not admitted. Generated decimal powers use the exact `decimal_power` rule below.

For nonnegative `u128` values:

```text
relative_error(a, b) =
    0
        when a == 0 and b == 0
    ceil_div(2^32 * abs_diff(a, b), max(a, b))
        otherwise

ceil_div(n, d) =
    n / d + u128::from(n % d != 0)
```

Every operation is checked. The second branch necessarily has a positive
denominator. Arithmetic failure aborts the root operation.

## Constraint Semantics

A `LengthConstraint` is normalized when:

- `minimum` is finite and nonnegative;
- `maximum` is either `None`, meaning no caller or parent maximum, or a finite
  value greater than or equal to `minimum`.

Infinity and NaN are not stored as sentinels. Invalid intervals reject before
component measurement. A measured child size must satisfy both named dimension
constraints. Layout does not clamp an invalid measurement into apparent
success.

The target root policies are:

| Public request | Width constraint | Height constraint | Meaning |
| --- | --- | --- | --- |
| no sizing call | minimum zero, no maximum | minimum zero, no maximum | Select one finite automatic layout through the quality objective. |
| `max_width(width)` | minimum zero, finite maximum | minimum zero, no maximum | Reflow width-dependent content within the bound and derive height. |
| `size(width, height)` | minimum equals maximum | minimum equals maximum | Honor the exact canonical dimensions or return a fit error. |

`max_width` requires a positive width. It is an upper bound, not a promise to
fill unused width. `size` remains one exact pair; the renderer never silently
enlarges, shrinks, swaps, or partially honors it.

An unconstrained internal dimension is legal only during measurement. Every
selected root size, arranged box, backend dimension, and canonical scene
dimension is finite and representable before rendering begins.

## Layout Direction

Every normal-flow component follows the same direction:

```text
parent LayoutConstraints
          |
          v
complete child measurement
          |
          v
MeasuredCandidates<ChildLayoutPlan>
          |
          v
parent candidate combination
          |
          v
candidate-local provisional arrangement
          |
          v
root selection
          |
          v
selected-tree promotion
```

The parent derives child constraints from its constraints, resolved spacing,
complete sibling collections, alignment requirements, exact-total residual
inputs, and explicit composition policy. Each parent candidate names one child
candidate at each child address and assigns its provisional box before the
candidate is evaluated. After root selection, those same boxes become exact
final boxes.

`PanelBox` or its target equivalent remains an arranged rectangle. It is not an
incoming constraint or measurement result and must not conceal whether a
dimension was exact, bounded, or unconstrained.

Canvas is the render owner for composed output. It measures child Plot content
under Canvas-cell constraints, but a child Plot's standalone `size` request is
not an intrinsic preference and does not compete with Canvas ownership. Facets
inherit their owning Plot's root constraints and resolve their complete topology
inside that one Plot layout.

## Finite Candidate Space

### Pass-Local Protocol

After complete child collections are available, each cache-miss component uses
this exact pass-local order:

1. calculate, without retaining collections, the preliminary base-breakpoint
   counts with generated-tick, text-wrap, legend-flow, and other declared
   alternative-derived transitions omitted;
2. count every non-tick transition source's raw attempts and exact maximum result
   cardinality without shaping or evaluating it, in this order: text-wrap,
   legend-flow, track, then specialized-interior; calculate the exact maximum
   tick-independent-set capacity as the checked sum of preliminary and non-tick
   transition maxima;
3. claim cumulative generation slots and reserve the preliminary, non-tick
   source/transition, and tick-independent maximum capacities in that order;
   materialize the preliminary sets, then evaluate/materialize the non-tick
   streams in step-2 order, and form the actual sorted deduplicated tick-
   independent sets in the reserved capacity;
4. from those materialized tick-independent sets, count every generated-tick
   source/raw attempt and its exact maximum fit-transition cardinality without
   shaping or evaluating it; calculate the exact maximum final-base capacity as
   the checked sum of the actual tick-independent length and tick-transition
   maximum;
5. claim slots and reserve generated-tick source/transition and final-base maximum
   capacities in that order; evaluate/materialize the generated-tick streams and
   form the actual sorted deduplicated final base sets exactly once;
6. count every producer-independent aspect-expanded width and height set, claim
   its slots, reserve exact capacity, and materialize it exactly once;
7. for every completed independent dimension choice, count the remaining
   ordered discrete-alternative seeds and any declared width-dependent dimension
   choices, claim their slots, reserve exact capacity, and materialize them;
   source streams already stored in steps 3 and 5 are referenced rather than
   materialized again;
8. count the exact-total residual-assignment descriptors, claim their slots,
   reserve exact capacity, and construct them;
9. calculate the descriptor-count lower bound and exact assigned-box call-attempt
   count; reject before calls if either exceeds its remaining cumulative budget;
10. derive every child cell and obtain assigned-box child outcomes in descriptor
   and canonical address order;
11. calculate that component's complete pre-deduplication evaluation count from
   those outcomes;
12. reject if that count exceeds the remaining cumulative candidate budget;
13. reserve the complete candidate collection and plan, quality, and signature
   payloads at their calculated maximum capacities;
14. evaluate every raw alternative attempt or composite combination in mixed-
   radix order, including attempts later rejected;
15. form every admitted candidate's complete local provisional arrangement,
    ink, collision, overlay, and serialization inputs;
16. sort admitted candidates by complete structural signature;
17. deduplicate exact signatures.

Each raw-source count and maximum-capacity formula is closed and must equal the
materializer's raw cardinality and maximum exactly; the actual post-filter/post-
deduplication length may be smaller and must not exceed that reserved maximum.
Count-only transition stages do no shaping, collision, or source evaluation and
retain no hidden collection. For every storage class in steps 3 through 8, the
exact order is cumulative slot claim, fallible exact reservation, then
materialization. A failed claim performs no reservation, and a failed reservation
performs no materialization. Generation work is therefore bounded even though the
candidate-evaluation count is known only after assigned-child outcomes.
Every materialized descriptor has evaluation cardinality at least one, so its
count is a valid candidate-budget lower bound. The assigned-call count is the
checked sum of canonical children over descriptors whose boxes can be formed;
descriptors already marked as pre-box singleton rejections contribute zero calls.
The count-only operation retains no collection, creates no candidate plan, and
performs no backend lifecycle. A component with no admitted candidate after all
counted attempts returns `Unsatisfied`.

A dimension's base breakpoint set contains exactly:

- its normalized minimum constraint;
- its finite normalized maximum constraint, when present;
- every hard minimum supplied by its traced measurement inputs;
- every preferred length defined below;
- every text-wrap transition;
- every generated-tick fit transition;
- every legend-flow transition;
- every track or specialized-interior transition explicitly defined by the
  closed composition and specialized producers below.

Aspect-induced values are derived only from the immutable base set and then
unioned into the completed non-residual breakpoint set. Exact-total residuals
are parent track-vector alternatives and never re-enter a child breakpoint set.
Breakpoints outside the constraint interval are discarded. Remaining values
sort numerically and deduplicate before Cartesian combination. Components do not
sample continuous intervals, enlarge until a check passes, stop after finding a
passing candidate, or retain a best partial result.

For each panel dimension, its **tick-independent source set** is the completed
base breakpoint set obtained from the list above with generated-tick fit
transitions omitted. Its inputs are the normalized call constraints and the
non-tick hard minima, preferred lengths, text, legend, track, and specialized
source inputs derivable from prepared content plus the consumed frontier. The
producer derives, sorts, and deduplicates this finite set during pass-local steps
1 through 3; the frontier carries those typed derivation inputs and never a
materialized breakpoint or transition set.

Initial axis generation is a one-way stage: enumerate that source set in
ascending order, generate the complete no-interval axis stream once for each
span, count and reserve all resulting fit transitions, then union the transitions
into the owning panel's final base breakpoint set. The union is not fed back into
the source set during that pass. A caller-interval stream is span-independent,
but its measured label fit transitions go through the same one-way union. An
assigned-box axis call instead uses only its exact assigned parallel span and
regenerates its complete source stream for that span. Thus every producer and
cache identity has a finite input source, and no axis calls itself recursively.

A Leaf uses only the dimension radix declared by its producer below. A composite
uses one ordered `DimensionDescriptor` radix:

```text
Natural
Target(width: Option<LayoutScalar>, height: Option<LayoutScalar>)
```

`Natural` occurs first only when neither axis is exact and derives outer
dimensions from its child combination and tracks. If either axis is exact,
`Natural` is omitted and every emitted `Target` specifies every exact axis. A
`Target` also occurs for each aspect/preference-induced completed outer length;
`None` leaves that axis natural. `W` contains exactly an exact width, when
present, plus this composite's own preferred and aspect-induced outer widths.
`H` is defined symmetrically. Child hard minima, text/tick/legend transitions,
and natural track sums are exercised by `Natural` and residual construction and
do not become composite targets merely because they occur in a descendant base-
breakpoint set. A finite maximum that is only a bound is likewise excluded. Sort
and deduplicate `W` and `H`, then emit every
width-only target, every height-only target, then every Cartesian `(w,h)` target,
while requiring every exact constrained axis to be `Some` at its exact value.
Thus exact `.size(w,h)` emits the combined pair, and one exact axis appears in
every descriptor. Targets finally sort by width then height with `None` before
`Some`, and exact duplicates deduplicate. A target dimension invokes
sequence/Grid residual and assigned-box rules; it is never an ignored outer
breakpoint. A bounded nonexact axis does not create a target merely to fill its
maximum.

### Text Alternatives

Text segmentation and shaping use one pinned profile. Unicode scalar input is not
normalized. Extended grapheme clusters follow UAX #29 and line-break classes and
default rules follow UAX #14, both from Unicode 16.0.0, with no locale tailoring;
mandatory BK/CR/LF/NL behavior and Unicode GL/WJ nonbreaking behavior apply
exactly. Paragraph bidi levels follow UAX #9 from the same Unicode release,
defaulting an all-neutral paragraph to left-to-right. Tables are first-party
pinned data and never come from the host.

Within a paragraph, segment shaping runs only when bidi direction, Unicode script,
or the resolved face changes. Face fallback follows `03_THEME_SYSTEM.md` for each
scalar by scanning the canonical candidate list for its used `(FontStack,weight)`;
it never scans host discovery order or selects one face per stack entry.
Consecutive scalars with the same selected face remain in one run. A selected
face identity is the exact 32-byte SHA-256 file digest
followed by its `u32` collection index and compares in that byte/integer order.
Root preparation retains each used `(stack,weight)` and its complete ordered
font-file identity list as font-environment provenance. Those lists and the
selected face for every glyph are immutable frontier and frozen-plan inputs;
font candidates are not `MeasuredCandidates`, layout breakpoints, or layout
candidate accounting units.
Shape each run with locked `rustybuzz 0.20.1`, language `und`, default
variation coordinates, an empty caller feature list, and that engine's standard
required/default OpenType features; discretionary caller features are not
admitted. Store exact face file identity/index, direction, script, glyph ids,
clusters, offsets, and advances in the frontier/plan. A “complete sequence as one
run” below means one logical shaping operation over this exact ordered run list,
never separately shaped words or summed standalone glyph estimates.

All face and glyph values are converted from signed design units by the exact
rational `design_value * resolved_font_size_quanta / units_per_em`. Advances,
offsets, baselines, and typographic metrics use nearest-even rounding; lower glyph
ink edges use outward-negative rounding and upper edges use outward-positive
rounding. A face's typographic ascender/descender come from OS/2
`sTypoAscender/sTypoDescender` when that table exists, otherwise from hhea
`ascender/descender`; a nonnegative descender is invalid. Across fallback runs,
`A` is the maximum scaled positive ascender and glyph ink-above value of every
used face, and `D` is the maximum scaled absolute descender and glyph ink-below
value. Text with no glyph uses the resolved primary face's scaled metrics. The
same exact scaled rustybuzz advances/offsets and outward glyph boxes feed wrapping,
ink, outlined paths, preserve-text anchors, and PNG planning. A missing/invalid
metric, shaping-engine failure, or arithmetic failure is fatal during preparation
or generation; it is not a recoverable text fit rejection.

`b` mandatory line breaks partition present text into exactly `b+1` paragraphs,
preserving empty leading, interior, and trailing paragraphs. Every paragraph
produces at least one line, so present empty text produces one empty line and
each leading/trailing mandatory break contributes its empty line. Within each
nonempty paragraph, legal breaks are exactly the Unicode line-break opportunities
between resolved grapheme clusters. There is no separate caller-preserved-span
input: Unicode nonbreaking characters participate in that algorithm, and each
maximal sequence with no legal internal opportunity is one unbreakable sequence.

Within each nonempty paragraph, index its start, every legal internal break, and
its end as boundaries `0..=m`. For every pair `i<j`, shape the complete grapheme
subsequence from boundary `i` through boundary `j` as one run and round that
run's total advance toward positive infinity as a wrap transition. No pair
crosses a mandatory paragraph break. Individual nearest-even glyph advances are
not summed to produce this hard extent. At each transition width,
line breaking is greedy: beginning with the next unconsumed grapheme, select the
longest sequence ending at a legal break whose outward advance does not exceed
the width. A mandatory break ends the line. An unbreakable sequence that does
not fit rejects that transition.

The text hard minimum is exactly the maximum outward ink span of one maximal
unbreakable sequence under the pinned shaping profile; the empty maximum is zero.
The natural unwrapped width is the maximum outward ink span of one complete
paragraph, also zero for no nonempty paragraph. These ink spans, not summed glyph
advances or `66ch`, supply the text hard/natural breakpoint inputs.

Pass-local count-only step 2 calculates the checked boundary-pair count as the
exact maximum transition cardinality but performs no shaping and claims no slot.
In materialization step 3, claim one candidate-generation slot per pair and
reserve that exact maximum capacity before shaping; fail before shaping when the
count exceeds the remaining generation budget or reservation fails. Store each
pair's rounded transition in its claimed slot, then sort and deduplicate the
widths. Form the text Leaf's completed width set from the generic normalized
minimum, finite maximum, hard minimum, and these transitions; text defines no
preferred-length breakpoint and the later `66ch` quality target does not add one. Apply interval
filtering, then process every distinct completed width in ascending numeric order
as one raw greedy attempt. Thus a finite constraint endpoint is evaluated even
when it is not a wrap transition. Process one explicit unwrapped-vector attempt last. The latter
counts even when it duplicates the result at the largest transition; present
empty text has zero boundary-pair transitions but still evaluates every generic
completed constraint/preference width before its final unwrapped attempt. This
stream is the complete text alternative stream. Every attempt counts even when
an unbreakable sequence rejects it. Admitted alternatives deduplicate by ending-
grapheme vector and sort by line count then ending-grapheme vector. Quality
widths shape each complete selected line as one run and outward-round that total.
Nongreedy partitions are not admitted.

An independently measured text component returns its sorted/deduplicated admitted
collection under the universal component rule. A text-bearing prepared record
instead contributes its raw text stream inline to its owning producer. This
applies to Heatmap labels, categorical value labels, Waterfall base/terminal/value
labels, panel text, annotation/significance labels, attached mark labels, and
every other prepared text record. Within one owning seed,
emit rejected text attempts first in `(record_traversal_order,raw_attempt)` order
as singleton `text` rejections for annotation/overlay owners or
`specialized_geometry` rejections for specialized-interior owners, then enumerate the Cartesian
product of each present record's admitted raw alternatives in traversal order.
Retain raw duplicates until complete owning-candidate signature deduplication. Its
cardinality is the sum of rejected attempts plus the product of admitted-stream
lengths; no present record with zero admitted alternatives contributes a product.
The selected raw text geometry determines the record's hard pitch, shelf, cell,
ink, and plan values. Inline records create no text measurement call or cache
entry.

For one admitted ending-grapheme vector, shape every selected line as one run.
Let `line_left` and `line_right` be its outward-rounded ink edges relative to its
start anchor; both are zero for an empty line. Let
`line_span=line_right-line_left`. The candidate width is
the maximum `line_span`, or zero for no lines; the transition width controls the
greedy breaks but is not returned as blank flow width. Let `A` be the maximum of
the outward-rounded resolved-face typographic ascender and all selected glyph ink
above the baseline. Let `D` be the maximum of the outward-rounded positive
resolved-face typographic descender and all selected glyph ink below the baseline.
These face terms define `A` and `D` even when every selected line is empty. Define:

```text
line_pitch = max(A + D, resolved_role_line_height)

text_height = 0                            when line_count == 0
text_height = A + D +
              (line_count - 1)*line_pitch otherwise
```

`resolved_role_line_height` uses rounding category 3 because it bounds hard
line occupancy. The first baseline is `A`; baseline `i` is `A+i*line_pitch`; the
last baseline is absent for zero lines and otherwise uses `i=line_count-1`.
Every present text component exposes its first baseline as its sole sequence-
alignment baseline; its last baseline remains available for ink and component
contracts but is not used for child alignment. Present empty text therefore
exposes baseline `A`, while an absent `empty_slot` exposes none. A fixed glyph
exposes `round_ties_even_exact(height/2)` as its sequence-alignment baseline, so a
legend key aligns its vertical center to the label's first baseline under the
declared legend-entry rule.
Let `block_width` be the candidate width above. For a line with outward edges
`line_left,line_right`, its stored x translation is:

```text
start  = -line_left
center = round_ties_even((block_width - line_span) / 2) - line_left
end    = block_width - line_span - line_left
```

The text role's closed alignment discriminant is `start,center,end`; every line
uses that one role alignment. These translations place each line's ink inside
`[0,block_width]` and define the final per-line anchor consumed by serialization.

Text-role alignment is normally a renderer-owned mapping and is never a
`TextTheme` input. The sole theme-owned exception is
`CompositionTheme::root_title_block_alignment`. Every present Plot/Canvas root
title, subtitle, and caption consumes the same owner-local resolved
`Start,Center,End` value. It controls both each line's stored translation and
the horizontal cross-axis placement of each nonfill root text child within the
root's post-inset content span. All built-in themes resolve it to `center`.
`Start` and `End` are physical local left and right, not bidi-logical directions.

The remaining renderer-owned mappings are:

- TextPanel title/body, legend title/label, guide title, and generic standalone
  text use `start`;
- Canvas/facet composition headers and row titles, axis titles, tick labels, and
  continuous-guide numeric labels use `center`;
- panel/annotation text at a left-edge or left-corner anchor uses `start`, at a
  right-edge or right-corner anchor uses `end`, and at top, bottom, center,
  projected, attached, bracket-shelf, or Waterfall-shelf anchors uses `center`.

This resolved role value is an immutable frontier/plan/signature input and is
identical for measurement and serialization. No current producer enumerates text
alignment alternatives; resolved alignment is never a candidate radix.

Mandatory breaks add no spacing beyond one line pitch. Every empty paragraph
uses one empty line with these vertical metrics. A semantically absent text
component, as distinct from present empty content, uses the empty-slot component
instead. These formulas are the complete text
candidate dimensions and baselines; a separate height-breakpoint Cartesian
choice is not generated.

### Tick Alternatives

For a categorical axis, the complete tick vector contains exactly one tick and
one present label for every category in semantic order. There is no numeric tick-
count seed, dropping, stride, rotation, or empty-ticks policy. For each category,
enumerate the complete text alternatives of its resolved display label; enumerate
their Cartesian product in semantic-category order. Let `label_span[i]` be the
selected label's outward axis-parallel ink span and let `label_clearance` be the
traced nonnegative adjacent-label gap. Each product contributes exactly one
categorical label-fit transition:

```text
categorical_axis_hard_span =
    category_count * max(label_span[i] + label_clearance)
```

The empty maximum and product are zero. Combine this hard span with the owning
categorical interior's hard/preferred panel spans as ordinary breakpoints. At
each admitted or exact nonnegative panel span `S` with `n > 0`, category `i` uses
these checked exact-rational category-6 coordinates:

```text
band_start[i] = round_ties_even_exact(S * i / n)
band_end[i]   = round_ties_even_exact(S * (i + 1) / n)
center[i]     = round_ties_even_exact(S * (2*i + 1) / (2*n))
```

Products use checked `u128`; `round_ties_even_exact` compares the doubled
remainder with the positive denominator and resolves equality to an even integer.
The first edge is zero and the final edge is `S`. Place labels at those centers
and reject the seed if any outward interval crosses its closed semantic band or
the panel endpoints. Perpendicular label ink,
ticks, stroke, gaps, and title determine the gutter by the axis rule below. These
label products, hard-span transitions, and final interval checks are the complete
categorical-axis producer and accounting order.

A present caller exact-tick list must contain `1..=10_000` entries; an empty or
larger present list rejects during semantic preflight and never selects automatic
generation. Absence selects automatic generation. One valid exact list
contributes exactly one indivisible alternative and takes precedence over an
optional automatic `TickInterval` regardless of setter order. Its positions must
be finite, strictly ascending, and inside the final finite, strictly ascending
display domain. Its labels remain verbatim and need not be unique. An empty exact
label is an intentionally unlabeled tick: its tick and grid position remain, but
it creates no text attempt, text plan, baseline, extent, collision interval,
gutter, or paint. No empty exact-label alternative reaches fit-transition
calculation.

Automatic range broadening is independent of generated-tick density. In the
absence of exact limits, the canonical continuous/anchored nice-range producer
retains the private renderer invariant
`automatic_range_nice_seed_count=5` together with its separately owned five-
percent margin and nice-step grammar. That value affects only display-domain
snapping; it is not a generated-tick target, ceiling, candidate count, quality
input, or public control.

Automatic numeric generation has two policy modes. With no `TickInterval`, it
uses typography- and span-dependent renderer policy. With an interval, it uses
the caller's preferred data-space interval, tries deterministic coarser
intervals only when a complete requested-interval axis candidate cannot survive,
and retains one documented midpoint source after every interval vector as the
terminal one-tick fallback. `TickInterval::additive(step)` requires a finite
positive `step` and a linear numeric axis. `TickInterval::log10_decades(n)`
requires a positive integer `n` and a Log10 axis. A kind/scale mismatch rejects during semantic
preflight. Categorical axes do not admit this policy. Repeating one axis setter
replaces its previous interval; an exact `AxisTick` list suppresses that axis's
interval regardless of builder-call order.

All generated linear-step sources use these private bounds:

```text
MAX_GENERATED_TICKS  = 10_000
MAX_EXACT_TICK_INDEX = 2^53
```

For one finite positive `step` and closed finite display domain `[lo,hi]`, first
calculate `first_f=ceil(lo/step)` and `last_f=floor(hi/step)` by prescribed
binary64 operations. The source rejects unless both results are finite,
integral, and inside `[-MAX_EXACT_TICK_INDEX,MAX_EXACT_TICK_INDEX]`. Convert them
exactly to signed integers, calculate `count=last-first+1` in checked signed
wide-integer arithmetic, and reject unless `count` is in
`1..=MAX_GENERATED_TICKS`. Emit each position by prescribed binary64
`(index as f64)*step`; reject the source if any result is nonfinite, outside the
closed domain except for an exact endpoint, or not strictly greater than its
predecessor. Rejecting an unrepresentable fine step does not reject its later
coarser sources.

With no caller interval, let `S` be the source's finite provisional
axis-parallel panel span and define with checked integer arithmetic:

```text
automatic_seed_pitch = max(1 quantum, 6ch)
upper_seed_count =
    1 + min(9_999, max(1, S / automatic_seed_pitch))
```

An assigned-box axis call uses its exact assigned `S`. An initial call processes
each finite span in the owning panel producer's tick-independent source set in
ascending order; that set and its one-way use are defined under **Axis
Planning**. For each `k` from `upper_seed_count` through `1` in descending order,
emit one raw nice-step source. If `hi-lo` is finite, use
`raw_step=(hi-lo)/k`. Otherwise let `scale=max(abs(lo),abs(hi))` and use the
prescribed operation order
`raw_step=((hi/scale)-(lo/scale))*(scale/k)`. Reject that raw source unless its
result is finite and positive.

With `TickInterval::additive(step)`, emit one raw source for each coarsening step
in the sequence `step * (1,2,3,5)`, then ten times those four values, and so on.
Calculate each new decade base by prescribed binary64 multiplication by ten;
within a decade use multiplier order `1,2,3,5`, omit exact duplicate steps, and
stop after processing the first finite decade base greater than
`max(abs(lo),abs(hi))` or when the next base cannot be finite. Multiplier one
therefore tries the caller's requested interval exactly; later sources are
deterministic coarser integer multiples. Apply the common linear-step source
rule above. A too-dense or unrepresentable source rejects before materializing
its vector; later coarser sources are still processed. These are complete
interval vectors: they never contribute evenly indexed subsequences or an
adaptive nice-step source. After all coarsening sources, this mode contributes
one final singleton at `f64::midpoint(lo,hi)` so a domain containing no
representable aligned multiple still has a bounded candidate. That singleton's
interval-source ordinal follows every coarsened interval source.

For a no-interval nice-step source, calculate the candidate step as follows.

`decimal_power(e)` is the correctly rounded ties-to-even binary64 value of the
mathematical rational `10^e`, calculated by exact integer arithmetic for
`e >= 0` and exact rational division for `e < 0`; it does not call a platform
transcendental function. The admitted exponent table is `-323..=308`. Choose the
greatest table exponent `e` with `decimal_power(e) <= raw_step`. If one exists,
let `magnitude=decimal_power(e)` and use candidate steps in this order:

```text
10*magnitude, 5*magnitude, 2.5*magnitude, 2*magnitude, magnitude
```

If no table exponent qualifies, the sole candidate step is `raw_step`. Every
listed multiplication uses prescribed binary64 arithmetic. Evaluate each finite
positive step by the common linear-step source rule. Select the admitted step
minimizing `abs_diff(count,k)`; a tie keeps the earlier, larger listed step. If
`lo.next_up() >= hi`, no step is evaluated and the seed emits `[lo,hi]`. A seed
with fewer than two emitted positions instead emits `[lo,hi]`. Reject when no
step is admitted or the endpoint replacement is not finite, closed-domain,
strictly ascending, and inside `1..=MAX_GENERATED_TICKS`.

One additional ordinary-linear no-interval raw seed is the singleton
`f64::midpoint(lo,hi)`. The midpoint must be finite and inside the domain. It is
always the final ordinary-linear no-interval source, not a conditional fallback.
Forest replaces it with the sole expression-labeled midpoint below. Every source
is evaluated; no first-, nearest-, or largest-fitting local selection branch is
run.

Generated labels first use this locale-free grammar after normalizing negative
zero:

- absolute value at least `1e12`: scientific notation with two fractional
  decimal digits;
- absolute value at least `1_000_000`: divide by `1_000_000`, format with one
  fractional digit, trim a trailing fractional zero and point, then append `M`;
- absolute value at least `10_000`: divide by `1_000`, apply the same trimming,
  then append `k`;
- nonzero absolute value below `0.01`: scientific notation with two fractional
  decimal digits;
- absolute value at least `100`: fixed notation with zero fractional digits;
- absolute value at least `1`: fixed notation with one fractional digit;
- otherwise: fixed notation with two fractional digits;
- fixed notation trims trailing fractional zeros and a trailing decimal point.

Decimal conversion uses round-to-nearest, ties-to-even. Scientific notation uses
one digit before the point, lowercase `e`, no `+` for a nonnegative exponent, a
`-` for a negative exponent, and the exponent's shortest unsigned decimal
magnitude with no leading zeros. If labels are not unique, retry the entire list
in scientific notation at precisions `2..=17` in ascending order and retain the
first unique result. Failure to produce unique generated labels rejects the raw
seed.

Forest is the sole family-owned preliminary-list producer. For its finite
strictly ascending base-10 logarithmic position domain `[log_lo,log_hi]`, define
the intersection with the exact decimal-power table by prescribed binary64
min/max comparisons:

```text
clamped_lo = max(log_lo, -323.0)
clamped_hi = min(log_hi, 308.0)
```

The representable intersection exists exactly when `clamped_lo <= clamped_hi`;
otherwise no integral-power source exists. When it exists, prepare every integral
exponent from `ceil(clamped_lo)` through `floor(clamped_hi)` when at least two
exist. The position is that integer converted exactly to binary64. Its label is
the result of applying the generated-label grammar to the exact decimal value
`10^exponent`, without a transcendental conversion.

With `TickInterval::log10_decades(d)`, replace that preliminary source with one
source for each positive checked integer exponent step
`d * (1,2,3,5)`, followed by ten times those multipliers and so on. Multiplier
one tries the requested number of decades exactly. Each source contains the
ascending integral exponents divisible by that step in the clamped position
domain and uses exact decimal-power labels. Calculate steps in checked unsigned
wide-integer arithmetic and stop after processing the first decade base greater
than the maximum absolute clamped exponent or when the next base cannot be
represented. A source with no exponent positions rejects. Every admitted source
contains at most `632` positions. These are complete interval vectors: they
never contribute evenly indexed subsequences or adaptive linear sources. After
all coarsening sources, this mode contributes one final singleton at
`f64::midpoint(log_lo,log_hi)` of the original logarithmic position domain, with
an interval-source ordinal after every coarsened interval source. When the
representable intersection is empty, charge the requested source as one rejected
source block, emit no coarsening vectors, and proceed to that midpoint.

Without a caller interval, emit the complete integral-power source first and then
its subsequences when it exists. Otherwise process the adaptive no-interval
linear nice-step sources in their raw span/count order but omit their ordinary-
linear midpoint. A
rejected source contributes its singleton rejection block; an admitted source
emits its complete list then its subsequences before the next source. Each
prepared no-interval source list of `n >= 2` contributes every evenly indexed
subsequence of size `2..=min(upper_seed_count,n)`, in descending requested-size
order. For requested size `d`, source count `n`, and destination index `i`:

```text
source_index(i, d, n) =
    ((i * (n - 1)) + ((d - 1) / 2)) / (d - 1)
```

After those no-interval source blocks, Forest contributes one singleton at
`f64::midpoint(log_lo,log_hi)` of the original logarithmic position domain.
For either policy mode, a Forest midpoint source's label is the ASCII expression
`10^` followed by that exponent position under the generated fixed/scientific
grammar. No `powf`, `log10`, platform libm, or debug-format result participates.
Adaptive Forest linear-source positions use that same expression grammar; only
the admitted integral-power source uses exact decimal-power value labels.
No other family contributes a preliminary tick list.

Every raw tick seed or subsequence attempt counts before validation. A rejected
source is its singleton rejection block. Every admitted raw list retains its
source block and evaluates its complete label-alternative and fit suffix even
when an earlier source emitted an equal position/label list. Raw lists do not
deduplicate or reorder before those suffixes. The raw source kind and ordinal are
part of every complete axis signature and frozen axis plan, so equal
position/label vectors from distinct requested, coarsened, adaptive, or midpoint
sources remain distinct through root quality. Only complete axis candidates with
equal source identity and every other signature field later deduplicate under the
universal rule.

Tick fit uses one monotone conservative projection, not search over rounded
centers. Let `D=2^64-1`. For tick value `v`, calculate the final binary64
normalized projection in the axis's increasing physical direction as:

```text
p = 0                                  when v == lo
p = 1                                  when v == hi
p = (v - lo) / (hi - lo)               when hi - lo is finite
p = ((v/scale) - (lo/scale)) /
    ((hi/scale) - (lo/scale))           otherwise

scale = max(abs(lo), abs(hi))
```

The specified binary64 operation order is exact and uses no fused substitution.
Require `p` to be finite and in `[0,1]`, and define:

```text
k = round_ties_even_exact(p * D)
```

The multiplication and rounding operate on `p`'s exact binary rational value,
so its numerator product fits `u128`; admitted tick keys must be strictly
ascending. At panel span `S`, the final category-6 center is
`round_ties_even(S*k/D)`.

For each exact or generated numeric tick vector, enumerate the complete text
alternatives for every resolved nonempty label, then their Cartesian product in
ascending tick order. Exact nonempty label text remains verbatim, including
mandatory breaks. An exact empty label contributes no text record and retains
only its tick/grid position. Generated decimal labels normally have one
unwrapped alternative. Tick-label records are candidate-local axis payloads, so
this Cartesian product is their complete seed and count rather than a separate
text-component traversal.

For both categorical and numeric inline labels, retain each label's raw text-
attempt stream before text deduplication. Within one tick-vector source block,
emit every rejected text attempt first in `(label_index,raw_text_attempt)` order
as one singleton rejected axis block with kind `tick`. Then enumerate the
Cartesian product of every label's admitted raw alternatives, retaining duplicate
alternatives as distinct raw choices until complete axis-candidate signature
deduplication. The exact block cardinality is:

```text
sum(rejected_raw_text_attempts_per_label)
    + product(admitted_raw_text_attempts_per_label)
```

The product over zero nonempty label records is one. If any present nonempty
label has no admitted alternative, the product is zero and its rejected
singleton blocks provide the diagnostic.
This inline substream is charged only as axis evaluations; it creates no text
cache entry or structural address.

For each selected shaped label, let `L` and `R` be its outward-rounded axis-parallel ink
extents before and after its anchor. The fit transition is the maximum of these
checked lower bounds:

```text
ceil_div(D * (2*L + 1), 2*k)                 for each k > 0
ceil_div(D * (2*R + 1), 2*(D-k))             for each k < D
ceil_div(D * (R[i] + L[i+1] + 1),
         k[i+1] - k[i])                      for each adjacent pair
```

A tick at `k=0` requires `L=0`; a tick at `k=D` requires `R=0`. Violation means
no fitting span. This monotone conservative transition guarantees containment
and nonoverlap after center rounding. The final hard gate still checks the
actual rounded intervals. Exact and generated lists both contribute this hard
fit transition; absence of a representable transition rejects the alternative.

### Legend Alternatives

For `n > 0` semantic entries, a horizontal legend contributes one alternative
for every column count `1..=n` under both row-major and column-major fill. Its
row count is `ceil_div(n, columns)`. A vertical legend contributes one
alternative for every row count `1..=n` under both fill directions. Its column
count is `ceil_div(n, rows)`.

Top and bottom edges admit horizontal alternatives. Left and right edges admit
vertical alternatives. A corner uses the edge named first by its
`LegendPosition`. Automatic placement contributes the family-default edge,
then top, right, bottom, and left, with duplicate edges removed.

The family-default edge is `right` for every current PlotKind, for discrete
legends and continuous guides, and for the one Canvas-shared legend/guide. Thus
the current automatic seed order is exactly `right,top,bottom,left`; no theme,
backend, data shape, or map order selects a different default.

An outside placement also retains one edge-parallel alignment. On right/left,
suffix `Top` maps to `start`, no suffix maps to `center`, and suffix `Bottom` maps
to `end`. On top/bottom, suffix `Left` maps to `start`, no suffix maps to
`center`, and suffix `Right` maps to `end`. Every automatic edge candidate uses
`center`. The parent aligns the guide's nonfill box to the corresponding start,
center, or end of the data-panel-parallel track after insets; this field survives
assigned-box routing and appears in the arrangement decision/signature. Inside
positions continue to use their exact typed panel anchor rather than this outside
track rule.

An automatically placed legend or guide retains one prepared
`outside_guide_auto` structural address. Its edge is a candidate field, not a
reparenting operation. For each Plot-frame parent candidate, route that child
reference to exactly one side track set named by its selected edge; explicit
placements use their fixed side address. The semantic gate requires the identity
exactly once, and the topology census counts it once regardless of edge
alternatives.

Each Plot frame owns and measures every `outside_guide_auto` component directly,
after its four `track_set` children in canonical guide order. A side track set
owns and measures only explicitly placed guides at that side; for an automatic
guide it stores a non-owning candidate reference supplied by the Plot-frame
descriptor. The selected automatic guide remains a direct Plot-frame child in
the candidate tree while its provisional box lies in the routed track set. Canvas
body applies the same rule to its directly owned `canvas_shared_legend` child.
Thus routing never duplicates a measurement call, cache entry, tree child, or
preorder position.

The parent descriptor retains that base child's selected edge through assigned-
box remeasurement. The assigned call still returns its complete collection, but
the parent forms one stable-signature-ordered routed view containing exactly the
candidates with the retained edge. Candidates at other edges were evaluated and
remain charged to the child call but do not enter this parent descriptor's
Cartesian product. An empty routed view rejects that descriptor once with a
parent-constructed `legend_or_guide` reason using the automatic child address and
the enclosing raw descriptor ordinal; a successful child outcome need not carry
a rejection reason. This is the sole topology-dependent assigned-
child filter; it neither changes the child's returned collection nor reroutes the
parent after track construction.

The closed sort orders are edge `top,right,bottom,left`, edge-parallel alignment
`start,center,end`, orientation `horizontal,vertical`, and fill direction `row-major,column-major`.
Alternatives then sort by rows, columns, and semantic entry-slot vector.
Root-size percentages do not participate.

When `n == 0`, the complete legend component is absent and contributes the one
zero-size empty-slot candidate; a detached title is not rendered and no legend
quality tuple is emitted.

For each alternative and complete title/entry-text candidate combination, an
entry width is exact key width plus traced key-to-label gap plus label width; its
key baseline is its declared vertical-center baseline and its label baseline is
the selected text first baseline. Its height is
`max(key_baseline,label_baseline) +
max(key_height-key_baseline,label_height-label_baseline)`, exactly matching the
universal baseline-sequence rule. Each column width is the maximum
entry width assigned to that column, and each row height is the corresponding
maximum entry height. The entry-grid dimensions are checked track sums plus
traced inter-column and inter-row gaps. The legend dimensions are the complete
title-plus-grid `VerticalSequence` dimensions plus traced outer padding and
border ink. Every distinct resulting width and height is that alternative's
legend-flow transition. These dimensions, and text transitions inside the title
and labels, are the complete legend breakpoint source; no root-size percentage,
trial width, or post-allocation flow branch adds another transition.

Canvas semantic preflight first materializes one kind-tagged shared-guide
descriptor under `canvas.legend.shared-entry-exact-merge`. Compatibility and
continuous-domain merging are semantic preparation, not guide-layout
alternatives or domain feedback. Guide generation receives that immutable final
descriptor and, for a continuous guide, its final merged domain; it only formats
and measures the shared labels.

### Continuous Guide Alternatives

A continuous guide derives orientation only from its ordered placement edge:
top/bottom are horizontal and left/right are vertical, using the same closed edge
order as discrete legends. Automatic placement contributes the family-default
edge, then top, right, bottom, and left with duplicates removed. No independent
orientation alternative exists.

Guide label values derive only from the final finite continuous-color domain. For
`lo < hi`, emit `lo`, then a diverging center only when `lo < center < hi`, then
`hi`; a sequential guide has no center label. For `lo == hi`, emit one label at
the gradient midpoint. Format the complete value list with the generated numeric
grammar and uniqueness retries in that order; formatting failure is one
`legend_or_guide` rejection. Then apply the inline raw text stream in label order.
A shared Canvas guide performs this once after its final merged domain and never
reuses per-Plot label bytes.

Its resolved gradient thickness is the traced guide key width. Let start, center,
and end labels have outward before/after extents `(Ls,Rs)`, `(Lc,Rc)`, and
`(Le,Re)` along the gradient axis, and let `g` be the traced required label gap.
Start and end label anchors are the gradient endpoints; an admitted center label
anchors at its midpoint. The least label-fitting gradient length is the maximum
of:

```text
Rs + g + Le
2 * (Rs + g + Lc)     when a center label exists
2 * (Rc + g + Le)     when a center label exists
```

The gradient length is the maximum of that value and the traced minimum gradient
length. A finite traced maximum caps it; a required value above the cap rejects
the alternative. The complete guide-axis flow span extends from `-Ls` before the
gradient start through `Re` after its end, so endpoint overhang is represented
rather than clipped. The missing-value block is key then label in reading order.
The gradient is not a measured child. The guide Leaf first constructs every
inline title/label/missing-block raw text stream in the order above, then each
guide descriptor constructs one gradient record at the calculated length. No
part requires an initial child outcome or child cache entry; all work is charged
through the enclosing guide combination.

For the constant-domain single midpoint label, replace the nonconstant fitting
formula with `2*max(Lc,Rc) + 2*g`. The exact inline guide plan then applies the
declared Vertical/Horizontal sequence and projected-overlay formulas directly:
left/right use title, a horizontal gradient-plus-label-column body, then missing
block; top/bottom use title, a gradient/label-row Overlay, then missing block.
All inline title/label text combinations, gradient lengths, boxes, gaps, and
missing key/label geometry are one Leaf seed stream in that listed order; no
anonymous measured `track_set`-like primitive or child call exists.

A vertical guide's inline plan is a title followed by a `HorizontalSequence` containing the
gradient and one label column whose endpoint/center labels are projected
overlays, followed by the optional missing-value block. A horizontal guide is a
title followed by the gradient with one projected label row, followed by the
optional missing-value block. Traced title, gradient-to-label, missing-block, and
outer gaps use the sequence/Grid arrangement formulas inside the Leaf; they are
not measured child components. Every complete title/label text
combination produces one natural guide size and one flow transition. These edge,
text, gradient, label, and missing-block combinations are the complete continuous-
guide producer.

### Preferred And Scientific Requirements

For these formulas, `em` and `ch` use tick-label typography. Let `m` be the
number of prepared primary mark geometries, including coincident geometries and
geometries with transparent paint. Define:

```text
mark_pitch =
    1em                                                        when m == 0
    max(1em, maximum outward ink width or height of one mark)  otherwise

density_side =
    ceil_sqrt(max(1, m)) * mark_pitch
```

`ceil_sqrt(x)` is the least integer whose square is at least `x`.

A standard panel's hard width and height are the maxima of one quantum and all
traced text, glyph, mark, interval, annotation, axis-projection, collision, and
representable-geometry requirements in the corresponding dimension. It has no
universal `80x60` floor. Its preferred size is:

```text
preferred_width =
    max(hard_width, 66ch, density_side)

preferred_height =
    max(hard_height, ceil_div(3 * preferred_width, 4), density_side)
```

Its preferred aspect is `4:3`.

The `standard_interior` is a `Leaf`. Its completed width set contains its
constraint endpoints, `hard_width`, `preferred_width`, and admitted aspect-
induced widths; its height set contains the corresponding values. After interval
filtering and deduplication, it evaluates the complete width-major Cartesian
product of those ascending sets. Each candidate stores the final projection and
prepared scale inputs for that exact panel size. This is the
standard producer used by the ordinary PlotKinds listed below and the sole owner
of their panel dimension radix.

A measured `mark_layer` is a size-dependent `Leaf`, not intrinsic geometry. Its
immutable frontier inputs reference the owning panel producer's completed
dimension-source inputs. Under its call-local constraints it regenerates and
filters those same panel-size sets; exact assigned-box constraints therefore
leave the assigned panel pair. For each admitted pair, it deterministically
projects every prepared mark and stores the complete stroke/glyph/interval ink
geometry, enumerating any declared mark alternative in source-row order. It adds
no independent size breakpoint, preference, or aspect. Initial sizing is finite
because it uses the owning producer inputs, while assigned remeasurement produces
geometry for the exact containing panel.

For a categorical dimension with `n` semantic slots:

```text
hard_pitch =
    max(1 quantum, maximum outward occupied span of one slot)

preferred_pitch =
    max(hard_pitch, 2em)

hard_span =
    n * hard_pitch

preferred_span =
    n * preferred_pitch
```

For an ordinary categorical Leaf, the categorical dimension set contains exactly
the normalized constraint endpoints, `hard_span`, `preferred_span`, and admitted
aspect-induced values. Its orthogonal dimension uses the standard set. Apply the
generic interval filtering and numeric deduplication, then enumerate the complete
width-major Cartesian product. Exact constraints therefore leave only their
exact value; a bounded nonexact endpoint remains an admitted Leaf breakpoint.
The mapped categorical families below use this radix. Beeswarm replaces only the
dependent-height portion with its explicit width-dependent producer; Heatmap and
Matrix replace both dimensions with their explicit track producers.

The occupied span includes every mark, interval, value label, and inside-band
annotation assigned to the slot. Axis labels and outside guides remain
normal-flow decorations.

An aspect ratio is a coprime positive `u64` pair `(a,b)` meaning
`width:height = a:b`. For every width breakpoint `w`:

```text
exact height:
    h = (w * b) / a
    admitted only when the remainder is zero

preferred heights:
    floor((w * b) / a)
    ceil((w * b) / a)
```

Apply the symmetric operation from every height breakpoint. Products use
checked `u128`; results must fit `LayoutScalar` and constraints. Aspect expansion
runs once over the pre-aspect sets and does not recurse. The Beeswarm exception
below has a width-independent radix but a packing-dependent height radix; it
applies `4:3` only from each completed width to that width's preferred height
choices and performs no symmetric height-to-width expansion.

Current production PlotKinds emit no exact-aspect requirement. The exact-aspect
vector is therefore empty for current production inputs. Standard, categorical,
beeswarm, waterfall, and marginal-central panels emit preferred `4:3`.
Heatmap and Matrix cells emit preferred `1:1`. No other production aspect is
admitted. Within one component, exact and preferred aspect pairs each sort
lexicographically by `(a,b)` and deduplicate before expansion and quality use.

### Exact-Total Residuals

Let an exact parent dimension provide outer total `T_outer`. Let `I` be its
checked leading-plus-trailing inset. If `T_outer < I`, the raw parent attempt is
counted and rejected before candidate admission without performing subtraction;
otherwise let inner total `T=T_outer-I`. Let `G` be the checked sum of internal
gaps and every inflexible child or track extent in this candidate. Let `n`
ordered flexible tracks have completed non-residual breakpoint sets `B[0]`
through `B[n-1]`.

If `T < G`, the raw parent attempt is counted and rejected before subtraction.
Each flexible track interval has its traced hard minimum. Its finite maximum is
the minimum of `T-G` and every explicit finite maximum owned by that track's
fill policy; absence of another maximum leaves `T-G`. A `B[i]` value outside
that interval is not admitted. Exact child remeasurement later rejects a vector
whose assigned track cannot satisfy the child's complete constraints.

For every residual receiver `j` in ascending track order, choose one breakpoint
for each `i != j` in mixed-radix track order and derive:

```text
track[j] =
    T - G - sum(track[i] for i != j)
```

Each such choice is one raw residual-vector attempt and counts even when the
derived track rejects. The exact receiver-attempt count is:

```text
sum(product(len(B[i])) for every i != j, for j in 0..n)
```

If `n > 0` and this receiver count plus the balanced product is zero because the
filtered `B[i]` sets leave no raw vector, emit one count-only singleton rejection
at the residual-vector position with kind `track_or_residual`. It performs no
assigned child call and guarantees a diagnostic ordinal; it is not an admitted
track vector.

Retain the vector only when the residual track satisfies its constraints.

After all receiver attempts, every vector containing one base breakpoint per
track generates one balanced raw attempt, for exactly
`product(len(B[i]))` additional attempts. Let:

```text
remaining =
    T - G - sum(base)
```

A negative result rejects the vector. Otherwise, repeatedly divide `remaining`
by the number of tracks with headroom. For active count `a`, let
`q=remaining/a` and `r=remaining%a`. Track `i` in active order proposes grant
`q+1` when `i<r`, otherwise `q`; its actual grant is the minimum of that proposal
and its current headroom. Add all actual grants, subtract their checked sum from
`remaining`, and remove every track that reached its finite maximum. If a round
grants zero or no track has headroom while `remaining>0`, reject the vector.

When `n == 0`, exactly one empty vector attempt is counted: it is admitted when
`T == G` and otherwise is one singleton `track_or_residual` rejection at that
vector ordinal. Raw receiver and balanced vectors are parent alternatives
that replace their source base choices; they are not combined with those choices
a second time and do not add values to any `B[i]`. Admitted track vectors sort
lexicographically and deduplicate, but every raw duplicate consumes one
parent-combination evaluation in the receiver-then-balanced raw order. Sorting
and deduplication apply only to the returned admitted collection after all raw
evaluations; they do not change raw mixed-radix accounting. A bounded but
nonexact parent does not manufacture residual to fill unused space.

### Specialized Candidate Producers

The complete specialized producers are:

- Heatmap: retain every declared row and column. A column hard width is the
  maximum complete cell hard width in that column; a row hard height is the
  corresponding maximum. Preferred cell pitches are the hard pitch or `2em`,
  whichever is greater. Every cell contributes preferred `1:1`. Missing cells,
  highlighted cells, merged-row records, and declared trailing slots remain in
  topology.
- Beeswarm: its independent dimension radix is exactly the completed standard
  width set. It has no pre-packing height set. At each candidate panel width,
  process each semantic category independently and reset the prior-point set at
  each category boundary. Within one category, visit points by source-row index.
  `x[i]` is the final category-6 projected numeric center. `r[i]` is the maximum
  outward x/y half-extent of the point's complete final glyph including stroke,
  rounded toward positive infinity. Point `i` may use baseline offset zero or a
  tangent offset from prior same-category point `j`:

  ```text
  y[j] +/- ceil_sqrt((r[i] + r[j])^2 - (x[i] - x[j])^2)
  ```

  when `abs(x[i]-x[j]) < r[i]+r[j]`. For each point, branch first to baseline,
  then by prior point `j` in ascending source-row order, with the negative
  tangent before the positive tangent. If `m[i]` prior points satisfy the
  inequality, exactly `1+2*m[i]` branches exist at point `i`, and the raw vector
  count is the checked product of those branch counts. Combine category products
  by checked arithmetic and apply the generation-slot limit before constructing
  any offset vector. Enumerate every resulting
  offset vector in that mixed-radix order, count and reject overlaps after full
  construction. Overlap exists exactly when the checked integer predicate
  `(x[i]-x[j])^2 + (y[i]-y[j])^2 < (r[i]+r[j])^2` holds for a same-category
  pair; equality is tangent and admitted. The complete packing seed is the
  Cartesian product of category vectors in semantic-category order. For each
  seed, derive its categorical `hard_pitch` from the maximum outward vertical
  extent, then `hard_span` and `preferred_span` by the categorical formulas.
  Under an exact height, emit that exact height only when it is at least the hard
  span. Otherwise emit the hard span, the preferred span, and the floor/ceiling
  `4:3` preferred heights derived from this width, filtering by constraints and
  deduplicating numerically. Thus packing completes the dependent height radix
  before raw Beeswarm evaluation counting. Beeswarm packing is evaluated only at
  panel-width breakpoints supplied by the standard width producer; it contributes
  no additional width breakpoint and performs no search for rounded-projection
  transition widths.
- Marginal: enumerate the Cartesian product of central, top-marginal, and
  right-marginal collections. Top-marginal width equals central width;
  right-marginal height equals central height. Preferred top height is
  `max(top_hard_height, ceil_div(central_height,4))`; preferred right width is
  `max(right_hard_width, ceil_div(central_width,4))`. Arrange a two-by-two Grid:
  top marginal at `(0,0)`, central at `(1,0)`, right marginal at `(1,1)`, and
  one empty slot at `(0,1)`, with traced marginal gaps. Exact remeasurement makes
  top and central widths equal and right and central heights equal.
- Matrix: retain the complete declared variable square. A column width is the
  maximum complete hard width of its cells; a row height is the corresponding
  maximum. A cell's preferred square side is
  `max(hard_width, hard_height, 12em)`. A column's preferred pitch is the
  maximum preferred square side of every cell in that column; a row's preferred
  pitch is the corresponding row maximum.
- PointInterval, Coefficient, Forest, Dumbbell, HorizontalBar,
  StackedHorizontalBar, Beeswarm, and Waterfall use the categorical producer on
  height and standard producer on width.
- Bar, StackedBar, Boxplot, Violin, Raincloud, and Interaction use the
  categorical producer on width and standard producer on height.
- Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath, and Regression
  use the standard producer in both dimensions.

The ordinary families in the first two mapping bullets have component type
`categorical_interior`; Beeswarm and Waterfall instead use their named specialized
types. Families in the third bullet have type `standard_interior`. Heatmap,
Marginal, and Matrix use their named types. This is the exhaustive ordinary-
versus-specialized interior type mapping.

Heatmap and Matrix use this exact shared track producer. Before aspect expansion,
column set `B_x[c]` contains that column's hard and preferred pitch, and row set
`B_y[r]` contains that row's hard and preferred pitch. For every declared cell
`(r,c)`, preferred `1:1` expansion adds each immutable pre-aspect `B_x[c]` value
to `B_y[r]` and each immutable pre-aspect `B_y[r]` value to `B_x[c]`; expansion
runs once, then every set sorts and deduplicates. Hard pitches remain track
minima. For a nonexact axis, enumerate the complete Cartesian product choosing
one value from each track set in ascending track order. For an exact axis, replace
that product with the complete receiver-then-balanced exact-total residual vectors
over those same sets. Combine x vectors first and y vectors second; outer size is
the checked track sum plus declared internal gaps and insets, and interval
filtering applies to that outer size. Each admitted pair arranges every prepared
cell in its row/column rectangle and stores its exact mark geometry. Matrix
density quality is calculated from those final cell geometries and adds no
separate density-vector radix. These vector products and rejected residual
attempts are the complete Heatmap/Matrix candidate and accounting source.

Waterfall's categorical body deterministically stores one bar per category and
one connector per adjacent category pair; those are topology, not alternatives.
Its admitted base and terminal total labels occur in that order. For the labels
that are in the final domain, enumerate every dense row-index vector with values
in `0..label_count`, row zero present, and no unused row below a used row. Sort
vectors lexicographically, reject a vector when outward label intervals in one
row overlap after the traced horizontal gap, and derive top-band height as
checked top/bottom padding plus row line heights plus traced inter-row gaps.
When `label_count == 0`, the sole vector is `[]` and its top-band height is zero.
For a nonempty vector, row zero is nearest the categorical body and increasing
rows proceed upward. For each used row `k`, let `BA[k]` be the maximum selected
label first baseline and `BD[k]` the maximum label height minus that baseline;
`row_height[k]=BA[k]+BD[k]`. With `body_top` as the categorical-body top edge:

```text
cursor = body_top - bottom_padding
for k = 0..used_row_count:
    row_bottom[k] = cursor
    row_top[k] = cursor - row_height[k]
    common_baseline[k] = row_top[k] + BA[k]
    cursor = row_top[k] - inter_row_gap   # except after the last row
band_top = row_top[last] - top_padding
```

Each label centers horizontally on its exact Waterfall shelf target and translates
vertically so its exposed first baseline equals `common_baseline[k]`. Horizontal
overlap testing uses those final translated ink intervals; the row's vertical ink
is contained by its `BA/BD` box. The plan/signature stores the row-index vector,
row edges, common baselines, and every final label translation. This is the sole
row order and geometry; no centered/first-fit baseline variant exists.
Category value labels remain inside their categorical slots and contribute to
that slot's hard pitch. The top band, category-label extents, deterministic
connectors, and ordinary outside-guide measurements are the complete Waterfall
reserve and shelf inputs; no first-fit row, fixed mark-panel floor, or draw-time
reserve is admitted.

No PlotKind may add another hard floor, preference, aspect, pitch, packing
alternative, marginal ratio, or specialized breakpoint outside these rules.

### Canonical Traversal And Type Order

A structural address is a sequence of `(role_discriminant, slot_index)` pairs.
Role discriminants use this declaration order:

```text
root, root_title, root_subtitle, root_body, root_caption,
plot_body, canvas_body, canvas_grid, facet_grid, canvas_shared_legend,
canvas_header, canvas_row_title, canvas_range_group, canvas_cell,
facet_header, facet_row_title, facet_range_group, facet_cell,
data_panel,
panel_interior,
specialized_interior, mark_layer,
mark_item,
heatmap_cell, heatmap_merged_row, heatmap_value_label, heatmap_highlight,
beeswarm_category, beeswarm_point,
marginal_central, marginal_top, marginal_right,
matrix_cell, matrix_mark,
categorical_category, categorical_series_slot, categorical_body_mark,
categorical_interval, categorical_value_label,
waterfall_bar, waterfall_connector, waterfall_base_label,
waterfall_terminal_label,
legend_title, legend_entry, legend_key, legend_label,
guide_title, guide_gradient, guide_label, guide_missing_block,
surface, text_panel_content, text_panel_title, text_panel_body,
inside_guide,
annotation, annotation_target, other_overlay,
axis_top, axis_right, axis_bottom, axis_left,
axis_title,
outside_guide_top, outside_guide_right, outside_guide_bottom,
outside_guide_left, outside_guide_auto,
text, glyph
```

Closed component type discriminants use this declaration order:

```text
empty_slot, canvas, plot, plot_body, plot_frame, canvas_body, canvas_grid,
facet_grid, track_set,
data_panel, standard_interior, axis, discrete_legend, continuous_guide,
composition_text, text_panel, text_panel_content,
legend_entry,
annotation_layer, overlay_layer,
heatmap_interior, beeswarm_interior, marginal_interior, marginal_subpanel,
matrix_interior, categorical_interior, waterfall_interior,
mark_layer, text, glyph
```

PlotKind discriminants use their public declaration order:

```text
Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory,
LinePath, Interaction, Coefficient, Forest, Dumbbell, Heatmap, Bar,
HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin,
Raincloud, Regression, Marginal, Matrix, Beeswarm, Waterfall
```

The Plot-root component has type `plot`; the Canvas-root component has type
`canvas`. `root` is only a structural role in an address and is not a third
component type.

Traversal is canonical preorder. Canvas and facet coordinates are row-major,
including declared empty and trailing slots. A spanning header, row title, range
group, or cell occurs exactly once at its minimum covered row and minimum covered
column; covered coordinates do not repeat it. Within one origin coordinate,
roles are header, row title, range group, then cell, with declaration order as
the final tie. That ordered list assigns the unique `slot_index`; the complete
row/column span remains in the signature and topology census. Plot preorder
visits title, subtitle, Plot frame, then caption. Within the Plot frame, preorder
visits the Plot body and each data-panel node with its complete standard-or-
specialized panel-interior, mark-layer,
inside-guide, annotation, and other-overlay subtree first; it then visits top,
right, bottom, and left axes followed by top, right, bottom, and left outside-
guide track sets, then directly owned automatic guides. A data-panel child is not repeated as a Plot sibling.

For every other repeated role, `slot_index` is the zero-based index after that
parent's children are put in the exact producer/traversal order in this section:
source-row order for marks, declaration order for layers and annotations,
semantic order for entries/categories, target-address then declaration order for
resolved overlays, and grapheme order for text.
Singleton roles use zero. No allocator, map iteration, or post-filter compaction
assigns a slot index.

Exact and generated tick marks and labels are candidate-local records within one
`axis` component, not independent components. They have no structural address or
topology slot. Their payload order is ascending exact position/order key and then
label bytes, and that complete ordered payload is retained in the axis plan and
structural signature.

Specialized interiors traverse as follows:

- Heatmap: cells row-major by semantic row then column, including missing cells;
  merged-row records by ascending row; value labels row-major; highlights
  row-major.
- Beeswarm: categories in semantic order, then points by source-row index.
- Marginal: central panel, top marginal, right marginal.
- Matrix: cells row-major by variable row then variable column, including
  diagonal cells; marks within a cell use source-row order.
- Horizontal categorical: categories in semantic order, series slots in
  semantic order, then body mark, interval, and value label.
- Waterfall: categories in semantic order, bars in category order, connectors
  by lower adjacent category index, then base label, terminal label, and category
  value labels.
- Legend: title, then entries in semantic order; each entry visits key before
  label.

Each Heatmap, Beeswarm, Matrix, categorical, or Waterfall item named
by those specialized traversals appends its corresponding declared role and
semantic/source-order slot to its owning specialized component address. Generic marks use `mark_item`; generic categories
and series use the categorical roles. These are prepared record addresses and
consume topology slots, but they are not independently measured components and
therefore add no component-type discriminant. Legend title, entry, key, and label
roles instead address the independently measured typed children defined by the
composition mapping below. Marginal central, top, and right roles likewise
address measured `marginal_subpanel` children in that declared order. Every measured component pairs its structural address
with exactly one closed component type above. Candidate-local ticks, line breaks,
flow slots, packing vectors, and residual vectors remain payload records without
structural addresses as stated here and below.

Ordered mark layers retain declaration order; marks within a layer retain
source-row order. Text uses grapheme order. Normal-flow content precedes overlays.

Within one component, alignment guides sort by axis `x,y`, guide kind
`start,baseline,center,end`, then target structural address, and exact duplicates
deduplicate. Aspect vectors use the pair order defined above. These intra-
component orders precede the enclosing component's preorder position.

Shared-alignment producers are closed. Each declared Canvas range group creates
one group per shared axis over exactly the occupied member cells in that range.
A fixed facet x or y policy creates one corresponding group over every facet
cell; a free dimension creates none. For member cell edges `(cell_start,
cell_end)` and its arranged data-panel edges `(panel_start,panel_end)` on that
axis, emit:

```text
leading_extent  = panel_start - cell_start
trailing_extent = cell_end - panel_end
```

Negative values reject hard fit. The `start` guide owns `leading_extent`; the
`end` guide owns `trailing_extent`. The group target is the maximum leading and
maximum trailing extent over members in canonical address order. Its quality
entries are `abs_diff(member_leading,target_leading)` then
`abs_diff(member_trailing,target_trailing)` for every member. These are the sole
shared `alignment_error_vector` entries and the exact values carried by alignment
feedback. Local sequence baselines, centers, and outside-guide alignments affect
arrangement/signatures but emit no convergence alignment request. With no declared
shared group, the alignment vector is empty.

For one component, raw alternatives combine in this exact mixed-radix decision
order, with the first decision point most significant:

1. producer-independent Leaf dimension radix, or composite
   `DimensionDescriptor`, in its order above;
2. component discrete seed in its producer order, including specialized packing
   alternatives at the producer-independent dimensions;
3. producer-dependent dimension choice in ascending numeric order, using one
   no-op choice when the producer has no dependent dimension;
4. base child candidate at each child address in canonical preorder, with
   candidates in complete-signature order;
5. Grid column natural/residual vector followed by Grid row natural/residual
   vector, each in receiver-then-balanced order when exact and otherwise the sole
   natural-track vector; a sequence has only its main-axis vector;
6. assigned-box measurement candidate for each child in canonical address and
   complete-signature order.

Beeswarm therefore orders width, category-packing seed, then that seed's derived
height. Its complete two-dimensional size is not formed before packing. For a
two-axis exact ordinary Grid, every column-vector attempt contains the complete
row-vector stream before the next column vector; Heatmap and Matrix use the same
x-before-y order.

`raw_descriptor_ordinal` is the zero-based position in the conceptual stream
formed by that nested order before deduplication. For each dimension choice,
seed blocks occur in producer order. A seed rejected before it produces child,
track, or assigned-box tuples contributes one singleton rejection block at that
seed's position; valid seeds contribute their complete suffix product before the
next seed. Rejected track/residual prefixes use the same singleton-block rule at
their position within a valid seed. This rule assigns every pre-tuple rejection
one exact ordinal without inventing absent suffix choices.

`descriptor_count` is exactly the number of blocks in that conceptual stream
after decisions 1 through 5 and before decision 6 assigned-child candidate
expansion. A recoverably rejected prefix at any earlier decision contributes one
singleton descriptor block; a valid dimension/seed/dependent-dimension/base-child/
track-vector prefix contributes one descriptor block. `evaluation_count` is one
for each rejected block plus, for each valid block, the assigned-child Cartesian
cardinality or its one empty/unsatisfied rejection. Residual prefixes and
singleton rejection blocks therefore count once, while assigned-child
combinations do not increase `descriptor_count`. The pass-local lower bound and
cached accounting tuple use this definition.

Aspect-induced lengths participate only through the sorted width and height
sets. For base child collections of lengths `c[0]..c[n-1]`, one pre-track
descriptor has raw combination count `product(c[i])`. Each combination then
multiplies by its raw natural/residual-vector count and by the returned assigned-
box child-collection lengths. Counts sum across all preceding descriptor
choices. The complete checked count must fit the remaining candidate budget
before that cache miss is evaluated. Discovery order never affects the returned
sorted collection or selected root.

### Closed Composition Grammar

All ordinary and composite candidates use the following grammar. Layout uses a
local upper-left origin, positive x to the right, and positive y downward.
Candidate boxes store ordered `(x0,y0,x1,y1)` quantum edges. Backend coordinate
conversion is a later frozen-plan operation.

The closed composition primitives are:

- `Leaf`: no child. Its closed producer declares its applicable dimension radix:
  text uses one width transition and derived height; fixed glyph/geometry leaves
  use their one intrinsic size; standard and categorical interiors use their
  declared sets; and width-dependent specialized leaves order each completed
  panel width, its packing vectors, then each vector's derived heights. A Leaf does not add an unused width/height
  Cartesian radix.
- `VerticalSequence`: ordered normal-flow children separated by traced gaps.
- `HorizontalSequence`: the corresponding left-to-right sequence.
- `Grid`: ordered row and column tracks, gaps, and child origin/span records.
- `Overlay`: children share one containing box and use their typed anchor or
  collision alternative; they do not create normal-flow tracks.

The component mapping is exact:

| Component | Primitive and children |
| --- | --- |
| Empty slot | `Leaf`; one zero-size candidate |
| Plot root | `VerticalSequence`: title, subtitle, Plot frame, caption |
| Plot body | Transparent one-child `Overlay`: one fill data panel when unfaceted or one fill facet grid when faceted, fixed at `panel_top_left` |
| Plot frame | `Grid`: Plot body; top, right, bottom, and left axes; four typed outside-guide track sets; then directly owned automatic guides routed by reference |
| Canvas root | `VerticalSequence`: root title, subtitle, Canvas body, caption |
| Canvas body | `Grid`: Canvas grid plus the optional shared Canvas legend at its selected edge |
| Canvas grid | `Grid`: declared header/row-title components, occupied or empty cells, and trailing slots; range groups are track-sharing records |
| Facet grid | `Grid`: declared strip/row-title components, facet-cell Plot frames, shared axes/guides, and empty/trailing slots; range groups are track-sharing records |
| Composition header/row title/strip | `Leaf`; the complete text producer under `composition_text` |
| Discrete legend | `VerticalSequence`: optional title then one `Grid` from the selected legend alternative |
| Legend entry | `HorizontalSequence`: one key glyph then one label text component |
| Continuous guide | `Leaf`; one exact orientation-selected inline title/gradient/labels/missing-block producer |
| Text panel | `Overlay`: one inline fill surface record, then one nonfill text-panel-content child |
| Text-panel content | `VerticalSequence`: required title text then required body text |
| Text | `Leaf`; the complete text producer |
| Axis | Size-dependent side-selected perpendicular `Grid`; exact assigned panel span regenerates tick sources, tick and label layers are projected `Overlay`s, and total thickness is the required gutter |
| Data panel | `Overlay`: standard or specialized panel interior first, then ordered mark layers, inside guides, annotations, and other overlays |
| Outside-guide track set | Top/bottom `VerticalSequence` or left/right `HorizontalSequence` of explicitly placed guide children plus routed references to Plot-frame-owned automatic guides; component type is `track_set` |
| Standard interior | `Leaf` under the exact standard-panel producer |
| Categorical interior | `Leaf` under the exact one-categorical-axis/one-standard-axis producer |
| Heatmap interior | `Grid` under the exact Heatmap row/column producer |
| Marginal interior | Two-by-two `Grid` under the exact Marginal producer |
| Marginal subpanel | `Leaf`; one standard-dimension collection with its exact prepared marks |
| Matrix interior | `Grid` under the exact Matrix variable-square producer |
| Waterfall interior | `Leaf`; the exact producer stores deterministic body/connector records and complete total-label row alternatives inline |
| Other specialized interior | `Leaf` under its exact specialized producer |
| Mark layer | Size-dependent `Leaf` under the exact owning-panel sets and prepared mark producer |
| Glyph | `Leaf`; one traced intrinsic-geometry candidate |
| Annotation or other overlay | `Leaf` under its closed typed anchor/collision producer |

The Plot root's `root_body` child is the `plot_frame`; the Canvas root's
`root_body` child is `canvas_body`; a facet cell's component at its
`facet_cell` address is also a `plot_frame`. A Plot frame's `plot_body` child has
type `plot_body`. Its unfaceted child uses role `data_panel` and type `data_panel`;
its faceted child uses role and type `facet_grid`. Canvas body uses role and type
`canvas_grid` for its Grid and `canvas_shared_legend` for its optional shared
legend/guide. Present Plot/Canvas root title, subtitle, and caption children at
their named roles have type `text`; an absent role has type `empty_slot`. Every
present root text plan carries the same owner-local
`root_title_block_alignment`. Each present axis title has role `axis_title` and
type `text`, while tick marks and labels remain the candidate-local axis payload
defined above. The four
`outside_guide_top,right,bottom,left` children have type `track_set` in that
order. A discrete legend's optional `legend_title` is a measured
`text` child and each `legend_entry` is a measured `legend_entry` child whose
`legend_key` and `legend_label` children have type `glyph` and `text`. Legend
flow changes only the measured entry boxes' Grid cells. A continuous guide has no
measured child: `guide_title`, `guide_gradient`, `guide_label`, and
`guide_missing_block` are inline addressed records generated and counted by its
Leaf producer. Thus no legend or guide record is both inline and independently measured.

At a data-panel address, `annotation` children have type `annotation_layer` and
`inside_guide` or `other_overlay` children that are not typed legends/guides have
type `overlay_layer`. Mark layers have type `mark_layer`. Their prepared marks,
annotation targets, anchors, Waterfall bars/connectors, Heatmap/Matrix cells, and
other specialized traversal items are candidate-independent topology records
inside the owning measured component, not child measurement calls. Their finite
geometry alternatives are exactly the owning producer's seed radix.

An `annotation_layer` or `overlay_layer` is a size-dependent `Leaf`. It has no
measured text child. In prepared-record traversal order it evaluates the inline
raw text streams above, pairs each admitted text choice with the record's exact
glyph/stroke/bracket geometry, then enumerates that owner's fixed
`OverlayAnchor` subsequence. The resulting outward ink rectangle is the Leaf
candidate size; call constraints filter it, while assigned panel constraints
remain the containing box used only by projected/attached/shelf anchor geometry.
The owning component stores every selected text line, baseline, anchor, ink edge,
and serialization input in its plan/signature. This text-then-anchor stream is the
complete panel-text, attached-label, significance, and generic-overlay candidate
and accounting source.

The structural-signature `optional_PlotKind_discriminant` is `Some(kind)` exactly
for a component whose nearest root owner is a Plot, including facet-cell
descendants and Plot children embedded in a Canvas. It is `None` for Canvas-owned
components before entering a child Plot, including Canvas chrome, Grid, and a
shared Canvas legend/guide.

Each `canvas_header` and `canvas_row_title` Grid child has type
`composition_text`. Each `facet_header` and `facet_row_title` child has the same
type under the owning Plot theme. A `canvas_range_group` or `facet_range_group`
address is a prepared nonvisual topology record that names exact covered tracks
and contributes no child collection or box. At `canvas_cell`, an occupied value
has type `plot` for an internally erased Plot or `text_panel` for a `TextPanel`;
an unoccupied declared/trailing coordinate has type `empty_slot`. A TextPanel's
`text_panel_content` child has type `text_panel_content`; its
`text_panel_title` and `text_panel_body` children have type `text`. The inline
surface has the TextPanel address plus a `surface` record suffix, returns zero
natural size, and fills the final TextPanel box after its nonfill content derives
the natural minimum. An exact assigned TextPanel box therefore adds blank chrome
rather than forcing residual into text children. Under a nonexact call the outer
TextPanel uses each content natural size; under an exact axis it emits that exact
outer extent when the content fits and rejects otherwise. The content child owns
the fixed `panel_top_left` anchor after the traced TextPanel chrome inset; this is
its sole Overlay placement and is stored in its plan. At
`facet_cell`, the component has type `plot_frame`; its data and axes remain under
the owning Plot root rather than creating a nested Plot component. These mappings
are exhaustive for Canvas/facet Grid children.

The Plot frame has five row roles in order `outside_top,axis_top,panel,
axis_bottom,outside_bottom` and five column roles in order
`outside_left,axis_left,panel,axis_right,outside_right`. The Plot body occupies
the central cell. An unfaceted body contains one data panel. A faceted body
contains one facet Grid whose cells each contain one Plot frame and central data
panel. For a faceted Plot only, the root Plot-frame axis roles are empty; an
unfaceted Plot retains its axes and outside guides there. For a fixed facet dimension, the
facet Grid owns one shared outer axis/guide in the same five-by-five frame grammar
around its cell Grid and the corresponding cell-frame axis roles are empty. For
`FreeX`, `FreeY`, or `Free`, each free dimension instead has one panel-local
domain and the corresponding axes/guides in every facet-cell frame; that
dimension's shared outer roles are empty. Cell-frame addresses nest under the
row-major `facet_cell` address. A horizontal axis or guide spans only the central column; a
vertical axis or guide spans only the central row. Inside guides, annotations,
and other overlays use the central data-panel box. An absent role contributes
the one zero-size empty-slot candidate. No frame role, track, or span is inferred
from iteration order.

Canvas body uses the same five-by-five outer frame with the Canvas Grid in its
central cell and, under shared-legend policy, exactly one Canvas-owned discrete
legend or continuous guide in the edge track selected by its candidate. Hidden policy
uses an empty slot; per-Plot policy leaves the Canvas-body legend slot empty and
keeps each legend in its Plot body. The shared legend has one
`canvas_shared_legend` address, Canvas-theme chrome, and the exact prepared entry
and glyph identities required by `03_THEME_SYSTEM.md`; automatic edge routing
uses the stable-address rule above.

For an initial sizing call, compare each finite parent maximum with the checked
leading-plus-trailing inset before subtraction. If the inset is larger, return
`Unsatisfied` with `constraint`, this component address, and sentinel ordinal zero;
perform no child call, construct no descriptor, and charge no parent evaluation.
This pre-descriptor path cannot coexist with an ordinary descriptor stream in the
same call. An invalid negative `LengthConstraint` is never constructed. Otherwise
each child minimum is zero and its finite maximum is the checked difference;
sibling sizes are not guessed or subtracted. A `None` parent maximum remains
`None`. After a parent descriptor derives one child cell, it performs one assigned-box
measurement call for every child in canonical address order. A fill dimension
uses equal minimum and maximum at the cell extent. A nonfill dimension uses zero
minimum and the cell extent as finite maximum; the returned size is aligned
inside that cell. The closed fill policy is:

- Plot frames, data panels, standard or specialized panel interiors, mark layers,
  Plot bodies, Canvas bodies, Canvas/facet Plot cells, and explicit surface or
  background layers fill both dimensions; Canvas `TextPanel` cells also fill both
  dimensions so their chrome occupies the assigned cell;
- horizontal axes and top/bottom outside-guide track sets fill width; vertical
  axes and left/right outside-guide track sets fill height. An individual discrete legend or
  continuous guide inside that full-span track is nonfill on the panel-parallel
  axis and uses its retained outside alignment. Within a continuous guide, only
  the inline gradient record fills the guide's assigned gradient-axis slot after
  the selected label combination derives its exact length;
- text, glyphs, legend entries, individual legends/guides, annotations, and other
  overlays do not fill;
- `empty_slot` is nonfill in both dimensions and always returns its sole zero-size
  candidate;
- a Grid or sequence fills a dimension exactly when its parent slot fills that
  dimension.

Every assigned-box call is an ordinary keyed child measurement call and is
charged or reused by the cache rules. Its returned collection, not the initial
sizing collection, supplies the child plan to the parent candidate. It does not
add an assigned or residual length to the child's base breakpoint set. Equality
with the initial constraints produces an ordinary cache hit only when that
component has no assigned geometry context. An annotation/overlay initial call
has no such context and returns complete anchor-local size/text alternatives;
its assigned call receives the parent descriptor's context and finalizes exact
projected, attached, and shelf geometry. It does not reject against sibling
obstacles.

For a sequence, an absent optional role remains an addressed `empty_slot` child
but is removed only from gap adjacency: emit one traced gap exactly between each
pair of consecutive present non-empty children and no leading, trailing, or
empty-adjacent gap. The resulting active gaps are `g[i]` below and retain their
source relationship in the signature. Grid row/column gaps instead belong to the
declared track topology and remain present between consecutive declared tracks
even when every cell in one track is empty.

For child sizes `(w[i],h[i])`, traced nonnegative insets `(l,t,r,b)`, and traced
gaps `g[i]`, let `BA` be the maximum stored baseline among baseline-aligned
horizontal-sequence children, `BD` the maximum `h[i]-baseline[i]` among those
children, and `N` the maximum height of non-baseline-aligned children. Empty
maxima are zero and `H_cross=max(N,BA+BD)`. Sequence natural sizes are exactly:

```text
vertical_width  = l + max(w[i]) + r
vertical_height = t + sum(h[i]) + sum(g[i]) + b

horizontal_width  = l + sum(w[i]) + sum(g[i]) + r
horizontal_height = t + H_cross + b
```

The empty maximum and sums are zero. Main-axis boxes accumulate from the leading
inset with each exact gap. Cross-axis `start` uses the leading inset, `end` uses
the containing edge minus the child extent, `center` uses category-6 nearest-
even half residual, and `baseline` places a child at `t+BA-baseline[i]`. Thus the
common baseline is exactly `t+BA` and its declared box contains every aligned
child. A negative geometric residual rejects the candidate; checked
arithmetic overflow aborts the root operation under the numeric contract.

Child alignment is a fixed renderer invariant except for the one resolved
`CompositionTheme` root title-block alignment. Neither fixed nor themed
alignment is a candidate radix. Present root title, subtitle, and caption
children use that value on the horizontal cross axis. A `composition_text`
header, row title, or facet strip centers in both axes of its spanned Grid cell.
A legend-entry `HorizontalSequence` baseline-aligns its key and label; any other
horizontal sequence baseline-aligns exactly the children that expose a baseline
and start-aligns children without one. Axis titles and projected labels use their
explicit axis-centering rule, and typed overlays use their anchor rule. Every
remaining nonfill sequence/Grid child uses `start` on each nonfill axis. Fill
axes normalize their stored alignment decision to `start`. No normal-flow
component selects alignment by search.

For an exact sequence main axis, treat every nonfill child extent and every gap
as `G`, and every fill child as one flexible track under "Exact-Total Residuals".
Enumerate all receiver and balanced vectors, remeasure fill children at their
exact assigned main-axis lengths, and combine every returned candidate. On the
cross axis, exact parent space remeasures fill children and aligns nonfill
children by the rule above. Plot frame and Canvas body are the sole flexible
main-axis children of their root vertical sequences; ordinary root title,
subtitle, and caption children are inflexible.

Axis child order is side-specific. Top and left visit title, labels, ticks, then
stroke from the outer edge toward the panel. Bottom and right visit stroke,
ticks, labels, then title from the panel toward the outer edge. The axis's
orientation selects vertical versus horizontal perpendicular tracks; its side
selects this direction, so no backend reversal is inferred. The axis-parallel
track is exactly the assigned data-panel span. The axis stroke occupies that
span. Assigned measurement regenerates every no-interval or caller-interval
source for that exact span and returns every fitting alternative; it does not
select a tick list or alter the containing panel. Each tick center is the final
category-6 center from the tick-projection
formula; its tick glyph and shaped label are positioned in separate Overlay
layers at that center, with traced tick length and tick-to-label gap in the
perpendicular direction. Axis title placement is centered over the assigned
span. Top/bottom titles have rotation zero, left titles rotate exactly `-90`
degrees (bottom-to-top), and right titles rotate exactly `90` degrees
(top-to-bottom) in the downward-positive coordinate system. Shape the title
unrotated, then transform every ink corner around its centered baseline anchor by
the exact quarter-turn `(x,y)->(y,-x)` or `(x,y)->(-y,x)`; no trigonometric
operation occurs. Left/right flow width/height and ink extents are therefore the
exact swapped/transformed values. The frozen plan stores angle, anchor, and
transformed ink. Canonical SVG emits `rotate(-90 x y)` or `rotate(90 x y)` using
the exact coordinate spelling, and Null geometry consumes the same transformed
corners. The maximum outward perpendicular label extent, plus ticks, gaps, stroke,
and title tracks, is the exact gutter thickness. Labels never form an axis-
parallel sequence.

An outside-guide track set places its first canonical guide nearest the panel and
each subsequent guide outward, independent of side. The top/left geometric
sequence is therefore reversed when boxes are assigned; bottom/right uses
forward assignment. Guide dimensions sum in the perpendicular direction rather
than sharing one Plot-frame track by maximum. An `outside_guide_auto` candidate
is referenced by only the selected edge's track set and keeps its original
structural address and slot index.

For one Grid child combination, initialize each track to its traced hard minimum.
Visit single-track children in canonical address order and raise their track to
the maximum required child extent. Then visit spanning children by increasing
span length and canonical address. For a child whose required extent exceeds its
covered track sum plus internal gaps, distribute the shortfall over covered
flexible tracks by quotient and remainder; the first remainder tracks in track
order receive one additional quantum. A finite track maximum is capped and
removed before repeating. Shortfall with no remaining headroom rejects the
combination. This procedure runs columns before rows. A track is flexible exactly
when at least one child filling that dimension occupies it; every other track is
inflexible.

Canvas and facet coordinate Grids have one closed exception for declared sparse
topology. Every declared row and column, including wholly empty and trailing
tracks, is noncollapsing and flexible. Its renderer-owned hard minimum is one
quantum and its preferred minimum is `1em`, resolved from Canvas composition text
for Canvas and facet/header text for facets. Its pre-content `B[i]` is exactly
those two values, sorted/deduplicated; occupied child requirements raise each
chosen value by the ordinary Grid procedure. A nonexact composition Grid
enumerates the complete Cartesian product of these track choices in column-then-
row mixed-radix order and deduplicates resulting vectors. An exact composition
Grid uses the same completed sets in receiver-then-balanced residual allocation,
so empty tracks participate rather than collapse. Empty-slot children remain
zero-size and add no second minimum. No other Grid treats an unoccupied track as
flexible.

Each distinct resulting natural flexible-track value enters that track's
completed breakpoint set. Except for the declared composition-track product
above, a nonexact Grid uses the natural vector and does not
fill unused bounded space. An exact Grid includes inflexible track extents in
`G` and enumerates the receiver and balanced vectors from "Exact-Total
Residuals" over only the flexible completed sets. For every natural or exact
vector, grid lines are cumulative checked sums of leading inset, tracks, and
gaps. A child's assigned cell is the rectangle from its origin through its
complete span. Fill-child exact remeasurement and nonfill alignment then produce
the candidate-local child boxes.

An Overlay is measured under its supplied box constraints. Under nonexact
constraints, its first underlying child and traced hard/preferred panel
requirements determine each containing-box candidate. Under exact constraints,
the supplied dimensions are the containing box. Plot-frame fill remeasurement
therefore supplies each Data panel's exact candidate box before its overlays are
combined. Each overlay child owns and enumerates its typed anchor alternatives
exactly once in its returned collection and may test static prior-pass obstacles;
it never receives sibling obstacles. The Overlay parent adds no
second anchor radix. The parent combines those child candidates under the
containing-box maxima and assigns their provisional boxes from the anchor stored
in each child plan using the start/end/center/baseline formulas. A collision,
ink, or domain consequence is stored in that parent candidate and, if selected,
transfers only through the closed convergence rules; it does not mutate another
already-evaluated candidate.

The closed `OverlayAnchor` discriminant order is:

```text
projected, panel_center, panel_top, panel_right, panel_bottom, panel_left,
panel_top_right, panel_bottom_right, panel_bottom_left, panel_top_left,
attached_start, attached_end, bracket_shelf, waterfall_shelf
```

Each typed overlay owner admits a fixed subsequence: cell/point/value labels use
`projected`; panel text and inside legends use their configured panel anchors;
attached labels use `attached_start,attached_end`; significance uses
`bracket_shelf`; and Waterfall uses `waterfall_shelf`. Panel anchors align the
outward ink box to the named containing-box edge(s) after the traced inset.
`panel_top` and `panel_bottom` center on the containing box's x axis;
`panel_left` and `panel_right` center on its y axis; `panel_center` centers both
axes; and corners align both named edges. Each attached owner carries a closed
attachment axis `x` or `y`: `attached_start` places the box before the target
mark on that axis and centers it over the target mark on the orthogonal axis,
while `attached_end` places it after and uses the same orthogonal center.
`projected` centers the owner's typed text/glyph anchor at its final projected
data point plus traced offsets. Attached anchors place the ink box after the
target mark's start or end edge plus traced gap. A bracket shelf spans its two
resolved targets, starts above their maximum outward ink edge, and adds the
canonical level index times traced level step; its label is centered over the
span after the traced label gap. Waterfall shelf geometry is the complete row
producer above.

Significance levels are deterministic consequences, not alternatives. For each
complete selected significance-label/target combination, normalize a bracket's
axis-parallel span to `(start,end)` and sort brackets by
`(start,end,lower_target_address,upper_target_address,annotation_address)`. Its
occupied interval is the union of that span and its centered label ink interval,
expanded by traced horizontal clearance. Visit brackets in that order and assign
the smallest nonnegative level whose prior occupied intervals do not intersect;
touching after clearance is allowed. Each tested level claims one generation
work unit and the root resource limit bounds the procedure. Let
`level_step=max(traced_level_step,maximum_label_height + traced_label_gap +
bracket_tick_height)` for this combination. For a vertical-value bracket in the
downward-positive system:

```text
shelf_y[level] = minimum_target_ink_top - traced_data_gap - level*level_step
label_baseline = shelf_y[level] - traced_label_gap - label_descent
label_x        = round_ties_even_exact((start + end) / 2)
```

The horizontal-value orientation applies the exact x/y-swapped quarter-turn.
End ticks extend from the shelf toward each target by the traced tick height. The
union of bracket stroke and label ink is the reserve/collision rectangle. One
complete combination produces exactly this one dense level assignment; inability
to fit its reserve rejects that combination with `constraint`; an actual
anchor/obstacle intersection uses `overlay_collision`. No
unlisted level-order alternative exists.

For each admitted anchor, form the complete outward ink rectangle, then reject
it when it intersects any typed collision obstacle after required clearance.
Touching at exact clearance is nonintersecting. Anchors are never shifted,
scored, or retried outside their listed order. Multiple overlay children combine
their anchor candidates by the ordinary Cartesian protocol; their final
rectangles join the obstacle list for subsequent children in canonical address
order. For each complete Cartesian combination, the parent starts with static
prior-pass obstacles from its consumed frontier, then tests and appends children
sequentially. A sibling collision rejects that parent combination once and never
causes another child call or cache key. These rules are the complete generic anchor, collision, rejection, and
ordering producer.

For each primitive descriptor, the raw parent-combination count is the product
of the complete assigned-box child-collection lengths in canonical child order,
using the routed view length for an automatic legend or guide when every such
view is nonempty. The empty-view rejection below has cardinality one instead.
Counts sum over primitive alternatives, breakpoint pairs, and track vectors;
overlay alternatives are already represented by child-collection lengths and
are not multiplied again. Initial sizing and assigned-box child calls are counted
independently before their parent combination. These formulas, the closed table above, and the
specialized producers are the complete source of ordinary/composite candidates;
no component may invent another constraint derivation, track allocator, box
placement rule, or candidate source.

Each ordered parent descriptor has exactly one of three cardinalities:

- a descriptor rejected before assigned boxes can be formed contributes one raw
  rejected parent evaluation at that descriptor ordinal and performs no assigned-
  box call;
- a descriptor for which any assigned child returns `Unsatisfied`, or whose
  required automatic-guide routed view is empty, contributes one raw rejected
  parent evaluation at that descriptor ordinal;
- otherwise its cardinality is the checked product of all assigned-child
  collection lengths, enumerated in canonical mixed-radix order.

Rejected residual/track attempts retain a count-only descriptor ordinal even
though they do not enter the admitted vector collection. Once assigned boxes can
be formed, every assigned child call executes in canonical address order even
after an earlier child returned `Unsatisfied`; a later fatal child error therefore
wins, while no fatal outcome leaves the enclosing descriptor as the single
charged rejection. Summing these descriptor cardinalities is the exact complete
pre-deduplication count and preserves one deterministic failure order.

### Attempt Rejection And Root Gates

A counted raw attempt is rejected before candidate admission only when a producer
section explicitly rejects its seed/vector, its normalized size violates the
component call's constraints, an assigned box cannot be formed, or a required
child call returns `Unsatisfied`. Such an attempt has no partial plan and is not
returned in `MeasuredCandidates`. An enclosing parent descriptor with an
unsatisfied child is still one charged rejected parent evaluation; it does not
multiply by an empty collection length. Checked arithmetic failure is never an
attempt rejection; it aborts the root operation.

Every admitted candidate has a complete finite provisional plan. Root planning
then assigns gates without removing that plan: semantic cardinality or implicit
content loss sets `semantic_gate`; unowned out-of-root ink, collision,
hard-minimum, or representable-geometry failure sets `hard_fit_gate`; and a
complete nonmatching exact aspect sets `exact_aspect_gate`. Only the ordered root
filters remove those complete gate-one candidates. These are the exhaustive two
dispositions; an implementation cannot move a named raw producer rejection into
a root gate or discard a complete root candidate before its ordered gate.

Out-of-root normal-flow ink covered exactly by a valid reserve consequence under
the closed owner rule is provisionally owned rather than unowned: it does not set
`hard_fit_gate` on the pass that must transfer that consequence. This exception
does not authorize clipping or final overflow. Stable convergence additionally
requires every painted ink edge to lie inside the selected root box after the
consumed reserves are applied.

## Quality Objective

Bottom-up measurement does not imply one unique size. Legends can use different
row and column arrangements; text has different heights at different widths;
data panels are flexible; and a composed grid can admit several track sizes.
Only the root evaluator compares complete finite root candidates, using these
ordered decision tiers; non-root parents preserve every admitted combination:

1. Preserve all semantic content and exact caller policies without implicit
   clipping, elision, substitution, or omission.
2. Satisfy hard text, glyph, mark, annotation, cell, category-pitch, and
   representable-geometry requirements.
3. Honor exact scientific aspect requirements.
4. Among otherwise eligible candidates, retain the closest surviving caller
   tick intervals before evaluating any soft visual quality.
5. Honor preferred data aspect and shared alignment requirements.
6. Prefer readable line lengths, tick density, legend shape, mark density, and
   data-panel proportions.
7. Minimize excess output area and whitespace after the preceding requirements
   are satisfied.
8. Apply stable structural tie-breaking independent of iteration order, pointer
   identity, hash-map order, backend, or output format.

The root evaluator first applies eligibility and nonexact viability filters,
then minimizes this exact lexicographic tuple among surviving candidates:

```text
(
  semantic_gate,
  hard_fit_gate,
  exact_aspect_gate,
  exact_aspect_error_vector,
  tick_interval_rank(maximum, sum, preorder_vector),
  preferred_aspect_error_vector,
  alignment_error_vector,
  line_rank(maximum, sum, preorder_vector),
  tick_rank(maximum, sum, preorder_vector),
  legend_rank(maximum, sum, preorder_vector),
  density_rank(maximum, sum, preorder_vector),
  panel_rank(maximum, sum, preorder_vector),
  root_area,
  normal_flow_whitespace,
  root_width,
  root_height,
  complete_structural_signature
)
```

Each gate is zero or one:

- `semantic_gate` is zero exactly when every required semantic identity occurs
  once and no content is clipped, elided, abbreviated, substituted, or omitted
  without its explicit caller policy.
- `hard_fit_gate` is zero exactly when all constraints, hard minima, outward ink
  bounds, collision requirements, and representable-geometry requirements pass.
- For exact aspect `(a,b)`, an error entry is
  `abs_diff(width*b, height*a)`. `exact_aspect_gate` is zero exactly when every
  entry is zero.

A candidate survives gate filtering only when all three gates are zero. A
nonzero gate cannot become selectable even when every candidate has that same
violation; the operation returns the first-empty-frontier error instead.

Each root candidate stores
`EligibilityReason=(filter_ordinal,component_address,axis,violation,local_ordinal)`
records only for filters it has reached. Gate reasons for filters 1 through 3 are
complete during root planning. For filters 4 through 11, calculate and append the
current filter's complete reasons only for current survivors immediately before
applying that filter; never calculate a later canonical dimension, byte count, or
format reason for an already removed candidate. Filter ordinals are the exact
1-through-11 order under "Candidate
Eligibility And Nonexact Viability". Within filters 1 through 3, `violation`
uses this closed order:

```text
missing_semantic, duplicate_semantic, clipped_or_omitted_semantic,
transformed_semantic, constraint, hard_minimum, outward_ink, collision,
representable_geometry, exact_aspect
```

`axis` has closed order `none,x,y`; component-local/root-wide failures use
`none`, while axis-specific filters retain `x` or `y`. Filters 4 through 11 use
their filter identity as the violation and the root address. `local_ordinal` is the
canonical traversal position of the violated identity/requirement, or zero for a
root-wide check. At each reached filter, record every applicable current-filter
violation, sort by this tuple, and deduplicate. When one ordered filter empties the survivor frontier, return the
least reason at that filter among the candidates it removed. Stable-convergence
ink failure uses the same `outward_ink` reason. This is the exact owner-aware
layout-fit/representability diagnostic; gate bits alone are never returned as an
error.

For preferred aspect `(a,b)`, an error entry is:

```text
relative_error(width * b, height * a)
```

Alignment entries are the exact member-to-group extent differences defined by
the shared-alignment producer above. Empty aspect or alignment vectors are `[]`.
All vectors follow canonical preorder.

For each generated numeric axis carrying a `TickInterval`, its selected complete
source contributes its zero-based source ordinal in canonical axis preorder:
requested interval first, deterministic coarsenings next, terminal midpoint
last. `tick_interval_rank` is the raw
`(maximum,checked_u128_sum,preorder_vector)` of those ordinals, normalized to
`(0,0,[])` when no axis carries the policy. It is not densely reranked with soft
quality suppliers. Consequently, if any eligible root candidate retains every
requested interval, no candidate with a coarsened interval can beat it through
line, spacing, legend, density, panel, or area ranks. If no all-requested root
candidate survives, maximum then sum then canonical preorder selects the closest
joint coarsening before soft quality is considered.

For each rank family and canonical quality-supplier key, collect the raw local
tuples appearing among root candidates that survived gates and nonexact
viability. Sort and deduplicate those tuples lexicographically. The zero-based
index of each distinct tuple is its dense rank. Equal tuples have equal rank,
the best tuple has rank zero, and no unused rank occurs.

A supplier key is `(owning_component_address,supplier_kind,local_index)`, with
kind order `component,prepared_record,tick_or_guide_label`; local indices use the
owning producer's canonical text traversal. Every selected text block, including
inline prepared-record and candidate-local tick/guide text, supplies one Text
tuple under that key. Other rank families use their owning measured component as
local index zero. A candidate's preorder vector contains one rank for each supplier of the
family. An empty family normalizes to `(0,0,[])`. `maximum` is the greatest
vector entry or zero; `sum` is the checked `u128` sum.

The raw local tuples are exactly:

### Text

Let `line_width` be the greatest outward-shaped selected line advance,
`natural_width` the greatest unwrapped paragraph advance, and `finite_bound`
the exact finite width maximum supplied to that text subproducer when one exists.
For a measured text component it is the call constraint maximum; for inline text
it is the selected cell, band, anchor, or shelf width after its traced insets. An
unbounded inline owner supplies `None`. The same bound generated that block's raw
text stream; quality does not invent another wrapping width.

```text
excess_72ch =
    max(0, line_width - 72ch)

target_width =
    min(natural_width, 66ch, finite_bound) when finite_bound exists
    min(natural_width, 66ch)               otherwise

target_distance =
    abs_diff(line_width, target_width)

raggedness =
    sum((line_width - line_advance)^2)
```

Raggedness includes every line except the final line of each mandatory-break
paragraph. A paragraph with zero or one line contributes zero. The tuple is:

```text
(excess_72ch, line_count, target_distance, raggedness)
```

### Generated Ticks

Let `panel_span` be the axis-parallel panel span. Let `widest_label` be the
greatest outward axis-parallel label extent: width for a horizontal axis and
height for a vertical axis.

```text
spacing_target =
    max(6ch, widest_label + 1em)
```

For sorted projected centers and label intervals:

```text
center_gap[i] = center[i + 1] - center[i]
gap[i] =
    interval_start[i + 1] - interval_end[i]

spacing_error[i] = abs_diff(center_gap[i], spacing_target)
```

Overlap has already failed the hard gate. `gap_spread` is zero for fewer than
two gaps; otherwise it is `max(gap)-min(gap)`. For automatic generation without
a caller interval, `single_tick_gate` is one for a singleton and zero otherwise.
The raw tuple is:

```text
(
  single_tick_gate,
  maximum_spacing_error,
  sum_spacing_error,
  gap_spread
)
```

For no-interval generation, maximum and sum use the spacing errors above and
normalize to zero for fewer than two centers. For a caller `TickInterval`, the
first three fields are zero; its `gap_spread` stabilizes label alternatives from
one already-selected interval source, while source precedence is the earlier
root-level `tick_interval_rank`. An exact caller tick list has the sole tuple
`(0,0,0,0)` and is never compared with generated alternatives. Thus neither the
public policy nor the default renderer policy contains a target tick count.

### Legend Flow

For horizontal placement, `primary_line_count` is row count. For vertical
placement, it is column count.

```text
empty_slots =
    rows * columns - entry_count
```

A horizontal line size is one nonempty row's occupied width, including
inter-entry gaps but excluding title, outer padding, and border. A vertical line
size is the corresponding column height. "Occupied" is the bounding span of the
intrinsic assigned entry boxes plus only the gaps between nonempty slots; empty
trailing Grid tracks and distributed cell slack are excluded. `line_imbalance`
is zero for zero or one nonempty line; otherwise it is maximum line size minus
minimum line size.

```text
(primary_line_count, empty_slots, line_imbalance)
```

### Density

For `m > 0` primary marks:

```text
actual_mark_pitch =
    floor_sqrt(panel_width * panel_height / m)
```

`floor_sqrt(x)` is the greatest integer whose square is no greater than `x`.
For `m == 0`, actual mark pitch equals preferred mark pitch.

```text
shortfall(actual, preferred) =
    0                                 when actual >= preferred
    relative_error(actual, preferred) otherwise
```

Each data-panel address contributes exactly this local tuple:

```text
(
  shortfall(actual_mark_pitch, preferred_mark_pitch),
  x_pitch_shortfall_vector,
  y_pitch_shortfall_vector
)
```

For a categorical dimension, its vector has one entry per semantic slot in
semantic order, pairing that slot's actual track pitch with its preferred pitch.
For Heatmap and Matrix, the x vector has one entry per column and the y vector
one per row in track order. A noncategorical/noncell dimension contributes `[]`.
Beeswarm categories contribute the y vector. No minimum across tracks replaces
these entries. The density rank family orders these complete local tuples by
canonical data-panel preorder.

### Panel Proportion

```text
(
  shortfall(panel_width, preferred_panel_width),
  shortfall(panel_height, preferred_panel_height)
)
```

`root_area` is checked `root_width*root_height`. Direct root normal-flow child
boxes must be inside the root and pairwise nonoverlapping, so their exact union
is their checked area sum:

```text
normal_flow_whitespace =
    root_area - sum(direct_normal_flow_child_area)
```

The complete structural signature is an unhashed sequence in canonical preorder.
Every measured-component record compares fields in exactly this order:

```text
(
  canonical_address,
  component_type_discriminant,
  optional_PlotKind_discriminant,
  normalized_constraints,
  candidate_size,
  track_vectors,
  arranged_boxes,
  baselines,
  ink_edges,
  text_breaks_and_line_translations,
  tick_policy_discriminant_source_kind_and_raw_source_ordinal,
  tick_position_order_keys_and_label_bytes,
  legend_or_guide_edge_shape_and_slots,
  domain_order_keys,
  exact_and_preferred_aspect_inputs,
  preference_inputs,
  raw_rank_tuples,
  arrangement_decisions,
  overlay_decisions
)
```

Each addressed prepared topology record uses this one tagged-union field census,
with an empty sequence/`None` for fields inapplicable to its declared address
role:

```text
(
  canonical_address,
  semantic_type_discriminant,
  source_index_path,
  field_and_domain_order_keys,
  exact_content_utf8_bytes,
  optional_target_addresses,
  selected_presence,
  selected_face_file_id_and_index,
  selected_glyph_ids_clusters_offsets_and_advances,
  selected_text_breaks_translations_and_baselines,
  selected_layout_scalar_geometry,
  selected_nonspatial_f64_order_keys,
  selected_resolved_style_scalars_and_rgba_bytes,
  selected_role_decisions
)
```

Semantic discriminants use their owning typed enum declaration order;
source/index paths and addresses compare lexicographically as unsigned integers;
field/domain binary64 values use `f64_order_key`; content and face ids compare as
UTF-8/byte sequences; glyph ids are unsigned; all spatial geometry is
`LayoutScalar`; style scalar enums use declaration order and RGBA uses channel
order; role decisions use the explicit producer orders in this specification.
`selected_role_decisions` includes inline text raw alternative ordinal and local
quality tuple, anchor, packing/shelf,
visibility, and specialized alternative identity. `None` precedes `Some`, false
precedes true, signed integers compare numerically, and all sequences compare
elementwise then by length. No opaque semantic object, pointer, map, or
serializer-dependent blob occurs. The record has no component type, call
constraints, or independent candidate size. Candidate-local unaddressed records
occur only in the corresponding owning measured-component field. Thus every
topology record is represented exactly once without pretending it was a
measurement call.

Arrangement decisions use `Natural` before `Target`, `false` before `true` for
fill flags, and the declared `start,baseline,center,end` alignment order. Overlay
decisions use the declared `OverlayAnchor` order followed by obstacle address and
rectangle. Other enums use their declaration orders in this specification.
`None` sorts before `Some`; sequences compare elementwise and then by length.

Candidates deduplicate only when complete signatures are equal. Every quality
input occurs in the signature, so equal signatures necessarily produce equal
quality keys. The resulting key is a total order independent of discovery
order, pointer identity, allocation, map order, backend, and output format.

There is no universal `640x480` Plot candidate, `960x640` Canvas candidate, or
fixed per-cell multiplier in the target. A standard Cartesian panel instead
derives its finite readable preference from resolved typography, tick geometry,
mark extents, data density, and renderer-owned aspect policy. Specialized
families may add stronger typed requirements.

The quality key is a private deterministic renderer policy. It is
not a theme field, public callback, pluggable scorer, or backend-selected mode.

## Measurement Lifecycle

One public preflight or terminal operation performs these ordered stages:

1. Normalize every numeric input and root constraint.
2. Resolve Canvas and Plot theme ownership, text-role requirements, and scales;
   do not enumerate required scalars or host fonts yet.
3. Validate semantic content, data, exact caller policies, scientific
   requirements, domains, categories, facet membership, legend identities, and
   annotations; resolve every traced measurement input and calculate the
   canonical prepared-text/first-use-pair, topology, and initial-frontier payload
   censuses without retaining those collections.
4. After the topology limit, perform the work-charged required-scalar census,
   bounded font discovery, stage-1 reservations, and complete font/face/glyph
   resolution; materialize `MeasurementFrontier[1]` with empty prior-pass
   feedback, then measure leaves and composites under explicit
   `LayoutConstraints`; every
   successful call returns its complete `MeasuredCandidates` collection, with
   each candidate's local provisional arrangement and plan complete.
5. For every root candidate, complete root-level provisional arrangement,
   overlays, painted ink, collision, alignment, admitted domain consequences,
   and canonical serialization inputs; then apply ordered eligibility and
   nonexact viability filters. Canonical dimensions are calculated only at their
   filter, and canonical SVG bytes only for candidates reaching the SVG filter.
6. Construct dense ranks and total keys from all survivors and select one root
   candidate.
7. Construct the complete next-pass frontier and convergence state. If either
   differs from the immediately preceding complete state, return to stage 4
   with that next frontier.
8. After an unchanged state, promote the selected candidate's existing
   provisional boxes to final boxes, calculate its exact canonical SVG bytes if
   exact-size policy skipped that filter, reserve all final first-party storage,
   and freeze one immutable `PlotPlan` or `CanvasPlan` without layout
   recomputation.
9. Validate the frozen plan through one balanced Null-backend lifecycle and
   render that same plan through the concrete backend lifecycle for a terminal
   operation.

No provisional arrangement is renderable or final. Stage 7 is the sole feedback
edge. Final drawing does not measure, wrap text, select ticks, choose legend
flow, resize tracks, derive spacing, generate candidates, alter domains, or
perform layout accounting.

## Layout Input Ownership And Trace

The ownership ledger is interpreted broadly: one recorded property owner covers
the property's declaration or derivation and every measurement, generation,
convergence, accounting, arrangement, frozen-plan, and draw consumer of that
same value. Reuse of one owned value across stages does not create another
property identity.

Every layout input has one design-time trace containing:

- the ownership-ledger key or target-only layout key that owns it;
- its canonical component address;
- its source category;
- its validated or normalized value;
- every stage that consumes it;
- the immutable measured-plan or frozen-plan field that preserves it.

The exhaustive source categories are:

```text
caller semantic content and exact policies;
resolved theme typography, root-title-block alignment, spacing, stroke, and
glyph geometry;
font stack/weight pairs, canonical candidate identities, provenance, selected
faces, glyphs, advances, baselines, and ink metrics;
scale domains, categories, ticks, labels, and guide identities;
Canvas, facet, Matrix, Heatmap, and other declared topology;
mark, interval, cell, category, annotation, and collision geometry;
constraints, breakpoints, alternatives, aspects, residuals, and tracks;
overlay reserves, alignment requests, and admitted domain feedback;
candidate counts, topology counts, byte counts, and reservation capacities;
canonical dimensions and SVG serialization inputs.
```

The stage trace is exhaustive:

- measurement consumes constraints, prepared semantic content, resolved
  appearance, typography, scientific geometry, and child collections;
- candidate generation consumes alternative seeds, breakpoints, preferences,
  aspects, residuals, topology, and packing inputs;
- quality evaluation consumes gates, local tuples, ranks, dimensions,
  whitespace, and structural signatures;
- convergence consumes the selected candidate, all feedback consequences, and
  the complete next-pass frontier;
- accounting consumes every raw candidate count, topology count, plan capacity,
  state capacity, and byte count;
- candidate-local arrangement consumes each parent candidate's named child
  candidates, resolved gaps, alignments, tracks, and overlay decisions; final
  promotion consumes only the selected candidate tree;
- the frozen plan stores every position, size, text line, tick, legend slot,
  domain, style, glyph, mark geometry, clip, and overlay decision required by
  Null and concrete rendering.

A value used by one of these stages without such an owner and immutable trace is
an unowned layout input and blocks implementation. The trace is a normative
design and test census; it does not authorize a public property map, dynamic
dispatch, runtime string keys, or duplicate plan representation.

## Component Contracts

One context-free `intrinsic_size()` operation is not admitted. Measurement is
constraint-dependent and returns a complete nonempty typed candidate collection.

### Text

Text preparation resolves the exact face for every glyph before measurement.
The same resolved faces, weight, size, and line height are used for preserved
text geometry, outlined glyphs, and PNG rendering.

Text measurement distinguishes:

- minimum unbroken content width;
- natural unwrapped width;
- exact height and line breaks under a finite width constraint;
- first and last baselines;
- actual ink extent.

An unbreakable grapheme sequence that exceeds a finite width
causes a fit error. It is not clipped, replaced, or silently allowed to escape
the selected root.

### Axes And Data Panels

Axis measurement returns its complete exact or generated tick alternatives,
measured labels, collision intervals, fit transitions, minimum usable panel
span, and required gutter. Exact caller ticks remain indivisible. Generated
ticks are selected only through root filtering and the total key.

A data panel is flexible. Its typed measurement input contains every hard
geometry requirement, the exact preferred-size producer identified in this
specification, its closed preferred-aspect list, alignment guides, and the
complete mark, cell, category, and annotation geometry from which those values
derive. No renderer may add a local panel floor, preferred size, aspect, pitch,
or post-allocation fit rule.

### Legends And Guides

A discrete legend prepares semantic entries before layout and measures finite
flow candidates using the exact final title, labels, typography, keys, and
glyphs. A continuous guide similarly measures its title, numeric labels, key,
missing-value section, and admitted orientation. Each parent candidate names one
guide candidate from the complete Cartesian combination; only root selection
chooses globally. No canvas-size percentage chooses typography or flow.

An outside legend is a normal-flow sibling of the data panel. An inside legend
is an overlay whose occupied region participates in collision planning. The two
placements do not share an ambiguous size owner.

### Grids And Composition

Facet and Canvas layout aggregate measured child requirements into explicit row
and column tracks. Track sizing preserves data-panel alignment separately from
unequal decoration gutters. Children whose width or height changes are
remeasured before final arrangement.

Canvas `TextPanel` is a normal-flow grid child and contributes requirements
before track allocation. Plot panel-text annotations remain overlays and do not
become Canvas text panels.

Declared Canvas coordinates, header spans, row titles, range groups, Cartesian
facet coordinates, and trailing wrap slots preserve their current structural
meaning. Automatic layout does not pack occupied cells or collapse empty rows or
columns. A future compact-layout policy requires a separate explicit contract.

### Specialized Families

Specialized families use the same candidate-collection, breakpoint, residual,
quality, convergence, accounting, reservation, and arrangement protocol as
ordinary panels. Their complete producers and preferred formulas are the closed
rules in "Finite Candidate Space"; they do not own another selector.

Heatmap, Beeswarm, Marginal, Matrix, horizontal-categorical, and Waterfall plans
store every specialized decision named by the traversal table. Final drawing
consumes those stored decisions without recomputing packing, cells, marginal
bands, Matrix tracks, category bands, connectors, value-label shelves, guide
reserves, or aspect choices.

Generic facet and Canvas compatibility remains a typed family policy.
Unsupported nesting rejects before candidate generation and cannot become valid
through generic candidate combination.

## Relative Spacing

Visual spacing is resolved from the local components it separates, not from the
root canvas dimensions and not from one global scale factor.

Examples include:

- text leading from the owning `TextTheme` line height;
- Plot and Canvas root title/subtitle gaps and caption-to-content clearance from
  `CompositionTheme` relationships resolved against the adjacent text metrics;
- axis gaps from tick-label, axis-title, tick, and stroke metrics;
- legend padding and key gaps from legend text and glyph metrics;
- annotation clearance from annotation text, stroke, and mark extents;
- facet and Canvas gaps from `CompositionTheme` ratios resolved against the
  owning composition typography.

Theme components own validated dimensionless ratios or role-local lengths only
when the value is a visual choice. The renderer owns scientific geometry,
minimum representability, convergence budgets, and collision policy. Every
relative value resolves to one finite logical length during measurement and is
stored in the immutable plan used for drawing.

## Overflow And Content

The automatic default preserves supplied semantic content. Clipping, elision,
abbreviation, label dropping, scrolling, and scale-to-fit text are not implicit
ways to satisfy a constraint.

Only an explicit current caller policy already identified in the ownership
ledger may authorize a narrower behavior before T6, and its name and default must
make the omission or transformation clear. A new omission policy requires a
separate approved contract and ledger owner; an internal fit branch, renderer
default, or theme value cannot create one. Existing default elision, failed-wrap
fallback, theme visibility, and post-allocation fit paths are re-adjudicated
before automatic sizing becomes current. Automatic numeric tick selection is not
label dropping because generated ticks are a renderer-owned representation; an
exact tick list remains indivisible caller content.

Ink outside normal flow is represented explicitly. A component either requests
reserve, participates in overlay collision, requests an admitted domain change,
or returns a fit error. It does not silently enlarge its parent's final box after
arrangement.

The consequence owner is closed by source, not selected heuristically:

- ink from a normal-flow child outside its assigned flow box produces one reserve
  request targeted at the nearest containing sequence/Grid component, with each
  extent equal to the checked outward edge difference;
- each member in a declared shared Canvas range group or fixed facet axis group
  produces the start/end alignment extents above, targeted at every corresponding
  axis track. The group identity is `(owning Canvas-or-facet address,
  range-group address, axis role)` and members sort by component address;
- projected semantic marks and data-anchored annotations that require a value
  outside an automatic domain produce one domain envelope targeted at their
  owning scale address; the same requirement against an exact caller domain is a
  fit error;
- each inside guide, legend, annotation, or other typed overlay's painted
  obstacle produces one collision record targeted at every later collision-
  participating overlay in canonical order within the same containing panel;
- all remaining painted ink outside an exact assigned box is a fit error.

These cases are tested in that order, but their source classes are disjoint. Each
source/target pair has exactly one consequence under these rules; its record
identity is `(source_address,target_address,consequence_kind)`. No
component may choose between reserve, collision, domain feedback, and failure.

## Convergence

One root operation admits at most eight complete passes. Pass `p` consumes
`MeasurementFrontier[p]` and produces a selected provisional arrangement,
complete state `State[p]`, and `MeasurementFrontier[p+1]`.
`MeasurementFrontier[1]` is constructed once from prepared inputs with empty
prior-pass overlay, collision, alignment, and domain-feedback consequences.
Lifecycle stage 7 constructs later frontiers; it records immutable inputs and selected consequences
but does not evaluate candidates for the next pass.

For every canonical component address, the next frontier contains:

- component type and PlotKind;
- prepared semantic-content identity;
- resolved theme and exact font, face, glyph, and text metrics;
- constraint-derivation inputs: root policy, parent insets, fill policy, track
  limits, and transferred reserve/alignment values; call-local normalized
  constraints are not stored in the shared frontier subtree;
- complete alternative-generation source inputs and producer discriminants;
- complete breakpoint-generation source inputs;
- preferred and exact aspect inputs;
- residual policy, gaps, receiver order, and immutable track limits;
- domains, exact ticks, optional `TickInterval`, the normalized constraints and
  typed hard/preferred/transition inputs from which tick-independent panel source
  sets are derived, resolved `em`/`ch`, and generated-tick inputs; materialized
  source spans, calculated `upper_seed_count`, and coarsening vectors remain
  pass-local;
- legend entries, placements, orientations, and fill inputs;
- topology, packing, track-sharing, and alignment inputs;
- every overlay reserve, collision result, and admitted domain consequence that
  can affect another measurement.

The frontier is call-independent. It never stores call-local constraints,
materialized breakpoint sets, ordered seed collections, cardinality counts,
assigned totals, or residual descriptors. Those values are derived only after
cache lookup from the lookup constraints plus these source inputs, and calculated
values needed for convergence evidence live in `State[p]`. Initial and assigned-
box calls can therefore share one frontier subtree while retaining distinct cache
keys and per-call descriptor streams.

Lifecycle stage 7 transfers consequences from the selected candidate only. For
each target component address, transfer is closed as follows:

- reserve requests carry nonnegative top/right/bottom/left outward extents;
  combine requests by coordinate-wise maximum;
- alignment requests carry `(group,axis,guide_kind,leading_extent,
  trailing_extent)`; each group's next target uses the maximum leading and
  trailing extents, and records every member address in canonical order;
- automatic-domain requests carry finite lower/upper bounds. For each automatic
  scale target, the next record is always the minimum-lower/maximum-upper union
  of its immutable prepared base domain, its consumed prior cumulative envelope
  when present, and every selected-candidate request; absence of a new request
  never removes the prior envelope. Any change to an exact caller domain is a fit
  error rather than feedback;
- collision consequences retain the complete sorted obstacle rectangles and
  selected typed overlay alternative; exact duplicate records deduplicate;

Selected tick, label, legend, track, and packing outputs remain in `State[p]` but
do not transfer independently; any effect on the next pass occurs only through
the reserve, alignment, automatic-domain, or collision record above. No other
consequence transfers. Two records with the same nonaggregating typed
identity but different values are an internal render error. The resulting
records sort by target address and typed field order and become the exact next-
frontier inputs; "may enter" or discovery-order aggregation is not admitted.

On the next pass, reserve extents increase the target component's corresponding
hard inset and add the resulting containing extent to its base breakpoint set.
Alignment targets raise the applicable leading/trailing Grid track hard minima
for every group member before natural-track construction. An automatic-domain
envelope becomes that axis's cumulative automatic domain and deterministically
recomputes its scale projection, generated-tick inputs, and projected geometry.
Collision records become the exact obstacle list consumed by the closed overlay
producer; an anchor intersecting one is rejected. These four applications occur
before pass-local breakpoint/seed counting. No transferred record directly
mutates a selected size, box, tick list, or plan.

The complete state additionally contains the selected root size, complete
candidate descriptor and quality key, selected child candidate identities,
the normalized constraints of every selected initial and assigned-box call in
mixed-radix call order, every assigned geometry context in the same order, sizes,
baselines, ink extents, spacing, provisional boxes, text breaks, ticks,
legends, tracks, domains, overlay decisions, structural topology/capacity
censuses, calculated per-selected-call accounting metadata, and the complete next
frontier. Operational cache-hit counts, pass-local evaluations already charged,
and the cumulative evaluation counter are excluded from `State[p]`; they cannot
affect convergence equality or cycle identity.

Equality is exact structural equality after canonical normalization. Equality
with the immediately preceding complete state proves convergence only when every
painted ink edge is inside the selected root box; stable out-of-root ink returns
the ordered hard-fit error instead. Equality with
any earlier non-immediate state is a repeated nonfinal state and fails
immediately. A changed eighth state fails for budget exhaustion. Successful
convergence uses two through eight passes inclusive.

A measurement-cache entry is scoped to one root operation and stores one complete
`CachedMeasurement`: the `MeasurementOutcome`, including `Unsatisfied`, plus the
canonical fixed accounting tuple
`(descriptor_count,evaluation_count,rejected_count,
admitted_before_dedup_count,returned_count)`. A pre-descriptor `Unsatisfied` uses
all-zero counts. A cache miss calculates and stores this tuple before commit; a
hit returns both outcome and tuple, adds the tuple to the selected call's
`State[p]` record, and charges no evaluations a second time. Its lookup key is the component's
normalized constraints, optional complete `AssignedGeometryContext`, and complete
consumed frontier subtree. The context is constructed before the assigned-child
lookup from the parent descriptor and compares by its full structural signature;
it is never appended after lookup. That subtree
recursively contains every descendant component's prepared inputs and feedback,
so lookup occurs before descriptor construction or child calls while still
invalidating on any descendant change. Equal constraints alone are insufficient.

For that tuple, `rejected_count` is exactly the number of charged raw evaluations
that fail before producing a complete component candidate under "Attempt
Rejection"; `admitted_before_dedup_count` is the number that produce complete
provisional candidates before signature deduplication, including candidates whose
root gate later fails; and `returned_count` is the final deduplicated collection
length, or zero for `Unsatisfied`. Therefore
`evaluation_count = rejected_count + admitted_before_dedup_count` and
`returned_count <= admitted_before_dedup_count`. Root eligibility/viability
filters occur after the cached root collection and never alter these counters.
A changed alternative-generation input, breakpoint-generation input, residual,
domain, font metric, tick input, legend entry, alignment input, topology,
packing input, overlay consequence, or descendant frontier record forbids reuse.
Assigned-box constraints and any assigned geometry context derive
deterministically from the parent key/candidate and the closed grammar; they do
not add a post-lookup key fragment.

A position change does not forbid reuse when position is absent from the
component frontier. If that position changes collision, alignment, overlay, or
domain feedback, the changed consequence is present in the frontier and forbids
reuse. A cache hit performs no candidate evaluations. No cache entry survives
root success, cycle failure, budget failure, or another root error.

### Candidate Evaluation Accounting

After its children have returned complete collections and before evaluating one
cache miss, calculate that component's exact raw evaluation count:

- each leaf descriptor counts once: a valid seed is charged once for each
  applicable breakpoint tuple, while a seed rejected before any tuple is charged
  exactly once; text transition/derived-height descriptors, tick-list
  descriptors, and width-specific specialized vectors follow this rule;
- each composite combination, including a Plot or Canvas root combination,
  counts once in addition to its already-counted child candidates; root status
  adds no second charge;
- candidates later rejected by a gate or viability filter count;
- candidates later deduplicated count;
- one child candidate reused by several parent combinations is evaluated once
  as a child, while each parent combination counts;
- a cache hit counts zero.

Residual receiver and balanced vectors are not leaf evaluations. Each is one
radix of its enclosing composite descriptor and is charged once through that
composite combination; an invalid residual vector still charges that one parent
descriptor and is not charged separately. The closed raw counts for text
transitions, adaptive span/count tick sources, caller-interval coarsening
sources, their final midpoint sources, Forest subsequences, residual vectors,
and Beeswarm branch products are defined in their producer sections. Every raw
tick source, including a source rejected by the `10_000`-position ceiling and a
source duplicating an earlier vector, is counted before evaluation. All sums and
products use checked `u128`. If one cache miss's complete count exceeds the
remaining cumulative budget, that miss fails before evaluating any of its
attempts. Earlier cache misses in the pass remain charged and their temporary
results are discarded with the failed root operation. At most `1_000_000`
evaluations are admitted across all components and passes.

## Canonical Dimensions And Output

A selected logical root dimension `q` converts exactly once as:

```text
canonical_dimension(q) =
    ceil_div(q, 1024)
```

The result must be positive and fit `u32`. A `max_width` result must not exceed
the exact integral caller bound after this conversion. An exact `size` root uses
the caller dimensions multiplied by `1024`; conversion must reproduce those
dimensions exactly. Conversion never truncates, clamps, or retries selection.

Every signed plan coordinate, baseline, extent, radius, and stroke width that
reaches canonical SVG uses one exact CSS-decimal spelling. For signed quantum
value `q`, take its checked `u128` magnitude, let `whole=magnitude/1024` and
`remainder=magnitude%1024`, and format:

```text
remainder == 0:
    optional_minus + decimal(whole)

otherwise:
    fraction10 = remainder * 9_765_625
    optional_minus + decimal(whole) + "." +
    zero_pad_10(decimal(fraction10)), with trailing zeros removed
```

Zero has no minus sign; exponent notation and locale separators are forbidden.
The multiplier is exact because `10^10/1024 = 9_765_625`. Canonical SVG uses the
same top-left origin and downward-positive y direction as the frozen plan, so no
coordinate inversion or integer pixel rounding occurs in that serializer. A
private bottom-up backend adapter, when needed, uses checked
`root_height_quanta-y` and its inverse exactly once; Null geometry and canonical
SVG consume equal post-adapter quantum values. Root `width`, `height`, and
`viewBox` dimensions use the integral `canonical_dimension` values above, while
all internal spatial attributes and path operands use this quantum spelling.
The non-rendering byte counter and concrete serializer call the same token-length
and token-emission function, so coordinate precision cannot change the counted
canonical SVG length. Any required checked transform or decimal-capacity failure
aborts before backend begin; no `.1`/`.2`/`.3` formatting, saturation, or clamp is
admitted.

One root operation has this finite resource envelope:

| Resource | Inclusive limit |
| --- | ---: |
| Complete measurement passes | `8` |
| Cumulative candidate evaluations | `1_000_000` |
| Cumulative candidate-generation slots | `1_000_000` |
| Cumulative candidate-generation work units | `1_000_000` |
| Cumulative measurement-call attempts | `1_000_000` |
| Measurement-cache entries | `1_000_000` |
| Prepared structural topology slots | `1_000_000` |
| Canonical system-font discovery entries | `1_000_000` |
| Cumulative system-font bytes inspected or hashed | `536_870_912` |
| Cumulative font-preflight logical work units | `1_000_000` |
| Canonical SVG bytes admitted for automatic or bounded layout | `67_108_864` |
| PNG source or target axis | `32_768` |
| PNG source or target pixels | `100_000_000` |

The topology-slot count is exactly the length of the prepared canonical
structural-address census before candidate generation. It contains every
declared Canvas/facet coordinate including empty and trailing coordinates; every
spanning item once at its origin; every Matrix/Heatmap cell; every category,
Beeswarm point, Marginal subpanel, Waterfall category and connector, legend
entry, annotation target, mark-layer item, and other prepared component address
named by the traversal grammar. No unenumerated "other specialized" class exists.
Candidate-local tick/label records, text line breaks, legend-flow slots, packing
vectors, and residual vectors are not structural topology slots; their exact
pre-evaluation descriptor maxima are reserved as generation storage in stage 2.
Selected copies inside evaluated candidate plans are instead part of the exact
candidate/local-plan payload census reserved in stage 3 and do not claim
generation slots.
Existing stricter semantic family limits remain.

The font magnitudes reuse adjudicated resource classes rather than introduce
unrelated caps. Discovery entries are prepared pre-render records and therefore
reuse the inclusive `1_000_000` topology-record magnitude. Font-preflight logical
operations reuse the inclusive `1_000_000` generation-work magnitude. The byte
limit is the least power of two not below the current maximum admitted RGBA source
or target backing store, `4 * 100_000_000` bytes: `536_870_912`. These derivations
and their limit-minus-one/equal/plus-one evidence are part of the resource
contract; they do not imply pooling between the independent counters.

Semantic stage 3 count-only enumerates the exhaustive `PreparedTextRequirement`
census before font work; reservation stage 1 materializes that same census. In
canonical component preorder and each producer's declared text-role
order, it contains every present caller string, resolved semantic/display string,
exact label, generated literal, annotation/status string, legend/guide string,
header, and other text-bearing prepared record that any candidate may measure or
paint, paired with its effective `(stack,weight)`. Within a record, scalars retain
UTF-8 source order. An intentionally empty exact tick label contributes the record
identity but zero scalars. Font-family names, SVG syntax, metadata names, and
content suppressed by an explicit caller policy are not rendered text and do not
contribute.

A closed formatter whose exact output depends on later candidate geometry instead
contributes its complete finite scalar alphabet under the effective text role,
in ascending scalar-value order after the source records. The automatic numeric
and Forest grammars contribute exactly ASCII `0123456789-.ekM^`; a producer with
another generated formatter or token must declare its literal scalars or finite
alphabet in its producer section, and no unlisted draw-time text source is
admitted. This alphabet is intentionally a preflight superset: when automatic
numeric generation is active, every listed scalar must have a fallback face even
if the selected candidate's final labels use only a subset. Records then sort by
canonical component preorder, declared text-role order, and local index to assign
each structurally equal `(stack,weight)` pair its zero-based first-use ordinal;
`FontStack` entry order remains its declared order and the pair is not separately
ordered by family-name bytes. The scalar census traverses pair ordinal first,
then `(component_address,text_role,local_index)`, source scalars in UTF-8 order,
and finally that record's generated alphabet in scalar-value order. Duplicate
scalar values deduplicate only in the fixed per-pair presence map, while every
consumed occurrence still charges font-preflight work. This is the complete
required-scalar membership and traversal rule.

Font preflight is bounded before any font-sized collection grows. The source
grammar is exactly the platform location, regular-file, and extension rules used
by locked `fontdb 0.23.0` system discovery, executed by a project-owned bounded
walker rather than `Database::load_system_fonts`. At reservation stage 1,
fallibly reserve exactly `1_000_000` fixed discovery slots before walking. Every
encountered directory or file claims one slot; directory symlinks are not
followed, and claim `1_000_001` fails before processing that entry. Duplicate
canonical file paths deduplicate; admitted files sort by platform source-location
ordinal and then lossless platform-native path-unit order. Reserve the exact
metadata-slot count for those files, then open each in canonical order, obtain its
file-handle length, and reject before reading it when checked cumulative bytes
would exceed `536_870_912`.

On fontconfig hosts, the initial configuration file, included configuration
files and directories, and retained include-directory entries consume this same
cumulative discovery-entry envelope before the resulting font roots are walked.
Configuration bytes consume the same cumulative byte envelope: obtain length
from one admitted regular-file handle, reject the cumulative length before
reservation, read at most that admitted length into exactly reserved temporary
stage-1 storage, reject short or changing files, and only then parse. Includes
are iterative, canonical-file deduplication and lexicographic directory order
match locked `fontdb 0.23.0`, and a resource or reservation failure is fatal
rather than an ignorable fontconfig result. Missing optional paths and bounded
fontconfig syntax or relevant-directive failures remain ignorable exactly as in
locked `fontdb 0.23.0`; an admitted file's I/O failure, mutation, or resource
failure is fatal. Internal XML entity declarations are rejected as a bounded
fontconfig syntax failure. Before XML parsing, count
`2 * less-than-sign bytes + equals-sign bytes + 1` as the deterministic parser-
capacity work claim and set the third-party parser node limit to
`2 * less-than-sign bytes + 1`. Each relevant `dir`, `include`, `reset-dirs`, or
generic-alias directive also claims one discovery entry before retained
directive storage grows.
Temporary configuration bytes are discarded after root and generic-family
extraction; only admitted font bytes require retention through scene
construction.

Actual bytes read are charged against the same bound and cannot exceed the
admitted handle length. Hashing and metadata decoding stream through fixed
scratch into the already-reserved source-byte storage and metadata slot; no
complete unreserved file buffer is created. Every admitted file's exact bytes are
retained in that stage-1 storage through scene construction so the explicit
`fontdb` load, identity hash, glyph resolution, outlined SVG, and PNG all consume
the same bytes. There is no selected-files-only re-read or second retention rule.

Font-preflight logical work charges one unit for each Unicode scalar consumed by
the required-scalar census, claimed fontconfig or font-tree discovery entry,
decoded face, decoded ID-16 or ID-1 name occurrence, candidate occurrence counted
or materialized, identity record sorted or deduplicated, and candidate/scalar
glyph coverage probe. Every sort and deduplication, including a fontconfig
include-directory path sort, bulk-claims its input length as in candidate
generation. Fontconfig additionally charges its declared node-capacity work
before parsing and one path comparison for every previously admitted canonical
configuration path examined during first-use deduplication.
Reservation stage 1 first reserves one fixed `0x110000`-bit Unicode-scalar
presence map; the charged census marks that map without growing a collection,
then reserves the exact ordered required-scalar payload from its population
count. Count candidate occurrences, provenance entries, selected-face records,
and required scalar probes by checked arithmetic; claim work and reserve their
exact stage-1 capacities before materialization. The
entry, byte, and work limits are independent and cumulative for one root
operation. A changing file, short read, parser failure, arithmetic overflow,
limit failure, or reservation refusal aborts before measurement and backend
creation; there is no next-font, built-in-font, or path-order fallback.

One candidate-generation slot is one fixed-size stored breakpoint, seed header,
alternative index, line/break index, packing offset, track value, residual value,
or descriptor range. A vector consumes one header plus one slot per stored
element; a text alternative consumes its header plus every stored ending index.
Before materializing a collection, calculate its exact slot total by checked
`u128`, including all vector payloads, and claim it from the cumulative
`1_000_000` budget. Claims persist across components and passes; count-only
arithmetic claims zero. No generation slot owns a growing collection or copies
prepared semantic payload. This bounds all pre-candidate breakpoint, text,
packing, seed, and residual storage independently of candidate evaluations.
Candidate-plan payloads may duplicate selected values, but their complete checked
slot/byte census is multiplied by the already-budgeted candidate cardinality and
must be reserved successfully at reservation stage 3 before evaluation; it is not
part of the earlier generation-slot counter.

Candidate-generation work uses an implementation-independent logical census,
not executed loop or comparator counts. The exhaustive charge events are: one
unit for each Unicode scalar consumed by each declared segmentation, bidi, line-
break, or wrapping pass; each shaping glyph consumed or emitted; each prepared
input record consumed by a declared producer pass; each generation slot counted;
each generation slot materialized; and each packing/overlap pair, bracket level,
tick step, or residual-track operation tested. A sort of `n` logical records
bulk-claims exactly `n` units before sorting, and its exact-signature
deduplication bulk-claims exactly `n` more before scanning, independent of sort
implementation and actual comparisons. Candidate evaluations have only their
separate counter and do not also claim work. If the contract requires two passes
over one sequence, both logical passes charge; loop fusion, vectorization,
memoization outside the specified measurement cache, or a different library sort
does not change the charge. Straight-line checked arithmetic within one claimed
event is fixed work. A claim above `1_000_000` fails immediately; count-only
passes claim work even though they claim no storage slot. Transition shaping
streams glyph outputs through fixed scratch and stores only its claimed
transition scalar; selected-line glyph arrays belong to the later exact
candidate-plan census. Font-file discovery, hashing, occurrence preparation, and
glyph probing consume the separate bounded font-preflight resources above rather
than candidate-generation work. Candidate-generation work, candidate-generation
slots, measurement-call attempts, measurement-cache-entry claims, and candidate
evaluations use independent, non-pooling counters cumulative across every pass of
the root operation.

Every root, initial-sizing, and assigned-box component call claims one cumulative
measurement-call attempt immediately before cache lookup; cache hits still claim
because traversal work occurred. A claim above `1_000_000` fails before lookup.
Before an assigned-call descriptor stream begins, its exact remaining call count
must fit the remaining budget as the pass-local lower-bound check requires; calls
then claim those attempts in canonical order. This bounds cache-hit and miss
traversal independently of candidate evaluations.

After the fixed cache-slot reservation, every cache miss atomically claims one
slot before descriptor construction or child calls. The checked cumulative claim
counter includes misses that later cache `Unsatisfied` without a parent
evaluation. A claim when `1_000_000` slots are already occupied fails immediately
with the resource-limit error; a hit claims no slot. Thus every stored outcome
has one pre-reserved entry and insertion never grows the cache.

Each fixed slot contains the key references, state tag, fixed accounting tuple,
and inline `FitReason` storage. A claim leaves the slot uncommitted. If initial children make the parent
unsatisfied, the parent writes the least reason inline and commits the slot
immediately after the final nonfatal sibling outcome, before descriptor or
candidate-payload reservation. Any later `Unsatisfied` is committed the same way
when its descriptor stream is exhausted. A `Candidates` outcome commits only
after stage-3 candidate/cached-payload reservation and complete evaluation. A
fatal root error discards every claimed or committed slot with the operation.

### Reservation Stages

First-party allocation occurs in these exact stages:

1. after topology counting, reserve the fixed Unicode-scalar presence map and
   exactly `1_000_000` fixed font-discovery slots, perform the work-charged
   required-scalar census and bounded font walk, reserve exact canonical font-
   file metadata slots and admitted source-byte storage, materialize and parse
   those files under the byte/work gates, count the resulting font occurrence,
   candidate-list, selected-face, glyph-probe, and provenance payloads, reserve
   and materialize those exact capacities, then reserve complete prepared
   semantic/topology and initial-frontier payloads plus exactly `1_000_000` fixed-
   size measurement-cache entry slots;
2. in the pass-local order and only after each cumulative slot claim, reserve
   preliminary breakpoint, non-tick source/transition, tick-independent,
   generated-tick source/transition, final-base, aspect-expanded, remaining
   alternative-seed, and residual-descriptor storage;
3. after candidate cardinality calculation and the cumulative budget check,
   reserve every cache-miss candidate collection, immutable local-plan payload,
   quality payload, structural signature, and cached outcome payload;
4. before retaining one complete state, reserve that state and its next-frontier
   payload; a cache entry key references the already-reserved frontier subtree
   and constraints, and owns no growing key collection;
5. after stable root selection, use the selected nonexact candidate's already-
   calculated canonical preserve-text SVG byte count or calculate it now for an
   exact-size candidate, calculate the serializer scratch census, and reserve the
   final immutable plan, canonical SVG buffer, serializer scratch, and
   `RenderedScene` storage before Null-backend begin.

Every capacity uses checked `u128`, checked conversion to `usize`, and fallible
exact reservation. Refusal aborts at that stage. The renderer does not retry
with smaller capacity, choose another candidate after reservation refusal, omit
content, or switch representation. No first-party collection may grow beyond
its reserved census.

A non-rendering SVG byte counter applies the same canonical preserve-text token-
length functions as the concrete SVG serializer to each nonexact root candidate
that reaches its SVG filter and to the selected exact candidate before final
reservation. It creates no backend lifecycle and emits no bytes. Concrete
canonical serialization must produce exactly the selected candidate's counted length.
Outlined SVG is a later derived publication and is not the canonical byte count.

### Candidate Eligibility And Nonexact Viability

Each pass filters root candidates in this order:

1. semantic gate;
2. hard-fit gate;
3. exact-aspect gate;
4. finite positive logical root size;
5. canonical-dimension conversion;
6. bounded-width final-width check, for `max_width`;
7. nonexact PNG source-axis limit;
8. nonexact PNG source-area limit;
9. nonexact canonical preserve-text SVG byte limit;
10. nonexact default-PNG target-axis limit;
11. nonexact default-PNG target-area limit.

Dense ranks and total keys are constructed only from survivors. If one filter
removes the final candidate, that filter's error is returned. This
first-empty-frontier rule determines candidate failure order independently of
discovery order.

For positive canonical source axis `d`:

```text
default_300_dpi_target(d) =
    ((25 * d) + 4) / 8
```

The calculation is checked integer arithmetic. Automatic and bounded candidates
must have source and predicted target axes at most `32_768`, source and target
products at most `100_000_000`, and exact canonical preserve-text SVG length at
most `67_108_864`.

Exact-size roots skip filters 7 through 11. They have no canonical-SVG byte cap
during layout, preserve-text SVG publication, or outlined SVG publication.
Their final byte count must still convert to `usize`, and exact final reservation
must succeed. A later PNG terminal applies the PNG source-axis, source-area,
canonical SVG parser-input limit of `67_108_864` bytes, target-axis, and target-
area limits without mutating the exact canonical scene. Outlined SVG conversion
has no layout byte cap; its own checked allocation refusal remains format-local.

For every root policy, a later custom `png_with_options` terminal validates its
call-local options, rounded-positive target dimensions, target axes, and target
area in output-format stage 21. An automatic or bounded scene that passed the
default-300-DPI prefilter may therefore reject a larger custom target without
mutating that scene or rerunning layout. Preview and default PNG apply their own
call-local option and target checks in the same stage.

### Complete Failure Order

Stages 1 through 6 run once for the root operation. Each component first claims
one measurement-call attempt, then performs cache lookup from constraints and its
consumed frontier subtree. A hit returns
the stored outcome. On a miss, stages 7 through 13 are recursive: the component
first claims its cache slot, then initial sizing
children run in canonical address order, including every sibling after a
nonfatal `Unsatisfied`; only when every initial outcome is `Candidates` does the parent construct its
descriptors; assigned-box child calls run in descriptor and address order; and
the parent finishes counting and evaluation. A child's fatal error wins
immediately, while `Unsatisfied` returns normally and rejects only its enclosing
descriptor. Stages 14 through 21 resume once the root collection exists. The
exact order is:

1. numeric and constraint normalization;
2. theme and scale resolution plus effective text-role style resolution;
3. semantic, data, exact-policy, and scientific-requirement validation, then
   count-only canonical `PreparedTextRequirement`/first-use `(stack,weight)` pair,
   topology, and initial-frontier censuses;
4. checked topology arithmetic;
5. topology-slot limit;
6. reservation-stage-1 scalar-presence-map reservation, work-charged required-
   scalar census and exact payload reservation, bounded font discovery and
   metadata/source-byte reservation, streaming metadata/hash materialization,
   font-preflight-work-gated occurrence/candidate/glyph/provenance census and
   exact reservation, then prepared semantic/topology, initial-frontier, and
   measurement-cache reservation/materialization followed by complete bounded
   face/glyph resolution;
7. measurement-call limit, cache-entry capacity claim on a miss, then initial
   sizing-child outcomes through this same protocol;
8. local preliminary/non-tick/tick-independent/generated-tick/final-base and
   remaining alternative count arithmetic, cumulative generation-work and per-
   collection slot claims before ordered reservation/materialization, aspect
   expansion, and residual descriptor construction;
9. descriptor candidate-count lower-bound and assigned-call-attempt limit, then
   assigned-box child outcomes through this same protocol;
10. local complete candidate-count arithmetic, counting an unsatisfied-child
    descriptor once;
11. cumulative candidate-evaluation limit;
12. candidate, local-plan, quality, signature, and cached-outcome reservation;
13. complete local candidate evaluation and provisional planning;
14. ordered root candidate gates and nonexact viability;
15. dense-rank construction and root-key selection;
16. complete state and next-frontier reservation;
17. convergence equality, repetition, or pass exhaustion;
18. final plan, canonical SVG, serializer, and scene reservation and freeze;
19. balanced Null-backend lifecycle;
20. concrete backend begin, drawing, end, and scene construction;
21. call-local output-format validation and publication.

A checked arithmetic failure aborts at its current stage. No backend is created
before stages 1 through 18 succeed. No failure creates an artifact, partial
scene, previous-plan fallback, truncated candidate set, fixed-size fallback,
omitted content, alternate renderer, or second layout route.

Phase T6 exposes the canonical dimensions as `Output::width()` and
`Output::height()`. They report the already-rendered scene and remain unchanged
across chained output publications. Raster target dimensions may differ through
`PngOptions` without changing those canonical values.

## Public Surface

Phase T6 atomically establishes this public sizing surface:

- no sizing call selects automatic layout;
- `Plot::max_width(width)` and the Canvas equivalent set a finite root width
  maximum and leave height automatic;
- `Plot::size(width, height)` and `Canvas::size(width, height)` set exact
  canonical dimensions;
- `Output::width()` and `Output::height()` expose final canonical dimensions;
- pre-render `Plot::width()` and `Plot::height()` are removed.

Phase T6 also exposes one optional per-axis automatic interval policy:

- `TickInterval` is an opaque immutable crate-root value with
  `Clone + Copy + Debug + PartialEq`;
- `pub fn TickInterval::additive(step: f64) -> Result<Self>` constructs a
  finite positive linear data-space interval and rejects nonfinite or nonpositive
  input;
- `pub fn TickInterval::log10_decades(n: u32) -> Result<Self>` constructs a
  positive integral Log10-decade interval and rejects zero;
- `pub fn Plot<K>::x_tick_interval(self, interval: TickInterval) -> Self where
  K: kind::NumericX` and
  `pub fn Plot<K>::y_tick_interval(self, interval: TickInterval) -> Self where
  K: kind::NumericY` attach that policy to the named axis; repeating the setter
  replaces the prior value;
- an exact `x_ticks` or `y_ticks` list takes precedence for that axis regardless
  of setter order; a present exact list outside `1..=10_000` rejects rather than
  revealing the stored interval;
- absence selects typography- and panel-span-aware automatic generation;
- no public target count, maximum count, visual-density scalar, or generated-tick
  callback is admitted.

The interval is a preference rather than exact tick positions. The renderer
tries it first, then deterministic coarser intervals when no complete candidate
using the closer source survives. A terminal one-tick midpoint source is admitted
only after all interval sources and is ranked after them; it prevents an
otherwise valid finite domain with no representable aligned multiple from making
the plot unrenderable. Exact `AxisTick` remains the strict position/label control.

No public pre-render measurement operation, `SizeRequest` enum, layout trait,
constraint object, plan object, callback, or plugin is admitted. A broader public
constraint surface requires a demonstrated caller need and a separate contract.

## Validation And Evidence

Focused evidence must cover:

- lower, upper, nearest-even, ink-edge, exact-size, signed-coordinate, and
  negative-zero normalization;
- `i64::MIN`, `i64::MAX`, empty-interval, checked `i128`, checked `u128`, and
  checked `usize` boundaries;
- nonspatial finite `f64` ordering across negative, zero, and positive values;
- one-candidate leaves, multiple-candidate children, complete parent Cartesian
  combination, one selected root tree, and no local quality pruning;
- complete text, tick, legend, aspect, residual, track, packing, and specialized
  candidate generation;
- no-interval numeric axes producing more than five ticks on sufficiently wide
  panels, typography/span seed-count boundaries, exact `10_000` generated
  positions, and `10_001` rejection;
- additive interval exact-source success, deterministic `1,2,3,5` decimal
  coarsening, zero alignment, `2^53` index boundaries, source-vector
  nonemptiness, no interval subsequence, terminal midpoint order, Log10-decade
  coarsening, scale mismatch, exact-tick precedence, setter replacement, and
  categorical rejection;
- present exact-list `0`, `1`, `10_000`, and `10_001` boundaries plus absence-
  selects-automatic behavior;
- unlabeled exact ticks retaining tick/grid positions with no text plan, gutter,
  collision, or paint;
- `Start`, `Center`, and `End` root title-block alignment with all built-ins
  centered and title/subtitle/caption owner consistency;
- canonical font matching, occurrence generation, identity deduplication,
  generic-family resolution, style/stretch/weight distances, ordering, primary
  and per-scalar fallback selection, exact provenance, discovery-entry/byte/work
  boundaries, changing and short files, reservation refusal, and invariance under
  path, nonidentity metadata, and discovery-order permutations;
- shared-guide automatic-domain union, equal and unequal exact domains, exact
  domains containing or excluding automatic envelopes, kind and descriptor
  conflicts, contributor-order invariance, and one Canvas-typography label pass;
- interval-source precedence over every soft rank, joint-axis interval ordering,
  dense-rank equality and gaps, text raggedness, tick gap spread, legend line
  imbalance, density shortfall, panel shortfall, area, whitespace, and exact
  structural-signature formulas;
- equal tick position/label vectors from unequal source kinds or ordinals
  remaining distinct through quality and frozen-plan identity;
- exact-aspect gate rejection and preferred-aspect ordering;
- ordinary Plot, facet, sparse Canvas, Heatmap, Beeswarm, Marginal, Matrix,
  horizontal-categorical, and Waterfall traversal and type order;
- equal constraints with changed alternatives, breakpoints, domains, fonts,
  topology, overlays, or descendants producing cache misses;
- unchanged complete frontier producing a cache hit with zero evaluations;
- convergence on passes two and eight, changed pass eight, and non-adjacent
  repetition;
- evaluation counting for leaves, composites, roots, rejected candidates,
  duplicates, parent reuse, and cache hits;
- candidate-generation logical-work accounting invariant under loop fusion,
  vectorization, sort implementation, and comparator count;
- injected reservation refusal at each of the five reservation stages before
  backend begin;
- automatic and bounded candidate filtering before dense ranking;
- nonexact source-axis, source-area, SVG-byte, target-axis, and target-area
  boundaries;
- exact preserve-text and outlined SVG above `67_108_864` bytes remaining
  format-local while later PNG rejects without scene mutation;
- one frozen layout and every traced value shared by Null and concrete rendering;
- no unowned input and no measurement, candidate generation, convergence,
  accounting, arrangement, or domain mutation during final drawing.

Numeric evidence includes limit-minus-one, exact-limit, and limit-plus-one for
eight passes; each `1_000_000` candidate-evaluation, candidate-generation-slot,
candidate-generation-work, measurement-call, measurement-cache-entry, and
prepared-topology-slot boundary; `1_000_000` font-discovery entries;
`536_870_912` font bytes; `1_000_000` font-preflight work units; `32_768` axes;
`100_000_000` pixels; and `67_108_864` nonexact SVG bytes. Default-PNG evidence
includes source axis `10_485` succeeding and `10_486` failing target prediction,
and source `3_200 x 3_200` succeeding while `3_200 x 3_201` fails target work.

Candidate-order evidence covers trichotomy, antisymmetry, transitivity,
duplicate-signature deduplication, and selection invariance under discovery-
order permutations. Ownership evidence contains a complete census from each
layout input's owner through every consumer and immutable plan field.

Snapshot evidence distinguishes explicit-size preservation from intentional
automatic-default changes. Every automatic snapshot records its complete
selected quality key and the competing candidate frontier; increased size alone
is not an acceptance reason.
