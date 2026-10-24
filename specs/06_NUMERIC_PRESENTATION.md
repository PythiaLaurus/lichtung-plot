# Numeric Presentation

## Ownership

NumericFormat controls spelling; TickGeneration controls complete generated axis
sources. Both are Plot-local immutable policies, separate from scientific data/
transforms, complete Theme typography, exact caller ticks, guide composition and
root sizing. Neither changes scientific values, statistics, availability, domain
provenance, clipping, source identities or color mapping.

There is one normalized rounding/emission engine with private consumer-default
profiles and explicit mathematical operand kinds. No compatibility formatter,
runtime version selector, host/environment locale inference or Canvas format
override is provided. Exact ticks,
categories, guide identities, significance, PointLayer/panel/null-row strings and
Waterfall names are never parsed as numeric data. Data-alpha mapping remains a
separate [paint contract](04_VISUAL_PROPERTY_OWNERSHIP.md).

Source owners are `src/numeric/format.rs`, `src/numeric/decimal.rs`,
`src/numeric/profiles.rs`, `src/numeric/set.rs`, `src/numeric/inputs.rs`,
`src/numeric/locale.rs`, `src/numeric/locale/data.rs`, `src/numeric/emission.rs`,
`src/numeric/prefixes.rs`, `src/numeric/prefixes/selection.rs`, `src/numeric/specifier.rs`,
`src/numeric/context.rs`, `src/numeric/function.rs`, `src/numeric/table.rs`,
`src/numeric/source.rs`, `src/numeric/unit.rs`,
`src/spec/ticks.rs`, `src/layout/panel/sources.rs` and the shared
`src/layout/panel/sources/{count,interval,geometric}.rs` position owners. Mathematical power
display is specified in [certified conversion](07_LOG10_DISPLAY.md); geometric fitting
and sealed text preparation in [layout](05_LAYOUT_SYSTEM.md).

## Formatting Functions And Semantic Materialization

`value_function` and `set_function` replace only emission, retaining the normalized
built-in fields. They accept higher-ranked fallible Fn views with Send + Sync +
'static bounds; Plot clones share an immutable handle. Equal handles plus equal
normalized fields compare equal; independent handles never gain equality from
their output, address, closure type or Debug. `builtin()` clears the handle alone.
Calls are serial in owner/source order. Applications must return stable results
for stable semantic inputs; results are frozen before publication.

Borrowed NumericSetContext exposes owner, role, ActualSet/IntrinsicProbe purpose,
ordered values/identities including duplicates, optional resolved domain, locale
and common factors. Value contexts add original occurrence index and member factors.
Raw Canvas cells, FacetCell single/grid levels, category keys and signed operand
bits precede any aliases. Ordinary operands are Binary64; Forest positions and
logarithmic domain endpoints are mathematical PowerOfTen exponents. Geometric
members retain the original anchor/ratio and canonical arbitrary-size signed index
in NumericOperand::Multiplicative, borrowing its decimal index text. Raw bounds are
present only when retained, never reconstructed by inverting a rounded logarithm.

Factors/unit/explanation metadata resolve before callback invocation. Built-in
helpers bypass only the callback and lazily convert the complete enclosing set,
with one adaptive precision and the already exposed frozen factors. Ignored helpers
perform no numeric emission or field-padding allocation. Helper errors remain
fallible; successful results are reused within the invocation. In function mode,
automatic promotion resolves at the policy's minimum precision before invocation;
helper precision refinement cannot replace those exposed factors.

Returned strings are final authored display text: no trimming, digit substitution,
humanization or second numeric rounding. NumericLabels must supply exactly one
string per occurrence. An optional Text replaces the one common explanation with
ordinary content/style/layout retention rules. Collisions retain consumer-specific
complete-source behavior, never hidden callback retries at another precision.
Function errors are fatal with semantic owner/source context. Empty owners invoke
no function and contribute no explanation. Exact ticks suppress all numeric-function
work for that axis, including intrinsic probes.

The operation-local PresentationTable keys complete normalized policy/handle,
owner, purpose, original ordered identities/bits, resolved domain/provenance,
generation and eligibility window. It is required semantic storage, independent
of optional layout caches. Repeated candidate queries consume the retained result;
distinct reached sets/domain consequences can invoke new contexts. Shared color
guides suppress replaced local formatting and invoke only after compatible policies
and final domains merge. Actual returned text is admitted before measurement.
Final accepted-root sealing clears candidate storage and rejects further
materialization; selected results/context survive in immutable plans. Null, SVG,
outlining, PNG and Output replay invoke no numeric functions or font planning.

## NumericFormat Value

NumericFormat is opaque Clone/Debug/PartialEq/Eq with private normalized
fields. A consuming `field(value)` edit returns Result and replaces only its
field, validating the complete result. Its paired `get_field()` reader retains
the existing borrowed/copy result and const-call capability where applicable.

