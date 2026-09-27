# T5c0 Mark And Annotation Disposition

This is the T5c0 source-to-owner decision for T5c1–T5c4. The reconciled keyed
rows in `specs/04_VISUAL_PROPERTY_OWNERSHIP.md` (T2e E-M1–E-M14, T2f
F-M1–F-M22 and their T2h relationships) remain the exhaustive inventory; the
tables here group those keys by implementation owner. T5b already retains mark
geometry and selected payloads. It does not yet supply every painted-ink hard
requirement or native annotation consequence. No public style or omission policy
is introduced by this document.

## Style, Content And Geometry Ownership

| Current source cluster / representative keys | Resolved owner and consumer |
| --- | --- |
| `marks.scatter.*`, `marks.line.*`, `marks.regression.observation-final-style`, grouped path and point-outline styles | Complete point/path/outline `MarkTheme` roles. Freeze actual mark style before ink measurement and reuse that exact style for a point/line guide glyph. Explicit local point/outline controls override only their named fields. T5d supplies opaque mapped data color, never the point chrome. |
| `marks.bar.vertical-*`, horizontal/stacked siblings, `marks.bar.*-error-*`, `marks.interval-estimate.*` | `MarkTheme` body fill, interval, uncertainty/cap, reference and supplemental-point roles. Distinct vertical body/group spans (`0.6`/`0.76`) and horizontal spans (`0.55`/`0.74`) remain separate geometry defaults; share proven style consumers only. `BarSpacing`, source interval/point geometry and grouped occupancy remain caller/renderer inputs. Grouped-bar significance consumes the validated body/slot span but owns its target and shelf separately. |
| `marks.histogram.*` sampled/precomputed, `marks.density.*`, `marks.qq.*`, `marks.regression.fit-*` and `.band-*` | `MarkTheme` bin/rug/KDE/point/reference/uncertainty roles; supplied bins and sampled bin statistics are distinct scientific sources. Band tessellation, KDE support, half-open bin assignment and clip ordering remain renderer invariants. Remove duplicate route-local style literals, not source routes or finite representability checks. |
| `marks.boxplot.*`, `marks.violin.*`, `marks.raincloud.sampled-*`/`.precomputed-*`, `marks.interaction.*` | `MarkTheme` body, summary/median, outlier, raw-overlay, mandatory-strip and interaction point/connector roles. Raw overlay and Raincloud strip are different semantic roles despite both using `PointStyle::new`. Sampled and supplied Raincloud consume the same resolved roles without conflating scientific inputs or activating the dormant second overlay. Exact `InteractionStyle` retains local precedence. |
| `marks.heatmap.*`, `marks.missing.heatmap-*`, `marks.beeswarm.*`, `marks.waterfall.*`, `marks.marginal.*`, `marks.matrix.*` | Specialized component mark roles for cells, point piles, cumulative spans, orientation-specific densities and Matrix cell glyphs. Missing fill/cross/merged row is neutral `MarkTheme` paint, never a data-scale endpoint. Keep Waterfall increase/decrease, connector/reference, Marginal orientation and Matrix cell semantics separate unless T2h proves exact parity. T5d owns mapping and gradients only. |
| Point-layer accent, regression-band slot zero, precomputed-Histogram slot zero, Waterfall sign mapping | Unkeyed primary/supplemental accents resolve from `MarkTheme`; keyed hue/continuous mappings and the final opaque color come from T5d `ColorScales`. `PointLayer::color`, per-item/categorical colors and explicit sign controls take precedence over theme defaults; missing neutral is not a qualitative palette slot. |
| `marks.point-layer.private-point-style-conversion`, bar point layers, supplemental guides and raw overlays | Exact caller fields (color, radius, independent fill/stroke alpha, outline, line pattern) validate once and freeze before candidate measurement. Layer presence, anchor, declared order, source geometry and data clipping remain caller/renderer policy. Retire the dormant generic line-width field rather than adding a fallback style route. |
| `annotations.point-layer-label-*`, panel text, significance, numeric/value text, null rows, Waterfall totals | `TypographyTheme` owns complete text/automatic-contrast appearance; `AnnotationTheme` owns non-text gaps, shelves, strokes, collision clearance and transparent-output planning backdrop. Caller content/number format/`Significance` target and placement remain explicit. T5c3 receives a typed resolved target; T5d supplies final scale identity through that same input. |
| Originating bar/distribution/path guide styles, grouped-bar significance spans and sampled/supplied Raincloud duplicates | Reuse final resolved mark styles in guides and final validated geometry in each corresponding consumer. Do not reconstruct a legend-only glyph, couple significance to the bar point's data color, or count a Raincloud strip twice. |

