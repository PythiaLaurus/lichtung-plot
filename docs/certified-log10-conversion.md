# Certified Fractional Log10 Conversion

`src/numeric/power/certified.rs` implements the private mathematical-power core
proved in spec07. Its input is the exact retained `Log10Operand`, not a rounded
physical binary64 power. Its result retains the certified integer coefficient and
quantum together with text from the one decimal emitter. This closes private
arithmetic/executable certification; numeric policy storage, consumer/font/cache
transport and the public cutover remain separate execution owners.

## Production Algorithm And Certificate

1. Validate the resolved format/operand scale and reserve the symbolic output and
   integer-conversion buffers. Integral powers and analytically rounded zero use
   the existing symbolic branch.
2. For a fractional operand, retain the already selected physical quantum `t`
   and multiplier `r=n-t`. Start numerical precision at 128 bits.
3. Derive `ln(10)` bounds from the existing outward atanh helper for `ln(2)` and
   `ln(5/4)`. Only this proved constant construction is reused from scientific
   logarithms; the conversion/result is not a binary64 logarithm.
4. Form exact outward bounds on `F*ln(10)/2^(d+4)`, preserving the full dyadic
   fraction. Evaluate exactly p nonconstant positive exponential terms with
   outward divisions and add twice the first omitted upper term.
5. Perform exactly four outward square pairs. Check the proved lower-at-least-1,
   ordered interval and upper-below-11 invariants.
6. Construct the admitted integer decimal multiplier (or denominator 10 for
   `r=-1`), then apply the **same nearest-even integer quantizer** as ordinary
   conversion to both endpoints at that one fixed quantum.
7. Accept only equal coefficient integers. Otherwise checked-double precision,
   discard per-operand scratch and repeat. Carry/notation/zero/affix emission
   occurs only after certification, without another rounding.

The source realizes spec07's atanh/Taylor/tail/four-square recurrences unchanged.
Its one-sided errors have `C0<=8p+32`, `C_(j+1)=22*C_j+1`, hence final width at
most `2*(234256*C0+11155)*2^-p`. A nonintegral rational exponent has irrational
power, so it is separated from every rational decimal half boundary. Interval
convergence therefore gives eventual agreement when actual representation and
storage are available. There is no hard refinement cap, approximate midpoint,
platform transcendental call, notation fallback or coarser source after a fatal
arithmetic/storage refusal.

## Real Storage And Source Census

Before each refinement, checked arithmetic constructs

```text
B(p,r) = max(1084, 2p+9, p+1077, p+1+ceil(log2(p+2)), p+5+4*max(r,0)).
```

This bounds full products before division/shifts. Check the corresponding integer
storage layout and precision/index growth before creating scratch values. Decimal
output planning precedes index narrowing or powers; exponent notations need only
bounded mantissas and symbolic signed exponents. An overflow or actual fallible
reservation failure is fatal, not label collision. No accounting ledger/fees or
unrelated logarithm scratch constant is introduced.

| Source phase | Live state / conversion responsibility |
| --- | --- |
| Reduction/plan | Retained exact exponent/fraction, quantum/multiplier and the output/conversion buffers; no physical power |
| Constant miss | Sequential atanh helper workspaces, keeping only the completed first pair during the second helper; borrow the completed cached pair for the dyadic products |
| Taylor | Unit, two x endpoints, two current terms, two sums and one current denominator/product; no array of terms |
| Squares | Two current sum endpoints and one pair's bounded products/ceiling arithmetic; exactly four pairs |
| Endpoint quantization | Retained bounded mantissa pair, one quantum power/denominator and exact quotient/remainder/parity; drop rejected attempt scratch before advancing precision |
| Emission | One rounded value, a reusable base-10^9 chunk buffer, a nine-byte stack digit array and one final String; no reversed digit string or physical integral-power coefficient |

The source schedule stays within spec07's conservative 32 noncache integer-handle
envelope (cache payloads and test-only traces are separate). The bit bounds and
source ownership are mathematical/representation evidence, not process RSS or a
promise to make num-bigint's internal allocator fallible.

`DecimalBuffer` now owns first-party conversion storage for ordinary and Forest
emission. Before power/refinement work, PowerPlan reserves enough chunks for
`max(1025,4r+5)` coefficient/exponent bits (small symbolic/zero branches need only
the reduction-sized bound). Since `2^29<10^9`, `ceil(bits/29)` bounds chunk count.
Conversion uses exact successive quotient/remainder pairs, then emits leading
and padded nine-digit chunks directly at the chosen decimal positions. Coefficient
and exponent conversions reuse that buffer sequentially. Checked actual digit/
output lengths and fallible growth remain; the dependency-owned `to_radix_be`/
`to_str_radix` buffers are no longer the numeric emission path.

