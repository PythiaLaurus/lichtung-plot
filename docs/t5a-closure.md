# T5a Exact-Root Closure

**Status: COMPLETE (2026-09-14), after final review corrections and full `just verify`.**
Implementation baseline: clean `31fb8a2` (T5a6c4). This is current evidence for
spec 91 T5a6d and parent T5a6/T5a, not completion of T5b onward or a rewrite of
earlier reports. This docs-only closure reads existing final logs; it runs no
build/tests and makes no code, snapshot, staging, commit, or push changes.

## Current Boundary

- Exactly 20 native ordinary PlotKinds: eight `StandardCoordinate` and twelve
  `OneCategoricalAxis`. Categorical Trajectory is an additional input mode of
  Trajectory, not a twenty-first kind.
- Exactly five `SpecializedLegacy` interiors and two
  `ParentCompositionAdapter::{Facet, Canvas}` routes remain. Earlier spec-91
  summaries call these `SpecializedDeferred` and `ParentAdapter`; those are not
  additional source types. There is no current `ActiveSources` API.
- `DeferredMarkPlan` and `DeferredAnnotationPlan` retain the Mark/Annotation
  consequence boundary, including `DeferredLegacyPanelMinimum::SIZE = (80, 60)`.
  That named T5c parity floor is not a new scientific intrinsic or an unnamed
  ordinary floor. The five specialized packing policies remain T5e work.
- Public surface is unchanged: 87 crate-root re-exported names and two public
  modules (`kind`, `stats`) in `src/lib.rs`. This counts names at the root, not
  transitive methods/variants. No new public sizing, interval, alignment, wrapping,
  resource, or cache control is exposed. Plot defaults to exact 640x480 and Canvas
  to exact 960x640; automatic/width-only sizing remains the atomic T6 change.

## Removed Paths

`CanvasLayout`, `CanvasLayoutSpec`, `CanvasRootText::Unmeasured`, parallel root
baselines, `single_panel`, `resolve_canvas_layout`, and `base_canvas_layout` are
gone. `measured_single_panel` was already absent at the baseline. `RootFramePlan`
always contains a `RootTitleBlockPlan`; there is no unchecked root construction,
clamped residual box, estimated title height, or failed-measurement fallback.
The test-only builder helper now prepares every family and requires fit too.

`LegacyRootAxes` retains only the existing specialized/composition axis-inset
estimate. It cannot construct a panel, wrap a title, inspect root dimensions, or
substitute a theme/font. Removing estimated title/caption height from this inset
also prevents specialized axis feedback from spending an independent root-caption
reserve. Native ordinary frames never consume it.
`specialized_axis_feedback_cannot_spend_the_root_caption_reserve` pins the
correction with a multiline Beeswarm axis: adding a multiline root caption keeps
the extra axis gutter identical and the axis ink above the actual caption box.

The dead axis-ink `GeometryBackend` branch and `RetainedInteriorPayload`'s
`LegacyCombined` alternative are deleted. All selected bodies now pass their one
`PlotRenderResult` directly. `GeometryBackend`, its lifecycle, and raw-text
adapter are `cfg(test)` only: strict non-test compilation enforces the source
boundary, while test oracles still compare selected geometry with painted ink.
Production uses admitted `GeometrySnapshot` records, not backend discovery.

Source searches find no old root/combined/categorical-legacy/generic-floor symbol,
ordinary axis/title stabilization loop, or fixed five-tick ceiling. Remaining
Histogram bin-edge retries are pre-font statistical representability, not layout
retries; Matrix's eight-pass label packing and Heatmap/Waterfall packing/clamps
belong to the named specialized boundary. Numeric source coarsening, Unicode
line trials, and guide edge/flow trials remain declared local alternatives.

Publication selection occurs only at the owning standalone/root theme boundary;
Canvas children use the resolved inherited or explicit complete theme. Surfaces
are explicit retained RGBA primitives, not backend white/background fallbacks.
Fonts are censused after semantics, resolved from admitted immutable bytes, and
sealed before measurement. No production draw-time font discovery, fallback,
wrapping, axis selection, or root-size reread is introduced.

