# Repository Guidelines

## Source Of Truth

Treat `specs/` as the architectural source of truth. Prefer root `Justfile`
recipes over ad hoc command sequences.

## Standalone Boundary

`lichtung-plot` is a standalone scientific renderer. It must not depend on,
import, or dispatch into any other `lichtung-*` crate. It owns its errors,
visual annotation types, rendering lifecycle, output contracts, and font
policy.

Do not add network access, asset provisioning.

Do not add Python files or a `tools/` directory. Keep development and test
support within the existing Rust project structure.

## Current Contract

Keep one current API and representation. When replacing a project-owned API,
migrate repository callers and remove the old implementation rather than
adding aliases, version selectors, fallback chains, or deprecations.

Use modern Rust module layout: `foo.rs` plus `foo/` siblings. Do not add
`mod.rs` files. Keep public modules limited to deliberate extension points.

## Testing

Run the narrowest relevant check while iterating. Before completing a change,
run `just verify` if you modified code. If the change only affects documentation,
do not run the test suite. Snapshot changes must be inspected and committed with
the source change that intentionally alters SVG output.

Compiler validation must use `cargo clippy --locked --offline --all-targets --
-D warnings` (or `just lint`), never `cargo check`. Pass this requirement to every
subagent; a successful non-linting build is not a substitute for strict Clippy.

## Commits

Use Conventional Commits with mandatory scopes. Keep commits to one logical
change.

Before any Git push, satisfy the T5a3p1 corpus-history cleanup gate in
`specs/91_IMPLEMENTATION_PLAN.md`. Do not publish a ref that retains the full
Unicode conformance corpus in its ancestry; a forward deletion is insufficient.

After achieving a phased objective, commit the completed work before moving on.
Do not leave finished work uncommitted unless the user explicitly asks not to
commit.
