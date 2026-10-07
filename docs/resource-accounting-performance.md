# Resource-accounting performance study

## Findings

**Base:** `582b79946682588e824a790b59362b7ce98ad046`. **Measured:** 2026-10-07,
on the local Linux/WSL2 host described below. All experimental Rust changes and
raw evidence are retained in the detached worktree
`/tmp/opencode/accounting-cost-study`. This report and the portable raw timing
table [resource-accounting-measurements.csv](resource-accounting-measurements.csv)
are the only main-checkout files written by this study. The separate
`resource-accounting-static-audit.md` was preserved. No commit, push, dependency installation, network access, font
provisioning, or subagent was used.

The important results are:

* The **exact existing 2,048-word reference fixture** reproduces **999,771
  processing visits**, but these arise from **415,556 visit-claim calls**, not
  999,771 locks. Across all seven resource classes it makes **516,328 ledger
  claim calls**. Preserve-text rendering takes a median **101.158 ms** normally
  and **96.978 ms** with bookkeeping disabled. The median **paired** difference
  is **3.745 ms / 3.835%**. Near-quota admission is not an efficiency certificate.
* Disabling only threshold enforcement, while retaining accounting, provides
  **no consistent timing benefit**. Disabling bookkeeping produces measurable
  but workload-dependent differences: tiny/noisy on simple scatter, several
  milliseconds on long captions, and a larger relative difference on the
  supplemental dense 128-point beeswarm. These are not universal percentages or
  isolated mutex-cost measurements.
* Normal accounting refuses the complete 100-point/2,048-word caption, both
  4,096-word variants, and the 128-point beeswarm. Both unlimited policies finish
  those unchanged sources with identical output. The near-million-visit reference
  caption succeeds as preserve-text SVG but **fails during outlined-SVG
  conversion**. Its unlimited outlined outputs are byte-identical.
* The complete 4,096-word reference caption takes **181.078 ms** with counting
  and **172.355 ms** without bookkeeping as preserve-text SVG. Its normal refusal
  at **84.266 ms** is an incomplete operation, not a faster successful render.
* Cumulative admitted bytes are not physical memory. For example, dense `bee128`
  charges **273,245,121 bytes**, but fresh-process peak RSS is about **40,500 KiB**.
  Disabling bookkeeping leaves the scene, fonts, caches, scratch algorithms,
  ledger objects, and their allocation requirements in place. RSS shows no
  consistent meaningful saving attributable to bookkeeping.

These measurements complement the static audit; they do not revise its source
coverage findings or calibrate a safe production threshold.

## 1. Experimental policies and isolation

One optimized test executable runs all three policies on the same thread. A
thread-local `Cell<u8>` selects the policy before fixture construction, and stays
constant throughout each render. There is no per-claim atomic mode load.

| CSV policy | Full name | Behavior |
| --- | --- | --- |
| `normal` / N | Normal | Original inclusive limits, checked counter arithmetic, mutexes, successful increments, and refusal formatting. |
| `count` / C | CountNoLimit | The same mutexes and checked cumulative increments, with every root/font ledger threshold predicate bypassed. |
| `off` / O | NoBookkeepingNoLimit | Early success before ledger locking, incrementing, or resource-error construction. The accounting-only font-environment wrappers also return before their `RwLock` access and resource-handle clone. |

The bypass covers all seven ledger classes: **visits, planning records,
topology slots, measurement/intrinsic attempts, cache entries, font discovery
entries, and cumulative font/source/scratch bytes**. It does not replace limits
with a larger finite allowance. In the authoritative **ledger-only PNG follow-up**,
all three PNG policy gates—axis size, pixel area, and outlined-parser-input
bytes—retain their original limits and execute identically in N/C/O. This
includes `count_parser_bytes`, its emission into `CountBuffer`, checked byte
additions, and limit comparisons before deriving the parser input. The
representative PNG is below those limits in all modes.

The earlier PNG run was a combined-policy experiment: C/O bypassed the explicit
PNG gates and passed `usize::MAX` to the parser byte counter. It **did not skip
the byte-counting route**, `CountBuffer` emission, checked additions, or its
comparisons. Its raw results remain historical evidence and are not pooled into
the ledger-only follow-up. SVG never enters these PNG gates; the original SVG
timings and all claim/RSS results remain selected unchanged.

The checked source/capacity/geometry arithmetic, fallible reservations, finite
and representable dimensions, ownership tickets, font sealing and glyph coverage
remain active. The eight root rounds, eight guide alternatives, numerical tick
source constants, and optional 256-entry exact text-cache policy are unchanged.
No source-counting or sizing loop was short-circuited. Statistical data, source
paint, theme defaults, ordering, shaping, and packing are identical across modes.

In particular:

* `FontBudget` delegates visits/bytes to the **same generation ledger** as the
  root. The experiment preserves its Arc identity and `same_operation` checks;
  it does not substitute a dummy environment or disconnect the source from fonts.
* `DISCOVERY_ENTRY_LIMIT` also supplies the existing discovery vector's reservation
  capacity. That reservation is unchanged. The off-mode reservation query returns
  the unchanged production configuration without taking the configuration mutex.
  Enforcement is bypassed separately; no enormous “unlimited” vector is reserved.
* Font database/cache locks needed for actual font operations and palette/text/
  child caches remain. “No bookkeeping” does not mean “no renderer locks.”
* Font discovery happens once per root in all successful diagnostic cases and
  starts afresh for every timing sample. There is no global font cache. OS file
  caches and the process allocator can remain warm.
* `usage()` values are not used to select a different production representation.
  The existing test-only `plan_text_quanta_with_policy` audit had an unconditional
  pre-query usage read even with its logging environment variable unset. All
  three associated audit blocks are compiled only into the separate diagnostic
  build, so that read/lock is absent from **every primary timing arm**.

