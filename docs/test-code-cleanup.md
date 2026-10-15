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
