# Public API Naming, Text, Axis Controls And Composed Layout

## Scope And Target Contracts

Implement the requested capabilities through one numeric/text, Grid and root
lifecycle:

- Remove the `with_` prefix from public configuration methods. On the same type,
  a property with both reading and writing uses `property(value)` and
  `get_property()`; a read-only or write-only property keeps its bare name.
- Extend NumericFormat with explicit locale/grouping/currency conventions, general
  display multipliers and rounding, configurable SI/compact presentation, shared
  multiplier/unit explanations, D3-style numeric format strings and single-value/
  complete-set formatting functions that retain automatic tick generation.
- Extend TickGeneration with exact counts, data-interval density, anchored
  multiplicative intervals and position-generation functions. Add explicit
  categorical-label stride/offset/endpoint selection without removing categories,
  marks, tick positions or grid geometry.
- Consistent plain-text inputs, complete instance typography and horizontal
  alignment, including independently configured TextPanel title/body text.
- One shared text-layout policy for general anchoring, fixed/data-relative rotation
  and explicit automatic tick angles, general ellipsis and configurable font fit.
  Reuse the complete typography value and one measurement/rendering path, including
  public single-label presentation and independently configured authored/generated roles.
- Explicit rectangular Canvas Plot spans, including a 2x2 Grid with two independent
  left panels and one right panel spanning both rows.
- `Plot::max_width(u32)` and `Canvas::max_width(u32)` as a fourth root sizing mode:
  maximum outer CSS-pixel width with content-derived height. The owning Plot request
  also controls its facets.

This document is a plan, not evidence that these APIs exist. Current contracts
remain in [rendering](02_RENDERING_CONTRACT.md), [Theme](03_THEME_SYSTEM.md),
[layout](05_LAYOUT_SYSTEM.md) and [numeric presentation](06_NUMERIC_PRESENTATION.md).

The [public API appendix](92_PUBLIC_API_PLAN.md) completes this plan's concrete
names, signatures, value variants, defaults and migration types. This document
owns behavior, implementation order and verification; implement both together.
The following map connects each public change to its complete signature inventory:

