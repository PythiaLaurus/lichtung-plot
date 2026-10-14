# Development

**Status**: Canonical

## Toolchain

The repository pins Rust 1.96 with rustfmt and Clippy in
`rust-toolchain.toml`. `Cargo.lock` is tracked.

## Commands

Use the root recipes:

```text
just fmt
just lint
just test
just docs
just package
just verify
```

`just verify` is the complete read-only gate. It checks formatting, compiles and
lints every target with warnings denied, runs tests, builds documentation, and
verifies Cargo's independently unpacked package.

Compiler validation uses `just lint`, which runs
`cargo clippy --locked --offline --all-targets -- -D warnings`. Do not substitute
`cargo check`, including in delegated work. Targeted tests remain useful while
iterating, but do not replace strict Clippy or the final verification gate.

Do not add Python files or a `tools/` directory. Development/test support stays
within the existing Rust project structure. Unicode fixture provenance belongs
beside the fixtures, not in a separate tooling tree.

## Dependency Changes

Dependencies are declared directly in `Cargo.toml`; workspace inheritance and
path dependencies are prohibited. A dependency change updates and commits
`Cargo.lock`, then runs `just verify`.

No dependency may introduce application-owned analysis, report, runtime,
network-provisioning, or first-party Lichtung behavior into the renderer.

## Commit Messages

Use Conventional Commits with a scope and describe the actual behavior,
architecture, contract or documentation change. Both the subject and body must
remain understandable after temporary planning documents and stage labels are
removed.

Do not include planning identifiers or numbered implementation stages in commit
subjects or bodies. Documentation changes follow the same rule. Use descriptions
such as `docs(api): map public cutovers and consumers` or
`refactor(color): retain validated foundational paint values`, rather than naming
an internal milestone. Review the complete message before committing.

## Visual Changes

Changes to layout, themes, text, SVG serialization, clipping, or marks require
focused semantic checks consistent with `02_RENDERING_CONTRACT.md`. Inspect source
completeness, scientific values, effective styles, clipping, containment and
relative text/mark relationships when those contracts are affected.

## Test Layout

Integration tests are independent Cargo test targets declared in `Cargo.toml`.
Each target lives in `tests/<domain>/suite.rs`, with sibling modules for one
cohesive plot family or rendering contract. This keeps targets independently
runnable without scattering loose Rust files across `tests/` or routing every
test through one monolithic integration crate. Shared test-only helpers live in
`tests/support/`.

The current focused targets are:

```text
annotations       categorical       composition
distributions     estimates         heatmap
interactions      layout            lines
marginal-matrix   output            rendering
statistics
```

Run one target with `cargo test --test <target>`. New tests belong in the target
whose public behavior they exercise. Add a target only for a genuinely distinct
contract; do not create chronological, catch-all, or `general` suites.

## Stable Test Expectations

The user rejected whole-scene byte/geometry baselines as volatile implementation
details. Do not assert a stored SVG byte length/hash, a full SVG or clipping-summary
snapshot, or incidental automatic panel dimensions and placement coordinates.
Those tests and the snapshot target/tooling have been removed. Historical output
hashes and reviewed snapshots in evidence documents describe their recorded
revisions; they do not impose current test expectations.

Keep checks with an independent contract:

- Explicit caller-requested root/PNG dimensions and output-format limits.
- Scientific values, source rows, visible mark counts and complete required text.
- Containment, non-overlap, linked-axis agreement and selected-style ownership.
- Validation failure categories and absence of published output on failure,
  without fixing an incidental internal failure location or diagnostic string.
- Exact numerical transformations/rounding with independent mathematical
  expectations, and narrowly specified serialization syntax/escaping.
- Equality of outputs produced by equivalent current routes, cache settings or
  immutable replay. This compares both sides in the same implementation rather
  than freezing an old revision's output.

If a fixture only checked a volatile baseline, remove it. Mixed fixtures keep
their meaningful assertions. Do not replace deleted goldens with measured
coordinates, broad tolerances around those coordinates or redundant smoke tests.
Benchmark hashes/byte sizes may remain observations for a specific comparison;
they are not permanent cross-revision acceptance gates.
