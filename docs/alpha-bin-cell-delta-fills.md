# Exact-Bin And Original-Record Fill Alpha

Source baseline: `aaa678d`. Histogram bins, finite Heatmap cells and Waterfall
input-delta bodies complete **15 of 15 fill consumers** and the private actual
mark-consumer census of **49 components: 17 point, 17 line, 15 fill**.

## Units, Final Paint And Local Identity

- Histogram resolves each prepared exact `(lower, upper)` interval with only its
  selected owning facet fields. Auto/sample bin selection and supplied bin edges
  are scientific inputs; neither moves to fit an alpha table. Each actual bin,
  including a transparent or zero-height bin, retains its selected member. The
  final bin polygon owns its fill style, replacing the former global payload fill
  throughout draw, retained geometry and automatic ink. Intrinsic ink uses the
  same actual final bin fills. KDE and rug strokes remain independently owned.
- Heatmap resolves only actual finite input cells by original observation slot,
  retaining the diagnostic source ID independently through normalization, explicit
  category ordering, scoped row selection and topology construction. Absent/null
  cells, missing crosses and fully-null merged rows consume no finite-cell alpha.
  Finite zero-alpha cells still retain scientific values, domains and annotations.
- Heatmap has one final finite-cell color resolver shared by pre-font annotation
  requirements, intrinsic text, assigned padding/tracks and final cell/text plans.
  Thus final alpha selects the truthful contrast role before fonts. Shared color
  remapping changes RGB through the existing owner while retaining cell alpha.
  Independent normal/highlight/missing outlines keep their own paints.
- The Heatmap intrinsic fast path now compares both the complete fill binding and
  retained actual membership. Equal final bytes from different binding instructions
  cannot reuse a stale analysis; immutable actual-member storage is shared by Arc
  and compared by value.
- Waterfall retains original-slot alpha beside each input delta and cumulative
  span. Selected body fill and mark intrinsic ink replace only that byte. Sign RGB,
  cumulative arithmetic, connector/reference strokes, total text and category slots
  remain independent. No alpha is manufactured for the base or terminal totals.

## Focused Evidence

**Nine library contracts passed** with nonzero offline filters:

- Actual Histogram bins in two facets retain independently expected edges
  `[1,4.5,8]` / `[11,14.5,18]`, counts `[4,4]` and bytes `[128,64]` / `[230,0]`.
  A reversed exact-interval table plus an unused pooled facet resolves correctly;
  missing exact keys fail before additional font discovery, even when the supplied
  actual key is transparent.
- Sampled Histogram KDE/rug ownership retains scientific bins/curve/rugs under bin
  bytes `[0,230]` and independent curve byte 0. Existing bin/optional-overlay hard
  ink coverage also passes.
- Heatmap retains slots/source IDs `(2,300)`, `(3,400)`, `(0,100)` in explicitly
  ordered finite cells with bytes `[230,64,0]`, separately from a missing grid cell.
  The zero-alpha maximum remains a scientific guide-domain sample.
- Final Heatmap alpha 0 selects the dark 11px annotation role instead of the opaque
  light 22px role; padding, track containment and capture use that complete role.
  Equal-byte Observations/Single bindings invalidate the intrinsic cache separately
  from the existing equal-text numeric-policy witness.
- Pre-font Heatmap census requests only the role selected by actual data bytes,
  including alpha 255 overriding theme-zero fill and alpha 0 selecting the backdrop
  contrast. Existing Shared-domain role reselection and transparent missing-row
  role/backdrop coverage pass.
- Waterfall retains original slots `[0,1]`, independent IDs `[400,100]`, bytes
  `[0,230]` and exact cumulative spans `10->12->11`, while connectors/references
  retain their owning styles. Existing breadth, positive-ink clearance and
  transparent connector refusal/acceptance relationships pass.

The richer exact-bin contract replaces the old Histogram
`auto_uses_freedman_diaconis_for_nonzero_iqr` test in place, retaining its Auto
two-bin scientific case while adding exact intervals, real facets and completeness.
The new Heatmap original-member contract replaces
`layout::interior::preference::tests::path_count_does_not_multiply_connectors_or_supplemental_ink`,
a four-vertex fixture-count check superseded by retained Line/Trajectory
source-order/observation contracts. No deletion is banked. Discovery remains
**671 library tests including 17 ignored**, with the ignored-only listing executed.

**Twelve existing public integration contracts passed**: five Heatmap triangle,
category-order, annotation/contrast, merged-null-row and native PNG contracts;
deterministic Histogram statistics and supplied/sample recorded-mark parity;
three Waterfall precision/overflow/source-row contracts; and two Canvas Shared
continuous/diverging Heatmap owner/domain/missing-key contracts.

Strict `just lint`, `just fmt-check` and diff checks passed. Actual private paint
consumers are complete. Unified alpha explanations, parent/feedback guide identity,
public API migration and integrated full/release verification remain subsequent
work; the current API still exports no alpha attachments.
