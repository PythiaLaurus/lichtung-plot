# Test-code audit: size, coverage and avoidable confirmation

## Main findings

**Baseline: `f417320`, after the accounting tail cleanup and complete-caption
performance report.** The new test cleanup is a proposal, not an implemented
deletion campaign. The preceding volatile-output cleanup is already complete.

1. There is substantial test weight: **75,966 Rust code lines**, about **48.4%**
   of tracked Rust code, including test-only helpers and observations.
2. The project already has **in-process library end-to-end tests**. Cargo's
   unit/integration compilation categories do not describe their semantic depth.
3. Some current tests are demonstrably tautological or repeat an unchanged
   successful operation after the old quota purpose disappeared.
4. Larger potential reductions are in repeated producer/capture/parent fixtures,
   private implementation witnesses and benchmark acceptance tuples. These are
   mixed with valuable contracts and need selective consolidation.
5. Independent numerical/Unicode evidence, source completeness, real failure
   behavior and a small public-output end-to-end backbone should be retained.

## Rust size: what the 157,006-line concern consists of

Current `cloc` 2.06 counts **156,936 Rust code lines in 245 tracked Rust files**.
This is close to the user's observed 157,006; it excludes comments, blanks,
`target/`, dependency source and temporary measurement/audit worktrees.

| Source category | Rust code lines |
| --- | ---: |
| Handwritten production implementation, excluding test-only spans and separate generated tables | 67,852 |
| Separate generated Unicode table files | 13,017 |
| Test-only code inside/reachable from `src/`, including helpers and hooks | 59,627 |
| Ordinary integration targets under `tests/` | 13,174 |
| Shared integration helpers | 473 |
| Existing benchmark/diagnostic support | 2,692 |
| Examples | 101 |
| **Total** | **156,936** |
| **Test-related subtotal** | **75,966** |

`tests/` alone contains 16,339 code lines. Counting only that directory would miss
most test-related code. `tests/benchmarks` is actually included into the **library
test executable** by `src/orchestrator/tests.rs:25`, not declared as an independent
integration target.

The handwritten production implementation is itself sizeable. This audit does
not establish that all 156,936 lines are unnecessary or that test deletion alone
will reduce the renderer to an unspecified original size. It does establish a
large, concrete cleanup surface without changing scientific algorithms.

### Largest source test families

| Family, including adjacent root file where applicable | Test-related code lines |
| --- | ---: |
| `src/plots/` | 14,735 |
| `src/layout/` and `src/layout.rs` | 12,258 |
| `src/orchestrator/` and `src/orchestrator.rs` | 9,899 |
| `src/decorations/` | 5,610 |
| `src/theme/` and `src/theme.rs` | 4,541 |
| `src/render/` | 4,192 |
| `src/canvas/` and `src/canvas.rs` | 3,605 |

Large individual test spans include `canvas.rs` (2,458 code lines),
`decorations/plan/tests.rs` (2,189), `orchestrator/tests/lifecycle.rs` (2,143),
`render/svg_backend.rs` (1,934) and `theme/resolved.rs` (1,816). Size prioritizes
review; it is not an automatic deletion decision.

## Compilation category versus actual coverage

The latest complete gate ran 1,345 library tests and 470 integration tests
successfully; 17 library tests are intentionally ignored. Listing the current
library executable reports 1,362 tests including those ignored tests. Static AST
inventory finds 1,367 library-side and 472 integration `#[test]` functions across
all platform branches; conditional compilation accounts for the different
Linux executable counts. Static inventory is not a new run result.

The 17 ignored cases comprise **seven external Unicode conformance/regeneration
checks and ten release measurement cases**. They are not seventeen failures.

| Layer | Actual examples | Interpretation |
| --- | --- | --- |
| Local unit/oracle | `src/numeric/tests.rs`, Unicode focused conformance, statistics, coordinate rounding | Small independent mathematical or validity checks |
| Internal component integration | prepared axes, native interiors, feedback, capture with Recording/Geometry backends | Several owners collaborate; often compiled as library tests |
| Public in-process end-to-end | `tests/output/{multiformat,png,paths}.rs`, composition/rendering targets | Caller values → preparation/fonts/layout/capture → real SVG/PNG conversion → file publication/readback |
| Decoded-output end-to-end | `tests/heatmap/suite.rs::constant_scale_mark_and_guide_stops_agree_with_decoded_png_pixel` | Real PNG frame decoding and color agreement using positions obtained from the emitted SVG |
| Test-build end-to-end with internal witnesses | `src/orchestrator/tests/lifecycle.rs::chained_terminals_reuse_one_canonical_scene_and_plan_each_png` | Real public terminals plus counters proving no scene/shape reconstruction |
| API compile/doc tests | `lib.rs`, `builder.rs` and other public documentation | Consumer visibility/capability boundaries; neither rendering E2E nor meaningless simply because runtime assertions are absent |

Concrete public-output examples include reusable Output equality, SVG text versus
paths, call-local PNG sizing/density, actual file replacement, preservation of an
existing artifact on failure and no staging residue. These are library E2E
contracts. There is no separate app/CLI to drive and no dedicated `e2e` target;
adding a new runner or directory merely to obtain that label is unnecessary.

The external Cargo integration targets compile the library **without its
`cfg(test)` implementation**. This matters because library tests can use
synthetic fonts, injected backends and relaxed test-only preparation paths.
Replacing all external output tests with internal mocks would lose that boundary.

