# Local-Search Measurements

### T5b7 Native-Composition Shared-Guide Comparison

The T5b7 checkout retains the same two-Scatter, three-entry adequate Canvas
(1200×720) and 12-entry pressured Canvas (480×360) release fixtures and the
same three interleaved Auto/selected-edge samples. The ignored
`orchestrator::tests::benchmarks::legends::shared_legend_measurements` test
passed with byte-identical Auto/selected-edge SVG and PNG, decoded pixels,
and exact shared-guide owner/attempt traces. Measurements are stage-attributed;
they are not a claim about performance on other machines or inputs.

| Canvas fixture | Earlier T5b1b2b1 child misses | T5b7 child misses | T5b7 root rounds / guide trials | T5b1b2b1 end-to-end min/median/max ms | T5b7 end-to-end min/median/max ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Adequate | 12 | 8 | 2 / 2 | 38.898 / 40.711 / 52.767 | 35.810 / 37.427 / 51.703 |
| Pressured | 18 | 12 | 3 / 7 | 34.656 / 34.760 / 34.796 | 31.593 / 31.667 / 43.010 |

Both fixtures have zero child-cache hits: changed assigned cells are genuinely
remeasured. The separate two-guide `PerPlot` Canvas regression records a
feedback-replayed cache hit for an unchanged neighbor and a fresh trial for a
dirty panel. The shared facet fixture likewise retains 2/3 root rounds, 2/4
guide trials, and 8/12 child misses for adequate/pressured assignments.
Release reproduction:
`cargo test --release --locked --offline --lib benchmarks::legends::shared_legend_measurements -- --ignored --nocapture --test-threads=1`.
The old T5a and T5b1b2b1 snapshots and timings below retain their original
checkout attribution; T5b deliberately changes composed geometry.

### T5b1b2b1 Prepared Scatter Shared Guides

The T5b migration keeps the historical T5a tables below as stage-attributed
evidence. On the T5b1b2b1 checkout, all-Scatter Shared Canvas guide identities,
glyphs, and titles are resolved before fonts. Their containing-frame body probes
remain for geometric fit, but the two Hue-only child probes per root round are
gone. Three ignored release legend samples passed and retained Auto/selected-edge
SVG, PNG, and decoded-pixel equality. Adequate/pressured Canvas examples record
12/18 child misses instead of 16/24, respectively, with no child hits; both
still retain their actual 2/3 root rounds. The 3-sample adequate Canvas
end-to-end minimum/median/maximum was 38.898/40.711/52.767 ms and pressured
Canvas 34.656/34.760/34.796 ms. These uncontrolled measurements are not a
universal speedup claim. The prior historical timings and hashes have not been
rewritten. Log: `/tmp/opencode/t5b1b2b1-legends-release.log`.

This is a measured cost and coverage report for T5a3q, not an optimization.
The final aggregate release run in
`/tmp/opencode/t5a3q-all.log` passed all 10 requested ignored measurement tests
(0 failed, 702 filtered out; 6.61 seconds of test execution). The subsequent
`just verify` passed strict Clippy, 695 library tests (17 ignored), 441 integration
tests including five unchanged snapshots, API documentation, and package
verification (207 files). The verification log is `/tmp/opencode/t5a3q-verify.log`.

## T5a6b1 Native-Gutter Audit (2026-09-14)

This benchmark-only audit supersedes earlier current-checkout claims, not their
historical evidence. All T5a3q/T5a5 tables, timings, hashes, and run reports below
are unchanged. Inputs, dimensions, label sources, counter definitions, and
publication/equality gates are preserved. Only `tests/benchmarks/legends.rs`,
`tests/benchmarks/suite.rs`, and this document were edited; no production code,
snapshots, `just verify`, or commit are part of this audit.

The compact-right coverage follow-up below adds a separate genuine compact-flow
fixture. References use the renamed `preferred_right_control` identifier for the
unchanged 630x480 control; historical measured values remain unchanged.

The initial normal run reproduced four stale contracts: native intrinsic gutter
control, public reflow/cache-off equivalence, populated legends, and controlled
child replay (**6 passed, 4 failed, 10 ignored**). The initial ignored release
run passed nine groups and stopped the outside group at its one-round assertion.
Those results are in `/tmp/opencode/t5a6b1-benchmarks-initial-normal.log` and
`/tmp/opencode/t5a6b1-benchmarks-initial-release.log`. The diagnostic run in
`/tmp/opencode/t5a6b1-benchmarks-probe.log` captured every unchanged outside
fixture/policy plus the natural gutter and controlled-child traces. Its temporary
probe was removed; it is not acceptance evidence.

Final commands and logs:

```sh
cargo test --locked --offline --lib benchmarks -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1
cargo clippy --locked --offline --all-targets -- -D warnings
```

- `/tmp/opencode/t5a6b1-benchmarks-normal.log`: **10 passed, 0 failed, 10 ignored,
  931 filtered out**, 50.84 seconds.
- `/tmp/opencode/t5a6b1-benchmarks-release.log`: **all 10 ignored groups passed,
  0 failed, 0 ignored, 941 filtered out**, 8.07 seconds.
- `/tmp/opencode/t5a6b1-benchmarks-clippy.log`: strict locked/offline all-target
  Clippy passed with warnings denied. Scoped Rust formatting and diff checks pass.

