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
- `Plot` also owns an optional explicit complete immutable `Theme`. The standalone
  owner supplies `Theme::publication()` when it is absent; a Canvas owner instead
  supplies its complete root value.
- `Plot::erase` is the sole public transition from an exact family marker to
  `Plot<kind::Any>`. Omitting `K` denotes this erased form, which retains common,
  introspection, preflight, and terminal-output operations but no family-specific
  configuration.
- `Canvas` owns a heterogeneous grid of internally erased plots and `TextPanel`
  values. `Canvas::panel` accepts any exact `Plot<K>` and erases it at the storage
  boundary. Canvas also owns one complete root theme, initialized with
  `Theme::publication()` and replaceable through `Canvas::theme`.
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

The current stable plot introspection surface is limited to `kind()` and
`preflight()`. Frozen canonical CSS-pixel dimensions belong to `Output::width()`
and `Output::height()`. Family configuration remains available through
exact marker-gated builders, not broad getters that return irrelevant defaults.

`Theme` is one complete immutable public appearance value. Its eight components
and nested roles have private fields, validated construction, readonly inspection
and typed consuming edits. `ThemeBuilder` is construction-only; built-ins are
ordinary `publication()`, `nature()`, `science()` and `chi()` constructors, with
no retained selector identity. `03_THEME_SYSTEM.md` and spec94 define their
component and construction contracts. `Canvas::theme(Theme)` and
`Plot::theme(Theme)` are the sole owner attachments. The Canvas call is optional
and Publication is its default root. An unthemed composed Plot inherits that root;
an explicit Plot theme completely replaces it locally. An unthemed standalone
Plot selects Publication, and facets add no theme scope.

All eighteen text roles own typed `FontStack` values. Literal `FontFamily` names
and closed generic preferences are distinct; no raw CSS stack parser or default
font constant is public. Plot-local `CategoricalColorScale` selects a palette or
exact raw-key `CategoricalColorMap`; `ContinuousColorScale` selects kind, optional
gradient and automatic/exact `ColorDomain` on both Heatmap and Beeswarm. Their data
colors are opaque `OpaqueColor`, independent of role alpha and non-data accents.
Canvas shared-guide spacing belongs only to its `LegendTheme`.

Resolution remains private, with operation font/query state alongside the single
complete appearance. Retained components drive measurement, source preparation
and frozen draw operands. `Color` supplies complete exact RGBA guides,
supplemental points and Dumbbell endpoints. Exact styles have readonly inspection
and `with_*` edits; Interaction selectors validate raw identity at construction.

Common Plot title/subtitle alignment setters retain independent instructions.
Canvas panel title/subtitle setters supply defaults for embedded Plots only;
explicit Plot instructions win, then the complete Plot theme. A subtitle falls
back through its own optional theme default before following the effective title.
Caption alignment and Canvas-owned root prose remain complete-theme properties.
Physical Start/End and all concrete resolved alignments are frozen before capture.

`05_LAYOUT_SYSTEM.md` is the approved replacement layout architecture for that
same migration. T5a through T5e implement its private parent-constraint
allocation, deterministic component measurement, and immutable arrangement
protocol. The reopened
T5a1 and T5a3 phases repair the existing implementation against that target;
historical completion does not establish revised-target conformance.
T5a0s records the accepted observable layout, label, sizing, output, font, and
alignment behavior; its documentation closure precedes implementation.
Plot and Canvas now derive dimensions from content when no sizing call is made.
Consuming `width(u32)` fixes width and derives content height;
`size(width,height)` fixes both dimensions. Each call replaces the complete
request, so `width` clears an earlier explicit height. Zero dimensions reject at
preflight. Canvas owns embedded dimensions independently of each child's
standalone request. There are no pre-render Plot dimension getters, public
maximum-width builders or general constraint objects.

The public layout module remains removed in both the current and target
contracts; `AxisTick` is exported only from the crate root. Public geometry uses
Canvas terminology and concise geometry nouns rather than renderer-internal
`Figure*` or redundant `*Input` names.

Opaque crate-root `TickGeneration` holds one concrete Automatic, preferred
Additive/Log10Decades, Target or Maximum choice. Default and `automatic()` select
Automatic. Validated counts accept `1..=10_000`; target permits nice-step deviation,
whereas maximum bounds complete in-domain generated sources, including endpoints.
`x_tick_generation` and `y_tick_generation` use the existing sealed NumericX/Y
gates and replace the selected axis's whole choice, including explicit Automatic
reset. Exact AxisTick lists remain authoritative and are not capped. Actual
categorical non-Automatic choices and interval scale-kind mismatches reject before
fonts. Prepared axes, composed equality and feedback retain this complete value;
there is no interval-only public type or competing optional count chain.

Opaque crate-root `NumericFormat` and its closed notation/precision/zero policies
configure generated numeric axes, continuous guide numbers, existing Heatmap and
point-only Coefficient value labels, and Waterfall totals. One exact decimal
rounding/emission engine handles ordinary binary64 operands and certified
mathematical Forest powers. Full normalized policies and semantic sets travel
through scoped preparation, fonts, intrinsic/cache identity and frozen replay.
`06_NUMERIC_PRESENTATION.md` and `07_LOG10_DISPLAY.md` own those current contracts.
Exact ticks and caller-owned strings remain verbatim; numeric policies are not
Theme components or Canvas overrides.

`PointLayer` is an optional ordinary-line overlay with renderer-owned visual
defaults and selective caller overrides.

Crate-root `LegendComposition::{Combined,Separate}` selects semantic sections of
one guide, independently of appearance and visibility/placement. All twenty-five
exact `kind::Legend` families expose the Plot setter; Canvas exposes
its Shared selection. Combined is the default. Standalone/facets use the owning
Plot, PerPlot each Plot, and Shared the Canvas for all contributions without
mutating stored child selections. Effective sections and producer-owned keys are
prepared before fonts and retained through exact measurement/dependency identity
and one frozen draw plan. `08_DISCRETE_GUIDES.md` defines truthful factorization,
ordering and unsupported independent-pair refusal. Any and parent capabilities
do not expand. Native continuous-color content and alpha symbols use the same
descriptor, finite measurement, font census and frozen draw plan.

Opaque crate-root `AlphaValues` and `AlphaEncoding`, with closed `AlphaSemantics`,
own eager immutable mappings and explicit semantic-unit bindings. Three independent
consuming Plot setters replace point-fill, admitted scientific-stroke and admitted
area/body/cell alpha through sealed PointAlpha/LineAlpha/FillAlpha sets of
17/17/15 families. Theme resets one target; Single explicitly broadcasts; observation
slots, keyed actual series/aggregates and exact bins do not reduce into each other.
Normal RGB/role/exact paint resolves before byte replacement. No callback survives
construction, and zero preserves scientific membership. Declared mappings alone
add alpha explanations; Hidden and Undeclared retain complete prepared identity
without guide text. Scoped child/height and feedback equality retain complete
inputs, declarations and actual members independently of extents/output bytes.
The public contract and full component census are in `../docs/public-data-alpha.md`.

## Excluded Responsibilities

The crate does not provide analysis routing, dynamic plugins, report
registration, runtime provisioning, network access, font downloads, browser
automation, or PDF conversion. These responsibilities belong to callers or
other libraries.
