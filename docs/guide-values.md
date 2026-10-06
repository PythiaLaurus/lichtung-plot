# Complete private guide chrome values

## Actual owning representation

`src/theme/guide.rs` implements the guide constructor schema in §7 of
`specs/94_T6_FOUNDATION_CONSTRUCTION.md`. `ResolvedTheme.legend` now owns one
`LegendTheme`. `LegendAppearance` and the independent
`GLYPH_EDGE_PAD_FACTOR` constant are removed, with their repository consumers
migrated to the current value. The public preset selector and current Shared-gap
override remain their existing APIs.

| Complete value | Retained inputs | Invariants |
| --- | --- | --- |
| `LegendSpacing` | `padding`, `key_width`, `key_label_gap`, `outside_gap`, `shared_gap`, `row_gap`, `column_gap`, `title_gap`, `glyph_edge_padding`: `RelativeSpacing` | Positive key width; other relationships nonnegative |
| `GradientLengths` | `minimum`, `preferred`: `RelativeSpacing` | Both positive; exact preferred >= minimum |
| `LegendFrameTheme` | `fill: Color`, `outline: Option<OutlineTheme>`, `radius: RelativeSpacing` | Complete independent optional outline; nonnegative radius |
| `LegendTheme` | Complete spacing, gradient lengths, optional frame | Revalidates complete children and cross-field invariants |

Every input has its field-named readonly getter and consuming `with_field` edit
returning `Result`. The direct shared-gap convenience edits the sole nested
`spacing.shared_gap` field. Repeated edits replace that field; cloned inputs
remain independent. Frame/outline absence is structural, not transparent paint
or an invalid width-zero construction sentinel.

Data-only `new` and consuming edits validate without fonts, rendering, or a data
domain. Exact gradient ordering uses admitted integer cross-products. Operational
construction uses `GradientLengths::new_in` and `LegendTheme::new_in` on the
existing root resources; it never creates a fresh production ledger. These are
admission paths to the same representation, not alternate modes or fallbacks.

## Exhaustive source mapping

P(p,m) is the exact represented binary64 source p divided exactly by the original
integral role metric m. It does not decode `p / f64::from(m)` as another binary64
input, and does not replace source dyadics with nominal decimal fractions.

| Previous field/source | Sole target field | Preserved construction basis | Local metric |
| --- | --- | --- | --- |
| `padding` | `spacing.padding` | P(8.8,11) | Selected legend-label CSS size |
| `key_width` | `spacing.key_width` | 1/1 | Legend label |
| `key_label_gap` | `spacing.key_label_gap` | P(8.8,11) | Legend label |
| `outside_gap` | `spacing.outside_gap` | P(8.8,11) | Owning legend label |
| `shared_gap` | `spacing.shared_gap` | P(17.6,11) | Canvas legend label |
| `gradient_min_length` | `gradient_lengths.minimum` | 40/11 | Legend label |
| `gradient_preferred_length` | `gradient_lengths.preferred` | 160/11 | Legend label |
| `row_gap` | `spacing.row_gap` | P(3.3,11) | Legend label |
| `column_gap` | `spacing.column_gap` | P(13.2,11) | Legend label |
| `title_gap` | `spacing.title_gap` | P(7.8,12) | Selected legend-title CSS size |
| `frame_fill` | `frame.fill` | `(255,255,255,204)` | Complete RGBA |
| `frame_outline` | `frame.outline.color` | `(200,200,200,255)` | Independent complete RGBA |
| `frame_stroke_width` | `frame.outline.width` | 1.0f32 | Existing CSS stroke/ink normalization |
| `frame_radius` | `frame.radius` | 2/11 | Legend label |
| Former edge factor | `spacing.glyph_edge_padding` | exact `new_in(0.2)` bits | Legend label |

All presets retain the same original guide fonts, title/label sizes 12/11,
weights, RGBA channels, glyph producers, and semantic content. Construction owns
seven P inputs, four native ratios, and one dyadic edge relationship. Standalone
getters expose those exact values; spatial consumers do not use `as_f64`.

## Measured consequences and exact boundaries

`LegendMetrics::from_theme` is fallible and consumes the active operation
resources after role selection. It retains one admitted share of the semantic
component and one set of operation-admitted `LegendLengths` products. The exact
products are not `Clone`/`Copy`; all new integer operands are admitted before
their operations. Scalar metric fields are canonical consequences for existing
glyph/placement consumers, not retained raw-pixel configuration.

