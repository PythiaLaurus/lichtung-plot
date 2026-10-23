# Guide System

## One Explanatory Owner

A guide is one typed PreparedGuide containing ordered sections whose bodies are
Symbols, ContinuousColor or Alpha. It has one frame/chrome envelope, one font
census traversal, one placement/collision owner, one measurement/cache path and
one frozen LegendDrawPlan. Combined/Separate is not a second renderer. Native
color-plus-alpha is admitted within this envelope; categorical-RGB and continuous-
color Shared contributors remain incompatible. Manual caller-authored entries or
graphics are outside this contract.

Source owners are `src/decorations/legend/discrete.rs`,
`src/decorations/legend/discrete/composition.rs`,
`src/decorations/legend/discrete/projection.rs`,
`src/decorations/legend/descriptor.rs`, `src/decorations/legend/alpha.rs`,
`src/decorations/legend/measurement.rs` and `src/decorations/legend/continuous.rs`.
Appearance/local metrics belong to [Theme](03_THEME_SYSTEM.md), scientific alpha
to [property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md), source/precision to
[numbers](06_NUMERIC_PRESENTATION.md), containing placement/feedback to
[layout](05_LAYOUT_SYSTEM.md).

## Public Policy And Scope

LegendPolicy is automatic, Hidden or explicitly visible at one typed position.
LegendComposition is closed Combined(default)/Separate, attached by consuming
Plot::legend_composition on all 25 exact Legend families or Canvas for Shared.
Both store concrete complete choices. Any has no Legend configuration. Repeated
setters replace their own input, independently of Theme, Labels, visibility,
placement, sizing or setter order.

Inside positions retain both geometric anchor and data-clearance edge.
InsideTopRight and InsideRightTop share a corner but request Top versus Right
clearance respectively; analogous edge-qualified corners remain distinct even
when their current boxes coincide. Explicit anchors do not move to another edge
as fit recovery.

| Owner | Composition / chrome / prepared glyphs |
| --- | --- |
| Standalone Plot | Plot selection/Theme/source keys |
| Generic facets | Owning Plot selection/Theme, parent-ordered symbol union; no new Theme scope |
| Canvas PerPlot | Each Plot selection and complete resolved Theme |
| Canvas Shared | Canvas selection/chrome/text, each source Plot's complete glyphs/neutral RGB |
| Canvas Hidden / Hidden Plot contribution | No explanatory sections/projections/aliases/font work |

Shared ignores stored child composition without mutation or conflict; switching
back to PerPlot restores its meaning. Explicit child legend position rejects under
Shared because it cannot be honored. Hidden still validates scientific bindings,
styles, exact color coverage and domains, and visible marks still coordinate raw
color identities. An unsupported Separate explanation may be suppressed by Hidden;
invalid scientific input may not.

Heatmap/Beeswarm own measured native right guide tracks; Matrix its native trailing
grid track. Transporting their descriptor for identity, census or Shared union
must not create another outer panel guide. Hidden suppresses native placement.
Matrix remains standalone native, Marginal nongeneric-facetable; no parent
capability is inferred from the common guide representation.

## Typed Identities And Provenance

Raw field/category/layer identities retain their validated source bytes separately
from aliases. Section identity owns title compatibility; entry identity owns
deduplication. Symbols retain typed category(field,value), observed
pair(group-field/value,hue-field/value), Bar sign, Dumbbell endpoint, Regression
observation/fit or supplemental-layer-label identity. Never encode a pair/domain
by joining strings. Alpha member identity is declaration ordinal, not alpha byte
or displayed label.

Section identities distinguish Combined(primary title), GroupStyle(field),
Joint(field), HueColor(field), Literal(domain), SupplementalSeries,
ContinuousColor(field) and Alpha(field,semantics,target,unit). Primary titles are
Field, ordered Pair, Series or BarSign. Literal domains remain BarSign,
DumbbellEndpoints and RegressionReference. Distinct domain types cannot alias
because their current title happens to be series.

Each nonempty symbol entry retains resolved label, complete key and all actual
source occurrence records: producer, structural contributor address, source ordinal,
mapping, projection and selected neutral RGB. Scientific observed-pair domains may
remain for marks, but no parallel flat semantic guide is retained. Authoritative
prepared None is absence, not permission to probe a body or reconstruct late keys.

The closed key grammar has at most one fill handle, one line and one point,
painted in that order. It does not draw error bars/raw clouds/references/mini-plots
merely because marks have those components. Retain complete point shape/radius/
paint/outline, line width/pattern/paint and fill/outline with structural absence.
All actual painted extents come from this same materializer in measure and draw.

