# One Typed Guide Envelope And Continuous-Color Content

Source baseline: `0ed8c27`.

## Current Representation And Ownership

`PreparedGuide` is the sole immutable guide descriptor. Its ordered sections carry
`GuideSectionBody::Symbols` or `ContinuousColor`; the former exclusive
`PreparedGuideDescriptor`, `GuideKind`, `MeasuredContinuousGuide`, and Canvas's
parallel measured-guide enums are removed. Symbol identities, provenance,
Combined/Separate factorization and complete key equality retain their contracts.

`ContinuousDescriptor` still owns final scientific domain, resolved scale, stops,
numeric members/presentation, missing semantics and formatting. Its content is
measured by `continuous::ColorContent` and frozen as `MeasuredColorBody`. Neither
owns a title, frame, padding, placement attempt, independent cache or renderer.
`Legend` measures all section headings, applies the single finite track allocation,
and produces one frame and one `LegendDrawPlan`. Normal/compact variants and the
existing root-local `GuideMeasurements` cache are common to both section bodies.

Canvas Shared first resolves and installs its final continuous scientific domain
and gradient. Local prepared contributions then retain those final operands and
are merged under the Canvas composition. Categorical-symbol/continuous-color
Shared incompatibility still rejects before fonts. PerPlot retains local bodies;
Hidden has explicit absence. Native Heatmap/Beeswarm use the same measured Legend
beside their scientific body. Native facets keep their own domains and missing
blocks: each actual scope owns its descriptor and font census rather than entering
a false parent-wide continuous union. Matrix's native ownership remains local.

Selected producers retain their prepared contribution, including the two native
color families. Fonts traverse the same section descriptors; the duplicate local
and Canvas continuous-label census paths are deleted. Feedback retains the complete
typed descriptor, and scoped child/height dependencies retain its value equality.

## Exact Capacity And Ink

The common allocator retains exact title/body/gap expressions. A section's finite
integer capacity is reduced by its exact title occupancy without separately
rounding a fractional body capacity. Color labels retain their scientific center
fractions, complete projected ink and required adjacent separation; a constant
domain reserves its singleton half-extents and both gaps. Missing outlines increase
the common insets and key gap, independently of frame paint.

Before adding exact outer edges, the complete required inner capacity is admitted
in integer quanta. This prevents cancelling edge/title fractions from producing
an envelope one quantum too small to admit its own measured content. Exact repeat
assignment succeeds; one quantum below the independently exercised singleton
boundary refuses. The existing 76/82 px horizontal and 100/120 px finite vertical
symbol-allocation witnesses remain unchanged, as does the half-quantum native
width witness. Actual selected title, gradient, label, missing and frame ink must
all lie inside the assigned envelope.

## Focused Verification

All commands use locked offline dependencies. Strict `just lint`, `just fmt-check`
and `git diff --check` pass.

- 16 `decorations::legend` tests: source projections, both continuous orientations,
  exact singleton/center clearance, broad frame/missing outlines, absent frame,
  complete ink and unchanged independent symbol allocation witnesses.
- 3 `layout::interior::sizing::tests`, including the exact half-quantum native width.
- 3 `canvas::lifecycle_tests` and the actual scoped child reuse test.
- Both affected Heatmap font-census tests (ordinary values and final Shared-domain
  contrast-role selection).
- All 15 `orchestrator::lifecycle::natural_sizing_tests`; the two native-facet
  ownership cases were rerun after correcting their scoped union/census routing.
- `every_canonical_plot_kind_renders_through_one_lifecycle`: 25 actual families,
  both prepared compositions, source-drop preserved/outlined SVG and PNG, one
  selected capture, final Null validation, and zero capture/post-freeze work.
- Eight public composition contracts: merged Heatmaps, variable and constant
  Beeswarms, diverging/missing Heatmaps, custom cross-family gradient/chrome,
  complete symbol Shared validation, Hidden, and sparse inside facet guides.

The existing scoped cache test now also measures actual Beeswarm color descriptors:
independently constructed equal values reuse the same layout without shaping;
different raw fields with equal title/extents/output invalidate; cached/uncached
frozen SVG is equal. No new unit test was added. The nine existing continuous
tests migrated to `continuous/tests.rs` and exercise the common envelope. Discovery
remains **671 library tests, including 17 ignored**; the ignored listing ran.

Actual declared-alpha member/glyph census and explanatory sections are the next
implementation leaf. Full alpha parent/feedback closure, public cutover and the
integrated full/release/time/RSS gates remain tracked in spec97.
