# Planned Public API

## Status And Reading Rules

This is the signature appendix to [the implementation plan](91_IMPLEMENTATION_PLAN.md).
It specifies the target API, not APIs already implemented by this checkout.
The implementation plan owns behavior, sequencing, scientific relationships and
verification; this appendix owns the concrete public names, types and defaults.
Implement both together. An unresolved implementation algorithm is not permission
to replace a signature, narrow a capability or introduce another public route.

All listed names are crate-root exports except sealed capability traits in `kind`.
Implementation modules remain private. `Result<T>` means `crate::Result<T>` with
the existing `Error`. Tables list Rust method signatures; implementation bodies,
documentation, `pub`/existing `const` qualifiers and routine `#[must_use]`
annotations are omitted. Preserve existing public const-call capabilities unless
the capability's specified type/behavior change actually prevents them.

Configuration structs have private fields and consuming edits. Existing public
read capabilities remain available under the confirmed naming rule. The appendix
does not require a getter for every newly introduced edit. Borrowed callback
contexts are library-created read-only views, not a second configuration hierarchy.
Owned policies remain cloneable; floating policy values retain normalized finite
inputs and use complete equality, not equality of one rendered result. Function
handles compare by shared identity as specified in the implementation plan.
These signatures complete the already planned capabilities; they do not authorize
additional text producers, scientific transformations or deferred features.

## Migration Summary

| Current surface | Target change |
| --- | --- |
| Public configuration `with_property(value)` | `property(value)`; a paired reader becomes `get_property()` |
| Read-only or write-only properties | Keep the bare name |
| Free-form display setters accepting `Into<String>` | Accept `Into<Text>` through the inventory below |
| `TitleAlignment` | Replace with `TextAlignment`, with `Start`, `Center`, `End` |
| `TextTheme` and `SignificanceMarkStyle` font sizes | Use `f64` CSS pixels instead of `u32`, including constructors, edits and readers |
| `HeatmapLabelFit` and `Plot::heatmap_label_fit` | Remove; migrate Elide/Wrap to x/y `TextLayout` |
| `TickGeneration: Copy` | Remove `Copy`; retain `Clone`, `Debug`, `Default`, `PartialEq` |
| Tick count arguments/read results using `u32` | Use `usize` consistently with source lengths when extending counts |
| `TickGeneration::maximum_count()` | `get_maximum_count()` once the consuming maximum edit is introduced |
| `PanelTextAnnotation::corner()` | Replace with `position() -> &TextPosition`; inspect `as_corner()` on that value |
| `Labels::resolve` | Promote the current implementation to public |

The naming-only delivery step preserves current parameter types. Fractional-size,
count and position migrations land with their corresponding working capability,
including all repository callers. No aliases, deprecated names or old execution
paths remain. Existing `TextPanel::title/body`, annotation text and display-name
readers continue returning `&str`; they borrow the retained `Text` source.

## Text Inputs And Appearance

### Text

`Text` is an owned immutable value with one original string, an optional complete
`TextTheme` and an optional whole `TextLayout` request. It implements `Clone`,
`Debug`, `PartialEq`, `Eq`, and conversions from `String`, `&str` and `&String`.
Its equality includes exact appearance, request presence and all instructions.
Retain equality on the existing TextPanel/annotation values containing it.

| Method | Meaning |
| --- | --- |
| `Text::new(source: impl Into<String>) -> Self` | Retain verbatim source; shared display validation occurs at the receiving constructor/preflight |
| `Text::as_str(&self) -> &str` | Borrow the original source |
| `Text::style(self, style: TextTheme) -> Self` | Replace the complete exact appearance |
| `Text::get_style(&self) -> Option<&TextTheme>` | Inspect local appearance presence |
| `Text::layout(self, layout: TextLayout) -> Self` | Supply one whole layout request, including an explicit default reset |
| `Text::get_layout(&self) -> Option<&TextLayout>` | Distinguish omission from a present reset |
| `Text::alignment(self, alignment: TextAlignment) -> Self` | Edit alignment in that same request |
| `Labels::resolve(&self, raw: &str) -> Result<String>` | Existing humanized/raw/alias resolution for one name |

Plain source replacement retains existing slot instructions. A present local
appearance replaces the whole exact appearance; a present layout replaces the
whole request. Its omitted fields resolve to owner defaults, not the replaced
request's fields. These rules also govern callback-supplied explanatory Text.

### Fractional Appearance

The target constructor is
`TextTheme::new(stack: FontStack, color: Color, font_size: f64, weight: TextWeight, line_height: f64) -> Result<Self>`.
Its `font_size(self, value: f64) -> Result<Self>` and
`get_font_size(&self) -> f64` retain positive finite CSS sizes without integer
rounding. Other current edits/readers follow the naming rule with their present
types. The complete target surface is:

| Edit on `TextTheme` | Paired reader |
| --- | --- |
| `stack(self, value: FontStack) -> Result<Self>` | `get_stack(&self) -> &FontStack` |
| `color(self, value: Color) -> Result<Self>` | `get_color(&self) -> Color` |
| `font_size(self, value: f64) -> Result<Self>` | `get_font_size(&self) -> f64` |
| `weight(self, value: TextWeight) -> Result<Self>` | `get_weight(&self) -> TextWeight` |
| `line_height(self, value: f64) -> Result<Self>` | `get_line_height(&self) -> f64` |

Keep complete appearance validation and add `Eq` over valid normalized values to
TextTheme, which currently has PartialEq only. Its private positive finite size/
leading fields admit reflexive equality; compare the complete stack, RGBA, weight
and those fields. Text/annotation equality must not discard appearance or replace
it with the current rendered result. Layout float fields likewise have validated
finite normalized representations before they enter an Eq policy.

Likewise use
`SignificanceMarkStyle::new(font_size: f64, weight: TextWeight, gap: f64) -> Result<Self>`,
`font_size(self, value: f64) -> Result<Self>` and `get_font_size(&self) -> f64`.
Its fixed pixel gap remains independent, and it remains `Copy`; layout attaches
to the owning significance text role rather than being stored in this style.
Its other paired methods are `weight(self, value: TextWeight) -> Result<Self>` /
`get_weight(&self) -> TextWeight` and `gap(self, value: f64) -> Result<Self>` /
`get_gap(&self) -> f64`, preserving the current nonnegative finite gap contract.

### Receiving Slots

Each signature retains its current receiver, return type, capability and timing,
except for the displayed text parameter/type changes listed here:

| Receiver | Target text signature |
| --- | --- |
| `Plot<K: kind::Marker>`, `Canvas` | `title(self, text: impl Into<Text>) -> Self`; `subtitle(self, text: impl Into<Text>) -> Self`; `caption(self, text: impl Into<Text>) -> Self` |
| `Plot<K: kind::Marker>` | `x_label(self, text: impl Into<Text>) -> Self`; `y_label(self, text: impl Into<Text>) -> Self` |
| `Plot<K: kind::ContinuousColor>` | `color_label(self, text: impl Into<Text>) -> Self` |
| `TextPanel` | `new(title: impl Into<Text>, body: impl Into<Text>) -> Result<Self>` |
| `Canvas` | `column_header(self, start_column: usize, span: usize, text: impl Into<Text>) -> Result<Self>`; `row_header(self, start_row: usize, span: usize, text: impl Into<Text>) -> Result<Self>` |
| `Canvas` | `row_title(self, row: usize, text: impl Into<Text>) -> Result<Self>` |
| `PanelTextAnnotation` | `new(corner: PanelCorner, text: impl Into<Text>) -> Result<Self>` |
| `HeatmapNullRowAnnotation` | `new(row_label: impl Into<String>, text: impl Into<Text>) -> Result<Self>` |
| `WaterfallLabels` | `new(base: impl Into<Text>, terminal: impl Into<Text>) -> Result<Self>` |
| `PointLayer` | `legend_label(self, text: impl Into<Text>) -> Self` |

