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
Plot / Canvas -> preflight -> private canonical scene
                                      |
                                      +-> publication SVG
                                      +-> preserve-text SVG
                                      +-> PNG
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
- Every constructor-required semantic channel is owned by its closed plot input.
  Required trajectory, line-path, interaction, and stacked-series groups and
  required heatmap and beeswarm color values cannot be replaced by a later
  setter.
- `FacetSpec` owns one complete generic-facet layout and axis-sharing policy:
  row, column, grid, or wrap. Mixed grid and wrap storage is not representable.
- `Plot<K = kind::Any>` owns one selected `PlotKind` directly and all its
  rendering configuration. Semantic constructors return an exact zero-sized
  family marker `K`; sealed capability traits expose only configuration
  supported by that family. A publicly constructible plot never has an
  unselected family state.
- `Plot` also owns an optional explicit built-in theme selection. The standalone
  owner selects Publication when it is absent; a Canvas owner instead supplies
  its root selection.
- `Plot::erase` is the sole public transition from an exact family marker to
  `Plot<kind::Any>`. Omitting `K` denotes this erased form, which retains common,
  introspection, preflight, and terminal-output operations but no family-specific
  configuration.
- `Canvas` owns a heterogeneous grid of internally erased plots and `TextPanel`
  values. `Canvas::panel` accepts any exact `Plot<K>` and erases it at the storage
  boundary. Canvas also owns one root built-in theme selection, initialized to
  Publication and replaceable through `Canvas::theme`.
- `Significance<Attached>` and `Significance<Pairwise>` own placement-typed
  display annotations. Plots erase admitted annotations only at their private
  storage boundary.
- `Error` owns the renderer's I/O, data, and rendering failure domains.
- `Output` privately owns one rendered scene and can publish additional formats
  without rerunning layout. The SVG publication form and PNG publication options
  are local to each terminal call; plots, canvases, and outputs do not retain
  sticky output configuration.
- Backends, rendered scenes, resolved themes, projections, axes, frames,
  grids, and renderer-facing layout values are private implementation details.
  `AxisTick` remains one root-level caller input for exact labeled numeric tick
  positions; it is not a public layout extension point.
- Built-in publication SVG with text outlining, preserve-text SVG, and PNG
  output are unconditional first-party implementations.

## Validation Boundary

`Plot` and every plot stored by `Canvas` pass through one crate-private
canonical plot validator before any backend canvas is opened. That validator
owns axis validation, family-option ownership, aligned value lengths,
categorical ordering, channel semantics, intervals, facets, and supplemental
numeric geometry. `Canvas` applies composition-only rules after canonical plot
validation; composition never weakens or skips a standalone plot invariant.

Renderer methods may perform additional geometry-dependent checks while
building the private scene. `preflight` executes the same validation, planning,
and private rendering path as terminal output through a null backend.

## Public Surface

The current stable plot introspection surface is limited to `kind()`, `width()`,
`height()`, and `preflight()`. Family configuration remains available through
exact marker-gated builders, not broad getters that return irrelevant defaults.

`Theme` is the public built-in visual selection. Resolved theme data is
private. `03_THEME_SYSTEM.md` is the approved replacement architecture for the
staged migration tracked in `91_IMPLEMENTATION_PLAN.md`. `Canvas::theme(Theme)`
and `Plot::theme(Theme)` are the sole current owner attachments. The Canvas call
is optional and Publication is its default root. An unthemed composed Plot
inherits that root; an explicit Plot selection completely replaces it locally.
An unthemed standalone Plot selects Publication, and facets add no theme scope.
Internally those attachments retain complete immutable appearance values, with
per-operation font/query state alongside the single resolved appearance. Retained
component values drive measurement, source preparation and frozen draw operands.
The complete public Theme/component/color/scale construction surface becomes
current at its owning T6 public cutover.

`05_LAYOUT_SYSTEM.md` is the approved replacement layout architecture for that
same migration. T5a through T5e implement its private parent-constraint
allocation, deterministic component measurement, and immutable arrangement
protocol while preserving current exact public sizing behavior. The reopened
T5a1 and T5a3 phases repair the existing implementation against that target;
historical completion does not establish revised-target conformance.
T5a0s records the accepted observable layout, label, sizing, output, font, and
alignment behavior; its documentation closure precedes implementation.
Automatic root sizing, consuming exact-width/automatic-height `width(u32)`,
rendered-output dimension accessors, and removal of pre-render Plot dimension
getters are not current behavior or public API until Phase T6 completes. The
target keeps exact `size(width,height)` and does not add public maximum-width
or general constraint objects.

The public layout module remains removed in both the current and target
contracts; `AxisTick` is exported only from the crate root. Public geometry uses
Canvas terminology and concise geometry nouns rather than renderer-internal
`Figure*` or redundant `*Input` names.

`PointLayer` is an optional ordinary-line overlay with renderer-owned visual
defaults and selective caller overrides. `NumericLabelFormat` remains a narrow
fixed-decimal value-label format rather than a general axis, colorbar, or
scientific-notation extension point.

## Excluded Responsibilities

The crate does not provide analysis routing, dynamic plugins, report
registration, runtime provisioning, network access, font downloads, browser
automation, or PDF conversion. These responsibilities belong to callers or
other libraries.
