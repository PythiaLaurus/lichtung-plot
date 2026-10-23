# Visual Property Ownership

## Responsibility Boundaries

Every visual input has one semantic owner. Equal initial values do not couple
independent roles or authorize a fallback to another role. Preparation, intrinsic
requirements, measurement, collision checks, guide keys and frozen drawing must
consume the same selected property and its actual consequences.

Public configuration uses `property(value)` edits with same-type paired
`get_property()` readers. This naming convention does not add property owners or
change exact-style precedence, structural absence or validation timing; read-only
inspection retains its bare name.

| Concern | Owner | Boundary |
| --- | --- | --- |
| Scientific values, availability, grouping, statistics and geometry | Closed input and scientific producer | Theme/format/alpha cannot mutate membership or statistical definitions |
| Titles, labels, annotations, exact ticks and supplied strings | Caller content and typed semantic policies | Theme cannot silently remove or rewrite them |
| Visibility, placement, ordering, faceting and range sharing | Plot/Canvas policy | Appearance does not invent a policy or new scope |
| Non-data appearance | Complete Theme components | No family-local theme, global state or sparse runtime merge |
| Raw category/numeric-to-RGB mapping | Plot scale over Theme ColorScales | Display aliases are not keys; mapped colors are opaque |
| One element's exact paint/geometry | Its typed exact style | Override only the properties that style actually owns |
| Scientific component alpha | Plot-local AlphaEncoding for that target | Replace one final byte, independently of outlines/text/layers |
| Numeric spelling | NumericFormat and consumer default profile | Separate from data transformation, typography and tick generation |
| Explanatory content | Prepared source bindings and LegendComposition | Guide chrome cannot infer mappings or choose representative rows |
| Layout, clipping, orientation, draw order and feedback | Private renderer | Not Theme fields or application callbacks |
| SVG/PNG publication options | Terminal call | No sticky Plot/Canvas/Output format state |

[Theme construction](03_THEME_SYSTEM.md), [layout](05_LAYOUT_SYSTEM.md),
[numbers](06_NUMERIC_PRESENTATION.md) and [guides](08_GUIDE_SYSTEM.md) define the
corresponding value and algorithm contracts. Source owners include
`src/theme/value.rs`, `src/theme/paint.rs`, `src/builder/configuration.rs`,
`src/alpha.rs`, `src/alpha/encoding.rs` and `src/decorations/legend/source.rs`.

## Scope And Precedence

Resolve complete appearance first: explicit Plot Theme, otherwise inherited Canvas
Theme when composed, otherwise Publication. Canvas-owned surfaces/prose/headers/
TextPanel/Shared chrome use Canvas; Plot-owned marks/axes/annotations/prose use
Plot. Facets use their owning Plot and Matrix remains a Plot scope.

For a property with these admitted owners, exact element style takes precedence
over the relevant Plot scale, then the selected complete Theme. A Plot scale
replaces only that scale. An exact field edit replaces only its field. Neither
performs general component merging. Instructions remain independent of Theme
attachment order.

Heading instructions use crate-root `TextAlignment::{Start, Center, End}`.
Title alignment resolves Plot instruction > Canvas panel instruction > Plot Theme.
Subtitle resolves its Plot instruction > Canvas panel instruction > optional Plot
Theme subtitle > final effective title. Caption resolves its Text instruction over
the independent Theme caption field. Canvas root prose uses its own Text requests
over the Canvas Theme, separately from embedded-panel defaults.
Canvas panel instructions do not affect Canvas root prose, headers or TextPanel.
Resolve concrete physical Start/Center/End before measurement and freeze it.

Prose content and instructions share one Text slot. Plain replacement retains
earlier style/layout; present exact appearance and whole layout replace their
respective instructions. A present default layout resets the previous request.
Convenience alignment edits update the same request in call order, even before
content is present. Full source/style/request-presence equality owns cache reuse.

Display-text syntax belongs to one shared validator, independently of these
appearance owners. Explicit prose, titles, headers and annotations retain source
whitespace, mandatory breaks and present empty-line metrics. Automatic names use
the selected Labels policy; public `Labels::resolve` exposes the same preparation
without changing raw scientific identities or domain-level collision checks.

TextPanel retains independent Text values for title/body. Each local complete
TextTheme replaces that slot's Canvas role, and TextLayout alignment defaults to
Start independently. TextPanel inset uses the selected body size; title/body gap
uses the maximum selected size. Root padding remains owned by the root body role.
Text/TextPanel equality includes complete exact appearance and layout presence,
including an explicit default reset.

PanelTextAnnotation retains Text with complete appearance/request equality.
Its local TextTheme replaces the complete Plot panel-text role, including the
font metric used by panel_text_padding. The corner continues to place the whole
block; TextLayout alignment chooses physical line alignment inside that block.

