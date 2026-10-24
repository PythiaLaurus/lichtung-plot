# Certified Mathematical Numeric Conversion

Numeric formatting functions receive retained mathematical exponent operands as
NumericOperand::PowerOfTen, not rounded physical powers. Their built-in helpers
use this certified engine with complete-set precision and frozen exact factors.
Domain views retain scale exponents and expose finite raw limits only when
genuinely retained; no inverse of a rounded logarithm is invented for a callback.

Data-space exact-count interpolation without retained raw endpoints uses positive
rational affine combinations of the two canonical powers. Each endpoint supplies
outward C/2^p bounds with symbolic decimal decades; cross-multiply and combine
before binary64 nearest-even conversion. Refine until both bounds select one cell.
Integral powers are rational and terminate exactly. Nonintegral dyadic powers are
positive monomials in one root of 10 whose power-of-two degree polynomial is
Eisenstein at 2; the positive rational combination cannot cancel its nonconstant
basis terms into a rational rounding boundary. Therefore refinement terminates.
Physical results outside finite binary64 or collapsed representable positions
refuse rather than inventing operands; scale-space powers remain symbolic.

## Mathematical Operand

For finite binary64 exponent e and validated decimal scale s in[-324,324], format
the positive mathematical value

```text
E = exact_binary64(e)+s
V = 10^E
```

An explicit finite general multiplier is another exact dyadic factor. Decode its
magnitude as C*10^b, retaining its sign separately, and display
`sign * C * 10^(E+b)`. Zero factors bypass refinement and emit zero. Integral
exponents use the same exact-decimal quantizer as ordinary operands, with symbolic
arbitrary-size exponents; no physical powf value is formed.
For fractional exponents, enclose M*C and compare against the next exact decimal
threshold to choose the effective decade. Irrationality proves that a nonzero
rational factor cannot put a fractional power exactly on that rational threshold.
Multiply certified endpoint numerators by C and include b in the fixed quantum's
decimal shift before the same signed rounding. Negative factors therefore use
negative floor/ceiling cells, not positive rounding followed by sign attachment.
Exact finite divisors retain C/D*10^b rather than rounding a quotient. Integral
powers use that same rational quantizer; fractional enclosures multiply endpoint
numerators by C and denominator by D. Decade and prefix-boundary comparisons cross
multiply these rationals only after symbolic decade screening, so arbitrary-size
physical exponents never require materializing an irrelevant power of ten. The
irrationality proof still excludes equality to nonzero rational thresholds.
Percent/per-mille multiply by exact decimal powers 2/3 in b. General uses the
significant quantum and selects decimal/scientific only from the rounded result;
Integer uses quantum 1 followed by exact arbitrary-size radix conversion. Reserve
integer output before constructing physical coefficients, with a conservative
four bits per decimal place. The reduction/quantum table below states the neutral
multiplier case; general factors shift its effective decade and fixed r exactly
as described above, without changing the original exponent operand.

Projection/source order/domain feedback remain exponent-space. Exact caller ticks
stay verbatim. Omitted Forest power-expression grammar is a private profile of
the [numeric engine](06_NUMERIC_PRESENTATION.md); this contract governs explicit
raw-value policies. Exponent 0.5 with Fractional(2) is `3.16`;
scale 2 plus `%` gives `316.23%`.
Neither e+s rounded in binary64 nor a temporary powf result is an operand. Do not
claim an exact inverse of a rounded scientific logarithm.

Every selected finite exponent is supported mathematically, including original-
domain midpoints beyond integral-seed clamps. Scientific notation can represent
powers outside binary64 physical-value range. Representation/allocation refusal
is distinct from the mathematical definition and never produces approximate text.
Source owners are `src/numeric/power.rs`, `src/numeric/power/certified.rs` and the
common coefficient emitter in `src/numeric/decimal.rs`.

## Exact Reduction And Decimal Quantum

Decode exponent bits. Normal significand is2^52+fraction with shift
stored_exponent-1023-52; subnormal significand is fraction with shift-1074. Apply
sign and cancel powers of2:

```text
exact(e) = A/2^d, 0<=d<=1074
D = 2^d
A+s*D = n*D+F, 0<=F<D
E = n+f, f=F/D
V = M*10^n, M=10^f, 1<=M<10
```

A/n use signed arbitrary-size integers, never narrowed to i64/i128/index types.
Euclidean floor, not truncating negative division, is required: -0.5 reduces to
n=-1,f=1/2; -0.25 to -1,3/4; -2^-1074 to -1,(2^1074-1)/2^1074. Scale addition is
exact: e=2^60,s=1 yields integral 1152921504606846977. Retain exact F and denominator
bit index; no 53-bit approximation after subtracting a negative floor.

