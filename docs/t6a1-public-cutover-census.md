# T6A1 — Current Public Cutover And Consumer Census

## Basis And Reading Rules

This is the documentation census for T6A1, taken from the working tree after
T5e. It records implemented surfaces and their approved replacement owners;
it does not expose the target API or settle T6A2 construction decisions.

The subsequent construction decisions are closed in
`specs/94_T6_FOUNDATION_CONSTRUCTION.md`. Open A2 items below record this census's
handoff checkpoint, not remaining product decisions after that adjudication.

Authority: [spec93](../specs/93_T6_EXECUTION_PLAN.md), especially T6A1/A2,
the stage map, T6N and T6L;
[spec03](../specs/03_THEME_SYSTEM.md), including public foundations, component
boundaries, exact alpha, scales and fonts;
[spec05](../specs/05_LAYOUT_SYSTEM.md), including sizing, numeric sources,
font requirements and guide measurement; and the
[property ledger](../specs/04_VISUAL_PROPERTY_OWNERSHIP.md), including its T3j
adjudication overlay. Implemented public contracts remain
[spec01](../specs/01_ARCHITECTURE.md) and
[spec02](../specs/02_RENDERING_CONTRACT.md).

Source identifiers and function names below take precedence over historical
ledger line references. The ledger's historical counts are not counts of
current exports or new T6 policies. Its six classification names remain exactly
**Theme component**, **Theme data scale**, **Plot-local data scale**,
**Exact element style**, **Explicit policy**, and **Renderer invariant**.
Numeric presentation and legend composition do not silently add visual-property
records or alter those historical counts.

`Current` means reachable today, not merely spelled `pub` inside a private
module. `Cutover` means the leaf that makes the replacement public and migrates
callers; preparatory private leaves are separately identified. An explicitly
unassigned signature/deletion is a decision gap, not an invented leaf contract.
Existing tests are evidence locations, not newly executed verification.

Inspection covered source owners and unit fixtures, doctests, examples,
integration/benchmark/snapshot fixtures, README, docs and specifications.
Symbol searches and source inspection supply the inventories below; truncated
search output is not used as an exhaustive count. No test, compiler, build,
network or output-generation run
belongs to this documentation census.

## 1. Actual Current Crate-Root Exports

The following is the complete explicit root re-export inventory from
[`src/lib.rs`](../src/lib.rs). Grouping does not change any export's category or
ownership. A type whose fields change is still listed, rather than treating its
whole name as a deletion.

| Current root names | Implementation owner | T6 disposition |
| --- | --- | --- |
| `Plot` | [`src/builder.rs`](../src/builder.rs), `builder/{configuration,accessors,layout,facets}.rs`; lifecycle in `orchestrator/` | Retain type/erasure; replace affected inputs and getters at the leaves below |
| `Canvas`, `CanvasCategoryLabels`, `TextPanel` | [`src/canvas.rs`](../src/canvas.rs), `canvas/` | Retain types, category-label/content policy; new sizing/alignment/composition inputs have their own cutovers |
| `Error`, `Result` | [`src/error.rs`](../src/error.rs) | Retain standalone errors/result; no new error owner |
| `DEFAULT_FONT_FAMILY`, `DEFAULT_FONT_FAMILY_STACK` | [`src/font.rs`](../src/font.rs) | Both are current public raw string constants; typed replacement/retirement decision in §4 and §12 |
| `HistogramBin`, `HistogramDimensions`, `HistogramGeometry`, `HistogramKdePoint`, `HistogramRug`, `RaincloudDensityPoint`, `RaincloudDimensions`, `RaincloudGeometry`, `RaincloudGroup`, `RaincloudPoint` | [`src/geometry.rs`](../src/geometry.rs) | Retain caller-computed scientific geometry and identities; these are not root sizing getters |
| `AxisValues`, `HistogramInput`, `RaincloudInput` | [`src/input.rs`](../src/input.rs) | Retain scientific input representations |
| `Labels` | [`src/labels.rs`](../src/labels.rs) | Retain raw identity/display-label separation; section-aware validation is private T6L work |
| `BarPointLayer`, `GuideSpec`, `GuideStyle`, `PointLayer` | [`src/layers.rs`](../src/layers.rs) | Retain layer/guide identities and geometry; migrate tuple/opacity fields at T6B3 |
| `AxisTick` | Definition in [`src/layout/panel.rs`](../src/layout/panel.rs), constructor in [`src/layout/panel/axis_frame_grid.rs`](../src/layout/panel/axis_frame_grid.rs), re-exported through private `layout` | Retain exact position and verbatim label; no public layout module |
| `Output` | [`src/output.rs`](../src/output.rs) | Retain terminal publication; add frozen canonical dimensions at T6C2 |
| `Attached`, `FacetCell`, `Pairwise`, `Significance`, `SignificanceMark`, `SignificanceTarget` | [`src/significance/spec.rs`](../src/significance/spec.rs) through private `significance` | Retain typed placements, targets and caller marks; no numeric-format substitution for stars |
| `AxisScale`, `Bounds`, `CoefficientInput`, `EstimateStatus`, `InteractionStyle`, `LinePathStyle`, `LinePattern`, `PointShape`, `QqLine`, `RegressionBand`, `SignificanceMarkStyle`, `TextWeight` | [`src/spec/estimate.rs`](../src/spec/estimate.rs) | Retain scientific/visibility/closed glyph vocabulary; exact-style qualifications in §3; `AxisScale` is not `HeatmapScale` |
| `BarInput`, `BarSamples`, `Bootstrap`, `ConfidenceInterval`, `ErrorBar`, `PercentileInterval`, `StandardDeviation`, `StandardError` | [`src/spec/bar.rs`](../src/spec/bar.rs) | Retain scientific summaries and interval algorithms |
| `CategoricalColors`, `CategoryOrder`, `ValueSort` | [`src/spec/categorical.rs`](../src/spec/categorical.rs) | Replace only `CategoricalColors` at T6B4; retain ordering policies |
| `FacetScales`, `FacetSpec` | [`src/spec/facet.rs`](../src/spec/facet.rs) | Retain scoped domain/topology policy; propagate new presentation inputs through existing prepared scopes |
| `BarSpacing`, `CanvasLegendPolicy`, `DumbbellEndpoints`, `HeatmapAnnotations`, `HeatmapLabelFit`, `HeatmapLayout`, `HeatmapNullRowAnnotation`, `HeatmapScale`, `HistogramBins`, `HistogramComposite`, `HistogramKde`, `HistogramStat`, `LegendPolicy`, `LegendPosition`, `NumericLabelFormat`, `PanelCorner`, `PanelTextAnnotation`, `PlotKind`, `RawOverlay`, `WaterfallLabels` | [`src/spec.rs`](../src/spec.rs) | Replace endpoint tuple fields at T6B3, `HeatmapScale` at T6B4, `NumericLabelFormat` and `HeatmapAnnotations::Values` payload at T6N6; retain the other names/policies |
| `PngCompression`, `PngOptions`, `PngSizing` | [`src/spec/output.rs`](../src/spec/output.rs) | Retain format-local output configuration; `PngSizing::Width/Height` is not root sizing |
| `Theme` | [`src/theme/resolved.rs`](../src/theme/resolved.rs) through private `theme` | Replace closed enum with complete immutable value at T6B4 |
| `BooleanValues`, `CategoryValues`, `NumericValues`, `SourceRows` | [`src/values.rs`](../src/values.rs) | Retain scientific values, normalization and original row identities |

The only public root modules are [`kind`](../src/kind.rs) and
[`stats`](../src/stats.rs). `stats::{distribution,regression}` remain deliberate
display-statistics extension points. `kind` currently exposes `Any`, all concrete
markers named in §6, `Marker`, `ExactFamily`, and these sealed capabilities:
`SourceRows`, `Hue`, `Group`, `Series`, `ContinuousColor`, `NumericX`, `NumericY`,
`Guides`, `HeatmapOptions`, `CategoryOrder`, `SignedBars`, `ZeroLine`,
`EstimateStatus`, `Markers`, `CoefficientOptions`, `CategoricalColors`,
`PanelText`, `PointOutline`, `SignificanceStyle`, `Significance<P>`, `Legend`,
`RawOverlay`, `Facet`. There is no capability implementation for `Any` beyond
`Marker`.

There is no current root `Color`, `OpaqueColor`, `FontFamily`, `FontStack`,
`ThemeBuilder`, public component theme, palette/gradient value, `TickInterval`,
`TitleAlignment`, `NumericFormat` or `LegendComposition`. `SYSTEM_FONT_FAMILIES`
is not a current export or definition: its ledger entry describes an earlier
implementation. Private `Rgba`, `Rgb`, `ResolvedTheme`, `TextAppearance`,
`StrokePattern`, backend `TextStyle`/`BracketStyle`/`GradientStop`, `LegendEntry`,
`PanelBox`, `RootSize` and layout constraints are not hidden public exports.

## 2. Replacement And Exposure Registry

