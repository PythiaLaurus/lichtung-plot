# Certified Forest Log10 Display

**Status: approved architectural target; private exact-power implementation and
independent executable numerical certification complete; consumer/public cutover
pending.** This document does not describe a current public numeric-format API
or mark its remaining integration gates complete.

**Accounting simplification:** spec95 supersedes every cumulative work/byte
charging requirement in this design. The pricing schedules in §8 and charge-delta
fixtures in §11.5 are historical design evidence, not implementation requirements.
Keep the exact mathematical bounds, real output/precision/index checks, fallible
storage and cache identity. Do not implement their old ledger, fees or quota
tests. The current execution and verification contracts are §9 and §12 below.

Private exact exponent reduction, symbolic integral powers and early output
reservation are implemented with independent evidence in
[`docs/exact-log10-operands.md`](../docs/exact-log10-operands.md), using the common
ordinary decimal emitter from `docs/exact-decimal-conversion.md`. Fractional
mantissa implementation, shared exact quantization, fallible conversion storage
and independent executable root/algebraic certification are complete in
[`docs/certified-log10-conversion.md`](../docs/certified-log10-conversion.md).
Numeric consumer/public cutover and integrated font/cache/refusal/replay gates
remain pending. The source/evidence records distinguish those remaining gates
from private numerical certification.

## 1. Authority And Evidence Boundary

[Spec93](93_T6_EXECUTION_PLAN.md), especially “Numeric Conversion And Precision”
and “Certified Forest Log10 Operand”, and [spec94](94_T6_FOUNDATION_CONSTRUCTION.md)
§12 own the approved product contract. This document supplies its complete
conversion, proof, resource and independent-oracle design. Their public
`NumericFormat` domains and attachment/precedence rules apply unchanged.

Source inspection at repository revision `e6febe3` establishes that:

* `src/numeric.rs::log10` currently certifies the opposite conversion: the
  nearest-even binary64 logarithm of a binary64 input. Its private `ln_bounds`
  uses outward integer atanh series, beginning at 128 fractional bits and
  doubling. Its `rounded_ratio` converts a rational to binary64, not to a decimal
  display result.
* `src/numeric/tests.rs` contains exact-power/exceptional-input tests, recorded
  MPFR 4.2.1 logarithm vectors and a digest, interval/refinement tests, rational
  binary64 tie tests, scratch/work refusal tests and Forest refusal propagation.
  Those are existing logarithm evidence, **not** inverse-power display evidence.
* At that historical revision, `OperationGenerationResources` owned cumulative
  counters and quotas. Those have since been removed under spec95; current
  operation context retains identity and actual caches only.
* The project already pins `num-bigint = 0.4.8`. No new dependency, network
  access, asset provisioning, Python support or separate tool project is needed.

The pseudocode and proofs below define the mathematical algorithm; the current
private source and executable certificates are mapped by the evidence links
above. Document-level mathematics alone is not a test run or certification, and
no new MPFR result, benchmark or allocation trace is claimed here. The existing
logarithm scratch constant is not a bound for this algorithm.

## 2. Exact Operand And Display Contract

For a selected **finite binary64 exponent** `e` and the validated integer decimal
scale `s` in `-324..=324`, the presentation operand is

```text
E = exact_binary64(e) + s
V = 10^E                         (mathematical real, strictly positive)
```

The user-selected example is `e = 0.5`, Decimal/Fractional(2): `3.16`.
With scale 2 and suffix `%` it is `316.23%`. Scaling is exact exponent addition;
neither `e + f64::from(s)` nor multiplication of a binary64 power is permitted.
`powf`, midpoint approximations and parsing a formatted label into data are not
conversion steps. A rounded logarithm of an observation is not asserted to be an
exact inverse of that observation.

Projection, source addresses, tick ordering and scientific domain feedback remain
exponent-space. Every selected finite exponent is covered, including original
domain midpoint sources outside the integral-seed range. Exact caller tick text
still takes precedence. Omitted-format Forest power grammar is the approved
private profile of the one presentation core, not a second public format mode.

Notation and precision mean:

* Decimal fractional `d`: round the **whole physical value** to quantum `10^-d`.
* Scientific fractional `d`: choose the unrounded decade `n = floor(E)`, then
  round to quantum `10^(n-d)`; normalize a carry afterward.
* Engineering fractional `d`: choose `g = 3 floor(n/3)`, then round to quantum
  `10^(g-d)`; normalize a carry afterward.
* Significant `a`: in every notation round the whole value to quantum
  `10^(n-a+1)`. This is not a count of engineering mantissa fractional places.

Here `d` is `0..=17` and `a` is `1..=17`. An adaptive owner selects one of these
precisions uniformly for its entire semantic set, as specified in spec93.
Numerical interval refinement does not alter that public display precision.

Nearest/ties-to-even is applied **once** at the selected quantum. Carry
normalization, place-value zeros, affixes and trailing-zero trimming are exact
serialization of that same rounded value. Forest `V` is positive even for a
negative exponent or exponent `-0.0`; a rounded zero here is positive. The
negative-zero policy therefore cannot add a minus sign to a Forest power.

## 3. Exact Reduction Without Exponent Narrowing

Decode the binary64 bits, not their decimal spelling. For a normal number the
unsigned significand is `2^52 + fraction_bits` and the binary shift is
`stored_exponent - 1023 - 52`; for a subnormal it is `fraction_bits` with shift
`-1074`. Apply the sign, strip common powers of two, and represent

```text
exact(e) = A / 2^d,              0 <= d <= 1074
E        = (A + (s << d)) / 2^d
```

`A` is signed arbitrary-size integer storage. Zero has denominator 1 and no
negative sign. A nonnegative binary shift produces an integral `A`, with no
conversion of its magnitude to `i64`, `i128` or a machine indexing type.

Perform Euclidean division by `D = 2^d`:

```text
A + s*D = n*D + F,              0 <= F < D
f = F/D,                        E = n + f
```

For a negative numerator, truncating signed division is insufficient. If its
truncating remainder is negative, subtract one from the quotient and add `D`
to the remainder. Equivalently, for a negative nonintegral magnitude `qD+r`,
use `n = -q-1` and `F = D-r`. Retain `n` as `BigInt` throughout.
After checking the reduction identity, drop the division quotient/denominator
scratch; the retained exact fractional operand is `F` and the bounded denominator
bit index `d`. Reconstruct a power-of-two denominator only in its guarded block.

Examples that fix the floor rule:

| Exact exponent and scale | `n` | `f` |
| --- | ---: | --- |
| `-0.5` | `-1` | `1/2` |
| `-0.25` | `-1` | `3/4` |
| least positive subnormal `2^-1074` | `0` | `1/2^1074` |
| least negative subnormal `-2^-1074` | `-1` | `(2^1074-1)/2^1074` |
| `2^60`, scale 1 (other rows scale 0) | `1152921504606846977` | `0` |

The unrounded mantissa is `M = 10^f`, so `V = M*10^n` and `1 <= M < 10`.
This decade is known exactly without inspecting approximate interval endpoints.
For engineering use Euclidean division of `n` by 3, including for negative `n`;
`h = n-g` is exactly one of `0,1,2` and may then be narrowed to a small integer.

### Integral Symbolic Branch

When `F = 0`, retain `V = 10^n` symbolically. Do not run logarithm/exponential
series or construct `10^abs(n)`.

For significant precision, the coefficient is exactly `10^(a-1)` and the quantum
exponent is `n-a+1`. For scientific fractional precision it is `10^d` at quantum
`n-d`. For engineering fractional precision it is `10^(h+d)` at quantum `g-d`.
These coefficient powers have exponent at most 19, irrespective of `n`.

Decimal fractional precision gives an exact integer coefficient `10^(n+d)` when
`n+d >= 0`, retained as a one followed by a symbolic zero run until output
admission. When `n+d < 0`, its nearest integer is zero: `10^(n+d) <= 1/10 < 1/2`.
Decimal significant precision does not erase tiny nonzero powers; their leading
zero run is admitted before materialization. Symbolic scientific/engineering
exponents use an admitted conversion of `n` or `g`, not an exponent clamp.

## 4. Exact Decimal Quantization And Carry

For nonintegral `E`, choose the following quantum exponent `t` and multiplier
power `r = n-t` **before** numerical refinement:

| Policy | `t` | `r` |
| --- | --- | --- |
| Decimal fractional `d` | `-d` | `n+d` |
| Scientific fractional `d` | `n-d` | `d` |
| Engineering fractional `d` | `g-d` | `h+d` |
| Any significant `a` | `n-a+1` | `a-1` |

