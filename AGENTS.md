# Repository Guidelines

## Source Of Truth

Treat `specs/` as the architectural source of truth. Prefer root `Justfile`
recipes over ad hoc command sequences.

## Standalone Boundary

`lichtung-plot` is a standalone scientific renderer. It must not depend on,
import, or dispatch into any other `lichtung-*` crate. It owns its errors,
visual annotation types, rendering lifecycle, output contracts, and font
policy. Callers own analysis and convert scientific results into plot-owned
display values before calling this crate.

Do not add network access, asset provisioning, managed runtimes, dynamic
dispatch, analysis algorithms, report generation, or plot-level PDF output.
SVG is always available; PNG is the sole optional output feature.

## Current Contract

Keep one current API and representation. When replacing a project-owned API,
migrate repository callers and remove the old implementation rather than
adding aliases, version selectors, fallback chains, or deprecations.

Use modern Rust module layout: `foo.rs` plus `foo/` siblings. Do not add
`mod.rs` files. Keep public modules limited to deliberate extension points.

## Testing

Run the narrowest relevant check while iterating. Before completing a change,
run `just verify`. Snapshot changes must be inspected and committed with the
source change that intentionally alters SVG output.

## Commits

Use Conventional Commits with mandatory scopes. Valid scopes are `plot`,
`docs`, `workspace`, `deps`, and `ci`. Keep commits to one logical change.

## Generated Artifacts

Track `Cargo.lock` and SVG snapshots. Do not commit `target/`, generated PNG or
SVG examples, editor state, caches, or manual visual-audit output.