Let engineering g=3*floor(n/3) by Euclidean division, h=n-g in{0,1,2}. Choose one
quantum10^t and fixed multiplier exponent r=n-t before refinement:

| Policy | t | r |
| --- | --- | --- |
| Decimal Fractional(digits) | -digits | n+digits |
| Scientific Fractional(digits) | n-digits | digits |
| Engineering Fractional(digits) | g-digits | h+digits |
| Any Significant(a) | n-a+1 | a-1 |

Desired coefficient is K=Q_mode(M*10^r), physical rounded value R=K*10^t.
Q_mode is the numeric policy's exact rounding operation (NearestEven by default).
Fractional digits are 0..20, significant a is 1..21. Apart from Decimal fractional,
r is 0..22. In Decimal fractional r<=-2, M<10 proves coefficient 0 for nearest,
TowardZero and Floor, or 1 for AwayFromZero and Ceiling, without refinement;
only negative r=-1 requires intervals. Adaptive precision chooses this concrete
display precision uniformly at its set owner, independently of numerical refinement.

### Symbolic Integral Branch

F=0 means exact 10^n. Do not run series or build 10^abs(n). Significant coefficient
is 10^(a-1), scientific fractional 10^digits, engineering fractional 10^(h+digits),
with small exponent<=22. Decimal fractional n+digits>=0 is a symbolic one followed
by that many zeros; n+digits<0 rounds to 0, or to one quantum under AwayFromZero/
Ceiling, since the positive value is <=0.1 quantum. Decimal significant tiny
nonzero powers require their full leading-zero output, not rounding to binary64
zero. Symbolic exponents remain arbitrary-size until checked output conversion.

### Endpoint Quantization And Carry

For interval endpoint Y/S, S=2^p:

```text
r>=0: numerator=Y*10^r, denominator=S
r=-1: numerator=Y, denominator=10*S
q=numerator/denominator; z=numerator%denominator
NearestEven: K=q+[2*z>denominator or (2*z==denominator and q odd)]
NearestAway: K=q+[2*z>=denominator]
NearestTowardZero: K=q+[2*z>denominator]
TowardZero/Floor: K=q
AwayFromZero/Ceiling: K=q+[z!=0]
```

Accept only equal endpoint integer K at the **same t**, before text, trimming or
affixes. Each rounding operation is monotone, including ties; equal endpoint
rounding certifies the intervening real. Do not independently choose decades from endpoints: upper
mantissa may exceed10 while true M<10, without changing the exact selected quantum.

For positive K with c digits, rounded decade is n_R=t+c-1; Scientific mantissa is
K*10^(t-n_R), Engineering uses g_R=3*floor(n_R/3). These are exact decimal-point/
zero-run operations, not another numeric rounding. Significant carry K=10^a
rewrites as10^(a-1) at t+1. Scientific fractional carry10.00 becomes1.00 with
exponent+1; Engineering1000.00 becomes1.00 with exponent+3, preserving requested
fractional occupancy. Engineering significant fractional places are
max(0,a-1-h_after); integral place-value zeros remain.

Decimal fractional keeps its digits including rounded0; Decimal significant keeps
fractional significance where expressible. Trim removes fractional zeros/empty
point only. Append literal affixes after normalization. V is positive even for
e=-0; a negative multiplier supplies a negative sign, independently of e's sign.
Scientific/Engineering
rounding at these quantum choices cannot produce0.

## Certified Mantissa Algorithm

Start p=128, S=2^p, u=1/S; recompute at p,2p,4p,... until certification. Every
division uses exact integers and stated floor/ceiling. Precision growth and full
intermediate sizes are checked before constructing scratch. No hard-case table,
hidden precision cap, midpoint approximation or remaining-work quota is used.

### Logarithm Constants

Use ln10=3*ln2+ln(5/4), ln2=2*atanh(1/3), ln(5/4)=2*atanh(1/9). For z=1/3 or1/9,
N_l=floor(p/3)+1:

```text
Plo=floor(S*z); Phi=ceil(S*z)
Qlo=floor(Plo*Plo/S); Qhi=ceil(Phi*Phi/S)
L=0; H=0
for k=0..N_l-1:
    L+=floor(Plo/(2*k+1)); H+=ceil(Phi/(2*k+1))
    Plo=floor(Plo*Qlo/S); Phi=ceil(Phi*Qhi/S)
return (2*L,2*H+1)
```