The desired coefficient is `K = RN_even(M*10^r)` and the rounded physical value
is `R = K*10^t`. Apart from Decimal fractional precision, `0 <= r <= 19`.
For Decimal fractional precision, if `r <= -2`, `M < 10` proves
`M*10^r < 1/10`; return a symbolic rounded zero immediately. The only negative
power requiring intervals is `r = -1`, with denominator 10.

Given an endpoint `Y/S` with `S = 2^p`, form exact integers

```text
r >= 0:  numerator = Y * 10^r,  denominator = S
r == -1: numerator = Y,         denominator = 10*S
q = numerator / denominator
z = numerator % denominator
K = q + [2*z > denominator
         or (2*z == denominator and q is odd)]
```

Both endpoint results must be the **same integer `K` at the same `t`**. This
comparison happens before trimming, affixes or text conversion. A common
approximate mantissa string is not a certificate. Integer nearest-even is
monotone, including at half-integer boundaries: its selected value at a tie is
between its left and right limits. Equal endpoint results therefore certify every
intervening value, even when an endpoint itself is a rational tie.

Do not choose a decade separately for each endpoint. For example an upper
endpoint can exceed 10 while the exact `M` is below 10; independently formatting
it could select a different quantum and cause spurious disagreement or a second
rounding. The exact `n,g,t,r` above remove this ambiguity. Endpoints need not be
clipped to the known mantissa range for correctness.

### Post-Rounding Serialization

Determine the decade of `R` from the admitted coefficient digits and `t`, or
symbolically in the integral branch. Scientific output has one integral mantissa
digit, lowercase `e`, no positive exponent sign and no leading exponent zeros.
Engineering uses the resulting decade's Euclidean multiple of three, including
`e0`; a nonzero mantissa is in `[1,1000)`.

Concretely, if `K>0` has `c` decimal digits, the rounded physical decade is
`n_R = t+c-1`, an exact integer sum. Scientific mantissa is `K*10^(t-n_R)`;
engineering chooses `g_R=3*floor(n_R/3)` and mantissa `K*10^(t-g_R)`.
These are decimal-point positions/zero runs, not new numerical products or a
second nearest-even operation. `K=0` uses the positive Decimal zero grammar;
Forest exponent notations cannot round to zero because their `r>=0` and `M>=1`.

* Significant carry `K = 10^a` is exactly rewritten as coefficient `10^(a-1)`
  and quantum `t+1`; it introduces no new rounding. Scientific keeps `a-1`
  fractional places. Engineering keeps `max(0,a-1-h_after)` fractional places;
  integral place-value zeros are mandatory even when `a < h_after+1`.
* Scientific fractional carry rewrites `10.00…` as `1.00…` with exponent
  incremented by one, retaining `d` fractional places. Engineering fractional
  carry rewrites `1000.00…` as `1.00…` with exponent incremented by three.
  Any inserted fractional zeros are exact because the carried result is an
  integral power of ten.
* Decimal significant output retains the chosen significant places where they
  are fractional. Decimal fractional output retains exactly `d` fractional
  places, including a rounded zero. Integral place-value zeros are not precision
  padding and cannot be trimmed.
* `TrailingZeros::Trim` removes fractional trailing zeros and an empty decimal
  point only, after normalization. Affixes are concatenated literally afterward.

In particular Engineering/Significant(3) can carry to `1.00e3`; engineering
fractional precision is not implemented by first rounding a scientific
mantissa. Sign handling for the shared ordinary-value converter remains its own
exact-rational contract; this positive-power conversion never introduces a
negative endpoint or negative physical result.

## 5. Certified Production Mantissa Algorithm

Fix `p = 128` initially, `S = 2^p`, `u = 1/S`. All divisions below use exact
integers and explicitly stated floor/ceiling. Precision is a checked machine
index **only after its size/work plan is admitted**; the scientific exponent is
never such an index. Recompute at `p,2p,4p,…` until endpoint quantization agrees.

### 5.1 Logarithm Constants And A Closed Error Bound

Use

```text
ln(10) = 3*ln(2) + ln(5/4)
ln(2)   = 2*atanh(1/3)
ln(5/4) = 2*atanh(1/9)
```

The identity follows from `10 = 2^3*(5/4)`; the atanh identity is
`ln((1+z)/(1-z)) = 2 sum(k>=0) z^(2k+1)/(2k+1)`.
For either `z`, let `N_l = floor(p/3)+1`. The target constant helper uses the
same recurrence as existing `ln_bounds`:

```text
Plo = floor(S*z);                 Phi = ceil(S*z)
Qlo = floor(Plo*Plo/S);           Qhi = ceil(Phi*Phi/S)
L = 0; H = 0
for k = 0 .. N_l-1:
    L += floor(Plo/(2*k+1))
    H += ceil(Phi/(2*k+1))
    Plo = floor(Plo*Qlo/S)
    Phi = ceil(Phi*Qhi/S)
return (2*L, 2*H+1)
```

This reuse is an algorithm choice, not evidence that the helper is already
accessible to or sufficient for the new caller. Zero is not a constant input.
The following proof covers both constants and every `p >= 128`.

1. Initial powers have one-sided error at most `u`. Fixed-point squares have
   one-sided error at most `(2/3+u+1)u < 2u`. In real units `Qhi/S < 1/8`,
   since `z^2 <= 1/9` and `2u < 1/72`.
2. If a power's one-sided error is at most `2u`, the next upper error is at most
   `(1/8)*2u + (1/3)*2u + u < 2u`. The lower error is at most
   `(1/9)*2u + (1/3)*2u + u < 2u`. This proves the invariant for every loop
   iteration, including the ceil recurrence after the true term is very small.
3. Division by the positive odd divisor contributes at most one further unit.
   Each summand's one-sided error is at most `3u`; doubling the sum gives at
   most `6*N_l*u`.
4. The omitted positive tail satisfies

   ```text
   2 sum(k>=N_l) z^(2k+1)/(2k+1)
       <= 2*z^(2*N_l+1)/((2*N_l+1)*(1-z*z))
       <= (3/4)*9^-N_l < 2^-p = u.
   ```

   The last inequality uses `9 > 8` and `3*N_l > p`. Thus the added upper
   unit covers the entire tail, and the lower partial sum is a lower bound.

Each returned endpoint differs one-sidedly from its logarithm by at most
`(6*N_l+1)u`. Form the exact constant interval

```text
L10 = 3*L2 + L54
U10 = 3*U2 + U54
D_l = 24*N_l + 4
```

so `L10/S <= ln(10) <= U10/S`, and each endpoint error is at most `D_l*u`.
Its width is at most `2*D_l*u`. This bound tends to zero as `p` doubles.

For a simple range bound, the geometric series gives `ln(2) < 3/4` and
`ln(5/4) < 9/40`, hence `ln(10) < 99/40 < 4`. Also
`D_l*u < 4-99/40` for every `p >= 128`: a linear function of `p` times `2^-p`
is decreasing here, and the inequality already holds at 128.
Consequently `U10 < 4*S`.

### 5.2 Exact Dyadic Product And Small Exponential

For `f = F/2^d`, set

```text
Xlo = floor(F*L10 / 2^(d+4))
Xhi = ceil (F*U10 / 2^(d+4))
x = f*ln(10)/16
A_p = 2*N_l + 2
```

These are integer endpoint bounds for `S*x`, with `0 <= Xlo <= Xhi <= S/4`.
Each one-sided error in real units is at most
`(D_l/16+1)u <= A_p*u`. This multiply uses the **entire** exact dyadic `F`;
there is no 53-bit approximation to `f` after subtracting a negative floor.

Choose exactly `N_e = p` included nonconstant Taylor terms:

```text
Tlo = S; Thi = S; Ylo = S; Yhi = S
for k = 1 .. p:
    Tlo = floor(Tlo*Xlo/(S*k))
    Thi = ceil (Thi*Xhi/(S*k))
    Ylo += Tlo
    Yhi += Thi
Ohi = ceil(Thi*Xhi/(S*(p+1)))
Yhi += 2*Ohi
```

`Ylo/S <= exp(x) <= Yhi/S`. Positivity and monotonicity prove enclosure of
each included term. For the upper endpoint argument `b = Xhi/S <= 1/4`, the
ratio of every term after the first omitted term to its predecessor is at most
`b/(p+2) <= 1/4`. Thus its tail is at most `4/3` times, and in particular at
most **twice**, the first omitted term. Ceil `Ohi` encloses that term.

