# T6A2 — Authoritative Foundation Construction Design

**Status**: Approved construction target; both user product decisions closed.
Private foundational values and consumer resolution are complete. Public exact
styles and complete Theme/font/scale exposure are complete. Spec93 tracks the
remaining narrow instruction and presentation cutovers.

## 1. Authority, Scope And Decisions

This is the construction, validation, inspection and owner contract for T6's
approved foundations. It derives from the committed T6A1 census
[`docs/t6a1-public-cutover-census.md`](../docs/t6a1-public-cutover-census.md)
(`6ef761f`), [spec03](03_THEME_SYSTEM.md), [spec05](05_LAYOUT_SYSTEM.md),
[spec93](93_T6_EXECUTION_PLAN.md), and the active adjudication overlay in
[spec04](04_VISUAL_PROPERTY_OWNERSHIP.md). It specifies the replacement API;
implementation and current exposure are recorded by spec93 and the current
architecture/rendering contracts rather than inferred from signature sketches.

Source inspection includes `src/theme/{resolved,scale,color,contrast}.rs`,
`src/layers.rs`, `src/spec{,/estimate,/categorical}.rs`, `src/font/canonical.rs`,
the builder's configuration/accessor owners, axis/title/legend measurement,
Canvas/facet tracks and TextPanel consumers, significance, and the ordinary and
specialized mark producers. Source identifiers take precedence over historical
ledger line numbers. Historical ledger counts are not counts of public fields.

The admitted scope is complete immutable themes, existing component roles,
colors/fonts/scales and typed exact styles, plus the already approved numeric
presentation and section-composition policies. The helper values below construct
those roles; none is an independently inherited component or a plot-family theme.
All fields of these values are private. Backend styles, layout trees and generic
property maps are not public construction inputs.

The following decisions close T6A1's construction questions:

| Decision | Authoritative result | Owning leaf |
| --- | --- | --- |
| Shared Canvas gap | **Remove `Canvas::shared_legend_gap` and its field. Configure only `LegendTheme::shared_gap`, a local dimensionless relationship.** Remove the override/policy compatibility check and migrate callers. This explicit user decision supersedes both the remove-override and retain-override statements around historical ledger lines 1838–1840 and 3408. | B4; private theme path B1/B2 |
| Getter/edit vocabulary | Validated-value getters use the field name; consuming edits use `with_field`. No public `_spec` inspection vocabulary and no getter/mutator name collisions. | B1 representation; B3/B4 exposure |
| Complete construction | Every role group has a fully typed constructor. ThemeBuilder starts complete and edits only construction state. | B1/B2; public B4 |
| Font profile access | Obtain a typed stack through a built-in Theme's typography role. Retire both `DEFAULT_FONT_FAMILY` and `DEFAULT_FONT_FAMILY_STACK`; no replacement raw or typed global constant. | B4; sentinel removal B1/B2 |
| Continuous selection/domain | One `ContinuousColorScale` owns kind, center, optional gradient replacement and automatic/exact domain on both Heatmap and Beeswarm. Delete both `.color_limits` routes, Heatmap color `.scale`, `HeatmapScale` and old gradient fields. | B2 consumers; B4 public migration |
| Categorical gate | Rename the sealed capability to `kind::CategoricalScale`; expose `categorical_color_scale`. Preserve its actual fifteen families. | B4 |
| Interaction selector | `InteractionStyle::new` becomes fallible and validates lexical raw identity. Shape/pattern remain its only visual edits. | B3 |
| Alignment reachability | `TitleAlignment` and fully consumed `CompositionTheme` construction/accessors are public at B4. The four Plot/Canvas instruction setters are B5. | B4/B5 |
| PointLayer label paint | Automatic text uses the complete `point_layer_label` role. An explicitly supplied element color replaces text RGBA; an explicitly supplied alpha replaces text alpha only. Automatic accent paint does not override text. | B2 resolver; B3 public exact inputs |
| Gradient shape | Opaque validated ordered stops; general finite sequences, with exactly one diverging neutral stop at semantic offset 0.5. Current three-stop assumptions migrate at B2. | B1/B2; B4 public |
| Unified built-in sequential gradient | **All four built-ins use the Heatmap sequence `#F7FBFF` at 0 to `#08306B` at 1, shared by Heatmap and Beeswarm.** Retire Beeswarm's `#3182BD` to `#DE2D26` default. Callers can replace the sequence through the typed Plot scale or Theme ColorScales value. | Explicit user decision; B2 construction/consumers, B4 public migration |
| Distribution point ownership | Optional raw observations and the mandatory Raincloud strip have independent complete point/radius roles, even though current private storage shares them. Both built-in radii remain 2. | B1/B2, public B4 |
| Remaining admitted chrome | Legend glyph-edge comfort and composed root/embedded-prose padding have typed theme owners with explicit ledger/consumer traces in §7. | B1/B2, public B4 |

The shared sequential-default product decision is closed. Section 13 records its
approved cutover and intentional Beeswarm default-color change. Construction
decisions below are pinned; implementation/public acceptance evidence remains
with the owning leaves.

`Result<T>` in signatures means the crate's existing `Result<T>`/`Error`. Code
blocks are normative signature sketches, not source files or executable examples.

## 2. Uniform Value And Validation Contract

### Inspection And Consuming Edits

For the component/role constructors in §§4–8 and ColorScales, every parameter is
one retained field unless an explicit derived relationship is stated. Its getter
and edit signatures are generated by this **normative exhaustive rule**, including
nested role groups. Other foundational/scale/policy and exact-style values have
the exhaustive method lists explicitly written in their sections.

| Retained field `f: T` | Getter | Construction edit |
| --- | --- | --- |
| Copy scalar/closed value (`u8`, `u32`, `f32`, `f64`, `bool`, `Color`, `OpaqueColor`, closed discriminant) | `f(&self) -> T` | `with_f(self, value: T) -> Result<Self>` |
| Owned validated value/group (`TextTheme`, `FontStack`, `RelativeSpacing`, a palette/gradient/component/role group) | `f(&self) -> &T` | `with_f(self, value: T) -> Result<Self>` |
| `Option<T>` with Copy `T` | `f(&self) -> Option<T>` | `with_f(self, value: Option<T>) -> Result<Self>` |
| `Option<T>` with owned `T` | `f(&self) -> Option<&T>` | `with_f(self, value: Option<T>) -> Result<Self>` |
| Retained string | `f(&self) -> &str` | Only the explicitly listed validated string edit, if present |
| Retained ordered collection | Borrowed slice, or the explicitly listed borrowed iterator | Whole validated reconstruction/replacement; no mutable access |

These rules fully specify the accessors and consuming edits of every field in the
component signatures/tables. Edits validate the resulting complete value, replace
only that field, and return no partially valid value. Cross-field failures, such
as a preferred gradient length smaller than its minimum, reject immediately.
Repeated edits replace the same field. Editing a cloned component cannot mutate
its source Theme. No unchecked/public struct-literal construction is possible.

Explicit exceptions are listed for foundational factories, ThemeBuilder,
plot/policy attachments, and the retained data/content methods of exact styles.
`_spec` may remain a private consuming-helper suffix, never a second public API.
Replaced project-owned methods and repository callers migrate in the owning
cutover; old spellings are not retained as aliases.

All foundation/component values implement `Clone + Debug + PartialEq`.
Colors, font values, palettes/maps, normalized gradients/scales and exact rational
spacing additionally implement `Eq` when their normalized representation permits
it. Copy applies to colors and closed discriminants, not complete themes or
arbitrary-size relative ratios. Runtime font environments, caches and composition
context are excluded from appearance equality and from public Theme storage.

### Units, Rounding And Timing

* Font sizes are positive `u32` **actual CSS pixels**, as used by current layout
  and SVG. Historical comments saying points are corrected; there is no `4/3`,
  DPI or points conversion. Integral sizes/radii/tick lengths normalize by checked
  multiplication by 1024.
* Present CSS stroke widths are finite positive `f32`. Their exact binary32
  rational value is used for normalization; promoting to binary64 is exact.
  Width zero is not an absence sentinel. An absent outline/grid/frame is `None`.
* Role-local spacings are dimensionless `RelativeSpacing`. Zero is valid unless
  a particular field requires a positive extent. Local metrics, including exact
  mark/key/stroke ink, determine requirements; root width/height never scales
  cosmetic spacing. Statistical/data geometry retains its separate owner.
* `TextTheme::line_height` is the existing finite positive binary64 multiplier,
  not a CSS length or a rational-spacing setter. Multiply its exact represented
  rational by integral role font-size quanta and round hard line occupancy upward.
* Construction validates data-independent invariants without system-font
  discovery, rendering or resolving a data domain. Preflight after owner/scale
  selection validates capacity, keys, used fonts, domains, shared compatibility,
  fit and representability before backend begin. Custom contrast is caller-owned;
  opaque automatic light/dark candidates and backdrop are structural requirements.
* Negative floating zero normalizes to positive zero for nonnegative quantities,
  gradient offsets, centers and domains. Nonfinite values reject. Identity uses
  normalized exact inputs, never display strings, approximate getter values or
  pointer/cache identity.

Relative resolution uses spec05's `1/1024` CSS-pixel domain. For a relationship
`r` and declared local metric `M`, compute the exact rational `r * M` before
rounding. The metric/rounding columns below are normative:

* **S**: spacing/preference/position operands round nearest-even once.
* **H**: hard minimum, reserve or occupancy rounds upward once.
* A field marked **S/H** has one exact product used in those two spatial roles;
  this is not two stored theme values. Hard sums use the complete exact
  requirement, including spacing, before upward rounding, rather than summing
  already-rounded estimates. Frozen positions use the selected canonical operands.
* Ink/stroke lower and upper edges round outward, as in spec05; this remains
  separate from nearest-even centers/widths. All sums/products/conversions are
  checked. Nonzero painted geometry that collapses to no representable geometry
  refuses at preflight, not by imposing a new public minimum or clamping a value.

### Exact Relative Spacing

```rust,ignore
impl RelativeSpacing {
    fn new(value: f64) -> Result<Self>;
    fn ratio(numerator: u32, denominator: u32) -> Result<Self>;
    fn as_f64(&self) -> f64;
}
```

`new` requires finite `value >= 0` and decodes the exact binary64 dyadic rational.
`ratio` requires `denominator > 0`; numerator zero is allowed. Both construct
**one reduced nonnegative rational representation**, with zero canonically
`0/1`. There is no remembered constructor mode or floating/rational fallback.
The existing pinned integer dependency can represent the finite, bounded
binary64 numerator/denominator sizes; checked fallible storage is required.
`as_f64` is nearest-even inspection, never the spatial spacing-resolution or
identity operand. The explicitly named nonspatial scientific-geometry consumers
in §6 retain their existing binary64 arithmetic; this exception does not allow
cosmetic gaps to round-trip through the getter. No public arbitrary-integer
rational or metric-resolution API is introduced.

For example `ratio(6,14)` preserves exactly 6px against a 14px role, including
upward hard rounding. `new(6.0/14.0)` means the different exact binary64 value
of that division and must not be silently reinterpreted as `3/7`. Integral built-in
pixel gaps use exact length/metric ratios. Nonintegral built-in pixel literals
retain the exact represented source length divided exactly by their original local
metric, using the same reduced rational representation (§7). Built-in binary64
geometry fractions whose arithmetic is preserved instead use `new` with the
original literal bits.
Spacing ratios may exceed one; this is not a unit-fraction type. Individual
fractions/breadths below add their own range restrictions.

## 3. Colors And Fonts

### Complete Paint And Opaque Data Identity

```rust,ignore
impl OpaqueColor {
    const fn rgb(red: u8, green: u8, blue: u8) -> Self;
    fn hex(value: &str) -> Result<Self>;
    const fn red(&self) -> u8;
    const fn green(&self) -> u8;
    const fn blue(&self) -> u8;
}
impl Color {
    const fn rgb(red: u8, green: u8, blue: u8) -> Self;
    const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self;
    fn hex(value: &str) -> Result<Self>;
    const fn red(&self) -> u8;
    const fn green(&self) -> u8;
    const fn blue(&self) -> u8;
    const fn alpha(&self) -> u8;
    const fn with_alpha(self, alpha: u8) -> Self;
}
impl From<OpaqueColor> for Color { /* alpha = 255 */ }
```

Channels are private and `Copy + Eq`. Hex parsing accepts ASCII case-insensitive
hex digits in exactly `#RRGGBB` for OpaqueColor and exactly `#RRGGBB` or
`#RRGGBBAA` for Color. No trimming, shorthand, CSS expressions, named colors or
floating channels. Alpha is inclusive `0..=255`. There is no conversion that
silently discards a Color's alpha to admit it to a data scale.

Mapped colors are always opaque. A mark role installs its own alpha once. An
exact Color replaces all RGBA channels. An independent exact alpha, where the
style actually exposes one, replaces the corresponding alpha after resolution,
regardless of setter order. It never multiplies a theme or exact alpha, never
changes an independent outline, and is never re-quantized through floating opacity.

### Typed Font Preferences

```rust,ignore
enum GenericFontFamily { Serif, SansSerif, Monospace }

impl FontFamily {
    fn named(name: impl Into<String>) -> Result<Self>;
    fn generic(family: GenericFontFamily) -> Self;
    fn name(&self) -> Option<&str>;
    fn generic_family(&self) -> Option<GenericFontFamily>;
}
impl FontStack {
    fn new(families: impl IntoIterator<Item = FontFamily>) -> Result<Self>;
    fn families(&self) -> &[FontFamily];
}
```

FontFamily's named/generic representation is private. A named family is a
nonempty, already trimmed, control-free literal family identity. It is one name,
not parsed CSS: punctuation belongs to the name and preserve-text serialization
quotes/escapes it as necessary. It cannot smuggle in another stack entry. Even a
name spelled like a CSS keyword is a quoted literal named-family identity, not a
fourth generic or CSS directive. Only the three typed GenericFontFamily choices
have generic semantics. Named identity retains original bytes; matching still
uses the current Unicode-16 NFKC-casefold policy. The former raw parser's generic
token recognition is not another public constructor or a runtime parse fallback.

FontStack requires a nonempty ordered sequence, retains supplied order and
duplicates, and exposes no mutable entries or raw CSS constructor. The canonical
font owner consumes typed entries directly; CSS serialization is publication
syntax, not a string that is parsed again for font selection. Candidate ordering,
weights 400/600/700, fixed Normal style/stretch, host generics, base-plus-mark
selection, bounded discovery and retained same-byte faces remain spec03/spec05.

Every built-in text role preserves this typed order:
`named("Noto Sans")`, `named("Noto Sans CJK SC")`, `named("DejaVu Sans")`,
`named("Liberation Sans")`, `named("Arial")`, `generic(SansSerif)`.
After B4, callers obtain it with, for example,
`Theme::publication().typography().root().title().stack().clone()` and inspect
`families()[0]`. No `DEFAULT_*` export or global first-family sentinel survives.

B1/B2 migrate `orchestrator/backend.rs::{stack_for,plan_raw_bracket}` and
`render/geometry.rs::plan_raw_text` to an explicit retained role/stack operand.
The string `"Noto Sans"` must mean that actual named family when supplied, not
"substitute operation default stack". Production text and bracket paths consume
resolved/retained text plans; test-only raw adapters use typed inputs. B4 deletes
the two root constants and migrates layout fixtures, benchmark fonts and the
public-contract fallback test listed in A1.

## 4. Surface And Complete Typography

```rust,ignore
impl SurfaceTheme {
    fn new(canvas_root: Color, plot_panel: Color) -> Result<Self>;
}
impl TextTheme {
    fn new(stack: FontStack, color: Color, font_size: u32,
           weight: TextWeight, line_height: f64) -> Result<Self>;
}
impl RootTextRoles {
    fn new(title: TextTheme, subtitle: TextTheme, caption: TextTheme) -> Result<Self>;
}
impl AxisTextRoles {
    fn new(axis_title: TextTheme, tick_label: TextTheme) -> Result<Self>;
}
impl GuideTextRoles {
    fn new(legend_title: TextTheme, legend_label: TextTheme) -> Result<Self>;
}
impl CompositionTextRoles {
    fn new(facet_header: TextTheme, composition_header: TextTheme,
           text_panel_title: TextTheme, body: TextTheme) -> Result<Self>;
}
impl ContrastTextRoles {
    fn new(light: TextTheme, dark: TextTheme) -> Result<Self>;
}
impl AnnotationTextRoles {
    fn new(significance_label: TextTheme, estimate_detail: TextTheme,
           waterfall_total: TextTheme, point_layer_label: TextTheme,
           panel_text: TextTheme, contrast: ContrastTextRoles) -> Result<Self>;
}
impl TypographyTheme {
    fn new(root: RootTextRoles, axes: AxisTextRoles, guides: GuideTextRoles,
           composition: CompositionTextRoles,
           annotations: AnnotationTextRoles) -> Result<Self>;
}
```

Surface paints may be transparent. TextTheme requires positive CSS font size and
finite positive line-height multiplier. ContrastTextRoles additionally requires
both candidate colors opaque, including after an edit. Complete candidate stack,
size, weight and leading may differ; select the complete role before fonts and
measurement. General TextTheme construction has no universal contrast threshold.

The following table is the exhaustive **18-role** construction/default and
consumer mapping. `F` is the preset foreground; `T/S/C/A` are the preset title,
subtitle, caption and axis-title sizes in the next table. All built-in weights
are Normal. `L(n) = (f64::from(n)+6.0)/f64::from(n)` preserves the exact current
binary64 operation order/bits; the other leading multiplier is the current `1.2`.

For Publication title size 20, `L(20)` is the raw binary64 `1.3`, bits
`0x3ff4cccccccccccd`, slightly greater than the rational `13/10`. Its exact product
with `20 * 1024` quanta rounds upward to **26625 quanta**, or 26px plus one quantum.
Do not replace it with a rational 13/10 or an absolute 26px pitch to obtain 26624.
Keep the retained multiplier and actual selected text-plan occupancy; a displayed
decimal spelling alone is not proof of default geometry equality.