The atanh expansion is positive. Initial power error<=u, square error<2u and
Qhi/S<1/8 for p>=128. Assuming power error<=2u, next upper error is at most
(1/8)*2u+(1/3)*2u+u<2u; lower uses 1/9 and is no larger. Dividing by the odd
denominator adds<=u; doubled sum error<=6*N_l*u. Omitted tail satisfies

```text
2*sum(k>=N_l) z^(2k+1)/(2k+1)
 <= 2*z^(2*N_l+1)/((2*N_l+1)*(1-z*z))
 <= (3/4)*9^-N_l < 2^-p
```

since9>8 and3*N_l>p. The extra upper unit covers tail. Each returned endpoint
has one-sided error<=(6*N_l+1)u. Thus

```text
L10=3*L2+L54; U10=3*U2+U54; D_l=24*N_l+4
L10/S <= ln10 <= U10/S
```

Each error<=D_l*u and width<=2D_l*u ->0. Geometric bounds ln2<3/4 and
ln(5/4)<9/40 give ln10<99/40; D_l*u<4-99/40 at 128 and decreases, so U10<4S.

### Dyadic Product And Positive Exponential

Using the full exact F:

```text
Xlo=floor(F*L10/2^(d+4)); Xhi=ceil(F*U10/2^(d+4))
x=f*ln10/16; A_p=2*N_l+2
```

These bound S*x with 0<=Xlo<=Xhi<=S/4 and each error<=(D_l/16+1)u<=A_p*u.
Include exactly p nonconstant Taylor terms:

```text
Tlo=S; Thi=S; Ylo=S; Yhi=S
for k=1..p:
    Tlo=floor(Tlo*Xlo/(S*k)); Thi=ceil(Thi*Xhi/(S*k))
    Ylo+=Tlo; Yhi+=Thi
Ohi=ceil(Thi*Xhi/(S*(p+1)))
Yhi+=2*Ohi
```

Positivity/monotonicity encloses each included term. At upper argument<=1/4,
later term ratios<=1/4, so tail<=4/3 times first omitted term, hence twice Ohi
is conservative. Recurrence one-sided error obeys
epsilon_k<=(1/(4k))*epsilon_(k-1)+u<=4u/3. Included-sum error<=4pu/3 and omitted
term rounding<=8u/3. Since(p+1)!>=2^p, twice exact first omitted<=2^(-3p-1).
exp(z)<=4/3 on[0,1/4], so parameter error<=4A_p*u/3. Combined one-sided error is
at most C0*u with C0=4*(A_p+p+3)<=8p+32, including parameter, sum, recurrence and
tail, even when late upper terms ceil to one unit.

### Four Outward Squares And Convergence

Perform exactly four pairs `Ylo=floor(Ylo^2/S)`, `Yhi=ceil(Yhi^2/S)`. Stage j
encloses exp(2^j*x), final M. Exact stage values are<=10; do not extend the
small-argument4/3 bound to all squares. With C_(j+1)=22*C_j+1, upper stage
values remain<11 and one-sided error<=C_j*u:

```text
C4=234256*C0+11155 <=234256*(8p+32)+11155
```

At128, C4<2^28 and C4*2^-p decreases for p>=128, proving the<11 invariant used
in the square error bound(11+10)*C_j*u+u. Final interval width<=2*C4*2^-p ->0
along precision doubling. Enclosures need not nest; each contains the same M.

### Certification And Termination

For reduced rational E=a/b, if10^E=c/d were reduced rational, prime2 valuation
of(c/d)^b=10^a gives b*(v2(c)-v2(d))=a, hence b divides a and b=1. Nonintegral
rational E therefore gives irrational V, also for negative a. Decimal rounding
boundaries(k+1/2)*10^t for nearest modes and k*10^t for directed modes are rational.
A nonintegral power lies a positive distance from adjacent boundaries; shrinking
mantissa enclosures times fixed exact 10^r
eventually lie wholly in one cell. Endpoint rounding then agrees. Integral E
already has exact symbolic handling.

The result is correctly rounded with eventual mathematical termination given
sufficient available representable storage. Checked doubling/shift/layout/byte
length failure and actual reservation refusal remain fatal. They cannot become
axis coarsening, adaptive collision, lower display precision or best-effort text.

## Representation And Output Bounds

Compute conservative output capacity symbolically before giant powers/zero runs,
using bounded arbitrary-size planning expressions. Only a representable length
converts to usize and passes isize::MAX allocation layout, including affix bytes.

