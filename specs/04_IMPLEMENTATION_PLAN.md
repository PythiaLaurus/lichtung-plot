# Implementation Plan

**Status**: Active remediation ledger

## Purpose

This document records the current API-boundary remediation sequence. The
completed implementation history remains available in Git through commit
`355155a`; it is not a second current plan.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative. This
ledger closes implementation gaps against those contracts and does not
authorize compatibility aliases, deprecated replacements, alternate data
representations, or dormant public extension points.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each phase as one logical change before starting the next phase.

## Contract Rules

- Keep one current API and migrate every repository caller atomically.
- Delete replaced methods, variants, modules, test shims, and report text. Do
  not retain aliases, compatibility wrappers, or private copies without a
  current renderer consumer.
- Public modules are deliberate extension points. A root re-export does not
  require its defining module to be public.
- Only an exact plot-family marker may cross the explicit erasure boundary.
  `kind::Any` is the result of erasure, not an input to erasure or Canvas
  insertion.
- Tests construct renderer-native values. They do not recreate a generic
  dataframe or string-indexed column conversion layer inside this repository.
- Run focused checks while iterating. Before completing each phase, inspect any
  snapshot changes and run `just verify`.

## Remediation Inventory

| Issue | Required outcome | Phase | Status |
| --- | --- | --- | --- |
| 1 | Internalize `decorations` and `TitleBlock` | R1 | PENDING |
| 2 | Delete overlay and regression report-provenance APIs | R1 | PENDING |
| 3 | Delete public kind and theme inventories and report naming | R1 | PENDING |
| 4 | Restrict erasure and Canvas insertion to exact families | R2 | PENDING |
| 5 | Remove duplicate default PNG compression identity | R3 | PENDING |
| 6 | Expose `Theme` only from the crate root | R1 | PENDING |
| 7 | Use `HistogramKdePoint` for every precomputed histogram KDE | R4 | PENDING |
| 8 | Move PNG configuration to explicit terminal operations | R3 | PENDING |
| 9 | Give `PointLayer` automatic style defaults and exact label semantics | R5 | PENDING |
| 10 | Replace dataframe-shaped test shims with native values | R7 | PENDING |
| 11 | Keep `NumericLabelFormat` narrow and defer general notation | R6 | PENDING |

Issues 8, 9, and 11 were resolved in the canonical contracts before their
implementation phases were admitted here.

## Phase R1: Public Surface Cleanup

**Status**: PENDING

Remove accidental module exposure and application/report-oriented
introspection while preserving renderer-owned behavior.

Scope:

- Make `decorations` crate-private. Make `title_block` and `TitleBlock`
  crate-private and retain them only as renderer implementation details.
- Make `theme` crate-private while retaining the root-level
  `lichtung_plot::Theme` export.
- Delete `RawOverlay::provenance()` and its fixed report strings. Retain
  `is_visible()` because rendering consumes it.
- Delete `RegressionBand::provenance()` and its fixed report strings. Retain the
  crate-private legend wording because the renderer consumes it.
- Delete `PlotKind::ALL`, `PlotKind::variant_name()`, `PlotKind::label()`, and
  `PlotKind::key_segment()`.
- Replace the validator's sole `PlotKind::key_segment()` use with diagnostic
  wording local to that validation concern. Do not introduce a general kind
  naming or report helper.
- Delete `Theme::ALL` and `Theme::variant_name()`. Rewrite internal theme tests
  to exercise the explicitly named built-in themes without restoring a
  runtime inventory API.
- Remove tests and documentation whose only purpose was to guarantee the
  deleted report, inventory, or public-module surface.

Acceptance:

- `lichtung_plot::Theme` remains usable, while `lichtung_plot::theme` and
  `lichtung_plot::decorations` are not public paths.
- No callable provenance method or fixed report-provenance text remains on
  `RawOverlay` or `RegressionBand`.
- No general inventory, variant-name, report-label, or generated-key API
  remains on `PlotKind` or `Theme`.
- All internal title, subtitle, caption, legend, and regression-band rendering
  behavior remains intact.
- Compile-fail API-boundary coverage and focused library, lifecycle, and theme
  tests pass.
- `just verify` passes.

Commit: `refactor(api)!: remove accidental introspection surface`

## Phase R2: Exact-Family Erasure

**Status**: PENDING

Make the architecture's exact-family boundary mechanically enforceable.

Scope:

- Add one sealed exact-family marker bound implemented by every concrete plot
  family and not implemented by `kind::Any`.
- Keep `kind::Marker` as the bound admitted by `Plot<K>` itself.
- Restrict `Plot::erase()` to exact family markers.
- Restrict `Canvas::panel()` to exact family markers and continue erasing only
  at Canvas's private heterogeneous-storage boundary.
