# Axes and numeric presentation

[简体中文](../zh-CN/axes.html)

Keep three decisions separate: scientific coordinates/domain, tick **positions**
(`TickGeneration`) and numeric **spelling** (`NumericFormat`). Limits do not select
a formatter; suffixes do not change data. Exact `x_ticks`/`y_ticks` are indivisible
and authoritative in either setter order, including empty labels that keep ticks.
Generated caps never truncate exact ticks. Actual categorical axes reject numeric
instructions; Any/Matrix and the special Heatmap axes do not acquire generic numeric setters.

## Position sources

| `TickGeneration` constructor | Meaning |
| --- | --- |
| `automatic()` | Reset the complete choice to the measured default |
| `target(n)` | Approximately n positions; nice-step deviation/coarsening is allowed |
| `maximum(n)` | Bound the complete generated in-domain list, including endpoints |
| `additive(step)` | First-tried positive linear spacing |
| `log10_decades(step)` | First-tried integral Forest Log10 exponent spacing |
| `exact_count(n, space)` | Exactly n endpoint-including affine positions; zero is empty |
| `interval(step, origin, space, requirement)` | Origin-aligned fixed/preferred lattice |
| `multiplicative(anchor, ratio, requirement)` | Anchored geometric lattice in either direction |
| `function(callback)` | Fallible positions from the final actual view |

Counts use `usize`; there is no universal 10,000-position ceiling. Target/Maximum
must be positive. `maximum_count(Some(n))` adds one positive caller cap without
replacing the source; `None` clears it. A cap cannot be smaller than ExactCount.
Counts and parameters have read-only inspection without exposing implementation variants.

ExactCount(1) defaults to Lower; `.singleton(...)` selects Lower/Midpoint/Upper/
Position. For n≥2 endpoints remain fixed. On [0,10], count 4 requests
[0,10/3,20/3,10], with each affine position independently rounded once. It cannot
coarsen when labels fail to fit. `TickSpace::Data` uses raw units; Scale uses
scientific scale coordinates. On Log10 [0.1,10], three Scale positions correspond
to exponents [-1,0,1], unlike the Data midpoint 5.05.

For `interval(20,5,Data,Fixed)`, [5,95] contains [5,25,45,65,85]. Fixed keeps the
complete source or fails a real cap/fit/arithmetic constraint; a valid empty list
stays empty. Preferred may try complete coarser sources and its documented final
candidate. Each origin+k×step uses exact integer arithmetic and one binary64
rounding, including endpoint rounding cells, not epsilon membership or f64 indices.

Multiplicative anchors/ratios are finite positive, with ratio≠1. Ratios below one
retain their direction, not a rounded reciprocal. Each member uses the original
anchor and exact signed index, not a previous rounded member. Log10 operands stay
mathematical even beyond physical f64 powers; callbacks see `Multiplicative`
operands with canonical signed decimal indices. A negative linear view or gap may
be empty; a view containing zero and positive values has an infinite near-zero
tail and refuses. Preferred uses anchored stride sublattices, then an original
middle member, never an invented off-lattice midpoint.

```rust
{{#include ../../../crates/lichtung-plot/examples/07_tick_generation.rs:example}}
```

## Actual-view callbacks and categorical labels

TickContext exposes axis, final domain/provenance, raw owner and actual CSS span.
Intrinsic preferences do not invoke the locator. Return Data or explicitly typed
Log10Exponents; raw limits exist only when genuinely retained. Normal view handling
retains unsorted/repeated in-window occurrences in source order, with original
indices. Equal coincident labels may share cells; incompatible painted text is a
layout nonfit, not a new uniqueness restriction. Empty output retains marks/frame/
title. There is no repair, fallback locator or implicit smaller-count retry.

```rust
{{#include ../../../crates/lichtung-plot/examples/10_tick_functions.rs:example}}
```

