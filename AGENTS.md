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

## Commits

Use Conventional Commits with mandatory scopes. Keep commits to one logical
change.

After achieving a phased objective, commit the completed work before moving on.
Do not leave finished work uncommitted unless the user explicitly asks not to
commit.
