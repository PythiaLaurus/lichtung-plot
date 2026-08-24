# Implementation Plan

**Status**: Active execution ledger

## Purpose

This document preserves the agreed implementation sequence across context
compaction and handoff. `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` are
the target contracts. This file records temporary code-contract gaps, phase
status, acceptance criteria, and verification evidence; it does not authorize a
second API or compatibility aliases.

Update this ledger when a phase starts, completes, changes scope, or discovers a
new blocker. A phase is complete only after its acceptance criteria and focused
tests pass and `just verify` succeeds.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`.

## Completed Foundations

| Work | Status | Evidence |
| --- | --- | --- |
| Typed plot-family constructors and capability markers | COMPLETE | `aab43eb` |
| Specialized plot contracts | COMPLETE | `74af33b` |
| Atomic legend policy | COMPLETE | `1fe4620` |
| Atomic facet ownership | COMPLETE | `247976a` |
| Plot-wide source-row ownership | COMPLETE | `0bacb15` |
| Typed interaction styles and estimate statuses | COMPLETE | `6a134f2` |

These foundations remain subject to the corrective phases below. Completion of
an earlier phase does not waive a later audit finding.

## Documentation Foundation

**Status**: COMPLETE

Verification: `just verify` passed on 2026-08-24.

Scope:

- Update architecture and rendering contracts to the decisions recorded here.
- Add this durable execution ledger.
- Add `05_DEFERRED_FEATURES.md` for explicitly non-current work.

Acceptance:

- Current contracts contain no obsolete last-write-wins group/hue semantics for
  independent collective dimensions.
- Every agreed correctness and final API phase appears below.
- Deferred features do not create placeholder public APIs.
- `just verify` passes.

## Correctness Phase C1: Canonical Validation

**Status**: COMPLETE

Verification: focused `composition` and backend lifecycle tests plus
`just verify` passed on 2026-08-24.

Extract one crate-private canonical plot validator from the standalone
lifecycle. It must validate axes, heatmap-only option ownership, aligned values,
category ordering, channels, intervals, facets, and supplemental configuration.
Call it from standalone rendering and from `Canvas` before Canvas-only rules.
Do not call full standalone `preflight` from Canvas and do not render twice.

Acceptance:

- Standalone Plot and Canvas reject the same plot-level invalid input with the
  same diagnostic.
- Alignment, estimate-status alignment, incompatible guides, and supplemental
  option conflicts fail before backend begin.
- Valid supplemental geometry still renders in Canvas.
- Focused `composition` tests and `just verify` pass.

## Correctness Phase C2: Ordered Waterfall Geometry

**Status**: COMPLETE

Verification: focused `rendering` tests plus `just verify` passed on
2026-08-24.

Build waterfall spans, range, and terminal in one sequential checked pass.
Reject each non-finite cumulative endpoint with source-row context. Use the
final running value for total labels and guides.

Acceptance:

- Floating-point cancellation cannot make the total disagree with the final
  span.
- A separately overflowing delta sum cannot reject finite sequential geometry.
- Intermediate cumulative overflow is an early data error.
- Focused `rendering` tests and `just verify` pass.

## Correctness Phase C3: Independent Collective Pairs

**Status**: COMPLETE

Verification: focused `categorical`, `interactions`, `lines`, and
`marginal-matrix` tests plus `just verify` passed on 2026-08-24.

Replace the hidden one-hue-per-group invariant with typed observed collective
identity. Density, ordinary line, interaction, marginal, matrix, and
`LinePathStyle::Hue` group geometry by observed `(group, hue)` pair. Hue-only
input creates hue marks. Trajectory and neutral line-path retain group connectors
with row-level hue points.

Rules:

- Pair order is first-observed group, then first-observed hue within group.
- Multiple groups may share a hue; one group may own multiple hue marks.
- Interaction duplicate cells use `(condition, group, hue)`.
- `InteractionStyle` remains group-keyed; `CategoricalColors` remains hue-keyed.
- Same-name group/hue channels collapse only when aligned values are identical;
  otherwise they reject.
- Marginal validation uses only retained finite pairs.
- Insufficient pair-local KDE input rejects with exact group and hue context.

Acceptance:

- Reversing source rows cannot create first- or last-hue selection.
- Every affected family covers group-only, hue-only, crossed pairs, repeated
  hues, equal channels, conflicting same-name channels, and sparse facets where
  applicable.
- Focused `categorical`, `interactions`, `lines`, and `marginal-matrix` tests and
  `just verify` pass.

## Correctness Phase C4: Exact Legend Semantics

**Status**: PENDING

Replace mandatory-point legend storage with a private closed glyph
representation that can express point, line, fill, and family-specific
composites. Replace joined-string raw keys with typed legend identity. Merge
exact entries for facets and Canvas by raw identity and full glyph signature.

Rules:

- Distinct dimensions use title `group × hue` and entry `group, hue` after each
  identity is independently resolved through `Labels`.
- Same semantic fields with identical values collapse to one identity.
- Distinct fields display both entry values even when the values are equal.
- Interaction entries preserve point shape, outline, connector pattern, and
  color.
- Density entries preserve curve line and translucent fill.
- Other families include only the point, line, and fill components visible in
  their rendered marks.
- Shared legends reject conflicting representations and never synthesize generic
  theme points over exact renderer entries.

Acceptance:

- Standalone, faceted, Canvas per-plot, and Canvas shared legends agree.
- Sparse facets produce one stable chart-wide union of observed pairs.
- Caller colors and hidden point outlines are preserved exactly.
- Focused legend, composition, interaction, and visual evidence plus
  `just verify` pass.

## Correctness Phase C5: Facet Scale Context

**Status**: PENDING

Carry the originating `FacetScales` through private panel render context after
the layout itself is removed. Do not store a second public facet representation
and do not retain recursive facet ownership on panel plots.

Acceptance:

- `Fixed` and `FreeY` unavailable estimate panels may use a shared x domain.
- `FreeX` and `Free` unavailable estimate panels require explicit x limits.
- Point-interval, coefficient, and forest families have focused coverage.
- Focused `estimates` tests and `just verify` pass.

## Correctness Phase C6: Remaining Rendering Semantics

**Status**: PENDING

- Suppress hue legends for neutral line paths whose markers are hidden.
- Validate exact numeric tick collisions from projected neighboring positions.
- Preserve every exact `AxisTick` or reject; never omit, merge, move, or rotate
  exact ticks.
- Correct the raw-overlay default test so one branch omits `.raw_overlay`.
- Prove exact `FacetCell` significance placement, not only bracket count.

Acceptance:

- Focused `lines`, `layout`, `distributions`, and `composition` tests pass.
- `just verify` passes.

## Final API Phase A1: Constructor-Owned Inputs

**Status**: PENDING

- Add exact closed input variants for trajectory, line-path, interaction,
  stacked bars, heatmap, and beeswarm where needed.
- Store required group/series and continuous-color values in those variants.
- Remove `.group` replacement capability from trajectory, line-path,
  interaction, and stacked bars.
- Remove `.color` replacement capability from heatmap and beeswarm; retain
  color label and scale configuration.
- Make `Plot::coefficient` accept exact `CoefficientInput`.
- Make `ConfidenceIntervalAlgorithm` private.

No old constructor or setter alias remains.

## Final API Phase A2: Unified Positioned Series

**Status**: PENDING

Replace `.group` and `.hue` last-write-wins aliases with one `.series` method on
ordinary bars, horizontal bars, boxplots, sample-derived violins, and
sample-derived rainclouds. Stacked bars remain constructor-only. Precomputed
raincloud geometry continues to reject sample-only configuration during
preflight; source mode does not add public typestate markers.

## Final API Phase A3: Contextual Builder Names

**Status**: PENDING

Apply the complete rename atomically through concrete `Plot<kind::...>`
implementations. Do not retain aliases.

| Current method | Final contextual method |
| --- | --- |
| `heatmap_scale`, `axis_scale` | `scale` |
| `heatmap_layout` | `layout` |
| `heatmap_annotations` | `annotations` |
| `heatmap_color_limits`, `beeswarm_color_limits` | `color_limits` |
| `bar_spacing` | `spacing` |
| `histogram_bins` | `bins` |
| `histogram_stat` | `stat` |
| `histogram_composite` | `composite` |
| `bar_point_layer` | `point_layer` |
| `qq_line` | `reference_line` |
| `regression_band` | `band` |
| `line_path_style` | `style` |
| `interaction_connectors` | `connectors` |
| `interaction_styles` | `styles` |
| `waterfall_labels` | `total_labels` |

`bins`, `stat`, `reference_line`, and every other builder use consuming
`self -> Self` style.

## Final API Phase A4: Selective Value Builders

**Status**: PENDING

- `NumericLabelFormat::default()` uses two decimal places and no suffix.
- Precision and suffix become validated selective builders; remove the duplicate
  `with_suffix` constructor.
- `SignificanceMarkStyle::default()` remains publication defaults and gains
  validated selective font-size, weight, and gap builders.
- Remove or internalize old getters where their names conflict with final
  consuming builders.

## Final API Phase A5: Plot Identity And Introspection

**Status**: PENDING

- Store `PlotKind` directly, never `Option<PlotKind>`.
- Replace `plot_kind() -> Option<PlotKind>` with `kind() -> PlotKind`.
- Retain public Plot introspection only for `kind`, `width`, `height`, and
  `preflight`.
- Internalize family-option getters and renderer configuration accessors.

## Final API Phase A6: Tick Surface

**Status**: PENDING

- Retain root `AxisTick` and exact `.x_ticks` / `.y_ticks` replacement.
- Remove `.x_tick_count` and `.y_tick_count`.
- Resolve automatic tick density deterministically from panel dimensions,
  numeric range, and measured labels.
- Do not expose caller-selected automatic density in the current API.

## Final API Phase A7: Internal Rendering Surface

**Status**: PENDING

- Make renderer-facing layout modules and types private.
- Retain `AxisTick` only at `lichtung_plot::AxisTick`.
- Remove public `Plot::palette`, `Theme::palette`, `ThemePalette`, and palette
  module exports. `Theme` remains the public built-in selection.
- Remove dead public `Scale`, `AxisSpec.scale`, and `Axis.log_scale` state unless
  a current internal renderer path consumes it.

## Final API Phase A8: Facet Construction

**Status**: PENDING

Make `FacetSpec::grid` return `Result<Self>` and reject equal semantic dimension
names at construction. Migrate every repository caller. `FacetSpec::wrap`
remains fallible for zero columns.

## Final API Phase A9: Geometry And Canvas Vocabulary

**Status**: PENDING

Rename public geometry atomically:

| Current | Final |
| --- | --- |
| `RaincloudGeometryInput` | `RaincloudGeometry` |
| `RaincloudDimensionsInput` | `RaincloudDimensions` |
| `RaincloudGroupInput` | `RaincloudGroup` |
| `HistogramCompositeGeometryInput` | `HistogramGeometry` |
| `HistogramDimensionsInput` | `HistogramDimensions` |
| `HistogramBinInput` | `HistogramBin` |
| `HistogramRugInput` | `HistogramRug` |

Replace internal and surviving public `Figure*` terminology with `Canvas*`,
including layout, insets, specifications, helpers, diagnostics, and tests. Do
not retain old aliases.

## Final Completion

**Status**: PENDING

The plan is complete only when:

- Every phase above is `COMPLETE` with verification evidence.
- Greps find no replaced API names, compatibility aliases, obsolete Figure
  terminology, broad family getters, optional plot kind, or public palette and
  layout internals.
- `05_DEFERRED_FEATURES.md` has no placeholder implementation in the public API.
- `just verify` passes from a clean prepared environment.
