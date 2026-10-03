# T5e7/8 — All-Producer Ownership And Evidence Census

This is a documentation census of the current implementation and test assertions.
`specs/05_LAYOUT_SYSTEM.md` (private protocol, specialized geometry, topology,
feedback/cache and lifecycle) and `specs/92_T5B_T5E_EXECUTION_PLAN.md` (T5e6–8 and
explicit data windows) are authoritative. The T5e0 document is the historical
starting inventory with a current-owner handoff table.

Paths below are repository-relative; shorthand `plots/foo.rs` means
`src/plots/foo.rs`. Test names identify evidence to inspect, not a new test run.
The phase owner records the broad closure gate and quantitative release results
separately. This census does not mark T5e7/T5e8 complete or admit T6.

## Counts And Composition Admission

| Source census | Actual count and boundary |
| --- | --- |
| Public scientific kinds / private routes | 25: 8 `StandardCoordinate`, 12 `OneCategoricalAxis`, 5 `SpecializedGrid`, exhaustively classified in `src/layout/interior.rs`. |
| Selected mark representations | 16 `MarkPayload` variants: Scatter, Line, Histogram, Density, Qq, Regression, Estimate, Dumbbell, Bars, Distribution, Interaction, Heatmap, Matrix, Marginal, Beeswarm, Waterfall. Line covers 3 kinds, Estimate 3, Bars 4, Distribution 3: `16 + 2 + 2 + 3 + 2 = 25`. These shared owners are not missing families. |
| Prepared scientific owners | `PreparedInteriorInputs` retains the same 16 family payload slots plus the common scoped `PreparedAxes` vector. `PreparedReference` has 13 variants; Scatter, the 3-kind Line owner and Matrix use their own preparation paths. Counting that intermediate enum alone is not the 25-kind census. |
| Coordinate measurement dispatch | The 20 ordinary kinds use the common coordinate/axis boundary and 13 `measure_marks` dispatch arms. Bars split vertical/horizontal measurement and Distribution splits Boxplot/Violin measurement while retaining shared payloads. The 5 specialized kinds use their family `measure` functions. |
| Native composition parents | 2: owning Plot facets and Canvas. They allocate tracks and retain children under one root operation; they are not additional scientific kinds or nested root drivers. |
| Guide-source dispatch | `PreparedInteriorInputs::guide_entries` covers all 20 ordinary kinds plus Marginal; Histogram, Qq, PointInterval, Coefficient, Forest and Waterfall explicitly return no discrete entries. Heatmap/Beeswarm supply continuous descriptors; Matrix owns its local guide and has no parent-owned guide entry. Actual populated entries and visibility, not just the kind, determine contribution. |
| Canvas Plot contributors | 24 kinds: all 20 ordinary plus Heatmap, Marginal, Beeswarm, Waterfall. Matrix is rejected before fonts. TextPanel is an additional Canvas content type, not a 26th PlotKind. |
| Generic facet contributors | 23 kinds at the composition gate: all ordinary plus Heatmap, Beeswarm, Waterfall. Matrix and Marginal are self-managed and rejected. Individual input/channel/facet policies still apply. |
| Nested composition | A faceted Plot cannot be a Canvas child, regardless of kind. Canvas has no Canvas-child insertion type. Facet cells are frames under their owning Plot, not independent nested roots. |

The active exclusion owner is `CompositionChildPolicy::validate_child`, together
with `Canvas::validate_structure` and orchestrator configuration validation.
`exhaustive_route_census_has_twenty_ordinary_five_specialized_two_parents` in
`src/layout/interior/tests.rs` checks classification only. The lifecycle and
parent fixtures below provide execution evidence beyond that count.

### Actual Source/Topology Quantities

Prepared topology is distinct from selected paint calls, plan records and work
visits. Filtering a nullable row or clipping a point does not compact its source
identity. Scientific zeros retain their documented semantics.

