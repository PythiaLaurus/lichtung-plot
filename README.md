# lichtung-plot

A standalone, strongly typed scientific renderer for owned immutable display
values, with 25 plot families, facets, Canvas composition, themes and text
adaptation. It publishes outlined SVG, preserve-text SVG and PNG from frozen
rendered scenes. Applications own inferential analysis; the renderer has no
network access, asset provisioning or dependency on another `lichtung-*` crate.

## Guides · 使用指南

- [English Book](docs/en/src/introduction.md)
- [中文 Book](docs/zh-CN/src/introduction.md)
- [Book and gallery build instructions · 文档与图库构建](docs/README.md)
- [Rust API reference](https://docs.rs/lichtung-plot)

The two Books are equal-status editions with equivalent scope. They share actual
runnable example sources and gallery outputs, not hand-copied plotting programs.
两套 Book 地位相同、范围等价，共享真实可运行示例及图库产物，不手工维护多套绘图代码。

## First Example

From the workspace root, with suitable system fonts installed:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start -- target/examples/first-plot
```

Each example requires one caller-selected output directory. The renderer never
downloads fonts. See [installation](docs/en/src/installation.md) /
[安装](docs/zh-CN/src/installation.md) for local dependency paths and font requirements.

## Workspace And Development

- [`crates/lichtung-plot/`](crates/lichtung-plot/): independent renderer, tests,
  fixtures, runnable examples, package README and licenses; the default member.
- [`dev/book/`](dev/book/): nonpublished Rust command assembling/checking the Books
  and executable gallery; no production dependency from the renderer.
- [`specs/`](specs/): one root architectural source of truth.
- [`docs/`](docs/): peer-language user guides and gallery catalog.

The workspace pins Rust 1.96.0 and tracks one lockfile. Use focused tests and
`just lint` for code changes. `just book-check` checks both editions without rendering;
`just book` generates fresh gallery outputs and invokes installed mdBook 0.5.4,
writing `target/book/site/index.html`. `just verify` is the broad source-verification
gate; it includes structural Book checks but does not regenerate the gallery/site.
See [development](specs/90_DEVELOPMENT.md) for verification scope and test limits.
