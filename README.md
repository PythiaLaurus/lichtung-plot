# lichtung-plot

A standalone, strongly typed scientific renderer for owned immutable display
values, with 25 plot families, facets, Canvas composition, themes and text
adaptation. It publishes outlined SVG, preserve-text SVG and PNG from frozen
rendered scenes. Applications own inferential analysis; the renderer has no
network access, asset provisioning or dependency on another `lichtung-*` crate.

## Guides · 使用指南

- [English guide source](docs/en/src/index.md)
- [中文指南源码](docs/zh-CN/src/index.md)
- [Book and gallery build instructions · 文档与图库构建](docs/README.md)
- [Rust API reference](https://docs.rs/lichtung-plot)

The two guides are equal-status editions with equivalent scope. Each opens with
real gallery images linking to code walkthroughs on the same page, with an in-page
contents menu for the main tutorial. Both share actual runnable examples and
gallery outputs, not hand-copied plotting programs.
两套指南地位相同、范围等价，各自以真实图库开篇，图片直接跳到同页代码讲解，主要教程
通过页内目录导航。两者共享可运行示例及图库产物，不手工维护多套绘图代码。

## First Example

From the workspace root, with suitable system fonts installed:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start
```

Examples write to the working directory by default; an optional directory after
`--` redirects output. The renderer never downloads fonts. See the installation
sections' sources in [English](docs/en/src/sections/installation.md) /
[中文](docs/zh-CN/src/sections/installation.md) for local dependency paths and font requirements.

## Workspace And Development

- [`crates/lichtung-plot/`](crates/lichtung-plot/): independent renderer, tests,
  fixtures, runnable examples, package README and licenses; the default member.
- [`crates/lichtung-plot-ndarray/`](crates/lichtung-plot-ndarray/): owned native
  values from logical ndarray views, named tables and labeled grids.
- [`docs/book-helper/`](docs/book-helper/): nonpublished Rust command checking and
  assembling the single-page guides and executable gallery; no renderer dependency.
- [`specs/`](specs/): one root architectural source of truth.
- [`docs/`](docs/): peer-language user guides and gallery catalog.

The workspace pins Rust 1.96.0 and tracks one lockfile. Use focused tests and
`just lint` for code changes. `just book-check` checks both editions without rendering;
`just book` generates fresh gallery outputs and invokes installed mdBook 0.5.4,
writing `target/book/site/index.html`. `just verify` is the broad source-verification
gate; it includes structural Book checks but does not regenerate the gallery/site.
See [development](specs/90_DEVELOPMENT.md) for verification scope and test limits.
