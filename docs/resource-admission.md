# Resource Admission And Release Evidence

## Current Contract

PF4 uses one cumulative processing-visits ledger for the root's font preparation,
text planning, candidate/list traversal, and collision tests. The root creates
the shared handles before preparation; environment and resource clones retain
the same ledger. `share_generation_resources` accepts the existing ledger and
refuses a different ledger without changing either owner's consumption. It does
not reset a started operation. Separate publications from one `Output` reuse the
scene; independent Plot operations still prepare their own fonts.

| Resource | Inclusive Limit |
| --- | ---: |
| Actual processing visits, shared across font and text | 1,000,000 |
| Cumulative retained planning records | 1,000,000 |
| Prepared structural topology slots | 1,000,000 |
| Measurement/intrinsic call attempts and root cache entries | 1,000,000 each |
| Font discovery entries | 1,000,000 |
| Cumulative font/source/scratch byte admission | 536,870,912 |
| Root feedback rounds and guide placement trials | 8 each |
| Optional exact text-cache insertion stop | 256 entries |
| PNG axis / pixels / parser-input bytes | 32,768 / 100,000,000 / 67,108,864 |

Processing visits count actual scalar inputs to variable passes, normalization
input/decomposition nodes, each actual preceding comparison in quadratic mark
ordering, candidate/unit/endpoint/list visits, and collision pair tests. Fixed
Unicode table searches and the old 64/8/256 per-rule allowances are not priced.
Neither hashing, copying, storage reservation, nor materialization receives a
processing surcharge. Actual copied records and bytes still require admission.
This exemption applies to hashing key bytes, not to traversal of variable
root-owned lists. Guide-owner lookups and retained-face lookups for shaping,
metrics, and outlining admit each examined entry before comparison. Hits stop
charging at the match; misses charge the entire examined list. Retained files
are then accessed directly by the face's stored file ordinal, without a second
linear file search. No fixed Unicode-table comparison price was introduced.

Proposal-prefix and cluster vectors admit both records and checked byte capacity
through the existing text/Unicode reservation helpers. Positioned geometry text
uses the existing record-and-byte reservation helper, before clip processing or
retaining the plan. Growth admits the complete replacement capacity, with no
processing charge for allocation or copying.

Canonical Rustybuzz input is admitted before buffer construction and shaping.
Locked Rustybuzz 0.20.1 consumes at most five trailing pre-context scalars and
five leading post-context scalars; only those context scalars are included.
Actual output glyph records and bytes are admitted in bulk before the renderer
copies glyphs or probes their ink. This is not a second output-processing price.
The Unicode callback remains private and takes only `(bytes, visits)`.

Child-cache keys retain only the computed values that parent coordination may
replace, with fallible record/byte admission using the existing environment.
They do not copy entire Plots or create another quota. Value equality, including
the existing exact range-bit checks, remains the fallback. Independent nested
capture/replay reservations retain their own record admission; text processing
continues through the owning environment.

Refusals remain typed and terminal. Quota exhaustion does not omit content, alter
font selection, shorten a line, or trigger a smaller fallback representation.
Physically inadequate boxes still produce fit failures. PNG gates remain local
to PNG output. No global cache, network access, or font provisioning was added.

## Verification

Evidence was collected on the existing Linux workspace after committed PF2/PF3
`5c0acdf` and PF6 `23db99f`, with the same locked dependencies and installed fonts.
PF4 implementation and PF5 evaluation/report are complete; the primary's final
`just verify` passed in `/tmp/opencode/pf4-final-verify.log`.

- Strict `cargo clippy --locked --offline --all-targets -- -D warnings` passed.
- Full `cargo test --locked --offline --lib`: 685 passed, 8 opt-in tests ignored.
- Final integrated verification: all 441 integration tests including five
  snapshots passed, rustdoc passed, and package verification passed with 202
  files, 5.6 MiB uncompressed and 1.2 MiB compressed.
