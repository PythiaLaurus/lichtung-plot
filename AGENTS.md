# Repository Guidelines

## Source Of Truth

Treat `specs/` as the architectural source of truth. Prefer root `Justfile`
recipes over ad hoc command sequences.

## Workspace

The root Cargo.toml is a virtual workspace. The standalone renderer lives in
`crates/lichtung-plot/` and is the default member. Keep source, tests, fixtures,
examples, package README and license texts together in that member. Keep one
root Cargo.lock and pinned toolchain; share dependency versions and lints through
the workspace manifest. Add members only with working consumers, not empty shells.

Keep specifications at the workspace root, divided by responsibility; member
documentation references shared contracts rather than duplicating them. Root
lint/test/API-documentation recipes explicitly select the workspace, not only
default members. Package verification selects distributable members, excluding
workspace-local development commands such as `docs/book-helper`.

## Standalone Boundary

The `lichtung-plot` core crate is a standalone scientific renderer. It must not
depend on, import, or dispatch into any other `lichtung-*` crate. It owns its errors,
visual annotation types, rendering lifecycle, output contracts, and font
policy. Adapters and bindings depend on its public API, never the reverse.
Core-only consumers must not compile adapter or binding dependencies.

Do not add network access, asset provisioning.

Python package code, stubs, tests and examples belong only in `bindings/python/`
when that binding is implemented. Do not add Python-based renderer/development
support elsewhere or a `tools/` directory. Keep Rust development and test support
within workspace members.

## Current Contract

Keep one current API and representation. When replacing a project-owned API,
migrate repository callers and remove the old implementation rather than
adding aliases, version selectors, fallback chains, or deprecations.

Use modern Rust module layout: `foo.rs` plus `foo/` siblings. Do not add
`mod.rs` files. Keep public modules limited to deliberate extension points.

## Testing

Keep the whole workspace's test suite within these shared size limits, including
bindings and automated documentation/gallery scenarios; new members do not gain
separate allowances:

| Layer | Reasonable scale | Upper limit |
| --- | --- | --- |
| Unit tests | 300–800 tests | 800 |
| Module integration, excluding core E2E | 100–300 tests | 300 |
| Core E2E | 30–80 scenarios | 80 |

These are ceilings, not minimum quotas. Meaningful new coverage may grow within
them; adding a unit test does not require deleting another. Classify by the
contract exercised, not merely Cargo's library/integration target boundary, and
count ignored tests too. Count independently exercised E2E scenarios rather than
only driver functions. Do not evade limits with merged functions, parameterized
packaging, relabeling or moving tests between suites. Remove low-value/redundant
tests and obsolete test-only support; do not fill available space with accessor
echoes, per-family smoke/replay copies or mechanical configuration cross-products.
See `specs/90_DEVELOPMENT.md` for ownership and semantic expectations.

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

Compiler validation must use `cargo clippy --locked --offline --workspace --all-targets --
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
