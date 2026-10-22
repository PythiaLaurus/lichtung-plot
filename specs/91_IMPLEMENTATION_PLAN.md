# Unified Text, Canvas Panel Spans And Maximum-Width Root Sizing

## Scope And Target Contracts

Implement the requested capabilities through the existing text, Grid and root
lifecycle:

- Consistent plain-text inputs, complete instance typography and horizontal
  alignment, including independently configured TextPanel title/body text.
- Caller-selected categorical tick-label rotation, with presets and custom angles
  using one measurement/rendering path, plus public single-label presentation.
- Explicit rectangular Canvas Plot spans, including a 2x2 Grid with two independent
  left panels and one right panel spanning both rows.
- `Plot::max_width(u32)` and `Canvas::max_width(u32)` as a fourth root sizing mode:
  maximum outer CSS-pixel width with content-derived height. The owning Plot request
  also controls its facets.

This document is a plan, not evidence that these APIs exist. Current contracts
remain in [rendering](02_RENDERING_CONTRACT.md) and [layout](05_LAYOUT_SYSTEM.md).

### Rectangular Canvas Plot Spans

Add this consuming placement method beside the existing single-cell `panel`:

```rust
pub fn panel_span<K: kind::ExactFamily>(
    self,
    row: usize,
    column: usize,
    row_span: usize,
    column_span: usize,
    plot: Plot<K>,
) -> Result<Self, Error>
```

The motivating planned use is:

```rust
let canvas = Canvas::grid(2, 2)?
    .panel(0, 0, top_left)?
    .panel(1, 0, bottom_left)?
    .panel_span(0, 1, 2, 1, right_tall)?;
```

The final call gives one Plot the entire right column. Its assigned rectangle
extends across both row tracks and the intervening gap; its scientific body is
continuous, with one set of local prose, axes and guides.

Required placement semantics:

- Row/column coordinates remain zero-based. Spans are positive and end at checked
  exclusive boundaries within the declared Grid. Reject zero, overflow and bounds
  violations before mutating placement.
- `panel(row,column,plot)` is the 1x1 placement operation through the same private
  validation and insertion path. Support row spans, column spans and rectangles.
- The entire footprint is occupied. Reject overlaps with Plot or TextPanel owners
  in either insertion order, including a single-cell placement at a covered
  nonanchor coordinate. Validate the whole footprint before installing its owner.
- The minimum covered coordinate is the sole owner/anchor. A covered nonanchor
  slot is occupied, not empty, and never contains a copied Plot or an independent
  scientific/text producer. Preserve canonical anchor-row-major order.
- Retain declared rows, columns, gaps and empty/trailing tracks. Placement is
  explicit; it does not compact or repack the Grid or change root size policy.
- Preserve Canvas-admissible family restrictions. Matrix and already-faceted
  Plots remain excluded; native admissible families retain their current route.
  TextPanels remain single-cell participants in the shared occupancy model.
- Canvas assigns the complete spanned child geometry. Embedded standalone width,
  size and future maximum-width requests do not override that assignment.

The separate owner and composition rules below are part of the span feature, not
optional follow-up work after exposing the setter.

### Maximum-Width Roots

| Request | Width selection | Height |
| --- | --- | --- |
| Omitted | Natural preferred outer width, chosen once | Measured at the selected width |
| `width(w)` | Exactly `w` | Measured at `w` |
| `max_width(m)` | Natural preferred outer width capped by `m` | Measured at the selected width |
| `size(w,h)` | Exactly `w` | Exactly `h`, fit or fail |

For an ordinary representable natural width `P`, the new policy is
`W = min(P,m)`. Apply the cap to the existing checked natural outer-width
requirement before conversion to integral public dimensions. Round the selected
requirement upward using the existing CSS-dimension rule; an integral cap must
never be exceeded by rounding. A soft natural preference that exceeds `u32` must
not itself reject a representable capped result. Genuine checked arithmetic,
allocation and hard-content failures remain errors.

The cap covers the complete root: padding, prose, axes, guides, scientific body,
Canvas cells/headers and gaps. It does not name a data-panel width or a PNG target.
It is not a request to fill the cap when natural content is narrower.

Required setter and composition semantics:

- The latest `width`, `max_width` or `size` call replaces the entire root request.
  `max_width` clears an earlier exact height and exact width; bounds do not stack.
- Zero rejects during root preflight before font discovery or backend begin;
  setters do not panic. A superseded zero request has no remaining effect.
- Canvas alone assigns embedded Plot geometry. Embedded standalone requests,
  including `max_width(0)`, are ignored just like existing child width/size requests.
- The root chooses width once from prepared intrinsic preferences. Later gutter,
  alignment, ink, domain or content-height feedback cannot reselect it or raise it
  beyond the cap. Height remains content-driven under the existing eight-round
  coordinator.
- Complete required text, scientific sources, annotations and guide sections must
  fit. Preserve the selected typography, mark metrics and owner-local spacing;
  no fit-driven padding compression, font shrink, omission or root-width search.
- Output reports the actual selected canonical width/height. SVG modes and PNG
  options replay that frozen scene; PNG scaling does not rerun root layout.

This scope does not admit height-only sizing, minimum dimensions, maximum height,
public generic constraints, nested Canvas objects or implicit topology changes.
Component-box-responsive spacing is outside this plan.
Residual root-bound proposals remain in [deferred features](99_DEFERRED_FEATURES.md).

## Existing Owners And Required Changes

The tables and details here cover Canvas/root sizing. The text consolidation and
rotation requirements and their owners are specified in the following section.

