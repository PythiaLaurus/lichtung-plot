# Data Alpha: Explicit Units, Owned Mappings And Automatic Explanation

Source baseline: `f97bd8c`. This closes the source-backed alpha design gate after
the user selected explicit per-observation, per-series and per-aggregate inputs.
It describes the final replacement and implementation handoff; the API is not
exported by this documentation commit.

## 1. Semantic Unit And Paint Component Are Separate

A Scatter point consumes one observation's alpha. A Line path consumes one
explicit series value; a bar/violin body consumes one explicit aggregate value.
A family that displays both observations and collective geometry can configure
them independently. No alpha mean, first/last row, repeated-row agreement test,
or scientific regrouping is introduced.

Three consuming Plot attachments select independent complete instructions:

```rust,ignore
impl<K: kind::PointAlpha> Plot<K> {
    fn point_alpha(self, alpha: AlphaEncoding) -> Self;
}
impl<K: kind::LineAlpha> Plot<K> {
    fn line_alpha(self, alpha: AlphaEncoding) -> Self;
}
impl<K: kind::FillAlpha> Plot<K> {
    fn fill_alpha(self, alpha: AlphaEncoding) -> Self;
}
```

Point alpha replaces point-fill alpha, line alpha replaces the specified
scientific stroke alpha, and fill alpha replaces the specified area/cell/body
fill alpha. Point/fill outlines, unlisted reference/chrome/text roles and independently
owned supplemental layers remain independently owned. These are distinct
components, not aliases or a generic opacity applied to an enclosing SVG group.

Each target stores one concrete `AlphaEncoding`, Default = Theme. Repeated calls
replace that target's complete encoding; `theme()` explicitly resets it. Setter
order relative to Theme, color scales and exact paints does not change resolution.
Resolve the normal color/role/exact paint, then replace only the target alpha
with the selected byte. Never multiply alphas. Zero does not delete an observation,
change statistics, suppress a semantic category, or remove source membership.

## 2. Final Public Values And Construction Boundary

The two input values below are opaque crate-root values with private representation;
the closed semantic classification is public. No application closure is retained.

```rust,ignore
enum AlphaSemantics { Undeclared, Continuous, Categorical, Threshold }

impl AlphaValues { // Clone + Debug + PartialEq + Eq
    fn new(name: impl Into<String>, values: impl IntoIterator<Item = u8>) -> Result<Self>;
    fn linear(source: NumericValues, domain: (f64, f64), range: (u8, u8)) -> Result<Self>;
    fn log10(source: NumericValues, domain: (f64, f64), range: (u8, u8)) -> Result<Self>;
    fn continuous(source: NumericValues, domain: (f64, f64), f: impl FnMut(f64) -> u8) -> Result<Self>;
    fn categorical(source: CategoryValues, f: impl FnMut(&str) -> u8) -> Result<Self>;
    fn threshold(source: NumericValues, boundaries: impl IntoIterator<Item = f64>,
                 alphas: impl IntoIterator<Item = u8>) -> Result<Self>;
    fn select(&self, indices: impl IntoIterator<Item = usize>) -> Result<Self>;
    fn name(&self) -> &str;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get(&self, index: usize) -> Option<u8>;
    fn semantics(&self) -> AlphaSemantics;
}

impl AlphaEncoding { // Clone + Debug + Default + PartialEq + Eq
    fn theme() -> Self;
    fn observations(values: AlphaValues) -> Self;
    fn single(values: AlphaValues) -> Result<Self>;
    fn keyed(keys: impl IntoIterator<Item = CategoryValues>, values: AlphaValues) -> Result<Self>;
    fn bins(bounds: Bounds, facets: impl IntoIterator<Item = CategoryValues>,
            values: AlphaValues) -> Result<Self>;
    fn is_theme(&self) -> bool;
    fn values(&self) -> Option<&AlphaValues>;
}
```

Alpha names are nonempty, already trimmed and control-free. `new` eagerly owns
its iterator's bytes and is Undeclared: it supports arbitrary caller-side,
multi-field calculations without inventing an explanation. Empty arrays remain
valid construction inputs; actual alignment/completeness is checked at attachment
preparation. Missing/null and nonfinite mapped source values reject immediately
with a source index; they are never a convention to drop scientific rows.

