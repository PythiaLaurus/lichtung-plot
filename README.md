# lichtung-plot

`lichtung-plot` is a strongly typed scientific plotting library for Polars
dataframes. It produces one canonical SVG scene, can convert its text to vector
paths, and can rasterize that scene to PNG.

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

Call `RenderedSvgScene::svg_with_text_as_paths` when the SVG must carry glyph
outlines instead of selectable `<text>` elements. This uses the same local font
policy and embeds no font files.

## Features

| Feature | Default | Capability |
| --- | --- | --- |
| `text-as-path` | Yes, through `png` | Convert canonical SVG text to self-contained vector paths |
| `png` | Yes | Enable text outlining and rasterize the canonical SVG scene to PNG |

SVG output is always available. Use `default-features = false` for an SVG-only
build. PDF output is deliberately outside this crate.

## Development

The repository pins its Rust toolchain and tracks `Cargo.lock`. Run the complete
read-only verification before committing:

```bash
just verify
```

Focused recipes include `just test`, `just test-png`, `just lint`, and
`just lint-png`.