The off arm still evaluates claim arguments, executes the mode branch, retains
resource objects/identity checks, and performs all real work. Thus this measures
the removable claim-path work in this experiment, not complete compile-time
erasure of every accounting-related instruction. Additional branches in N/C,
compiler/code-layout effects, and measurement noise mean it is **not a rigorous
numerical lower bound**, nor can the difference be assigned solely to mutexes.

The current reproducer uses `experiment-tracked-ledger.patch`,
`experiment-policy.patch`, and `experiment-driver-ledger.patch` in the temporary
worktree. Earlier patches remain as versioned historical evidence. Non-test
builds of those hooks select normal enforcement. No production API or cap in
the main checkout changed.

## 2. Inputs and timed boundary

All inputs are deterministic formulas/literal strings; there is no random seed
or sampled subset. Each policy gets the complete same fixture. `Plot`/`Canvas`
construction is outside timing; each render prepares a fresh root.

| Case | Complete input and CSS-pixel dimensions |
| --- | --- |
| `scatter100` | 100 points: x = 0…99, y = `(i*37)%101`, labels Predictor/Response; 960×720. |
| `caption200` | Same 100 points; the existing ten-word “Measured responses remained stable across independent trials under controlled conditions.” sentence repeated 20 times in one paragraph; 1200×1000. |
| `caption2048` | Same 100 points; `"Measured ".repeat(2048)`; 1600×8000. |
| `caption4096` | Same 100 points; `"Measured ".repeat(4096)`; 1600×16000. |
| `reference200` | Exact existing `three_paragraphs_200` fixture: x/y = `[0,1]`, labels x/y, sentence repeated 7/7/6 times separated by newlines; 800×900. |
| `reference2048` | Exact existing `long_2048` fixture: those two points, 2,048 occurrences of `Measured ` (18,432 source bytes); 1600×8000. |
| `reference4096` | Same two-point reference source, extended to 4,096 occurrences; 1600×16000. |
| `scientific_groups` | The 100-point scatter, three cycling categorical treatments Control/Exposed/Recovery, `Theme::Science`, title and caption; 1200×900. |
| `matrix4` | Unchanged publication benchmark fixture: four variables × 100 rows, value `(i*(j*12+7))%101`, labels `Variable j`; 1400×1400. |
| `mixed_canvas` | 1×2 scatter100 and three-estimate horizontal bar, estimates 12/19/15 and original long treatment labels; 1600×800. |
| `shared_canvas` | 1×2 scatter100 children with Science/Nature themes and one shared x-range group; 1600×800. |
| `bee64` | 64 coincident values at x=0, one Dense category, continuous colors `i/64`; 1600×1200. |
| `bee128` | Supplemental 128 coincident values, colors `i/128`; 1600×3000. |

The two reference inputs come from
`src/theme/resolved.rs::caption_publication_work_evidence_keeps_complete_required_sources`.
That original test was also run successfully, without changing its assertions.
The 100-point caption variants deliberately retain the scatter workload; they
are not presented as a reproduction of the two-point fixture's totals.

An initial supplemental bee128 probe used 1600×1200: N refused its visit limit,
and C reached a genuine hard-track fit failure. The command failed after about
0.10 seconds; that incomplete attempt is not performance evidence of a valid
render or a quota false positive. The subsequent 1600×3000 fixture fits and uses
that same height for every policy. No points were removed and no caps increased.
Only 64/128-point dense swarms were measured; **512-point behavior is unmeasured**.
The observed 128-point completion times do not establish that a 512-point render
would be prohibitively expensive, and no such claim is made.

### Timing procedure

Primary wall timing covers the real in-memory `render_scene()` pipeline:
preparation including font discovery, layout/feedback, capture, null validation,
SVG replay/serialization, scene construction, obtaining the requested output
bytes, and destruction of that local scene. Preserve-text SVG is the primary
format. Outlined SVG includes actual glyph outlining and the public terminal's
copy of derived bytes. Each outlined sample starts from a fresh root, not a
previously cached outlined scene.

The PNG sample also starts from a fresh scatter100 root, outlines, rasterizes,
and actually encodes source-sized RGBA PNG with balanced compression. PNG
decoding/hash validation is outside timing. File publication, directory work,
hashing, byte comparisons, source construction, and CSV writing are outside all
primary timings. Preserve mode adds a copy of the borrowed canonical SVG into
the harness's result vector; this common copy is disclosed rather than treated
as filesystem publication time.

Each case/format uses **two warmup triples and eleven measured triples**. Each
triple executes each policy exactly once. Policy order rotates and reverses,
cycling through all six permutations. Same-iteration samples form the pairs.
Cases/formats are sequential; compilation and instrumentation were not run
concurrently with timing. None of the eleven measured samples was discarded.

The final report covers 26 case/format groups: 858 measured render attempts and
156 warmup attempts, selecting the original SVG series and the new ledger-only
PNG series. The PNG follow-up alone reran two warmup triples and eleven measured
triples; no SVG timing, claim census, or RSS series was rerun for this correction.
There are 70 successful case/format/policy combinations and
eight consistently refusing combinations. Within each successful combination,
all 13 outputs were compared byte-for-byte, including against the other
successful policies. Refusal-versus-completion pairs are excluded from overhead
calculations.

### Instrumentation separation

Primary runs do **not** activate `orchestrator::measurements::start()`. Claim
collection and `track_caller` are compiled out without `study-claims`. No added
per-claim frequency counters or observers run in the timing build. Existing
`cfg(test)` hooks—such as font shape/discovery counters, scene-route evidence,
and inactive observation checks—are still compiled, identically for all modes.
These are optimized test-binary measurements, not a stripped production binary.