| Surface / ledger class | Current storage and implementation owner | Target owner | Exact cutover and migration/deletion |
| --- | --- | --- | --- |
| Built-in selection; routing **Renderer invariant**, appearance in owned components | `Theme` enum; `define_themes!` exhaustive `resolve` match; private preset functions in `theme/resolved.rs`; optional `Plot.theme`, mandatory `Canvas.theme` | Complete immutable `Theme`; construction-only `ThemeBuilder`; private owner resolution | T6B1 retains validated foundational values; T6B2 constructs/resolves complete themes; **T6B4** changes public `Theme`, preset calls, attachments and all callers; delete closed selector/match as runtime preset identity, not built-in constructor data |
| RGBA exact paint, **Exact element style** | `GuideStyle`, both point layers, `DumbbellEndpoints`; conversions in `layers.rs`, `spec/estimate.rs`, `plots/dumbbell.rs`, backend `Rgba` | Public `Color`, exact independent paint/alpha | Private validated color in T6B1; **T6B3** exposes the Color needed by the changed public signatures, migrates fields/callers and deletes tuple/opacity routes; complete remaining foundation exposure closes at T6B4 |
| Font and complete text defaults, **Theme component**; face selection **Renderer invariant** | Raw strings in `TextAppearance.stack` and legacy `ResolvedTheme.font_family`; `font/canonical.rs` parser and pair resolution | `FontFamily`, `FontStack`, complete `TextTheme` roles in `TypographyTheme` | T6B1 storage, T6B2 complete construction, **T6B4** public typed font/components; raw constant replacement spellings and sentinel cleanup require A2 pinning (§12) |
| Theme default mappings, **Theme data scale** | `ResolvedTheme.colors`, private defaults in `theme/scale.rs`; `CategoryColors`, `ContinuousColors` | `ColorScales`, `QualitativePalette`, `SequentialGradient`, `DivergingGradient`, `OpaqueColor` | T6B1/B2 single storage; **T6B4** public construction and attachment migration; remove old default-scale carriers after moving every producer |
| Primary/supplemental accents, **Theme component** | Same `ResolvedTheme.colors` seeds, separately resolved `AccentColors` | `MarkTheme` with independent `AccentPalette` | T6B1/B2 separate complete value ownership; **T6B4** expose; delete shared legacy seed carrier as the source of both responsibilities |
| Exact keyed mapping, **Plot-local data scale** | `CategoricalColors.entries`; `Plot::categorical_colors`, `CategoryColors::resolve` | `CategoricalColorMap` / `CategoricalColorScale` (map or palette) | **T6B4** replaces current root type/setter/storage and migrates raw keys; setter spelling/capability-name decision remains A2 |
| Continuous local selection, **Plot-local data scale** | Heatmap `HeatmapScale`, separate color limits, private gradient vectors; Beeswarm separate color limits/private gradient | `ContinuousColorScale` with optional typed gradient replacement, shared Heatmap/Beeswarm applicability | **T6B4** removes old scale representations and migrates attachments/accessors/prepared/scoped copies; division between new scale and existing `color_limits` setters needs A2 |
| Root horizontal alignment, **Theme component** plus narrow **Explicit policy** instructions | Private `TitleAlignment`, `CompositionAppearance` alignment fields, optional Plot/Canvas instructions, `RootAlignments::resolve` | `CompositionTheme` concrete title/caption and optional subtitle defaults; four approved overrides | **T6B5** exposes Plot title/subtitle and Canvas panel title/subtitle setters and caption alignment. Component/type reachability at preceding B4 needs a signature decision (§12); no public Canvas-root override or TextPanel alignment is specified |
| `Canvas::shared_legend_gap`, ledger target **Theme component**, conflicting legacy exact-override relationship | `Canvas.shared_legend_gap: Option<f64>`; `canvas.rs` validates and uses it over `LegendMetrics.shared_gap` | `LegendTheme` shared spacing; exact override disposition requires reconciliation | Ledger assigns public theme connection to T6, but spec93 gives no exact removal leaf; **unassigned pending A2**. Reconcile the remove-override and retain-override ledger statements (§12), including callers and policy validation |
| Preferred interval, **Explicit policy**, source order **Renderer invariant** | Private enum `layout/panel/sources.rs::TickInterval`; `AxisSpec.interval`; `Sources` and frozen `TickSource` | Opaque validated root `TickInterval` | **T6C1** exposes `additive`/`log10_decades` and marker-gated setters; remove public access to raw variants by keeping representation private; retain exact-tick precedence |
| Root size request, **Explicit policy**, allocation **Renderer invariant** | Exact Plot `width/height`, Canvas pair, `size`; private optional requests via lifecycle/Canvas planning | One automatic / width / exact sizing request; frozen `Output` dimensions | **T6C2** atomically changes omission defaults, consuming width setters, size replacement, removes Plot getters, adds Output getters and migrates internal/test/doc callers; C3 closes parent/output evidence |
| Numeric spelling, semantic **Explicit policy**; conversion/admission **Renderer invariant** | `NumericLabelFormat`, source-formatting functions, fixed Waterfall totals, generated axis/guide grammar | One `NumericFormat` engine and normalized effective policies | N1/N2 certified core; N3 replaces private format implementations; N4a axes, N4b guides, N4c values/totals; N5 identities/composition; **T6N6** exports format/attachments and deletes `NumericLabelFormat` and old public callers; N7 closes evidence |
| Discrete semantic composition, **Explicit policy**; keys/flow/cache **Renderer invariant**, chrome/text **Theme component** | Flat `(title, Arc<[LegendEntry]>)`, flat facet fields/selected lists, exact composite glyphs | One ordered typed section descriptor; `LegendComposition` independent of appearance/placement | **T6L1** replaces flat representation and moves Combined consumers/layer identity; L2 projections, L3 flow, L4 parents/fonts/keys; **T6L5** public enum/Plot/Canvas setters and current-contract caller migration; L6 evidence |

B6, C3, N7 and L6 are their respective evidence closures, not delayed deletion
owners. T6Z1 reconciles examples/current specifications and the property census
after the owning public cutovers; it must not become the first migration of a
replaced public caller.

## 3. Exact Styles: Actual Signatures And Narrow Precedence

| Current public surface | Actual behavior / downstream owner | T6 disposition |
| --- | --- | --- |
| `GuideStyle::new(color: (u8,u8,u8), width: f32, pattern: LinePattern) -> Result<Self, Error>` | Private fields; width finite and strictly positive; `GuideStyle::line_style` calls private `LinePattern::line_style`. Guide projection/capture in `plots/scatter.rs`, `plots/line.rs`, `plots/shared_x/prepared.rs` and coordinate producers; exact stroke does not borrow theme color | B3 replaces RGB input with complete `Color`; retain width/pattern/GuideSpec geometry. No current public color getter, opacity or font control to delete |
| `PointLayer::new(NumericValues, NumericValues) -> Self`; `.color((u8,u8,u8)) -> Self`; `.radius(u32) -> Result<Self,Error>`; `.opacity(f64) -> Result<Self,Error>`; `.labels(CategoryValues)`, `.legend_label(impl Into<String>)` | Color optional, radius 4, opacity 0.8; opacity accepts finite `(0,1]`. Conversion creates a plain point without outline, quantizes opacity once to replacement fill alpha. `PreparedLine` owns coordinates, point labels and optional layer legend. An exact color still consumes its append-order accent slot | B3 removes tuple/opacity field/control, resolves complete exact color/alpha over automatic accent. Radius/alpha **defaults** belong to MarkTheme (`layers.point.default-radius/default-opacity`); caller edits remain Exact element style. A2 specifies selective edits and absent-override resolution; retain data, labels, radius, append slots and actual role paint |
| `BarPointLayer::new(category, Option<CategoryValues>, value, color: (u8,u8,u8), radius: u32, opacity: f64) -> Result<Self,Error>`; getters `category_values`, `series_values`, `values`, `color() -> (u8,u8,u8)`, `radius() -> u32`, `opacity() -> f64` | Immediate radius >0 and finite `[0,1)` opacity validation: the actual range check accepts 0 and rejects 1, despite its strict-between documentation; exact plain point, no outline and no legend. Prepared ordinary bar centers/series in `plots/bar_data.rs` | B3 migrates constructor, color/opacity storage and getters together. Target alpha includes both transparent and opaque endpoints. Exact target constructor/accessor spelling is A2, not an automatic conversion to a second legacy API |
| `DumbbellEndpoints::new(left_label, left_color: (u8,u8,u8), right_label, right_color: (u8,u8,u8)) -> Result<Self,Error>`; `left_label/right_label -> &str`, `left_color/right_color -> (u8,u8,u8)` | Nonempty trimmed control-free distinct labels. `plots/dumbbell.rs` applies endpoint point radius/outline and private point alpha, constructs matching literal guide keys. Current opaque tuple is not complete RGBA replacement | B3 changes both constructor paints/getters and final mark/guide conversion to `Color`; preserve identities, neutral connector and point geometry. Do not multiply exact alpha by the point-role alpha |
| `SignificanceMarkStyle::new(font_size: u32, weight: TextWeight, gap: f64) -> Result<Self,Error>`; `.font_size`, `.weight`, `.gap`; `Default` | Size >0; gap finite >=0; defaults 16/Normal/8px. **No public color, opacity or font-family field.** `effective_significance_label` retains role stack/paint/line height and applies exact size/weight. Current `significance_mark_style_spec` supplies DEFAULT when absent, hiding the gap default in a local-style fallback | Retain exact typography/clearance and its gate. The unconfigured 8px label-gap source belongs to AnnotationTheme (`annotations.significance.gap-default`), while an explicitly supplied gap is Exact element style. B1/B2 must preserve absent-versus-exact scope; no new per-mark family/paint setter is authorized |
| `InteractionStyle::new(group: impl Into<String>) -> Self`; `.shape(PointShape)`, `.pattern(LinePattern)`; getters `group`, `point_shape`, `line_pattern` | **No RGB, alpha, size or font-family input.** Constructor is currently infallible and does not lexically validate group; prepared `resolve_trace_styles` rejects unknown/duplicate/incomplete groups. Shape/pattern replace only those fields; mapped paint, radius, interval/connector widths, outline/alpha remain role-owned | Retain exact glyph controls; B4 exposes their default role owners and L2 uses producer-owned projections. Ledger requires lexical validated selector construction at T6, but spec93 does not assign this signature correction to a leaf. A2 must assign it; do not invent a B3 Color field |
| `CategoricalColors::new(impl IntoIterator<Item=(String,(u8,u8,u8))>) -> Result<Self,Error>` | Nonempty exact map; normalized raw labels, unique/nonempty/trimmed/control-free; private admitted lookup; unused keys and repeated colors allowed, missing used keys reject | B4 replaces with `OpaqueColor` map/scale owner, not RGBA element paint; retain alias-independent key semantics |
| `HeatmapScale::{Sequential,Diverging { center: f64 }}`; Heatmap `.scale`, `.color_limits`; Beeswarm `.color_limits` | Continuous data selection, not exact element style; automatic/exact domains and missing-neutral role resolve before fonts | B4 typed continuous replacement on actual continuous-capable families; `AxisScale::{Linear,Log10}` and Forest `.scale` remain scientific axis controls |

[`render/backend.rs`](../src/render/backend.rs) already has private complete
`Rgba`, `quantized_alpha` (nearest-even `opacity * 255`), independent point
outline/fill and fill/stroke paints. Exact B3 migration must use that one paint
route, remove floating opacity naming/storage/conversion for migrated callers,
and retain independent stroke alpha. Spec03 says an exact `Color` replaces
complete RGBA; a separate alpha edit, if exposed, replaces color alpha regardless
of call order. It does not prescribe every existing style's constructor signature
or authorize data-mapped alpha.

RGB-looking tuples in `GuideSpec::Segment` endpoints are `(f64,f64)` scientific
coordinates and remain. Numeric limits, category centers, density widths,
statistical intervals and closed patterns are not paint deletions.

## 4. Theme, Components And Font Policy Owners

### Closed Presets And Current Owner Resolution

