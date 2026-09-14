# Local-Search Measurements

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
