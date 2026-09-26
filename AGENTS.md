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

Run the narrowest tests that exercise the changed behavior and its affected
callers, plus strict Clippy for code changes. A change spanning several modules
does **not** by itself justify running all tests: identify the affected call
paths, run focused unit tests in those modules and representative integration
tests for the contracts they share. Expand to neighboring tests only when a
failure or an uncovered interaction gives a concrete reason. Small stages can
be committed after those checks.

Treat `cargo test --lib`, `cargo test --tests`, `just test`, and `just verify` as
full-suite commands, not routine regression discovery. Run one only for an
explicit user request, a required broad-phase closure gate (such as completing
T5b, T5c, T5d, or T5e), or a demonstrated cross-cutting failure/invariant that
cannot be validated with a bounded set of focused tests. State the specific
reason before running it; do not repeat a full suite after fixing one localized
failure when its focused test and relevant neighbors suffice. Documentation-only
changes do not require tests. Inspect snapshot changes and commit them with the
source change that intentionally alters SVG output.

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