`define_themes!` in `theme/resolved.rs` generates exactly
`Theme::{Publication,Nature,Science,CHI}` and the exhaustive private match to
`publication_theme`, `nature_theme`, `science_theme`, `chi_theme`. They construct
one `ResolvedTheme` through `resolved_theme`; there is no public resolved-preset
inspection. T6B4 replaces these enum calls with
`Theme::{publication,nature,science,chi}()` complete constructors. Code relying on
`Copy`, enum matching or preset-identity debug strings must migrate as well as
constructor spelling. `Clone` complete values are not a new runtime mode.

Current resolution call families:

- `builder/configuration.rs::theme` stores explicit Plot selection;
  `builder/layout.rs::canvas_layout` resolves the owner-local default for its
  private helper; `orchestrator/lifecycle.rs` resolves standalone selection and
  shares prepared fonts; `builder.rs` erasure and admitted clones preserve it.
- `canvas.rs::ResolvedCanvasThemes::new` resolves one root and explicit child
  replacements; inherited children share the root. `Canvas::theme` stores the
  selection. Canvas titles/headers/text panels/shared-guide chrome belong to
  Canvas, while local Plot prose/marks/axes and facet children belong to Plot.
- `orchestrator/facets.rs` and `orchestrator/facets/{planning,rendering,sizing}.rs`
  use the owning Plot resolved value. Matrix remains a Plot-owned specialized
  root, not a separate theme scope.
- Theme unit preset arrays/matches (`expected_preset`, `expected_theme`) and
  producer/unit fixture preset literals need B4 migration; they are real callers,
  not additional public selectors. Preserve exact built-in appearance evidence.

All four private built-ins currently set title alignment Center, optional
subtitle alignment None and caption alignment Start. The private
`RootAlignments::resolve` follows explicit Plot instruction, Canvas panel
instruction when embedded, then the complete Plot theme for each heading;
subtitle falls back to the final effective title only when all subtitle
instructions are absent. Root caption uses its independent theme field.
`Theme::customize()` / `ThemeBuilder::build()` are target construction-only
operations at B4, not current methods or runtime component merging.

### Actual Private Component Records To Derive A2 From

All rows below are **Theme component** except the explicitly separate data-scale
row. Complete public construction/accessor signatures belong to A2; these private
records are owner evidence, not permission to re-export backend structs.

| Target owner | Current complete/private records and consumers | Single-representation migration |
| --- | --- | --- |
| `SurfaceTheme` | `SurfaceAppearance::{canvas_root,plot_panel}`; `canvas_surface_override` carries composed backdrop context; SVG surface operands and contrast selection | Validated public paints at B1/B2; B4 exposure. Preserve transparent root/panel composition; context override is not a public theme field |
| `TextTheme`, `TypographyTheme` | `TextAppearance::{stack,paint,font_size,weight,line_height_multiplier}`; `FoundationalTypography::{title,subtitle,caption,axis_title,tick_label,significance_label,estimate_detail,waterfall_total,point_layer_label,panel_text,contrast_light,contrast_dark,legend_title,legend_label}` | B1 typed stack/paint and validated metrics; B2 complete role construction; B4 exposure. Exact significance size/weight wins; contrast light/dark text and planning backdrop have opacity constraints |
| Additional typography roles | Legacy `font_family`, `foreground`, `detail_font_size`, `label_font_size`, `composition_title_font_size`, `facet_header_font_size`; `legacy_text_appearance` feeds Canvas TextPanel title/body, composition/facet headers and remaining legacy text consumers | A2 must map admitted body/detail/facet/header/Canvas roles completely, move consumers to named roles at B1/B2, then delete generic legacy carriers. Current private role list alone is not the complete spec03 public inventory |
| `AxisTheme` | `AxisAppearance::{frame,ticks,grid,spacing}`; `FrameAppearance` includes `source_show_top_right_spines`; stroke paint/width/private pattern, tick length, six axis-spacing fields; `PanelAxisPlan` and Matrix axes | B1/B2 validate/adjudicate complete sides/strokes and relative relationships; B4 exposure. Frame topology must follow current/overlay semantics, not expose misleading historical flag as a public component |
| `LegendTheme` | `LegendAppearance` padding, key width, key-label/outside/shared gaps, gradient min/preferred length, row/column/title gaps, frame fill/outline/width/radius; `LegendMetrics` resolves owner-local measurements | B1/B2 complete construction; B4 exposure. Continuous/discrete chrome share owner; L3 section gap reuses `title_gap + row_gap`, not a new spacing field |
| `CompositionTheme` | `CompositionAppearance` title/optional subtitle/caption alignments, `RootTitleBlockAppearance` three gaps/clearances, TextPanel inset/gap, facet/grid gaps, column/row header padding and row-title horizontal/vertical padding | B1/B2 validated relative relationships; B4 components, B5 alignment public contract. Root/component geometry stays private. Resolve optional instructions before measurement |
| `MarkTheme` | `MarkAppearance` primary/raw-overlay points and sizes; bar/line-band/regression-band/density/marginal/Matrix/box/violin/histogram/Waterfall fills; fixed Raincloud fill; cell/highlight/missing outlines and missing fill/cross; zero/reference/connector/summary/uncertainty/path/curve/rug/interval stroke roles; `DataFillAppearance`, `FixedFillAppearance`, `FillOutlineAppearance`, `PointAppearance`, `PointSizeAppearance`, `DataLineAppearance`, `StrokeAppearance` | B1/B2 role-based validated values, not one component per plot. B4 expose admitted field taxonomy and separate `AccentPalette`; private pattern/outline structures need closed public input decisions in A2 |
| `AnnotationTheme` | `AnnotationAppearance` bracket stroke, point-label gap, Waterfall shelf padding/gap/row-gap, opaque planning backdrop; exact significance gap remains separate | B1/B2 validate non-text geometry/relative relationships and backdrop; B4 expose. Text metrics/paint remain Typography; no target/placement/statistics fields |
| `ColorScales` (**Theme data scale**) | `ResolvedTheme.colors` plus `theme/scale.rs` default gradient stops; opaque mappings and admitted palette cache | Separate default qualitative/sequential/diverging values from Mark accents at B1/B2; B4 exposure and old carrier deletion |

Private spacing fields are currently resolved pixel-like scalar values. A2 must
derive the spec03 dimensionless relationships from actual consuming metrics and
the active ledger overlay; blindly publicizing all existing `f64` fields would
not satisfy construction/ownership or remove legacy storage.

### Both Raw Font Constants And Exact Family Controls

Current exact public values:

```text
DEFAULT_FONT_FAMILY = "Noto Sans"
DEFAULT_FONT_FAMILY_STACK = "Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif"
```

`DEFAULT_FONT_FAMILY_STACK` currently supplies built-in `resolved_theme` role
stacks and expected-theme unit fixtures. Direct repository uses include
`src/layout/text.rs` unit fixtures,
[`tests/benchmarks/fonts.rs`](../tests/benchmarks/fonts.rs) and
[`tests/rendering/public_contracts.rs`](../tests/rendering/public_contracts.rs)
(`chinese_text_uses_system_font_fallback_stack_without_embedding`).
`DEFAULT_FONT_FAMILY` is also a private sentinel comparison in
`orchestrator/backend.rs::{stack_for,plan_raw_bracket}` and
`render/geometry.rs::plan_raw_text`, selecting `operation_default_stack` when a
raw style carries that token. These comparisons are current code even though
comments still describe the earlier point-layer migration.

There is **no current public exact font-family/stack setter** on Plot, Canvas,
TextPanel, GuideStyle, either point layer, DumbbellEndpoints, Significance or
InteractionStyle. Significance's exact size/weight/gap is not a family control.
Private backend `TextStyle.font_family` and `BracketStyle.font_family` do not
make such controls public. Current line labels already consume
`typography.point_layer_label` for stack/size/weight/leading, but `plots/line.rs`
sets their paint to the resolved layer point's full `style.color`, including
automatic accent paint and quantized alpha, not just an explicitly configured
color. A2 must preserve or explicitly adjudicate that current text-paint precedence
against the complete role contract. Significance labels consume the effective role;
legend text uses named legend roles, not a direct raw-constant bypass.

Target role policy is complete `TextTheme` stacks, with optional construction-only
bulk stack edits through `ThemeBuilder`. `FontFamily` admits validated named or
serif/sans-serif/monospace generic entries; `FontStack` is nonempty/ordered, not
arbitrary CSS. `font/canonical.rs` owns the current family parser/preferences,
canonical `(stack,weight)` pairs, host-source admission, face ordering and
retained base-plus-mark selections. B1 moves to typed retained inputs, B4 makes
the public types current; it must not add fonts or new discovery routes.

The ledger assigns `typography.builtin-font-stack.current-public-constant` to
typed font replacement and records both current raw constants as T6 retirement
work. Spec03 specifies typed fonts but no replacement public constant/profile
factory. A2 must decide typed profile access (or role access via built-in Theme),
the precise B4 deletions and removal of sentinel semantics. Do not delete these
root exports blindly or retain them as aliases to an unvalidated second route.

## 5. Actual Data-Scale Consumer Matrix

Current complete consumer evidence is
[`docs/t5d5-scale-closure.md`](t5d5-scale-closure.md); the earlier
[`t5d0 census`](t5d0-scale-consumer-census.md) is historical. Implementation owners
are [`theme/scale.rs`](../src/theme/scale.rs),
[`theme/color.rs`](../src/theme/color.rs) and the named producers below.

