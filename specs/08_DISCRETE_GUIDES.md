# Discrete Guides

**Status**: Approved architectural target. Private Combined/Separate semantic
construction, producer bindings/projection and authoritative pre-font
transport/census are implemented. Multiple-section flow, effective parent-policy
selection and public controls remain pending.

## 1. Authority And Boundary

This document closes the discrete-section architecture admitted in
[`93_T6_EXECUTION_PLAN.md`](93_T6_EXECUTION_PLAN.md), including the precise neutral
paint decision there. [`94_T6_FOUNDATION_CONSTRUCTION.md`](94_T6_FOUNDATION_CONSTRUCTION.md)
owns complete theme construction, paint precedence, typography and exact local
spacing. [`05_LAYOUT_SYSTEM.md`](05_LAYOUT_SYSTEM.md) owns root constraints,
containing-frame recovery, Unicode text, canonical arithmetic and resources.
[`01_ARCHITECTURE.md`](01_ARCHITECTURE.md) and
[`02_RENDERING_CONTRACT.md`](02_RENDERING_CONTRACT.md) remain the implemented
public contract until the corresponding atomic cutover. Combined implementation
evidence is in `../docs/typed-guide-identities.md` and
`../docs/prepared-discrete-sections.md`, `../docs/discrete-producer-bindings.md`
and `../docs/factorized-discrete-sections.md`. This target does not claim the
public Separate renderer or the full acceptance matrix below is implemented.

The source census is taken after the private Color/OpaqueColor carrier migration.
`docs/t6a1-public-cutover-census.md` is useful historical consumer evidence; its
Rgba references and open construction questions do not supersede the current
Color carrier or spec94's closed decisions. Source identifiers below are the
live owners, not proposed exported renderer interfaces.

There is one discrete guide representation and one measured guide lifecycle for
Combined and Separate. Sections explain existing encodings; they do not change
marks, source rows, statistics, domains, sparse topology or glyph visibility.
There is no general group-style mapping, scoped-alias API, independent section
placement, layout callback, new resource cap or continuous/discrete composition.
The renderer remains standalone and offline. No dependency or asset work follows
from this contract.

## 2. Public Policy And Effective Owner

```rust,ignore
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum LegendComposition {
    #[default]
    Combined,
    Separate,
}

impl<K: kind::Marker> Plot<K> {
    fn legend_composition(self, value: LegendComposition) -> Self
    where K: kind::Legend;
}
impl Canvas {
    fn legend_composition(self, value: LegendComposition) -> Self;
}
```

The enum is a crate-root public policy at cutover. Backend styles, section
records, provenance, layout constraints and inspection of prepared guides stay
private. The existing sealed Legend capability is not widened, and erased Any
does not acquire the setter. Both Plot and Canvas store a concrete Combined
default; no optional inheritance or automatic composition mode exists.

| Rendering owner | Effective composition and guide appearance |
| --- | --- |
| Standalone Plot | Plot selection; Plot guide chrome/text and producer keys |
| Generic facets | One owning Plot selection, chrome/text and shared discrete guide; facets add no theme scope |
| Canvas PerPlot | Each Plot selection and resolved Plot guide chrome/text; a child theme wholly replaces inheritance |
| Canvas Shared | Canvas selection for every contributor; Canvas chrome/text; complete keys prepared under each resolved Plot theme |
| Canvas Hidden | No discrete sections, projected keys or guide font requirements |
| Plot Hidden within Shared | No entry contribution or factorization; visible marks still participate in color-domain coordination |

Canvas Shared ignores stored child composition selections, including explicitly
set ones. It neither rejects them nor overwrites them. Returning to PerPlot uses
those child selections again. Canvas composition is harmless stored policy under
PerPlot/Hidden and on an all-continuous composition. Setters replace only their
own input and are independent of call order with theme, visibility, placement,
labels, sizing and Canvas legend policy. An explicit child legend edge remains
incompatible with Shared; section selection does not relax that placement rule.

Hidden suppression skips guide-only semantic/display validation, projection and
font work. Scientific input/style/scale validation still applies to visible
marks. In particular, hiding an unsupported pair guide does not waive invalid
categorical data, a missing exact color mapping or an invalid Interaction style.

Current surface/owner anchors: `src/kind.rs:174-184`, `src/spec.rs:513-603`,
`src/canvas.rs:3333-3341`, `src/decorations/legend.rs::GuideKind::contributing_prepared`.

## 3. One Typed Semantic Model

### Descriptor, Identities And Counts

The following is the closed private data schema. Names are target internal names;
they are not compatibility aliases or additional public construction values.

```rust,ignore
struct RawField(Arc<str>);
struct RawCategory(Arc<str>);
struct RawLayerLabel(Arc<str>);

enum PrimaryTitleIdentity {
    Field(RawField),
    Pair { group: RawField, hue: RawField },
    Series,
    BarSign,
}
enum LiteralDomain { BarSign, DumbbellEndpoints, RegressionReference }
enum GuideSectionIdentity {
    Combined(PrimaryTitleIdentity),
    GroupStyle(RawField),
    Joint(RawField),
    HueColor(RawField),
    Literal(LiteralDomain),
    SupplementalSeries,
}
enum GuideEntryIdentity {
    Category { field: RawField, value: RawCategory },
    Pair {
        group_field: RawField, group_value: RawCategory,
        hue_field: RawField, hue_value: RawCategory,
    },
    BarSign { decreasing: bool },
    DumbbellEndpoint { label: Arc<str> },
    RegressionObservation,
    RegressionFit { band: RegressionBand },
    SupplementalLayer { label: RawLayerLabel },
}
struct PreparedGuideEntry {
    identity: GuideEntryIdentity,
    display_label: Arc<str>,
    key: ResolvedGuideKey,
    sources: Arc<[GuideKeySource]>,
}
struct PreparedGuideSection {
    identity: GuideSectionIdentity,
    display_title: Arc<str>,
    entries: Arc<[PreparedGuideEntry]>,
}
struct GuideCounts {
    sections: u32,
    entries: u32,
    points: u32,
    lines: u32,
    fills: u32,
    primary_source_keys: u32,
    observed_pair_source_keys: u32,
    labeled_layer_source_keys: u32,
    source_occurrences: u32,
}
struct PreparedDiscreteGuide {
    composition: LegendComposition,
    counts: GuideCounts,
    sections: Arc<[PreparedGuideSection]>,
}
enum PreparedGuideDescriptor {
    Discrete(Arc<PreparedDiscreteGuide>),
    Continuous(ContinuousDescriptor),
}
```

RawField/RawCategory preserve the already validated normalized scientific names
and values, before aliases. RawLayerLabel preserves exact supplied bytes; it is
not revalidated as a categorical selector or normalized as a category. Endpoint
labels retain their existing validated caller bytes. Painted-string Unicode
validation remains the text owner's responsibility, including for layer labels.

Every section is nonempty. Absence is `None`, not a descriptor with zero sections;
constructors reject inconsistent lengths/counts. Counts are exact checked u32
integers; sums/products and resource calculations use checked u128, and admitted
allocation lengths convert to usize fallibly. Counts fit the existing million
record/topology envelope; no floating cardinality, saturating counter or second
limit exists. Points/lines/fills count retained key components, not source marks.
Source occurrences count actual retained provenance records, not a fabricated
group-by-hue product. Primary/pair/layer source counts are counted before
projection: a three-pair source has three primary and three pair source keys,
whether it yields three Combined entries or four Separate entries. The latter
retains six projection occurrences, two per source pair. Repeated facet/Canvas
contributions count their real occurrences; deduplication changes entry counts,
not this source census. No source-key Vec is retained to obtain these counts.
Section/entry ordinals are checked u32 indices. Statistics such as source-row
and primary-vertex counts remain separate producer integers.

A section owns its semantic title identity and resolved title. An entry does not
carry a second title. Category/Pair field names must agree with their containing
section. SupplementalLayer is deliberately permitted in Combined's one primary
section without becoming a categorical identity. Combined's Series title can
contain typed Regression, endpoint and supplemental identities; the former
literal `series` title compatibility remains, but entry types cannot alias one
another accidentally. Separate gives those domains their distinct section types.
No string-valued arbitrary domain or renderer plug-in is part of this schema.

### Source Mapping Provenance

The key grammar remains at most one fill handle, one line handle and one point,
painted in that order. It does not acquire data-space error bars, raw-observation
clouds, extra reference lines or a mini-plot. Exact role-produced key styles and
their structural outline absence are retained before measured key geometry.

Each source contribution has typed, producer-known provenance:

