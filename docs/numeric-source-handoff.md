# Numeric Presentation Source Handoff

Source review: `387b07a`, after complete public foundations, preferred intervals
and public automatic/width/exact root sizing. This refresh implements no numeric
policy or conversion. The approved product, consumer/set and mathematical
contracts remain `specs/06_NUMERIC_PRESENTATION.md`,
`specs/07_LOG10_DISPLAY.md` and spec94 §12; their public cutover and executable
certification remain pending.

## Baseline Changes

The original numeric design census at `e6febe3` is historical. In the inspected
source:

- `Theme` is the complete immutable public value. Plot retains `Option<Arc<Theme>>`
  and Canvas retains its complete root Theme. Typography has eighteen independent
  `TextTheme` roles with typed ordered `FontStack`, paint, size, weight and leading.
  Neither policy nor operation font state belongs in Theme identity.
- Heatmap and Beeswarm both retain `ContinuousColorScale`, including arbitrary
  semantic stops, exact/automatic domain and optional diverging center. Built-in
  sequential defaults use `#F7FBFF` to `#08306B`. No old Heatmap-only scale or
  family-selected gradient is a numeric migration input.
- `TickInterval` is a validated opaque public value. Axis source generation widens
  decade steps internally; exact ticks suppress the generated path. Conditional
  categorical Trajectory x is the actual pre-font rejection fixture.
- Plot, Canvas and `ComposedInputs` retain private `RootSize`; absence is natural,
  `width` replaces the whole request, and `size` fixes both dimensions. Only
  assigned geometry has `canvas_dimensions()`. Output owns frozen CSS dimensions.
  Numeric intrinsics participate before the one natural width is selected.
- `src/builder/admission.rs`, `Plot::admitted_clone` and cumulative resource
  accounting no longer exist. `Plot` derives Clone; `erase` still exhaustively
  destructures/reconstructs every field. `data::source` keeps real checked lengths,
  fallible storage and original-row/operation identity, not pricing.

Spec95 supersedes all historical fee, comparison-charge, scratch-quota and
charge-delta instructions in the numeric design. Keep exact arithmetic, useful
mathematical temporary bounds, actual representability and fallible first-party
buffers. Do not recreate accounting parameters or a fallible comparison protocol
whose only former failure was a removed quota. The existing integer dependency
may allocate internally; no claim of catching every third-party allocator OOM is
made.

## Current Conversion Owners And Deletions

| Actual owner | Current behavior | Replacement responsibility |
| --- | --- | --- |
| `src/spec.rs::NumericLabelFormat`, `NumericLabelDisplay` | precision `usize` 0..=12, literal nonempty/control-free suffix, fixed-two default; Rust fixed conversion preserves actual/rounded negative zero | One normalized seven-field policy and exact ordinary conversion; retire the current type, display and suffix spelling at the public cutover |
| `src/layout/panel/sources.rs::format_label_parts` | Ordered Automatic, integral Forest literal and nonintegral `10^` expression branches in one bounded stack buffer | Built-in profiles of the one numeric engine; no retained old arithmetic behind a fallback |
| `format_label`, `format_guide_label` | Distinct fallible string-retention adapters over those same conversion branches | Adapters may retain source/error responsibility, never separate rounding |
| `NumericTickPlan::measure_selected_source` | Complete source precision search `0,2..=17`, adjacent-label uniqueness then full geometric fit | Retain source provenance, operand, effective policy and common resolved precision; do not assume default monotonicity proves arbitrary configured distinctness |
| `descriptor::{domain_labels,numeric_labels}` | One to three finite guide members; signed-zero scientific equivalence; whole required set retries `0,2..=17` | Compatible local/final Shared policy, canonical scientific members and resolved precision before fonts |
| `plots/heatmap.rs::prepare` | `format_source` for every Some cell when Values is present; prepared labels/owners distinct from guide labels | Complete active finite per-panel value set and the same prepared strings for census, intrinsic and paint |
| `plots/interval_estimate.rs::prepare_model` | Adjacent numeric text formats raw `tuple.estimate`; significance is a separate `(String,bool)` path | Available point-only Coefficient set, raw operand and uniform resolved precision; preserve interval rejection and unavailable/significance semantics |
| `layout/text/requirements.rs::specialized_text_census` | Coefficient census independently calls `format.display` on raw column values | Consume the same prepared available numeric occurrences, not a second conversion/default or clipped point subset |
| `plots/waterfall.rs::prepare` | Retains `(base,String)` and `(ordered terminal,String)`; `"{name} {value:.3}"` | Retain names/raw candidates, eligible-set masks and common precision; replace numeric conversion while keeping literal separator and ordered scientific additions |
| `src/numeric.rs` | Certified nearest-even binary64 `log10`, outward integer intervals and rational-to-binary64 conversion | Keep scientific logarithm ownership; it is not an implemented decimal formatter or inverse-power display certificate |