`continuous` is an explicit declaration, not inference from `f` or its result.
It materializes every distinct clamped finite source coordinate and its required
representatives in numeric order, invoking `f` once per distinct coordinate.
`categorical` invokes `f` once per distinct raw source category in first-observed
order. All bytes, selected semantic members and complete mapping metadata are
owned before the constructor returns. Numeric members retain the original finite
source values (normalizing zero), separately from any clamped evaluation keys;
threshold members also retain their resolved interval membership. `select` copies selected members/bytes
(allowing repeated indices), retains the complete immutable mapping declaration,
and rejects out-of-range indices. This supports one pooled custom mapping shared
by independently selected plot inputs without retaining or re-invoking a function.

Undeclared arrays never gain an automatic alpha guide from their name, cardinality,
numeric origin or the 256 possible byte values. Callbacks supplied to construction
are absent from Plot/Canvas, preparation, layout, capture and Output replay.

## 3. Numerical Rules

Linear and Log10 require finite strictly ascending domain endpoints. Log10 also
requires positive endpoints and positive source values; transformed `log10`
endpoints must remain finite and distinguishable. Domain validation precedes
materialization. A constant source is valid within a caller-chosen nonconstant
domain; no inferred domain broadening or invented constant-data interval occurs.

Clamp finite out-of-domain source coordinates to the declared endpoints. For
Linear, use the existing finite normalized-fraction arithmetic: exact endpoints,
ordinary `(x-low)/(high-low)` when the span is finite, otherwise its scaled
opposite-extreme form. Move this arithmetic to one private numeric mapping owner
and migrate the current continuous-color callers; remove the old method rather
than retaining an alias. Keep the existing color constant-domain midpoint rule;
alpha's stricter domain validation makes that branch inapplicable to alpha.

Log10 normalizes finite binary64 `log10(x)` coordinates over transformed endpoints.
This paint mapping is not the certified mathematical Forest display conversion;
it neither changes coordinates nor enters the label power engine. Preserve
explicit endpoint values. For either map, calculate the binary64 expression
`a0 + t * (a1-a0)` in that order, round nearest/ties-to-even once to a byte, and
check the finite `0..=255` result. Ascending, descending, constant and zero-containing
byte ranges are valid. No floating opacity reaches a backend.

Independent witnesses include Linear `[0,0.25,0.5,0.75,1]` to `(0,255)` giving
`[0,64,128,191,255]`; midpoint `(0,1)` gives 0 while midpoint `(1,2)` gives 2.
Log10 source `[1,10,100,1000,10000]` on `[1,10000]` gives the same five bytes.
Adjacent, subnormal, opposite-extreme, clipped and invalid domains require focused
coverage, without changing the ordinary color interpolation contract.

Continuous explanations use original-domain endpoints, their `f64::midpoint`,
and the two recursively formed quarter midpoints, deduplicated by normalized
finite numeric identity. These are at most five truthful numeric examples, not
an inverse, a smooth ramp or a claim of equal alpha spacing. Apply the same stored
rule/function evaluation to the representatives as to marks. All-equal output
bytes do not erase the continuous declaration or distinct numeric members.

Threshold boundaries are finite and strictly ascending. There are exactly m+1
alpha bytes for m boundaries, with intervals `< b0`, `[b0,b1)`, ..., `>= blast`;
equality belongs to the interval on the right. Empty boundaries plus one byte
represent one declared interval. Labels are generated from actual bounds, never
independent caller entries. Distinct numeric representatives/bounds use the common
locale-free numeric engine, retrying whole-set scientific fractional precision
2..=17 when default labels coincide. Stored labels and numeric membership precede
font discovery and fit; no layout-dependent precision or entry dropping occurs.

## 4. Explicit Binding Rules

- Theme resets to the owning role; it carries no mapped values or guide.
- Observations align with the original complete input slots, including scientifically
  filtered rows. Use `PlotData::row_index`, not arbitrary public SourceRows IDs,
  to attach a byte before sorting/filtering/grouping. Keep the original slot and
  diagnostic source ID distinct wherever preparation currently discards the slot.
- Single requires exactly one value and explicitly broadcasts that value to the
  target's actual units. This is a caller instruction, not an implicit default or
  a reduction of source rows.
