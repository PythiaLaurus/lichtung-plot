# Local-Search Measurements

This is a measured cost and coverage report for T5a3q, not an optimization.
The final aggregate release run in
`/tmp/opencode/t5a3q-all.log` passed all 10 requested ignored measurement tests
(0 failed, 702 filtered out; 6.61 seconds of test execution). The subsequent
`just verify` passed strict Clippy, 695 library tests (17 ignored), 441 integration
tests including five unchanged snapshots, API documentation, and package
verification (207 files). The verification log is `/tmp/opencode/t5a3q-verify.log`.

## T5a5 Audit (2026-09-11)

### Final Benchmark Closure (2026-09-11)

The completed implementation's final release run,
`/tmp/opencode/t5a5-final-benchmarks.log`, passes **all 10 ignored measurement
groups**, zero failed, zero ignored, 804 filtered out (8.41 seconds of test
execution). It uses the ignored release command recorded below. The final
`just verify` log, `/tmp/opencode/t5a5-final-verify.log`, passes formatting,
strict locked/offline all-target Clippy, **797 library tests (17 ignored)**,
**442 integration tests including six snapshots**, warning-denied API docs and
package verification (**219 files**). All ten normal benchmark contracts pass in
that full run. These are existing final-run results, not tests rerun for this
documentation-only closure.

The final source includes the Density configured-display-domain correction,
admission of owned measurement-time axis copies, and scoped terminal Tick misses
through Marginal/Matrix dispatch. Their regression tests pass in the final gate.
The reflow/facet fixes and output-equality gates described below remain closed.
All three final long-Heatmap Elide samples assert the same 400-prefix attribution:
`[(0, 200, false), (1, 200, true)]`. This does not explain the former 500 golden
or establish a speedup from it.

T5a5 is COMPLETE in `specs/91_IMPLEMENTATION_PLAN.md`; there is no known unresolved
T5a5 benchmark acceptance item. The preparation-run and superseded audit notes
below describe earlier runs, not the final checkout's status. All historical
**T5a3q tables, timings and hashes remain unchanged**: current adaptive ticks,
assigned cells, gutters and root paths are not an output-equivalent performance
baseline for T5a3q. The 8.41-second duration is a local observation, not a speedup
claim. Whole-family interior extraction and deferred adapters remain T5a6a work;
this closure does not expand scope or claim a commit.

### Superseding Preparation Results (2026-09-11)

This note supersedes the earlier T5a5 audit observations below, including its
unresolved reflow/facet defects, Heatmap mismatch, and failing run results.
Those observations and all T5a3q tables/hashes are retained as history, not
current-policy profiles. This benchmark-only completion changes no production
code, fixture dimensions, label sources, counter definitions, or output gates.

The unchanged long Heatmap Elide fixture now asserts **400 actual prefix
candidates**, attributed to root ordinal 0: **200, unsuccessful**, and ordinal
1: **200, successful**. Their sum is asserted against the aggregate counter.
The normal contract and all three release samples report this same attribution
(`heatmap_prefix_roots` and `paired_root,heatmap_long_elide` in the logs below).
`src/plots/heatmap.rs::fitted_label` increments `HeatmapPrefixTrial` for each
grapheme-prefix-plus-ellipsis candidate before its width check, including the
first rejected extension. Full-label and ellipsis-only width checks are not
prefix trials; cached text measurements still count candidate visits.
`src/orchestrator/measurements.rs::count` attributes each increment to the active
root round. Neither definition was changed to obtain 400.

**Golden-count change explanation:** 500 was a previous expectation, not a
current measured profile. No recorded root attribution here establishes why
that expectation was 500, so this update does not invent a historical cause or
claim an optimization from 500 to 400. The historical T5a3q table's 300 is also
left untouched. The new golden records current defined work under the current
axis/feedback policy. All six elided labels must remain distinct; Wrap retains
complete labels; shared-prefix Elide still rejects ambiguity at 96 candidates
and publishes no artifacts.

