# What the 999,771 processing visits actually count

## Answer in brief

**A visit is a caller-defined logical charge, not one CPU operation, one function
call, one glyph, or one unit of elapsed time.** A single `claim_visits(n)` call
adds `n` visits. Producers use different definitions of `n`: scalar counts,
UTF-8 byte counts, actual comparisons, list lengths, and arithmetic work bounds.
Consequently, there is no universal physical “price per visit.”

This report completely partitions the **999,771 visits** of the existing
two-point, 2,048-word reference figure into **25 named action groups**, derived
from **169 actual charging sites**. Nothing is left in an unexplained large
“other” bucket. The portable numerical evidence is
[resource-accounting-visit-breakdown.csv](resource-accounting-visit-breakdown.csv).

The largest **ledger amounts**, not measured CPU hot spots, are:

| Action | Visits | Share of visits | How it is priced |
| --- | ---: | ---: | --- |
| Script/font scalar-item passes | 96,815 | 9.684% | One per scalar/item in each executed pass |
| Fontconfig/filesystem preparation | 88,456 | 8.848% | XML scalar counts, discovery entries, path comparisons, and list passes |
| Text-cache key hashing input | 82,959 | 8.298% | Encoded source/style/owner byte lengths |
| Four text-input/paragraph scans | 77,452 | 7.747% | Number of scalars in each pass |
| Selected-line preparation | 75,505 | 7.552% | Scalars in the selected line |
| Shaper input admission | 75,505 | 7.552% | Run scalars plus bounded shaping context |

The reference contains **2,048 words but 18,432 ASCII scalars/bytes**. The same
source participates in several different passes and trial lines. The entire
figure also includes axes, labels, font discovery, cache lookup, and layout.
Thus 999,771 is neither “999,771 words” nor “999,771 shape calls.”

## Scope: historical prices, current observation-only behavior

The census runs only in the existing detached worktree
`/tmp/opencode/accounting-cost-study`, based on
`582b79946682588e824a790b59362b7ce98ad046`, with the prior experiment's
**CountNoLimit** policy. Its former seven thresholds are disabled while its
original accounting and real rendering algorithms remain active. This pass adds
only diagnostic attribution metadata; it does not change a price, dataset,
layout loop, cache, font selection, or source representation.