- Keyed requires at least one uniquely named category column, with columns of equal
  length to the alpha values and no null selector values. Each complete tuple is
  unique. Preserve raw identity bytes,
  without alias lookup, Unicode normalization or whitespace folding. Canonicalize
  column order by raw field name; preserve the values' original tuple associations.
  Required fields are the actual series/category keys in the matrix below, with
  duplicate same-field group/hue keys collapsed only under existing source equality
  validation. Additional keys may be chosen from the owner's actual facet fields;
  the fixed schema is matched exactly, with no partial-key or fallback lookup.
- Bin bindings require finite strictly increasing lower/upper pairs, unique exact
  interval keys, aligned alpha and optional facet columns. Numeric zero has one
  identity. Actual prepared Histogram bin edges must match exactly; do not move
  edges or change bin selection to match alpha. Supplied geometry provides directly
  caller-owned bin keys. Auto bins can use an explicit Single instruction or an
  exact complete interval map; no per-sample alpha aggregation is admitted.

Every actual consumed unit must resolve before fonts. Unknown selector fields,
unsupported binding kinds, duplicate tuples, length mismatches and absent unit
keys are Data errors. Extra unused tuples are allowed for pooled/facet mappings
but contribute no mark, guide entry or font requirement. Alpha zero does not
exempt a real unit from completeness. Optional absent components contribute no
alpha guide/text; schema validation still applies. Supplemental PointLayer and
BarPointLayer inputs retain their separate exact-paint ownership.

## 5. Actual Family / Component Census

`R` means original observation/record slots; `K` means explicitly keyed actual
series/category units (or explicit Single); `B` means exact Histogram bin intervals.
Every admitted target also accepts Theme and explicit Single. An em dash is no
attachment capability, not a request to synthesize an absent component.

| Family | Point alpha | Line alpha | Fill alpha | Current producer/source owner |
| --- | --- | --- | --- | --- |
| Scatter | R: retained joint-finite x/y points | — | — | `plots/scatter.rs::PreparedScatter::{new,project}`; currently drops row slots in point tuples |
| Histogram | — | Single: optional KDE stroke | B: each prepared bin | `plots/histogram.rs::HistogramSemantics::{new}` / `project`; separate KDE/rug styles; rugs remain independent |
| Density | — | K: group/hue curve | K: same curve's area | `plots/density.rs::{DensityCurve,PreparedDensity}`; actual CollectivePair, not KDE vertices |
| Qq | R: original observations following stable quantile sorting | Single: optional reference | — | `plots/qq.rs::{PreparedQq,prepare_qq,project}`; retain slots before sorting values |
| PointInterval | R: available estimates | R: available interval strokes/caps | — | `plots/point_interval.rs::prepare`, `plots/interval_estimate.rs::PreparedEstimate`; unavailable slots remain |
| Coefficient | R: available estimates | R: present uncertainty intervals | — | `plots/interval_estimate.rs::{prepare_model,project}`; point-only estimates gain no invented intervals |
| Forest | R: available estimates | R: present uncertainty intervals | — | Same estimate owner; raw values/Log10 projection unchanged |
| Line | R: visible primary markers | K: actual group/hue path | K: complete configured band | `plots/line.rs::{LineGroup,LinePoint,LineMark}`; supplemental layers separate |
| Trajectory | R: retained condition markers | K: graphical group path | — | Same Line owner; path grouping excludes row-varying hue |
| LinePath | R: visible primary markers | K: group/hue under Hue style, group under Neutral | — | Same Line owner; preserve source vertex order |
| Interaction | R: condition estimates | K: actual group/hue trace, connector and uncertainty strokes | — | `plots/interaction.rs::{Trace,Cell}`; configured shape/pattern independent |
| Dumbbell | R: the pair's two endpoint fills | R: pair connector | — | `plots/dumbbell.rs`; endpoint RGB and independent outlines preserved |
| Heatmap | — | — | R: eligible finite input cells | `plots/heatmap.rs`; missing blocks/crosses/row outlines remain independent |
| Bar | — | K: present uncertainty stroke at category/series | K: category/series body | `plots/bar_data.rs::{PreparedBars,PreparedBar,BarShape}` / `plots/bar.rs`; attached points separate |
| HorizontalBar | — | K: same uncertainty unit | K: same body unit | Same Bar owner / `plots/horizontal_bar.rs`; sign coloring independent |
| StackedBar | — | — | K: category/series segment | Same Bar owner / `plots/stacked_bar.rs`; never reduce sample alpha |
| StackedHorizontalBar | — | — | K: category/series segment | Same Bar owner / `plots/stacked_horizontal_bar.rs` |
| Boxplot | R: actual outliers/raw overlay observations | K: category/positioned-series summary strokes | K: same group's box body | `plots/{distribution_sample,boxplot,categorical_distribution}.rs`; retain observation slots beside source IDs |
| Violin | R: configured raw observations | K: present inner-summary stroke | K: category/positioned-series violin body | `plots/violin.rs`, same distribution owner; inner fill/outline remain independent |
| Raincloud | R: mandatory strip observations | K: inner-summary stroke | K: category/positioned-series cloud body | Same sample owner or `plots/violin/precomputed.rs`; supplied points align in original group/point order, not the zero-row synthetic PlotInput |
| Regression | R: retained joint-finite observations | Single: fitted stroke | Single: configured fitted band | `plots/regression.rs`; no alpha derived from regression sample rows |
| Marginal | R: central observations | — | K: group's two marginal densities | `plots/marginal.rs::{HueGroup,PreparedMarginal}`; density outlines independent |
| Matrix | R: off-diagonal observations | — | K: group's diagonal densities | `plots/matrix.rs::{HueGroup,PreparedMatrixAxes}`; native Plot-local owner remains |
| Beeswarm | R: prepared observations | — | — | `plots/beeswarm.rs`; continuous color domain and packing inputs unchanged |
| Waterfall | — | — | R: input delta-step bodies | `plots/waterfall.rs::{PreparedWaterfall,WaterfallSpan}`; base/terminal, connectors and totals independent |