| Field | Input contract |
| --- | --- |
| notation | Decimal, Scientific, Engineering, General, Integer(NumericRadix) |
| precision | Fractional(u8), Significant(u8), AdaptiveFractional{min,max}, AdaptiveSignificant{min,max} |
| decimal_scale | i16 in -324..=324, exact decimal-power shift |
| prefix / suffix | Optional owned literal text, nonempty and control-free when supplied |
| negative_zero | Preserve or Normalize |
| trailing_zeros | Keep or Trim |
| rounding | NearestEven (default), NearestAway, NearestTowardZero, TowardZero, AwayFromZero, Floor, Ceiling |
| multiplier | Finite signed f64, exact represented factor, default 1; either signed zero normalizes to positive zero |
| locale | Complete NumericLocale, default invariant ASCII; named pinned or custom |
| grouping | bool, default false; a missing locale pattern remains ungrouped |
| sign | NegativeOnly (default), Always, Space, Accounting |
| symbol | None (default), Currency, RadixPrefix; radix prefix requires nondecimal Integer |
| field | Optional NumericField(width: usize, fill: char, NumericAlignment); no width quota |
| rate | Value (default), Percent, PerMille; exact factors 1,100,1000 |
| prefixes | NumericPrefixes::none (default), si, compact/decimal_compact, or factor; one replacing mode |
| unit | Optional NumericUnit(text, PerLabel/Shared), with configurable literal separator defaulting to one space |
| explanation | NumericExplanation::automatic (default) or authored Text, with complete local style/layout instructions |

Fractional bounds are 0..=20, significant 1..=21, adaptive min<=max in matching
bounds. `new(notation,precision)` supplies scale 0, no affixes, Preserve, Keep
and NearestEven rounding.
Default is Decimal/Fractional(2) with those same fields. Prefix/suffix retain exact
bytes including spaces/combining marks: no trim, normalization, inferred units or
inserted spacing. `prefix(value)` / `suffix(value)` accept supplied strings, not an
empty-string reset; reconstruct without an affix to omit it. Equality includes
all fields and bounds, absent/present affixes and literal bytes.
The `rounding(value)` edit validates the complete policy and retains that choice
through preparation, feedback, copies and Shared compatibility.
The `multiplier(value)` edit composes its exact dyadic factor with the decimal shift
before the one quantization. No intermediate binary64 product is formed, even for
overflowing/underflowing products. Negative factors reverse the displayed sign;
zero makes every finite ordinary or mathematical-power operand display zero.

Percent presentation is explicit scale 2 plus suffix `%`: raw 0.125 with
Fractional(1) becomes `12.5%`. Suffix alone merely appends `%`.
No scientific rescaling occurs.

### Explicit Tokens And Field Emission

NumericLocale::invariant uses ASCII digits/signs, period decimal, comma grouping
with Cycle[3], percent/per-mille and dollar/empty currency. Named locales transcribe
all 59 definitions from d3-format 3.1.0 commit
08806c62aa8f7b3fe64925f3989f1491c950a71c. `named_ids` is the exact catalog;
unknown/case-changed IDs reject without a host fallback. Source/license attribution
and the independently collected upstream normalized-token digest accompany the
Rust constants. Missing named sign/numeral/rate fields use pinned defaults; named
minus is U+2212, unlike invariant ASCII minus. No runtime JSON, network or assets.

Custom locales own a nonempty decimal token, ten distinct nonempty digit strings,
optional NumericGrouping, NumericSymbols and NumericCurrency. Tokens preserve
Unicode and spacing without controls/mandatory breaks; currency affixes may be
empty. Grouping sizes are positive, rightmost first, with Cycle, Last or Stop
continuation. NumericField has a single valid fill scalar, usize width and physical
Left/Right/Center/AfterSign; width counts final Unicode scalars, never CSS pixels.

After one exact quantization, the internal numeric digit runs receive grouping,
locale decimal/sign/digit tokens, currency or rate suffix and accounting closure.
Currency replaces the automatic rate suffix but not its exact factor. Literal
prefix/suffix enclose this presentation and are never digit-substituted. Field
fill runs last; zero AfterSign fill uses locale zero digits before grouping,
does not split multiscalar numeral tokens, and may exceed the minimum width.
Center gives an odd extra fill scalar to the right. Checked output/field storage
is fallible; no display alphabet registration or global text budget is required.

General requires significant precision and chooses fixed decimal after rounding
when -4<=rounded decade<precision, scientific otherwise. Integer requires
Fractional(0), uses the selected rounding, then emits arbitrary-size integer digits
in Decimal/Binary/Octal/HexLower/HexUpper; no fixed-size cast or second rounding.

### Exact Prefix And Factor Selection

