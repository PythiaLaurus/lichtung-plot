# Resource accounting: conclusions and recommended direction

## Executive conclusion

The resource-accounting system is a **logical admission policy**, not a measured
CPU or memory model. Its unit mixture, incomplete coverage and inconsistent
pricing make the aggregate unsuitable as a performance objective. The original
one-million-visit threshold rejects complete, reasonably sized figures that
finish quickly on the measured host. Keeping a fixture just below that threshold
does not demonstrate efficient code.

There is nevertheless real bookkeeping overhead. The exact 2,048-word reference
figure takes roughly **101 ms with normal accounting and 97 ms without
bookkeeping** for preserve-text SVG. Its paired median difference is **3.745 ms
/ 3.835%**, not the entire 101 ms. Disabling bookkeeping leaves substantial real
text processing, shaping, allocation, layout and output work in place.

The right response is **not** to increase a magic number, reduce the fixture,
add caches solely to cross the threshold, or instrument every missing scalar
operation. First decide which operational guarantees are actually needed;
separate those guarantees from descriptive profiling and performance optimization.

## Scope and evidence

The audited production baseline is `582b79946682588e824a790b59362b7ce98ad046`,
which completes the owner-local composition/prose component. Its final correctness
review and six focused real-owner regressions, formatting and strict Clippy passed
before this study. Further feature implementation is paused for this analysis.

Three artifacts contain the detailed evidence:

- [Static audit](resource-accounting-static-audit.md): caps and historical basis,
  ownership, actual producer paths, twelve findings, byte/record semantics and
  a census covering 25 Plot kinds plus composition/font/output owners.
- [Performance study](resource-accounting-performance.md): precise experimental
  hooks and limitations, thirteen fixtures, three policies, twenty-six format
  groups, output equality, claim frequency, peak RSS and reproducibility.
- [Raw timing samples](resource-accounting-measurements.csv): all **858 selected
  measured attempts**, arranged in 286 rows of corresponding three-policy trials,
  including execution order and refusals. Warmups are excluded from this table.

The experiment lives only in `/tmp/opencode/accounting-cost-study`, a detached
worktree at the same revision. Production source/caps in the main branch were not
modified. Its three policies are normal accounting/enforcement, counting without
threshold enforcement, and no bookkeeping/no threshold enforcement. Structural
validation, checked arithmetic, actual algorithms, font selection, caches and
bounded layout coordination remain. Final PNG comparisons retain identical PNG
gates in all policies. Experimental opt-outs are not a proposed public API.

The timing build has no new per-claim instrumentation. Counts and lifecycle traces
come from a separate instrumented build. Each timing group has two warmups and
eleven measured triples in rotating/reversed order. Successful output is compared
byte-for-byte; all successful caption modes also verify the complete ordered word
vector decoded from SVG. Memory observations use 117 separate processes.

These are optimized test-executable measurements on one Ryzen 9 7950X3D WSL2 host,
with the same installed fonts, warm OS caches and uncontrolled scheduling. They
are not a portable latency promise, an allocator profile or a measure of full
compile-time removal of all accounting scaffolding. Both experimental builds
passed strict locked/offline all-target Clippy; focused normal-policy, experiment
policy, public-route parity and source-completeness tests pass.

## What the near-million number means

For the exact reference figure—two Scatter observations, an 18,432-byte caption
containing 2,048 repetitions of `Measured `, and an exact 1600×8000 root—the
instrumented complete preserve-text operation reports:

| Quantity | Value | Actual meaning |
| --- | ---: | --- |
| Charged visits | 999,771 | Sum of heterogeneous logical amounts |
| Visit-claim invocations | 415,556 | Number of visit-budget calls; a call can charge zero, one or many units |
| All ledger claim invocations | 516,328 | All seven resource classes, excluding delegation/usage/cache locks |
| Measurement/intrinsic attempts | 1,152 | Only named measurement boundaries, not all functions or claims |
| Shaping calls | 2,403 | Actual whole-render Rustybuzz calls, including axes and caption |
| Cumulative admitted bytes | 56,366,327 | Logical requests/estimates, not measured heap or RSS |
| Normal preserve-text peak process RSS | 49,020 KiB median | Separate fresh-process observation, including fonts/framework/buffers |