Raw target names, category/series keys, label-alias keys, field names and source
columns remain semantic strings. `CategoryValues` stays the aligned source type
for column-backed point labels. Its displayed values and alias outputs use common
display syntax, with appearance/layout attached to their owning role. In
particular, Dumbbell endpoint identity strings do not become arbitrary Text keys.
`SignificanceMark` retains its typed p-value/star/literal grammar; its rendered
text receives the significance-role layout below.

`TextAlignment` is `Start`, `Center`, `End`, retaining the physical left/center/right
meaning of TitleAlignment. Plot's `title_alignment(self, value: TextAlignment) -> Self`
and `subtitle_alignment(self, value: TextAlignment) -> Self`, and Canvas's
`panel_title_alignment(self, value: TextAlignment) -> Self` and
`panel_subtitle_alignment(self, value: TextAlignment) -> Self`, keep their current
owner/precedence. Theme heading constructors/edits/readers replace TitleAlignment
with TextAlignment without changing their other types. A heading edit writes the
same retained slot request as `Text::alignment`.

## TextLayout And Layout Values

`TextLayout: Clone + Debug + Default + PartialEq + Eq` contains optional
instructions. Default leaves owner choices unspecified; explicit default request
presence is retained. The following consuming edits all return `Self`:

| Method on `TextLayout` | Input |
| --- | --- |
| `alignment(self, value: TextAlignment)` | Physical line alignment |
| `anchor(self, value: TextAnchor)` | Block/baseline anchor |
| `offset(self, value: TextOffset)` | One frame-qualified offset |
| `rotation(self, value: TextRotation)` | Convenience edit of orientation to a fixed screen angle |
| `orientation(self, value: TextOrientation)` | Replace that same orientation instruction |
| `rotation_mode(self, value: TextRotationMode)` | Anchor rotation or post-rotation bounds alignment |
| `bounds(self, value: TextBounds)` | Text-local finite maxima and line cap |
| `wrapping(self, value: TextWrapping)` | Legal soft wraps or mandatory breaks only |
| `direction(self, value: TextDirection)` | Paragraph direction |
| `overflow(self, value: TextOverflow)` | Complete/derived view and terminal handling |
| `overlap(self, value: TextOverlap)` | Required nonoverlap or deliberate overlap |
| `visibility(self, value: TextVisibility)` | Explicit label visibility/reduction |
| `fit(self, value: TextFit)` | Permitted size program and grouping |
| `fit_priority(self, value: TextFitPriority)` | Order of complete versus abbreviated fitting |

No actual selected size, derived string, scientific position or mutable font
environment is retained in these public request values.

### Anchors, Coordinates And Orientation

| Public value | Constructors/variants and exact inputs |
| --- | --- |
| `TextAnchor` | `fractions(horizontal: f64, vertical: f64) -> Result<Self>`; `first_baseline(horizontal: f64) -> Result<Self>`; `center_baseline(horizontal: f64) -> Result<Self>` |
| `TextAnchor` presets | `top_left`, `top_center`, `top_right`, `center_left`, `center`, `center_right`, `bottom_left`, `bottom_center`, `bottom_right`, each `() -> Self` |
| `TextOffsetFrame` | `Screen`, `TextLocal` |
| `TextOffset` | `new(dx: f64, dy: f64, frame: TextOffsetFrame) -> Result<Self>` |
| `TextRotation` | `degrees(value: f64) -> Result<Self>`; `horizontal`, `tilt_up`, `tilt_down`, `vertical_up`, `vertical_down`, each `() -> Self` |
| `TextRotationMode` | `RotateAboutAnchor`, `AlignAfterRotation` |
| `DataCoordinate` | `numeric(value: f64) -> Result<Self>`; `category(raw: impl Into<String>) -> Self`; `category_series(category: impl Into<String>, series: impl Into<String>) -> Self` |
| `DataPosition` | `new(x: DataCoordinate, y: DataCoordinate) -> Self` |
| `TextCoordinate` | `data(value: DataCoordinate) -> Self`; `panel_fraction(value: f64) -> Result<Self>`; `figure_fraction(value: f64) -> Result<Self>` |
| `TextPosition` | `new(x: TextCoordinate, y: TextCoordinate) -> Self`; `corner(value: PanelCorner) -> Self`; `as_corner(&self) -> Option<PanelCorner>` |
| `TextOrientation` | `fixed(value: TextRotation) -> Self`; `data_direction(from: DataPosition, to: DataPosition) -> Self`; `automatic_ticks(angles: Vec<TextRotation>) -> Self` |

Anchor fractions are arbitrary finite values, not clamped to 0..1. Presets use
0, 0.5 and 1 in the declared top-down block frame. Offset inputs are finite signed
CSS pixels. Rotation presets mean 0, -45, +45, -90 and +90 degrees; normalize
finite turns to [-180,180), including signed zero, without tenth-degree rounding.
Data directions use the actual projection of the two typed references. Automatic
ticks try the retained order, without a hidden candidate-count limit. An empty
angle program has no fitting visible-angle candidate; an unused empty text owner
does not consume one. The numeric reference constructor accepts finite values;
category identity, actual-axis applicability and projection errors arise at their
actual resolution. This configuration contract is not a pass over locator results.

Figure fractions refer to the owning output root, including a Canvas root for an
embedded Plot. Panel fractions use the actual scientific panel; frames can differ
between x and y. Finite fractions outside 0..1 remain meaningful references subject
to the declared clip/overflow and final-root requirements.

Add `PanelTextAnnotation::at(position: TextPosition, text: impl Into<Text>) -> Result<Self>`.
Both annotation constructors store one TextPosition. `position(&self) -> &TextPosition`
replaces the old corner reader; `text(&self) -> &str` remains. A corner retains its
existing semantic inset/default-offset behavior instead of being reconstructed
from an unrelated drawing coordinate pair.

### Bounds, Wrapping, Overflow And Visibility

| Public value | Constructors/variants |
| --- | --- |
| `TextBounds` | `new(max_width: Option<f64>, max_height: Option<f64>, max_lines: Option<usize>) -> Result<Self>` |
| `TextWrapping` | `Legal`, `NoSoftWrap` |
| `TextDirection` | `Auto`, `LeftToRight`, `RightToLeft` |
| `TextOverlap` | `RequireNonOverlap`, `Allow` |
| `TextElision` | `Start`, `Middle`, `End`, `Left`, `Right` |
| `TextFailure` | `Error`, `Hide`, `Clip`, `Visible` |
| `TextEllipsis` | `new(position: TextElision) -> Self`; `marker(self, text: impl Into<String>) -> Self`; `max_lines(self, value: Option<usize>) -> Self`; `distinct(self, value: bool) -> Self` |
| `TextOverflow` | `complete() -> Self`; `clip() -> Self`; `visible() -> Self`; `hide() -> Self`; `ellipsis(value: TextEllipsis, failure: TextFailure) -> Self` |
| `LabelEndpoints` | `None`, `First`, `Last`, `Both` |
| `TextVisibility` | `all() -> Self`; `hidden() -> Self`; `every(stride: usize, offset: usize, endpoints: LabelEndpoints) -> Result<Self>`; `greedy(endpoints: LabelEndpoints) -> Self`; `parity(endpoints: LabelEndpoints) -> Self`; `separation(self, value: f64) -> Result<Self>`; `boundary_tolerance(self, value: Option<f64>) -> Result<Self>` |