## Paint And Geometry

Each final fill, stroke, text, bracket, outline and gradient stop has one RGBA
paint. Opaque mapped RGB receives its owning role alpha once. Exact `Color`
replaces complete RGBA; a separately admitted alpha instruction replaces that
alpha afterward in either setter order. Never multiply alpha or apply an enclosing
SVG group opacity as a second paint route. Independent fill/stroke/outline paints
remain independent. Structural `None` is absence; zero width or transparent paint
must not stand in for an absent component.

Neutral Trajectory/LinePath path RGB comes only from
`MarkTheme.neutral_paints.path`, paired with `data_lines.path_line` width/alpha/
pattern. Hidden-raw-overlay Boxplot flier RGB comes only from `neutral_paints.box_flier`,
paired with primary point appearance and the flier radius. Neither borrows text,
axis, reference or palette paint. A guide's projection-neutral RGB instead comes
from the source Plot's legend-label text RGB, ignoring its text alpha.

Optional distribution observations use `points.raw_overlay`; mandatory Raincloud
sampled/supplied strips use the independent `points.mandatory_strip`. Outline
visibility applies once to each final style. RawOverlay cannot hide the mandatory
strip. Ordinary Line uses the `path` radius, Trajectory/LinePath the `trajectory`
radius, consistently in marks, intrinsics and originating keys.

Intrinsic and automatic-domain ink use actual final paints, including a data
override making a Theme-zero component visible. Alpha zero preserves scientific
domains, statistics, ordering and source membership; painted-ink calculations can
exclude nonpainting ink, while semantic guide components keep structural occupancy.
Point batches coalesce only contiguous identical final styles, preserving every
coincident point and original painting order.

## Exact Elements And Annotations

`GuideStyle` owns complete Color, finite positive CSS stroke width and closed
LinePattern. Its consuming edits are `color(value)`, `width(value)` and
`pattern(value)`, with paired `get_color()`, `get_width()` and `get_pattern()`
readers retaining their const-call capability. Edits validate the complete style;
its geometry remains owned by GuideSpec.

An ordinary Line `PointLayer` has independent data and optional exact color, alpha
and radius. Automatic marks use MarkTheme supplemental radius/alpha and stable
accent slots. The complete point-layer-label text role remains unchanged by
automatic accent/mark alpha; explicit layer Color replaces text RGBA, explicit
alpha replaces text alpha, and radius affects only point geometry. Keys use the
final mark style, not label paint. Every layer consumes its append position even
with an exact paint. An unkeyed primary uses accent slot 0, first layer slot 1;
a categorically mapped primary uses no accent slot, first layer slot 0. Extension
does not cycle.

PointLayer consumes `color(value)`, `alpha(value)` and `radius(value)` edits.
Its const readers `get_color()`, `get_alpha()` and `get_radius()` preserve optional
request presence. Radius validates immediately; color and byte-alpha edits remain
infallible, with alpha editing retaining its const-call capability.

PointLayer labels retain the position policy from `src/plots/line.rs`: add final
radius plus resolved nominal gap to both projected coordinates, apply Rust
`f64::round()` to whole CSS pixels (halfway away from zero), then canonicalize.
Use horizontal Start, vertical Middle, zero rotation and Start-aligned lines.
Required visible labels contribute complete unclipped ink; do not clamp to root
origin or reinterpret the nominal gap as a painted-edge minimum.

`BarPointLayer` owns complete Color and positive radius at construction, uses
prepared bar centers, plain circles/no outline and no layer legend. Its read-only
paint and radius queries remain `color()` and `radius()`. Dumbbell
endpoint labels/Colors are constructor-owned; each exact RGBA supplies its literal
key, while Theme radius/shape/independent outline apply and the connector is separate.

`SignificanceMarkStyle` overrides positive CSS font size, closed weight and finite
nonnegative CSS gap only. Absence uses the complete Theme text role and Theme gap;
explicit Default is exactly 16px/Normal/8px. Stack, color, leading and bracket
geometry remain Theme-owned. Preserve this distinction through fonts, intrinsic
reserves, shelf packing, backend validation and capture. Frozen canonical brackets
retain selected tick height; no backend constant can reconstruct it.

Exact mark-style edits use `font_size(value)`, `weight(value)` and `gap(value)`;
paired readers are `get_font_size()`, `get_weight()` and `get_gap()`. Construction
and consuming edits validate the complete value; size remains positive u32 CSS
pixels and the readers remain const.