| Families / current producer | Current raw domain, palette/accent/gradient route | B4 target handoff |
| --- | --- | --- |
| Scatter (`plots/scatter.rs`) | Observed finite hue raw keys; unkeyed primary accent zero | Categorical scale or independent Mark accent; same complete point key |
| Line, Trajectory, LinePath (`plots/line.rs`) | Path/row hue; neutral connectors separate; layers consume stable accents, including exact overrides | Categorical scale for actual hue, AccentPalette for unkeyed/layers; preserve neutral row-hue key semantics |
| Density (`plots/density.rs`) | Hue independent of graphical group; group-only curves accent zero | Same opaque color with independently retained curve/fill alpha |
| Bar, HorizontalBar, StackedBar, StackedHorizontalBar (`plots/bar_data.rs`, `bar.rs`, `horizontal_bar.rs`, `stacked_bar.rs`) | Category when ungrouped, positioned series/hue otherwise; signed HorizontalBar uses two accent slots and typed sign keys | Palette/map semantics on actual domain; exact BarPointLayer paint remains narrower; sign mapping is not a categorical map |
| Boxplot, Violin, Raincloud (`plots/categorical_distribution.rs`, `boxplot.rs`, `violin.rs`, `violin/precomputed.rs`) | Category/positioned series, supplied Raincloud hue; common raw domain and independent body/strip/outlier paints | Same categorical map/palette owner on sampled/supplied paths; retain scientific inputs and opacity role splits |
| Interaction (`plots/interaction.rs`, `interaction/native.rs`) | Hue, or group when no hue; exact shape/pattern by group | Categorical scale for hue/group; producer retains provenance for L2, not color-equality inference |
| Marginal, Matrix (`plots/marginal.rs`, `matrix.rs`) | Parent hue domain or unkeyed accent; linked/diagonal densities and points share mapped RGB | Same categorical owner across cells/strips; no Matrix numeric global setter or new group-style mapping |
| Histogram, Qq, Regression, PointInterval, Coefficient, Forest | Unkeyed accent zero for applicable bins/observations/estimates/band; separate neutral reference/fit roles | MarkTheme AccentPalette; these do not acquire categorical setters simply by drawing colored marks |
| Waterfall (`plots/waterfall.rs`) | Positive/negative spans use independent accent slots | Mark accent owner; connector/reference/total typography remain separate |
| Dumbbell (`plots/dumbbell.rs`) | Exact caller endpoint tuples, matching literal guide keys | B3 complete `Color`; connector stays MarkTheme neutral stroke |
| Heatmap (`plots/heatmap.rs`, `heatmap/guide.rs`) | Sequential/diverging finite active-cell domain, exact saturation, constant midpoint/neutral, pre-font contrast; missing marks are neutral | ColorScales default plus `ContinuousColorScale` optional gradient/domain replacement; same scale for cells/guide/contrast |
| Beeswarm (`plots/beeswarm.rs`) | Sequential numeric color domain and retained sampled guide stops; x is independent | Same continuous attachment applicability; target divergence/custom gradient behavior requires complete scale construction, not just renaming HeatmapScale |
| Facets / Canvas Shared | Parent domain before slicing; row-major Canvas union includes hidden-entry plots with visible marks; complete glyph compatibility; final continuous domain installed in children | Propagate selected typed values once, retain exact maps/unused keys, final-domain stop sampling and producer-owned glyphs; no Canvas recoloring |

The current continuous defaults in `theme/scale.rs` are separate
`HEATMAP_SEQUENTIAL_STOPS`, `HEATMAP_DIVERGING_STOPS`,
`BEESWARM_SEQUENTIAL_STOPS`; current private replacements are
`Plot.heatmap_gradient` / `beeswarm_gradient` with test-only setters. Public
gradients must consume `OpaqueColor`, not public backend `GradientStop`.
Current diverging descriptor logic requires three semantic stops with its middle
stop at 0.5; spec03's ordered gradient construction needs an explicit A2 shape
contract and owning B-stage migration if it is broader than that representation.

## 6. T6 Public Bindings Against Actual Sealed Capabilities

[`src/kind.rs`](../src/kind.rs) is the current sealed matrix. The following lists
are the exact current implementations relevant to new T6 bindings, not inferred
from `PlotKind` runtime dispatch:

| Capability | Current concrete markers |
| --- | --- |
| `NumericX` | Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Coefficient, Forest, Dumbbell, HorizontalBar, StackedHorizontalBar, Regression, Marginal, Beeswarm, Waterfall |
| `NumericY` | Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath, Interaction, Bar, StackedBar, Boxplot, Violin, Raincloud, Regression, Marginal |
| `ContinuousColor` | Heatmap, Beeswarm |
| `CategoricalColors` | Scatter, Line, Trajectory, LinePath, Density, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Interaction, Violin, Raincloud, Marginal, Matrix |
| `Legend` | Scatter, Density, Line, Trajectory, LinePath, Interaction, Dumbbell, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Regression, Marginal, Matrix |
| `CoefficientOptions` | Coefficient |
| `SignificanceStyle` | Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Coefficient |
| `Guides` | Scatter, Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Regression, Raincloud, Marginal, Beeswarm, Waterfall |

All concrete markers are `ExactFamily`; `SourceRows` includes every concrete
marker. `Facet` includes all except Marginal and Matrix. Matrix cannot be embedded
in Canvas; Marginal has Canvas support but not generic facets. Raincloud supplied
geometry keeps its existing preflight restriction on optional positioned series.
No public format or composition setter is licensed on erased `Any` by a concrete
plot's runtime kind.

Binding choices required by the approved contracts:

- B4 `theme(Theme)` stays common on `Plot<K: Marker>` and Canvas. Typed palette/map
  attachment uses the existing categorical capability's **actual set**, not Hue
  alone (which would lose bars/distributions/Interaction group-only colors).
  Whether the capability retains the obsolete `CategoricalColors` name and the
  new setter's spelling are unpinned A2 choices.
- B4 continuous scale/gradient attachments use `ContinuousColor` for Heatmap and
  Beeswarm; Heatmap-only layout/annotations stay concrete/`HeatmapOptions`.
  Forest `.scale(AxisScale)` remains a distinct concrete scientific setter.
- B5 Plot alignment setters are common; Canvas panel defaults apply to composed
  Plot prose. The approved surface does not add alignment to TextPanel or arbitrary
  rotated/anchored text.
- C1 interval setters use `NumericX` / `NumericY`; the plan specifies constructor
  names `TickInterval::{additive,log10_decades}` but not all accessor/input
  signatures or setter spellings. Private `Log10Decades` stores `u64`; A2 must pin
  the opaque construction boundary rather than expose raw enum variants.
- C2 consuming `Plot::width(u32)`, `Canvas::width(u32)`, existing `size(u32,u32)`
  are common sizing operations; Output getters inspect the frozen scene.
- N6 `x_tick_format(NumericFormat)` and `y_tick_format(NumericFormat)` use
  `NumericX` / `NumericY`; `color_label_format` uses `ContinuousColor`;
  `estimate_labels` remains Coefficient-only (`CoefficientOptions` today),
  `total_format` is concrete Waterfall; `HeatmapAnnotations::Values` changes its
  payload. NumericFormat and proposed closed input discriminants are root values,
  not public backend/module extension points.
- L5 `legend_composition` uses `Legend` on Plot and is common on Canvas. Heatmap
  and Beeswarm have continuous guides but no `Legend` implementation; do not widen
  the discrete gate to them.

A capability means that the family can have that numeric axis, not that every
configured instance does. Trajectory can have categorical x; LinePath requires
numeric x.
Actual-axis applicability must
reject before fonts; exact tick lists suppress generated interval/format text
regardless of setter order. Matrix cell axes consume the private default numeric
engine but Matrix is absent from NumericX/NumericY and has no new global setter.

## 7. Root Dimensions: Getter Migration Versus Geometry

Current `Plot::size` replaces two `u32` fields (default 640×480).
`builder/accessors.rs::{width,height}` exposes these configured dimensions for
every marker including Any. Canvas defaults to 960×640 and has `size`, but no
current public zero-argument width/height getters. Output has only consuming
`svg`, `svg_preserve_text`, `png`, `png_preview`, `png_with_options`; it currently
has no dimension getters.

The private engine already accepts optional width/height in
`orchestrator/lifecycle.rs`, `canvas.rs::preflight_plan_with_sizing`,
[`layout/interior/sizing.rs`](../src/layout/interior/sizing.rs),
[`canvas/sizing.rs`](../src/canvas/sizing.rs) and
`orchestrator/facets/sizing.rs`. T6C2 must route one full request into that engine:
omission automatic, width exact/content-height, size exact pair, latest sizing
setter replaces the entire request. It must not preserve a previous exact height
after width, apply child standalone preferences to Canvas cells, or consult the
old default pair when automatic sizing misses.

Current `Canvas::validate_structure` additionally rejects configured roots below
160×120 on the public preflight route (`canvas.rs`). The active ledger overlay
explicitly retires `validation.canvas.minimum-size` in favor of complete content fit and
resource bounds. T6C2 must remove this surviving universal gate, retain positive
dimension validation at preflight, and test genuinely valid smaller roots as
well as failures from actual measured requirements. It is not a scientific child
minimum or an automatic sizing preference.

| Actual current Plot-getter call path | C2 migration owner |
| --- | --- |
| `orchestrator/lifecycle.rs` exact request, positive-size preflight, root construction, content-height assignment and final ink enclosure | Use request inputs before planning and selected root dimensions after planning; do not call public Output during planning |
| `orchestrator/facets.rs`, `facets/planning.rs` root title/frame allocation and final assignments | Use owning root request/frozen selected dimensions; facet cells remain parent allocated |
| `layout/interior/coordinate.rs` root frame setup | Consume current request/selected root size from lifecycle |
| `decorations/plan.rs` owned-ink overflow, required annotation/legend containment, root panel bounds and test helper | Use assigned containing geometry, not an infallible pre-render size |
| `decorations/plan/tests.rs`, `orchestrator/facets/parent_tests.rs`, `layout/interior/coordinate/tests.rs`, `plots/violin/precomputed/tests.rs` | Migrate concrete root-box assertions/backend begin fixtures; retain the underlying scientific/ink checks |
| `tests/rendering/public_contracts.rs::erased_plot_retains_common_operations_and_exact_output` | Remove exact/erased pre-render getter assertions; compare emitted/frozen Output canonical dimensions and erasure output parity |
| spec01/spec02 introspection/default-size statements; current examples/docs with omitted-size expectations | C2 current-contract migration; explicit `.size` stays exact, omitted roots must no longer promise historical defaults |

[`scene.rs`](../src/scene.rs) retains `CanonicalSceneSize.dimensions` from the
checked final `RenderedScene::new` pair and verifies finalized SVG dimensions.
This is the owner for new Output width/height access, not the PNG target plan.
PNG source/target dimensions, pixels and parser-byte gates remain publication
local; width/height getters continue reporting canonical CSS pixels after any
PNG operation or refusal, not DPI-scaled raster dimensions.

**Keep unrelated geometry/size uses:** `PanelBox::{width,height}` in layout,
Canvas, decorations, significance and producers; `ResolvedTextPlan` and text
blocks' measured extents; `RootSize`, `GeometryBox`, projection spans and selected
guide boxes; `PlotData::height` (row count); public scientific
`RaincloudDensityPoint.normalized_width`; SVG/PNG `IntSize` and raster target
aspect calculations. Representative lookalikes occur in
`tests/benchmarks/{text,annotations,legends}.rs`, `render/svg_backend.rs`,
`data/source.rs` and Matrix cells. A textual `.width()`/`.height()` replacement
would corrupt these call paths. README/docs/examples contain sizing calls, not a
public Canvas getter migration.