Here is an explicit accumulated-error bound, rather than an assumption about
the smallness of late terms. Against the exact terms of `exp(Xlo/S)` or
`exp(Xhi/S)`, the recurrence's one-sided error obeys

```text
epsilon_0 = 0
epsilon_k <= (1/(4*k))*epsilon_(k-1) + u <= 4*u/3.
```

Included-sum error is at most `4*p*u/3`. The extra upper omitted-term error
is at most `8*u/3`. Further,

```text
(p+1)! >= 2^p
first_omitted <= (1/4)^(p+1)/(p+1)! <= 2^(-3*p-2)
twice_first_omitted <= 2^(-3*p-1).
```

The factorial inequality follows by multiplying the `p` factors `2..p+1`,
each at least 2. It also proves tail convergence independently of recurrence
rounding. Since `exp(z) <= 1/(1-z) <= 4/3` on this argument range, the mean
value theorem bounds perturbation from `x` by `4*A_p*u/3`. Both one-sided
errors are therefore at most

```text
(4/3)*(A_p+p+2)*u + 2^(-3*p-1)
    <= C0*u,                    C0 = 4*(A_p+p+3) <= 8*p+32.
```

This deliberately conservative integer bound includes parameter, recurrence,
sum and tail errors. It tends to zero and remains valid even if upper late
terms repeatedly ceil to one unit.

### 5.3 Four Outward Squares And Global Convergence

Perform exactly four steps:

```text
Ylo = floor(Ylo*Ylo/S)
Yhi = ceil (Yhi*Yhi/S)
```

After step `j`, these enclose `exp(2^j*x)`. After step 4 they enclose `M`.
The exact stage values are at most 10, not at most 2: a proof that used the
small-argument bound after all four squares would be invalid.

Define `C_(j+1) = 22*C_j+1`. Inductively, every upper stage value is below 11.
Indeed, if endpoint error is `C_j*u`, a square's error is at most
`(11+10)*C_j*u+u <= C_(j+1)*u`; the lower error is no larger. The explicit
final coefficient is

```text
C4 = 22^4*C0 + (22^4-1)/21
   = 234256*C0 + 11155
   <= 234256*(8*p+32) + 11155.
```

At `p=128` this is less than `2^28`; `C4*2^-p` is decreasing for `p >= 128`.
Thus `C_j*u < 1` for all stages and all admitted precisions, proving the
upper-below-11 invariant used in the amplification calculation. The resulting
mantissa enclosure has width at most `2*C4*2^-p`, which tends to zero along
`p = 128*2^j`. No claim of nesting is required; each independently computed
enclosure contains the same real `M` and converges to it.

## 6. Certification And Termination Theorem

If a rational exponent `E = a/b` in lowest terms, `b > 0`, had rational
`10^E = c/d > 0` in lowest terms, taking the prime-2 valuation of
`(c/d)^b = 10^a` would give `b*(v2(c)-v2(d)) = a`. Thus `b` divides `a`;
coprimality forces `b=1`. This argument also covers negative `a`. Hence every
nonintegral rational `E` gives an **irrational** positive `V`.

Every rounding boundary for a fixed decimal quantum is rational, of the form
`(k+1/2)*10^t`. A nonintegral `E` is separated from its two neighboring
boundaries by a positive distance. Multiplication by the fixed exact `10^r`
preserves convergence of the mantissa enclosure, however large a finite `r`
is. Eventually the enclosure lies entirely inside one rounding cell, so both
endpoint coefficients agree. Integral `E` is handled exactly without this
argument. Nearest-even behavior at rational endpoint ties is already explicit.

Therefore the algorithm terminates with the correct rounded result **given
sufficient representable/admitted resources**. This is not a uniform promise
that every input fits a particular operation's remaining ledger. Precision
doubling, counter conversion, byte multiplication, vector capacity or shift
overflow is a fatal arithmetic refusal. Byte/work refusal is likewise fatal.
Neither can be reinterpreted as label collision, axis coarsening, lower display
precision or a best-effort label.

There is no hidden maximum precision, series-term cap or hard-case lookup table.
The checked doubling rule and existing cumulative ledger are the stopping policy.
For any fixed successful computation, only finitely many stages and terms occur;
for an unsuccessful operation, admission fails before its guarded work.

## 7. Output Admission Before Powers Or Zero Runs

Lengths are first computed as exact nonnegative `BigInt` expressions, using only
bounded reduction-sized storage. Only an admitted length is converted to `usize`;
every allocation additionally requires the standard `isize::MAX` byte bound.
Add prefix/suffix UTF-8 byte lengths with checked arithmetic. No quota changes.

The following conservative **numeric-body capacity bounds** include an optional
carry. They may be tightened for the exact integral symbolic branch, where no
rounding carry occurs.

| Decimal policy | Body bound `L` |
| --- | --- |
| Fractional `d`, already proved zero | `1 + (d>0 ? 1+d : 0)` |
| Fractional `d`, otherwise | `max(1,n+2) + (d>0 ? 1+d : 0)` |
| Significant `a`, `n >= a-1` | `max(1,n+2)` |
| Significant `a`, `n < a-1` | `max(1,n+2) + 1 + (a-1-n)` |

For the last row, a carry reduces fractional places by one rather than increasing
them. Trimming can only shorten these bounds. Admit the chosen conservative
capacity, not an uncomputed hope that a long fractional string will trim away.
Refusing an unadmitted upper-bound reservation is deterministic resource policy.

For exponent notations, `r <= 19`; mantissa digits and fractional padding are
bounded independently of `abs(n)`. The displayed exponent's magnitude needs at
most 1025 significant bits (§8), so an integer-to-decimal bound
`digits(b) = max(1,ceil(b/3))` suffices, because `2^3 < 10`. Account also for its
possible minus sign and the literal `e`. Thus even extreme integral binary64
exponents have short symbolic output, without constructing their physical power.
One uniform exponent-notation body bound is 365 bytes: three integral mantissa
digits, decimal point, at most 17 fractional digits, `e`, possible exponent minus
and at most `ceil(1025/3)=342` exponent digits. Use the policy-specific smaller
bound when available; affixes and buffer headers/capacity are separate.

For Decimal fractional precision with `r >= 0`, output admission bounds `n`
and hence `r = n+d`: only then narrow `r` and build a decimal power. For `r=-1`
the denominator is only 10. For the proved-zero branch no huge negative power
is built. Decimal significant output with a huge negative `n` must admit its
leading zeros; it cannot silently become zero or switch to scientific notation.

The accepted coefficient has at most `b_K = 4r+5` bits when `r>=0`, because
`K < 11*10^r+1 < 16*10^r`; for `r=-1`, two bits suffice. These bounds do not
depend on `p`. Thus coefficient digit/chunk reservation is part of the initial
output plan, not something discovered by materializing the coefficient. Exponent
conversion uses at most 1025 bits. The symbolic integral branch reserves only
its small mantissa/exponent conversion and final zero-run output buffers.

Admission order is: guarded representation/planning arithmetic, output-capacity
bytes, then individually guarded power/conversion work and the selected emission
pass. The initial capacity bound is **not** a writing-visit claim: rounding,
carry and trimming can change the actual number of emitted digits. Defer that
claim until the certified result's emission instructions and actual byte/run
lengths are known. A known affix length may be claimed as a complete copy count
when that copy pass is selected, not as hypothetical future work during capacity
planning. Any scan needed to determine digits or trimming admits each actual
examination, including its terminating byte/end position, before it happens.

Reservation credit may cover the later single final buffer allocation; it is
not refunded or silently reused for a second allocation. Conservative capacity
bytes and actual-work visits are separate claims. A symbolic request larger than
`usize`, `isize` or remaining bytes refuses before a giant power, digit vector,
zero string or output buffer exists. Already-small rounded-zero Decimal requests
remain possible for arbitrarily negative integral exponents.

## 8. Derived Integer, Live-Storage And Work Bounds

This section specifies the resource model and the concrete admission mechanism,
not just an asymptotic complexity assertion. It extends the existing numerical
helper's boundary: first-party materializations, buffers and loops are admitted;
opaque `num-bigint` internal limb loops/workspace and allocator behavior are not
promised to be fallible. Admission is not a third-party OOM guarantee.