The separate `--features study-claims` build collects a BTreeMap of resource,
caller file/line, claim invocations, and amount. Collection occurs exactly once
at each ledger leaf; font/environment delegation is not counted again. The
diagnostic retains original accounting and observer overhead and **must not be
used for timing comparisons**. Its C sums are asserted equal to the shared
root's independently observed usage for all six generation classes. O usage is
asserted zero. Discovery is a separate seventh class.

## 3. Release wall-time results

Entries are **median [minimum, maximum] milliseconds**, eleven samples. `R`
means refusal, not successful output. More digits and every raw duration are in
`summary-ledger.csv`, the selected `*-timing.csv` files, and the repository's
portable raw timing table. `summary.csv` retains the earlier combined-policy
PNG summary and is historical for that format.

### Preserve-text SVG

| Case | N | C | O |
| --- | ---: | ---: | ---: |
| scatter100 | 20.196 [20.063,20.467] | 20.094 [19.876,20.351] | 20.134 [19.979,20.236] |
| caption200 | 28.040 [27.804,28.594] | 28.277 [27.816,28.609] | 27.530 [27.142,27.948] |
| caption2048 | R 95.718 [94.780,98.322] | 96.012 [95.228,98.708] | 92.935 [91.086,95.379] |
| caption4096 | R 81.389 [79.695,81.804] | 174.807 [173.162,176.184] | 165.930 [163.747,169.272] |
| reference200 | 39.801 [39.392,41.221] | 40.018 [39.097,42.318] | 39.387 [38.335,41.181] |
| reference2048 | 101.158 [99.953,104.022] | 102.269 [98.881,106.885] | 96.978 [95.803,99.267] |
| reference4096 | R 84.266 [82.499,85.096] | 181.078 [178.538,183.205] | 172.355 [169.781,175.580] |
| scientific_groups | 21.965 [21.597,22.173] | 22.122 [21.498,22.441] | 21.725 [21.496,22.013] |
| matrix4 | 25.439 [24.818,25.836] | 25.305 [24.833,25.688] | 24.864 [24.252,24.985] |
| mixed_canvas | 22.555 [22.093,22.948] | 22.631 [22.002,22.961] | 22.252 [21.881,22.605] |
| shared_canvas | 22.371 [21.687,22.507] | 22.246 [21.560,23.380] | 21.814 [21.070,22.423] |
| bee64 | 26.300 [25.838,26.931] | 26.854 [25.614,27.099] | 25.825 [25.346,26.366] |
| bee128 | R 33.931 [33.099,34.564] | 37.153 [36.408,41.018] | 34.707 [33.991,38.168] |

The reference cases were measured in a later invocation of the same renderer,
after adding their fixtures to the driver. Cross-case comparisons are not paired
and also differ in width, labels, paragraphs, and source data.

### Outlined SVG and representative PNG

| Case/format | N | C | O |
| --- | ---: | ---: | ---: |
| scatter100 outline | 20.764 [20.580,21.237] | 20.716 [20.398,21.035] | 20.555 [20.232,20.935] |
| caption200 outline | 39.949 [38.996,41.288] | 39.558 [39.017,40.550] | 39.006 [38.493,39.548] |
| caption2048 outline | R 97.422 [94.518,99.797] | 226.513 [222.741,229.086] | 221.342 [217.644,225.307] |
| caption4096 outline | R 82.661 [80.339,83.385] | 441.119 [430.238,464.505] | 430.488 [419.520,441.861] |
| reference200 outline | 50.848 [38.252,51.343] | 50.669 [39.902,51.323] | 50.130 [37.919,51.020] |
| reference2048 outline | R 102.570 [100.209,107.575] | 234.921 [224.643,241.139] | 233.094 [217.863,237.516] |
| reference4096 outline | R 84.880 [83.542,87.202] | 439.520 [431.722,445.061] | 432.241 [421.405,445.102] |
| scientific_groups outline | 23.094 [22.528,23.536] | 22.966 [22.558,23.254] | 22.560 [22.214,23.194] |
| matrix4 outline | 27.117 [26.567,27.510] | 27.221 [26.309,27.408] | 26.423 [25.496,26.697] |
| mixed_canvas outline | 23.882 [23.507,24.444] | 23.867 [23.339,24.333] | 23.586 [23.023,24.266] |
| shared_canvas outline | 23.056 [22.594,23.235] | 22.892 [22.544,23.181] | 22.644 [22.082,22.911] |
| bee64 outline | 26.523 [26.001,27.258] | 26.787 [25.569,27.263] | 25.708 [25.325,26.695] |
| scatter100 PNG, ledger-only | 39.334 [38.640,39.876] | 39.075 [38.664,39.666] | 39.169 [38.945,39.823] |

Bee128 outlined public-output parity was checked, but it has no outlined timing
series. The reference200 outlined timing has a conspicuously broad range; its
largest paired C/O percentage is 35.35%. It is retained, not filtered away.

The superseded combined-policy PNG medians [min,max] were N 39.544
[39.047,39.754], C 39.643 [38.990,40.149], and O 39.347 [38.473,40.466] ms.
These are separate historical samples, not additional repetitions of the
ledger-only experiment. No causal performance difference between the two runs
is inferred.

### Correct paired overhead calculation

For each measured iteration `i`:

```text
delta_ms_i(A,B) = time_ms_i(A) - time_ms_i(B)
overhead_percent_i(A,B) = 100 * delta_ms_i(A,B) / time_ms_i(B)
```

The following are medians of those **eleven individual paired quantities**.
They are not percentages calculated by subtracting the independent medians in
the previous tables. Positive means the first policy took longer.

