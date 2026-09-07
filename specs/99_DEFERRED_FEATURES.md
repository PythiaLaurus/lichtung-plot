# Deferred Features

**Status**: Deferred and non-current

## Contract

This document records concrete features deliberately excluded from the current
API. It is not a compatibility promise, implementation authorization, or reason
to add placeholder enums, flags, builders, storage, or fallback behavior. A
feature moves into the current contract only after its semantics are designed,
its admitted implementation phase in `91_IMPLEMENTATION_PLAN.md` is implemented,
and the canonical architecture and rendering contracts are atomically updated
to describe that implemented behavior. Approval or planning alone is not a
current API change.

T5a0s in `91_IMPLEMENTATION_PLAN.md` is an explicit user-approval gate for the
revised layout proposal. Exact-width/automatic-height sizing, narrow long-label
controls, independent caption alignment, and font fallback consequences are
being reconsidered there, not admitted by this document. Existing exclusions
stay non-current until answers are recorded and contracts updated.

## Legend Composition Modes

The current legend uses one exact combined entry for every observed independent
`(group, hue)` collective mark. A future legend-style design may add a caller
choice between combined entries and separate group-style and hue-color sections.

Before admission, that design must specify ordering, titles, repeated hues,
same-field collapse, sparse facets, Canvas shared-legend compatibility, label
mapping, collision planning, and behavior for row-level hue families. No current
legend mode or style option reserves this extension.

## Broader Numeric Formatting

The current `NumericLabelFormat` owns fixed-decimal precision and one optional
literal suffix only for heatmap cell values and point-only coefficient estimate
labels. It does not select scientific, engineering, compact, percentage, or
locale-aware notation and does not apply to axes, colorbars, waterfall totals,
point labels, or legends.

Broader numeric formatting will be reconsidered after the deferred legend
composition and styling contracts, as part of a later cohesive review of newly
admitted formatting needs across those numeric surfaces. That review must define
scaling, rounding, negative zero, extreme finite values, width measurement,
collision handling, and deterministic SVG/PNG agreement. No notation enum,
format string, callback, prefix, or cross-surface formatter is reserved now.

## Data-Mapped Alpha

The approved target in `03_THEME_SYSTEM.md` admits RGBA element paints but keeps
categorical and continuous data-color scales opaque. Mark and typography styles
own one final alpha value; alpha is never multiplied with a second opacity
property.

A future alpha scale would need to define its numeric or categorical domain,
range, legend representation, interaction with RGBA exact styles, missing
values, accessibility, facets, Canvas coordination, and precedence. No current
alpha-scale enum, callback, or placeholder reserves that feature.

## Broader Automatic Tick Control

The approved T6 target in `05_LAYOUT_SYSTEM.md` admits one optional per-axis
preferred data interval. `TickInterval::additive` applies to linear numeric axes;
`TickInterval::log10_decades` applies to Log10 axes. The renderer tries the
requested complete interval vector first, then deterministic coarser complete
vectors when no closer source fits, and finally one terminal midpoint source.
Interval vectors never use evenly indexed subsequences, and source precedence is
resolved locally at the assigned span under a declared source order and cap, not
by global root ranking. Exact `AxisTick` values remain the strict
position/label control and take precedence regardless of setter order.

Target counts, maximum counts, exact generated counts, visual-density scalars,
arbitrary multiplicative ratios, categorical stride/drop policies, and tick
generation callbacks remain deferred. A broader design must define precedence,
nice-step deviation, small-panel behavior, facets, Canvas shared dimensions,
and resource limits without introducing global search or a second axis-planning
route. No count, density, callback, or generic tick-generator placeholder is
reserved.

## Broader Text Alignment And Anchoring

The approved T6 target admits exactly one closed `CompositionTheme` root
title-block alignment with `Start`, `Center`, and `End`. It applies jointly to
Plot and Canvas root title, subtitle, and caption text, and every built-in theme
uses `Center`.