Work follows [spec06](06_NUMERIC_PRESENTATION.md), “Fallible Distinctness And
Comparison Work”, and spec05's PF4 **single shared `processingVisits` ledger**:
known complete inputs/passes can be admitted in blocks before processing;
data-dependent visits are admitted one actual operation at a time. Complexity
upper bounds for conversion, ordering and the oracle are analysis/guard bounds,
not claims that their unvisited tails were processed. Conservative bit/capacity
reservations remain valid byte admissions and are never refunded. There is no
second work-reservation counter, worst-case visit prepayment or hidden switch
between accounting modes.

### 8.1 Significant-Bit Bounds

The largest finite binary64 magnitude is `(2^53-1)*2^971 < 2^1024`.
Adding a scale of magnitude at most 324 cannot bridge its gap `2^971` to
`2^1024`. In fractional decoding the unshifted numerator has at most 53 bits;
adding `s*2^d` gives at most `max(53,d+9)+1 <= 1084` bits. This also bounds
the reduction's signed numerator after canonicalization; integral decoding
needs at most 1024. The denominator `2^d` needs at most 1075 bits, and
`F < 2^d` needs at most 1074. `n` needs at most 1024 magnitude bits;
`n +/- small_precision`, engineering `g` and a carry are safely bounded by 1025.

For nonzero `r >= 0`, `10^r < 2^(4r)`; use the uniform bound
`b10(r) = 1+4r`, including `r=0`. Define
`ell(p) = ceil(log2(p+2))`, computed by checked integer bit length, not libm.

| First-party value or intermediate | Sufficient significant-bit bound |
| --- | ---: |
| Exponent reduction numerator/remainders | `1084` |
| Constant powers, fixed-point sums and `ln(10)` endpoints | `p+3` |
| Constant square/product plus ceiling numerator | `2p+3` |
| Exact `F * L10` or `F * U10`, plus ceiling numerator | `p+1077` |
| Taylor product or sum | `2p+4` |
| Taylor denominator `S*k`, including `k=p+1` | `p+1+ell(p)` |
| Four-square product plus ceiling numerator (`Y < 11*S`) | `2p+9` |
| Quantization power | `1+4r` |
| Quantization numerator and rounding carry, `r>=0` | `p+5+4r` |
| Quantization denominator/doubled remainder, `r=-1` | `p+5` |
| Symbolic displayed exponents | `1025` |

Thus a single checked per-attempt materialization envelope is

```text
B(p,r) = max(1084, 2p+9, p+1077, p+1+ell(p), p+5+4*max(r,0)).
```

This covers every full product **before** a right shift/division. It is not
enough to price only the smaller result. A moved product subsequently shifted
right retains the product's capacity credit; do not reprice it by its smaller
bit length. The power builder's prefix accumulator and intermediate square
never exceed `10^r`; its literal base is 10 only when `r>0`. Reduced numerator bounds must not be confused
with the possibly 1074-bit fractional remainder for a tiny negative exponent.

### 8.2 Capacity Bounds And Historical Charging Design

The capacity/live-expression bounds remain mathematical evidence. Cumulative
admissions, materialization prices and visit allowances below are superseded by
spec95 and must not be restored in a future implementation.

Use checked arithmetic in `u128` for planning; convert claims to the existing
ledger's types fallibly. Let

```text
H = max(size_of::<BigUint>(), size_of::<BigInt>())
limbs64(B) = ceil(B/64)
C(B) = H + 16*max(4,limbs64(B)+1)
```

`C` is the prescribed first-party integer **capacity allowance**: round to
64-bit limb storage, include a spare limb and doubled growth capacity, a small
allocation floor, and the actual target's integer header. It is deliberately
more conservative than `numeric.rs::admit`'s header-free `16*ceil((2p+128)/64)`.
It is a ledger allowance for materialized integer operands, not a claim about
every private dependency allocation or Rust allocator usable-size. Internal
dependency temporaries/OOM behavior remain outside this boundary; a successful
ledger claim must never be described as a proof of total process RSS or universal
fallible `BigUint` arithmetic.

All first-party `Vec<T>`/`String` allocations have separate bounds, including
`size_of::<Vec<T>>()`/`size_of::<String>()`, and payload capacity, not just length.
Use fallible reserved storage and explicit checked growth. For a planned capacity
`c`, the first-party allocation allowance is header plus `2*c*size_of::<T>()`
(bytes for strings). On an allocation result with larger reported capacity,
claim the excess before populating or retaining it; on refusal, drop the empty
reservation and abort. No infallible collection, unpriced `collect`, automatic
unobserved vector growth or format-driven buffer growth is part of this design.
The pre-allocation claim guards the bounded requested allocation; the allocator's
own rounding is not a pre-observable guarantee. A twofold allowance must not be
mistaken for a language guarantee that `try_reserve_exact` returns exactly `c`.

No array of Taylor terms or logarithm powers is needed. The following sequential
schedule bounds simultaneously live first-party integer handles; counts include
ephemeral expression results, not just named variables:

| Phase | Retained inputs/state | Expression scratch | Live upper bound |
| --- | ---: | ---: | ---: |
| Decode, exact scale and Euclidean reduction | 6 | 6 | 12 |
| Quantum, output length and conversion-buffer planning | 8 | 8 | 16 |
| One constant helper; keep completed first constant while computing second | 16 | 10 | 26 |
| Combine constants and form dyadic `x` | 14 | 8 | 22 |
| Taylor recurrence, sums and omitted term | 14 | 10 | 24 |
| Four outward squares, old endpoints retained until new pair exists | 12 | 6 | 18 |
| Power build and two endpoint quantizations, sequential | 16 | 10 | 26 |
| Coefficient/exponent digit conversion, sequential | 12 | 6 | 18 |

These totals include `n,F`, quantum/engineering exponent context, unit/ceiling,
small coefficient/power and current endpoint state as applicable. Drop constants,
powers and products after their last use; the reduction denominator is not kept
through the series. Up to six general context integers and four block results
are covered by each phase's retained/scratch counts. The noncache simultaneously
significant integer bits are at most `32*B`; a conservative noncache live-storage
allowance is `32*C(B)`, plus explicitly reserved output/conversion buffers.
Retained constant-cache entries add two
integer payloads per entry and their entry/vector headers. One adaptive set
does not create an unbounded second array of interval workspaces: format one
operand at a time, retain admitted label records, then compare the complete set.

**Cumulative bytes are not peak bytes.** Before an arithmetic block, charge
`m*C(B)` for its planned `m` materializations even if earlier temporaries have
been dropped or capacity could be reused. A moved object is one materialization,
a clone is another. If a block uses a smaller proved bound it may price that
bound, but it cannot borrow uncharged capacity from a later block. Integer
headers already contained in a cache entry header need not be counted twice;
payload growth always is counted. Straight-line expression construction is
split as needed so its expression scratch respects the table.

The following conservative block allowances are an implementable schedule.
They bound expression-result counts, including ceiling additions, by resolving
each expression left-to-right; they do not assert that all results are live:

| Guarded block | Integer materialization allowance | Visit admission |
| --- | ---: | ---: |
| Decode/reduce/construct context | `24*C(1084)` | 1, plus any actual scan iterations |
| Quantum/output/conversion-buffer plan | `32*C(1084)` | 1 |
| Refinement setup, unit and ceiling | `4*C(B)` | 1 per attempted precision |
| One atanh setup | `12*C(B)` | 1 |
| One atanh iteration (two divides, two sums, two power recurrences) | `16*C(B)` | 1 |
| One atanh final doubling/tail | `4*C(B)` | 1 |
| Combine `ln(10)` pair | `6*C(B)` | 1 |
| Dyadic `x` endpoints, including ceiling denominator | `8*C(B)` | 1 |
| One Taylor iteration (two products/divides and two sums) | `12*C(B)` | 1 |
| Omitted term and upper-tail addition | `6*C(B)` | 1 |
| One outward-square pair | `6*C(B)` | 1 |
| One power-builder bit step | `4*C(B)` | 1 |
| One endpoint's quotient/remainder/tie decision | `8*C(B)` | 1 |
| Certified carry/decade/decimal-point normalization | `16*C(B)` | 1 |
| One base-`10^9` conversion step | `4*C(B)` | 1 |

