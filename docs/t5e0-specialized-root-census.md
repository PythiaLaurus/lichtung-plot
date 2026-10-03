# T5e0 — Historical Specialized Packing And Root-Request Handoff

This preserves the **T5e0 starting inventory**, after T5b native parents, T5c
mark/annotation requirements and T5d resolved scales. The legacy routes, floors,
strips and exact-only private request described below are historical, not the
current implementation. The current owner table follows the retained inventory;
`docs/t5e7-producer-census.md` contains the all-producer closure census.
Target algorithms and preferences remain those in `specs/05_LAYOUT_SYSTEM.md` §§ Root Sizing,
Tracks, Specialized Geometry; `specs/92_T5B_T5E_EXECUTION_PLAN.md` owns the
stage order. T5a already closed font preparation, selected capture, dummy
root plans, backend discovery and post-freeze reconstruction. Those are
retained foundations, not new T5e deletions.

| Stage and then-active route | Prepared/hard input and then-remaining packing policy | Replacement/deletion boundary |
| --- | --- | --- |
| T5e1 Heatmap, `SpecializedLegacy::Heatmap` | `HeatmapData` retains the full normalized x/y grid, active/triangular cells, source values, resolved fills/guide stops, highlights, missing cells, merged null rows, value-label text and final contrast. `DeferredHeatmapMinimum` supplies the 40px body and 60/90px height parity floor; `prepare_intrinsics` adds complete cell ink and required text. `measure` still carves a right colorbar reserve, computes label widths/elision and equal projected cells from its residual panel. | Allocate measured x columns before constrained cell text produces y-row requirements, using T5b Grid/track quanta; hard cell ink and full required labels, soft `2em` pitch and square preference. Preserve explicit Heatmap Wrap/Elide ownership, colorbar and missing semantics. Remove `DeferredHeatmapMinimum`, local reserve/equal pitch and the Heatmap route from `SpecializedLegacy` once selected cells and guide fit are native. |
| T5e2 Matrix, `SpecializedLegacy::Matrix` | `PreparedMatrixAxes` owns variable names, group identities, per-variable numeric ranges, diagonal KDEs and retained axis sources. `intrinsic_body_minimum` measures point/density and longest variable-label ink, multiplying one cell by `n` with 8px gaps. `measure` reserves legend/labels, tries at most eight local cell widths/heights, enforces a 40×40px legacy cell floor, and then shares x/y tick plans across the variable square. | Allocate declared `n×n` topology and spans once with column/row maxima and preferred side `max(hard_width,hard_height,12em)`. Keep every diagonal/off-diagonal cell and common `PanelAxisPlan`; delete the local stabilization loop, hard 40px parity floor and Matrix specialized route, not scientific KDE/source-row preparation. |
| T5e3 Marginal, `SpecializedLegacy::Marginal` | `PreparedMarginal` contains original central pairs, raw hue/group identities, linked x/y KDE grids and domains. Its intrinsic measures complete point and density ink but still adds fixed 12px gap and 96px right/82px top bands. `measure` subtracts those fixed strips, then projects central points and two density polygons; changed shared domains can rebuild *render grids* from retained samples. | Use the native two-by-two Grid: top `(0,0)`, empty `(0,1)`, central `(1,0)`, right `(1,1)`. Derive top/right preferred sizes once from central natural preference (quarter central height/width), share exact x/y tracks and check one assigned frame. Delete fixed strips and Marginal specialized route; retain source statistics and final raw-key RGB. |
| T5e4 Beeswarm, `SpecializedLegacy::Beeswarm` | `PreparedBeeswarm` retains normalized category levels, finite numeric/color rows with source identities, final palette/guide stops and points. `finish` currently sorts rows by category/x/color/source row, assigns 48 x bins, then alternates ±0.09 category offsets by occupancy layer capped at 0.36. `intrinsic_body_minimum` measures complete glyph ink and the resulting occupied offset; `measure` still reserves its local colorbar and at least 100px numeric body. | In source-row order per category at assigned numeric width, try zero then ordered/deduplicated tangent offsets, checking every previously placed glyph pair under the admitted work ledger; never repack a prior point. Use occupied hard pitch and categorical tracks, with `4:3` natural panel preference. Delete bin/alternate-jitter producer and Beeswarm specialized route; retain exact color and source identity. |
| T5e5 Waterfall, `SpecializedLegacy::Waterfall` | `PreparedWaterfall` retains signed source deltas, exact cumulative endpoints, categories and base/terminal total strings. Intrinsic supplies complete bar/connector/reference ink and currently adds an annotation-height estimate. `measure` calls `plan_annotations` over base then terminal, scanning prior shelf intervals, subtracts the selected band's height and enforces `MINIMUM_MARK_PANEL_HEIGHT`; selected bars/connectors and two required labels are frozen. | Assign native categorical height and base/terminal greedy shelves in source order, with exact text ascent/descent, baseline, band reserve and typed annotation fit. Preserve crossing connectors, signed zero and explicit data-window policy; remove fixed minimum/band and Waterfall specialized route, not its finite cumulative semantics. |

