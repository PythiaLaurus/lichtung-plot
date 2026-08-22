# Architecture

**Status**: Canonical

## Purpose

`lichtung-plot` is a standalone, strongly typed scientific renderer for owned
immutable display values. It owns plot construction, validation, layout, rendering,
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
caller-owned data or analysis result
        |
        v
owned native value construction
        |
        v
Plot / Canvas -> preflight -> private canonical scene -> SVG or optional PNG
                                                   |
                                                   v
                                             opaque Output
```

The renderer never calls back into an application and never computes a
hypothesis test. The caller owns all inferential decisions and passes explicit
display values.

## Ownership

- `NumericValues`, `CategoryValues`, `BooleanValues`, and `SourceRows` own
  immutable semantic input values.
- `Plot` owns one selected plot family and all its rendering configuration.
- `Canvas` owns a typed grid of `Plot` and `TextPanel` values.
- `Significance` owns one explicit attached or pairwise display annotation.
- `Error` owns the renderer's I/O, data, and rendering failure domains.
- `Output` privately owns one rendered scene and can publish additional formats
  without rerunning layout.
- Backends and rendered scenes are private implementation details.
- Built-in SVG, optional text outlining, and optional PNG output remain
  first-party implementations.

## Excluded Responsibilities

The crate does not provide analysis routing, dynamic plugins, report
registration, runtime provisioning, network access, font downloads, browser
automation, or PDF conversion. These responsibilities belong to callers or
other libraries.