Source bindings distinguish mapped raw RGB from role RGB and exact-element RGB,
and mapped Interaction group shape/pattern from constant role geometry. Each
paint independently retains Role/ExactElement/Data alpha binding. Explanation
masks record Constant/Explained/Neutralized responsibilities for RGB, style and
alpha. Equal channel values do not infer a binding. Exact projected-key equality
includes masks and all geometry/paints; provenance is retained for source/cache
validation but not a hidden family discriminator for otherwise compatible keys.

All cardinalities/ordinals are checked integers, never floating estimates or
saturating counters. Entry/key-component counts differ from scientific mark
counts and actual retained source occurrences. Projection/deduplication does not
fabricate group-by-hue products or erase occurrence identity. Real lengths,
capacity conversions and fallible storage apply; counts are not resource prices.

## Symbol Production And Composition

Combined retains one primary **symbol** section with complete observed keys,
same-field collapse and compatible primary title; continuous/alpha sections may
follow. Separate explains actual independent mappings in class order:
GroupStyle,Joint,HueColor,Literal,SupplementalSeries. Within each class use first
contribution/source-domain order, not field sort, title sort or hash order.

| Family / actual contribution | Complete symbol key | Separate behavior |
| --- | --- | --- |
| Scatter with observed finite hue | Primary point/scatter radius/actual outline | HueColor |
| Line with hue, plus labeled PointLayers | Series line, optional visible markers, configured complete band iff every retained group point has bounds | HueColor, same-field Joint, or unsupported different group/hue |
| Trajectory row hue | Mandatory hue point only; neutral connector absent from key | HueColor, no fabricated group channel |
| Neutral LinePath row hue with visible markers | Hue point only | HueColor; hidden markers contribute no primary key |
| Hue LinePath | Path line plus visible point, no band | HueColor/Joint or unsupported independent pair |
| Density hue | Density line+area, independent outline | HueColor/Joint or unsupported independent pair |
| Bar/HorizontalBar positioned series | Actual bar fill/outline, no uncertainty/point | HueColor; HorizontalBar sign mode uses observed literal signs |
| StackedBar/StackedHorizontalBar | Actual series bar fill/outline | HueColor |
| Boxplot series | Summary line+box fill, no raw/flier point | HueColor |
| Violin series | Violin body only, no optional raw/inner-summary component | HueColor |
| Raincloud sampled/supplied hue | Mandatory-strip point+violin fill, actual outline policy | HueColor |
| Interaction hue or explicitly configured styles | Visible point+connector with actual group shape/pattern; interval is not extra key line | Independent mapped style+hue -> GroupStyle+HueColor; group-only/same-field -> Joint; unsupported pair rejects |
| Dumbbell endpoint metadata | Two exact endpoint points, left/right order, no connector | Literal endpoints |
| Regression explicit Visible only | Observation point then fit line+configured band | Literal observation/fit |
| Marginal hue | Central point+marginal density fill/outline | HueColor/Joint or unsupported pair |
| Matrix hue | Off-diagonal point+diagonal density fill/outline | Same semantics under native Plot-local owner |

Histogram,Qq,PointInterval,Coefficient,Forest,Waterfall have no category/reference
RGB key merely from their marks; Heatmap/Beeswarm instead have continuous color.
Declared alpha is independent of those symbol contribution conditions and can
explain every exact family through its admitted actual components.

Only an **explicitly configured** InteractionStyle table is a mapped group-style
channel, even when configured styles repeat default Circle/Solid. A Theme common
shape/pattern or plain graphical group is not one. Group-only unstyled Interaction,
Line,Density,Marginal,Matrix create no group keys. Interaction connector key
presence follows configured trace grammar, not local segment count; a sparse
one-cell trace does not remove a configured connector explanation or invent a
mark segment. A zero-width scientific Line band with complete bounds still
retains its configured band component. Optional Boxplot/Violin observations never
create point keys; Raincloud uses mandatory-strip appearance once, not primary
point or one symbol per row.

### Truthful Factorization

GroupStyle projection changes only hue-mapped RGB to the source Plot's complete
legend-label role RGB, ignoring text alpha. Preserve role/exact alpha, fixed paint,
presence, outline and geometry. Data-bound context alpha is independently set255
first. A mapped outline receives neutral RGB with its own alpha; a fixed outline
does not change even if it equals mapped RGB. Do not use Mark neutral roles,
axis/annotation foreground or Canvas text RGB as this authority.