## 8. Numeric Presentation And Mathematical Consumers

| Current owner and consumers | Current operand/default | Private migration / public cutover |
| --- | --- | --- |
| `layout/panel/sources.rs::{NumericTickPlan::measure_in_call,format_label,format_label_parts}`; axes in `axis_frame_grid.rs`; terminal intrinsics in `layout/interior/coordinate.rs`; Matrix/facet/shared axes consume the same private grammar | Ordinary finite positions; magnitude grammar: scientific at >=1e12 or nonzero <0.01, M at >=1e6, k at >=1e4, fractional 0 at >=100, 1 at >=1, 2 below. Fixed trailing fraction zeros trim, signed zero normalizes; whole-source collision retry uses scientific precision 2..=17 | N3 one built-in profile; N4a policy-derived intrinsics/axis sources, preserving old default operation order; N6 NumericX/Y attachments |
| Forest source positions in `sources.rs`; `plots/interval_estimate.rs`, `shared_x/prepared.rs`; `numeric.rs::log10` for scientific transformation | Exponent-space projection; integral seeds spell raw power literals (`0.01`, `0.1`, `1`, `10`, `100`, `1000`, `10k` etc.; other integrals `1.00e{e}`); adaptive/midpoint grammar is `10^` plus exponent format | N2a/b/c certifies configured `10^(exact(e)+scale)` decimal rounding; N4a integration. Preserve default grammar; do not replace scientific log10 with presentation or use powf to approximate configured display |
| `decorations/legend/descriptor.rs::{numeric_labels,generated_labels}` through `layout::format_guide_label`; Heatmap `prepare_guide_labels`, Beeswarm `prepare`/descriptor; `MeasuredContinuousGuide` | One to three finite domain endpoints/optional interior center; constant domain one label; signed zeros equal; whole-set unique default/2..=17 precision before fonts; Shared formats once after final union | N3 shared engine, N4b local/Shared policies and compatibility; N6 `color_label_format`. Required labels indistinguishable at exact precision reject, not arbitrary guide elision |
| `spec.rs::{NumericLabelFormat,NumericLabelDisplay}`; Heatmap `PreparedHeatmap` cell annotation preparation calls `format_source`; cell minima/contrast/font census/fit and frozen annotations | Public `new(usize)`, consuming `precision(usize)`, `suffix(Into<String>)`; precision 0..=12, suffix nonempty/control-free/literal; default two fractional digits, signed zero and trailing zeros retained. Only finite active prepared cell values; merged fully-null rows retain caller text | N3 migrate fixed values, N4c prepared adaptive panel set/minima/census; N6 delete format type and change `HeatmapAnnotations::Values` payload |
| `plots/interval_estimate.rs` prepared point-only Coefficient labels, intrinsics, attached-label geometry/capture | Same NumericLabelFormat; drawable finite estimates only; intervals plus `estimate_labels` reject; required unavailable marker is not formatted numeric data | N3/N4c one prepared estimate set; N6 retain concrete/`CoefficientOptions` restriction and replace format parameter |
| `plots/waterfall.rs::prepare`, `text_census_annotations`, `intrinsic_body_minimum`, `plan_annotations` | Complete strings currently built with `"{label} {value:.3}"`; source base/ordered terminal value, labels from WaterfallLabels or defaults; measured outside-mark shelves. Visible/windowed total policy stays owned by renderer | N3 preserve fixed-three built-in profile/name spacing; N4c two-total adaptive set and shelves; N6 new Waterfall `total_format`, WaterfallLabels stays name owner |

Test-only `range_ticks.rs::{format_tick,distinct_tick_labels,nice_ticks}` are
helpers around current grammar, not another production engine. `nice_range` and
its seed count/margins/step calculation are mathematical domain preparation;
`numeric.rs` exact log10 and decimal-power helpers are scientific/source
consumers, not label-format APIs. Statistics, color interpolation, coordinates,
SVG float serialization, diagnostic `format!` calls and source identifiers stay
with their current owners. No label is parsed back into data.

Current user-value formats have no notation, significant/adaptive precision,
prefix, scaling, negative-zero or trailing-zero control. T6N admits Decimal /
Scientific / Engineering; fractional/significant fixed/adaptive precision;
exact decimal scale -324..=324; literal prefix/suffix; explicit zero/zero-trimming
policies. Proposed discriminants in spec93 still require signature closure.
Percentage means scale 2 plus `%`; suffix `%` alone retains its current literal
semantics. Target `NumericFormat::default()` preserves fixed-value two-decimal
behavior; unconfigured axes/guides/Forest/Waterfall use private profiles of that
same engine rather than a public legacy switch.

Spec93's target construction is opaque
`NumericFormat: Clone + Debug + PartialEq + Eq`, with
`new(NumericNotation, NumericPrecision) -> Result<Self>`; proposed precision inputs
are `Fractional(u8)`, `Significant(u8)`,
`AdaptiveFractional { min: u8, max: u8 }`, and
`AdaptiveSignificant { min: u8, max: u8 }`. Fractional 0..=17, significant 1..=17,
adaptive min <= max, and decimal scale `i16` in -324..=324 validate. Consuming
notation/precision/decimal-scale/prefix/suffix/negative-zero/trailing-zero edits
replace only their own input. `NegativeZero::{Preserve,Normalize}` and
`TrailingZeros::{Keep,Trim}` are proposed closed discriminants; scalar getters
follow A2's validated-value convention. N0 closes final signatures, N6 is their
public cutover. Configured Forest exponent 0.5 at fractional precision 2 must
display `3.16` (scale 2 plus `%` gives `316.23%`), not `10^0.5`; precision applies
to the certified mathematical power, with exponent-space source identity intact.

Cross-surface owners affected by N4/N5:

- `layout/text/requirements.rs`: current `NUMERIC_ALPHABET` and exact tick/value
  spans; configured grammar and real affixes must contribute finite boundary
  units before fonts, including numeric scalar plus suffix combining marks.
- `font/canonical.rs` and `layout/text.rs`: retain whole-unit/scalar fallback and
  shaped text from those actual requirements; no guessed numeric samples.
- `builder.rs` admitted cloning/erasure, facet scoped copies, prepared bars/
  estimates/specialized sources and axis source identity: retain normalized
  effective format and operand, never rely only on equal domain/label lengths.
- `decorations/plan/children.rs`, `canvas/sizing.rs`, coordinated-domain/feedback
  keys and `theme/resolved.rs` text query keys: changed policy invalidates even
  same-width/same-domain reuse; root natural width is chosen once.
- `ContinuousContributor` / `ContinuousDescriptor` and Canvas final-domain
  installation: pre-font normalized policy equality; no first-contributor format.
  Canvas range sharing may retain differing panel spelling; a truly shared
  displayed axis needs compatible policy.

Exact tick strings, category/legend aliases, point labels, significance stars,
panel text, fully-null-row annotations and Waterfall names remain verbatim. No
new annotation surface or Matrix global numeric configuration is admitted.

## 9. Flat Discrete Guide: Producers, Shared Callers And Cache Hits

### Current Representation And All Owner Families

[`decorations/legend/descriptor.rs`](../src/decorations/legend/descriptor.rs)
defines `PreparedDiscreteGuide = (String, Arc<[LegendEntry]>)` and
`PreparedGuideDescriptor::{Discrete,Continuous}`. This is a private flat Canvas
descriptor, not a root export. Flat facet storage is
`orchestrator/facets.rs::PreparedFacetGuide` (title/entries/hue levels), and
`PlotRenderResult.legend_entries` plus selected `Legend` inputs remain flat too.
Replacing only the tuple alias would leave multiple flat representations.

`decorations/legend.rs::LegendIdentity` is currently typed `Single`, `Pair`,
`Literal`, `BarSign`; `LegendEntry` retains raw identity, resolved title/label,
complete optional point/line/fill glyph, and `contributes_hue_level`.
`resolve_entries`, `merge_exact_entries`, `resolved_title`, `same_title` and
`same_identity` enforce one compatible primary title and exact glyph/label
deduplication. Private equality includes shape, point radius/outline/alpha,
stroke width/pattern/RGBA and fill/outline/RGBA, not only RGB.

| Actual producer / identity | Current guide contribution | T6L migration owner |
| --- | --- | --- |
| Scatter | Hue point Single | L1 ordered section input; L2 truthful color keys |
| Ordinary Line / hue LinePath | Actual collective Single/Pair with complete point/line/ribbon components or complete path-line key | L1 Combined pairs; L2 reject independent pair decomposition without mapped group-style channel |
| Trajectory / neutral LinePath | Row-hue points, neutral connector not hue key; hidden neutral LinePath markers suppress inert guide | L2 hue section only; no invented group or recolored connector |
| Density | Actual collective Single/Pair curve+fill | L1 preserve Combined; L2 unsupported independent pair factorization, not synthetic group styles |
| Bar orientations/stacked | Positioned-series fill keys; signed HorizontalBar BarSign | L1 preserve typed domains; L2 single color/sign explanation with complete key |
| Boxplot/Violin/Raincloud | Prepared series/hue complete producer body/summary/strip glyph | L1/L2 single truthful series/color section and exact sampled/supplied glyph |
| Interaction | Group glyph when exact group styles are configured, hue/group/Pair otherwise; shape/pattern group mapping is independent provenance | L2 separate group-style/hue projections only when encoding supports it; same field/group-color remains joint |
| Dumbbell | Literal endpoint labels/paints | L1/L2 literal typed section, B3 exact RGBA retained |
| Regression | Explicit-visible literal fit/band guide | L1/L2 literal/reference explanation, no categorical fabrication |
| Marginal / Matrix | Complete hue or collective keys from linked density/points; Matrix owns its local guide | L1 producers and Matrix local consumer; L2 no general group-style mapping |
| Labeled ordinary-Line PointLayer | `PreparedLine::guide_entries_with_colors` currently calls `chart.point_legend_identity(label)` and `.without_hue_level()` | **L1** adds true typed layer identity; label is not raw hue or append index. L2 projects full exact point; Separate final `series` section, Combined retains primary title or literal series |
| Histogram/Qq/PointInterval/Coefficient/Forest/Waterfall | No discrete contribution; Heatmap/Beeswarm continuous only | No empty sections or broadened public Legend gate |

### Current Callers To Migrate Together