The final release run was not run concurrently with another audit command.
Toolchain remains Rust 1.96.0, LLVM 22.1.2, `x86_64-unknown-linux-gnu`.
Publication/system fonts, three interleaved paired samples, timing boundaries,
and uncontrolled host/cache conditions remain as described below. Every paired
case now asserts equality of the **entire counter vector and shape count across
all three samples**, in addition to fixture-specific exact goldens. Raw
`outside_counts`, `outside_trace`, `shared_trace`, `paired_counter`, `paired_root`,
`paired_guide`, and new `paired_gutter` records retain measured work and ownership.
Times in this section are **minimum / median milliseconds**, not speedup claims.

### Actual Gutter And Guide Paths

The native root's **16 px left inset is root padding**, not an intrinsic axis
reservation. Root title/caption space is independent of axis feedback. The common
panel planner submits absolute selected-axis requirements, including the existing
4 px clearance and outward rounding, starting from zero. The former assertion
`required.left + 4 <= canvas.insets.left` therefore tested the wrong owner.

`historical_reflow_native_measured_gutter_control` now checks the actual three
rounds, including consumption of each preceding request and final stability:

| Round | Consumed [left, right, bottom, top] | Requested | Body width (px) | Guide |
| --- | --- | --- | ---: | --- |
| 0 | [0, 0, 0, 0] | [79, 5, 59, 5] | 367.7890625 | Preferred right, provisional |
| 1 | [79, 5, 59, 5] | [94, 5, 59, 5] | 283.7890625 | Preferred right, provisional |
| 2 | [94, 5, 59, 5] | [94, 5, 59, 5] | 268.7890625 | Preferred right, accepted |

The unchanged public 640x400, 18-entry/36-mark fixture has the same gutter changes
and preferred-right path. It no longer reaches compact flow or an 80 px body.
This is changed measured-frame allocation, not an optimization of the old reflow
profile. Auto, explicit right, and child-reuse-disabled explicit right retain
identical outlined SVG, preserved SVG, PNG, and decoded pixels; complete guide
frames, body minima, containment, and all marks remain checked.

The caller-title fixtures are also unchanged: `starvation` is 640x480 with 28
`M`s, and `preferred_right_control` is 630x480 with 20 `M`s; both retain two long guide
entries/four marks. The preserved-text contract checks their complete titles.
Under actual native allocation, `preferred_right_control` fits **preferred right**
in two rounds, including explicit right. It is not a compact-flow coverage claim.
Starvation first obtains provisional preferred right, then
rejects preferred and compact right **guide-only** in round 1 before accepting
top. Explicit right fails at the guide entry's hard unbroken requirement and
publishes no artifacts, rather than reaching the former x-axis-title failure.

| Outside fixture/policy | Rounds | Guide trials / body checks | Proposals / exact trials (= shapes) |
| --- | ---: | ---: | ---: |
| Small Auto/right/bottom | 2 | 2 / 2 | 0 / 59 |
| Starvation Auto, selected top | 2 | 4 / 2 | 10 / 83 |
| Starvation explicit top/bottom | 2 | 2 / 2 | 0 / 57 |
| Starvation explicit right, failure | 2 | 3 / 1 | 10 / 69 |
| Historical reflow Auto/right | 3 | 3 / 3 | 0 / 339 |
| Historical reflow bottom | 2 | 2 / 2 | 0 / 236 |
| `preferred_right_control` Auto/right | 2 | 2 / 2 | 10 / 63 |
| `preferred_right_control` bottom | 2 | 2 / 2 | 0 / 57 |

These are exact per-case expectations, not inferred from phase names or relaxed
round bounds. Normal contracts now also reach every measured right/bottom
control. Shared traces explicitly retain the remaining edge/flow coverage:
adequate roots use provisional then accepted preferred right; pressured Canvas
uses provisional right, then rejects both right flows at the Body phase before
provisional top, repeating both rejections before accepted top in round 2.
Pressured facets use provisional preferred right, guide-only rejected preferred
right then provisional compact right, and accepted preferred right in round 2.
Thus compact provisional and rejected work is still exercised, but none of these
unchanged fixtures now claims a final accepted compact-right arrangement.

| Shared fixture | Rounds | Trials / checks | Proposals / exact (= shapes) | Child hits / misses / replays |
| --- | ---: | ---: | ---: | ---: |
| Adequate Canvas | 2 | 2 / 4 | 0 / 97 | 0 / 16 / 0 |
| Pressured Canvas, top | 3 | 7 / 6 | 72 / 589 | 0 / 24 / 0 |
| Adequate facets, Auto/right | 2 | 2 / 4 | 0 / 41 | 0 / 16 / 0 |
| Pressured facets, Auto/right | 3 | 4 / 6 | 148 / 370 | 0 / 24 / 0 |

The independent controlled child fixture now takes **two rounds, 3 hits / 4
misses / 1 replay** for both Canvas and facet owners. Each round has a legend
probe miss followed by an identical-input hit and a panel miss; only the stable
second round has a panel hit and feedback replay. The contract asserts this
complete event sequence and payload identity. It does not establish reuse in
the end-to-end shared fixtures, which still have zero hits/replays.

### Current Legend Timings