```rust,ignore
enum GuideEncoding {
    HueOnly { field: RawField },
    Joint { field: RawField },
    IndependentPair { group: RawField, hue: RawField },
    UnsupportedPair { group: RawField, hue: RawField },
    RowHue { field: RawField },
    Literal(LiteralDomain),
    SupplementalLayer,
}
enum PaintSlot { PointFill, PointOutline, LineStroke, FillPaint, FillOutline }
enum StyleSlot { PointShape, LinePattern }
enum PointRole { Primary, RawOverlay, MandatoryStrip, Supplemental }
enum FillRole {
    BarBody, BoxBody, ViolinBody, LineBand, RegressionBand,
    DensityArea, MarginalDensity, MatrixDensity,
}
enum LineRole {
    SeriesLine, PathLine, DensityLine, InteractionConnector,
    SummaryStroke, RegressionFit,
}
enum SourceNeutralPaintRole { Path, BoxFlier }
enum MarkPaintRole {
    PointFill(PointRole), PointOutline(PointRole),
    FillPaint(FillRole), FillOutline(FillRole), LineStroke(LineRole),
    Neutral(SourceNeutralPaintRole),
}
enum MarkStyleRole { PointShape(PointRole), LinePattern(LineRole) }
enum RgbBinding {
    Mapped { field: RawField, value: RawCategory },
    Role(MarkPaintRole),
    ExactElement,
}
enum StyleBinding {
    Role(MarkStyleRole),
    InteractionGroup { field: RawField, value: RawCategory },
}
struct PaintBindings {
    point_fill: Option<RgbBinding>,
    point_outline: Option<RgbBinding>,
    line_stroke: Option<RgbBinding>,
    fill_paint: Option<RgbBinding>,
    fill_outline: Option<RgbBinding>,
}
struct StyleBindings {
    point_shape: Option<StyleBinding>,
    line_pattern: Option<StyleBinding>,
}
enum Projection { Complete, GroupStyle, HueColor }
enum LayerLabelContext { Categorical, LiteralSeries }
struct GuideKeySource {
    producer: PlotKind,
    contributor_address: ComponentAddress,
    source_ordinal: u32,
    encoding: GuideEncoding,
    paint_bindings: PaintBindings,
    style_bindings: StyleBindings,
    projection: Projection,
    group_neutral_rgb: Option<OpaqueColor>,
    layer_label_context: Option<LayerLabelContext>,
}
```

MarkPaintRole/MarkStyleRole are closed private discriminants for the named
consumed spec94 roles in the producer table below: primary/mandatory-strip/
supplemental points, bar/box/violin/band/density fills and their outlines,
series/path/density/Interaction lines, summary/regression strokes, neutral path
and flier RGB. They are not public strings or a new theme representation. Absent
components/outline slots have None bindings and no dormant style or paint input.
An actual exact paint has ExactElement provenance even when its RGB equals a
mapped color. Role versus mapped fill-outline is taken from DataOutlineTheme's
selected variant, never inferred from channels or alpha.

The final key/mask schema is also closed:

```rust,ignore
enum RgbExplanation { Constant, ExplainedColor, NeutralizedColor }
enum StyleExplanation { Constant, ExplainedGroup, NeutralizedGroup }
struct GuideExplanationMask {
    point_fill: Option<RgbExplanation>,
    point_outline: Option<RgbExplanation>,
    line_stroke: Option<RgbExplanation>,
    fill_paint: Option<RgbExplanation>,
    fill_outline: Option<RgbExplanation>,
    point_shape: Option<StyleExplanation>,
    line_pattern: Option<StyleExplanation>,
}
enum GuideExplanation {
    Complete,
    Projected(GuideExplanationMask),
}
struct ResolvedGuideKey {
    point: Option<PointStyle>,
    line: Option<LineStyle>,
    fill: Option<FillStyle>,
    explanation: GuideExplanation,
}
```

PointStyle/LineStyle/FillStyle here are the single private normalized backend
operands derived from spec94, never exported component inputs or a parallel
guide style representation. They retain complete Color, positive integral point
radius, closed shape/pattern, normalized stroke widths and structural optional
outlines with their exact consumed width/ratio inputs. Spatial realization uses
the canonical roles in §8. A structural outline None has no ignored paint/width.

Combined, Joint and literal/layer keys use Complete explanation. Their full-key
merge rule remains exact visible-glyph equality; differing source-role provenance
cannot manufacture a new Combined incompatibility. Provenance is still retained
for cache/source validation. Separate hue-only/row-hue entries have unchanged
glyphs but a HueColor projected explanation: mapped RGB ExplainedColor, constant
style/other paint Constant. Group projection marks its mapped group style
ExplainedGroup and changes hue-mapped RGB to NeutralizedColor; hue projection
marks mapped RGB ExplainedColor and changes only group-mapped style slots to
NeutralizedGroup. The containing typed section supplies the explained field/domain;
the mask is not a second string-keyed semantic identity. Exact projected-key
equality includes this mask and all visible geometry/paints. Source producer/
address/role names are audit/cache provenance, not a family discriminator for
merging otherwise equal explanatory keys.
Merged entries retain admitted immutable source records; no source wins by being
first. Occurrence records are reused by Arc where identical; copying is charged.

Do not store a second flat list of complete pair entries alongside sections.
Prepared scientific inputs may retain their real observed pair domain for marks;
the resolved guide is the one composed descriptor. A shared style materializer
can return a complete key and its bindings while the producer also constructs
marks, but it must not invoke a panel measurement or font lookup to prepare keys.

## 4. Exhaustive Producer Census And Truthful Composition

The live sealed Legend set has exactly **17** concrete kinds. The table below
lists each once. Component counts describe one source key; entry counts follow
retained semantic identities and deduplication, not the number of source rows.
Current private role names map to spec94's complete role paths at foundation
cutover; their present numeric defaults are evidence, not new layout constants.

| Kind | Live prepared function / contribution condition | Complete key and actual source style | Separate result |
| --- | --- | --- | --- |
| Scatter | `PreparedScatter::{guide_entries_in_order,guide_entries_with_colors}`, `src/plots/scatter.rs:231-269`; one key per retained finite hue, none for unkeyed data | Point only; `point_style` uses primary point, scatter radius 3, effective outline policy; no line/fill | HueColor; group never fabricated |
| Line | `PreparedLine::{guide_entries_in_order,guide_entries_with_colors}`, `src/plots/line.rs:277-369`; only primary identities with hue, plus labeled PointLayers | series_line width 1.8, path radius 3 when markers visible; line_band fill alpha 46 iff every retained point of that group has an interval; P+L+F, P+L, L+F or L. Complete bounds, including a zero-width scientific ribbon, retain the configured ribbon component | HueColor for hue only; Joint for same-field collapse; different group+hue is UnsupportedPair |
| Trajectory | Same PreparedLine owner; constructor graphical groups determine connectivity, hue can vary per row | Hue point only, primary point/trajectory radius 4; neutral path_line width 1.4 is NOT in the hue key; mandatory trajectory points survive | RowHue becomes HueColor; no group keys |
| LinePath | Same owner; `LinePathStyle::Neutral` versus `Hue` is retained provenance | Neutral: row-hue point only when markers visible, otherwise absent. Hue: path_line width 1.4 plus trajectory-radius-4 point iff markers visible; no ribbon. Path source order and repeated x remain scientific input | Neutral RowHue -> HueColor; hue-styled paths collapse same-field Joint, otherwise reject different group+hue without mapped styles |
| Density | `PreparedDensity::{guide_entries_in_order,guide_entries_with_colors}`, `src/plots/density.rs:158-193`; hue-bearing observed curves only | density_line width 1.6 plus density_area fill alpha 56; no point even when points.primary changes; independent role outlines retained | HueColor, same-field Joint, or UnsupportedPair; group-only curves remain unkeyed/no guide |
| Bar | `bar_data::legend_entries`, `src/plots/bar_data.rs:1403-1444`; positioned-series hues only | bar_body fill alpha 255 and its selected outline; no point/uncertainty stroke; category-only coloring has no detached category guide | HueColor for the positioned series |
| HorizontalBar | Same bar producer, including sign policy | ordinary series fill as Bar; signed keys use actual presence of `end < start`, bar_body and sign accent slots; key order nondecreasing then decreasing when present | HueColor or Literal(BarSign), no series keys in sign mode |
| StackedBar | Same producer via `stacked_bar` measurement | Same bar_body fill key per constructor-owned series; stack geometry is not a guide layout alternative | HueColor |
| StackedHorizontalBar | Same producer/orientation adapter | Same bar_body complete fill key per constructor-owned series | HueColor |
| Boxplot | `PreparedDistribution::guide_entries_with_colors`, `src/plots/categorical_distribution.rs:73-90`; sample hue/series levels | summary_stroke line + box_body fill; no primary/raw-overlay/flier point in key, regardless of RawOverlay mode or outlier count | HueColor |
| Violin | Same function, lines 92-106, Violin branch | violin_body fill alpha 178 only; no point, density_line or inner-summary key component; optional raw observations do not manufacture a key | HueColor |
| Raincloud | Sampled `PreparedDistribution` Violin branch; supplied `PreparedRaincloud::guide_entries`, `src/plots/violin/precomputed.rs:109-124` | Point + violin_body fill. Current source uses `distribution_sample::overlay_style` with shared raw_overlay role/radius 2 and outline `true`; spec94 splits this onto mandatory_strip and applies the actual outline policy once. Mandatory strip presence is independent of RawOverlay | HueColor for sample series or supplied hue; sampled/supplied geometry use identical selected strip role |
| Interaction | `PreparedInteraction::{guide_entries,guide_entries_with_colors}`, `src/plots/interaction.rs:207-267`; hue present or explicit styles configured | primary point/interaction radius 4 with resolved trace shape; interaction_connector width 1.8 with trace pattern; component presence follows markers/connectors. interaction_interval width 1.2 remains a mark, not an extra key line | Configured independent style+hue -> GroupStyle + HueColor; configured group-only -> Joint; same-field hue -> Joint even without configured style; different pair without configured styles rejects |
| Dumbbell | `DumbbellData::guide_entries`, `src/plots/dumbbell.rs:119-138`; caller endpoint metadata | Two point keys in left/right order, primary point/dumbbell radius 4 and actual outline; endpoint exact Color from spec94 replaces complete fill RGBA. Connector not included | Literal(DumbbellEndpoints) |
| Regression | `PreparedRegression::guide_entries`, `src/plots/regression.rs:192-220`; only explicit Visible policy, not Automatic | Observations point/primary radius 3 first; regression_fit width 1.5 second, plus regression_band alpha 64 iff PointwiseMean95. Fit stroke is a fixed role, not the observation accent; no synthetic categorical mapping | Literal(RegressionReference) |
| Marginal | `PreparedMarginal::{guide_entries,guide_entries_with_colors}`, `src/plots/marginal.rs:130-163`; hue-bearing retained collective groups | primary point/marginal radius 3 + marginal_density fill alpha 115, currently mapped outline width 1; no density_line component | HueColor, same-field Joint, or UnsupportedPair; standalone and Canvas, not generic facets |
| Matrix | local `matrix_guide_entries`, `src/plots/matrix.rs:1207-1238`, `matrix_legend` and `natural_guide_preference` | primary point/matrix radius 2 + matrix_density fill alpha 102, currently mapped outline width 1; diagonal density and off-diagonal points share raw hue; no density_line component | HueColor, same-field Joint, or UnsupportedPair; Plot-local specialized root only |