| Public path below `TypographyTheme` | Current source/consumer | Built-in color / CSS size / leading |
| --- | --- | --- |
| `root.title` | `FoundationalTypography.title`; Plot/Canvas root and embedded Plot headings | F / T / L(T) |
| `root.subtitle` | `subtitle`; same owners, independently aligned | `(80,80,80,255)` / S / L(S) |
| `root.caption` | `caption`; independent Plot/Canvas caption | `(80,80,80,255)` / C / L(C) |
| `axes.axis_title` | `axis_title`; standard, specialized, Matrix and shared facet axes | F / A / 1.2 |
| `axes.tick_label` | `tick_label`; generated/exact numeric and categorical ticks | F / 16 / 1.2 |
| `guides.legend_title` | `legend_title`; discrete/continuous guide title | `(60,60,60,255)` / 12 / 1.2 |
| `guides.legend_label` | `legend_label`; entries, continuous numbers, missing key; T6L neutral group RGB authority | `(40,40,40,255)` / 11 / 1.2 |
| `annotations.significance_label` | `significance_label`; stars/attached marks/bracket labels before optional exact size/weight | F / 16 / 1.2 |
| `annotations.estimate_detail` | `estimate_detail`; Coefficient values and unavailable-row marker | F / 14 / 1.2 |
| `annotations.waterfall_total` | `waterfall_total`; prepared base/terminal shelves | F / 14 / 1.2 |
| `annotations.point_layer_label` | `point_layer_label`; exact supplied strings on appended points | F / 10 / 1.2 |
| `annotations.panel_text` | `panel_text`; caller panel annotation | F / 14 / 1.2 |
| `annotations.contrast.light` | `contrast_light`; Heatmap values and fully-null-row text | white opaque / 11 / 1.2 |
| `annotations.contrast.dark` | `contrast_dark`; same complete-selection route | black opaque / 11 / 1.2 |
| `composition.facet_header` | Legacy `facet_header_font_size` plus `legacy_text_appearance`; facet strip planning, rendering, intrinsic and census | F / S / 1.2 |
| `composition.composition_header` | Legacy `composition_title_font_size`; Canvas column/row headers and row titles | F / T / 1.2 |
| `composition.text_panel_title` | Legacy composition title; Canvas TextPanel title | F / T / 1.2 |
| `composition.body` | Legacy `label_font_size`; Canvas TextPanel body | F / A / 1.2 |

| Preset | F (opaque RGB) | T | S | C | A | Axis width | Bracket width | Grid RGB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Publication | `(0,0,0)` | 20 | 15 | 15 | 17 | 0.8f32 | 1.0f32 | `(220,220,220)` |
| Nature | `(40,40,40)` | 19 | 14 | 14 | 16 | 0.8f32 | 0.8f32 | `(230,230,230)` |
| Science | `(0,0,0)` | 21 | 16 | 15 | 17 | 1.0f32 | 1.0f32 | `(200,200,200)` |
| CHI | `(60,60,60)` | 19 | 15 | 15 | 17 | 0.8f32 | 0.8f32 | `(220,220,220)` |

Every one of the fourteen existing FoundationalTypography fields maps once in
this table. Four distinct admitted legacy roles become complete values; sharing
their initial bytes does not couple later edits. There is no remaining generic
font, foreground, body-size, detail-size or composition-size fallback.
The deferred text census's generic `body` temporary at
`layout/text/requirements.rs::push_deferred_plot_requirements` must use the actual
facet/annotation consumer role rather than guess all deferred text is detail 14.

B2 removes `legacy_text_appearance` and all generic legacy typography carriers.
Facet strip reserve uses actual complete selected text occupancy, replacing the
current **`8 + line_count * (facet_size + 6)`** estimate in
`layout/canvas.rs::PanelRequirements::measure` and
`orchestrator/facets/tracks.rs::allocate`, plus the independently reconstructed
14px first-line offset in `orchestrator/facets/rendering.rs::plan_facet_strip`.
Actual strip text currently uses the complete legacy appearance's **1.2** leading.
Use that preserved role and §7's width/clearance relationships for one measured
strip, including its actual baselines and outward glyph ink.

This is an intentional B2 complete-ink/measure-paint correction, not default-byte
preservation or another leading control. Preserve the prepared raw facet labels,
their display strings, present strip/source cardinality, and every selected text
line/glyph; compare these source/line counts before attributing changed reserve,
cell allocation, baseline, fit or output. Do not remove a source or trim the
snapshot to hide a changed strip. Root leading bits and every retained text role
default remain pinned above. The approved automatic PointLayer label-paint
correction is separately specified in §11.

## 5. Validated Compound Paint And Glyph Inputs

These are role-construction values owned by the foundations, not re-exports of
`render::backend::{LineStyle,FillStyle,PointStyle,PointOutline,BracketStyle}`.
They contain no arbitrary dash string, draw geometry, clipping, z-order or backend
protocol. A private adapter derives final backend paint operands from them once.

```rust,ignore
enum StrokePattern { Solid, ShortDash, MediumDash, LongDash }

impl StrokeTheme {
    fn new(color: Color, width: f32, pattern: StrokePattern) -> Result<Self>;
}
impl SolidStrokeTheme {
    fn new(color: Color, width: f32) -> Result<Self>;
}
impl OutlineTheme {
    fn new(color: Color, width: f32) -> Result<Self>;
}
impl PointOutlineTheme {
    fn new(color: Color, width_ratio: RelativeSpacing) -> Result<Self>;
}
impl DataOutlineTheme {
    fn fixed(stroke: OutlineTheme) -> Self;
    fn mapped(width: f32, alpha: u8) -> Result<Self>;
    fn fixed_stroke(&self) -> Option<&OutlineTheme>;
    fn mapped_width(&self) -> Option<f32>;
    fn mapped_alpha(&self) -> Option<u8>;
}
impl DataFillTheme {
    fn new(fill_alpha: u8, outline: Option<DataOutlineTheme>) -> Result<Self>;
}
impl FixedFillTheme {
    fn new(color: Color, outline: Option<OutlineTheme>) -> Result<Self>;
}
impl PointTheme {
    fn new(fill_alpha: u8, shape: PointShape,
           outline: Option<PointOutlineTheme>) -> Result<Self>;
}
impl SizedPointTheme {
    fn new(appearance: PointTheme, radius: u32) -> Result<Self>;
}
impl SupplementalPointTheme {
    fn new(radius: u32, fill_alpha: u8) -> Result<Self>;
}
impl DataLineTheme {
    fn new(width: f32, alpha: u8, pattern: StrokePattern) -> Result<Self>;
}
```

All present widths and PointOutlineTheme's width ratio are strictly positive;
alpha includes both endpoints. PointTheme outline absence is structural and the
retained `point_outline(false)` policy can suppress a present outline after role
selection. SupplementalPointTheme deliberately retains the current plain-circle,
no-outline geometry; it offers only the existing radius/alpha defaults.
SizedPointTheme requires a positive integral CSS radius and supplies one complete
point role with independently editable appearance/radius; its fields follow §2.
PointOutlineTheme resolves its exact ratio against the final positive integral
CSS point radius: width/center normalization is nearest-even, and hard radius plus
outline occupancy and painted edges are outward/upward under §2. It never resolves
against another role's radius or changes fill alpha. Fixed CSS outline/stroke
widths likewise use nearest-even width normalization and outward painted edges.
Mapped outlines take the same opaque data color as the fill, then install their
own alpha. Fixed outlines keep their complete Color. DataOutlineTheme's choice
and payload are private; replace its whole value through the parent edit or
construct the other choice, not through raw enum variants.

The closed stroke grammar is exactly current `Solid`, `3 2`, `4 3`, `5 3`.
The root foundation owns that closed vocabulary; the backend imports/converts it
privately. Existing exact `LinePattern::{Solid,Dashed}` remains its narrower
two-choice vocabulary, mapping Dashed to LongDash. No additional Interaction or
exact Guide dash controls follow merely from constructing a theme stroke.

| Current compound record | Complete target field mapping |
| --- | --- |
| `StrokeAppearance.{paint,width,pattern}` | `StrokeTheme.{color,width,pattern}` |
| `BracketStrokeAppearance.{paint,width}` | `SolidStrokeTheme.{color,width}`; solid bracket topology is fixed |
| `FillOutlineAppearance.{paint,width}` | present `OutlineTheme.{color,width}` |
| `DataFillAppearance.{fill_alpha,outline_paint,outline_width}` | DataFillTheme fill alpha plus structural optional DataOutlineTheme; Fixed paint or mapped color/independent alpha, present positive width |
| `FillOutlinePaint::{Fixed,Data}` | private validated DataOutlineTheme choice; old `Data` has outline alpha 255 |
| `FixedFillAppearance.{paint,outline_paint,outline_width}` | FixedFillTheme color plus structural optional OutlineTheme |
| `PointAppearance.{fill_alpha,shape,outline}` | PointTheme fields, including complete independently painted optional PointOutlineTheme |
| backend `PointOutline.{color,width_ratio}` | PointOutlineTheme complete Color and exact local radius relationship |
| `DataLineAppearance.{width,alpha,pattern}` | DataLineTheme fields |
| `PointSizeAppearance` | all twelve PointRadii fields in §6 |

At B1/B2, current fill outline widths of zero become `None`, rather than an
invalid present OutlineTheme. No absent outline paint participates in identity or
guide compatibility. Built-in present widths preserve their original f32 bits;
colors/alpha preserve their already quantized channels.

## 6. Complete Mark Construction And Exhaustive Current Field Coverage

### Typed Role Groups

```rust,ignore
impl PointRadii {
    fn new(scatter: u32, path: u32, trajectory: u32, regression: u32,
           qq: u32, marginal: u32, matrix: u32, interval_estimate: u32,
           dumbbell: u32, interaction: u32, beeswarm: u32,
           box_flier: u32) -> Result<Self>;
}
impl PointRoles {
    fn new(primary: PointTheme, raw_overlay: SizedPointTheme,
           mandatory_strip: SizedPointTheme, radii: PointRadii,
           supplemental: SupplementalPointTheme) -> Result<Self>;
}
impl BodyFillRoles {
    fn new(bar_body: DataFillTheme, box_body: DataFillTheme,
           violin_body: DataFillTheme, inner_summary: FixedFillTheme,
           histogram_bin: DataFillTheme, waterfall_bar: DataFillTheme) -> Result<Self>;
}
impl BandFillRoles {
    fn new(line_band: DataFillTheme, regression_band: DataFillTheme) -> Result<Self>;
}
impl DensityFillRoles {
    fn new(density_area: DataFillTheme, marginal_density: DataFillTheme,
           matrix_density: DataFillTheme) -> Result<Self>;
}
impl FillRoles {
    fn new(bodies: BodyFillRoles, bands: BandFillRoles,
           densities: DensityFillRoles) -> Result<Self>;
}
impl DataLineRoles {
    fn new(series_line: DataLineTheme, path_line: DataLineTheme,
           density_line: DataLineTheme, interaction_interval: DataLineTheme,
           interaction_connector: DataLineTheme) -> Result<Self>;
}
impl NeutralPaintRoles {
    fn new(path: OpaqueColor, box_flier: OpaqueColor) -> Result<Self>;
    fn path(&self) -> OpaqueColor;
    fn box_flier(&self) -> OpaqueColor;
    fn with_path(self, color: OpaqueColor) -> Result<Self>;
    fn with_box_flier(self, color: OpaqueColor) -> Result<Self>;
}
impl ReferenceStrokeRoles {
    fn new(bar_zero: StrokeTheme, waterfall_reference: StrokeTheme,
           beeswarm_reference: StrokeTheme, point_interval_reference: StrokeTheme,
           estimate_reference: StrokeTheme, qq_reference: StrokeTheme) -> Result<Self>;
}
impl ConnectorStrokeRoles {
    fn new(waterfall_connector: StrokeTheme,
           dumbbell_connector: StrokeTheme) -> Result<Self>;
}
impl SummaryStrokeRoles {
    fn new(uncertainty: StrokeTheme, summary: StrokeTheme,
           estimate_interval: StrokeTheme, median: StrokeTheme) -> Result<Self>;
}
impl CurveStrokeRoles {
    fn new(histogram_curve: StrokeTheme, histogram_rug: StrokeTheme,
           regression_fit: StrokeTheme) -> Result<Self>;
}
impl StrokeRoles {
    fn new(references: ReferenceStrokeRoles, connectors: ConnectorStrokeRoles,
           summaries: SummaryStrokeRoles, curves: CurveStrokeRoles) -> Result<Self>;
}
impl CellMarkRoles {
    fn new(fill_alpha: u8, cell_outline: Option<OutlineTheme>, missing_fill: Color,
           missing_row_outline: Option<OutlineTheme>,
           missing_key_outline: Option<OutlineTheme>,
           missing_cross: StrokeTheme, missing_cross_inset: RelativeSpacing) -> Result<Self>;
}
impl CategoryMarkGeometry {
    fn new(vertical_body_breadth: RelativeSpacing,
           horizontal_body_breadth: RelativeSpacing,
           vertical_group_span: RelativeSpacing,
           horizontal_group_span: RelativeSpacing,
           uncertainty_cap_ratio: RelativeSpacing) -> Result<Self>;
}
impl MarkExtentTheme {
    fn new(categories: CategoryMarkGeometry,
           waterfall_body_category_breadth: RelativeSpacing,
           point_interval_cap_half_height: u32,
           sampled_rug_height: RelativeSpacing,
           marginal_density_fraction: RelativeSpacing,
           matrix_density_fraction: RelativeSpacing) -> Result<Self>;
}
impl MarkTheme {
    fn new(accents: AccentPalette, points: PointRoles, fills: FillRoles,
           data_lines: DataLineRoles, neutral_paints: NeutralPaintRoles,
           strokes: StrokeRoles, cells: CellMarkRoles,
           extents: MarkExtentTheme) -> Result<Self>;
}
```

This has eight complete typed MarkTheme inputs, rather than a forty-argument
scalar constructor or publicly writable backend-shaped record. Every grouped
field is constructible and inspectable under §2. The group taxonomy is points,
fills, mapped lines, role-specific neutral RGB, neutral strokes, cells/missing
marks and extents, not one
component per plot family. Source-specific role names identify already admitted
consumers without creating extra inheritance scopes.

All radii/cap half-height are positive integral CSS pixels. Category breadths and
group spans are positive fractions no greater than one; cap ratio is positive
and at most one half. The independent Waterfall body-category breadth is also in
`(0,1]`, with final outline/connector containment checked at preflight. Density
fractions are in `(0,1]`; sampled rug height is
nonnegative. Missing-cross inset is nonnegative and smaller than one half.
Final category/stroke/cap containment remains preflight geometry validation.

### Every Current MarkAppearance Field

The following **41 rows** cover every current `MarkAppearance` field exactly
once; this private record is **not the entire admitted MarkTheme scope**. In
particular the ledger already distinguishes optional raw-overlay and mandatory
strip roles that current storage conflates. Paths start at MarkTheme except the
explicitly reassigned highlight row.
`F` means opaque preset foreground; `W` white; **M** means mapped opaque data
outline with alpha 255; **absent** means structural `None`, not width zero.
Widths below are CSS f32 values and preserve source bits.

| Current field | Target path / typed value | Preserved built-in value |
| --- | --- | --- |
| `primary_point` | `points.primary: PointTheme` | fill alpha 209, Circle, white outline ratio `new(0.1)` |
| `bar_body` | `fills.bodies.bar_body: DataFillTheme` | alpha 255; fixed F outline 1.0 |
| `line_band` | `fills.bands.line_band: DataFillTheme` | alpha 46; outline absent |
| `regression_band` | `fills.bands.regression_band: DataFillTheme` | alpha 64; outline absent |
| `density_area` | `fills.densities.density_area: DataFillTheme` | alpha 56; outline absent |
| `marginal_density` | `fills.densities.marginal_density: DataFillTheme` | alpha 115; M outline 1.0 |
| `matrix_density` | `fills.densities.matrix_density: DataFillTheme` | alpha 102; M outline 1.0 |
| `box_body` | `fills.bodies.box_body: DataFillTheme` | alpha 255; fixed F outline 1.0 |
| `violin_body` | `fills.bodies.violin_body: DataFillTheme` | alpha 178; fixed F outline 1.0 |
| `raincloud_box` | `fills.bodies.inner_summary: FixedFillTheme` | opaque W; fixed F outline 1.0; shared existing Violin/Raincloud inner-summary role |
| `histogram_bin` | `fills.bodies.histogram_bin: DataFillTheme` | alpha 209; fixed F outline 0.8 |
| `waterfall_bar` | `fills.bodies.waterfall_bar: DataFillTheme` | alpha 255; fixed F outline 0.8 |
| `raw_overlay_point` | `points.raw_overlay.appearance: PointTheme`; its current Raincloud consumers migrate to the independent `points.mandatory_strip.appearance` | both initially fill alpha 209, Circle, white outline ratio `new(0.1)` |
| `raw_overlay_radius` | `points.raw_overlay.radius: u32`; its current Raincloud consumers migrate to independent `points.mandatory_strip.radius` | both initially 2 |
| `heatmap_cell_outline` | `cells.cell_outline: Option<OutlineTheme>` | W, 0.5 |
| `heatmap_highlight_outline` | **`AnnotationTheme.cells.highlight_outline: OutlineTheme`** | F, 2.5; ledger's annotation-highlight owner, not a duplicated Mark field |
| `heatmap_missing_fill` | `cells.missing_fill: Color` | `(224,224,224,255)` |
| `heatmap_missing_row_outline` | `cells.missing_row_outline: Option<OutlineTheme>` | W, 0.5 |
| `heatmap_missing_key_outline` | `cells.missing_key_outline: Option<OutlineTheme>` | `(120,120,120,255)`, 0.8 |
| `heatmap_missing_cross` | `cells.missing_cross: StrokeTheme` | `(120,120,120,255)`, 1.0, Solid |
| `bar_zero` | `strokes.references.bar_zero: StrokeTheme` | F, 0.8, Solid |
| `waterfall_connector` | `strokes.connectors.waterfall_connector: StrokeTheme` | F, 0.7, ShortDash |
| `waterfall_reference` | `strokes.references.waterfall_reference: StrokeTheme` | F, 0.7, MediumDash |
| `raincloud_median` | `strokes.summaries.median: StrokeTheme` | F, 1.5, Solid; existing Violin/Raincloud median role |
| `beeswarm_reference` | `strokes.references.beeswarm_reference: StrokeTheme` | F, 0.8, Solid |
| `interaction_interval` | `data_lines.interaction_interval: DataLineTheme` | width 1.2, alpha 255, Solid |
| `interaction_connector` | `data_lines.interaction_connector: DataLineTheme` | width 1.8, alpha 255, Solid |
| `point_sizes` | `points.radii: PointRadii` | all twelve values in the next table |
| `uncertainty_stroke` | `strokes.summaries.uncertainty: StrokeTheme` | F, preset axis-width **value copied at construction**, Solid |
| `summary_stroke` | `strokes.summaries.summary: StrokeTheme` | F, preset axis-width value copied at construction, Solid |
| `series_line` | `data_lines.series_line: DataLineTheme` | width 1.8, alpha 255, Solid |
| `path_line` | `data_lines.path_line: DataLineTheme` | width 1.4, alpha 255, Solid |
| `histogram_curve` | `strokes.curves.histogram_curve: StrokeTheme` | F, 1.0, Solid |
| `histogram_rug` | `strokes.curves.histogram_rug: StrokeTheme` | F, 0.7, Solid |
| `density_line` | `data_lines.density_line: DataLineTheme` | width 1.6, alpha 255, Solid |
| `regression_fit` | `strokes.curves.regression_fit: StrokeTheme` | F, 1.5, Solid |
| `estimate_interval` | `strokes.summaries.estimate_interval: StrokeTheme` | F, 1.4, Solid |
| `point_interval_reference` | `strokes.references.point_interval_reference: StrokeTheme` | F, 0.8, Solid |
| `estimate_reference` | `strokes.references.estimate_reference: StrokeTheme` | F, 1.0, MediumDash |
| `qq_reference` | `strokes.references.qq_reference: StrokeTheme` | F, 1.2, Solid |
| `dumbbell_connector` | `strokes.connectors.dumbbell_connector: StrokeTheme` | F, 1.2, Solid |