| Producer / concrete assertion | Quantity retained or admitted |
| --- | --- |
| Ordinary preparation: `resource_audit_ordinary_topology_precedes_every_prepared_row_materialization` (`src/layout/interior/coordinate/tests.rs`) | Original row count for each of the 20 ordinary fixtures, plus a categorical Trajectory. A 3-row nullable Scatter admits 3 slots even though a row has no selected point. Repeated attachment shares prepared data and adds no source-row topology. |
| Heatmap: `heatmap_topology_prices_declared_cells_merged_missing_highlight_and_text_targets` (`plots/heatmap/extraction.rs`) | 28 slots in its fixture: 5 original rows + 6 declared grid cells + 2 columns + 3 rows + 4 ordinary cells + 1 merged row + 2 missing-cross strokes + 1 merged text + 1 highlight target + 3 value texts. A declared grid is not just its nonmissing source values. |
| Matrix: `declared_square_and_source_mark_topology_refuse_before_fonts` (`plots/matrix.rs`) | 20 slots: 4 declared cells + 12 off-diagonal source points + 4 diagonal density paths. Production first admits `n*n` cells and `n*(n-1)*original_rows` point slots, then diagonal density topology. The square includes diagonals; KDE vertices are admitted storage/work, not extra variable cells. |
| Marginal: `marginal_declared_subpanels_refuse_topology_before_fonts_or_curves` (`plots/marginal/extraction.rs`) | 13 slots: 3 linked subpanels + 6 original central marks + 2 density targets for each of 2 groups. The empty `(0,1)` Grid location is not a fourth painted subpanel. |
| Beeswarm: topology and facet-partition tests (`plots/beeswarm/extraction.rs`) | Standalone fixture: 12 original points + 2 categories = 14. Its two scoped facet partitions share original point topology once and add 2 + 1 category slots = 15. Coincident points remain separate source records. |
| Waterfall: `waterfall_topology_prices_categories_bars_connectors_and_named_totals_before_fonts` (`plots/waterfall/extraction.rs`) | 13 slots: 4 categories + 4 bars + 3 adjacent connectors + 2 named totals, including a zero delta. |
| Canvas / facets / tracks | Declared rows, columns, sparse and trailing cells and spans survive. Empty cells paint nothing; flexible declared tracks retain their one-quantum hard and composition-`1em` preferred minimum. Headers/spans occur once, not once per covered cell (`src/layout/kernel/tracks.rs`, `src/canvas/tracks.rs`, `src/orchestrator/facets/tracks.rs`). |

These are owner-local fixture counts, not a claimed total operation cost or a
conversion from source rows to layout trials.

## Common Prepared → Intrinsic → Measure → Freeze Contract

1. `src/plots/shared_x/prepared.rs` validates and prepares scientific inputs,
   domains, raw identities, labels and scoped axes with the root's generation
   resources. Parent shared domains and guide compatibility resolve before the
   font census. `src/layout/text/requirements.rs` collects actual display text,
   required annotations and generated alphabets; font selection retains same-byte
   provenance under `src/font/canonical.rs`.
2. Post-font `prepare_intrinsics`, family `intrinsic_body_minimum` and
   `src/layout/interior/preference.rs` supply traced hard requirements and soft
   standard/categorical preferences. `coordinate::Intrinsics` is scoped to
   prepared axes; Heatmap has its own keyed intrinsic cache. Intrinsic hits admit
   calls before lookup. Natural width is composed once, including the preferred
   Right configuration of automatic outside guides; it is not an exact-size trial.
3. `src/layout/interior/sizing.rs`, `src/canvas/sizing.rs` and
   `src/orchestrator/facets/sizing.rs` compose content at that width. Parents assign
   exact frames, columns before width-dependent rows. Coordinate measurement
   freezes `PanelAxisPlan` before family marks use its projection. Specialized
   producers select one track/packing/shelf plan under their assignment.
4. `src/decorations/plan.rs` retains a `PanelPlan` with selected marks,
   annotations, axes and guide. `src/orchestrator/lifecycle.rs`, facets and Canvas
   converge through `src/layout/kernel.rs::plan_exact_root`, capture one selected
   `TextReplayPlan`, reserve canonical output, and validate it through one Null
   lifecycle. Concrete output replays this same plan.

The driver's historical function name does not restrict it to exact requests:
`RootSize` accepts exact pair, exact width/content height, and fully automatic.
Initial automatic width is frozen after the first measurement; later height
consequences may replace or contract. Exact pairs cannot resize to cure a miss.
Public defaults remain Plot 640×480 and Canvas 960×640 until T6. An embedded
Plot's standalone size is not a cell preference.

## Scientific Producer Matrix: All 20 Ordinary + 5 Specialized