There is a real orchestration gap: `Justfile::test` uses
`cargo test --all-targets`, which does not run doctests; `just docs` builds docs
but does not execute their examples. The compile-fail documentation must not be
described as validated by those two commands. The proposal addresses its place
in API validation without inventing another test framework.

## Concrete low-value or mixed confirmations

| Location / test | Observation | Recommended disposition |
| --- | --- | --- |
| `src/plots/histogram.rs:897` `histogram_domains_and_kde_maxima_match_mark_preparation` | Nested value/bin/stat loops call **the identical `prepare_histogram` function with identical operands** for expected and actual | Delete that loop; separately review/retain the curve/maximum branch's meaningful relationship |
| `src/plots/regression.rs:77` `regression_axis_channel_and_fit_preserve_source_results` | Calls `axis_ranges` twice unchanged and reserves x/y buffers that never participate in any assertion or result | Delete; genuine OLS/band/range evidence exists elsewhere |
| `src/layout/interior/coordinate/tests.rs:871` `every_native_route_admits_intrinsics_and_selected_candidates_at_exact_resource_caps` | No resource caps remain; simply repeats each successful intrinsic/measurement call without examining either result | Delete this obsolete smoke matrix; preserve meaningful intrinsic and selected-payload tests |
| `src/plots/{scatter.rs,line/tests.rs,heatmap/extraction.rs,matrix.rs,marginal/extraction.rs,waterfall/extraction.rs,bar_data/extraction.rs,interval_estimate/extraction.rs}` semantic fixtures | Extra fresh-operation preparations/project calls are discarded, remnants of exact-quota success checks | Remove the redundant invocations; retain scientific values/order/source identity and genuine repeated-cache checks |
| `src/layout/interior/tests.rs:57` | Compares two directly written different enum variants | Delete the literal inequality; retain route/family rejection where it tests a real boundary |
| `src/theme/resolved.rs:3037` `indexed_text_queries_preserve_matching_and_collision_order` | Reimplements a binary search in the test and counts **that copied loop**, not the production lookup | Remove the copied-search price/witness; retain actual hit, collision and provenance invalidation checks |
| `tests/benchmarks/legends.rs:248` `assert_outside_trace` | Fixes exact rounds, shaping/proposal counts and a complete internal trace for specific figures | Convert to observations plus complete fit, explicit placement, bounded coordination and equivalent-current-route checks |
| Repeated poisoned-capture fixtures in plot extraction modules and `layout/interior/*` | Each rebuilds similar preparation/counter/poison/replay apparatus | Consolidate shared immutable-replay proof while keeping producer-specific source/ink/style facts |
| Theme edit/schema tests, e.g. `src/theme/composition/tests.rs` | Some branches manually assign a private field in a clone and compare to the setter result; mixed with valuable sibling/immutability checks | Remove storage echoes and repeated validation witnesses only where a retained constructor/consumer contract covers them |
| `src/builder.rs` marker inventory versus `tests/rendering/public_contracts.rs` contextual marker inventory | Constructor-family typing and setter-preserved typing are **related but different** boundaries; much setup repeats | Consolidate fixtures/compile ownership; do not blindly delete a zero-assert function |
| `tests/output/png.rs::scatter_png_smoke` versus multiformat publication cases | Repeats basic PNG header/dimension/density coverage | Fold into one useful public-output case; retain decoded frame and call-local conversion differences |

These examples are source-reviewed recommendations. They are not a claim that
every test in those files is useless. In particular, a function with no direct
`assert!` may invoke an assertion helper (the strict-numeric family matrix does),
and positive typing tests genuinely rely on compilation.

Protect independent oracles: the Log10 MPFR digest is a checksum of independently
specified mathematical results, not an SVG revision golden; Unicode provenance
hashes identify pinned standards data. Correct KDE normalization, Student-t
intervals, affine invariance, checked overflow, source-row identity and no-partial-
publication tests have different failure modes and remain meaningful.

## Inventory method and limitations

[The complete per-file CSV](test-code-census.csv) covers 245 tracked Rust files.
A temporary read-only Rust/syn program resolves the library module tree and
test-only external modules, including `#[path]`; it marks positive test-only
items/statements/fields and nested test bodies. Separate masked production/test
copies retain original line positions. `cloc` counts both copies, and their code
lines reconcile **exactly** with the original 156,936 (difference zero).

Shared macro definitions and conditional derives are source text, not expanded
binary sizes. The inventory is a syntactic size partition, not a claim about
compiler output, dynamic coverage, mutation detection or runtime. Named tests
include all platform branches. Automatic terminal-call markers are only review
hints: helper calls and assertions can hide terminal calls, and a malformed-scene
terminal test is not a full scientific-input E2E scenario.

No test was deleted by a classifier, assertion count or file-size threshold.
The audit helper passed strict Clippy and remains only under
`/tmp/opencode/accounting-refactor/src/bin/`; it does not edit project sources.
Raw AST/mask/cloc evidence is `/tmp/opencode/test-census*` and
`/tmp/opencode/test-cleanup-*-cloc.json`. Portable CSV was compared to the generated
inventory. No new dependency, Python file, `tools/`, corpus or test framework was
added to the repository.

Reproduction of whole-file counts:

```sh
cloc --vcs=git --include-lang=Rust --skip-uniqueness --by-file --json .
```

The requested cleanup plan and approval boundary are in
[`specs/96_TEST_CODE_SIMPLIFICATION_PLAN.md`](../specs/96_TEST_CODE_SIMPLIFICATION_PLAN.md).