The remaining **eight** kinds complete the 25-kind census:

| Kind | Discrete behavior |
| --- | --- |
| Histogram | No discrete contribution; samples/supplied bins, rugs and KDE do not create sections |
| Qq | No discrete contribution, including Quartiles/Identity reference policies |
| PointInterval | No discrete contribution, including reference strokes and estimate markers |
| Coefficient | No discrete contribution, including numeric details/unavailable markers |
| Forest | No discrete contribution, including Log10 ticks and estimate reference |
| Waterfall | No discrete contribution; positive/negative accents and totals do not acquire a sign guide |
| Heatmap | Continuous only; retain ContinuousDescriptor and missing block, never section it |
| Beeswarm | Continuous only; same continuous boundary |

`src/layout/interior/guide.rs::PreparedInteriorInputs::guide_entries` exhaustively
dispatches the 16 parent-capable Legend kinds, six empty cases and two continuous
errors. Its Matrix error is intentional parent ownership, not absence of a Matrix
guide. Matrix's local producer must migrate to the same descriptor/materializer;
it remains excluded from Canvas and generic facets. Marginal remains excluded
from generic facets. No parent-capability expansion is authorized.

### Presence And Defaults Are Producer Decisions

Only an explicitly configured InteractionStyle mapping is an independent
group-style channel. `resolve_trace_styles` validates exactly one style per raw
group. Its default Circle/Solid values may repeat across configured groups and
still form an actual caller mapping. A theme's common point shape/line pattern
is not a group mapping. Unconfigured Interaction retains default trace shape/
pattern and current contribution conditions; group-only unconfigured Interaction
has no guide. Line/Density/Marginal/Matrix group-only marks likewise stay without
group keys. Same-field collapse is allowed without an independent style channel
because it retains one complete key rather than attempting a decomposition.

Interaction guide component grammar follows the visible-marker/connector policy,
not incidental source vertex count. A connector key can explain configured trace
connection even for a sparse one-cell trace, whose local payload has no segment;
this does not add a connector to the marks. Existing configured-style restrictions
on hidden markers/connectors remain unchanged (`orchestrator/validation.rs`).

Optional raw observations have zero guide point components for Boxplot/Violin.
Raincloud has one mandatory-strip point component per key, not one per strip row.
Do not substitute primary points for strip points or always enable outlines.
The existing shared overlay helper's outline `true` is foundation migration
evidence, not permission to resurrect a caller-hidden outline in this target.

Current Line style/ribbon parity is independently visible in
`src/plots/line.rs:791-837,883-932`; point/line/ribbon key presence is not guessed
from RGB or from whether projected geometry happens to have positive area.

## 5. Projection, Collapse And Scientific Ambiguity

### Group Neutral RGB

For GroupStyle projection, replace only RGB of slots bound to the independent
hue field with RGB of the **owning Plot's resolved**
`TypographyTheme.guides.legend_label.color`. Keep each original mapped paint's
alpha. The current private source is `theme.typography.legend_label.paint`;
spec94's target path is `typography().guides().legend_label().color()`.

For a source fill `(7,114,178,127)` and label paint `(40,50,60,0)`, the group key
fill is exactly `(40,50,60,127)`. Text alpha 0 is not installed or multiplied.
The group key retains that group's shape/pattern, radius, widths, structural
outline presence, every non-hue-mapped RGBA and all other geometry. A mapped
DataOutline receives the same neutral RGB and its own original alpha; a fixed
PointOutline/DataOutline is unchanged even if its RGB equals the hue.

This neutral is NOT `MarkTheme.neutral_paints.path` or `.box_flier`, the old
generic foreground, AxisTheme, an annotation role, or Canvas guide-label paint.
Spec94's NeutralPaintRoles own actual neutral marks: Trajectory/neutral LinePath
connector RGB and Boxplot's hidden-overlay flier RGB, with their own mark-role
alpha. They are never guide-projection neutral roles and do not read text paint.

### Hue Neutral Style

HueColor projection retains all original final paints and constant geometry.
Only slots with an InteractionGroup binding change: mapped PointShape becomes
Circle; mapped connector LinePattern becomes Solid. The closed substitutions
are fixed producer explanatory defaults, not whatever a theme's primary shape
or pattern happens to be. A theme-owned shape/pattern in a hue-only producer is
not changed. Radius, width, alpha, actual outline absence and fixed outline RGBA
are never neutralized. Absent components stay absent.

The projected style can have a different occupied extent from the originating
style; measure that exact projected glyph. Group triangle and hue circle keys
need not have equal width/height, but neither may use the other's approximate
extent. Canvas draws the retained producer key; it cannot recolor a generic key
with its palette or rebuild these masks after fonts.

### Validation And Identity

For Separate IndependentPair, group and hue fields must differ, both raw values
must be present, and mapped group-style provenance must exist. All source
occurrences of a raw group must project to the same GroupStyle key; all of a raw
hue must project to the same HueColor key. Component presence, remaining geometry,
paint/mapping mask and labels participate. Any incompatibility is a pre-font
Error identifying contributor, fields, raw value and mismatched slot/component.
Combined preserves complete observed pair keys and does not require factorization.

Unknown factorization is not a neutral-style fallback. An UnsupportedPair errors
under Separate even for one observed pair or equal colors/styles. Fully typed
same-field equality is different: equal raw field names require equal row values
under `data::observed_collectives`; collapse to Joint/Field once, retain complete
key, and never emit duplicate group/hue sections. Equal values in different
fields stay different identities. Raw aliases do not decide collapse.

Repeated explicitly selected styles and colors are legal. These are possibly
many-to-one mappings, not new perceptual-separation validation. A transparent
mapped component remains semantically present; composition does not omit its
entry to evade fit or fonts.

RowHue has no mapped group style and is not an UnsupportedPair: graphical group
owns connectivity, not a key. HueColor contains actual point glyphs and no neutral
connector. Hidden neutral LinePath points leave no primary hue section.

Global categorical color order is not entry membership. In particular, current
`PreparedLine::guide_entries_with_colors` iterates global hue levels for row-hue
keys (`line.rs:345-356`), and some prepared categorical panel domains retain
absent series slots. The replacement records observed membership from retained
prepared rows/groups before projection, filters keys by that membership, then
orders them by the coordinated domain. A value present only in a Hidden Canvas
contributor cannot leak a key/font requirement through a visible row-hue panel's
global color list. A facet's one root guide may contain any value actually present
in another occupied facet; that is parent union, not local fabricated presence.
Preserve color indices and empty dodge slots independently of these key counts.