The production reflow and selected-edge defects reported in the earlier audit
are now fixed in the tested checkout. The original `historical_reflow_640x400`
public path selects compact right in two rounds, not top in one round:
preferred right is rejected guide-only in each round; compact right is first
provisional, then accepted. It records 4 guide trials, 2 body checks, and 180
text proposals. Auto and explicit right now have identical outlined SVG,
preserved SVG, encoded PNG, and decoded pixels. The separate 630x480
`compact_right` control still has 1 round, 2 trials, 2 checks, and 11 proposals;
it is not substituted for or compared for equality with the historical fixture.

Pressured facets now select right without a provisional top detour, and explicit
right succeeds with identical output. Both policies record 3 rounds, 4 trials,
6 checks, and 0 child hits / 24 misses / 0 replays. The normal
`measured_gutter_reflow_preserves_complete_frames_without_cache` contract also
checks both reflow and pressured facets with child reuse disabled, preserving
output equality, complete frames, minimum body size, and all marks. The seeded
15 px gutter control remains separate from the natural public lifecycle.

| Current fixture | Root rounds | Guide trials / body checks | Child hits / misses / replays |
| --- | ---: | ---: | ---: |
| Adequate Canvas/facets | 2 | 2 / 4 | 0 / 16 / 0 |
| Pressured Canvas, selected top | 3 | 7 / 6 | 0 / 24 / 0 |
| Pressured facets, Auto/explicit right | 3 | 4 / 6 | 0 / 24 / 0 |

| Current text fixture | Root rounds | Prefix trials | Exact trials / shapes |
| --- | ---: | ---: | ---: |
| Short Heatmap Wrap/Elide | 2 | 0 | 14 / 14 |
| Long Heatmap Wrap | 2 | 0 | 67 / 67 |
| Long Heatmap Elide | 2 | 400 | 115 / 115 |
| Shared-prefix Heatmap Wrap | 2 | 0 | 89 / 89 |
| Shared-prefix Heatmap Elide, failure | 1 | 96 | 29 / 29 |

Matrix retains 1 root round, 3 local passes (6 for tight long labels), and
12 tick sources / 12 lists. At 1200x1200 both label variants have 72 metric
lookups and 9 shapes; at 600x600 both have 36 lookups, with 6 shapes for short
labels and 90 for long labels. Sparse/dense inside placement retains 2 rounds,
28/2005 geometry scans, 2 annotation comparisons, and no automatic corner
candidates. Controlled child reuse remains 2 hits / 2 misses / 1 replay.

Completion runs use the three audit commands below, with full output in:

- `/tmp/opencode/t5a5-preparation-benchmarks-normal.log`
- `/tmp/opencode/t5a5-preparation-benchmarks-release.log`
- `/tmp/opencode/t5a5-preparation-benchmarks-clippy.log`

The normal run passed **10 contracts, 0 failed, 10 ignored, 767 filtered out**
(46.17 seconds). The release run passed **all 10 ignored groups, 0 failed,
0 ignored, 777 filtered out** (7.97 seconds): significance search; font observer
overhead, synthetic coverage, and system preparation; inside, outside, and
shared legends; publication efficiency; paragraph search; specialized text
search. These durations are local execution observations, not comparable
performance claims. Strict all-target Clippy passed with warnings denied
(5.24 seconds), after adding a scoped function-length expectation to keep the
Heatmap fixture and its correctness assertions together. The changed Rust file
passes `rustfmt --check`, and the owned paths pass `git diff --check`.
No `just verify` or commit was requested or performed.

### Earlier Audit Observations (Superseded)

The tables and hash comparisons below remain the historical **T5a3q** report.
They are not current-policy baselines. T5a5 intentionally changes adaptive tick
sources, assigned label cells, and measured gutter feedback. Running the command
below on T5a5 measures that checkout; reproducing the historical tables requires
the T5a3q source state and its font/host conditions. No new timings or hashes have
been substituted into those tables.

The benchmark-only audit preserves the original 640x400, 18-entry/36-mark reflow
input as `historical_reflow_640x400`. Its current public path is one root round:
preferred right fails guide-only, compact right fails during body planning, and
preferred top succeeds (3 guide trials, 2 body checks, 90 text proposals).
Auto and explicit top have byte-identical outlined SVG, preserved SVG, PNG, and
decoded pixels. Explicit right fails and publishes nothing. This is an observed
behavior, **not a claim that compact right cannot legitimately fit**.