## Matrix Protocol

Every matrix row includes the following common gates, not just the final Null
counter. Paths below are relative to `src/` unless prefixed `tests/`.

| Gate | Source, Consumption, And Evidence |
| --- | --- |
| P: semantic/pre-font | `orchestrator/lifecycle.rs`, `orchestrator/facets.rs`, and `canvas.rs` validate configuration, admit topology before prepared row materialization, retain `plots/shared_x` family statistics/domains/categories/exact ticks/guides, materialize the `layout/text/requirements.rs` census, then prepare fonts. Invalid sources fail before discovery. Source records/bytes and actual processing share the operation without pricing copies as processing. Original facet rows use source/operation-bound topology tickets. |
| N: native intrinsic/assignment | `layout/interior/coordinate.rs::Intrinsics` is retained on prepared axes after fonts and before parent allocation for standalone/facet/Canvas roots. Hard title spans and measured mandatory-line category slots supplement the named Mark/Annotation floor. Hits admit calls but do not repeat analysis. One assigned `PanelBox`, semantic ranges, category counts/orientation, exact/interval policies, and role typography produce both frozen axes before mark projection. Native standalone `canvas_layout` reserves measured root text, not endpoint-width estimates. |
| S: specialized assignment | Prepared family inputs consume the assigned panel through the named specialized measurement function. Common numeric sources, resolved axis typography, `PanelAxisPlan` or retained Matrix axes, and the common outer-root title plan are active; specialized packing/guide allocation is explicitly not native. |
| F: feedback | `decorations/plan.rs` emits actual axis-gutter, owned-ink, and automatic-domain requests before root stability is checked. Native gutters replace the whole reserve and can shrink; domain/owned-ink consequences aggregate monotonically. Parent adapters coordinate shared domains/gutters and guide trials. Explicit limits do not grow. |
| K: cache identity | `decorations/plan/children.rs::Inputs` keys exact origin/extent, axis-label allocation, `StandardPanelInputs`, composed policies, complete resolved theme, facet scales, and relevant consumed feedback. Source/prepared fonts are fixed by the owning operation. Changed constraints cause fresh work; stable hits replay retained feedback with current-round request IDs. Standalone roots do not invent a separate panel cache. |
| C: capture/replay | Successful root: one selected-payload capture, one final Null validation, zero production measurement-backend constructions/begins, zero preparation/projection/axis/intrinsic/shaping work during capture. Selected marks, annotation placements, axes, clips, glyphs, styles, canonical primitive operands, and standalone legend calls are retained. Count-only canonical reservation is not a backend pass. Null and SVG replay the same `TextReplayPlan`; outlined SVG and PNG derive from the same frozen scene. |
| A: admission | Shared source/font/text visits and retained records, structural slots, calls/cache entries, discovery entries, and bytes keep their existing independent caps. Exact inclusive caps and one-short refusals are exercised before the refused work/storage. Failed trials remain charged; rollback discards effects, not costs. Final canonical storage refusal occurs before Null. See `docs/resource-admission.md`, not its historical test totals, for the resource definitions. |

`E0` below is `orchestrator/tests/lifecycle.rs::every_canonical_plot_kind_renders_through_one_lifecycle`:
all 25 branches, positive preparation counters, zero **all** capture-work counters,
one capture/Null, zero measurement-backend constructors/begins. After dropping the
source, it replays SVG, outlines it, generates exact source-size PNG, verifies
640x480, and repeats outputs without reads, preparation, projection, selection,
or shaping. `E1` is the shared `layout/interior/coordinate/tests.rs` suite: all
20 ordinary kinds plus categorical Trajectory and supplied Raincloud inputs,
hard intrinsic misses before projection, exact candidate/intrinsic caps, exact
tick failures/addresses, default versus explicit 640x480 byte equality, 400x300
and 801x601 roots, multiline root/axis separation, and zero layout backends.

## Closure Matrix