Coordinate the actual raw color field, not just the spelling of a hue attachment:
Interaction without hue colors marks by group (`interaction.rs:154-175,237-243`).
Its configured group-only Joint guide and hidden/no-entry group-colored marks
therefore use that group field's parent color domain. Positioned-series fields
retain their existing color domain too. Binding this to `hue_dimension_key()`
alone would leave group-only Shared entries with unrelated local palette indices.
The private prepared color-source accessor carries the typed field/observed
values independently of composition/visibility; no new public color setter or
group-style mapping is introduced. Exact maps remain exact rather than recolored.

Sparse pairs stay sparse in Combined. Separate reports marginal domains and does
not assert all combinations occurred. Example: `(A,control)`, `(A,treatment)`,
`(B,treatment)` gives three complete pair keys versus two group plus two hue keys;
Separate has four keys and loses the legend's explicit missing `(B,control)`
association. It is not universally smaller. A dense configured five-group/four-
hue Interaction has 20 pair keys versus nine independent keys plus two titles.

## 6. Titles, Labels, Literal Domains And Supplemental Series

Inferred field titles and categorical values resolve through the effective
global Labels policy. Combined Pair joins resolved fields with ` × ` and values
with `, `, as today. Separate field sections use the resolved field name directly.
Raw field/value identity remains independent of display text.

Literal titles are exactly `Change` for BarSign and `series` for endpoints,
Regression and supplemental-only guides. BarSign entries are verbatim
`Nondecreasing`/`Decreasing` for actual observed signs; Regression's two current
fit-policy strings and `Observations` stay verbatim. Endpoint labels stay caller
verbatim. Layer handling below is not a reason to humanize these literal strings.
Qq/estimate/reference strokes without existing discrete entries acquire no literal
section. Separate literal domains with coincident `series` titles reject if mixed;
Combined retains the common Series primary-title compatibility. This is an
explicit semantic distinction, not a type inferred from a rendered title.

Within a section, duplicate raw entry identity merges only with exactly equal
display label and full resolved key. Distinct raw identities with the same
display label reject. Equal labels across differently titled sections are legal.
Distinct section identities with equal nonempty display titles reject, including
Joint(field) versus HueColor(field). Equal empty titles are permitted by the
approved nonempty-title rule. Empty titles have zero title occupancy/no gap;
an empty entry label retains its key and semantic identity and has no painted
text, baseline or label ink. Do not use an empty label to omit an entry.
Titles/labels use strict Unicode wrapping; no ellipsis, shrink or identifier split.

Labels aliases are globally keyed by raw string, not `(field,value)`. A single
raw `same` in two fields cannot receive different field-local aliases. This
contract adds no scoped label mapping. Shared contributors resolving the same
identity differently reject before fonts; Canvas Labels remains its existing
effective panel-label policy when supplied.

### Actual PointLayer Behavior

`PreparedLine::guide_entries_with_colors` currently constructs a layer key through
`point_legend_identity(label)` and `without_hue_level()` (`line.rs:358-366`). That
is not yet a true layer identity. Replace it with SupplementalLayer keyed by
exact supplied label bytes, never append index or primary hue field. The layer's
complete plain point has its resolved radius, fill RGBA and absent outline;
spec94's supplemental defaults and exact Color/alpha precedence apply once.

Preserve the **actual context-dependent existing label text behavior**. Retain
LayerLabelContext from the source Plot's current `legend_title_key` /
`point_legend_identity` routing: presence of a categorical hue/title instruction
selects Categorical, otherwise LiteralSeries. This is determined before entry
visibility filtering, even when a neutral LinePath's hidden markers leave only
its labeled layer as a guide contribution:

* In Categorical context (Single/Pair, including collapsed same-field),
  resolve the supplied layer label through effective global Labels.
* In LiteralSeries context, copy supplied layer-label bytes verbatim,
  as the current literal `series` path does.

This display choice depends on prepared source label context, not whether the
primary currently emits entries, composition, append index, Canvas's final title
or a first pair. Shared contributors with the same layer identity but different
resulting labels reject. Duplicate labels with
different resolved full point styles reject; exact duplicates merge once. Layer
and categorical identities with equal display text remain distinct and reject
when attached in the same Combined section; they can coexist in differently
titled Separate sections. Point annotations neither create entries nor acquire
this Labels transformation.

Combined appends layer keys after primary keys under its primary display title.
With no primary entry it has one Combined(Series) section. Separate appends one
SupplementalSeries section titled `series` after primary sections, or only that
section if alone. Labeled layers do not contribute hues to color coordination;
unlabeled layers create no key. Exact overrides still consume the layer's normal
accent position. A hue-driven primary consumes no accent slot; unkeyed primary
consumes slot zero, as spec94 specifies. Filtering entry visibility does not
reassign accent slots or per-point labels.

## 7. Parent Union And Canonical Ordering

Combined has exactly one section. PrimaryTitleIdentity must match exactly across
contributing primary entries: Field(raw name), Pair(ordered raw fields), Series
or BarSign. Resolved title must also match. Supplemental-only contributors do
not choose the global primary title: collect their typed layer contributions,
then attach them to the one compatible primary section after all contributors
are known; if no primary exists use Series. Their display labels remain those
already resolved under their source context. This rule avoids first-panel wins.

Separate unions exact GuideSectionIdentity, then requires exact titles and
entry compatibility. Canonical class order is **GroupStyle, Joint, HueColor,
Literal, SupplementalSeries**. Within a class, use first contributing occurrence
in source-owner order, not raw name, enum domain order, hash iteration or title.

* Standalone fields/values follow prepared producer domains after normal retained
  row filtering. Combined pairs keep group-first observed-pair ordering.
* Facets prepare full parent domains before slicing. Only source occurrences with
  an actual guide contribution create entries; an empty semantic cell creates
  none. Sort entries by parent domain ordinal, not which occupied facet is first.
* Canvas contributors are declared row-major Plot slots, including gaps; fields
  and values union by that order and then their prepared source-domain order.
  Hidden Plot entries add no section, but their visible mark color identities
  still coordinate global colors. Never use legend presence as the color domain.
* Literal keys preserve observed source order: BarSign nondecreasing/decreasing;
  endpoints left/right; Regression observations/fit; layers append order before
  duplicate merging. Shared literal union preserves first contribution order.

Cross-family section sharing requires identical section identity, title, entry
label and complete projected explanatory key, including mapped/constant masks,
component presence, alpha, radius, shape, pattern, widths and structural outlines.
Equal RGB is insufficient. Equal final keys from different producer families are
allowed; do not add a hidden family discriminator. Source-role names/address
provenance are retained for validation/reuse but do not make otherwise identical
explanatory keys incompatible. A point-only Scatter/Trajectory can share when
the actual resolved key and explanation match; Density line+fill cannot merge
with a point merely because the color field is the same.

Group projections from different Plot themes must agree in their retained neutral
RGB too. Canvas guide text uses Canvas typography, but the key still uses source
Plot label RGB. Different group fields can coexist with a common hue field only
if section titles are unambiguous and repeated hue keys match exactly.

Keep `Canvas::validate_shared_guide_kinds` before fonts. One Shared guide is still
Discrete or Continuous; multiple titles do not authorize mixing those kinds.
PerPlot suppression/placement uses actual Plot policy and appearance; Shared
owns one outside guide, suppressing local placement rather than merging local
measured guide boxes. Facet inside selection owns the one shared key set even
when drawn at a selected occupied cell. No topology compaction accompanies it.

## 8. One Measured Guide, Exact Geometry And Bounded Flow

### Outer Frame And Entry Measurements

All geometry is canonical spec05 `i64` quanta (1/1024 CSS pixel); strings and
scientific identities never become spatial equality keys. Exact rational local
spacing follows spec94: complete hard requirements round upward; positions/
preferences round nearest-even; lower/upper ink edges round outward. Checked
wide intermediates precede admission/reservation. No individually rounded gap
sum or floating cardinality replaces a complete exact requirement.

The guide has one optional outer frame, one horizontal padding at each edge and
one vertical `padding + glyph_edge_padding` reserve at each edge. Add comfort
exactly once, not once per section/key. Frame outline/radius minimum and outward
stroke ink are traced independently. Frame absence removes chrome, not padding,
content, key occupancy or collision bounds. Use spec94's declared S/H operands;
its default complete vertical hard inset is 11265 quanta while the placement
operand is 11264. Do not silently rewrite it as 11px.

Let W be assigned inner width after complete hard horizontal insets; optional H
is assigned inner height after the one outer vertical hard reserve. Negative or
unusable space returns a typed fit miss. Sections stack top-to-bottom in semantic
order in **both** orientations, each receiving W. Section gap is the exact sum
of title_gap against title em and row_gap against label em; no new field exists.