All ordinary rows share prepared `PreparedAxes`, coordinate intrinsics,
`PanelAxisPlan`, `SelectedMarkPlan`/geometry, and the enclosing annotation/frame
owner above. **S** below means standard density/`66ch`/soft 4:3 preference;
**Cx/Cy** means semantic category tracks on x/y with occupied hard pitch and
`2em` preferred pitch. Numeric domains/gutters are consumed from the root; only
automatic numeric axes can receive mark-ink domain expansion. Explicit windows
remain exact per dimension. Family paths name the actual source and selected
payload owner, not merely route tags.

| Kind | Prepared scientific source | Intrinsic → assigned measurement/arrangement → frozen data | Identity and owned consequences |
| --- | --- | --- | --- |
| Scatter | `plots/scatter.rs::PreparedScatterInputs` | S; complete stroked points → `measure_marks` → `ScatterPayload` | Retained row/group identity, selected point/layer styles; automatic x/y ink clearance. |
| Line | `plots/line.rs::PreparedLineInputs` | S; path/ribbon/marker ink → `measure_marks` → `LinePayload` | Ordered path observations, ribbon endpoints and supplemental layers survive; crossing geometry is projected before clipping. |
| Trajectory | Same Line owner, condition/group order retained | S, with categorical x bands when configured → Line measurement/payload | Raw condition and display labels remain distinct; category slots do not become numeric-domain feedback. |
| LinePath | Same Line owner, declared path/group semantics | S → Line measurement/payload | Source path order differs from condition ordering; final path and guide styles retained. |
| Histogram | `plots/histogram.rs::PreparedHistogramInputs` | S; prepared bins and overlays → `measure_marks` → `HistogramPayload` | Bin endpoints/counts and nonzero span intent retained; automatic bin/overlay ink consequences, no detached discrete entries. |
| Density | `plots/density.rs::PreparedDensityInputs` | S; prepared samples/KDE curves and complete fill/stroke → `measure_marks` → `DensityPayload` | Collective/raw color keys retained; root clearance reprojects retained curves without repeating source KDE preparation. |
| Qq | `plots/qq.rs::PreparedQqInputs` | S; quantiles/observation and reference ink → `measure_marks` → `QqPayload` | Scientific quantiles remain fixed through x/y clearance; no detached entries. |
| Regression | `plots/regression.rs::PreparedRegressionInputs` | S; retained observations/OLS fit and paint ink → `measure_marks` → `RegressionPayload` | Domain remeasurement does not recompute OLS; selected observation/fit/guide operands frozen. |
| PointInterval | `plots/interval_estimate.rs::PreparedEstimateInputs` | Cy; marker/interval/attached text → `measure_marks` → `EstimatePayload` | Typed category/row targets, bounds and text retained; automatic numeric headroom, no detached entries. |
| Coefficient | Same Estimate owner, coefficient input semantics | Cy → Estimate measurement/payload | Optional intervals/detail text preserve their input policies and row identities; exact ticks/windows do not coarsen. |
| Forest | Same Estimate owner, transformed logarithmic semantics | Cy → admitted logarithmic ticks and Estimate measurement/payload | Raw interval intent survives log transform; exact ticks/shared ranges transform once; arithmetic/resource errors are fatal. |
| Dumbbell | `plots/dumbbell.rs::PreparedDumbbellInputs` | Cy; both endpoints, connectors and labels → `measure_marks` → `DumbbellPayload` | Endpoint names/colors and grouped targets remain source-owned; complete ink expands only automatic numeric x. |
| Bar | `plots/bar_data.rs::PreparedBarInputs` | Cx; bodies/caps/points/value and attached labels → `plots/bar.rs::measure_marks` → `BarPayload` | Raw category/series or sign guide identity, uncertainty endpoints and source values retained. Numeric y clearance. |
| StackedBar | Same Bars owner, stacked signed endpoints | Cx → vertical Bar measurement/payload | Declared category/series slots and positive/negative cumulative geometry survive; zero/negative intent is not inferred from rounded width. |
| HorizontalBar | Same Bars owner, horizontal orientation | Cy → `plots/horizontal_bar.rs::measure_marks` → `BarPayload` | Numeric x clearance and exact axis-swapped labels/intervals; same raw series/color/annotation targets. |
| StackedHorizontalBar | Same Bars owner, horizontal stacks | Cy → horizontal Bar measurement/payload | Signed stacks retain category/series membership and endpoints; explicit x window clips data, not required text. |
| Boxplot | `plots/categorical_distribution.rs::PreparedDistributionInputs` | Cx; statistics/body/whisker/point ink → `plots/boxplot.rs::measure_marks` → `DistributionPayload` | Group/category targets and prepared statistics retained; automatic numeric y ink clearance. |
| Violin | Same Distribution owner, retained density samples/curves | Cx → `plots/violin.rs::measure_marks` → `DistributionPayload` | Retained curves, source points and final series glyph/color ownership; no draw-time density or style assembly. |
| Raincloud | Same Distribution owner, samples or supplied geometry | Cx → Violin/supplied `violin/precomputed.rs` selection → `DistributionPayload` | Supplied geometry is its own scientific input policy, not a missing fourth payload variant; source span/collapse checks remain. |
| Interaction | `plots/interaction.rs::PreparedInteractionInputs` | Cx; sparse trace markers, intervals/connectors → `measure_marks` / `interaction/native.rs` → `InteractionPayload` | Raw condition/trace/group keys, sparse membership, style patterns and source order retained; complete composite guide key uses those same paints. |
| Heatmap | `plots/heatmap.rs::{PreparedHeatmapInputs,HeatmapData}` | Complete cell ink, column-local text and cached summaries → `assigned_columns`, constrained annotations, `assigned_rows`, `select_cells` → `HeatmapPayload` | Full normalized grid/triangular policy, merged missing rows, highlights and value text; raw cell identities versus display aliases. Soft square/`2em` tracks; common retained continuous guide. |
| Matrix | `plots/matrix.rs::PreparedMatrixAxes` | Variable-label/point/density requirements and `12em` preferred cells → native Grid, variable axis cache → `MatrixPayload` | Every row-major variable cell, diagonal density and off-diagonal source mark retained; cell `PanelAxisPlan` source/semantics enter feedback even without painted axis ink. Root-only composition eligibility. |
| Marginal | `plots/marginal.rs::PreparedMarginal` | Central S preference plus density-stroke hard sizes → two-by-two Grid with quarter-central top/right preferences → 3 `LinkedPanelPlan`s in `MarginalPayload` | Central/top x and central/right y align exactly. Original grouped pairs/source IDs retained; domain-dependent render grids use retained samples. Automatic clearance per dimension; Canvas allowed, generic facets excluded. |
| Beeswarm | `plots/beeswarm.rs::PreparedBeeswarmInputs` | Complete conservative glyph radius → `beeswarm/packing.rs::pack` at assigned x width, occupied Cy tracks → `BeeswarmPayload` | Category then source-row order; zero then ordered/deduplicated tangencies, all prior pairs checked, no prior-point repacking. Raw numeric color identity and retained common continuous guide. |
| Waterfall | `plots/waterfall.rs::PreparedWaterfallInputs` | Original deltas/endpoints, complete bar/connector ink and total text → Cy tracks plus base/terminal `plan_annotations` greedy shelves → `WaterfallPayload` | Original row IDs and signed/nonzero intent; named base/terminal labels, exact baselines/reserve. Numeric automatic-domain requests and explicit out-of-window total visibility remain separate policies. |