| PointRadii field | Default | Current role consumer |
| --- | ---: | --- |
| `scatter` | 3 | Scatter observations |
| `path` | 3 | ordinary Line observations, intrinsics and originating guide points |
| `trajectory` | 4 | Trajectory/LinePath observations at their actual consuming path |
| `regression` | 3 | Regression observations |
| `qq` | 3 | Q-Q observations |
| `marginal` | 3 | Marginal central observations |
| `matrix` | 2 | Matrix off-diagonal observations |
| `interval_estimate` | 4 | Coefficient/Forest/PointInterval estimates |
| `dumbbell` | 4 | exact painted endpoints |
| `interaction` | 4 | group estimate points |
| `beeswarm` | 3 | tangent-packed observations |
| `box_flier` | 2 | Boxplot flier role |

Current source-path choices are preserved explicitly at B2: ordinary Line uses
`path` radius 3; Trajectory and LinePath use `trajectory` radius 4. Intrinsics,
marks and originating guide glyphs use the same selected role. The historical
field name `path` is not a reason to change LinePath's default radius. No new
marker-presence setting follows from these radius roles.

### Constructed Neutral Path And Flier Paint Owners

`NeutralPaintRoles` has private fields and is one complete role-construction
value inside MarkTheme, not a replacement generic foreground or independently
inherited component. Its two `OpaqueColor` inputs are non-data RGB responsibilities.
They deliberately have **no alpha field**: the existing path-line and primary-point
appearance roles each provide the only fill/stroke alpha for their consumer.
OpaqueColor is already used for non-data Mark accents; using it here does not
create a mapped data aesthetic or a new Plot-local neutral-color setter.

| Current foreground consumer / admitted owner | Single constructed replacement | Preserved preset appearance |
| --- | --- | --- |
| `plots/line.rs::PreparedLine::group_color` neutral branch, then the `path_line` materialization in `measure`; ledger `marks.neutral.current-foreground` and `marks.line.grouped-path-final-style` | `marks.neutral_paints.path` supplies neutral Trajectory/LinePath RGB; `marks.data_lines.path_line` supplies its existing width, alpha and closed pattern | RGB F from §4; path width 1.4f32, alpha 255, Solid |
| `plots/boxplot.rs::intrinsic_body_minimum` fallback when raw overlay is hidden, and `measure`'s actual flier materialization; ledger `marks.neutral.current-foreground` and `marks.boxplot.hidden-overlay-flier-role` | `marks.neutral_paints.box_flier` supplies RGB; `marks.points.primary` supplies fill alpha, shape and independent optional outline; `marks.points.radii.box_flier` supplies radius; existing outline visibility applies once | RGB F; primary fill alpha 209, Circle, radius 2, white outline ratio `new(0.1)` when visible |

For Publication/Nature/Science/CHI each neutral input is constructed explicitly
as opaque RGB `(0,0,0)` / `(40,40,40)` / `(0,0,0)` / `(60,60,60)`, respectively.
Equal initial RGB may be copied during built-in construction, but neither neutral
input reads a text role, AxisTheme stroke, reference/summary stroke, qualitative
palette or accent at resolution. The two neutral inputs are independently editable
through the signatures above; MarkTheme's `neutral_paints` getter/replacement follows
§2. No remaining `ResolvedTheme.foreground` getter or hidden generic seed supplies
either path.

Private materialization is explicit: a neutral path's final Color is
`Color::rgba(path.red(), path.green(), path.blue(), path_line.alpha())`, paired
with that same path_line width/pattern. A flier's final fill Color similarly uses
box_flier RGB and **primary.fill_alpha exactly once**, paired with primary shape,
the selected independent outline and the flier radius. It does not first accept
an RGBA neutral alpha and then multiply/override it with a second role alpha.
An independent outline keeps its own complete Color; a path alpha edit does not
change flier alpha or reference/text paint, and a primary-point alpha edit retains
its existing effect on fliers. This preserves the current shared appearance roles
without duplicating alpha/geometry defaults in NeutralPaintRoles.

B2 changes both the producer and intrinsic/guide/capture consuming helpers.
The Line unkeyed branch still uses accents; hue LinePath still uses its opaque
categorical mapping and the same path_line appearance. Neutral paths alone use
the path neutral input; row-hue observation points retain their separate mapped
point paint. Boxplot optional observations still use their raw-overlay role and
actual categorical mapping; the hidden-overlay flier branch uses its neutral RGB.
Prepared/scoped copies and final style identities retain the selected semantic
branch and complete resulting paint. This is not color-equality inference or a
late fallback. B4 exposes the consumed typed roles/getters/edits and retires the
legacy foreground route with the rest of the complete Theme cutover.

### Independent Optional Observations And Mandatory Strip

| Admitted ledger owner | Construction/consumer authority | Preserved built-in default |
| --- | --- | --- |
| `marks.distribution.raw-overlay-final-point-style` (historical line 4445) | `points.raw_overlay: SizedPointTheme`; optional Boxplot/Violin raw observations after RawOverlay presence and point-outline policy | radius 2; alpha 209; Circle; white outline with `new(0.1)` ratio |
| `marks.raincloud.mandatory-strip-final-point-style` (4446) | `points.mandatory_strip: SizedPointTheme`; sampled and supplied Raincloud strips, strip packing/fit and originating point-bearing guide glyphs after point-outline policy | independent radius 2; alpha 209; Circle; white outline with `new(0.1)` ratio |

The active overlay does not merge these semantic roles. Equal built-in values may
be cloned during construction, but neither becomes the other's runtime default or
fallback. An edit to one role cannot change the other, its radius, its fit or its
guide. Mandatory strip presence remains Raincloud's scientific mark grammar;
RawOverlay policy cannot hide it, and theme construction adds no presence control.

B2 splits `distribution_sample.rs::overlay_style`'s current shared materialization
and the direct `raw_overlay_point/raw_overlay_radius` reads in
`violin.rs::{raincloud_strip_pitch,intrinsic_body_minimum}` and
`violin/precomputed.rs`. Every Raincloud sampled/supplied planner, mark draw,
pre-font intrinsic and originating legend key consumes the final mandatory-strip
style; optional observations consume the raw-overlay style. The ledger's
`marks.raincloud.{sampled,precomputed}-strip-planning-radius`, final-style
application and `marks.raincloud.precomputed-originating-legend-point-style`
records are consumers of this one strip role, not independent literal-2 controls.
Helpers that need occupied point ink receive the selected final style, including
the applied outline policy, rather than reconstructing it from a shared theme
field with `outline_visible = true`.

### Admitted Defaults Outside MarkAppearance

| Target field | Current owner/default | Metric and resolution |
| --- | --- | --- |
| `accents: AccentPalette` | legacy `ResolvedTheme.colors`, separately from categorical seeds | stable slot domain; §9 |
| `neutral_paints.path` | `PreparedLine::group_color`'s current foreground branch | opaque preset F RGB; alpha/width/pattern solely from `data_lines.path_line` |
| `neutral_paints.box_flier` | Boxplot hidden-overlay foreground branch in intrinsic and marks | opaque preset F RGB; alpha/shape/outline solely from `points.primary`, radius from `points.radii.box_flier` |
| `points.mandatory_strip.{appearance,radius}` | shared current `raw_overlay_point/raw_overlay_radius`, separated by ledger 4446 | complete independent point role; radius 2, alpha 209, Circle/white outline `new(0.1)` |
| `points.supplemental.radius` | `PointLayer::new`, 4 | integral CSS radius, exact normalization |
| `points.supplemental.fill_alpha` | PointLayer 0.8 quantized once, **204** | one inclusive u8 alpha, no floating carrier |
| `cells.fill_alpha` | opaque Heatmap mapped fill, 255 | once on mapped RGB; automatic contrast consumes this final cell fill, not the opaque scale color alone |
| `cells.missing_cross_inset` | `plots/heatmap.rs`, `new(0.16)` | smaller normalized cell side; S/H; existing bounded cross construction and stroke-overhang clearance remain private |
| `extents.categories.vertical_body_breadth` | `bar_data::category_bar_geometry`, `new(0.6)` | category cell fraction |
| `extents.categories.horizontal_body_breadth` | same horizontal path, `new(0.55)` | category cell fraction; do not assume orientation parity |
| `extents.categories.vertical_group_span` | `category_center`/geometry, `new(0.76)` | category cell fraction; centers/Attached/BarPointLayer reuse one value |
| `extents.categories.horizontal_group_span` | same horizontal path, `new(0.74)` | category cell fraction |
| `extents.categories.uncertainty_cap_ratio` | Bar cap half-length, `new(0.2)` | final projected body breadth; S/H, independent stroke edges |
| `extents.waterfall_body_category_breadth` | `marks.waterfall.body-category-breadth`; `waterfall.rs`'s 0.6 intrinsic and 0.2/0.8 body-edge literals | independent `new(0.6)` fraction; one selected value derives Waterfall body edges, edge room and connector room below |
| `extents.point_interval_cap_half_height` | interval-estimate endpoint caps, 4 | integral local CSS geometry; no axis/data-domain conversion |
| `extents.sampled_rug_height` | Histogram maximum × `new(0.035)` | prepared finite scientific maximum; sampled-only; supplied rug geometry unchanged |
| `extents.marginal_density_fraction` | `new(0.90)` in both strips | usable stroke-inset local density span; S/H |
| `extents.matrix_density_fraction` | diagonal density `new(0.85)` | usable assigned local cell density span; S/H |

Category/rug scientific calculations retain their named binary64 operation order,
using the normalized relationship's nearest-even binary64 value as a nonspatial
geometry operand, then canonicalize resulting positions by spec05. They are not
cosmetic gaps resolved from root size. Default constructors retain the exact
original literal bits, rather than switching these calculations to decimal-ratio
arithmetic and silently changing scientific projection.

### Waterfall's Independent Body-Category Breadth

`marks.waterfall.body-category-breadth` (ledger 5905, migration 6419) already owns
this appearance responsibility. It is **not** the ordinary vertical Bar 0.6
role, and is not HorizontalBar's 0.55 role. The separate
`MarkExtentTheme.waterfall_body_category_breadth` private field, typed readonly
getter and validated `with_waterfall_body_category_breadth` edit follow §2.
Validate the exact retained fraction in `(0,1]`; scientific cumulative values,
sign accents, body paint/outline, connector style and category order remain their
existing independent owners.

One private prepared geometry derivation uses the selected breadth `b` (its
nearest-even binary64 nonspatial operand) in this exact, common operation order:

```text
free = 1.0 - b
low = free / 2.0
high = 1.0 - low
edge_room = low
connector_room = free
```

Every named operation rounds binary64 nearest-even without fused substitution;
the selected value is not changed, clamped or replaced by an ordinary-bar value.
The original literal bits are preserved for the default:

| Derived operand | Default bits/value | Actual source use |
| --- | --- | --- |
| `b` | `0x3fe3333333333333`, 0.6 | `mark_intrinsic_body_minimum`: positive body-span pitch requirement `quantum / b` |
| `low`, `edge_room` | `0x3fc999999999999a`, 0.2 | outline edge-room requirements, `index + low`, `band.0 + low * (band.1 - band.0)`, and next connector endpoint |
| `high` | `0x3fe999999999999a`, 0.8 | `index + high` and `band.0 + high * (band.1 - band.0)` |
| `connector_room` | `0x3fd999999999999a`, 0.4 | adjacent connector's complete stroke pitch requirement |

In particular do not substitute `high - low` for the declared `b` in the
intrinsic denominator: at the default those binary64 results need not have the
same bits. Keep the existing body operation order in `waterfall.rs::measure`:
scientific y hints are `index + low/high`; physical edges are
`band.0 + low/high * (band.1 - band.0)`, not recomputed center plus/minus a
separately rounded half-width. The connector starts at the **selected body's
actual top** and ends at `next.0 + low * (next.1 - next.0)`; retain exactly one
connector per adjacent source span, with no added categorical extent constant.

For every present span the hard pitch includes `quantum / b`; body lower/upper
outward outline extents require `lower_ink / edge_room` and
`upper_ink / edge_room` only when the respective required ink is positive.
For an actual adjacent connector with positive occupied stroke extent, include
`connector_ink / connector_room`. With positive room these are checked finite
requirements; overflow or nonzero canonical/source-span collapse refuses before
backend creation. Zero required ink contributes zero without evaluating `0/0`.
If room is zero and the associated required outline/connector ink is positive,
there is a terminal containing-geometry fit miss rather than division by zero,
a substituted breadth, or an unpriced expansion. Thus breadth 1 is constructible
but may fail with a present outward outline or occupied connector; a single body
with no such ink is not rejected by an arbitrary breadth ceiling below one.
Final `validate_span` and complete body/connector containment remain mandatory,
including narrow spans and subquantum positive strokes. Do not invent a fixed
panel/pitch floor or silently omit a connector/outline that cannot fit.

B2 routes Waterfall intrinsic pitch, both scientific/physical rectangle edges and
all connector endpoints through this one prepared value, retaining it in scoped
inputs, measurement/feedback identities and immutable capture. B4 exposes its
validated construction/edit with MarkExtentTheme. Default literal replacement
preserves the named operations at `waterfall.rs`'s current 96–109, 455–461 and
487–494 consumers; custom values affect all of those consumers coherently. An edit
to Bar breadth cannot alter Waterfall, and editing Waterfall cannot alter Bar.

Grouped slot count/order, Comfort's existing 0.82 conversion, distribution
packing span 0.72, box/violin widths, KDE/statistical algorithms, clipping and
mark-presence policies retain their renderer/scientific owners. A significance
grouped anchor consumes the actual final body/series centers; it does not acquire
an independently adjustable span capable of desynchronizing marks and labels.
The old historical anchor-span and duplicate extent relationships are reconciled
as derived consumer relationships, not additional public controls.

## 7. Axes, Guides And Composition

### Axis Construction

```rust,ignore
impl FrameSides {
    fn new(top: bool, right: bool, bottom: bool, left: bool) -> Result<Self>;
}
impl FrameTheme {
    fn new(sides: FrameSides, stroke: StrokeTheme) -> Result<Self>;
}
impl TickTheme {
    fn new(stroke: StrokeTheme, length: u32) -> Result<Self>;
}
impl AxisSpacing {
    fn new(x_tick_label_gap: RelativeSpacing, y_tick_label_gap: RelativeSpacing,
           x_title_tick_label_gap: RelativeSpacing,
           x_title_without_tick_labels_gap: RelativeSpacing,
           y_title_tick_label_gap: RelativeSpacing,
           y_title_edge_padding: RelativeSpacing) -> Result<Self>;
}
impl AxisTheme {
    fn new(frame: Option<FrameTheme>, cell_frame: Option<FrameTheme>, ticks: Option<TickTheme>,
            grid: Option<StrokeTheme>, spacing: AxisSpacing) -> Result<Self>;
}
```

FrameSides requires at least one side; all-absent frame is `None`. Tick length is
positive integral CSS pixels; tick absence is `None`. No x/y label/title content
flag exists in AxisTheme. `frame` owns the ordinary coordinate-panel outline;
`cell_frame` owns the variable-grid cell outline already identified by the ledger's
`axes.matrix.cell-frame-final-style`. These are separate visual roles in one
AxisTheme, not separate plot-family themes or an inherited fallback. Each has
independent getters and validated `with_field` construction edits. Standard
panels consume `frame`; Matrix cells consume `cell_frame`, each emitting precisely
its selected side segments once. Current `source_show_top_right_spines` is dead preset metadata
in the current standard full-four-side planner and is deleted, not exposed under
its misleading historical name. All four built-ins preserve the current four-side
ordinary frame and bottom/left-only cell frame, each with F/preset-width/Solid.
Ticks retain length 5 with F/preset width/Solid, and grid preset RGB/0.5/MediumDash.
This corrects the earlier construction summary that incorrectly generalized the
ordinary default to Matrix. Adding explicit side selection must not silently
change the existing default cell topology. There is no selector testing whether
a theme is built-in/custom, no optional instruction layered over frame defaults,
and no renderer-imposed mask: a custom cell_frame may select any nonempty subset,
or None, independently of the ordinary frame.