The five route entries in `layout/interior.rs::SpecializedLegacy` were scheduled
for deletion by their named stages, not replacement by an alternate renderer.
Shared/faceted
parent eligibility stays unchanged: Matrix and Marginal remain self-managed
and cannot enter generic facets; Matrix cannot be a Canvas child, and a
faceted Plot cannot be nested in Canvas. Canvas may retain the other valid
specialized child kinds. These are pre-font composition gates, independent
of each family's packing transition.

## Historical Root Ownership Handoff For T5e6

`layout/kernel.rs::plan_exact_root` already owns preparation once, the
at-most-eight-round resource/feedback driver and one selected frozen plan.
It currently takes one exact `(u32,u32)` and `RootSize` validates the
assignment. `Plot::size` stores exact pixels (default 640×480);
`Canvas::size` does likewise (default 960×640). `builder/layout.rs::
legacy_root_axes` and `layout/canvas.rs::LegacyRootAxes` still supply
specialized axis inset estimates to the measured `RootFramePlan`; native
standard/categorical panel soft preferences already live in
`layout/interior/preference.rs`. `layout/kernel/tracks.rs` allocates the
checked quantized Grid/sequence tracks used by the T5b parents.

The one private request boundary must add **automatic width+height** and
**exact width/automatic height** alongside the existing exact pair, under
the same driver. Automatic outer width uses bottom-up text/guide/body
preference once (`66ch` for prose; a preferred RIGHT outside guide is an
intrinsic input, not a committed edge); constrained height then measures at
that width. Never start with a hidden 640×480/960×640 trial or repeat width
selection after edge recovery. Private tests exercise the new requests;
public constructors/getters/default exact behavior remain until T6. Canvas
owns cell dimensions even if an embedded Plot stores a standalone size.
Existing invalid dimension timing is retained: zero/invalid exact inputs
fail at the current preflight root boundary, and there is no new public
`max_width`, policy, exact aspect or extra hard floor.

## Historical Verification Inputs

- Heatmap: `plots/heatmap/{extraction,tests}.rs`, `tests/heatmap`, label-fit
  and missing-row contracts, final T5d sampled stops/contrast; the ignored
  `benchmarks::text::specialized_text_search_measurements` group.
- Matrix/Marginal: prepared-semantic, tick/cache and selected-capture units,
  `tests/marginal_matrix` and the `matrix_clip_structure` and
  `marginal_clip_structure` snapshots. Their local search and fixed strips
  are current baselines, not target policies.
- Beeswarm/Waterfall: `plots/{beeswarm,waterfall}/extraction.rs`, exact-root
  geometry/resource tests, the Beeswarm clip snapshot and inspected
  Publication/Science and Waterfall parent SVG fingerprints. T5e4 expects
  intentional Beeswarm geometry changes; T5e5 must keep signed scientific
  endpoints and source-aware collapse tests.