P/F/K/C/A apply to every row; N or S identifies its measured owner. The evidence
column adds family-specific semantics, admission, frozen-operand and fit tests.
There are exactly **27 route rows: 20 native + five specialized + two parents**.
The shared Mark/Annotation consequence boundary is recorded separately below;
it is not a twenty-eighth route. No row claims native T5c mark-density or T5e
specialized packing.

| Route | Prepared Semantic Source | Intrinsic/Measurement And Feedback | Frozen/Admission/Fit Evidence |
| --- | --- | --- | --- |
| Scatter | `plots/scatter.rs`: aligned points, source rows, hue order | N; numeric x/y; F | E0/E1; `plots/scatter.rs` tests: semantic/projected exact caps, poisoned capture, clipped-shape parity; `scatter_smoke_snapshot` |
| Line | `plots/line.rs`: ordered series, intervals, point layers | N; numeric x/y; F | E0/E1; `plots/line/tests.rs`: source order, caps, poisoned capture; `native_line` snapshot |
| Trajectory | `plots/line.rs`: group paths and numeric or categorical conditions | N; bottom category bands when categorical; F | E0/E1; line tests, categorical raw/display and midpoint tests; `native_trajectory`, `native_categorical_trajectory` snapshots |
| LinePath | `plots/line.rs`: source-order neutral/hue paths | N; numeric x/y; F | E0/E1; line source/capture/style tests; `native_line_path` snapshot |
| Histogram | `plots/histogram.rs`: retained bins/counts/KDE/rug | N; numeric x/y; F | E0/E1; `layout/interior/distributions.rs`: statistics, exact source/projected caps and zero capture work; `native_histogram` snapshot |
| Density | `plots/density.rs`: group KDE support and curves | N; numeric x/y; F | E0/E1; distributions tests, retained display limits/support; `native_density` snapshot |
| Qq | `plots/qq.rs`: quantiles and clipped reference | N; numeric x/y; F | E0/E1; distributions tests, finite/extreme reference geometry; `native_qq` snapshot |
| Regression | `plots/regression.rs`: fit and band curves | N; numeric x/y; F | E0/E1; distributions tests, statistics/interval retention; `native_regression` snapshot |
| PointInterval | `plots/interval_estimate.rs`: prepared rows/bounds | N; left categories, numeric bottom; F | E0/E1; `plots/interval_estimate/extraction.rs`, `layout/interior/estimates.rs`: exact admission, immutable tuples and capture; `native_point_interval` snapshot |
| Coefficient | Same owner: status/value/significance display | N; left categories, numeric bottom; F | E0/E1; estimate status, source labels and poisoned capture; `native_coefficient` snapshot |
| Forest | Same owner: raw ratios and nonzero interval intent, budgeted correctly rounded log domain, exact ticks transformed once | N; left categories, logarithmic bottom; F | E0/E1; sole-log-source/raw exact tick preparation, `numeric::tests`, source-intent collapse tests; `native_forest` snapshot |
| Dumbbell | `plots/dumbbell.rs`: endpoint pairs/order/styles | N; left categories, numeric bottom; F | E0/E1; dumbbell semantic/projected caps and no-read capture; `native_dumbbell` snapshot |
| Bar | `plots/bar_data.rs`: sample/estimate cells, intervals, overlays | N; bottom categories; F plus conditional annotation domain | E0/E1; `plots/bar_data/extraction.rs`, `layout/interior/bars.rs`: zero anchor, exact caps and capture; `native_bar` snapshot |
| StackedBar | Same owner: signed stacks and series order | N; bottom categories; F | E0/E1; stack extrema/midpoints and selected style tests; `native_stacked_bar` snapshot |
| HorizontalBar | Same owner: transposed values/overlays | N; left categories; F plus conditional annotation domain | E0/E1; horizontal raw/display, label suppression and exact-limit tests; `native_horizontal_bar` snapshot |
| StackedHorizontalBar | Same owner: transposed signed stacks | N; left categories; F | E0/E1; signed-stack/source-row/cap tests; `native_stacked_horizontal_bar` snapshot |
| Boxplot | `plots/categorical_distribution.rs`: summaries/outliers/overlays | N; bottom categories; F plus significance domain | E0/E1; `plots/categorical_distribution/extraction.rs`, `layout/interior/categorical_distributions.rs`: all overlays, raw slots, caps and capture; `native_boxplot` snapshot |
| Violin | Same owner: retained KDEs and summaries | N; bottom categories; F plus significance domain | E0/E1; distribution tests and same-assignment pre-native fingerprints; `native_violin` snapshot |
| Raincloud | Same owner: sample or supplied density/box/point geometry | N; bottom categories; F plus significance domain | E0/E1; supplied-input sharing, exact curves/overlays and capture; `native_raincloud` snapshot |
| Interaction | `plots/interaction.rs`: first-observed conditions, sparse trace groups/bounds | N; bottom categories; F | E0/E1; `plots/interaction/{native,extraction}.rs`: independent labels, sparse connectors, exact caps/operands and no-work capture; `native_interaction` snapshot |
| `SpecializedLegacy::Heatmap` | `plots/heatmap.rs`: semantic grid, missing cells, text, color scale | S; categorical axes, color guide, rotated/elided packing; F | E0; `plots/heatmap/extraction.rs`: exact source/cell/guide caps, raw keys, poisoned capture; `tests/heatmap`, composed and standalone fit tests |
| `SpecializedLegacy::Beeswarm` | `plots/beeswarm.rs`: sorted values, bins/offsets, categories/colors | S; numeric/category axes and continuous guide; F | E0; `plots/beeswarm/extraction.rs`: pre-font sources, candidate caps, bounds/baselines, all outputs; `beeswarm_clip_structure_snapshot` |
| `SpecializedLegacy::Waterfall` | `plots/waterfall.rs`: cumulative spans, labels, exact deltas | S; category/numeric axes and total-label packing; F | E0; `plots/waterfall/extraction.rs`: intermediate endpoint validation, exact caps, typed label fit, poisoned capture and output reuse; `tests/categorical/waterfall.rs` |
| `SpecializedLegacy::Marginal` | `plots/marginal.rs`: paired samples, hue groups and KDE curves | S; central/top/right assigned panels and frozen axes; F | E0; `plots/marginal/extraction.rs`: pre-font/candidate caps, selected geometry and capture; `marginal_clip_structure_snapshot` and full SVG fingerprints |
| `SpecializedLegacy::Matrix` | `plots/matrix.rs`: variable domains, pairs, densities and groups | S; cell spans and orthogonal tick reuse; canonical source-fit cells and frozen per-cell axes; packing remains T5e | E0; `plots::matrix::semantic_tests`: exact source/selected caps, no layout in capture, output reuse; `plots/matrix/axis_tests.rs`: independent calls/cache, odd-quantum ink and mark/tick/grid identity; root-title tests and expanded `matrix_clip_structure_snapshot` axis-text coverage |
| `ParentCompositionAdapter::Facet` | `orchestrator/facets`, `plots/shared_x/prepared.rs`: canonical cells, reconciled fixed/free domains, global categories/hues | Parent-assigned cells/shared gutters; native child N or specialized S; F/K; outer root uses `RootFramePlan` | `orchestrator/facets/parent_tests.rs`, native family facet tests: prepared-once, selected capture, constraints, cache-off bytes, root text/strip ownership; nested Matrix/Marginal rejected |
| `ParentCompositionAdapter::Canvas` | `canvas.rs`: declared sparse topology, inherited themes, shared axes/guides, text census | Assigned exact root; existing equal tracks/`PanelRequirements`, child N/S, F/K; mandatory root plan | `canvas/admission_tests.rs`: exact parent caps, mixed all-24-admitted-family roots, zero capture work/backends; Canvas tests: frozen heading/row/header plans, themes, 960x640 byte baseline; nested Matrix/facets rejected |