Let `E(role)` mean positive integral CSS font-size quanta of that complete role.
Ratios in this table are exact; `A` is the preset axis-title size from §4.

| AxisSpacing field | Exact built-in ratio | Local metric | Rounding/consumer |
| --- | --- | --- | --- |
| `x_tick_label_gap` | 8/16 | E(tick_label) | S/H; Bottom label offset/gutter |
| `y_tick_label_gap` | 6/16 | E(tick_label) | S/H; Left label offset/gutter |
| `x_title_tick_label_gap` | 8/A | E(axis_title) | S/H; title after present labels |
| `x_title_without_tick_labels_gap` | 16/A | E(axis_title) | S/H; title after ticks with no label ink |
| `y_title_tick_label_gap` | 16/A | E(axis_title) | S/H; Left title clearance |
| `y_title_edge_padding` | 12/A | E(axis_title) | H for outer gutter; S for corresponding placement |

Tick/stroke outward extents are added separately from those spacings. Hidden tick
marks contribute no tick length/stroke operand, while requested label/title text
still contributes complete occupancy. Empty exact strings retain tick positions
but contribute no text gap/ink. Axis interval/format is a policy, not these fields.

### Legend Construction

```rust,ignore
impl LegendSpacing {
    fn new(padding: RelativeSpacing, key_width: RelativeSpacing,
           key_label_gap: RelativeSpacing, outside_gap: RelativeSpacing,
           shared_gap: RelativeSpacing, row_gap: RelativeSpacing,
           column_gap: RelativeSpacing, title_gap: RelativeSpacing,
           glyph_edge_padding: RelativeSpacing) -> Result<Self>;
}
impl GradientLengths {
    fn new(minimum: RelativeSpacing, preferred: RelativeSpacing) -> Result<Self>;
}
impl LegendFrameTheme {
    fn new(fill: Color, outline: Option<OutlineTheme>,
           radius: RelativeSpacing) -> Result<Self>;
}
impl LegendTheme {
    fn new(spacing: LegendSpacing, gradient_lengths: GradientLengths,
           frame: Option<LegendFrameTheme>) -> Result<Self>;
    fn shared_gap(&self) -> &RelativeSpacing;
    fn with_shared_gap(self, value: RelativeSpacing) -> Result<Self>;
}
```

`shared_gap` above is the sole convenience access/edit of the nested retained
`spacing.shared_gap`, not another field. It is the only Shared-gap configuration
owner after B4; it is meaningful when Shared is selected and harmless theme
appearance when no shared guide is present. No theme/policy pairing rejects it.
`key_width` and gradient minimum/preferred are strictly positive; preferred is
at least minimum under their same metric. Other spacings/radius are nonnegative.
Frame absence removes chrome, not guide content or required glyph occupancy.

All **14 current LegendAppearance fields** map as follows. Define
`P(p,m) = exact_binary64(p) / m`, reduced exactly into RelativeSpacing, where `p`
is the current pixel literal and `m` its original positive integral local em.
This private built-in assembly operation uses checked rational division, **not**
`RelativeSpacing::new(p / f64::from(m))`. It adds no public factory, retained
constructor mode, legacy pixel field or second representation. A caller can clone
the exact built-in relationship through its typed getter.

| Current field | Target path | Exact built-in ratio/paint | Local metric / rounding |
| --- | --- | --- | --- |
| `padding` | `spacing.padding` | P(8.8,11) | E(legend_label); S/H |
| `key_width` | `spacing.key_width` | 1/1 | E(legend_label); S/H |
| `key_label_gap` | `spacing.key_label_gap` | P(8.8,11) | E(legend_label); S/H |
| `outside_gap` | `spacing.outside_gap` | P(8.8,11) | E(legend_label); S/H |
| `shared_gap` | `spacing.shared_gap` | P(17.6,11) | E(Canvas legend_label); S/H |
| `gradient_min_length` | `gradient_lengths.minimum` | 40/11 | E(legend_label); H |
| `gradient_preferred_length` | `gradient_lengths.preferred` | 160/11 | E(legend_label); S; complete label hard minimum remains H |
| `row_gap` | `spacing.row_gap` | P(3.3,11) | E(legend_label); S/H |
| `column_gap` | `spacing.column_gap` | P(13.2,11) | E(legend_label); S/H |
| `title_gap` | `spacing.title_gap` | P(7.8,12) | E(legend_title); S/H |
| `frame_fill` | `frame.fill` | `(255,255,255,204)` | complete Color |
| `frame_outline` | `frame.outline.color` | `(200,200,200,255)` | independent outline Color |
| `frame_stroke_width` | `frame.outline.width` | 1.0f32 | positive CSS width; S and outward ink |
| `frame_radius` | `frame.radius` | 2/11 | E(legend_label); S; containing ink outward |

Thus defaults preserve the exact represented pixel inputs 8.8/11/8.8/8.8/17.6,
40/160, 3.3/13.2/7.8 and radius 2, before their spatial-role rounding. Exact key
glyph ink can exceed the key-width preference and supplies
the hard key slot; padding/gaps are not multiplied by glyph size. Text baselines,
key overhangs and frame stroke remain separate exact occupancy operands in
`LegendMetrics`, discrete intrinsics and continuous-guide measurement. B2 derives
them from typed local relationships rather than retaining the old pixel fields.

The admitted edge comfort is outside the private appearance record today:

| Ledger/current source | Target field / default | Metric, application and consumers |
| --- | --- | --- |
| `legend.container.current-glyph-edge-padding-factor` (3504); `decorations/legend.rs::GLYPH_EDGE_PAD_FACTOR = 0.2` | `LegendSpacing.glyph_edge_padding: RelativeSpacing`, `new(0.2)` | E(legend_label); S/H; added to ordinary padding **once at each vertical edge**, not horizontally and not once per entry/section |

B2 removes that constant and routes its natural-size, assigned-measurement and
edge-padding/box consumers through `LegendMetrics`' one resolved comfort value.
The private vertical edge inset is `padding + glyph_edge_padding`; horizontal
inset remains padding. Section flow uses the one outer frame's edge inset once,
not a separate comfort multiplier for each section. Exact glyph ink still supplies
hard extents independently; comfort neither estimates glyph height nor permits
clipping. This field follows §2's typed getter/consuming-edit rule.

#### Binary Literal And Composite-Rounding Preservation

| Current binary64 pixel literal | Exact source bits | Built-in local metric |
| --- | --- | --- |
| 8.8 | `0x402199999999999a` | legend_label 11 |
| 17.6 | `0x403199999999999a` | legend_label 11 |
| 3.3 | `0x400a666666666666` | legend_label 11 |
| 13.2 | `0x402a666666666666` | legend_label 11 |
| 7.8 | `0x401f333333333333` | legend_title 12 |

P(p,m) preserves those exact inputs; the nominal ratios 4/5, 8/5, 3/10, 6/5 and
13/20 are not equal to them and cannot silently replace them in built-in identity
or hard rounding. The edge factor likewise retains `0.2`'s binary64 bits
`0x3fc999999999999a`; it is not silently rewritten as exact 1/5.

Per-field input preservation does not prove equality of a rounded composite.
For example the current binary64 expression `8.8 + 0.2 * 11.0` rounds to 11.0,
while the exact sum of represented 8.8 and represented 0.2 times 11 is slightly
above 11. A direct upward hard normalization of that exact sum yields 11265
quanta, rather than 11264 from the rounded 11.0. The target rule is pinned: retain
the exact padding+comfort sum, round its **placement** operand S once to 11264,
and its **hard edge reserve** H once to 11265. Likewise form complete row/column
gap products/sums, title offsets and section gaps from their exact local products;
use S for positions/preferences and H for complete hard requirements as in §2.
Do not first round each gap to binary64 or sum individually rounded hard-gap
estimates. B2 records the source ordered-arithmetic expectation and the target
operand/result for each affected boundary. This explicit one-quantum
complete-ink/normalization correction is attributed to B2 with its changed
operand, containment and output evidence; preserving printed decimal spelling
is insufficient and a silent wholesale hash update is forbidden.
There is no preset-specific arithmetic switch or float round-trip fallback to
hide such a boundary. The separate raw line-height 1.3 rule in §4 remains intact.

T6L section gap is exactly the sum of resolved `title_gap + row_gap` under their
declared metrics, using S/H as appropriate. There is no new section-spacing field
or per-section component. Continuous guides use these same chrome/text owners.

#### Keyed Continuous-Guide Successors Beyond The Private Record

The historical T2f table assigns the following keys to theme appearance. Their
`A-guides-heatmap` overlay membership connects them to one assigned measured guide
under spec05; it does **not** by itself delete the keys or prove that every old
family-local cap becomes a new public component field. This table pins the
successor responsibilities against the current
`decorations/legend/continuous.rs::MeasuredContinuousGuide::{allocation_metrics,
measure,select}` owner. Theme typography/spacing values remain theme-owned;
containing allocation, complete text/glyph measurement and the resulting maximum
extent remain private renderer responsibilities. No new guide controls are added.

| Exact ledger key(s) / historical source | Successor field(s) and derived owner | B2/B4 migration disposition |
| --- | --- | --- |
| `guides.beeswarm.title-wrap-height-budget` (5528), `guides.heatmap.title-wrap-height-budget` (5576): `(min(panel_height,150 or 170) * 0.9).max(1)` | `TypographyTheme.guides.legend_title` supplies the complete title role; `LegendTheme.spacing.padding` and frame ink supply effective insets. Parent-assigned guide width minus its two effective insets is the positive logical wrap width; the selected text plan supplies actual title height/ink. `spacing.title_gap` supplies the independent present-title separation. | The historical height-based rotated-title budget/cap/ratio/floor is superseded by spec05's assigned-width title measurement already used by the common guide. B2 routes typed roles/insets into this owner and retires any surviving duplicate formula; B4 exposes these consumed existing fields. There is no independently configurable title-height ceiling or hidden `max(1)` rescue. |
| `guides.beeswarm.title-column-reserve`, `guides.heatmap.title-column-reserve`, `guides.beeswarm.title-lane-offset`, `guides.heatmap.title-lane-offset` | Actual `legend_title` text plan plus `spacing.title_gap` and outer padding; one measured title band/baseline in the current guide, reused at selection/capture. | The old rotated title-column offsets/progression are historical attribution, not separate typed title-lane controls. The common measured heading retains its current orientation/placement policy. B2 removes role reconstruction; B4 connects complete text/chrome fields, not raw family-local widths. |
| `guides.beeswarm.maximum-gradient-height` (5550): 150px cap; `guides.heatmap.maximum-gradient-heights` (5626): 160px with missing key, 180px without | **Derived final assignment maximum**, not `gradient_lengths.preferred`. Vertical finite maximum is assigned inner height minus measured title height, active title gap, complete missing-block height and first/last endpoint overhangs. Horizontal maximum is assigned inner width minus first/last overhangs. The theme supplies `gradient_lengths.minimum/preferred`, padding and gaps; it does not supply a hard maximum. | Historical family/missing-presence caps are superseded by the spec05 parent-allocation/common-guide rule, rather than renamed to the soft preferred field. B2 retains that private maximum calculation and its complete fit miss; B4 exposes minimum/preferred only. Record this keyed succession at current-contract/ledger reconciliation without changing historical source evidence or inventing a max setter. |
| `guides.heatmap.missing-swatch-reserve` (5624): fixed 34px subtraction | `TypographyTheme.guides.legend_label`, `LegendTheme.spacing.key_width/row_gap`, and the actual missing-key outline extent from `MarkTheme.cells.missing_key_outline`. The common guide measures one missing row and adds its row separation once. | B2 replaces duplicate literal reserve with the same measured missing-block requirement used by final placement; B4 exposes those already admitted complete roles/gaps. Missing presence remains the prepared descriptor's semantic flag, not a theme policy. |
| `guides.heatmap.missing-swatch-placement-and-size` (5634): clamped bottom `max(bottom-32,3)`, fixed 12×12 square | `spacing.key_width` supplies the positive square side; `spacing.padding/key_label_gap/row_gap` and the measured `legend_label` role supply its row box/spacing. MarkTheme supplies the independent missing fill and key-outline paint/width, not a cell cross or merged-row glyph. The immutable guide arranges this complete row inside its assigned frame. | B2 uses the common measured missing-key box and uncut ink check, with no canvas-origin clamp, detached swatch or constant 12/32/3 reconstruction. B4 exposes the existing key/chrome/paint owners, not another missing-size field. |
| `guides.heatmap.missing-label-placement`, `marks.missing.heatmap-guide-glyph-style`, `guides.heatmap.missing-entry-presence` | `spacing.key_label_gap`, complete `legend_label`, and `cells.missing_fill/missing_key_outline`; private Start/vertical-middle label alignment and exactly one active missing descriptor block. | Retain semantic presence, exact glyph/style and alignment; B2 consumes those typed owners for measurement/draw and B4 connects them publicly. Zero active missing entries produce no missing reserve/box/font work; individual missing cells do not each create a guide row. |
| `guides.beeswarm.endpoint-column-reserve`, `guides.heatmap.endpoint-column-reserve`, both `endpoint-label-placement` keys | Actual `legend_label` plans/endpoint overhangs plus `spacing.key_label_gap/row_gap`; `spacing.key_width` supplies gradient thickness. | B2 consumes final role metrics/gaps and actual numeric-label set, not old family-local reserve/offset constants; B4 exposes those existing theme fields. Required endpoint/center labels remain numeric policy/source responsibility. |
| `guides.beeswarm.outer-reserve-gap`, `guides.heatmap.outer-reserve-gap`, both `panel-gap` keys | `LegendTheme.spacing.outside_gap`, separate from guide-internal title/key/missing gaps | B2 uses one owner-local measured guide-to-body separation; B4 exposes the existing relative spacing, without conflating it with Shared `shared_gap` or missing-row reserve. |

The precise current common-guide missing block is:

```text
k = resolved spacing.key_width
o = retained missing-key outline width / 2 (zero when structurally absent)
g = resolved effective key-label gap, including required outline clearance
missing_row_height = max(measured_missing_label_height, k + o)
missing_block_height = missing_row_height + resolved spacing.row_gap
missing_block_width = k + g + measured_missing_label_fit_width
```

Complete outward key/label/frame ink is checked as well; these operands are not
permission to replace glyph bounds with a font-size estimate. The missing square
has side k and the label shares the row's measured vertical center. The same
block height is used in the maximum-gradient calculation and in the final guide
height/placement. Its absence contributes zero. Local `select` only translates the
retained plan; it cannot recompute a 34px reserve or clamp the swatch above root
origin after its parent accepted a smaller allocation.

Minimum, preference and maximum are distinct. Let R be the complete projected
label/geometry hard requirement including the theme minimum, P the theme's
preferred gradient length, and M the finite derived assignment maximum. If
M < R the guide misses. Otherwise ordinary measurement selects
`max(R, min(P,M))`; compact selects R. An unbounded-height intrinsic call has no
finite hard maximum: its current local surrogate `max(P,R)` is a natural-size
operand, not an invented theme cap or infinity value. A required length greater
than P may succeed when the assigned containing guide fits it. The retired
150/160/180 values never constrain that result or silently become P.

These keyed successors explain changed historical owner formulas under the
approved measured-guide architecture. B2 owns the typed common-consumer handoff;
B4 owns its public component/current-contract migration and corresponding ledger
relationship reconciliation. This document leaves the historical ledger tables
intact and adds no standalone maximum/title-budget/missing-box API, representation
selector or family-specific guide component.

### Composition Construction

```rust,ignore
enum TitleAlignment { Start, Center, End }

impl RootTitleSpacing {
    fn new(title_subtitle_gap: RelativeSpacing,
           title_block_to_content_clearance: RelativeSpacing,
           caption_to_content_clearance: RelativeSpacing) -> Result<Self>;
}
impl RootPadding {
    fn new(left: RelativeSpacing, right: RelativeSpacing,
           bottom: RelativeSpacing, top: RelativeSpacing) -> Result<Self>;
}
impl TextPanelSpacing {
    fn new(inset: RelativeSpacing, gap: RelativeSpacing) -> Result<Self>;
}
impl HeaderSpacing {
    fn new(column_header_padding: RelativeSpacing, row_header_padding: RelativeSpacing,
           row_title_horizontal_padding: RelativeSpacing,
           row_title_vertical_padding: RelativeSpacing) -> Result<Self>;
}
impl FacetStripSpacing {
    fn new(horizontal_inset: RelativeSpacing,
           content_clearance: RelativeSpacing) -> Result<Self>;
}
impl CompositionTheme {
    fn new(root_title_alignment: TitleAlignment,
           root_subtitle_alignment: Option<TitleAlignment>,
           root_caption_alignment: TitleAlignment,
           root_title_block: RootTitleSpacing, root_padding: RootPadding,
           panel_edge_clearance: RelativeSpacing, text_panel: TextPanelSpacing,
           facet_gap: RelativeSpacing, canvas_grid_gap: RelativeSpacing,
           headers: HeaderSpacing, facet_strip: FacetStripSpacing) -> Result<Self>;
}
```

All spacing relationships are nonnegative. All **12 current CompositionAppearance
fields**, the three RootTitleBlockAppearance fields, and the admitted remaining
facet-strip relationships have these single owners:

| Current field/source | Target path | Exact built-in relationship | Local metric / rounding |
| --- | --- | --- | --- |
| `root_title_alignment` | same name | Center | closed physical alignment |
| `root_subtitle_alignment` | same name | None | independent optional instruction |
| `root_caption_alignment` | same name | Start | independent physical alignment |
| `root_title_block.title_subtitle_gap` | same nested field | 0/1 | max(E(title),E(subtitle)); S/H; only adjacent present roles |
| `root_title_block.title_block_to_content_clearance` | same nested field | 14/max(T,S) | max(E(title),E(subtitle)); S/H; top block presence |
| `root_title_block.caption_to_content_clearance` | same nested field | 12/C | E(caption); S/H; caption presence |
| `layout/canvas.rs::ROOT_PADDING.left` and embedded prose `+16` | `root_padding.left` | 16/A | E(body) of owner; per left edge, S/H |
| `ROOT_PADDING.right` and embedded prose `-16` | `root_padding.right` | 16/A | E(body) of owner; per right edge, S/H |
| `ROOT_PADDING.bottom` and embedded prose `+8` | `root_padding.bottom` | 8/A | E(body) of owner; per bottom edge, S/H |
| `ROOT_PADDING.top` and embedded prose `-12` | `root_padding.top` | 12/A | E(body) of owner; per top edge, S/H |
| `PanelRequirements::measure_header` empty-top and axis-edge additions 4 | `panel_edge_clearance` | 4/16 | E(tick_label) of Plot owner; S/H; one comfort addition per exposed measured axis edge |
| `text_panel_inset` | `text_panel.inset` | 16/A | E(body); S/H; per edge, current identical horizontal/vertical inset |
| `text_panel_gap` | `text_panel.gap` | 12/max(T,A) | max(E(text_panel_title),E(body)); S/H |
| `facet_gap` | same name | 12/S | E(facet_header); S/H; declared topology |
| `canvas_grid_gap` | same name | 12/A | E(body) of the owner; S/H; Canvas/Matrix/Marginal use the same field and metric |
| `column_header_padding` | `headers.column_header_padding` | 16/T | E(composition_header); S/H |
| `row_header_padding` | `headers.row_header_padding` | 20/T | E(composition_header); S/H |
| `row_title_horizontal_padding` | `headers.row_title_horizontal_padding` | 16/T | E(composition_header); S/H; total span subtraction once |
| `row_title_vertical_padding` | `headers.row_title_vertical_padding` | 8/T | E(composition_header); S/H; one header-to-body reserve |
| Facet width `-16` and intrinsic `+16` | `facet_strip.horizontal_inset` | 16/S | E(facet_header); S/H; total span inset once |
| Facet's independent strip/content offsets | `facet_strip.content_clearance` | 8/S | E(facet_header); S/H; text/strip occupancy adds actual selected role metrics |

Role metrics for the top block are defined from its complete title/subtitle roles
even if only one of those contents is present. Thus the built-in 14px clearance
does not change merely because title or subtitle is omitted; absence removes the
whole adjacency. Gap resolution does not use wrapped line count or root height.
Caption clearance likewise resolves to 12px in each preset despite caption sizes.

`canvas_grid_gap`'s one local metric is intentionally the owner's body em in all
consumers, not whichever adjacent child happens to be rendered first. Composed
children cannot alter Canvas-owned gaps. This gap metric is **not** the declared
track preference metric below. Declared empty tracks retain topology and their
specified owner-local preference; gaps do not compact around them.

#### Exact Declared-Track Em Owner, Natural Requirements And Identity

Spec05's declared Canvas/facet Grid rule and ledger
`layout.composition.deterministic-track-allocation` preserve declared empty and
trailing tracks. Current `canvas/tracks.rs` and
`orchestrator/facets/tracks.rs::allocate` pass `Some(em)` on **both** axes to
`layout/kernel/tracks.rs::allocate_grid_axis`. The source em readings have these
single typed successors:

| Grid owner / current scalar | Exact target metric | Default Publication / Nature / Science / CHI |
| --- | --- | --- |
| Canvas root: `composition_title_font_size` | `E(Canvas typography.composition.composition_header)` = checked `composition_header.font_size() * 1024` quanta | 20 / 19 / 21 / 19 CSS pixels |
| Facets: owning Plot's `facet_header_font_size` | `E(Plot typography.composition.facet_header)` = checked `facet_header.font_size() * 1024` quanta | 15 / 14 / 16 / 15 CSS pixels |

Each declared track is flexible under the existing declared-track exception,
with hard minimum at least **one quantum** and preferred minimum its owner's
**one font-size em**. For an otherwise unoccupied track with no larger traced
requirement the exact pair is `(hard=1, preferred=em)` quanta. Occupied or spanning
child requirements may raise those values through spec05's same deterministic
hard/preferred allocation. No leading multiplier, measured header glyph, body
size, TextPanel title size, first child's style or root dimensions replace em.
The owning role supplies it even when no header/strip content is painted; header
presence does not collapse a declared track or select a fallback role.

For a natural inner Grid dimension with n declared tracks and resolved gap qg,
the track contribution is exactly
`sum(final_preferred_track_quanta) + (n-1) * qg`, in checked spatial arithmetic.
Measured gutters, outer/header bands and insets are added at their actual parent
owners exactly once. Columns are allocated before width-dependent children/rows;
the owning natural-width choice is not restarted. At exact allocation the same
hard/preferred vector feeds the existing proportional shrink/surplus rule; one
em is a preference, not an exact root minimum. Empty-slot children remain zero-size
and sparse/trailing addresses are never repacked. A nondeclared unoccupied Grid
track does not gain this exception or a new em floor.

B2 migrates the legacy scalar reads in **both** natural summary and constrained
allocation paths to these typed roles. B4 exposes those already active typography
roles; no separate `declared_track_em` field or public track control is added.
The prepared intrinsic/track and measured-cache/feedback identities retain owner
scope, declared shape/addresses, the exact consumed em quanta, axis, gap/insets,
child hard/preferred/span/fill requirements, assigned constraints, scoped domains,
and consumed gutter/owned-ink/content-height feedback. Equal topology/width without
the same em is insufficient for reuse. Actual header/text sources and full used
typography/font provenance additionally key their own measurements when present.

Changing `composition.body` can legitimately change its separately consumed grid
gap, padding or TextPanel requirements, but **cannot change Canvas's declared em**.
Changing TextPanel title likewise cannot change it. Changing `composition_header`
font size must change that em without requiring a visible header. Facet declared
em changes only with the owning `facet_header` font size; Canvas header, body and
TextPanel roles are not substitutes. Cache fixtures isolate this rule with fixed
gap/padding inputs and no affected prose/TextPanel sources, asserting the empty
track hard/preferred pair and invalidation on the actual owner metric, rather
than falsely promising that unrelated body edits never affect overall root size.

#### Root, Panel And Header Ledger Trace

The overlay says existing-key targets retain their owner unless explicitly
overridden. Its retirement of universal minima does **not** retire every current
comfort inset. The following trace distinguishes theme values from private
aggregation and the corrected common Plot title-block representation:

| Ledger key(s) | Active target and current consumer | Authoritative construction/relationship |
| --- | --- | --- |
| `composition.outer.current-fixed-inset-aggregate` (3461) | Theme component, CompositionTheme; current `ROOT_PADDING` is left/right 16, bottom 8, top 12. No explicit overlay override retires these four current chrome inputs. | RootPadding's four local relationships above; remove the constant as an appearance source at B2. This is chrome, not a caller root-size request or universal panel minimum. |
| `composition.outer.current-wrapping-width-budget` (3458), `composition.outer.current-shared-axis-inset-aggregate` (3462) | Renderer invariants; measured containing span and complete shared-axis requirements | Compute width minus resolved left+right padding, then add actual shared-axis/text requirements once; no second absolute inset aggregate. |
| `layout.canvas.outer-inset-floors` | Historical standalone Renderer-invariant aggregate, with old 62/30/top-30 floors; current common root no longer implements that historical floor formula | Keep the private frame/axis allocation responsibility; do not revive the old floors as extra theme fields. Current common-root appearance operands use RootPadding from the composed-chrome owner, while traced actual axes/ink determine further requirements under spec03/spec05. |
| `canvas.panel-header.{title,subtitle}.current-left-inset` (3432/3436), `composition.panel-header.current-wrap-inset` (3440) | Theme-owned padding responsibility; current embedded prose uses `canvas/plot_text.rs::PlotLocalTextPlan::measure`'s per-edge 16, not the historical independently drawn +8 anchor | Embedded Plot prose consumes its resolved Plot RootPadding left/right and common root span. The still-live facet width subtraction of total 16 consumes `facet_strip.horizontal_inset`; do not confuse total 16 with two per-edge 16 values. |
| `canvas.panel-header.{title,subtitle}.current-baseline-offset` (3433/3434), `composition.panel-header.current-title-subtitle-gap` (3439) and `current-bottom-clearance` (3446) | Theme-owned relationship sources; current embedded prose is already one RootTitleBlockPlan with title/subtitle/caption, not the former two separately drawn header roles | Derived baselines/occupancy come from the complete text plan, `root_title_block.title_subtitle_gap` (current 0) and `title_block_to_content_clearance` (current 14). Facet strip content clearance has its existing distinct 8px-equivalent owner. The old 4px `PANEL_HEADER_GAP` branch with supplied headings is not a second runtime title block or public component. |
| `composition.panel-header.empty-top-reserve` (3444), y-title/y-tick overhang maxima (3447/3448) | Theme-owned empty-edge comfort plus Renderer-invariant aggregation; current production Canvas probes/assigned panels call `measure_header` with both heading inputs None | `panel_edge_clearance` supplies 4px-equivalent comfort. For the no-heading path: left/right/bottom are measured axis requirement + clearance; top is max(clearance, measured top + clearance). Facet strip occupancy participates by maximum. Remove the obsolete supplied-heading/count branch at B2; actual Plot prose is measured separately. |
| `composition.outer-title.current-top-gap` (3459), `layout.canvas.title-subtitle-top-gap` | Theme component; overlay explicitly moves the latter to CompositionTheme | One `root_title_block.title_block_to_content_clearance`, not another hardcoded 14. |
| `composition.outer-caption.current-extra-reserve` (3460), `layout.canvas.caption-extra-reserve` | Theme component; overlay explicitly moves the latter to CompositionTheme | One `root_title_block.caption_to_content_clearance`, not another hardcoded 12. |
| `canvas.text-panel.current-horizontal-inset` (3391), `current-vertical-inset` (3395) | Theme components retained by the overlay; current TextPanel uses one 16px-equivalent inset per edge | `text_panel.inset` is the single current relationship used independently at each horizontal/vertical edge; it is separate from RootPadding despite initially equal values. |
| `canvas.text-panel.current-title-body-gap` (3394), `current-draw-gap` (3419) | Theme components with duplicate planning/drawing source | One `text_panel.gap`, current 12px-equivalent; consume the measured sequence gap once. |
| `validation.canvas.minimum-size`, `composition.grid.minimum-panel-fit`, `layout.panel.minimum-{width,height}` | Explicit overlay retirements of universal minima | Content/resource fit only; none of these retirements removes the preceding theme-owned padding or turns it into invariant literals. |

B2 routes `RootFramePlan::measure`, both `canvas/sizing.rs` preference/content
paths and `orchestrator/facets/sizing.rs` through resolved owner RootPadding.
`canvas/plot_text.rs` uses those same four fields from the embedded Plot theme
when its local prose is present; when absent its current span is the whole cell.
Preserve the current embedded distinction: horizontal padding insets the prose
span, while the returned scientific body retains the cell's full horizontal span;
top/bottom prose padding and measured reserves constrain its vertical body.
Canvas-owned root padding never replaces an explicit child's Plot-owned padding.
These are one component's chrome relationships, not extra alignment or sizing APIs.

Historical named sources disappear only through the explicit relationships above:
the current root block replaces dead separate-heading reconstruction; font/axis
ink supplies measurements; padding/clearance remains owned by a typed component.
This introduces no dormant public panel-heading block or per-header alignment.

#### Padding Cardinality: Total Span Versus Per Edge

| Relationship | Current application to preserve |
| --- | --- |
| RootPadding left/right 16, bottom 8, top 12 | Each named edge applies once; root text width subtracts left+right = 32px-equivalent. Embedded presence rule is as above. |
| TextPanel inset 16 | Once at each edge: total horizontal and vertical inset are each 32px-equivalent. |
| `headers.row_header_padding` 20 | One addition to the measured row-header band width; not 20 on both sides. |
| `headers.column_header_padding` 16 | One addition to the tallest measured column-header band height. |
| `headers.row_title_horizontal_padding` 16 | One total-span subtraction in wrap width and one matching addition in hard/preferred span requirement; not 32. |
| `headers.row_title_vertical_padding` 8 | One addition to each present row-title reserve, consumed by its same measured placement. |
| `facet_strip.horizontal_inset` 16 | One total-span subtraction/addition; not a new per-edge 16. |
| Legend glyph-edge comfort | Once at each outer vertical edge, alongside ordinary padding; never repeated per entry/section. |

At B4 all of this component, TitleAlignment and caption alignment are already
consumed by themes; none is a dormant exported placeholder. B5 adds exactly:

```rust,ignore
impl<K: kind::Marker> Plot<K> {
    fn title_alignment(self, alignment: TitleAlignment) -> Self;
    fn subtitle_alignment(self, alignment: TitleAlignment) -> Self;
}
impl Canvas {
    fn panel_title_alignment(self, alignment: TitleAlignment) -> Self;
    fn panel_subtitle_alignment(self, alignment: TitleAlignment) -> Self;
}
```

These store independent optional instructions, repeated calls replace their own
input, and theme attachment order does not alter them. Resolve title as Plot >
Canvas panel instruction > Plot theme. Resolve subtitle as Plot > Canvas panel
instruction > optional Plot theme subtitle > final effective title. `None`
contributes no instruction. Caption independently uses its theme field. Canvas
root prose uses only Canvas theme fields. Physical Start/End, line alignment,
block placement, excluded headers/TextPanel and frozen-plan rules remain spec03.

## 8. Complete Annotation Construction

```rust,ignore
impl SignificanceGeometry {
    fn new(bracket_stroke: SolidStrokeTheme, label_gap: RelativeSpacing,
           tick_height: RelativeSpacing, data_gap: RelativeSpacing,
           level_step: RelativeSpacing, panel_padding: RelativeSpacing,
           attached_extra_clearance: RelativeSpacing) -> Result<Self>;
}
impl CellAnnotationGeometry {
    fn new(highlight_outline: OutlineTheme,
           value_horizontal_padding: RelativeSpacing,
           value_vertical_padding: RelativeSpacing,
           null_row_horizontal_padding: RelativeSpacing,
           null_row_vertical_padding: RelativeSpacing) -> Result<Self>;
}
impl EstimateAnnotationGeometry {
    fn new(panel_inset: RelativeSpacing, guide_clearance: RelativeSpacing,
           numeric_label_gap: RelativeSpacing) -> Result<Self>;
}
impl ValueShelfSpacing {
    fn new(padding: RelativeSpacing, gap: RelativeSpacing,
           row_gap: RelativeSpacing) -> Result<Self>;
}
impl AnnotationTheme {
    fn new(significance: SignificanceGeometry, cells: CellAnnotationGeometry,
           estimates: EstimateAnnotationGeometry, point_label_gap: RelativeSpacing,
           panel_text_padding: RelativeSpacing, waterfall_shelf: ValueShelfSpacing,
           planning_backdrop: OpaqueColor) -> Result<Self>;
}
```

All relationships are nonnegative. Backdrop is opaque by type, calculation-only
and never painted. Bracket/highlight stroke widths are positive. Text appearance
is exclusively TypographyTheme; target/placement/content/statistics are not
component fields. Existing highlight outline moves from MarkAppearance rather
than becoming a second source.

All six existing AnnotationAppearance fields and BracketStrokeAppearance map
below, together with already admitted annotation defaults still in consuming
helpers. Padding in the table means the **total** horizontal/vertical text-fit
addition unless explicitly marked per edge; do not halve or double it at migration.

| Current field/source | Target path | Exact built-in ratio/paint | Local metric / rounding |
| --- | --- | --- | --- |
| `bracket_stroke.{paint,width}` | `significance.bracket_stroke.{color,width}` | F / preset bracket width | S stroke and outward ink |
| absent exact significance gap, currently DEFAULT 8 | `significance.label_gap` | 8/16 | E(effective significance_label); S/H |
| `SIGNIFICANCE_BRACKET_GEOMETRY.tick_height` 7 | `significance.tick_height` | 7/16 | E(effective significance_label); S/H |
| same `data_gap` 6 | `significance.data_gap` | 6/16 | E(effective significance_label); S/H plus actual mark/stroke ink |
| same `level_step` 26 | `significance.level_step` | 26/16 | E(effective significance_label); S/H plus complete bracket/text collision requirements |
| same `panel_padding` 2 | `significance.panel_padding` | 2/16 | E(effective significance_label); S/H; actual half-stroke+quantum minimum remains invariant |
| vertical/horizontal Attached safety addition 2 | `significance.attached_extra_clearance` | 2/16 | E(effective significance_label); S/H; shared orientation role |
| `point_label_gap` 2 | same name | 2/10 | E(point_layer_label); S/H for the nominal gap; actual placement retains radius+gap and whole-pixel snapping/vertical-middle origin (§11), not a newly defined minimum painted-edge gap |
| `PANEL_TEXT_PADDING` 8 | `panel_text_padding` | 8/14 | E(panel_text); per edge, S/H for placement/complete fit |
| `waterfall_shelf_padding` 4 | `waterfall_shelf.padding` | 4/14 | E(waterfall_total); per edge, S/H |
| `waterfall_shelf_gap` 4 | `waterfall_shelf.gap` | 4/14 | E(waterfall_total); S/H |
| `waterfall_shelf_row_gap` 4 | `waterfall_shelf.row_gap` | 4/14 | E(waterfall_total); S/H |
| `planning_backdrop` white RGBA | same name, OpaqueColor | white | contrast calculation only |
| current highlight outline | `cells.highlight_outline` | F / 2.5f32 | independent complete annotation outline |
| Heatmap cell fit/wrap `+6`/`-6` | `cells.value_horizontal_padding` | 6/11 | E(selected contrast role); total, H for fit / S placement |
| Heatmap cell vertical fit `+5` | `cells.value_vertical_padding` | 5/11 | E(selected contrast role); total, H |
| fully-null row `+12`/`-12` | `cells.null_row_horizontal_padding` | 12/11 | E(selected contrast role); total, S/H |
| fully-null row vertical `+4` | `cells.null_row_vertical_padding` | 4/11 | E(selected contrast role); total, H |
| unavailable estimate `PANEL_PADDING` 6 | `estimates.panel_inset` | **6/14** | E(estimate_detail); per edge, S/H |
| unavailable estimate `GUIDE_CLEARANCE` 6 | `estimates.guide_clearance` | **6/14** | E(estimate_detail); S/H plus actual reference ink |
| Coefficient adjacent numeric gap 7 | `estimates.numeric_label_gap` | 7/14 | E(estimate_detail); S/H; preserve current center-relative gap semantics |