Typed significance targets use original category/series/facet identities, never
synthetic concatenated keys. Anchors use final prepared mark centers. The occupied
shelf interval includes span, centered complete label ink, tick and half-stroke
clearance. Fit must contain the complete bracket/text union.

Coefficient numeric labels use the selected estimate-detail role and a
center-relative gap: try `round(x+gap),round(y)` with Start, then `round(x-gap),round(y)`
with End, both vertical Middle and zero rotation. Do not add point radius. Attached
Coefficient significance uses its significance gap, not the numeric gap. Available,
unavailable, numeric and significance sources retain separate membership.

## Data-Alpha Values And Construction

`AlphaValues`, `AlphaEncoding` and closed
`AlphaSemantics::{Undeclared,Continuous,Categorical,Threshold}` are crate-root
values. Fields/tables/targets/preparation stay private. Values compare by complete
normalized content, not allocation identity. No application closure survives
construction into Plot/Canvas, layout, capture or Output.

| AlphaValues constructor | Required meaning |
| --- | --- |
| `new(name, bytes)` | Eager immutable u8 array, Undeclared; no inferred explanation |
| `linear(source, domain, range)` | Explicit continuous linear mapping |
| `log10(source, domain, range)` | Explicit continuous mapping in renderer log10 coordinates |
| `continuous(source, domain, f)` | Declared continuous custom rule, eagerly evaluated |
| `categorical(source, f)` | Complete first-observed raw category declaration/table |
| `threshold(source, boundaries, alphas)` | Complete ordered interval declaration |
| `select(indices)` | Selected occurrences with the unchanged complete declaration; repeated valid indices allowed |

These factories and `select` return Result. Numeric sources are NumericValues,
categorical sources CategoryValues, domains `(f64,f64)`, ranges `(u8,u8)` and
threshold boundaries/bytes finite iterators. Readonly name, len, is_empty, get and
semantics expose content without mutable tables or renderer state.

Names are nonempty, already trimmed and control-free. Empty value arrays are
constructible; attachment preparation checks actual alignment/completeness.
Null/nonfinite mapped sources reject immediately with original source index.
Numeric members retain original finite values separately from clamped evaluation
keys, normalizing signed zero. `select` rejects out-of-range indices.

Continuous domains are explicit finite strictly increasing pairs. Constant source
data is valid; no inferred domain widening occurs. Log10 requires positive source
values/endpoints and finite distinguishable transformed endpoints. Finite values
outside a domain clamp; nonpositive Log values reject rather than clamp.
`src/numeric/mapping.rs::normalized_fraction` is the shared overflow-safe owner
for alpha and continuous colors. Log mapping uses `src/numeric.rs::log10`, not
the mathematical power-display primitive.

Calculate binary64 `a0 + t * (a1-a0)` in that order, then quantize once nearest/
ties-to-even to u8. Ascending, descending, constant and zero-containing ranges
are valid. Quarters on [0,1] to [0,255] yield `[0,64,128,191,255]`; midpoint [0,1]
yields 0 and midpoint [1,2] yields 2. No floating opacity reaches drawing.

Custom continuous construction evaluates each distinct clamped source/required
representative coordinate once, in numeric order. Categorical construction
evaluates each raw category once, in first-observed order. Retain the whole finite
table, including unused evaluated coordinates. Equal current bytes/examples do
not establish rule equality. Pooled construction followed by `select` preserves
the same declaration for multiple Plots.

Continuous representatives are original-domain endpoints, their `f64::midpoint`
and the two recursive quarter midpoints, deduplicated by normalized numeric
identity: at most five examples, not an inverse or inferred smooth ramp.
Thresholds have m finite strictly increasing boundaries and m+1 bytes, with
`<b0`, `[b0,b1)`, ..., `>=last`; equality belongs to the right. Empty boundaries
and one byte mean `all values`. Numeric example/bound labels use the common
whole-set uniqueness engine before fonts; no fit-driven precision/entry dropping.

## Explicit Units And Actual Consumers

`point_alpha`, `line_alpha` and `fill_alpha` each replace one complete target
encoding. Default is Theme; `AlphaEncoding::theme()` resets only that target.
The targets affect point fill, the admitted scientific stroke, or the admitted
area/body/cell fill after normal paint resolution. They do not affect independent
outlines, text, unlisted reference/chrome roles, PointLayer or BarPointLayer.

| Encoding | Binding contract |
| --- | --- |
| Theme | Use the owning complete role, with no mapped input or explanation |
| Observations | Full original input slots, including filtered/unavailable rows; not SourceRows diagnostic IDs |
| Single | Exactly one value, explicitly broadcast to actual units |
| Keyed | Actual series/aggregate selectors, optionally actual facet fields; no implicit row reduction |
| Bins | Exact Histogram lower/upper pairs and optional facet tuples |