Bounds use finite nonnegative CSS maxima, with None meaning no explicit maximum.
Zero maxima/line caps are actual requested constraints, not invented disappearance
of a source; complete content can miss them, while explicit clipping/hiding can
consume them. Mandatory source breaks survive either wrapping choice.

An explicit ellipsis starts with marker U+2026 and a one-line cap; setters replace
these conveniences, including an empty marker or no extra ellipsis line cap.
General and ellipsis line caps intersect. Start/End are logical retained-source
directions; Left/Right are physical. Distinctness defaults to the actual owner,
including the current Heatmap requirement, rather than becoming universal.

The four simple overflow factories encode complete views with the respective
terminal Error/Clip/Visible/Hide action. Ellipsis admits derived views before its
selected terminal action. The same terminal handling applies after permitted
font trials, including a real size-floor miss; it does not recover failed
projection, font, arithmetic or allocation operations.

Visibility All/Hidden is applicable to every text owner. Every/Greedy/Parity and
boundary reduction consume actual label groups. Every requires positive stride
and offset less than stride; it uses retained original occurrence indices and
unions eligible endpoint labels once. The endpoint argument explicitly protects
the selected first/last eligible labels during overlap reduction. Greedy consumes
the stable geometric order, breaking coincident-position ties by original occurrence,
and admits unprotected labels only when they fit the retained protected/earlier
labels. Parity repeatedly removes alternating unprotected candidates in that order
until the requested nonoverlap predicate fits or no reduction helps. A protected
set that cannot meet the requested fit consumes the selected terminal handling;
it is not silently unprotected. These are local label programs, not whole-figure
quality objectives.
Greedy/Parity start with zero extra separation, following Vega labelSeparation;
`separation` supplies a finite nonnegative CSS-pixel gap for that reduction
predicate. It applies only to those two modes and rejects an inapplicable edit,
rather than silently ignore a configured clearance. It does not alter ticks,
category positions, mark spacing or the owner's other declared reservations.

An optional boundary tolerance is in CSS pixels, as in Vega labelBound: positive
values permit that much overhang, negative values request an inset. Boundary
hiding is resolved before overlap reduction and cannot resurrect an ineligible
category. Every's endpoint inclusion is subject to that separately explicit
boundary policy. The retained plan records all eligible and visible indices.

### Font Fit

| Public value | Constructors/variants |
| --- | --- |
| `TextFitGrouping` | `Independent`, `Uniform` |
| `TextFitPriority` | `ContentFirst`, `PreferredSizeFirst` |
| `TextFit` | `fixed() -> Self`; `shrink_to(minimum: f64) -> Result<Self>`; `range(minimum: f64, maximum: f64) -> Result<Self>`; `grouping(self, value: TextFitGrouping) -> Self` |

The factories use positive finite CSS sizes. Fixed uses the complete appearance's
preferred size; shrink resolves [minimum, preferred]; range allows both growth
and shrink inside its explicit interval. A preferred value outside an explicit
range is a preference, not permission to try an out-of-range size. An impossible
resolved interval fails as a real constraint. Equal bounds normalize to fixed at
that bound, not to a different inherited preferred size.
For priority ordering, the preferred permitted size is the preferred appearance
size clamped to the resolved interval. This does not remove any other permitted
sizes from the declared fitting program.

An explicit fitting factory initially fits members independently, as in ggfittext;
Uniform selects one size within the common allowed interval for the actual
owner/role group. It does not couple separate axes, independent authored slots or
light/dark contrast roles. Group membership and every consumed assignment are
dependencies. ContentFirst is the confirmed combination default; PreferredSizeFirst
tries permitted abbreviation at the preferred permitted size before reducing it.
Terminal handling comes only from TextOverflow, not a competing fit-failure field.

## Text Role Attachments And Capabilities

All following edits consume `self`, take the listed arguments and return `Self`.
They retain one request at the actual owner. Authored Text's present whole request
replaces its role default; it is not a fieldwise layout cascade. Shared Canvas
guide chrome consumes Canvas instructions; replaced local guide layouts are not
another contributing default. Unused roles add no font or shaping work.

| Receiver/bound | Method and arguments |
| --- | --- |
| `Plot<K: kind::XAxisText>` | `x_tick_text_layout(layout: TextLayout)` |
| `Plot<K: kind::YAxisText>` | `y_tick_text_layout(layout: TextLayout)` |
| `Plot<K: kind::Facet>` | `facet_row_text_layout(layout: TextLayout)`; `facet_column_text_layout(layout: TextLayout)` |
| `Plot<kind::Matrix>` | `matrix_row_text_layout(layout: TextLayout)`; `matrix_column_text_layout(layout: TextLayout)` |
| `Plot<K: kind::Legend>` | `guide_text_layout(role: GuideTextRole, layout: TextLayout)` |
| `Canvas` | `guide_text_layout(role: GuideTextRole, layout: TextLayout)` |
| `PointLayer` | `label_text_layout(layout: TextLayout)` |
| `Plot<kind::Heatmap>` | `value_text_layout(layout: TextLayout)`; `null_row_text_layout(layout: TextLayout)` |
| `Plot<K: kind::EstimateDetailText>` | `estimate_detail_text_layout(layout: TextLayout)` |
| `Plot<kind::Waterfall>` | `total_text_layout(layout: TextLayout)` |
| `Plot<K: kind::SignificanceStyle>` | `significance_text_layout(layout: TextLayout)` |

`GuideTextRole` is the closed enum `Title`, `Entry`, `Number`; it addresses real
guide roles, not arbitrary string keys. Title/Entry/Number can be independently
configured even when their omitted Theme appearance is shared.

New sealed `XAxisText` and `YAxisText` capabilities cover the current exact plot
families' corresponding coordinate axes, including Matrix cell axes. Matrix row
and column methods configure its actual left-edge and bottom-edge variable titles;
x/y tick methods configure the corresponding native cell-axis labels. The KDE
diagonal has no separate heading producer and gains no unused attachment. This
adds no Matrix numeric format or generator attachment. New `EstimateDetailText`
covers PointInterval, Coefficient and Forest, the actual unavailable-row/adjacent-
detail consumers in `src/plots/interval_estimate.rs`; numeric estimate-label
formatting remains Coefficient-only. Dumbbell endpoint text is guide-entry text,
not an estimate-detail producer, and uses its guide attachment.
None of these capabilities is implemented by erased Any. Applicability for actual
numeric/category axes remains separate from the text capability.

Text inputs own layout for prose, TextPanel, Canvas headers/row titles and authored
annotations. There is no additional string-keyed role dispatcher for those slots.
Exact tick strings receive their axis layout while preserving verbatim source.

## Numeric Formatting Types And Methods

### Core Policy

Retain `NumericFormat::new(notation: NumericNotation, precision: NumericPrecision) -> Result<Self>`.
Retain its current seven edits after removing `with_`:

| Edit on `NumericFormat` | Paired reader |
| --- | --- |
| `notation(self, value: NumericNotation) -> Result<Self>` | `get_notation(&self) -> NumericNotation` |
| `precision(self, value: NumericPrecision) -> Result<Self>` | `get_precision(&self) -> NumericPrecision` |
| `decimal_scale(self, value: i16) -> Result<Self>` | `get_decimal_scale(&self) -> i16` |
| `prefix(self, value: impl Into<String>) -> Result<Self>` | `get_prefix(&self) -> Option<&str>` |
| `suffix(self, value: impl Into<String>) -> Result<Self>` | `get_suffix(&self) -> Option<&str>` |
| `negative_zero(self, value: NegativeZero) -> Result<Self>` | `get_negative_zero(&self) -> NegativeZero` |
| `trailing_zeros(self, value: TrailingZeros) -> Result<Self>` | `get_trailing_zeros(&self) -> TrailingZeros` |

The decimal shift remains -324..=324. Literal prefix/suffix keep their existing
nonempty control-free grammar; they do not imply units or scaling. Add these edits:

| Method on `NumericFormat` | Return |
| --- | --- |
| `multiplier(self, value: f64)` | `Result<Self>` |
| `rounding(self, value: NumericRounding)` | `Result<Self>` |
| `locale(self, value: NumericLocale)` | `Result<Self>` |
| `grouping(self, value: bool)` | `Result<Self>` |
| `sign(self, value: NumericSign)` | `Result<Self>` |
| `symbol(self, value: NumericSymbol)` | `Result<Self>` |
| `field(self, value: Option<NumericField>)` | `Result<Self>` |
| `rate(self, value: NumericRate)` | `Result<Self>` |
| `prefixes(self, value: NumericPrefixes)` | `Result<Self>` |
| `unit(self, value: Option<NumericUnit>)` | `Result<Self>` |
| `explanation(self, value: NumericExplanation)` | `Self` |
| `builtin(self)` | `Self`, selecting built-in emission while retaining all other fields |

`NumericFormat::from_specifier(specifier: &str) -> Result<Self>` is the one public
parser factory. It returns this same type. No public standalone `format(value)`
API is introduced by callback helpers. Existing numeric attachment signatures
continue accepting NumericFormat under their existing capabilities.

| Choice | Exact variants/constructor |
| --- | --- |
| `NumericNotation` | Retain `Decimal`, `Scientific`, `Engineering`; add `General`, `Integer(NumericRadix)` |
| `NumericRadix` | `Decimal`, `Binary`, `Octal`, `HexLower`, `HexUpper` |
| `NumericPrecision` | `Fractional(u8)`, `Significant(u8)`, `AdaptiveFractional { min: u8, max: u8 }`, `AdaptiveSignificant { min: u8, max: u8 }`; fractional bounds 0..=20, significant bounds 1..=21 |
| `NegativeZero` | Retain `Preserve`, `Normalize` |
| `TrailingZeros` | Retain `Keep`, `Trim` |
| `NumericRounding` | `NearestEven`, `NearestAway`, `NearestTowardZero`, `TowardZero`, `AwayFromZero`, `Floor`, `Ceiling` |
| `NumericSign` | `NegativeOnly`, `Always`, `Space`, `Accounting` |
| `NumericSymbol` | `None`, `Currency`, `RadixPrefix` |
| `NumericAlignment` | `Left`, `Right`, `Center`, `AfterSign` |
| `NumericField` | `new(width: usize, fill: char, alignment: NumericAlignment) -> Result<Self>` |
| `NumericRate` | `Value`, `Percent`, `PerMille` |

General notation uses significant precision: after the single quantization, use
fixed decimal if -4 <= the resulting decimal exponent < selected precision,
otherwise scientific. Decimal, Scientific and Engineering keep their current
fractional/significant meanings. Integer notation uses Fractional(0) and the
selected rounding mode, never an unchecked integer cast. A typed incompatible
precision/notation request fails construction; parser-specific normalization of
irrelevant integer precision is explicit below.

Value contributes an exact factor of 1, Percent multiplies by 100 and PerMille
multiplies by 1000, with percent/per-mille suffixes from the selected locale.
These factors multiply the general multiplier and decimal scale before the
selected prefix divisor and the one quantization. Literal prefix/suffix never
imply a rate or scientific unit.
RadixPrefix emits 0b/0o/0x for the corresponding nondecimal integer radix; currency
uses literal locale affixes. Their deliberate inapplicable syntax normalizations
are listed below, rather than hidden behind another formatter.
Typed RadixPrefix requires a nondecimal integer notation; other typed combinations
reject through the complete validator. The parser removes its documented unused
`#` syntax before invoking that same validator.

Default NumericFormat retains Decimal/Fractional(2), multiplier 1, decimal shift
0, NearestEven, NegativeOnly, NumericSymbol::None, no field/grouping, Value rate,
no selected prefixes/unit and built-in emission. NegativeZero/TrailingZeros keep
their current defaults. New explicit options do not replace omitted consumer
profiles or their historically specified k/M division operands.

### Locale Registry And Complete Custom Values