The final sealed sets contain **17 PointAlpha, 17 LineAlpha and 15 FillAlpha**
attachments (49 combinations). Any gains none. Optional markers, bands, overlays,
reference lines and estimates keep their existing visibility/availability rules.
All 25 kinds can now produce an automatic alpha guide, so their public Legend
capability must deliberately extend to all exact kinds at cutover; migrate the
old eight excluded-family negatives and retain Any as negative. This does not
change Matrix Canvas admission, generic facet sets, scientific inputs or layers.

For observations, preserve painting order when different alpha values split an
existing homogeneous scatter call: use contiguous style batches, not alpha-based
regrouping. Keep every coincident point. Resolve group/bin bytes once during
scientific preparation. Selected payloads retain final per-unit styles; do not
read alpha sources in draw. Intrinsic and automatic-ink calculations must consume
the same resolved paint, including a data override making a theme-zero component
visible. Keep structural legend-key occupancy independent of alpha zero.

## 6. Automatic Guides, Projections And Parents

Alpha declarations produce component-specific explanatory symbol sections, under
both existing Combined and Separate group/hue composition. Alpha sections never
infer or join categorical identity from matching bytes or labels. Their identity
contains the source field, declared semantic kind, target component and semantic
unit class. Their automatic title qualifies the resolved source name with target
and unit, distinguishing observation point alpha from series/fill alpha. No
caller-selected unrelated entries, labels, graphics or manual append API is added.

Continuous sections contain the stored numeric examples, even with equal byte
outputs. Categorical/threshold sections contain only actually consumed categories/
intervals, including transparent ones, in their declared order. Metadata retains
the complete mapping table/domain. Point, line or fill symbols come from actual
resolved component geometry and outline presence. Their target alpha is the
declared byte; target RGB uses source legend-label RGB as an explicitly neutral
context (without text alpha), including when mark RGB is fixed. Independent
outlines retain their paint/alpha unless their RGB explains another mapped channel.
Mapped group shape/pattern is neutralized for the alpha-only explanation using
the existing Circle/Solid projection convention; fixed geometry is retained.

Retain every active role glyph and its actual members. The existing one-key
symbol grammar uses deterministic producer-role order to select an actual
component geometry for the numeric/category examples; the complete projected
glyph set still participates in Shared compatibility and descriptor identity.
This is geometry selection, never a representative observation's alpha or a
collective reduction. It preserves paired endpoints, summary/median and highlighted
cell responsibilities without adding a mini-plot or arbitrary guide graphics.

