# Architecture

**Status**: Canonical

## Purpose

`lichtung-plot` is a standalone, strongly typed scientific renderer for Polars
dataframes. It owns chart construction, validation, layout, rendering,
plot-owned display annotations, renderer errors, and artifact publication.

## Repository Topology

```text
lichtung-plot/
├── Cargo.toml
├── Cargo.lock
├── Justfile
├── src/
├── tests/
├── examples/
├── specs/
└── docs/
```

The package is the repository root. `src/lib.rs` defines the public surface.
Implementation modules use `foo.rs` plus `foo/` siblings; `mod.rs` is not
admitted.

## Dependency Boundary

The crate has no dependency on any `lichtung-*` crate. In particular, it does
not know about analysis specifications, insights, pipelines, reports, runtime
initialization, asset caches, Python, R, or application capability IDs.

External applications integrate by converting their domain values at their own
boundary:

```text
caller-owned analysis result
        |
        v
lichtung_plot::ComparisonAnnotation
        |
        v
Chart -> preflight -> canonical SVG scene -> SVG, outlined SVG, and optional PNG
```

The renderer never calls back into an application and never computes a
hypothesis test. The caller owns all inferential decisions and passes explicit
display values.

## Ownership

- `Chart` and `Figure` own plotting configuration while borrowing dataframes.
- `ComparisonAnnotation` owns visual endpoint and p-value metadata.
- `Error` owns the renderer's I/O, data, and rendering failure domains.
- `RenderedSvgScene` owns exact SVG bytes and canonical CSS dimensions and can
  derive a text-as-path SVG without rerunning chart layout.
- `Backend` is the public custom-renderer extension point.
- Built-in SVG, optional text outlining, and optional PNG output remain
  first-party implementations.

## Excluded Responsibilities

The crate does not provide analysis routing, dynamic plugins, report
registration, runtime provisioning, network access, font downloads, browser
automation, or PDF conversion. These responsibilities belong to callers or
other libraries.