Shared consequence boundary: `DeferredMarkPlan`, `DeferredAnnotationPlan`,
`significance`, `orchestrator/layers`, and `decorations/panel_text` retain the named
80x60 floor, selected mark/annotation ink, and domain/owned-ink feedback. They have
no independent root. E0/E1 and family poisoned-capture tests cover source/projected/
annotation admission, selected caps/clips/glyphs, monotonic significance growth,
exact-limit refusal, and no post-freeze packing. Their final native requirements
remain T5c.

## Final Review Closure

All reported review findings are closed. The final source and full gate include:

- Independent cumulative measurement/intrinsic calls and root-cache insertions,
  not aliases for records or processing. Calls are admitted before lookup on hits
  and misses; owner tickets prevent duplicate wrapper charges. Direct text,
  numeric axes, Matrix ticks, child measurement, and later root rounds retain
  fatal refusal. `font::canonical::tests::generation_limits_are_independent_inclusive_and_shared`,
  the `resource_audit_*` tests, and `plots::matrix::axis_tests` pin the boundaries.
- Storage-only copies/reservations require records and checked bytes, not source
  processing prices. `source_levels` charges each input and only examined level
  entries; list searches stop at the match, and sorting admits its input once.
  `data::source::tests` and `builder::facets::interval_tests` test these units and
  source/operation-bound facet topology admission before materialization.