HueColor keeps RGB and constant geometry. Only mapped InteractionGroup slots
neutralize shape/pattern to Circle/Solid. Theme/fixed shapes, radius, widths,
independent outlines and role/exact alpha remain actual. Measure the exact
projected glyph, whose ink can differ from its source shape.

All occurrences of one raw group/hue must project to the same corresponding
key/label/mask; mismatch rejects before fonts. Unsupported independent pairs reject
even with one pair or equal RGB/style. No representative-pair fallback or
perceptual-injectivity requirement. Same raw field names collapse only after
aligned-value equality; distinct fields stay distinct despite equal labels.
RowHue is point color, not unsupported pair or group-colored connector.

Combined retains sparse observed associations. Separate shows marginal mappings
and does not assert all combinations occurred: three observed pairs can become
two group plus two hue keys, not necessarily a smaller guide. Neither changes
marks/statistics/domains/topology.

## Titles, Labels And Parent Union

Resolve inferred field/value text through effective Labels separately from raw
identity. Combined ordered pair title joins with ` × `, label with `, `; same-field
collapses. Distinct fields always show both, even equal display values. Separate
field title uses its resolved name. Literal titles are Change for sign and series
for endpoints/Regression/supplemental-only. Signs are Nondecreasing/Decreasing
only when observed; endpoint labels and Regression literals retain their owners.

Within a section, same entry identity merges only with exact label/full-key
equality; distinct identities with equal display labels reject. Across sections
equal labels are valid if titles differ. Distinct section identities with equal
**nonempty** title reject. Explicit empty titles and entry labels retain their
empty-line metrics and normal spacing, with no painted glyphs; entry key/identity
remain present. All guide display text uses the shared Unicode control and
mandatory-break validator, preserving aliases verbatim. Strict text fit remains.
Labels aliases are global raw-string mappings, not field-scoped entries.

Supplemental identity is exact supplied layer label, never append index or hue
value. Source Categorical context resolves its label through Labels; LiteralSeries
context preserves supplied bytes. Determine context before contribution filtering,
independently of final Canvas title/composition/first pair. Equal layer identity
with incompatible label/point style rejects. Combined appends layers after primary
keys under primary title, or Series if no primary exists. Separate appends a
SupplementalSeries section. Layers do not add hues to coordinated color domains;
unlabeled layers have no key. Their exact override still consumes accent position.

Parent color-domain coordination uses actual raw color fields, not explanation
presence or hue attachment spelling. Interaction without hue can color by group;
positioned series keep their own raw color field. Hidden-entry visible marks
still coordinate palette indices but cannot leak absent keys/font requirements
through another panel's global color list. Filter actual member contributions,
then order by coordinated domain. Facets prepare parent category domains before
slicing and order by parent ordinal; native color guides retain actual scoped
owners/domains for census. Canvas traverses declared row-major slots, then source
order. Literal order is signs nondecreasing/decreasing, endpoints left/right,
Regression observation/fit, layers append, then first-contribution stable union.

Combined Shared requires one compatible typed primary title+resolved title.
Supplemental-only panels do not choose it: attach their retained typed layers
after all primary contributors are known. Separate unions typed sections with
exact title and repeated-entry label/full projected key equality. Cross-family
compatible keys can merge; equal RGB alone is insufficient. Group neutral RGB
still comes from each Plot and must agree, while chrome/text comes from Canvas.

## Continuous Color And Alpha Sections

Continuous contributors retain raw field, scale kind, resolved title, full semantic
gradient, diverging center, missing block, exact/automatic domain provenance and
effective numeric policy. Shared requires exact compatibility of all except
automatic envelopes: union finite automatic lo/hi; if any exact domain exists,
all exact domains agree and contain every automatic envelope. Generate one final
label set and install final-domain mapping before contrast/fonts. Never merge
already formatted child labels or projected local stops, choose first contributor's
gradient/format, or treat this preparation as feedback.

A local placement-suppression flag does not mean retained source/presentation is
absent. Heatmap/Beeswarm source descriptors reflect actual retained numeric
presentation, returning optional descriptor when genuinely absent; Shared retains
its prepared source when suppressing local placement. Descriptor transport never
creates a second native track. Missing preparation is an invariant error, not
a body-probe/fallback request.