`SpecializedLegacy`, `DeferredHeatmapMinimum`, `ParentCompositionAdapter`,
`legacy_root_axes`, `LegacyRootAxes` and `MINIMUM_MARK_PANEL_HEIGHT` have no
remaining definitions or uses in `src/`. Concrete replacements are the measured
columns/rows, linked subpanels, tangent packing and shelves above. Scientific
KDE/bin/bootstrap preparation, bounded tick-source ordering and the root's
eight-round driver are retained work, not legacy packing to delete.

## Text, Guides, Annotations And Parent Contributions

| Producer / prepared input | Intrinsic → assigned owner → frozen representation | Identity, feedback and admission |
| --- | --- | --- |
| Root and embedded Plot title/subtitle/caption | `src/decorations/title_block/plan.rs::{RootTitleBlockIntrinsics,RootTitleBlockPlan}`; legal-break prose widths and role metrics → local vertical root flow (embedded cells use `src/canvas/plot_text.rs::PlotLocalTextPlan`) → retained placed text | RootTitle/RootSubtitle/RootCaption addresses, independent resolved alignments and role typography. Plot-local captions occur once; Canvas caption is separate. Calls/text records/bytes share the root. |
| TextPanel title/body | `src/canvas/text_panel.rs::TextPanelIntrinsics` → one-column/two-row Grid, title/body measured after assigned width → `TextPanelPlan` | TextPanelContent/Title/Body addresses keyed by declared Canvas slot index; top-left after owner chrome, extra fill space stays chrome. Natural height is width-dependent. |
| Canvas headers and row titles | Prepared metadata/font requirements → `src/canvas/tracks.rs` spanned requirements and measured text → `CanvasHeaderPlan` records in `CanvasPlan.headers` / `CanvasPlan.row_titles` | Canvas-owned theme, span/declaration identity and canonical origin; headers/row titles occur once. Existing row-title versus present Plot title/subtitle conflict is validated. |
| Facet strips and shared axes/titles | `src/orchestrator/facets/{data,planning,tracks,sizing}.rs` → scoped Grid requirements and fixed/free-domain allocation → `FacetPlan` | Declared row-major cells, original source-row partitions and centered strip text. Fixed dimensions coordinate outer axes; free dimensions retain scoped axes. Measured gutter/alignment effects return to the root. |
| Numeric/categorical axes and grid | Prepared domains/exact ticks/categories/closed numeric alphabet → `src/layout/panel/{sources,axis_frame_grid}.rs` → `PanelAxisPlan` | Axis role, exact source kind/ordinal/cap, label bytes, projection keys, semantics and complete glyph/stroke geometry retained. Exact lists indivisible; category labels cannot stride/drop. Call admission precedes text/selection; misses keep original scope. |
| Discrete guides | Prepared actual `LegendEntry` set, raw typed identities, display title/aliases and complete glyphs → `src/decorations/legend.rs` intrinsic/assigned flow → `LegendDrawPlan` | Complete point/line/fill ink, frame stroke/radius, entry baselines and text. Plot/facet/Canvas frame owns bounded edge/flow attempts; explicit edge stays fixed. Calls include hits; rejected attempts retain work. |
| Continuous guides | Final-domain `ContinuousDescriptor`, unique numeric labels, stops, center/missing semantics → `src/decorations/legend/continuous.rs::MeasuredContinuousGuide` → common `LegendDrawPlan` | Actual scientific label fractions, endpoint ink, frame and missing-key outline all enter allocation and feedback. Constant singleton length is at least `2*max(Lc,Rc)+2*gap` and the traced minimum; it has no adjacent-pair substitute. Local Heatmap/Beeswarm and shared Canvas use the common selected draw representation. |
| Significance / attached and panel text / overlays | Prepared labels and typed targets → `src/significance.rs`, `src/decorations/panel_text.rs`, family attachment producers and `src/orchestrator/layers.rs` → `SelectedAnnotationPlan`, placed text/bracket/guide calls | Declaration/source-target identity survives packing order. Complete required text/bracket ink must fit its assigned box. Named owned-ink/domain requests require a resolving predicate; unrelated collision is terminal. Pair/shelf/lookup work admits actual examined entries. |
| Canvas Grid / range groups / empty cells | Prepared coordinates, spans, scoped scientific domains and composition roles → `src/canvas/{tracks,sizing}.rs`, `src/layout/kernel/tracks.rs` → `CanvasPlan` | CanvasGrid/CanvasCell/Header/RowTitle/RangeGroup identities; original sparse indices survive. Range groups are nonvisual, not anonymous measured children. Shared unions install before height and final child measurements. |