The audit found a production gutter-feedback defect in
`src/decorations/plan.rs`, at the `next_gutters` / `requirements.assign_fit`
check in `plan_panel_attempt`: the previously measured compact guide leaves an
80 px body. Its native axes require a 89.26953125 px left gutter, which requests
15 px beyond the intrinsic reservation after the owned 4 px gap. The old guide
is checked against the new gutter and rejected before being remeasured with the
changed assignment. `historical_reflow_measured_gutter_control` seeds that 15 px
feedback in a controlled first round, then calls the production planner on the
unchanged fixture. Preferred right is rejected guide-only and compact right is
accepted with an 80 px body and the same measured gutter requirement. This
demonstrates a self-consistent fitting plan, not public output equality or a
measurement of the natural public lifecycle. Production was not changed here.

A separate public `compact_right` control uses **630x480, two long entries/four
marks, and a 20-character `M` x-axis title**. Preferred right fails the body check;
compact right succeeds in one root round (2 trials, 2 checks, 11 text proposals).
The height accommodates wrapped entries and the width puts the complete title
between the preferred and compact body allocations. Probes at 625x480 selected
top, at 635x480 selected compact right, and at 640x480 selected preferred right.
This is a different workload and rejection phase, not an equivalent replacement
for T5a3q's 18-entry guide-only reflow case. Its Auto/explicit-right equality is
checked independently; historical/new fixtures are never compared for equality.

Other current-policy observations are asserted rather than relaxed to bounds:

| Fixture | Root rounds | Guide trials / body checks | Child hits / misses / replays |
| --- | ---: | ---: | ---: |
| Adequate Canvas/facets | 2 | 2 / 4 | 0 / 16 / 0 |
| Pressured Canvas, selected top | 3 | 7 / 6 | 0 / 24 / 0 |
| Pressured facets, Auto selected right | 4 | 6 / 8 | 0 / 32 / 0 |

Sparse inside placement now has two rounds for gutter feedback, 28 geometry
scans and 2 annotation comparisons; its panel left edge moves from 79.2 to
115.2 px. Dense inside placement retains two rounds, 2005 scans and 2 comparisons.
Both still have zero automatic corner candidates and retain containment, mark
clearance, and the caller annotation. Controlled child reuse still asserts
2 hits, 2 misses, 1 replay, separately from end-to-end cache utilization.
Matrix local passes remain 3 (6 for tight long labels), but native tick counts
are 12 sources / 12 lists, with 36 metric lookups at 600x600 and 72 at 1200x1200.
The long Heatmap Elide contract still expects 500 prefixes, but the current
unchanged 1200x500 fixture measures 400: 200 in each of two root rounds, with only
the second round successful. Per-round counters expose this unresolved mismatch;
no historical/current root-path comparison yet justifies replacing 500. The
ambiguous Elide contract expects 96 and failure without artifacts. Normal text
contracts cover the Heatmap and Matrix variants but the combined contract
currently stops at the long Elide mismatch. Font, paragraph-cache, significance, lifecycle, and artifact
contracts retain their existing semantic checks.

**Unresolved selected-edge defect:** pressured facets at 560x360 finish on right
under Auto after a provisional top round, but explicit right fails with
`available panel size is 70.7x222.8`. Auto's trace is provisional right; rejected
preferred/compact right followed by provisional top; provisional right; accepted
right. The ordinary `shared_legend_selected_edge_contract` and ignored
`shared_legend_measurements` retain their success/equality gates and expose this
failure. No output-equivalent pressured-facet pair or speedup is claimed for
T5a5. The historical T5a3q top pair below is unchanged.

Audit commands (no `just verify` or commit):

```sh
cargo test --locked --offline --lib benchmarks -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1
cargo clippy --locked --offline --all-targets -- -D warnings
```

Final audit results: **8 normal contracts passed, 1 failed, 10 ignored**;
**9 ignored release groups passed, 1 failed**; strict all-target Clippy passed.
Both failures are the pressured-facet explicit-right defect above, not relaxed
counter checks. Logs are `/tmp/opencode/t5a5-benchmarks-normal.log`,
`/tmp/opencode/t5a5-benchmarks-release.log`, and
`/tmp/opencode/t5a5-benchmarks-clippy.log`. Concurrent production edits caused
transient compilation failures during probing; the final runs compiled and
reproduced the same selected-edge failure in debug and release. These runs do
not establish performance comparability with the historical measurements.