B2 routes every intrinsic, preparation, fit/collision, scoped copy and final
operand through these same resolved values. The actual current Heatmap source
uses two distinct, already matching vertical additions, rather than one generic
row-summary padding:

| Prepared source membership | Current sizing/planning consumers | Current final consumer | B2 single authority |
| --- | --- | --- | --- |
| `annotations.values` / prepared finite cell values, indexed by cell and then row | `assigned_rows`, `natural_body_height` and `intrinsic_body_minimum -> analyze_intrinsics`: `plan.height() + 5.0` | `check_annotation_tracks` value-cell check: height >= plan.height() + 5.0 | `cells.value_vertical_padding`, exact default 5/11 × selected contrast-role em |
| `annotations.rows` / fully-null merged-row annotation summaries, indexed by semantic row | the same sizing paths: `plan.height() + 4.0`; `analyze_intrinsics` null-row summaries use `unwrapped.height() + 4.0` | `check_annotation_tracks` merged-row check: height >= plan.height() + 4.0 | `cells.null_row_vertical_padding`, exact default 4/11 × selected contrast-role em |

The `+4` is not a smaller intrinsic promise for a value cell's final `+5`: these
are different prepared source collections. Correct the earlier mismatch claim,
preserve each collection's actual source count and semantics, and migrate both
sets of literal consumers at B2. If an affected branch is found to apply the wrong
field, correct that branch to its collection's authority with explicit B2
complete-ink evidence; do not globally replace every `+4` with `+5`. Horizontal
value padding remains total 6; merged-row horizontal padding remains total 12.

Significance absence is an actual `None` exact override. An absent style uses the
theme's complete text role and **theme-owned 8px-equivalent label gap**; a supplied
style replaces size/weight/gap only. Delete the private accessor that materializes
`SignificanceMarkStyle::DEFAULT` when absent. One private effective significance
value retains selected text, exact-or-theme gap, bracket/geometry and provenance
for bars, Coefficient, comparisons, font census, reserves and final capture.
No backend `SIGNIFICANCE_BRACKET_GEOMETRY` default can survive as a separate
planner/draw authority.

### Significance Geometry: Planner Through Backend Validation And Draw

The ledger's `annotations.significance.{tick-height-default,data-gap-default,
level-step-default,panel-padding-default}`, `backend.bracket.geometry-carrier`,
`backend.bracket.label-gap-carrier`, and final geometry/style records are one
owner trace, not permission to change only the significance planner.

| Current geometry operand | Actual current consumers | B2 handoff |
| --- | --- | --- |
| `SIGNIFICANCE_BRACKET_GEOMETRY.tick_height = 7` | `significance.rs::bracket_metrics` computes path/collision requirements; **`render/backend.rs::validate_bracket` separately reads the same global constant** and passes it to `render/coordinates.rs::canonical_bracket` | Retain the resolved theme tick height in the final private bracket geometry/style operand. Both planner and canonical backend validation must consume that retained value. |
| `data_gap = 6` | bracket-to-mark placement/range/headroom/collision planning via BracketMetrics | Resolve once from SignificanceGeometry and retain its planning consequence and dependency identity; drawing consumes the resulting path, not a new data-gap constant. |
| `level_step = 26` | comparison-level packing and occupied bracket/text bounds | Same selected geometry authority; complete collision fit may require more actual occupancy, but no backend independent level-step policy. |
| `panel_padding = 2` | significance containing-edge fit, including actual half-stroke+one-quantum requirements | Same resolved padding and frozen containment result; no backend global padding reconstruction. |
| exact-or-theme `label_gap` | planner, BracketStyle, `canonical_bracket` label position, and planned-label equality | Same retained effective gap in planning/label placement/validation; preserve absent/exact scope. |

Canonical consumers include `render/primitives.rs::CanonicalPlannedBracket::new`,
`render/geometry.rs::{GeometrySnapshot::bracket,draw_bracket,draw_planned_bracket}`,
`render/svg_backend.rs::{draw_bracket,draw_planned_bracket,draw_canonical_bracket}`
and the orchestrator's validating/capture/replay backend. B2 must either capture
the canonical path/label placement already constructed with the resolved geometry,
or pass its retained tick-height operand into the shared validator before that
capture; the one frozen canonical representation then serves every replay path.
No draw/validation method may quietly fall back to 7 after a custom tick-height
edit. The actual backend reread today is tick height, not all four planning fields;
data-gap/level/panel operands reach draw through their frozen planning consequences.
Private backend fixtures migrate their required operands along with the planner.
All malformed/representability checks and complete text-plan equality remain.

## 9. Palette, Map, Gradient And Scale Construction

### Categorical And Accent Values

```rust,ignore
enum PaletteCapacity { Fixed, Extendable }

impl QualitativePalette {
    fn fixed(colors: impl IntoIterator<Item = OpaqueColor>) -> Result<Self>;
    fn extendable(seeds: impl IntoIterator<Item = OpaqueColor>) -> Result<Self>;
    fn colors(&self) -> &[OpaqueColor];
    fn capacity(&self) -> PaletteCapacity;
}
impl AccentPalette {
    fn new(seeds: impl IntoIterator<Item = OpaqueColor>) -> Result<Self>;
    fn seeds(&self) -> &[OpaqueColor];
}
impl CategoricalColorMap {
    fn new(entries: impl IntoIterator<Item = (String, OpaqueColor)>) -> Result<Self>;
    fn entries(&self) -> impl ExactSizeIterator<Item = (&str, OpaqueColor)> + '_;
    fn color(&self, raw_identity: &str) -> Option<OpaqueColor>;
}
impl CategoricalColorScale {
    fn from_palette(palette: QualitativePalette) -> Self;
    fn from_map(map: CategoricalColorMap) -> Self;
    fn palette(&self) -> Option<&QualitativePalette>;
    fn map(&self) -> Option<&CategoricalColorMap>;
}
```

Palette/map/accent and scale-choice storage is private. Every color sequence and
map is nonempty. Fixed preflight checks required/available capacity; extendable
and accents use the current first-party OKLab/OKLCH algorithm without cycling.
Supply prefixes are exact, stable and independent of requested final count.
Repeated colors are permitted. Constructor iteration/allocation is checked and
fallible; render-time extension retains the operation-local palette cache and
checked fallible storage, with no work/storage ledger under spec95.

Map keys are the actual consumer's normalized semantic raw identities. Construction
requires original input keys to be nonempty, already trimmed, control-free and
unique by UTF-8 bytes. It does not silently trim, collapse interior whitespace,
casefold or Unicode-normalize ordinary CategoryValues identities, which currently
retain their bytes. A consumer with an existing normalization contract supplies
those normalized keys; display aliases/humanization are never keys. Preserve input
order for inspection; admission lookups need not use that order as an algorithm.
Missing used identity rejects before fonts; unused entries and repeated colors
are valid. `color` performs exact admitted-key lookup, not alias lookup or
on-demand normalization of display text.

Mark accents are an independent always-extendable value, not extra slots in an
exact categorical map. An unkeyed primary consumes slot 0; the first appended
PointLayer consumes slot 1. A categorically mapped primary consumes no accent
slot, so its first layer uses slot 0. Every appended layer consumes its position
even when its final paint is exact. Signed bars/Waterfall consume the existing
stable sign slots, not a newly invented categorical map.

### Ordered Opaque Gradients

```rust,ignore
impl ColorStop {
    fn new(offset: f64, color: OpaqueColor) -> Result<Self>;
    fn offset(&self) -> f64;
    fn color(&self) -> OpaqueColor;
}
impl SequentialGradient {
    fn new(stops: impl IntoIterator<Item = ColorStop>) -> Result<Self>;
    fn stops(&self) -> &[ColorStop];
}
impl DivergingGradient {
    fn new(stops: impl IntoIterator<Item = ColorStop>) -> Result<Self>;
    fn stops(&self) -> &[ColorStop];
    fn neutral(&self) -> OpaqueColor;
}
impl ColorScales {
    fn new(qualitative: QualitativePalette, sequential: SequentialGradient,
           diverging: DivergingGradient) -> Result<Self>;
}
```

ColorStop has private fields, finite normalized offset in `[0,1]` and opaque color.
Sequences are supplied in strictly increasing order: no sorting, repair, repeated
offset, epsilon coalescing or transparent stop. Sequential has at least two stops
and exact endpoints 0 and 1. Diverging has at least three, exact endpoints and
exactly one explicit neutral at **0.5**, with at least one stop on each side.
Extra ordered stops on either side are permitted. There is no new arbitrary stop
count cap: unknown-length construction and preflight storage grow with real
length checks and fallible reservations. This is a general finite ordered
input, not public backend GradientStop or raw `(f64, RGB)` variants.

`neutral()` is derived from the unique 0.5 stop, not separately editable storage.
Whole gradients reconstruct through `new` or replace a parent's validated field;
there is no mutable slice or public callback/interpolator. Exact stop offsets and
colors participate in normalized equality. First-party OKLab interpolation and
deterministic gamut handling remain `theme/color.rs`.

The current diverging default preserves the three semantic stops:
`0: (49,130,189)`, `0.5: (247,247,247)`, `1: (222,45,38)`.
The **approved unified sequential default** in every built-in preserves Heatmap's
two stops `0: #F7FBFF (247,251,255)` and `1: #08306B (8,48,107)` and also serves
Beeswarm. The old Beeswarm `#3182BD` to `#DE2D26` default is retired (§13).
Qualitative and accent initial sequences independently preserve all curated colors in
`publication_theme`, `nature_theme`, `science_theme`, `chi_theme`; no generic
foreground/neutral reference is taken from a scale slot.

| Preset | Exact independent qualitative/Mark accent seed prefix, in order |
| --- | --- |
| Publication | `(0,114,178)`, `(213,94,0)`, `(0,158,115)`, `(204,121,167)`, `(86,180,233)`, `(230,159,0)`, `(240,228,66)`, `(0,0,0)` |
| Nature | `(40,86,130)`, `(130,160,188)`, `(200,215,230)`, `(120,130,90)`, `(180,175,130)` |
| Science | `(255,0,0)`, `(0,0,255)`, `(0,175,0)`, `(255,175,0)`, `(160,32,240)` |
| CHI | `(171,199,232)`, `(243,169,175)`, `(174,224,174)`, `(244,213,152)`, `(197,176,213)` |

Each built-in qualitative palette is extendable. Its initially equal AccentPalette
is separately owned; later edits cannot couple their extension/capacity domains.

### One Complete Continuous Selection And Domain Authority

```rust,ignore
enum ContinuousScaleKind { Sequential, Diverging }

impl ColorDomain {
    fn exact(minimum: f64, maximum: f64) -> Result<Self>;
    fn minimum(&self) -> f64;
    fn maximum(&self) -> f64;
}
impl ContinuousColorScale {
    fn sequential(gradient: Option<SequentialGradient>,
                  domain: Option<ColorDomain>) -> Result<Self>;
    fn diverging(center: f64, gradient: Option<DivergingGradient>,
                 domain: Option<ColorDomain>) -> Result<Self>;
    fn kind(&self) -> ContinuousScaleKind;
    fn center(&self) -> Option<f64>;
    fn domain(&self) -> Option<&ColorDomain>;
    fn sequential_gradient(&self) -> Option<&SequentialGradient>;
    fn diverging_gradient(&self) -> Option<&DivergingGradient>;
    fn with_domain(self, domain: Option<ColorDomain>) -> Result<Self>;
    fn with_sequential_gradient(self, gradient: Option<SequentialGradient>) -> Result<Self>;
    fn with_diverging_gradient(self, gradient: Option<DivergingGradient>) -> Result<Self>;
    fn with_center(self, center: f64) -> Result<Self>;
}
```

ColorDomain and ContinuousColorScale have private fields/choice payloads.
ColorDomain requires finite **maximum >= minimum**, including a constant domain.
Both normalize signed zero. A diverging center is finite; a supplied exact domain
must contain it inclusively. A constant diverging domain therefore equals the
center. `None` domain means automatic from prepared finite active values; `None`
gradient means the complete owner's matching ColorScales default. Wrong-kind
gradient/center edits reject immediately; they are never inert settings or
implicit kind switches. Construct a different complete kind to change kind.

Defaults/semantics are identical on Heatmap and Beeswarm:

1. Unconfigured selection is sequential, automatic domain, theme sequential
   gradient. Heatmap keeps active/null exclusion; Beeswarm retains its current
   strict finite numeric color input. A configured diverging Beeswarm is accepted
   and fully consumed, including center, neutral, domain, marks and three-label
   guide; no Heatmap-only divergence dispatch remains.
2. Automatic sequential domain is observed finite minimum/maximum. Constant
   domain paints the gradient's semantic midpoint and has one required label.
3. Automatic diverging domain uses the current finite symmetric-around-center
   calculation, including constant-at-center. Unrepresentable symmetry rejects
   before fonts with the existing explicit-domain remedy. Exact limits are exact
   and saturate outside values; they need not contain every local observation.
4. Use one final-domain stop remapping for both marks and guides. For an interior
   center, let `c = normalized_fraction(final_domain, center)` using the current
   overflow-safe arithmetic. Preserve exact endpoint/neutral cases `u=0`, `0.5`,
   `1` as `0`, `c`, `1`. For an additional semantic stop `u<0.5`, calculate
   `(u*2.0)*c`; for `u>0.5`, calculate
   `c + ((u-0.5)*2.0)*(1.0-c)` in that named binary64 order without FMA. Evaluate
   noncenter mark values through current `ContinuousColors` normalized-fraction,
   neighboring-stop/local-fraction and OKLab arithmetic over that complete
   remapped sequence. The exact center directly returns neutral. Thus the
   three-stop default retains its current noncenter color operation order.
5. An endpoint center consumes only the noncollapsed branch, mapping its semantic
   half to `[0,1]`; constant-at-center consumes neutral. All extra branch stops
   participate. Non-strict or nonfinite remapped offsets reject before fonts;
   they are not repaired/coalesced. Canonical guide serialization collision of
   distinct colors rejects rather than dropping a stop or changing mark colors.
   Preserve current retained observed-color stop sampling and its six-decimal
   offset refusal rule. No `semantic.len() == 3` consumer restriction remains.
6. Required numeric labels remain domain endpoints plus an interior center,
   at most three, regardless of gradient stop count. Continuous gradient stops
   are not new required numeric labels or discrete sections.

B2 migrates `theme/scale.rs::ContinuousColors`, Heatmap's color-domain/gradient
helpers, Beeswarm preparation/guide arrays and Shared-domain installation,
`decorations/legend/descriptor.rs::{final_stops,ContinuousContributor}`, and
continuous guide/contrast preparation to this single private typed authority.
Beeswarm's two-string array and hardcoded `center: None` are removed. Heatmap's
three-stop-only assumptions cannot remain behind the broader public constructor.
Final mark fill alpha comes from the cell/point role. Numeric guide stops describe
the opaque scale's chromatic mapping and remain opaque, preserving the current
Beeswarm distinction between point alpha 209 and gradient-stop alpha 255. Their
sampled RGB must match the mark's scale-derived RGB; they do not borrow point alpha
or introduce data-mapped alpha. Automatic Heatmap label contrast, in contrast,
consumes the actual RGBA cell fill and surfaces/backdrop before selecting text.

Shared compatibility retains exact raw field/title, kind, center, entire semantic
gradient, missing block and effective presentation policy. Automatic envelopes
union; all exact domains must agree and contain every automatic envelope. Install
the one final domain before label generation, color/contrast/font preparation and
capture. Recompute branch remapping and observed-color sampling from that final
domain, not from a previously projected local gradient. Facets use the complete
parent's prepared domain before slicing. Backend interpolation is not authority.
Discrete mark-bearing guide glyphs still retain each producer's complete exact
RGBA style; the numeric gradient's opaque scale identity is a different consumer.

### Actual Public Scale Attachments

```rust,ignore
impl<K: kind::CategoricalScale> Plot<K> {
    fn categorical_color_scale(self, scale: CategoricalColorScale) -> Self;
}
impl<K: kind::ContinuousColor> Plot<K> {
    fn continuous_color_scale(self, scale: ContinuousColorScale) -> Self;
}
```

Each setter replaces its entire one stored value. `kind::CategoricalScale` has
exactly Scatter, Line, Trajectory, LinePath, Density, Bar, HorizontalBar,
StackedBar, StackedHorizontalBar, Boxplot, Interaction, Violin, Raincloud,
Marginal and Matrix. Rename the actual capability and its impls/constraints;
delete `kind::CategoricalColors`, `CategoricalColors`, `categorical_colors` and
the old map storage. Do not narrow to Hue or extend to other accent-only families.
ContinuousColor remains exactly Heatmap and Beeswarm. Neither gate applies to Any.

B4 deletes Heatmap `.scale(HeatmapScale)`, both families' `.color_limits`,
`heatmap_scale`, both legacy color-limit fields, `heatmap_gradient`,
`beeswarm_gradient`, test-only raw gradient setters and their getters/scoped copies.
Their replacements are one typed scale field and consuming private accessor/copy.
Migrate all scientific producer, facet, Canvas, unit/integration/example/doc
callers at that cutover. Forest `.scale(AxisScale)` and `AxisScale::{Linear,Log10}`
remain their existing scientific axis control and are not renamed or deleted.

## 10. Complete Theme And Construction-Only Builder