The named registry is all locale definitions in **d3-format 3.1.0**, commit
`08806c62aa8f7b3fe64925f3989f1491c950a71c`, whose
[locale directory](https://github.com/d3/d3-format/tree/08806c62aa8f7b3fe64925f3989f1491c950a71c/locale)
and [license](https://github.com/d3/d3-format/blob/08806c62aa8f7b3fe64925f3989f1491c950a71c/LICENSE)
are the provenance. Its exact names are:

```text
ar-001 ar-AE ar-BH ar-DJ ar-DZ ar-EG ar-EH ar-ER ar-IL ar-IQ ar-JO ar-KM ar-KW
ar-LB ar-LY ar-MA ar-MR ar-OM ar-PS ar-QA ar-SA ar-SD ar-SO ar-SS ar-SY ar-TD
ar-TN ar-YE ca-ES cs-CZ da-DK de-CH de-DE en-CA en-GB en-IE en-IN en-US es-BO
es-ES es-MX fi-FI fr-CA fr-FR he-IL hu-HU it-IT ja-JP ko-KR mk-MK nl-NL pl-PL
pt-BR pt-PT ru-RU sl-SI sv-SE uk-UA zh-CN
```

Transcribe this fixed catalog to first-party Rust constants during implementation,
including source/license attribution and source-to-constant checks. No runtime
JSON loading, locale dependency, host/environment lookup or provisioning is added.
Missing upstream numeral/sign fields use that pinned formatter's documented
defaults; plus and per-mille defaults are `+` and U+2030. D3's grouping arrays use
Cycle semantics. Custom grouping also supports repeated-last and terminating
patterns, as used by [Matplotlib ScalarFormatter.useLocale](https://matplotlib.org/stable/api/ticker_api.html#matplotlib.ticker.ScalarFormatter)
through [Python localeconv grouping](https://docs.python.org/3/library/locale.html#locale.localeconv).
Last and Stop express those repeat/termination conventions directly, without
embedding platform-specific 0/CHAR_MAX sentinels in the positive-size vector.

| Type | Public construction/read surface |
| --- | --- |
| `NumericLocale` | `invariant() -> Self`; `named(name: &str) -> Result<Self>`; `named_ids() -> &'static [&'static str]` |
| `NumericLocale` | `custom(decimal: impl Into<String>, digits: [String; 10], grouping: Option<NumericGrouping>, symbols: NumericSymbols, currency: NumericCurrency) -> Result<Self>` |
| `NumericLocale` readers | `decimal(&self) -> &str`; `digits(&self) -> &[String; 10]`; `grouping(&self) -> Option<&NumericGrouping>`; `symbols(&self) -> &NumericSymbols`; `currency(&self) -> &NumericCurrency` |
| `NumericGroupRepeat` | `Cycle`, `Last`, `Stop` |
| `NumericGrouping` | `new(separator: impl Into<String>, sizes: Vec<usize>, repeat: NumericGroupRepeat) -> Result<Self>`; readers `separator(&self) -> &str`, `sizes(&self) -> &[usize]`, `repeat(&self) -> NumericGroupRepeat` |
| `NumericSymbols` | `new(positive: impl Into<String>, negative: impl Into<String>, percent: impl Into<String>, per_mille: impl Into<String>) -> Result<Self>`; `positive(&self) -> &str`; `negative(&self) -> &str`; `percent(&self) -> &str`; `per_mille(&self) -> &str` |
| `NumericCurrency` | `new(prefix: impl Into<String>, suffix: impl Into<String>) -> Result<Self>`; `prefix(&self) -> &str`; `suffix(&self) -> &str` |

Invariant is the explicit-policy default: ASCII digits, decimal `.`, western
grouping `[3]` with `,` available but disabled in NumericFormat, signs `+`/`-`,
`%`, U+2030 and currency `$`/empty suffix. This preserves current plain numeric
spelling; choosing named en-US explicitly uses the pinned named-locale data.
Names match the published IDs exactly, and unknown IDs fail without host fallback.
Digits are ten nonempty distinct tokens, grouping sizes are positive, and tokens
consume their field-appropriate Unicode syntax. A locale with no grouping pattern
performs no grouping even when a format requests grouping, as in d3-format.

### SI, Compact, Factors And Units

`NumericSelection<T>` is the closed enum `PerValue`, `WholeSet`, `Fixed(T)`.
`NumericPromotion` is `KeepThreshold` or `PromoteOnRounding`. The relevant factory
requires that choice, so no unspecified promotion default becomes a restriction.

| Value | Exact surface |
| --- | --- |
| `SiPrefix` | `from_exponent(exponent: i8) -> Result<Self>`; `exponent(&self) -> i8`; `symbol(&self) -> &str` |
| `NumericCompactRule` | `new(threshold: f64, divisor: f64, prefix: impl Into<String>, suffix: impl Into<String>) -> Result<Self>`; `threshold(&self) -> f64`; `divisor(&self) -> f64`; `prefix(&self) -> &str`; `suffix(&self) -> &str` |
| `NumericFactor` | `AutomaticDecimal`, `Value(f64)`, `PowerOfTen(String)` |
| `NumericPrefixes` | `none() -> Self`; `si(selection: NumericSelection<SiPrefix>, promotion: NumericPromotion) -> Result<Self>`; `decimal_compact(selection: NumericSelection<usize>, promotion: NumericPromotion) -> Result<Self>`; `compact(rules: Vec<NumericCompactRule>, selection: NumericSelection<usize>, promotion: NumericPromotion) -> Result<Self>`; `factor(value: NumericFactor) -> Result<Self>` |
| `NumericUnitPlacement` | `PerLabel`, `Shared` |
| `NumericUnit` | `new(text: impl Into<String>, placement: NumericUnitPlacement) -> Self`; `separator(self, value: impl Into<String>) -> Self`; `text(&self) -> &str`; `get_separator(&self) -> &str`; `placement(&self) -> NumericUnitPlacement` |
| `NumericExplanation` | `automatic() -> Self`; `text(value: impl Into<Text>) -> Self`; `style(self, value: TextTheme) -> Self`; `layout(self, value: TextLayout) -> Self` |

SI covers the [BIPM prefix table](https://www.bipm.org/en/measurement-units/si-prefixes),
including the 2022 additions and neutral exponent 0. Fixed prefixes also admit
deci/centi/deca/hecto; automatic engineering selection uses multiples of 3 from
-30 through 30. Micro is U+00B5, matching the checked formatter/scales precedent.
Automatic selection saturates at the end prefix. PerValue selects individually;
WholeSet uses the greatest absolute nonzero scaled member; all-zero sets use 0.
Unrounded threshold selection uses absolute magnitudes. Carry comparisons use
the original signed operand and selected rounding cells, so floor/ceiling behave
correctly for both signs. They do not quantize a positive surrogate and attach
the sign afterward.

Compact thresholds are strictly increasing finite values and divisors are finite
positive values. An empty rule list is equivalent to None for PerValue/WholeSet;
Fixed must index an existing rule. Below the first threshold use factor 1 and empty
affixes. `decimal_compact` is the scales short-scale convenience with exact
threshold/divisor pairs 1000/K, 1000000/M, 1000000000/B, 1000000000000/T, empty
prefixes and those literal suffixes, retaining the reference's uppercase K.
Lowercase k, spaces and other names use custom rules. It normalizes to the same
custom-rule policy, not a separate emitter. Fixed selects the declared rule index.
KeepThreshold uses the unrounded mathematical magnitude,
as in [scales scale cuts](https://github.com/r-lib/scales/blob/v1.4.0/R/label-number.R).
PromoteOnRounding supports carry promotion, as in
[D3 SI formatting](https://github.com/d3/d3-format/blob/08806c62aa8f7b3fe64925f3989f1491c950a71c/src/formatPrefixAuto.js):
select the final rule using independent rounding-cell boundaries and the original
exact operand, then quantize once at that final rule. Never reround an emitted
mantissa. WholeSet starts from the greatest-magnitude unrounded rule. With explicit
promotion, resolve the common rule against all members' signed rounding cells,
retaining the furthest required promotion independent of member order; then freeze
that factor before any label quantization. Equal-magnitude opposite signs cannot
make it depend on whichever member was visited first, and individual labels cannot
promote it again. Fixed requires KeepThreshold, since
the caller has already selected the prefix/rule; construction rejects the
contradictory PromoteOnRounding request rather than ignore an option.

Factor(Value) uses the exact represented nonzero finite divisor; PowerOfTen
retains a mathematical power, not a physical f64. Its string is a signed ASCII
integer (`[+-]?[0-9]+`); factor construction normalizes leading zeros/plus and
negative zero. Use this exact decimal interchange for arbitrarily sized integral
exponents, matching the existing certified-power engine's BigInt exponent, rather
than impose an i64 bound or expose num-bigint as public API. AutomaticDecimal
selects the power of ten of the greatest absolute nonzero member, or factor 1 for
all zero, following the ordinary scientific common-factor convention. Prefix/factor
selection is one closed mode; choosing another replaces it, without stacking
competing selectors. It composes exactly with display multiplier/rate/decimal shift.

Unit placement controls repeated versus common factor/unit text. Shared placement
requires one complete-set factor; if per-value selection yields different factors,
that requested common representation cannot be resolved. Unit separators default
to one ordinary space, are literal and configurable, and are not prepended to an
otherwise empty standalone explanation. An empty unit with a shared factor gives
the `1,2,3` plus `×10⁶` workflow. With no unit request, selected prefixes stay on
each label. A configured authored explanation replaces generated content in the
same slot; styling/layout edits modify that one slot, without adding another unit.

### Specifier Normalization And Emission Order

The reference is pinned
[d3-format locale/normalization](https://github.com/d3/d3-format/blob/08806c62aa8f7b3fe64925f3989f1491c950a71c/src/locale.js)
and its [type table](https://github.com/d3/d3-format/blob/08806c62aa8f7b3fe64925f3989f1491c950a71c/src/formatTypes.js).
The parser supplies the following explicit fields, retaining invariant locale and
renderer rounding/negative-zero defaults rather than ECMAScript arithmetic:

| Type | Notation | Precision when omitted | Additional normalization |
| --- | --- | --- | --- |
| `f` | Decimal | Fractional(6) | Value rate |
| `e` | Scientific | Fractional(6) | Value rate |
| `g` | General | Significant(6) | Value rate |
| `r` | Decimal | Significant(6) | Fixed-decimal significant emission |
| `s` | Decimal | Significant(6) | PerValue SI, PromoteOnRounding |
| `%` | Decimal | Fractional(6) | Percent rate |
| `p` | Decimal | Significant(6) | Percent rate |
| `d`, `b`, `o`, `x`, `X` | Integer with selected radix | Fractional(0) | Parsed precision is irrelevant and normalizes to 0, as in the reference integer grammar |
| `n` | General | Significant(6) | Grouping enabled |
| Omitted | General | Significant(12) | Trimming enabled; an explicit precision replaces 12 |

Default sign is NegativeOnly, fill is space, alignment is Right, field width is
absent, grouping is off and symbol is None. `<`, `>`, `^`, `=` map to the four
NumericAlignment values. Sign `-`, `+`, space and `(` map to NumericSign.
`$` selects Currency; `#` selects RadixPrefix for b/o/x/X and normalizes to None
for the other types, matching the reference's documented lack of a radix affix.
`0`, or fill `0` with `=`, normalize to zero fill AfterSign. Comma enables grouping;
`~` selects Trim. Width is a checked usize scalar count, with no string budget.
Explicit fractional/significant precision outside the supported bounds rejects
instead of D3's clamping. Unknown types, malformed/trailing input and text-coercing
`c` reject as already specified; omitted type remains supported.
Normalize irrelevant integer precision before applying fractional/significant
bounds; do not impose a u8 parse limit on a field the integer grammar discards.
Without width there is no NumericField, and otherwise unused fill/alignment/zero
syntax normalizes away. A zero-width typed field requests no padding, without
changing numeric precision or display content.

Currency and rate composition follows the pinned suffix rule: `$` supplies both
locale currency affixes and replaces the automatically emitted percent/per-mille
suffix, without removing its exact rate multiplier. For example `$.1%` still
multiplies by 100; a caller wanting an additional literal `%` can request the
ordinary suffix explicitly. Center padding puts an odd extra fill scalar on the
right. Custom fill and locale tokens follow their single-line numeric grammar,
not prose mandatory-break acceptance.

Typed and parsed equivalent policies have equal identity. For example `.3s`
normalizes to Decimal/Significant(3), PerValue SI with carry promotion; `.1%` to
Decimal/Fractional(1) and Percent rate, not merely a literal `%` suffix.

One emission sequence resolves exact factors, selected prefix divisor and precision;
quantizes once; selects notation/carry from that retained result; splits sign,
integer, fraction and exponent runs; groups numeric integer digits; applies
locale decimal/rate/currency/sign tokens, prefix/unit/affixes and accounting
closure; then fills/aligns the field. Existing literal prefix/suffix enclose that
presentation, with prefix before the sign and suffix after accounting closure.
Selected compact prefix/suffix surround the numeric presentation inside those
literal affixes. A supplied SI unit combines the symbol with its unit (`km`), with
the unit separator before that combined unit (`1 km`); without a unit the prefix
remains the ordinary numeric suffix (`1k`). Other literal units follow the complete
number/prefix text using their declared separator. Shared placement moves the
selected common factor/unit text to its one explanation rather than repeat it.
Zero/sign-aware padding participates in grouping as in the pinned reference.
Locale numeral substitution applies only to numeric digit runs, including
exponent/zero-padding digits, not literal affixes, currency/unit text or custom
fill. Field width counts final Unicode scalars,
including multiscalar numeral tokens; fill is sized after those lengths are known.
Do not split a token to hit width or reprocess final callback strings.

## Numeric Formatting Functions

Expose these consuming emission-mode edits on NumericFormat:

```rust
pub fn value_function<F>(self, function: F) -> Self
where
    F: for<'a> Fn(&NumericValueContext<'a>) -> Result<String>
        + Send + Sync + 'static;

pub fn set_function<F>(self, function: F) -> Self
where
    F: for<'a> Fn(&NumericSetContext<'a>) -> Result<NumericLabels>
        + Send + Sync + 'static;
```

They select one owned immutable shared handle, replacing the previous emission
mode while retaining built-in options. `builtin()` clears that selection. Closures
run serially; the bounds preserve thread-safe Plot cloning/storage. Applications
can capture owned state; no mutable renderer/context escapes. These declarations
are method signatures, not standalone free functions.
Keep NumericFormat's `Clone + Debug + Default + PartialEq + Eq` contract. Debug
need not print captured application state; equality uses normalized fields and
shared handle identity, never closure output. NumericOperand/Identity and the
library-created borrowed views are Clone + Debug; returning a value view does not
clone a renderer, callback or materialized numeric set.

`NumericLabels` owns Vec<String> and optional Text explanation:
`new(labels: Vec<String>) -> Self`, `explanation(self, value: impl Into<Text>) -> Self`,
`labels(&self) -> &[String]`, `get_explanation(&self) -> Option<&Text>`, and
`into_parts(self) -> (Vec<String>, Option<Text>)`. No returned label is postformatted.
A callback explanation replaces configured/generated content in the one common
slot using ordinary Text replacement rules; absence retains the configured result.

### Read-Only Numeric Views

| View/export | Surface |
| --- | --- |
| `CoordinateAxis` | `X`, `Y` |
| `NumericRole` | `XAxis`, `YAxis`, `ContinuousColor`, `HeatmapValues`, `CoefficientLabels`, `WaterfallTotals` |
| `NumericPurpose` | `IntrinsicProbe`, `ActualSet` |
| `NumericRangeOrigin` | `Automatic`, `Explicit`, `ParentResolved` |
| `NumericOwner<'a>` | `plot_kind(&self) -> Option<PlotKind>`; `canvas_cell(&self) -> Option<(usize, usize)>`; `facet_cell(&self) -> Option<&'a FacetCell>`; `is_shared(&self) -> bool` |
| `NumericIdentity<'a>` | `Tick { axis: CoordinateAxis, source_index: usize }`, `GuideTick { source_index: usize }`, `HeatmapCell { row: &'a str, column: &'a str }`, `EstimateRow { category: &'a str }`, `WaterfallBase`, `WaterfallTerminal`, `Probe { index: usize }` |
| `NumericOperand<'a>` | `Binary64(f64)`, `PowerOfTen { exponent: f64 }`, `Multiplicative { anchor: f64, ratio: f64, index: &'a str }` |
| `NumericValue<'a>` | `operand(&self) -> NumericOperand<'a>`; `identity(&self) -> NumericIdentity<'a>` |
| `NumericDomain<'a>` | `lower(&self) -> NumericOperand<'a>`; `upper(&self) -> NumericOperand<'a>`; `data_limits(&self) -> Option<(f64, f64)>`; `scale_limits(&self) -> (f64, f64)`; `scale(&self) -> AxisScale`; `origin(&self) -> NumericRangeOrigin` |
| `NumericFactors<'a>` | `multiplier(&self) -> f64`; `decimal_scale(&self) -> i16`; `rate(&self) -> NumericRate`; `divisor(&self) -> &'a NumericFactor`; `prefix(&self) -> &'a str`; `suffix(&self) -> &'a str`; `unit(&self) -> Option<&'a NumericUnit>` |
| `NumericSetContext<'a>` | `owner(&self) -> NumericOwner<'a>`; `role(&self) -> NumericRole`; `purpose(&self) -> NumericPurpose`; `values(&self) -> &'a [NumericValue<'a>]`; `domain(&self) -> Option<NumericDomain<'a>>`; `locale(&self) -> &'a NumericLocale`; `common_factors(&self) -> Option<NumericFactors<'a>>`; `format_builtin(&self) -> Result<NumericLabels>` |
| `NumericValueContext<'a>` | `set(&self) -> &NumericSetContext<'a>`; `index(&self) -> usize`; `value(&self) -> &NumericValue<'a>`; `factors(&self) -> NumericFactors<'a>`; `format_builtin(&self) -> Result<String>` |

Operand f64 fields retain represented values, including source signed zero.
Multiplicative retains the supplied positive anchor/ratio and an exact signed
integer index rendered as canonical ASCII decimal; it does not expose a lossy
float index or an arbitrary public machine-integer ceiling. Ratio directions may
be normalized internally, but this view must still describe the exact operand.
NumericFactor::AutomaticDecimal is an input request; a resolved divisor never
returns that variant. Mathematical powers are not materialized through powf.

Domain data_limits is present only for genuinely retained finite raw endpoints.
Scale limits are scientific coordinates, not CSS geometry; no inverse of a rounded
logarithm is invented. Raw identity strings are semantic keys, before display
aliases. Owner descriptions identify semantic sources, not kernel/backend handles.
Complete private owner/domain provenance still participates in operation identity.
Make existing FacetCell identity inspectable for these contexts through
`single_level(&self) -> Option<&str>` and
`grid_levels(&self) -> Option<(&str, &str)>`. They borrow the raw levels of its
existing Single/Grid representation, without another public facet-address type.

Helpers bypass only the callback emission mode and use the same built-in exact
engine, whole-set precision, factors, units and locale. Value helpers use their
complete semantic set, not isolated per-value adaptive precision. No helper calls
the user function recursively. Contexts cannot outlive their borrowed operation;
all returned strings/labels/Text are owned. Existing once-per-context/final-seal
rules apply independently of optional layout caching.

## Tick Generation And Category Selection

### Factories And Count Migration

`TickGeneration` has these target constructors/edits:

| Method | Complete meaning |
| --- | --- |
| `automatic() -> Self` | Default/reset locator with no explicit maximum |
| `additive(step: f64) -> Result<Self>` | Existing preferred, zero-origin data interval on linear axes |
| `log10_decades(step: u32) -> Result<Self>` | Existing preferred integral base-ten exponent interval |
| `target(count: usize) -> Result<Self>` | Existing approximate count, with positive count |
| `maximum(count: usize) -> Result<Self>` | Existing Automatic plus a positive explicit maximum |
| `exact_count(count: usize, space: TickSpace) -> Self` | Complete exact-count source; zero is empty, singleton initially Lower |
| `interval(step: f64, origin: f64, space: TickSpace, requirement: TickRequirement) -> Result<Self>` | Built-in fixed/preferred data or scale interval |
| `multiplicative(anchor: f64, ratio: f64, requirement: TickRequirement) -> Result<Self>` | Explicit anchored geometric source |
| `singleton(self, position: TickSingleton) -> Result<Self>` | Configure the ExactCount singleton choice |
| `maximum_count(self, maximum: Option<usize>) -> Result<Self>` | Set/clear the one positive generated maximum, checking an ExactCount contradiction |
| `get_maximum_count(&self) -> Option<usize>` | Read the same maximum for any primary locator |

`TickSpace` is `Data` or `Scale`; `TickRequirement` is `Fixed` or `Preferred`;
`TickSingleton` is `Lower`, `Midpoint`, `Upper`, `Position(f64)`. Singleton position
uses the selected coordinate space and applies only when the exact count is one.
For n>=2 the choice remains part of the complete request but cannot move endpoints.
`singleton` on a different primary locator is an inapplicable policy edit, not an
unused extra selection channel. Scalar configuration checks retain real finite/
positive/coherent requirements; no independent function-result validator is added.

Use usize for count arguments, cap values, `target_count(&self) -> Option<usize>`
and internal materialized lengths in the count-extension migration. Counts are
checked against actual platform/storage/output representation, not 10000. Existing
positive Target/Maximum meaning is preserved; ExactCount(0) is the explicit empty
constructor, not a new Automatic fallback. Clearing the cap does not change a
primary locator. `is_automatic(&self) -> bool` tests the complete default/reset.
`Some(0)` is not a positive maximum; use ExactCount(0) for an explicitly empty
source rather than reinterpret Maximum's existing contract.

Keep `additive_step(&self) -> Option<f64>` and `decade_step(&self) -> Option<u32>`
as projections of the existing constructor choices; also expose
`interval_parameters(&self) -> Option<(f64, f64, TickSpace, TickRequirement)>` and
`multiplicative_parameters(&self) -> Option<(f64, f64, TickRequirement)>` for the
extended choices. The normalized Interval retains the source's admitted scale
binding as well as its step/origin/space/requirement, so the old linear-only
Additive and Log10-only integral constructor contracts are not confused with a
generic context-relative Scale request. That binding is input semantics, not a
legacy implementation or public version selector; all use one source owner.

The full interval and multiplicative factories require the caller's origin/anchor
argument; examples use 0 and 1, without adding another implicit view-following mode.
Scale interval means the actual scientific scale coordinate: linear data on Linear,
exponent on Log10. Data interval always means raw data units. ExactCount's Data and
Scale choices follow the same distinction. Position/formatting callbacks retain
their explicit raw/mathematical operand forms.

### Owned Position Function

```rust
pub fn function<F>(function: F) -> Self
where
    F: for<'a> Fn(&TickContext<'a>) -> Result<TickPositions>
        + Send + Sync + 'static;
```

This is an associated TickGeneration factory. It owns one shared function handle.
`TickPositions` is `Data(Vec<f64>)` or `Log10Exponents(Vec<f64>)`;
`From<Vec<f64>>` means Data. No labels, styles, AxisTick or backend references are
part of the return type. Exponent results are for actual Log10 axes and retain
their mathematical value operands.

`TickContext<'a>` has read-only `axis(&self) -> CoordinateAxis`,
`domain(&self) -> NumericDomain<'a>`, `owner(&self) -> NumericOwner<'a>` and
`span(&self) -> f64` in actual assigned CSS pixels. It is produced only after the
final parent domain/assignment exists, without hypothetical width probes or a
density/preferred-count hint. No context mutates scientific input or layout.

Normal window/projection/layout handling consumes drawable results in original
occurrence order with a stable geometric view where needed. Unsorted, repeated
and off-window positions are not separate source errors. A function error or
actually failed operation ends rendering without retry/repair/default locator.
Maximum applies to the final in-domain count before first-party copies/formatting.

### Category Selection And Sealed Gates

`CategoryLabelSelection` is private-field `Clone + Debug + Default + PartialEq + Eq`.
It exposes `all() -> Self`, `every(stride: usize) -> Result<Self>`,
`offset(self, offset: usize) -> Result<Self>` and
`endpoints(self, value: LabelEndpoints) -> Self`. Every starts at offset 0 with
no forced endpoints. Stride 1/offset 0 normalizes to All. Endpoints identify the
first/last full scientific category, before any optional visibility reduction.
Offset must be less than the retained positive stride. All is normalized stride
1/offset 0, so only offset 0 applies to it; endpoint edits on All are redundant and
normalize back to All. For other strides retain the configured endpoint intent
even when the current domain happens to include those endpoints already.

Add the consuming Plot methods
`x_category_labels(self, value: CategoryLabelSelection) -> Self where K: kind::CategoricalX`
and
`y_category_labels(self, value: CategoryLabelSelection) -> Self where K: kind::CategoricalY`.
Keep current `Canvas::category_labels(CanvasCategoryLabels)` as the separate
whole-axis sharing instruction. Neither selection edits ticks, grid or marks.

New sealed `CategoricalX` is implemented by Trajectory, Interaction, Heatmap, Bar,
StackedBar, Boxplot, Violin and Raincloud. `CategoricalY` is implemented by
PointInterval, Coefficient, Forest, Dumbbell, Heatmap, HorizontalBar,
StackedHorizontalBar, Beeswarm and Waterfall. Conditional Trajectory still checks
its actual axis; Any and Matrix acquire no categorical selection attachment.
Existing NumericX/NumericY generation/formatting gates remain independent.

## Canvas Placement And Root Requests

Retain the exact planned consuming signature:

```rust
pub fn panel_span<K: kind::ExactFamily>(
    self,
    row: usize,
    column: usize,
    row_span: usize,
    column_span: usize,
    plot: Plot<K>,
) -> Result<Self>;
```

`panel` stays the 1x1 convenience on the same owner/occupancy path. New
`Plot::max_width(self, width: u32) -> Self` and
`Canvas::max_width(self, width: u32) -> Self` follow
the current `pub const fn`/`#[must_use]` builder style. Public root dimensions
remain u32 CSS pixels; PNG scaling is separate. Existing width/size methods and
Output dimension queries keep their signatures. Latest root sizing edit replaces
the complete request; embedded requests do not override Canvas assignment.

No public layout kernel, root constraints type or coordinator entry is exported.
`plan_root` is the private replacement specified by the implementation plan.

## Planned Call Examples

These examples use the target signatures. Move them into compiled public examples
as their corresponding capabilities land; they are not executable examples of
today's checkout. The behavior sections in the implementation plan supply the
independent numerical and geometric expectations.

### Equivalent SI Policies And A Common Decimal Factor

```rust
use lichtung_plot::{
    NumericFactor, NumericFormat, NumericNotation, NumericPrecision,
    NumericPrefixes, NumericPromotion, NumericSelection, NumericUnit,
    NumericUnitPlacement, Result,
};

fn policies() -> Result<(NumericFormat, NumericFormat)> {
    let typed = NumericFormat::new(
        NumericNotation::Decimal, NumericPrecision::Significant(3),
    )?.prefixes(NumericPrefixes::si(
        NumericSelection::PerValue, NumericPromotion::PromoteOnRounding,
    )?)?;
    assert_eq!(typed, NumericFormat::from_specifier(".3s")?);

    let millions = NumericFormat::new(
        NumericNotation::Decimal, NumericPrecision::Fractional(0),
    )?.prefixes(NumericPrefixes::factor(
        NumericFactor::PowerOfTen("6".to_owned()),
    )?)?.unit(Some(NumericUnit::new("", NumericUnitPlacement::Shared)))?;
    // The complete 1e6/2e6/3e6 set displays 1/2/3 with one ×10⁶ explanation.
    Ok((typed, millions))
}
```

### Formatting Functions Keep Built-In Helpers

```rust
use lichtung_plot::{NumericFormat, NumericIdentity};

let labels = NumericFormat::default().value_function(|context| {
    let ordinary = context.format_builtin()?;
    Ok(match context.value().identity() {
        NumericIdentity::Tick { source_index: 0, .. } => format!("start: {ordinary}"),
        _ => ordinary,
    })
});
let common_text = NumericFormat::default().set_function(|context| {
    Ok(context.format_builtin()?.explanation("Adjusted values"))
});
// Attach either policy through the existing x/y/value-format methods.
```

### A View-Dependent Position Source

```rust
use lichtung_plot::{Error, Result, TickGeneration, TickPositions};

fn warning_positions() -> Result<TickGeneration> {
    TickGeneration::function(|context| {
        let (lower, upper) = context.domain().data_limits().ok_or_else(|| {
            Error::Data("this application locator needs retained raw bounds".into())
        })?;
        let mut positions = vec![lower];
        if lower <= 50.0 && 50.0 <= upper {
            positions.push(50.0);
        }
        positions.push(upper);
        Ok(TickPositions::Data(positions))
    }).maximum_count(Some(100))
}
```

The error in this example is application code's requested raw-domain requirement,
not a library result validator. A Log10 application can instead deliberately
return `TickPositions::Log10Exponents` using the context's retained scale bounds.

### Independent Category Selection And Text Layout

```rust
use lichtung_plot::{
    kind, CategoryLabelSelection, LabelEndpoints, Plot, Result, TextLayout,
    TextRotation,
};

fn month_labels(plot: Plot<kind::Bar>) -> Result<Plot<kind::Bar>> {
    Ok(plot.x_category_labels(
        CategoryLabelSelection::every(2)?.offset(0)?.endpoints(LabelEndpoints::None),
    ).x_tick_text_layout(
        TextLayout::default().rotation(TextRotation::tilt_up()),
    ))
}
```

### A Fixed Data Reference And Bounded TextPanel Title

```rust
use lichtung_plot::{
    DataCoordinate, PanelTextAnnotation, Result, Text, TextAnchor, TextBounds,
    TextCoordinate, TextElision, TextEllipsis, TextFailure, TextFit,
    TextFitPriority, TextLayout, TextOffset, TextOffsetFrame, TextOverflow,
    TextPanel, TextPosition, TextRotation,
};

fn authored_text() -> Result<(PanelTextAnnotation, TextPanel)> {
    let note = PanelTextAnnotation::at(
        TextPosition::new(
            TextCoordinate::data(DataCoordinate::numeric(2.0)?),
            TextCoordinate::data(DataCoordinate::numeric(3.0)?),
        ),
        Text::new("Fixed data reference").layout(TextLayout::default()
            .anchor(TextAnchor::bottom_left())
            .offset(TextOffset::new(0.0, 0.0, TextOffsetFrame::Screen)?)
            .rotation(TextRotation::degrees(30.05)?)),
    )?;
    let title = Text::new("Treatment group with extended follow-up").layout(
        TextLayout::default()
            .bounds(TextBounds::new(Some(140.0), None, Some(2))?)
            .fit(TextFit::range(10.0, 14.0)?)
            .fit_priority(TextFitPriority::ContentFirst)
            .overflow(TextOverflow::ellipsis(
                TextEllipsis::new(TextElision::End).max_lines(Some(2)),
                TextFailure::Error,
            )),
    );
    Ok((note, TextPanel::new(title, "Independent complete body")?))
}
```

## Delivery And API Acceptance

The implementation plan's 22 steps implement these signatures with their real
consumers. Before each public exposure, update the owning current contracts,
repository callers, examples and relevant compile-fail documentation together.
All listed context/choice names must be exported and usable without importing a
private module; every method must have a live owner and complete identity.

Verification includes independent public construction for every policy family,
parsed/typed policy equality, both callback result forms, lifetime/thread/clone
contracts, read capability preservation and the deliberate Copy/type removals.
Choose focused semantic witnesses and compile-fail capability checks within the
three-layer test limits in `90_DEVELOPMENT.md`, not an accessor echo suite. Literal
examples use real planned names; every method, callback context and scalar type
has a specified signature rather than a placeholder. Algorithm proofs and focused
strict-Clippy gates remain the implementation plan's obligations, not claims that
this appendix has implemented or compiled the future API.