## Conditions And Sources

Reproduction command from `specs/91_IMPLEMENTATION_PLAN.md`, T5a3q:

```sh
cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1
```

- Build: optimized release test binary, `lichtung-plot` 0.1.0; T5a3q toolchain
  `rustc 1.96.0 (ac68faa20 2026-05-25)`, LLVM 22.1.2,
  `x86_64-unknown-linux-gnu`. No custom release profile is declared.
- Host inspected alongside the report: AMD Ryzen 9 7950X3D, 16 cores/32 logical
  CPUs, 47 GiB visible RAM; WSL2 Linux `6.18.33.2-microsoft-standard-WSL2`,
  Microsoft hypervisor. No CPU pinning, frequency isolation, or controlled
  scheduling is established. OS font/file caches are uncontrolled.
- Public fixtures use `Theme::Publication` and system discovery, not a bundled
  benchmark font. The stack is `Noto Sans, Noto Sans CJK SC, DejaVu Sans,
  Liberation Sans, Arial, sans-serif`. The system probe recorded 457 discovery
  entries and 9 candidate faces. `fc-match "Noto Sans"` currently reports DejaVu
  Sans Book; this is host context, not proof of every renderer-selected face.
- Each paired case has three deterministic samples, interleaved with alternating
  policy order. Outside/facet Auto comparisons have an untimed selection probe
  before the pair. Other local measurements have three fresh environments/themes
  with first and cached-repeat phases; there is no general warmup protocol.
- End-to-end time starts before construction and ends after public outlined SVG
  publication plus source-balanced PNG preview publication. Temporary-directory
  setup, subsequent preserved-text replay, reads, hashes, PNG decoding, and
  semantic assertions are outside timing. Expected failures produce no artifacts
  and are summarized separately.
- All instrumentation is opt-in, crate-private, `cfg(test)` observation. There
  are no global-cache changes or changes to placement priority, wrapping, ticks,
  font policy, or limits in the measurement scope.

Sources are `tests/benchmarks/suite.rs` (lifecycle, summaries, original controls),
`legends.rs` (placements and child-cache contracts), `text.rs` (paragraph,
heatmap, matrix), `annotations.rs` (significance), and `fonts.rs` (coverage and
observer overhead). Raw `paired_counter`, `paired_root`, `paired_guide`, and
`child_cache` records distinguish real work and owners from theoretical caps.

**Timing boundary change:** `root_measure` now sums the complete feedback rounds
in `src/layout/kernel.rs`: round creation, measurement, dimension validation,
feedback finishing, convergence/error checks, and history work. It no longer
times only the `measure` closure. Final freezing/finalization is outside this
timer. Historical `root_measure` timings therefore are not directly comparable.
Preparation (including fonts), capture, null validation, publication, SVG
outlining, PNG rasterization/encoding, and SVG replay serialization remain
separate observations; nested stages must not be added as independent costs.

Unless stated otherwise, table timings are **min / median / max milliseconds**
over three samples, rounded to three decimals. Small differences are not a
statistical significance claim.

## Auto Versus Selected Edge

Each successful Auto pair has identical outlined SVG, preserved-text SVG,
encoded PNG, and decoded PNG dimensions/type/pixels to its actually selected
explicit edge. Auto accepts the first complete fitting arrangement in the
existing right/preferred, right/compact, top, bottom, left order; it does not
search for an aesthetically best placement. Explicit right still permits reflow.