Automatic still normalizes actual signed zero before choosing a branch. With no
scientific retry: abs>=1e12 or 0<abs<0.01 uses scientific fractional 2; abs>=1e6
uses the **rounded binary64** `value / 1_000_000.0` at fixed 1 plus `M`; abs>=10000
uses `value / 1000.0` at fixed 1 plus `k`; abs>=100 uses fixed 0 without integral
trimming; abs>=1 uses fixed 1, otherwise fixed 2. Only the fixed fractional
branches trim trailing zeroes/point. Retry precision is scientific fractional
2..=17, not the public Significant maximum. Thresholds use the original value;
`999999` may therefore remain `1000k`. Explicit decimal scale instead shifts the
exact rational, without an intermediate binary64 multiply/divide.

Integral Forest seeds/subsets/intervals use the unchanged literal table for
exponents -2..=11 (`0.01`, `0.1`, `1`, `10`, `100`, `1000`, `10k`, `100k`, `1M`,
`10M`, `100M`, `1000M`, `10000M`, `100000M`); other admitted integral seeds use
`1.00e{exponent}`. Other selected Log10 sources use `10^` plus Automatic-formatted
exponent. Configured Forest will instead round mathematical `10^(exact(e)+scale)`.
The original-domain midpoint is not restricted to the integral seed clamp.

`range_ticks::{format_tick,distinct_tick_labels}` are test-only adapters, not an
independent conversion oracle. Tick-step `decimal_power`, scientific `log10`,
color interpolation, diagnostics and SVG coordinate serialization remain separate
owners.

## Transport And Identity Checklist