Matrix cells, Heatmap cells, generic marks and guide inline records do not each
manufacture a measurement call or nested solver. Marginal's three linked
subpanels are explicitly measured children. Structural role/slot paths in
`src/layout/kernel.rs::ComponentAddress` are distinct from original row IDs and
raw hue/category/collective keys. An alias changes painted text, not a data-color
key or significance target. Address storage is admitted and shared; pointers
alone do not establish reusable scientific/layout identity.

## Feedback, Input Keys And Replay

`FeedbackRound` transfers axis gutters, shared alignment, cumulative owned ink,
automatic x/y domains, and replaceable `ContentHeight` consequences. A request
names source, target, kind and the miss it can resolve. Stable effective feedback
with a remaining provisional miss fails; an empty request list is insufficient.
Fatal arithmetic/font/resource/reservation errors never select another fit trial.
Rejected local trials roll back effects, not cumulative work. Automatic guide
owners share their per-round eight-trial cap across ancestor-driven calls.

`src/decorations/plan/children.rs::ChildPlans` belongs to one immutable Canvas or
faceted source. Its key compares owners, exact assigned box/origin, axis-label
width, content-height and row-fill flags, standard appearance, retained composed
Plot policies, full resolved theme, facet scales and relevant consumed feedback
including descendants. Panel keys also compare guide role and actual entry input.
Prepared source rows/configuration/fonts are fixed by that borrowed operation.
Hits admit a call before lookup and visits for examined entries, share the
selected Arc, and replay retained observations/current-round requests. Changed
dependencies remeasure; provisional or unstable entries are discarded.