Resolved style precedes measurement: exact caller fields override their own role
fields, independent fill/stroke alpha replace rather than multiply, and one
resolved RGBA/glyph is shared by painted ink, guide identity, and frozen draw.
Current face/font and backdrop policy resolve before automatic light/dark text
selection. T5d may change the data color source, not this ordering. Geometry
orientation, anchors, rotation, clipping, statistical definitions, and z-order
remain renderer invariants (`specs/03_THEME_SYSTEM.md` § Component Boundaries).

## Prepared Primary Mark Count

For the *soft* standard-panel density preference in `specs/05_LAYOUT_SYSTEM.md`,
`m` counts source-backed selected semantic primary instances **before**
projection/window clipping. Coincident, transparent, scientific-zero and
exact-window-hidden marks still count. Raw samples used only to compute a
summary, generated tessellation/KDE vertices, optional raw overlays, reference
primitives, and a rendered glyph for the same instance do not multiply it.
Missing categorical slots remain occupied for pitch/annotation requirements
without becoming fabricated painted estimates. This is independent of the
structural-address census and of checked operation-work admission.

| Plot kind | One counted prepared primary instance |
| --- | --- |
| Scatter | One retained finite x/y source point. |
| Line, Trajectory, LinePath | One source-backed prepared vertex in its selected path/condition run; a shared endpoint or connector is not counted twice. |
| Histogram | One finite prepared bin, sampled or supplied; input samples, rug ticks and KDE vertices are not additional bins. |
| Density | One retained group curve, not its KDE grid vertices or contributing samples. |
| Qq | One ordered prepared quantile pair; generated reference lines are secondary. |
| Regression | One retained observation point; the fitted line and confidence polygon add ink, not primary observations. |
| PointInterval, Coefficient, Forest | One estimable categorical/term tuple with numeric point or interval; unavailable text-only slots affect pitch but add no estimate. |
| Dumbbell | One prepared endpoint pair with its connecting span; its two glyphs and connector complete one instance's ink. |
| Bar, HorizontalBar, StackedBar, StackedHorizontalBar | One occupied prepared bar/positioned-series/stack segment, including a zero; an absent series slot is not a segment. |
| Boxplot | One group summary; fliers and optional raw overlays complete ink without multiplying group summaries. |
| Violin | One group density body, not KDE samples, the inner box or raw-overlay observations. |
| Raincloud (sampled or supplied) | One group cloud/summary; mandatory strip points and supplied density vertices are separately measured ink. |
| Interaction | One populated factor×trace estimate slot; sparse holes are not marks, connectors/interval endpoints complete occupied geometry. |
| Heatmap | One active declared Cartesian cell, including a missing cell; a merged null-row polygon does not collapse source-cell cardinality. |
| Beeswarm | One retained source-backed categorical point, including coincident or transparent points; offset/bin work is not extra marks. |
| Waterfall | One finite cumulative delta step/span including zero; connector/reference/total labels are distinct roles. |
| Marginal | One retained central-panel source point; derived top/right density vertices and guide keys are not central marks. |
| Matrix | Per selected Matrix cell, one retained off-diagonal paired source point or one diagonal group density; unselected cells and KDE vertices do not contribute. |

## Data Windows, Elision And Omission

The approved case table is in `specs/92_T5B_T5E_EXECUTION_PLAN.md` § Approved
Data-Window Decision. These are its concrete consumers, including the narrow
current visibility policies. A newly required annotation is not made optional
merely by sharing its data point's clip rectangle.