```rust,ignore
impl Theme {
    fn new(surfaces: SurfaceTheme, typography: TypographyTheme, axes: AxisTheme,
           legend: LegendTheme, composition: CompositionTheme, marks: MarkTheme,
           annotations: AnnotationTheme, color_scales: ColorScales) -> Result<Self>;
    fn publication() -> Self;
    fn nature() -> Self;
    fn science() -> Self;
    fn chi() -> Self;
    fn surfaces(&self) -> &SurfaceTheme;
    fn typography(&self) -> &TypographyTheme;
    fn axes(&self) -> &AxisTheme;
    fn legend(&self) -> &LegendTheme;
    fn composition(&self) -> &CompositionTheme;
    fn marks(&self) -> &MarkTheme;
    fn annotations(&self) -> &AnnotationTheme;
    fn color_scales(&self) -> &ColorScales;
    fn customize(&self) -> ThemeBuilder;
}
impl ThemeBuilder {
    fn new(theme: Theme) -> Self;
    fn with_surfaces(self, value: SurfaceTheme) -> Self;
    fn with_typography(self, value: TypographyTheme) -> Self;
    fn with_axes(self, value: AxisTheme) -> Self;
    fn with_legend(self, value: LegendTheme) -> Self;
    fn with_composition(self, value: CompositionTheme) -> Self;
    fn with_marks(self, value: MarkTheme) -> Self;
    fn with_annotations(self, value: AnnotationTheme) -> Self;
    fn with_color_scales(self, value: ColorScales) -> Self;
    fn with_font_stack(self, value: FontStack) -> Result<Self>;
    fn build(self) -> Result<Theme>;
}
```

Theme is complete immutable appearance. `customize` clones complete source
values; `ThemeBuilder::new` consumes a complete value. Builder components are
private and always present. Typed component replacements need no delayed sparse
merge; build revalidates complete data-independent/cross-field invariants.
`with_font_stack` performs one construction-only bulk replacement across all
eighteen complete text roles, leaving their other fields unchanged. Subsequent
component/role replacement is an ordinary latest construction edit, not a runtime
font precedence layer. ThemeBuilder implements Clone/Debug, not render attachment.

The Theme getters above are also the builder's readonly getter signatures for its
complete working components. Theme itself has no general mutable/edit interface;
component construction edits produce values installed through ThemeBuilder.
The generic §2 `with_field` rule applies to component/group values, not Theme's
top-level fields or builder methods with the explicitly listed signatures here.

Plot common `theme(self, Theme) -> Self` stores an optional explicit complete
Theme; Canvas common `theme(self, Theme) -> Self` stores the complete root Theme.
Standalone omission resolves Publication; Canvas child omission inherits the
Canvas value; an explicit Plot theme replaces it completely. Facets inherit only
their owning Plot. Matrix remains a Plot scope. Canvas-owned surfaces/headers/
TextPanel/shared-guide chrome use Canvas; Plot-owned marks/axes/prose use Plot.

Private resolution stores typed components once, derives local normalized metrics
and retains per-operation font/cache state alongside appearance. It does not
rebuild presets late, retain runtime enum identity, merge components, or expose a
public resolved-theme snapshot. Canvas surface context override remains private.

| Current ResolvedTheme field/source | Replacement/deletion owner |
| --- | --- |
| `colors` | independent MarkTheme accents and ColorScales qualitative values; delete generic shared seed carrier |
| `font_family` | every complete TextTheme.stack; delete raw carrier/sentinel |
| `foreground` | explicit complete axis/text/stroke/bracket paints and **MarkTheme.neutral_paints.path / box_flier** RGB; those neutral consumers combine only with their single existing path-line / primary-point alpha and geometry authorities (§6). Preset construction may copy initial channels; no generic foreground, axis/text inference or color-equality fallback survives. |
| `detail_font_size` | estimate_detail, waterfall_total, panel_text and actual consuming complete guide/contrast roles; delete generic size |
| `label_font_size` | axis_title and body independently complete; delete generic size |
| `composition_title_font_size` | composition_header and text_panel_title as independent text roles; **Canvas declared-track em specifically reads composition_header.font_size**, even without headers; delete generic size |
| `facet_header_font_size` | facet_header and its owning Plot's declared facet-track em; no body/Canvas-header/TextPanel-title substitution; delete generic size |
| `surfaces`, `typography`, `legend`, `axes`, `marks`, `annotations`, `composition` | single validated component representation and private metric adapter |
| `canvas_surface_override` | private owner-compositing context; not Theme construction/equality |
| `font_environment`, `text_queries` | operation state; not Theme component construction/equality |
| `define_themes!` enum/exhaustive runtime `resolve` match | B4 complete constructor API; private preset data may remain as constructor data, not mode/selector |

## 11. Exact Styles And Property-Level Precedence

### GuideStyle

```rust,ignore
impl GuideStyle {
    fn new(color: Color, width: f32, pattern: LinePattern) -> Result<Self>;
    fn color(&self) -> Color;
    fn width(&self) -> f32;
    fn pattern(&self) -> LinePattern;
    fn with_color(self, color: Color) -> Result<Self>;
    fn with_width(self, width: f32) -> Result<Self>;
    fn with_pattern(self, pattern: LinePattern) -> Result<Self>;
}
```

Positive finite width and narrow closed pattern are unchanged. Color is complete
exact RGBA, including transparent/opaque endpoints. No separate opacity, theme
paint borrow or font input. GuideSpec's existing horizontal/vertical/segment
scientific geometry and preflight coordinate validation remain unchanged.

### PointLayer

```rust,ignore
impl PointLayer {
    fn new(x: NumericValues, y: NumericValues) -> Self;
    fn with_color(self, color: Color) -> Self;
    fn with_alpha(self, alpha: u8) -> Self;
    fn with_radius(self, radius: u32) -> Result<Self>;
    fn color(&self) -> Option<Color>;
    fn alpha(&self) -> Option<u8>;
    fn radius(&self) -> Option<u32>;
    fn labels(self, labels: CategoryValues) -> Self;
    fn legend_label(self, label: impl Into<String>) -> Self;
}
```

PointLayer stores optional independent exact color/radius/alpha, initially all
`None`. It retains its current data/label identities and lexical/alignment
preflight rules. Radius override validates positive immediately; alpha includes
0 and 255. The getter inspects an optional caller instruction, not an imagined
effective style without an owner. Internal x/y/label/legend accessors retain
their current data vocabulary. Old `.color(tuple)`, `.radius` setter and
`.opacity(f64)`/floating storage migrate, with no alias route.

Resolve mark fill as:

1. Stable append-slot opaque accent plus `MarkTheme.points.supplemental.fill_alpha`
   (built-in 204), radius from its theme default (built-in 4).
2. Exact color, when supplied, replaces the entire mark RGBA.
3. Explicit alpha, when supplied, replaces that resolved mark alpha.
4. Exact radius replaces only radius. Geometry stays plain Circle with no outline.

Resolve text as the complete `TypographyTheme.annotations.point_layer_label`:

1. Automatic layer accent and automatic mark alpha do **not** change its color.
2. Explicit layer Color replaces text RGBA only. Stack/size/weight/leading remain
   the text role's complete values.
3. Explicit layer alpha replaces text alpha only, even if color is absent.
   With no exact color the text retains role RGB, not accent RGB.

Thus color-only, alpha-only, radius-only, neither, and both in either call order
are distinct meaningful cases. An opaque exact Color is intentionally opaque
without an alpha edit; a separate alpha wins without multiplication. No added
label-paint setter or automatic accent-to-text mapping is authorized. B2 replaces
`plots/line.rs`'s unconditional `paint: style.color` with this selective resolver;
B3 exposes typed caller instructions. Guide keys always reuse final mark paint,
not the independently resolved label paint. Exact layers still consume slots.

#### Existing Label Placement Policy And Complete Uncut Ink

The actual current `plots/line.rs` label operand is
`offset = f64::from(final_point.radius) + point_label_gap`, added to **both**
projected coordinates, followed by independent **Rust `f64::round()` to whole
CSS pixels** (halfway away from zero). It then uses `PlacedText::middle` with
**horizontal `TextAnchor::Start`, vertical `PlannedTextOrigin::Middle`, zero
rotation**, and Start-aligned text lines. “Middle” here is the vertical origin,
not a horizontal Middle anchor or baseline origin. A TextAppearance comment is
not authority to replace these actual operands.

Preserve this observed positioning policy at B2/B3. Resolve the theme's nominal
gap locally, preserve radius+gap and the order of addition/whole-pixel snapping,
then canonicalize/freeze the selected placement. Do not silently switch to
nearest-even quantum placement, baseline placement, offset on only one axis,
painted-edge+gap without the current radius, or an added label-paint/anchor setter.
The gap is the current nominal radius-relative offset, not a new guaranteed
minimum distance between arbitrary glyph and point ink after whole-pixel snapping.

`annotations.point-layer-label-gap` owns the relationship;
`annotations.point-layer-label.anchor-orientation` retains Start/zero;
`annotations.point-layer-label-coordinate-clamp` and
`marks.line.supplemental-clipped-draw-order` require validated finite placement and
complete shaped bounds. The old historical canvas-origin clamp is already absent
from current source and must not return. Labels follow the current visible-data
range gate, but a present required label's **unclipped** final glyph ink supplies
fit/collision/owned-ink requirements: clipping a point layer is not permission to
cut its required text. If the snapped placement fails complete containment, use
only the already admitted root feedback or reject; do not detach/relocate the
label silently. Any necessary complete-ink correction is explicit B2 evidence
with original source/line counts, selected coordinate/anchor, ink and containing
box, rather than an undocumented placement-policy change.

The analogous Coefficient numeric relationship is different:
`annotations.coefficient.numeric-label-gap` owns `estimates.numeric_label_gap`,
currently 7px, from the **projected estimate center**. In
`plots/interval_estimate.rs`, right placement is `(round(x + gap), round(y))` with
Start anchor; the next existing choice is `(round(x - gap), round(y))` with End
anchor. Both retain vertical-middle origin and zero rotation. The current
right/left fit order and automatic-domain headroom consequence remain; **do not
add point radius or outline extent to 7**, or reinterpret it as painted-edge+7.
Complete glyph/point/reference ink still participates in actual fit/collision
validation under `annotations.coefficient.adjacent-label-placement-and-fit`.
This same prepared center/placement semantics carries the existing Coefficient
attached significance label with its effective significance gap, not the numeric
7px field. Typography/policy selection cannot mutate either scientific center.

### BarPointLayer And DumbbellEndpoints

```rust,ignore
impl BarPointLayer {
    fn new(category: CategoryValues, series: Option<CategoryValues>,
           value: NumericValues, color: Color, radius: u32) -> Result<Self>;
    fn category_values(&self) -> &CategoryValues;
    fn series_values(&self) -> Option<&CategoryValues>;
    fn values(&self) -> &NumericValues;
    fn color(&self) -> Color;
    fn radius(&self) -> u32;
}
impl DumbbellEndpoints {
    fn new(left_label: impl Into<String>, left_color: Color,
           right_label: impl Into<String>, right_color: Color) -> Result<Self>;
    fn left_label(&self) -> &str;
    fn left_color(&self) -> Color;
    fn right_label(&self) -> &str;
    fn right_color(&self) -> Color;
}
```

BarPointLayer removes the opacity argument/field/getter together; radius getter
stays a positive exact u32. Full alpha is obtained from Color, inclusive endpoints,
with plain point geometry, exact prepared bar centers and no layer legend.
No separate alpha/radius convenience control is added to that constructor-owned
style. Existing row/series/facet/finite coordinate validation stays preflight.

Dumbbell label validation remains nonempty, trimmed, control-free and distinct.
Each Color replaces the entire corresponding endpoint fill and literal guide
key, with no point-role alpha multiplication. Theme radius/shape/independent
outline and outline visibility still apply; neutral connector is independent.
B3 migrates both constructors, fields, getters, mark and guide operands and every
tuple caller together.

### SignificanceMarkStyle And InteractionStyle

```rust,ignore
impl SignificanceMarkStyle {
    fn new(font_size: u32, weight: TextWeight, gap: f64) -> Result<Self>;
    fn font_size(&self) -> u32;
    fn weight(&self) -> TextWeight;
    fn gap(&self) -> f64;
    fn with_font_size(self, size: u32) -> Result<Self>;
    fn with_weight(self, weight: TextWeight) -> Result<Self>;
    fn with_gap(self, gap: f64) -> Result<Self>;
}
impl Default for SignificanceMarkStyle { /* explicit 16px / Normal / 8px */ }

impl InteractionStyle {
    fn new(group: impl Into<String>) -> Result<Self>;
    fn group(&self) -> &str;
    fn point_shape(&self) -> PointShape;
    fn line_pattern(&self) -> LinePattern;
    fn with_point_shape(self, shape: PointShape) -> Result<Self>;
    fn with_line_pattern(self, pattern: LinePattern) -> Result<Self>;
}
```

Significance exact size is positive actual CSS u32, weight closed, gap finite
nonnegative exact CSS f64. Gap normalizes signed zero and uses exact represented
binary64 S/H rounding. Explicit default() is a caller exact style; it is not the
absence path. A supplied style overrides size/weight/gap only; role stack/color/
leading and theme bracket geometry remain authoritative. There is no color,
alpha or family edit. B3 migrates existing consuming names/private `_spec`
callers to this one inspection/edit contract and retains its actual capability.

Interaction lexical construction requires a nonempty, already trimmed,
control-free raw group selector, retains its bytes, and adds no whitespace collapse,
casefold or alias lookup. It still defaults to Circle/Solid. Prepared
group resolution validates known groups, uniqueness and complete supplied group
coverage before fonts. Shape/pattern replace only their matching fields; they
do not replace data color, alpha, radius, outline or interval/connector width.
The constructor correction, getter/edit spellings and repository call migration
are **B3**, not a late unassigned B4/L task. No Color input is invented.

## 12. Adjacent Approved Policy Signature Handoff

These policies are not theme fields. TickGeneration and NumericFormat are public;
remaining section/integrated closure gates stay with spec93. This
section closes getter naming and validated
construction inputs without claiming numerical certification or section evidence.
Detailed design is closed in `06_NUMERIC_PRESENTATION.md`,
`07_LOG10_DISPLAY.md` and `08_DISCRETE_GUIDES.md`. Later source refreshes verify
their transport/mapping to migrated foundations. Exact numeric implementation and
independent certification are complete; current public evidence is in
`../docs/public-numeric-formatting.md`. Spec97's numeric selection extension replaces
the interval-only carrier and attachments with the complete current value below.

```rust,ignore
impl TickGeneration {
    fn automatic() -> Self; // also Default
    fn additive(step: f64) -> Result<Self>;
    fn log10_decades(decades: u32) -> Result<Self>;
    fn target(count: u32) -> Result<Self>;
    fn maximum(count: u32) -> Result<Self>;
    fn is_automatic(&self) -> bool;
    fn additive_step(&self) -> Option<f64>;
    fn decade_step(&self) -> Option<u32>;
    fn target_count(&self) -> Option<u32>;
    fn maximum_count(&self) -> Option<u32>;
}
impl<K: kind::NumericX> Plot<K> {
    fn x_tick_generation(self, generation: TickGeneration) -> Self;
}
impl<K: kind::NumericY> Plot<K> {
    fn y_tick_generation(self, generation: TickGeneration) -> Self;
}
```

TickGeneration's representation is private. Additive requires positive finite
binary64; decades positive u32, preserving the approved public signature. The
current private source's u64 arithmetic widens that input internally; it does not
authorize a larger public argument type. Counts validate `1..=10_000`; Target is
approximate, Maximum bounds every generated in-domain position including endpoints.
Setters store one complete concrete per-axis instruction, replacing the previous
choice; Automatic resets it. Actual categorical non-Automatic choices reject before
fonts; scientific interval scale/kind mismatch rejects. Exact lists suppress
generated selection/format text irrespective of setter order and are not capped.
Spec05 owns source semantics; spec97 owns the extension evidence.

```rust,ignore
enum NumericNotation { Decimal, Scientific, Engineering }
enum NumericPrecision {
    Fractional(u8), Significant(u8),
    AdaptiveFractional { min: u8, max: u8 },
    AdaptiveSignificant { min: u8, max: u8 },
}
enum NegativeZero { Preserve, Normalize }
enum TrailingZeros { Keep, Trim }

impl NumericFormat {
    fn new(notation: NumericNotation, precision: NumericPrecision) -> Result<Self>;
    fn notation(&self) -> NumericNotation;
    fn precision(&self) -> NumericPrecision;
    fn decimal_scale(&self) -> i16;
    fn prefix(&self) -> Option<&str>;
    fn suffix(&self) -> Option<&str>;
    fn negative_zero(&self) -> NegativeZero;
    fn trailing_zeros(&self) -> TrailingZeros;
    fn with_notation(self, value: NumericNotation) -> Result<Self>;
    fn with_precision(self, value: NumericPrecision) -> Result<Self>;
    fn with_decimal_scale(self, value: i16) -> Result<Self>;
    fn with_prefix(self, value: impl Into<String>) -> Result<Self>;
    fn with_suffix(self, value: impl Into<String>) -> Result<Self>;
    fn with_negative_zero(self, value: NegativeZero) -> Result<Self>;
    fn with_trailing_zeros(self, value: TrailingZeros) -> Result<Self>;
}
enum LegendComposition { Combined, Separate }
```

NumericFormat is opaque Clone/Debug/PartialEq/Eq, with normalized exact policy.
Construction defaults scale 0, absent affixes, Preserve negative zero and Keep
trailing zeros. Default() is Decimal/Fractional(2) with those fields. Fractional
0..=17, significant 1..=17, adaptive min<=max in the matching bounds, scale
-324..=324, and nonempty/control-free literal affixes validate immediately. No
empty-string reset or extra notation/scaling control is introduced. Reconstruct
without an affix to omit it. This uses the same `with_field` vocabulary, replacing
spec93's provisional collision-prone consuming bare-field names at N0/N6.

N6 attachments remain exactly x/y tick formats under NumericX/Y, color label
format under ContinuousColor, HeatmapAnnotations::Values payload, Coefficient's
point-only estimate_labels, and Waterfall total_format. Configured Forest
exponent 0.5/Fractional(2) means certified mathematical `3.16`; exact scale 2 and
suffix `%` gives `316.23%`. N2's proof/oracle/resource gates remain mandatory.
Private omitted-policy profiles preserve existing default grammars of one engine.

The post-foundation source handoff is complete at `387b07a` in
`../docs/numeric-source-handoff.md`. Its actual copy/cache/font/natural-sizing
owners replace historical admission names, without changing these signatures or
marking numeric implementation/certification complete. Spec95 removes historical
numeric work/byte fees; exact arithmetic and real storage/length checks remain.