`CategoryLabelSelection::every(stride)` selects labels without deleting categories,
marks, ticks or grids. Offset must be below stride; First/Last/Both/None endpoint
policies union full-domain endpoints without duplicates. Stride one is All. x/y
choices are independent and use final explicit/value/facet-local category order.
An unlabeled category can still require text in an annotation or guide.

```rust
{{#include ../../../crates/lichtung-plot/examples/09_category_labels.rs:example}}
```

## Numeric policies

NumericFormat supports Decimal/Scientific/Engineering, General significant digits
and Integer Decimal/Binary/Octal/HexLower/HexUpper. Fractional precision is 0..=20,
significant 1..=21; adaptive bounds must be ordered. Decimal scale is -324..=324.
Formatting retains binary64's exact value: fractional-20 `0.1` becomes
`0.10000000000000000555`. A percent suffix alone appends `%`; decimal scaling or
`NumericRate::Percent` explicitly supplies the factor.

`multiplier` applies a finite signed factor exactly before decimal scaling/rounding,
without an intermediate f64 product. Choose NearestEven (default), NearestAway,
NearestTowardZero, TowardZero, AwayFromZero, Floor or Ceiling. NegativeZero and
TrailingZeros are independent; trimming fractional zeros does not trim integer zeros.
Engineering exponents are multiples of three, including e0.

Pinned d3-format 3.1.0 locales include 59 named IDs and complete custom token/
grouping definitions. No host locale is inferred. Grouping, accounting signs,
currency and rate suffixes compose explicitly; currency replaces the rate suffix,
not its factor. `NumericField` width counts Unicode scalars, not CSS pixels.
Literal nonempty prefix/suffix text is not localized as numeric digits.

Specifier grammar is `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`:
types f/e/g/r/s/%/p/d/b/o/x/X/n or omitted. Examples include `.3s`, `,.2f`, `.1%`,
`#08X` and Unicode fill. Omitted type uses trimmed General/12 significant digits;
explicit types default to 6 except integer precision zero. Invalid combinations fail.

SI supports PerValue/WholeSet/Fixed selection and rounding carry promotion, saturating
at quecto/quetta. Decimal compact and custom exact thresholds/divisors are separate.
KeepThreshold uses unrounded magnitude; PromoteOnRounding freezes an order-independent
whole-set choice. Fixed requires KeepThreshold. Factors may be finite exact divisors,
arbitrary-size powers of ten or AutomaticDecimal. PerLabel/Shared units and one
measured explanation slot avoid redundant factors; required explanations cannot disappear.

`value_function` replaces labels individually; `set_function` returns NumericLabels
for the complete ordered semantic set and its common explanation. Lazy builtin
helpers preserve adaptive precision, locale and factors without recursive callback
invocation. Final returned text is not rounded/localized again. Contexts distinguish
intrinsic probes and actual sets, raw identities, owners and domains. Clones share
callback identity; independently constructed callbacks are not Shared-compatible
merely because output text matches. `builtin()` removes callbacks without resetting options.

Callbacks run serially once per reached context per terminal render, independently
of optional layout reuse. Candidates may create distinct sets. Exact ticks and
empty owners suppress callbacks; exceptions/cardinality failures propagate. Frozen
Output publications never invoke callbacks again.

Consumers are generated numeric axes, Heatmap/Beeswarm color guides, finite active
Heatmap cells, available point-only Coefficient labels and in-window Waterfall
totals. Adaptive precision is uniform over the complete semantic set, not chosen
per label to fit. Generated axes may try a different full source on text collision;
fixed value labels may repeat. Forest Log10 formats certified mathematical powers,
not rounded physical f64 powers. Shared guides require equal complete policies and
format their merged domain; shared ranges alone do not force identical ticks.

```rust
{{#include ../../../crates/lichtung-plot/examples/05_numeric_formats.rs:example}}
```

See the [numeric gallery case](gallery-numeric-policies.md) and
[NumericFormat API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.NumericFormat.html).