- All four focused independent-review regressions passed: proposal storage,
  geometry text storage, guide lookup traversal, and retained-face traversal.
- All six official Unicode tests passed with
  `LICHTUNG_UNICODE16_SOURCE_DIR=/tmp/opencode/UCD-16.0.0` and the `official` filter
  plus `--ignored`. This includes contracted/expanded line-break comparisons.
- Tests cover concurrent font/text claims, inclusive million-visit and record
  boundaries, unchanged consumption on refused rebinding, actual ordering
  comparisons, collision pairs, finite shaping context, actual GSUB output
  records, and byte refusal before first-party glyph retention.
- Proposal storage exercises byte allowances of zero, prefix-size minus one,
  exact prefix size, total prefix/cluster size minus one, and exact total size.
  Geometry byte exhaustion leaves text length and capacity at zero and performs
  no clip-processing visits. Guide traversal tests first/last/missing owners and
  refusal without trial/event mutation or allocation. Shaping, metrics, and
  outlining test first/last/missing faces and refusal before engine invocation,
  glyph retention, or outline callbacks. Record and byte consumption remain
  unchanged when a face lookup is refused. The guide ordinal/miss lookup routes
  use the same fallible traversal policy.
- The 2,048-word caption also publishes preserve-text SVG in an adequate finite
  1,600 by 8,000 CSS-pixel box. The prior required synthetic-score refusal was
  removed; layout algorithms and Unicode expected fixtures were not changed.

Independent review `ses_f793c53d9ffedekZJtzvGR365p` rechecked and closed both
byte-admission and variable-list-visit findings, with no new blocker. The final
integrated suite and separately invoked six official Unicode tests are passing
evidence, not commands run by this docs-only closure. No corpus import or new
history rewrite is part of PF4. PF1 proof `51d3b23` records the completed all-ref
cleanup with repair trees preserved; mapped clean baseline `65f3cf4` and consumer
tip `e8764ae` replace historical `77d8bf2` and `b6f8321`. Old IDs remain historical
via the full mapping in spec 91. No push occurred.

The intended primary closure commit subject is
`refactor(layout): use coarse shared processing admission`; no PF4 commit hash
is assigned here before the primary commits.

## Release Measurements

Commands:

```bash
cargo test --release --locked --offline --lib publication_efficiency_measurements -- --ignored --nocapture --test-threads=1
cargo test --release --locked --offline --lib typography_performance_evidence_and_root_cache_budgets -- --nocapture --test-threads=1
```

Publication measurements use three runs, no warmup, uncontrolled OS caches,
outlined SVG and source-size balanced PNG. Times are elapsed wall time, not CPU
time. Admission metrics include preparation and the complete operation and are
identical across all three runs. The `separate10` row sums ten independent root
budgets, so its aggregate visits may exceed one million; no individual root does.

| Case | Min Wall ms | Median Wall ms | Records | Visits | Admitted Bytes | Shape Calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| scatter100 | 36.991 | 47.258 | 1,374 | 102,419 | 28,181,683 | 9 |
| long_label_bar | 31.308 | 31.837 | 1,661 | 105,689 | 26,885,163 | 11 |
| caption200_1para | 82.001 | 82.297 | 18,202 | 167,595 | 30,044,088 | 35 |
| caption200_3para | 83.227 | 85.226 | 17,900 | 167,254 | 30,298,101 | 37 |
| facets2x2 | 44.519 | 44.564 | 6,841 | 111,997 | 41,385,622 | 13 |
| mixed_canvas | 46.005 | 46.738 | 7,414 | 113,788 | 33,287,680 | 79 |
| matrix4 | 70.301 | 71.511 | 7,132 | 105,245 | 155,184,003 | 13 |
| separate10 | 381.875 | 382.686 | 13,740 | 1,024,190 | 281,816,830 | 90 |
| reuse10 | 148.973 | 149.671 | 1,374 | 102,419 | 28,181,683 | 9 |