| Owner | Responsibility |
| --- | --- |
| `src/layout/kernel.rs`, `src/layout.rs` | One complete root request, mode validation, selected-width locking, final bounds validation and the existing prepare/measure/freeze lifecycle |
| `src/builder.rs`, `src/builder/configuration.rs`, `src/builder/accessors.rs`, `src/builder/facets.rs` | Plot storage, public setter, exact assigned geometry, erasure/copying and composed-input equality |
| `src/orchestrator/lifecycle.rs` | Standalone/faceted dispatch and automatic-height selection |
| `src/layout/interior/sizing.rs` | Unfaceted natural outer-width composition and selected-width height proposal, including native scientific families |
| `src/orchestrator/facets/sizing.rs`, `src/orchestrator/facets/tracks.rs`, `src/orchestrator/facets/planning.rs` | Owning-root preference, sparse declared facet columns, shared axis titles and width-dependent cell/strip heights |
| `src/canvas.rs`, `src/canvas/sizing.rs`, `src/canvas/tracks.rs` | Placement/occupancy, owner preparation and capture, span-aware natural/assigned geometry, headers, Shared/PerPlot guides and content-height measurements |
| `src/canvas/plot_text.rs`, `src/canvas/text_panel.rs`, `src/layout/interior.rs` | Local prose within one owner assignment, TextPanel occupancy and existing child-family restrictions |
| `src/layout/kernel/feedback.rs`, `src/canvas/dependencies.rs`, `src/decorations/plan/children.rs` | Audit actual request/constraint consumers and operation-local reuse identity |

### Canonical Placement Ownership And Declared Geometry

Replace fixed-slot placement assumptions with one canonical owner record containing
the payload and validated footprint. Derive occupancy lookup from these records;
it is not another store of Plot payloads. Migrate existing Plot/TextPanel consumers
to this model without retaining a legacy placement path. Preparation/theme/domain
arrays may remain Grid-sized with values only at owner anchors.

Preserve owner addresses as anchor slot indices, not compact insertion ordinals.
In the motivating 2x2, anchors are slots 0, 1 and 2; slot 3 is covered by owner 1.
Keep one `CanvasCell` address per owner. Preparation, intrinsic/text census and
source/guide contributions enumerate each owner once. Child measurement may repeat
under existing feedback/local-trial rules, never once per covered slot. Retain one
owner-scoped height consequence and selected plan/capture. Shared guide source
occurrences must not be duplicated even when visible keys would otherwise merge.

Separate declared-track geometry from owner assignments in `CanvasGridAssignment`.
Headers/row-title positions currently index first-column `cells`; changing those
entries to tall owner boxes would corrupt row boundaries. Retain actual row/column
edges for composition chrome and obtain each owner's full rectangle from the
covered interval. Covered slots must not become child measurement destinations.

### Span-Aware Grid Measurement And Feedback

Build `GridRequirement` values from each owner's complete footprint. Reuse
`allocate_grid_axis` and its canonical spanning hard/preferred allocation rules;
do not divide a Plot's minimum by its span or put the entire minimum into the
anchor's first track. Columns still precede width-dependent measurement and rows.

Measure local prose, native scientific bodies and PerPlot guides at the full
assigned span width. An owner's height requirement applies to its entire row
interval. Derive spanning boxes from actual track boundaries, including internal
gaps/bands exactly once. Apply the owner's leading/trailing gutter and owned-ink
requirements at its outer boundaries, without repeated interior padding or an
internal panel separator. Preserve adjacent single-cell panels' own reservations.

Use the same footprint-aware requirements for natural sizing, exact allocation,
`complete_cell_gutters`, group alignment and content-height feedback. Replace the
separate Shared-guide minimum-body calculation in `build_canvas_attempt`, which
currently puts each Plot's minimum into one row/column, with requirements composed
under the same span rules. A guide trial cannot reserve space using stale 1x1
minima and then assign a different spanned body.

Height and child-reuse dependencies include consumed footprint/topology, full
assigned geometry, outer insets and relevant guide/group feedback. Record one
height consequence and one selected plan per anchor; equal width alone cannot
authorize reuse after footprint or assignment changes. Preserve one operation,
the existing root feedback bound and frozen publication lifecycle.

### Span Interactions With Composition Policies

- **Shared ranges:** coordinate-based `x_range_group` members must name configured
  Plot anchors. Reject covered nonanchor references rather than treating them as
  aliases; keep groups declarable before placement and validate resolved ownership
  during preflight. Shared domains and alignment visit distinct owners once and
  apply leading/trailing requirements to the relevant footprint boundaries.
- **Category labels:** `EveryPlot` remains the default. For `FirstColumnPerRow`,
  all Plot owners participating in a shared row band must cover the same row
  interval and retain the existing rendered-category compatibility checks. Resolve
  occupancy to the first-column owner, process the band once, and compare each
  distinct owner once, including horizontal spans. Preserve the current rejection
  of missing/non-Plot peers. Reject mismatched vertical intervals before fonts;
  the motivating tall-right/short-left layout uses `EveryPlot`.
- **Row titles:** a title band before row `r` cannot cut through an owner with
  `start_row < r < end_row`. Reject that conflict during structural preflight.
  Preserve the existing prohibition on conflicting local Plot title/subtitle
  content for the row's distinct owners. Titles at a span's outer boundary use the
  declared row geometry, not an anchor's enlarged box.
