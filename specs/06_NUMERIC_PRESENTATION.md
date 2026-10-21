# Numeric Presentation

**Status**: Current public numeric presentation contract. Exact ordinary and
certified power conversion, configured consumers and the atomic public replacement
are implemented. All-family/all-root and scoped/output closure is recorded in
`../docs/scientific-numeric-closure.md`; integrated T6 verification and actual
release/time/peak-RSS closure pass in `../docs/scientific-foundation-closure.md`.

## Authority And Source Basis

This specification owns the admitted, locale-free numeric presentation contract:
its public value, consumer applicability, default spelling, semantic set
membership, prepared transport, font requirements, identity and migration.
`93_T6_EXECUTION_PLAN.md` owns execution/status; `94_T6_FOUNDATION_CONSTRUCTION.md`
owns the already closed constructor/getter/`with_field` vocabulary. The separate
`07_LOG10_DISPLAY.md` specification owns the Forest power theorem,
interval algorithm, termination proof, temporary bounds and independent oracle.
This document states that primitive's required mathematical interface, not a
substitute for its independent executable certification evidence.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` describe the matching public
contract. Appearance and layout authority remain
with `03_THEME_SYSTEM.md` and `05_LAYOUT_SYSTEM.md`. Numeric policy is semantic
presentation, not an appearance component or a new scientific transformation.

The original source-backed migration census below was inspected at **`e6febe3`**. That
baseline includes the paint-value migration: complete paints are
`theme::{Color,OpaqueColor}` from `theme/paint.rs`; backend `Rgba` has already been
deleted. It still has the closed public Theme, exact default root dimensions,
raw role-stack inputs and `NumericLabelFormat`. The historical A1 census in
`docs/t6a1-public-cutover-census.md` is read with spec94's adjudications and
`docs/t6b1a-paint-values.md`, not as evidence that its earlier paint carriers still
exist. Source symbols take precedence over historical line numbers.

The post-foundation source refresh at **`387b07a`** is complete in
[`docs/numeric-source-handoff.md`](../docs/numeric-source-handoff.md). It maps the
historical tables to complete Theme/role/font/scale values, the then-current opaque
interval carrier, root requests, actual conversion/copy/cache/font
paths and guide index drift. The current generation carrier is TickGeneration,
as specified in spec05. This recorded handoff is not a numeric public
cutover or executable certification. In particular, `builder/admission.rs` and
`Plot::admitted_clone` no longer exist; derived Clone and exhaustive `erase` are
the actual Plot transport sites.

The private ordinary conversion/emission primitive is implemented with independent
rational evidence in [`docs/exact-decimal-conversion.md`](../docs/exact-decimal-conversion.md).
The private Forest exponent/symbolic and fractional certification gates are now
recorded in `docs/exact-log10-operands.md` and `docs/certified-log10-conversion.md`.
The subsequent private policy/default migration is complete in
[`docs/unified-numeric-presentation.md`](../docs/unified-numeric-presentation.md):
the historical public wrapper moved onto one normalized policy and default/value
conversion core. Configured axes, guides and prepared annotations subsequently
landed with complete scoped identity; the public replacement now exports that
same `NumericFormat` directly and removes the wrapper and all repository callers.
Current API/evidence is in `docs/public-numeric-formatting.md`; earlier documents
describe their recorded implementation checkpoints.

**Accounting simplification:** spec95 supersedes every historical ledger, fee,
comparison-charge, quota and charge-delta instruction here. Keep real checked
lengths, fallible first-party storage, exact arithmetic/temporary bounds, full
identity and lifecycle. Do not recreate accounting-only helpers or parameters.

The numeric public signatures and product choices are **design-closed here**.
The execution plan's actual-input census refresh followed the theme,
font, scale, spacing, tick-interval and sizing migrations. It maps
the same contracts to current fields/callers and records source drift;
it does not reopen signatures or defer the consumer/default design. Design
closure is not implementation, numerical certification or test closure.

## Scope And Single Ownership

Admitted controls are Decimal, Scientific and Engineering notation; exact or
bounded adaptive fractional/significant precision; exact decimal display scale
`-324..=324`; literal prefix/suffix; negative-zero and fractional trailing-zero
rules. Consumers are generated numeric axes, continuous color guides, existing
Heatmap cell values, existing point-only Coefficient labels and Waterfall totals.

Scientific values, availability, statistics, domains, color mapping, selected tick
positions, projection, clipping and source identities keep their current owners.
Exact tick labels, category and discrete legend identities, PointLayer strings,
significance marks, panel text, fully-null-row annotations and Waterfall names
are not parsed or reformatted as numbers. No numeric point-label surface, Matrix
global binding, locale, general format string, callback, arbitrary floating
multiplier, shared multiplier annotation or data-mapped alpha is admitted.

There is one normalized formatter representation and one rounding/emission
engine. Consumer defaults are private profiles of that representation, not a
public notation, version selector, compatibility adapter or fallback to an old
formatter. The engine accepts different mathematical operand kinds and a
prescribed built-in operand derivation; it does not retain parallel old/new
conversion implementations. Policy is attached to Plot's existing owners, never
inherited through Theme, mutable global configuration or a new Canvas override.

## Public Value And Attachments

These signatures repeat spec94's closed contract so the numerical specification
is self-contained. `Result<T>` is the crate's existing result/error type. Fields
are private; root re-exports do not expose the implementation module.

```rust,ignore
enum NumericNotation { Decimal, Scientific, Engineering }
enum NumericPrecision {
    Fractional(u8),
    Significant(u8),
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
```

`NumericFormat` implements `Clone + Debug + PartialEq + Eq`; the closed input
discriminants are Copy values. Equality includes the seven normalized fields,
including absent versus present affixes and the complete adaptive bounds. There
is no remembered constructor path or runtime font/layout state in the value.

Construction validates fractional `0..=17`, significant `1..=17`, and adaptive
`min<=max` in the matching range. Scale is an `i16` within `-324..=324`. Affixes are
nonempty and contain no `char::is_control()` scalar; retain their literal bytes,
including spaces and combining marks. Do not trim, normalize, infer units or add
spacing. No empty-string reset is introduced; reconstruct without an affix to
omit it. Each edit validates the complete result and replaces only its field.

`new` supplies scale 0, absent affixes, Preserve and Keep. `Default` is
Decimal/Fractional(2) with those same inputs. Percentage is explicit scale 2 and
suffix `%`; suffix `%` alone remains a literal append. For example raw `0.125`
with Decimal/Fractional(1), scale 2 and suffix `%` displays `12.5%`.

| Attachment, consuming and returning Self | Existing applicability | Stored target owner |
| --- | --- | --- |
| `Plot<K>::x_tick_format(NumericFormat)` | `K: kind::NumericX` | Optional format on `x_axis_spec` |
| `Plot<K>::y_tick_format(NumericFormat)` | `K: kind::NumericY` | Optional format on `y_axis_spec` |
| `Plot<K>::color_label_format(NumericFormat)` | `K: kind::ContinuousColor` | Optional continuous-guide format on Plot |
| `HeatmapAnnotations::Values(NumericFormat)` | Existing Heatmap annotation policy | Existing annotation payload, replaced atomically |
| `Plot<kind::Coefficient>::estimate_labels(NumericFormat)` | Existing `CoefficientOptions` set is Coefficient only | Existing optional estimate-label format |
| `Plot<kind::Waterfall>::total_format(NumericFormat)` | Exact Waterfall family | Optional total-number format, separate from names |

Repeated attachments replace their own complete format, independently of limits,
intervals, themes and label names. No format method is added to erased `Any`.
An actual categorical axis rejects a numeric-format instruction before fonts;
a family capability is not evidence that every instance has a numeric axis.
On a valid numeric axis, exact ticks override interval and formatter regardless
of setter order. Their positions/labels and current validation remain unchanged;
suppressed generated policy adds no alphabet, affix unit or formatting work.
This explicit precedence is not an inert unsupported setter.

Source: `src/kind.rs:109-139`, `src/builder/configuration.rs:96-103,180-198,568-574`,
`src/spec.rs:219-350`; target signatures: spec94 `1904-1948`.

## Mathematical Conversion And Spelling

### Operand Contract

| Operand kind | Mathematical number to round | Scientific identity retained separately |
| --- | --- | --- |
| Ordinary finite binary64 value `v` | Exact represented rational `v * 10^s` | Raw value/source row or selected data-space position |
| Configured Forest Log10 position `e` | Mathematical `10^(exact(e)+s)` | Selected finite binary64 exponent and exponent-space projection |
| Built-in compact derivation | Exact rational of the prescribed rounded binary64 division result | Original value, selected source and default profile |
| Unconfigured Forest expression | Existing power grammar using the exponent operand | Exponent/source kind; not a configured raw-value operand |

For explicit policy, decimal scale is exact algebra, not `v * powf(10,s)` or
`e+s` in binary64. Round once, nearest/ties-to-even, directly to a decimal quantum.
The result is an integer decimal coefficient plus decimal exponent and sign,
not an intermediate `f64`. Do not parse text back into scientific data.

Decode finite binary64 into its sign, integer significand and binary exponent.
For ordinary inputs, derive an exact decimal coefficient/exponent using powers
of two/five, shift the decimal exponent by `s`, and quantize with exact quotient,
remainder and parity. Reuse the existing pinned `num-bigint`; no additional
dependency is part of this contract. Intermediate integer and output growth is
checked and admitted before first-party retention/arithmetic.

Configured Forest uses the separately certified primitive. Exponent `0.5` at
Fractional(2) displays `3.16`, and scale 2 plus `%` displays `316.23%`. Do not format
the exponent, print a power expression, use platform transcendental arithmetic,
or claim that exponentiating a rounded scientific `log10` reconstructs every
original observation. All selected finite exponents are covered, including
original-domain midpoint sources beyond the integral seed clamp. The mathematical
power need not fit binary64. Integral powers have a symbolic fast path; irrational
fractional powers require direct decimal certification. Certification/resource
refusal is fatal, not an axis collision or coarsening request.

Source: `src/numeric.rs:1-13,24-86,174-202`,
`src/layout/panel/sources.rs:381-396,435-468`, `Cargo.toml:13-18`.

### Exact Precision And Zeroes

| Notation | Fractional(d) | Significant(p) |
| --- | --- | --- |
| Decimal | Exactly d digits after the decimal point before trimming | Quantize the whole nonzero value to p significant digits, then emit ordinary decimal notation |
| Scientific | d digits after the normalized mantissa's decimal point | p significant mantissa digits before trimming |
| Engineering | d digits after the engineering mantissa's decimal point | p significant digits in the whole engineering mantissa, not p digits after its point |

For nonzero magnitude `a`, let `k=floor(log10(a))` mathematically, determined by
exact integer comparisons or the Forest primitive's exact range reduction, not
platform `log10`. Significant precision uses quantum `10^(k-p+1)`. Carry changes
the emitted exponent/decimal position; it does not cause a second numeric
rounding. If the rounded decimal coefficient carries a digit, discard only the
redundant coefficient zero(s) while preserving the rounded value and requested
significant occupancy where decimal notation can express it.

Decimal Significant/Keep fills fractional positions needed to show the requested
precision. Leading fractional zeroes are place holders, not significant digits:
`1` at p=3 is `1.00`; exact binary64 `0.125` at p=5 is `0.12500`; exact `0.001953125`
at p=3 is `0.00195`. If the rounding quantum is integral, preserve integral
place-value zeroes: `12345` at p=3 is `12300`, and the exact tie `999.5` at p=3
becomes `1000`. Do not add a new decimal-point certainty marker or exponent to
communicate significance in an integer; Decimal notation is still Decimal.

Zero has one integral `0` and p-1 fractional zeroes for Significant(p), including
Scientific/Engineering mantissa zero with exponent `e0`. Fractional(d) uses d
fractional zeroes. NegativeZero::Normalize removes a sign from both actual -0 and
a negative input rounded to zero; Preserve retains it. A Forest power is positive
even for exponent -0 and cannot produce negative zero. Apply sign policy after
rounding, then append literal affixes around the complete number.

TrailingZeros::Trim removes trailing fractional zeroes and an empty decimal point
only. It does not remove leading fractional zeroes, integral place-value zeroes,
exponent digits or affix characters. No Unicode minus, grouping or localized digit
is generated. Scientific uses lowercase `e`, no positive exponent sign and no
exponent leading zeroes. Engineering always emits `e`, including `e0`; its
exponent is divisible by three and nonzero mantissa magnitude is `[1,1000)` after
carry normalization. Engineering has no SI-prefix switch.

| Independently specified exact input/policy | Required number text |
| --- | --- |
| `1.25`, Decimal/Fractional(1) | `1.2` |
| `1.75`, Decimal/Fractional(1) | `1.8` |
| `12345`, Scientific/Fractional(2) | `1.23e4` |
| `12345`, Engineering/Fractional(2) | `12.34e3` (even tie) |
| `999.5`, Engineering/Significant(3) | `1.00e3` |
| `0`, Decimal/Significant(3), Keep | `0.00` |
| `-0`, Scientific/Significant(3), Preserve/Keep | `-0.00e0` |
| `-0.00390625`, Decimal/Fractional(2), Preserve | `-0.00` |
| Same input, Normalize | `0.00` |
| `1`, Decimal/Significant(3), scale -3 | `0.00100` |
| `1000`, Decimal/Significant(3), Trim | `1000` |

These are policy expectations, not evidence of a completed implementation.

### Representability And Refusal

Ordinary scale extremes are resolved as exact decimal shifts: `f64::MAX` scaled
by 10^324 remains mathematically finite and printable in Scientific notation;
the smallest positive subnormal scaled by 10^-324 is not rounded to binary64 zero
before presentation. A finite Forest exponent can require an enormous ordinary
decimal string. Compute its symbolic length first, then admit or refuse before
generating powers/zeroes. Do not impose a new exponent clamp or widen root quotas.

Checked byte/record/precision arithmetic, resource admission and fallible growth
refuse through the existing errors. No approximation, shorter notation, lower
precision, truncated affix or previous best label is returned after refusal.
Exact label collisions and geometry fit misses retain their separate contracts.

## Preserved Default Profiles Of The Same Engine

Default-byte preservation is a conversion/operation-order requirement, not merely
matching a few familiar examples. It is independent of separately approved theme,
font, gradient and automatic-sizing changes. Compare defaults at fixed source,
effective role, domains and allocation when attributing a numeric change.

### Default Profile Matrix

| Consumer | Presence without a new format | Private effective profile | Conversion/sign/trim |
| --- | --- | --- | --- |
| All ordinary generated numeric axes | Existing axis presence | Automatic magnitude profile | Actual +/-0 normalize; prescribed binary64 compact division; branch-specific trim |
| Matrix numeric cell axes | Existing visible cell axes only | Same Automatic profile | Same engine; no global Matrix override |
| Forest Linear x | Existing axis presence | Automatic profile | Ordinary raw positions |
| Forest Log10 integral seeds/subsets/intervals | Existing selected sources | Default power-literal profile | Exact literal or `1.00e` plus integral exponent |
| Forest Log10 adaptive/midpoint | Existing selected sources | Default power-expression profile | Literal `10^` plus Automatic-formatted exponent; no parentheses |
| Local/Shared continuous guide numbers | Existing required guide presence | Automatic profile with complete-set retry | Shared final domain before formatting; signed-zero scientific equivalence |
| Heatmap values | Hidden unless Values is requested | `NumericFormat::default()` when that value is supplied | Fixed 2, Preserve, Keep, scale 0, no affixes |
| Point-only Coefficient values | Absent unless estimate_labels is requested | Supplied format; default value is fixed 2 | Preserve/Keep; unavailable text is separate |
| Waterfall base/terminal numbers | Existing domain-visible totals | Fixed 3 profile | Preserve/Keep, scale 0, no numeric affixes |
| Exact ticks, names, caller strings | Existing caller presence | No numeric profile | Verbatim |

### Automatic Profile: Ordered Arithmetic And Punctuation

Source authority: `src/layout/panel/sources.rs:658-734`. Apply these branches in
this order to finite `v`, first replacing actual signed zero with positive zero:

| Branch on abs(v), before rounding | Exact operand derivation and decimal operation | Trim | Generated suffix |
| --- | --- | --- | --- |
| Scientific retry at fractional precision q>0 | Same binary64 v, nearest-even scientific q after the mantissa point | None | None |
| abs(v)>=1e12, or 0<abs(v)<0.01 | Same v, scientific fractional 2 | None | None |
| abs(v)>=1_000_000 | **Binary64 `v / 1_000_000.0`**, then fixed fractional 1 of that exact division result | Fractional trailing zeroes/point | `M` |
| abs(v)>=10_000 | **Binary64 `v / 1000.0`**, then fixed fractional 1 of that exact division result | Fractional trailing zeroes/point | `k` |
| abs(v)>=100 | Same v, fixed fractional 0 | None; do not trim integral zeroes | None |
| abs(v)>=1 | Same v, fixed fractional 1 | Fractional trailing zeroes/point | None |
| Otherwise, including zero | Same v, fixed fractional 2 | Fractional trailing zeroes/point | None |

The binary64 division is rounded once before decimal conversion, with the exact
current operand order and no fused or decimal-exact replacement. The unified
engine then rounds that derived exact rational, reproducing the current Rust
fixed/scientific formatting result and prescribed trimming. Custom decimal_scale
has the different exact algebra defined above. Preserve the division result for
negative values too; threshold choice is by absolute value, sign is not stripped.

Automatic examples include `-10000 -> -10k`, `-1000000 -> -1M`,
`-0.001 -> -1.00e-3`, `100.5 -> 100`, `101.5 -> 102`, and actual `-0 -> 0`.
Scientific negative exponent spelling is `e-3`, not `e-03`; positive is `e12`, not
`e+12`. Keep scientific mantissa zeroes. Thresholds use the unrounded operand:
do not reclassify a `999999` input because its k-branch rounds to `1000k`.

A complete Automatic source starts with that magnitude profile. If distinct
values have equal labels, retry the whole list with scientific fractional
precisions **2 through 17**, including retained mantissa zeroes. The private upper
retry is 18 significant digits; it is not the public Significant maximum of 17.
Axis sources then perform existing complete-list geometric fitting/coarsening;
guides perform their existing complete required-set rejection.

### Default Forest Power Grammar

Integral seed exponents are already source-clamped; the literal table is:

| Exponent | Literal |
| --- | --- |
| -2, -1, 0, 1, 2, 3 | `0.01`, `0.1`, `1`, `10`, `100`, `1000` respectively |
| 4, 5, 6, 7, 8 | `10k`, `100k`, `1M`, `10M`, `100M` respectively |
| 9, 10, 11 | `1000M`, `10000M`, `100000M` respectively |
| Other admitted integral seeds | `1.00e` followed by exact signed integral exponent, without + or leading zeroes |

Default adaptive/midpoint labels use ASCII `10^` followed by the Automatic
exponent text, including scientific retry of that exponent where needed. They
are not evaluated powers and do not acquire custom affixes or parentheses.
Configured raw-value formatting bypasses that default operand grammar and uses
the certified mathematical power, through the same presentation engine.

### Fixed Values And Waterfall Names

Historical fixed values delegated to Rust `"{value:.precision$}"` and appended a
literal suffix; the exact engine reproduces that result at precisions 0..=12,
including -0, subnormals rounded to zero and finite extremes. Public maximum 17
is a deliberate extension, not an alias for the old usize constructor.

Waterfall keeps `name + ASCII space + formatted number`, default names `base` and
`total`. Numeric prefix/suffix surround only the number after that separator.
Names are not formatter affixes. For default 10 and terminal 13 the strings remain
`base 10.000` and `total 13.000`. The semantic cumulative terminal still uses its
existing ordered binary64 additions; numeric presentation does not recompute it.

Source: `src/spec.rs:283-340`, `src/plots/waterfall.rs:250-307`.

## Exhaustive Consumer Matrix

The route partition is eight StandardCoordinate plus twelve OneCategoricalAxis
families, and five SpecializedGrid families (`src/layout/interior.rs:40-44`).
The following matrices cover **all twenty ordinary and five specialized scientific
kinds**, plus TextPanel as the twenty-sixth composition input family. No omitted
row is implicitly licensed by a runtime dispatcher.

Legend: **A** = unconfigured Automatic numeric profile; **P** = unconfigured
Forest power profile on Log10; **C** = verbatim categorical source; **G** = required
continuous numeric guide; **V2** = opt-in fixed-two default value; **T3** = existing
fixed-three total. A/V2/T3 indicate defaults, not public notation choices. Ordinary
rows use the native coordinate producer and applicable private prepared scopes.

### Twenty Ordinary Families

| Kind / mark source | Actual x / y and default profile | Public numeric attachments | Values/guides and source-specific obligations |
| --- | --- | --- | --- |
| Scatter / `plots/scatter.rs` | Numeric A / Numeric A | x_tick_format, y_tick_format | Both raw constructor columns are NumericValues; paired finite source, no categorical Scatter mode or numeric value labels |
| Line / `plots/line.rs` | Numeric A / Numeric A | x, y | Actual ordinary Line x must be numeric; optional bounds, string PointLayers and discrete legend unchanged |
| Trajectory / `plots/line.rs` | Numeric A **or categorical C** / Numeric A | x, y capability, actual x checked | Only this line-family preparation accepts categorical conditions; categorical x format rejects before fonts, y remains configurable |
| LinePath / `plots/line.rs` | Numeric A / Numeric A | x, y | Constructor x is NumericValues; source-order path and row hue do not become numeric labels |
| Histogram / `plots/histogram.rs` | Numeric A / Numeric A | x, y | Sample-derived and caller-supplied bin geometry share axes; count/density/scientific heights are not rescaled |
| Density / `plots/density.rs` | Numeric A / Numeric A | x, y | Support and density ordinate stay scientific; generated y anchored-zero policy unchanged |
| Qq / `plots/qq.rs` | Numeric A / Numeric A | x, y | Theoretical and observed coordinates retain prepared range validation; no observation labels |
| Regression / `plots/regression.rs` | Numeric A / Numeric A | x, y | Fit/band/statistics and literal legend entries unaffected |
| PointInterval / `plots/interval_estimate.rs`, `point_interval.rs` | Numeric A / Categorical C | x | Availability/interval clipping unchanged; no estimate_labels attachment |
| Coefficient / `plots/interval_estimate.rs` | Numeric A / Categorical C | x, existing estimate_labels | V2 only when requested and point-only; intervals plus estimate_labels reject; Not estimable and stars separate |
| Forest / `plots/interval_estimate.rs` | Numeric A Linear or P Log10 / Categorical C | x | Configured Log10 raw value uses certified operand; no new estimate-label attachment |
| Dumbbell / `plots/dumbbell.rs` | Numeric A / Categorical C | x | Both endpoint identities/paints remain literal; no generated endpoint values |
| Bar / `plots/bar_data.rs`, `bar.rs` | Categorical C / Numeric A | y | Single/grouped and sample/supplied summaries retain zero-anchored numeric axis; no bar-value labels |
| HorizontalBar / `plots/bar_data.rs`, `horizontal_bar.rs` | Numeric A / Categorical C | x | Sign legend and point layers unchanged; formatting follows actual horizontal numeric side |
| StackedBar / `plots/bar_data.rs`, `stacked_bar.rs` | Categorical C / Numeric A | y | Ordered stack sums and source spans stay scientific; no segment labels |
| StackedHorizontalBar / same owners | Numeric A / Categorical C | x | Same stack grammar in horizontal orientation; no y numeric attachment |
| Boxplot / `plots/categorical_distribution.rs`, `boxplot.rs` | Categorical C / Numeric A | y | Category/group slots, summaries and fliers are not format inputs |
| Violin / `plots/categorical_distribution.rs`, `violin.rs` | Categorical C / Numeric A | y | Optional raw overlays and density/bandwidth unaffected |
| Raincloud / `plots/categorical_distribution.rs`, `violin/precomputed.rs` | Categorical C / Numeric A | y | Sampled/supplied geometry and mandatory strip unchanged; no group/value-format surface |
| Interaction / `plots/interaction.rs`, `interaction/native.rs` | Categorical C / Numeric A | y | Condition x and style/hue identities remain strings; estimates use y ticks only |

Every ordinary row permits generic facets and supported Canvas embedding under
the existing parent rules. A Canvas cannot embed an already faceted plot. Facets
retain local row membership with their existing parent fixed/free numeric domains.

Axis orientation source: `src/plots/shared_x/prepared.rs:1159-1354`,
`src/layout/interior/coordinate.rs:256-283`. Conditional Trajectory source:
`src/plots/line.rs:432-456`; actual Scatter fields: `src/input.rs:138-143`.

### Five Specialized Families And TextPanel

| Kind / source | Actual axes/defaults | Additional numeric surface/default | Public binding and parent boundary |
| --- | --- | --- | --- |
| Heatmap / `plots/heatmap.rs`, `heatmap/guide.rs` | C / C; no numeric tick grammar | G Automatic; opt-in finite active-cell V2 | color_label_format and Values payload only; generic facets and Canvas; categorical x/y numeric format unsupported |
| Matrix / `plots/matrix.rs` | Numeric A on its own cell axes; diagonal density projection is internal | No new numeric guide/value surface; its guide is discrete | No NumericX/Y/ContinuousColor binding; no global format; own Grid, cannot be generic-faceted or Canvas-embedded |
| Marginal / `plots/marginal.rs` | Central Numeric A / Numeric A; strips copy central tick positions without labels | No numeric strip ordinate/value surface; guide is discrete | x/y formats on central axes; no new strip format; Canvas permitted, generic facets excluded |
| Beeswarm / `plots/beeswarm.rs` | Numeric A / C | G Automatic, independent color domain | x_tick_format and color_label_format; no numeric point labels; generic facets and Canvas |
| Waterfall / `plots/waterfall.rs` | Numeric A / C (value x, ordered steps y) | Domain-visible T3 base/terminal shelves | x_tick_format and total_format; total_labels owns names; generic facets and Canvas |
| TextPanel / `canvas/text_panel.rs` | No scientific axis | Caller title/body strings only | No NumericFormat attachment, parsing or numeric inheritance; existing Canvas prose owner |

Source: `src/plots/shared_x/prepared.rs:1355-1421`,
`src/plots/matrix.rs:50-57,376-407`, `src/plots/marginal.rs:633-708`,
`src/layout/interior.rs:73-88`, `src/kind.rs:181-185`.

### Scatter Axis Modes And Exact-Source Precedence

At the source baseline there is no `ScatterAxis` public value/type and no
categorical Scatter input. The actual per-axis modes are independent axes with
range provenance Automatic, Configured or Shared, and supplied/exact/generated
tick-source provenance. This distinction must survive the formatter migration:

| Scatter axis mode | Range/projection owner | Text contract |
| --- | --- | --- |
| Automatic range/generated ticks | Prepared paired finite extent, automatic domain feedback | Selected candidate uses effective format; geometry may coarsen sources, never change policy |
| Explicit limits/generated ticks | Caller finite ascending window; automatic-domain change forbidden | Same configured/default number policy, constrained to existing source/fit rules |
| Coordinated Shared range/generated ticks | Parent union and alignment under current provenance | Per-panel format retained; range sharing alone does not share text |
| Exact ticks in any valid range mode | Caller exact data positions, current containment validation | Verbatim labels suppress generated format and interval requirements |

Do not reinterpret Scatter as a mixed category/numeric source to exercise a new
setter. Trajectory is the real conditional-category fixture. Line's AxisValues
storage is not proof of categorical applicability: its preparation rejects that
case (`src/input.rs:158-169`, `src/plots/line.rs:433-455`).

## Semantic Format Sets And Adaptive Precision

An exact format never increases precision, changes notation or repairs rounded
duplicates. Required generated axis candidates must be unique; a collision is a
source miss before geometric fitting, preserving current coarser-source order.
Required continuous guide values must be distinguishable; collision is a pre-font
data-presentation error. Exact value annotations/totals can intentionally display
the same rounded number.

Adaptive precision chooses the smallest admitted precision in `[min,max]` at
which every pair of **distinct scientific members** has different number text.
Apply that precision uniformly to the complete owner set, not per label. Equal
values, including signed-zero scientific equivalence, can repeat; Preserve may
still retain different zero signs in their occurrence strings. Affixes are equal
within an owner and cannot rescue numeric precision by changing per-value units.

Notation, scale, affixes, trailing/sign rules and membership are fixed semantic
inputs. Increasing precision is solely a uniqueness decision, never a response to
insufficient ink width. Adaptive exhaustion is an axis source miss or a pre-font
presentation error for guides/values/totals. Resource/certification refusal stops
immediately. An empty value/total set has no uniqueness work or fabricated label;
retain the policy and choose its minimum only if a resolved precision is needed
for identity. A singleton succeeds at its minimum. Axis source emptiness continues
to follow the existing source planner.

### Membership Matrix

| Owner-local set | Concrete scientific members | Excluded members / ordering and visibility |
| --- | --- | --- |
| Generated numeric axis | Every selected position of one complete source in the current effective range; Log10 uses exponent identities | No omitted collisions, ink-selected subsequence or values from another candidate; positions preserve source order |
| Continuous guide | lo, strictly interior diverging center if present, hi; constant domain one value | Missing key, color samples and gradient stops are not numbers to label; final descriptor order is lo/center/hi |
| Heatmap panel Values | Every Some finite raw value in the panel's active prepared cells; retain each cell occurrence | Null/inactive cells and merged fully-null-row caller text excluded; no color saturation, point-window or narrow-cell filtering; render/census cell order retained |
| Coefficient estimate_labels | Every available finite point-only prepared estimate in that panel | No interval bounds, unavailable row, reference, significance glyph or PointInterval/Forest annotation; do not exclude a Coefficient estimate merely because its point is outside an exact x window |
| Waterfall panel totals | Base and ordered cumulative terminal eligible under the semantic effective window, with distinct named occurrences | Deltas, intermediate endpoints, categorical step labels and name text excluded; eligibility is domain policy, not shelf fit or collision |

Heatmap source: `src/plots/heatmap.rs:1417-1459,1499-1519`; its active-layout
validation forbids supplied inactive finite values. Coefficient source:
`src/plots/interval_estimate.rs:593-598,713-720,817-898,971-993`. In particular,
Coefficient prepares/places its adjacent label for each available tuple before
mark clipping; changing numeric precision must not introduce a new outside-window
omission rule.

### Waterfall Eligibility And Potential Text

Retain both numeric total candidates and their named identities regardless of
clipping. Current preparation materializes both strings; the total shelf consumes
one only if `projection.contains_data(value, source_y)` with
`source_y = category_count`. Thus x and categorical y window membership matter;
being on the closed boundary is eligible. A shelf-width/collision failure is not
permission to remove an otherwise eligible total.

Current intrinsics skip total values outside `user_x_limits`; the current
foundational/annotation census filters to x limits only when its explicit filter
flag is enabled; conservative composed/facet census can retain both. Preserve
that distinction between **potential font requirements** and **actual paint
eligibility**. Do not use an overinclusive census to make a suppressed total
increase adaptive precision, and do not narrow the census to just the current
shelf items if an admissible coordinated window can paint another candidate.

Resolve adaptive eligible membership after the parent fixed/free/shared initial
domains are prepared and before fonts. Automatic total domains contain both
totals; immutable explicit exclusions are not broadened locally. Parent union may
make a formerly local outside-window total eligible; evaluate the parent's actual
resolved window rather than the original local extent. If a legitimate subsequent
domain consequence changes eligibility, recompute the set from the **retained
numeric candidates**, not old strings, under the same policy. Retain a finite
potential-text grammar for both candidate names/numbers before fonts so this
cannot append a new font requirement during feedback. Record membership/precision
in the outcome identity. Under ordinary automatic expansion both totals were
already members, so expansion changes placement, not precision.

For hidden candidates, exact/default strings may remain in semantic storage as
today, but no suppressed font occurrence is charged as painted content. Adaptive
uniqueness is over eligible scientific members only. If the y window suppresses
the total row entirely, there is no eligible total set; this does not remove bars,
connectors or their prepared source slots.

Source: `src/plots/waterfall.rs:147-165,193-205,257-267,714-735,761-811`;
`src/layout/text/requirements.rs:1388-1412`.

For Waterfall's two retained candidates, the possible eligibility masks are a
finite four-state set: neither, base only, terminal only, both. Before fonts,
resolve the initial mask and any other mask that an admitted domain consequence
can actually reach. Prepare their adaptive precision/results or their explicit
presentation refusal; do not wait until measurement to discover that a reachable
new required set cannot be formatted. An immutable exact exclusion makes its
excluded masks unreachable and must not cause an unnecessary precision/error.
Actual processing/storage is admitted for examined reachable sets; immutable
results may share storage. During feedback, a changed mask selects/rebuilds only
the already admitted mathematical/result context and never restarts font
resolution or an unbounded local layout search. Thus presentation validation is
pre-font even when the final eligible shelf set is selected later.

### Parents And Regeneration

Facet Fixed/FreeY share x domains; Fixed/FreeX share y domains. The format comes
from the owning Plot and is copied into every prepared scope. Value sets remain
per-panel; sharing an axis domain does not merge Heatmap/estimate/total value
annotations into one set. Categorical shared axes retain exact raw-level/order
checks; do not compare formatted numbers as category keys.

Canvas x-range groups coordinate numeric domains and gutters but permit different
per-panel format policies. A genuinely shared displayed axis requires equal
effective format/operand context before selecting one shared label source. Existing
parent rules, not a new global format control, determine which owner paints it.

Canvas Shared continuous guides require equal normalized **effective** policies
across contributors before fonts, including private default versus explicit policy
semantics. Equal current strings or widths are insufficient. Reject mismatch; no
first-contributor-wins or Canvas format override. Merge scientific domains under
existing exact/automatic rules, then format one final required set. Child local
guides are suppressed, while value labels and final fill/contrast roles remain.
No independently formatted child guide may reject a locally ambiguous precision
before the owner knows it will be suppressed in favor of the final Shared set.

Current descriptor merge: `src/decorations/legend/descriptor.rs:131-215`;
Canvas preparation order: `src/canvas.rs:4063-4121`; facet range reconciliation:
`src/plots/shared_x/prepared.rs:769-786,1090-1124`.

Heatmap/Beeswarm Shared color-domain installation is preparation, not root
feedback. It regenerates guide numbers, fills/stops and contrast choices before
census. Heatmap raw value numbers do not change when colors saturate or domains
merge. Coordinate feedback may regenerate selected axis positions, mathematical
operands, labels and resolved adaptive precision through the prepared grammar.
Known value sets retain their raw members; domain-sensitive Waterfall eligibility
is the narrow rule above. Formatting is never a new feedback kind or root-width
reselection.

Storage order is not inferred from drawn geometry. At the refreshed baseline,
`descriptor::domain_labels` already prepares canonical lo/interior-center/hi or a
single scientific constant for both families; Heatmap appends optional `Missing`
separately. Native local guides copied those numeric lists in order. The historical
natural `guided_preference` reindexing adapters have since been removed; both
consumers use the canonical retained descriptor (`docs/coordinated-numeric-guides.md`).
No stop sample, projected label position
or arbitrary child label chooses the scientific list. Preserve explicit
occurrence/index relationships and attribute changed census order/content to its
actual source migration, not to the old baseline's storage claim.

Source: `src/plots/heatmap.rs:1543-1563`, `src/plots/heatmap/guide.rs:19-44`,
`src/plots/beeswarm.rs:177-182,229-239`,
`src/decorations/legend/descriptor.rs:330-352`.

### Deterministic Distinctness And Comparison Work

Build a retained occurrence map and sorted distinct scientific members without
changing paint/source order. Normalize signed-zero equality for distinctness only;
keep occurrence sign/provenance for Preserve output. Use bounded deterministic
ordering and full comparisons, with checked lengths and fallible retained
buffers. The current `data::source::sort_source_by` is a nonallocating heapsort;
it has no comparison pricing or quota failure. Spec95 removes the former
accounting-only requirement for a Result-returning comparator. If a comparison
actually performs fallible conversion/arithmetic, propagate that real failure;
never store an error and fabricate an Ordering.

For explicit notation, equal numeric output can be checked using its canonical
rounded coefficient/exponent identity or full text comparisons. Prove any
adjacent-only optimization for the whole configured grammar; do not reuse a
default monotonicity comment as evidence for arbitrary future conversions. A
bounded label-index ordering with full comparisons is valid if that proof is
not used. Default guides' at-most-three pair checks remain small. Hash-only
identity, omitted collisions and an unbounded quadratic distinctness scan are
not adaptive shortcuts. Failed precisions retain their real failure/uniqueness
meaning without cumulative accounting scores.

## Field-Level Transport And Deletion Census

This table identifies historical fields at `e6febe3` and the required
replacement responsibility. New private field spellings below describe owners,
not additional public setters. The refreshed handoff above supplies current
names/copy paths and deletions. Affix storage may be immutably shared; copied
bytes/records retain checked lengths, fallible storage and complete equality.

| Current field/call path | Required transport and retained identity | Source anchor |
| --- | --- | --- |
| `NumericLabelFormat.{precision: usize,suffix}` and `NumericLabelDisplay` | One NumericFormat's seven fields; old constructor/display/string production deleted at cutover | `src/spec.rs:230-340` |
| `AxisSpec.{provenance,limits,exact_ticks,interval}` | Add optional complete format to the axis owner; retain complete exact/supplied/default source context, not an inferred format from labels | `src/layout/panel.rs:51-60` |
| `Plot.{x_axis_spec,y_axis_spec}` | Format travels with the spec across limits/interval edits and planned/shared ranges; exact labels suppress its effective generated path | `src/builder.rs:450-453` |
| `Plot.heatmap_annotations`, `estimate_labels` | Replace payload/optional type with the same NumericFormat; presence remains separate from format fields | `src/builder.rs:478-481` |
| `Plot.color_label`, continuous scale fields | Add independent optional guide-number format; do not fold it into title or gradient/domain selection; B-stage scale replacement is consumed under its then-current name | `src/builder.rs:469-477` |
| `Plot.waterfall_labels` | Add independent optional total-number format; names and literal space remain separate owners | `src/builder.rs:537`, `src/builder/configuration.rs:568-574` |
| `Plot::new` and explicit `erase` destructure/reconstruction | Initialize absence consistently and move every new field, without capability changes on Any | `src/builder.rs:580-604,999-1150` |
| Derived `Plot::clone` and exhaustive `erase` (refreshed owner) | Retain every complete policy/affix; checked explicit copies remain fallible. The historical admitted_clone/suffix-only accounting helper was deleted under spec95 | `src/builder.rs`, refreshed handoff |
| `ComposedInputs.{x_axis_spec,y_axis_spec,resolved_facet_x_range,...}` | Include retained normalized x/y formats and any per-child effective presentation replacement; admit copies/affix comparisons; do not rely on equal endpoints | `src/builder.rs:561-577`, `src/builder/facets.rs:128-174,196-218` |
| `with_resolved_facet_ranges`, planned-domain edits | Preserve format while replacing only range/provenance; Log10 resolved_x is transformed domain context, not decimal display scale | `src/builder/facets.rs:73-95`, `src/decorations/plan.rs:602-606` |
| `PreparedAxes.{ranges,categories,display_categories,labels,specs,logarithmic}` plus scope tuple | Retain effective numeric format/operand and exact/category suppression under each scope; no formatting during category-key normalization | `src/plots/shared_x/prepared.rs:10-20` |
| Explicit spec reconstruction in shared-axis `prepare` | Copy/admit format along with interval/limits/exact ticks; Log10 exact projection changes positions only | `src/plots/shared_x/prepared.rs:1435-1468` |
| `PlotRenderContext::layout_with_ticks` explicit spec reconstruction | Copy format/operand metadata here as well; category and prepared_limits substitution cannot reset it to default | `src/spec.rs:939-1008` |
| `display_axis_spec`, `AxisSpec::layout_ticks`, `NumericTickPlan` | Range/display adapters preserve policy; source selection records resolved precision, operand kind and complete labels for the retained result | `src/layout/panel/range_ticks.rs:127-138`, `src/layout/panel/geometry.rs:174-229`, `src/layout/panel/sources.rs:497-605` |
| `coordinate::InteriorRoute::measure` and `interval_estimate::log_tick_plan` | The special no-exact Log10 branch must pass the effective x format; the ordinary layout_ticks branch passes each spec format; exact/category branches remain explicit | `src/layout/interior/coordinate.rs:333-413`, `src/plots/interval_estimate.rs:1104-1134` |
| `PreparedInteriorInputs.axes` and per-family `panels: Vec<(Option<String>,...)>` | Attach shared immutable scoped data to the matching Plot; formats belong to the prepared owner, not unscoped array position or a reconstructed Plot clone | `src/layout/interior.rs:96-119,153-172`, `src/plots/shared_x/prepared.rs:403-477` |
| `PreparedMatrixAxes.{ranges,labels,groups,hues,diagonals}` | Cell engine uses built-in automatic profile; preserve cell owner/signatures with no public format storage or Matrix setter | `src/plots/matrix.rs:50-57,376-407` |
| `HeatmapData.{cells,fills,layout,value_labels,value_owners,colorbar_values,color}` | Retain distinct value/guide policies, raw cell set, resolved precision and actual concatenated labels; guide-domain replacement must not replace value policy or numbers | `src/plots/heatmap.rs:26-42,91-127,1499-1536` |
| `PreparedEstimate.{estimates,adjacent,range,kind,point_interval}` | Numeric adjacent text retains policy/raw set/resolved precision separately from significance boolean, availability and transformed bounds | `src/plots/interval_estimate.rs:17-30,713-729` |
| `PreparedWaterfall.{labels,range,spans,totals}` | Retain two raw candidate values/names, total policy, eligibility/resolved precision and combined text; shelves use those retained identities | `src/plots/waterfall.rs:33-44,250-274,761-811` |
| `PreparedBeeswarm.{rows,scale,values,colorbar_stops}` | Guide numbers depend on effective guide policy and final domain; rows/colors stay scientific; target diverging center comes from the approved B-stage scale | `src/plots/beeswarm.rs:22-31,177-205,229-250` |
| `ContinuousSemantics`, `ContinuousContributor`, `ContinuousDescriptor` | Contributor and descriptor retain normalized effective numeric policy and required raw values/resolved precision alongside current field/title/stops/center/missing/domain/exact/labels; samples are not label members | `src/decorations/legend/descriptor.rs:29-59` |
| Canvas final Shared-domain installation | Pass the descriptor's compatible policy and final required set to children before fonts; do not independently choose another child precision | `src/canvas.rs:4091-4119`, `src/plots/heatmap.rs:91-127`, `src/plots/beeswarm.rs:229-250` |
| Local descriptor literals and natural `guided_preference` | Copy/admit the same effective policy, raw required list and selected precision as the prepared guide; neither local measurement nor natural width may reconstruct a default-only descriptor | `src/plots/heatmap/guide.rs:33-44`, `src/plots/beeswarm.rs` guide measure, `src/layout/interior/sizing.rs:151-224` |

`format_label`, `format_label_parts`, `format_guide_label`, fixed
NumericLabelDisplay and Waterfall's `write!(..., "{value:.3}")` are deletion
targets as separate conversion paths. Call adapters may remain to provide source
or text admission, but all delegate to the one engine and do not own numeric
rounding. Test-only `range_ticks::{format_tick,distinct_tick_labels}` migrate to
the current engine rather than preserving an old formatter as an oracle.
`nice_range`, `decimal_power` used for tick steps, scientific `log10`, source
generation, color interpolation and SVG coordinate serialization retain their
different mathematical/serialization ownership.

## Measurement, Grid And Identity Dependencies

### Actual Cache And Signature Owners

| Owner and current fields | Numeric migration obligation | Geometry versus semantic distinction |
| --- | --- | --- |
| Coordinate `Intrinsics.{standard,minimum_body,without_y_categories}` cached on PreparedAxes OnceLock | Terminal-source text uses effective format/operand; retain normalized axis presentation identity in the summary and verify it with typography | Domain-specific midpoint text is an intrinsic input; panel width is later assigned geometry, not permission to reuse a summary of another policy |
| Natural `plot_size` y low/mid/high probe | Use effective policy and exact ticks/category precedence; preserve its sizing-estimate role, not guide uniqueness rejection for an artificial probe set | A probe estimates preference, never selects the final axis source or permanently fixes adaptive precision |
| Specialized natural/body minima | Beeswarm/Waterfall coordinate summary, Marginal central axes and Matrix private axes use the same engine; Heatmap uses actual retained value plans | Geometry minima depend on real glyph/stroke occupancy, not string length or nominal precision |
| `HeatmapIntrinsicCache` complete appearance/scale key | Add value presentation/label-set identity or enforce its immutable prepared identity explicitly; changed format at the same color domain cannot reuse old text minima | Complete selected contrast roles and surfaces remain dependencies; numeric policy is not a paint |
| `PanelAxisPlan.{projection,semantics,standard,x_axis,y_axis,...}` | Retain effective policy/operand/resolved precision per axis and observe them before/with selected sources/text | Frozen cells, tick segments, gutters and overhangs are derived outcomes, not policy inputs |
| `AxisSemanticInputs.{provenance,scale,policy,interval}` | Add effective generated format/suppression and operand context; affix-bearing identity cannot remain a Copy-only four-scalar record without an owned/shared value | Raw scientific AxisScale::Log10 differs from decimal_scale; preserve both distinctly |
| Facet `reconcile_facet_axis` / `facet_axis_signature` | Category signatures remain raw keys/order; shared numeric displayed-axis compatibility includes effective format/operand, not a category string signature | Tick labels, projected cells and union domains are results of separate semantic owners |
| Grid child `decorations::plan::children::Inputs` | Retained ComposedInputs carry complete numeric dependencies; same-domain/same-width changed policy misses; retain root/child identity and scales/feedback | owners, geometry, axis_label_width, content_height/fill flags, standard/theme, scales and ChildFeedbackInputs remain independent inputs |
| Canvas `HeightMeasurements` / `HeightInputs` | Include effective presentation identity (x/y/guide/value/totals as applicable) and any regenerated set identity; retain width/ranges/provenance/resolved_x/insets/y_labels/legend/continuous_hidden/scope | A cached height is a consequence of these inputs, not a source value; equal height must not conceal a changed policy |
| `PreparedCanvasInputs` and `PreparedCanvasRound` | Final descriptor, scoped interiors/census/intrinsics retain one policy version per operation; shared_x range replacement preserves it | Body probes, tracks/root boxes and selected guide geometry are derived per-round outcomes |
| `theme::TextQueryCache` / `CachedTextQuery` | Query the actual complete new source with the selected role and pair; full source comparisons still defeat fingerprint collisions | Do not add formatter semantics to glyph shaping unnecessarily: identical final text/role/provenance can share glyph analysis even when an upstream numeric policy differs |
| Feedback observation, history and child replay | Observe policy fields/operand/set/source/precision plus actual texts; compare exact normalized state; resource counters remain excluded | Selected source/collision/track decisions are payload, not new feedback kinds |
| `ProducerText` and selected/frozen text replay | Reuse retained complete source/owner plans, never regenerate numeric text at capture, Null validation, SVG or PNG draw | Equal source lookup is a text-plan lookup, not proof of numeric policy equality |

Source anchors: `src/layout/interior/coordinate.rs:15-57,150-166`,
`src/layout/interior/sizing.rs:442-477`, `src/plots/heatmap.rs:499-534`,
`src/layout/panel/semantics.rs:26-63`,
`src/layout/panel/axis_frame_grid.rs:60-101,171-220`,
`src/orchestrator/facets/legend.rs:224-315`,
`src/decorations/plan/children.rs:25-119`, `src/canvas/sizing.rs:19-118`,
`src/canvas.rs:801-826`, `src/theme/resolved.rs:107-165,217-317`,
`src/spec.rs:1016-1042`.

For terminal-source intrinsics, an explicit adaptive format operates on the
singleton terminal midpoint and therefore uses its minimum precision. Retain the
prepared base range/policy/operand that generated this intrinsic; it is not a
claim that a later expanded-domain candidate already fits. Final candidate fitting
and root gutter consequences still validate the current text.

Natural-width estimates are not synthetic required guide sets. For an exact
explicit policy, measure the low/mid/high probes at its requested precision. For
an explicit adaptive policy, measure the finite envelope of those probes at each
allowed precision and retain their maximum complete width; admit each actual
conversion/measurement. This is a bounded preference estimate, not uniqueness
rejection or a permanently selected axis precision. With no override, retain the
current default low/mid/high magnitude-probe behavior. Exact labels and categorical
sources instead use their actual text. No natural estimate parses labels, changes
source density or causes width to be chosen again after feedback.

Semantic format equality and retained-key comparisons include affix bytes with
actual comparison admission. Upstream children/height/axis keys distinguish a
changed policy even if it currently produces the same width/source string. The
lower text cache intentionally has a different boundary: source, stack, size,
weight and prepared_only determine shaping; width, paint, leading, alignment,
owner, pair ordinal, exact override and preferred lines determine presentation.
Retain its full equality check after indexed fingerprints. Numeric policy does
not needlessly duplicate that key once its resulting complete text is fixed.

### Complete Dependency Audit: Twenty-Six Inputs

These are the twenty-six semantic/context input classes to exercise in identity
and transport evidence, **not twenty-six new public fields**. Existing downstream
text-cache keys may legitimately omit an upstream class once its complete derived
source/role is fixed; the owner table above specifies that boundary.

| Input class | Required distinction |
| --- | --- |
| 1. Notation | Decimal/Scientific/Engineering, even when one number has equal text |
| 2. Precision | Exact versus adaptive, kind and complete min/max |
| 3. Decimal scale | Exact signed shift; not scientific domain or AxisScale |
| 4. Prefix | Absence/presence and all literal bytes/units |
| 5. Suffix | Same, including leading marks and spaces |
| 6. Negative-zero policy | Preserve/Normalize despite a currently positive set |
| 7. Trailing-zero policy | Keep/Trim despite a currently integral set |
| 8. Effective default/explicit profile | Default automatic/power/fixed rules versus explicit normalized format |
| 9. Operand kind | Ordinary exact binary64 versus mathematical power versus default compact derivation |
| 10. Axis/scientific scale | Linear/Log10 source and raw/projected exponent identity |
| 11. Exact/supplied/generated precedence | Exact text suppresses generated policy; category source remains categorical |
| 12. Preferred interval | Additive/decade source instruction and parameter |
| 13. Range provenance | Automatic/Configured/Shared at equal endpoints |
| 14. Prepared base and consumed domain | Endpoints, transformed resolved_x and named envelope consequences |
| 15. Source positions and provenance | Full candidate, ordinal/kind/parameter, source-row identity |
| 16. Owner/scope | Root address, facet/Canvas slot and render_scope |
| 17. Eligible value membership | Availability, active cells and Waterfall window eligibility; no ink filtering |
| 18. Resolved precision and occurrences | Selected common precision, duplicate occurrences/signs and raw-to-label map |
| 19. Continuous descriptor semantics | Field/title/stops/center/missing/exact-domain and final required values |
| 20. Complete role/font environment | Ordered stack, weight, size, leading, paint, pair and retained face provenance |
| 21. Actual source/unit content | Entire concatenated text and retained base-plus-mark boundary selections |
| 22. Assigned constraints/geometry | Width, height, panel, label capacity/obstacles, root request |
| 23. Content-height and fill mode | Separate root/child sizing behavior under the same width |
| 24. Visibility and legend authority | y label visibility, annotation presence, local/Shared suppression |
| 25. Parent scales/alignment/insets | Facet policy, group membership, current own/parent reserves and gutters |
| 26. Consumed feedback and exact plan state | Current domain/ink/alignment dependencies, complete selected/frozen text outcomes |

## Pre-Font Text Grammar And Fallback Faces

### Known Labels Versus Later Generated Sources

Preparation resolves applicable policies, parent domains, Shared compatibility and
known value/guide sets before font census. Domain-known labels contribute their
**actual concatenated strings**, including affixes and the Waterfall name/space.
Their base-plus-mark units come from that complete text, not detached affix scalar
coverage. Select Heatmap's complete contrast role after final fills/backdrop before
requesting its text. Shared color-domain changes precede this role selection.

Later axis candidates and any legitimate domain-sensitive regenerated text
contribute a **finite closed grammar**, not guessed example labels. Numerical
value space, output length and certification precision are not enumerated.

| Effective generated source | ASCII numeric/punctuation alphabet before literal additions |
| --- | --- |
| Explicit Decimal | `-.0123456789` (omit an impossible sign where provable) |
| Explicit Scientific/Engineering | `-.0123456789e` |
| Configured Forest raw value | Corresponding explicit notation alphabet; no generated `^` or parentheses |
| Built-in Automatic | `-.0123456789Mek` |
| Built-in Forest power/expression | `-.0123456789M^ek` |
| Exact ticks/categories | No numeric generated alphabet; actual caller/display strings only |

Retain alphabets in canonical scalar order, including the fixed numeric pitch
probe `0` when measurement consumes it. Scale/precision refinement can change
length/exponent/sign/carry but cannot introduce `+`, grouping, locale digits,
Unicode minus, infinity or NaN. Literal affixes can of course contain otherwise
unsupported grammar characters, subject to constructor and real font validation.

Source: `src/layout/text/requirements.rs:33-45,1846-1897`,
`src/layout/panel/sources.rs:9-12,512-528`,
`src/layout/text/requirements.rs:1380-1386` (Matrix built-in alphabet).

### Finite Numeric-Affix Boundary Units

The baseline requirement has `source` and `generated_alphabet`, but no dedicated
generated-unit list. Add a private retained boundary-unit requirement (or equally
explicit inline requirement records) and feed it through the canonical pair/unit
preparation path; a character union alone cannot satisfy it.

1. Admit actual prefix/suffix strings and their internal `selection_units` spans.
2. Derive the finite possible first/last scalar contexts from the effective grammar
   and source sign/notation/trim rules. A conservative closed superset is valid;
   sample label guesses are not.
3. For a suffix beginning with visible combining marks, join that leading mark
   sequence to **each possible numeric terminal scalar**, then obtain canonical
   selection units from the concatenation. Examples are `0\u{0301}` through
   `9\u{0301}`, and default `k\u{0301}`/`M\u{0301}` where those terminals are possible.
4. Derive applicable prefix/numeric boundary units through the same actual
   concatenation/selection operation; do not assume arbitrary graphemes are the
   renderer's narrow base-plus-mark units. A prefix ending in a base-plus-mark
   unit retains that unit; a leading mark without a base retains the existing
   permitted scalar treatment, not an invented dotted-circle glyph.
5. For Waterfall's potential later-generated full labels, retain actual name/space
   boundary units too; the literal separator and name are caller/renderer content,
   not number grammar characters inferred from a palette or axis title.
6. Charge strings, scalars, unit traversal, comparison/deduplication and retained
   records at actual occurrences under the existing shared ledger. Finite grammar
   derivation is not authorization for an unpriced Cartesian value enumeration.

This requirement extends the current actual-source unit path, rather than opening
late font fallback. Source scalars alone can choose different faces than a complete
base-plus-mark unit; preserve the current preference for one face covering the
whole unit when possible.

### When Faces Are Selected

The prepared census fixes canonical `(stack,weight)` pair ordinals and coverage.
During canonical font resolution, before any numeric text measurement:

1. Discover admitted generation-host candidates in existing order.
2. Resolve each pair's required scalars.
3. Resolve and retain whole-unit face selection, or the existing allowed scalar
   fallback when no candidate covers the complete unit, for every actual and
   generated boundary unit.
4. Seal prepared pairs/units.

Measurement/shaping must only consume those retained identities. It cannot add a
numeric-affix unit, choose a new face, reparse a stack or discover a missing suffix
glyph when a new tick appears. Unknown unit/missing coverage is fatal before
backend begin. Pair/role requirements must be correct for both standalone and
Canvas/facet traversal, including selected contrast roles and suppressed content.

Actual baseline route: `PreparedTextCensus::prepare_system_fonts` then
`prepare_pairs_in` (`src/layout/text/requirements.rs:515-537,642-666`),
`PreparedFontEnvironment::prepare_source_units`
(`src/font/canonical.rs:1423-1527`), `seal_prepared_pairs` (`1415-1420`), and
`ResolvedFontPair::selection_for_unit` (`690-715`). The narrow unit definition is
`selection_units` (`728-747`); it is not whole-word/emoji fallback.

### Domain Feedback And Coverage

Generated axes can cross a notation threshold or choose a higher adaptive
precision during root feedback. Their full grammar and affix units were prepared
already. Domain-known value labels usually keep their strings; changing a domain
does not rescale Heatmap values or Coefficient estimates. Any permitted
Waterfall set regeneration uses retained numeric candidates and names through its
prepared potential grammar, not new caller strings. Shared continuous guide
merging/installation remains pre-font preparation and is not a later feedback
operation. Freeze final strings, precision, face/glyph plans and placements once;
outlined SVG and PNG replay them without numeric conversion or fresh shaping.

## Storage, Determinism And Failure Boundaries

Keep source/text ownership, checked real lengths and fallible buffer growth.
The approved accounting simplification removes production quotas, work/byte
prices and accounting-only parameters. Long affixes are constructor-valid but
may fail actual representation, allocation or layout fit; there is no arbitrary
small affix cap. Mathematical certificates, exact coefficient arithmetic and
Forest temporary-size bounds remain required in their numeric owner.

The existing root/guide coordination rules and PNG-local format limits in spec05
remain. A formatter cannot introduce a second convergence driver or change the
candidate sequence. Retain useful actual refinement/cache diagnostics in the
existing test framework without cumulative resource scores or charging guards.

Format choice is deterministic from exact operands/policy and complete set
identity. Actual source text, fonts/units, constraints and consumed feedback
determine measurement. Cache enablement and encounter order cannot choose another
precision or spelling. Different host fonts can affect fit under the existing
font contract, not numeric conversion. Preserve-text SVG retains source spelling;
outlined SVG/PNG share frozen glyph geometry. Diagnostic float formatting and SVG
coordinate serialization are different owners and are not replaced by this API.

| Failure | Required handling |
| --- | --- |
| Invalid format field or actual categorical-axis instruction | Existing Data error before fonts |
| Exact/generated numeric labels collide | Axis complete source miss; no selected subsequence |
| Required guide/adaptive value set cannot be distinguished within bounds | Data presentation error before fonts |
| Mathematical certificate, arithmetic, allocation or font failure | Fatal; no coarsening, notation fallback or partial label |
| Complete measured label/guide/value/shelf cannot fit | Existing typed fit miss/root named consequences; no shrink/elision/precision lowering |
| Exact caller tick labels collide geometrically | Existing caller-owned strict fit rejection; no formatting rescue |

## Verification Design

No tests/builds are part of this documentation closure. The following are
implementation acceptance fixtures, not reports of checks already performed.
Use existing Rust test support and the separate numeric proof/oracle owner; no
Python files, tools directory, network, provisioning or new dependency.

### Independent Arithmetic And Default Evidence

Use exact input bits/rational identities and literal expected strings. Do not
calculate expected output with the engine being tested. For ordinary dyadic
rounding, an independent small-rational quotient/remainder oracle and handwritten
tie/carry cases are sufficient; large extremes can use independently recorded
decimal coefficients/power shifts. For Forest, use the separately implemented
repeated certified integer-square-root/product oracle and small-denominator
algebraic boundary comparisons; store exponent bits, scale, precision, interval
certificate and provenance in bounded Rust fixtures. The separate proof gate must
close before fixture success is described as certification.

For default preservation only, current Rust fixed/scientific conversion is an
independent spelling reference **after** the prescribed binary64 compact division;
do not keep the old project formatter. Include negative branches, negative
exponents, threshold neighbors via next_up/next_down, compact rounding carries,
branch-specific trimming and existing default Forest literal table. Distinguish
conversion-byte equality from later approved theme/sizing geometry changes.

### Twenty-Six Concrete Fixture Groups

| Group | Independently calculable inputs / expected result or invariant |
| --- | --- |
| 1. Fixed even/odd ties | `1.25 -> 1.2`, `1.75 -> 1.8`, negatives symmetric; `100.5 -> 100`, `101.5 -> 102` at fractional 0 |
| 2. Significant Decimal leading/fractional zeroes | `1,p3 -> 1.00`; `0.125,p5 -> 0.12500`; `0.001953125,p3 -> 0.00195`; integral `12345,p3 -> 12300` |
| 3. Zero occupancy/sign | +/-0 at Significant(3), every notation; exact expected `0.00`, `0.00e0` and Preserve negative counterparts; Trim retains integral zero |
| 4. Rounded negative zero | exact `-0.00390625`, fractional 2 gives `-0.00` or `0.00`; prefix/suffix do not change sign decision |
| 5. Scientific carries/exponent spelling | `9.5` at fractional 0 gives `1e1`; exact `0.125` at fractional 2 gives `1.25e-1`; no e-01/e+1 |
| 6. Engineering ties/carries | `12345,frac2 -> 12.34e3`; `999.5,sig3 -> 1.00e3`; exact `0.125,frac2 -> 125.00e-3`; negative cases |
| 7. Exact percent versus suffix | `0.125,frac1,scale2,% -> 12.5%`; suffix-only remains `0.1%`; literal spaces/prefix remain byte-exact |
| 8. Scale boundaries | Accept -324/324, reject -325/325; exact 1 shifted negative has independently counted decimal zeroes; no intermediate overflow |
| 9. Ordinary binary64 extremes | f64::MAX/MIN, MIN_POSITIVE and first subnormal under all requested notation/scale extremes; direct rational expected coefficients, no premature inf/zero |
| 10. Automatic threshold neighbors | Both signs around 0.01, 1, 100, 10000, 1e6, 1e12; selected pre-round branch and exact division operation order preserved |
| 11. Default grammar punctuation | Negative scientific exponent, retained scientific fractional zeroes, compact suffix, fixed trim, `1000k` threshold carry and complete Forest literal table |
| 12. Exact/adaptive sets | Raw `[0.125,0.1171875]`: Decimal frac2 collision `0.12`; adaptive 2..3 resolves at 3 to `0.125`/`0.117`; duplicate raw values need no increase |
| 13. Significant adaptive constraints | Adjacent representable ordinary values, signed-zero equality, singleton/empty sets; exhaustion is explicit, not a notation switch |
| 14. Forest mathematical result | exponents 0.5/-0.5/0/-0; fractional 2 yields `3.16`/`0.32`/`1.00`/`1.00`; scaled 0.5 with % gives `316.23%` |
| 15. Forest certification/refusal | Independently generated hard boundary/refinement vectors; exact quota and one-unit-short; fatal refusal cannot become another tick source |
| 16. Forest scope and identity | Integral seed, adaptive and original-domain midpoint beyond clamp; huge integral scientific power versus ordinary-output early refusal; exponent next to 0 remains distinct even when sig17 labels collide |
| 17. Exact-source precedence | For numeric x/y, both formatter/interval/exact setter orders; unused suffix glyph unavailable in test font must not participate when exact ticks win |
| 18. All twenty ordinary consumers | Table-driven typed sources covering actual orientations, Histogram sample/supplied and sampled/supplied distribution callers; raw projection/source counts unchanged |
| 19. Five specialized plus TextPanel | Heatmap guide/value independent; Matrix cell default parity/no global binding; Marginal central versus strip ticks; Beeswarm x versus color; Waterfall x versus totals; numeric-looking TextPanel strings untouched |
| 20. Real categorical rejection | Trajectory Category x format/interval fails before discovery; numeric Trajectory succeeds; Line/LinePath numeric restriction, no invented categorical Scatter fixture |
| 21. Visible value membership | Heatmap null/active/triangular/merged rows; Coefficient unavailable/interval restriction and outside-window label behavior; no crowding-based membership |
| 22. Waterfall windows | Both/one/no eligible totals, closed x boundary, y row exclusion, equal numeric totals with different names, fixed/free facets and parent-union eligibility; suppressed total does not force adaptive precision |
| 23. Continuous Shared | Compatible final union reformats once, constant and diverging center cases, exact-domain containment; mutate each policy field at same domain and reject before fonts |
| 24. Font boundary faces | Actual concatenated labels and suffix starting U+0301; individual scalar selections use candidate A for the base and C for the mark, while later B covers the whole unit; retain B's whole-unit preference, allowed scalar fallback when no whole face exists, and fatal missing coverage before measurement |
| 25. Twenty-six dependency classes | Vary every class in the identity table with other dependencies equal; upstream same-domain/same-width caches miss while equal final text can reuse lower glyph analysis; test early comparison refusal |
| 26. Width/feedback/output closure | Automatic, exact-width and exact-pair roots; cached/uncached parity, current-domain label regeneration, one chosen root width, exact frozen preserve-text/outlined-SVG/PNG agreement and refusal without output |

Groups are contract tests, not one test per private setter assignment. Negative
boundaries include precision 18/significant 0, reversed adaptive ranges, empty/
control affixes, invalid actual-axis instructions, exact source containment,
nonfinite data/exponents at the consuming boundary, arithmetic overflow and every
admitted scratch/string/comparison boundary. Tests must demonstrate no mutation or
later processing after the refused guard, not just assert that some error occurs.

### Focused Existing Neighbors

| Change owner | Existing focused evidence to retain/extend |
| --- | --- |
| Exact/value conversion | `src/spec/tests.rs` numeric defaults, selective validation, counted census extremes; `plots/interval_estimate/extraction.rs::semantics_and_label_strings_obey_exact_limits_before_fonts` |
| Axis grammar/source order | `layout/panel/sources/tests.rs::generated_labels_are_unique_and_covered_by_the_closed_census`, `numeric_formatting_uses_nearest_even_and_declared_thresholds`, exact-source/interval cases |
| Fonts/census | `layout/text/requirements.rs` automatic/exact alphabet and contrast-role tests; `font/canonical.rs::retained_units_prefer_complete_faces_without_changing_scalar_or_primary_selection` plus new generated boundary requirements |
| Continuous guides | `decorations/legend/descriptor.rs` unique-order/signed-zero and close-endpoint uniqueness tests; `tests/heatmap/suite.rs::local_and_shared_guides_publish_the_same_distinct_close_numeric_labels` |
| Heatmap values/fit | retained census/refusal tests in `plots/heatmap/tests.rs`; integration suffix and strict cell-fit tests in `tests/heatmap/suite.rs` |
| Estimate presence/fit | `tests/estimates/model_displays.rs::coefficient_fixed_value_labels_are_point_only_and_default_is_inert`; estimate attached-label pitch and expansion tests |
| Totals/parents | `plots/waterfall/extraction.rs` topology/typed shelf/reserve cases; `tests/categorical/waterfall.rs::waterfall_full_svg_fingerprints` with numeric default-byte attribution |
| Composed identity | `builder/facets/interval_tests.rs` copied/composed source identity; `decorations/plan/children.rs` cache tests; `canvas/sizing.rs` measured-height dependencies; kernel changed-feedback/cycle tests |
| Text-cache boundary | `theme/resolved.rs::root_text_queries_share_payloads_but_not_profiles_or_constraints` and indexed matching/fingerprint tests |
| Public boundaries/replay | typed compile-pass/fail fixtures and `tests/rendering/public_contracts.rs`; representative affected visual snapshots and frozen-output integration |

Run the narrowest affected filtered unit tests and representative integrations
for each code leaf, plus formatting and strict Clippy. Compiler validation is
`cargo clippy --locked --offline --all-targets -- -D warnings` (or `just lint`),
never `cargo check`. Do not use a full suite for routine regression discovery.
The execution plan owns the required integrated broad-phase closure gate; this
documentation adds no per-leaf full-suite gate and reports no checks as passed.
Intentional SVG snapshots belong to the source behavior change after inspection,
not an omnibus closure repin.

## Bounded Migration And Design Closure

This architectural contract is enduring. The execution plan owns temporary leaf
IDs/status, sequencing and commits. Use these independently reviewable work units
without accumulating unfinished owners into a repository-wide closure patch:

| Work unit | Required completed responsibility before the next consumer/public cutover |
| --- | --- |
| Documentation closure | This full consumer/default/transport design plus separate numeric-certification proof/oracle contract; public signatures stay closed |
| Post-foundation source refresh | After theme/font/scale/spacing and sizing/tick implementation, map changed fields, clone sites, exact reconstruction, role access, caches and tests to this contract; record actual source/paint/size baseline |
| Exact ordinary decimal core | Admitted rational conversion/scaling/rounding and all notation/zero cases with independent evidence |
| Certified Forest core | Separate proof gate, exact exponent reduction/symbolic powers, interval-to-decimal certification and independent Rust vectors; no platform fallback |
| Unified private profile storage | Move old public-name storage onto the one engine during preparation; retire default/value/guide/total conversion bodies rather than keep adapters with old arithmetic |
| Axis consumer integration | All twenty ordinary routes plus applicable specialized/private Matrix axes, exact/category precedence, terminal/natural intrinsics, complete alphabets and units |
| Guide consumer integration | Local and final Shared policies/values, compatible descriptor transport, pre-font suppressed-child handling, final fills/contrast roles |
| Value/total consumer integration | Complete per-panel scientific sets, Waterfall eligibility/potential text, source strings, row/cell/shelf minima and typed fit behavior |
| Composition identity integration | All explicit spec/Plot clone and erasure sites, scope copies, signatures, Grid child/height inputs, font query boundary, feedback and frozen replay |
| Atomic public replacement | Export NumericFormat/discriminants and fully consumed attachments; migrate every repository NumericLabelFormat caller and remove that type/old methods/display implementation; update current contracts/examples in the same owning cutover |
| Evidence closure | Audit the two consumer matrices, seven policy fields, twenty-six dependency classes and concrete fixtures; inspect default attribution and affected snapshots; required broad execution gate runs once at its owner |

The completed source refresh is actual-input confirmation after other approved
implementation, not remaining product design. In particular, spec94's getter/with_field signatures,
numeric limits, raw-value Forest choice, actual capability sets, lack of Matrix
global binding and default profile ownership are not still unresolved.

Refresh must account for the approved common Heatmap/Beeswarm sequential gradient,
typed FontStack, complete contrast/annotation roles, relative metrics, removed
pre-render dimensions and new root requests. Those changes can move source lines,
alter selected font/paint/geometry dependencies, and intentionally change unrelated
default output. Numeric default conversion evidence must identify that attribution,
not falsely promise the full `e6febe3` scene bytes after every T6 migration.

Before public cutover, a preparatory implementation may retain the current public
name over the replacement private representation. At cutover there is one current
API: remove NumericLabelFormat and its precision/suffix spelling, compile-fail
example, root export and repository uses; no alias/deprecation/version selector.
Private format storage must not become dormant public scope. Unsupported surfaces
have no binding or a pre-font actual-axis rejection, never an ignored format.

The public consumer replacement is complete; integrated evidence closure belongs
to the execution plan. The separate proof and executed core certificates own numerical
certification. This document introduces no unresolved numeric product choice or
authorization to widen scope. Documentation design closure does not mark an
implementation leaf complete.