- `layout/interior/guide.rs::PreparedInteriorInputs::guide_entries` dispatches
  prepared family keys. `plots/matrix.rs::{matrix_guide_entries,matrix_legend}`
  is a local specialized consumer. Flat prepared and selected guide key comparisons
  in Scatter/Line/bar extraction fixtures must change with L1.
- `orchestrator/facets/legend.rs::prepare_discrete_guide` resolves/merges complete parent
  contributions before fonts, with full observed parent order. `facets.rs` and
  `facets/sizing.rs` use flat title/entry fields for natural/assigned allocation.
- `canvas.rs::{prepared_discrete_shared_guide,preflight_shared_legend}` unions
  row-major contributors and rejects complete glyph/title conflicts. Hidden local
  entries are skipped, but `prepared_point_path_hue_groups` /
  `canvas_hue_groups` domain coordination retains their
  visible marks. Shared descriptor text census, natural intrinsics, assigned
  frame trials, selected plan and output capture all destructure/use flat fields.
- `decorations/plan.rs::{plan_panel_with_context,plan_panel}` and
  `decorations/legend.rs::{discrete_guide_intrinsics,natural_preference,
  measure_assigned,flow_assigned_entries,visit_calls,select}` consume one title and
  flat entries. `LegendDrawPlan` freezes calls and complete collision/ink geometry.
  Significance and inside-guide clearance treat that whole guide as one obstacle.
- `decorations/plan/children.rs::ChildPlans::plan` currently compares flat Arc
  entry sequences via `LegendEntry::same_child_input` before a cache hit, then
  `Inputs::matches` checks owner/theme/constraints/consumed state and
  `round.replay_child` replays retained feedback. Hit lookup still admits its call
  and visits. `Legend::observe_feedback` / `FeedbackRound::observe_legend_glyph`
  retain complete glyph consequences; equal outer extent is not sufficient.
- `layout/text/requirements.rs` and Canvas/facet prepared guide census own
  pre-font title/entry order; `spec.rs::PlotRenderResult`, selected local/shared
  plans and capture must retain typed sections, not refactor only font strings.

L1 replaces every flat discrete representation with one ordered section input,
including Combined. L2 retains mapping provenance before fonts: group-style
projections neutralize only hue-mapped RGB to the **owning Plot legend-label text
RGB**, keeping mapped paint alpha and all other geometry; hue projections retain
actual complete paint and neutralize only mapped group shape/pattern. L3 measures
one outer frame, stacks sections in semantic order and uses one global flow
variant/bounded owner budget. L4 includes composition/ordered section identities,
full keys, titles/text/typography/constraints/feedback in all reuse identities and
frozen plans. L5 makes Combined/Separate public, default Combined.

Standalone/facets use Plot composition; PerPlot preserves child composition;
Shared uses Canvas composition without treating stored child selections as
conflicts; Hidden has no section/font work. Shared Separate unions typed section
identities with exact titles/labels/projected keys and rejects equal nonempty
titles on distinct sections. Combined preserves current primary-domain
compatibility. Continuous guides remain unsectioned and mixed guide kinds reject.

## 10. Repository Caller Migration Inventory

These are repository-owned caller families, including source unit fixtures and
public documentation. Import-only suite files must migrate with their called
constructors. The owning cutover must also rerun a symbol search then: this census
does not promise that future commits cannot add callers.

| Caller family / concrete paths | Owning migration |
| --- | --- |
| Root and type doctests: `src/lib.rs` Theme/inheritance and private-module checks; `src/theme/resolved.rs` private resolve compile-fail; `src/builder.rs` HeatmapScale wrong-family and PointLayer/wrong-family compile-fail examples; `src/spec.rs` NumericLabelFormat removed-constructor compile-fail | B4 enum/scale and typed-boundary migration; B3 exact-layer signatures where used; N6 format doctest. Preserve compile-fail intent, not obsolete import failure |
| `examples/01_quick_start.rs`, `02_themes.rs`, `03_inject_stats.rs`, `04_canvas_themes.rs`; root `README.md`; `docs/getting-started.md` | B4 preset construction, theme lists/ownership demonstration; C2 omitted sizing expectations. No currently located exact tuple/format examples in these example files |
| `src/theme/{resolved,scale,color,contrast}.rs`, `src/font/canonical.rs`, `src/layout/text{,/requirements,/shaping_tests}.rs` unit fixtures | B1/B2 validated values and role ownership, B4 complete-value construction/constant migration; N4/N5 alphabets/cache/fallback fixtures |
| Source layer/endpoint/guide fixtures: `src/builder.rs`, `src/input.rs`, `src/plots/{scatter,line,dumbbell,bar_data}.rs`, `plots/line/tests.rs`, `plots/shared_x/prepared.rs`, `plots/bar_data/extraction.rs`, `plots/interaction/{native,extraction}.rs`, `plots/interval_estimate/extraction.rs`, `src/layout/interior/{estimates,preference}.rs`, `src/decorations/plan/tests.rs`, `src/orchestrator/tests/lifecycle.rs` | B3 constructor/paint/alpha migration; preserve source identities, exact styles, append order and retained draw checks |
| Source style/value fixtures: `src/spec/tests.rs`, `src/significance/tests.rs`, `src/orchestrator/tests/focused_plot_options.rs`, `src/plots/heatmap/{tests,extraction}.rs`, `src/plots/interval_estimate{,/extraction}.rs`, `src/plots/waterfall.rs` | Retained significance override precedence at B stages; N3/N4/N6 fixed-value engine, payload and total policies; Interaction lexical correction remains unassigned |
| Source exact-size/parent/cache fixtures named in §§7–9; `src/canvas.rs` private sizing/shared descriptor tests; `src/canvas/sizing.rs`; `src/layout/interior/coordinate/tests.rs` | C2 concrete request/getter migration; N5/L1/L4 input and cache equality; keep cache-disabled/frozen capture evidence |
| `tests/rendering/{suite,public_contracts,contract}.rs`, `tests/layout/{suite,layout_and_legends}.rs`, `tests/composition/{suite,themes,facets,canvases}.rs` | B3/B4 types and Theme values, BarPointLayer tuple/opacity constructors and PointLayer color/opacity; C2 exact/erased dimension assertions; L1/L5 Combined/Shared callers and aliases |
| `tests/lines/{suite,numeric_layers,effect_bands,paths}.rs`, `tests/interactions/{suite,styles,connectors}.rs` | B3 exact styles, B4 categorical maps/presets, N6 axis formats when added, L1/L5 layer/pair/shape keys and local/Shared composition |
| `tests/categorical/{suite,dumbbell,beeswarm,waterfall,grouping,plots}.rs`, `tests/distributions/{suite,sample_plots,raw_overlays,composites}.rs`, `tests/marginal_matrix/{suite,contracts,parity}.rs` | Endpoint/style/preset/map migration where present; C2 defaults; continuous/default numeric profiles and section producers; retain scientific fixtures/fingerprints |
| `tests/annotations/{suite,significance_layout,pairwise_significance,panel_text}.rs`, `tests/estimates/{suite,model_displays,estimate_status,point_interval}.rs`, `tests/heatmap/suite.rs` | Retain exact significance controls/content; N6 value format and HeatmapScale B4 migration; narrow-axis applicability and exact label tests |
| `tests/output/{suite,multiformat,png,paths}.rs`, `tests/support/{visual,composition}.rs`, `tests/visual_regression/suite.rs` and committed snapshots | C2 Output canonical dimensions/output reuse; B3/B4 paint/preset constructors; N/L owning behavior changes inspect snapshots/fingerprints with source counts/values/paints/geometry |
| `tests/benchmarks/{suite,fonts,text,legends,annotations}.rs` | B4 raw stack/preset fixtures; N5/L4 scoped keys/work/capture where affected. Historical measurement records remain attributed, not converted into new passing evidence |
| Current `specs/01_ARCHITECTURE.md`, `02_RENDERING_CONTRACT.md`, `docs/output-contracts.md`, `docs/getting-started.md`; target `specs/03_THEME_SYSTEM.md`, `05_LAYOUT_SYSTEM.md`, `93_T6_EXECUTION_PLAN.md` | Update implemented claims at corresponding B4/B5/C2/N6/L5 cutovers; final reconciliation Z1. Target docs remain targets until implemented |
| `specs/04_VISUAL_PROPERTY_OWNERSHIP.md`, `91_IMPLEMENTATION_PLAN.md`, `92_T5B_T5E_EXECUTION_PLAN.md`; `docs/t5c0-property-disposition.md`, `t5d0-scale-consumer-census.md`, `t5d5-scale-closure.md`, `local-search-benchmarks.md`, T5e audit/closure documents | Preserve historical sources/attribution; reconcile active ownership by justified current overlay, not blind replacement of quoted historical API or counts |

## 11. Existing Meaningful Evidence And Output Attribution

### Affected Call Paths And Named Tests

