# Explicit Alpha For Actual Series And Traces

Source baseline: `60b6377`. Line, Trajectory, LinePath, Density and Interaction
now resolve scientific strokes by their actual series keys. This brings the
completed stroke-consumer census to **12 of 17 families**.

## Scoped Units And Final Ink

- The private `ScopedAlpha` preparation operand combines the producer's required
  raw unit fields with only the owning facet fields selected by the binding.
  Facet lookup uses original observation slots, validates one actual scope, and
  preserves raw values. Theme/Single bindings require no selector-value lookup.
- Line, Density and Interaction resolve actual group/hue pairs. Trajectory and
  Neutral LinePath resolve graphical groups without regrouping by row-varying
  hue; Hue LinePath includes its actual group/hue path. Same-field group/hue
  identity continues through the existing source-equality validation.
- Every actual series retains its selected alpha member and byte before fonts.
  Missing keys fail even when another actual key maps to zero; pooled unused
  tuples create no mark, guide source or font requirement. No sample-row alpha
  agreement/reduction, source reordering or callback execution is introduced.
- Line's common scientific preparation is shared with numeric-domain queries;
  actual stroke binding happens for the prepared mark scope. A Canvas domain
  query does not pretend that a full faceted source is one alpha-key scope.
- Line/Density intrinsic strokes and Interaction interval/connector requirements
  use final paint. Their selected strokes preserve RGB, width, pattern and path
  order while replacing only alpha. Line/Density automatic clearance now uses
  actual selected primitive ink rather than one global role-style envelope.
  Independently painted supplemental points and required labels retain their
  existing separate ownership.

## Focused Evidence

**19 library tests passed**: four Line contracts, all thirteen Density tests,
Interaction raw-selector/style matching and the categorical Trajectory contract.

- Line retains sorted point bytes `[17,90,230]`, one keyed series stroke byte 128
  and the independent supplemental point/label byte 61.
- LinePath retains original vertex slots `[0,1,2]` and point bytes `[230,17,90]`
  alongside one explicitly keyed path stroke byte 64.
- Trajectory retains categorical axis identity and point bytes `[0,128,255]`
  alongside its graphical-group stroke byte 128.
- Raw Interaction groups `"A B"` and `"A  B"` match separate keys despite reversed
  selector-table order. Each trace's intervals and connector retain its own byte
  0 or 64, independently of condition point alpha and mapped dash/shape.
- The new Density scope contract prepares eight interleaved observation rows
  using four consumed series/facet alpha values plus one unused pooled tuple.
  Actual scopes retain bytes `[0,128]` and `[230,0]`, replacing a theme-zero stroke.
  Selection rereads no source column. An incomplete map with a zero-valued real
  key fails with a Data error before any additional font discovery.

The Density contract replaces
`admitted_grouped_source_support_matches_the_authoritative_preparer` in place;
that former test compared two paths through the same preparer. No deletion is
banked. Discovery remains **671 library tests including 17 ignored**, recorded in
`/tmp/opencode/alpha-series-stroke-{count,ignored}.log`.

**12 existing public integration tests passed**: source-ordered numeric path;
sorted numeric and categorical Trajectory; supplemental point/ribbon projection;
precomputed Line band; faceted group/hue colors; deterministic Density curves and
its broad opposite-sign/tiny-density numerical case; styled Interaction and sparse
faceted traces; Canvas range-group exact-window label suppression; and faceted
Line repeated x/hue keys with one global guide. Each explicit
`cargo test --locked --offline` filter ran a nonzero number of tests.

Strict `just lint`, `just fmt-check` and `git diff --check` passed. Aggregate
uncertainty/summary strokes in Bar, HorizontalBar, Boxplot, Violin and Raincloud
remain. Fill consumers, unified alpha guides/parents, public cutover and integrated
full/release verification are subsequent leaves.