For each entry, measure the exact key grammar using the same geometry materializer
as frozen drawing (current `LegendEntry::glyph_ink` / `Legend::visit_calls`).
Fill handles keep current local 0.32-label-em half-height and full handle width;
line endpoints/caps, point shape/radius and all painted outlines participate.
Transparent components still retain semantic/key structure. Nominal handles,
font-size heuristics or a different unprojected glyph cannot bound painted ink.

Let P[s,i] be key/chrome plus natural label preferred width, M[s,i] its complete
hard unbroken requirement, a[s,i]/d[s,i] its baseline-aligned upward/outward
ascent/descent, and h[s,i]=a+d. The key center aligns to the first text baseline;
use complete text-plan ink/advance and key extents, not just line-height. For an
empty label, the key defines height. Title plans start-align and include their
full occupied ink/line height at W. Empty title occupancy is zero. Text/key hard
width checks precede unnecessary shaping of later entries.

P and M include the full hard key slot, not merely nominal key_width. Derive its
left/right outward extents from the actual glyph, reserve their complete span,
then the exact key_label_gap and label requirement. Point radius and stroke
normalization are the originating mark roles; handle width/rectangular fill
height are guide-local metrics consumed identically by measurement and selected
drawing. Nondefault width, outline ratio or point shape must change occupied key
space even when the text and nominal handle do not change.

### Global Normal/Compact Widths

For horizontal (outside Top/Bottom, including suffix variants):

```text
normal:  E[s,i] = min(P[s,i], W)
compact: E[s,i] = min(P[s,i], C2)
```

For vertical (outside Left/Right, including suffix variants; inside anchors use
the existing vertical guide orientation):

```text
normal:  C[s] = min(max_i P[s,i], W)
compact: C[s] = min(max_i P[s,i], C2)
C2 = floor((W - column_gap) / 2) in layout quanta
```

Column gap's finite placement operand is used in C2. Each trial checks its actual
complete hard width, including exact gap sums, afterward. Nonpositive C2 or an
allocation below M misses. Labels measure once at E/C minus their own exact
key-plus-gap width; titles still measure at W. Actual narrower label ink does
not shrink the fixed entry allocation during packing.

Compact is one global variant, not a choice per section. If every entry's width
and all consumed inputs equal the normal trial, reuse its outcome and do not
evaluate another placement variant. A changed width remeasures dependent labels,
baseline heights and section capacities; old heights are not compact evidence.

### Horizontal And Automatic-Height Flow

Each horizontal section greedily forms rows in entry order at W. A row's height
is **max(ascent) + max(descent)**, not max(individual h): different entries can
own the largest ascent and descent. Exact inter-entry gaps, row gaps and final
complete ink must fit. The section title occupies its own full-width row above
the body. Stack actual section heights and gaps and test the total against H;
there is no arbitrary per-section height partition for horizontal flow.

With vertical automatic height, every section uses one column at C[s], derives
its complete height, and stacks with section gaps. Do not invent a finite height
to force extra columns or start an automatic grow-until-fit loop.

### Vertical Finite Height: One Allocation, No Redistribution

After measuring titles and entries at the trial's fixed widths, define capacities
in canonical integer quanta, with exact gap products inside the upward rounding:

```text
T[s] = occupied title height + active exact title gap
hard[s] = ceil(T[s] + max_i h[s,i])
preferred[s] = ceil(T[s] + sum_i h[s,i]
                    + (entry_count[s] - 1) * exact_row_gap)
section_gap_reserve = ceil((section_count - 1) * exact_section_gap)
A = H - section_gap_reserve
```

Here preferred is the capacity required by the preferred one-column content, a
hard occupancy calculation, not a new soft cosmetic length. Thus preferred >=
hard. Fractional T/gaps retain exact rational operands until these complete
ceilings. Capacity/band boundaries remain integer quanta. Counts subtract only
after proving each section nonempty.

If A is negative or below sum(hard), return a LegendOrGuide miss. Otherwise:

* For A >= sum(preferred), each section receives preferred; extra capacity is
  unused slack, not added to an inflexible guide or divided into new attempts.
* For smaller A, run spec05's one proportional shrink with minima hard and
  preferences preferred. Compute floor shares using checked integer numerators;
  remaining shrink quanta go by descending fractional remainder, ties by
  ascending section ordinal. Capacities sum to A and never cross hard.

Each section subtracts its own exact title occupancy from its assigned capacity
and greedily packs whole entries top-to-bottom into equal-width columns. An
entry fits when the complete exact prefix, preceding gaps and its own occupied
height fit the capacity. Otherwise start the next column; no entry or heading
is split. The section title spans its complete column body; no continuation
title needs repetition. Complete column widths/gaps must fit W.

Stack **actual returned occupied heights**, translating immutable section plans.
Unused assigned capacity is not offered to another section, and failure never
retries an allocation. Within a column use original entry order; column width is
the fixed C[s], not the width of whichever entry triggered the break. No section
interleaving, permutations, sibling products, width scans or independent variant
choices exist. A policy miss is possible although another distribution would fit.

The sufficient per-entry bound is proven directly: each assigned capacity >=
ceil(T+max h), so every entry can fit by itself below its title. Greedy column
flow therefore either consumes the next entry or fails width/geometry; it cannot
loop on a nonfitting entry. Actual section height <= capacity; their sum plus
ceil of exact section gaps <= H. Horizontal/vertical actual lower/upper ink is
then checked against the one final box; these bounds do not waive font or ink
overhang failures. Single-section flow reduces to its existing normal/compact
capacity policy. There is no special old flat renderer for that case.

### Worked Capacity And Flow Fixtures

These are independently specified **inner** capacities after frame insets; test
text/key fixtures must establish the stated measured widths/heights separately.
They are not golden output generated from the new algorithm.

1. Horizontal, W=100, title occupancy 12 per section, section gap 8, row/column
   gaps 4/5. Section A entry widths [60,50], heights [10,10]; B width [90], height
   20. Normal A needs two rows: 12+10+4+10=36; B needs 32; total **76**. H=76 fits;
   H=76-1 quantum misses. Compact C2=47.5 requires freshly measured widths
   [47.5,47.5]/[47.5] and label heights [20,20]/[30]: A one row height 20, total
   A=32, B=42, sum with gap **82**. Compact can be taller and fail at H=76.
2. Vertical W=100, column gap 10, title occupancy 10, section gap 10, row gap 5.
   A: C=45, heights [20,20,20], hard=30, preferred=80. B: C=90, heights [20,20],
   hard=30, preferred=55. H=100 gives A=90, deficit 45 over capacities [50,25].
   Shrink [30,15] gives section capacities **[50,40]**. A body capacity 40 forces
   three columns (155>100); B body 30 forces two (190>100); normal misses.
   Compact C2=45, with independently measured heights unchanged, keeps A's
   three-column miss; it does not take spare B capacity to rescue A.
3. Same vertical data at H=120: A=110px, deficit 25px. Allocation operates in
   **quanta**, not rounded CSS pixels: capacity differences [51200,25600],
   total 76800, deficit 25600. Floor shrink [17066,8533], remainders
   [51200,25600]; the remaining one quantum goes to A. Assigned capacities are
   **[64853,47787] quanta**, exactly [63.3330078125,46.6669921875]px, not [63,47]px.
   A body 53.3330078125 fits two 20px entries in column one and one in column
   two; body occupied height 45, section actual 55, width 100. B body
   36.6669921875 has two columns: normal width 190 misses. Compact B C=45 with
   the same measured heights fits width 100; B section actual 30. Total actual
   guide height is **55+10+30=95**, not 120. Unused capacities [8533,17067] quanta
   are not redistributed. All operands of examples 1-3 are CSS pixels unless
   labeled quanta; convert integral CSS inputs by checked multiplication by 1024.
4. Integer-quanta allocation independent of fonts: hard=[2,2,2], preferred=
   [5,5,5], A=10. Deficit 5, capacity 9, floor shrink [1,1,1], equal remainders;
   the two extra quanta shrink sections 0 and 1. Capacities **[3,3,4]**.
5. Baseline boundary: a row with entry (a,d)=(18,2) and (5,15) occupies **33**, not
   20. A large stroked triangle can raise either operand independently of font
   size. A two-line title's full ink/leading likewise precedes body allocation.

## 9. Natural Sizing, Placement And Collision

Intrinsic summaries contain hard/unwrapped/preferred operands, not alternative
measured guides. Natural preferred guide width is the max of section title
preferences and complete entry preferences, plus one horizontal chrome reserve.
At that width, preferred Right uses one column per section and their stacked
height. The source Plot or Canvas composes this with body/axes/prose once.

Automatic outside guide preference is initially Right. Initial root width is
chosen once from that bottom-up composition. Later assigned-width height and
edge trials use this same descriptor and coordinated domains; a chosen Top or
compact flow can change content-derived height but never restart root width.
Exact-width and exact-pair roots use their actual constraints. Shared Canvas guide
preference is composed once, not once per contributing panel. Embedded Plot's
standalone request is not its Canvas cell preference.