| Public change | Signature owner | Delivery steps |
| --- | --- | --- |
| Property naming and retained reads | [Migration summary](92_PUBLIC_API_PLAN.md#migration-summary) and the naming rule below | 1 |
| Text inputs, appearance, placement and adaptation | [Text](92_PUBLIC_API_PLAN.md#text-inputs-and-appearance), [TextLayout](92_PUBLIC_API_PLAN.md#textlayout-and-layout-values), [role attachments](92_PUBLIC_API_PLAN.md#text-role-attachments-and-capabilities) | 2, 3, 16-19 |
| Built-in NumericFormat, locale, specifiers and common units | [Numeric policy](92_PUBLIC_API_PLAN.md#numeric-formatting-types-and-methods) | 5, 6 |
| Single/set numeric formatting functions | [Function signatures and read-only views](92_PUBLIC_API_PLAN.md#numeric-formatting-functions) | 4, 7; multiplicative operand extension in 11 |
| Numeric tick locators, counts and position functions | [TickGeneration](92_PUBLIC_API_PLAN.md#tick-generation-and-category-selection) | 8-11, 13 |
| Categorical-label selection | [Selection and sealed gates](92_PUBLIC_API_PLAN.md#category-selection-and-sealed-gates) | 12 |
| Rectangular Canvas placement | [Canvas placement](92_PUBLIC_API_PLAN.md#canvas-placement-and-root-requests) | 14, 15 |
| Maximum-width roots | [Root requests](92_PUBLIC_API_PLAN.md#canvas-placement-and-root-requests) | 20-22 |

### Reference-Backed Capabilities And Configurable Defaults

For the requested numeric/text capabilities, support the configuration space
established by existing plotting libraries. A familiar default is a convenience,
not permission to exclude their other supported choices. Cite the actual property
or behavior, distinguish capability parity from byte-for-byte imitation, and ask
the user about a proposed product rule with no established precedent. Do not turn
an implementation shortcut, existing storage type or local search constant into
a user-facing restriction. Explicit user decisions, including rejection of relative
tick density and internal-word emergency wrapping, remain controlling.

The following references establish the capabilities used in this plan:

| Capability | Checked precedent | Required configurable surface |
| --- | --- | --- |
| Data interval and origin | [Matplotlib MultipleLocator](https://matplotlib.org/stable/api/ticker_api.html#matplotlib.ticker.MultipleLocator), [Plotly dtick/tick0](https://plotly.com/python/reference/layout/xaxis/#layout-xaxis-dtick), [scales breaks_width](https://scales.r-lib.org/reference/breaks_width.html) | Data-unit interval plus a finite numeric origin; 0 is a default, not the only origin |
| Fixed count versus preferred count/interval | [Matplotlib LinearLocator/MaxNLocator/MultipleLocator](https://matplotlib.org/stable/api/ticker_api.html), [Qt Charts tickCount/tickAnchor/tickInterval](https://doc.qt.io/qt-6/qvalueaxis-qtcharts.html#tickCount-prop), current preferred intervals | Deliberate exact and adaptive requests, not an unconditional coarsening or failure rule for every request |
| Custom positions and empty axes | [ggplot2 breaks](https://ggplot2.tidyverse.org/reference/scale_continuous.html#arg-breaks), [Matplotlib FixedLocator/NullLocator](https://matplotlib.org/stable/api/ticker_api.html) | View-dependent position functions, explicit positions and valid no-tick output; separate source positions from label visibility |
| Logarithmic versus data-unit intervals | [Plotly dtick](https://plotly.com/python/reference/layout/xaxis/#layout-xaxis-dtick), [Matplotlib LogLocator](https://matplotlib.org/stable/api/ticker_api.html#matplotlib.ticker.LogLocator) | Raw additive spacing on a log projection and geometric/exponent spacing with configurable ratio/base |
| Label stride, overlap and boundary treatment | [Plotly ticklabelstep/ticklabeloverflow](https://plotly.com/python/reference/layout/xaxis/), [Vega labelOverlap/labelBound](https://vega.github.io/vega/docs/axes/) | Explicit label selection, show-overlap and optional overlap/boundary hiding while preserving ticks/data |
| More than a fixed global tick threshold | [Matplotlib Locator.raise_if_exceeds](https://matplotlib.org/stable/api/ticker_api.html#matplotlib.ticker.Locator.raise_if_exceeds) documents warning rather than a hard limit | Caller-selected maximum or no application-wide maximum for explicit sources; real representation/allocation checks remain |
| Text anchors, coordinates and offsets | [Plotly annotations](https://plotly.com/python/reference/layout/annotations/), [Matplotlib Text](https://matplotlib.org/stable/api/text_api.html), [ggfittext placement/justification](https://wilkox.org/ggfittext/reference/geom_fit_text.html), [ggplot2 numeric justification implementation](https://github.com/tidyverse/ggplot2/blob/v4.0.3/R/geom-text.R) | Independent horizontal/vertical or baseline anchors, fractional justification, typed data/panel/figure references and configurable offsets |
| Rotation placement | [Matplotlib rotation_mode](https://matplotlib.org/stable/gallery/text_labels_and_annotations/demo_text_rotation_mode.html) and [transform_rotates_text](https://matplotlib.org/stable/api/text_api.html#matplotlib.text.Text.set_transform_rotates_text), [Vega text angle/dx/dy](https://vega.github.io/vega/docs/marks/text/), [Plotly autotickangles](https://plotly.com/python/reference/layout/xaxis/#layout-xaxis-autotickangles) | Rotate-about-anchor and align-after-rotation, screen/text-local offsets, screen/data-relative directions and explicit automatic tick-angle candidates |
| Ellipsis and text bounds | [Vega limit/ellipsis/dir](https://vega.github.io/vega/docs/marks/text/), [ECharts overflow/ellipsis](https://github.com/apache/echarts-doc/blob/master/en/option/partial/text-style.md), [Qt text elision](https://doc.qt.io/qt-6/qfontmetrics.html#elidedText) | Configurable marker, direction, length/line bounds and overflow handling; Qt is supporting text-stack evidence for left/middle/right elision, not a claim about a Qt Charts API |
| Font fit and grouping | [ggfittext min.size/grow/reflow/outside](https://wilkox.org/ggfittext/reference/geom_fit_text.html), [Plotly uniformtext](https://plotly.com/python/text-and-annotations/#Controlling-Text-Size-with-uniformtext) | Configurable size bounds, reflow, independent/uniform fitting and explicit hide/show/outside handling at the size floor |
| Fractional text size | [Vega fontSize](https://vega.github.io/vega/docs/marks/text/), [Matplotlib fontsize](https://matplotlib.org/stable/api/text_api.html#matplotlib.text.Text.set_fontsize) | Positive fractional CSS sizes throughout measurement and output, not an integer-only fit grid |

These sources support capabilities, not every combined default. The user has
confirmed two combinations: data intervals use configurable numeric origins, without
a separate origin-following-the-view mode; shrink and ellipsis support both content-
first and preferred-size-first ordering, with content-first as the default. Any
further unsupported product choice needs a concrete workflow and user confirmation
before it is written as a requirement. Position functions have no additional
correctness-validation stage or strictness mode: drawable results proceed through
normal window/geometry handling, and a failed operation or explicit caller
constraint reports its error. Numerical proofs, finite algorithms and
the existing standalone/frozen-output contract remain implementation obligations;
they do not themselves establish a new truncation, visibility or failure policy.

### Public Property Names And Validation

Apply the confirmed rule per concrete owning type and semantic property, including
macro-generated methods and public generic implementations:

| Surface | Planned write | Planned read |
| --- | --- | --- |
| TextTheme font size | `font_size(value)` | `get_font_size()` |
| NumericFormat precision | `precision(value)` | `get_precision()` |
| ThemeBuilder typography | `typography(value)` | `get_typography()` |
| Theme typography, which has no corresponding edit on Theme itself | None | `typography()` |
| Plot width | `width(value)` | None |
| Output width | None | `width()` |

Different types do not cause a collision: Plot's width configuration and Output's
width query both use bare names. Predicate and collection operations such as
`is_empty`, `is_theme`, `len`, indexed `get` and keyed lookup retain their semantic
names when there is no competing property edit. Constructors and conversions are
not mechanically renamed into getters. A future same-type property edit must
apply the rule to its existing reader in that logical migration.
If the new public writer name collides with a private property reader, rename
that reader too while retaining its visibility; do not expose it to resolve a
method-namespace conflict.

Keep existing public read capabilities during this naming change. The usage
assessment does not authorize deleting getters, adding getters for every setter,
exposing writable fields or introducing a read-only-view/builder hierarchy.
Preserve value types, borrowed/copy return contracts, consuming receiver style,
fallibility and existing validation timing. In particular:

- TextTheme edits continue through its complete constructor/validator; NumericFormat
  edits continue through its complete policy validator. Nested component edits
  retain sibling/cross-field checks, including ones generated by macros.
- ThemeBuilder's complete component replacements keep their validated-input
  contract; `build()` and theme resolution keep their existing complete checks.
- Plot/Canvas instructions already validated during preflight, such as zero root
  dimensions, retain that lifecycle. Renaming does not move checks to a new phase.
- Private storage continues to prevent direct assignment from bypassing those
  construction checks. No delayed-invalid draft value is introduced by naming.

Inventory each public `with_*` declaration, its property reader and repository
callers. Migrate declaration macros, inherent methods, function-item references,
production calls, unit/integration tests, examples, ordinary/compile-fail doctests
and documentation together. Remove the replaced names rather than adding aliases,
deprecated methods or a second spelling. Internal helpers need renaming when they
refer to the replaced API; this is not a blanket ban on English `with` inside
unrelated method names such as `png_with_options`.

Primary owners are `src/theme/value.rs`, `src/theme/component.rs`,
`src/theme/typography.rs`, `src/theme/composition.rs`, `src/theme/axis.rs`,
`src/theme/guide.rs`, `src/theme/mark.rs`, `src/theme/mark_paint.rs`,
`src/theme/paint.rs`, `src/theme/resolved.rs`, `src/theme/scale_values.rs`,
`src/numeric/format.rs`, `src/layers.rs`, `src/spec.rs` and `src/spec/`.
Apply the same names to the newly planned Text/numeric APIs below. Current API
contracts and user guides use the implemented property naming; later capability
steps migrate their planned types and behavior with the receiving consumers.

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
- Text follows its explicit wrapping/overflow/visibility/font-fit policy; scientific
  sources, eligible labels, annotation identities and guide sections remain retained.
  Preserve mark metrics and declared spacing ownership. Width capping does not
  itself enable font shrink, abbreviation, padding compression or root-width search.
- Output reports the actual selected canonical width/height. SVG modes and PNG
  options replay that frozen scene; PNG scaling does not rerun root layout.

This scope does not admit height-only sizing, minimum dimensions, maximum height,
public generic constraints, nested Canvas objects or implicit topology changes.
Component-box-responsive spacing is outside this plan.

## Existing Owners And Required Changes

The tables and details here cover Canvas/root sizing. The shared text/presentation
requirements and their owners are specified in the following section.

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

## Unified Text, Placement And Adaptive Fit

### Confirmed Text Boundaries

Keep ordinary Unicode legal wrapping and caller-authored mandatory line breaks.
Do not introduce emergency word/identifier/grapheme splitting or a caller-supplied
internal-break rule. A name such as `LightGBM` never acquires internal line breaks
to fit. Under complete-text policy it stays intact when passed as explicit or raw
display text, even when a narrow allocation cannot fit it; explicit ellipsis below
derives a separate abbreviated view without changing that source.

Keep the current default `Labels::humanized()` for automatically sourced names.
Explicit title/axis-label/TextPanel strings remain verbatim. Humanization changes
display text during label preparation, independently of fit; it is never invoked
because a string failed to fit. Preserve `Labels::raw()`, alias precedence, raw
scientific identities and domain-level display-collision checks.

Omission retains the owner's existing orientation and complete-text behavior.
The caller can select a fixed angle or an explicit automatic tick-angle policy,
authored newlines, legal soft wrapping or no soft wrapping, and overflow/font-fit
instructions. No adaptation activates merely because a complete default failed.
Label hiding/reduction requires its own explicit policy and never removes data or
tick positions. Preserve Heatmap's existing omitted Elide and explicit Wrap
semantics as configurable owner defaults within the shared engine.

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
  optional `TextLayout` request for placement and adaptation.
- Plain `String`/`&str` convert into this same representation without local style
  instructions. Relevant free-form setters/constructors accept `impl Into<Text>`;
  strings are the ergonomic ordinary input to one implementation route.
- `Text::new(...)`, `style(TextTheme)`, `layout(TextLayout)` and the existing planned
  `alignment(TextAlignment)` convenience edit give x/y titles, prose and TextPanel
  content the same construction model. Alignment edits the one layout instruction;
  it is not separately stored. TextPanel title/body retain independent values.
- Reuse the complete `TextTheme` for stack, RGBA, positive CSS font size, closed
  weight and positive line-height multiplier. The renamed `font_size`, `weight`,
  `color`, `stack` and `line_height` edits operate on that one appearance type;
  paired public readers use `get_`. An exact style replaces the complete role;
  do not introduce a parallel sparse font-style cascade or a backend-style public API.
  The appendix inventories all receiving slots and preserved `&str` readers.
  Retain Eq on existing text-bearing annotations/TextPanel by complete normalized
  appearance/request equality, not by discarding their newly retained style.
- Replace public `TitleAlignment` with shared `TextAlignment` and migrate all
  repository callers. Start/Center/End retain physical left/center/right meaning,
  including for RTL text. Remove the old type/export rather than adding an alias.
- Authored prose, x/y titles, TextPanel, headers/row titles and panel annotations
  accept the common instance value. Exact ticks and automatic/generated text use
  common validation and typed role-level TextLayout attachments below. Styling
  functions and rich-text parsing are not part of the layout policy.

Each text-bearing slot retains source and local instructions once. Existing
heading-alignment convenience setters edit that slot's one layout instruction;
they do not create a second precedence channel. Preserve content-only setter
behavior: a string replacement does not erase an explicit heading alignment.
An exact appearance edit replaces the whole exact style. A supplied Text's present
layout request replaces the whole prior request, resetting omitted fields to owner
defaults; an absent request preserves existing slot instructions. Explicit
`layout(TextLayout::default())` is a present reset, distinguishable from plain
content conversion. Convenience edits change a field of the one current request;
attaching that Text supplies its resulting whole request. Apply these operations
in call order without a second partial-layout cascade.
Theme attachment order remains independent of these local instructions.

Style resolution is local exact style over the selected owner's complete Theme
role. Resolve before font census and measurement; omitted appearance uses the
existing Plot/Canvas ownership and role defaults. Existing Canvas panel heading
defaults remain below explicit child instructions. TextPanel alignments default
to Start independently. X and Y titles can now have distinct local styles despite
sharing the Theme axis-title default.

Every line consumes physical alignment within its measured block. With no explicit
anchor, preserve the owner's existing block-alignment/placement behavior over the
post-inset region. For an axis title, line alignment is resolved before its selected
transform. Container assignment, reference locations and scientific projection
remain parent/semantic geometry. Font-dependent spacing consumes its declared
role's final selected metric: a local title size changes its title clearance,
while root padding still uses the root owner's body metric. Do not resolve gaps
against stale Theme sizes or mutate shared typography when one text owner shrinks.

Prepared text requirements must enumerate actual selected stacks/weights and
source text, including local styles. Intrinsics, constrained measurement, guide
compatibility, height caches, child reuse and frozen drawing consume the same
resolved inputs and alignment. Preserve actual ink/leading and complete equality;
equal strings or dimensions alone do not authorize reuse.

### One TextLayout Policy And Deliberate Attachments

Treat anchoring, orientation, overflow and font fit as one text-system
extension: public configuration expresses intent; the existing layout engine
resolves actual placement/content/size. TextTheme remains a complete deterministic
appearance with a preferred positive CSS font size. Do not put parent
coordinates, a shrink algorithm or mutable selected sizes into Theme components.

Expose an immutable crate-root TextLayout value with private instructions for
alignment, anchor, offset/frame, orientation/rotation placement, finite text bounds,
soft wrapping, overflow/overlap, font fit/grouping and adaptation priority. Reuse
TextAlignment and the fixed TextRotation value; add validated layout values through
the same private text module. All construction/edits follow the naming rule.
TextLayout defaults mean the actual owner's existing defaults, including axis
orientation and Heatmap elision, not a universal horizontal/unwrapped placement.
Retain omitted versus explicitly selected defaults until owner resolution.
Preserve whole-request presence separately from its per-field owner defaults;
otherwise a plain string and an explicit reset would become indistinguishable.
Resolve existing exact annotation edits, including SignificanceMarkStyle and
PointLayer paint, into effective appearance first. Font fit consumes that preferred
size without changing the edit's field ownership or scaling explicit pixel gaps.

`Text::layout(value)` attaches the policy to authored content. Convenience edits
write that same request. Automatic text, exact tick strings and labels supplied
as data columns receive the same policy through typed owning-slot/group editors;
do not require callers to replace all labels with independently styled Text values.
Add fields only with working consumers and migrate every affected caller when
replacing a project-owned surface.

| Text owner | Shared policy attachment and boundary |
| --- | --- |
| Plot/Canvas title, subtitle, caption; x/y titles; TextPanel title/body; headers/row titles | The existing common Text input owns its layout; title/body and x/y remain independent |
| Numeric/category/exact axis tick labels | Axis-specific `x_tick_text_layout`/`y_tick_text_layout` on sealed actual-axis capabilities, stored once per axis; no separate numeric/category rotation engine |
| Facet strips and Matrix variable titles | Typed header-role instructions, with actual local/shared owner assignment; Matrix's actual left-edge/bottom-edge variable titles do not become categorical ticks or gain numeric format/generator attachments |
| Discrete/continuous guide titles, entries and numbers | Typed guide-owner role instructions; Canvas Shared chrome resolves Canvas-owned policy, independently of replaced local guide layouts |
| PanelTextAnnotation and point-layer labels | Common Text for authored annotations and a layer-label TextLayout for column-backed labels; the semantic owner retains corner/data reference, occurrence and paint rules |
| Heatmap values/null-row text, estimate details, Waterfall totals and significance labels | Typed existing annotation-role instructions, retaining required sets, contrast-role selection, shelves/bracket paths and scientific anchors |

Enumerate those real consumers during implementation rather than add a string-keyed
role registry, per-value style callback or backend handle. An instruction must be
consumed coherently or rejected with actual-owner context; it cannot silently do
nothing on a supported text slot. Erased plots retain supplied policy but do not
gain broad setters through Any.
Use the appendix's exact role methods and sealed `XAxisText`, `YAxisText` and
`EstimateDetailText` gates. Matrix's KDE diagonal does not produce a separate
heading, so this plan adds no unused diagonal-text method or new text producer.
Add positive fractional CSS font sizes to the complete appearance and resolved
text pipeline; migrate integer-only font/metric storage and callers together.
Use the existing canonical spatial representation for actual scaled geometry,
without rounding a permitted 12.5px request to 12px before measurement. Font sizes
remain appearance inputs; selected fit sizes belong only to resolved plans.

Replace the earlier unpublished category-only rotation setter proposal with the
general axis TextLayout attachment. Category-label selection keeps its categorical
capabilities and remains independent of text layout. Native categorical families
include conditional Trajectory x, Interaction, Heatmap, Bar, StackedBar, Boxplot,
Violin and Raincloud on x, and PointInterval, Coefficient, Forest, Dumbbell,
Heatmap, HorizontalBar, StackedHorizontalBar, Beeswarm and Waterfall on y.
General axis text layout also serves actual numeric/exact axes. Preserve explicit
policy applicability and caller-selected visibility, not implicit omission.

### General Anchoring: A Label Attached By Its Bottom-Left Corner

Workflow: a two-line point label keeps its bottom-left corner at its projected
data reference as its content or size changes. Separately, a TextPanel body can
remain centered within an enlarged assigned cell. Line alignment still controls
the individual lines inside that positioned block.

Use one TextAnchor representation for independent horizontal/vertical presets,
including the nine physical box combinations and first/center baseline references
established by Matplotlib/Vega. Also support finite fractional block justification,
with (0,0) top-left and (1,1) bottom-right. ggplot2's numeric justification passes
through without a 0..1 clamp; preserve that capability with checked geometry.
Preset/custom-equivalent anchors normalize identically. Baseline references use
actual measured metrics. Normalize signed zero and retain checked arithmetic and applicability.

An anchor selects a point of the measured block rectangle, not the clipped ink or
the first glyph. The owner supplies the reference: corresponding post-inset cell
point for box placement, projected coordinate for a point label, or its existing
tick/title/corner reference. Authored annotations additionally accept typed data,
panel-fraction and owning figure/Canvas references, with independent x/y reference
kinds where meaningful, as in Plotly. Extend the existing annotation representation
rather than add a parallel generic drawing callback. A PanelCorner constructor
remains a convenience for that semantic reference; strings still use common Text.
Use raw category/series identities for categorical references and the real numeric
transform for data references. Explicit limits/facet/Canvas assignment resolve
these references before text placement, with source provenance retained.

A finite signed CSS offset can be selected in physical screen coordinates or
text-local coordinates that rotate with the block, following Plotly shifts and
Vega dx/dy. Omitted offset retains the existing owner translation. Explicit offset
and its selected reference frame have one resolved meaning, independent of style
attachment; parent padding is not applied twice.
Thus an explicit zero offset can attach directly to a point without changing its
scientific coordinates. Text never carries an untyped interchangeable data/CSS
coordinate pair.

In rotate-about-anchor mode, a screen offset places a local point p using
`reference + offset + R(angle) * (p - block_anchor)`; a text-local offset is inside
the rotation. In align-after-rotation mode, align the transformed block bounds to
the reference instead. Both are established rotation placement behaviors, not
independent rendering pipelines. Round checked canonical
positions consistently with the shared geometry engine; convert the internal y-up
frame exactly once. Keep default middle/baseline and existing axis multiline
ordering when instructions are omitted;
replace their scattered placement arithmetic with one resolved transform rather
than retain independent legacy and general-anchor paths.

Fit uses transformed logical extents, actual ink and the explicit overflow/overlap
policy below. Natural/required geometry includes offsets/overhangs, or the actual
clip/hidden result when requested. PanelCorner identifies a panel reference,
distinct from the text block anchor. Update PanelTextPlan reassignment and guide/
annotation collision dependencies together with drawing; an allow-overlap request
must not be rejected by a stale unconditional rectangle-collision check.

### Public Single-Label Presentation

Promote the existing `Labels::resolve(&self, raw: &str) -> Result<String, Error>`
to a documented public method and reuse its current implementation. This gives
`Labels::humanized().resolve("long_sample_name")?` to explicit-text callers and
also honors Raw/aliases through the same policy. Do not introduce a second public
humanizer or copy the tokenizer. Document that it may change case/separators and
acronyms, returns ordinary display text, and never inserts fit-dependent breaks.
The single-label method has no domain collision context; `resolve_domain` remains
responsible for detecting collisions among distinct raw identities.

### General Explicit Rotation Through One Value

Expose one validated `TextRotation` value with horizontal, upward/downward 45-degree
tilts and upward/downward vertical presets, plus `degrees(f64) -> Result<Self>`.
Factories can present enum-like named choices, but their retained result is one
canonical angle. Preset 45 degrees and custom 45 degrees have identical geometry
and cache identity. Reject nonfinite inputs, normalize full turns/signed zero and
resolve the common retained angle before fit measurement.

Angles are degrees from physical horizontal in the top-down output frame, positive
clockwise. The upward/downward tilt presets are -45/+45, vertical presets -90/+90
and horizontal 0. Normalize equivalent turns to a single range,
with one representative for the half-turn endpoint and zero.

The current canonical output stores tenths of a degree in
`src/render/coordinates.rs`. Replace that representation when exposing general
rotation so a finite custom angle such as 30.05 degrees is not silently restricted
to the old tenth-degree grid. Normalize a single retained angle, serialize it
without a second lossy output-only quantization and use it throughout measurement,
collision, capture, SVG text, outlines and PNG. Keep exact quadrantal
transforms and a shared sine/cosine operand for other angles so geometry and
outlined rendering cannot disagree at the same frozen angle.

Workflow: rotate a point annotation by 30 degrees, give the y title an explicit
horizontal angle, or tilt numeric/exact as well as categorical tick labels. Attach
the same TextRotation through TextLayout for every admitted role above. Angles
are absolute from physical horizontal: explicit 0 replaces an owner's vertical
default, while omission inherits it. Support explicitly data-relative directions
for data-anchored notes through a typed pair/direction reference, as in Matplotlib's
transform-relative text rotation; resolve it from the actual projection. It is
distinct from a screen-angle instruction and does not mutate scientific data.
Retain actual-axis capability checks, erasure and parent-owned preparation; color
scale traits do not stand in for text/axis capabilities.

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
region. Retain the existing immutable-plan/typed-miss protocol; fixed-angle fitting
does not search other angles, arbitrary word splits, root widths or all wrap partitions.
Unbreakable text can fit through rotation without first fitting the horizontal
cell width. Use that same fixed-angle capacity/containment predicate for all
permitted overflow/font-size candidates. A fixed angle never becomes an automatic
one. Separately support caller-enabled automatic tick orientation with a supplied
ordered finite candidate list, as in Plotly autotickangles. Try the caller's order
under the same real text-fit predicate, retaining the chosen angle and all consumed
dependencies; do not invent emergency word breaks or a root-wide angle optimizer.

Feed actual transformed thickness/overhangs into axis gutters, heading/strip/guide
tracks, TextPanel height and annotation/shelf ink as appropriate. Preserve tick and
mark positions, semantic order, category membership and linked projections. Update
natural preferences and native Heatmap/Beeswarm/Waterfall assigned-width producers.
Resolve multiline ordering once, so explicit horizontal y titles do not accidentally
inherit the old vertical column reversal. Derive preserved role-specific ordering
from the resolved orientation, not preset/custom spelling or request presence;
an explicit angle equal to the inherited angle has the same geometry. Existing
root rounds and guide alternatives remain the containing program, not a loop over
unrequested angles or whole-figure layouts.

### Explicit Overlap, Bounds And Visibility Policies

Support strict containment/nonoverlap, deliberate overlap, finite owner clipping,
and local overflow with full final-root containment. Plotly annotations/overflow,
Vega labelOverlap/labelBound and ggfittext floor handling establish that failure is
not the only available behavior. Defaults retain existing owners' strict behavior
unless an already documented owner default applies; explicit policies can select
the other behaviors. The physical output canvas remains a real final bound.

Axis label visibility is independent from numeric positions/grid lines and category
topology. Support all eligible labels, explicit stride/selection, show-overlap,
greedy/parity overlap reduction, and optional boundary-label hiding with configurable
clearance/tolerance. Apply an explicit category selection first, then an explicitly
requested overlap policy; retain both eligible and finally visible original-index
sets. Endpoint inclusion/priority is explicit, not guessed from the current count.
Greedy/parity also admit configurable minimum label separation, as in Vega
labelSeparation; zero is their initial extra gap, not the only allowed value.
An enabled visibility policy changes painted labels, never scientific membership,
numeric tick cardinality, generator interval or the formatter's semantic result.

Known finite text width/height and line limits are independent caller constraints,
not hidden defaults tied to the root size. Soft wrapping is independently selectable
between ordinary legal wrapping and no soft wraps; authored mandatory breaks remain.
At a font floor or ellipsis failure, explicit choices include typed refusal, hiding
that text, clipping, or drawing beyond its local box within its parent's allowed
region. Retain original text/accessibility/source identity and actual selected
clip/visibility in the frozen plan. A deliberate hidden/overlapped/clipped label
must not be passed off as a complete nonoverlapping label in tests or diagnostics.

Failures of required source interpretation, consumed-font coverage, scientific
transforms and real arithmetic/allocation remain errors at their actual operations.
Permission to hide, overlap or clip text does not make an unsuccessful consumed
operation succeed or change scientific coordinates.

### General Ellipsis: A Fixed-Width Guide Entry

Workflow: a fixed-width guide displays `Treatment group…` for
`Treatment group with extended follow-up`, while the original string and semantic
identity remain available. A caller can instead retain the suffix of a long sample
name or its two identifying ends. This is explicit display abbreviation, not a
numeric/category transformation or a source-deleting fit alternative.

Alongside complete/clip/visible/hide overflow, support start/middle/end ellipsis,
configurable marker, pixel bounds and a configurable line cap. U+2026 and one line
are defaults for an explicit ordinary ellipsis request, not restrictions. Permit
an explicitly empty marker for markerless truncation as well as a custom literal;
validate common display syntax and single-line marker applicability. Resolve
automatic/LTR/RTL paragraph direction and distinguish logical start/end from
physical left/right elision, following Vega/Qt. Do not assume that an RTL prefix
is always at the physical left. Alignment is independent.

Keep original bytes and full-source validation/domain-collision checks. Derive a
display view from original extended-grapheme spans plus a synthetic marker. End
keeps a prefix, Start a suffix and Middle retains both ends; use actual measured
capacities rather than hard-code an equal retained-grapheme split as the only rule.
Retain mandatory breaks within selected spans and enforce the declared line cap;
removed paragraphs are part of the explicit abbreviation. Never split an extended
grapheme, erase controls by byte slicing, trim whitespace or insert an illegal
wrap inside `LightGBM`. Explicit elision may abbreviate an identifier at a grapheme
boundary; it does not authorize internal-word line breaking.

Try the complete source first under the line/geometry constraints. When elision
is needed, use retained paragraph/cluster advances to propose a retained count,
then exact-shape/measure the marked candidate. Retreat to a fit and extend through
immediately neighboring retained-count candidates until the first nonfit, as a
source-bounded local program. Middle uses a bounded two-ended measured proposal,
not a prefix x suffix Cartesian search. Do not repeatedly rebuild/retain every
full prefix or claim the globally longest possible display string. A marker-only
view is a real abbreviation; markerless or hidden output is possible only through
the selected explicit policy. If the marker cannot fit, consume the configured
refusal/hide/clip/overflow action instead of impose one universal error rule.
Empty source retains its owner's existing empty-text semantics and never invents
a marker. In particular, exact empty tick labels keep zero text/gap/gutter while
retaining their position; authored empty blocks retain their primary line metrics.

Retain original paragraph bidi/isolate context and source mappings for kept spans;
give synthetic marker runs explicit provenance/context and shape their actual
neighbors. Selected soft hyphens and contextual substitutions use the display
view's real boundaries. Keep original and derived source addresses distinct in
ResolvedTextPlan, canonical text runs and SVG emission; do not assume every glyph
cluster is an offset into one unmodified original string.

Keep full-source semantic/domain validation before deriving views. Permit repeated
displayed abbreviations as in ordinary library text/label outputs, and support an
explicit distinct-display requirement for consumers that need it. Preserve current
Heatmap distinctness as its omitted owner default, not a universal truncation rule.
If distinctness is requested, a collision follows the selected failure policy;
do not invent alias rewriting or automatic distinguishing suffixes. Exact caller
labels retain their duplicate allowance.
NumericFormat still produces the complete final numeric spelling once per semantic
context; explicit text overflow derives a view of that result without parsing,
rerounding or reinvoking a formatting function. Shared unit explanations retain
their required slot and original text even if the caller explicitly abbreviates it.

Migrate Heatmap `fitted_label`/`fitted_unique_labels` to this one engine. Preserve its
omitted dense-grid End/`...` owner default, original-cell capacities and current
distinctness/refusal defaults; its existing Wrap maps to complete legal wrapping.
Replace HeatmapLabelFit/its setter with the common x/y TextLayout attachments and
migrate repository callers/docs in the same logical change. The owner-default
marker is an ordinary policy value, not a legacy formatter or fallback chain.
Selected-label metadata and full tick/grid geometry remain independent.

### Configurable Font Fit: Preferred 14px With A 10px Floor

Workflow: a 140px title allocation prefers 14px but explicitly allows shrinking
to 10px. A measured choice of 12px retains that size and its real geometry; a
permitted fitting fractional size is not excluded by an integer sampling grid.
Another title/body can keep a fixed size or choose a different floor. Complete
text with ordinary wrapping remains preferred when its actual width/height permit
it; automatic-height roots do not invent a finite height to force smaller text.

TextFit selects fixed preferred size, shrink-to-floor, or explicit bounded growth
and shrink, following ggfittext. Preferred appearance still comes from TextTheme;
minimum/maximum are positive CSS sizes, including fractional sizes. Validate the
resolved bounds and normalize an equal-size interval to fixed. Growth requires a
meaningful finite box or upper bound; an auto-height/root preference cannot invent
an infinite target. Keep stack, weight, paint and declared leading ownership.

Select a measured size within the requested bounds using retained design/shape
inputs and a justified finite local algorithm. Recheck actual wrapping/ink at
accepted sizes, include the real floor and upper boundary, and do not assume that
arbitrary size-dependent wrapping is globally
monotone. Source/metric-derived proposals and refinement need independent boundary
and termination evidence before publication; an arbitrary sample count cannot
exclude the user's permitted sizes or turn an untested fit into proven nonfit.
No promise of a global whole-figure optimum or new application resource quota is
introduced by this local font-fit capability.

Support both user-confirmed combination priorities. Content-first tries complete
text with permitted fitting before marked views; preferred-size-first tries
permitted abbreviation at the preferred size before reducing it. Content-first
is the default, and the request retains its explicit priority through all caches.
Reflow versus no soft reflow is selectable, with authored breaks preserved. A
floor nonfit consumes explicit refusal/hide/clip/local-overflow handling, as in
ggfittext/Plotly, rather than force one behavior for every text owner. Font,
arithmetic, allocation and shaping errors remain fatal instead of fit alternatives.

Support independent per-label sizes and uniform owner/group sizing, as established
by Plotly and ggfittext. Uniform fitting selects one permitted size for its full
active/eligible set; independent mode can fit members separately. Each policy
retains group identity and all resulting sizes/visibility without changing data,
ticks or raw category selection. Unrelated roles, including contrast appearances,
do not become coupled merely because initial sizes are equal.

At each candidate recompute exact scaled origins, outward ink, line pitch,
wrapping, anchor and transformed containment from retained design/font inputs.
Do not scale a rendered SVG group or reflow analysis with a mismatched font-size
profile. Release obsolete failed glyph payloads; use existing source-sized maps
and fallible storage, not all candidate scenes or a second paragraph-analysis
framework. Source/formatter functions do not run again for different font trials.

Natural preference uses complete content at preferred size. Hard requirements use
only explicitly admitted size/overflow/visibility rules; preserve real required
geometry and actual clip/ink proof for the chosen policy. Selected text metrics
update only their declared gaps/clearance and height/gutters, not unrelated mark
metrics or root padding. Existing Automatic geometry seeds use resolved preferred tick
typography; fixed data-interval positions depend only on step, origin and final domain.
Fitting a source at a smaller size does not request more ticks within that trial.
Real assignment/domain changes still flow through the existing bounded coordinator.

### Text Dependencies, Font Admission And Frozen Publication

All four controls execute during intrinsic/assigned measurement, before capture.
Retain original source, normalized request/default provenance, exact appearance,
reference frame/offset, anchor/orientation/rotation mode, bounds/line cap, overlap/
visibility/priority/grouping and selected sizes, display spans/marker, baselines and
ink in their consuming dependencies. Separate low-level shape reuse from complete layout
identity: equal current glyphs/strings/widths do not establish upstream equivalence.

Extend the same actual-text font-admission protocol planned for numeric results.
Known complete sources needed for fit analysis are prepared normally; a wholly
unselected category label adds no analysis/font work. A selected source's missing
font is fatal even if an eventual elision would omit that glyph. Admit an actual
needed marker and its complete adjacent selection units before first shaping;
an intact fitting source does not require an unused custom marker font. Reuse the
same operation, stable pairs/face choices and final seal, not a separate text
operation or guessed abbreviation alphabet.

Measure one policy through Plot, owning facets, Shared/PerPlot guides and spanning
Canvas. Parent-assigned reference/geometry changes invalidate affected layout/
height/ink plans; independent siblings retain valid reuse. Maximum width keeps its
selected root bound and runs permitted local text policies within that assignment.
Instructions do not directly mutate scientific values, explicit range requests,
statistics, category topology or root mode. Visible-label changes follow only their
declared policy and remain separate from the eligible set. Existing authorized
linked display-domain/owned-ink feedback still honors parent provenance and the
eight-round/guide-recovery program; text adaptation does not reset that program.

Freeze selected sizes, visibility/clip, display view, source mappings and canonical
transform before capture. Text SVG paints the selected display once and retains
full original text as nonpainting accessibility metadata when abbreviated; outlines
and PNG use the same retained glyph/placement plan. Output replay performs no
shrink/elision search, shaping, font admission, locator or formatter invocation.

### Text Implementation Owners

| Owner | Change |
| --- | --- |
| `src/text.rs` (new private module), `src/lib.rs`, `src/labels.rs` | Shared Text/TextLayout and validated anchor/rotation/overflow/fit values, display validation, deliberate exports and single-label resolution |
| `src/theme/resolved.rs`, `src/theme/typography.rs`, `src/theme/composition.rs`, `src/theme/axis.rs` | Complete preferred appearance, selected local metrics, coherent shape/layout cache profiles and resolved text inputs |
| `src/builder.rs`, `src/builder/configuration.rs`, `src/builder/accessors.rs`, `src/builder/facets.rs`, `src/kind.rs` | One text slot/role policy, actual capabilities, parent defaults, erasure/attachment and complete dependency identity |
| `src/canvas.rs`, `src/canvas/text_panel.rs`, `src/canvas/plot_text.rs`, `src/decorations/title_block/plan.rs`, `src/decorations/legend/` | Owner assignments/defaults, independent text slots, Shared guide roles and width/height reserves |
| `src/layout/text.rs`, `src/layout/text/requirements.rs`, `src/font/canonical.rs`, `src/layout/fit.rs` | Grapheme/source-view planning, actual marker admission, bounded size/group fit and coherent intrinsic/constrained inputs |
| `src/layout/panel.rs`, `src/layout/panel/cells.rs`, `src/layout/panel/axis_frame_grid.rs`, `src/layout/interior/sizing.rs` | Full positions plus selected-label metadata, transformed capacities, selected gutters and preferred/hard requirements |
| `src/spec.rs`, `src/layers.rs`, `src/decorations/panel_text.rs`, `src/plots/line.rs`, `src/significance.rs` | Reference/anchor separation, point/corner/bracket text policy, exact paint rules and required ink/collisions |
| `src/orchestrator/facets/planning.rs`, `src/orchestrator/facets/sizing.rs`, `src/canvas/sizing.rs`, `src/canvas/tracks.rs` | Shared/local role policy, spanning assignments and width-first text-height dependencies |
| `src/plots/heatmap.rs`, `src/plots/beeswarm.rs`, `src/plots/waterfall.rs`, `src/plots/interval_estimate.rs`, `src/plots/matrix.rs` | One elision engine, native full-cell geometry, complete role/set fit and contrast/shelf/header ownership |
| `src/render/backend.rs`, `src/render/coordinates.rs`, `src/render/geometry.rs`, `src/render/primitives.rs`, `src/render/svg_backend.rs`, `src/orchestrator/backend.rs` | Shared anchor transform, canonical angle/ink, derived source/marker provenance and frozen SVG/outline/PNG publication |

## Extended Numeric Presentation

### Complete Requested Scope And Scientific Boundary

Implement the full requested numeric capabilities together through NumericFormat,
not a smaller formatter justified by the present closed-alphabet preparation model.
Structured construction remains supported alongside the chosen D3/Plotly-style
specifier syntax. Single-value and whole-set functions format generated positions;
users must not supply every AxisTick merely to customize automatic tick text.

Numeric presentation remains distinct from TickGeneration, statistics, scientific
values/projection, color mapping, raw identities and exact caller strings. Retain
the existing NumericX/NumericY/ContinuousColor and value-annotation attachment
boundaries, including actual categorical-axis validation. Position generation and
explicit categorical-label selection have their separate contracts below;
formatting does not own them. Matrix-wide format attachment and Canvas overrides
of child formats remain outside this scope. Existing exact ticks
suppress automatic numeric formatting, explanation preparation and callbacks in
either setter order; preserve the independent actual-axis semantic checks.

Cover each current numeric surface: generated x/y axes, continuous color-guide
numbers, Heatmap Values, point-only Coefficient labels and eligible Waterfall
base/terminal totals. Owning facets and Canvas Shared/PerPlot guides must consume
the same policy/result types. Named scientific/string annotations stay literal.

### One Policy And Exact Conversion Pipeline

Extend the existing normalized NumericFormat and mathematical operand machinery.
Retain one emission engine and one complete-set resolver. Parser output, typed
options and callback context helpers use those owners; there is no D3 formatter
running beside a structured formatter or a default-versus-custom fallback chain.

The built-in policy covers notation/radix, exact or adaptive precision, decimal
shift, general multiplier, rounding, sign/zero/trimming, literal affixes, locale,
grouping/currency, field padding/alignment, SI/compact selection, unit/explanation
policy and one selected emission mode. Keep equality over complete normalized
inputs, not current labels. Validate incompatible combinations at construction;
document each option's applicable emission grammar rather than silently ignore it.

- Preserve the exact meaning/range of the existing decimal-power display shift.
  Add a finite signed general multiplier, including zero, retaining its exact
  represented binary64 rational. The general multiplier and decimal shift compose
  multiplicatively; percent/per-mille and prefix/unit scaling contribute explicit
  exact factors. Form one exact effective operand before quantization, without
  intermediate binary64 multiplication, overflow/underflow or double rounding.
  Normalize multiplier metadata consistently, including signed zero.
- Ordinary inputs retain their exact binary64 value. Mathematical Forest operands
  remain `10^(exact exponent)` with exact display factors; extend the certified
  interval/cell proof for those factors and rounding directions. Do not first
  materialize `powf` or pass an approximate physical value to a formatter function.
- Default rounding remains nearest/ties-to-even. Add explicit nearest/ties-away
  and nearest/ties-toward-zero, plus toward-zero, away-from-zero, floor and ceiling.
  Define negative halfway cases and rounded negative zero independently. Carry,
  exponent changes, SI promotion and trimming never round the number again.
- Extend explicit fractional precision to 0..=20 and significant precision to
  1..=21 for the D3-style numeric range, including matching adaptive bounds. Invalid
  bounds reject rather than silently clamp. Extend the exact/certified conversion
  proofs and replace obsolete bound-refusal cases when this capability lands.
- Keep omitted consumer profiles and explicit NumericFormat defaults distinct.
  Preserve the existing default magnitude branches, k/M thresholds, named rounded
  binary64 division operands, Forest expression grammar and Waterfall defaults.
  New explicit SI/compact settings do not silently replace omitted-policy spelling.

Resolve complete sets before geometric fit, retaining every source occurrence,
sign and original paint order. Adaptive precision remains one common precision
over the semantic set, independent of available width. Check uniqueness on the
complete final strings under the consumer's existing requirement: axis collisions
miss a whole generated source; required guide collisions are errors; fixed value
labels may intentionally repeat. Neither a callback nor a compact rule authorizes
dropping individual members or parsing display strings back into scientific data.

### Explicit Locale, Grouping And Currency

Provide immutable named-locale selection and complete caller-defined locale
values. Pin the advertised named registry's data version, contents and provenance
in Rust project sources; do not consult host locale, environment variables, ICU
installation, network services or downloaded assets. Publish the exact supported
names and reject unknown names. Custom definitions provide the same full behavior
without requiring a new named registry entry or a renderer plugin.

Locale values cover decimal/group separators, grouping sizes and repeat semantics,
ten digit tokens, positive/negative signs, percent/per-mille symbols, currency
prefix/suffix and spacing/accounting conventions. Support western 3-digit and
Indian 3-then-2 grouping, ordinary/nonbreaking/narrow-nonbreaking separators and
non-Latin digits. Validate nonempty/distinct digit tokens, positive group sizes,
field-appropriate Unicode/control syntax and coherent absence/disabled grouping.
Preserve exact supplied bytes; currency formatting does not perform exchange or
scientific-unit conversion.
The appendix pins all 59 named d3-format 3.1.0 locale IDs and their source/license,
and specifies complete custom locale constructors, grouping repeat modes and
read-only token access. Invariant ASCII spelling remains the explicit-policy
default; selecting a named locale is a deliberate request, not host inference.

Apply grouping to numeric digit runs, not affixes or arbitrary callback text.
Define exponent, radix, sign, accounting parentheses, decimal, currency, percent,
trimming and padding order in one normalization/emission table. Multi-scalar
tokens are real source text and contribute their actual shaping/ink. Localization
is explicit per NumericFormat; equal labels under one current domain do not make
different locales compatible in a Shared guide.

### SI, Compact Rules And Common Unit Selection

Support automatic per-value SI selection, a caller-fixed SI exponent and one
prefix selected for the complete owner set. Include the current standard SI
prefix range from quecto through quetta, with canonical micro-symbol spelling;
retain exact factors. At table extremes, retain a valid end prefix and allow its
mantissa outside the usual interval rather than inventing a unit or clipping a
value. Document this saturation rule independently of rounding/carry promotion.

Expose common decimal compact presentation and complete custom ordered rules:
threshold, exact divisor, prefix/suffix and unit spacing. Validate finite ordered
thresholds, positive divisors, uniqueness and text syntax before attachment. Users
can specify k/M/B, long words or application units and choose per-value or whole-set
rules. Threshold comparison uses the unrounded scaled mathematical magnitude.
Any round-boundary promotion has one documented rule in the emitter and must not
re-round a quantized value or silently change fixed/shared prefixes.
The appendix gives KeepThreshold/PromoteOnRounding as explicit choices and the
scales-backed K/M/B/T convenience over that same custom-rule representation.
Whole-set selection resolves its declared rule once, then freezes the common
factor; no individual label can promote it independently. Fixed selections require
KeepThreshold instead of ignoring a contradictory promotion request.

For automatic whole-set selection, use the greatest absolute nonzero displayed
mathematical member; all-zero sets select the neutral factor. The same rule is
independent of member order, text width, clipping and cache state. A caller-fixed
factor/prefix takes precedence explicitly. Record the selected factor once in the
complete-set result. Shared continuous guides choose after final-domain merging;
actual shared axes use the common axis owner, not separate local approximations.
Mathematical decimal factors and their callback views retain exact arbitrary-size
integral exponents using the appendix's decimal interchange, not a new i64 ceiling.

### Shared Multiplier And Unit Explanations Are Layout Content

Support both repeated per-label unit text and one common multiplier/unit
explanation. The motivating axis can display `1`, `2`, `3` with a single `×10⁶`
and optional unit. Support explicit and automatic factors, including a whole-set
SI prefix combined with a supplied unit. Never infer scientific units or a scale
by examining a literal prefix/suffix or existing axis-title string.

Extend NumericPresentation/FormattedSet to retain labels, operand/occurrence
identity, selected precision, selected common factor/unit and an optional required
explanation. Represent the explanation as ordinary shared text with the selected
owner's complete text style and optional explicit Text instructions. Default to
that owner's existing numeric-label role; do not add a parallel font/style system.
Generate exponent/sign text deterministically, including superscript digits when
used, and prepare the actual emitted Unicode source.

Each consuming owner supplies a real measured explanation slot:

- Coordinate axes reserve an outside-axis slot beside their tick/title block;
  account for perpendicular thickness and endpoint ink through axis feedback.
- Continuous color guides reserve one measured explanation within their container;
  Shared composition owns exactly one result after policy/domain merging.
- Heatmap Values has a cell-value explanation outside the cell body; Coefficient
  labels use their numeric-label block; Waterfall totals use the total shelf.
  Keep these independent of a differently formatted color guide or numeric axis.

Measure explanations under their actual text policy before accepting an owner
plan; include their minimum/preferred widths, height and ink in natural/assigned
sizing. Complete visible explanations are the default. Only explicit text policy
can abbreviate, clip, hide or allow local overlap; preserve the full explanation
and metadata in its retained slot. Tight space alone cannot silently remove it or
append it to SVG after geometry is frozen. Keep x/y and independent
value owners distinct. Facets, spanning Canvas owners and max-width roots must
reserve each actual explanation once and preserve the selected root width.

### D3-Style Specifiers Compile Into The Same Policy

Add `NumericFormat::from_specifier(&str) -> Result<Self>`. Use this grammar and the
appendix's [exact normalization/default table](92_PUBLIC_API_PLAN.md#specifier-normalization-and-emission-order):

```text
[[fill]align][sign][symbol][0][width][,][.precision][~][type]
```

Cover left/right/center/sign-aware alignment, custom fill, minus/plus/space/
accounting signs, locale currency `$`, alternate radix `#`, zero padding, minimum
field width, grouping, precision and trimming. Numeric types cover fixed and
general significant output, exponential, rounded significant decimal, SI, fixed/
significant percent and decimal/binary/octal/lowercase-or-uppercase hexadecimal
integer output; include the numeric `n` grouping shorthand and omitted type.
The accepted spellings include `.2f`, `,.2f`, `.3s`, `.1%` and `+.2f`, without
limiting the parser to those examples.

Support the same capabilities through typed policy values. Integer/radix emission
quantizes once using the selected rounding mode and works on exact mathematical
integers, not an unchecked fixed-size cast. Field width counts emitted Unicode
scalars, not bytes or CSS pixels; actual geometry always comes from shaping.
Specifier defaults can differ from NumericFormat::default, but each compiles to
explicit fields of that same type. Equivalent typed and parsed policies have
equal normalized identity and results. Percent syntax supplies an actual factor;
a literal percent suffix still does not imply scaling.

Specify syntax compatibility separately from JavaScript arithmetic. Exact binary64
and certified-power rounding use the renderer's explicit policy, with ties-to-even
by default, rather than attempting to reproduce every ECMAScript conversion.
Reject malformed input, trailing junk, unknown type codes and unsupported text
coercion (`c`); arbitrary authored strings use Text/exact ticks. Do not silently
reinterpret an unknown code as a general formatter or use printf/Rust formatting
as a second parser. Checked field/output lengths and actual allocation failures
remain errors, without an arbitrary application-wide string budget.

### Single-Value And Complete-Set Formatting Functions

Provide owned single-value and whole-set callback construction on NumericFormat.
The normal numeric attachments continue to receive one validated policy type.
Preserve Plot cloning and its existing threading bounds with an opaque shared
callback handle; library invocation is serial in canonical owner/source order.
Support fallible application results through the crate's error contract.
Use the appendix's `value_function`/`set_function` signatures, NumericLabels result
and borrowed NumericValueContext/NumericSetContext views. Their built-in helpers
bypass only the emission callback, not exact factors or complete-set resolution.
Expose the retained raw FacetCell levels through the specified read-only methods
rather than return an opaque owner key the application cannot inspect.

- A context exposes the exact operand kind, raw source identity/occurrence,
  consumer role and complete semantic-set context where applicable, explicit
  multiplier/unit/locale information and an optional call into the same built-in
  formatter. Forest context carries its exponent operand, not a lossy f64 power.
  Geometry, renderer/backend handles and mutable scientific input are not context.
- A single-value callback returns one complete display string per occurrence.
  A set callback receives the ordered complete set and returns the same number of
  labels in occurrence order, with optional explicitly authored common explanatory
  text. Validate cardinality; empty sets produce no invented member and singleton
  sets remain ordinary one-member inputs. An absent/empty numeric owner invokes
  no callback and contributes no numeric explanation or font requirements.
- Authored common explanation text replaces generated text in the same required
  slot; it does not append another marker. Omitting this text uses the configured
  common factor/unit explanation rather than silently suppressing it. Retain the
  typed factor/unit metadata independently of the authored spelling.
- Callback output is final authored text, checked by the common display validator
  and measured without trimming, humanization, locale substitution or a second
  numeric rounding pass. Built-in numeric options remain available to the optional
  context helper; they do not silently postprocess arbitrary callback output.
  Typed common scale/unit metadata is resolved before invoking the function.
- Built-in adaptive precision belongs to built-in/helper conversion. A function
  owns its returned spelling; collisions do not invoke it repeatedly at successively
  different hidden precision settings. Preserve consumer-specific complete-source
  collision/error behavior. Callback errors are fatal with owner/source context,
  not permission to try a default formatter or change tick-generation semantics.
- Intrinsic low/mid/high probes remain preferences, not required unique sets.
  Identify probe versus actual-source purpose explicitly in context/dependencies;
  use the same policy without treating a probe collision as a guide-set error.
  Reachable Waterfall masks retain their existing semantic validation requirement.

Give callback policies honest identity. Clones share the same immutable handle;
different handles compare unequal even when they currently return equal strings.
Retain all nonfunction policy fields and this handle identity in NumericInputs,
Shared compatibility and upstream dependencies. Never derive functional equality
from closure type, Debug output, a hash, current labels or a function's address
alone. Equivalent independent callbacks can be shared explicitly by cloning the
same handle. No public callback registry or runtime version selector is needed.

Materialize each reached owner/purpose/operand-set/policy/domain context once per
operation in a semantic presentation table, independently of optional layout-cache
enablement. This is not an unbounded cache across terminal calls. A repeated
measurement/reuse query consumes the retained result rather than reinvoking user
code. Distinct candidate sets and real domain consequences can require new calls.
Document the invocation boundary and application responsibility for stable results
from stable inputs; Output replay never depends on later application state.

### Actual Numeric Text Preparation And Final Font Sealing

The current generated-alphabet/boundary census seals prepared font pairs in
`PreparedTextCensus::prepare_pairs_in` before any measurement. Numeric axis sources
are later created lazily from assigned width and feedback, so arbitrary function
output cannot be prepared by enumerating a fixed alphabet beforehand. Remove that
assumption through one actual-source admission protocol:

1. Resolve scientific applicability, exact-tick suppression, Shared compatibility
   and known required sets before font work. Retain the single root OperationContext
   and immutable selected stack/weight identities; discover system fonts once.
2. Prepare actual known strings and register stable font pairs. Keep a private
   operation-bound numeric text-admission capability available during intrinsic
   and bounded measurement work. It can admit actual new scalars and complete
   base-plus-mark units to an existing pair as well as prepare a newly used pair;
   calling today's duplicate-rejecting `prepare_pair` again is insufficient.
3. When a complete axis candidate or another new required numeric context is
   reached, resolve/retain its strings and explanation, admit their actual font
   requirements, then measure them. Preserve source/cardinality bounds and the
   existing candidate order. Do not enumerate all possible numeric strings,
   preinvoke functions for hypothetical values or require user alphabet registration.
4. Pair/unit additions are monotone. Existing unit/face choices and already shaped
   plans cannot change when later text arrives. Selection is independent of
   registration order; used-font/cache identity reflects actual source and chosen
   faces, not an unrelated global admission epoch. Preserve whole combining-unit
   preference and font failure as fatal, not a fit retry.
5. After the selected root plan and final bounds are accepted, seal every prepared
   environment used by that root, verify selected text is fully admitted and
   capture once. Finalization, Null, text-preserving/outlined SVG, PNG and retained
   Output publication perform no new numeric conversion, callbacks, glyph admission,
   font discovery or scientific preparation.

Make lifecycle changes explicitly in the existing coordinator, font census and
measurement interfaces. Initial preparation remains once; font-unit admission is
controlled planning work before final sealing, not a second root operation or
new convergence driver. Replace phase guards/reservation calls coherently rather
than pass a stale PreparedStage into measured work or simply remove sealed checks.
All first-party materialization retains checked lengths and fallible storage; the
finite source program bounds library invocations, not arbitrary application-code
execution or third-party allocation.

Use this protocol for built-in and custom numeric results. Closed built-in grammar
metadata may remain an exact preparatory optimization, but cannot be a competing
fallback path or a prerequisite for other emitters. Remove obsolete boundary-only
helpers/tests when no actual consumer needs them. Preserve unused-role/source font
policy, complete local Text styles and final frozen pair provenance.

### Numeric Dependencies, Composition And Owners

NumericPresentation retains enough context for immutable selected reuse: complete
policy/callback handle, operand kind, raw bits/signs and occurrences, source purpose,
owner address, domain provenance, generation/exact precedence, selected precision,
prefix/common factor, locale, final labels/explanation and selected typography.
Extend numeric, child and content-height equality/feedback with their actual
consumed inputs. A matching output string, width or label count is not sufficient
upstream identity. Lower shaping caches can still share identical actual text and
used font inputs after numeric conversion.

Merge Shared continuous domains before formatting one final set/explanation.
Suppress replaced local formatting/callback work; locally ambiguous contributors
must not reject a valid final Shared set. Compare policy/handle identity before
invocation. Per-panel value sets and independent Canvas range-group formats keep
their own owners; an actually shared displayed axis requires compatible complete
presentation context. Cloning, erasure, facet extraction and parent domain updates
carry the same policy and selected metadata through the existing route.

| Owner | Required change |
| --- | --- |
| `src/numeric/format.rs`, `src/numeric/decimal.rs`, `src/numeric/decimal/`, `src/numeric/profiles.rs` | One normalized extended policy, exact factors/rounding/emission, explicit locale/SI/compact semantics and preserved omitted profiles |
| `src/numeric.rs`, new private `src/numeric/` siblings as needed, `src/lib.rs` | Cohesive locale/specifier/function implementation and deliberate root exports, without public implementation modules |
| `src/numeric/power.rs`, `src/numeric/power/certified.rs` | Mathematical powers with exact factors, directed rounding/cells and real representation refusal |
| `src/numeric/set.rs`, `src/numeric/inputs.rs`, `src/layout/panel/sources.rs` | Complete grouped results/explanations, callback identity/materialization, unchanged source order and actual-text admission before candidate measurement |
| `src/layout/kernel.rs`, `src/layout/text/requirements.rs`, `src/font/canonical.rs`, `src/theme/resolved.rs` | One operation-bound admission lifecycle, stable pair/unit additions, complete text-cache provenance and final sealing before capture |
| `src/layout/panel/cells.rs`, `src/layout/panel/axis_frame_grid.rs`, `src/layout/interior/sizing.rs`, `src/decorations/legend/` | Required axis/guide explanation geometry, actual ink, minimum/preferred sizing and shared numeric source compatibility |
| `src/plots/heatmap.rs`, `src/plots/interval_estimate.rs`, `src/plots/waterfall.rs`, `src/plots/waterfall/totals.rs` | Complete active value sets, independent explanation slots, required-set/mask semantics and retained source eligibility |
| `src/builder/`, `src/orchestrator/facets/`, `src/canvas.rs`, `src/canvas/dependencies.rs`, `src/canvas/sizing.rs` | Policy/handle preservation, final Shared domains, owner-local numeric/font preparation and full geometry/height reuse dependencies |

## Numeric Tick Generation And Categorical Label Selection

### Scope, Precedence And One Policy

Implement all five requested controls: exact generated counts, data-interval
density, anchored multiplicative intervals, explicit categorical-label thinning
and numeric position-generation functions. Numeric controls attach through the
existing x/y TickGeneration setters; categorical label selection has a separate
axis-specific value. NumericFormat continues to own number spelling, locale,
shared factors/units and formatting functions, not position generation.

Preserve Automatic, preferred Additive/Log10Decades, approximate Target, hard
Maximum and literal exact-tick behavior. Every x/y generation setter replaces
that axis's complete policy, independently of formatting, limits and the other
axis. Automatic resets the locator and its cap. Exact x_ticks/y_ticks take
precedence in either order: do not execute a replaced generation function, impose
its count/interval/maximum on the exact list, or format its literal labels. Actual
axis-kind/scale validation remains independent of that suppression.

Replace the single payload choice with one normalized immutable TickGeneration:
one closed primary locator and an optional validated generated-count cap. Primary
locators are Automatic, Interval, approximate Target, ExactCount, Multiplicative
and Function. Interval retains its step/origin, coordinate space and fixed/preferred
requirement: the current Additive and Log10Decades constructors select preferred
linear-data and integral-exponent intervals respectively; data-interval density
provides fixed data-space spacing and explicit adaptive behavior as separate choices.
Multiplicative likewise retains its anchor, ratio and fixed/adaptive requirement.
Label overlap/visibility/overflow is independently configurable. Reuse interval machinery,
without a second implementation for the newly exposed requirement.

Keep the existing maximum constructor's meaning by normalizing it to Automatic
plus its hard cap. This is one representation, not competing optional locators or
a legacy Maximum execution path. Add bare consuming cap edits under the naming
rule; a new same-name writer also migrates an existing reader to `get_`.

| Primary locator | Optional generated maximum |
| --- | --- |
| Automatic, including the existing Maximum constructor | Filters complete candidates in their existing order |
| Preferred Interval, including current Additive/Log10Decades, or approximate Target | Filters complete candidates, never individual positions |
| Fixed Interval in Data or Scale space | Entire in-domain interval list must satisfy an explicitly selected cap; no implicit enlarged step or truncation |
| ExactCount | Must be at least the exact count; otherwise construction rejects |
| Fixed Multiplicative | Entire lattice source must satisfy the cap; no truncation |
| Adaptive Multiplicative | Filters complete candidates under its declared adaptive program |
| Function | Final in-domain list must satisfy a selected cap; no truncation or reinvocation at a smaller count |

Consuming edits preserve and validate the complete policy, including contradictory
count constraints. Clearing a cap restores the primary locator's unconstrained
generated program. `is_automatic` means the complete default/reset policy, not an
Automatic primary with a nondefault cap. The current 10000 ceiling is not a
universal capability limit for the new explicit locators/functions. Support an
explicit count maximum or no application-level maximum, as distinct from actual
count/index/output representation and fallible allocation. Existing private
Automatic candidate bounds remain finite; they do not silently cap an explicit
request. Migrate public count validation and all affected source/capture consumers
coherently when removing the old shared ceiling. Data-interval density retains
its data-unit step and numeric origin in the complete locator policy.
The appendix fixes counts/caps and their read results as usize in that capability
migration; the earlier naming-only step retains u32. Use the one consuming
`maximum_count(Option<usize>)` edit and paired `get_maximum_count` reader.

TickGeneration must support an owned cloneable function handle. Replace Copy-based
storage/access assumptions with explicit borrowing/cloning throughout AxisSpec,
AxisSemanticInputs, source cursors, erasure, facets and composed-input snapshots.
Keep complete normalized policy equality and operation-bound function identity;
do not reconstruct an automatic choice while copying or compare only selected
positions. Add variants/refinements with their live consumers, without dormant
future selectors or compatibility wrappers.

### Exact Count: Four Positions Across A Known Range

Workflow: a publication's small multiples each require exactly four positions.
For a linear resolved range [0,10], generate [0,10/3,20/3,10]; formatting can display
`0,3.33,6.67,10`. Target(4) may instead choose [0,5,10], and Maximum(4) only imposes
an upper bound. The new requirement is exact cardinality, not a nice-step target.

For the standard exact-count locator, n>=2 includes both resolved display endpoints
with equal intervals in the selected data/scale coordinate space. Preserve both
raw-data and logarithmic-space interpretations through explicit coordinate choice;
do not use one scale's default to exclude the other. Zero requests a valid empty
tick list, as with LinearLocator(0), without removing marks/frame/title. A singleton
has a configurable lower/midpoint/upper or caller-specified in-domain position;
a lower-endpoint default follows
Matplotlib LinearLocator, while the existing Target(1) midpoint remains unchanged.
Thus a Log10 range from 0.1 to 10 with n=3 in scale space has coordinates
[-1,0,1] and mathematical value labels [0.1,1,10]; data-space selection instead gives
[0.1,5.05,10] before projection. Do not silently substitute one interpretation for
the other. Counts refer to positions, not printed decimal
digits or nonempty labels. They do not modify scientific limits or nice-range policy.

For linear endpoints lo/hi and n>=2, independently round each exact affine combination
`((n-1-i)*lo+i*hi)/(n-1)` once to binary64, retaining exact endpoint bits. Use
checked rational/scaled arithmetic rather than overflowing hi-lo or accumulated
step addition. Logarithmic interpolation operates on the canonical exponent-space
domain and retains mathematical power operands for presentation. It does not inherit
the old integral-source exponent clipping as a physical-value restriction.

Generate one complete source. If adjacent endpoints cannot represent n
distinct representable data positions, report a contextual generation failure;
do not perturb endpoints or lower n. Coincident painted/projected label geometry
is resolved by the explicit overlap/visibility policy, not an unconditional
nonoverlap rule for all uses.
Built-in adaptive formatting can choose its permitted common precision, but the
locator cannot replace the list with fewer ticks or a terminal midpoint. For n=1
the selected singleton follows its declared label policy. Any selected maximum is
checked before text work.

### Data-Interval Density: One Tick Every Twenty Data Units

Workflow: request one tick for every 20 units of the numeric data range. The final
resolved domain determines how many positions exist:

| Resolved data range | Numeric origin | Positions at step 20 | Count |
| --- | --- | --- | --- |
| [0,100] | 0 | [0,20,40,60,80,100] | 6 |
| [0,200] | 0 | [0,20,40,60,80,100,120,140,160,180,200] | 11 |
| [40,100] | 0 | [40,60,80,100] | 4 |
| [5,95] | 0 | [20,40,60,80] | 4 |
| [5,95] | 5 | [5,25,45,65,85] | 5 |

Accept a finite strictly positive interval s and finite numeric origin a in data
units. Positions belong to `a+k*s` for signed integral k. Origin 0 is the ordinary
default, following MultipleLocator/tick0/breaks_width, and can be replaced directly.
The user chose configurable numeric origins rather than an additional implicit
origin-following-the-view mode. A caller may set a to a known lower bound or use
the existing generation-function extension when that rule depends on the view.
For fixed spacing, retain in-domain lattice positions and do not add unrelated
endpoints implicitly; explicit positions/count/function sources cover other lists.
The same final domain/step/origin gives the same positions and count across widths,
fonts, root modes and cache settings. Parent
range resolution can change the list because its data-domain membership changes.
The interval is not a factor applied to a renderer-selected tick count or to a
pixel/font pitch, and is not estimated from the number of source observations.

Expose built-in fixed and preferred interval requirements rather than hard-wire
one response to crowded labels. The existing additive(step) remains a preferred
interval that may coarsen; fixed spacing preserves its full position list. Both
share the same normalized source/arithmetic owner and numeric origin. TextLayout
can explicitly allow overlapping labels, hide/reduce labels, clip/overflow text,
fit sizes or elide within that list. Default strict text fit can still refuse,
but it is one selected behavior. Label reduction never changes ticks/grid lines or
the data interval. No function is required to configure either interval or origin.

Compute first/last indices and cardinality from checked exact input ratios and
boundary comparisons, without overflowing hi-lo or unchecked float-to-index casts.
Compute every a+k*s independently and round once into its declared data-coordinate
representation; membership/boundary checks must match that representation, including
ordinary 0.1-step endpoint expectations. Do not accumulate rounded additions or
introduce an unproved tolerance/fatal-boundary rule. Use independent rational/
rounding-cell witnesses, preserving current preferred-source observable behavior
where promised. Check any caller-selected maximum before library materialization/
formatting. Large valid sources are not rejected solely because they exceed the
old 10000 threshold; actual representation/allocation failure is still an error.
A fixed interval can validly have no in-domain ticks, following fixed locators and
ggplot2 empty breaks. Preserve frame/title/data without a numeric explanation.
An explicitly adaptive interval retains its documented complete-candidate recovery.

On a Log10 axis, a data interval still refers to raw data units, not exponent or
screen distance; require positive admitted positions and retain their mathematical
display operands through projection. Resolve genuinely retained raw/mathematical
domain bounds without pretending that the rounded log transform has an exact
binary64 inverse. Existing preferred Log10Decades remains integral-exponent
spacing, and Multiplicative retains its ratio contract. Extend the common operand/
checked-boundary path as needed rather than reinterpret s by the active scale.

Resolve the final facet/Canvas data domain before generating this source, then fit
its full list within the actual assignment. Preserve the existing bounded root
lifecycle and exact-tick precedence. Function remains the separate application-
defined position source described below.

### Multiplicative Intervals: An Anchored Geometric Lattice

Workflow: a positive logarithmic value axis spans 1..64. With anchor 1 and ratio 2,
its positions are [1,2,4,8,16,32,64]. Ratio 1.5 yields [1,1.5,2.25,3.375,...].
Unlike Additive(2), which produces an additive lattice, or Log10Decades, which uses
integral powers of ten, this locator specifies an arbitrary multiplicative ratio.

Retain a configurable positive anchor and positive ratio r!=1 for the geometric
lattice `a*r^k`, with signed integral indices. A default anchor of 1 is not a fixed
origin. Reciprocal ratio directions describe the same indexed lattice through
exact normalization, rather than an unexplained r>1-only public restriction.
The positive lattice follows LogLocator's logarithmic value domain. On a linear
projection, intersect it with the actual view instead of reject a wholly negative
view that simply has no members. A view containing an infinite near-zero tail is
a genuinely unbounded fixed source, not a reason to invent a hidden exponent cutoff.
Do not add signed/mirrored geometric lattices without a supporting precedent or
user confirmation, or change the scientific transform to implement a locator.

Determine the complete in-domain integer-index interval using checked, certified
boundary comparisons. Check its count/cap before allocating positions or forming
large powers. Compute each member from its original anchor/index, not repeated
rounded multiplication of the previous value; retain deterministic boundary and
rounding rules without epsilon inclusion or unchecked float-to-index casts.

Use finite binary64 positions for a linear projection. On Log10 axes retain the
mathematical lattice operand and its checked scale coordinate; extend the shared
numeric operand/certified conversion machinery as needed rather than overflow or
underflow an intermediate physical pow, or label an approximate inverse transform.
Normalize exactly equivalent decimal-power cases through the same operand engine.
Real arithmetic, index, coordinate or output representation failures remain fatal.

Fixed geometric spacing retains the complete lattice; label visibility/overlap/
overflow is selected independently. Support explicit adaptive logarithmic source
selection as in LogLocator without silently changing a fixed ratio request. When
no member lies in the view, the empty generated list is valid; retain axes/marks
and produce no numeric-label explanation.
Distinguish that case from an oversized or unrepresentable nonempty source, which
must not be disguised as empty. Preserve raw/exact mathematical and occurrence
identity, with a stable geometric ordering for cell measurement.

### Category Label Selection: Twelve Bars With Six Month Names

Workflow: twelve month categories still produce twelve bars at the same positions.
An explicit stride of 2, offset 0 and no forced endpoints displays January, March,
May, July, September and November. It does not delete February's bar, merge groups
or renumber category positions. Default All makes every label eligible and retains
the owner's ordinary visible-label policy unless the caller explicitly changes it.

Expose one immutable CategoryLabelSelection with All and an every-k selection:
positive stride, zero-based offset smaller than stride and explicit endpoint
retention (none, first, last or both). Select indices i satisfying
`i % stride == offset`, then union any requested endpoint indices once. Normalize
stride 1/offset 0 to All; empty/one-category domains and strides larger than the
domain have defined results without checked-index overflow. The default every-k
factory uses offset 0 and no forced endpoints, matching the month example.

Add independent x/y category-label selection instructions and sealed capabilities
derived from actual categorical axes. Reuse actual-axis applicability information
with general TextLayout while keeping selection categorical-only. X families are
Trajectory, Interaction, Heatmap, Bar,
StackedBar, Boxplot, Violin and Raincloud; y families are PointInterval, Coefficient,
Forest, Dumbbell, Heatmap, HorizontalBar, StackedHorizontalBar, Beeswarm and
Waterfall. Trajectory checks its actual numeric-versus-categorical x axis; a
non-All category policy on numeric x rejects before fonts. Conversely, a
nondefault numeric generator on categorical x still rejects. Matrix variable
titles and erased Any gain no category setter, and color-scale capabilities do
not stand in for category-axis capabilities.

Resolve the full scientific category domain, explicit/value order, humanization/
aliases and display-collision checks first. Select labels against that final
ordered axis domain, not input-row order or a pre-facet global ordinal. Store one
selection policy and selected original indices with the complete axis; preserve
all raw identities, tick positions/segments, grid lines, projection category
counts, assigned track edges, marks and independent annotations/guide sources.
Do not implement thinning by deleting AxisTick entries or overwriting their labels
with empty strings, which loses provenance and native geometry.

Only eligible axis-label text participates in analysis. Actually consumed analysis
determines font needs; final visibility/clip and selected size determine painted
ink, gutters and capture under the declared policy. Other consumers of the same
category text retain their own requirements. Keep original category indices in
requirement addresses; selecting labels does not compact owner/source IDs. A suppressed name
does not waive raw-key, alias or domain-collision validation.

Give selected labels capacity between neighboring selected centers and the outer
axis boundaries, including required clearance and actual ink. A sparse label can
use intervening unlabeled category space without changing the underlying full
category cells. All retains the present per-category cell policy. Handle assigned
native Heatmap cells and Beeswarm/Waterfall width producers explicitly; avoid
indexing full assigned_cells with a shortened tick vector. The selected TextLayout
consumes the same required set, original centers and canonical transform. Its
explicit overflow/font-fit policies may adapt display content/size. An explicit
overlap/boundary hiding policy can select a visible subset of eligible labels;
retain both sets and endpoint intent without changing categories or ticks. Under
the ordinary all-selected-visible policy, fitting cannot silently hide a label.

Canvas FirstColumnPerRow still compares complete resolved category domains/orders,
plus compatible label-selection and text policies, before suppressing duplicate
whole axes.
It does not compare only the currently visible names. Apply selection once at the
actual axis owner; preserve the span-aware row-band/anchor rules. Facet local
subsets and genuinely shared category domains each use their final prepared order.

### Generation Functions: Business Positions From The Actual View

Workflow: always include the current endpoints and an in-range warning value 50.
The same rule returns [0,50,120] for view [0,120], [20,50,80] for [20,80] and
[60,120] for [60,120]. It adapts after facet/Canvas range resolution instead of
requiring the application to reconstruct all those final ranges beforehand.

Provide an owned fallible locator function through TickGeneration. Its immutable
context includes axis identity/scale, resolved domain/provenance and actual assigned
axis span, with no backend, mutable Plot, drawing hook or scientific-data mutation.
Expose exactly retained raw bounds and explicit mathematical bound/scale-coordinate
views for logarithmic domains;
never reconstruct them using an unchecked physical pow or imply exact inversion
of the rounded log10 transform. Call only after the actual domain and assignment
exist. Natural-width low/mid/high estimates remain preferences and do not invoke
the locator for hypothetical view boxes.

The ordinary result is a complete source-order list of raw data-space positions, with
no labels. Provide a deliberately typed exponent-space result for Log10 when a
caller wants mathematical power positions outside physical binary64. Keep the
coordinate form explicit; do not guess from magnitudes or treat ordinary returned
values as exponents. Raw logarithmic positions use the existing scientific
transform when projected and retain their original numeric display operand.
Exponent results retain PowerOfTen operands. Multiplicative sources likewise
retain their own mathematical operand instead of reducing everything to a bool
that means every logarithmic label is `10^projected_position`.

Process returned positions through normal window/projection/layout operations,
without an independent correctness-validation pass or strictness option. Unsorted,
repeated and off-window positions are not errors by themselves. Retain original
occurrence IDs and source order; build a stable geometric-order view when cells
need it. Normal view filtering/clipping handles off-window positions without
changing scientific limits or consuming their unused text. Coincident occurrences
retain their drawable geometry and declared label policy instead of failing a
source-level uniqueness test.

Draw when the requested operations can succeed. If an actually consumed coordinate
cannot be projected, the requested geometry cannot be laid out or a real storage/
arithmetic operation fails, return that operation's error with owner context. Do
not invent a separate result-acceptance rule or run a prevalidation pass to reject
otherwise drawable output. A function's own error likewise ends the operation;
it does not cause a repair, fallback or lower-count application call.

An explicit generated maximum applies to the final in-domain list before first-party
copies/formatting; exceeding that requested constraint is an error. It is not a new
universal 10000 cap on callback results. Empty output is valid and removes generated
ticks/grid lines/number explanations, not scientific marks or axis titles/frame.
A nonempty result uses its declared text overlap/visibility/overflow policy.
Explicit ticks suppress the function altogether in either setter order.

For example, returning [80,20,20,-10,120] for view [0,100] retains the in-view
occurrences [80,20,20] with original indices [0,1,2]. The two occurrences at 20
can share a painted position; internal geometric ordering does not rewrite their
source identities. No ordering, duplicate or domain-acceptance check rejects this
drawable source.

Pass retained in-domain positions and their operand kinds to NumericFormat,
including its own single/set formatting function. Thus the warning position 50
can be displayed as `50`, `50.0` or caller-authored text through the format policy;
the locator never returns an AxisTick or controls wording. Preserve one source
list and one corresponding complete formatted result for the same context.

Use the same honest shared-handle identity and operation-local materialization
principles as formatting callbacks, with separate locator/formatter roles.
Clones share identity; distinct functions remain different policies even when
their present positions coincide. Remove TickGeneration's Copy requirement rather
than leak functions into a global registry. Borrow or clone complete policy inputs
through standalone, facets and Canvas.

Materialize once per reached owner/axis/handle/policy/domain/provenance/assignment
context, independently of optional child/text cache enablement.
Assignment identity includes the geometry consumed by the public context, not an
unrelated orthogonal size, backend or incidental probe. Changed shared ranges or
consumed geometry may create a new context within the existing bounded measurement
program; repeated queries reuse the stored result.
Callbacks must return stable results for stable input; the source program bounds
library invocations, not arbitrary application execution. Failures retain owner
context and produce no partial output. Freeze positions, operands and formatted
text before capture; no locator or formatter runs during capture or Output replay.

### Composition, Source Programs And Implementation Owners

Make the source program distinguish soft ordered candidates from complete fixed
sources. ExactCount, fixed Interval/Multiplicative and Function do not inherit an
unrequested coarsening/terminal-midpoint path. Adaptive interval/geometric choices
have their declared candidate programs. Preserve the existing
Automatic/Target/Maximum and preferred-interval source order. Handle empty complete
sources explicitly rather than confuse them with a skipped/failed candidate.
Numeric maximum applies to positions including intentionally empty display labels,
not the count of visible strings. Category selection never filters numeric ticks.

Retain the complete generator, interval space/step/origin/requirement, multiplicative
anchor/ratio/requirement, cap, mathematical operand and source kind, domain provenance,
actual callback context/result and category policy/full order/eligible/visible
indices in their dependent consumers. Update source/axis feedback, composed
equality, height caches and child reuse. Equal positions or visible names do not
establish upstream equivalence. Track explicit/supplied versus generated precedence
through erasure and parent reassignment.

Resolve parent domains before invoking locators; shared ranges do not automatically
force identical per-panel policies or counts. An actually shared displayed axis
has one owner and requires compatible policy/context. Marginal strips copy the
central axis positions without independently invoking a function or formatting
labels. Native coordinate-axis users share the source program; Matrix retains its
private default route without gaining public attachments. New source/function
strings use the actual-text font admission and final sealing plan above.

Numeric/categorical controls can change measured gutters and content height, but
not scientific range provenance, statistical preparation, category topology or the
selected root-width rule. Preserve the existing eight-round root coordinator and
bounded guide/local alternatives. A spanning Canvas child uses its full assigned
axis span; a max-width root keeps its selected width and consumes the chosen text
policies, with final root containment established before publication.

| Owner | Required change |
| --- | --- |
| `src/spec/ticks.rs`, `src/spec.rs`, `src/lib.rs`, `src/kind.rs`, `src/builder/configuration.rs` | One validated extended generator, category-selection value, deliberate context/result exports and actual-axis sealed gates |
| `src/layout/panel/sources.rs`, `src/layout/panel/sources/tests.rs`, `src/numeric/`, `src/layout/panel/projection.rs` | Exact/data-interval/lattice source arithmetic, fixed/adaptive programs, explicit mathematical operands, normal window handling and complete callback results |
| `src/layout/panel.rs`, `src/layout/panel/semantics.rs`, `src/layout/panel/cells.rs`, `src/layout/panel/axis_frame_grid.rs` | Owned policies, full category positions plus selected-label metadata, capacity/ink, empty sources and frozen axis/source identity |
| `src/builder/facets.rs`, `src/builder/accessors.rs`, `src/plots/shared_x/prepared.rs`, `src/orchestrator/validation.rs` | Remove Copy assumptions, preserve parent provenance/choice, applicability checks and complete composed dependencies |
| `src/layout/text/requirements.rs`, `src/layout/interior/sizing.rs`, `src/plots/heatmap.rs`, `src/plots/beeswarm.rs`, `src/plots/waterfall.rs`, `src/plots/interval_estimate.rs` | Selected-label census/preferences, native full-cell geometry, numeric/explanation measurement and actual-source admission |
| `src/canvas.rs`, `src/canvas/dependencies.rs`, `src/orchestrator/facets/`, `src/plots/marginal.rs`, `src/plots/matrix.rs` | Final shared domains, whole-axis suppression, owner/span context, scoped reuse and existing native/private admission boundaries |

## Delivery Order

Apply the naming rule first, then consolidate text inputs/styles. Complete actual
numeric text admission before exposing emitters whose output is not covered by
today's grammar; complete grouped numeric results before shared explanations and
formatting functions. Extend the generator and add explicit category selection
through those common preparation/formatting paths before Canvas owner migration.
Canvas owner records can then retain the final axis/text/numeric context.
Complete spans and shared anchoring/rotation/overflow/font-fit measurement before
refactoring natural width for the cap. Each code-changing step runs focused checks
and strict Clippy; commit completed logical changes before proceeding. Update current
contracts with an exposed capability, not ahead of it. The public API appendix
fixes concrete value/factory names, signatures and defaults; it is part of the
acceptance boundary, not a list of placeholders for implementation to rename.
Expose each listed value/variant only with its working consumer. The naming-only
step retains current types; fractional sizes, usize counts and annotation positions
migrate in their capability steps with all affected callers and read contracts.

### Implementation Status

Step 1 is complete. Public configuration edits and same-type paired readers have
migrated through declaration macros, repository callers, function-item references,
ordinary/compile-fail examples and current specifications. Read-only properties,
types, fallibility, validation timing and construction lifecycle are preserved.
Step 2 is complete. One shared display-source validator retains whitespace,
mandatory breaks and present empty-line metrics through preparation and capture.
TextPanel/headers/panel notes admit multiline content; raw-key validation and
exact-tick empty-label suppression remain distinct. Public `Labels::resolve`
shares automatic resolution and complete-domain collision checks remain in place.
Step 3 is complete: shared crate-root `TextAlignment` has replaced the title-only
type through Plot/Canvas, Theme and frozen heading measurement. The common Text/
TextLayout values now serve Plot/Canvas prose, x/y titles, TextPanel title/body and
panel-corner annotations with complete local appearance, independent line alignment,
selected font/spacing metrics and full equality. Prose content replacement, shared
alignment edits and whole-layout reset precedence are implemented, including complete root-frame,
child and height-cache identity. Standard/native axes and shared facet titles retain
independent exact styles through fonts, intrinsics, selected gaps and frozen drawing.
Continuous color titles now retain complete Text through local/shared guide preparation,
compatibility, selected section gaps, font census, caches and frozen drawing.
Canvas headers and row titles now retain independent exact styles and line alignment
through font preparation, selected padding, track requirements and frozen feedback.
Heatmap null-row text now retains complete Text through contrast selection, census,
selected padding, intrinsic identity, track allocation and frozen placement.
Waterfall base/terminal Text now retains independent instructions through eligible
name-plus-number preparation, font census, shared shelf metrics and frozen placement.
PointLayer legend Text now preserves source-only identity and categorical/literal
display context, with complete appearance/layout compatibility, selected guide
metrics, font census and frozen entry alignment. The complete receiving-slot
inventory is migrated, preserving existing source readers and complete Eq contracts.
Steps 4–22 remain pending; undelivered target signatures are not current APIs.

### 1. Apply The Public Property Naming Rule

Rename all public configuration `with_*` methods and only their same-type paired
readers, including macro expansions and function-item references. Migrate every
repository caller and ordinary/compile-fail example, removing the old spellings.
Keep private field storage, public read capabilities, validation/fallibility and
construction lifecycle. In particular, distinguish Theme's read-only components
from ThemeBuilder's read/write components and Plot requests from Output results.

Update the current naming contracts in specs 01, 03, 04 and 06 and affected public
guides with the implemented API. Verify representative nested Theme edits,
NumericFormat use, supplemental styles, unchanged invalid-value refusals and
output queries through focused tests plus affected doctests. Strict all-target
Clippy compiles the migrated integration/example callers. This is a naming change,
not a reason to grow setter/getter echo tests or run every behavioral suite.

### 2. Unify Display-Text Validation And Expose Label Resolution

Apply one shared content validator to free-form display slots, retaining source
whitespace and explicit mandatory breaks. Remove accidental multiline rejection
from TextPanel/headers/panel notes while preserving semantic key validators and
explicit empty-tick behavior. Expose the existing label resolver and retain default
humanization, Raw/alias precedence and domain collision detection. Replace obsolete
input-refusal tests with shared-input preservation/real-control rejection witnesses.
Update current display-input contracts and user examples with this implemented
behavior in the same change.

### 3. Consolidate Authored Text, Styles And Alignment

Introduce common Text and its TextLayout request, initially with only the live
alignment instruction. Migrate receiving slots and replace TitleAlignment with
TextAlignment across repository callers. Resolve exact appearance/alignment once
before the font census, including selected local spacing metrics. Complete
independent TextPanel title/body alignment and x/y title styling through the shared
engine, with one representation behind strings and styled input. Update specs
01 through 05 where their text/ownership contracts change and the existing theme/
getting-started guides. Verify inherited/local precedence, used-font completeness,
width-dependent height, actual ink and frozen replay before committing.
Use the appendix's complete receiving-slot inventory, keep existing text readers
as `&str`, and preserve annotation/TextPanel Eq with complete appearance equality.
The fractional font-size signature migration belongs to step 16, not step 1 or 3.

### 4. Replace Closed Numeric Font Preparation With Actual-Text Admission

Migrate current built-in numeric producers to the operation-bound admission
protocol while preserving their spelling/source order and one scientific/font
discovery operation. Make stable scalar/unit extension and final sealing explicit
in the existing coordinator/census/measurement paths. Remove dependence on an
already sealed finite alphabet before lazy numeric sources can be measured.
Do not expose an unused callback interface or leave a second numeric font route.

Replace obsolete early-sealing tests with witnesses for stable earlier face/unit
choices, exactly one admission of an already reached source, one final seal and
no font work during capture/replay. Exercise an ordinary generated axis, a native
numeric axis, a final Shared guide and local Text styles. Update current font/text
lifecycle contracts only with this working migration; preserve the root round bound.

### 5. Extend Exact Built-In Formatting And Parse D3 Specifiers

Implement exact general multipliers, explicit rounding and extended precision in
the existing binary64/certified-power engine. Add normalized locale/grouping/
currency, notation/radix/sign/padding, SI and custom compact policies and the one
specifier parser. Integrate per-value and repeated common-prefix output with the
existing public numeric attachments and actual-text admission before advertising
them. Shared explanation slots arrive in the next step with their complete layout.

Document parser defaults, accepted type/flag/precision combinations, named locale
provenance and typed/parsed equivalence. Validate independent rational/certified
rounding, locale tokens, custom thresholds, carry/prefix boundaries, strict syntax
errors and preserved omitted profiles. Replace obsolete 17-digit upper-bound
refusal cases with the newly implemented precision contract. Update specs 06/07,
the numeric-formatting guide and `examples/05_numeric_formats.rs` together.
Implement the appendix's pinned 59-name registry, NumericLocale custom values,
NumericPrefixes selection/promotion and `from_specifier` normalization table;
do not leave their public defaults or scalar types to a second formatter design.

### 6. Add Grouped Scale And Required Unit Explanations End To End

Retain selected common factor/unit/precision and required explanation in the
complete-set result. Implement its measured slots on axes, continuous guides,
Heatmap value blocks, point-only Coefficient labels and Waterfall shelves. Resolve
Shared numeric domains before the one final result and include explanation/role
identity in owner, feedback, height and child-reuse dependencies.
Expose NumericUnit/NumericExplanation through the specified typed edits and retain
mathematical NumericFactor exponents without a machine-integer interchange bound.

Verify automatic/fixed factor selection, source-order independence, one Shared
explanation, distinct local axis/value policies, full text/ink containment and a
real exact-width refusal. Keep original scientific positions/values and current
semantic sets. Update numeric/layout/guide/property-ownership contracts and public
examples with the implemented behavior before committing.

### 7. Add Single-Value And Whole-Set Formatting Functions

Expose owned callback construction through the same NumericFormat attachment
surface, with exact operand/context views, fallible final text and complete-set
cardinality checks. Implement shared-handle identity and operation-local semantic
materialization independently of optional layout caches. Admit actual callback
text before measurement and retain the selected result through final sealing.
Use the exact fallible Fn bounds and read-only context/result signatures from the
appendix, including inspectable raw FacetCell levels and built-in helpers. At this
step operand views expose the live Binary64/PowerOfTen cases; Multiplicative arrives
with its source in step 11, not as a dormant public variant.

Verify automatic positions with custom labels, Forest exponent context, ordered
duplicate occurrences, final Shared-domain callbacks, callback errors/cardinality/
collision handling and zero calls under exact-tick suppression. Prove callback and
font counts stop at the freeze boundary across SVG/outline/PNG/Output replay;
do not replace behavioral checks with closure identity alone. Update numeric/font/
layout/guide contracts and callback documentation with those implemented semantics.

### 8. Normalize TickGeneration And Remove Copy Assumptions

Migrate the five current choices to the single primary-locator/count-cap policy,
including preferred Additive/Log10Decades through the common Interval representation
and Maximum as Automatic plus cap, without changing their source order,
singleton semantics, exact precedence or actual-axis refusal. Move production
consumers to explicit borrows/clones and preserve normalized choice/provenance in
AxisSemanticInputs, composed snapshots and feedback. Do not add unused future
locator variants or a parallel old Maximum implementation.

Adapt existing generation/composition identity tests and verify old-to-new behavior
for Automatic/Maximum, existing interval/Target behavior, immutable resets and source
bounds.
Update the current representation contract with the working migration, including
Clone rather than Copy and the precise complete-policy `is_automatic` meaning.

### 9. Add Exact Generated Counts

Expose exact counts with standard endpoint-including interpolation, explicit
data/scale-space choice and configurable singleton placement. Integrate a complete
source through shared coordinate and native numeric-axis callers, preserving
mathematical Forest operands. Add/refine the optional maximum edit with complete-
policy contradiction checks and the
confirmed reader naming rule; preserve existing maximum-constructor semantics.
Remove the shared 10000 ceiling from explicit count/source APIs and their consumers
in this coherent migration, retaining private Automatic candidate bounds and real
checked storage/output limits. An explicit maximum remains an enforced caller rule.
Migrate target/maximum/count readers and source lengths to usize here, including
the consuming `maximum_count` edit and paired `get_maximum_count` reader. Expose
`exact_count`, TickSpace and TickSingleton with their documented signatures.

Verify zero/empty semantics, independent interpolation/endpoints/rounding, all
singleton choices, data/log-space spacing, collapsed positions, selected text policies and
final Shared-domain selection. This source cannot fall into Automatic coarsening
or midpoint retry.
Update numeric/tick documentation and public examples with the implemented source.

### 10. Add Data-Interval Density

Expose a finite positive data-unit step and configurable numeric origin through
the shared Interval locator. Support fixed and preferred requirements explicitly;
reuse the current preferred interval's source/arithmetic owners and preserve its
documented recovery independently from label visibility/overflow.
Generate after final parent-domain resolution with checked index/count/boundary
arithmetic, cap-before-allocation and retained raw/mathematical operands. The
requested data interval never rescales with assigned pixels or selected fonts.

Verify all step-20 examples, including origin 5 on [5,95], negative/cross-zero
domains, fractional steps, ordinary decimal endpoints, extremes and valid empty lists.
Use independent rational membership/rounding expectations, including indices
beyond exact binary64 integer conversion. Check unchanged source lists across
width/font/cache changes, explicit overlap/hiding/strict refusal, sources above
10000, caller-cap refusal and a final Shared domain changing the count. Exercise a
raw-data interval on Log10 independently of exponent spacing. Update public tick
docs/examples and current contracts with this built-in requirement; no generator
function is needed for data-interval density.
Use `interval(step, origin, space, requirement)` with TickSpace/TickRequirement;
preserve the old constructors' admitted scale binding in the one normalized source.

### 11. Add Anchored Multiplicative Sources

Implement checked geometric index bounds, configurable anchor/ratio and explicit
fixed/adaptive source behavior. Extend the single numeric operand/certification
path for mathematical logarithmic lattice values before exposing the source. Preserve
anchor/index identity, cap-before-allocation, complete set formatting and final
font admission. Do not compute a new tick from the previously rounded value.

Verify the ratio-2 and ratio-1.5 examples, reciprocal indices, endpoint boundaries,
equivalent anchor shifts, ratios near one, empty views, real index/representation
failures, caller-capped lists and selected text nonfit/overlap behavior. Include a
Forest projection and independent rational/power certificates; no platform powf or
copied locator loop serves as the oracle. Update numeric/certification contracts and examples together.
Use `multiplicative(anchor, ratio, requirement)` and the live Multiplicative operand
view, preserving exact signed index interchange and either positive ratio direction.

### 12. Add Explicit Categorical Label Selection

Expose the complete All/stride/offset/endpoint value and sealed x/y instructions,
including conditional categorical Trajectory x. Resolve selection from the final
full category order and preserve every mark, tick position and assigned cell.
Integrate selected text requirements/preferences, neighbor capacity, native axis
widths, full-domain collision checks and Canvas/facet sharing dependencies.

Verify twelve bars with six selected names, independent y selection, endpoint/
offset/empty cases, full native Heatmap grid and an unselected unavailable glyph
without suppressing an independently used annotation. Replace obsolete always-all
assumptions with All-default and explicitly selected completeness witnesses.
Update categorical/text/layout/property contracts and examples. Reuse actual-axis
applicability with later general text layout while retaining category-only selection
gates.
Expose `x_category_labels`/`y_category_labels` under the appendix's exact
CategoricalX/CategoricalY sets, with CategoryLabelSelection and LabelEndpoints.

### 13. Add Owned Tick Position Functions

Expose immutable actual-view context and typed raw/exponent position results through
TickGeneration. Implement exact-tick suppression, normal window/projection handling,
shared-handle policy identity and operation-local materialization independent of
optional layout caches. Format retained operands through NumericFormat, including
its already implemented functions and measured factor/unit explanations.

Verify the warning-50 example, final parent domains, assigned-span context, empty
output, raw/exponent operands and ordinary view filtering with source-order and
duplicate occurrences retained. Demonstrate drawable unsorted/repeated/off-window
results, real projection/layout/storage failures, immediate function errors and
zero function calls for exact ticks. Include a valid large list and a caller-cap
refusal without introducing result validators or strictness configuration.
Prove repeated context queries/cache toggles and frozen publications reuse results
without locator/formatter/font work. Update tick/numeric/layout/guide lifecycle
contracts and user examples with this working interface before committing.
Use the appendix's TickContext and TickPositions result variants; preserve typed
Data versus Log10Exponents without adding result-acceptance settings.

### 14. Make Canvas Ownership And Track Geometry Explicit

Migrate current 1x1 Plot/TextPanel placements to canonical payload/footprint owner
records and derived occupancy. Preserve anchor IDs through preparation, theme/
domain inventories, text census, guide contribution, child plans and rendering.
Separate declared track boundaries from assigned owner rectangles. All existing
single-cell behavior must continue through this one representation; do not add a
second span-only pipeline or unused future API.

Validate occupancy, sparse tracks, header/row-title geometry, shared source
provenance and assigned projections using existing focused witnesses. This step
is a completed internal migration before exposing multi-cell placement.

### 15. Implement Rectangular Plot Spans End To End

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

### 16. Unify Text Anchors And Role Attachments

Extend live TextLayout with box/baseline/fractional justification, typed annotation
references, screen/text-local offsets, finite text bounds, wrapping and explicit
overlap/clip/visibility policies. Add fractional font size through complete
appearance, selected metrics and rendering storage. Add deliberate typed role
attachments for authored and automatic text. Replace scattered middle/baseline
placement arithmetic with one measured reference/anchor
transform, preserving omitted owner defaults, exact paint/coordinates and existing
axis multiline ordering. Carry request/default provenance through parent assignment,
prepared text, group identity, annotation collision and child-height dependencies.

Verify independent preset/custom anchor relationships, different content/size at
one fixed point, post-inset TextPanel placement, canonical fractional offsets,
panel-corner/inside-guide conflicts and unchanged source/mark positions. Update
current text/property/layout contracts and examples; every typed coordinate frame
has a real owner/transform/fit consumer rather than an untyped backend escape.
Migrate TextTheme/SignificanceMarkStyle font-size constructors, edits and readers
to f64 with their metric consumers. Expose the specified PanelTextAnnotation::at,
TextPosition reader replacement and real role attachments/sealed gates together.

### 17. Generalize Text Orientation And Rotation Placement

Add the one preset/custom-angle instruction through TextLayout across admitted
slots/groups, including numeric/exact/category ticks, titles, headers and annotation
roles. Replace the old tenth-degree restriction, resolve screen/data-relative
orientation before measurement and support rotate-about-anchor/align-after-rotation.
Implement explicit ordered automatic tick-angle candidates through this same engine.
Extend actual logical/ink fit, local gutters/tracks/shelves and native width
producers together; absolute 0 can replace a vertical default without inheriting
its old reversed-column behavior.

Verify the 30-degree point note, horizontal y title, categorical useful tilt and
rotated numeric/exact labels using independent transformed-ink/containment evidence.
Keep omitted owner directions, source membership and one frozen transform across
outputs; verify non-tenth angles and the two rotation placements independently.
Update public text/axis docs with the common attachment rather than publish the
earlier category-only setter proposal alongside it.

### 18. Generalize Ellipsis And Migrate Heatmap

Implement complete/start/middle/end display policies, configurable or empty markers,
direction and bounds/line caps with original/synthetic source provenance. Preserve
grapheme and bidi context, source-bounded trials, declared display-collision policy
and explicit no-marker-room handling. Extend actual-text admission for reached
marker units. Route Heatmap's existing Elide/Wrap defaults through this engine and remove
its replaced enum/setter/helper route with all repository callers migrated.

Verify the guide-entry workflow, suffix/middle and multiline limits, combining/
RTL/contextual cases, original-source accessibility, optional distinctness and the current
Heatmap dense/wrapped contracts. Exact tick literal spelling and NumericFormat
function results remain stored unchanged; only explicit text layout derives a view.
Update text/numeric/guide/layout contracts and examples with the working behavior.

### 19. Add Configurable Local Font Fit

Implement fixed/shrink/bounded-grow policies with fractional size bounds and a
justified finite metric-based fitting algorithm. Support independent/uniform groups,
both user-confirmed shrink/ellipsis priorities and explicit floor-failure actions,
recomputing exact metrics/spacing/anchor/ink while retaining original sources and
stable font selections. Feed only actual consequences
into current requirements/height/gutters; preserve generator preferred-size seeds.

Verify independently justified fit boundaries, useful fractional sizes, equal-bound
normalization, large checked ranges, both priorities and floor hide/show/clip/error
handling. Exercise nonmonotone wrapping, independent/uniform groups, contrast roles
and no repeated numeric callbacks per size. No unexplained sampled-size quota
can substitute for the requested bounds.
Exercise one combined anchor/rotation/ellipsis/floor request under assigned Canvas
geometry. Freeze selected view/size before capture and verify no fitting during
replay. Update current contracts and source examples after focused checks/Clippy.

### 20. Make Existing Root Modes Explicit

Replace the optional-pair request representation and coordinator signature for the
three existing modes first. Migrate all production and test callers, embedded
assignment helpers and equality consumers in the same logical change. Do not add
an unused future mode or a compatibility wrapper.

Validate exact-pair enforcement, automatic-height behavior, latest-setter
replacement, zero preflight, width locking and one prepare/capture lifecycle with
the focused checks below. Commit the completed representation change after those
checks and strict Clippy pass.

### 21. Separate Natural Width Composition From Height Work

Factor the necessary width-first paths in Plot, facets and the now span-aware
Canvas while retaining their current three-mode behavior. Use the same column
summaries/allocator for natural and assigned dimensions, including Plot footprints,
headers, row-header bands, shared axis gutters, local Plot prose and guide
preferences. Avoid a generic new layout DSL.

Validate current-mode semantic equivalence, sparse facet/Canvas topology,
unbreakable text requirements, native Matrix sizing and mixed/all-text Canvas
height measurement. Commit this refactoring after focused tests and strict Clippy.

### 22. Add And Document Maximum Width End To End

Add MaxWidth to the single request representation and expose both consuming
`#[must_use]` setters, following the existing `pub const fn` builder style. Apply
the common cap policy in all three root routes and enforce bounds throughout the
coordinator. Update actual request/height/cache dependencies and preflight handling.

Remove lower-value/obsolete tests and add the contract coverage below within the
three-layer test limits in `specs/90_DEVELOPMENT.md`. Update current contracts in
`specs/02_RENDERING_CONTRACT.md` and `specs/05_LAYOUT_SYSTEM.md`, API examples,
`docs/getting-started.md` and any affected README sizing guidance together with
the implemented behavior. Commit the completed feature after focused verification
and strict Clippy.

## Verification And Acceptance

### Public API Completeness

Treat the [API appendix](92_PUBLIC_API_PLAN.md) as the signature/default inventory
for these steps. Review crate-root exports, sealed bounds, generic Fn/lifetime/
Send/Sync contracts, consuming receivers, Result types, scalar migrations and
preserved reads against the working implementation. Keep each step's exposed
subset consumer-backed; the final API has no unspecified method/context/choice
names, dormant selectors or compatibility definitions.

Use meaningful public construction/compile-fail witnesses for capability gates
and independent semantic examples for typed/parsed policies, callback helpers and
position result variants. Verify cloned callback identity and thread-safe Plot
storage without treating pointer identity as rendering evidence. Preserve complete
appearance/request equality on text-bearing values and source signed-zero identity
in numeric contexts. Migration must cover all callers when removing Copy, changing
u32 sizes/counts or replacing the corner reader; do not add an accessor echo suite
or use available test capacity to inventory method names.

### Naming Relationships

Check the compiled public call paths for read/write, read-only and write-only
properties on their actual owning types. A nested Theme customization using the
renamed methods must retain the same effective paint/metrics and invalid-value
refusals; Output result queries remain available. Adapt existing witnesses and
doctests rather than add accessor echo tests. Confirm old public spellings and
stale documentation claims have no remaining caller or compatibility definition.

### Numeric Relationships

1. **One exact engine:** typed and parsed equivalent policies agree on complete
   identity and labels. Use independent rational/cell expectations for positive/
   negative halfway cases, each rounding direction, arbitrary factors, decimal
   shifts, carry, negative zero, subnormals/extremes, radix integers and extended
   precision. Forest factors use independent certified-root/interval evidence,
   not an approximate pow or the production formatter as expected output.
2. **Explicit locale and grammar:** western/Indian grouping, decimal separators,
   nonbreaking spaces, non-Latin digits, accounting signs/currency, percent, field
   alignment and multiscalar source tokens. Verify named/custom equivalence and
   deterministic explicit selection, strict syntax/bounds/unknown-name errors and
   exact literal affixes. Pin syntax fixtures with provenance; distinguish D3
   syntax from intentionally different numerical halfway behavior.
3. **SI/compact/common factors:** per-value, fixed and whole-set selection across
   zero, both signs, magnitude thresholds, carry and SI-table extremes; custom
   thresholds/divisors/words; one factor for the complete set independent of member
   order, geometry and cache enablement. Preserve omitted defaults through existing
   expected numeric contracts instead of a whole-scene golden.
4. **Required explanations:** the `1,2,3` plus `×10⁶`/unit workflow preserves source
   values/positions and has a retained explanation with measured text policy. Cover x/y axes,
   one continuous Shared guide and representative fixed value-label owners.
   Distinct local units/formats retain their slots. Complete/strict nonfit rejects;
   explicit abbreviation/hiding/clipping/overlap preserves metadata and its actual
   visible result. No failure produces partial publication.
5. **Function semantics:** single/set callbacks receive the declared exact operand
   and complete occurrence order, including duplicate values, signed zero and a
   Forest power outside finite physical binary64. Validate returned arity, common
   display syntax, required-set collisions, fatal callback errors and no accidental
   postformatting. Compare automatic positions under the same generation policy;
   exact ticks suppress all replaced function/explanation work in either order.
6. **Identity and domain consequences:** cloned handles authorize shared policy;
   distinct handles returning equal labels do not. Shared continuous contributors
   call the function for the merged final set without local replaced calls. A real
   domain/eligibility/purpose change resolves a new semantic result; repeated
   measurement or cache toggles reuse the same operation-local result. Value sets
   retain off-window members and reachable Waterfall-mask contracts where required.
7. **Actual font admission:** use output beyond the old numeric alphabet, including
   Unicode signs/digits and combining affixes. Prove source admission precedes its
   first metric query, full combining units select coherently, later admission
   preserves earlier face/glyph plans and unused roles/suppressed sources add no
   requirements. Seal before capture; post-seal additions remain a real error.
8. **Composed frozen publication:** representative coordinate/native owners,
   a facet, Shared Canvas and a spanning/capped composition retain explanations,
   complete source text, valid height/reuse identity and containment. Retained
   Output replay executes no callback/conversion/font work and preserves the same
   selected strings/geometry across text SVG, outlines and PNG. Use a bounded
   selection of distinct contracts, not every formatter x family x output product.

### Tick Generation And Category-Selection Relationships

1. **Current-mode preservation and composition:** existing Automatic, Additive,
   Log10Decades, Target and Maximum retain source order, exact precedence and
   documented singleton behavior through the normalized representation. Compare
   complete policies/provenance under clone/erasure/parent replacement. Exercise
   meaningful interval/cap combinations and reject contradictions before text work;
   do not test normalization solely by setting a property and reading it back.
2. **Exact cardinality and interpolation:** the four-position [0,10] example,
   zero/empty output, configurable singleton/default-lower placement, both endpoints,
   negative/opposite-extreme ranges, explicit data/log-space spacing, subnormals and adjacent
   binary64 endpoints. Use independent affine-rational rounding expectations and
   scale-space relationships. Existing Target(1) keeps its midpoint.
   Prove count never decreases on formatting/geometry failure and no unrelated
   midpoint appears. Explicit exact lists still override the generated requirement.
3. **Data-interval density:** the step-20 examples have 6,11,4,4,5 positions; the last
   uses origin 5. Final domain/step/origin, not pixels/fonts, determine the fixed
   lattice. Check negative/cross-zero domains, fractional steps, ordinary decimal
   endpoints, empty lists, subnormals/extreme indices and independent membership/
   rounding witnesses. Compare fixed positions/counts across width/typography/cache
   changes and final parent domains. Selected overlap/hide/clip/fit policies affect
   labels, not ticks. A fixed source cannot coarsen, truncate, insert endpoints or
   retry a midpoint. Verify preferred recovery separately, a drawable source above
   10000 and an explicit maximum refusal before first-party source materialization.
4. **Multiplicative lattice:** ratio 2 and 1.5, reciprocal directions, configurable
   anchors/negative indices, boundary inclusion, close-to-one ratios, real count/
   representation failures and valid empty views, including a negative linear view.
   Use independent rational/certified witnesses for membership, coordinates and
   labels. Fixed spacing has no accumulated drift, hidden coarsening, skipped powers,
   arbitrary exponent clamp or midpoint substitution. Adaptive logarithmic selection
   follows only its explicit program; an infinite fixed near-zero tail is a real
   source failure, not an empty result or an implicit cutoff.
5. **Category completeness and actual text:** twelve bars with the six selected
   month labels, plus offset/forced endpoints and zero/one/large-stride domains.
   Preserve full category counts, mark/source order, original tick/grid positions,
   complete native track geometry and independent annotations/guides. Check selected
   neighbor capacities, eligible/visible sets and the chosen text policy, including
   a true complete/strict nonfit and explicit overlap/boundary hiding. A glyph used
   only by an unselected label is unnecessary; the same glyph
   used by another actual annotation remains required and can still fail.
6. **Actual-axis and shared-category semantics:** conditional categorical Trajectory
   x, representative x/y families, native Heatmap/Beeswarm and a facet's final local
   order. Keep numeric generation and category selection separately gated. Shared
   Canvas row bands compare full domains/orders and compatible selection/text policies;
   sparse visible-name equality alone cannot make them compatible. Verify original
   requirement indices, whole-axis suppression and span-aware owner geometry.
7. **Function source and formatting separation:** the warning-50 rule for all three
   views, unsorted/repeated/off-window drawable positions, source occurrence order,
   raw/exponent results, empty output, large lists and explicit count maxima.
   Generator functions return positions only; feed retained in-domain operands
   through NumericFormat, including units/explanations. There is no independent
   result-validator/strictness mode. Exercise actual projection/layout/storage
   failures where those operations are consumed and a function error that directly
   ends rendering. No failure triggers another locator, input repair or a hidden
   lower-count application call.
8. **Bounded parent-aware replay:** final Shared domains, assigned-span context,
   identical context queries, meaningful context changes, cloned/distinct handles
   and optional layout-cache settings. Exact ticks invoke neither replaced locator
   nor formatter. Marginal strip copies do not invoke a second central locator.
   Freeze one selected source/text set before capture, then verify no new function,
   source or font work during SVG/outline/PNG/Output replay. Include a representative
   faceted root and a spanning-Canvas/max-width interaction, preserving existing
   family admission boundaries rather than building a mechanical root/family matrix.

### Text, Placement And Adaptive-Fit Relationships

1. **Common source syntax:** the same explicit text with newlines, CRLF, blank lines
   and whitespace is accepted by x/y titles, TextPanel, prose and headers, with
   preserved source and real line metrics. Validate shared shaping/line content
   rather than comparing their intentionally different default placements/styles.
   Replace the panel-note test that rejects all multiline/untrimmed text.
2. **Complete local styles:** choose different x/y-title and TextPanel title/body
   stacks/fractional sizes/weights/colors/leading, retaining unrelated Theme roles. Selected
   custom fonts are included before preparation is sealed; unused fonts remain
   unnecessary. Font-relative gaps and height consequences use selected metrics.
3. **Alignment and precedence:** independent Start/Center/End for title/body,
   multiline line placement, physical meaning with RTL text, inherited owner Theme,
   local exact appearance and existing heading convenience edits. Plain replacement
   preserves instructions; an explicit default-layout reset restores owner defaults.
   Verify source completeness, containment and frozen replay without scene snapshots.
4. **Label presentation:** direct `Labels::resolve` and the automatic path agree
   for existing humanization, Raw and aliases; default humanization is unchanged.
   Under complete/fixed-size policy, explicit/raw `LightGBM` remains intact and a
   too-narrow exact allocation refuses it rather than inventing internal breaks.
   Public resolution returns ordinary text and domain collisions are still rejected.
5. **Fixed-angle equivalence:** presets and equal custom angles normalize to the
   same value and actual geometry. Exercise both signs, quarter turns, a custom
   angle such as 30.05 degrees without tenth-degree quantization, full turns/signed zero
   and nonfinite rejection. Use independent transformed-corner/containment oracles,
   not the production measurement function as its own expected result.
6. **Useful tilts and declared fit:** a long categorical label that cannot fit
   horizontally can fit at its explicitly selected angle. Under All/all-visible,
   every label remains; category selection and explicit overlap reduction retain
   all positions plus eligible/visible provenance through the same rotation path.
   Check transformed outward ink
   and logical clearance, including explicit multiline labels and a genuine
   nonfit. Fixed-angle failure cannot choose another angle or split `LightGBM`.
   Explicit automatic orientation follows its supplied candidate order, consumed
   dependencies and the same fit predicate, independently of fixed-angle requests.
7. **Affected owners and output:** representative x-category, y-category and native
   Heatmap/Beeswarm routes, a facet, a Canvas spanning child and one capped root.
   Retain shared-axis alignment, exact geometry dependency identity, one capture
   and same-Output replay. Text-preserving SVG, outlines and PNG use the same
   canonical rotation and source/view mapping. Avoid mechanical family cross-products.
8. **Reference and block anchoring:** independent nine-preset/fraction relationships,
   out-of-box finite justification, baseline references, screen/text-local offsets,
   typed data/panel/figure frames, negative bearings, blank blocks and RTL alignment.
   Change content/font size at one fixed data reference
   and assert the chosen anchor relationship, not a frozen internal placement.
   Cover post-inset placement and corner/guide collision refusal or explicit overlap.
9. **General rotation:** a 30-degree point note, explicit horizontal y title and
   numeric/exact labels alongside the categorical witness. Verify rotate-about-anchor
   versus align-after-rotation, screen/data-relative and inherited orientation,
   multiline ordering and real transformed
   track/shelf/cell containment. Significance text transformation cannot rotate its
   bracket path or move its scientific target.
10. **Ellipsis content and Unicode:** start/middle/end, physical/logical directions,
    automatic/LTR/RTL, measured two-ended middle, default/custom/empty markers, finite
    bounds, line caps, zero/one/large source and selected no-marker-room handling.
    Preserve complete graphemes, retained mandatory breaks, bidi/
    isolates, combining units, contextual substitution and selected soft-hyphen
    provenance. Check original source/aliases separately from visible display and
    exercise optional distinctness versus allowed repeated views, including exact labels.
11. **Current defaults and admission:** compare complete/fixed owner behavior and
    Heatmap's dense Elide/explicit Wrap semantics through the shared engine; remove
    the replaced surface. An unused custom marker adds no font need; a reached
    marker is admitted before shaping. Missing fonts for consumed source analysis
    remain fatal, while wholly unselected/off-window owners remain unconsumed.
12. **Configurable font fit:** use independent metrics/boundaries for a fitting
    fractional size, real floor/upper/equal-size bounds, checked large ranges,
    bounded growth and nonmonotone reflow. Prove finite algorithm behavior without
    treating arbitrary integer samples or unvisited sizes as a nonfit oracle.
    Independent fitting can select separate sizes; an explicit uniform axis/guide
    group selects one size for its active set. Exercise floor refuse/hide/clip/show
    and keep unrelated TextPanel/contrast roles independent. Verify line height/clearance
    without changing explicit pixel gaps, mark metrics or generator seed inputs.
13. **Composition and policy order:** content-first defaults to complete smaller
    text before abbreviation; preferred-size-first can select a larger abbreviated
    view. Verify both on the same source/bounds, along with ellipsis/floor/angle/
    anchor under actual assignment and a real terminal miss. Natural preference stays complete
    at preferred size; only explicit adaptation relaxes hard requirements. Cover
    parent range/geometry changes, exact-tick format suppression, stable NumericFormat
    results across size trials, eligible/visible membership and optional-cache equality.
14. **Frozen views and bounded coverage:** inspect painted text separately from
    nonpainting original-source SVG metadata; full-source string presence alone is
    not evidence that a label was drawn. Capture/outline/PNG/replay use retained
    views/sizes/transforms with no new search, fonts or callbacks. Use representative
    guide/native/annotation roles, a faceted root and a spanning/capped Canvas;
    allocate independent contract coverage without a family x policy cross-product.

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
   a positive cap below a real complete/fixed-size unbreakable requirement fails
   before publication without truncation, partial output or modification of an
   existing destination.
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

- `src/theme/value.rs`, `src/theme/typography/tests.rs`,
  `src/numeric/format.rs`, `tests/lines/numeric_layers.rs`: meaningful retained
  nested-edit/validation behavior through the public naming migration.
- `src/numeric/decimal/tests.rs`, `src/numeric/power/tests.rs`,
  `src/numeric/set.rs` and cohesive private numeric siblings: independent conversion,
  parser/locale normalization, complete sets, explanations and callback identity.
- `src/font/canonical.rs`, `src/layout/text/requirements.rs`,
  `src/layout/panel/sources.rs`: actual-source admission, stable unit/face choices,
  candidate order, semantic materialization and final sealing.
- `src/layout/panel/sources/tests.rs`, `src/layout/panel/cells.rs`,
  `src/layout/panel/semantics.rs`, `src/builder/facets/generation_tests.rs`:
  exact/data-interval/lattice programs, full category geometry, complete policy/
  context identity, parent domains and locator/formatter invocation boundaries.
- `tests/heatmap/suite.rs`, `tests/estimates/model_displays.rs`,
  `tests/lines/numeric_layers.rs`, `tests/distributions/composites.rs`: existing
  automatic/exact ticks, continuous guides and fixed-value consumer boundaries.
- `src/layout/text.rs`, `src/theme/typography/tests.rs`,
  `src/layout/panel/axis_frame_grid/tests.rs`, `src/render/geometry.rs`,
  `src/orchestrator/backend.rs`: mandatory breaks/spacing, selected metrics,
  fixed-angle cell/ink constraints and canonical frozen transforms.
- `src/layout/text/shaping_tests.rs`, `src/decorations/panel_text/tests.rs`,
  `tests/annotations/panel_text.rs`, `tests/lines/numeric_layers.rs`: original/derived
  Unicode source mapping, fixed-reference anchoring, selected-size metrics, actual
  annotation ink and clipped-versus-consumed source policy.
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

Review affected layer counts, including ignored tests, before and after each
test-changing step. Follow `specs/90_DEVELOPMENT.md`: at most 800 unit tests,
300 module integration tests excluding core E2E, and 80 core E2E scenarios.
Meaningful additions need not replace an existing test; remove redundant tests
and obsolete helpers. Do not evade limits by merging functions, packaging cases,
relabeling or moving responsibilities between suites.

### Focused Commands

Use the narrowest applicable selections at each code-changing step and confirm
nonzero execution. If a named test is deliberately replaced, use its replacement
filter. Do not claim discovery as behavioral execution.

```sh
cargo test --locked --offline --lib construction_edits_preserve_source_and_bulk_font_is_not_a_runtime_override
cargo test --locked --offline --lib owned_policy_edits_preserve_literal_affixes_and_semantic_precision_bounds
cargo test --locked --offline --lib fractional_quantization_matches_independent_small_dyadic_rationals
cargo test --locked --offline --lib decimal_emission_obeys_notation_precision_and_zero_occupancy
cargo test --locked --offline --lib finite_extremes_keep_exact_scaled_values_and_refuse_unrepresentable_output
cargo test --locked --offline --lib fractional_powers_certify_literal_cells_carries_refinement_and_cache_independence
cargo test --locked --offline --lib semantic_sets_resolve_uniform_precision_preserve_occurrences_and_exhaust_powers
cargo test --locked --offline --lib common_step_admits_complete_counts_and_exact_integral_indices
cargo test --locked --offline --lib additive_sources_are_requested_first_then_complete_decimal_coarsenings
cargo test --locked --offline --lib dense_and_empty_interval_misses_do_not_poison_coarser_sources
cargo test --locked --offline --lib count_targets_allow_nice_deviation_and_maxima_filter_before_formatting
cargo test --locked --offline --lib common_step_uses_index_products_not_accumulated_addition_or_endpoint_tolerance
cargo test --locked --offline --lib logarithmic_interval_vectors_are_divisible_integral_exponents_not_subsequences
cargo test --locked --offline --lib midpoint_is_conditional_and_must_itself_fit
cargo test --locked --offline --lib source_semantics_reject_before_fonts
cargo test --locked --offline --lib generation_copy_and_composed_identity_include_both_axes
cargo test --locked --offline --lib generation_rendering_and_exact_precedence_are_cache_independent
cargo test --locked --offline --lib categorical_trajectory_generation_refuses_before_fonts_in_standalone_and_canvas
cargo test --locked --offline --lib forest_log_interval_is_applied_in_exponent_space_and_exact_ticks_override_it
cargo test --locked --offline --lib assigned_cells_keep_positions_including_empty_labels
cargo test --locked --offline --lib prepared_fonts_keep_their_operation_across_clones
cargo test --locked --offline --lib sealed_census_reuses_one_normalization_per_stack_and_one_selection_per_pair
cargo test --locked --offline --test lines numeric_layers::point_layer_edits_validate_radius_and_preserve_exact_alpha_endpoints
cargo test --locked --offline --test lines numeric_layers::point_labels_and_whole_layer_legend_identity_remain_distinct
cargo test --locked --offline --test lines numeric_layers::clipped_point_labels_do_not_require_unpainted_glyphs
cargo test --locked --offline --test lines numeric_layers::public_tick_generation_yields_to_exact_labels_in_both_setter_orders
cargo test --locked --offline --test lines numeric_layers::public_tick_generation_counts_replace_complete_choices_and_automatic_resets
cargo test --locked --offline --test heatmap local_and_shared_guides_publish_the_same_distinct_close_numeric_labels
cargo test --locked --offline --test heatmap heatmap_annotations_use_default_and_chained_formats_with_contrasting_text
cargo test --locked --offline --test heatmap heatmap_wrap_policy_preserves_complete_crowded_axis_labels
cargo test --locked --offline --test heatmap heatmap_reserves_long_y_labels_and_elides_crowded_x_labels
cargo test --locked --offline --test estimates model_displays::coefficient_fixed_value_labels_are_point_only_and_default_is_inert
cargo test --locked --offline --test estimates model_displays::forest_log_scale_renders_raw_power_ticks_and_rejects_nonpositive_values
cargo test --locked --offline --lib mandatory_breaks_preserve_crlf_and_all_blank_paragraphs
cargo test --locked --offline --lib whitespace_is_preserved_and_contributes_logical_advance_not_ink
cargo test --locked --offline --lib typography_reuses_analysis_and_keeps_complete_sources
cargo test --locked --offline --lib reflow_reuses_analysis_through_unwrapped_wrapped_and_blank_paragraphs
cargo test --locked --offline --lib selected_soft_hyphen_preserves_same_face_gpos_and_original_clusters
cargo test --locked --offline --lib script_extensions_brackets_and_leading_marks_use_paragraph_context
cargo test --locked --offline --lib origins_round_exact_cumulative_positions_once_and_ink_uses_those_origins
cargo test --locked --offline --lib text_anchors_and_rotation_follow_exact_planned_ink
cargo test --locked --offline --lib selected_role_padding_is_retained_for_fit_and_reassignment
cargo test --locked --offline --lib heading_instructions_are_role_local_order_independent_and_frozen
cargo test --locked --offline --lib unbreakable_category_fails_intact_and_cannot_rotate_or_drop
cargo test --locked --offline --lib parent_width_keeps_natural_category_text_or_wraps_only_at_legal_boundaries
cargo test --locked --offline --lib canonical_label_fit_accepts_touching_ink_and_rejects_one_quantum_beyond
cargo test --locked --offline --lib frozen_text_and_brackets_never_relayout_or_convert_during_count_replay_and_outline
cargo test --locked --offline --test rendering labels::direct_plot_humanizes_axes_categories_legend_title_and_entries
cargo test --locked --offline --test rendering labels::raw_mode_preserves_automatic_identity_text
cargo test --locked --offline --test rendering labels::ambiguous_display_labels_fail_during_preflight
cargo test --locked --offline --test annotations panel_text::panel_text_renders_in_named_clipped_layer_with_theme_style_and_corner_anchors
cargo test --locked --offline --test annotations panel_text::panel_text_rejects_colliding_inside_legend
cargo test --locked --offline --test annotations panel_text::separately_valid_panel_annotations_reject_pairwise_overlap_before_begin
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
cargo test --locked --offline --test composition canvases::shared_category_labels_reject_different_row_domains_preflight
cargo test --locked --offline --test composition facets::faceted_explicit_category_order_validates_globally_and_filters_panel_subsets
cargo test --locked --offline --test output multiformat::plot_and_canvas_svg_operations_keep_complete_text_and_call_local_replay
cargo test --locked --offline --lib layout::kernel::tests::
cargo test --locked --offline --lib layout::interior::sizing::tests::
cargo test --locked --offline --lib canvas::sizing::tests::
cargo test --locked --offline --lib zero_sizing_requests_refuse_before_font_discovery_or_backend_begin
cargo test --locked --offline --lib standalone_terminal_resolves_and_plans_once_with_one_final_lifecycle
cargo test --locked --offline --lib chained_terminals_reuse_one_canonical_scene_and_plan_each_png
cargo test --locked --offline --test layout root_sizing::standalone_sizing_calls_replace_the_complete_request_before_publication
cargo test --locked --offline --test layout root_sizing::automatic_roots_expand_for_complete_unbreakable_text_and_exact_width_refuses_it
cargo test --locked --offline --test layout root_sizing::owning_facets_keep_sparse_wrap_and_complete_prose_in_all_public_root_modes
cargo test --locked --offline --test layout root_sizing::matrix_root_keeps_native_cells_hue_guide_and_complete_prose
cargo test --locked --offline --test layout root_sizing::mixed_canvas_keeps_complete_content_and_ignores_standalone_child_requests
cargo test --locked --offline --test layout root_sizing::all_text_canvas_reflows_complete_content_and_replaces_root_requests
cargo test --locked --offline --test composition facets::faceted_shared_axis_titles_preserve_explicit_multiline_text
cargo test --locked --offline --test composition facets::shared_facet_axis_titles_clear_outside_legends
just fmt-check
just lint
```

Run newly written/replacement numeric, locator/category, callback/font,
text/anchor/rotation/ellipsis/font-fit, span and maximum-width witnesses and affected
callers; the commands above are not substitutes for new-contract coverage.
Select existing guide/cache neighbors only when their dependency paths change or
a failure exposes an interaction. Execute affected public doctests and `just docs`
when adding API documentation/examples. Keep real SVG/PNG publication witnesses
representative; do not recreate family-by-root-mode replay cross-products or
boundary-sized raster/parser workloads where focused arithmetic checks suffice.

No automatic full-suite gate is required merely for naming or these feature
boundaries. Expand beyond the focused set for a demonstrated uncovered invariant,
an explicit request or the required dependency-change gate, following
[development](90_DEVELOPMENT.md). A new dependency must be directly declared,
locked/offline and justify the full verification gate; a locale/parser feature is
not permission to provision assets or read ambient settings. Documentation-only
planning requires source/reference consistency and `git diff --check`, not
compiler/test execution.
