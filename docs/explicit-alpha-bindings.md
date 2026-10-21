# Explicit Alpha Bindings And Original Source Slots

Source baseline: `50154b2`. This closes the private binding and retained-input
portion of `data-alpha-design.md`. Actual observation, series and fill consumers
are the following leaves; they must call this prepared lookup boundary before
fonts and retain its selected operands. Public component attachments and alpha
guide rendering still belong to their later cutovers.

## One Complete Instruction Per Target

`AlphaEncoding` has a private closed representation: Theme, Observations, Single,
Keyed or Bins. Theme is Default and explicit reset. Single requires exactly one
owned alpha value and explicitly broadcasts it; a repeated row array is never
reinterpreted as a series/aggregate instruction. `values()` exposes the retained
opaque values without changing their declaration.

One immutable three-target `AlphaInputs` is stored by Plot construction, Clone and
exhaustive marker erasure. The private attachment replaces only Point, Line or
Fill's entire instruction. Composed retention and `same_composed_inputs` compare
the full input by value independently of numeric policies and guide dependencies.
HeightInputs also retains and compares it now, before the mark/guide migration.

## Complete Raw Selectors And Exact Bins

Keyed construction validates uniquely named, nonnull selector columns aligned
with the alpha values and requires at least one field. Field order is canonical
raw-name order; original row-to-alpha associations remain intact. Duplicate
complete tuples reject. Raw strings retain whitespace and Unicode bytes without
alias lookup or normalization. Empty aligned tables are valid, but cannot resolve
a real unit. Pooled tables may contain unused tuples.

Bin construction validates aligned, finite, nonmissing lower/upper pairs with
lower < upper, optional aligned facet columns, and unique complete interval/facet
keys. Signed zero has one identity. Lookup is exact: changing a prepared edge by
one next-up step misses instead of moving a bin or adjusting its key.

Prepared lookup distinguishes Observation, Series, Aggregate, Bin and Single
unit classes. It validates binding kinds, complete original observation length,
required actual unit fields and only the owning Plot's available facet fields.
Each real keyed/bin unit must match its complete selected schema. Missing keys,
unknown fields, conflicting raw values for a same-field unit, and invalid actual
bin bounds are Data errors, including for alpha zero. A resolved operand retains
both the selected source-member index and final byte for later truthful guides.
There is no partial-key search, implicit aggregation or guessed row representative.

## Original Slots And Prepared Ownership

`PlotData::observation_alpha` uses `row_index(local_row)`, not `source_row(local_row)`.
The existing source/operation contract now proves selected rows `[2,0]` with
diagnostic IDs `[302,300]` resolve alpha indices `[2,0]` and bytes `[230,0]`.
Local and original out-of-range slots reject; an original-slot diagnostic keeps
its independent public source-row ID.

Plot-owned schema preparation obtains actual facet names directly from FacetSpec.
Its observation length is the complete original native input length, or the
checked sum of original supplied Raincloud group point counts. The strengthened
supplied-geometry contract proves a synthetic zero-row PlotInput still aligns
two alpha values with its two actual supplied strip points.

PreparedAlpha owns its immutable encoding and declared unit class/length, so it
can outlive the source Plot. It contains no borrowed category input or application
function. Actual producers must preserve the selected member index beside any
sorting/filtering/grouping and final styles; the point/line/fill consumer leaves
own that migration and paint-aware intrinsic validation.

## Focused Verification And Replacement

**19 distinct focused library tests and four external contracts pass**:

- All nine alpha value/binding tests, including canonical field order, exact bin
  matching, schema/refusal/completeness, explicit broadcast/reset, temporary-source
  ownership and full three-target Clone/erase/composed retention.
- The original row-source/operation contract, supplied-geometry contract, height
  identity test, actual child-cache test, and all six existing composed generation
  tests.
- Height recording misses independently for point-byte, line-binding and fill-
  binding changes at equal width/domain. The actual child cache distinguishes
  Undeclared, Linear and custom Continuous declarations with identical alpha-127
  paints, and reuses unchanged complete values.
- External original SourceRows after compaction/reordering, facet source-row
  retention, sample/supplied Raincloud output parity, and Shared Beeswarm guide
  ownership/final labels.

Three new binding unit contracts replace these three thinner tests in this leaf:

- `source_storage_preserves_values_and_refuses_unrepresentable_lengths`
- `geometry_records_the_same_converted_subprecision_points_as_svg`
- `geometry_and_svg_reject_the_same_collapsed_line`

The original row-source/operation, supplied-geometry, height and child-cache
tests are strengthened in place for their changed contracts. No removed test is
moved or bundled elsewhere. Discovery remains **671 library tests, including 17
ignored**. Strict all-target Clippy, `just fmt-check` and `git diff --check` pass.
Logs are `/tmp/opencode/alpha-bindings-{lint,values,rows,supplied,heights,children,composed,row-caller,facet-caller,supplied-caller,parent-caller,count,ignored}.log`.
Count/ignored commands only list tests. Full-suite, public alpha output and
release/time/RSS closure are not claimed by this private binding leaf.