| Preserve-text case | N−O paired ms / % | C−O paired ms / % | C−O percentage min…max | N−C paired % |
| --- | ---: | ---: | ---: | ---: |
| scatter100 | 0.138 / 0.69% | 0.006 / 0.03% | −1.28…1.73% | 0.30% |
| caption200 | 0.597 / 2.17% | 0.674 / 2.48% | 0.27…3.95% | −0.40% |
| caption2048 | invalid: N refused | 3.329 / 3.49% | 2.47…6.44% | invalid |
| caption4096 | invalid: N refused | 9.031 / 5.45% | 3.27…7.07% | invalid |
| reference200 | 0.477 / 1.21% | 0.735 / 1.89% | −4.44…9.48% | −0.34% |
| reference2048 | 3.745 / 3.84% | 4.616 / 4.82% | 2.40…9.41% | −0.65% |
| reference4096 | invalid: N refused | 7.895 / 4.56% | 1.68…6.05% | invalid |
| scientific_groups | 0.239 / 1.11% | 0.344 / 1.59% | −0.53…3.01% | −0.25% |
| matrix4 | 0.575 / 2.31% | 0.533 / 2.14% | 1.00…4.34% | 0.43% |
| mixed_canvas | 0.381 / 1.71% | 0.356 / 1.57% | −0.71…3.20% | 0.04% |
| shared_canvas | 0.339 / 1.53% | 0.483 / 2.18% | −2.93…7.18% | −0.52% |
| bee64 | 0.503 / 1.96% | 0.860 / 3.35% | 0.01…6.15% | −1.19% |
| bee128 | invalid: N refused | 2.546 / 7.43% | −2.50…18.47% | invalid |

For the long outlined captions, C−O paired medians are 5.096 ms / 2.34%
(`caption2048`), 10.718 ms / 2.55% (`caption4096`), 4.897 ms / 2.10%
(`reference2048`), and 10.317 ms / 2.40% (`reference4096`). Actual outlining
dominates much of these longer output routes. For the ledger-only PNG follow-up,
N−O is **0.178 ms / 0.45%**, C−O is **−0.045 ms / −0.12%**, and N−C is
**0.013 ms / 0.03%**. The paired percentage ranges are respectively
−2.42…1.82%, −2.75…1.65%, and −1.11…2.07%; these samples do not establish a
PNG speedup. All selected paired results, including min/max absolute deltas,
are in `paired-ledger.csv`. The historical combined-policy PNG pairs remain in
`paired.csv`: N−O −0.123 ms / −0.31%, C−O 0.292 ms / 0.75%.

## 4. Claim frequency, units, and storage admissions

These are **separate instrumented preserve-text runs** of C, which completes all
sources. Each visit-claim invocation takes one generation-ledger lock in C, even
if the amount is zero or a large batch. Total ledger calls add the seven leaf
classes; they exclude delegation wrappers, resource-usage observations, cache
locks, identity checks, and accounting-only outer font `RwLock` accesses.

| Case | Visit-claim calls | Charged visit units | All ledger claim calls | Admitted records | Byte-claim calls | Cumulative admitted bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scatter100 | 32,753 | 158,259 | 49,536 | 7,729 | 10,893 | 27,693,531 |
| caption200 | 70,694 | 245,553 | 94,619 | 29,393 | 13,059 | 30,095,878 |
| caption2048 | 404,878 | 1,009,748 | 507,481 | 235,212 | 38,251 | 56,380,647 |
| caption4096 | 807,790 | 1,866,840 | 998,601 | 465,036 | 66,467 | 85,399,177 |
| reference200 | 54,154 | 206,420 | 73,100 | 24,355 | 10,948 | 29,970,141 |
| reference2048 | 415,556 | 999,771 | 516,328 | 234,560 | 37,832 | 56,366,327 |
| reference4096 | 807,684 | 1,855,356 | 996,077 | 464,010 | 65,896 | 85,337,679 |
| scientific_groups | 57,714 | 231,824 | 81,056 | 11,966 | 15,066 | 28,410,971 |
| matrix4 | 90,936 | 325,179 | 192,994 | 77,759 | 43,602 | 33,352,685 |
| mixed_canvas | 69,181 | 237,938 | 96,654 | 16,267 | 17,619 | 29,345,038 |
| shared_canvas | 81,904 | 259,962 | 113,653 | 20,838 | 19,346 | 29,626,374 |
| bee64 | 260,668 | 429,875 | 327,296 | 22,127 | 60,764 | 148,284,353 |
| bee128 | 1,342,896 | 1,565,799 | 1,462,525 | 62,895 | 111,525 | 273,245,121 |

All cases make **141 discovery claims charging 457 discovery units**. This is
one discovery operation, not 141 environments or 457 font files. The provenance
manifest lists 48 unique fontconfig-visible font files; canonical discovery and
configuration-entry accounting are different inventories.

For the exact near-million reference, the other independent classes are:

| Class | Leaf claim calls | Charged amount |
| --- | ---: | ---: |
| Planning records | 61,153 | 234,560 |
| Topology | 17 | 18 |
| Measurement/intrinsic attempts | 1,152 | 1,152 |
| Cache entries | 477 | 477 |
| Discovery | 141 | 457 |
| Bytes | 37,832 | 56,366,327 |
| Visits | 415,556 | 999,771 |

Thus “1,152 calls” in the existing observation API is not the total number of
resource method calls. Similarly, 999,771 is a sum of heterogeneous visit
amounts, not an observed CPU instruction count or a million lock acquisitions.
For successful N runs, these totals match C. O's ledger totals are all zero,
while its real source/scene work continues.

### Where the reference caption's visit claims originate

The file/line census propagates caller locations through canonical font wrappers.
Common higher-level admission helpers remain aggregation boundaries. Locations
refer to the temporary patched source; this is not a dynamic call-stack profile.

