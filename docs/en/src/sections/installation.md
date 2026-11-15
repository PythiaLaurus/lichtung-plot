<a id="installation"></a>
## Installation and first plot

[简体中文](../zh-CN/index.html#installation)

This development checkout uses Rust 1.96.0. The renderer is the default workspace
member at `crates/lichtung-plot`; [ndarray](#ndarray) and [Polars](#polars) adapters
are separate packages. The [Python binding](#python) is implemented, but has not
been published to PyPI. Until a release is published, a neighboring Rust application
can use the actual member path:

```toml
[dependencies]
lichtung-plot = { path = "../lichtung-plot/crates/lichtung-plot" }
```

For Python, use ordinary CPython 3.10 or newer and an already-built wheel matching
your operating system and architecture. The extension uses `abi3-py310`; that is not
a claim of support for PyPy or free-threaded Python. For example, replace this path
with the compatible wheel you have built or received:

```sh
python -m pip install --no-index --no-deps /absolute/path/to/lichtung_plot-0.1.0-cp310-abi3-linux_x86_64.whl
```

This installs a local artifact, not a package fetched from PyPI. Native Python
lists/tuples need no data provider. Install optional NumPy, pandas or eager Polars
separately in that environment when using them. The current Polars bridge requires
`polars==1.41.2` and `polars-runtime-32==1.41.2`, not PyArrow. Build recipes and
sampled provider versions are recorded in `bindings/python/README.md` and the shared
workspace specifications; neither the renderer nor the gallery installs dependencies.

No renderer initialization is needed. Install suitable system fonts before
rendering; the library never downloads them. The default ordered stack is Noto
Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif. Visible
text needs real coverage in the selected stack; see [output and fonts](#output).

<a id="first-plot"></a>
### Walkthrough: from values to a first figure

![Synthetic enzyme responses across substrate concentrations](gallery/first-plot/quick_start.svg)

Start with aligned substrate concentrations, reaction rates and enzyme identities.
The two response regimes use explicit illustrative saturation equations with fixed
positive multiplicative variation. They are not experimental measurements or fitted kinetics.
The complete example generates them in memory without downloading a dataset:

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-values}}
```

Wrap the two numerical series in named `NumericValues` and pass them to
`Plot::scatter`; `hue` keeps enzyme identity separate from the coordinates. The
shared gallery theme changes appearance, not scientific values. `base_path` is the
working directory joined with `quick_start`; `.svg` appends the SVG suffix and
returns the frozen Output rather than a mutable backend:

```rust
{{#include ../../../../crates/lichtung-plot/examples/01_quick_start.rs:scatter-publication}}
```

The example requests 840 × 560 CSS pixels so its complete labels remain readable
in the gallery. This is an example-specific publication choice, not a renderer
default. The shared theme is ordinary public-API configuration:

```rust
{{#include ../../../../crates/lichtung-plot/examples/support/gallery.rs:gallery-theme}}
```

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

[Complete source](examples/lichtung-plot/01_quick_start.rs) · [Shared output helper](examples/lichtung-plot/support/output.rs) ·
[Gallery theme](examples/lichtung-plot/support/gallery.rs) · [Synthetic inputs](examples/lichtung-plot/support/synthetic.rs) ·
[Full-size SVG](gallery/first-plot/quick_start.svg) ·
[Plot API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.Plot.html) · [Back to gallery](#gallery)