The containing frame tries Right, Top, Bottom, Left, each normal then effective
compact. Explicit outside edges can only normal/compact reflow at that edge;
explicit inside anchors remain fixed and use existing inside allocation/collision
policy. There is one guide edge/flow budget (at most eight automatic, two explicit
outside per owner per root round); sections add no trials. Ancestor remeasurement
does not reset it. A cached identical trial charges the existing call/lookup, not
a second evaluation. Resource/font/arithmetic failure is fatal, never an edge
retry. Provisional feedback stops local alternatives as spec05 requires.

Success includes the complete affected scientific body, axes, all labels and
annotations, not just guide fit. Axis wrapping/tick choices can change with
allocated residual space under their existing owners; there is no joint tick/
section objective or tick-count reduction chosen to rescue a section.

The selected complete guide box is one inside collision obstacle for marks,
panel text, significance brackets/labels and other admitted overlays. Sections
do not claim independent anchor alternatives or annotation feedback. Collision
and domain-clearance consequences are frozen at the selected guide edge/anchor,
including exact projected key/frame ink. No guide-only success is published.

Actual consumers: `layout/interior/sizing.rs::{GuidedPreference,guided_preference,
guided_body_height}`, `canvas/sizing.rs`, `canvas/tracks.rs`,
`orchestrator/facets/{sizing,tracks,planning}.rs`,
`decorations/plan.rs::{AssignedGuideFrame,plan_panel_with_context,plan_panel_attempt}`,
`canvas.rs::{build_canvas_plan,build_canvas_attempt}` and Matrix's local owner.

## 10. Addresses, Pre-Font Preparation And Flat Retirement

### Existing Slots And New Section Descendants

Retain existing guide slots and their root/child owners:

* Plot local: containing panel owner / `address_role(position)[0]`; Auto is
  `OutsideGuideAuto[0]`, explicit edge roles are the corresponding OutsideGuide
  role, inside is InsideGuide. `decorations/plan.rs:444-452` is the current scope.
* Facet shared Plot guide: root / `address_role(position)[0]`
  (`orchestrator/facets.rs:279-283`); routing to an occupied inside cell does not
  create another descriptor/semantic owner.
* Canvas shared: root / `CanvasSharedLegend[0]` (`canvas.rs:4703-4707,5127-5134`).
* Canvas PerPlot: existing row-major `CanvasCell[index]` child ownership, with the
  Plot guide beneath that child; no root shared slot becomes a second owner.
* Matrix: its existing Plot-local InsideGuide[0] scope in `matrix_legend`
  (`matrix.rs:1194-1198`) and its existing selected placement/collision owner.
  Do not use the generic dispatcher's Matrix error to bypass this migration.

Add closed AddressRole::LegendSection beneath each discrete guide slot. Section
s has `LegendSection[s]/LegendTitle[0]` and
`LegendSection[s]/LegendEntry[i]/{LegendKey[0],LegendLabel[0]}`. Empty title/label
has the structural slot but no painted-text requirement. All addresses are
parent-scoped copies; no anonymous second root, global entry ordinal, per-column
section address or edge-dependent reparenting is permitted. Flow rows/columns,
section capacities and routed guide references are inline plan records, not new
semantic children/call addresses. Slot topology and eventual retained records
are admitted separately under the existing ledger.

### Preparation Order And Font Census

1. Resolve effective owner, complete themes, policies and admitted scientific
   inputs; preserve source occurrence/domain order and actual style visibility.
2. Coordinate parent/facet/Canvas raw color domains, including hidden-entry marks.
   Obtain them from prepared scientific color sources, never projected sections.
3. Materialize complete producer keys with bindings under those resolved domains;
   apply effective composition, same-field collapse and exact projection checks.
4. Merge/order typed sections, resolve all final titles/labels, check collisions
   of semantic display identities, and retain one immutable descriptor per guide.
5. Census only those final visible descriptor strings under owning guide text
   roles, at the section/entry addresses. Include key/typography metric probes.
   Shared strings use Canvas typography; projection-neutral RGB used Plot paint.
6. Resolve fonts once through the canonical operation, then prepare intrinsics,
   allocate/measure, freeze selected calls and publish.

Descriptor None in a prepared operation is authoritative absence. Missing
scientific/guide preparation is an invariant error, not a request to probe a body
or reconstruct semantics. Shared-suppressed panels retain their prepared source
contribution for the parent and no local placement operand; Hidden panels retain
only their scientific color-domain source. Neither uses None as a fallback flag.

Delete the parallel inferred discrete title/value/layer font strings in
`layout/text/requirements.rs::push_deferred_plot_requirements`, including its
`legend_title_census`/discrete label paths, in favor of descriptor traversal.
Per-point labels, facet headers, continuous labels and numeric alphabets keep
their own census. Neither a hidden raw group identifier nor an omitted key
component becomes a guide font requirement. Canvas's current single title and
flat LegendLabel ordinal traversal (`canvas.rs:4374-4448`) migrates to the exact
same section preorder. Ordinary Plot and Matrix preparation must supply the
descriptor before `foundational_plot_requirements`, not reconstruct it afterward.

### Exhaustive Main Consumer Migration

| Current flat owner / function | Single replacement |
| --- | --- |
| `decorations/legend/descriptor.rs::PreparedDiscreteGuide` tuple | Struct above; Discrete carries its admitted Arc, Continuous unchanged |
| `decorations/legend.rs::{LegendIdentity,LegendEntry,resolve_entries,merge_exact_entries,resolved_title}` | Typed section/entry identity, provenance, per-section merge/label validation; no per-entry title compatibility fallback |
| `layout/interior/guide.rs::PreparedInteriorInputs::guide_entries` and all census-table producer functions | Return the one composed descriptor from prepared semantics; no flat materialization followed by a second composition route |
| `spec.rs::PlotRenderResult.legend_entries`, constructor and every caller | Retire field; retain `discrete_guide: Option<Arc<PreparedDiscreteGuide>>`. Noncontributing results pass None. Selected marks and guide share the source style materializer |
| `orchestrator/facets.rs::PreparedFacetGuide.{title,entries}` | One optional descriptor plus independently retained scientific coordinated domains; no duplicate flat title/keys |
| `orchestrator/facets/legend.rs::{prepare_discrete_guide,order_entries}` and sizing/planning callers | Parent-domain section ordering and exact descriptor union; panel shared suppression receives the descriptor by immutable reference |
| `canvas.rs::{prepared_discrete_shared_guide,preflight_shared_legend}` | Pre-font authoritative descriptor. Later body probes retain/use it; remove the `None -> collect result.legend_entries -> merge` reconstruction route |
| `canvas.rs::{prepared_point_path_hue_groups,canvas_hue_groups}` | Continue hidden-mark raw domain coordination from prepared producers; remove dependence on flat/projection entries and `contributes_hue_level` guesses |
| `decorations/plan.rs::{plan_panel,plan_panel_with_context,plan_panel_attempt}` | Select/measure an explicitly prepared descriptor or explicit absence. Do not replace a discovered flat Vec or derive a late resolved_title |
| `decorations/legend.rs::{discrete_guide_intrinsics,natural_preference,measure_assigned,flow_assigned_entries,observe_feedback,visit_calls,select}` | Section-aware single measurement/materialization; one outer box and frozen LegendDrawPlan |
| `decorations/plan/children.rs::{Inputs,Panel,ChildPlans::plan}` | Effective scoped composition and exact descriptor reference/value equality, not an Arc<[LegendEntry]> override |
| `layout/interior/sizing.rs::GuidedPreference`, Canvas/facet sizing/tracks, content-height keys | Same descriptor for initial preference, width-dependent height and selected frame; no semantic preparation inside sizing |
| `orchestrator/lifecycle.rs::PreparedPlotInputs`, Canvas prepared inputs and discrete text census | Prepared owner-local descriptor carried through fonts, intrinsics and selected output |
| `plots/matrix.rs::{matrix_guide_entries,matrix_legend,natural_guide_preference}` | Local pre-font descriptor, same measured section guide and retained draw plan; no late independent Matrix flat guide |
| Producer extraction/parity tests, benchmark guide fixtures, local/shared selected capture | Compare typed descriptors and exact source/key counts with originating marks; publication consumes selected operands only |

Combined's one section is not a `CombinedFlat` descriptor, alias, legacy Vec,
alternate renderer or mode-specific cache. Remove any transitional flat storage
before the replacement leaf closes. A traversal iterator over entries is allowed
for diagnostics/ledger counts; collecting it into a second semantic guide is not.
No late body-probe fallback, first-pair default, post-font label reconstruction
or draw-time source reads survive this retirement.

## 11. Scoped Identity, Feedback, Admission And Frozen Plans