All nine sets of outlined-SVG SHA-256, PNG SHA-256, and decoded RGBA8 SHA-256
match `/tmp/opencode/publication-efficiency-hashes.csv` exactly. All three runs
also match within each case. The baseline CSV/log files were left untouched.
Raw post-review evidence is in `/tmp/opencode/pf4-review-publication-efficiency.log`
and `/tmp/opencode/pf4-review-typography.log`; stage timings remain available
there. Earlier PF4 logs are retained but their admission counts predate the
review corrections and are superseded by these measurements.

The original publication baseline predates PF2/PF3/PF6 as well as PF4. These wall
times are not an isolated causal measurement of PF4. No speedup is inferred from
smaller accounting units, and these admitted-byte counts are not allocator or
RSS measurements. Fresh font-environment probes were approximately 18.3-18.4
ms in this earlier post-review run; the final rerun below measured a 21.022727 ms
median. No cross-operation/global font cache was introduced.

The separate text benchmark uses 768 CSS-pixel cold width, then 736 for reflow.
For 2,048 occurrences of `Measured `, it retains all 18,432 glyphs on 228 lines.

| Phase | Wall ms, One Sample | New Records | New Visits | New Admitted Bytes |
| --- | ---: | ---: | ---: | ---: |
| Cold | 47.795 | 174,961 | 515,677 | 21,652,693 |
| Exact hit | 0.009 | 0 | 2 | 0 |
| Changed-width reflow | 28.991 | 78,652 | 96,396 | 8,453,689 |
| Return-width hit | 0.009 | 0 | 3 | 0 |

Including initial font preparation, the final cumulative visits are **709,818**
and records **253,693**, both below one million. Cold/reflow actual shaping calls
are 456/455; cache hits perform no shaping. These counts demonstrate admission
and reuse, not a claim that the underlying quality algorithms do less work.

## Final Before/After Rerun

This measurement checkpoint follows PF2/PF3 `5c0acdf` and PF6 `23db99f` with
the reviewed PF4 worktree. Together with the final primary verification and
independent recheck above, it supplies closure evidence for PF4/PF5. The unchanged
publication command above ran all nine identical
inputs three times in the original order. Both baseline and final runs used
Rust 1.96.0 (`ac68faa20`, LLVM 22.1.2), optimized release, Linux x86-64 on an AMD
Ryzen 9 7950X3D under the Microsoft hypervisor, and one test thread. Installed
fonts and locked dependencies were not changed. No OS-cache, CPU-frequency,
scheduling, or warm-disk control is claimed.

| Case | Baseline Min ms | Baseline Median ms | Final Min ms | Final Median ms |
| --- | ---: | ---: | ---: | ---: |
| scatter100 | 36.898846 | 46.408242 | 34.946599 | 45.269001 |
| long_label_bar | 30.846397 | 31.785036 | 30.214369 | 30.478193 |
| caption200_1para | 81.361248 | 81.484864 | 79.588059 | 79.759550 |
| caption200_3para | 82.257938 | 82.908719 | 80.393906 | 80.603826 |
| facets2x2 | 45.415668 | 46.237701 | 44.761454 | 46.148677 |
| mixed_canvas | 46.197478 | 47.232104 | 47.125323 | 47.562766 |
| matrix4 | 72.373634 | 72.536659 | 68.940187 | 69.893880 |
| separate10 | 385.528764 | 386.541217 | 375.550547 | 471.785355 |
| reuse10 | 149.227051 | 150.995173 | 203.142728 | 205.149185 |

These are noisy elapsed-wall observations, not an isolated causal measurement
of any PF item or evidence of a twofold speedup from changed admission units.
No samples were discarded: the third three-paragraph sample took 1,312.165787 ms,
including 1,231.117979 ms in publication. Later separate/reuse samples were also
slower. The source of this variability was not isolated.