NumericPrefixes::si accepts PerValue, WholeSet or Fixed(SiPrefix), with explicit
KeepThreshold/PromoteOnRounding. Fixed covers the complete BIPM table, including
deci/centi/deca/hecto and 2022 additions; automatic engineering steps are multiples
of three saturated at -30/30. Micro is U+00B5. WholeSet ignores zero members when
choosing the greatest nonzero scaled magnitude; all-zero SI uses neutral 0.

Compact rules have strictly increasing finite thresholds, positive finite divisors
and literal single-line prefix/suffix. Below the first threshold the factor is one
with empty affixes. Empty automatic tables normalize to none; Fixed must index an
existing rule. decimal_compact normalizes to the custom 1000/K, 1000000/M,
1000000000/B, 1000000000000/T table from scales 1.4.0. Literal compact affixes enclose
the numeric presentation inside outer literal affixes and are not localized digits.

KeepThreshold uses unrounded absolute magnitudes. PromoteOnRounding compares the
original signed mathematical operand with exact rounding-cell boundaries, including
nonintegral threshold/divisor ratios and tie parity. It never rerounds emitted
mantissas. WholeSet evaluates every member at each candidate common factor and
freezes the furthest required promotion before formatting; opposite signs and
source order cannot select different common factors. Fixed requires KeepThreshold.

Factor(Value) divides by the exact nonzero finite represented value, including its
sign. PowerOfTen accepts arbitrary-size signed ASCII integral exponents and
normalizes plus/leading zeros/negative zero. AutomaticDecimal selects the decade
of the greatest nonzero scaled magnitude, or zero for all-zero sets. Multiplier,
rate and decimal shift compose before this divisor and the one quantization. The
shared engine retains numerator/denominator and symbolic exponent, never an f64
division or physical power. Output reservation precedes giant materializations.

### Specifier Compilation

NumericFormat::from_specifier compiles
`[[fill]align][sign][symbol][0][width][,][.precision][~][type]` into this same policy.
Defaults retain invariant locale, NearestEven and Preserve. Types f/e use fractional
6; g/r/s/n and p use significant 6; % uses fractional 6; omitted type uses General,
significant 12 and Trim. r is fixed-decimal significant; s is PerValue SI with
promotion; n enables grouping; %/p select Percent. d/b/o/x/X use Integer with
Fractional(0); their syntactically numeric precision is discarded before bounds
checking, with no u8 limit on that irrelevant syntax.

