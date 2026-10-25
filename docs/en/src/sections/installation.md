<a id="installation"></a>
## Installation and first plot

[简体中文](../zh-CN/index.html#installation)

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
text needs real coverage in the selected stack; see [output and fonts](#output).

<a id="first-plot"></a>
### Walkthrough: from values to a first figure

![A scatter constructed from owned numeric values](gallery/first-plot/quick_start.svg)

Start with two equally sized numerical series. The complete example generates a
deterministic sample in memory, so running it never downloads a dataset:

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-values}}
```

Wrap each series in a named `NumericValues` and pass them to `Plot::scatter`.
The explicit Nature theme changes appearance, not scientific values. `base_path`
is the working directory joined with `quick_start`; `.svg` appends the SVG suffix
and returns the frozen Output rather than a mutable backend:

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-publication}}
```

No fixed size is requested: the renderer sizes this complete figure from content.
To try the exact source rather than reconstructing a snippet, run it with no arguments.
From the workspace root:

```sh
cargo run --locked --offline -p lichtung-plot --example 01_quick_start
```

Every example runs without arguments. This example writes `quick_start.svg` to
the working directory and reports its frozen dimensions. If needed, append
`-- <directory>` to redirect output; that optional directory is created if absent
and relative paths are resolved from the working directory. Gallery builds choose
their isolated directories automatically. `--offline` requires cached Rust dependencies.

`.svg` publishes outlined SVG. A terminal render returns an opaque `Output` that
can publish more formats without rendering again. [Native data](#data) explains
what the constructor receives.

[Complete source](examples/01_quick_start.rs) · [Shared output helper](examples/support/output.rs) ·
[Full-size SVG](gallery/first-plot/quick_start.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)
