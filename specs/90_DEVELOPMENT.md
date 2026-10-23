# Development

## Toolchain, Dependencies And Commands

Use pinned Rust 1.96 with rustfmt/Clippy and tracked Cargo.lock. Prefer root
Justfile recipes; commands are locked/offline:

| Recipe | Contract |
| --- | --- |
| just fmt / just fmt-check | Format sources / read-only formatting check |
| just lint | cargo clippy --locked --offline --all-targets -- -D warnings |
| just test | All library/integration/example targets, excluding doctests |
| just test-doc | Execute public examples and compile-fail API boundaries |
| just docs | Build API documentation with warnings denied |
| just package | Verify Cargo's independently unpacked distributable package |
| just verify | Complete read-only format/lint/tests/doctests/docs/package gate |

Compiler validation must use strict Clippy, never cargo check or a nonlinting
build as a substitute. Pass that requirement to delegated work. Documentation
build alone does not execute doctests. Do not claim a discovery-only command,
filtered zero tests or ignored case as executed behavioral verification.

Declare dependencies directly in Cargo.toml, without workspace inheritance or
path dependencies. Dependency changes update Cargo.lock and run the complete
verification gate. Dependencies must not import application analysis/report/runtime
or other lichtung-crate responsibilities, network provisioning or assets.
Keep development/test support in existing Rust structure; no Python files or
tools directory. Modern module layout is foo.rs plus foo/ siblings.

## Focused Verification

For code changes, identify affected call paths and run the narrowest meaningful
unit filters plus representative integration contracts, formatting and strict
all-target Clippy. Several touched modules alone do not justify all tests. Expand
only for a concrete failure or uncovered interaction.

Treat cargo test --lib, cargo test --tests, just test and just verify as full-suite
commands. Run one for an explicit request, a required broad implementation closure
gate, a dependency change or a demonstrated cross-cutting invariant that cannot
be checked with a bounded selection. State the specific reason first. After a
localized repair, rerun its focused case/neighbors rather than repeat a successful
whole suite without new cause. Pure documentation needs source/spec consistency,
reference checks and git diff --check, not compiler/tests by default.

## Test Scale And Ownership

| Layer | Reasonable scale | Upper limit |
| --- | --- | --- |
| Unit tests | 300–800 tests | 800 |
| Module integration, excluding core E2E | 100–300 tests | 300 |
| Core E2E | 30–80 scenarios | 80 |

The upper bounds limit the maintained suite; the lower ends are not quotas to
fill. Meaningful coverage may grow within these limits without replacing a test
in the same change or preserving a historical count. Count ignored tests too.
Review the affected layer's count when changing tests.

Classify by responsibility, not Cargo target type. A unit test exercises an
isolated calculation or policy, including public numerical functions compiled
as a Cargo integration target. A module integration test checks a specific
relationship between cooperating components, such as source rows and marks,
guide compatibility or a layout constraint. Core E2E covers representative
public workflows through preparation, layout and real publication, including
composition and frozen-output reuse. SVG inspection alone does not turn a
feature-focused integration assertion into a separate core E2E workflow.

Count independent E2E scenarios, not only their driver functions. Configuration
cross-products still count as separate scenarios; do not compress them into a
loop to satisfy a ceiling. Ignoring, merging functions, parameterized packaging,
relabeling or moving tests between suites does not evade a limit. Delete
redundant/low-value tests and remove test-only hooks/helpers when their last
meaningful consumer disappears. Do not add getter/setter inventories, repeated
per-family smoke/determinism checks, arbitrary pixel/default-style expectations
or expensive boundary-sized workloads without a distinct useful contract.

Independent Cargo integration targets are declared in Cargo.toml, each with
tests/<domain>/suite.rs and cohesive sibling modules; shared helpers are in
tests/support. Targets are annotations, categorical, composition, distributions,
estimates, heatmap, interactions, layout, lines, marginal-matrix, output, rendering
and statistics.
Run one with cargo test --locked --offline --test <target>. Add a target only
for a distinct contract, not a chronological/catch-all/general suite.

Keep normal externally compiled library-to-file SVG/PNG tests and selected decoded
pixel/format checks. They cannot all become private mocks. Retain a small internal
lifecycle witness for invariants output equality cannot prove: source/operation
ownership, no repeated statistics/font work, one selected capture/final Null and
source-independent frozen replay. Every family needs a valid public dispatch
route; extra cross-products require distinct failure surfaces, not mechanical
family x preset x root x parent expansion.

## Semantic Expectations

Test independently justified scientific/API/output relationships:

- Original rows, availability, statistical values, source order, complete marks/
  required text and raw semantic identity.
- Explicit requested canonical/PNG dimensions and real output-format limits.
- Containment, nonoverlap, linked-axis agreement, selected paint/geometry ownership,
  domain/window distinctions and truthful guide compatibility.
- Error domain/source and no-partial/existing-artifact preservation, without
  freezing incidental internal error location, Debug representation or wording.
- Exact numeric transformations, ties/carries and serialization/escaping syntax
  with independent mathematical or standard-data expectations.
- Same-operation immutable replay and equivalent current routes/cache settings,
  comparing both sides in the current implementation.

