# Semantic test simplification: implementation evidence

Baseline: `1208316` / the `f417320` Rust census, 156,936 Rust code lines;
75,966 test-related lines. The approved scope is
`specs/96_TEST_CODE_SIMPLIFICATION_PLAN.md`. Original feature work remains paused.

## Remove self-confirmation and obsolete quota smoke

Removed the identical Histogram expected/actual loop, the Regression range
self-comparison and unused buffers, the no-cap native-route smoke matrix, the
literal enum inequality and the copied cache binary-search comparison witness.
Removed discarded fresh-operation preparation/project calls from mixed semantic
fixtures, retaining actual data/order/geometry/attachment and cache collision
assertions. Existing independent statistics and public output checks remain.

Formatting and strict Clippy passed. Forty focused tests passed, covering the
retained semantic fixtures, Histogram internal/supplied maxima, Regression styles,
route classification and actual cache/provenance behavior. Full suites were not
run for this localized change. Rust cloc: **156,936 → 156,816**, net **120 code
lines removed**. Logs: `/tmp/opencode/test-cleanup-first-{lint,tests}.log`.

## Consolidate frozen replay and representative parent coverage

`orchestrator/tests/lifecycle.rs::every_canonical_plot_kind_renders_through_one_lifecycle`
remains the common proof for all 25 families: one capture/Null lifecycle, no
deferred work in capture, dropping the source before SVG/outline/PNG output and
no source reads/projection/shaping/preparation during frozen output. Removed
duplicate full prepare/census/poison/capture/replay harnesses for Bars, Estimates,
Dumbbell, categorical distributions and distribution/reference families.

Producer-specific source values, order, statuses, styles, signed stacks and
projection/ink tests remain. Mixed Marginal/Matrix/Beeswarm/Interaction/Waterfall
fixtures retain their linked axes, source counts, labels and drawn-ink checks;
their repeated generic poison/counter tails are removed. Scatter, annotated Line
and specialized Heatmap still exercise direct selected-body poisoned inputs;
the core frozen-command tests remain. Source-drop output and direct retained-body
poisoning are thus both represented without copying the harness into every family.

All-family dispatch remains once as a root, with representative mixed Canvas
children (coordinate/categorical/grid/distribution), the Matrix nesting refusal,
numeric/categorical root sizing and numeric/categorical/supplied-geometry child
reuse. Removed all-family multiplication of generic natural-size/cache trials.
Actual dirty Canvas/Facet neighbors and relevant/unrelated feedback tests remain.
Also removed a migration-era distribution projection self-comparison and repeated
context-fallback rejection already covered by the common exact-source test.

Strict Clippy/formatting passed. Eleven focused replay/geometry/root checks
(including the 25-family fixture), four projection/exact-source checks and three
public Bar/overlay/estimate-status integration checks passed. Rust cloc:
**156,816 → 155,642**, net **1,174** code lines removed in this change, **1,294**
since baseline. Logs: `/tmp/opencode/test-cleanup-{replay,parent}-*.log`.

## Remove representation witnesses and fixed benchmark traces

Removed Canvas/Plot theme-address and stage-string logs, the theme-resolution
counter and their assertion helpers. Mixed-child paints, root/header/TextPanel
typography, sparse source cells, surface roles, preflight versus publication and
real public theme-inheritance output remain checked by values rather than object
addresses. Removed the scene-buffer address/route log; chained SVG/PNG output
still checks actual bytes, call-local options, one scene and unchanged font work.

Removed the complete-theme Arc/header-sharing inventory and the test-only ratio
sharing bridge. Component tests retain independent field edits, immutable sibling
values, validation/absence, exact rounding and effective styles. The unused guide
relationship inventory and test-only lookup/logging code were deleted with their
last consumers.

Legend measurement cases retain complete labels/marks, explicit-edge behavior,
bounded root coordination, cache event consistency and current-route output
equality. Removed fixed shape/trial/proposal tuples and full historical traces.
Repeated benchmark observations are reported rather than frozen; output hashes
and real PNG decoding remain in the measurement owner. Removed redundant decoding
of a second byte-identical PNG in the equality helper.

Strict Clippy/formatting passed. Thirty-four focused witness/benchmark/schema
checks passed across the initial run and localized rerun; six public theme-owner
checks and sixteen additional paint/axis/measurement checks passed. The sole
localized failure was an ambiguous test label: a row header named identically to
an axis title selected the axis text. A unique caller-provided header fixed the
oracle; no renderer behavior changed. Rust cloc: **155,642 → 154,526**, net
**1,116** code lines removed, **2,410** since baseline. Logs:
`/tmp/opencode/test-cleanup-{witness,schema}-*.log`.

## Consolidate public output and API compile ownership

`tests/output/multiformat.rs` now owns the combined Plot/Canvas outlined versus
complete selectable text, explicit dimensions and same-Output replay contract.
PNG default/repeat equality is part of the call-local preview/custom/publication
case rather than a separate smoke. Invalid scale, non-finite/zero DPI and
excessive target size share the existing PNG refusal owner; actual error class
and no artifact remain checked. Suffix completion, replacement, existing-file
preservation, staging cleanup and OS-specific cases remain in `output/paths.rs`.
Removed the extra ScatterData wrapper, temp-directory loop and PNG parsers in
favor of the existing shared support, and a duplicate default-theme render test
already owned by `composition/themes.rs`.

