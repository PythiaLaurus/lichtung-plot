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
focused tests and inspection of affected snapshots. Update snapshots only when
the new output is intentional and consistent with `02_RENDERING_CONTRACT.md`.

## Test Layout

Integration tests are independent Cargo test targets declared in `Cargo.toml`.
Each target lives in `tests/<domain>/suite.rs`, with sibling modules for one
cohesive plot family or rendering contract. This keeps targets independently
runnable without scattering loose Rust files across `tests/` or routing every
test through one monolithic integration crate. Shared test-only helpers live in
`tests/support/`. Visual snapshots live beside the visual-regression target that
owns and reviews them.

The current focused targets are:

```text
annotations       categorical       composition
distributions     estimates         heatmap
interactions      layout            lines
marginal-matrix   output            rendering
statistics        visual-regression
```

Run one target with `cargo test --test <target>`. New tests belong in the target
whose public behavior they exercise. Add a target only for a genuinely distinct
contract; do not create chronological, catch-all, or `general` suites.

## Snapshot Tests

The five committed snapshots are generated directly by tests in
`tests/visual_regression/suite.rs`. Those tests construct fixed data, publish the
canonical preserve-text SVG with `Plot`, and pass either the bounded SVG or a
small clipping/layer summary to `insta::assert_snapshot!`. No script, Python
process, or external plotting library generates them.

Run them with:

```text
cargo test --test visual-regression
```

On a missing or changed baseline, Insta writes a `.snap.new` candidate and the
test fails. Review intentional changes with `cargo insta review`; never accept a
snapshot merely to make CI green. Snapshots are tracked because CI needs an
approved expected output to compare against on a fresh checkout.

Snapshots are regression baselines, not independent correctness oracles.
Semantic behavior, validation errors, geometry bounds, and numeric properties
remain covered by explicit assertions in the focused targets. Keep snapshots
small and reviewable; broad multi-plot manifest snapshots are prohibited.