| Branch / current key | Target disposition |
| --- | --- |
| Ordinary and specialized data marks under explicit `Plot::x_limits`/`y_limits` | `src/decorations/plan.rs` retains source identities/crossing endpoints; `src/render/geometry.rs` clips only painted data marks after finite/nonzero-intent checks. An exact-edge stroked marker may be clipped. Automatic axes request complete required ink; one exact axis does not disable the other's automatic clearance. |
| Point-layer text whose target point lies outside either data range | `annotations.point-layer-label-data-range-gate` remains the recorded point-visibility consequence. When the point is visible, shape and fit the complete text. Remove `annotations.point-layer-label-coordinate-clamp`; never move a visible required label to canvas origin or omit it to force fit. |
| Waterfall total candidate outside a data window | `annotations.waterfall.out-of-range-candidate-elision` affects renderer-generated invisible-point totals only; visible totals are measured outside the mark clip. It is not an exemption for caller-supplied required annotations. |
| Dense Heatmap category label | Exact caller `heatmap.labels.fit-policy` selects Wrap or Elide. Elide retains whole graphemes (`heatmap.labels.grapheme-safe-elision`); its `heatmap.labels.ellipsis-token` must itself fit or reject. No other text role inherits Elide. |
| Inactive Heatmap cell, merged missing row, unavailable estimate | `guides.heatmap.missing-entry-presence` scans active cells only. Missing cells/rows keep their semantic slots and neutral paint; unavailable estimates retain the occupied slot and required status text. Never map missing paint to a gradient endpoint. |
| Density after explicit-window clipping | `marks.density.visible-segment-threshold` skips only a non-drawable clipped segment with fewer than two distinct visible points; its prepared group and any required label remain. Malformed source/nonzero-collapsed geometry still rejects. |
| Marginal, distribution and Matrix density fallback/skip routes | `layout.marginal.visible-support-kde-replanning-fallbacks`, `marks.marginal.x-density-maximum-fallback`, `.y-density-maximum-fallback`, `marks.distribution.kde-constant-support-fallback`, and `layout.matrix.nonconvergence-failure` reject unrepresentable/unstable required curves under the approved bounded owner instead of silently substituting `1e-12`, skipping a group, or publishing a last trial. Preserve T5a's scientifically valid automatic support expansion. |
| Kind-specific mark clipping and layer order | The distinct `marks.*.clipped-*` entries for bars, boxplot, density, violin/raincloud, point/path, scatter, dumbbell, estimates, Qq, Heatmap, Beeswarm, Interaction, Marginal, Matrix and Waterfall retain their source/mark/annotation z-order. Supplemental guides use `marks.guide.panel-clipping`; plot-local overlay presence/visibility never leaks into an adjacent cell. |
| Hidden flier/overlay and empty significance comparison | `marks.boxplot.hidden-overlay-flier-role` retains its independent outlier style when caller hides/shows the raw overlay. `annotations.significance.empty-comparison-range-route` has no invented bracket when the caller supplies no comparison, but still validates any required ceiling and data geometry. |
| Automatic tick downsampling versus exact ticks | `axes.ticks.candidate-fit-and-downsampling` selects deterministic automatic candidates; it does not silently omit exact tick labels. `PanelAxisPlan`, root headings/captions and guide text measure complete required glyphs in assigned frames or return typed fit misses. |
| Significance/exact target, panel-text, value label and mandatory annotation | `src/significance.rs` consumes typed targets/anchors, owned reserves and automatic-domain requests. An incompatible exact domain rejects rather than clipping the required glyph/bracket. Panel-text width fit rejects instead of inheriting Heatmap elision. |
| Dormant sampled Raincloud overlay, Matrix label fallback, omitted bar interval endpoint | Remove the unreachable second Raincloud observation layer; T5e retires Matrix's fallback packing after T5c supplies native requirements; T5c2b includes the actual bar interval endpoint in automatic range. These are intentional corrections, not new caller omission controls. |

T5c1 resolves complete private styles before any candidate is measured;
T5c2a–d and T5c3 consume these semantic instances, exact data windows and
full painted extents on the existing bounded feedback driver. No alternate
draw route or new public API is admitted here.
