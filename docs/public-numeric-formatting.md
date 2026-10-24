# Scientific Numeric Formatting

NumericFormat, NumericNotation, NumericPrecision, NegativeZero and TrailingZeros
are crate-root values. Fields/implementation are private and validated.

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
coordinates and colors are unchanged. Notation is Decimal/Scientific/Engineering.
Fractional precision accepts 0..=20, significant 1..=21; adaptive min/max must be
ordered within matching bounds. Decimal scale accepts -324..=324.
These digits come from the exact binary64 value: at Fractional(20), `0.1` displays
`0.10000000000000000555`. Certified powers use the same extended precision bounds.

Configure a property with its bare name, such as `precision(policy)` or
`decimal_scale(2)`. Validated consuming edits replace exactly one field; paired
readers are `get_notation()`, `get_precision()`, `get_decimal_scale()`,
`get_prefix()`, `get_suffix()`, `get_negative_zero()` and `get_trailing_zeros()`.
`new` supplies scale 0, no affixes, Preserve/Keep; Default adds
Decimal/Fractional(2).
Affixes are nonempty control-free literal text including spaces/combining marks.
Reconstruct without an affix to omit it; no empty-string reset or format string.

Rounding is once nearest/ties-to-even after exact decimal scaling. Carry does not
round again. Normalize removes actual/rounded negative-zero sign; Trim removes
fractional zeros/empty point, not integral place-value zeros. Engineering always
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