Source scientific inputs and immutable configuration belong to the root operation.
Fresh operations resolve changed themes/styles/domains before fonts. Within an
operation, parent-scoped copies preserve effective composition, prepared
descriptor, source identity, global hue/group/pair/condition domains, Labels,
exact style inputs and visibility. Canvas Shared overrides effective composition
in that copy without mutating the public child selection. Hidden copies retain
scientific domain records but no consumed discrete descriptor.

The live `decorations/plan/children.rs::Inputs` checks geometry, axis_label_width,
content_height/fill_content_height, StandardPanelInputs, retained ComposedInputs,
complete theme, facet scales and relevant feedback. `builder/facets.rs::{
same_composed_inputs,retain_composed_inputs}` currently retains computed ranges,
global domains, scope, visibility and legend policy but lacks section policy.
Add effective composition and the descriptor references actually consumed by the
child. `Panel.legend_entries` and `LegendEntry::same_child_input` retire with them.

Retain an explicit, ordered private dependency input for each consumed guide:

```rust,ignore
enum GuideUse { Hidden, SharedSuppressed, Local }
enum DependencyUse { AllocationOnly, DescriptorAndAllocation }
struct ConsumedGuideDependency {
    owner: ComponentAddress,
    use_kind: DependencyUse,
    descriptor: Option<Arc<PreparedDiscreteGuide>>,
    assigned_edges: [i64; 4],
    orientation: GuideOrientation,
    compact: bool,
    feedback: ChildFeedbackInputs,
}
enum GuideOrientation { Horizontal, Vertical }
struct ScopedGuideInputs {
    composition: LegendComposition,
    use_kind: GuideUse,
    local_descriptor: Option<Arc<PreparedDiscreteGuide>>,
    dependencies: Arc<[ConsumedGuideDependency]>,
}
```

AllocationOnly has no descriptor operand and records only the exact containing
allocation and relevant feedback actually used by that child. For example a
Shared-suppressed panel consumes the residual body allocation, not another Plot's
neutral key RGB. DescriptorAndAllocation retains full descriptor equality as well
as those spatial inputs; the parent measuring/placing the Shared guide consumes
that dependency. Dependencies are enumerated by the actual preparation/measurement
call in canonical owner order, not discovered by scanning unrelated siblings.
An empty list is meaningful. The local key, parent-scoped source and declared
dependency uses are retained in ComposedInputs, child and height keys; changing a
use cannot preserve a hit. Hidden has no local descriptor. GuideOrientation/compact
are retained only where selected/assigned measurement consumes them; incoming
normal/compact measurement identity is separate from scientific source identity.

Complete consumed descriptor equality checks composition, section count/order/
boundaries, raw identities, titles/labels, full keys/explanation masks, retained
source provenance and selected neutral RGB. Equal outer extent, equal text bytes,
an Arc pointer or an unchanged palette alone cannot authorize reuse. Immutable
same-operation sharing may avoid copies, but equal-value fallback is admitted
and exact. Relevant roles/typography/constraints/origin/orientation/compact widths
and parent allocation are also in the consumed measurement identity.

Feedback remains selective. `FeedbackRound::{child_inputs,child_inputs_equal,
child_stable,replay_child}` currently scopes state to owner and owning Grid
(`children.rs:49-66`, `kernel/feedback.rs:195-285`). Retain consumed descendant
references explicitly when an outside/shared guide or coordinated alignment lies
outside that simple subtree. Their current descriptor/allocation/feedback values
must match and their owners must be stable before a hit. Do not compare every
unrelated root sibling or incorrectly exclude a shared consequence just because
its address is outside CanvasCell. Zero insets remain semantically inactive under
the current feedback rules. Changed relevant guide metadata invalidates reuse
even if the parent's box or consumed numeric domain did not change.

`canvas/sizing.rs::HeightInputs` currently compares width, ranges/provenance,
resolved x, local/parent insets, labels, legend policy, continuous suppression
and scope. Retain effective composition and consumed descriptor/allocation and
descendant dependencies there too. Width equality cannot reuse a height measured
with different sections, full glyph geometry, title leading or coordinated domains.
Height feedback replaces the previous consequence, including contraction; it
does not aggregate guide height as cumulative owned ink or rechoose root width.

Retained sections, keys, strings, addresses, text/glyph plans, column slots and
feedback use checked lengths and fallible storage. Immutable Arc sharing and
complete cache identity remain. Spec95 removes accounting fees, cumulative
claims and resource quotas; no comparison/call ticket is required. Real
reservation/arithmetic/font errors remain fatal rather than factorization errors
or fit retries. More sections do not grant additional guide placement attempts.

Observe selected composition/section identity, title/entry geometry, capacities,
flow slots and exact glyph consequences, not just outer width/height. Replayed
ChildFeedback reproduces guide-trial events and relevant observations under the
same owner budget. Rejecting an ancestor placement does not reset child trials.

Before publication freeze all section boxes/translations, text lines/glyphs,
key point/line/fill operands, optional frame and selected anchor/edge, together
with complete collision bounds. `LegendDrawPlan` remains the sole guide drawing
operand carrier. Backend/PNG/outlined SVG consume this plan with retained font
identities; preserve-text uses its selected line structure. No backend consults
Plot, themes, provenance, section selection, data scales or source files again.
Private measured-metadata poisoning after selection must leave output unchanged.

Categorical source domains/section identities are immutable after preparation
within a root operation. Numeric-axis gutter/domain feedback can alter allocated
space and guide flow but does not introduce a new categorical key or string after
fonts. Selected producer results attach the already prepared descriptor rather
than reconstructing it from projected marks. They may materialize actual mark
styles through the same retained source inputs/helper and check key parity; that
is bounded producer work, not a new guide planning or font route.

When foundation changes land before section implementation, refresh the named
producer inputs in a bounded source check: role paths, optional outlines,
supplemental defaults, mandatory-strip split, complete Color and style masks.
That check updates evidence against these fixed owners; it is not fresh layout
planning, new decisions or authorization for another renderer/default route.

## 12. Required Evidence And Replacement Order

### Existing Meaningful Tests To Migrate, Not Claim As New Evidence

| Contract | Live test names / files |
| --- | --- |
| Source key/mark parity | `prepared_guide_glyphs_match_selected_marks_without_fonts_or_a_panel_search`, `custom_primary_point_paints_match_prepared_guides_and_hidden_outline_stays_hidden` in `src/plots/scatter.rs`; `prepared_path_family_glyphs_match_selected_marks_before_fonts`, `custom_series_line_style_is_shared_by_selected_marks_and_guide_identity` in `src/plots/line/tests.rs` |
| Actual distribution/strip styles | `custom_raincloud_overlay_point_changes_mark_and_exact_guide_glyph`, `custom_violin_body_preserves_selected_geometry_and_changes_exact_guide` in `src/plots/categorical_distribution.rs`; `visible_modes_draw_exactly_one_fixed_style_point_per_observation` in `tests/distributions/raw_overlays.rs`; `poisoned_capture_repeats_no_sources_statistics_projection_axes_or_shaping` in `src/plots/categorical_distribution/extraction.rs` |
| Ribbon scientific grammar | `generic_line_band_uses_exact_precomputed_geometry`, `line_band_rejects_invalid_bound_data_at_runtime` in `tests/lines/effect_bands.rs` |
| Pair identities/collapse | `explicit_group_and_hue_render_only_observed_pair_paths`, `distinct_equal_pair_values_keep_independently_resolved_legend_text`, `equal_same_field_channels_collapse_legend_text`, `conflicting_same_name_group_and_hue_channels_reject_for_every_pair_family` in `tests/categorical/grouping.rs` |
| Row hue/no fabricated group | `changing_hue_colors_points_but_not_connector_or_group_legend`, `neutral_path_with_hidden_markers_suppresses_hue_legend`, `trajectory_group_identities_are_not_preflighted_as_legend_text` in `tests/lines/paths.rs` |
| Independent Interaction styles/sparse facets | `interaction_style_legend_matches_each_trace_glyph`, `faceted_interactions_keep_sparse_global_pair_styles_and_colors`, `crossed_group_hue_interactions_render_only_observed_pair_traces` in `tests/interactions/styles.rs` |
| Layer source/count/style identity | `point_labels_and_whole_layer_legend_identity_remain_distinct`, `repeated_explicit_colors_and_exact_duplicate_legend_entries_are_accepted`, `conflicting_duplicate_point_layer_legend_identity_is_rejected`, `explicit_point_layer_consumes_its_automatic_palette_position`, `canvas_absent_hue_levels_do_not_consume_accent_slots` in `tests/lines/numeric_layers.rs` |
| Shared exact/hidden behavior | `shared_canvas_legend_merges_typed_collective_pairs`, `hidden_canvas_legends_do_not_preflight_suppressed_glyphs`, `hidden_discrete_guide_still_coordinates_visible_mark_colors` in `tests/composition/canvases.rs`; `shared_canvas_legend_rejects_conflicting_caller_color_mapping`, `shared_canvas_legend_rejects_conflicting_point_outline_mapping` in `tests/interactions/styles.rs` |
| Bounded complete guide flow/ink | `assigned_discrete_flow_matches_horizontal_and_vertical_worked_examples`, `compact_vertical_flow_remeasures_labels_and_fits_two_columns`, `entries_in_one_row_share_a_baseline_and_union_their_ink_heights`, `local_and_shared_discrete_guides_allocate_complete_custom_glyph_and_frame_ink`, `glyph_changes_invalidate_feedback_even_when_legend_extent_is_unchanged` in `src/decorations/legend.rs` |
| Actual cache/feedback | `actual_child_hit_is_invalidated_by_legend_only_paint_and_geometry_changes`, `child_keys_include_exact_origin_computed_policy_and_relevant_feedback`, `later_panel_cache_replays_feedback_without_remeasurement` in `src/decorations/plan/children.rs`; `rejected_ancestor_attempts_do_not_reset_child_guide_budget` in `src/layout/kernel/feedback.rs` |
| Matrix/local and natural root owners | `many_hue_matrix_legend_preserves_all_labels` in `src/plots/matrix.rs`; `matrix_content_roots_compose_the_native_hue_guide_and_variable_square`, `owning_facet_discrete_guide_preference_is_composed_once_at_its_explicit_edge` in `src/orchestrator/lifecycle.rs` |
| Selected output/containing fit | `standalone_capture_uses_selected_legend_operands_not_measured_style_or_position` in `src/orchestrator/lifecycle.rs`; `shared_guide_recovery_measures_the_complete_canvas_body`, `content_shared_guide_recovers_after_measured_gutters_without_reselecting_width` in `src/canvas.rs`; `inside_legend_resolved_box_does_not_overlap_bracket_or_label` in `tests/annotations/significance_layout.rs` |