- Native intrinsics are retained before allocation in all root forms and are
  rebuilt for a new operation. `prepared_intrinsic_hits_are_admitted_without_analysis_or_storage`
  and `post_font_canvas_intrinsics_are_scoped_once_before_allocation_and_rebuilt_for_new_roots`
  distinguish analysis from admitted hits. The final
  `native_vertical_category_intrinsics_use_mandatory_lines_and_keep_fit_scope`
  regression uses actual multiline label height and preserves the caller's typed
  fit address/ordinal on refusal before projection.
- `layout/panel/projection.rs` is the common mark/tick/grid coordinate owner.
  Finite subtraction/division stays direct; scaled arithmetic handles overflowing
  differences or extrapolation numerators. Canonical origin/span and exact
  categorical counts preserve half-band ratios and nearest-even origin parity,
  rather than projecting category centers through an approximate numeric key.
  Numeric and categorical odd-origin tests, overflow extrapolation, and
  `matrix_marks_ticks_and_reference_grid_share_large_fractional_cell_coordinates`
  pin the actual drawn coordinates.
- Source selection and frozen axes use the same canonical label ink and anchors
  in `layout/panel/cells.rs`, including Matrix's retained final cell axes.
  `canonical_label_fit_accepts_touching_ink_and_rejects_one_quantum_beyond`,
  `automatic_selection_and_frozen_axis_agree_at_quantum_fit_boundary`, and Matrix's
  `selected_source_cells_keep_logical_and_ink_clearance_at_odd_quanta` cover both
  acceptance and rejection without silently dropping a tick.
- `src/numeric.rs` provides correctly rounded ties-to-even Log10 using outward
  exact-integer intervals, refining until both bounds round identically. The new
  direct pin `num-bigint = "=0.4.8"` reuses the already locked/cached package; the
  lockfile adds the root dependency edge, not a new package version. This is not
  a CORE-MATH port. Spec 05 now specifies this implementation with the same
  approved rounded-result contract. Existing root work/scratch limits guard it;
  refusal is fatal through Forest values/references/limits/ticks and nice ranges.
  Fixed MPFR vectors/digests, every binary exponent, near-one cancellation, and
  scratch/work refusals pass. This is not a fresh MPFR oracle-generation claim.
- Forest keeps one logarithmic source and raw exact-tick transforms only in
  preparation. The final
  `native_forest_selects_sole_log_source_and_transforms_raw_exact_ticks_only_in_preparation`
  regression checks complete source identity under changed assigned ranges and
  panels; raw nonzero interval intent remains independently validated below.
- Standalone legend capture replays selected calls and complete operands, even if
  measured style/position inputs are subsequently poisoned:
  `orchestrator::lifecycle::retained_legend_tests::standalone_capture_uses_selected_legend_operands_not_measured_style_or_position`.
  This closes frozen replay, not native T5b legend flow or composition tracks.

