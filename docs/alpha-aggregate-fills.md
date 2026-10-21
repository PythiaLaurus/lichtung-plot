# Explicit Alpha For Categorical Bodies And Stack Segments

Source baseline: `b3bf2fa`. Bar, HorizontalBar, StackedBar, StackedHorizontalBar,
Boxplot, Violin and sampled/supplied Raincloud connect **7 of 15 fill consumers**.

## Final Paint And Scientific Units

- The shared final `fill_style` replaces only the selected body fill alpha byte;
  stroke color/alpha/width remain independent. It uses the existing `FillStyle`
  and retained primitives rather than enclosing opacity or another draw path.
- Prepared bars and distribution groups retain independent fill and line members.
  A common `ScopedAlpha::aggregate` combines the actual raw category and optional
  positioned-series keys. Selected owning facet fields retain their existing exact
  matching and original-slot preparation. No sample-row alpha is reduced.
- Every actual body or stack segment resolves, including transparent or scientific
  zero-height segments. Duplicate sample aggregation, category/series ordering,
  positive/negative stack accumulation, interval calculations and source ranges
  remain scientific inputs independent of alpha.
- Both bar orientations, Boxplot and Violin use actual per-body final fill for
  intrinsic painted bounds. Selected body styles already drive retained geometry,
  automatic ink, capture and draw. Supplied Raincloud resolves category/optional-hue
  body keys from its geometry dimensions, independently of its zero-row input.
- Boxplot summaries/outliers and Violin/Raincloud inner fill/outline, summary
  strokes and observation strips retain their independent paints.

## Focused Evidence

**Seven library contracts passed** with nonzero offline filters:

- The faceted ordinary-bar contract now compares independent uncertainty bytes
  `[128,64]` / `[0,230]` and body bytes `[0,230]` / `[64,0]` in both orientations.
  Theme-zero body fill is replaced while the independently mapped outline keeps
  alpha 61. Scientific means, actual interval hard ink and source-read counts retain
  their previous independent witnesses.
- The new stacked-body contract verifies both orientations with six source rows,
  five actual segments, duplicate-row cancellation to a retained zero segment,
  positive accumulation from 2 to 6, a separate negative segment to -2 and explicit
  bytes `[128,230,0,64,17]`. Transparent/zero segments remain prepared and selected;
  outlines and absence of uncertainty remain independent.
- Existing Boxplot, Violin, supplied-geometry and sampled/supplied strip contracts
  now assert independent final body bytes alongside their point/summary bytes.
  Supplied category/hue bodies retain `[191,0]`, while inner fill stays unchanged;
  Violin's positioned bodies retain `[128,64]` independently of summaries `[0,230]`.
- Existing discrete bar source provenance still distinguishes fixed/mapped outlines
  with equal RGB and consumes only actually observed series.

The new stacked-body test replaces
`layout::interior::preference::tests::standard_distribution_counts_exclude_samples_and_generated_vertices`,
which checked four fixture counts without testing scientific accumulation or paint
ownership. No deletion is banked. Library discovery remains **671 including 17
ignored**.

**Seven existing public integration contracts passed**: duplicate-row stacked sums;
post-aggregation bar value ordering; positioned/stacked series legends; sparse
faceted dodge slots; supplied/sample Raincloud recorded-mark parity; visible
Boxplot overlays retaining all source rows; and separated Raincloud cloud/box/strip
geometry. Strict `just lint`, formatting and diff checks passed.

Remaining fill consumers are Histogram bins, Density areas, Line bands, Regression
bands, Heatmap finite cells, Marginal densities, Matrix diagonal densities and
Waterfall input deltas. Unified guides/parents, public cutover and integrated
full/release verification remain subsequent leaves.