| Fixture and policy | Root rounds time | End-to-end time |
| --- | --- | --- |
| Small Auto | 0.633 / 0.672 / 0.695 | 25.038 / 25.217 / 25.268 |
| Small selected right | 0.657 / 0.664 / 0.678 | 25.029 / 25.266 / 25.363 |
| Body-starved Auto | 1.233 / 1.236 / 1.270 | 25.153 / 25.177 / 25.303 |
| Body-starved selected top | 0.721 / 0.734 / 0.774 | 24.421 / 24.587 / 24.821 |
| Reflow Auto | 5.465 / 5.467 / 5.773 | 35.824 / 35.904 / 36.446 |
| Reflow selected right | 5.454 / 5.466 / 5.469 | 35.789 / 35.815 / 36.048 |
| Adequate facets Auto | 1.696 / 1.705 / 1.717 | 29.880 / 30.063 / 30.091 |
| Adequate facets selected right | 1.655 / 1.692 / 1.698 | 29.764 / 30.033 / 30.106 |
| Pressured facets Auto | 7.420 / 7.478 / 7.585 | 35.001 / 35.188 / 35.408 |
| Pressured facets selected top | 4.021 / 4.048 / 4.161 | 31.715 / 31.845 / 32.346 |

Small is 800x480 with 3 entries/6 marks; body starvation is 640x480 with
2 long entries/4 marks and a 28-character `M` x-axis title; reflow is 640x400
with 18 long entries/36 marks. Shared fixtures have two panels: adequate
1200x720 with 3 entries, pressured facets 560x360 and Canvas 480x360 with
12 long entries. Data and complete label sources are generated in `legends.rs`.

| Auto fixture | Root rounds | Guide trials / body checks | Observed decision |
| --- | ---: | ---: | --- |
| Small | 1 | 1 / 1 | Preferred right accepted |
| Body-starved | 1 | 3 / 3 | Preferred and compact right rejected by body; top accepted |
| Reflow | 1 | 2 / 1 | Preferred right rejected guide-only; compact right accepted |
| Adequate Canvas/facets | 2 | 2 / 4 | Round 0 right provisional; round 1 right accepted |
| Pressured Canvas/facets | 2 | 4 / 4 | Round 0 right provisional; round 1 rejects both right flows, accepts top |

These are actual counts per render, not bounds. In the reflow pair both policies
perform 90 text proposal visits and 249 exact text trials. Body-starved Auto
performs 12 proposals/52 exact trials versus selected top's 0/28 and one guide
trial/body check. Pressured facet Auto performs 72 proposals/327 exact trials
versus selected top's 0/147; selected top has two guide trials, four body checks,
and two root rounds. Shared attempts retain one shared-guide owner, not one
guide per child. A provisional round is not final success.

The median root-time difference is about 0.503 ms for body starvation and
3.430 ms for pressured facets, with end-to-end median differences of 0.591 and
3.343 ms respectively. These are differences of sample medians, not isolated
selector timings. Small/reflow pairs do essentially the same work; the small
Auto end-to-end median is actually slightly lower. Reflow's Auto maximum
(36.446 ms) also illustrates why tiny differences should not be overinterpreted.
For context, reflow Auto spends a median 17.924 ms in preparation including fonts
and 8.678 ms in PNG rasterization/encoding, versus 5.467 ms in root rounds.

Explicit bottom is a different arrangement, not an output-equivalent shortcut:

| Additional policy | Root rounds time | End-to-end time |
| --- | --- | --- |
| Small bottom, success | 0.855 / 0.882 / 0.893 | 24.616 / 24.666 / 24.979 |
| Body-starved bottom, success | 0.993 / 1.091 / 1.126 | 24.745 / 25.032 / 25.299 |
| Reflow bottom, success | 4.871 / 4.931 / 5.013 | 35.267 / 35.303 / 35.710 |
| Body-starved right, failure | 0.930 / 0.952 / 1.016 | 18.786 / 18.852 / 19.011 |

All three bottom cases record two guide trials/two body checks. Explicit
body-starved right rejects both flows with an x-axis-title error and publishes
nothing; its shorter elapsed time is not a faster successful render. Separate
right controls also ran for small and reflow (end-to-end medians 24.978 and
35.850 ms).

## Shared And Inside Limits

Canvas has no public explicit shared-edge selector, so there is no invented
Canvas Auto-versus-explicit pair. Its adequate/pressured Auto root times are
1.378 / 1.438 / 1.449 and 7.605 / 7.662 / 7.732 ms; end-to-end times are
32.605 / 34.108 / 34.241 and 34.552 / 34.623 / 34.823 ms.