| Fixture/policy | Root time, min / median | End-to-end, min / median |
| --- | ---: | ---: |
| Small Auto | 1.374 / 1.390 | 27.160 / 27.431 |
| Small selected right | 1.388 / 1.457 | 27.214 / 27.226 |
| Starvation Auto | 2.207 / 2.245 | 27.072 / 27.224 |
| Starvation selected top | 1.648 / 1.682 | 26.482 / 26.491 |
| Historical reflow Auto | 7.784 / 7.857 | 39.632 / 39.710 |
| Historical reflow selected right | 7.780 / 7.826 | 39.827 / 40.072 |
| `preferred_right_control` Auto | 1.731 / 1.779 | 25.796 / 25.944 |
| `preferred_right_control` selected right | 1.717 / 1.739 | 25.937 / 25.979 |
| Small bottom | 1.411 / 1.428 | 27.005 / 27.161 |
| Starvation bottom | 1.612 / 1.685 | 26.524 / 26.680 |
| Historical reflow bottom | 5.538 / 5.549 | 36.667 / 36.796 |
| `preferred_right_control` bottom | 1.607 / 1.623 | 26.406 / 26.419 |
| Starvation right, failure | 1.796 / 1.798 | 19.941 / 20.087 |
| Adequate Canvas | 3.476 / 3.491 | 37.048 / 38.574 |
| Pressured Canvas | 14.839 / 14.857 | 41.679 / 41.752 |
| Adequate facets Auto, paired with right | 2.339 / 2.343 | 32.800 / 32.837 |
| Adequate facets selected right | 2.325 / 2.368 | 32.776 / 32.842 |
| Pressured facets Auto, paired with right | 11.707 / 11.757 | 41.953 / 42.113 |
| Pressured facets selected right | 11.758 / 11.916 | 42.126 / 42.396 |
| Inside sparse | 1.195 / 1.281 | 32.421 / 32.805 |
| Inside dense | 1.391 / 1.409 | 40.132 / 40.291 |

All six Auto/selected-edge pairs pass full outlined/preserved SVG, encoded PNG,
and decoded-image equality, including pressured facets with child reuse disabled.
Bottom is a different output, not an equivalent shortcut. Inside placement still
has two rounds, zero automatic corner candidates, 28/2005 sparse/dense geometry
scans, and two caller-annotation comparisons. The final sparse/dense panel boxes
are `[131,17,648,516]` / `[95,17,684,516]` in SVG x/y/width/height coordinates;
both legends are `[641.388671875,38.1396484375,116.4716796875,104.400390625]`.
Requested corner, containment, caller note, mark counts, and mark clearance pass.

### Other Release Groups

Heatmap prefixes remain 400 for long Elide, exactly 200 in each of two rounds
with only the second successful. Ambiguous Elide fails at 96 without artifacts;
Wrap retains complete labels and successful Elide retains six distinct strings.
Matrix already has two retained measurement passes, four for tight long labels,
not the T5a5 historical three/six; capture does not repack. All Matrix cases have
one root round, 12 tick sources/lists, and 72/36 adequate/tight metric lookups.

| Text fixture | Prefix trials / Matrix passes | Exact trials (= shapes) | Root min / median | End-to-end min / median |
| --- | ---: | ---: | ---: | ---: |
| Heatmap short Wrap | 0 / 0 | 14 | 0.641 / 0.664 | 33.004 / 33.229 |
| Heatmap short Elide | 0 / 0 | 14 | 0.673 / 0.682 | 32.857 / 33.187 |
| Heatmap long Wrap | 0 / 0 | 67 | 2.057 / 2.076 | 37.904 / 37.998 |
| Heatmap long Elide | 400 / 0 | 115 | 3.431 / 3.483 | 37.256 / 37.544 |
| Heatmap shared-prefix Wrap | 0 / 0 | 89 | 2.442 / 2.449 | 37.763 / 37.775 |
| Heatmap shared-prefix Elide, failure | 96 / 0 | 29 | 0.946 / 0.960 | 20.737 / 20.754 |
| Matrix adequate short | 0 / 2 | 9 | 0.943 / 0.943 | 43.397 / 43.729 |
| Matrix adequate long | 0 / 2 | 9 | 1.053 / 1.068 | 48.276 / 48.312 |
| Matrix tight short | 0 / 2 | 6 | 0.808 / 0.816 | 28.524 / 28.821 |
| Matrix tight long | 0 / 4 | 90 | 2.529 / 2.561 | 34.220 / 34.438 |

Paragraph first-call proposals/exact/shapes remain 0/1/1 short and 99/44/44
multiline; cached repeats are all zero with shared retained payloads. Significance
still performs one growth and 64 bisections, with 66/260/521 level checks for
1/4/8 brackets, on both first and repeated calls. First calls shape one label,
repeats zero; final ranges remain equal.

| Local fixture | First min / median | Cached-repeat min / median |
| --- | ---: | ---: |
| Short paragraph | 0.175066 / 0.198142 | 0.000434 / 0.000434 |
| Multiline paragraph | 2.401425 / 2.440209 | 0.000975 / 0.000975 |
| Significance, 1 bracket | 0.136825 / 0.139859 | 0.002600 / 0.002600 |
| Significance, 4 brackets | 0.145167 / 0.153292 | 0.004767 / 0.004983 |
| Significance, 8 brackets | 0.175391 / 0.178100 | 0.008233 / 0.009750 |

Synthetic font counts are unchanged for whole/split coverage: for candidate/base
pairs 2/2, 2/16, 8/2, 8/16, first and new scalar probes are respectively
2, 16, 8, 64; batch probes are 6, 34, 24, 136; first unit probes are 6, 48, 18,
144. Cached scalar/unit probes are zero. These are measured probe counts, not
layout work. Total synthetic times sum setup and the separately measured phases.

| Font fixture | Whole-coverage min / median | Split-coverage min / median |
| --- | ---: | ---: |
| 2 candidates / 2 bases | 0.149300 / 0.157200 | 0.147000 / 0.148100 |
| 2 candidates / 16 bases | 0.166200 / 0.170700 | 0.165200 / 0.169800 |
| 8 candidates / 2 bases | 0.176800 / 0.178800 | 0.175700 / 0.175700 |
| 8 candidates / 16 bases | 0.246100 / 0.254600 | 0.247400 / 0.261300 |