Do not freeze whole-scene SVG length/hash, scene/clipping snapshots, automatic
panel dimensions or internal placement coordinates. Do not repin such goldens,
replace them with measured coordinates/broad tolerances, or treat benchmark hashes
as cross-revision contracts. An explicit input-derived dimension, independent
rounding boundary or actual containment relation remains meaningful.

Avoid expected=actual calls to the same function/input, setter/getter echoes,
discarded repeated unwraps and copied production search/layout algorithms masquerading
as oracles. Preserve distinct missing/finite/availability/facet/style/error policies
and genuinely different mark grammars. Pointer equality is not general immutability
proof; full cache-key collision/operation identity tests remain valuable. No generic
testing DSL, mutation system or new universal fault-injection framework is required.

Numeric conversion uses independent rational/interval/root certificates, not its
own formatter as oracle. Unicode uses pinned standard data/provenance. Statistical
oracles for KDE/OLS/bootstrap and real arithmetic/length/representation failures
are independent correctness evidence, not volatile scene baselines.

## Resources And Performance

Production retains real checked lengths/products/indices/spatial arithmetic,
fallible first-party buffers, source/operation identity, sealed fonts, complete
cache equality, finite layout policies and PNG-local bounds. Do not reintroduce
unified visits/records/cache/topology/discovery/estimated-byte accounting, a global
score, budget API, quota switch or permanent no-op claim interface. Third-party
allocator behavior is not promised recoverable.

Use existing test/benchmark diagnostics only for clearly named observations:
shaping input/output, candidate/refinement counts, cache hits/misses, measurement/
capture/replay and elapsed time. Unlike units must not be added to a resource
score or interpreted as CPU instructions/memory. Counters do not impose exact
cross-revision tuples unless they directly express a real lifecycle boundary.

Performance comparisons use successful same-source workloads, same fonts/options/
format and complete output, without deleting text/points or changing algorithms
to improve timing. Distinguish cold discovery, retained Output reuse, compilation
and nested timer scopes. Measure command/process RSS independently from retained
payload or cumulative allocation traffic; neither visits nor requested bytes is
peak memory. Report actual scope and uncertainty, not unsupported speedup claims.

## Unicode Sources And Publishable History

Retain runtime tables, focused independently expected default cases, pinned hashes,
generation provenance and licenses. Full official Unicode conformance corpora
must not be stored in the repository/package, compressed into another blob or
recreated by regeneration. Explicitly invoked ignored conformance/regeneration
checks read LICHTUNG_UNICODE16_SOURCE_DIR and verify official 16.0.0 hashes before
parsing; missing/mismatched input fails. No automatic download, provisioning or
dependency-cache lookup. Regeneration checks source/table agreement without
writing the full corpus back into the tree.

Before any requested push, inspect current refs/server and prove that **every ref
intended for publication lacks the full corpus in its ancestry**. Forward deletion
is insufficient. The corpus includes BidiCharacterTest.txt, BidiTest.txt,
DerivedNormalizationProps.txt, GraphemeBreakTest.txt, LineBreakTest.txt and
NormalizationTest.txt. Preserve the published base and unrelated work; any needed
unpublished-suffix rewrite requires guarded ref updates and tree comparisons,
without a backup ref retaining the unwanted ancestry. A normal push does not
send unreachable reflog-only objects; this is not a physical-erasure claim.
Do not clear unrelated recovery history or assume force-push authorization.

Historical execution is not fresh-checkout conformance. State which focused,
external/ignored, release or package checks actually ran. Do not mark missing
external data as successful full conformance.

## Documentation And Commits

For requested capabilities with precedents in established plotting libraries,
preserve their meaningful configuration choices. Cite the actual API/behavior in
implementation planning; a conventional default is not a restriction to that one
value. Distinguish user requirements, existing repository contracts, reference-
backed capabilities and new proposals. Ask the user about product behavior with
no established basis rather than silently promote a local implementation choice
to a contract. Do not reinstate a capability the user explicitly rejected through
a different name. Implementation difficulty calls for architectural planning, not
unannounced capability reduction.

Position-generation functions do not need an independent correctness-validation
stage or a strictness switch. Unsorted, repeated or off-window positions are not
errors by themselves. Preserve drawable results through normal geometry/window
handling; report failures from the actual consumed operation or an explicit caller
constraint, rather than inventing additional input-acceptance rules.

Deferred entries require explicit user direction; renderer exclusions and agent
suggestions alone do not create future work. When a capability enters an
implementation plan, remove it from the deferred-feature inventory in the same
change. For partial admission, retain only the unadmitted scope and its review
checkpoint. Implementation status belongs in the plan until
current contracts can be updated with working behavior.

Formal specifications define current rules by subsystem and refer to actual source
owners. Low-numbered specifications reference files only beneath specs/ and src/.
Keep plans/status, execution logs, commit receipts, historical censuses and
performance samples out of formal contracts and user guides. User documentation
explains use; it is not an agent work journal. Source comments/API examples should
describe current behavior rather than migration stages. No compatibility route is
authorized by old documentation.

Use Conventional Commits with mandatory scope and one logical change. Subjects
and bodies describe enduring behavior/architecture, including documentation
changes; never temporary planning identifiers or numbered stages. Inspect status,
diff and recent style, stage only intended files, and commit completed objectives
before starting unrelated work. Push only when explicitly requested and after
the corpus-ancestry checks above.