Every measured shared end-to-end fixture, including selected-edge facets,
records **0 child-cache hits, 16 misses, 0 feedback replays**. Changed assignments
invalidate reuse; repeated rounds are not evidence of cache hits. Separately,
`shared_legend_child_cache_replay_contract` is a controlled identical-input
fixture demonstrating **2 hits, 2 misses, 1 replay** for each owner type
(Canvas/facet). It checks shared payload identity and body feedback replay.
That normal contract test is not one of the aggregate's ignored timed tests,
and its result must not be substituted for end-to-end cache utilization.

Automatic inside-corner selection is unreachable through the current public
populated-panel path. The inside fixtures explicitly request `InsideTopRight`:
800x600, 3 guide entries, 6 sparse or 400 dense marks, and a fixed bottom-left
`Caller note` annotation. Both record **0 automatic inside candidates**.

| Inside fixture | Root rounds | Geometry scans / annotation comparisons | Root time | End-to-end time |
| --- | ---: | ---: | --- | --- |
| Sparse | 1 | 14 / 1 | 0.406 / 0.411 / 0.445 | 27.892 / 29.730 / 29.737 |
| Dense | 2 | 2005 / 2 | 1.259 / 1.280 / 1.324 | 36.183 / 37.749 / 37.907 |

Both final legends have `[x,y,width,height] = [632.97,50.56,116.47,104.4]`
inside panel `[79.2,30,690.8,501.4]`. Assertions check the requested corner,
panel containment, retained annotation, mark count, and clearance from every
mark. Dense range feedback is real work; neither these scans nor fixed annotation
comparisons should be described as an automatic corner search.

## Text And Specialized Layout

Paragraph planning uses a prepared Publication theme, 320 px width, and caption
typography. Setup is excluded. The short source is `Measured response`; the
multiline source is seven repetitions of the measured-responses sentence in
`text.rs`, followed by a newline and the independent-observations sentence.

| Paragraph phase | Time (ms) | Proposals / exact trials / shapes | Lines / retained glyphs |
| --- | --- | ---: | ---: |
| Short first | 0.143863 / 0.163476 / 0.167160 | 0 / 1 / 1 | 1 / 17 |
| Short cached | 0.000399 / 0.000399 / 0.000498 | 0 / 0 / 0 | 1 / 17 |
| Multiline first | 2.128076 / 2.201551 / 2.202745 | 99 / 44 / 44 | 22 / 707 |
| Multiline cached | 0.000796 / 0.000896 / 0.000896 | 0 / 0 / 0 | 22 / 707 |

The exact trials include rejected extensions, not merely retained lines. Cached
plans share payloads and preserve the original source. Nanosecond-scale repeats
are timer-scale observations, not general cache-latency guarantees.

Heatmaps are 1200x500 with six column categories, two row categories, and six
input values (0 through 5). Short names are Alpha through Foxtrot; long labels
append `deliberately long category label`. The ambiguity case prepends
`Shared deliberately long prefix`. Wrap and Elide are explicit policies, not
two candidates in a policy search.

| Heatmap case | Prefix trials | Exact trials / shapes | Root time | End-to-end time |
| --- | ---: | ---: | --- | --- |
| Short Wrap | 0 | 14 / 14 | 0.497 / 0.514 / 0.550 | 30.555 / 30.611 / 30.922 |
| Short Elide | 0 | 14 / 14 | 0.527 / 0.558 / 0.562 | 30.808 / 30.987 / 31.237 |
| Long Wrap | 0 | 50 / 50 | 1.636 / 1.657 / 1.729 | 35.005 / 35.442 / 35.478 |
| Long Elide | 300 | 115 / 115 | 2.946 / 3.007 / 3.035 | 34.316 / 34.548 / 34.699 |
| Shared-prefix Wrap | 0 | 50 / 50 | 1.630 / 1.646 / 1.696 | 34.567 / 34.828 / 35.279 |
| Shared-prefix Elide, failure | 96 | 29 / 29 | 0.883 / 0.902 / 0.913 | 18.740 / 18.791 / 18.887 |

Long Elide retains six distinct strings: `Alpha deliberat...`,
`Bravo deliberat...`, `Charlie delibera...`, `Delta deliberate...`,
`Echo deliberatel...`, and `Foxtrot delibera...`. Wrap retains complete labels
without ellipses. Shared-prefix Elide rejects ambiguous output and publishes
nothing. Long Elide's greater planning cost but lower total time than Wrap is
not contradictory: the output text differs, so downstream costs differ too.

