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
