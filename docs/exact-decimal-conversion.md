# Exact Ordinary Decimal Conversion

The private `src/numeric/decimal.rs` primitive converts an exact finite binary64
operand with a resolved precision into Decimal, Scientific or Engineering text.
It implements the ordinary-number mathematical/emission contract in spec06.
The existing public `NumericLabelFormat` and renderer consumers remain current
until their owning migrations; the primitive has no public export or mode selector.
Subsequent Forest certification is recorded in `certified-log10-conversion.md`;
the current wrapper/default conversion and retained-source integration are
recorded in `unified-numeric-presentation.md`. Adaptive owner-set integration
follows the execution plan. The checks below record the ordinary primitive's own
implementation checkpoint.

## Exact Operand And One Rounding

Decode the sign, integer significand `m` and binary shift `b` directly from bits.
Normal values have `m=2^52+fraction` and `b=stored_exponent-1023-52`; subnormals
have `m=fraction`, `b=-1074`. Preserve the sign of zero. Strip common factors of
two from nonzero `m`, adjusting `b` exactly.

- For `b>=0`, the decimal coefficient is the integer `m<<b` and exponent 0.
- For `b<0`, `m*2^b = (m*5^-b)*10^b`. Retain that integer coefficient and exponent.
- Add the admitted decimal scale to the exponent. No floating multiply/divide,
  platform log/power, decimal parse-back or intermediate finite-range conversion
  occurs.

The decimal digit count of the exact coefficient determines the mathematical
decade `k=exponent+digits-1`. Quantization uses:

| Resolved precision | Quantum exponent |
| --- | --- |
| Decimal fractional d | `-d` |
| Scientific fractional d | `k-d` |
| Engineering fractional d | `3*floor(k/3)-d` |
| Significant p in any notation | `k-p+1` |

At a finer quantum, multiply the integer coefficient by the exact power of ten.
At a coarser quantum, divide by the exact power of ten and compare twice the
remainder to the denominator; increment only above half or at half with an odd
quotient. Retain the rounded coefficient, exact quantum and original sign. This
is the only numerical rounding.

## Emission And Boundaries

The common `RoundedDecimal` emitter has an arbitrary-size signed decimal exponent
so later certified/symbolic powers can reuse it. Scientific normalization and
engineering Euclidean multiples of three use the rounded coefficient's decade,
not another rounding. Redundant carry zeroes are discarded only when zero; an
attempt to discard a nonzero coefficient digit is an internal Render error.

Fractional precision retains d places after the point. Significant precision
retains requested fractional occupancy where possible, while integral place-value
zeroes stay mandatory. Zero has one integral digit and p-1 fractional places, or
d fractional places; exponent notations use `e0`. Preserve/Normalize is applied
after rounding, including negative inputs rounded to zero. Trim removes only
fractional zeroes/point, before exponent and literal affixes are appended. No
positive exponent sign, leading exponent zeroes or locale punctuation is emitted.

Finite-binary64 decoding bounds the power-of-five exponent by 1074, positive
binary shift by 1023, and exact coefficient by 767 decimal digits. With admitted
precision and scale the quantization power of ten is at most 1398. These are
mathematical bounds, not new resource quotas. Existing pinned num-bigint owns its
internal integer allocation; no claim of catching all third-party allocator OOM
is made.

The emitter computes symbolic point/zero-run lengths, checked total bytes
including sign/exponent/affixes, and the isize allocation-layout bound before
reserving the output String. Real first-party growth is fallible. Huge positive
or negative ordinary decimal point positions refuse before producing zero runs.
An invalid precision/scale/affix or nonfinite input is Data; representability or
actual output allocation failure is Render. There is no shorter notation,
precision decrease or fallback label after refusal.

`ResolvedFormat` is a borrowed effective policy with an already chosen exact
precision. It does not decide adaptive membership/uniqueness or inspect geometry.
Its temporary module-local non-test dead-code allowance reflects preparation
before consumer migration; it does not introduce a dormant public API.

## Independent Evidence And Test Replacement

Three new arithmetic tests replace exactly the three legacy default/constructor/
selective-field inventory tests in `src/spec/tests.rs`. No tests are merged or
moved into integration suites. The existing counted-census contract remains and
now compares all old accepted precisions to the exact primitive as well.

- The small-rational test uses independent bounded u128 arithmetic on source
  `n/2^b`, every fractional precision 0..=17, scales -3..=3 and both signs:
  **30,492** conversions. Its integer quotient/remainder oracle supplies expected
  text without any floating conversion or production helper.
- Handwritten dyadic/integer cases verify nearest-even ties, significant leading
  and place-value zeroes, single-round scientific/engineering carries, signed and
  rounded zero, literal combining-mark affixes, exact percent versus suffix-only
  semantics and fractional-only trimming.
- Maximum finite, minimum normal and first subnormal at extreme scales use literal
  17-significant expected coefficients. Independent rounding-cell inequalities
  certify those coefficients against binary rational numerators/denominators:
  `(2*K-1)*D < 2*N < (2*K+1)*D`. Production instead decodes through powers of five.
  The observed scientific results include `1.7976931348623157e632`,
  `2.2250738585072014e-632` and `4.9406564584124654e-648`, without overflow/underflow.
- Exact scale endpoints have independently counted zero runs; nonfinite inputs,
  out-of-range scales and unrepresentable signed output positions refuse.
- Existing default-axis evidence now also compares scientific retry conversions
  around both signs of every magnitude threshold and compact operands **after**
  the prescribed binary64 division to independent Rust spelling. Rust conversion
  is a reference for preserved defaults, not the configured decimal-scale oracle.

## Completed Checks

All commands are locked/offline. Focused library checks passed:

| Command filter | Result |
| --- | --- |
| `cargo test --locked --offline --lib numeric::` | 8 passed: 3 new decimal tests plus all 5 existing scientific logarithm tests |
| `cargo test --locked --offline --lib spec::tests::` | 1 passed, retained counted-census/default spelling contract |
| `cargo test --locked --offline --lib numeric_formatting_uses_nearest_even_and_declared_thresholds` | 1 passed, existing source grammar and conversion-neighbor evidence |

Representative current public callers passed: Heatmap invalid precision/suffix,
Heatmap literal suffix/selected borders, and Coefficient point-only fixed values
with inert default (**3 external tests**). This verifies preserved current callers,
not integration of the future policy surfaces.

`just lint` passed strict all-target Clippy; formatting/whitespace checks passed.
Before and after discovery is **671 library tests**, including **17 ignored**.
No full suite, public cutover, Forest inverse-power certification, renderer
default-change claim or output snapshot repin is part of this leaf.