Matrices use three nonconstant variables, 12 observations, and nine cells;
names are `V0` through `V2` or `Variable {index} measured response over time`.

| Matrix | Local passes / root rounds | Shapes | Root time | End-to-end time |
| --- | ---: | ---: | --- | --- |
| 1200x1200 short | 3 / 1 | 10 | 0.725 / 0.731 / 0.824 | 38.193 / 38.545 / 38.568 |
| 1200x1200 long | 3 / 1 | 10 | 0.833 / 0.854 / 0.884 | 43.124 / 43.172 / 43.222 |
| 600x600 short | 3 / 1 | 10 | 0.708 / 0.718 / 0.790 | 25.907 / 26.117 / 26.216 |
| 600x600 long | 6 / 1 | 94 | 2.248 / 2.286 / 2.307 | 31.219 / 31.239 / 31.296 |

All four record 90 tick sources, 54 distinct tick lists, and 90 metric lookups.
Tight long labels additionally record 108 text proposals/94 exact trials.
Assertions retain two labels per variable and nine cells at least 40x40 px.
The six local passes are not six root-feedback rounds or six tick searches.

## Significance

The direct planner uses an 800x600 panel, equal-height bars at 10, initial range
`(0,10)`, and 1/4/8 nested two-star brackets from Group 0 to successive groups.
Font preparation is excluded; each theme is called twice.

| Brackets | First time (ms) | Cached-label repeat (ms) | Growth / bisections / level checks | Final upper range |
| --- | --- | --- | ---: | ---: |
| 1 | 0.119124 / 0.125420 / 0.198273 | 0.002299 / 0.002398 / 0.002498 | 1 / 64 / 66 | 10.695187 |
| 4 | 0.108830 / 0.125919 / 0.146106 | 0.004497 / 0.004497 / 0.004497 | 1 / 64 / 260 | 12.422360 |
| 8 | 0.133015 / 0.134115 / 0.138211 | 0.007495 / 0.007495 / 0.007795 | 1 / 64 / 521 | 15.831135 |

First calls shape one shared label; repeats shape none but perform the same
growth, bisections, and level checks. The final ranges are identical on repeat,
with ordered brackets fitting the panel. Thus label caching does not eliminate
the significance search. The first one-bracket maximum is visibly noisy.

## Font Coverage And Overhead

Synthetic SFNT fixtures vary 2/8 candidate faces and 2/16 base characters, each
combined with U+0301. Both whole-unit coverage on the last face and split
base/mark coverage on separate faces are exercised. Counts below are identical
for whole/split variants and all three runs; probes are actual coverage tests,
not layout trials.

| Candidates / bases | First scalar / new scalar probes | Batch scalar probes | First unit probes | Cached scalar / unit probes |
| --- | ---: | ---: | ---: | ---: |
| 2 / 2 | 2 / 2 | 6 | 6 | 0 / 0 |
| 2 / 16 | 16 / 16 | 34 | 48 | 0 / 0 |
| 8 / 2 | 8 / 8 | 24 | 18 | 0 / 0 |
| 8 / 16 | 64 / 64 | 136 | 144 | 0 / 0 |

First/new scalar phases each introduce half the bases. The independent batch
phase prepares all bases plus the mark; first unit preparation processes one
base-plus-mark unit per base. Scalar preparation probes the full candidate
matrix. Split coverage verifies per-scalar fallback without claiming a face
covers the whole unit. Each environment resolves the family pair once.

Representative total phase sums (including separately logged synthetic setup)
are 0.150 / 0.165 / 0.171 ms for 2 candidates/2 bases/whole coverage and
0.254 / 0.255 / 0.265 ms for 8/16/whole; 8/16/split is
0.252 / 0.254 / 0.276 ms. These are sums of measured phases, not render times.