System preparation records 457 discovery entries, nine candidates, and first/new/
repeat scalar probes of 99/36/0, with zero unit probes. Discovery min/median is
17.856001/17.880300 ms; scalar first is 0.079100/0.079200, new 0.044800/0.044800,
and cached repeat 0.000800/0.000800 ms. The six observer-overhead pairs each execute
32 operations: active counts are 8,704 scalar plus 4,608 unit probes, and both
modes execute 13,312 legacy-observed probes with equal results. Inactive work is
6.187600/6.218101 ms, active 6.165600/6.246650 ms, and paired active-minus-inactive
is -0.441836/0.011750 ms. The negative minimum is host/timing noise, not evidence
that observation accelerates work or a production-overhead measurement.

Publication controls retain all three samples' byte-hash stability. Current
native geometry is not asserted equal to historical T5a3q hashes. `calls` records
give these actual root/capture counts; the repeated-publication rows represent
ten SVG/PNG publication pairs, not ten operations when reusing `Output`.

| Publication control | Root rounds / captures | Shapes | Root min / median | End-to-end min / median |
| --- | ---: | ---: | ---: | ---: |
| scatter100 | 2 / 1 | 31 | 1.177 / 1.197 | 37.897 / 40.593 |
| long_label_bar | 2 / 1 | 22 | 0.867 / 0.928 | 33.613 / 33.788 |
| caption200_1para | 2 / 1 | 69 | 5.919 / 5.922 | 90.474 / 90.639 |
| caption200_3para | 2 / 1 | 71 | 5.982 / 6.020 | 91.230 / 91.953 |
| facets2x2 | 2 / 1 | 17 | 2.505 / 2.579 | 52.334 / 52.413 |
| mixed_canvas | 2 / 1 | 79 | 2.677 / 2.755 | 52.974 / 53.581 |
| matrix4 | 1 / 1 | 11 | 1.893 / 1.913 | 75.667 / 76.414 |
| separate10 | 20 / 10 | 310 | 12.620 / 12.672 | 440.046 / 440.795 |
| reuse10 | 2 / 1 | 31 | 1.214 / 1.258 | 184.137 / 184.363 |

No new production bug was found in these benchmark paths. This is benchmark
closure for the tested dirty T5a6b1 checkout, not full-repository verification or
phase completion; it leaves those gates and all historical comparisons separate.

### Compact-Right Coverage Follow-Up (2026-09-14)

This follow-up closes the final-accepted-compact coverage gap identified above.
The former `compact_right` case is now **`preferred_right_control`** in every
repository caller and fixture reference. It still uses 630x480, two long entries,
four marks, and the 20-character `M` x-axis title, with exactly the same expected
paths and counts. Older raw logs retain the former identifier. The historical
640x400 reflow, starvation, small, shared, inside, and cache controls are not
changed or weakened; historical numeric tables and hashes are not rewritten.

The new **`native_compact_right`** public Scatter fixture is **520x240, eight
entries, 16 marks**. For `i` in 0..16, x is `i`, y is `(i * 7) % 13`, and hue
is `Complete explanatory condition 0` for the first pair, then `Condition 1`
through `Condition 7` for the remaining pairs. Its one long label and seven short
labels make compact two-column packing useful without changing any old fixture.
There is no custom title, seeded gutter, forced compact policy, or planner bypass.

Discovery used a temporary ignored Rust test calling public `preflight`, with
no probe files or published artifacts; the probe was removed. Auto and explicit
right both selected compact at all six nearby combinations of widths 510/520/530
and heights 230/240. At height 250 they selected preferred right, confirming the
height pressure being exercised rather than treating the name as evidence.
The fixed 520x240 fixture is now reached by both the normal populated-legend
contract and the ignored outside-legend measurement group.

Both Auto and explicit right assert this exact complete guide trace:

| Root round | Edge | Compact | Phase | Outcome |
| --- | --- | --- | --- | --- |
| 0 | Right | false | Body | Provisional |
| 1 | Right | false | GuideOnly | Rejected |
| 1 | Right | true | Body | Accepted |

Round 0 requests absolute gutters `[79,5,59,5]` from zero; round 1 is successful.
Preferred uniform-width columns no longer fit the reduced-height assignment;
compact flow wraps the long label and retains two populated columns. Assertions
retain all eight complete labels and 16 marks, and check the two column positions
in the published preserved SVG. The Auto/actually-selected-right pair asserts
identical outlined SVG, preserved SVG, encoded PNG, and decoded PNG metadata and
pixels. No equality with a different fixture or the bottom arrangement is claimed.

| New fixture policy | Rounds | Guide trials / body checks | Proposals / exact trials (= shapes) |
| --- | ---: | ---: | ---: |
| Auto, final compact right | 2 | 3 / 2 | 6 / 125 |
| Explicit right, final compact | 2 | 3 / 2 | 6 / 125 |
| Explicit bottom, preferred | 2 | 2 / 2 | 0 / 97 |

All three release samples assert these exact counts and the full counter-vector
and shape-count stability checks. Raw `outside_counts`, `paired_counter`,
`paired_root`, `paired_guide`, `paired_gutter`, and `paired_summary` records use
`native_compact_right`, `native_compact_right_selected_OutsideRight`, and
`native_compact_right_bottom` (plus the separate right control).
Release timings are **minimum / median milliseconds** over three interleaved
samples, with the existing font/host/timing conditions and no speedup claim:

| New fixture policy | Root time | End-to-end |
| --- | ---: | ---: |
| Auto | 2.731192 / 2.773116 | 27.193943 / 27.272158 |
| Selected explicit right | 2.734658 / 2.760008 | 27.111934 / 27.254609 |
| Explicit bottom | 2.207617 / 2.234916 | 26.632126 / 26.810659 |

Follow-up verification commands:

```sh
cargo test --locked --offline --lib benchmarks::legends -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib benchmarks::legends -- --ignored --nocapture --test-threads=1
cargo clippy --locked --offline --all-targets -- -D warnings
```

Normal legends: **5 passed, 0 failed, 3 ignored, 943 filtered out**, 38.75 seconds.
Ignored release legends: **all 3 groups passed, 0 failed, 0 ignored, 948 filtered
out**, 3.62 seconds, including all existing controls and the new compact pair.
Strict all-target Clippy and scoped formatting/diff checks pass. Only
`tests/benchmarks/legends.rs` and this document changed in the follow-up. No new
production bug was found, no production code was edited, and no `just verify`
or commit was run.

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
`preferred_right_control` control still has 1 round, 2 trials, 2 checks, and 11 proposals;
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

A separate public `preferred_right_control` uses **630x480, two long entries/four
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

## T5a6d Closing Audit (2026-09-14)

This is scoped benchmark/test closure on the dirty T5a6d checkout, not phase
completion or a byte-equivalent historical performance baseline. All preceding
tables and hashes are unchanged. No production code, existing fixture inputs,
dimensions, label sources, counter definitions, or snapshots changed in this audit.
No round-count expectation needed adjustment: the approved dirty legend count
updates already matched the actual traces. No `just verify` or commit was run.

Final commands:

```sh
cargo test --locked --offline --lib benchmarks -- --nocapture --test-threads=1
cargo test --release --locked --offline --lib benchmarks -- --ignored --nocapture --test-threads=1
cargo test --locked --offline --lib -- plots::categorical_distribution::extraction plots::interaction::extraction --nocapture --test-threads=1
cargo clippy --locked --offline --all-targets -- -D warnings
```

Logs under `/tmp/opencode/`:

- `t5a6d-benchmarks-normal.log`: **10 passed, 0 failed, 10 ignored, 999 filtered
  out**, 52.39 seconds.
- `t5a6d-benchmarks-release.log`: **all 10 ignored groups passed, 0 failed,
  1009 filtered out**, 8.23 seconds.
- `t5a6d-extraction-contracts.log`: **15 passed, 0 failed, 1004 filtered out**,
  42.30 seconds, including both corrected fingerprint contracts and two new
  odd-quantum scientific-coordinate contracts.
- `t5a6d-benchmarks-clippy.log`: strict locked/offline all-target Clippy passed.
  Scoped Rust formatting and diff checks also passed.

The release run was not concurrent with another audit command. Toolchain, system
fonts, three interleaved samples, timing boundaries, and uncontrolled host/cache
conditions remain as described above. Durations are observations, not speedups.

### Projection Correction

The former projection always scaled its fraction and deferred spatial rounding
until primitive construction. The approved current projection uses the direct
finite-difference fraction (scaled only when necessary), exact projection keys,
and canonical spatial coordinates shared by marks, ticks, and grids. Consequently,
width/strip arithmetic and triangle construction now start from canonical points.
Replacing the current goldens is a **coordinate correction**, not preservation of
the pre-native byte baseline.

A temporary test-only oracle reconstructed the former projected operands from
the unchanged fixture values, quartiles, KDE, and deterministic strip inputs.
Drawing those operands with the current styles/order/clip reproduced **all seven
historical fingerprints exactly**. Its current-projection distribution recipe
also matched every retained point before SVG serialization. Parsed comparisons
covered **42 mark-only SVGs**: two original assignments, three themes, four
distribution cases and three Interaction visibility modes. Only **135 x-coordinate
operands** differed, each by exactly **1/1024 px**. All y-coordinates, density
outlines, element counts/order, clip attributes, and paint/style attributes matched.
These SVGs contain no label text; this is not a label-source change.

Historical provenance, retained here rather than silently overwritten:

| Fixture | Historical fingerprint | Current fingerprint prefix | Changed operands across six SVGs |
| --- | --- | --- | ---: |
| Boxplot, `8e3395a`, SHA-256 | `5f0b079da3bc344bb21b9937d2b6f460171f452cbdcd1f001a05dd7da035da61` | `d7a7ac878e71` | 48 |
| Violin, `8e3395a`, SHA-256 | `ca6fc08c37c333caecbc10376266944bd18847440be4387be972b27eb0f52a6c` | `ccb910d69f5c` | 60 |
| Sample Raincloud, `8e3395a`, SHA-256 | `5d570309defa397541d5cdcb5fa45f45058e596dc0cc60ac7b6acffac8d02da3` | `9adc9496e02d` | 12 |
| Supplied Raincloud, `8e3395a`, SHA-256 | `090b57970de9e895110807340ea5116fc47e1b2ef95fac81cc61b03d6315da7d` | `f8051c52be375` | 3 |
| Interaction markers/connectors, `a6f9091`, FNV-1a | `fac25eefcd84c63d` | `66849c3c47ba` | 6 |
| Interaction markers only, `a6f9091`, FNV-1a | `3ef60d2fc29c86ec` | `178ec0a9046a` | 6 |
| Interaction connectors only, `a6f9091`, FNV-1a | `bae40bd86df8a981` | `bae40bd86df8` | 0 |