Discrete guide chrome, title offsets, entry chrome, cumulative row/column gaps,
and complete edge reserves retain exact operands through their sums. The existing
assigned flow helper compares complete content+chrome requirements before one
upward hard rounding. Its borrowed `FlowCapacity` contains the actual outer
assignment and unrounded reserves; it cannot accidentally add an already-ceiled
content gap to an already-ceiled edge. No new layout/reflow driver is introduced.

Entry minima and measured entry widths retain `ExactLength` through the complete
outer-width guard. Assigned inner capacity subtracts exact edge reserves; label
capacity subtracts exact entry chrome and floors only that final remainder.
`ExactLength::subtract` admits cross-products and subtraction before work and
reports a negative remainder as unavailable capacity. Canonical preferred slots
remain allocation choices; a tighter assignment can use its exact remainder.
Allocation follows `specs/05_LAYOUT_SYSTEM.md`: with exact inner capacity W,
vertical entries share one total column width C=min(max(P[i]),W) for normal flow
or C=min(max(P[i]),C2) for compact flow, where C2=floor((W-g)/2) in layout quanta.
The compact floor follows the complete exact edge/gap subtraction and division.
Horizontal entries use min(P[i],W) or min(P[i],C2). Every exact M[i] must fit its
chosen allocation before assigned label measurement; an entry cannot raise that
allocation to its minimum. Each label then measures once at the final floored
remainder after its own exact key/chrome. Differing key widths do not change the
uniform vertical total width. The former float-only `compact_half_width` helper
is removed.

Vertical flow advances by the preceding column's maximum occupied width, not the
incoming entry's width. This is equal to C in production and also prevents an
independently supplied nonuniform flow from overlapping earlier columns.

The pinned edge example is now explicit:

- Exact P(8.8,11) × 11 plus represented 0.2 × 11 is slightly above 11px.
- Placement S is **11264 quanta**; hard edge H is **11265 quanta**.
- Both edges together round once to **22529**, not 22530 from two independent
  upward-rounded edge estimates.

A live flow fixture supplies three one-third-quantum relationships (two edge
padding operands and one row gap). Two 20px rows require exactly **40961 quanta**,
fit at that boundary, and refuse at 40960. Separate gap ceilings would incorrectly
demand 40962. Entry keys/labels, selected role metrics, and measured ink remain
their actual inputs.

The assigned-width regression exercises `Legend::measure_assigned` itself, in
both orientations and separate normal/compact variants. With no frame and the
selected 11px label role,
padding and key-label gap each use `ratio(1,33792)`: one third quantum. Let B be
the actual measured canonical key width plus the hard width of
`UnbreakableLegendWidthBoundary`. Normal flow retains the complete, single-line
entry at **B+1 quantum** and reports a fit miss at **B**. Compact correctly
rejects that same B+1 assignment: its selected cap is below the exact minimum.
Its own outer-width boundary is **2B+3 quanta**, where C2=B+1 contains M=B+1/3;
at **2B+2 quanta**, C2=B and the compact trial misses. The earlier complete-sum
fix addressed `complete three-third width must fit, compact=false`; the revised
fixture also caught the subsequent invalid minimum raise with
`compact must reject an entry minimum above its assigned cap`.