- **Headers:** existing external row/column headers retain their declared spans
  and boundaries, including beside a first-column vertical span. Their geometry
  does not establish additional Plot owners or split a spanning Plot.
- **Guides and capture:** PerPlot/Shared/Hidden retain existing semantics. Shared
  categorical/continuous/alpha sources and prepared font requirements use one
  contribution per real owner, independent of the number of covered slots.

### One Mode-Aware Root Request

Replace the current optional-width/optional-height request encoding with a closed
representation of the supported modes. The final mode set is Natural, ExactWidth,
MaxWidth and ExactSize. Keep measured/frozen dimensions concrete and checked; a
maximum is never an assigned width merely because it is present in a request.

Replace `plan_exact_root(width,height,...)` with one mode-aware root entry point,
named `plan_root(request,...)`, and migrate every repository caller. Remove the
old signature and pair-based request decoding rather than keeping an adapter or
parallel coordinator. Exact assigned-box helpers must still reject unresolved
requests. Preserve the parent's ability to install concrete child geometry.

Use one shared width-selection rule across Plot, owning facets and Canvas. Retain
the original request/cap for validation while subsequent rounds receive the
already selected width. Check the first measured result against its mode before
accepting feedback; reject later width changes and final out-of-bound results
before capture. Preserve exact-size validation and fatal/fit error distinctions.

Carry complete request identity through storage, erasure, source attachment and
the equality/feedback dependencies that consume it. Equal output dimensions do
not make a capped request interchangeable with an exact-size request. Keep the
existing one-operation cache boundary and complete assigned-geometry keys.

### Width Selection Before Height Measurement

Reuse the existing scientific, text and guide preferences. Do not add a second
definition of natural width for the capped mode. Root padding, axis estimates,
guide gaps and prose widths participate exactly once under their current owners.

The existing Canvas/facet natural-width probes call Grid allocation that continues
into width-dependent height measurement. Separate root width selection from final
selected-width height work, reusing column requirements, canonical span handling
and allocation rules. Preserve sparse/trailing tracks and the current initial
preference's exclusion of later feedback. Do not require a successful complete
uncapped root layout or conversion of uncapped width/height to public dimensions.

Preserve existing local height dependencies in the preference calculation. In
particular, `shared_axis_requirements` measures a vertical facet axis title against
the natural Grid height, and its perpendicular thickness contributes to natural
outer width. Retain the necessary scoped Grid-height/title measurement in the
preference pass; do not silently substitute a one-line title, zero height or a new
width formula. Propagate its genuine typed errors. After selecting width, measure
actual content and shared-title gutters at that width under the existing feedback
rules. This local dependency does not authorize iteration over root widths.

For an unfaceted Plot, select width after composing body/prose/guide preferences
and before `natural_body_height` and width-constrained prose measurement. Preserve
Heatmap, Beeswarm, Waterfall, Matrix and Marginal native sizing routes. After width
selection, use the current exact-width/automatic-height machinery and named
feedback rather than a second fit algorithm or an extra root operation.

## Unified Text And Explicit Category-Label Rotation

### Confirmed Text Boundaries

Keep ordinary Unicode legal wrapping and caller-authored mandatory line breaks.
Do not introduce emergency word/identifier/grapheme splitting or a caller-supplied
internal-break rule. A name such as `LightGBM` must remain intact when passed as
explicit or raw display text, even when a narrow allocation cannot fit it.

Keep the current default `Labels::humanized()` for automatically sourced names.
Explicit title/axis-label/TextPanel strings remain verbatim. Humanization changes
display text during label preparation, independently of fit; it is never invoked
because a string failed to fit. Preserve `Labels::raw()`, alias precedence, raw
scientific identities and domain-level display-collision checks.

Rotation is always a caller instruction. Omission retains the existing direction.
Do not search for an angle or select between rotation and additional word breaks.
The caller can combine explicit rotation, manually inserted newlines and ordinary
legal wrapping. Required labels still fit completely or produce a typed fit miss;
existing explicit Heatmap Wrap/Elide semantics retain their own owner.

### One Plain-Text Content Contract

Put shared display-text validation in the existing Rust module structure, with
one private owner such as `src/text.rs`. All displayed strings use the same Unicode
control and mandatory-break rules before entering the existing shaping/wrapping
engine. In particular:

- Accept `\n`, CRLF and the engine's supported mandatory separators consistently.
  Treat CRLF as one break. Preserve leading, interior and trailing blank lines and
  caller whitespace; neither trim nor humanize explicitly supplied display text.
- Preserve an explicitly present empty text value and its engine-defined empty
  line metrics. Absence remains an owner-level `None` or explicit visibility policy,
  not an accidental result of trimming. Preserve exact-tick empty-label semantics:
  an empty label suppresses its text/gap while retaining the scientific position.
- Continue to reject unsupported controls such as tabs according to the common
  engine contract. Do not add a tab-expansion or platform-dependent policy.
- Replace independent `char::is_control`/single-line/trim checks on free-form display
  content with the shared validator. Fallible constructors may validate earlier,
  but the accepted content is the same as preflight-validated builder inputs.
- Keep raw category/series/field keys, font-family names, target identities, paths
  and NumericFormat grammar under their semantic validators. Display aliases are
  distinct from those keys and accept the common display-text syntax.

Apply the contract to Plot/Canvas prose, x/y axis titles, TextPanel title/body,
Canvas headers/row titles, exact tick display labels, label aliases, guide display
text and caller-authored panel/point-label text. Remove the current multiline
prohibition on `PanelTextAnnotation` display content and fit its complete block at
the existing corner. Keep its corner ownership and existing family/facet policy.
Audit other named display labels by semantic use rather than relaxing every
string validator in the repository.

