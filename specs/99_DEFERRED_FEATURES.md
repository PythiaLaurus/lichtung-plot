# Deferred Features

**Status**: Deferred and non-current

## Contract

This document records concrete features deliberately excluded from the current
API. It is not a compatibility promise, implementation authorization, or reason
to add placeholder enums, flags, builders, storage, or fallback behavior. A
feature moves into the current contract only after its semantics are designed in
the canonical architecture and rendering specifications and its implementation
phase is added to `91_IMPLEMENTATION_PLAN.md`.

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

## Caller-Selected Automatic Tick Density

The current renderer chooses automatic numeric tick density deterministically
from range, panel dimensions, and measured labels. Callers may replace an axis's
complete generated tick list with exact `AxisTick` values, but cannot request an
automatic target count.

A future density option must define whether its value is a target, maximum, or
exact count; how nice-number selection may deviate; how text fit and small
panels override it; and how facets and Canvas shared dimensions coordinate it.
No current tick-count setter or dormant field reserves this feature.

## Broader Root Constraints

The approved target in `05_LAYOUT_SYSTEM.md` admits automatic root layout, one
finite maximum width with automatic height, and one exact width-height pair. It
does not expose the private `LayoutConstraints`, `LengthConstraint`, candidate
collection, or measurement types.

A broader public surface would need demonstrated workflows for minimum width,
minimum height, maximum height, exact single-dimension sizing, and interactions
with scientific aspect, wrapping, legends, facets, Canvas, and exact content fit.
No public constraint object, builder, callback, or alternate measurement API is
reserved now, and the private complete-candidate protocol is not an extension
surface.

## Compact Composition

Automatic layout preserves declared Canvas coordinates, header spans, row
titles, range groups, Cartesian facet coordinates, and trailing wrap slots. It
does not pack occupied cells or collapse empty rows and columns.

A future compact composition policy must define coordinate remapping, shared
axes, header spans, range groups, empty semantic combinations, and deterministic
ordering. No implicit sparse-grid compaction or placeholder policy is reserved.

## Caller-Selected Layout Objective

The layout quality ordering in `05_LAYOUT_SYSTEM.md` is one deterministic
renderer policy. A future caller-selected objective would need typed semantics
for aspect, density, whitespace, legend flow, text measure, grid balance, and
failure under exact constraints. No scoring callback, dynamic strategy, string
mode, or public layout plugin is reserved.