The actual two-entry vertical regression retains an unbreakable long label and
a short `i` label with differently sized point keys. At W=2A-2 quanta, the first
minimum A exceeds the compact cap; the pre-fix owner accepted that nonuniform
trial and failed the fixture's `compact must not enlarge the first column beyond
C2 and overlap the second` assertion. The fixed owner refuses before assigned
entry text measurement. At W=2A, both compact columns have total width A;
normal columns also fit at their own two-column preferred-width boundary.
One quantum below the compact boundary misses as well.
The fixture captures every selected point and label primitive, verifies complete
ink containment within each uniform column, and checks every cross-column ink
pair for zero overlap without frame or clip concealment.

An independent flow-owner regression supplies occupied widths 80px then 40px at
one entry per column. The incoming-width bug accepted them in 100px; the fixed
flow refuses 100px and accepts exactly 120px with column origins 0px and 80px.
The existing uniform worked example remains 70+10+70=150px; compact width-65
columns remeasure labels and fit only when their minima permit.

Continuous guides retain exact key/label columns, missing-key blocks, title
separation, padding, projected minimums, and final frame extents. Projected
requirements divide exact gap+actual-label extents by the existing represented
semantic fraction separation on the same ledger. Numeric stop/fraction semantics
and the historical separate scientific calculations remain unchanged.

The projected hard minimum and finite maximum are separate from the theme's
preferred length. An exact assigned minimum does not fail merely because its
column and edge were rounded upward separately. The singleton dense-font fixture
verifies minimum/equal/minus-one allocation in both orientations while retaining
all labels and stops. Missing-key outline comfort remains independently measured.

`MeasuredContinuousGuide::vertical_widths` is the common admitted measuring owner
for natural preference and native-right allocation. It uses the measured guide's
effective allocation metrics, including frame and missing-outline comfort, and
retains complete exact key+gap+label/title+padding requirements until the final
whole minimum/preferred width. Neither caller reconstructs widths from rounded
fields or adds a one-quantum allowance.

The natural/native-right regression uses no frame and half-quantum padding and
key-label gap (`ratio(1,22528)` at 11px). The actual measured key and endpoint
labels `0.000000000012345` and `0.000000000098765` define B. The complete width
ceil(B+1.5 quanta) is **B+2 quanta**: natural preference and the actual
`continuous_right` allocator agree, while measured and native allocation refuse
**B+1 quantum**. Before the fix, natural preference failed with
`continuous guide value column exceeds its assigned width`. The fixture uses
Heatmap plot data and the native Grid allocation path, not just an isolated
arithmetic helper.

Frame geometry retains the existing stroke safety inset
`max(ceil(half_stroke),0.5)` for a present frame; radius containment is derived
from the selected exact local radius. Absent frame means no background draw call
and no frame minimum/inset. Spacing, title/labels, keys/gradient, and active
missing-key content remain present. Backend zero-width outline operands are only
the final consequence of `outline: None`, never a theme construction input.

## Admission, comparison, and feedback

Twelve immutable relationships are retained for the default present frame:
nine spacing, two gradient, one radius. Absence of the frame retains eleven.
`LegendTheme::share` admits one visit per actual handle and zero copied payload
records/bytes. Measured guide source retention is priced separately from the
existing owning object/slot storage and text copies.

`LegendTheme::same` admits one fixed frame descriptor and then each actually
examined normalized relationship. A shared equal default guide costs **13
visits**. Independent integers use the existing admitted limb length/digit
witnesses; terminating mismatches and refusal stop before later relationships.
Neither approximate scalars nor unpriced aggregate `PartialEq` compares the
variable-size values on operational matching paths. Feedback records normalized
relationship identity and complete optional paints before measured consequences.

Whole-theme copies now admit **42 appearance handles** (previous 30 plus the 12
guide relationships); default shared matching costs **44 visits**, including
axis and guide descriptor witnesses. The child-copy fixture exhausts the
one-million retained-record allowance and permits no new buffer bytes, verifying
every shared guide payload address alongside fonts/scales/other relationships.
A terminating child tick-font mismatch costs **42 visits**, with later role names
and feedback tails unexamined.

The builtin constructor oracle is independent of measured usage. Seven encoded
source numerator/denominator*metric pairs undergo **36 Euclidean steps**. Each P
construction costs 15 fixed visits plus 6 per step, with a header and admitted
dyadic/division/reduction operands. Four integral ratios cost 4+8+7+6 visits; the
edge dyadic costs 5; the two actual complete gradient-ordering validations cost
2 each. Total guide construction is **355 visits** and, on this target,
**14160 bytes**, with zero new retained record slots. The generic byte certificate
uses actual value/Arc/native integer header sizes rather than an opaque total.

Plot/Canvas constructor tests add this independent guide certificate to their
existing seed/point/axis formulas and verify zero/minus/equal work/byte refusal
before fonts/backend begin. The full preset visit figures become Publication
440, Nature 427, Science/CHI 434. Inherited Canvas children continue sharing the
root; explicit children construct and admit their own complete values.

## Reviewed output normalization

The source defaults are preserved exactly; a rounded composite need not reproduce
the prior ordered floating intermediate. Five SVG expectations were inspected and
updated with these source changes:

| Fixture | Inspected consequence and cause |
| --- | --- |
| Native Bar | Outside gap S changes raw 8.8px to 9011 quanta. Body width `430.3154296875 -> 430.31640625`; guide frame x `546.615234375 -> 546.6162109375`. The guide frame width/height, 6 bodies, uncertainty sources, attached marks, bracket, text and all RGBA remain intact. Exact title+row offsets change selected guide baselines/key positions by one quantum. |
| Native HorizontalBar | Same outside-gap boundary: body width `343.5390625 -> 343.5400390625`; guide x `504.8388671875 -> 504.83984375`. Six bodies, references/uncertainties, bracket and full multiline axis prose remain present; guide size/paints are unchanged. |
| Native StackedBar | Body width follows the same `430.3154296875 -> 430.31640625` boundary. All six stack polygons, their order/paint, bracket and guide sources remain; no statistical input or topology changes. |
| Native StackedHorizontalBar | Body width `385.3154296875 -> 385.31640625`; six stack polygons and all guide/axis/bracket sources remain. Only quantum-level projected attributes and exact guide offsets change. |
| Beeswarm structural | Body width `141.3037109375 -> 141.3046875`; guide title x +1 quantum and high endpoint y +1 quantum. All four observation circles, both numeric endpoints and the title are retained; sequence/RGBA are unchanged. |

Each expectation was reviewed from its generated output and diff before
replacement. No wholesale snapshot update or blanket hash rebaseline was used.
Marginal and Matrix structural snapshots pass unchanged. The intentional
one-quantum hard edge correction is separately certified by the exact aggregate
fixtures, not disguised by display decimals or a preset-specific arithmetic mode.

The two width-review fixes require no additional expectation changes: the native
Bar-family and Beeswarm snapshots pass against the reviewed expectations above.

## Verification and retained cutover boundaries

Focused tests cover all constructors/edits and siblings, source bits, exact edge
and complete-flow boundaries, all current discrete key/ink and continuous/missing
contracts, dense/compact allocation, all assigned edges, fatal cache/resource
refusal, immutable sharing, presets, Shared Canvas/override behavior, sparse
facets and representative actual SVG snapshots. Verification uses offline locked
test filters and strict `just lint`, plus formatting and whitespace checks.

The uniform-column correction is checked through the complete discrete guide
test module, including the unchanged assigned-call certificate and actual
custom glyph/frame ink. Native Canvas/facet guide selection retains byte-identical
complete SVG output with child reuse enabled/disabled across its source families,
with at most eight guide trials per owner/root round. Rejected ancestor trials
retain cumulative guide budgets. These are bounded caller/replay checks; the
source-specific two-entry fixture additionally captures full selected key/label
ink rather than claiming containment from allocated rectangles alone.

Representative bounded commands:

```text
cargo test --locked --offline --lib theme::guide::tests
cargo test --locked --offline --lib decorations::legend::continuous::tests
cargo test --locked --offline --lib local_and_shared_discrete_guides_allocate_complete_custom_glyph
cargo test --locked --offline --lib assigned_discrete_width_keeps_three_fractional_reserves_until_the_full_boundary
cargo test --locked --offline --lib compact_vertical_columns_reject_unequal_minima_above_the_uniform_cap
cargo test --locked --offline --lib vertical_flow_advances_by_the_preceding_occupied_column_width
cargo test --locked --offline --lib continuous_natural_and_native_right_width_keep_half_quantum_reserves_exact
cargo test --locked --offline --lib complete_flow_reserve_rounds_fractional_padding
cargo test --locked --offline --lib compact_vertical_flow_remeasures_labels
cargo test --locked --offline --lib native_real_compositions_remeasure_gutters_and_select_guides_identically_without_cache
cargo test --locked --offline --lib rejected_ancestor_attempts_do_not_reset_child_guide_budget
cargo test --locked --offline --lib multiple_guide_owners_keep_independent_budgets_through_rejected_trials
cargo test --locked --offline --lib decorations::plan::children::tests
cargo test --locked --offline --lib preset_construction_
cargo test --locked --offline --lib one_assigned_frame_reserves_guide_and_gap
cargo test --locked --offline --test composition shared_legend_gap_reduces_canvas_legend_separation
cargo test --locked --offline --test composition shared_diverging_heatmap_keeps_center
cargo test --locked --offline --test composition sparse_cartesian_facets_draw_global_legend
cargo test --locked --offline --test visual-regression native_bar_family_snapshots
cargo test --locked --offline --test visual-regression beeswarm_clip_structure_snapshot
just lint
just fmt-check
git diff --check
```

Composition, mark, and annotation component assembly remain separate leaves.
The current public Shared-gap override and policy checks are retained for B4.
Beeswarm's sequence, scale/domain public APIs, root prose leading, producer glyph
styles, scientific data/clips, and renderer alignment/orientation policies retain
their current contracts. No new dependencies, assets, network access, public
component exports, or fallback construction are introduced.