The bounds are intentionally not minimal. With a cache miss, interval and
endpoint work, excluding power/string conversion, has exactly
`2*N_l + p + 14` visits in this schedule: setup, two `(setup + N_l + final)`
helpers, constant combination,
dyadic product, Taylor terms, tail, four squares and two quantizations. Context,
output planning, final normalization, cache lookup/insertion and power/conversion
loops are additional explicit units. Recount the schedule if source structure changes; do not hide
another first-party loop inside an existing visit.
This is the count of a **completed** miss attempt, not an upfront claim for all
its future blocks. The fixed atanh/Taylor counts `N_l` and `p` describe actual
predetermined iterations; each entered block/iteration uses the table's guard.
The conversion row applies once per actual division iteration, not once per
potential chunk slot reserved in a vector.

### 8.3 First-Party Loops And Conversion Buffers

The visit unit matches `numeric.rs`: one admitted arithmetic loop iteration,
not one CPU operation and not one limb multiplication. `num-bigint` internal
arithmetic is not counted as first-party visits. If implementation moves a limb
algorithm into project-owned code, its actual outer/inner iterations must be
priced; a single visit cannot stand for arbitrary first-party limb work.
Known full-pass entry/write counts can be admitted in a block immediately before
that selected pass, with no second charge for its covered iterations. Variable
comparisons, byte examinations and stopping iterations cannot use their upper
bounds as such a block claim. Every successful claim remains cumulative after
an early stop or fatal refusal; no visits are added for later unentered work.

Build `10^r` by left-to-right checked binary exponentiation over the already
admitted small/indexable `r`: accumulator starts at 1 and each exponent bit,
from most significant to least significant, squares it and, if set, multiplies
by 10. After that bit the accumulator is `10^prefix`; `prefix<=r` proves the
size bound for both intermediate operations. There are exactly `bit_length(r)`
bit steps, each with at most one square and one multiply. For `r=0` return 1
directly. No `.pow(u32)` narrowing of a larger exponent or unpriced project-owned
repeated multiplication loop.

Convert a nonsymbolic positive integer of at most `b` bits by repeated exact
division by `10^9`, storing least-significant `u32` chunks. An exact bound on
chunk count is `m = max(1,ceil(b/29))`, since `2^29 < 10^9`.
Reserve one chunk vector of capacity `m`, one small chunk-emission scratch array
of 9 bytes, and the final admitted string; no reversed digit string is required.
`m` bounds capacity and loop complexity only; it is **not** a conversion-visit
claim. For positive input, use a fallible division loop that admits one actual
iteration's quotient/remainder materialization bytes and then its one visit
before division or examination. After computing/storing that chunk, the admitted
iteration tests the quotient and stops if it is zero. Thus the last actual
division/stop decision is covered without an unadmitted look-ahead or a claim
for unused chunk slots. Zero has an explicitly admitted branch instead. The
actual chunk count is the resulting vector length, at most `m`.

Leading chunk emits without leading zeros; each later chunk emits nine digits.
Determine leading-chunk occupancy with actual guarded examinations, then charge
the selected emission pass's known actual character/chunk counts before writing.
Iterating a `BigUint` digit iterator or copying a retained sequence likewise
prices actual examined/copied entries: a complete known copy can be a block;
an early-stopping examination uses individual guards. The huge symbolic integral
power branch never converts its physical coefficient: it writes an admitted zero
run directly. Zero runs, affixes and padding after a carry are charged at their
actual selected write lengths, independently of the conservative buffer capacity.

For an adaptive Forest set of `v` values, retain one vector of records
`{normalized_exponent_bits: u64, label: String}` and two `Vec<usize>` index
buffers, each with capacity `v`. The record size includes the label header;
each label's payload has its own admission. Index-buffer bytes are bounded by
`2*(size_of::<Vec<usize>>() + 2*v*size_of::<usize>())`, with the capacity-excess
rule above. The original semantic record order is retained.

Initialize indices in semantic order, then use stable bottom-up mergesort on
**complete UTF-8 label bytes**, merging runs of lengths `1,2,4,...` into the
other index buffer and swapping buffers after each pass. For `v>=2`, there are
`h=ceil(log2(v))` passes. Each pass makes at most `v` label comparisons and
exactly `v` index writes on completion. There are at most `ceil(v/2)` run starts
per pass; those actual run-entry visits plus one pass-entry/orchestration visit
are at most `v` for `v>=2`. Each actual comparison has one entry visit and at
most `L_max+1` examined byte/end positions, where `L_max` bounds the complete
label length. Thus `v + v*h*(L_max+4)` is a **complexity upper bound only**,
including index initialization, not a sorting-work admission.

Admit the known `v` index-initialization writes before initialization. At entry
to each actual full merge pass, admit its pass-entry visit and known `v` output
index writes; covered writes are not charged again. Admit each actual run start
before its bounded run setup. Before each actual label comparison, admit its
one comparison-entry visit, which covers fixed length/index metadata. Then,
before examining **each next byte/end position**, claim one visit. A mismatch
returns immediately; equal strings require the separately admitted terminal end
position. No `L_max` claim, suffix-tail claim or future-comparison prepayment is
made. Checked run endpoints and doubling cannot wrap.

The merge helper and byte comparator return `Result` and propagate refusal
immediately. An opaque standard sort, an infallible comparator that fabricates
an `Ordering` after saving an error, or a comparator with an unpriced byte loop
does not satisfy this contract. The pass/run/write and variable comparison
visits all use the same ledger; partial failed passes retain their claims and
cannot publish a collision decision or partial ordering after refusal.

Scan neighboring sorted indices once. Equal labels may repeat only if their
normalized exponent bits are equal; otherwise the precision attempt collides.
Normalization maps both exponent zero signs to the same key, and `10^E` is
strictly increasing, so bit identity suffices for Forest value equality at a
common scale. Each actually entered adjacent label comparison admits its one
comparison-entry visit (also its scan-iteration entry), then the fallible byte
loop's actual examined positions. Only for equal labels does it admit the
additional fixed exponent-identity comparison. Return on the first unequal
identity/collision; later pairs have no visit claims. `v*(L_max+3)` remains a
**complexity upper bound only**: one comparison/iteration entry, at most
`L_max+1` byte/end positions and at most one identity comparison per actual pair.
Do not preclaim that bound or charge the entry twice. The last examined pair,
mismatch byte or equality end position is admitted before its decision. Sets
of size zero or one need neither sorting nor a neighbor scan. This fixes the
first-party vector counts and actual-work guards; no unpriced sort, hash order
or quadratic scan.
Each public precision attempt and each formatting revisit is admitted. On a
collision advance uniformly through the validated public range; on exhaustion
return the owning source miss/presentation error. A resource refusal at any
step aborts immediately. The shared non-Forest owners substitute their exact
normalized scientific-value identity in the same admitted set procedure.

### 8.4 Precision-Keyed Operation-Local Constant Cache

Cache only the exact integer enclosure pair `(L10,U10)` keyed by `p`, within the
same operation ledger/lifetime. “Exact” here means the retained integer bounds,
not a rational equality to the irrational constant. There is no global cache or
font/environment dependency. The deterministic doubling sequence permits a small
ordered vector rather than a hash map.

Use the checked doubling ordinal `j` as the vector index for `p=128*2^j`;
before lookup admit one lookup visit, including its one precision-key comparison.
Because every operand starts at 128 and doubles, an operation-local cache prefix
cannot have holes: a miss is exactly at its current length. On a hit, borrowed
bounds avoid materialization; copying them charges two integer materializations. A hit
does not charge imaginary series iterations, and still charges the operand's
refinement/dyadic/Taylor/quantization work. On a miss, charge the helper blocks
above, then one insertion visit, one existing cache-entry claim and entry/vector
capacity bytes before retaining the pair. No new quota is introduced. If a cache
entry also becomes a retained planning record, claim that existing record too;
do not claim axis placement/measurement calls for this arithmetic cache.

For `j+1` retained precisions, the cache has exactly `j+1` entries and
`2*(j+1)` integer payloads, with checked explicit vector growth. Its peak payload
is bounded by `2 sum(i=0..j) (C(B(128*2^i,0))-H)`, plus entries/vector header
and capacity. Pin vector growth to capacities `1,2,4,...`: before an insertion
that exceeds capacity, checked-double the requested capacity, admit the new
vector's header/payload capacity and one move visit per existing entry, then
fallibly reserve and move entries without cloning their integer payloads.
The old vector's capacity remains live until those moves finish and is included
in that transient storage count; its byte claims are never refunded. The capacity
excess rule applies to the new reservation. Entry claims count actual entries,
not unused capacity. Cumulative insertion/allocation claims are retained after a drop.
Cache hits cannot bypass exhausted work/bytes or change the mathematical result.