Independent title/subtitle/caption alignment, general `TextTheme` alignment,
panel-header, composition-header, row-title, TextPanel, legend, guide, axis,
tick, or annotation alignment, arbitrary text rotation, and backend text-anchor
access remain unavailable. A broader design must define physical versus
bidi-logical direction, per-role precedence, multiline and rotated behavior,
collision planning, and SVG/PNG agreement. No generic alignment field, anchor
setter, callback, or placeholder is reserved.

## Broader Root Constraints

The approved target in `05_LAYOUT_SYSTEM.md` admits automatic root layout, one
finite maximum width with automatic height, and one exact width-height pair. It
does not expose the private `LayoutConstraints`, `LengthConstraint`, intrinsic,
allocation, or measurement types. Automatic width derives from the existing
preferred-width metric, then height from measurement; bounded width assigns
`min(preferred_width, max_width)` without searching widths. Root viability checks
canonical preserve-text bytes and PNG source/default-300-DPI target dimensions
for the one produced layout rather than selecting an alternative. Actual
outlined parser bytes and requested PNG options are terminal-local checks;
later PNG failure does not invalidate an otherwise valid automatic SVG.

A broader public surface would need demonstrated workflows for minimum width,
minimum height, maximum height, exact single-dimension sizing, and interactions
with scientific aspect, wrapping, legends, facets, Canvas, and exact content fit.
No public constraint object, builder, callback, or alternate measurement API is
reserved now, and the private parent-constraint protocol is not an extension
surface.

## Compact Composition

The approved automatic-layout target preserves declared Canvas coordinates,
header spans, row titles, range groups, Cartesian facet coordinates, and trailing
wrap slots. It does not pack occupied cells or collapse empty rows and columns.

A future compact composition policy must define coordinate remapping, shared
axes, header spans, range groups, empty semantic combinations, and deterministic
ordering. No implicit sparse-grid compaction or placeholder policy is reserved.

## Caller-Selected Layout Objective

The deterministic parent allocation and local layout rules in
`05_LAYOUT_SYSTEM.md` are private renderer policy. A future caller-selected
objective would need typed semantics for aspect, density, whitespace, legend
flow, text measure, grid balance, and
failure under exact constraints. No scoring callback, dynamic strategy, string
mode, or public layout plugin is reserved.

## Global Layout Optimization

Exhaustive global optimization is deferred. The approved repair replaces
universal complete-candidate collections, sibling Cartesian products, exhaustive
text break partitions, and root-wide dense-rank/total-key selection with separate
intrinsics for parent allocation and one immutable measured result or typed fit
miss at assigned constraints. Local alternatives are permitted only with a
declared owner, deterministic order, and finite cap; root feedback is limited to
eight rounds and does not authorize unbounded local retries.

This deferral does not prohibit bounded containing-frame recovery: an automatic
legend fitting by itself is insufficient when its allocated plot fails. The
T5a0s/D1 proposal tries supported edges and compact reflow while measuring the
affected children under changed constraints; explicit placements stay fixed.
Its finite scope and user-visible failure behavior require approval before code.

A future optimization proposal would need demonstrated scientific workloads,
measured resource/performance evidence, semantic completeness, reproducibility,
and a coherent replacement contract. No runtime strategy switch, dormant search
route, global scoring callback, or placeholder objective is admitted now.
Existing exhaustive-search milestones remain historical achievements, not an
active optimization requirement.

This deferral does not change the approved canonical ordered font-stack/weight
and scalar-fallback policy, admit a broad cluster-fallback rewrite, alter sparse
Canvas/facet semantics, or make target public APIs current before T6. Repairing
Common/Inherited script resolution and Unicode 16 coherence is active repair,
not a new fallback policy. Preserve-text SVG remains viewer-dependent; retained
outlines and PNG use generation-host glyph geometry. Content omission, font
shrink, hidden roots, and asset provisioning are not optimization escape paths.