| Call path | Existing source/integration evidence (test names) |
| --- | --- |
| Built-ins, immutable owner inheritance, custom role identity | [`theme/resolved.rs`](../src/theme/resolved.rs): `built_in_themes_preserve_every_resolved_value`, `built_in_themes_have_complete_foundational_records`, `runtime_font_preparation_does_not_affect_appearance_equality`, `root_text_queries_share_payloads_but_not_profiles_or_constraints`; [`tests/composition/themes.rs`](../tests/composition/themes.rs): `unthemed_canvas_child_inherits_the_root_theme`, `explicit_plot_theme_completely_replaces_only_its_inherited_root`, `facets_use_their_owning_plot_theme_without_a_new_scope`, `theme_owners_are_isolated_across_render_calls` |
| Exact RGBA/opacity and independent outlines | [`render/backend.rs`](../src/render/backend.rs): `rgba_construction_and_rgb_conversion_preserve_channels`, `legacy_opacity_rounds_nearest_even_once_before_complete_paint`; [`render/svg_backend.rs`](../src/render/svg_backend.rs): `filled_paths_emit_independent_fill_and_stroke_alphas`, `outlined_points_emit_independent_fill_and_outline_alphas`, `png_preserves_surface_alpha_and_panel_over_root_compositing`; [`tests/lines/numeric_layers.rs`](../tests/lines/numeric_layers.rs): `separate_point_layers_preserve_row_count_radius_and_opacity`, `point_layer_builders_validate_radius_and_opacity_immediately`, `horizontal_vertical_and_diagonal_guides_render_closed_styles` |
| Layer accent order, labels and exact key conflicts | Same numeric-layers file: `automatic_point_layers_follow_an_independent_accent_sequence`, `explicit_point_layer_consumes_its_automatic_palette_position`, `point_labels_and_whole_layer_legend_identity_remain_distinct`, `conflicting_duplicate_point_layer_legend_identity_is_rejected`, `clipped_point_labels_do_not_require_unpainted_glyphs` |
| Exact BarPointLayer → positioned bar centers/intervals → retained capture | [`plots/bar_data.rs`](../src/plots/bar_data.rs): `bar_interval_caps_and_exact_supplemental_points_extend_category_pitch`, `bar_axis_sources_preserve_intervals_stacks_facets_bounds_and_point_extents`; [`plots/bar_data/extraction.rs`](../src/plots/bar_data/extraction.rs): `bar_geometry_matches_payload_with_caps_points_and_attached_text`, `bar_poisoned_capture_repeats_no_preparation_projection_axis_or_shaping`; [`tests/rendering/public_contracts.rs`](../tests/rendering/public_contracts.rs): `contextual_builders_preserve_exact_plot_markers`; [`tests/rendering/contract.rs`](../tests/rendering/contract.rs): `bar_point_layers_retain_rows_at_exact_grouped_centers_in_both_orientations`, `bar_point_layers_reject_nonfinite_mismatch_and_facets` |
| Exact PointLayer and translated guide/body geometry | [`tests/layout/layout_and_legends.rs`](../tests/layout/layout_and_legends.rs): `translated_spectrum_legend_does_not_cover_graphics_or_axes` |
| Dumbbell endpoint input → marks → guide | [`tests/categorical/dumbbell.rs`](../tests/categorical/dumbbell.rs): `dumbbell_renders_connectors_endpoint_legend_and_all_replicates`, `dumbbell_endpoint_type_rejects_empty_or_duplicate_labels`, `dumbbell_output_is_byte_deterministic_and_registered`; [`theme/resolved.rs`](../src/theme/resolved.rs): `estimate_reference_and_dumbbell_roles_do_not_alias_each_other` |
| Interaction exact styles/categorical map/facets/Shared | [`tests/interactions/styles.rs`](../tests/interactions/styles.rs): `interaction_style_legend_matches_each_trace_glyph`, `interaction_styles_require_one_style_per_group`, `interaction_styles_reject_duplicate_groups`, `faceted_interactions_keep_sparse_global_pair_styles_and_colors`, `crossed_group_hue_interactions_render_only_observed_pair_traces`, `shared_canvas_legend_preserves_caller_colors_and_composite_trace_glyphs`, `shared_canvas_legend_rejects_conflicting_point_outline_mapping` |
| Significance exact typography → annotation clearance/retention | [`spec/tests.rs`](../src/spec/tests.rs): `significance_mark_style_default_and_selective_values_are_exact`, `significance_mark_style_rejects_invalid_selective_values_immediately`; [`significance/tests.rs`](../src/significance/tests.rs): `planner_retains_the_exact_effective_significance_label`, `thick_bracket_stroke_expands_only_an_automatic_numeric_window`; [`tests/annotations/significance_layout.rs`](../tests/annotations/significance_layout.rs): `exact_per_bar_marks_render_with_independent_weight_in_both_orientations`, `category_series_comparisons_render_exact_spans_in_both_orientations` |
| Typed fonts, canonical pairs and base-plus-mark context | [`font/canonical.rs`](../src/font/canonical.rs): `canonical_candidate_order_retains_least_occurrence_per_identity`, `unmatched_family_produces_no_candidates`, `sealed_census_reuses_one_normalization_per_stack_and_one_selection_per_pair`, `retained_units_prefer_complete_faces_without_changing_scalar_or_primary_selection`, `formatted_census_counts_units_across_write_boundaries`; [`tests/rendering/public_contracts.rs`](../tests/rendering/public_contracts.rs): `chinese_text_uses_system_font_fallback_stack_without_embedding` (conditional missing-host-glyph branch is part of this test) |
| Opaque scales/prefix capacity/continuous domains | [`theme/scale.rs`](../src/theme/scale.rs): `exact_raw_keys_allow_unused_and_repeated_colors_but_require_used_coverage`, `fixed_capacity_and_accent_slots_do_not_cycle_or_consume_hidden_overrides`, `extended_palette_is_reused_only_within_one_operation_and_same_seed_prefix`, `sampled_guide_preserves_mark_colors_and_refuses_collapsed_distinct_stops`; [`theme/color.rs`](../src/theme/color.rs): `candidate_order_is_prefix_stable_and_never_cycles_a_seed`; [`tests/heatmap/suite.rs`](../tests/heatmap/suite.rs): `heatmap_explicit_color_limits_are_exact_and_saturate_values`, `constant_scale_mark_and_guide_stops_agree_with_decoded_png_pixel` |
| Continuous final-domain compatibility before fonts | [`decorations/legend/descriptor.rs`](../src/decorations/legend/descriptor.rs): `merged_domain_precedes_unique_numeric_labels_and_rejects_missing_conflicts`, `ordered_sequential_stops_are_retained_and_mismatched_colors_reject_before_fonts`; [`canvas.rs`](../src/canvas.rs): `custom_heatmap_and_beeswarm_share_one_gradient_or_reject_conflicts_before_fonts`, `shared_diverging_guide_replays_identically_with_child_cache_disabled`, `mixed_shared_guide_kinds_reject_before_font_discovery` |
| Root title/subtitle/caption and embedded ownership | [`theme/resolved.rs`](../src/theme/resolved.rs): `root_alignment_resolves_physical_x_and_anchor`, `built_in_line_heights_preserve_current_geometry`; [`canvas.rs`](../src/canvas.rs): `odd_canvas_root_policy_is_independent_of_local_panel_alignments`; [`tests/visual_regression/suite.rs`](../tests/visual_regression/suite.rs): `root_title_block_explicit_odd_size_snapshot`, `canvas_root_title_block_odd_size_snapshot` |
| Independent instruction resolution, call order and full physical-span alignment | [`decorations/title_block/plan/tests.rs`](../src/decorations/title_block/plan/tests.rs): `root_alignment_precedence_preserves_independent_optional_instructions`, `private_plot_inputs_survive_erasure_theme_order_and_replacement`, `independent_physical_alignment_does_not_change_wrapping_or_fit` |
| Numeric interval ordering/exact precedence/fatal refusal | [`layout/panel/sources/tests.rs`](../src/layout/panel/sources/tests.rs): `requested_interval_fit_wins_and_dense_requested_source_coarsens_locally`, `dense_and_empty_interval_misses_do_not_poison_coarser_sources`, `logarithmic_interval_vectors_are_divisible_integral_exponents_not_subsequences`, `logarithmic_empty_intersection_charges_one_miss_then_original_midpoint`, `exact_source_overrides_intervals_and_provenance_survives_layout_cloning`, `numeric_source_resource_and_font_failures_remain_fatal` |
| Private sizing, parent recovery, canonical output | [`canvas.rs`](../src/canvas.rs): `private_sparse_canvas_roots_use_content_and_ignore_standalone_plot_sizes`, `content_shared_guide_recovers_after_measured_gutters_without_reselecting_width`, `content_height_feedback_replays_identical_guided_canvas_with_cache_disabled`; [`canvas/sizing.rs`](../src/canvas/sizing.rs): `retained_height_rejects_width_domain_gutter_and_visibility_changes`; [`layout/interior/coordinate/tests.rs`](../src/layout/interior/coordinate/tests.rs): `every_native_root_preserves_default_and_explicit_dimensions_and_avoids_layout_backends` (current public default assertion must migrate); [`tests/output/multiformat.rs`](../tests/output/multiformat.rs): `output_reuses_one_plot_render_for_exact_svg_bytes`, `output_reuses_one_canvas_render_for_exact_svg_bytes`, `output_png_options_are_call_local_across_preview_custom_and_publication` |
| Frozen CSS size versus PNG-local refusal | [`scene.rs`](../src/scene.rs): `canonical_scene_rejects_incomplete_zero_and_mismatched_output`, `svg_routes_bypass_png_source_limits`, `png_parser_gate_uses_outlined_bytes_without_poisoning_svg`; [`tests/output/png.rs`](../tests/output/png.rs): `png_width_and_height_preserve_aspect_ratio` exercises raster sizing rather than pre-render Plot getters |
| Current numeric default grammar, census and guide uniqueness | [`layout/panel/sources/tests.rs`](../src/layout/panel/sources/tests.rs): `numeric_formatting_uses_nearest_even_and_declared_thresholds`, `generated_labels_are_unique_and_covered_by_the_closed_census`; [`layout/text/requirements.rs`](../src/layout/text/requirements.rs): `automatic_numeric_alphabet_is_the_exact_sorted_closed_grammar`, `automatic_numeric_axes_contribute_the_closed_alphabet_and_exact_ticks_do_not`; [`decorations/legend/descriptor.rs`](../src/decorations/legend/descriptor.rs): `numeric_guide_precision_is_unique_in_every_order_and_normalizes_signed_zero`, `close_endpoints_retry_a_complete_unique_format_under_one_budget` |
| Fixed Heatmap/point-only Coefficient values and prepared admission | [`spec/tests.rs`](../src/spec/tests.rs): `counted_numeric_census_formatting_preserves_precision_suffix_and_extreme_values`; [`plots/heatmap/extraction.rs`](../src/plots/heatmap/extraction.rs): `cell_label_minimum_precedes_constrained_heatmap_discovery_and_is_retained`; [`plots/interval_estimate/extraction.rs`](../src/plots/interval_estimate/extraction.rs): `semantics_and_label_strings_obey_exact_limits_before_fonts`; [`tests/heatmap/suite.rs`](../tests/heatmap/suite.rs): `heatmap_annotations_use_default_and_chained_formats_with_contrasting_text`, `local_and_shared_guides_publish_the_same_distinct_close_numeric_labels`; [`tests/estimates/model_displays.rs`](../tests/estimates/model_displays.rs): `coefficient_fixed_value_labels_are_point_only_and_default_is_inert` |
| Forest mathematical/log operand and exact raw ticks | [`plots/interval_estimate/extraction.rs`](../src/plots/interval_estimate/extraction.rs): `raw_interval_intent_survives_correctly_rounded_logarithms`, `logarithmic_shared_limits_and_raw_exact_ticks_are_transformed_once`; [`tests/estimates/model_displays.rs`](../tests/estimates/model_displays.rs): `forest_narrow_log_range_uses_expression_labeled_adaptive_exponents`, `forest_log_scale_projects_exact_ticks_from_raw_data_space`; existing `src/numeric/tests.rs` log10 oracles are not certification of the future inverse-power formatter |
| Waterfall total preparation → shelves → ink/frozen output | [`plots/waterfall.rs`](../src/plots/waterfall.rs): `overlapping_totals_use_separate_exact_baseline_shelves`, `total_ink_keeps_exact_numeric_targets_and_only_expands_automatic_x`, `custom_total_typography_reallocates_shelf_without_changing_span_inputs`; [`tests/categorical/waterfall.rs`](../tests/categorical/waterfall.rs): `waterfall_full_svg_fingerprints` |
| Flat Combined keys/flow/inside collision/Shared/cache consequences | [`decorations/legend.rs`](../src/decorations/legend.rs): `resolved_pair_strings_merge_and_title_obey_exact_limits`, `entry_compatibility_uses_operation_resources_without_font_discovery`, `local_and_shared_discrete_guides_allocate_complete_custom_glyph_and_frame_ink`, `assigned_discrete_flow_matches_horizontal_and_vertical_worked_examples`, `compact_vertical_flow_remeasures_labels_and_fits_two_columns`, `glyph_changes_invalidate_feedback_even_when_legend_extent_is_unchanged`, `repeated_legend_measurement_admits_calls_even_with_cached_text`; [`tests/composition/facets.rs`](../tests/composition/facets.rs): `sparse_inside_facet_legend_merges_exact_observed_pairs`; [`tests/composition/canvases.rs`](../tests/composition/canvases.rs): `shared_canvas_legend_merges_typed_collective_pairs`, `hidden_canvas_legends_do_not_preflight_suppressed_glyphs`; [`tests/annotations/significance_layout.rs`](../tests/annotations/significance_layout.rs): `inside_legend_resolved_box_does_not_overlap_bracket_or_label` |
| Existing Shared gap override and validation | [`tests/composition/canvases.rs`](../tests/composition/canvases.rs): `shared_legend_gap_reduces_canvas_legend_separation` checks the actual gap effect and rejects a negative input; retain/migrate it with the A2-adjudicated override disposition |