`theme()` and `observations(AlphaValues)` are infallible value selections.
`single(AlphaValues)`, `keyed(CategoryValues columns, AlphaValues)` and
`bins(Bounds, CategoryValues facet columns, AlphaValues)` validate construction
and return Result. `is_theme()` and `values()` inspect the private choice;
values is absent only for Theme. Actual scientific binding is checked during
preparation, not guessed by these constructors.

Keyed inputs require a nonempty set of uniquely named nonnull category columns,
lengths aligned with values and unique complete tuples. Canonicalize columns by
raw field name without changing tuple associations. Required fields are actual
series/aggregate keys; additional fields may only be the owner's actual facets.
Same-field group/hue keys collapse only after existing aligned-value validation.
No alias lookup, Unicode normalization, whitespace folding, partial-key matching
or fallback is allowed. Extra unused tuples are valid and create no marks/fonts.

Bin intervals are finite lower<upper pairs, unique by exact normalized endpoints
and facet tuple, with aligned lengths. Actual prepared edges must match exactly;
do not change bin selection or reduce sample alpha into bins. Signed zero has one
key identity. Single provides explicit broadcast for auto bins.

Every actual consumed unit resolves before fonts, including alpha zero. Unknown
fields, wrong binding kind/schema, duplicate tuples, length mismatch or missing
actual key is a Data error. Optional absent components still validate kind/schema
but produce no actual members or explanation. Supplied Raincloud observations
follow original group/point order and actual point count, not synthetic Plot rows.

In the table, R means original observations/records, K keyed actual series or
aggregates, B exact bins, S Single only. Every admitted target also accepts Theme
and Single. A dash means no setter capability; it does not synthesize a component.

| Family | Point alpha | Line alpha | Fill alpha |
| --- | --- | --- | --- |
| Scatter | R retained x/y points | — | — |
| Histogram | — | S optional KDE | B prepared bins |
| Density | — | K group/hue curve | K same curve area |
| Qq | R stable quantile-sorted observations | S optional reference | — |
| PointInterval | R available estimates | R uncertainty/caps | — |
| Coefficient | R available estimates | R present uncertainty | — |
| Forest | R available estimates | R present uncertainty | — |
| Line | R visible primary markers | K group/hue path | K complete configured band |
| Trajectory | R condition markers | K graphical-group path | — |
| LinePath | R visible markers | K group/hue under Hue, group under Neutral | — |
| Interaction | R condition estimates | K group/hue trace connector/uncertainty | — |
| Dumbbell | R paired endpoint fills | R pair connector | — |
| Heatmap | — | — | R eligible finite input cells |
| Bar | — | K present category/series uncertainty | K category/series body |
| HorizontalBar | — | K present category/series uncertainty | K category/series body |
| StackedBar | — | — | K category/series segment |
| StackedHorizontalBar | — | — | K category/series segment |
| Boxplot | R actual fliers/raw overlay | K category/series summary | K category/series box |
| Violin | R configured raw observations | K present inner summary | K category/series violin |
| Raincloud | R mandatory sampled/supplied strip | K inner summary/median | K category/series cloud |
| Regression | R retained observations | S fitted stroke | S configured band |
| Marginal | R central observations | — | K linked marginal densities |
| Matrix | R off-diagonal observations | — | K diagonal densities |
| Beeswarm | R packed observations | — | — |
| Waterfall | — | — | R input delta bodies |

These are exactly 17 PointAlpha, 17 LineAlpha and 15 FillAlpha family attachments.
Any has none. Existing marker/band/reference/availability policies remain in force;
point-only Coefficient data does not gain intervals. Scientific regrouping,
mean/first/last alpha and repeated-row equality as collective input are forbidden.

## Explanation And Retained Identity

Undeclared bytes never acquire an explanation from names, cardinality, numeric
origin or the 256 byte values. Declared Continuous retains all required numeric
examples even when bytes are equal or zero; Categorical/Threshold explains actual
consumed members in declaration order, including zero. Empty actual components
contribute no section. [Guide rules](08_GUIDE_SYSTEM.md) own projection, title,
member order, Shared compatibility and one-frame measurement.

`src/decorations/legend/alpha.rs` prepares one immutable per-scope three-target
input containing full configured encodings, declarations and actual target/role/
unit members, independently of explanatory visibility. Hidden/Undeclared removes
guide text, not this identity or scientific validation. Child, content-height and
feedback keys compare complete normalized values, source slots and diagnostic IDs,
including unused custom evaluation coordinates. Equal output bytes, extents,
hashes or Arc pointers alone are insufficient. Frozen drawing consumes final
per-unit paints and guide/text operands without rereading alpha sources.
