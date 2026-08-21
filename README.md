# lichtung-plot

`lichtung-plot` is a strongly typed scientific plotting library for Polars
dataframes. It produces one canonical SVG scene and can optionally rasterize
that scene to PNG with the `png` feature.

This first standalone-boundary release intentionally has no dependency on the
Lichtung analysis, pipeline, asset, or reporting crates. Callers supply
precomputed inferential annotations as ordinary plotting data.

The renderer is maintained in its own repository. Its architecture and output
contracts live in [`specs/`](specs/); user guides live in [`docs/`](docs/).

```rust
use lichtung_plot::Chart;
use polars::df;

let data = df!(
    "x" => [1.0, 2.0, 3.0],
    "y" => [1.5, 2.5, 2.0],
)?;
let mut chart = Chart::new(&data);
chart.x("x").y("y").scatter();
let svg = chart.render_to_svg_bytes()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Fonts

SVG output references this ordered system-font stack and does not download or
embed fonts:

```text
Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif
```

PNG rendering scans system font directories and selects the first installed
family from that order. Exact text appearance therefore depends on the host's
installed fonts, while chart data, geometry, layout, and SVG structure remain
deterministic for the same inputs and environment.

## Features

| Feature | Default | Capability |
| --- | --- | --- |
| `png` | No | Rasterize the canonical SVG scene to PNG |

SVG output is always available. PDF output is deliberately outside this crate.

## Development

The repository pins its Rust toolchain and tracks `Cargo.lock`. Run the complete
read-only verification before committing:

```bash
just verify
```

Focused recipes include `just test`, `just test-png`, `just lint`, and
`just lint-png`. The repository policy gate mechanically rejects first-party
Lichtung dependencies, path dependencies, legacy `mod.rs` layout, and retired
PDF/error boundary symbols.
