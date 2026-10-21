# Owned Alpha Values And Eager Mapping Materialization

Source baseline: `e80b59c`. This implements the owned-value portion of
`data-alpha-design.md` privately. `AlphaValues` and `AlphaSemantics` are not yet
crate-root exports; explicit bindings, actual mark consumers and the unified
automatic guide envelope have their own following implementation leaves.

## Owned Values And Complete Declarations

`src/alpha.rs` owns the name, final byte occurrences, semantic members and complete
immutable declaration. Clone shares immutable storage, while equality compares
values. No function, borrowed source or floating backend opacity is retained.

- `new` validates the nonempty, trimmed, control-free source name and eagerly owns
  the byte iterator. Empty arrays are valid; zero bytes keep their positions.
  Cardinality and a suggestive name do not create a guide declaration.
- Linear and Log10 own the declared original domain, range, numeric examples and
  original finite source members. Log10 additionally retains its validated finite
  binary64 coordinate endpoints. Original out-of-domain members remain distinct
  from their clamped evaluation keys.
- Explicit custom continuous construction evaluates the union of distinct clamped
  source coordinates and required original-domain representatives in numeric order,
  exactly once per coordinate. It retains the complete finite evaluation table.
  An empty source still evaluates its declared representatives.
- Categorical construction evaluates each raw category once in first-observed
  order, without whitespace or Unicode normalization. It owns that complete table
  and the original occurrences' category indices. An empty source calls no function.
- Threshold construction owns all finite ascending boundaries, m+1 bytes, generated
  interval labels and each numeric occurrence's original value plus interval index.
  Equality belongs to the interval on the right. No boundaries plus one byte is
  the declared `all values` interval.

All mapped sources validate null/nonfinite values, with source indices, before
any application function runs. Log10 also rejects nonpositive members and domain
endpoints, and rejects indistinguishable transformed endpoints. Constant source
data is valid inside an explicit nonconstant domain; no domain is inferred.

`select` permits repeated valid indices, copies exactly those bytes/members and
retains the complete declaration. It rejects an out-of-range index. A pooled
custom mapping can therefore outlive its original source and supply selected
inputs without re-evaluation. The contract distinguishes two selected mappings
whose visible bytes and representatives agree but whose unused finite evaluation
table differs; independent equal construction compares equal by value.

## Shared Numerical Owners

`src/numeric/mapping.rs::normalized_fraction` is the only finite coordinate
normalization owner for alpha and continuous colors. The former
`ContinuousColors::normalized_fraction` method is removed, and its mark, diverging
center, gradient-stop, continuous guide and Beeswarm test callers are migrated.
The arithmetic retains exact endpoints, the ordinary finite-span quotient,
opposite-extreme scaled normalization and color's constant-domain midpoint.

Alpha evaluates `a0 + t * (a1-a0)` in that order and rounds nearest/ties-to-even
once, checking the resulting finite byte range before conversion. Independent
witnesses establish `[0,64,128,191,255]` for linear quarters and logarithmic decades,
descending and constant ranges, midpoint ties 0.5→0 and 1.5→2, clamping, adjacent
endpoints, subnormals and opposite extremes. Log10 paint coordinates reuse the
existing platform-independent binary64 log10 owner; Forest's mathematical power
display engine is not involved.

`src/numeric/set.rs::guide_presentation` now owns the existing whole-set guide
grammar and precision search. It replaces the descriptor-local one-to-three-value
helper, serving color domains, up to five continuous alpha representatives and
arbitrary threshold boundaries. It preserves the omitted Automatic grammar and
whole-set Scientific Fractional 2..=17 collision retries. Label-index sorting checks
distinctness without imposing numeric or lexical ordering on the returned members.
For boundaries 1 and next-up(1), the independent selected labels are
`1.0000000000000000e0` and `1.0000000000000002e0`, both fractional precision 16.

The temporary private-module dead-code expectation and consuming-constructor
Clippy expectation are staging boundaries, to be removed when actual consumers
and public exports make them unnecessary. No public placeholder is introduced.

## Focused Verification And Test Replacement

**23 distinct focused library tests and four external contracts pass**:

- Six independent alpha contracts: linear quantization/finite domains, logarithmic
  validation/clamping, eager continuous construction, raw categorical construction,
  threshold membership/labels, and selected complete-recipe identity.
- The existing numeric semantic-set test, all nine continuous guide tests, the
  actual shared Beeswarm mapping test, all five scale-value tests, and Shared
  numeric-guide pre-font policy compatibility.
- External Heatmap finite extremes, exact constant sequential/diverging domains,
  local/Shared close numeric labels, and a real Shared gradient across both
  continuous-color families without borrowing their alpha or chrome.

The six new unit contracts replace these six thinner GeometryBackend tests in
the same change, with no test moves or banked deletions:

- `surfaces_validate_without_becoming_collision_obstacles`
- `point_envelopes_and_weighted_overlap_keep_the_exact_local_rules`
- `zero_width_lines_are_omitted_after_geometry_validation`
- `scatter_and_polyline_record_individual_local_obstacles`
- `geometry_records_signed_text_without_unsigned_saturation`
- `linear_gradient_records_its_rectangular_bounds`

The remaining geometry tests retain canonical-coordinate parity, transparent
paint/invalid-input behavior, shape-specific painted unions, clipping, text ink,
and lifecycle coverage. No removed assertions were bundled into another test.
Discovery remains **671 library tests, including 17 ignored**. Strict all-target
Clippy, `just fmt-check` and `git diff --check` pass. Logs are
`/tmp/opencode/owned-alpha-{lint,values,label-sets,continuous-guides,beeswarm,scales,shared-labels,color-extremes,color-constant,color-labels,color-shared,count,ignored}.log`.
Count/ignored commands only list tests. This leaf does not claim public API,
actual alpha mark/guide rendering, a full-suite run, or release/time/RSS closure.
