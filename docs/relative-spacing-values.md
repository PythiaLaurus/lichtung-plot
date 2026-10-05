# Private Exact Relative Spacing Values

The private foundation implements spec94 §2 and its §7 `P(p,m)` assembly
operation on base `5f2c705`. `RelativeSpacing`, its constructors and all metric
resolution interfaces are crate-private.

## Identity and inspection

`src/theme/spacing.rs` retains one reduced nonnegative rational using the existing
pinned `BigUint` dependency. An immutable `Arc` payload shares the numerator and
denominator across point styles, guide observations and clones. Equality compares
the reduced integers; it does not compare the inspection float or pointer identity.

| Construction | Exact retained value |
| --- | --- |
| `new(f64)` | The represented finite nonnegative binary64 dyadic |
| `ratio(u32,u32)` | Mathematical division, reduced with bounded Euclidean gcd; denominator must be positive |
| Private `from_pixels(p,m)` | Exact represented binary64 pixel length divided exactly by positive integral source em |

All zero inputs, including negative floating zero, normalize to `0/1` and inspect
as positive zero. `new(0.6)` differs from `ratio(3,5)`, even though their binary64
inspection values match. Equivalent mathematical ratios compare equal.

The immutable inspection result is computed during construction: `new` preserves
the normalized source bits, and the two division factories use IEEE nearest-even
division of exactly represented operands **only for inspection**. Their retained
integers are assembled independently. Each admitted factory is bounded above by
`f64::MAX`, so `as_f64() -> f64` is finite, allocation-free and infallible; sufficiently
small ratios may inspect as zero. Cosmetic resolution never reads this cache.

## Local metrics and exact boundaries

`LocalMetric` accepts a checked positive integral CSS em/radius or a positive
canonical `LayoutScalar`. Integral CSS metrics multiply by 1024 with checked
arithmetic. The caller supplies its selected role's metric.

`product` produces an unrounded nonnegative quantum requirement. `ExactLength`
supports exact addition, integral repetition, adding canonical integral ink and
rational comparison. These transient arithmetic operands have no clone, inspection
float or appearance identity. The selected boundary rounds once with floor,
nearest-even or ceil and checks the signed layout-quantum limit. Hard aggregates
can therefore include every gap before upward rounding.

Independent integer expectations in the focused tests include:

| Operand | Expected result |
| --- | --- |
| `ratio(6,14)` × 14px, H | 6144 quanta |
| `ratio(5,14)` × 14px, H | 5120 quanta |
| `new(5.0/14.0)` × 14px, H | 5121 quanta |
| Represented `1.3` × 20px, H | **26625 quanta** |
| Exact represented `8.8 + 0.2 × 11`, S / H | **11264 / 11265 quanta** |
| Twice that complete edge requirement, H | 22529 quanta |
| Three exact one-third-quantum gaps plus 1024 quanta, H | 1025 quanta |

The `P(p,m)` tests restore and compare the exact represented source lengths for
all five nonintegral legend literals: 8.8, 17.6, 3.3 and 13.2 against em 11;
7.8 against em 12. Integral 2, 11, 40 and 160 also preserve exact source lengths.
Tests pin source bits, distinguish a rounded `new(p/m)` from exact assembly, and
exercise finite binary64 extremes, subnormal ties, zero, signed metric rejection,
rounding parity and exact-limit overflow.

## Active production consumers

- `TextLineHeight::Multiplier` uses this exact product and H boundary during both
  initial measurement and reflow, using the selected integral font size and actual
  font environment's operation resources. Its retained leading remains `f64`.
  The former text-local binary64 product/shift implementation is removed; maximum
  text widths use the existing exact layout-scalar floor conversion.
- `PointOutline::width_ratio` stores `RelativeSpacing`. Its fallible private
  `publication(resources)` constructor preserves `0.1`'s source bits through
  `new_in`, rather than substituting `1/10`. Point appearance clones share
  immutable integer storage.
- Outline resolution uses the final integral point radius and returns canonical
  nearest-even width quanta directly. Geometry bounds and captured SVG primitives
  consume that same resolver. Positive collapse, layout overflow and backend
  exact-coordinate overflow refuse. Existing outward painted-edge rules consume
  the normalized width.
- Guide compatibility and feedback observations retain the exact relationship.
  Tests distinguish `new(0.6)` from `ratio(3,5)` and merge equivalent reduced ratios,
  with observation storage admission covering only the shared handle and entry.