Examples at the original fractional assignment: a Boxplot box edge changes
`117.8896484375 -> 117.890625`; a Violin inner cap/box edge changes
`149.130859375 -> 149.1318359375`; a sample strip point changes
`504.705078125 -> 504.7041015625`; an Interaction triangle vertex changes
`155.2021484375 -> 155.203125`. The complete operand audit is
`t5a6d-mark-svg-audit.log`. The temporary oracle was removed; there is no retained
alternate projection implementation. Full current goldens remain in the two
extraction tests. Added contracts check quartiles, medians, observations, density
support, sparse intervals, and actual tick/grid alignment at odd quantum origins
and spans, including exact preservation of raw category and caller tick labels.

### Current Work

`paired_resources` and `publication_resources` now record all six resource fields:
records, processing visits, font/source/scratch bytes, measurement calls, cache
entries, and topology slots. **40 paired case/policy profiles plus nine publication
controls, three samples each (147 records)** assert resource stability, all 32
counters, capture counters, lifecycle/producer/shape counts, and complete
root/guide/gutter/intrinsic/child traces, excluding durations. Publication controls
also retain their three-sample SVG/PNG/decoded hash checks. Capture counters are
all zero in this run. Resource totals are admitted work, not live-memory peaks;
measurement calls are not interchangeable with root rounds or shapes.

| Fixture/policy | Rounds / trials / body checks | Proposals / exact trials (= shapes) | Measurement calls / cache entries / topology slots |
| --- | ---: | ---: | ---: |
| Small Auto/right | 2 / 2 / 2 | 0 / 27 | 256 / 69 / 10 |
| Starvation Auto, top | 2 / 4 / 2 | 10 / 31 | 230 / 68 / 8 |
| Starvation selected top | 2 / 2 / 2 | 0 / 25 | 218 / 65 / 8 |
| Starvation right, expected refusal | 2 / 3 / 1 | 10 / 31 | 173 / 57 / 8 |
| Historical reflow Auto/right | 3 / 3 / 3 | 0 / 111 | 480 / 151 / 40 |
| Preferred-right control Auto/right | 2 / 2 / 2 | 10 / 31 | 212 / 62 / 8 |
| Native compact Auto/right | 2 / 3 / 2 | 6 / 45 | 232 / 86 / 20 |
| Adequate Canvas | 2 / 2 / 4 | 24 / 52 | 1353 / 120 / 19 |
| Pressured Canvas, top | 3 / 7 / 6 | 72 / 133 | 1285 / 150 / 55 |
| Adequate facets Auto/right | 2 / 2 / 4 | 0 / 29 | 1189 / 108 / 10 |
| Pressured facets Auto/right | 3 / 4 / 6 | 148 / 178 | 999 / 150 / 28 |

All **seven genuine Auto/selected-edge pairs** retain outlined SVG, preserved SVG,
encoded PNG, and decoded metadata/pixel equality. Cache-disabled reflow/facet
controls also pass. The original 520x240 compact fixture still rejects preferred
right guide-only and accepts compact right in round 1, with eight complete labels,
16 marks and two populated columns. No replacement fixture was needed. Its Auto
root/end-to-end medians are **1.266100 / 23.499500 ms**, selected-right medians
**1.252200 / 23.464300 ms**. Bottom controls and expected no-artifact refusals
remain separate, not equivalent shortcuts.

Native panel intrinsic counts are one per standalone panel, two per shared
fixture; root-title intrinsic count is one. Standard coordinate measurements are
4 for successful two-round outside cases, 3 for the right-side refusal, 6 for
three-round historical reflow, 20 for adequate shared fixtures, 30 for pressured
Canvas and 26 for pressured facets. These include actual probes, not an assumed
one-measurement-per-round rule. End-to-end shared
child hits/replays remain zero, with 16/24 adequate/pressured misses; controlled
identical-input reuse remains 3 hits / 4 misses / 1 replay for both owners.

| Publication control | Rounds / captures | Shapes | Measurement calls / cache entries / topology slots |
| --- | ---: | ---: | ---: |
| scatter100 | 2 / 1 | 13 | 218 / 92 / 103 |
| long_label_bar | 2 / 1 | 54 | 134 / 44 / 6 |
| caption200_1para | 2 / 1 | 257 | 383 / 109 / 104 |
| caption200_3para | 2 / 1 | 259 | 383 / 113 / 104 |
| facets2x2 | 2 / 1 | 17 | 1241 / 77 / 104 |
| mixed_canvas | 2 / 1 | 63 | 442 / 123 / 108 |
| matrix4 | 1 / 1 | 11 | 662 / 82 / 2 |
| separate10 | 20 / 10 | 130 | 2180 / 920 / 1030 |
| reuse10 | 2 / 1 | 13 | 218 / 92 / 103 |

For scale, compact Auto records **4559 records / 132897 visits / 28639434 bytes**;
pressured Canvas **22216 / 677677 / 39440402**; scatter100
**5149 / 108349 / 33881255**. `separate10` charges ten operations, while
`reuse10` retains one operation's resources despite ten SVG/PNG publication pairs.
Increases as well as decreases in shape counts above are measured, not replaced
with a blanket claim that retained intrinsics reduce every workload.