These tests exercise existing owners; they are not evidence that new public
validated construction, arbitrary custom role values, configured numeric policies,
certified fractional powers, Separate sections or new sealed bindings already
work. Their new compile-pass/fail and behavior fixtures belong to the respective
implementation leaves, with full component field fixtures derived in A2.

### Current Output Fingerprints And Attribution

The current literal expectations below are read from committed test fixtures,
not regenerated in A1. Hashes are full-SVG FNV-1a unless explicitly labeled
SHA-256; font environment and exact fixture sizing matter. A default-preserving
mechanical migration must not repin them merely to hide a mismatch.

| Fixture owner | Current expectation location / sample | Attribution and affected policies |
| --- | --- | --- |
| `tests/interactions/styles.rs::native_interaction_preserves_exact_root_svg_bytes` | Publication plain `(11007,13027462214619244865)`, styled `(11352,15258619364319190699)`; Science plain `(10614,15450700033276445789)`, styled `(10959,11883049564942211587)`; fixed 800×600 | Complete composite guide ink/fractional baseline closure, [T5e guide-cache attribution](t5e7-guide-cache-closure.md); B3/B4 paint/default preservation and L1 Combined migration |
| `tests/categorical/beeswarm.rs::beeswarm_parent_svg_fingerprints` | Exact ordered Publication/Science arrays in test (ten cases per theme); first Publication `(11011,7711629207207043693)`, Science `(10612,8085217306636702031)` | Native tangent packing, measured guides and final local/Shared numeric grammar; [tangent history](t5e4-tangent-packing.md), [guide-cache history](t5e7-guide-cache-closure.md). Its intermediate historical table is not the current golden |
| `tests/categorical/waterfall.rs::waterfall_full_svg_fingerprints` | Exact ordered arrays in test (nine cases per theme); first Publication `(11176,12620242395058704048)`, Science `(10713,14370975614463528702)` | Native category tracks/total shelves and coordinated root/Canvas domains; [root audit](t5e-root-audit.md). N3 built-in three-decimal totals must retain existing default bytes |
| `tests/marginal_matrix/parity.rs::marginal_full_svg_fingerprints` | Current full eight-case table in test and [Marginal audit](t5e-marginal-audit.md); guide-bearing Publication case 1 `(23474,8840269100508290452)`, Science case 1 `(22918,1735915456015330580)` | Complete point+density-fill guide key/outline changes, with independent scientific/KDE/projection assertions; historical terminal-fit-envelope hashes are separately attributed |
| `tests/visual_regression/suite.rs` | Committed `tests/visual_regression/snapshots/visual_regression__*.snap`: odd Plot/Canvas headings, native path/distribution/estimate/bar/distribution-family/Interaction and specialized clip structures | [Final snapshot inspection](t5e-snapshot-closure.md) records counts, values, paints and reprojection; [root audit](t5e-root-audit.md), [guide-cache audit](t5e7-guide-cache-closure.md) and [Marginal audit](t5e-marginal-audit.md) supply other affected snapshot attribution |
| Native Interaction snapshot's historical emitted artifact | [Snapshot closure](t5e-snapshot-closure.md) records SHA-256 `268a8c79b05be4268c417f1e7a257763b71b39075eeb1c4cc318b6fc73e51f59` and PNG 641×401 | Recorded closure artifact identity, not a freshly computed A1 result and not the 800×600 fingerprint fixture above |

C2 intentionally changes no-size root output; B3 changes admitted exact RGBA
capabilities, while already quantized default alpha must not be quantized twice.
Configured N/L policies intentionally change text/guide allocations, but omitted
formats and Combined preserve specified defaults. Changes must be attributed to
their owning behavior and inspected with meaningful scientific/style/geometry
assertions; deleting guide strings before comparing output is not sufficient.

## 12. Explicit Decision Gaps And Conflicting Scope

1. **A2 complete construction remains open.** Pin every component accessor,
   validated constructor and construction-only edit, typed font profile access,
   closed stroke/outline/spacing inputs and exact element alpha edit signatures.
   The private structs contain runtime context/caches and legacy pixel/generic
   carriers; they cannot simply become the public component fields.
2. **Raw font constants have a ledger retirement assignment but no exact spec03
   replacement spelling.** Decide how callers obtain the built-in typed stack/
   first family, assign B4 root deletions, and remove default-family sentinel
   comparisons with actual role/capture evidence. Spec03 requires typed fonts;
   it does not itself define typed `DEFAULT_*` constants.
3. **Scale attachment signatures/capability names are unpinned.** Target palette,
   map and gradient types are specified, but public setter spelling, retained
   `color_limits` versus complete scale domain editing, constructor/accessor
   details and `kind::CategoricalColors` naming need A2. Use its actual family set
   and ContinuousColor, not a new blanket Any/Hue gate.
4. **Interaction lexical constructor validation lacks an exact leaf assignment.**
   Ledger `interaction.styles.group-selector` says corrected validated
   construction at T6; current `new` is infallible, while spec93 B3 is defined as
   Color/alpha migration. A2 must assign the signature correction explicitly.
   Existing shape/pattern tests do not cover that new lexical construction rule.
5. **Alignment component exposure has a B4/B5 dependency question.** B4 exposes
   complete components; CompositionTheme includes TitleAlignment fields. B5
   owns public heading/caption alignment. Pin when the discriminant and component
   accessors become public so B4 is complete and no target-only dormant API is
   introduced. Current private Canvas root instruction fields are not approval
   for additional public Canvas-root alignment setters.
6. **Legacy composition/typography carriers and relative spacing need complete
   mappings.** Canvas TextPanel/header/facet roles still use legacy fields;
   MarkAppearance has family-specific private role fields; the target is role
   based. A2 must account for every admitted ledger field without retaining
   duplicate generic defaults or exposing backend structs. The active overlay,
   not historical fixed-size/minimum rows, adjudicates layout relationships.
7. **`Canvas::shared_legend_gap(f64)` has conflicting ledger disposition.**
   Current `canvas.rs` accepts finite nonnegative exact CSS pixels, rejects its
   use outside Shared, and selects it over `LegendMetrics.shared_gap`.
   `canvas.legend.shared-gap` assigns target Theme component / LegendTheme and
   says the current explicit override is removed; the separate
   `canvas.legend.shared-gap-current-fallback` relationship says the override
   remains exact policy. The T3j group trace does not resolve that contradiction,
   and spec93 assigns no exact deletion leaf. A2 must reconcile classification,
   validated units, precedence, deletion timing, callers and policy validation;
   no choice is inferred here.
8. **Gradient/general continuous scope is broader than a rename.** Current
   divergence requires a three-stop middle neutral, and Beeswarm has no public
   divergent selector. A2 must pin approved gradient/domain shapes and private
   migration owners for the complete ContinuousColorScale contract. Exposing
   test-only raw backend stop vectors would violate the target.
9. **N0 and L0 remain later actual-input closure gates.** A1 maps current owners,
   consumers and capabilities; it does not certify N2 proofs/oracle vectors,
   finalize NumericFormat discriminant/accessor signatures, price adaptive sets
   or derive every producer's factorization key. Separate unsupported pair
   encodings must remain explicit pre-font errors, not invented style mappings.
10. **Current-spec stale implementation claims need staged reconciliation.**
    Spec02's built-in ownership paragraph still assigns surfaces/legend typography
    to staged T5/T6 although current private owners and T5e evidence implement
    them. Historical ledger font/global-palette line references also describe
    removed code. Identify these as drift at the owning cutover/Z1, not additional
    current public exports or permission to alter historical attribution.
    The ledger override `canvas.legend.shared-entry-exact-merge` also says to
    reject independently titled sections; approved spec93 T6L now admits bounded
    typed Separate sections. L1/L5 must reconcile that semantic target using the
    user-approved section contract without silently changing property counts.
11. **Evidence limits are explicit.** No new test result, rendered artifact hash
    or package/compile result was produced. Existing exact matrices/hashes above
    are attributed to their test/document owners.

## Documentation Verification And Handoff

The sole A1 deliverable is this file. Acceptance for this docs-only work is
source/spec consistency review and `git diff --check`. Both `git diff --check`
and `git diff --no-index --check /dev/null docs/t6a1-public-cutover-census.md`
passed; the second check includes this new, untracked file. Implementation tests and
strict Clippy are requirements of later code stages, not actions for A1.
Execution status and the A1 commit remain the phase owner's responsibility.
T6A2 follows that commit and resolves the explicitly unpinned construction,
binding, ownership and deletion decisions above before T6B starts.