System discovery takes 17.591 / 17.829 / 17.955 ms. First scalar resolution
takes 0.080049 / 0.081648 / 0.082947 ms (99 probes, 11 retained scalars);
new text takes 0.042373 / 0.042473 / 0.052766 ms (36 probes, 15 retained);
repeat takes 0.000899 / 0.000899 / 0.001099 ms (0 probes). There are 9 candidates
and no unit probes in this system scalar test. Discovery is once per fresh
environment, not globally cached.

Only **font active-versus-inactive observer overhead** was measured: six
alternating pairs, each sample summing 32 fresh operations with 8 candidates,
17 scalars and 16 units. Setup and observer start/finish are outside work time.

| Observer mode | Min / median / max ms per 32 operations |
| --- | --- |
| Inactive | 6.255301 / 6.295575 / 6.384515 |
| Active | 6.296171 / 6.367225 / 6.412097 |
| Paired active minus inactive | 0.027582 / 0.048517 / 0.107328 |

Active samples record 8,704 scalar and 4,608 unit probes. Both modes execute
13,312 actual probes according to the legacy counter and produce equal
selection/resource results. Pairwise differences range from 0.432% to 1.708%.
Both modes still contain test hooks and legacy counters: this does **not**
measure production zero-hook overhead, whole-render instrumentation overhead,
or observer start/finish cost. No overhead correction is applied to other tables.

## Output Equality

A direct comparison of the nine complete `hash,` rows in
`/tmp/opencode/publication-efficiency-baseline.log` and the final aggregate log
found no differences: **all 27 SHA-256 values match**, as do byte lengths,
dimensions, PNG color type and bit depth. These are the original no-guide
controls, not evidence about Auto placement cost. Each control's three current
runs also checks byte-hash stability internally.

| Control | Dimensions | SVG / PNG / decoded hash comparison |
| --- | --- | --- |
| scatter100 | 960x720 | All 3 unchanged |
| long_label_bar | 960x720 | All 3 unchanged |
| caption200_1para | 1200x1000 | All 3 unchanged |
| caption200_3para | 1200x1000 | All 3 unchanged |
| facets2x2 | 1200x1000 | All 3 unchanged |
| mixed_canvas | 1600x800 | All 3 unchanged |
| matrix4 | 1400x1400 | All 3 unchanged |
| separate10 | 960x720 | All 3 unchanged |
| reuse10 | 960x720 | All 3 unchanged |

For compact identification, the successful Auto/selected-edge pairs have these
SHA-256 **prefixes**; complete hashes are in the aggregate `paired_hash` rows.
Equality assertions compare full bytes and decoded images, not prefixes.

| Pair | SVG prefix | PNG prefix | Decoded prefix |
| --- | --- | --- | --- |
| Small/right | `1c81d888cbe1` | `e37b4a12e557` | `9f875205f587` |
| Body-starved/top | `25b1193b8c7f` | `3df7512e8c90` | `481fbb7d7919` |
| Reflow/right | `c27d9084648e` | `f04da7219b66` | `7f4748c87ace` |
| Adequate facets/right | `83968cc61db5` | `e15476c1d073` | `5e6051ebe954` |
| Pressured facets/top | `c894552f990b` | `0fc423e76d6b` | `bd2735e0be65` |

## Recommendations

These priorities require separately approved future scope; none authorizes an
algorithm, cache, API, or font-policy change.

1. Investigate pressured shared-guide and long-label reflow work first if planning
   latency matters: the fixtures demonstrate real rejected arrangements and
   repeated text work. Preserve first-fit order and distinguish invalidated child
   assignments from opportunities for safe reuse.
2. For total publication latency, profile operation-scoped font setup and PNG
   rasterization/encoding before assuming placement dominates. Existing `Output`
   reuse is an available workflow; these results do not justify a global cache.
3. Profile long-label elision and tight matrix text separately. Keep uniqueness,
   complete wrapping, cell fit, and exact glyph geometry as correctness gates;
   do not replace policies merely because their output-dependent totals differ.
4. Extend measurement confidence before proposing broad optimizations: more
   samples/hosts/fonts, controlled scheduling, and separately scoped observer
   overhead outside fonts. Public automatic-inside coverage and an explicit
   Canvas shared-edge comparison would require an API/reachability decision,
   not relabeling the existing fixtures.

These small deterministic workloads establish local costs and coverage, not a
universal speedup, a maximum latency, or permission to begin an optimization.