Alignment </>/^/=, sign -/+/space/(, currency $, alternate #, zero padding, comma,
explicit precision and ~ map to typed fields. # is removed for inapplicable types.
Zero padding selects AfterSign; unused fill/alignment without width normalize away.
Width is checked usize Unicode scalars. Explicit relevant precision outside 0–20
fractional/1–21 significant rejects instead of clamping. Unknown types, text c,
malformed/trailing syntax and allocation failures reject. Typed/parsed equivalent
policies have equal identity, including `.3s` and `.1%`. Syntax compatibility does
not substitute JavaScript arithmetic for the exact renderer rounding contract.

### Common Unit And Factor Content

NumericUnit never infers a conversion from its text. PerLabel SI combines the
selected symbol and unit after the configured separator (`1 km`); other units
follow the complete prefix/number/suffix text. Without a unit request, selected
prefixes remain ordinary per-label affixes. Shared requires equal resolved factors
and affixes throughout the complete set, removes those repeated selected affixes
and retains one required explanation. Literal outer prefix/suffix and automatic
rate/currency presentation remain independent.

A common decimal power uses deterministic superscript exponent text (`×10⁶`,
`×10⁻⁶`), retaining its arbitrary-size mathematical exponent in metadata. Shared
SI with a supplied unit uses the combined prefixed unit, e.g. `km`; other factors
use the selected compact affixes or explicit divisor text followed by the unit.
An empty unit requests factor-only content. Separators are not prepended to an
otherwise empty explanation. Configured authored Text replaces generated source
in the same slot; style/layout edits retain that one content slot.

FormattedSet retains occurrence-ordered labels, original members, selected
precision, each resolved factor/affix, the unit and optional explanation. Its
copies, guide descriptors, axis plans, feedback and child dependencies retain
those values, not just equal current strings. Axis explanations occupy measured
outside-axis blocks after ticks/titles, including perpendicular thickness and
endpoint ink. Continuous guides own one explanation within their container,
after final Shared domain/policy merging. Heatmap cell values, point-only
Coefficient labels and Waterfall totals have independent blocks outside their
scientific marks/total labels; their axes and color guides remain separate.

Explanations use ordinary Text with the owner's numeric-label role by default
(Heatmap contrast resolves against its panel surface). Actual sources and local
font pairs are admitted before measurement. Natural width/height include these
blocks and assigned plans reserve their complete logical/ink extents. No implicit
abbreviation, omission, post-capture SVG insertion or extra font system is used;
insufficient requested width/height is a real fit refusal. Final capture/replay
consumes retained plans without additional font planning.

### Attachments And Actual Consumers

| Consuming attachment | Applicability / owner |
| --- | --- |
| x_tick_format / y_tick_format | Existing sealed NumericX/NumericY; generated axis presentation |
| color_label_format | ContinuousColor: Heatmap and Beeswarm guide numbers |
| HeatmapAnnotations::Values(format) | Finite active prepared panel cells |
| Plot<kind::Coefficient>::estimate_labels | Existing point-only restriction; intervals plus labels reject |
| Plot<kind::Waterfall>::total_format | Base/terminal numeric part, separately from WaterfallLabels names |

Each attachment replaces its own complete policy independently of limits, labels,
generation, Theme or setter order. Any gains no numeric attachment. Family
capability does not waive actual categorical-axis rejection before fonts.
Exact ticks suppress replaced numeric conversion and text admission in either order,
without suppressing actual-axis semantic validation.

| Family | Actual x / y | Additional numeric surface |
| --- | --- | --- |
| Scatter, Line, LinePath, Histogram, Density, Qq, Regression | Numeric / Numeric | None |
| Trajectory | Numeric or categorical conditions / Numeric | Categorical x rejects format/generation instructions |
| PointInterval, Coefficient, Forest, Dumbbell | Numeric / Category | Coefficient point-only labels; Forest Linear or Log10 operand |
| Bar, StackedBar, Boxplot, Violin, Raincloud, Interaction | Category / Numeric | None |
| HorizontalBar, StackedHorizontalBar | Numeric / Category | None |
| Heatmap | Category / Category | Continuous color labels, optional cell Values |
| Matrix | Its own numeric cell axes, default profile | No public NumericX/Y/ContinuousColor or global format |
| Marginal | Central Numeric / Numeric | Strips copy central positions without labels; no strip format |
| Beeswarm | Numeric / Category | Independent continuous color labels |
| Waterfall | Numeric / ordered steps | Eligible base/terminal totals |
| TextPanel | No scientific axes | Literal title/body only |

Scatter and ordinary Line do not accept categorical x merely because an internal
storage type is shared with Trajectory. Scientific parent/facet admission remains
the rendering contract, not a formatter expansion.

## Exact Mathematical Conversion

| Operand | Number to round |
| --- | --- |
| Ordinary finite binary64 v | Exact represented rational v times exact 10^scale |
| Configured Forest Log10 position e | Mathematical10^(exact(e)+scale), preserving exponent-space geometry |
| Private default compact derivation | Exact rational of its prescribed rounded binary64 division result |
| Omitted-format Forest expression | Default power grammar using the exponent operand |

Decode binary64 sign/significand/exponent for both the ordinary source and multiplier.
Derive their exact decimal coefficient/exponent using powers of2/5, multiply those
coefficients and add exponents and decimal scale,
and rounds once by exact signed quotient/remainder/parity. Do not multiply by an
intermediate f64 power, use platform transcendentals, parse text back into data
or format an approximate physical Forest power. Forest exponent 0.5/Fractional(2)
is `3.16`; scale 2 plus `%` gives `316.23%`. This does not claim an inverse of
rounded log10.

For nonzero magnitude with mathematical decade k, Significant(p) quantizes at
10^(k-p+1). Decimal Fractional(d) uses 10^-d; Scientific fractional uses its
normalized mantissa, Engineering fractional its unrounded engineering mantissa.
Engineering exponent is Euclidean multiple of3, nonzero mantissa in[1,1000).
Carry changes decimal point/exponent, not numeric rounding a second time.

NearestEven chooses the even coefficient on an exact half-cell; NearestAway and
NearestTowardZero choose the larger/smaller magnitude on a tie. TowardZero and
AwayFromZero truncate/increment magnitude on a nonzero remainder. Floor increments
negative magnitudes, Ceiling positive magnitudes. Exact values never increment.
For example, at Fractional(0), -1.5 becomes -2 with Floor/NearestAway, -1 with
Ceiling/NearestTowardZero; 2.5 becomes 2 with NearestEven and 3 with NearestAway.
Actual or rounded negative-zero normalization runs only after that quantization.

Decimal Significant/Keep retains requested fractional significance where
expressible: 1 at p=3 is `1.00`, 0.125 at p=5 is `0.12500`,
12345 at p=3 is `12300`. Integral
place-value zeros remain even when significance cannot be encoded unambiguously
without adding notation. Zero has one integral0 and p-1 fractional zeros for
significant, d for fractional; exponent notation zero uses e0.

Normalize removes the sign after rounding for actual or rounded negative zero;
Preserve retains it. Mathematical Forest powers are positive before the explicit
multiplier. Trim removes only
fractional trailing zeros and empty decimal point, never integral zeros, exponent
or affix text. Scientific uses one integral mantissa digit, lowercase e, no plus
or exponent-leading zeros. Engineering always emits exponent including e0; no
automatic SI prefixes. Append literal affixes around the final number.

| Exact input / policy | Required text |
| --- | --- |
| 1.25 / Decimal Fractional(1) | 1.2 |
| 1.75 / Decimal Fractional(1) | 1.8 |
| 12345 / Scientific Fractional(2) | 1.23e4 |
| 12345 / Engineering Fractional(2) | 12.34e3 |
| 999.5 / Engineering Significant(3) Keep | 1.00e3 |
| -0 / Scientific Significant(3) Preserve Keep | -0.00e0 |
| -0.00390625 / Decimal Fractional(2) Preserve | -0.00 |
| 1 / Decimal Significant(3) scale -3 Keep | 0.00100 |

Exact scale extremes remain mathematical, not intermediate binary64 overflow/
underflow. Scientific output can represent f64::MAX shifted324 or subnormal
shifted-324. Enormous Decimal Forest output computes symbolic length and fails
real representation/reservation before giant powers/zero runs. No precision
quota, exponent clamp, shorter notation, truncated affix or approximate fallback.

## Complete Sets And Default Profiles

Exact precision never increases or changes notation to repair collisions.
Generated candidates must distinguish distinct scientific values: collision is
a complete-source miss before geometric fitting. Required continuous guides reject
indistinguishable numbers before fonts. Fixed value annotations/totals may
intentionally repeat rounded strings.

Adaptive chooses the smallest precision in inclusive[min,max] distinguishing all
distinct scientific members, uniformly across the complete owner set. It is
independent of glyph width, fit and visible ink. Equal values/signed-zero identity
may repeat; Preserve can retain occurrence signs. An empty set invents no label;
singleton uses minimum. Exhaustion is axis source miss or pre-font required-set
presentation error; real arithmetic/certification/allocation failure is fatal.

| Owner set | Membership |
| --- | --- |
| Generated axis candidate | Every selected position in that complete current-domain source, with exponent identity for Log10 |
| Continuous guide | Final domain lo/interior-center/hi or one constant; not missing key or stop samples |
| Heatmap Values | Every finite Some active prepared cell, with every occurrence; no saturation/window/narrow-cell filtering |
| Coefficient labels | Every available point-only estimate in panel, including outside exact x window; not bounds/reference/unavailable/significance |
| Waterfall totals | Base/ordered cumulative terminal eligible in closed effective x/y window, with separate named occurrences |

Retain occurrence-to-member maps and deterministic sorted distinct members without
changing paint/source order. Normalize zero only for scientific distinctness;
retain signs/provenance for output. Use complete comparisons and checked fallible
storage, not hash-only identity or unbounded quadratic distinctness. An adjacent-
only collision optimization must be valid for its whole grammar. A fallible
comparison propagates failure, never fabricates Ordering. The shared heapsort is
unstable; source-order-sensitive keys need explicit stable tie order.

### Consumer Defaults

Omission uses private profiles of the same engine, not NumericFormat::default
everywhere. Default axes/Matrix cells/continuous numbers use Automatic magnitude
grammar. Heatmap Values consumes its supplied format; point-only Coefficient
requires the supplied format. Waterfall omission is Fractional(3)/Preserve/Keep.
Explicit NumericFormat::default remains Fractional(2)/Preserve/Keep.

Automatic normalizes actual signed zero and chooses by **unrounded abs(v)**:

| Branch, in order | Conversion / trim |
| --- | --- |
| abs>=1e12 or 0<abs<0.01 | Scientific Fractional(2), keep mantissa zeros |
| abs>=1000000 | Binary64 v/1000000.0, then exact conversion of that result to Fractional(1); trim and append M |
| abs>=10000 | Binary64 v/1000.0, then exact conversion to Fractional(1); trim and append k |
| abs>=100 | Fixed Fractional(0), preserve integral zeros |
| abs>=1 | Fixed Fractional(1), trim |
| Otherwise | Fixed Fractional(2), trim |

Preserve named binary64 division order, rather than exact decimal rescaling of
the original operand. For example `12350 -> 12.3k`, `1150000 -> 1.1M`,
`999999 -> 1000k`;
do not reclassify after rounding. Scientific exponent spelling is e12/e-3,
not e+12/e-03. Whole-source collisions retry scientific fractional precision 2..17,
retaining zeros and the first unique complete set. This private retry reaches 18
significant digits. Omitted-profile retries keep that existing range independently
of the public Significant maximum 21.

Omitted Forest integral labels are: exponents-2,-1,0,1,2,3 ->
0.01,0.1,1,10,100,1000;4..8 ->10k,100k,1M,10M,100M;9..11 ->
1000M,10000M,100000M; other admitted integral seeds ->1.00e plus exact exponent.
Adaptive/midpoint sources use literal `10^` plus Automatic-formatted exponent,
without parentheses or physical pow. Explicit policy uses certified mathematical
power instead. No public compatibility mode selects these profiles.

Waterfall combines `name + ASCII space + formatted number`, default names base/
total and precision 3, e.g. base 10 is `base 10.000`. Numeric affixes wrap only the
number. Preserve the ordered cumulative terminal, not a recomputed delta sum.

### Parent Domains And Waterfall Eligibility

Facet formats come from owning Plot; value sets stay per-panel even with shared
axis domains. Canvas range groups can use different per-panel formats. A truly
shared displayed numeric axis requires equal effective format/operand context.
Shared continuous contributors compare complete effective policy before fonts,
not equal current strings/widths. Then merge domains and format one final set.
Suppress local guide formatting before a locally ambiguous set can reject a
valid final Shared guide. Install final colors/contrast/guide numbers before census.

Retain both Waterfall numeric candidates/names. Eligibility uses effective closed
x/y projection at source_y=category_count; shelf fit cannot remove eligible text.
Parent union can change local eligibility. Before fonts resolve each reachable
mask(neither/base/terminal/both), its uniform precision/result or refusal. Immutable
exact exclusions make masks unreachable and must not create unnecessary errors.
Retain actual combined strings for every reachable result, separately from
actual paint membership. Later domain consequences select already prepared
numeric/result context, not new font discovery or a new layout loop.

## TickGeneration And Exact Ticks

TickGeneration is opaque Clone/Debug/Default/PartialEq/Eq, not Copy. One private
primary locator is Automatic, Target or Interval; an independent optional positive
maximum caps generated positions. Maximum is Automatic plus that cap, never a
parallel locator implementation. Interval retains its admitted Linear/Log10 scale
binding and exact represented step/origin/space/requirement; the additive/decade readers project those
constructor semantics. Default and automatic() reset the entire request.
is_automatic tests the complete default, including absence of a cap. Additive is
finite positive f64; Log10Decades positive u32; target/maximum counts and all caps
are positive usize, without a 10000 ceiling. Getters are is_automatic, additive_step,
decade_step, target_count, get_maximum_count. The consuming maximum_count edit sets
or clears one positive cap without replacing the primary, rejecting an ExactCount
contradiction. Consumers explicitly borrow/clone the complete policy through axis
semantics, feedback, facets and composed child/height dependencies.

x_tick_generation/y_tick_generation use the existing 17 NumericX/15 NumericY gates
in `src/kind.rs`. Each replaces that axis's whole instruction. Any/Heatmap/Matrix
gain no attachment. Actual categorical axes reject non-Automatic choices before
fonts; interval scale-kind mismatch rejects. Counts do not change scientific
limits, transforms, format or nice-range policy.

Exact x_ticks/y_ticks have nonempty finite strictly ascending data positions in
the resolved display domain, indivisible verbatim labels, including empty labels.
They replace all generated sources in either setter order. Never merge/cap/drop
exact positions to satisfy a generated maximum; exact collisions geometrically
reject instead of moving/rotating/omitting labels. Actual-axis semantic validation
still applies. Forest exact positions are supplied in raw data space and projected
by the scientific transform; generated Forest positions are exponents.

### Complete Source Program

An axis tries complete candidates in order and accepts first whose numerical
labels and actual geometry fit. Source-local cardinality/unrepresentability can
miss before materialization; real arithmetic/allocation/font failures remain
fatal. No selected-label subsequences or joint-axis source ranks are introduced.

Automatic linear uses `pitch=max(1 quantum,6ch)` and
`upper_seed=1+min(9999,max(1,span/pitch))`; try desired counts down to1, then the
original-domain f64::midpoint singleton. No universal five-tick ceiling/target.
For seed k, raw_step=(hi-lo)/k; nonfinite subtraction uses
`((hi/scale)-(lo/scale))*(scale/k)`, scale=max(abs(lo),abs(hi)). Nonfinite/
nonpositive raw step locally misses.

Choose greatest table decade<=raw_step and test positive finite factors
10,5,2.5,2,1 times it (or raw_step alone if no table exponent). Minimize abs(count-k),
ties keeping earlier larger step. Adjacent endpoints or fewer than2 resulting
positions use validated[lo,hi]. Automatic candidate-count estimates use binary64
ceil(lo/step)/floor(hi/step) within[-2^53,2^53]. Actual lattice materialization uses
the shared exact rounding-cell owner below, with checked complete count before
allocation and no lossy float conversion of a lattice index.

Additive first tries requested step, then step times1,2,3,5 by decades, omitting
exact duplicates. Stop after first finite decade base beyond max absolute domain
or before next nonfinite base; original midpoint last. No interval subsequences.
Nice-range broadening separately retains seed5,5% margin and its own grammar;
these are not generated-count controls.

Forest Automatic intersects integral sources with[-323,308]. With>=2 integrals,
try full list first, then evenly indexed subsets in descending size up to measured
upper count, index `((i*(n-1))+((d-1)/2))/(d-1)`. With<2 use adaptive linear sources
over exponent domain. Original-domain midpoint last, not re-clamped to integral
range. Omitted labels use power grammar; explicit format uses mathematical power.
Log10Decades tries positive widened checked d*(1,2,3,5) by decades containing only
divisible integral exponents; no adaptive/subsequence alternatives, then midpoint.
Empty intersection records requested miss rather than inventing exponents.

Target(n) starts linear adaptive seed n (not measured6ch upper), then n-1..1 and
midpoint. Nice-step deviation is allowed: [0,1] target4 can give[0,0.5,1], target5
[0,0.25,0.5,0.75,1] when labels fit. Integral Forest starts d=min(n,integral_count),
full only when d equals full count, otherwise same subset formula and smaller d
down to2, then midpoint; <2 integrals uses adaptive exponent-space sources.

Maximum(n) preserves Automatic's source order, filtering by **actual complete
in-domain position count** before allocation/format/measurement. Endpoints and
unlabeled positions count. Do not select another factor within a seed merely to
meet maximum or drop positions after selection. Forest skips oversized full list
then continues the same subsets. For either count1, select original midpoint
directly, still subject to fit; adjacent-domain two endpoints cannot bypass1.
Private Automatic candidates retain a 10000 bound; explicit Target/interval/count
sources have real usize/storage bounds and any caller-selected cap instead.
Linear Target has checked at most n+1 candidates, integral Forest at most d. Emit
midpoint at most once. Unrepresentable source lengths or real allocation failures
are contextual rendering failures, not permission to truncate a list.

### Exact Generated Counts

ExactCount(count, TickSpace) is one complete source, including endpoints for n>=2.
Independently round `((n-1-i)*lo+i*hi)/(n-1)` once to binary64 through the common
signed rational converter, retaining exact endpoint bits. Never compute an
overflowing hi-lo or accumulate a rounded step. The common converter also serves
certified log10 and handles zero, subnormals, ties and binade carries.

Zero means a valid empty generated list, without an invented numeric owner,
callback or explanation; marks, frame and title remain. One defaults to Lower,
with explicit Lower/Midpoint/Upper/Position singleton selection in the chosen
space. Position is finite and must be in-domain when n=1; for n>=2 singleton
intent is retained but cannot move endpoints. singleton on another primary rejects.
Source count and cap are checked before text work. A collapsed rounded/projected
source or selected text nonfit is a contextual failure, never a coarser source or
terminal-midpoint retry.

Scale interpolation on Log10 retains mathematical exponent operands without old
integral-source clipping. Data interpolation retains ordinary finite physical
operands and then projects them. Genuinely retained raw limits, including supplied
Forest limits, are used as-is; otherwise the canonical power endpoints are combined
with positive rational coefficients through certified enclosures before one binary
rounding. This does not expose an invented inverse as domain data_limits.
The selected complete set uses ordinary NumericFormat functions/precision/factors,
actual-text admission and frozen output contracts, after final parent-domain resolution.

### Fixed And Preferred Origin-Aligned Intervals

Interval retains finite positive step, finite numeric origin, Data/Scale space,
Fixed/Preferred requirement and admitted scale binding. The generic interval
factory is context-relative; additive is preferred zero-origin Linear Data,
log10_decades is preferred zero-origin integral Log10 Scale. Their original readers
project those bound choices, while interval_parameters reads every interval tuple.
Zero origins normalize to positive zero. Complete equality/feedback retains all
fields, including binding and requirement; equal current positions are insufficient.

Positions are the independently rounded original `origin+k*step`. Express each
finite binary operand in integer 2^-1074 units. The inclusive represented domain's
preimage is bounded by midpoint(previous(lo),lo) and midpoint(hi,next(hi)); midpoint
ties include an endpoint exactly when its significand is even. Use virtual finite
binade neighbors at +/-MAX. Exact signed floor/ceiling division of these boundaries
minus the origin determines first/last arbitrary-size integer indices. Check the
complete usize count and caller cap before positions/text allocation. There is no
2^53 index ceiling, decimal tolerance, accumulated rounded addition or source
truncation. This admits ordinary 0.1-step endpoint 1 consistently with representation.

Fixed emits one full source, allows a valid empty list and never substitutes a
midpoint on cap/representation/text refusal. Empty fixed owners suppress numeric
probes/functions/explanations. Preferred shares the same arithmetic and tries step
times 1/2/3/5 by decades before the original-domain scale midpoint. Oversized
complete candidates can miss its caller cap; actual allocation/font errors remain
fatal. The source program is finitely bounded by binary64 decimal bases. Neither
mode depends on assigned pixels, typography or optional layout reuse.

Data on Log10 resolves genuinely retained raw endpoints, or certified mathematical
power endpoints in the finite data representation, and rounds each original lattice
member before scientific projection. Underflowing lower mathematical endpoints
admit only positive representable members; unrepresentable nonempty upper ranges
refuse instead of becoming empty. A represented endpoint keeps its canonical scale
coordinate. Callback domain raw bounds are never invented. Scale on Log10 retains
mathematical exponent operands. Only the old integral-decade binding retains the
[-323,308] integral-source intersection; generic Scale intervals do not inherit it.

### Anchored Multiplicative Lattices

Multiplicative owns the supplied finite positive anchor and ratio other than one,
plus Fixed/Preferred requirement and the same optional maximum. Normalize internal
ordering with increasing j and k=direction*j, retaining the supplied ratio bits;
never form a rounded reciprocal. Members are the exact rationals anchor*ratio^k.
Linear coordinates round those rationals once; Log10 coordinates certify their
logarithms directly. Both retain the original mathematical operands for labels and
function contexts, not accumulated products, physical powers or rounded exponents.
Exact decimal powers normalize through the common symbolic power formatter.

The common rational-log owner encloses boundary quotients and refines until exact
integer first/last indices are established, including represented endpoint midpoint
cells and tie parity. Rational factorized equality resolves exact boundaries. The
complete arbitrary-size count meets its cap/platform bounds before positions or
large powers can allocate. Fixed emits precisely that full list; empty views
including wholly negative linear views produce no numeric probe/text/explanation.
A linear view containing positive values and zero has an unbounded tail and fails,
without an arbitrary positive cutoff. Nonempty unrepresentable/collapsed sources
are failures, never empty or silently thinned sources.

Preferred first tries the full lattice subject to its cap. Let B be the existing
bounded measured 6ch seed. For desired counts B down to 2, use stride
max(1,ceil((last-first)/(desired-1))) and keep precisely indices divisible by that
stride in the original anchor-index frame. Skip consecutive duplicate strides.
The final source is the original lattice member at floor((first+last)/2), not an
invented domain midpoint. There are at most B+1 candidates; each is a complete
sublattice with the same formatting/admission/fit boundary. Actual arithmetic,
coordinate, font and storage failures remain fatal rather than becoming recovery.

FormattedSet stores one canonical Binary64/PowerOfTen/Multiplicative member enum
and the whole resolved text/precision/factors/unit/explanation/context. All numeric
axis feedback observes that complete set, including omitted-format mathematical
members, rather than factors alone. Optional layout reuse cannot replace source
identity with equal projected positions or current strings. Canvas provisional
gutter probes retain recoverable fit misses so other declared guide candidates can
run; no numeric source is repaired or coarsened to rescue a fixed source.

## Pre-Font Requirements And Retained Dependencies

Resolve applicability, exact/category suppression, final domains, Shared policy
compatibility and required sets before fonts. Known labels contribute full
concatenated strings/affixes/names and base-plus-mark spans. Register generated-axis
roles and actual metric0 without expanding a numeric alphabet or hypothetical
affix boundaries. Retain the actual complete strings of each reached candidate,
then admit all their font requirements before measurement or geometric rejection.
A font error in a later member remains fatal even if an earlier label cannot fit.

One operation-bound admission path extends the canonical pair's missing scalars
and complete combining units, including an affix mark joined to its actual digit.
It can register a newly used pair without rediscovery or replacing prior choices.
Isolated scalar coverage is insufficient to preserve whole-unit preference.
Do not enumerate hypothetical values, terminal digits or uncalled producer output.
The accepted root seals fonts once after bounded measurement and before capture;
capture verifies source coverage and pair provenance without new font work.

Terminal-source intrinsic uses singleton midpoint/minimum adaptive precision as
an intrinsic input, not proof a later source fits. Natural low/mid/high probes are
preferences, not required guide sets: exact policies use requested precision;
adaptive probes retain maximum complete width across allowed precisions, without
uniqueness rejection or selecting final precision. Default probes retain magnitude
grammar. Exact/category sources use actual text. No width reselection on feedback.

Prepared/scoped axes, value sets, guides, child/intrinsic/content-height cache keys
and feedback retain complete policies, operand/default kind, both TickGeneration
choices, exact/supplied/generated precedence, range provenance/base/consumed domains,
source positions, occurrences/signs, resolved uniform precision, owner/scope,
visibility, complete role/font provenance, assigned geometry, parent alignment/
insets and relevant feedback. Equal text/width/height does not establish upstream
semantic equality. Erasure/cloning/parent range replacement preserves these inputs.

Lower text caches intentionally key actual complete text and used shaping/
presentation inputs after conversion; identical text/role/provenance can share
glyph analysis without duplicating upstream numeric policies. Full equality still
checks indexed fingerprint collisions. Cache enablement/order cannot select
another number spelling or precision. Unrelated admissions and the final seal do
not invalidate earlier shaped plans or introduce a global admission-epoch key.

Freeze final strings, selected precision, face/glyphs and placement. Capture, Null,
SVG/PNG and Output reuse perform no numeric conversion, source selection or new
font work. Real storage/precision/length failures remain fatal; layout fit retains
its typed misses, without lowering precision, clipping required text, truncating
affixes or introducing another convergence driver.
