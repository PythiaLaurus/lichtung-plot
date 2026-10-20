# Stacked discrete guide measurement

The measured discrete owner now consumes every prepared section under one outer
frame. Combined and Separate use the same intrinsic, assigned, flow, feedback
and selected-call path. Runtime semantic preparation still selects Combined;
private actual-producer fixtures exercise Separate before its public cutover.

## Retained measurement and geometry

`src/decorations/legend/measurement.rs` replaces the one-title/flat measurement.
Its `flow.rs` implements one stable pass per section: horizontal baseline-aligned
rows or vertical equal-width columns. No first-section measurement/drawing path
remains. `LegendLayout` owns ordered section records, canonical i64 extents,
title baselines, entry key/label positions, assigned section capacities and
retained immutable text blocks. Section slots remain beneath the original guide
owner; flow columns do not create semantic addresses.

Every section receives the common inner width. Normal and compact allocations
remain global variants. Compact uses the finite column-gap placement operand in
`floor((W - gap) / 2)`; assigned labels are measured at their changed widths.
The normal trial reports whether any allocation would change. Title and entry
hard widths are checked before assigned label shaping. Constructible sibling
sections/labels retain canonical fit failures; arithmetic/font errors stay fatal.

Natural preference uses the maximum complete section title/entry width and the
same automatic-height vertical stack. A finite height is never fabricated for
automatic-height flow. The conservative text advance guard remains one quantum
in the complete natural requirement.

Finite vertical height uses the existing canonical track allocator. Each section
has a hard title-plus-tallest-entry requirement and a one-column preferred
capacity, including exact active title/row gaps. Exact section gaps are reserved
once. Track maxima pin excess to unused slack; the sole proportional shrink uses
largest remainders with section-ordinal ties. Whole entries greedily form columns
inside those capacities. Actual returned heights are stacked; unused capacity is
not redistributed and an unsuccessful section does not trigger another allocation.

Exact rational padding/gaps remain exact until complete hard ceilings or final
position rounding. One outer horizontal inset and one vertical padding/comfort
reserve are applied. Frame absence retains content, padding and key occupancy.
The existing default edge arithmetic (11264 placement / 11265 hard quanta per
edge, 22529 for the complete two-edge reserve) remains independently tested.

Key measurement and selected drawing share `ResolvedGuideKey::visit_calls` and
the canonical geometry materializer. The full outward left/right key span,
stroked points/triangles, lines and rectangular fills contribute to occupancy.
Transparent paints retain structural component occupancy; selected calls retain
their original alpha. Text ascent/descent include actual line ink as well as
advance/leading. A horizontal row takes independent maximum ascent and descent.
The selected guide's complete key/text/frame ink is checked against one box.

Selection freezes all section titles, labels, key components and the one optional
frame in the existing `LegendDrawPlan`. Feedback records ordered section geometry,
capacities, text plans and key consequences, including the distinction between
automatic and finite incoming height. The former duplicate title/entry arrays,
font-stack carrier, flat flow records and title-half-height placement heuristic
are removed.

## Independent and actual-source evidence

Four focused tests replace four lower-value tests in the same change:

* `horizontal_sections_stack_actual_rows_and_compact_remeasures_can_be_taller`:
  spec08's independently prescribed normal rows occupy **76px**; one quantum
  less misses. Separately supplied compact widths/heights occupy **82px** and
  refuse the 76px assignment. Titles/sections keep their semantic order.
* `a_shared_baseline_reserves_the_independent_maximum_ascent_and_descent`:
  entry extents **(18,2)** and **(5,15)** share a baseline and require **33px**,
  rather than the individual maximum 20px. One quantum less misses.
* `vertical_sections_shrink_once_in_quanta_and_never_redistribute_unused_capacity`:
  the prescribed H=100 fixture allocates **[50,40]px** and misses at section A.
  H=120 allocates **[64853,47787] quanta**; normal misses at B, while compact
  returns a **100×95px** occupied guide with **[8533,17067] quanta** unused.
  Automatic height retains one column per section and occupies **145px**.
* `factorized_keys_and_multiline_titles_freeze_in_one_contained_frame`:
  an actual sparse Interaction produces two sections/four point+line keys with
  TriangleUp/Circle group styles, radius 23 and independent alpha-211 5px point
  outlines. Both orientations retain the multiline title and every source key
  inside one frame. Removing the frame preserves dimensions/content. Repeated
  selection yields identical occupied geometry with no further shaping.

The existing independent allocator tie test additionally checks hard=[2,2,2],
preferred=[5,5,5], available=10 gives **[3,3,4] quanta**. The allocator algorithm
is unchanged; its existing checked implementation is reused by the guide.

Retired tests are the duplicate continuous sampled-stop center check and small
insufficient-length probe, the axis constructor/edit inventory and the low-level
absent-fill-outline normalization inventory. Continuous asymmetric-center,
one-quantum singleton/frame, unusable-without-shaping and complete-ink contracts
remain independent. Exact spacing/allocation/font and actual parent/cache/replay
tests are retained. No tests were moved to integration or ignored suites.

## Verification

**38 distinct focused library tests** pass:

* Four new section measurement contracts.
* Sixteen native coordinate/parent/cache/feedback contracts.
* Existing selected-metadata-poisoning replay, Matrix natural-root guide and
  both-contributor-orders Shared title authority contracts (one each).
* Ten remaining continuous-guide contracts sharing guide metrics/chrome.
* The independent largest-remainder allocator test.
* Two exact axis-spacing, one complete guide-edge and one present-width
  validation tests.

**36 existing external tests** pass: twenty layout/legend placement/collision
contracts and sixteen Shared composition/guide callers. These include actual
inside/outside allocation, complete Shared keys, facets, hidden contributions,
continuous/discrete separation and owning Shared gaps.

Commands use `cargo test --locked --offline` with the named module/test filters.
`just lint`, `just fmt-check` and `git diff --check` pass. Library discovery stays
**671 tests, including 17 ignored**. Logs are `/tmp/opencode/stacked-guides-*.log`.
No full suite or release benchmark is claimed by this checkpoint.

Owner-wide measurement reuse and existing per-round placement-attempt accounting
remain the following leaf. The 32-element observation-counter list in
`orchestrator/measurements.rs` is instrumentation, not an admitted guide-flow cap.
Spec08 retains eight automatic/two explicit outside attempts under the existing
owner; spec95 does not authorize a new resource quota. Parent/effective policy,
consumed dependencies, public selection and integrated closure remain pending.