### Common Authored Text And Style Resolution

Expose one crate-root `Text` value through a private implementation module:

- It retains one source string, an optional complete exact `TextTheme`, and an
  optional horizontal `TextAlignment` instruction.
- Plain `String`/`&str` convert into this same representation without local style
  instructions. Relevant free-form setters/constructors accept `impl Into<Text>`;
  strings are the ergonomic ordinary input to one implementation route.
- `Text::new(...)`, `with_style(TextTheme)` and `with_alignment(TextAlignment)`
  give x/y titles, prose and TextPanel content the same typed construction model.
  TextPanel title/body remain separate Text values and can be styled independently.
- Reuse the complete `TextTheme` for stack, RGBA, positive CSS font size, closed
  weight and positive line-height multiplier. Existing `with_font_size`,
  `with_weight`, `with_color`, `with_stack` and `with_line_height` remain the edits
  on that one appearance type. An exact style replaces the complete role; do not
  introduce a parallel sparse font-style cascade or a backend-style public API.
- Replace public `TitleAlignment` with shared `TextAlignment` and migrate all
  repository callers. Start/Center/End retain physical left/center/right meaning,
  including for RTL text. Remove the old type/export rather than adding an alias.
- Authored prose, x/y titles, TextPanel, headers/row titles and panel annotations
  accept the common instance value. Exact tick labels and automatic/generated
  text use common content validation and their existing role styles; this change
  does not create per-generated-value styling callbacks or a rich-text parser.

Each text-bearing slot retains source and local instructions once. Existing
heading-alignment convenience setters edit that slot's one alignment instruction;
they do not create a second precedence channel. Preserve content-only setter
behavior: a string replacement does not erase an explicit heading alignment.
Supplied Text attributes and convenience edits use latest explicit instruction
for the same property; an exact appearance edit replaces the whole exact style.
Theme attachment order remains independent of these local instructions.

Style resolution is local exact style over the selected owner's complete Theme
role. Resolve before font census and measurement; omitted appearance uses the
existing Plot/Canvas ownership and role defaults. Existing Canvas panel heading
defaults remain below explicit child instructions. TextPanel alignments default
to Start independently. X and Y titles can now have distinct local styles despite
sharing the Theme axis-title default.

Both block placement and every line consume the selected alignment over the
post-inset width. For a vertical axis title, alignment is along the title's logical
line before its fixed axis transform. Container padding, anchoring and scientific
projection remain parent geometry. Font-dependent spacing consumes its declared
role's final selected metric: a local title size changes its title clearance,
while root padding still uses the root owner's body metric. Do not resolve gaps
against stale Theme font sizes after selecting an exact style.

Prepared text requirements must enumerate actual selected stacks/weights and
source text, including local styles. Intrinsics, constrained measurement, guide
compatibility, height caches, child reuse and frozen drawing consume the same
resolved inputs and alignment. Preserve actual ink/leading and complete equality;
equal strings or dimensions alone do not authorize reuse.

### Public Single-Label Presentation

Promote the existing `Labels::resolve(&self, raw: &str) -> Result<String, Error>`
to a documented public method and reuse its current implementation. This gives
`Labels::humanized().resolve("long_sample_name")?` to explicit-text callers and
also honors Raw/aliases through the same policy. Do not introduce a second public
humanizer or copy the tokenizer. Document that it may change case/separators and
acronyms, returns ordinary display text, and never inserts fit-dependent breaks.
The single-label method has no domain collision context; `resolve_domain` remains
responsible for detecting collisions among distinct raw identities.

### Preset And Custom Rotation Through One Value

Expose one validated `TextRotation` value with horizontal, upward/downward 45-degree
tilts and upward/downward vertical presets, plus `degrees(f64) -> Result<Self>`.
Factories can present enum-like named choices, but their retained result is one
canonical angle. Preset 45 degrees and custom 45 degrees have identical geometry
and cache identity. Reject nonfinite inputs, normalize full turns/signed zero and
apply the existing canonical angle precision before fit measurement.

Angles are degrees from physical horizontal in the top-down output frame, positive
clockwise. The upward/downward tilt presets are -45/+45, vertical presets -90/+90
and horizontal 0. Normalize equivalent turns to a single range before rounding,
with one representative for the half-turn endpoint and zero.

The current canonical text output stores tenths of a degree in
`src/render/coordinates.rs`. Retain and document that precision for custom input,
including its existing rounding rule, and use that canonical value throughout
measurement, collision, capture, SVG text, outlined paths and PNG. Do not measure
at the original angle and round only during output. Keep exact quadrantal
transforms and a shared sine/cosine operand for other angles so geometry and
outlined rendering cannot disagree at the same frozen angle.

Add `x_category_label_rotation`/`y_category_label_rotation` Plot instructions with
sealed axis-specific capabilities based on actual categorical axes. Do not use
`kind::CategoricalScale`, which describes color scales. X-side families are
Interaction, Heatmap, Bar, StackedBar, Boxplot, Violin and Raincloud. Y-side families
are PointInterval, Coefficient, Forest, Dumbbell, Heatmap, HorizontalBar,
StackedHorizontalBar, Beeswarm and Waterfall. Matrix variable titles are not
categorical ticks and do not acquire these setters. Preserve capability state
through erasure, owning facets and Canvas child preparation.

