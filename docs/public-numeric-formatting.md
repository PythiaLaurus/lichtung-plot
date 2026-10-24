# Scientific Numeric Formatting

NumericFormat, NumericNotation, NumericPrecision, NumericRounding, NegativeZero
and TrailingZeros are crate-root values. Fields/implementation are private and validated.

```rust
use lichtung_plot::{NumericFormat, NumericNotation, NumericPrecision};

let percentage = NumericFormat::new(
    NumericNotation::Decimal,
    NumericPrecision::Fractional(1),
)?
.decimal_scale(2)?
.suffix("%")?;
# let _ = percentage;
# Ok::<(), lichtung_plot::Error>(())
```

Raw 0.125 displays `12.5%`. Suffix `%` alone appends literal text without scaling; data,
coordinates and colors are unchanged. Notation is Decimal/Scientific/Engineering,
General (significant precision), or Integer with Decimal/Binary/Octal/HexLower/HexUpper
radix and Fractional(0). General switches notation after rounding; integer values
are rounded mathematically before radix conversion, without a fixed integer limit.
Fractional precision accepts 0..=20, significant 1..=21; adaptive min/max must be
ordered within matching bounds. Decimal scale accepts -324..=324.
These digits come from the exact binary64 value: at Fractional(20), `0.1` displays
`0.10000000000000000555`. Certified powers use the same extended precision bounds.

`multiplier(value)` applies a finite signed factor exactly before decimal scaling
and rounding. No intermediate f64 product overflows, underflows or rounds away
digits. For example `0.1` multiplied by `10` at Scientific/Significant(21) displays
`1.00000000000000005551e0`, not a pre-rounded `1`. Negative factors reverse the
display sign; zero selects zero. Scientific coordinates and source data do not change.

Use `rate(NumericRate::Percent)` or PerMille for an exact factor and the locale's
automatic rate suffix. `locale(NumericLocale::named("fr-FR")?)` selects a pinned
d3-format 3.1.0 definition; `named_ids()` lists all 59 IDs. Custom locales provide
decimal/digit/sign/currency tokens and optional grouping with Cycle/Last/Stop rules.
Nothing reads the host locale or downloads assets.

`grouping(true)`, `sign(NumericSign::Accounting)` and
`symbol(NumericSymbol::Currency)` compose in the same policy. Currency replaces the
automatic rate suffix, not the rate factor. A `NumericField::new(width, fill,
alignment)` supplies minimum Unicode-scalar width, not CSS geometry; multiscalar
digits stay intact. Zero AfterSign fill participates in numeric grouping. Literal
prefix/suffix contents are never localized as if they were numeric digits.

`NumericFormat::from_specifier(".3s")?` selects significant-3 SI with carry promotion;
`,.2f`, `+.2f`, `.1%`, `#08X` and Unicode fill/alignment are also supported. The
grammar is `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`; types are
f/e/g/r/s/%/p/d/b/o/x/X/n or omitted. Omitted type is trimmed General with 12
significant digits; explicit types default to 6, except integer precision is zero.
Malformed/unknown/text-coercing types and out-of-range relevant precision reject.
Typed equivalents have the same policy identity and exact arithmetic.

Use `prefixes(NumericPrefixes::si(selection, promotion)?)` for PerValue/WholeSet/
Fixed SI. Automatic SI saturates at quecto/quetta; fixed selection includes all
standard prefixes. `decimal_compact` supplies K/M/B/T, while `compact` accepts
custom exact thresholds/divisors and literal affixes. KeepThreshold uses unrounded
magnitude; PromoteOnRounding uses original signed rounding cells and freezes one
order-independent WholeSet result. Fixed requires KeepThreshold. `factor` selects
an exact finite divisor, arbitrary-size PowerOfTen exponent or AutomaticDecimal
common factor. These display policies never change scientific positions.

`unit(Some(NumericUnit::new("m", NumericUnitPlacement::PerLabel)))` combines SI
symbols and units (`1 km`). Shared placement removes repeated selected affixes and
adds one measured explanation. It requires a common factor across the complete
set. For example Factor(PowerOfTen("6")) plus an empty Shared unit displays
`1`, `2`, `3` with `×10⁶`. Unit separators are literal and configurable.
`explanation(NumericExplanation::text(Text::new("Adjusted values")))` replaces
generated content in that same slot; `automatic().style(...).layout(...)` controls
appearance without replacing the generated source. Axes, color guides, cell values,
Coefficient labels and Waterfall totals have independent containing slots. Required
explanations contribute to natural sizing and cannot silently disappear at tight
explicit dimensions.

Configure a property with its bare name, such as `precision(policy)` or
`decimal_scale(2)`. Validated consuming edits replace exactly one field; paired
readers are `get_notation()`, `get_precision()`, `get_decimal_scale()`,
`get_prefix()`, `get_suffix()`, `get_negative_zero()` and `get_trailing_zeros()`.
`new` supplies scale 0, no affixes, Preserve/Keep; Default adds
Decimal/Fractional(2).
Affixes are nonempty control-free literal text including spaces/combining marks.
Reconstruct without an affix to omit it; there is no empty-string affix reset.

Rounding defaults to nearest/ties-to-even after exact decimal scaling. Use
`rounding(NumericRounding::Floor)` for rounding toward negative infinity, or select
NearestAway, NearestTowardZero, TowardZero, AwayFromZero or Ceiling. At Fractional(0),
Floor displays -1.5 as `-2`, Ceiling as `-1`; NearestEven displays 2.5 as `2`,
NearestAway as `3`. The same policy applies to certified mathematical powers.
Carry does not round again. Normalize removes actual/rounded negative-zero sign;
Trim removes fractional zeros/empty point, not integral place-value zeros. Engineering always
uses a multiple-of-three exponent, including e0, without inferred SI prefixes.

| Attachment | Actual consumer |
| --- | --- |
| x_tick_format / y_tick_format | Generated numeric axes on supported families |
| color_label_format | Heatmap/Beeswarm continuous guide numbers |
| HeatmapAnnotations::Values | Finite active panel cells |
| Coefficient estimate_labels | Available point-only estimates |
| Waterfall total_format | Eligible base/terminal numeric totals, not names |

Attachments replace their own complete value independently of Theme, generation,
limits and labels. Actual categorical axes reject before fonts. Exact ticks
suppress generated format work in either order. Any/Matrix gain no format
attachment; caller ticks/categories/significance/annotation strings stay verbatim.

Adaptive precision starts at minimum and resolves uniformly for the complete
semantic set, independently of glyph fit. Coefficient includes available
estimates outside an explicit x window; Heatmap excludes missing/inactive cells;
Waterfall uses totals in the effective closed x/y window. Required sets reject
adaptive exhaustion. Fixed value labels may deliberately repeat rounded text;
generated axes instead try another complete source if numbers collide.

Configured Forest Log10 ticks format mathematical `10^(exact(exponent)+scale)` with
certified intervals, not a rounded physical f64 power. Exponent 0.5/Fractional(2)
gives `3.16`; scale 2 plus `%` gives `316.23%`. Shared continuous guides require equal complete
effective policies, then format one final merged-domain set before fonts. Canvas
range groups can retain independently formatted panel axes.

Run `cargo run --locked --offline --example 05_numeric_formats` for existing
axes/Heatmap/Coefficient/Waterfall/Forest usage. Complete rounding/default/set
requirements are in the [numeric specification](../specs/06_NUMERIC_PRESENTATION.md)
and its [power-conversion proof](../specs/07_LOG10_DISPLAY.md).