- Do not add a second erasure method, an erased-panel insertion method, a
  runtime escape hatch, or an implementation for `kind::Any`.

Acceptance:

- Every concrete constructor result can call `erase()` and can be passed to
  `Canvas::panel()`.
- `Plot<kind::Any>` can do neither, including a second call to `erase()`.
- Erasure still moves all owned input and configuration without changing plot
  identity or render output.
- An erased plot retains its common `kind()`, `width()`, `height()`,
  `preflight()`, and terminal-output operations; compile-pass coverage exercises
  those operations after erasure.
- Compile-fail doctests cover both rejected erased-input operations.
- Focused builder and composition tests pass.
- `just verify` passes.

Commit: `feat(api)!: enforce exact-family erasure`

## Phase R3: Terminal PNG Publication

**Status**: PENDING

Move PNG configuration from inert plot state to explicit publication calls and
remove duplicate public identities.

Scope:

- Delete `PngCompression::Default`.
- Make `PngCompression::Balanced` the enum default and make
  `PngOptions::default()` resolve to 300 DPI with balanced compression.
- Migrate every repository caller, test, example, and document from `Default`
  to `Balanced`; do not retain an alias or decoder.
- Remove `.png_options(...)` from `Plot` and `Canvas` and remove stored
  `PngOptions` from `Plot`, `Canvas`, `Output`, erasure, and private accessors.
- Make `Plot`, `Canvas`, and `Output` expose the same PNG operations:
  `png(path)` for 300-DPI publication, `png_preview(path)` for source-sized
  preview output, and `png_with_options(path, options)` for exact caller-owned
  configuration.
- Keep PNG options call-local and non-sticky across chained `Output` calls.
- Delete `svg_and_png` from `Plot`, `Canvas`, and `Output`. Callers that need
  both formats use independent same-scene chaining such as
  `plot.svg(svg_path)?.png(png_path)`.
- Remove pair-publication code and tests that have no remaining consumer while
  retaining atomic publication for each individual output call.
- Feature-gate PNG option types and their root exports together with PNG
  terminal operations.

Acceptance:

- Exactly one public balanced PNG compression identity exists, and it is the
  default.
- All compression presets retain their existing encoder mappings apart from
  removal of the duplicate variant.
- Plain PNG output uses `Dpi(300.0)`, writes matching physical-resolution
  metadata, and scales canonical CSS pixels from 96 DPI.
- Preview PNG output preserves canonical SVG pixel dimensions, uses balanced
  compression, and writes no physical-resolution metadata.
- One `Output` can publish preview, publication, and custom PNGs from the same
  scene without one call's options affecting another.
- SVG-only builds expose no PNG option type, builder, terminal method, or stored
  PNG configuration.
- No combined SVG-and-PNG public operation or transactional pair-publication
  implementation remains.
- Focused output tests pass in both the no-default-features and default-feature
  configurations.
- `just verify` passes.

Commit: `feat(api)!: move png options to publication`

## Phase R4: Canonical Histogram KDE Points

**Status**: PENDING

Use one named public point representation for caller-computed histogram KDE
geometry.

Scope:

- Change `HistogramKde::Precomputed` from `Vec<(f64, f64)>` to
  `Vec<HistogramKdePoint>`.
- Migrate validation, rendering, tests, examples, and documentation to named
  `x` and `y` fields. Do not retain tuple conversion as a second project-owned
  representation.
- Preserve the distinction between a precomputed KDE layer over sample-owned
  histogram input and complete self-contained `HistogramGeometry`.
- Do not change source-row or facet ownership. Complete precomputed histogram
  geometry continues to reject sample-only channels before backend begin.

Acceptance:

- All precomputed histogram KDE paths use the same public point type as
  `HistogramGeometry::kde`.
- Invalid KDE ordering, non-finite coordinates, negative density, and safety
  limits retain focused coverage through the named point representation.
- Sample-derived and complete-geometry histogram modes retain their existing
  validation and rendering behavior.
- Focused histogram, geometry, distribution, and facet tests pass.
- `just verify` passes.

Commit: `refactor(api)!: unify histogram kde points`

## Phase R5: Ergonomic Point Layers

**Status**: PENDING

Make supplemental ordinary-line points concise by default while preserving
selective identity, annotation, and visual control.

Scope:

- Replace the five-argument fallible constructor with infallible
  `PointLayer::new(x, y) -> Self`.
- Default radius to 4 and opacity to 0.8. Without hue, reserve one primary-line
  palette slot; with hue, reserve one slot per level in the resolved hue domain,
  including Canvas-coordinated levels absent from the local plot. Resolve each
  automatic layer from the next slot plus its zero-based append index. Every
  layer consumes its append position even when it has an explicit color.