Complete theme/legend relationship storage and complete production spacing sums
are the subsequent role-consumer migration. `P(p,m)` and aggregate operations are
implemented and tested here; the current legend appearance still stores its pixel
fields. Narrow dead-code expectations identify those pending assembly/aggregate
entry points and the inspection getter.

## Resource admission

Construction admits the value handle, immutable payload and Arc counters before
building retained integers. Operation-aware constructors and every exact
expression use the supplied `OperationGenerationResources`. Operand bit lengths
determine bounded work and byte claims, including integer headers, conservative
limb capacity and scratch operands, before shifts, products, integer clones,
division/remainder, sums and comparisons. Euclidean iterations admit individually.

Built-in construction requires `Theme::resolve(resources) -> Result<ResolvedTheme>`.
Each preset passes the supplied ledger into its two retained outline constructors;
there is no infallible `PointOutline::default` path. Plot's existing private
`new_with_theme_factory` receives the prepared stage's actual generation resources
and propagates construction errors before font preparation. Canvas passes the same
prepared ledger into its root and every explicit Plot preset; an inherited child
shares the root selection. The prepared-stage guard verifies that the supplied
generation ledger is the one owned by the stage's font resources.

Production text and point geometry/capture share that operation ledger;
prepared SVG and Null storage retain it too. Standalone backend fixtures explicitly
supply their ledger. Value clones share storage without allocating integer buffers.
Tests exercise zero visit/byte allowances, header-only refusal, exact entry
allowances, one-less refusal and cumulative shared-ledger exhaustion. Admission
does not promise allocator-OOM recovery inside the pinned integer dependency or
the standard Arc allocator.

On the verified 64-bit target, one publication-outline constructor claims **5
visits and 240 bytes**: a value handle, two-integer/inspection payload and Arc
counters, followed by four admitted one-limb operands with headers and conservative
limb capacity. A preset's two outlines claim **10 visits / 480 bytes**. A Canvas
root plus one explicit child claims **20 visits / 960 bytes**; its inherited child
constructs no additional preset. The byte tests derive these expectations from
the retained layouts and one-limb operand bounds rather than a prior measured run.
The real Plot factory and Canvas prepared-stage tests check all four presets at
zero, exact and one-less work/byte limits, with no font discovery or backend begin.

## Verification and default output review

The resumed focused verification is **40 distinct tests, all passing**: 28 unit
tests and 12 integration tests. Strict `just lint` passed (`cargo clippy --locked
--offline --all-targets -- -D warnings`), along with formatting and diff checks.
No full suite was run.

| Focused path | Passing tests |
| --- | ---: |
| Exact spacing values and arithmetic | 9 |
| Outline identity, resolution and constructor admission | 3 |
| Real Plot/Canvas prepared-ledger constructor guards | 2 |
| Caption workload, text-history equality, child comparison/clone and builder admission | 5 |
| Complete preset values/records, theme clone/matching, leading, Canvas reuse and guide identity | 9 |
| Shared-legend composition and theme ownership/defaults | 8 |
| Default visual regression | 4 |

The caption workload now passes with the retained-font work changes documented in
[`font-work-admission.md`](font-work-admission.md). The existing sources and sizes
are unchanged: 144 repeated `A` words, both 200-word caption forms and the
2048-word `Measured` fixture all retain their complete output.

The repaired feedback-history assertion independently counts **90** visits:
32 source bytes, 13 owner bytes, one candidate, one typed family plus its nine
name bytes, one line, one run and 32 glyphs. A first-glyph mismatch costs **59**,
leaving the other 31 glyphs unexamined. The former serialized stack-source pass is
not an operation performed by the current representation.

Child input comparison counts four shared immutable role stacks plus the changed
tick family's terminating byte: **16** visits, with later roles/feedback untouched.
Theme cloning admits eight RGB entries (**24 bytes**) and **18 immutable handle
operations**, sharing each role's entry/name payload. Tests check exact admission,
one-less refusal and preservation of the admitted record prefix when an atomic
color-entry claim refuses. Builder admission continues to share scientific sources.

The four visual tests cover **seven SVG snapshots**: Plot and Canvas root title
blocks, Scatter, Line, Trajectory, LinePath and categorical Trajectory. All existing
snapshots matched; no snapshot files or expected geometry were updated. Default
radii 1–4 still resolve `0.1` outline widths to 102, 205, 307 and 410 quanta, and
the selected title-leading check retains **26625 quanta**.