The precision-keyed constant cache has no global or font dependency. Its checked
ordinal corresponds to `128*2^j`; a miss cannot skip an earlier enclosure. Storage
grows fallibly by requested capacities 1,2,4,... and hits borrow the retained exact
integer bounds. Repeated formatting and a fresh cache must give identical
rounded coefficient/quantum and text.

## Independent Oracle And Executed Catalogue

`src/numeric/power/tests/oracle.rs` is a separate Rust integer-root algorithm.
It independently decodes through common denominator `2^1074` and checks both
floor inequalities. Repeated certified square roots enclose `10^(1/2^j)`;
outward products of selected fractional bits enclose the mantissa. No production
reduction, logarithm constant, exponential, quantizer, formatter or acceptance
routine supplies an oracle answer.

Each Newton root starts above the root, obeys the proved iteration guard and
checks `a*a<=Z<(a+1)^2`. Every fractional bit is traversed, including all 1074
bits of the negative least subnormal's fraction. Intermediate-stage cross-checks
extend the denominator index by at most four; the same root error argument holds
for d<=1078, P>=128 (`3d*2^-P<1/2`). Oracle width is checked against `90d` units.

Ten exact fractional inputs at production precisions 128/256/512 compare all five
Taylor/square stages against independent root-product enclosures at 128 extra
bits: **150 stage comparisons**. They check common-scale overlap, declared error
width and retained bound sizes, without assuming production enclosures nest.

The literal coefficient/quantum/text catalogue has **23 vectors**. Every vector
gets an independent half-cell containment certificate, starting at 256 oracle
bits; every executed catalogue certificate completed at 256. Half/quarter cases
also use strict algebraic power inequalities after cancelling the physical
decade. The Rust constants retain exact input bits, scale, notation/precision,
coefficient, quantum and expected bytes. Bounded `--nocapture` records retain
oracle and actual production precision observations.

Representative observed results:

| Exact selected exponent / policy | Certified text or relationship |
| --- | --- |
| 0.5, Decimal/Fractional(2) | `3.16` |
| 0.5, exact scale 2, prefix `rate `, suffix `%` | `rate 316.23%` |
| -0.5, Decimal/Fractional(2) | `0.32` |
| -0.5, Engineering/Fractional(2) | `316.23e-3` |
| +/-0.25, +/-0.75, Decimal/Fractional(2) | `1.78`, `0.56`, `5.62`, `0.18` |
| 0.5, Scientific/Fractional(2), scale +/-324 | `3.16e324`, `3.16e-324` |
| `1-2^-53`, Scientific/Fractional(2) | `1.00e1`, from the original quantum |
| `3-2^-51`, Engineering/Fractional(2) or Significant(3) | `1.00e3`; Trim preserves literal affix zeroes |
| First two positive subnormal exponents, Scientific/Significant(17) | Both `1.0000000000000000e0`, with distinct mathematical operands; no refinement can create public-precision uniqueness |
| Least negative subnormal exponent, same policy | Same positive result despite its full 1074-bit fractional remainder |
| Original-domain midpoint 308.5, Scientific/Fractional(2) | `3.16e308`, beyond integral seed clamp |
| Largest positive/negative fractional binary64 exponent magnitudes | `3.16e4503599627370495`, `3.16e-4503599627370496`, with exact BigInt quantum |
| -0.5, Decimal/Fractional(0) | Positive rounded `0`, certified with denominator 10 |

For exact exponent 39.5, Decimal/Fractional(0), the observed refinement was
**128 -> 256** and the final integer was

```text
3162277660168379331998893544432718533720
```

Its independent strict certificate is `(2J-1)^2 < 4*10^79 < (2J+1)^2`, also
checked by the algebraic oracle. The production-start rejection and successful
precision bound are independently justified in spec07; no renderer snapshot or
engine-generated expected result is used.

## Validation And Boundary Of Completion

Two new proof-oriented library tests replace the two low-value feedback getter/
field-update inventories. Actual child dependency/replay and real eight-round
parent allocation tests remain and pass. No existing test is merged, moved to
integration or made ignored. Library discovery remains **671, including 17 ignored**.

Focused checks passed:

- `numeric::`: **12** tests, including all ordinary conversion and existing MPFR
  scientific-logarithm evidence as well as exact/symbolic/certified powers.
- Retained counted numeric census, default source grammar, actual child
  dependency/replay and real parent allocation: **4** more library tests.
- Existing public Forest default power grammar/positive-data validation and
  narrow adaptive-expression rendering: **2** external tests.
- Strict all-target `just lint`, formatting and whitespace checks.

This is **16 distinct focused library tests and 2 external tests**, not a full
suite. Private numerical certification is complete. Effective policy/default
profile unification, adaptive owner sets, fonts/affix boundary selection,
composition keys, configured consumer refusal propagation and frozen SVG/PNG
publication remain their scheduled integration gates; this evidence does not
claim a current public NumericFormat API or configured-power rendering.