Alpha sections follow all Symbols, then ContinuousColor, then Point/Line/Fill
classes. Within class use first source contribution. Identity is raw source,
declared semantics, target and actual unit. Title is
`{resolved source} ({unit} {target} alpha)` with unit
observation/series/aggregate/bin/single. Member ordinals preserve declaration
identity, including equal/zero bytes. Complete recipes come from the property
contract, not inference from observed alphas.

Retain every active actual role glyph/unit member. Canonical role order is
Primary,Outlier,RawOverlay,MandatoryStrip,LeftEndpoint,RightEndpoint,Line,Summary,
Median,Fill,Highlight. This selects actual geometry for one-key examples, never
a representative row alpha. Complete projected glyph set still controls Shared
compatibility and cache identity. Explained-target RGB uses source legend-label
RGB without its alpha, mapped group shape/pattern becomes Circle/Solid, fixed
geometry/independent outlines remain actual. Existing context keys neutralize
every data-bound target alpha to 255, including Undeclared inputs; role/exact alpha
without data binding remains unchanged.

Continuous alpha retains every stored numeric example if an actual component
consumes the mapping, even when output bytes coincide. Categorical/Threshold
retains only consumed declaration members in declaration order, including0.
Shared requires equal **complete declaration** including unused custom coordinates/
category order/table/threshold bounds, compatible titles and projected glyphs,
then unions actual members. Equal bytes, examples or extents are insufficient.
Reject alias/member-label conflict before fonts. Undeclared/Hidden skips only
explanatory work; complete prepared scientific three-target inputs, role glyphs
and unit membership remain in child/height/feedback identity.

## One Measured Frame And Bounded Flow

Geometry uses exact rational local lengths and canonical i64 quanta. Hard sums
round upward once, positions/preferences nearest-even, ink outward. One outer
horizontal padding applies per edge, vertical padding+glyph_edge_padding per edge,
with independent frame stroke/radius ink. No per-section comfort duplication.
Frame absence removes chrome, not content/padding/structural key occupancy.

Assigned inner width W subtracts complete hard horizontal insets; optional inner
height H subtracts one outer vertical reserve. Negative/unusable space misses.
All sections stack vertically in both orientations and get full W. Exact section
gap=title_gap+row_gap under their separate role metrics. Titles measure at W and
empty title activates no gap. No per-section position/variant/search.

Key measurement visits the same complete frozen-call grammar as drawing. Fill
handle half-height is0.32 label-em, handle width is guide-local; actual stroke
caps/outline/point shape/radius add their full ink. Key center aligns first text
baseline. Label/key ascent/descent can have different maxima. Empty label keeps
key height. P[i] is complete preferred key+gap+label width, M[i] hard unbroken
requirement. The hard key slot includes actual overhang, not nominal handle alone.

### Normal And Compact Widths

Horizontal normal entry E[i]=min(P[i],W); compact min(P[i],C2). Vertical section
normal column C=min(max P[i],W); compact min(max P[i],C2), where
`C2=floor((W-column_gap)/2)` in quanta. Nonpositive capacity or width<M misses.
Measure each label at assigned entry/column width minus exact key+gap. Do not
shrink allocated width because measured ink is narrower. Compact is one global
variant; identical allocations reuse the result, changed widths remeasure labels.

Horizontal flow greedily appends entries in order while width+gap fits, then a
new row. Row height is max(ascent)+max(descent), not max(individual height).
Stack actual sections/title bands/gaps and test full H; no arbitrary partition.
Vertical automatic height uses one column per section and derives total height,
without inventing a finite height to force more columns.

### Finite Vertical Capacity

After measuring title and entries at fixed trial widths, form exact requirements:

```text
T[s] = occupied_title_height + active_exact_title_gap
hard[s] = ceil(T[s]+max(entry_heights))
preferred[s] = ceil(T[s]+sum(entry_heights)+(entry_count-1)*exact_row_gap)
A = H-ceil((section_count-1)*exact_section_gap)
```

Counts subtract only for nonempty sections. Preferred here is hard one-column
occupancy, not a cosmetic estimate. If A<sum(hard), miss. At A>=sum(preferred),
assign preferred and leave slack. Otherwise use the existing one proportional
hard-to-preferred shrink, exact integer remainders/ties by section ordinal.
Subtract each actual title and greedily pack whole entries top-to-bottom into
equal-width columns at its capacity, then check total column widths/gaps<=W.
No split entry/repeated continuation title, redistribution of unused sibling
capacity, allocation retry, permutations or width scan. Stack actual occupied
heights, not assigned capacities. Another theoretical distribution may fit while
this deterministic policy misses.