Measure at the caller-selected angle from the start. The current label path checks
an unrotated plan against one axis-parallel capacity; changing only the final SVG
transform would reject useful tilts or accept overflowing labels. Extend the
common label-cell measurement to account for transformed logical extents and
outward ink in both parallel and perpendicular directions. For an unrotated
rectangle `(w,h)`, its transformed axis-aligned extents are
`(|cos|*w+|sin|*h, |sin|*w+|cos|*h)`; include the actual anchor, per-line ink,
baselines and required cell clearance rather than using this rectangle alone.

Try intact explicit paragraphs at the fixed angle first. Where ordinary legal
wrapping is needed, select an owner-defined deterministic inline capacity and
validate the resulting complete rotated plan against its assigned cell/outer
region. Retain the existing immutable-plan/typed-miss protocol; no search over
angles, arbitrary word splits, root widths or all wrap partitions is introduced.
Unbreakable text can fit through rotation without first fitting the horizontal
cell width. If the selected policy still cannot contain it, return a fit miss.

Feed actual transformed perpendicular thickness and endpoint overhangs into the
existing axis-gutter/owned-ink feedback. Preserve tick positions, semantic order,
all category members and linked scientific projections. Update natural preferences
and specialized assigned-width producers, including Heatmap Wrap/Elide and the
Beeswarm/Waterfall `assigned_tick_width` routes. Explicit Heatmap elision uses the
same selected angle; it must not be activated as an automatic escape elsewhere.
The number of root rounds and local guide alternatives remains unchanged.

### Text Implementation Owners

| Owner | Change |
| --- | --- |
| `src/text.rs` (new private module), `src/lib.rs`, `src/labels.rs` | Shared authored text/content validation, alignment/rotation values, deliberate root exports and public single-label resolution |
| `src/theme/resolved.rs`, `src/theme/typography.rs`, `src/theme/composition.rs`, `src/theme/axis.rs` | Reused complete appearance, selected local metrics, common alignment and resolved text inputs |
| `src/builder.rs`, `src/builder/configuration.rs`, `src/builder/accessors.rs`, `src/builder/facets.rs`, `src/kind.rs` | One text slot/instruction representation, exact axis capabilities, erasure/attachment and dependency identity |
| `src/canvas.rs`, `src/canvas/text_panel.rs`, `src/canvas/plot_text.rs`, `src/decorations/title_block/plan.rs` | Authored content, independent TextPanel alignment/styles, preserved explicit breaks and width-dependent reserves |
| `src/layout/text.rs`, `src/layout/text/requirements.rs`, `src/font/canonical.rs`, `src/layout/fit.rs` | Common engine reuse, selected-font census and coherent intrinsic/constrained input identity |
| `src/layout/panel.rs`, `src/layout/panel/cells.rs`, `src/layout/panel/axis_frame_grid.rs`, `src/layout/interior/sizing.rs` | Complete text on axes, fixed-angle capacity/anchor/gutter/ink measurement and natural preferences |
| `src/orchestrator/facets/planning.rs`, `src/orchestrator/facets/sizing.rs`, `src/canvas/sizing.rs`, `src/canvas/tracks.rs` | Shared/local title styles, spanning assignments and width-first content dependencies |
| `src/plots/heatmap.rs`, `src/plots/beeswarm.rs`, `src/plots/waterfall.rs`, `src/spec.rs`, `src/labels.rs` | Native categorical routes and display-versus-identity validation |
| `src/render/coordinates.rs`, `src/render/geometry.rs`, `src/render/primitives.rs`, `src/render/svg_backend.rs`, `src/orchestrator/backend.rs` | One canonical text angle/transform and frozen text/outline/raster replay |

## Delivery Order

Consolidate text inputs/styles before migrating Canvas ownership, so the new owner
records retain the final text representation. Complete spans and fixed-angle label
measurement before refactoring natural width for the cap. Each code-changing step
runs affected focused checks and strict Clippy; commit completed logical changes
before proceeding. Update current contracts with an exposed capability, not ahead
of it. Proposed Text/rotation API spelling is reviewed with its implementation;
the behavioral requirements above are the acceptance boundary.

### 1. Unify Display-Text Validation And Expose Label Resolution

Apply one shared content validator to free-form display slots, retaining source
whitespace and explicit mandatory breaks. Remove accidental multiline rejection
from TextPanel/headers/panel notes while preserving semantic key validators and
explicit empty-tick behavior. Expose the existing label resolver and retain default
humanization, Raw/alias precedence and domain collision detection. Replace obsolete
input-refusal tests with shared-input preservation/real-control rejection witnesses.
Update current display-input contracts and user examples with this implemented
behavior in the same change.

### 2. Consolidate Authored Text, Styles And Alignment

Introduce the common Text value, migrate receiving slots and replace TitleAlignment
with TextAlignment across repository callers. Resolve exact appearance/alignment
once before the font census, including selected local spacing metrics. Complete
independent TextPanel title/body alignment and x/y title styling through the shared
engine, with one representation behind strings and styled input. Update specs
01 through 05 where their text/ownership contracts change and the existing theme/
getting-started guides. Verify inherited/local precedence, used-font completeness,
width-dependent height, actual ink and frozen replay before committing.

### 3. Make Canvas Ownership And Track Geometry Explicit

Migrate current 1x1 Plot/TextPanel placements to canonical payload/footprint owner
records and derived occupancy. Preserve anchor IDs through preparation, theme/
domain inventories, text census, guide contribution, child plans and rendering.
Separate declared track boundaries from assigned owner rectangles. All existing
single-cell behavior must continue through this one representation; do not add a
second span-only pipeline or unused future API.