The minimal subsequent change from `2017a53` sets the single private
`ENFORCE_RESOURCE_QUOTAS` constant to false at all three central quota predicates.
It retains existing counting, allocation behavior and configuration fields while
disabling all seven ledger quota refusals. The broader drafted test migration and
discovery-storage change were withdrawn; they are not part of this implementation.
The former million-visit threshold is **not enforced**. Checked counter arithmetic
and actual data/font/geometry failures remain separate. See the current override
in [the resource contract](../specs/05_LAYOUT_SYSTEM.md#resources-and-lifecycle).

The tables below preserve the inspected baseline's **999,771 visits, 234,560
records and 56,366,327 byte claims**. The minimal quota-switch change does not
alter their producer prices. These quantities remain distinct dimensions;
matching totals and hashes do not establish performance of a newer build.

Only this report and its new CSV are written in the main checkout. Existing
production changes and reports are preserved. No commit, network access, font
provisioning, dependency change, or new timing campaign is part of this census.

## 1. Definitions and exact invariants

The unchanged fixture is `reference2048`: scatter x/y = `[0.0, 1.0]`, labels
`x`/`y`, caption `"Measured ".repeat(2048)`, 1600×8000 CSS pixels, preserve-text
SVG. These are **whole-root** measurements, not caption-only attribution.

The diagnostic asserts:

| Independent resource | Leaf claim invocations | Sum of charged amounts |
| --- | ---: | ---: |
| Processing visits | **415,556** | **999,771** |
| Planning records | 61,153 | 234,560 |
| Cumulative bytes | 37,832 | 56,366,327 |
| Measurement/intrinsic attempts | 1,152 | 1,152 |
| Cache entries | 477 | 477 |
| Font discovery entries | 141 | 457 |
| Topology slots | 17 | 18 |
| **All leaf claim invocations** | **516,328** | **Not summed across unlike resource units** |

It also asserts 2,403 shape calls, the complete caption's ordered word vector,
and the original preserved SVG SHA-256:

```text
2de3a51ba96dc055f3d409aed790fb1f4e85747eb10f8526b708d6548e13d644
```

For the visit counter specifically:

| Actual amount passed to one leaf claim | Number of claims |
| --- | ---: |
| 0 | 1,248 |
| 1 | 398,827 |
| Greater than 1 | 15,481 |
| **Total** | **415,556** |

The bulk calls account for **600,944 visits**; the unit calls account for
398,827. A zero-amount claim still invokes the counting path in this experiment.
In particular, 1,203 zero-visit calls come from font Unicode storage callbacks
that also pass byte information. Zero logical visits does not mean zero CPU
work or zero allocation.

`claim_calls` below means **leaf ledger invocations**, not function entries.
For example, a loop inside one `itemize_bidi_run` invocation can make many
one-unit claims. The 1,152 measurement/intrinsic attempts in the public
observation vocabulary are another independent counter, not all renderer calls.

## 2. Complete partition and prices

The IDs match the CSV. “Per-claim range” is the observed minimum and maximum
amount in this fixture, not a theoretical bound for every input. Formulas are
the existing producer rules; mixed rules within a row are identified explicitly.

| ID | Charged action | Leaf claims | Visits | Observed per-claim range | Rule / responsible function |
| --- | --- | ---: | ---: | ---: | --- |
| 01 | Input, paragraph and bidi-input scans | 600 | 77,452 | 1…18,432 | Four scalar-count passes in `text::measure_text` |
| 02 | Script/font scalar-item processing | 96,815 | 96,815 | 1 | Five executed item passes in `paragraph_items` and `selected_paragraph_items` |
| 03 | Locate bidi-run endpoints | 63,949 | 63,949 | 1 | One per lower-bound iteration in `itemize_bidi_run` |
| 04 | Build glyph-cluster width prefixes | 43,040 | 62,235 | 1…18,432 | Glyph/cluster/boundary visits plus a cluster-length sort-input charge in `proposal_prefixes` |
| 05 | Break analysis and coordinate mapping | 31,305 | 64,940 | 1…18,432 | Original/reduced scalar passes plus each offset/grapheme comparison |
| 06 | Prepare each selected/trial line | 2,403 | 75,505 | 1…18,432 | `line_span.chars().count()` in `prepare_line` |
| 07 | Admit actual shaper input | 2,403 | 75,505 | 1…18,432 | Run scalars + up to five pre-/five post-context scalars in `shape_retained_run` |
| 08 | Intrinsic/reflow/line-selection control | 7,603 | 7,603 | 1 | Paragraphs, chunks, endpoints, width-run checks and analysis headers; detailed below |
| 09 | Compare cached shaping profiles | 42,488 | 42,488 | 1 | One metadata check and one per source byte actually compared in `compare_profile` |
| 10 | Hash text query keys | 1,409 | 82,959 | 6…18,440 | `8 + source_bytes`, then 6 style bytes; presentation `36 + owner_bytes + optional 8` |
| 11 | Search/update query indices and reuse | 19,902 | 19,902 | 1 | Each binary-search step, candidate and listed cache-result event |
| 12 | Copy source/owner into text cache | 246 | 23,464 | 12…18,448 | Copied source byte count + owner byte count in `cache_text_query` |
| 13 | Literal-byte and font-stack equality | 56,842 | 56,842 | 1 | Stack/header/entry checks and actual byte comparisons in `FontStack::same` / `same_bytes` |
| 14 | Hash the literal font stack | 1,950 | 17,550 | 1…25 | Header 8; family entry 1; named family `9 + name_bytes`; generic family 2 |
| 15 | Copy/share/serialize font stacks | 545 | 27,154 | 1…124 | Let B = family count + named UTF-8 bytes: copy B, CSS 2B, Arc share 1 |
| 16 | Find prepared font pair/face | 15,468 | 15,468 | 1 | Each examined retained pair/face in `prepared_pair` / `find_face` |
| 17 | Fontconfig and filesystem discovery | 848 | 88,456 | 0…13,610 | XML scalar count, directives, path/entry comparisons and sort/dedup input lengths |
| 18 | Font name tables and candidates | 2,764 | 6,636 | 1…74 | Table/name input counts, examined candidates/aliases/scalars, list pass lengths |
| 19 | Unicode normalization of font names | 4,157 | 4,694 | 0…22 | Input counts and decomposition/order/recomposition nodes; storage callbacks charge zero visits |
| 20 | Census required text before fonts | 227 | 37,121 | 0…36,864 | Source scalars + selection units + soft-hyphen flag; generated alphabet length; entry events |
| 21 | Axis ticks, cell assignment and handles | 13,890 | 16,970 | 1…111 | Decimal exponent/factor scans, tick-list lengths, cell events, axis sharing/equality |
| 22 | Exact relative-spacing arithmetic | 5,991 | 35,038 | 1…32 | `max(1,ceil(bits/64))² × operands`, plus separately priced fixed-one metadata/GCD events |
| 23 | Correctly rounded Log10 | 324 | 324 | 1 | Input, power comparison, refinement, series and final-rounding admission sites |
| 24 | Source/scatter preparation | 43 | 100 | 0…22 | Source lengths and individual source/mark/color/panel events |
| 25 | Geometry scratch, root feedback, palette | 344 | 601 | 0…8 | Scratch element counts, list lengths and individual feedback/frame/palette events |
| **Total** | **169 charging sites, 25 groups** | **415,556** | **999,771** | | |

The CSV additionally retains each group's zero/unit/bulk claim frequencies,
`mean_per_claim = visits / claim_calls`, percentage of all visits, symbolic rule,
and every charging function assigned to that group. A mean is a descriptive
mixture of actual claim amounts, **not a new unit price**. The unrounded integers
are authoritative; displayed percentages can differ slightly after rounding.

Rows 24–25 are only 701 visits combined (about 0.0701%), and are fully identified:
row 24 includes 88 source/data/preparation units, 11 scatter mark/intrinsic
events, and one panel lookup. Row 25 is 524 geometry-scratch units, 33 feedback
units, four root-frame units, three palette-resolution units, and 37 scale-value
units. An empty layer source adds a zero-amount invocation. There is no hidden
large residual.

## 3. Concrete meaning of “one action costs how much?”

### The original large text-helper aggregate is now resolved

The previous report could only attribute **245,715 claims / 448,499 visits** to
`layout/text.rs::claim_processing_visits`. Those calls are now exactly rows
**01–06 and 08** above:

```text
77,452 + 96,815 + 63,949 + 62,235 + 64,940 + 75,505 + 7,603 = 448,499
600 + 96,815 + 63,949 + 43,040 + 31,305 + 2,403 + 7,603 = 245,715
```

This is semantic attribution of the charging passes. It does not put all work
in a public `caption()` call or in one particular root round.

* **Four text-input passes:** each charges 19,363 scalar visits over 150 claims:
  required-scalar gathering, paragraph segmentation admission, grapheme-input
  admission, and bidi-input admission. Together: 77,452. The caption contributes
  18,432 scalars to a full pass; other figure text contributes the remainder.
  A batch of 18,432 adds that amount with **one** leaf claim, not 18,432 locks.
* **Five scalar-item passes:** constructing script/font items, resolving the
  remaining script segment, bracket/common-script processing, following-face
  propagation and preceding-face propagation each charge 19,363. Each scalar
  iteration calls the helper with 1, giving 96,815 separate unit claims.
* **Bidi-run boundary lookup:** 63,949 is the number of priced binary-search
  iterations locating the two endpoints of runs. It is not 63,949 bidi runs,
  paragraphs, or glyphs. There are 2,403 shape calls in the complete figure.
* **Cluster prefixes:** three contributions of 19,334 cover visited glyph
  clusters, the cluster-count sort admission, and accumulation into boundaries;
  2,047 intervening boundary steps and 2,186 prefix steps give 62,235. The sort
  charge is its input count, not an observed number of sort comparisons.
* **Break analysis:** the ASCII-reduced analysis source is an internal line-break
  representation; the original text is still shaped and emitted in full. Charges
  are 19,334 for the original scalar pass, 4,998 + 4,998 + 4,859 for reduced
  Unicode passes, 7,184 offset remapping comparisons, and 21,381 + 2,186
  grapheme/intersection comparisons: **64,940** total.
* **Intrinsic/trial control (row 08):** 2,389 intrinsic paragraph/hard-segment/
  preferred-chunk claims, 10 reflow paragraph claims, 271 unwrapped-fit checks,
  2,405 line-selection endpoint/proposal claims, 2,403 exact-width run checks,
  and 125 retained-analysis header events: **7,603**. These are separate from
  the character work performed by the line preparation and shaping they invoke.

### Preparation and shaping have separate prices

`prepare_line` charges the number of scalars in its selected source span before
line bidi processing. `shape_retained_run` charges its run input plus the bounded
pre/post context consumed by the shaper. Both happen to sum to 75,505 here, over
2,403 claims each. They are **two distinct admission sites/passes**, not a wrapper
recorded twice by the diagnostic.

The resulting mean is 31.421 visit units per claim in each group, with observed
amounts from 1 to 18,432. This does not mean “each glyph costs 31.421 visits.” A
single invocation can process a whole paragraph, a hard-width segment, a trial
line, or a small axis label. Rustybuzz's internal GSUB/GPOS work is not counted
by this input charge. Output glyph records and their byte storage are charged
to other ledgers.

### Cache hits and font identity are not free in the visit ledger

The concrete text-key formulas are:

```text
new shaping state:     claim(8 + source UTF-8 bytes), font-stack hash, claim(6)
presentation key:      claim(36 + owner UTF-8 bytes + (preferred_lines ? 8 : 0))
new cached payload:    claim(copied source bytes + owner bytes)
profile comparison:   claim(1), then claim(1) for each source byte actually compared
```

Here the first/last two key stages contribute 20,563, 900 and 61,496 visits,
respectively, for row 10's 82,959. Font-stack hashing is separate row 14, so it
is not counted twice. `compare_profile` contributes 7,312 metadata comparisons
and 35,176 actual byte comparisons. Searches contribute 14,900 lower-bound
steps, 2,090 candidate checks, 2,319 insertion-position steps, 472 reuse-result
checks and 121 source-Arc shares on insertion: row 11's 19,902.

`same_bytes` is shared by font names, text source checks and owner identity.
Row 13 is therefore deliberately **not** labeled “font-name bytes only.” It
contains 1,200 stack/header checks, 3,630 family-entry checks, 3,875 byte-string
header checks and 48,137 byte comparisons. This census identifies the comparison
operation, not the dynamic owner of every use of that shared function.

For a font stack with B = family entries + named-family UTF-8 bytes, this fixture
has B = 62. Stack copies charge 62; CSS's counting and materialization passes
charge 124; a share charges 1. The CSS charge is an input-based convention, not
the resulting SVG string length. Hashing uses its distinct encoded-field rule:
8 for the stack header, 1 for each family, `9 + name_bytes` for a named family,
and 2 for a generic family. Byte counts here are **visit prices for passes**;
separate byte-admission calls still exist.

### Environmental work is part of this root

Of row 17's 88,456 visits, **87,125 come from 37 XML scalar-count claims** in
`parse_bounded_fontconfig`. The observed amounts range from 258 to 13,610 per
XML input. These are host configuration sources, not caption words. The rest
includes admitted directives, examined paths/entries and discovered-file passes.
This is one fresh root-local font discovery, with no global font cache.

The discovery ledger separately records 141 calls / 457 discovery units.
`FontBudget::claim_discovery(n)` also claims `n` visits. One event can therefore
appear in two **different resource dimensions**, without double-counting the
visit leaf itself. It is not a count of 457 font files.

Font-name Unicode work in row 19 is 956 casefold-input units, 956 NFC-input
units, 956 decomposition nodes, 870 ordering steps, and 956 recomposition nodes:
4,694 visits. Its storage callbacks add zero visits on 1,203 invocations.
No expensive combining-mark insertion comparisons were observed in this fixture;
the general algorithm charges each actual such comparison when it occurs.

### Census and exact arithmetic use still other units

`PreparedTextCensus::new_with_stage` charges generated alphabet length and, for
literal text, `scalar_count + selection_unit_count + soft_hyphen_flag`. Selection
units are font-coverage units, not whitespace-separated words. This caption's
large census claim is **36,864**, twice 18,432 in this ASCII case. The complete
row is 36,896 source/selection units + 32 alphabet units + 193 individual census
events = 37,121.

For exact relative spacing, `admit_math` computes:

```text
L = max(1, ceil(bits / 64))
visits = L * L * operands
byte claim = (16 * L + size_of::<BigUint>()) * operands
```

This is a conservative arithmetic/storage convention before a BigUint
expression, not a measurement of its actual limb instructions. Copies, shifts,
GCD, products and divisions are not physically equal-cost operations merely
because this rule prices them similarly. Some metadata/header and small-integer
GCD events instead use a fixed one-unit charge.

The 35,038 spacing visits include:

| Exact-spacing action | Visits |
| --- | ---: |
| Rational reduction: setup, 1,837 GCD-loop admissions, final division | 14,734 |
| Rounding | 9,032 |
| Addition | 2,742 |
| Dyadic conversion | 2,540 |
| Ordering comparisons (`before`) | 2,364 |
| Relative-spacing × local-metric product | 2,166 |
| Pixel inspection/conversion (`pixels`) | 936 |
| Explicitly identified remaining spacing actions | 524 |

That last 524 is retain-header 198, ratio construction/GCD 144, equality 66,
add-quanta 40, subtraction 24, feedback observation 24, source-pixel conversion
14, quanta construction 10, and comparison 4. The raw site table distinguishes
these actions and their observed 1–32 visit amounts.

`numeric.rs` uses a different convention for certified Log10: one visit per
admission event, while scratch bytes depend on precision and temporary count.
This fixture has two input checks, 46 exact-power comparisons, two refinement
entries, six series setups, 258 series-iteration admissions, six final-bound
steps and four rounding admissions: **324**, not a count of all BigUint work.

The axis generator likewise charges **9,641 examined decimal exponents** and
155 step factors, one each. Its remaining tick/cell/handle prices are input
lengths or individual events. A “visit” in that loop, in a UTF-8 equality loop,
and in exact rational arithmetic has no common calibrated CPU duration.

## 4. Records, byte claims and memory are separate questions

This partition concerns visits only. It neither adds 234,560 planning records
to 999,771 nor converts the 56,366,327 cumulative byte claims into visits or RSS.
Typical storage prices remain caller-supplied record counts and checked
`capacity × size_of::<T>()` byte amounts; growth can charge a complete replacement
capacity. They are not a complete census of live allocations or simultaneous
retention. One operation can issue record, byte and visit claims separately.

The static audit's concrete **4n record claims for 2n array elements** finding
belongs to `Labels::resolve_domain_source`: an initial reservation of both arrays
is followed by per-item reservations of both. It is a producer-specific static
finding, not evidence that this caption has 4n unused glyph slots. Likewise,
potential-output reservations and uncharged storage elsewhere cannot be inferred
from the visit total. See
[resource-accounting-static-audit.md](resource-accounting-static-audit.md) for
those coverage and pricing qualifications. This census does not repair or
reclassify them as physical allocation measurements.

## 5. Which metrics answer performance questions properly?

Keep **logical workload**, **accounting frequency**, and **machine cost** distinct:

| Question | Suitable measurement | What the visit total cannot establish |
| --- | --- | --- |
| How long does a user wait? | Wall latency for a specified successful pipeline/output; median and dispersion; enough samples for stable tail estimates | A calibrated latency or an SLA |
| How much CPU is consumed? | User + system CPU time; optionally cycles/instructions on a supported profiler, with host conditions recorded | Instruction count, CPU seconds, or equal-cost operations |
| Where is time spent? | Stage timings or sampling profile for font preparation, text analysis, intrinsics, line trials, capture, serialization, outlining, raster/encoding | A CPU hot-spot ranking from visit percentages |
| How much does bookkeeping cost? | Paired identical-output optimized runs, with counters enabled versus disabled and all actual algorithms/caches preserved | Per-lock nanoseconds from `delta / visits`, especially across unlike producers |
| Why is this input harder? | Separate semantic counts: source scalars/bytes, comparisons, trial endpoints, shape calls/input scalars/output glyphs, collision pairs, arithmetic operand sizes | A universal weighted work score |
| How much memory is needed? | Peak RSS in fresh processes; allocator live/peak/allocated-byte telemetry when separately measured | RSS, peak live payload, or exact allocator traffic from cumulative byte admissions |
| Did performance work preserve the result? | Exact SVG bytes/dimensions, complete content, encoded/decoded raster hashes, font identity and policy | Correctness or completeness from low counts alone |

There is **no new timing measurement here**. Claim maps, amount histograms and
caller metadata substantially alter diagnostic overhead. The census test's
printed process duration is not a renderer benchmark or a per-action price.
Nor were font/Unicode helpers separately timed, so the actual nanoseconds of
one comparison, one scalar pass, one shape, or one rational operation are unknown.

The prior optimized A/B study remains the relevant measured baseline:

* Exact reference2048 preserve SVG: normal median 101.158 ms, count-without-limits
  102.269 ms, bookkeeping-off 96.978 ms. The median **paired** N−O difference is
  3.745 ms / 3.835%; C−O is 4.616 ms / 4.818%.
* These compare whole pipelines, including fresh font discovery. They are not
  times for one caption pass, one mutex, or one visit. Normal/count differences
  are noisy and do not demonstrate a benefit from removing only the comparison.
* Old reference2048 fresh-process RSS is about 49,000 KiB. This is whole-process
  memory, not 56,366,327 bytes of live payload.

Full conditions, the ledger-only PNG correction, and raw timing data are in
[resource-accounting-performance.md](resource-accounting-performance.md) and
[resource-accounting-measurements.csv](resource-accounting-measurements.csv).
Their eleven paired repetitions do not by themselves establish a precise p95
or a portable percentage. No conclusion about current-production latency or
512-point swarm speed is extrapolated from this census or those old samples.

For future performance work, preserve the existing results and measure the
relevant stage on identical workloads. If an action-level cost is needed,
measure that action with its real input size/context/cache state and use bounded
instrumentation or a sampling profiler; keep such profiling separate from the
primary end-to-end timing run. Do not optimize merely toward 999,999 visits or
invent conversion weights that pretend every visit is equivalent machine work.

## 6. How attribution was obtained and verified

The existing experiment already instruments the seven ledger leaves once and
propagates locations through canonical font wrappers. The extension adds
`#[cfg_attr(all(test, feature = "study-claims"), track_caller)]` to the common
text visit/Unicode/record bridges, source visit bridge, numerical admission, and
relative-spacing admission helpers.

`FnMut` Unicode callbacks cannot forward `track_caller` automatically through
their closure body. A diagnostic-only scoped caller-location guard therefore
wraps the actual callback invocation in font NFC/casefold, layout line breaking,
and offset/grapheme mapping. It restores its prior location on drop. Without
`study-claims`, the macro expands to the original callback alone. Primary code
has no new histogram or metadata operations. The shapes, reservations, checks,
loops and amounts are unchanged in both builds.

The instrumented census collects a histogram of **amount → number of leaf
calls**, keyed by resource/file/line. The site table is reconciled with the
existing leaf totals. Enclosing function names are resolved from the source
after rendering, reviewed against the charging expressions, and assigned by an
explicit classifier; an unknown site/function fails instead of silently joining
an “other” bucket. The partition asserts the exact 999,771 visit sum and 415,556
visit-call sum, including zero calls. Wrapper and leaf calls are not both counted.

This is a **charging-site** attribution, not a full dynamic call-stack or
caption-owner profile. Shared routines such as `same_bytes` remain shared-operation
categories. The complete figure includes all root preparation and its other
texts. No owner-local or phase-local attribution is implied where it was not
instrumented.

Commands, all in `/tmp/opencode/accounting-cost-study`:

```bash
cargo fmt --all
cargo clippy --locked --offline --all-targets --features study-claims -- -D warnings
cargo clippy --locked --offline --all-targets -- -D warnings
cargo test --release --locked --offline --features study-claims --lib accounting_operation_breakdown -- --ignored --nocapture --test-threads=1
cargo test --release --locked --offline --features study-claims --lib accounting_operation_partition -- --ignored --nocapture --test-threads=1
STUDY_VERIFY_OPERATION_CSV=/home/robot-machete/projects/lichtung-plot/docs/resource-accounting-visit-breakdown.csv cargo test --release --locked --offline --features study-claims --lib accounting_operation_partition -- --ignored --nocapture --test-threads=1
STUDY_PROVENANCE=provenance-operation cargo test --release --locked --offline --features study-claims --lib accounting_cost_provenance -- --ignored --nocapture --test-threads=1
```

The first census pass exposed that a location captured *inside* the TLS closure
identified the guard itself. Capturing it before entering that closure fixed
the attribution. It did not change any amounts. The completed pass and explicit
partition have the exact original totals/hash. Strict Clippy is used for both
builds; no full suite, main-checkout build, or performance series was run.

Retained temporary evidence:

* `operation-sites.csv`: all resource sites, enclosing functions, calls, units,
  min/max amounts, and zero/unit/bulk frequencies.
* `operation-amounts.csv`: full amount histogram at each site.
* `operation-visit-sites.csv`: all 169 visit sites assigned to the 25 groups.
* `operation-visit-breakdown.csv`: the portable group table, checked byte-for-byte
  against the main-checkout CSV; SHA-256
  `0035fc5dbf5b68dfec990445ee1cce8b5dee902358832a0cede9b1510f63ed2d`.
* `operation-breakdown.log`, `operation-partition.log`, `operation-verified.log`,
  `operation-lint*.log`, `provenance-operation.log`, and the operation experiment
  patches: verification, source/compiler/font provenance, and reproducer.

The current complete temporary patch set is `experiment-operation-tracked.patch`,
`experiment-operation-policy.patch`, and `experiment-operation-driver.patch`.
The latter two add the private experimental modules to the tracked-file patch.
The recorded diagnostic policy SHA-256 is
`374d843e9a5e5743131ecfd3126737777866b901d2dadb32e1bbab80c4362bd9`;
the driver/partition SHA-256 is
`190bb04be699e4e9e465addda032fb3bec1b0d6e73e74733d3df0e95ba006302`.
`provenance-operation.log` additionally hashes every modified attribution bridge,
the locked dependencies, and the host's font files.

The main CSV preserves the complete grouped numerical partition and formulas.
The full per-site histograms and temporary Rust reproducer are not copied into
the production source tree. Earlier A/B CSVs, logs, manifests, and patches remain
historical evidence and are not overwritten by this census.