Other groups retain their correctness gates: Heatmap long Elide is still 400
prefixes (200 in each round), ambiguous Elide refuses at 96 without artifacts,
and complete long/shared-prefix Wrap now measures 58/74 shapes. Heatmap intrinsic
count is one; its specialized legacy-panel constructor counts are genuinely
4 for success and 1 for ambiguous refusal, not asserted to be native zero.
Matrix keeps one root round, 2 local passes (4 for tight long labels), 12 tick
sources/lists and 72/36 adequate/tight metric lookups; tight long labels measure
162 proposals / 132 shapes and 270 calls / 58 cache entries / 2 topology slots.
Inside sparse/dense still has 28/2005 geometry scans and two annotation comparisons.
Paragraph first/repeat counts remain 0/1/1 versus zero (short), 99/44/44 versus
zero (multiline). Significance retains 1 growth, 64 bisections and 66/260/521
level checks. Synthetic coverage counts and system 457-entry/nine-candidate
discovery are unchanged; observer overhead remains noisy, not a speedup claim.

**Remaining failures in the requested scope: none.** Broader repository
verification, integration snapshots, phase completion, and commits remain outside
this audit's acceptance claim.

### Final Numerical Follow-Up (2026-09-14)

This follow-up tests the final categorical-band projection, budgeted Log10,
overflow-safe extrapolation, and revised source pricing. It supersedes the
preceding current-checkout resource totals, not their historical evidence.
Production code, existing scientific inputs/limits, label sources, and semantic
coordinate expectations were not changed by this follow-up. No `just verify`,
commit, or production test-helper change was needed.

#### Parent SVG Audit

The initial parent run reproduced four failing fingerprint tests. The existing
historical worktree hooks then rendered the unchanged owned fixtures: all 20
Beeswarm and eight Marginal fingerprints matched their historical goldens, as did
all 18 Waterfall fingerprints. Interaction's four historical native-root SVGs
were reproduced by restoring only the triangle x vertices calculated from the
former unrounded centers; both full byte lengths and FNV-1a hashes matched.
The worktrees' production sources were read-only. Temporary capture/comparison
code was removed; no alternate renderer or golden-update switch remains.

Parsed comparisons of **all 50 complete preserved SVGs** found **28 byte-identical
cases and 22 intentionally changed cases**. Every element count/order, text value,
style/color/opacity/font attribute, clip rectangle, and panel rectangle matched.
There were **99 coordinate operands changing by exactly 1/1024 px**, plus **16
larger endpoint-label x-anchor changes**. Thus **not every change is one quantum**:

| Family | One-quantum operands | Larger endpoint-label anchors | Intentional geometry |
| --- | ---: | ---: | --- |
| Beeswarm | 50 | 4 | Zero-line/mark alignment; mixed Heatmap category tick/grid centers; endpoint anchors |
| Waterfall | 29 | 8 | Exact band/mark coordinates and canonical title midpoint; endpoint anchors |
| Marginal | 16 | 4 | Numeric mark/grid alignment and title midpoint; endpoint anchors |
| Interaction | 4 | 0 | Triangle vertices constructed after canonical center projection |

The 16 larger shifts are **0.83984375 through 1.6181640625 px**, exclusively on
numeric endpoint labels in Beeswarm case 1, Waterfall cases 1/7, and Marginal
case 3. They follow the already-approved actual-ink/quantized-anchor placement in
`layout/panel/cells.rs`, replacing the former symmetric extent clamp, not changed
tick positions, labels, font styles, or scientific cells. For example, Beeswarm
Publication `-1` moves `87.5478515625 -> 86.4189453125`, while its zero mark/line
moves `389.986328125 -> 389.9873046875` to match the existing zero tick. Tests of
touching-ink containment and candidate/frozen placement agreement also pass.

Only the 22 changed current goldens were replaced. Their complete historical
provenance is retained below as **byte length / FNV-1a**; unchanged goldens remain
in the tests. This records a geometry correction, not historical byte parity.

| Historical fixture/case | Publication | Science |
| --- | --- | --- |
| Beeswarm `0a41f1c`, 1 | `10379 / 83d5eca486d917d1` | `10226 / 1144714c8a32166c` |
| Beeswarm `0a41f1c`, 2 | `10951 / bea109825dc2c961` | `10796 / bd9e9735ded8a2e9` |
| Beeswarm `0a41f1c`, 9 | `16222 / 9edbb760dcfa349d` | `15082 / 67a240d1a084097d` |
| Waterfall `0e8c344`, 0 | `11635 / 39b89057ec91b04b` | `11459 / 70f36f878cd864b2` |
| Waterfall `0e8c344`, 1 | `13598 / 8f329eda9944aa4b` | `13382 / 85837f290de55d6a` |
| Waterfall `0e8c344`, 2 | `9303 / 268aaf9ae942bc06` | `9167 / 0e248ebbcbeacf4f` |
| Waterfall `0e8c344`, 7 | `21510 / 349acc3e0230408e` | `19759 / 63688b079b6c7e4a` |
| Waterfall `0e8c344`, 8 | `11882 / 7ec8c493f7437d22` | `11703 / e6fb3656ea16f6cc` |
| Marginal `13ce394`, 0 | `21380 / 9bcc78574b22cb3d` | `20933 / 55db04f2b5752211` |
| Marginal `13ce394`, 3 | `20234 / 47391d40cb3505d0` | `19913 / 2c3ac60f7bab067e` |
| Native Interaction `31fb8a2`, styled | `11253 / ea4e8528961fec9a` | `10860 / 05f00aad0c71f249` |

