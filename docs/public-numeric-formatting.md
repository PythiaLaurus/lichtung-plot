# Scientific Numeric Formatting

`NumericFormat`, `NumericNotation`, `NumericPrecision`, `NegativeZero` and
`TrailingZeros` are available from the crate root. The format is an immutable,
validated value; its implementation modules remain private.

```rust
use lichtung_plot::{NumericFormat, NumericNotation, NumericPrecision};

let percentage = NumericFormat::new(
    NumericNotation::Decimal,
    NumericPrecision::Fractional(1),
)?
.with_decimal_scale(2)?
.with_suffix("%")?;
assert_eq!(percentage.decimal_scale(), 2);
assert_eq!(percentage.suffix(), Some("%"));
# Ok::<(), lichtung_plot::Error>(())
```

This displays raw `0.125` as `12.5%`. A `%` suffix without scale 2 merely appends
the literal percent sign. Scientific values, coordinates and colors are unchanged.

## Construction And Inspection

The seven readonly getters are `notation`, `precision`, `decimal_scale`, `prefix`,
`suffix`, `negative_zero` and `trailing_zeros`. Their corresponding consuming
`with_field` edits replace exactly one field and validate the complete result.
`new` accepts notation and precision, supplying scale zero, absent affixes,
Preserve and Keep. `Default` is Decimal/Fractional(2) with those same fields.

Fractional precision is 0..=17; significant precision is 1..=17. Adaptive variants
carry inclusive `min`/`max` within the corresponding range, with `min<=max`.
Decimal scale is -324..=324. Prefixes/suffixes are nonempty and control-free;
spaces and Unicode combining marks retain their exact bytes. Reconstruct without
an affix to omit it. There is no empty-string reset, callback or format string.

Rounding is nearest with ties to even after exact decimal scaling. Scientific
and Engineering carry normalization does not round a second time. Normalize
removes the sign from actual or rounded zero; Trim removes fractional zeros and
an empty point, never integral place-value zeros.

## Actual Consumers

| Consuming attachment | Owner / applicability |
| --- | --- |
| `x_tick_format(format)` | Existing NumericX families, generated numeric x source |
| `y_tick_format(format)` | Existing NumericY families, generated numeric y source |
| `color_label_format(format)` | Heatmap/Beeswarm continuous guide numbers |
| `HeatmapAnnotations::Values(format)` | Finite active prepared panel cells |
| `estimate_labels(format)` | Existing point-only Coefficient labels |
| `total_format(format)` | Waterfall base/terminal total numbers |

Every attachment replaces its own complete value independently of themes,
intervals, limits, titles and total names. Actual categorical axes reject before
fonts even when their family supports numeric instances. Exact ticks override
generated interval/format requirements in either setter order. Any and Matrix gain
no numeric-format extension. Caller tick/category/significance/annotation strings
and Waterfall names remain verbatim.

Adaptive precision resolves one complete semantic set uniformly, starting at
`min`. It is independent of glyph width and fit. Coefficient membership includes
available estimates outside an explicit x window; Heatmap excludes missing and
inactive cells; Waterfall includes only totals in the closed effective x/y window.
Required sets reject adaptive exhaustion. Fixed value annotations may deliberately
share rounded text; generated axes instead coarsen an indistinguishable source.

Configured Forest Log10 ticks format mathematical `10^(exact(exponent)+scale)`
with certified integer intervals, retaining exponent-space source positions.
For exponent 0.5, Fractional(2) yields `3.16`; adding scale 2 and `%` yields
`316.23%`. No rounded physical binary64 power is substituted.

Shared continuous guides compare complete effective policies before fonts, merge
compatible scientific domains, then resolve one final set. Canvas range groups
permit independently formatted panel axes. Complete policies, original members,
common precision and actual strings are retained for scoped layout/cache/feedback
and frozen replay, including potential Waterfall window outcomes.

Run `cargo run --locked --offline --example 05_numeric_formats` for a composed
axes/Heatmap/estimate/Waterfall/Forest example, written to the system temporary
directory. Earlier evidence records the implementation checkpoints in
`configured-numeric-axes.md`, `coordinated-numeric-guides.md`,
`prepared-numeric-annotations.md` and `composed-numeric-identities.md`.

## Atomic Replacement And Verification

The former `NumericLabelFormat`, its precision/suffix methods and the temporary
wrapper/test constructor are removed. Every repository caller uses this one
public value and `with_field` vocabulary. Obsolete fixed-only scalar support and
private-cutover dead-code allowances are removed. The coefficient/quantum
certificate remains available to independent arithmetic tests without retaining
its otherwise-unused rounded integer result in production after text emission.

Locked/offline focused checks passed: fourteen numeric/core/oracle tests,
twenty-two source/font census tests, the numeric recorded-height cache test and
the adaptive Heatmap intrinsic/fit test: **38 library tests**. Seven external
checks passed: three Heatmap annotation contracts, invalid public construction,
Coefficient labels, diverging complete contrast roles, and custom transparent-cell
SVG/PNG role selection. No unit tests were added or moved; discovery remains
**671 including 17 ignored**.

All **103 doctests** passed, including public construction and compile-fail private
fields/modules, construction-edit vocabulary and sealed x/y/color/estimate/total
capabilities. Strict all-target Clippy, warning-free API documentation and package
verification passed, as did formatting/whitespace checks.

The composed example executed successfully at its requested width 1200 and
content-derived height. Its actual retained text includes `12.5%`, `87.5%`,
`1.01 units`, `1.02 units`, `base $100.00`, `total $110.50` and Forest `1.00e0`.
Source audit finds no old type/display/helper or wrapper-policy calls in Rust
sources, tests or examples. Spec07's independent `10^39.5` integer golden already
matches the executed coefficient ending `...3720`; no golden repin was needed.

Independent mathematical certificates remain in `exact-decimal-conversion.md` and
`certified-log10-conversion.md`; the matching current contracts are spec01/02/06/07.
Integrated all-surface numeric closure remains the following evidence leaf.