## 9. Deterministic Core Order And Fatal Refusal

The target core accepts the exact exponent bits, a normalized concrete precision
policy and any actual cache dependency. It takes no accounting context. Its order is:

1. Validate finiteness and decode; check real reduction storage.
   Add scale exactly, perform Euclidean floor reduction and construct `n,F,D`.
2. Compute `g,h,t,r`, the symbolic integral/zero decision and the output body,
   affix and conversion-buffer size plan using bounded exact integer arithmetic,
   using the proved size bounds.
3. Fallibly reserve output/conversion capacity; reject failed
   representation/checked arithmetic before attempting a power or writing zeros.
   The output plan supplies real capacity bounds.
4. Resolve a symbolic integral/zero result directly if applicable. Otherwise set
   `p=128` and construct checked `B(p,r)`.
5. At every refinement, check actual representation and allocation requirements
   before constructing its scratch values. Cache hits preserve exact identity.
6. Fetch/build the `p`-keyed `ln(10)` enclosure, form dyadic `x`, run exactly `p`
   positive Taylor iterations and the tail step, then four outward-square pairs.
7. Build only the representable/indexable quantum power (or denominator 10), quantize
   both endpoints as integers at the same `t`, and compare. Do not format either
   endpoint as a label or normalize it separately.
8. On disagreement, drop per-operand scratch, checked-double `p`, derive actual
   size bounds and return to step 5. On agreement, normalize the one rounded
   physical value and serialize into checked, fallibly reserved buffers. Check
   actual selected digit, zero-run and affix lengths before writing. Freeze the
   resulting label operand.

Removing prices does not change the algorithm, interval proofs or allocation
boundary. No new precision quota or fallback representation is introduced.
Fatal refusal propagates through axis source/intrinsic/measurement/preparation
paths. Only an actually certified but nonunique complete candidate is a normal
axis source miss. Continuous/value-set adaptive exhaustion is the spec93
pre-font presentation error. Later geometry/font/replay consumes the certified
retained text.

## 10. Independent Rust Oracle: Roots, Not Logarithms

The oracle is a test-only Rust owner within the existing Rust project. It may use
the same pinned integer dependency and ledger type, but must not call production
logarithm constants, exponential series, range reduction helpers, interval
quantizer, formatter or precision-acceptance code. It decodes exponent bits
independently and verifies claimed reduction with the exact identity and bounds.
This separates algorithmic defects while retaining offline reproducibility.

### 10.1 Repeated Certified Roots Of Ten

For oracle precision `P >= 128`, `Q = 2^P`, begin with the exact pair
`L0 = U0 = 10*Q`. For `j=1..d` independently compute

```text
Lj = floor_sqrt(L_(j-1)*Q)
Uj = ceil_sqrt (U_(j-1)*Q)
```

Then `Lj/Q <= 10^(1/2^j) <= Uj/Q`. For
`f = sum(j=1..d) b_j/2^j`, stream the fractional bits from most significant
to least significant. Begin product endpoints at `Q`; whenever `b_j=1`, replace
them with `floor(product_lo*Lj/Q)` and `ceil(product_hi*Uj/Q)`.
The product encloses `10^f`. No root array is necessary: one current root pair,
one accumulator pair and admitted square-root scratch suffice. A negative tiny
exponent's `F` really can contain 1074 set bits; do not replace it with an
approximate fraction or skip root iterations based on a binary64 epsilon.

Every root endpoint is at least 1 in real units. The square-root derivative on
`[1,10]` is at most `1/2`, so each root endpoint's one-sided error satisfies
`e_j <= e_(j-1)/2 + 1/Q < 2/Q`. Product rounding can be incorporated as an
additional relative factor of at most `1 +/- 1/Q`, because each intermediate
product is at least 1. Writing `v = 1/Q`, after at most `d` multiplications,

```text
product_lo >= M*(1-3v)^d >= M*(1-3d*v)
product_hi <= M*(1+3v)^d <= M*(1+6d*v).
```

The last inequality follows from `(1+3v)^d <= exp(3d*v) <= 1+6d*v` when
`3d*v <= 1/2`, which holds for `d<=1074`, `P>=128`; the exponential bound here
is a proof inequality, not an oracle exponential computation. Thus one-sided
errors are at most `30d/Q` and `60d/Q`, and interval width is at most `90d/Q`.
They converge as `P` doubles. Integral exponents bypass roots entirely.

### 10.2 Integer Square Root, Bracket Proof And Iteration Guard

For a nonnegative integer `Z`, `Z=0` is explicit. Otherwise let `b=bits(Z)` and
start Newton iteration at `w = 2^ceil(b/2)`:

```text
next = floor((w + floor(Z/w))/2)
if next < w: w = next; repeat
else: a = w; stop
verify a*a <= Z < (a+1)*(a+1)
floor_sqrt(Z) = a
ceil_sqrt(Z) = a if a*a == Z else a+1
```

The initial `w` is at least `sqrt(Z)`. If `a0=floor(sqrt(Z))`, then for any
integer `w>=a0`, `w+floor(Z/w) >= 2*a0`, by `Z>=a0^2` and the arithmetic/geometric
mean inequality; hence no iteration falls below `a0`. For `w>a0`, `Z<w^2`, so
`next<w`. At `w=a0`, `next>=w`. Strict integer descent and this invariant prove
the returned floor root, and the final **two exact square comparisons** certify
it without trusting Newton's stopping test alone.

This loop also has a proved source guard rather than an unbounded repeated
subtraction algorithm. Until reaching `a0`, write real relative error
`rho=(w-sqrt(Z))/sqrt(Z)`. Initially `rho<=1`; each decreasing Newton step obeys
`rho_next <= rho^2/(2*(1+rho)) <= rho^2/2` as long as it remains above the root.
After `k` such steps `rho <= 2^(-(2^k-1))`. With

```text
k = ceil(log2(ceil(b/2)+2))
H_root(b) = k+2,
```

the absolute error is below 1. An integer still above the floor root must then
be `a0+1`; its next iteration is `a0`, and one further iteration confirms stop.
Thus at most `H_root(b)` loop visits are needed, including the confirmation.
An exceeded guard is a fatal internal arithmetic/certificate failure, not an
oracle approximation. Count and admit **each actual Newton iteration** before
its quotient/sum, plus initialization and bracket verification.
`H_root(b)` is an iteration guard/analysis bound, not prepaid visit credit. The
final iteration that confirms `next>=w` has its own admission before computing
the quotient or examining that stopping condition; no unused iterations are
charged and no performed iteration is charged twice.

Root radicands have at most `2P+4` bits, Newton quotient/sum and final squares
at most `2P+6`; use
`B_o = max(1084,2P+9,P+8+4*max(r,0))` for oracle materializations, including
rounding-cell cross-products. The exact oracle admission schedule is:

| Oracle block | Materialization allowance | Visits |
| --- | ---: | ---: |
| Unit, initial root pair and accumulator pair | `8*C(B_o)` | 1 per precision attempt |
| One fractional-bit entry and pair of root radicands | `4*C(B_o)` | 1 |
| One Newton initial bracket | `2*C(B_o)` | 1 |
| One Newton iteration | `4*C(B_o)` | 1 |
| One floor-root verification and ceiling decision | `6*C(B_o)` | 1 |
| One selected-bit outward product pair | `6*C(B_o)` | 1 |
| One half-boundary construction/comparison | `8*C(B_o)` | 1 |

Decode, size planning, power construction and any fixture digit buffers follow
§8's separate schedules, independently implemented. Every block claims bytes
before visits and runs only after both succeed. Per fractional bit there are
two initialization/verification pairs and at most two guarded Newton loops,
plus at most one product pair: at most `2*H_root(2P+4)+6` visits. Thus root
cascade work is bounded by `d*(2*H_root(2P+4)+6)` visits, not one visit for an
entire cascade. These are complexity bounds only, not aggregate work admissions.
Charge each entered bit/root/Newton block once and the selected-bit product
block only when its bit is set; stopping and bracket decisions remain guarded.
Streaming has at most 24 live integer handles, or `24*B_o`
significant bits, excluding fixture records. Actual source loops, boundary
comparisons and output copies have their own admissions; no third-party
square-root helper with unexamined source work is used.

### 10.3 Independent Rounding-Cell Checks

