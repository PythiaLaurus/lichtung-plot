# Numeric Tick Generation

TickGeneration is one opaque immutable crate-root choice. Attach it with
Plot::x_tick_generation or Plot::y_tick_generation on numeric-axis families.
Each call replaces the axis's complete choice independently of format/limits:

| Constructor | Meaning |
| --- | --- |
| automatic(), also Default | Deterministic range/span/measured-label policy; explicit reset |
| additive(step) | First-tried positive finite linear spacing |
| log10_decades(step) | Positive integral exponent spacing for Forest Log10 |
| target(count) | Approximately count positions, allowing nice-step deviation and coarsening |
| maximum(count) | At most count complete generated in-domain positions, including endpoints |

Counts accept 1..=10000. Getter methods is_automatic, additive_step, decade_step,
target_count and maximum_count inspect the choice without exposing raw variants.
Actual categorical axes reject non-Automatic instructions before font discovery;
intervals must match scale kind. Any/Heatmap/Matrix have no public axis attachment.

The renderer selects the first complete fitting candidate. Preferred intervals
can coarsen locally; generated maximum filters complete sources, not labels after
selection. For domain [0,1], target(4) can produce [0,0.5,1], target(5)
[0,0.25,0.5,0.75,1] when they fit. Count 1 selects original-domain midpoint, whose
label still must fit. These choices never change scientific values/domains.

Exact x_ticks/y_ticks remain indivisible and authoritative in either setter
order, including intentionally empty labels. A generated maximum cannot truncate
an exact list. Exact positions are caller data-space values; collisions reject
rather than drop/move/rotate supplied text. NumericFormat separately controls
generated number spelling.

Facets/Canvas retain complete per-axis instructions alongside independent range
and numeric format policies. Shared ranges alone do not make panel tick choices
identical. Frozen Output reuses selected ticks/text without generation or layout.

See `examples/07_tick_generation.rs` and the
[numeric specification](../specs/06_NUMERIC_PRESENTATION.md) for complete source
ordering and bounds.
