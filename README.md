# lichtung-plot

A standalone, strongly typed scientific renderer for owned immutable display
values, with 25 plot families, facets, Canvas composition, themes and text
adaptation. It publishes outlined SVG, preserve-text SVG and PNG from frozen
rendered scenes. Applications own inferential analysis; the renderer has no
network access, asset provisioning or dependency on another `lichtung-*` crate.

## Guides

- [English guide source](docs/en/src/index.md)
- [Chinese guide source](docs/zh-CN/src/index.md)
- [Book and gallery build instructions](docs/README.md)
- [Rust API reference](https://docs.rs/lichtung-plot)

Language editions have equal status and equivalent scope. Each opens with
real gallery images linking to code walkthroughs on the same page, with an in-page
contents menu for the main tutorial. Both share actual runnable examples and
gallery outputs, not hand-copied plotting programs.

## First Example

From the workspace root, with suitable system fonts installed:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start
```

Examples write to the working directory by default; an optional directory after
`--` redirects output. The renderer never downloads fonts. See the installation
sections' sources in [English](docs/en/src/sections/installation.md) /
[Chinese](docs/zh-CN/src/sections/installation.md) for local dependency paths and font requirements.

## Workspace And Development

- [`crates/lichtung-plot/`](crates/lichtung-plot/): independent renderer, tests,
  fixtures, runnable examples, package README and licenses; the default member.
- [`crates/lichtung-plot-ndarray/`](crates/lichtung-plot-ndarray/): direct plotting
  from logical views, selected columns, wide variables and masked grids, with
  unchanged native renderer exports.
- [`crates/lichtung-plot-polars/`](crates/lichtung-plot-polars/): eager Polars
  field/direct-column plotting frontends and unchanged native renderer exports;
  wide/grid roles, independent layers and scientific result bindings.
- `bindings/python/`: all 25 exact-family interfaces across four input routes,
  reusable Plot/Canvas/Output, immutable configuration values, alpha, independent
  layers and supplied results. See
  `bindings/python/README.md` and `specs/91_WORKSPACE_AND_INTERFACES_PLAN.md` for
  bounded verification and remaining delivery gates.
- `docs/book-helper/`: nonpublished Rust command checking and assembling the
  single-page guides and executable gallery; it has no renderer/adapter dependency.
- [`specs/`](specs/): one root architectural source of truth.
- [`docs/`](docs/): peer-language user guides and gallery catalog.

The workspace pins Rust 1.96.0 and tracks one lockfile. Use focused tests and
`just lint` for code changes. `just book-check` checks both editions without rendering;
`just book` generates fresh gallery outputs and invokes installed mdBook 0.5.4;
set `LICHTUNG_PYTHON` to an absolute interpreter with the Python wheel installed
for its Python examples. The build writes
`target/book/site/index.html`. `just verify` is the broad source-verification
gate; it includes structural Book checks but does not regenerate the gallery/site.
See [development](specs/90_DEVELOPMENT.md) for verification scope and test limits.

## Release Preparation

The coordinated Rust/Python `0.1.0` release is being prepared, not announced as
published. `specs/95_RELEASE.md` defines candidate verification, artifact/platform
requirements, approval boundaries and the crates.io → PyPI → GitHub Release
sequence. Local verification does not upload packages, push tags or deploy a site.