| Actual field/copy/key | Required migration input |
| --- | --- |
| `builder.rs::Plot::{x_axis_spec,y_axis_spec,heatmap_annotations,estimate_labels,continuous_color_scale,color_label,waterfall_labels}` | Axis optional policy; separate guide-number and total-number optional policies; replace existing value payloads, independently of scale/title/names |
| `Plot::new`, derived Clone and exhaustive `Plot::erase` | Initialize/move/retain every new field and affix without granting erased Any a capability |
| `builder/facets.rs::{retain_composed_inputs,same_composed_inputs}` | Both manually copied AxisSpecs and full normalized presentation/operand context; compare content, not equal endpoints/width/text alone |
| `with_resolved_facet_ranges`, planned-domain edits and `range_ticks::display_axis_spec` | Replace only range/provenance. The last adapter clones the whole spec; Forest transformed `resolved_facet_x_range` is not decimal scale |
| `shared_x/prepared.rs::PreparedAxes` and explicit `prepare` AxisSpec literals | Ranges/categories/display categories/labels/specs/logarithmic plus effective policy, operand and exact/category suppression per scope; projected exact ticks change positions only |
| `spec.rs::PlotRenderContext::layout_with_ticks` | Its second manual AxisSpec reconstruction also transports policy/operand through category/prepared-limit substitution |
| `coordinate::InteriorRoute::measure`, `AxisSpec::layout_ticks`, `interval_estimate::log_tick_plan` | Ordinary and special no-exact Log10 branches receive the same effective policy; exact/supplied sources remain explicit |
| `coordinate::Intrinsics`, PreparedAxes OnceLocks | Terminal midpoint uses that prepared base range/policy/operand; adaptive singleton minimum is separate from final candidate precision; verify identity with complete typography |
| `layout/interior/sizing.rs::horizontal_gutter_preference` | Actual y categories, then exact labels, otherwise low/mid/high probes; hidden labels do not measure. Explicit adaptive policies use the bounded precision envelope, not artificial-set uniqueness |
| `canvas/tracks.rs` and standalone `plot_size` | Both use that same horizontal estimate before natural width selection; keep Matrix's native trailing-edge owner and zero external trailing preference |
| `HeatmapIntrinsicCache` | Current key has complete scale, contrast roles, axis roles, surfaces/backdrop and cell geometry; add value-policy/set identity or prove immutable prepared identity explicitly |
| `AxisSemanticInputs`, `PanelAxisPlan` and feedback observation | Current Copy record has provenance/AxisScale/range policy/interval only; retain owned/shared effective policy, operand and generated suppression, source/set/precision alongside result text |
| facet shared-axis signatures | Raw categories/order remain scientific; a shared displayed numeric axis compares compatible effective policy/operand, not formatted category keys |
| `decorations/plan/children.rs::Inputs` | Existing owners/geometry/label width/content-height/fill flags/ComposedInputs/theme/scales/consumed feedback remain; full new policy must survive coordinated copies and equality |
| `canvas/sizing.rs::HeightInputs` | Existing width/ranges/provenance/resolved_x/insets/y-label visibility/legend/suppression/scope remain; include applicable policy/regenerated set or enforce immutable prepared ownership explicitly |
| `ContinuousSemantics`, `ContinuousContributor`, `ContinuousDescriptor` | Add normalized effective guide policy, raw required values and selected precision; preserve title/field/selection/stops/center/missing/domain/exact and occurrence order |
| `theme::TextQueryCache`, selected `ProducerText`, frozen replay | Upstream policies distinguish semantic outcomes; identical complete text/role/font provenance can share lower glyph analysis. Capture/draw never format or reshape |

No Matrix public format is introduced. Its private numeric cell axes use the
same built-in profile. Marginal's central numeric axes carry policy; unlabeled
strips retain their current copied positions, not a new ordinate format. Heatmap
and Beeswarm value/color semantics remain independent of coordinate axes. All
twenty ordinary/five specialized families and TextPanel keep spec06's consumer
matrix and parent boundaries.

## Continuous Preparation And Observed Index Drift

`Canvas::preflight_plan` currently orders structure/interiors, categorical scale
validation, final Shared descriptor merge, Shared-domain installation, census,
font preparation and intrinsics. New Shared format compatibility belongs before
census/fonts. Suppressed local guide formatting cannot fail before the final
owner is known. Shared range coordination alone does not force equal panel text
policies or create a Canvas format override.

The original design's array-order paragraph is no longer current:

- `descriptor::domain_labels` returns canonical lo/interior-center/hi, or one
  scientific constant. Heatmap `prepare_guide_labels` appends `Missing` separately;
  Beeswarm `prepare` and `install_shared_domain` retain the same canonical list.
  Native local guide constructors copy their prepared numeric labels in order.
- `guided_preference` in `layout/interior/sizing.rs` still reconstructs descriptors
  with historical Heatmap `[0,2,1]` and Beeswarm `[1,0]`/constant `[1]` indices.
  Those adapters must be reconciled with the canonical prepared owner during the
  guide consumer migration, including diverging and constant domains. They are
  not evidence that prepared storage still has the old order, and must not be
  copied into the new numeric representation.