Merged constructor, contextual setter and significance compile-pass inventories
into the normal external `rendering/public_contracts.rs` target. It retains all
25 exact family markers, both Histogram/Raincloud input forms, all ten supported
significance family/placement combinations and erasure-kind agreement. Removed
the internal duplicate inventories and their helpers. Actual erased output now
also checks a selected nondefault theme, complete title and requested dimensions;
mixed typed Canvas acceptance is exercised by real composition publication.

`just test-doc` runs `cargo test --locked --offline --doc` and is an explicit
dependency of `just verify`. `just test` still runs target tests; documentation
builds are not described as running API examples. Canonical development guidance
records normal-library E2E, decoded Heatmap output, positive typing, negative
doctests and the separate internal immutable-replay witness owners.

Formatting and strict Clippy passed. All 21 output checks, two affected public
typing/erasure checks, the retained default-theme owner and 41 Plot compile-fail
doctests passed. Rust cloc: **154,526 → 154,243**, net **283** code lines removed,
**2,693** since baseline. Logs: `/tmp/opencode/test-cleanup-public-*.log`.

## Final census and closure

The approved source-reviewed consolidation is complete. The final Rust state is
`bf12d23`; subsequent closure edits only document results. The original
[`test-code-census.csv`](test-code-census.csv) remains the baseline. The separate
[`test-code-census-after.csv`](test-code-census-after.csv) records all 245 tracked
Rust files after cleanup, with the same AST-span/cloc 2.06 classification.

| Category | Baseline code | Final code | Net removed |
| --- | ---: | ---: | ---: |
| Handwritten implementation, syntactic classification | 67,852 | 67,846 | 6 |
| Separate generated Unicode tables | 13,017 | 13,017 | 0 |
| Test-only source, helpers and hooks | 59,627 | 57,263 | 2,364 |
| Ordinary integration targets | 13,174 | 13,124 | 50 |
| Shared integration support | 473 | 473 | 0 |
| Benchmark/diagnostic support | 2,692 | 2,419 | 273 |
| Examples | 101 | 101 | 0 |
| **Total** | **156,936** | **154,243** | **2,693** |
| **Test-related subtotal** | **75,966** | **73,279** | **2,687** |

The six syntactically classified implementation lines were four lines of an
unfulfilled Canvas lint attribute and two test-only resolution-hook lines inside
the theme-definition macro. This is not a production algorithm reduction. The
whole and partition sums reconcile exactly: `154243 == 154243`, difference zero.
Test-related code is now about 47.5% of Rust code, versus 48.4% at baseline. The
earlier 5–10k estimate was too high for the completed consolidation; the actual
net is 2,693 code lines, including 2,687 test/support lines.

| Execution / inventory | Baseline | Final |
| --- | ---: | ---: |
| Linux library tests passing | 1,345 | 1,332 |
| Linux library tests intentionally ignored | 17 | 17 |
| Linux integration tests passing, 13 targets | 470 | 462 |
| Doctests executed by complete gate | Not included | 53 passing |
| AST named library-side cases, all platform branches | 1,367 | 1,354 |
| AST named integration cases, all platform branches | 472 | 464 |

Final AST library-side cases comprise 1,332 source cases plus 22 benchmark cases.
The ignored set remains seven external Unicode/regeneration cases and ten
release measurements. No case was newly ignored. Four example targets compile
in `--all-targets`; they contain no test bodies. Counts distinguish compilation
from execution and do not assert a coverage percentage.

One final `just verify` passed: formatting, strict all-target Clippy, all 1,332
library and 462 integration checks, 53 doctests, warning-denied API docs and
Cargo's independently unpacked package verification. The 53 doctests comprise
five executable examples and 48 compile-fail boundaries. This single broad run
closes the completed cross-owner consolidation and the newly explicit doctest
gate; earlier changes used focused checks. Final log:
`/tmp/opencode/test-cleanup-final-verify.log`. The census summary is
`/tmp/opencode/test-cleanup-final-summary.log`; masked production/testing counts
and whole-file JSON are `/tmp/opencode/test-cleanup-*-cloc.json`.

### Retained contract owners

| Contract | Current main owner |
| --- | --- |
| All 25 families and frozen output without source/preparation work | `orchestrator/tests/lifecycle.rs::every_canonical_plot_kind_renders_through_one_lifecycle`, core frozen-command tests and retained direct Scatter/Line/Heatmap poison fixtures |
| Independent scientific values, source order/status and special geometry | Existing statistics/numeric/Unicode oracles, producer-specific ink checks and scientific integration targets |
| Complete text, SVG representations and reusable Plot/Canvas files | `tests/output/multiformat.rs`; internal `chained_terminals_reuse_one_canonical_scene_and_plan_each_png` owns the one-scene/no-new-font-work evidence |
| PNG options, aspect ratio, limits and actual decoded colors | `tests/output/png.rs`, call-local multiformat case and `tests/heatmap/suite.rs::constant_scale_mark_and_guide_stops_agree_with_decoded_png_pixel` |
| Suffixes, replacement, failure preservation and staging cleanup | `tests/output/paths.rs` and artifacts unit tests |
| Parent ownership, sparse cells, shared domains/guides and actual cache invalidation | Composition targets, Canvas/Facet lifecycle checks and child-key/feedback tests |
| Normal-library exact constructors/contextual capabilities/erasure | `tests/rendering/public_contracts.rs` |
| Negative visibility, exact/Any and unsupported-neighbor API boundaries | Rustdoc cases executed by `just test-doc`, included in `just verify` |

The current README also replaces the obsolete million-visit admission paragraph
with the implemented ownership/checked-boundary contract and current evidence.
Further foundation/public feature implementation remains paused for the user's
next direction.