Start oracle numerical precision at `P=256` and double on an undecided cell
comparison. For a proposed coefficient `K` at quantum `10^t`, construct the
exact rational half-boundaries `(2K-1)*10^t/2` and `(2K+1)*10^t/2` **symbolically**.
Cancel `10^n` before constructing powers. For `r=n-t>=0`, compare
`2*10^r*product_lo` with `(2K-1)*Q`, and `2*10^r*product_hi` with `(2K+1)*Q`.
For `r=-1`, compare `2*product_lo` with `10*(2K-1)*Q`, and similarly for the
upper boundary. Lower containment is `>=`, upper containment is `<=`;
equality is permitted at either boundary exactly when `K` is even. This avoids
giant physical powers for short symbolic exponent output. Accept only enclosure
containment,
including an endpoint equality only on the even-selected side. For `K=0`, the
positive cell begins at zero. This is independent cell-membership code, not a
call to the production endpoint quantizer. Oracle notation/zero-padding checks
operate on the independently certified physical coefficient and policy.

The irrationality theorem proves eventual decision for nonintegral exponents;
the independent root error bound proves its interval convergence. Oracle
resource exhaustion is reported as failure to validate, never as agreement.
Store input bits, scale, notation/precision/zero policy, expected bytes and
the oracle's exact bounding integers/precision in executable fixtures when
implemented; never substitute a digest without a defined ordered input stream.

### 10.4 Exact Power Comparisons For Small Denominators

For a reduced exponent `E=a/q` with `q` small (fixture cases use 2 or 4), and a
positive rational boundary `c/d`, monotonicity permits a separate exact check:

```text
a >= 0:  10^(a/q) < c/d  iff  10^a*d^q < c^q
a <  0:  10^(a/q) < c/d  iff  d^q < c^q*10^(-a).
```

Reverse inequalities similarly. Admit every power/product before construction;
this path is only for bounded fixture exponents, not for constructing a giant
physical power of an extreme binary64 integer. Its equality case is a literal
integer equality, independently detecting incorrect tie/carry expectations.
Symbolic integral fixtures need no root or physical-power allocation.

## 11. Concrete Fixture And Provenance Catalogue

All policies below default to scale 0, Keep trailing zeros, absent affixes.
`D/S/E` mean Decimal/Scientific/Engineering; `F(d)` fractional and `G(a)`
significant. Hex words are the exact `f64::from_bits` input, not approximate
decimal exponents. These are **mathematically specified golden expectations**,
with certificates below. The implemented exact/symbolic vectors and 23 fractional
root/algebraic cross-checks are recorded in `docs/certified-log10-conversion.md`;
owner-set/font/composition/public-output integration remains separately staged.
These are not MPFR-generated inverse-power data.

### 11.1 Fractional Values, Scaling And Whole-Value Precision

| Exponent bits | Exact exponent | Policy | Expected label |
| --- | --- | --- | --- |
| `0x0000000000000000` | `0` | D/F(2) | `1.00` |
| `0x8000000000000000` | `-0` | D/F(2), either negative-zero policy | `1.00` |
| `0x3fe0000000000000` | `1/2` | D/F(2) | `3.16` |
| `0x3fe0000000000000` | `1/2` | D/F(2), scale 2, suffix `%` | `316.23%` |
| `0xbfe0000000000000` | `-1/2` | D/F(2) | `0.32` |
| `0xbfe0000000000000` | `-1/2` | S/F(2) | `3.16e-1` |
| `0xbfe0000000000000` | `-1/2` | E/F(2) | `316.23e-3` |
| `0x3fd0000000000000` | `1/4` | D/F(2) | `1.78` |
| `0x3fe8000000000000` | `3/4` | D/F(2) | `5.62` |
| `0xbfd0000000000000` | `-1/4` | D/F(2) | `0.56` |
| `0xbfe8000000000000` | `-3/4` | D/F(2) | `0.18` |
| `0x3fe0000000000000` | `1/2` | S/G(3) or E/G(3) | `3.16e0` |
| `0x3fe0000000000000` | `1/2` | S/F(2), scale 324 | `3.16e324` |
| `0x3fe0000000000000` | `1/2` | S/F(2), scale -324 | `3.16e-324` |

The exact rounding-cell certificates, all strict inequalities, are:

```text
3.16:     631^2 = 398161 < 10*200^2 = 400000 < 633^2 = 400689
0.32:      63^2 =   3969 <    200^2/10 = 4000 < 65^2 = 4225
316.23:   63245^2 = 3999930025 < 100000*200^2 = 4000000000
                                   < 63247^2 = 4000183009
1.78:     355^4 < 10*200^4 < 357^4
5.62:     1123^4 < 1000*200^4 < 1125^4
0.56:     10*111^4 < 200^4 < 10*113^4
0.18:     1000*35^4 < 200^4 < 1000*37^4
```

Each numerator represents the half-quantum cell boundary over 200. These
integer inequalities are also a compact independent certificate to check in
Rust, without trusting rounded printed approximations of roots. Exponent
notation and scale variants use the same certified coefficient with exact
place-value changes, except E/F(2) at `-1/2`, whose coefficient uses the
`316.23` certificate.

### 11.2 Integral Extremes And Exact Scale Addition

| Exponent bits | Policy | Expected label or exact byte construction |
| --- | --- | --- |
| `0x4074400000000000` (`324`) | S/F(2) or E/F(2) | `1.00e324` |
| `0xc074400000000000` (`-324`) | S/F(2) or E/F(2) | `1.00e-324` |
| `0x4074400000000000` | D/F(2) | `"1" + 324*"0" + ".00"` (328 bytes) |
| `0xc074400000000000` | D/F(2) | `0.00` |
| `0xc074400000000000` | D/G(3) | `"0." + 323*"0" + "100"` (328 bytes) |
| `0x43b0000000000000` (`2^60`) | S/F(2), scale 1 | `1.00e1152921504606846977` |
| `0x43b0000000000000` | E/F(2) | `10.00e1152921504606846975` |
| `0x43b0000000000000` | S/F(2), scale -324 | `1.00e1152921504606846652` |
| `0x7fefffffffffffff` (`f64::MAX`) | S/F(2) | `"1.00e" + decimal((2^53-1)*2^971)` (314 bytes) |
| `0xffefffffffffffff` (`-f64::MAX`) | S/F(2) | `"1.00e-" + decimal((2^53-1)*2^971)` (315 bytes) |
| `0x7fefffffffffffff` | D/F(2) | fatal unrepresentable/unadmitted length before a physical power/zero run |
| `0xffefffffffffffff` | D/F(2) | `0.00` without a physical power |
| `0xffefffffffffffff` | D/G(3) | fatal unrepresentable/unadmitted leading-zero length |

These constructions specify every output byte, including long zero runs; they
are not arbitrary placeholders for a future oracle result. The maximum exponent
integer is exactly 309 decimal digits (`10^308 < (2^53-1)*2^971 < 10^309`),
so the two symbolic scientific lengths follow. `2^60 mod 3 = 1` proves its
engineering exponent/mantissa. The scale-1 case detects an incorrect binary64
addition that would retain `2^60`.

### 11.3 Near-Carry, Subnormal Floor And Adaptive Exhaustion

| Exponent bits | Exact exponent / policy | Expected label |
| --- | --- | --- |
| `0x3fefffffffffffff` | `1-2^-53`, S/F(2) | `1.00e1` |
| `0x3fefffffffffffff` | same, D/F(2) | `10.00` |
| `0x4007ffffffffffff` | `3-2^-51`, E/F(2) or E/G(3) | `1.00e3` |
| `0x4007ffffffffffff` | same, E/F(2), Trim | `1e3` |
| `0x0000000000000001` | `2^-1074`, S/G(17) | `1.0000000000000000e0` |
| `0x8000000000000001` | `-2^-1074`, S/G(17) | `1.0000000000000000e0` |
| `0x0000000000000001` and `0x0000000000000002` | one D/AdaptiveFractional `{min:0,max:17}` set | nonunique at every precision; complete axis candidate misses |
| same adjacent pair | one S/AdaptiveSignificant `{min:1,max:17}` set | nonunique at every precision; complete axis candidate misses |

For a near-carry input `m-delta`, `10^(m-delta) < 10^m` and
`10^(m-delta) >= 10^m*(1-3*delta)`, since `ln(10)<3` and `exp(-z)>=1-z`.
The deficits in these rows are strictly below half of the pre-carry physical
quantum: `0.005` for the first two and `0.005`/`0.5` for the engineering row.
Thus the single physical rounding is exactly the stated power of ten.