Validate occupancy, sparse tracks, header/row-title geometry, shared source
provenance and assigned projections using existing focused witnesses. This step
is a completed internal migration before exposing multi-cell placement.

### 4. Implement Rectangular Plot Spans End To End

Expose `panel_span` together with checked whole-footprint validation, span-aware
hard/preferred requirements, full-box child measurement, guide minima and feedback/
reuse identity. Implement the composition-policy rules above in the same feature.
Keep single-cell `panel` and TextPanel insertion on the shared occupancy path.

Replace lower-value/obsolete tests with the motivating three-owner layout and
other span contracts below. Update `specs/01_ARCHITECTURE.md`,
`specs/02_RENDERING_CONTRACT.md`, `specs/05_LAYOUT_SYSTEM.md`, public API examples
and `docs/getting-started.md` with implemented placement/ownership behavior.
Existing root modes must all support spanning owners at this step; maximum-width
support is added through their shared allocation path later.

### 5. Add Explicit Category-Label Rotation

Implement the one validated preset/custom-angle value, sealed x/y instructions
and canonicalization before measurement. Extend label-cell fit, transformed
gutters and native width producers at the selected fixed angle; reuse the same
transform operands through capture and all output modes. Verify no default angle
change or automatic split/drop, preset/custom equivalence, nonquadrantal actual
ink containment, explicit multiline labels and relevant Canvas/facet/native routes.
Document the API, sign convention and canonical precision with implemented behavior.

### 6. Make Existing Root Modes Explicit

Replace the optional-pair request representation and coordinator signature for the
three existing modes first. Migrate all production and test callers, embedded
assignment helpers and equality consumers in the same logical change. Do not add
an unused future mode or a compatibility wrapper.

Validate exact-pair enforcement, automatic-height behavior, latest-setter
replacement, zero preflight, width locking and one prepare/capture lifecycle with
the focused checks below. Commit the completed representation change after those
checks and strict Clippy pass.

### 7. Separate Natural Width Composition From Height Work

Factor the necessary width-first paths in Plot, facets and the now span-aware
Canvas while retaining their current three-mode behavior. Use the same column
summaries/allocator for natural and assigned dimensions, including Plot footprints,
headers, row-header bands, shared axis gutters, local Plot prose and guide
preferences. Avoid a generic new layout DSL.

Validate current-mode semantic equivalence, sparse facet/Canvas topology,
unbreakable text requirements, native Matrix sizing and mixed/all-text Canvas
height measurement. Commit this refactoring after focused tests and strict Clippy.

### 8. Add And Document Maximum Width End To End

Add MaxWidth to the single request representation and expose both consuming
`#[must_use]` setters, following the existing `pub const fn` builder style. Apply
the common cap policy in all three root routes and enforce bounds throughout the
coordinator. Update actual request/height/cache dependencies and preflight handling.

Replace lower-value/obsolete tests with the contract coverage below and keep the
library-test budget. Update current contracts in `specs/02_RENDERING_CONTRACT.md`
and `specs/05_LAYOUT_SYSTEM.md`, API examples, `docs/getting-started.md` and any
affected README sizing guidance together with the implemented behavior. Remove
the admitted maximum-width subset from the deferred list when this plan is
approved; preserve the remaining root-bound discussion. Commit the completed
feature after focused verification and strict Clippy.

## Verification And Acceptance

### Text And Rotation Relationships

1. **Common source syntax:** the same explicit text with newlines, CRLF, blank lines
   and whitespace is accepted by x/y titles, TextPanel, prose and headers, with
   preserved source and real line metrics. Validate shared shaping/line content
   rather than comparing their intentionally different default placements/styles.
   Replace the panel-note test that rejects all multiline/untrimmed text.
2. **Complete local styles:** choose different x/y-title and TextPanel title/body
   stacks/sizes/weights/colors/leading, retaining unrelated Theme roles. Selected
   custom fonts are included before preparation is sealed; unused fonts remain
   unnecessary. Font-relative gaps and height consequences use selected metrics.
3. **Alignment and precedence:** independent Start/Center/End for title/body,
   multiline line placement, physical meaning with RTL text, inherited owner Theme,
   local exact appearance and existing heading convenience edits. Verify source
   completeness, containment and frozen replay without whole-scene snapshots.
4. **Label presentation:** direct `Labels::resolve` and the automatic path agree
   for existing humanization, Raw and aliases; default humanization is unchanged.
   Explicit/raw `LightGBM` remains intact, independently of width, and a too-narrow
   exact allocation refuses it rather than inventing internal breaks. Public
   resolution returns ordinary text and domain collisions are still rejected.
5. **Fixed-angle equivalence:** presets and equal custom angles normalize to the
   same value and actual geometry. Exercise both signs, quarter turns, a custom
   angle such as 30 degrees, fractional rounding/full-turn/signed-zero boundaries
   and nonfinite rejection. Use independent transformed-corner/containment oracles,
   not the production measurement function as its own expected result.
6. **Useful tilts and strict fit:** a long categorical label that cannot fit
   horizontally can fit at its explicitly selected angle; all labels/positions
   remain. Check transformed outward ink and logical clearance, including explicit
   multiline labels and a genuine nonfit. Omission still fails intact rather than
   choosing an angle, changing font size or splitting `LightGBM`.