Canvas `HeightMeasurements` in `src/canvas/sizing.rs` has a separate consequence
key under the same ledger: normalized width, current numeric ranges/provenance,
resolved shared x, parent and local gutter/owned-ink insets, scope, tick visibility,
legend policy and continuous-guide suppression. Source/text/theme/guide semantics
are immutable under that prepared owner. Equal width alone cannot reuse height.
`ContentHeight` replaces width and height, including contraction; it is not a
cumulative ink reserve or the physical height donated by a taller sibling.

Final capture and `TextReplayPlan::replay` consume retained geometry/styles/text.
They do not prepare sources, project, select ticks/edges, pack, wrap/shape or
assemble new styles. Later outline conversion may extract paths from retained
same-byte faces and frozen glyph origins. It cannot reread fonts or alter layout.

## Operation Admission And Independent Refusal Evidence

`OperationFontResources` and `OperationGenerationResources` in
`src/font/canonical.rs` share one operation ledger, including preparation before
fonts, later rounds, child keys, guide trials and feedback. Limits are inclusive:
1,000,000 each for records, visits, calls, cache entries, topology and font
discovery; cumulative font/source/scratch bytes are 536,870,912. These classes
are independent; source rows or draw-call counts cannot stand in for trial work.

| Class / boundary | Meaningful implementation and test evidence |
| --- | --- |
| Retained records | Headers/elements, plan/key/feedback records and actual glyph/track/point payloads before retention. `generation_limits_are_independent_inclusive_and_shared` (`src/font/canonical.rs`); `later_panel_cache_replay_record_refusal_is_fatal_without_remeasurement` (`src/decorations/plan/children.rs`) exhausts records in round two at replay, with no remeasurement/finalization. |
| Processing visits | Known source/text passes admitted before work; actual candidate, collision, shelf and terminating lookup comparisons admitted before each guarded step. Coordinate exact-cap tests, Beeswarm `candidate_visits_bytes_and_dense_refusal_use_the_operation_ledger`, panel-text `collision_scans_admit_actual_pairs_and_stop_before_unused_tails`, Matrix `diagonal_projection_admits_every_examined_group_and_vertex`, and feedback scan/history tests cover real paths. Copying/storage is not a second processing charge. |
| Calls including hits | Root round, intrinsic, child, numeric axis, axis frame/grid and guide entry boundaries. `prepared_intrinsic_hits_are_admitted_without_analysis_or_storage`; `child_measurement_hits_and_insertions_use_independent_cumulative_limits`; Matrix `cache_hit_admits_the_last_call_even_with_no_cache_entries_left`; numeric-axis/axis-frame call-refusal tests. A shared call ticket avoids repricing a cache miss as another owner call. |
| Cache entries | Before actual insertion, independently of call/record/byte cost. The child hit test retains a real Arc hit after cache capacity is exhausted; changed input then refuses insertion. `font_cache_refusal_does_not_insert_or_prevent_existing_entry_reuse`; Matrix `cache_miss_reuses_its_call_ticket_and_admits_insertion_independently` (`plots/matrix/axis_tests.rs`). |
| Topology | Before original scientific materialization or declared Grid arrays. Exact owner counts above; `preparation_retains_shared_font_and_generation_resources_and_topology_limit` (`src/layout/kernel/tests.rs`) and sparse/span track tests cover the parent boundary. |
| Bytes / fallible growth | Source vectors/strings, address paths, full theme-key stacks, feedback buffers, glyphs and capture storage admit checked bytes before allocation. `complete_theme_clone_admits_exact_owned_bytes_and_records_before_copy`, Heatmap/Waterfall/Beeswarm selected-storage tests, Marginal linked-path test and `complete_parent_admission_has_exact_work_and_storage_boundaries` cover concrete retained payloads. |
| Feedback growth / rejected work | `src/layout/kernel/feedback.rs::reserve` geometrically grows unknown streams and prices the complete new buffer before allocation; actual payload/reference records remain separate. `large_feedback_stream_prices_geometric_buffers_without_quadratic_prefix_storage` and `feedback_growth_refuses_before_buffer_or_reference_mutation_at_the_exact_byte_boundary` test that correction. `parent_topology_admits_exact_budget_and_does_not_refund_rejected_trials` and guide-budget tests preserve cumulative rejected work. |
| Later rounds / failure precedence | `later_root_round_call_refusal_is_fatal_before_measurement_or_finalization`, `root_state_admission_refusals_are_fatal_before_freeze_or_another_round`, reservation-stage tests and `later_fatal_measurement_wins_over_an_earlier_typed_fit_miss` (`src/layout/kernel/tests.rs`). |