| Decimal policy | Conservative numeric-body capacity including carry |
| --- | --- |
| Fractional d, tiny symbolic shortcut | 1+(d>0 ?1+d:0) |
| Fractional d, otherwise | max(1,n+2)+(d>0 ?1+d:0) |
| Significant a, n>=a-1 | max(1,n+2) |
| Significant a, n<a-1 | max(1,n+2)+1+(a-1-n) |

Trim can shorten, not justify constructing an unrepresentable upper bound first.
For exponent notation r<=22. The ordinary power exponent magnitude needs<=1025
bits; a geometric index can produce a larger symbolic exponent. Reserve
`8+fractional+ceil((bits(n)+2)/3)` bytes for the numeric body, including exponent
carry and engineering adjustment, since 2^3<10. Fractional is d, or a-1 for
significant precision. This is a capacity bound, not a public exponent limit.
Affixes/storage are additional. Decimal fractional output bounds n/r before narrowing/building
10^r. Tiny integral rounded0 requires no giant negative power.

Finite exponent magnitude<2^1024 remains below it after scale addition; fractional
numerator needs<=1084 bits, denominator<=1075, F<=1074, n magnitude<=1024 and
small-shift/carry/engineering exponent<=1025. For nonnegative r use
b10(r)=1+4r and ell(p)=ceil(log2(p+2)) by integer bit length.

| Intermediate before division/shift | Sufficient significant bits |
| --- | ---: |
| Reduction numerator/remainders | 1084 |
| Constants/fixed sums/ln10 endpoints | p+3 |
| Constant product plus ceiling | 2p+3 |
| Exact F*constant plus ceiling | p+1077 |
| Taylor product/sum | 2p+4 |
| Taylor denominator including p+1 | p+1+ell(p) |
| Outward square product/ceiling, Y<11S | 2p+9 |
| Quantization power | 1+4r |
| Quantization numerator/carry r>=0 | p+5+4r |
| Denominator/doubled remainder r=-1 | p+5 |

Check envelope max(1084,2p+9,p+1077,p+1+ell(p),p+5+4max(r,0)) before scratch.
Do not check only a smaller post-shift result. Accepted coefficient needs<=4r+5
bits for r>=0, at most2 for r=-1, independent of refinement p. These bounds govern
real representation/layout arithmetic, not priced allocations or a cumulative
resource ledger. Internal num-bigint allocations are not promised fallible.

Operation-local cache stores exact integer ln10 enclosure pairs keyed by doubling
precision p. Ordinal j for128*2^j gives a contiguous vector prefix; no holes or
global/font cache. Borrowed hits preserve identical bounds; every operand still
runs its own dyadic/Taylor/square/quantization. Vector growth and final digit/
zero-run/string buffers use checked lengths and fallible first-party reservations.
No cache hit changes mathematical acceptance or bypasses real failure.

## Retained Geometric Operands

`src/numeric/lattice.rs` retains positive represented anchor a, positive represented
ratio r!=1 and signed arbitrary-size integer k. Its exact operand V=a*r^k is rational;
the callback borrows the canonical signed decimal index rather than an approximate
logarithm or physical pow. `src/numeric/logarithm.rs` encloses
`log10(a)+k*log10(r)+log10(abs(effective_factor))`, keeping decimal factor exponents
symbolic. A negative index reverses ratio-bound endpoints before multiplication.

For positive rational x=N/D, exact binary range reduction gives x=m*2^e with
1<=m<2. Enclose ln(m) using z=(m-1)/(m+1), hence 0<=z<1/3, and the same outward
atanh series and tail bound as above. Add e*ln2 with signed interval arithmetic,
then divide by the positive ln10 bounds with exact floor/ceiling in 2^-p units.
The constant error multiplied by fixed e/k still tends to zero under doubling.
An arbitrary dyadic log enclosure endpoint becomes one reduced Log10Operand and
uses the same mantissa Taylor/square engine, not a second formatter.

### Rational Equality And Termination

Unlike nonintegral powers, rational geometric values can equal decimal/binary
rounding boundaries. Strip powers of 2 and 5 from each represented factor and
threshold, retaining signed valuations symbolically. Cancel cross-products of the
remaining kernels. Equality requires matching valuations and one equation
`kernel^abs(k)=target`. A kernel of one is immediate; otherwise abs(k)>bits(target)
disproves equality. Remaining exact repeated squaring is bounded by the stored
target, stopping before any product would exceed it. No machine-sized index cast
or enormous irrelevant power is needed. The same test recognizes V=10^n exactly
and dispatches its symbolic exponent through the existing power engine.