### New Fixtures And Independent Expectations (All Pending)

The following names identify required future Rust fixtures, not executed tests.

* `discrete_kind_census_preserves_actual_key_components`: all 17 Legend kinds and
  eight remaining kinds, with exact entry/P/L/F/source counts. Line cases cover
  visible/hidden markers, full/missing/zero-area ribbon; distributions cover
  zero key raw-overlay points and mandatory-strip outline suppression. Regression
  covers Automatic absent versus Visible, band/fit component changes. Matrix is
  local, not dispatched as an absent guide.
* `independent_interaction_sections_have_marginal_not_cartesian_counts`: full
  two-group/two-hue crossing: Combined 4 keys each P+L, Separate 2 sections/4 keys
  each P+L; sparse three-pair crossing: Combined 3, Separate 4. Dense 5x4 fixture
  has 20 versus 9. Marks/intervals/source rows remain equal; compare independently
  listed raw identities/order, not simply the implementation's returned count.
* `group_neutral_uses_source_label_rgb_and_original_mapped_alpha`: distinct source
  Plot label/axis/annotation/neutral-path paints and Canvas label paint. Pin the
  `(40,50,60,127)` example above, fixed outline equality and mapped-outline alpha.
  A zero-alpha text role must not make its group key transparent.
* `hue_projection_changes_only_bound_interaction_style_slots`: triangle/dashed
  group becomes Circle/Solid for hue; radius, 2.5px custom connector width, independent
  fill/stroke alphas and hidden outline stay exact. Hue-only theme triangle/pattern
  remains untouched. Repeated explicit styles/colors remain accepted.
* `separate_unsupported_pairs_reject_before_fonts`: each unsupported pair family,
  including one-pair and equal-RGB cases; same-field complete key accepted;
  Hidden bypasses guide error/fonts but keeps missing-map/scientific validation.
  RowHue subjects with unpaintable identifiers create no group requirements.
* `layer_identity_and_display_context_are_independent_of_composition`: categorical
  source context humanizes/aliases a raw layer label, unkeyed context keeps it
  verbatim, hidden neutral-path points do not change that context;
  duplicate exact label/style merges, different alpha/radius rejects; a matching
  categorical label cannot alias the layer identity; source append/accent counts
  and per-point labels stay unchanged.
* `shared_sections_merge_exact_explanations_in_canonical_order`: different group
  fields with one hue domain; early supplemental-only contributor cannot choose
  Combined title; canonical class/first-occurrence order; same displayed title on
  different identities rejects; Joint/HueColor does not collapse. Customize
  Scatter/Trajectory to equal full point keys for positive cross-family sharing,
  then vary one radius/outline/mapping slot for a negative case. Hidden-only hue
  identities must not leak through visible row-hue global color lists; facet
  parent union must still retain values from other occupied cells.
* `section_flow_matches_independent_capacity_vectors`: exact §8 worked examples,
  independent fixed text/glyph metrics, canonical u32 counts and quanta. Pin
  normal/compact row/column membership, remeasured heights, capacities and actual
  returned height. Test no redistribution and equal-remainder ordinal ties.
* `section_hard_boundaries_include_title_and_large_glyph_ink`: nondefault large
  title leading/ink, thick line caps, stroked triangle, one-quantum insufficient
  capacity, long unbreakable first label, frame/outline absence and edge comfort
  applied once. Distinguish a guide fit miss from a fatal ledger/font refusal.
* `section_descriptor_is_the_only_font_and_selected_output_source`: exact final
  section addresses/role order, aliases and layer context, Hidden no unused
  strings, census before measurement; poison source/measured metadata after
  selection and compare frozen SVG/PNG/capture. No extra shaping on publication.
* `section_changes_invalidate_actual_scoped_cache_hits`: hit a real unchanged
  child, then vary effective composition, section boundary/identity, source label
  neutral RGB, glyph mask, typography, title or consumed descendant dependency
  without changing outer width/domain. Unrelated sibling feedback must not force
  a miss. Cache-enabled/disabled selected bytes and source counts agree.
* `section_height_feedback_keeps_initial_root_width`: automatic, exact-width and
  exact-pair standalone/facet/PerPlot/Shared; one initial preferred-Right width,
  selected Top/compact recovery and contracted height; complete axes/annotations
  remain present. A changed guide-only consequence cannot reuse stale HeightInputs.
* Preserve real section/source identity, checked storage and immutable replay.
  Normal/compact/ancestor replay share one guide owner and its existing attempt
  rules. More sections never grant more trials. Do not add resource-fee tests.
* Public compile/setter fixtures bind only the 17 Legend kinds; all owner precedence
  and setter orders are explicit. A common Canvas selection is harmless under
  PerPlot/Hidden and changes Shared only. No continuous/Any extension is tested
  merely through an obsolete import failure.

### Bounded Replacement Order

1. Replace semantic storage and all Combined callers atomically, including actual
   layer identity, result fields, Matrix and census sources. Preserve current
   scientific/component counts and specified Combined defaults; attribute layer
   collision corrections explicitly. No flat fallback remains at leaf closure.
2. Add producer mapping/projection validation under the same private descriptor;
   exercise truthful family cases and pre-font refusal without public placeholders.
3. Implement the one section flow/frame/frozen-key materializer and worked bounds;
   extend actual natural/assigned owners, not a second measuring backend.
4. Close facet/Canvas authority, raw color coordination, descriptor font preorder,
   consumed scoped/cache/feedback references and content-height identity.
5. Expose the small public selection and migrate its documented current contract
   and examples. Keep default Combined and the existing capability matrix.
6. Close focused end-to-end semantic, complete-ink, output and resource evidence.

Each code leaf uses focused affected unit tests and representative integrations,
format checking and `cargo clippy --locked --offline --all-targets -- -D warnings`
(or the equivalent Justfile lint recipe). Never use cargo check. Documentation
refinement needs source/spec review and diff checking, not tests/builds. Required
broad-phase verification stays at the owning approved closure gate; do not repeat
full suites for every descriptor consumer or localized correction. Inspect
intentional SVG changes with semantic/component counts and complete geometry.
Commit subjects/bodies describe enduring behavior with mandatory scopes, never
temporary planning identifiers. No implementation evidence is asserted here.

## 13. Closure Conditions

This target leaves no undecided neutral source, family applicability, layer text
policy, section ordering, Shared authority or finite-height allocation. The
implementation prerequisite is concrete: current flat producers do not retain
mapping provenance, all owners do not yet carry a pre-font descriptor, and current
cache/census paths do not consume section metadata. Those are specified migration
tasks, not a reason to invent a first-pair default, late reconstruction or another
flow search. Construction/section source refresh is bounded by the named owners.

The replacement is complete only when all 25 kinds and their valid parents are
accounted for, every retired flat field/function path is gone, fonts/caches use
the same prepared descriptor as sizing/selected output, and the new tests have
actual recorded results. Until then this document remains an approved non-current
architecture and its proposed acceptance evidence remains pending.
