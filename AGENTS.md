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

Do not grow the unit-test suite. Every new unit test must be offset by deleting
an existing unit test in the same logical change. The discovered library-test
count, including ignored tests, must not exceed its pre-change count on the same
platform (the current Linux baseline is 671, including 17 ignored). Further
deletions do not create a budget for later growth. Prefer replacing lower-value
tests with meaningful coverage of the changed contract, and remove obsolete
test-only support. Do not evade this rule with ignored tests, merged test
functions, parameterized-case packaging, or migration into integration suites.

Run the narrowest tests that exercise the changed behavior and its affected
callers, plus strict Clippy for code changes. A change spanning several modules
does **not** by itself justify running all tests: identify the affected call
paths, run focused unit tests in those modules and representative integration
tests for the contracts they share. Expand to neighboring tests only when a
failure or an uncovered interaction gives a concrete reason. Small stages can
be committed after those checks.

Treat `cargo test --lib`, `cargo test --tests`, `just test`, and `just verify` as
full-suite commands, not routine regression discovery. Run one only for an
explicit user request, a required broad implementation-closure gate, or a
demonstrated cross-cutting failure/invariant that
cannot be validated with a bounded set of focused tests. State the specific
reason before running it; do not repeat a full suite after fixing one localized
failure when its focused test and relevant neighbors suffice. Documentation-only
changes do not require tests. Test semantic contracts and relationships rather
than freezing a renderer revision's SVG length/hash, scene snapshot, automatic
panel size, or internal placement coordinates. Explicit requested output dimensions,
independently justified numerical rounding, containment, source completeness and
same-operation replay equality remain meaningful checks. Do not repin whole-scene
goldens to make a test pass; see `specs/90_DEVELOPMENT.md`.

Compiler validation must use `cargo clippy --locked --offline --all-targets --
-D warnings` (or `just lint`), never `cargo check`. Pass this requirement to every
subagent; a successful non-linting build is not a substitute for strict Clippy.

## Commits

Use Conventional Commits with mandatory scopes. Keep commits to one logical
change.

Commit subjects and bodies must describe enduring behavior or architecture,
never temporary planning identifiers or numbered implementation stages. Planning
labels will be removed from the final specifications; commit messages must remain
understandable without them. This applies to documentation commits as well as
code commits.

Before any Git push, satisfy the corpus-ancestry publication requirements in
`specs/90_DEVELOPMENT.md`. Do not publish a ref that retains the full
Unicode conformance corpus in its ancestry; a forward deletion is insufficient.

After achieving a phased objective, commit the completed work before moving on.
Do not leave finished work uncommitted unless the user explicitly asks not to
commit.