The prior seven extraction goldens still pass unchanged. The odd-quantum
Interaction contract now independently pins all three categorical centers to
`[162135, 322562, 482990]` CSS quanta and checks every center against actual tick
and grid SVG. The middle is an exact-band nearest-even tie; treating it as a
numeric u64 projection key would incorrectly produce `322563`.

#### Final Measurements

All 10 ignored release groups and all normal benchmark contracts pass with the
existing full counter/trace/resource stability assertions. There are still 147
resource records across 49 case/policy profiles, three samples each; all capture
counters are zero. All seven Auto/actually-selected-edge SVG/PNG/pixel comparisons,
cache-disabled reflow/facets, complete compact labels/two columns/16 marks, and
expected no-artifact refusals remain intact. No round, proposal, or shape golden
needed adjustment. The genuine compact path still takes two rounds, three guide
trials, two body checks, six proposals and 45 shapes.

Pricing is not an optimization baseline: copies/reservations now charge records
and bytes rather than processing visits; searches charge examined entries and
sorts charge their input visit pass, not every comparison/swap. Other newly
admitted work can increase totals. The retained-line admission contract keeps
**zero processing allowance**, tests **0/2/3/4 available records**, and independently
tests **zero, exact-minus-one, exact, and exact-plus-one byte allowances**. It
requires three records (two points and one mark), checks refusal before publishing
any mark, and verifies exact retained operands/styles and resource usage on success.
No meaningful refusal was weakened into a zero-work assumption.

| Final fixture | Records / processing visits / bytes | Measurement calls / cache entries / topology slots |
| --- | ---: | ---: |
| Small Auto/right | 3487 / 115605 / 29286824 | 256 / 69 / 10 |
| Historical reflow Auto/right | 12584 / 281444 / 35743585 | 480 / 151 / 40 |
| Native compact Auto/right | 4615 / 133393 / 28798060 | 232 / 86 / 20 |
| Pressured Canvas | 22528 / 679369 / 39765028 | 1285 / 150 / 55 |
| Pressured facets Auto/right | 20520 / 584613 / 37565044 | 999 / 150 / 28 |
| Heatmap long Elide | 10387 / 144214 / 29030455 | 546 / 134 / 4 |
| Matrix tight long | 8643 / 126215 / 30321730 | 270 / 58 / 2 |
| scatter100 / reuse10 | 5155 / 109066 / 34039047 | 218 / 92 / 103 |
| separate10 | 51550 / 1090660 / 340390470 | 2180 / 920 / 1030 |

Selected final **minimum / median milliseconds** below use the unchanged three
interleaved samples, timing boundaries, system fonts and uncontrolled host/cache
conditions. The release run was not concurrent with another audit command. These
are local observations, **not optimization or comparable-baseline claims**.

| Fixture/policy | Root time | End-to-end |
| --- | ---: | ---: |
| Small Auto | 0.927500 / 0.948000 | 28.458001 / 28.770801 |
| Historical reflow Auto | 3.403800 / 3.590300 | 37.245201 / 37.706500 |
| Historical reflow selected right | 3.444300 / 3.575800 | 37.612201 / 37.813301 |
| Native compact Auto | 1.278600 / 1.284900 | 24.953701 / 25.169000 |
| Native compact selected right | 1.285500 / 1.290800 | 24.944301 / 25.382600 |
| Pressured Canvas | 6.651700 / 6.780900 | 34.978301 / 35.204801 |
| Pressured facets Auto, paired with right | 7.944626 / 8.004099 | 40.182893 / 40.327626 |
| Pressured facets selected right | 7.965534 / 8.095424 | 39.793542 / 39.999809 |
| Heatmap long Elide | 3.350000 / 3.379001 | 37.099401 / 37.178801 |
| Matrix tight long | 3.459500 / 3.499900 | 35.380901 / 35.846701 |
| scatter100 | 0.754434 / 0.766242 | 39.767434 / 42.939326 |
| separate10 | 7.515842 / 7.746917 | 451.934501 / 465.564242 |
| reuse10 | 0.769816 / 0.843917 | 196.155488 / 196.171304 |

Final evidence, all under `/tmp/opencode/`:

- `t5a6d-final-benchmarks-release.log`: **10 passed, 0 failed, 1025 filtered out**,
  8.79 seconds; complete min/median records and all counters/resources/traces.
- `t5a6d-final-owned-contracts.log`: **26 passed, 0 failed, 10 ignored, 999 filtered
  out**, 95.14 seconds: all 10 normal benchmarks and 16 owned extraction contracts.
- `t5a6d-final-parent-contracts.log`: complete categorical **44**, interactions
  **23**, and marginal-matrix **19** integration tests passed, **86 total, 0 failed**.
- `t5a6d-final-numeric-boundaries.log`: **6 passed**, covering exact bands, native
  band installation, overflow extrapolation, Log10 work/scratch refusal, and the
  two actual-ink/candidate-placement boundaries. Strict all-target Clippy passed
  in `t5a6d-final-clippy.log`; scoped formatting/diff checks passed.
- `t5a6d-final-parent-svg-deltas.log` records every changed operand. Historical
  reproduction logs are `t5a6d-final-beeswarm-marginal-historical.log` and
  `t5a6d-final-waterfall-historical.log`; full SVG pairs remain in
  `t5a6d-parent-historical/` and `t5a6d-parent-current/`. The initial failing run
  and temporary capture log remain separate diagnostic evidence.

**Remaining real failures in this scope: none.** The larger endpoint-anchor
changes are explicitly accounted for above, not hidden as one-quantum deltas.
Full-repository verification, other integration targets/snapshots, phase status,
and commits remain outside this follow-up's acceptance claim.
