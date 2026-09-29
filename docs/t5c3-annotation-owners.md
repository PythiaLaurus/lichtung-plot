# T5c3 annotation and overlay owners

This is the implementation handoff for T5c3. `specs/05_LAYOUT_SYSTEM.md`
defines the placement and fit rules; `docs/t5c0-property-disposition.md`
defines which explicit visibility policies continue to apply.

| Consequence | Prepared source and final owner | Required change |
| --- | --- | --- |
| Pairwise significance | `SignificanceAnnotation::Pairwise`, prepared category/series targets, `src/significance.rs` | Grouped anchors now match the original typed target rather than a synthetic string; next sort by complete span and label ink, assign the lowest free shelf, request automatic numeric domain once and reject incompatible exact windows. Vertical and horizontal planners share this input. |
| Attached significance | `SignificanceAnnotation::Attached`, prepared Bar/Estimate target and rendered mark edge, `src/plots/bar_data.rs` and `src/plots/interval_estimate.rs` | Resolve a prepared category/series identity once, retain glyph and ink, and choose start/end anchors according to orientation. Missing or unrendered target remains a source error, not an invented mark. |
| Waterfall totals and cell/value labels | Prepared Waterfall/Heatmap/Bar/Estimate targets, their plot owners | Keep caller-required text separate from clipped data marks; account for full ink in the containing category, cell or annotation shelf. Waterfall's one shelf pack is finalized in T5e5 using these sources. |
| Panel text and inside guides | `src/decorations/panel_text.rs`, `src/decorations/legend.rs` | Fit exact selected text and guide geometry to their assigned panel; use the closed configured anchor, the parent-supplied obstacle order and an overlay-collision fit miss. |
| Supplemental point labels and guides | Prepared point/Bar layers and `src/orchestrator/layers.rs` | Preserve the recorded point-visibility gate; no coordinate clamp for a visible required label. Numeric guides are data-clipped marks; their labels are required text. |

The selected `SelectedAnnotationPlan` retains panel text and clipped numeric
guide geometry; significance remains in `PlotRenderResult.significance`.
The matching `SelectedMarkPlan` stores each family's immutable painter and
canonical ink. With all T5c3 inputs supplied, T5c4 removed the old deferred
boundaries and generic panel floor. The root driver in
`src/decorations/plan.rs` remains the only feedback search.