- Add consuming `color(...) -> Self`, `radius(...) -> Result<Self>`, and
  `opacity(...) -> Result<Self>` builders. Validate positive radius and finite
  opacity in `(0, 1]` immediately without retaining the old constructor.
- Retain optional per-point labels and optional whole-layer legend identity.
  Correct documentation to distinguish those two text roles and to describe
  arbitrary supplemental coordinates rather than dataframe columns or only raw
  observations.
- Continue accepting multiple appended layers. Permit repeated explicit colors;
  color uniqueness is not semantic identity.
- Preserve strict nonempty aligned finite x/y validation, optional label
  alignment, clipping, draw order over primary line geometry, automatic-range
  participation, generic-facet rejection, and exact legend conflict checks.

Acceptance:

- `PointLayer::new(x, y)` renders with deterministic publication defaults and
  requires no caller styling.
- Multiple automatic layers use stable palette order; manual style overrides
  affect points, point labels where applicable, and legend glyphs consistently.
- Point labels annotate individual rows and a legend label creates at most one
  whole-layer legend entry.
- Duplicate explicit colors are accepted, while conflicting duplicate legend
  identities still reject through exact legend validation.
- Focused line, layout, legend, composition, and compile-fail tests pass.
- `just verify` passes.

Commit: `feat(api)!: add automatic point layer styling`

## Phase R6: Narrow Numeric Label Format

**Status**: PENDING

Make the existing fixed-decimal scope explicit without expanding the public
formatter.

Scope:

- Retain the `NumericLabelFormat` name, precision builder, optional literal
  suffix, and current heatmap-cell and point-only coefficient-label consumers.
- Document that suffixes do not scale values and that the type does not own axis
  ticks, colorbars, waterfall totals, point labels, or legends.
- Record broader notation and cross-surface formatting in
  `05_DEFERRED_FEATURES.md` for reconsideration after deferred legend work.
- Do not add a prefix, notation enum, callback, format string, dormant variant,
  or unused storage.

Acceptance:

- Rustdoc and canonical specs state the exact two consumers and literal-suffix
  behavior.
- Existing fixed-decimal rendering and validation tests remain exact.
- No broader formatting API or placeholder is present.
- `just verify` passes.

Commit: `docs(api): clarify numeric label formatting`

## Phase R7: Native Test Values

**Status**: PENDING

Remove test-only dataframe emulation so tests exercise the standalone
renderer boundary directly.

Scope:

- Delete the unit-test `Fixture`, `FixtureColumn`, `DataFrame`, `Series`,
  `FixtureField`, `IntoFixtureColumn`, and `test_data!` surface.
- Delete the integration-test equivalents and the `df!` macro from
  `tests/support/data.rs`.
- Construct `NumericValues`, `CategoryValues`, `BooleanValues`, `SourceRows`,
  and `AxisValues` directly at test call sites.
- Where repeated setup is substantial, use narrowly named family-specific
  structs or helpers whose fields are native values. Do not add generic named
  columns, dynamic type dispatch, conversion by column-name string, dataframe
  aliases, or a replacement table macro.
- Preserve intentional nullable values, source-row identities, category order,
  and mismatched-length cases explicitly rather than normalizing them in a
  fixture layer.
- Remove now-empty test support modules and imports.

Migration order:

1. Migrate crate unit tests by renderer family and delete `src/test_support.rs`.
2. Migrate integration suites by renderer family and delete
   `tests/support/data.rs` when its final consumer is gone.
3. Search the full repository for every retired shim name and macro before
   completing the phase.

Acceptance:

- No test-owned generic dataframe, series, fixture-column, dynamic column
  lookup, `df!`, or `test_data!` abstraction remains.
- Tests continue to distinguish numeric, categorical, boolean, axis, and
  source-row semantics through production native types.
- Existing invalid-input tests still construct the invalid state they claim to
  exercise rather than failing earlier in test setup.
- Preserve every existing behavioral case, assertion, deterministic snapshot,
  and exact-byte or hash check. Delete only coverage whose sole subject is a
  retired fixture abstraction.
- Focused suites pass as each family migrates.
- `just verify` passes with no unintended snapshot changes.

Commit: `refactor(tests)!: use native renderer values`

## Final Completion

**Status**: PENDING

The currently authorized remediation is complete only when:

- Phases R1 through R7 are `COMPLETE` with commit and verification evidence.
- Repository searches find no retired API names, report strings, compatibility
  aliases, tuple KDE representation, or dataframe-shaped test shims.
- Public-path compile-fail tests enforce private modules and exact-family
  erasure.
- `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` describe the implemented
  current contract without documenting deferred choices as current behavior.
- `just verify` passes from the completed worktree.

Completion of these phases closes the currently identified API review findings.
