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

- `NumericValues`, `CategoryValues`, and `BooleanValues` own immutable semantic
  input values. `SourceRows` owns one plot-wide aligned provenance map attached
  only through `Plot::source_rows`.
- `HistogramInput` and `RaincloudInput` are closed source selections. Each owns
  either native samples or complete caller-computed geometry, so one constructor
  cannot be combined with a second source setter.
- `FacetSpec` owns one complete generic-facet layout and axis-sharing policy:
  row, column, grid, or wrap. Mixed grid and wrap storage is not representable.
- `Plot<K = kind::Any>` owns one selected plot family and all its rendering
  configuration. Semantic constructors return an exact zero-sized family marker
  `K`; sealed capability traits expose only configuration supported by that
  family.
- `Plot::erase` is the sole public transition from an exact family marker to
  `Plot<kind::Any>`. Omitting `K` denotes this erased form, which retains common,
  introspection, preflight, and terminal-output operations but no family-specific
  configuration.
- `Canvas` owns a heterogeneous grid of internally erased plots and `TextPanel`
  values. `Canvas::panel` accepts any exact `Plot<K>` and erases it at the storage
  boundary.
- `Significance<Attached>` and `Significance<Pairwise>` own placement-typed
  display annotations. Plots erase admitted annotations only at their private
  storage boundary.
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