For the adjacent positive subnormal pair, `0 < E <= 2*2^-1074` gives
`0 < V-1 < 12*2^-1074 < (1/2)*10^-17`, by
`exp(z)-1 <= z/(1-z)` for `z<1`. Both values round to the same label through
the maximum fractional precision and through 17 significant digits. For the
negative least subnormal, `0 < 1-V < 3*2^-1074`, also far inside the cell.
Nevertheless the positive pair's exact exponents and mathematical values are
distinct. No numerical refinement can repair this **public-precision** collision.
Do not misclassify it as a root/log interval failure or signed-zero equality.

### 11.4 A Production-Start Fixture That Must Refine

Use exponent bits `0x4043c00000000000`, exactly `39.5`, with D/F(0).
Its golden label is

```text
3162277660168379331998893544432718533720
```

Let this integer be `J`. The exact small-denominator certificate is

```text
(2J-1)^2 < 4*10^79 < (2J+1)^2.
```

It proves nearest integer rounding of `10^39*sqrt(10)`, independently of either
interval algorithm. The result is not attributed to an unperformed MPFR run.

This fixture **must** reject its production-start `p=128` enclosure. Each
logarithm helper's added tail unit makes its upper integer strictly larger
than its lower; hence `U10-L10>=4`. With `f=1/2`, the two distinct real arguments
`L10/32` and `U10/32` have floor/ceil endpoints differing by at least one unit.
The included first Taylor term is exactly those two endpoint integers; other
included differences and the upper tail are nonnegative. Hence base interval
width is at least `u`. Each outward square multiplies width by at least two,
because the lower endpoint is at least 1. Final width is at least `16u`.
Quantization multiplies by `10^39`, giving width at least
`16*10^39/2^128 > 2`. No monotone nearest-integer function can give equal
results at endpoints separated by more than 2. Thus actual precision doubling,
not a test-only reduced starting precision, is exercised. Later admitted
precision certifies by §6; the root oracle and integer cell certificate determine
the expected result independently.

There is also a closed sufficient precision for this fixture: **512 bits**, if
the required blocks are admitted. Both `V` and either cell boundary are below
`4*10^39`. The strict integer square certificate makes their squared difference
at least `1/4`, so each boundary is more than `1/(32*10^39)` away from `V`.
At `p=512`, `C4<2^30`, giving a one-sided physical error below
`10^39*2^-482`. It is smaller than that separation because
`32*10^78 < 2^317 < 2^482`. Thus the design must certify at 256 or 512 after
rejecting 128; a source realization reaching another successful precision would
contradict its stated interval bounds. A budget refusal before that decision is
still fatal and supplies no label. This is an analytic precision certificate,
not by itself an executed refinement trace. The current observed `128 -> 256`
trace is recorded separately in `docs/certified-log10-conversion.md`.

### 11.5 Historical Actual-Visit Boundary Fixtures (Superseded)

These fee-boundary fixtures document the former design only. Keep genuine
comparison, conversion, refinement and error-propagation behavior checks under
§12; do not implement quota refusal or exact-charge tests.

These are specified counter expectations for the actual-work contract, not
executed tests. Each fixture starts its counter observation **after** admitted
operand/label construction and buffer reservation; those earlier real costs are
not erased or declared free. Only the named phase's visit delta is isolated.

* **First-byte mismatch with a huge common suffix:** use D/F(0), prefix absent,
  suffix `100000*"z"`, exponent bits `0x0000000000000000` and
  `0x3fe0000000000000`. Complete labels are `"1" + suffix` and `"3" + suffix`.
  One label comparison costs exactly two visits: comparison entry, then first
  byte examination. The differing first byte is admitted before return; none
  of the common suffix is examined or charged by this comparison. Remaining
  comparison budget 2 succeeds; budget 1 admits entry and refuses before the
  first byte, propagating a fatal error. `L_max=100001` is not work consumption.
  For this two-record sort, initialization costs 2, pass entry 1, its known
  output writes 2, run entry 1 and its one comparison 2: delta 8, not the
  worst-case sorting bound. There is no additional opaque sort/comparator cost.
* **Terminal position is actual work:** comparing `"1"` with `"1"`, or with
  `"10"`, costs three visits: entry, the matching first-byte position and the
  terminal end/mismatch position. Budget 2 refuses before that last position;
  it cannot return equality or an order using an unadmitted termination check.
* **First-collision neighbor stop:** use three D/F(0) records with exponent bits
  `0x0000000000000000`, `0x0000000000000001`, `0x0000000000000002`, no affixes.
  All labels are `"1"`; stable label ordering preserves this occurrence order.
  Observe only the subsequent neighbor scan. Its first pair costs entry 1,
  byte/end positions 2, identity comparison 1: delta 4, then collision return.
  The second pair is not entered or charged. Budget 3 refuses before examining
  exponent identity and must return a fatal error rather than a source miss;
  budget 4 completes the collision decision.
* **Conservative conversion capacity, one actual division:** give the positive
  integer converter value 1 with proved size envelope `b=1025`, so it reserves
  `m=ceil(1025/29)=36` chunk slots. Observe the division phase alone. It enters
  exactly one admitted division iteration, stores remainder 1 and tests the
  zero quotient within that iteration: delta 1, not 36. With no remaining
  division visit, refusal precedes division/stop examination. Subsequent leading
  digit determination and the selected one-byte emission are separately guarded;
  neither gets a 36-slot or maximum-digit writing claim.

These fixtures also distinguish output/affix construction work from comparison
work: a full suffix copy pays its actual selected byte count, while a later
first-byte mismatch does not pay a suffix it never examines. They change no
oracle mathematics, display expectations, capacity bound or allocator promise.

## 12. Validation Obligations And Design Closure

The following are concrete implementation checks for these proved designs,
not deferred mathematical questions:

* Independently decode every catalogue bit pattern; assert `E=n+F/D`, the floor
  inequalities, zero normalization and exact scale addition. Include both least
  subnormals, integral extremes and a selected nonintegral midpoint beyond the
  integral-seed range. For the latter, `308.5`, bits `0x4073480000000000`,
  S/F(2) must be `3.16e308` by the square-root coefficient certificate.
* Check all catalogue integer inequalities using admitted exact Rust arithmetic;
  cross-check fractional results with the independently implemented root oracle.
  Test constant/Taylor/four-square enclosure invariants against it at successive
  precisions; compare after putting both algorithms on the same rational scale.
  Mathematical convergence does not imply every enclosure nests.
* Exercise the exact shared integer quantizer on rational synthetic boundaries
  (`5/2 -> 2`, `7/2 -> 4`, binade/decade carries), even though irrational Forest
  powers themselves cannot be exact ties. Verify normalization preserves `R`
  exactly, especially engineering fractional versus significant policies.
* Include `39.5` refinement, real representation/precision overflow and integral
  giant-output refusal. Preserve early-stopping comparison/conversion behavior,
  final-byte/end decisions and fatal merge/neighbor propagation. Use existing
  diagnostics for actual iteration counts when useful, without pricing guards.
* Verify real output capacities and live expression bounds against §8's
  mathematical schedule. Audit cache miss/hit/insertion and borrowed versus copied
  bounds. Cache-enabled/disabled successful output must be byte-identical.
* Verify complete-set adaptive collisions and exact-format source misses,
  fatal refusal propagation to Forest axis/intrinsic/preparation callers,
  exact-tick precedence, retained fonts/affix spans, composition identities and
  frozen SVG/PNG replay under the existing spec93 integration owners.

The display mathematics is closed here: exact floor/scaling, symbolic powers,
one-round quantization/carry, logarithm and Taylor enclosure recurrences, explicit
round-error/tail/squaring convergence, termination, real size bounds,
deterministic work order and an independently proved oracle are all
specified. There is no unresolved product choice or deferred “prove it” step.
The stated third-party allocation boundary is the existing resource-contract
boundary, not an invented promise to make an infallible dependency fallible.

The private source, real buffer/materialization census, independently executable
certificates and scoped compiler/test evidence are now recorded in
`docs/certified-log10-conversion.md`. Consumer/default transport, public policy,
font/cache/refusal propagation and integrated frozen-output evidence remain the
spec93 integration owners.
An implementation that changes a recurrence, precision rule, first-party loop,
capacity policy or dependency boundary must reconcile its proof and admissions
with this document before claiming certification. Code validation must use the
repository's focused checks and strict
`cargo clippy --locked --offline --all-targets -- -D warnings` (or `just lint`),
never `cargo check`. Documentation alone supplies no such run evidence.