| Reference2048 aggregation boundary | Visit-claim calls | Charged visits |
| --- | ---: | ---: |
| `src/layout/text.rs` | 245,715 | 448,499 |
| `src/font/canonical.rs` | 84,977 | 292,305 |
| `src/theme/resolved.rs` | 64,045 | 168,813 |
| `src/layout/panel/sources.rs` | 9,954 | 13,014 |
| `src/theme/spacing.rs` | 5,991 | 35,038 |
| `src/layout/panel/cells.rs` | 3,921 | 3,921 |
| `src/layout/text/requirements.rs` | 227 | 37,121 |

Examples from `claims.csv`: the common text admission helper at `text.rs:1451`
aggregates its 245,715 calls; font shaping admission at patched
`canonical.rs:3483` makes 2,403 calls charging 75,505 input/context units; the
fontconfig source-scalar charge at `canonical.rs:4331` makes only 37 calls but
charges 87,125 visits. These illustrate why charged amount alone cannot estimate
lock overhead. The full file/line records are retained, not just these examples.

Bee128's source helper at `data/source.rs` aggregates **1,197,835 visit claims /
1,248,625 visits**, including downstream packing calls through that helper; this
must not be mislabeled as purely source preparation. Its exact color arithmetic
at `theme/color.rs` makes **95,256 byte claims charging 243,855,360 bytes**.
Those are cumulative scratch admissions, not 243 MB of simultaneously live color
storage. For the reference caption, bytes attributed to canonical font paths are
29,015,914 and to text paths 23,978,622; those module-level amounts are likewise
not an allocator census.

No per-caller latency can be derived by dividing the timing delta by one of these
counts: claim types, argument work, outer locks, batching, and real algorithms
differ. Static coverage and over/undercharging issues remain the subject of the
separate static audit.

### Diagnostic lifecycle evidence

The diagnostic observes one fresh font discovery per root. C/O shaping counts
match exactly: scatter100 13; caption200 257; caption2048 2,367; caption4096 4,752;
reference200 253; reference2048 2,403; reference4096 4,765; scientific groups 46;
matrix4 19; mixed Canvas 64; shared Canvas 15; both beeswarms 39.

`stages.csv` retains stage invocations and elapsed values from the instrumented
run. For example, reference2048 has one preparation, two root-measure rounds,
one capture, one null validation, and one SVG replay. Filesystem publication and
terminal wrapper timers are zero because the diagnostic calls the in-memory
scene path. Stage timers can nest and include claim-map overhead; they are not
an additive decomposition of the primary timings and are not compared across
instrumented policies for speed claims.

## 5. Refusal ledger and validity

N's counters record only accepted claims. `claims.csv` records attempted leaf
calls, including the final refused amount, so its N visit sum can exceed the
accepted total. C's sums match its accepted totals because no claim is refused.

| Refusing route | N accepted visits at preserve refusal | Reported attempted boundary / error | Successful C/O result |
| --- | ---: | --- | --- |
| caption2048, preserve/outline | 999,986 | y-axis tick label planning, root visit limit; leaf attempt sum 1,000,110 | 1,009,748 visits for full preserve pipeline; all 2,048 caption words retained |
| caption4096, preserve/outline | 999,994 | root title block, cumulative claim 1,000,003 | 1,866,840 preserve visits; all 4,096 words retained |
| reference4096, preserve/outline | 1,000,000 | root title block, cumulative claim 1,000,001 | 1,855,356 preserve visits; complete source |
| bee128 preserve | 1,000,000 | axis source work admission, cumulative claim 1,000,001 | 1,565,799 preserve visits; all 128 source points |
| reference2048 outline | Preserve stage succeeds at 999,771 | planned glyph outlining exhausts retained root visit limit | Identical complete outlined SVG in C/O; preserve already succeeds in N |

Preserve diagnostic totals are not claimed as complete outlined totals. For the
last row the exact terminal counter at refusal was not separately instrumented;
the actual terminal error, timing, and successful unlimited bytes were recorded.

These are empirical policy refusals of renderable complete inputs under the same
algorithms and host fonts. They establish that crossing one million is not itself
proof of infeasible CPU or memory consumption. They do not establish that all
unbounded inputs terminate cheaply or that every valid source should be admitted
in production. In particular, no quota headroom optimization, word shortening,
smaller statistical dataset, or production-cap adjustment was used.

## 6. Exact output and public-route verification

Every successful timed result was compared as a complete byte vector, not just a
hash. Root dimensions are part of those exact bytes and are logged explicitly.
The separate public-parity test covers all 13 cases × three policies: it calls
actual `Plot`/`Canvas::svg_preserve_text`, reusable `Output::svg`, and, for
scatter100, `Output::png_preview`. Public bytes equal the in-memory pipeline;
paired errors equal where N refuses. This is a focused custom release test of
public entry points, not a claim that the entire integration suite was run.

The original reference-caption test counted 20 occurrences of `Measured` for
the 200-word three-paragraph source, and 2,048 occurrences for the single-word
reference source. The former was **not a check of all 200 sentence words**.
The original custom check similarly counted markers for the 100-point long
captions; it did not cover `reference4096` with a complete-word assertion.

The correction adds an untimed `accounting_cost_caption_words` check. It parses
preserved SVG as XML, obtains decoded text-node content in document order, splits
on whitespace, and compares the trailing root-caption token vector against
**every expected source word in order**, including punctuation. These standalone
fixtures place the caption in the trailing text block. This checks both 200-word
sentence variants, both 2,048-word variants, both 4,096-word variants, and the
five-word scientific-groups caption. All 18 successful case/policy combinations
pass; the three expected N refusals are checked separately. Public preserved-SVG
byte lengths and SHA-256 also match the original recorded artifacts for all 18.
The same full-word helper replaces marker counting in the temporary public-parity
test. No input construction or timed rendering algorithm changed, and no SVG
timing repetition was needed for this stronger assertion.