- One root feedback/cache/ledger: `layout/kernel/{feedback,tracks}.rs`,
  `layout/interior/coordinate.rs`, `decorations/plan/children.rs`,
  `canvas/tracks.rs` and `orchestrator/facets.rs`. Hits admit calls;
  materialization admits records/bytes before allocation; searches,
  comparisons and root trials charge visits/calls independently. Rejected
  trials keep their work price, but capture/Null/SVG replay do no preparation
  or selection. The T5e7/T5e8 gate tests three requests without replacing
  any of these owners.

## Current Implementation Owners At The T5e7/8 Census

This table describes source ownership, not a substitute for the phase completion
gate in spec 92. Paths below are relative to `src/`.

| Producer | Prepared → intrinsic → assigned measurement/arrangement → frozen owner | Concrete replacement |
| --- | --- | --- |
| Heatmap | `plots/heatmap.rs::{HeatmapData,HeatmapIntrinsicCache}` → column-local cell/text/axis requirements → `assigned_columns`, constrained annotations, `assigned_rows`, `select_cells` → `HeatmapPayload`, common `LegendDrawPlan` | Native quantized columns precede width-dependent rows; complete required cell text and missing/highlight geometry are checked. The deferred body/height floor and residual equal-pitch policy are gone. Explicit Wrap/Elide remains source-owned. |
| Matrix | `plots/matrix.rs::PreparedMatrixAxes` → variable-local hard requirements and `12em` cell preferences → common Grid allocation, variable tick cache and selected cell axes → `MatrixPayload` | The full declared variable square survives, with column/row maxima and unequal tracks. The local eight-pass cell stabilization and 40px cell floor are gone; the root's eight-round feedback driver is retained. |
| Marginal | `plots/marginal.rs::PreparedMarginal` → central point/axis and density-stroke requirements → linked two-by-two Grid and three `LinkedPanelPlan` children → `MarginalPayload` | Fixed 96px/82px strips are replaced by quarter-central natural preferences and exact shared spans. Retained samples still supply domain-dependent KDE render grids; that scientific work is admitted measurement, not replay. |
| Beeswarm | `plots/beeswarm.rs::PreparedBeeswarm` → complete glyph radius and category requirements → `plots/beeswarm/packing.rs::pack` at assigned numeric width, then categorical tracks → `BeeswarmPayload`, common `LegendDrawPlan` | Source-ordered zero/tangent first-fit packing replaces 48-bin alternating offsets. Every tested pair is charged; insufficient occupied height refuses without repacking prior points. The local 100px numeric floor is gone. |
| Waterfall | `plots/waterfall.rs::PreparedWaterfall` → signed spans, complete mark ink and total-label metrics → native category tracks plus `plan_annotations` base/terminal greedy shelves → `WaterfallPayload` | Measured baselines, shelf band and text containment replace estimated/fixed bands and the mark-panel floor. Original deltas, cumulative endpoints, source IDs and explicit total visibility policy remain. |
| Root requests and parents | `layout/kernel.rs::plan_exact_root`; `layout/interior/sizing.rs`, `canvas/sizing.rs`, `orchestrator/facets/sizing.rs` → bottom-up preferences → native Canvas/facet Grid allocation and bounded root feedback → `FrozenRoot` containing one `TextReplayPlan` | Private automatic and exact-width/content-height requests join exact pairs in the same driver. Width is chosen once; `ContentHeight` is a dependency-keyed replacement consequence, including contraction. `legacy_root_axes`, `LegacyRootAxes` and the old parent adapters are gone. Public exact defaults remain until T6. |

`InteriorRoute` currently has 8 standard-coordinate, 12 one-categorical-axis and
5 specialized-grid kinds. `CompositionChildPolicy` enforces the unchanged early
exclusions: Matrix/Marginal generic facets, Matrix Canvas children, and faceted
Plot children in Canvas. The five specialized names classify active native
family producers; they do not retain `SpecializedLegacy` packing. See the new
census for payload/source counts, admission boundaries and test coverage.
