# Deferred Features

**Status**: Deferred and non-current

## Contract

This document records concrete features deliberately excluded from the current
API. It is not a compatibility promise, implementation authorization, or reason
to add placeholder enums, flags, builders, storage, or fallback behavior. A
feature moves into the current contract only after its semantics are designed in
the canonical architecture and rendering specifications and its implementation
phase is added to `04_IMPLEMENTATION_PLAN.md`.

## Legend Composition Modes

The current legend uses one exact combined entry for every observed independent
`(group, hue)` collective mark. A future legend-style design may add a caller
choice between combined entries and separate group-style and hue-color sections.

Before admission, that design must specify ordering, titles, repeated hues,
same-field collapse, sparse facets, Canvas shared-legend compatibility, label
mapping, collision planning, and behavior for row-level hue families. No current
legend mode or style option reserves this extension.

## Legend Outline Box

A future legend-style design may allow callers to show or hide an outline box
around a legend and configure its supported first-party appearance. It must be
designed together with broader legend styling so inside clearance, outside
reservation, collision bounds, themes, facets, Canvas shared legends, SVG, and
PNG all consume one resolved box model.

The current API exposes no legend-box flag or style placeholder.

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

## Custom Palettes

The current API accepts built-in `Theme` values and keeps resolved palette data
private. A future custom-palette contract may admit caller-owned colors, fonts,
and selected visual grammar, but only after defining structural validation,
minimum color counts, deterministic fallback prohibition, contrast ownership,
font policy, serialization expectations, and exact facet/Canvas behavior.

Detached mutable palette inspection is not retained as a substitute for this
design.

## Caller-Selected Automatic Tick Density

The current renderer chooses automatic numeric tick density deterministically
from range, panel dimensions, and measured labels. Callers may replace an axis's
complete generated tick list with exact `AxisTick` values, but cannot request an
automatic target count.

A future density option must define whether its value is a target, maximum, or
exact count; how nice-number selection may deviate; how text fit and small
panels override it; and how facets and Canvas shared dimensions coordinate it.
No current tick-count setter or dormant field reserves this feature.