7. **Affected owners and output:** representative x-category, y-category and native
   Heatmap/Beeswarm routes, a facet, a Canvas spanning child and one capped root.
   Retain shared-axis alignment, exact geometry dependency identity, one capture
   and same-Output replay. Text-preserving SVG, outlines and PNG use the same
   canonical rotation and complete content. Avoid mechanical family cross-products.

### Canvas Span Relationships

Use independently meaningful requirements and geometric relationships, without
pinning automatic row sizes or internal placement coordinates:

1. **Motivating layout:** a 2x2 with two left owners and one right owner spanning
   both rows. The right owner receives the full covered rectangle, with continuous
   body/clip geometry and no gap through its interior; all three sources/prose
   appear exactly once. Verify owner-specific preparation/capture evidence rather
   than relying only on SVG ID uniqueness or visible-key deduplication.
2. **Other rectangles:** the transposed three-owner layout exercises column spans;
   a small multirow/multicolumn footprint exercises both directions together.
   Assert containment, nonoverlap and agreement with declared track boundaries.
3. **Whole-footprint occupancy:** reject zero spans, checked endpoint overflow,
   out-of-bounds and overlap with Plot/TextPanel owners in both placement orders.
   A covered nonanchor coordinate is occupied. Invalid placement must not reach
   font/backend work or create partial output.
4. **Natural and assigned sizing:** run representative spans through natural,
   exact-width/automatic-height and exact-pair roots. Preserve required local prose,
   axes/guides and native-family content; embedded standalone requests stay ignored.
   Keep empty/trailing declared tracks and prevent hard-minimum violations.
5. **Composition policies:** verify anchor-only range-group references, compatible
   same-row-band category sharing, rejection of mixed vertical footprints, row-title
   conflicts inside spans and correct headers beside first-column spans. Retain
   unit-cell category/row-title behavior without covered-slot aliases.
6. **Guides and lifecycle:** representative Shared categorical/alpha and continuous
   sources retain their true occurrence counts and domains. Guide-space trials use
   spanning minima. Full-span width/geometry or relevant parent feedback invalidates
   the affected height/child plan; unchanged sibling owners retain valid reuse.
7. **Publication:** same-Output replay equality, source-sized PNG dimensions and
   explicit raster scaling follow the frozen composed scene with one root capture.
8. **Combined cap:** after maximum width is implemented, cap the motivating layout
   and assert the entire root bound, complete sources, retained occupancy and
   containment. Use one representative combined witness, not a new family matrix.

### Maximum-Width Relationships

Use a bounded selection of meaningful contracts, not a new all-family x root x
output matrix:

1. **Nonbinding cap:** compare the current natural route and a larger cap on the
   same source/theme. Retain natural dimensions and complete semantic content;
   do not pin a renderer revision's automatic size.
2. **Binding cap:** use reflowable content whose natural width exceeds the chosen
   explicit cap. Actual width equals that cap, content stays complete and measured
   height is positive. Compare dimensions/content with the current `width(cap)`
   route rather than a copied layout algorithm.
3. **Request replacement:** cover replacement in both directions with width and
   size, including superseded zero requests and clearing an earlier exact height.
4. **Validation and refusal:** root `max_width(0)` fails before font/backend work;
   a positive cap below a real unbreakable requirement fails before publication
   without truncation, partial output or modification of an existing destination.
5. **Width-first numerics:** independently exercise an integral cap at a fractional
   quantum boundary and a checked soft preference above the public dimension range.
   The cap is applied before integral conversion; real arithmetic failures remain
   fatal. Test width-selection behavior through the root boundary, not helper
   setter/getter echoes.
6. **Root coverage:** representative standalone coordinate, native Matrix and one
   width-sensitive native guide route; owning sparse/wrapped facets with multiline
   shared axis titles; mixed Canvas with spanning Plot owners/headers and
   Shared/PerPlot guide consumption; all-text Canvas. Verify root bounds and
   complete sources without changing declared topology.
7. **Embedded requests:** Canvas allocation ignores child standalone maximum-width
   requests, including zero; compare with the equivalent child without that request.
8. **Lifecycle and outputs:** retain one scientific/font preparation, selected
   capture and final validation; freeze a width that stays fixed across feedback.
   Repeated same-Output SVG replay is equal, source-sized PNG follows actual root
   dimensions and explicit PNG scaling changes only raster dimensions.

Keep tests in their existing logical owners:

- `src/layout/text.rs`, `src/theme/typography/tests.rs`,
  `src/layout/panel/axis_frame_grid/tests.rs`, `src/render/geometry.rs`,
  `src/orchestrator/backend.rs`: mandatory breaks/spacing, selected metrics,
  fixed-angle cell/ink constraints and canonical frozen transforms.
- `src/layout/kernel/tracks.rs`: canonical span hard/preferred allocation and
  declared sparse-track relationships, distinct from placement-overlap rejection.
- `src/layout/kernel/tests.rs`: modes, invalid requests, bounds, quantization,
  selected-width locking, convergence and capture boundaries.
- `src/layout/interior/sizing.rs` and `src/canvas/sizing/tests.rs`: width-first
  preference/height dependencies where private behavior needs direct evidence.
- `src/canvas/lifecycle_tests.rs`, `src/orchestrator/tests/lifecycle.rs`: unique
  source/prose/guide ownership, assigned projection, early refusal and one lifecycle.
- `tests/layout/root_sizing.rs`: externally compiled sizing, complete sources,
  representative root owners and SVG/PNG publication relationships.
- `tests/composition/canvases.rs`, `tests/rendering/contract.rs`,
  `tests/output/multiformat.rs`: public placement/occupancy, headers/category/range
  policies, shared continuous guides and frozen publication.