The independent resource definitions and exact limits are in
`docs/resource-admission.md`. Correct rounding, scratch accounting, and admission
calls have real overhead; changed accounting totals are not an optimization or a
general causal speedup claim.

## Feedback Evidence

- `native_absolute_gutter_feedback_contracts_independently_of_multiline_caption`
  seeds overlarge gutters for every ordinary route, observes actual replacement,
  and requires identical final geometry/text to the unseeded run.
- `native_root_annotation_growth_is_monotonic_and_freezes_the_final_domain`
  uses real distribution/significance producers with root title/caption plans.
  Every requested domain contains the consumed domain; the next round consumes
  that request, and final marks/axes/significance use exactly the same range.
- `native_child_reuse_and_rejected_ancestor_trials_preserve_selected_bytes_and_charge_work`
  covers every ordinary kind through three rounds. Changed assignment/domain
  trials are rejected, selected geometry/feedback is restored, visits/bytes stay
  charged, and later-round child feedback replay occurs only with caching enabled.
  A controlled named deferred sibling owned-ink consequence keeps the root active
  without changing the native child's allocation. This is not a T5b track policy.
- `native_real_compositions_remeasure_gutters_and_reject_guides_identically_without_cache`
  uses actual facets/Canvas and native axes, proves rejected containing guide
  frames and at-most-eight owner trials per round, and compares cache-off bytes.
  These fixtures change all affected allocations; cross-round reuse is proved by
  the separate retained-child test, not inferred from changed geometry.
- `native_root_producers_obey_eighth_round_and_cycle_limits_under_parent_feedback`
  measures real native numeric/categorical axes and root text under controlled
  deferred parent-gutter consequences. Round eight succeeds only when feedback
  stabilizes; continued growth and a cycle never capture. This supplements, not
  replaces, the kernel's exact arithmetic, provisional/fatal, cycle and cap tests.

## Ledger Reconciliation

No new visual property or policy value was introduced. The 1860 historical keys,
1575 active-overlay identities (1552 existing + 23 target-only), and 1883-key
materialized target in spec 04 are unchanged. New private carrier names do not
create new property owners. All current T5a portions in the source-to-plan map
below are complete; split keys retain their explicit later-phase obligations.

| Ledger Responsibility | Active Source Through Frozen Draw | Remaining Split Scope |
| --- | --- | --- |
| Foundation typography/fonts/content; `A-foundation-text`, T5a portions of `B-foundation-resolved-plan` and `B-policy-inputs` | `theme/resolved.rs` complete roles -> prepared semantic census -> admitted `font/canonical` -> `layout/text` intrinsic/constrained glyph plans -> title/axis/annotation owners -> canonical text replay | Legend/facet/header/annotation style unification remains with its T5b/T5c owner; T6 public values remain unexposed |
| Root surfaces and root title/subtitle/caption; `A-panel-title-axis-plan`, both `composition.root-*.horizontal-alignment` keys | Resolved owner/precedence -> `RootTitleBlockPlan` intrinsic width and assigned-width measurement -> `RootFramePlan` reserves/observation -> retained placements, baselines and explicit RGBA surfaces -> same Null/SVG plan | **Embedded Plot caption track allocation, exactly-once rendering alongside Canvas caption, and fit refusal remain a T5b obligation carried by the composition adapter. They are not complete here.** |
| Axes/ranges/ticks/frame/grid; `A-axis-fit`, `B-core-axis-geometry`, T5a portions of `B-domain-convergence`; `axes.ticks.preferred-data-interval` | `shared_x` semantics -> `coordinate::Intrinsics`, source policy and label cells -> `PanelAxisPlan`/Matrix axes -> root feedback -> retained grid/frame/text/primitive replay | Specialized axis packing stays named `SpecializedLegacy`; T5b shared-guide/track integration, T5e packing, T6 public interval API |
| Kernel normalized constraints/measured plan/parent arrangement, exact-size policy, spatial quantum | `layout/kernel.rs` exact promotion -> admitted producer calls -> assigned boxes/checked canonical conversions -> `FrozenRoot` dimensions -> replay/Scene | Automatic and exact-width/auto-height roots and public dimension accessors are not made current |
| Frozen-root identity, bounded alternatives, both feedback keys | Operation-fixed prepared semantics/fonts plus keyed child inputs -> scoped requests/trial rollback -> stable eighth-or-earlier root -> single capture | Native guide/composition/specialized/annotation completion stays T5b/T5c/T5e; deterministic track-allocation target remains deferred |
| Bounded-work, topology, checked growth, pre-backend timing; T5a portions of `A-root-resource`, `A-cross-structural-resource`, `B-backend-ink-plan`, `B-concrete-frozen-draw`, `B-lifecycle-output-plan`, `B-cross-source-residual` | `data/source`, prepared/font ledger, text/axis/geometry reservations -> cumulative root/child work -> canonical count/storage admission -> Null -> concrete scene/format-local derived output | Existing exact-root representability/format-local gates are preserved; no claim of complete future-mode T5e/T6 or T7 ledger acceptance |