- Heatmap Shared replacement recolors fills/stops before role-sensitive census,
  but leaves raw cell numbers distinct. Both families currently regenerate guide
  strings under the default domain-label owner; new descriptor policy/results
  must replace that default-only regeneration.

Waterfall retains two scientific candidates regardless of x-window clipping.
Intrinsics and explicitly filtered census skip outside-x candidates; conservative
composed census may retain both. Actual shelves use `projection.contains_data`
with `source_y=category_count`, including closed x/y boundaries. Adaptive sets
follow effective-window eligibility, not the conservative font set or shelf fit.
The finite four eligibility masks and potential text preparation remain spec06's
approved pre-font solution for legitimate subsequent domain consequences.

## Font Preparation Boundary

`PreparedTextRequirement` currently holds actual `source`, a sorted
`generated_alphabet`, complete stack/weight and canonical owner/role/index/pair.
`push_axis_ticks` gives exact labels precedence over categories and generated
requirements. Unconfigured generated axes and private Matrix use the broad
`NUMERIC_ALPHABET`; preference metrics add actual role-local `0` probes.

The current path is:

1. Materialize canonical requirements/pairs, actual scalars and grammar alphabet.
2. `PreparedTextCensus::prepare_system_fonts` discovers the operation's fonts.
3. `prepare_pairs_in` resolves each `FontPairPreparationRequest` from its stack,
   weight and required scalars.
4. For **each actual source**, call `PreparedFontEnvironment::prepare_source_units`.
   Its `selection_units` retains narrow base-plus-mark spans and prefers the first
   candidate covering the whole span; absent such a candidate, retained `None`
   permits the existing selected-scalar fallback.
5. `seal_prepared_pairs`; later `ResolvedFontPair::selection_for_unit` rejects an
   unknown required unit rather than choosing another face at measurement.

Thus whole-unit fallback already exists, but a generated alphabet alone does not
prepare digit-plus-leading-suffix-mark units. Extend the retained requirement path
with the finite grammar-derived boundary spans before step 4: actual affix spans,
each possible terminal/leading-mark concatenation and applicable prefix/name
boundaries. Use the existing unit definition, not whole-word or emoji fallback.
Known guide/value labels contribute the complete concatenated source. Selected
Heatmap contrast roles follow final fills/backdrop; unused/suppressed policies
must not introduce glyph discovery.

## Arithmetic Handoff And Evidence

The ordinary core decodes exact finite binary64, derives a decimal integer
coefficient/exponent by powers of two/five, shifts scale exactly, and quantizes
once with quotient/remainder/parity. Decimal, Scientific and Engineering emission
normalizes carry without rerounding; signed zero, precision occupancy and
fractional-only trimming follow spec06. Retain a private resolved-precision
interface for later adaptive owners rather than selecting precision during fit.

Use handwritten rational ties/carries and an independent bounded small-rational
oracle. Record extreme coefficients/decimal shifts independently; Rust conversion
is a spelling oracle only for preserved defaults, including the prescribed
binary64 compact division. No engine-generated expected text, platform
transcendental fallback, new numeric dependency or restored resource quota.

The Forest core remains a separate executable proof gate: exact signed exponent
reduction, symbolic integral powers and output lengths precede outward mantissa
certification and the independent integer-square-root/product oracle. Existing
`numeric/tests.rs` MPFR logarithm vectors certify the opposite operation and must
not be relabeled inverse-power evidence.

Focused existing neighbors are spec06's named default/source/census/unit/guide,
Heatmap/estimate/Waterfall and composition tests. `src/spec/tests.rs` still has
three small legacy constructor/default/selective-field inventories plus the
meaningful counted-census extremes test; replace low-value inventories when
adding independent arithmetic coverage. Do not merge existing tests, move them
into integration suites or grow the 671-test Linux library baseline (17 ignored).

This documentation-only refresh used source/spec consistency review and
`git diff --check`, including the new document. No tests, builds, formatter
implementation, output parity or numerical certification are claimed here.
