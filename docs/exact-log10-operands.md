# Exact Log10 Operands And Symbolic Powers

The private `src/numeric/power.rs` implementation retains the exact mathematical
operand `10^(exact_binary64(e)+s)`. It closes spec07's exponent reduction,
integral symbolic branch and early output planning. Fractional mantissa
certification, independent root-oracle vectors and public/consumer integration
remain pending; current Forest rendering still uses its existing default grammar.

## Exact Reduction And Context

`Log10Operand` stores normalized selected exponent bits, decimal scale, a signed
arbitrary-size integer decade `n`, complete unsigned fraction `F`, and the bounded
denominator bit index `d`. Decode the binary64 significand/shift directly. Strip
common powers of two, then add the scale as an exact signed integer numerator:

```text
exact(e) + s = (A+s*2^d)/2^d = n + F/2^d,   0 <= F < 2^d.
```

Correct negative truncating remainders by decrementing the quotient and adding
the denominator. Neither the exponent nor its integer part is narrowed to a
machine index or scientific seed clamp. Both zero signs normalize to the same
positive-power operand. The denominator is temporary; only `F,d` survive.

In particular the least negative subnormal retains `n=-1`, `d=1074` and
`F=2^1074-1`, not an approximately rounded fraction 1. Exact exponent `2^60` plus
scale 1 retains `1152921504606846977`, despite binary64 addition losing that unit.
Finite input and scale validation precede these operations. Plan construction
rejects a different format scale rather than silently applying it twice.

`PowerPlan` retains the chosen exact quantum and multiplier `r=n-quantum`.
Significant precision uses `quantum=n-p+1`; Scientific fractional uses `n-d`;
Engineering fractional uses `3*floor(n/3)-d`; Decimal fractional uses `-d`.
The output buffer is allocation state, not numerical/cache identity.

## Symbolic Result And Output Order

For `F=0`, preserve an exact one with decimal exponent `n`. The common
`RoundedDecimal` emitter places its point/zero runs directly; it never constructs
`10^abs(n)` or the corresponding physical coefficient. The separately retained
quantum still records requested precision. For Decimal fractional precision with
`n+d<0`, the exact positive value is at most one tenth of the quantum and rounds
to positive zero. Preserve cannot add a negative sign from a negative exponent.

For a fractional operand with `r<=-2`, `1<=10^(F/2^d)<10` also proves rounded
Decimal fractional zero; the plan records this without evaluating a power. A
nonintegral operand otherwise cannot be published by the integral primitive:
it explicitly requires the subsequent certified mantissa owner.

Before physical coefficient/refinement work, compute spec07's conservative
numeric-body length as an exact BigInt expression. Decimal includes possible
carry, fractional padding or mandatory leading zeroes. Exponent notations use
the proved 365-byte body capacity bound: at most three integral mantissa digits,
point, seventeen fractional digits, `e`, optional exponent minus and at most
`ceil(1025/3)` exponent digits. This is a reservation bound, not an output cap.
Include complete literal affix bytes, check usize/isize representation, then
fallibly reserve the actual String.

The common emitter now accepts that same empty reserved buffer, verifies the
actual emitted length and reuses its capacity. There is one emission body, not a
separate Forest serializer or a duplicate speculative output allocation.
Unrepresentable lengths fail before giant powers/zero runs. An arbitrarily
negative integral exponent can still produce small Decimal rounded zero;
Significant instead preserves its nonzero value or refuses its leading-zero
length. There is no notation switch, exponent clamp, quota or fee.

## Independent Evidence

The reduction test independently represents each finite input as an integer
multiple of `2^-1074`. Across **96** signed source/scale combinations it verifies
the reduction identity and both floor inequalities, without using production
denominator reduction or signed quotient logic. Cases include both least
subnormals, largest subnormal/minimum normal, negative quarter/half, a near-integer
value, original midpoint 308.5 outside the seed clamp, `2^60` and finite extremes.

Handwritten integral output vectors include:

- `-1`, Engineering/Fractional(2): `100.00e-3`.
- `2^60`, Scientific/Fractional(2), scale 1:
  `1.00e1152921504606846977`.
- `2^60`, Engineering/Fractional(2): `10.00e1152921504606846975`.
- `+/-324`: exact exponent output and independently counted Decimal zero runs.
- `+/-f64::MAX`: symbolic scientific bodies of 314/315 bytes, independently
  determined from integer `(2^1024-2^971)`. Positive Decimal fractional and negative
  Decimal significant refuse unrepresentable lengths; negative Decimal fractional
  produces `0.00` without physical powers.
- Trim/combining-mark affixes leave literal suffix zeroes intact, including
  positive rounded zero from huge negative exponents.

Two new mathematical tests replace a retained-scope helper lookup test and the
duplicate `font_fixture_paths` benchmark smoke. Actual scoped preference behavior
and the stronger retained whole-unit/scalar font-selection contract remain and
were run. Benchmark helpers and ignored release coverage are retained; no test is
merged, moved to another suite or converted to ignored.

## Completed Checks And Current Caller Correction

Focused library checks passed: `numeric::` (10 tests: ordinary decimal, exact
power operands and all existing logarithm evidence), retained counted census,
retained whole-unit font selection and actual categorical preference (**13 total**).
Library discovery stays **671 including 17 ignored**.

The existing external Forest grammar fixture initially assumed three selected
sources at an automatic root's incidental width. Its exact-label neighbor also
required more text capacity than that natural root supplied. These current
grammar/exact-source fixtures now explicitly request 1000px width, retaining all
required labels, scientific validation and exact-source precedence. No renderer
padding, source density or default grammar changed. Three `forest_log_scale_`
neighbors and the narrow adaptive-expression fixture passed (**4 external tests**).

Strict `just lint`, formatting and whitespace checks passed. No full suite,
fractional-power certification, new public format or rendered configured-power
claim is part of this leaf.