Non-power values refine until logarithm endpoints have one common floor decade.
Quantize both mantissa bounds at that one decade/quantum. If adjacent coefficients
straddle an exact boundary, factorized equality resolves the shared signed
quantizer's tie. Otherwise separation and convergent enclosures prove eventual
agreement. Linear binary64 coordinates similarly compare adjacent rounding cells,
using exact rational midpoint equality and even parity. Log10 coordinates refine
their signed bounds directly; a rational V can have rational log10(V) only at an
integral decimal power, already handled symbolically. Thus no unresolved exact
binary midpoint remains.

### Geometric Index Bounds

`src/numeric/lattice/generation.rs` uses exact preimages of the represented domain
endpoints: rational binary midpoint cells on Linear, dyadic scale midpoint cells on
Log10, inclusive only for even endpoint significands. Let d be +1 for r>1, -1
otherwise; internal j increases with value and original k=d*j. Enclose
`(boundary-log10(a))/(d*log10(r))` with signed outward rational division. Refine
until its floor is unique, or neighboring floors identify an exact boundary via
factorized equality. Convert open/closed edges to inclusive integer first/last
indices. Their count remains symbolic until cap/platform checks. No log quotient
is rounded to a float index, and no epsilon or old integral exponent clamp applies.

For ordinary power mantissas d<=1074, the scratch envelope above is unchanged.
Geometric enclosure endpoints have d=p, so their full F*constant/ceiling products
fit the existing 2p+9 bound. Input kernels and index products also retain checked
stored lengths; internal num-bigint allocator behavior is not promised fallible.
Constant caching fills any skipped doubling levels, preserving the same contiguous
prefix and exact cached bounds for both operand kinds.

## Independent Numerical Verification

The independent Rust owner `src/numeric/power/tests/oracle.rs` must not call
production constants, exponential series, range reduction, endpoint quantizer,
formatter or acceptance test. Decode exponent bits independently and verify the
exact reduction identity/bounds. Shared pinned integer arithmetic is permitted.

For oracle precision P, Q=2^P, start L0=U0=10Q and stream roots:

```text
Lj=floor_sqrt(L_(j-1)*Q); Uj=ceil_sqrt(U_(j-1)*Q)
```

They enclose10^(1/2^j). Stream fractional bits of F/2^d most significant first;
starting accumulators at Q, multiply outward by current root iff bit is1.
Keep current root/accumulator pairs, not an array. Tiny negative exponents can
require all 1074 bits. Root one-sided error<2/Q, product endpoints bound M with
errors<=30d/Q and60d/Q, width<=90d/Q, using
(1-3/Q)^d>=1-3d/Q and(1+3/Q)^d<=1+6d/Q for P>=128. This converges independently
of production logarithms.

Integer root of positive Z starts w=2^ceil(bits(Z)/2), iterates
`next=floor((w+floor(Z/w))/2)` while next<w, then verifies
`a*a<=Z<(a+1)*(a+1)`. Strict integer descent never passes floor_sqrt. Relative
error squares/halves per Newton step; guard
`ceil(log2(ceil(bits(Z)/2)+2))+2` includes final confirmation. Exceeded guard or
failed exact bracket is verification failure, not approximation.

Start cell checks at P=256, double on undecided. Independently compare exact
symbolic half-boundaries(2K-1)*10^t/2 and(2K+1)*10^t/2, cancelling10^n first.
For r>=0 compare2*10^r*product endpoints against(2K+/-1)*Q; r=-1 puts10 on
boundary side. Permit endpoint tie only for even K; K0 starts at positive0.
Do not call production rounding to certify itself. Real oracle storage failure
means failure to validate. Exact small-denominator cases additionally compare
integer powers: for E=a/q and boundary c/d, a>=0 tests10^a*d^q versus c^q;
a<0 tests d^q versus c^q*10^-a. Only bounded fixture exponents construct these.

Independent expectations cover half ties/carries, negative floors, subnormals,
exact scale addition, symbolic extremes, notation/zero runs and actual refinement.
For example Decimal Fractional(0) at e=39.5 must refine 128 to 256 and round to
`3162277660168379331998893544432718533720`. This is a mathematical expectation,
not a renderer-scene golden. Keep exact input bits/policy and certificate bounds
for numerical witnesses; no output digest or execution receipt substitutes for
the oracle or proof.