These are independent shared-boundary and concrete-producer tests, not a claim
that every resource × family × sizing-mode combination has a separate fixture.

## Evidence Map And Its Coverage Boundaries

| Contract | Actual tests and what they assert |
| --- | --- |
| All 25 kinds, all 3 private requests | `every_canonical_plot_kind_renders_through_one_lifecycle` (`src/orchestrator/tests/lifecycle.rs`) constructs each actual public family. Automatic and 800px width-only portions require finite successful plans and exact requested width. Its exact 640×480 portion asserts one selected capture, one Null validation, zero capture counters, retained preparation, source-drop replay, preserve/outline stability and actual 640×480 PNG. Full per-family SVG/PNG replay assertions belong to that exact portion. |
| All valid Canvas kinds | The same fixture calls `mixed_family_parent_audit` (`src/canvas/admission_tests.rs`): 24 supported families in six four-child exact roots, Matrix pre-font rejection, zero capture work, identical cached/uncached SVG. It deliberately stays within the unchanged operation cap; it does not claim one 24-child root or every family in every content-sized parent. |
| Native ordinary execution beyond route tags | `ordinary_test_sources_cover_every_native_route_exactly_once`, `every_native_route_uses_exact_assigned_projection_and_frozen_axes_without_legacy_constructors`, `every_native_route_reports_intrinsic_fit_before_mark_projection_or_backend_creation`, `every_native_route_admits_intrinsics_and_selected_candidates_at_exact_resource_caps` (`src/layout/interior/coordinate/tests.rs`). |
| Content-sized Plot/facet roots | `src/orchestrator/lifecycle.rs::natural_sizing_tests`: 300px caption reflow versus short caption; Right→Top with unchanged width; sparse wrapped facets and complete multiline caption; Matrix hue guide/full square; free Waterfall shelf domains; scoped Beeswarm and nullable Heatmap guides/missing blocks; explicit owning facet-guide edges. |
| Content-sized Canvas and replacement height | `src/canvas.rs::natural_sizing_tests`: sparse/spanned mixed roots, standalone-size independence, shared/PerPlot guides, 300px recovery, taller prose fill, late width/domain contraction, coordinated Beeswarm/Waterfall, Hidden continuous guides in all 3 requests, cache-off identical replay. `src/canvas/sizing.rs` tests same-width gutter contraction and width/domain/gutter/visibility invalidation. |
| Eighth round, cycles and exact refusal | `parent_feedback_reallocates_real_geometry_and_converges_on_round_eight`, `content_roots_choose_width_once_and_freeze_eighth_round_content_height`, `feedback_cycles_exhaustion_and_unchanged_misses_never_finalize`, `both_content_modes_reject_feedback_cycles_before_finalization`, `automatic_width_reselection_and_invalid_requests_refuse_before_capture`, `exact_root_rejects_mismatched_measurement_and_nonpositive_size_before_finalization` (`src/layout/kernel/tests.rs`). Ordinary native root tests also exercise real producer eighth-round/cycle paths. |
| Real cache hits and complete output equality | `dirty_canvas_child_reuses_the_unchanged_neighbor_with_exact_output` (`src/canvas.rs`); `shared_legend_child_cache_replay_contract`, `outside_recovery_controls_replay_identical_without_cache`, `measured_gutter_reflow_preserves_complete_frames_without_cache` (`tests/benchmarks/legends.rs`). Evidence includes selected edge/flow/round traces, preserve/outline SVG, encoded PNG and decoded pixels; multiple rounds alone are not proof of a hit. |
| First capture and immutable replay | `specialized_producer_plans_survive_first_capture_without_shaping` (`src/decorations/plan/tests.rs`), `facet_producer_plans_survive_first_capture_without_shaping` (`src/orchestrator/facets.rs`), `canvas_producer_plans_survive_first_capture_without_shaping` (`src/canvas.rs`), per-family poisoned-capture tests, and `frozen_axes_and_grid_replay_without_layout_theme_or_shaping` (`src/layout/panel/tests.rs`). The all-25 fixture supplies the final source-drop/counter checks. |
| Required annotation / explicit-window distinction | `complete_bracket_stroke_cannot_escape_the_category_panel_even_with_exact_limits`, `thick_bracket_stroke_expands_only_an_automatic_numeric_window` (`src/significance/tests.rs`); Heatmap `required_heatmap_cell_text_must_fit_its_unclipped_cell`; Waterfall `clipped_base_keeps_the_visible_terminal_containment_failure_named`; `unclamped_projection_leaves_out_of_range_marks_for_backend_clipping` (`src/orchestrator/tests/lifecycle.rs`). Waterfall's explicit out-of-range total omission is its existing policy (`explicit_limits_omit_out_of_range_waterfall_annotations_from_svg`), not general permission to drop annotations. |
| Terminal-local PNG | `content_root_capture_is_immutable_and_png_refusal_remains_terminal_local` (`src/orchestrator/lifecycle.rs`) covers automatic and width-only plans, preserved/outlined SVG after excessive-DPI refusal and unchanged preparation/shaping/selection. `chained_terminals_reuse_one_canonical_scene_and_plan_each_png` (`src/orchestrator/tests/lifecycle.rs`) covers exact Plot/Canvas scene reuse and each PNG's dimensions/density. `src/scene.rs` tests inclusive axis/pixel/parser limits, actual outlined-glyph payload before buffer reservation, and preserved/outlined SVG surviving PNG refusal. Shared layout does not price hypothetical PNG viability. |
| Raw/source/style identity | `source_rows_drive_standalone_facet_and_canvas_diagnostics` (`src/orchestrator/tests/validation.rs`); mapped-row tests in Marginal/Matrix and `original_source_ids_survive_preparation_selection_and_admitted_feedback` in Waterfall; `displayed_series_alias_never_becomes_a_significance_target_or_color_key` in Bars; categorical Trajectory raw/display tests; `actual_child_hit_is_invalidated_by_legend_only_paint_and_geometry_changes` and full theme-key admission in child cache. `custom_connector_appearance_keeps_exact_group_patterns_and_guide_identity` in Interaction and custom gradient/constant-midpoint tests in Canvas/Beeswarm verify real style/scale inputs. |
| Guide ink and singleton correctness | `src/decorations/legend/continuous.rs`: actual-fraction/endpoint, large typography, broad frame/missing strokes and singleton tests. `constant_domain_large_typography_reserves_singleton_length_in_both_flows` accepts the compact minimum and rejects one quantum below it in both orientations. Discrete complete key/frame ink and small-role fixtures live in `src/decorations/legend.rs`. See `docs/t5e7-guide-cache-closure.md` for inspected output attribution. |
| Scientific SVG geometry and fingerprints | `tests/categorical/{beeswarm,waterfall}.rs`, `tests/marginal_matrix/parity.rs`, `tests/interactions/{styles,connectors}.rs`, Heatmap integration and native/clip snapshots in `tests/visual_regression`. Marginal tests independently check source points/KDEs/shared spans; Interaction checks projection/style/source order, not deletion-normalized strings. Historical hashes are attributed in the root, guide-cache and Marginal audit documents, not asserted as current parity. |

Release workload sources remain
`tests/benchmarks/{suite,text,annotations,legends,fonts}.rs`.
They retain publication cases, long captions/legal breaks,
specialized text fit/refusal, guide recovery/cache traces, significance and font
work. `matrix_publication_feedback_storage_stays_within_the_operation_ledger`
in `tests/benchmarks/suite.rs` covers the real Matrix publication feedback stream.
Release timings, resource totals, output hashes and final gate status belong to
the phase owner's evidence; this document supplies no inferred speedup or fresh
verification result.

## Census Outcome

Every scientific kind and admitted parent/text/guide/annotation contribution has
an identifiable prepared, intrinsic, assigned and frozen owner in the inspected
source. The exclusions reconcile to 24 Canvas kinds and 23 generic-facet kinds;
16 shared mark payloads reconcile to 25 scientific kinds. No new unowned producer
or unsupported composition route was identified by this documentation pass.
Broad gate completion remains separately owned. No tests were run for these
documentation-only changes.