Thus neither “one million instructions” nor “only a handful of operations” is an
accurate description. Hundreds of thousands of claim calls really occur, but
most are individually small. Their aggregate timing effect is measured in
milliseconds here. The accounting unit cannot assign equal CPU cost to a byte
comparison, an `exp` kernel evaluation and a BigUint operation allowance.

### What the caption actually does

The source-backed text path is a custom locally greedy exact-layout pipeline:

1. Shape the full paragraph to obtain its natural width and proposal advances.
2. Calculate hard width by preparing each segment between legal break boundaries.
   The repeated-word fixture has **2,048 such preparations**, without a local
   repeated-segment cache in that loop.
3. At the assigned width, propose a line from prefix widths, shape it exactly,
   retreat if necessary and extend only to the immediately next fitting endpoint.
   Stop at the first failed extension. There is no all-wrap or whole-tree search.
4. Retain the selected lines and use them for capture/Null/SVG replay; publication
   does not reshape the caption. Outlining does separately traverse glyph paths
   once for counting and once for emission.

Relevant owners are `src/layout/text.rs::{intrinsic_in_call,select_line,
itemize_bidi_run}` and `src/render/svg_backend.rs::{SvgOutlinePlan,
outline_planned_line}`. The itemizer binary-searches the relevant span; it does
not scan the whole paragraph for every word. Ordinary repeated words have
roughly linear shaped-text volume, but this is not a proof of universally linear
runtime under all contextual scripts/widths.

Repeated segment preparation, constructing full glyph payloads when only a hard
width is retained, profile/source comparisons, and repeated outline traversal
are reasonable optimization candidates. Their actual CPU/allocation contribution
requires profiling. Text alone is not a generally sufficient shaping-cache key:
font, script, direction, paragraph context and break semantics matter. No new
cache or alternate text algorithm is admitted by this report.

## Measured overhead and refusals

These are preserve-text medians in milliseconds, not isolated caption-only time:

| Complete input | Counting, limits bypassed | No bookkeeping | Paired overhead median |
| --- | ---: | ---: | ---: |
| Scatter, 100 observations | 20.094 | 20.134 | 0.006 ms / 0.03% (noise-scale) |
| 200-word caption with 100 observations | 28.277 | 27.530 | 0.674 ms / 2.48% |
| Exact 2,048-word reference figure | 102.269 | 96.978 | 4.616 ms / 4.82% |
| 4,096-word reference figure | 181.078 | 172.355 | 7.895 ms / 4.56% |
| Four-variable, 100-row Matrix | 25.305 | 24.864 | 0.533 ms / 2.14% |
| Mixed Canvas | 22.631 | 22.252 | 0.356 ms / 1.57% |
| 128 coincident Beeswarm points | 37.153 | 34.707 | 2.546 ms / 7.43% (wide spread) |

The paired statistics are medians of per-trial differences, not subtraction of
the independent medians in the table. Normal-versus-off for the successful
reference2048 is separately **3.745 ms / 3.835%**. Merely disabling threshold
enforcement while keeping bookkeeping has no consistent measured benefit.

More importantly, normal enforcement refuses:

- The 100-observation 2,048-word figure, which completes at **1,009,748 visits**
  with counting but no limit—only slightly beyond the current threshold.
- Both full 4,096-word inputs, which finish preserve-text rendering in about
  **166–181 ms** across the unlimited policies on this host.
- The valid-height 128-point coincident swarm, which finishes in about **35–37 ms**.
- **Outlined SVG of the same reference2048 scene whose preserve-text SVG succeeds.**
  Its retained font environment exhausts the visit ledger during glyph outlining;
  both unlimited policies produce the same 7,115,390-byte outlined SVG.

A refusal time is not a successful-render speedup. These examples disprove a
simple interpretation of the threshold as an infeasible-resource boundary; they
do not prove that arbitrary unlimited inputs always finish cheaply. Dense swarms
above 128 points, other hosts/fonts and multithreaded contention were not measured.

## Main accounting defects

The static findings were independently checked against production reachability,
not only public-looking helper names or specification claims. The clearest are:

| Problem | Why the current interpretation fails |
| --- | --- |
| Discovery reserves one million records even for a small actual catalog | This is a real upfront backing-capacity request with no matching record/byte debit; reserved space is not necessarily resident RSS |
| Prepared font-pair hits clone Vec/String payloads without storage admission | A priced lookup does not prove the returned copy is priced |
| Label-domain arrays charge 4n records while inserting 2n array elements | The same materialization is charged at initial reservation and again before each push |
| Alias lookups and early-exit source searches precharge whole lists | Unused entries can consume budget even when the actual lookup stops early |
| Sort and numerical work use incompatible pricing rules | Input count, estimated `n log n`, actual iterations and squared-limb estimates share one uncalibrated total |
| Some source/font/census/significance paths remain unpriced | Later exhaustion cannot guarantee that earlier variable work was bounded |
| Outline elements and canonical payload record coverage are incomplete | A final draw-command count is not a complete census of its variable retained payload |
| Byte growth uses both full-replacement and delta conventions | The total is neither a proven upper nor lower bound on real allocated or peak memory |

Not every omitted operation must become a new charge. The old specification
intentionally excludes some storage-only copying, hashing or dependency work,
while newer code sometimes charges it. That policy disagreement must be resolved
before treating either implementation as the universal rule. Similarly, a
prepared table is not automatically a cache entry merely because it is reusable.

The constants also lack a measured service-level basis. One million was inherited
across several different count classes. The 512 MiB magnitude was historically
chosen as a power-of-two envelope for a maximum RGBA image, then used for the
different quantity of cumulative font/source/scratch admission. This is not a
memory calibration. The dense swarm's **273 MB logical byte charge versus about
40 MiB process RSS** directly illustrates the distinction, not a universal ratio.

## Recommended response

### Immediate working discipline

- Stop reporting budget headroom as efficiency. Report actual timing, process
  memory, output completeness and named work metrics separately.
- Stop optimizing to make one fixture land below one million. Keep representative
  inputs intact and measure real regressions independently of admission outcome.
- Keep the corrected geometry, font identity and immutable replay contracts.
  This analysis is not a reason to change scientific rendering or remove checked
  arithmetic and fallible reservations.

### Policy redesign before blanket enforcement repair

Separate three responsibilities:

1. **Structural correctness/representability:** finite numbers, checked lengths,
   valid source ranges, typed construction, consistent fonts, source preservation,
   canonical geometry and bounded layout coordination. Retain these irrespective
   of accounting policy.
2. **Explicit protection for potentially large work:** actual input/topology,
   output/raster size and specific expensive loops such as collision packing or
   numerical refinement. Set scope and limits according to documented deployment
   needs and measurements; do not silently equate those units to character scans.
3. **Performance observation:** clearly named metrics collected for diagnosis,
   separated from a mandatory heterogeneous refusal score. Include claim-call
   frequency itself so instrumentation cost is visible.

For the current mixed visit score, the measured evidence argues against preserving
the same hard million-unit refusal rule as the default solely because it already
exists. Options include retiring its hard enforcement or making a separately
specified policy opt-in, while retaining targeted protections. Choosing that API
and deployment policy requires an explicit next decision; no new switch is added
by this study, and merely raising the cap is not the recommendation.

For memory, choose a truthful target: **retained/live owned payload**,
**cumulative requested allocation traffic**, or **a conservative per-algorithm
scratch envelope**. They can be useful separate metrics; the present mixture
cannot stand in for all three. Actual RSS or allocator peaks require separate
measurement, especially around fonts, shapers, outline conversion and PNG.

Fix concrete duplicate claims and unexplained upfront reservations as defects,
but do not respond to uncovered work by adding a mutex call to every byte. Once
the intended metric and enforcement scope are chosen, use appropriately batched
or owner-local mechanisms and measure their overhead. Profile the repeated-word
hard-width and outline paths before choosing optimizations; validate identical
source, glyph and output semantics rather than only a lower score.

## Current outcome

The requested implementation checkpoint is committed. The comprehensive analysis
and isolated experiments are complete, with raw timing evidence retained here.
The experimental bypass has not entered production, no production threshold was
raised, and no following feature migration was started. The next step is to agree
on the accounting system's intended contract and disposition using this evidence.