Each capacity>=ceil(T+max entry height), so one entry always fits below its title;
greedy flow consumes next or fails width/geometry, never loops. Actual heights
plus complete section gaps fit H, with final outward ink checked separately.
For independent integer example hard[2,2,2],preferred[5,5,5],A10 gives[3,3,4].
Baseline entries(18,2),(5,15) need33, not20. These mathematical relationships do
not freeze a scene's automatic dimensions.

### Continuous Bodies Under The Same Envelope

Orientation is horizontal Top/Bottom, vertical Left/Right. The section owns title;
color body measures gradient/labels/optional missing row without another frame.
Labels are final lo/interior-center/hi or one constant; use actual numeric-domain
fractions, not gradient-stop indices or assumed center0.5. Projected outward label
extents/required adjacent gaps determine hard gradient length, combined with
Theme minimum. Endpoint overhang is included outside gradient span, never clipped.

For finite assignment derive maximum from available width/height after measured
title/gaps/missing row/endpoint overhang. If below hard R, miss; normal length
max(R,min(preferred,maximum)), compact R. Natural unbounded measurement uses
max(R,preferred). Thickness is key_width. Missing row uses exact missing-fill/
outline, measured label and key gap, with one row separation; no fixed family
cap, detached swatch or origin clamp. Freeze fractions/stops/labels/row placements
and recheck actual adjacent ink clearance at final length.

Natural guide width is max(section title and complete entry preferences)+one
chrome reserve. Initial automatic Right contributes one-column preferred height
and width+outside gap once. Subsequent edge/compact choices use the initially
chosen root width. Placement tries and provisional stopping use layout's8/2 owner
budgets per round; sections add none. A successful trial contains complete body/
axes/annotations too. The whole selected guide is one inside collision obstacle.

## Preparation, Dependencies And Frozen Output

Resolve owner/Theme/policies/scientific inputs; coordinate raw color fields even
for Hidden-entry marks; prepare complete producer styles/bindings; project/merge/
order typed sections and validate final labels/titles; census final visible strings
under owning guide typography in section preorder; resolve fonts; derive intrinsics/
assigned measurements and freeze. No inferred raw-group/layer string census runs
beside authoritative descriptor traversal. Exact source styles serve marks/keys;
compatibility preparation requires no panel probe or font measurement.

Stable LegendSection ordinals descend from the original local/facet/CanvasShared/
native Matrix guide slot, with title/entry/key/label descendants. Flow columns,
section capacities and routed references are inline payload, not new semantic
owners/calls/addresses. Native facet color census visits actual scoped guides.
No edge-dependent reparenting or global entry index replaces source identity.

Scoped inputs distinguish effective composition, Local/Hidden/SharedSuppressed,
authoritatively prepared contribution, actual local slot and explicit ordered
consumed ancestor dependencies. A parent measuring Shared consumes descriptor+
allocation/orientation/compact/relevant feedback; a suppressed child can consume
allocation-only residual geometry without another Plot's neutral key RGB.
Enumerate actual uses at call sites, not by scanning every root sibling.

Complete descriptor equality retains ordered sections/boundaries/raw identities,
titles/labels/full keys/masks/source provenance, continuous recipe/domain/format,
alpha declaration/actual members and source neutral RGB. Child/height keys also
retain full Theme/used fonts, constraints/origin, scope, parent domains/insets/
visibility and relevant feedback. Equal text/extent/palette/hash/pointer is not
sufficient. Immutable pointer sharing is only an equality shortcut. Relevant
ancestor owners outside a subtree must be stable for reuse; unrelated siblings
are not dependencies. Complete alpha scientific identity remains independent of
explanatory descriptor consumption.

Content-height consequences replace previous height including contraction, not
cumulative owned ink or root-width reselection. Feedback observes selected
sections, actual flow/capacities/text/glyph consequences, not outer extent alone;
replay preserves guide-trial events under the same nonresetting owner budget.
Retained records/strings/glyphs/keys use checked lengths/fallible first-party
storage, with no accounting fees or quota tickets.

Freeze all section translations/boxes, text lines/glyphs, key operands, continuous
gradient/missing rows, optional frame, chosen edge/anchor and complete collision
bounds into LegendDrawPlan. Backends and later Output conversions consume it;
they do not consult Plot/Theme/provenance, factor mappings, regenerate labels,
read sources, discover fonts or rebuild layout. Preserve-text uses retained line
structure; outlined SVG/PNG use retained glyph geometry from the same operation.