Every final repetition had identical counters and admission totals within its
case; records, visits, and bytes match the preceding post-review table. Root
rounds changed from 2 to 1 for scatter/bar/captions/Matrix/reuse, from 3 to 2 for
facets/Canvas, and from 20 to 10 for ten separate operations. Producer calls
changed from 2 to 1 per root (20 to 10 for `separate10`). Shape counts are unchanged.
The separate case still creates ten roots; reuse still creates one root and
publishes ten SVG/PNG terminal pairs.

All **27 of 27** baseline SHA256 comparisons passed: SVG, PNG, and decoded RGBA8
for all nine cases. The test also verified equality across all three repetitions.
The original `publication-efficiency-baseline.log`, `publication-efficiency-summary.csv`,
and `publication-efficiency-hashes.csv` in `/tmp/opencode` were left byte-for-byte
unchanged. Final evidence is `/tmp/opencode/publication-efficiency-final.log` and
the shell-extracted `/tmp/opencode/publication-efficiency-final.csv`. The CSV
contains all raw stage samples, 90 stage min/median rows, per-run root/producer/
shape and admission counters, font samples and summaries, and explicit baseline
hash comparisons. Preparation includes fonts; null validation, capture, SVG
replay/serialization, outlining, PNG rasterization/encoding, and publication are
separate observed boundaries, not a fabricated complete decomposition of wall time.

### PF5 Evaluation

The isolated probe creates a fresh font environment for each repetition after
the publication workloads, then queries two texts and repeats the second text.
It is not a cold-filesystem benchmark. The first scatter in the final process
took 48.107645 ms, of which 29.806404 ms was preparation including fonts, not
isolated font setup.

| Probe | Baseline Min ms | Baseline Median ms | Final Min ms | Final Median ms |
| --- | ---: | ---: | ---: | ---: |
| Fresh environment | 18.499067 | 18.510384 | 20.402276 | 21.022727 |
| First query after preparation | 0.203289 | 0.203697 | 0.262825 | 0.286900 |
| New text query | 0.092163 | 0.092979 | 0.113981 | 0.145248 |
| Repeated query | 0.000917 | 0.000918 | 0.000999 | 0.001099 |

Final evaluation disposition: retain operation-local preparation and existing same-Output
reuse. Do not add a cross-operation/global font cache or change the agreed font
freshness/lifetime policy. These three-sample timings do not justify such a change.
PF5 is complete as evaluation/reporting, not a cache implementation. Source
parameters, content, shaping, and PF7's current line extension remain unchanged;
any removal of extension stays deferred to DF10 at T7 closure.

### Admission Controls

The existing release typography test freshly accepted the 2,048-word workload:
228 lines, all 18,432 glyphs, 515,677 cold visits and 96,396 reflow visits, with
709,818 cumulative visits including preparation and hits. Cold/reflow shape
calls were 456/455; exact/return hits used 2/3 visits and zero shaping. Single
elapsed samples were 66.648/41.234 ms for cold/reflow and 0.009/0.009 ms for hits;
these are not three-run medians. The public-box test also successfully published
all 2,048 words in a 1,600 by 8,000 preserve-text SVG.

Negative controls also passed in release: the existing
`adversarial_mark_ordering_refuses_before_quadratic_work_finishes` test refuses
2,000 adversarial combining marks with a typed root-processing-visits limit;
`root_visits_are_atomic_shared_and_cannot_be_rebound_after_preparation` admits
exactly 1,000,000 shared visits, refuses the next visit without increasing usage,
and rejects rebinding. No elapsed-time threshold was added. Positive acceptance
of 2,048 ordinary words does not imply acceptance of every similarly sized input.
Raw positive evidence is in `/tmp/opencode/publication-efficiency-final-typography.log`
and `/tmp/opencode/publication-efficiency-final-large-caption.log`. Strict locked,
offline all-targets Clippy passed. That measurement pass made no production edits
or commit and did not run `just verify`; the subsequent primary final suite above
passed and closes verification. This docs-only closure runs no tests/builds,
rewrites no history, and makes no commit or push.