Existing group/hue keys neutralize data-bound alpha to 255, including when the
alpha values have no declared explanation. A hue key cannot choose one row's
alpha as representative of a varying channel. Role/exact alpha without a
data-alpha binding is retained.
Add typed per-paint AlphaBinding and Explained/Neutralized/Constant alpha masks,
alongside existing RGB/style masks. Complete masks participate in equality.
Hidden skips guide-only preparation/fonts, while all actual alpha bindings and
scientific color completeness/domains still validate.

Use one final `PreparedGuide` containing ordered typed sections: existing category/
literal/layer symbols, optional native ContinuousColor body, then alpha Point/Line/
Fill sections. Replace the old exclusive Discrete/Continuous envelope and GuideKind;
do not add a second legend owner, descriptor fallback or independent alpha renderer.
Keep the continuous-color gradient and semantic rules in its body. Necessary
color-plus-alpha composition is admitted; unrelated categorical-RGB plus continuous
color Shared contributors retain their current incompatibility. Existing group/hue
factorization remains unchanged apart from explicitly neutralized data alpha.

The common guide owns one frame/chrome/spacing/collision obstacle, original
structural address, finite stacked allocation and frozen LegendDrawPlan. Extract
continuous body's content measurement from its former outer frame and title;
measure all section headings and bodies under the one envelope. Reuse the existing
exact track allocator, actual ink, normal/compact rules, and 8/2 owner budgets
within each root round. Sections never create placement owners or reset attempts.
The root-local guide cache keys the complete typed descriptor, actual source
bindings, fonts/theme, orientation/compact and canonical constraints.

Plot/facets use their owning mapping; Canvas PerPlot uses each child; Shared unions
visible prepared contributions under Canvas composition/chrome without changing
source mark mappings or source-neutral RGB. Explicit alpha domains/ranges are
already materialized and are never widened/reapplied by parents. Shared alpha
requires equal complete declared recipes (including categorical order/table,
threshold bounds/bytes or custom finite evaluation table), compatible titles and
complete projected glyphs. Different observation/member lists are allowed.
Pooled custom/categorical construction plus `select` provides equal declarations
without pretending that two opaque functions are identical. Equal output bytes,
extents or representative samples alone do not establish rule equivalence.

## 7. Retained Lifecycle, Migration And Verification

Add one immutable three-target alpha input to Plot construction/erase/Clone and
composed retention. Prepared per-scope sources retain actual unit membership,
resolved bytes and guide operands before fonts. Include complete alpha values,
bindings and declarations independently of formats/ranges in child and height
identities and typed feedback observations. Shared consumed dependencies retain
the complete combined descriptor or only actual allocation, as before; do not
scan unrelated siblings. Source declarations are compared by value, with immutable
Arc identity only a permitted equality shortcut, never a substitute for comparison.

Actual migration owners are `builder{,/accessors,/configuration,/facets}.rs`,
`data.rs::PlotData`, `plots/shared_x/prepared.rs`, `layout/interior{,/guide}.rs`,
all producers in the matrix, `decorations/legend{,/source,/identity,/discrete,
/descriptor,/measurement,/continuous}.rs`, `decorations/plan{,/children}.rs`,
`canvas{,/dependencies,/sizing,/tracks}.rs`, facet planning, font requirements and
feedback. Rename/replace the discrete-only prepared envelope throughout those
callers when the combined representation lands; remove exclusive-kind dispatch,
duplicated outer color-frame paths and temporary private staging exceptions.
No public alpha export/setter appears until that complete cutover.

Split implementation into owned mapping materialization, explicit binding/source
alignment, observation consumers, series/stroke consumers, aggregate/native/supplied
consumers, unified guide preparation/projection, common measurement/parent reuse,
public cutover and output closure. Each code leaf uses focused tests and strict
`cargo clippy --locked --offline --all-targets -- -D warnings`; no cargo check.
Every new unit test replaces an existing one in that same leaf, with deletion
review and no banked budget, packaging trick or integration migration. Discovery
must remain at most 671 including 17 ignored. Documentation leaves use source/spec
review and diff checks. Independent numerical/unit, source-ID, exact-paint, typed
declaration, unused-font, Hidden/Shared, equal-extent key, all-family/root and
cached/uncached source-drop SVG/PNG contracts precede completion. Public migration
also runs actual compile-pass/fail, doctests, warning-denied docs and packaging.
Add bounded alpha samples to the existing release measurement entry point;
execute final full/release/time/RSS gates only at integrated closure.