Selected exact SHA-256 evidence:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| scatter100 preserve, N/C/O | 28,674 | `a70dabaf76b4d7256721576095342efa5a9084bb7a9d9f2796a99152d6a9f39c` |
| reference2048 preserve, N/C/O | 113,880 | `2de3a51ba96dc055f3d409aed790fb1f4e85747eb10f8526b708d6548e13d644` |
| reference2048 outline, C/O | 7,115,390 | `f7f5ae6d469e7f779ea74ee4c16c65a0037f5bc23dc35da18abd8c526e53f760` |
| reference4096 preserve, C/O | 211,719 | `521bc2c39131a790246257d9557ef5c6530407f84874515461610cb11e266f2b` |
| reference4096 outline, C/O | 14,327,822 | `09eb52a300dac6830cd2cc117b0144108dc1d2578cb2d2df6e0dd3141f9d6d1b` |
| scatter100 encoded PNG, N/C/O | 47,748 | `e52a835b886706694924cea24ec9d447874ed7b2148b56923debf91c16dea822` |
| scatter100 decoded RGBA8, 960×720 | 2,764,800 | `7063a10297456b84f07c3f24f36923fdfee2dfaef9449cdebc322bd5ccd44c50` |

Every other artifact's dimensions, byte length, and full hash are in the retained
`*-hashes.csv` files. No N hash is fabricated for a refused route.
The ledger-only PNG's exact encoded bytes match across all modes, and its entire
hash manifest—including dimensions, encoded hash, and decoded RGBA hash—is
asserted identical to the earlier PNG manifest.

## 7. Fresh-process memory and CPU observations

`/usr/bin/time -v` launched the non-instrumented release test executable directly,
with exactly one selected render per process. Each of the 13 cases × three
policies was run in **three fresh processes** (117 launches). These are separate
from the warm, paired timings. There is no allocator replacement or unsafe code.

Peak RSS below is **whole-process KiB**, median [min,max]. It includes the Rust
test framework, loaded code, font loading, allocator state, renderer and result
buffers, and hashing after the timed render. It does not measure exact allocated
bytes, allocation count, or the ledger's memory footprint. R rows stop early and
are not equivalent memory-saving comparisons.

| Case | N peak KiB | C peak KiB | O peak KiB |
| --- | ---: | ---: | ---: |
| scatter100 | 40,264 [40,188,40,724] | 40,432 [40,228,40,448] | 40,440 [40,196,40,724] |
| caption200 | 41,208 [40,716,41,296] | 40,984 [40,984,41,028] | 41,000 [40,988,41,004] |
| caption2048 | R 47,500 [47,488,47,508] | 48,992 [48,808,49,540] | 49,048 [49,036,49,068] |
| caption4096 | R 50,412 [49,712,50,420] | 57,932 [57,924,58,352] | 57,948 [57,900,58,060] |
| reference200 | 41,200 [40,460,41,228] | 40,928 [40,480,41,200] | 40,780 [40,476,41,032] |
| reference2048 | 49,020 [48,876,49,520] | 49,284 [49,240,49,448] | 49,236 [49,000,49,324] |
| reference4096 | R 50,420 [50,160,50,624] | 58,160 [57,868,58,372] | 58,128 [57,904,58,184] |
| scientific_groups | 41,220 [40,996,41,256] | 41,180 [41,024,41,200] | 40,776 [40,764,40,984] |
| matrix4 | 42,572 [42,268,42,760] | 42,496 [42,308,42,808] | 42,792 [42,280,42,796] |
| mixed_canvas | 40,980 [40,760,41,456] | 41,000 [40,712,41,024] | 40,712 [40,688,41,028] |
| shared_canvas | 40,952 [40,944,41,024] | 40,772 [40,712,41,248] | 40,984 [40,944,41,140] |
| bee64 | 40,508 [40,476,40,512] | 40,516 [40,472,40,732] | 40,520 [40,504,40,688] |
| bee128 | R 39,920 [39,704,39,948] | 40,512 [40,432,40,520] | 40,688 [40,468,40,720] |

The same logs contain user/system CPU times. Their coarse summed medians are
0.11/0.10/0.11 seconds for reference2048 N/C/O; reference4096 C/O are 0.19/0.18
seconds; bee128 C/O are both 0.04 seconds. All cases' CPU ranges are in
`rss-summary.csv`. These include process initialization and have roughly
centisecond reporting resolution, so they cannot resolve the smaller paired
wall-time differences. Fresh-process render wall times are also logged; for
example, reference2048 is about 116–121 ms rather than its warm-process 97–102
ms. No fresh-process result is mislabeled as a cold-filesystem measurement.

## 8. Provenance, commands, and retained evidence

Host tag: **DESKTOP-V5I2Q8H / WSL2 / Ryzen 9 7950X3D**. Linux
`6.18.33.2-microsoft-standard-WSL2`, x86-64, 32 logical CPUs, Microsoft hypervisor.
Rust `1.96.0 (ac68faa20 2026-05-25)`, LLVM 22.1.2, Cargo
`1.96.0 (30a34c682 2026-05-25)`. One Rust test thread; no affinity pinning,
frequency control, OS-cache flushing, or exclusive-host scheduling. Affinity
availability was not established. All renderer comparisons are within the same
process for each invocation, but scheduler/cache/thermal noise remains.

Cargo's ordinary release profile is used: optimized level 3, default release
codegen/LTO/panic settings, no custom profile in `Cargo.toml`, no added target-CPU
flags. `RUSTFLAGS`, encoded Rust flags, release opt-level/LTO/codegen overrides,
target-directory override, and the font-work audit environment variable were
unset. `Cargo.lock` is unchanged; its SHA-256 is
`39287b5226252555e2a84b8d45ead5ae03c68d5b3d08dcf20009e5da32d28484`.
`provenance.log` records compiler/CPU details, source hashes, and paths/lengths/
SHA-256 of all 48 fontconfig-visible files. Installed fonts were only read.

