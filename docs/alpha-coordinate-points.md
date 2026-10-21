# Coordinate Observation Alpha And Painted Ink

Source baseline: `1cb8ea1`. Scatter, Qq and Regression now consume prepared
observation alpha. This is a bounded part of the observation-consumer leaf;
path, estimate, categorical-distribution and native/supplied point consumers
remain in that leaf before public cutover.

`alpha/marks.rs::ObservationAlpha` retains original slot, independent diagnostic
source-row ID and resolved byte/member index during scientific preparation.
Scatter and Regression retain this operand through joint-finite filtering. Qq
sorts by scientific value plus original occurrence order, so repeated values keep
their own alpha association. The complete source array is validated before fonts,
including scientifically discarded slots. Alpha zero does not change scientific
domains, duplicate counts or Regression fit inputs.

Projection replaces only the point-fill alpha after the normal RGB/role/exact
paint has resolved. Independent outlines retain their complete paint. The sole
selected point representation contains final styles in contiguous batches: an
alpha sequence 0,128,0 remains three ordered batches, including all coincident
points. Capture/draw never looks up source alpha or invokes an application function.
Fallible batch storage grows geometrically rather than reallocating per point.

One common painted-point ink owner serves pre-allocation intrinsic requirements
and automatic numeric-axis ink feedback. It uses actual resolved paint visibility,
including data alpha making a theme-zero fill visible, and only painted source
extrema for outward-ink requests. Scientific ranges retain every eligible point.
For fixed geometry, zero/nonzero fill states share at most two canonical-bound
evaluations; independent visible outlines still contribute with a zero fill.
The old Regression all-observation ink-range field is removed in favor of these
actual painted extrema. Other scientific fit/band ranges remain independent.

## Focused Verification

**19 distinct focused library tests and three external contracts pass**:

- The new Scatter test proves original slots/IDs after filtering, coincident point
  completeness, 0/128/0 paint order, independent outlines and Hidden pre-font
  refusal of a short array.
- All 14 Qq tests pass. The existing sorting test is strengthened to prove stable
  duplicate membership and actual selected alpha styles.
- All four Regression style contracts pass, including unchanged fit geometry after
  joint-finite filtering and positive local point-ink requirements when data alpha
  overrides a theme-zero fill. The initial test compared a root minimum dominated
  by unrelated coordinate chrome; its corrected witness compares the actual mark
  intrinsic owner. Production root sizing is not changed to satisfy the test.
- Public Scatter joint-null/NaN hue filtering, Qq deterministic/reference output,
  and Regression automatic frame clearance pass.

The two new Scatter/Regression tests replace `qq_supports_two_observations` and
`qq_rejects_too_few_observations` in this same change. The sorting test is replaced
in place, with no copied old test bodies or integration migration. Library
discovery remains **671, including 17 ignored**. Strict all-target Clippy,
`just fmt-check` and `git diff --check` pass. Logs are
`/tmp/opencode/alpha-observation-{lint,scatter,qq,regression,regression-alpha,scatter-caller,qq-caller,regression-caller,count,ignored}.log`.
The successful three Regression neighbors and focused corrected test establish
the four passing contracts; the first failed assertion is retained in its log.
No full-suite, public alpha API or release closure is claimed here.
