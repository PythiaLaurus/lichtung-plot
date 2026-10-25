# Installation and first plot

[简体中文](../zh-CN/installation.html)

This development checkout uses Rust 1.96.0. The renderer is the default workspace
member at `crates/lichtung-plot`; adapters and language bindings are not delivered
by this guide. Until a release is published, a neighboring Rust application can
use the actual member path:

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot/crates/lichtung-plot" }
```

No renderer initialization is needed. Install suitable system fonts before
rendering; the library never downloads them. The default ordered stack is Noto
Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif. Visible
text needs real coverage in the selected stack; see [output and fonts](output.md).

From the workspace root:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start -- target/examples/first-plot
```

Every example requires exactly one output directory and creates it if necessary.
The example writes `quick_start.svg` there and reports its frozen dimensions.
Relative directories are relative to the caller's working directory; no fixed
shared temporary destination is used. `--offline` requires cached Rust dependencies.

```rust
{{#include ../../../crates/lichtung-plot/examples/01_quick_start.rs:example}}
```

`.svg` publishes outlined SVG. A terminal render returns an opaque `Output` that
can publish more formats without rendering again. The
[first-plot gallery case](gallery-first-plot.md) displays the output of this exact
program. [Native data](data.md) explains what the constructor receives.