LegendComposition defaults Combined and is exposed at L5 with
`Plot<K: kind::Legend>::legend_composition(self, LegendComposition) -> Self` and
`Canvas::legend_composition(self, LegendComposition) -> Self`. Current seventeen
Legend-capable families remain exactly A1's gate; Heatmap/Beeswarm continuous
guides do not acquire it. PerPlot/Shared/Hidden authority, typed sections,
producer-owned neutral projections, section ordering and unsupported-factorization
errors remain spec93 T6L. Chrome has no composition field. No typed raw f64 policy
enum is exported in place of validated font/gradient/domain/scale values.

C2 root width/size request and Output's frozen CSS dimension getters remain
spec05/spec93. They do not turn component metrics into root-size policies or
resurrect pre-render Plot width/height inspection.

## 13. Approved Unified Sequential Default Cutover

The explicit user decision is the single ordered opaque sequence:

| Semantic offset | Color | Application |
| --- | --- | --- |
| 0 | `#F7FBFF`, RGB `(247,251,255)` | low endpoint of every built-in ColorScales sequential gradient |
| 1 | `#08306B`, RGB `(8,48,107)` | high endpoint of the same gradient |

Publication, Nature, Science and CHI all construct this same SequentialGradient;
Heatmap and Beeswarm consume it through the same resolved ColorScales authority.
This satisfies the ledger's
`color-scales.continuous.{heatmap,beeswarm}.complete-gradient-mapping` shared-default
assignment. Heatmap's default mapping is preserved. Beeswarm's old built-in
`#3182BD` (`49,130,189`) to `#DE2D26` (`222,45,38`) sequence is **retired**, with
its intentional default mark/guide RGB and SVG/PNG/snapshot changes attributed
to this approved decision at B2, not called byte-preserving refactoring.

B2 moves the chosen default into the complete theme construction and removes the
private family-selected default source; B4 closes public scale/Theme migration
and old gradient-field deletion. There is no family-specific default field,
compatibility selector or automatic hidden Beeswarm override. The diverging
three-stop default, qualitative/accent prefixes and independent mark alphas
remain pinned in their own owner tables.

Caller replacement is the existing typed construction path: build a validated
SequentialGradient from ColorStops/OpaqueColor, supply it through
`ContinuousColorScale::sequential(Some(gradient), domain)` and
`Plot::continuous_color_scale`, or edit `ColorScales::with_sequential` and install
that complete component through `ThemeBuilder::with_color_scales(...).build()`.
The retired blue/red colors remain valid caller-selected opaque colors; they
receive no special factory, alias or automatic fallback. No product decision
remains open for this default.

## 14. Implementation Owners, Deletions And Required Fixtures

### Owner/Cutover Table

| Stage | Single-representation work / deletion | Affected call paths and acceptance |
| --- | --- | --- |
| B1 | Private Color/OpaqueColor/fonts/relative-spacing/stops/components/role groups, including independent sized raw-overlay/mandatory-strip roles, typed neutral path/flier RGB, independent Waterfall breadth and admitted glyph-edge/root-padding values; remove raw writable foundational fields in that representation; replace zero-outline sentinel with absence | `theme/resolved`, canonical font inputs, backend conversion, constructor boundary fixtures |
| B2 | Complete Theme/builder/preset construction and resolution; all eighteen typography roles/local metrics; neutral path/flier materialization with one alpha; Waterfall one-breadth geometry; exact declared-track em owners/keys; keyed common continuous-guide successors; split raw/mandatory-strip consumers; glyph-edge comfort/root padding; effective significance geometry through backend validation; PointLayer selective resolver with preserved position policy; general continuous branch/Shared domain consumers including divergent Beeswarm | all named consuming helpers in §§4–11; retain one font environment/appearance identity and immutable measured operands; implement §13's approved shared gradient and attribute its intentional Beeswarm default change |
| B3 | Public exact Color and changed exact styles; remove RGB tuple and floating opacity routes; fallible Interaction selector and `with_field` vocabulary; migrate every affected constructor/edit/getter caller | guides, Line layers, Bar layers, Dumbbell, significance, Interaction, producer keys/capture and representative parent callers |
| B4 | Complete Theme/components/fonts/scales/helpers public only when consumed; closed selector removal, root font constant retirement, categorical gate rename, complete continuous attachment, Shared-gap method/field/compatibility-check removal | root exports, builder erasure/admission/scoped copies, Canvas/facets/defaults, examples/tests/docs/current contracts; no dual APIs |
| B5 | Four independent alignment instruction setters; caption evidence over already active B4 theme component | root/embedded/outer facet/Matrix, Canvas root independence, exact physical spans and setter-order fixtures |
| B6 | Integrated public custom construction/owner/contrast/scale/exact-style evidence | custom themes with distinct components/roles, explicit child replacement, exact property edits, shared key compatibility |
| C1/C2 | Validated opaque interval and one full sizing request/frozen Output dimensions | existing numeric gates; remove pre-render dimension getters and Canvas universal minimum at C2 |
| N0/N6 | Confirm actual numeric signatures/keys then public current-format migration using §12 vocabulary | delete NumericLabelFormat at N6; no second formatter or blanket Any setter |
| L0/L1–L5 | Actual producer/key closure, one section representation and public policy cutover | delete flat descriptors at L1, reconcile approved section semantics; L5 public attachments |

No replaced public caller first migrates at B6/Z1. Documentation status and
ledger/current-spec reconciliation are the phase owner's work; this file does not
edit those documents or rewrite historical ledger counts.

The C1/C2 implementation checkpoint is now complete: opaque preferred intervals
are recorded in `../docs/preferred-tick-intervals.md`; one private root request,
automatic defaults, consuming width and frozen Output dimensions are recorded in
`../docs/content-derived-root-sizing.md`. Pre-render Plot getters and the Canvas
universal minimum are retired. C3's public parent/mode/output closure is complete
in `../docs/public-sizing-closure.md`, including actual horizontal preference and
provisional-height repairs. Numeric/sectioned-guide consumers remain the following
stages, without reopening these construction signatures.

### Exhaustive Construction And Consumer Fixture Design

These are required **future implementation fixtures**, not tests executed by A2.
Use the current Rust fixture locations in A1 §11 and their independent expected
values, adding only focused cases that exercise new contracts.

Execution follows the current AGENTS.md/spec90 policy: every introduced unit test
replaces an existing unit test in the same logical change, with no increase in
discovered count or migration/packaging workaround. The families below identify
semantic contracts, not a requirement to restore deleted getter/default inventories.

| Fixture family | Required independent expectations / owning leaf |
| --- | --- |
| Full component coverage | Construct every typed group from nondefault distinguishable values; inspect every retained field by §2; one selective edit leaves all other fields unchanged; no public literal/mutable/backend construction. Cover all 41 MarkAppearance mappings, independent mandatory-strip role, two constructed neutral RGB roles with their existing single-alpha/geometry owners, independent Waterfall breadth, twelve radii, eighteen text roles, fourteen private legend fields plus glyph-edge comfort and keyed continuous-guide successors, six axis spacings, twelve private composition fields plus all four RootPadding relationships/panel-edge clearance and declared-track role metrics, and all annotation helpers. B1/B4 |
| Exact metric boundary | `ratio(6,14)` × 14px gives exactly 6144 quanta; equivalent reduced ratios compare equal; binary64 `new` retains its dyadic, signed zero canonicalizes; P(p,m) preserves every §7 source bit pattern through exact division; raw title 1.3 × 20px rounds upward to 26625, not 26624 quanta; independently inspect individual S/H and complete edge/title/row/column/section sums, including the 11px edge-inset trap; large finite ratio/metric overflow refuses. B1/B2 |
| Constructor refusal | zero font/radius/present width, NaN/infinite/nonpositive leading, negative spacing, zero denominator, empty fonts/palettes/map/gradient, duplicate raw keys, unordered/missing-endpoint/duplicate stops, missing 0.5 neutral, wrong-kind scale edits, out-of-domain center, opaque contrast invariant. B1/B4 |
| Preset preservation | Exact curated palette/accent prefixes, surfaces, text role colors/sizes/leading bits, stroke bits/patterns, field metric ratios/default pixels and already quantized alphas. Compare default scientific values/paint/complete ink; no blanket fingerprint repinning. Explicitly distinguish the approved label-paint correction, consumer consistency fixes and §13 selected shared-default change. B2/B4 |
| Separate strip/raw roles | Both default sized roles are radius 2/alpha 209/Circle/white outline. Editing optional raw appearance/radius leaves mandatory Raincloud sampled/supplied strip fit, mark and key unchanged; editing the strip reaches all of its packing/intrinsic/mark/key consumers and leaves optional overlays unchanged. Apply existing outline visibility once to each final style; preserve mandatory presence and actual source rows. B2/B4 |
| Neutral materialization ownership | Use mutually distinct path-neutral, flier-neutral, categorical-mapped, accent, axis, text/legend-label and reference-stroke RGB. Neutral Trajectory/LinePath paths take only path-neutral RGB and shared path-line alpha/width/pattern; hue paths retain mapped RGB and the same appearance. Boxplot hidden-overlay fliers take only flier-neutral RGB plus primary alpha/shape/outline and radius box_flier, while optional observations retain their mapped raw role. Alpha-only, RGB-only, radius/shape and outline-policy edits reach their actual owners once; foreground deletion and cache/scoped capture cannot substitute another equal-looking role. B2/B4 |
| Waterfall breadth authority | Default derivation retains the four pinned 0.6/0.2/0.8/0.4 bit patterns and original per-band arithmetic/connector cardinality. Custom breadth changes intrinsic body/outline/connector pitch and every rectangle/connector endpoint coherently without changing cumulative numeric spans, sign paint or Bar geometry. Check constructor 0/out-of-range/nonfinite refusal, breadth 1 with zero versus positive required edge/connector ink, very narrow positive spans, actual stroke representability and fixed allocation fit; no new floor/clamp/omission. B2/B4 |
| Chrome owner/cardinality | Custom glyph-edge comfort reaches natural/assigned/inside legend extents once per vertical edge and once per outer sectioned guide. Custom RootPadding reaches standalone/facet/Canvas root and embedded Plot prose with correct owner replacement/presence rules. TextPanel insets are per edge; row-header, column-header and row-title additions are once per total band/span. Verify cache/feedback operands and no reintroduced minimum floors. B2/B4 |
| Continuous-guide successor boundaries | Both families use measured title plans at assigned inner width, including long titles, not old 150/170×0.9 wrap budgets. Required gradient length above preferred succeeds under a sufficient finite assignment; insufficient derived maximum misses; no old 150/160/180 cap or preferred-as-maximum substitution. Missing/absent-missing cases use one measured key box/label/row gap in reserve, maximum and frozen placement, including custom key outline and font metrics, complete origin-edge fit and no 34/12/32/3 reconstruction. Local/facet/Shared consumers reuse the same descriptor/plan. B2/B4 |
| Declared-track metric/cache ownership | With no visible headers, sparse Canvas declared empty/trailing tracks have hard 1 quantum and preferred composition_header font-size em; facet tracks use the owning facet_header em. Changing the actual owner font size invalidates summaries/cache/feedback even at equal width/topology. With gap/padding held fixed and affected prose/TextPanel absent, body or TextPanel-title font-size edits do not alter the declared pair; visible headers are tested separately as additional measured requirements. Check checked natural preferred sums, exact shrink and stable addresses. B2/B4 |
| Palette/scales | fixed capacity minus/equal/plus; repeated/unused exact colors; missing used raw identity before fonts; stable extension prefix; automatic/exact layer slots with exact override consuming slot. Actual fifteen-family categorical compile-pass and neighboring unsupported/Any compile-fail. B2/B4 |
| Continuous complete behavior | both families × sequential/diverging × automatic/exact/constant domain; exact saturation, center endpoint/neutral, extra stop branches, general gradient Shared merge/install and observed-color/serialized-stop collision refusal. Wrong-kind edit fails immediately; divergent Beeswarm produces center marks/guide labels, not just stored configuration. B2/B4 |
| Exact alpha precedence | PointLayer neither/color-only/alpha-only/radius-only/color+alpha in both orders, alpha 0/255, exact opaque Color, theme radius 4/alpha 204 replaced by distinct custom values when absent. Text role RGB survives automatic accent and alpha-only edits, explicit Color replaces text RGBA only, explicit alpha wins without multiplication. Bar/Dumbbell exact fill versus independent outline/guide. B3 |
| Significance absence and geometry | custom default size/weight/gap versus None; explicit Default 16/Normal/8; selective style edits preserve stack/color/leading/bracket; custom tick-height edit reaches planner, validate_bracket/canonical_bracket, GeometrySnapshot and frozen SVG/Null replay rather than reverting to 7. Data-gap/level/panel edits reach frozen path/containment consequences; exact/local gaps and thick stroke fit use identical intrinsic/collision/final operands. B2/B3 |
| Label/strip/Heatmap operand fidelity | PointLayer radius+gap on both axes, half-away whole-pixel snaps, Start horizontal anchor and Middle vertical origin survive typed paint migration; unclipped glyph fit remains mandatory. Coefficient numeric gap 7 is center-relative, not radius+7. Facet correction preserves label/source/line/glyph counts while replacing size+6 reserve with selected 1.2-role occupancy. Heatmap value-cell source rows use +5 in every size/final path; fully-null summary rows use their separate +4. B2/B3 |
| Shared gap retirement | migrate `tests/composition/canvases.rs::shared_legend_gap_reduces_canvas_legend_separation` to a custom Canvas LegendTheme; reduced gap changes measured separation, theme configuration remains valid under PerPlot/Hidden, negative constructor spacing rejects; removed Canvas method compile-fails. B4 |
| Font/owner completeness | distinct facet/header/TextPanel title/body stacks/weight/leading; actual role census matches draw; literal Noto Sans has no sentinel substitution; built-in stack comes through Theme; transparent surfaces/backdrop contrast chooses complete opaque role before fonts; missing glyph/host/resource failures remain fatal. B2/B4 |
| Alignment/current boundaries | all TitleAlignment variants and optional subtitle instructions; no dormant component at B4; B5 instructions survive erasure and theme-order changes; complete explicit child Theme replacement, Canvas prose/header/TextPanel exclusion, Plot/Canvas captions independent. B4/B5 |
| Numeric/section policy boundaries | §12 getter/edit compile-pass without collisions, proper capabilities/no Any, exact ticks suppress policy requirements; later N/L numerical/provenance/flow/frozen-output fixtures remain spec93 rather than theme tests. Owning N/L leaves |

Expose types only with their consuming cutover: B3 may expose Color for exact
styles, while private helpers/components/fonts/scales become public together at
B4. A readonly getter is not permission for a second representation. Compile
fixtures must distinguish foundation inputs from private backend/layout values.

## 15. Documentation Verification And Handoff

A2 changes only this document. Verification is source/spec consistency review and
`git diff --check`, including the new untracked file through
`git diff --no-index --check /dev/null specs/94_T6_FOUNDATION_CONSTRUCTION.md`.
Source/spec consistency review is complete and both whitespace checks passed.
No implementation, tests, build, font discovery, network, output generation,
subagent or commit is evidence of this documentation task.

The source-field coverage summary for owner review is:

| Current record | Coverage |
| --- | --- |
| `ResolvedTheme` | all 17 fields, including seven legacy appearance carriers and private context/runtime fields; §10 |
| `SurfaceAppearance`, `TextAppearance` | 2 surface fields and all 5 text fields; §§3–4 |
| `FoundationalTypography` | all 14 current roles plus 4 admitted legacy complete roles; §4 |
| `MarkAppearance`, `PointSizeAppearance` | all 41 current mark fields and all 12 radius fields, independent raw-overlay/mandatory-strip sized roles, constructed neutral path/flier RGB and one-alpha consuming styles, plus separate Waterfall breadth beyond the current shared records; §6 |
| Mark/stroke compounds | StrokeAppearance 3, DataFillAppearance 3, both FillOutlinePaint choices, FixedFillAppearance 3, FillOutlineAppearance 2, PointAppearance 3, backend PointOutline 2, DataLineAppearance 3, and the complete typed SizedPointTheme inputs; §5 |
| `AxisAppearance` and compounds | all 4 aggregate fields, FrameAppearance 2, TickAppearance 2, AxisSpacing 6; misleading frame metadata explicitly deleted; §7 |
| `LegendAppearance` | all 14 fields, ledger-owned glyph-edge comfort, exact binary-literal/composite-rounding trace, and keyed title-extent/derived-maximum/missing-key continuous-guide successors; §7 |
| `CompositionAppearance`, `RootTitleBlockAppearance` | all 12 aggregate fields and all 3 nested root relationships, RootPadding's four relationships, current panel-edge comfort and admitted strip helpers, plus exact distinct Canvas/facet declared-track em owners and natural/cache consequences; source/overlay and total/per-edge traces; §7 |
| `AnnotationAppearance`, `BracketStrokeAppearance` | all 6 aggregate fields and both stroke fields, theme significance gap 8, all four planner geometry inputs through backend validation/frozen draw, distinct Heatmap value/null-row fit fields and remaining admitted geometry/clearance helpers; §8 |
| Exact-style records | GuideStyle 3, PointLayer 7 current fields, BarPointLayer 6 current fields, DumbbellEndpoints 4, SignificanceMarkStyle 3, InteractionStyle 3; retained data/content, typed replacements and absence/default/alpha precedence; §11 |
| Font/scale/default routes | both current font exports/sentinels, separate accent/categorical seeds, fixed/extendable map/palette inputs, all three current gradient constants and local domains/private replacements; §§3, 6, 9, 10, 13 |

Field coverage is explicit in §§4–8 and the ResolvedTheme/compound owner tables.
Current private records are a source census, not a limit that can erase admitted
semantic roles or live theme-owned constants. No current MarkAppearance field,
typography/foreground/font/seed carrier, exact-style default or local metric is
silently promoted as raw backend state. Both explicit user product decisions stay
closed. The four reviewed gaps now have specified constructors/owners, actual
consumer successors and focused future evidence requirements within the approved
scope. The phase owner's independent review closed the four gaps and accepted
T6A2's construction design. Public implementation, code validation and individual
consumer cutovers remain pending; this closure is not their acceptance evidence.