The non-instrumented executable is
`target/release/deps/lichtung_plot-0a8c7e65cb4fba85`; the claim-instrumented one is
`target/release/deps/lichtung_plot-86cebb3ff83ede00`. The reference fixture pass
adds driver code only; rendering/policy sources are unchanged from the final
primary pass. `experiment-driver.patch` retains the primary driver, SHA-256
`4d58e8811979d1b53f33989abb0d512ef30a54e111b2d4360c3096b7728f79db`;
`experiment-driver-reference.patch` retains its extension. That historical source
manifest remains in `provenance.log`.

For the PNG correction, the runtime part of `src/scene.rs` is restored to the
base implementation; its remaining diff is only the new policy-invariant PNG
gate test. Font selection, glyph work, accounting policy, and SVG render sources
are unchanged. `provenance-ledger.log` records the PNG timing/whole-word-check
checkpoint; `provenance-ledger-evidence.log` records the subsequent summary/export
helper extension. `experiment-tracked-ledger.patch` and
`experiment-driver-ledger.patch` reproduce the current temporary experiment.
The checkpoint hashes are:

| File/checkpoint | SHA-256 |
| --- | --- |
| `src/scene.rs`, PNG follow-up and current | `3a39c4cd1c66147393cb04200058b1a12e19b5e110af33c7d4206b318d12be83` |
| `src/accounting_study.rs`, PNG timing checkpoint | `099acadb1578bb57fab7f8645ad77e0058133025fbf794e3cfe9758c1af7bc5b` |
| `src/accounting_study.rs`, current export/verification extension | `e711df274a5fceabb6cf658960e4e37804737b3086742bab84850fe69366fecb` |
| `src/accounting_policy.rs`, unchanged | `ed66af41952033a380140c8f0b3d3d5f3e709a300b4c5330aaa4900087f7dea2` |

These compiler-output filenames are not immutable binary identities: the focused
test executable was rebuilt after the driver-only export extension. The manifests
distinguish source checkpoints rather than claiming that an old binary survived.

### Commands

Worktree creation was preceded by parent verification:

```bash
# Run in the main repository, before creating the detached worktree:
git status --short
git rev-parse HEAD
git log -1 --oneline 582b799
ls /tmp/opencode
git worktree add --detach /tmp/opencode/accounting-cost-study 582b799
```

All following commands use `/tmp/opencode/accounting-cost-study` as their working
directory. The root recipes' exact locked/offline Clippy command was used;
`cargo check` and full-suite commands were not used.

```bash
cargo fmt --all
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --features study-claims -- -D warnings

cargo test --release --locked --offline --lib accounting_cost_policy_contract -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib generation_limits_are_independent_inclusive_and_shared -- --nocapture --test-threads=1

STUDY_TAG=final cargo test --release --locked --offline --lib accounting_cost_timing -- --ignored --nocapture --test-threads=1
STUDY_CASE=scatter100 STUDY_FORMAT=png STUDY_TAG=png-final cargo test --release --locked --offline --lib accounting_cost_timing -- --ignored --nocapture --test-threads=1
STUDY_CASE=bee128 STUDY_FORMAT=preserve STUDY_TAG=bee128-final cargo test --release --locked --offline --lib accounting_cost_timing -- --ignored --nocapture --test-threads=1

# After adding the exact reference fixtures, strict Clippy was rerun in both builds.
cargo test --release --locked --offline --lib caption_publication_work_evidence_keeps_complete_required_sources -- --nocapture --test-threads=1
for case in reference200 reference2048 reference4096; do
  STUDY_CASE="$case" STUDY_TAG="$case" cargo test --release --locked --offline --lib accounting_cost_timing -- --ignored --nocapture --test-threads=1
done
cargo test --release --locked --offline --lib accounting_cost_public_parity -- --ignored --nocapture --test-threads=1
cargo test --release --locked --offline --features study-claims --lib accounting_cost_diagnostics -- --ignored --nocapture --test-threads=1
cargo test --release --locked --offline --lib accounting_cost_provenance -- --ignored --nocapture --test-threads=1

# Executed as original-case and reference-case batches; one render per process:
for case in scatter100 caption200 caption2048 caption4096 scientific_groups matrix4 mixed_canvas shared_canvas bee64 bee128 reference200 reference2048 reference4096; do
  for mode in 0 1 2; do
    for run in 0 1 2; do
      STUDY_CASE="$case" STUDY_MODE="$mode" /usr/bin/time -v -o "rss-$case-$mode-$run.log" target/release/deps/lichtung_plot-0a8c7e65cb4fba85 accounting_cost_fresh_process --ignored --nocapture --test-threads=1
    done
  done
done
cargo test --release --locked --offline --lib accounting_cost_summary -- --ignored --nocapture --test-threads=1
cargo fmt --all -- --check
git diff --check
```

The preceding commands document the original run; its `png-final` series is now
historical. The correction ran only these focused checks and PNG timings:

```bash
cargo fmt --all
cargo clippy --locked --offline --all-targets -- -D warnings
cargo clippy --locked --offline --all-targets --features study-claims -- -D warnings
cargo test --release --locked --offline --lib accounting_cost_png_gates_are_policy_invariant -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib accounting_cost_caption_words -- --ignored --nocapture --test-threads=1
STUDY_CASE=scatter100 STUDY_FORMAT=png STUDY_TAG=png-ledger STUDY_ROUNDS=11 STUDY_WARMUPS=2 cargo test --release --locked --offline --lib accounting_cost_timing -- --ignored --nocapture --test-threads=1
STUDY_LEDGER_FOLLOWUP=1 cargo test --release --locked --offline --lib accounting_cost_summary -- --ignored --nocapture --test-threads=1
STUDY_PROVENANCE=provenance-ledger cargo test --release --locked --offline --lib accounting_cost_provenance -- --ignored --nocapture --test-threads=1
# After the driver-only portable-export extension, strict Clippy passed again in both builds.
STUDY_PROVENANCE=provenance-ledger-evidence cargo test --release --locked --offline --lib accounting_cost_provenance -- --ignored --nocapture --test-threads=1
STUDY_LEDGER_FOLLOWUP=1 STUDY_VERIFY_PORTABLE=/home/robot-machete/projects/lichtung-plot/docs/resource-accounting-measurements.csv cargo test --release --locked --offline --lib accounting_cost_summary -- --ignored --nocapture --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Final Clippy passes are in `lint-ledger.log` and `lint-claims-ledger.log`;
earlier passes remain in `lint-reference.log` and `lint-claims-reference.log`.
The new PNG gate test checks inclusive source/target axis and pixel boundaries,
invalid dimensions, parser-byte boundaries, and the existing actual-glyph parser
refusal-before-derived-allocation test under **each** policy. It passes in release,
as do the whole-caption check, PNG-only timing, provenance, and portable-evidence
verification. No other timed workload or RSS/claim measurement was rerun.

The normal guard and experiment policy tests both pass; the latter verifies all
seven ledger caps accept above-limit claims only in C/O, C retains totals, O
retains zero usage, and operation identity remains shared. The original
reference-caption test, custom timing tests, public parity, diagnostics,
provenance, and evidence-summary tests pass. These are focused tests; this study
does not claim whole-suite validation of unlimited policies.

Early compilation found and corrected a nonexistent Canvas y-range-group method
and constant-assertion lint failures. The initial bee128 small-box probe failed
as documented above. Initial `primary`/`png` timings and the earlier bee128 series
remain in the worktree but are not pooled into the reported final statistics.
The final primary timing pass followed the correction that removes the off-mode
discovery-configuration mutex query; reference additions did not change it.

### Portable evidence in this repository

[`resource-accounting-measurements.csv`](resource-accounting-measurements.csv)
preserves all **858 selected raw nanosecond durations** as 286 rows of three
matched policy observations. `iteration` is the zero-based measured iteration
(warmups excluded); `order` lists policies in actual execution order, making each
original position reconstructible. `normal_ns`, `count_ns`, and `off_ns` retain
the raw integers. The three quoted status fields retain the source log's `ok`
or refusal string, with embedded quotes CSV-escaped; commas/newlines had already
been sanitized by the original timing logger. This is a lossless pivot of the
selected measured rows, not medians or synthesized samples.

Source selection is `final-timing.csv` for the original nine SVG cases,
`bee128-final-timing.csv` for bee128 preserve, `reference*-timing.csv` for the
reference SVGs, and **`png-ledger-timing.csv`** for PNG. The superseded 33 PNG
measurements are excluded. The generated main-checkout copy was compared
byte-for-byte with the Rust exporter output: **33,094 bytes**, SHA-256
`afca64dbe62710e6519b6b3b29f89b469b36ce868d082061f6d70da34f7487ab`.
It is sufficient to recompute all 78 selected timing distributions and all 62
valid paired comparisons with the formulas above.

The report retains resource/RSS tables and selected artifact hashes in the
repository. The full claim/caller/RSS raw logs, font manifest, experimental source,
and reproducer patches remain temporary-worktree evidence; they are **not**
embedded in the repository CSV and could be lost if that worktree is removed.

### Evidence inventory (paths below relative to the detached worktree)

* `final-timing.csv`, `png-ledger-timing.csv`, `bee128-final-timing.csv`, and
  `reference{200,2048,4096}-timing.csv`: all final measured samples, order, and
  success/refusal strings; matching hash CSVs contain every successful mode.
* `summary-ledger.csv`, `paired-ledger.csv`: selected Rust-generated independent
  distributions and correctly paired differences. `summary.csv`, `paired.csv`,
  `png-final-timing.csv`, `png-final-hashes.csv`, and `png-final.log` retain the
  earlier combined-policy PNG evidence without pooling or replacement.
* `portable-measurements.csv`, `portable-verified.log`: lossless raw timing pivot
  and main-checkout byte-equality verification.
* `claims.csv`: resource/file/line leaf frequencies and attempted amounts;
  `claim-totals.csv`, `caller-files.csv`: aggregates;
  `resources.csv`: accepted root usage, shape/discovery counts, and refusals.
* `stages.csv`: separately instrumented lifecycle observations.
* `rss-CASE-MODE-RUN.log`, `rss-renders.log`, `rss-reference-renders.log`,
  `rss-summary.csv`: all fresh-process resource observations and output hashes.
* `provenance*.log`, `png-ledger-gates.log`, `caption-whole-words.log`, other
  validation/test logs, all experimental Rust source, and
  `experiment-*.patch`: reproducible local setup and implementation evidence.

## 9. Interpretation

The observed performance question is narrower than the threshold question.
Bookkeeping has a real, modest cost for these workloads; it is not the dominant
cost of long-caption outlining or PNG conversion. Removing a comparison while
keeping its lock and counter is not a demonstrated optimization. Removing the
claim path saves work but does not remove its admitted fonts, scene payloads,
actual scratch allocations, or caches, and this experiment finds no comparable
RSS reduction.

The thresholds also refuse ordinary complete outputs whose measured unlimited
runtime and RSS are modest on this host. The 999,771-visit example is especially
instructive: one format passes by a tiny margin and another format of the same
scene is refused. Its count describes a policy boundary, not a machine-cost
target. Neither staying below it nor crossing it proves efficiency or excess.

This evidence supports separating decisions about admission policy from
optimizations justified by measured hot paths. It does not choose new limits,
justify a global font cache, establish a portable per-visit cost, or measure
multi-thread contention, other font installations, huge dense swarms, allocator
traffic, or all scientific plot families. Those remain unmeasured.
