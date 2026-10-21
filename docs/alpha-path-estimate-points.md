# Path, Interaction And Estimate Observation Alpha

Source baseline: `90131e0`. The observation-consumer implementation now also covers
Line, Trajectory, LinePath, Interaction, PointInterval, Coefficient and Forest.
Together with coordinate points this covers ten of the seventeen admitted point
families. Dumbbell, categorical distributions and native/supplied points remain
before the observation leaf can close.

Path preparation resolves each original row's alpha before numeric sorting or
grouping. Line's primary markers freeze contiguous final-style batches after their
connector; LinePath preserves source vertex order, and Trajectory retains every
condition's own operand. Primary data alpha never reaches independent PointLayer
paint or its labels. Optional hidden markers still validate their full input schema.

Interaction retains one observation operand in each actual condition estimate.
Selected traces preserve interval/connector ordering and freeze per-cell marker
batches, retaining group shape and independent outlines. Painted intrinsic and
full-ink measurement use those same final styles.

PointInterval carries the original observation operand through explicit/value
category ordering. The shared estimate owner retains all available/unavailable
slots and resolves Coefficient/Forest operands before display-only coordinate
transforms. Each selected estimate row owns its final point style. Unavailable
rows keep text/category occupancy and cannot gain a numeric point from alpha;
zero point alpha does not hide an independently owned interval or cap. Intrinsic
point ink uses available rows' actual final paint, through the common ink owner.

## Focused Verification

**Seven focused library tests and four external contracts pass**:

- All four Line tests, including new sorted-row alpha and source-ordered LinePath
  contracts. Existing supplemental exact-paint tests now also configure primary
  data alpha and prove their independent paints remain intact.
- Actual Interaction raw group/style matching, strengthened with prepared and
  selected per-cell alpha. The new render portion explicitly selects raw labels:
  its intentionally distinct `A B`/`A  B` identities collide under default
  humanization, and the production collision refusal is preserved.
- A new PointInterval contract proves `[1,2,0]` original slots after explicit
  category reordering, unavailable text without fabricated marks, and independent
  interval presence for a zero point alpha.
- The existing native categorical Trajectory contract now also proves actual
  selected point alphas, preserving raw/display identity and categorical axes.
- External shared estimate-status behavior across PointInterval/Coefficient/Forest,
  Line ribbon/marker ordering and determinism, exact raw Forest Log10 projection,
  and styled Interaction circle/triangle geometry.

The three new tests replace `qq_identity_and_hidden_modes_are_explicit`,
`qq_handles_large_finite_observations`, and `qq_explicit_range_is_resolved_exactly`
in this change. Their thinner constructor/finite/getter inventories are not copied
or moved. Existing public Qq reference/extreme/explicit-limit contracts remain.
Discovery is **671 library tests, including 17 ignored**. Strict all-target Clippy,
`just fmt-check` and `git diff --check` pass. Logs are
`/tmp/opencode/alpha-path-estimate-{lint,line,interaction,interval,trajectory,status-caller,line-caller,forest-caller,interaction-caller,count,ignored}.log`.
Full-suite, public alpha/guide output and release closure remain later gates.
