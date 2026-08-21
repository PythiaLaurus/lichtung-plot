# Development

**Status**: Canonical

## Toolchain

The repository pins Rust 1.96 with rustfmt and Clippy in
`rust-toolchain.toml`. `Cargo.lock` is tracked.

## Commands

Use the root recipes:

```text
just fmt
just check
just check-png
just lint
just lint-png
just test
just test-png
just docs
just package
just verify
```

`just verify` is the complete read-only gate. It checks formatting, repository
policy, default and PNG builds, lints, tests, documentation, and Cargo's
independently unpacked package.

## Dependency Changes

Dependencies are declared directly in `Cargo.toml`; workspace inheritance and
path dependencies are prohibited. A dependency change updates and commits
`Cargo.lock`, then runs `just verify`.

No dependency may introduce application-owned analysis, report, runtime,
network-provisioning, or first-party Lichtung behavior into the renderer.

## Visual Changes

Changes to layout, themes, text, SVG serialization, clipping, or marks require
focused tests and inspection of affected snapshots. Update snapshots only when
the new output is intentional and consistent with `02_RENDERING_CONTRACT.md`.

## Test Layout

`tests/integration.rs` is the sole Cargo-discovered integration-test root.
Suites below `tests/integration/` are grouped into the stable `plots`,
`rendering`, `composition`, `visual_regression`, and `statistics` domains. Plot
tests are grouped again by plot family rather than added as loose root files.
Shared fixtures and backend helpers live under `tests/integration/support/`, and
visual snapshots remain beside the visual-regression module that owns them.