- `tests/rendering/labels.rs`, `tests/annotations/panel_text.rs`,
  `tests/composition/foundations.rs`: existing label policy/defaults, multiline
  authored notes and used-font/local-style contracts.

Record same-platform discovered library counts before and after each test-changing
step. Current Linux ceiling is 671 including 17 ignored; each new unit-test function
must replace an existing one in the same logical change. Name replacements in the
change review and remove obsolete helpers. Do not evade the limit by merging
functions, packaging cases or moving unit responsibilities into integration tests.

### Focused Commands

Use the narrowest applicable selections at each code-changing step and confirm
nonzero execution. If a named test is deliberately replaced, use its replacement
filter. Do not claim discovery as behavioral execution.

```sh
cargo test --locked --offline --lib mandatory_breaks_preserve_crlf_and_all_blank_paragraphs
cargo test --locked --offline --lib whitespace_is_preserved_and_contributes_logical_advance_not_ink
cargo test --locked --offline --lib heading_instructions_are_role_local_order_independent_and_frozen
cargo test --locked --offline --lib unbreakable_category_fails_intact_and_cannot_rotate_or_drop
cargo test --locked --offline --lib parent_width_keeps_natural_category_text_or_wraps_only_at_legal_boundaries
cargo test --locked --offline --lib canonical_label_fit_accepts_touching_ink_and_rejects_one_quantum_beyond
cargo test --locked --offline --lib frozen_text_and_brackets_never_relayout_or_convert_during_count_replay_and_outline
cargo test --locked --offline --test rendering labels::direct_plot_humanizes_axes_categories_legend_title_and_entries
cargo test --locked --offline --test rendering labels::raw_mode_preserves_automatic_identity_text
cargo test --locked --offline --test rendering labels::ambiguous_display_labels_fail_during_preflight
cargo test --locked --offline --test annotations panel_text::panel_text_renders_in_named_clipped_layer_with_theme_style_and_corner_anchors
cargo test --locked --offline --test composition canvases::text_panel_content_stays_top_left_when_its_fill_cell_grows
cargo test --locked --offline --test composition foundations::active_custom_font_roles_are_required_while_unused_preferences_are_not
cargo test --locked --offline --lib overlapping_spans_are_canonical_and_infeasible_total_misses
cargo test --locked --offline --lib declared_sparse_tracks_remain_noncollapsing_while_empty_slots_stay_zero
cargo test --locked --offline --lib canvas_gutter_remeasurement_uses_the_assigned_projection
cargo test --locked --offline --lib shared_composition_uses_parent_policy_in_both_source_orders
cargo test --locked --offline --test rendering contract::canvas_text_cells_enforce_occupancy_and_fitting
cargo test --locked --offline --test composition canvases::continuous_shared_heatmaps_draw_one_merged_guide_without_local_colorbars
cargo test --locked --offline --test composition canvases::composed_row_title_spans_all_columns_and_rejects_panel_titles
cargo test --locked --offline --test composition canvases::composed_horizontal_bars_support_shared_headers_labels_and_range_groups
cargo test --locked --offline --test composition canvases::shared_category_labels_compare_rendered_explicit_orders
cargo test --locked --offline --test output multiformat::plot_and_canvas_svg_operations_keep_complete_text_and_call_local_replay
cargo test --locked --offline --lib layout::kernel::tests::
cargo test --locked --offline --lib layout::interior::sizing::tests::
cargo test --locked --offline --lib canvas::sizing::tests::
cargo test --locked --offline --lib zero_sizing_requests_refuse_before_font_discovery_or_backend_begin
cargo test --locked --offline --lib standalone_terminal_resolves_and_plans_once_with_one_final_lifecycle
cargo test --locked --offline --lib chained_terminals_reuse_one_canonical_scene_and_plan_each_png
cargo test --locked --offline --test layout root_sizing::standalone_sizing_calls_replace_the_complete_request_before_frozen_publication
cargo test --locked --offline --test layout root_sizing::automatic_roots_expand_for_complete_unbreakable_text_and_exact_width_refuses_it
cargo test --locked --offline --test layout root_sizing::owning_facets_keep_sparse_wrap_and_complete_prose_in_all_public_root_modes
cargo test --locked --offline --test layout root_sizing::matrix_roots_keep_native_cells_and_their_hue_guide_in_all_public_root_modes
cargo test --locked --offline --test layout root_sizing::mixed_canvas_owns_all_modes_and_ignores_standalone_child_requests
cargo test --locked --offline --test layout root_sizing::all_text_canvas_reflows_complete_content_and_replaces_root_requests
cargo test --locked --offline --test composition facets::faceted_shared_axis_titles_preserve_explicit_multiline_text
cargo test --locked --offline --test composition facets::shared_facet_axis_titles_clear_outside_legends
just fmt-check
just lint
```

Run the newly written/replacement text, rotation, span and maximum-width witnesses
as well as affected existing callers; the commands above are not substitutes for
new-contract coverage.
Select existing guide/cache neighbors only when their dependency paths change or
a failure exposes an interaction. Execute affected public doctests and `just docs`
when adding API documentation/examples. Do not run the heavyweight large-SVG/PNG
boundary workloads merely because they share the root-sizing test file.

No automatic full-suite gate is required for these bounded features. Expand beyond
the focused set only for a demonstrated uncovered invariant or an explicit request,
following [development](90_DEVELOPMENT.md). Documentation-only planning requires
source/reference consistency and `git diff --check`, not compiler/test execution.