Spec 05 remains the target source of truth. Its no-hidden-default automatic sizing,
native track allocation and final specialized/annotation requirements are not
silently claimed by these exact-size adapters. Historical source tables and
completed-stage records retain their original names, counts and verification.

## Verification

Final primary evidence is `/tmp/opencode/t5a6d-verify.log`, read through successful
package verification. This supersedes the earlier 966-pass implementation-only
checkpoint, not the historical evidence of preceding stages.

- `cargo fmt --all -- --check` and strict
  `cargo clippy --locked --offline --all-targets -- -D warnings` passed.
- `cargo test --locked --offline --all-targets`: **1,024 library tests passed,
  0 failed, 17 ignored** (1,041 total); **467 integration tests passed**.
- Integration counts: annotations 27, categorical 44, composition 57,
  distributions 67, estimates 38, heatmap 37, interactions 23, layout 20, lines 41,
  marginal/matrix 19, output 28, rendering 43, statistics 10, visual regression 13.
  Total: **1,491 passed, 0 failed, 17 ignored** (1,508 total). Four example targets
  ran zero tests.
- Warning-denied API documentation and locked/offline package verification passed:
  **270 files, 7.4 MiB uncompressed, 1.6 MiB compressed**.
- `/tmp/opencode/t5a6d-final-benchmarks.log` passes all **10 release measurement
  groups**, 0 failed, 0 ignored, 1,031 filtered out, in **8.74 seconds**. These cover
  significance, observer overhead, synthetic/system fonts, inside/outside/shared
  legends, publication, paragraph search, and specialized text. The earlier
  `t5a6d-final-benchmarks-release.log` and its tables remain a separate checkpoint.
- The normal gate's 17 ignored tests are ten opt-in release groups, six official
  Unicode 16 conformance tests, and one regeneration test. The latter seven still
  require explicit local UCD sources and were not run for this closure. Focused
  Unicode 16/version-boundary tests passed; no fresh official conformance rerun or
  Unicode 17 conformance claim is made.
- Public API/defaults are unchanged, but source, manifest, lockfile, and snapshots
  are not unchanged: the private numerical dependency and reviewed geometry fixes
  are part of this source closure. No network access, asset provisioning, Python
  file, `tools/` directory, or public extension point was added.

The final release observer comparison uses six pairs of 32 operations with test
hooks in both modes: inactive/active medians **6.853051/6.792400 ms**, paired median
**-0.071250 ms**. The negative difference is uncontrolled timing noise, not a
production-overhead or speedup result. Earlier measured tables and fingerprints
are left intact; no benchmark is rerun by this documentation update.

## Snapshot Evidence

All **27 snapshots** are accounted for by the thirteen passing visual tests and
review of the complete snapshot diff: **nine updated, eighteen unchanged**. No
snapshot was regenerated by this docs-only update. Under the common
`visual_regression__` prefix and `.snap` suffix, the inspected updates are:

| Snapshot | Intentional Change |
| --- | --- |
| `canvas_root_title_block_odd_size_snapshot` | One-quantum canonical y-axis title midpoint/rotation origin. |
| `root_title_block_explicit_odd_size_snapshot`, `scatter_smoke_snapshot` | Actual-ink numeric endpoint-label anchors. |
| `native_forest`, `native_histogram`, `native_point_interval` | Actual-ink numeric endpoint-label anchors, with unchanged tick sources/text. |
| `native_horizontal_bar` | Canonical category-center interval-cap endpoints and actual-ink numeric endpoint-label anchors. |
| `native_interaction` | One-quantum canonical x-axis title midpoint. |
| `matrix_clip_structure_snapshot` | Expanded structural oracle includes existing `X`, `Y`, `0`, and `5` axis text, not newly invented axis content. |

Separately, `docs/local-search-benchmarks.md`, **Final Numerical Follow-Up**,
records the parsed audit of **50 full parent SVGs: 28 byte-identical, 22
intentionally changed**. It found **99 one-quantum coordinate operands and 16
larger endpoint-label anchors** (0.83984375-1.6181640625 px), with unchanged
element/text/style/clip/panel structure. Those are parent-fingerprint counts, not
snapshot counts. The larger anchors follow canonical actual-ink placement and
must not be described as one-quantum changes. Historical fingerprint tables and
earlier measured output claims remain stage-specific provenance.

## Source-Intent Follow-Up

`Projection::validate_span` consumes the selected projected operands and retained
scientific nonzero intent. Cumulative addition, early coordinate quantization,
and correctly rounded scale transforms can erase distinctions before backend
validation; the helper restores fatal geometry rejection without reprojection,
requantization, epsilon widening, or a recoverable fit alternative.
Existing backend checks still govern wholly extrapolated spans.

- Line, Trajectory, and LinePath preserve the existing `canonical_line_points`
  global rule within each rendered polyline: any distinct source points that
  collapse must reject, including partial and nonadjacent collapse. Admitted
  scratch indices relate coincident projected points to the retained ordered
  source without reordering the path. Exact repeats and signed-zero equivalents
  remain legal where the source API permits them; collapse on only one axis is
  not a collapsed point when the other projected coordinate remains distinct.
- Estimate rows retain `nonzero_interval` from raw bounds before scale transforms.
  Forest bounds `1e16` and `1e16 + 2` both correctly round to logarithm `16.0`, but
  their nonzero interval still rejects under raw limits `1e15..1e17`. Exact equal
  bounds remain legal. Logarithm rounding and PointInterval's explicit clipping
  and boundary-touch behavior are unchanged.
- Bar/stack/Waterfall cumulative spans, Histogram width and positive height,
  ribbons, density support, sampled/supplied distribution bodies and boxes, and
  scientific interval/area geometry likewise preserve source nonzero intent.
  True zero values/areas, signed-zero equivalents, legal repeats, and explicit
  clipped boundary touches remain legal. No epsilon widening, reprojection, or
  recoverable fit retry substitutes for an unrepresentable scientific extent.

`plots/line/tests.rs` compares the source-aware rule with the canonical line
validator. `plots/interval_estimate/extraction.rs` pins unchanged logarithms,
retained raw intent, and fatal rejection before a selected payload is returned.
`tests/rendering/degeneracy.rs` contains twelve passing cross-family regressions,
including all three line families, zero-width bands, legal repeats, Forest log
intervals, signed/zero spans, polygon area, supplied Raincloud, and failure without
output artifacts.

Earlier source-intent follow-up validation passed: strict all-target Clippy,
34 focused unit tests, all 43 rendering tests, 41 line integration tests, and
38 estimate integration tests. That focused checkpoint did not run `just verify`,
commit, or regenerate snapshots. The subsequent final full gate above includes these regressions
and closes T5a6d/T5a6/T5a. T5b onward remain PENDING; their named obligations are
not T5a closure gaps.
