# Native data and scientific ownership

[简体中文](../zh-CN/data.html)

The renderer consumes `NumericValues`, `CategoryValues`, `BooleanValues` and
explicit scientific input types, not a dataframe runtime. They own immutable
snapshots. Container conversion belongs at the application boundary; later edits
to a caller's container cannot change a constructed plot.

Numeric values retain original aligned slots and validity separately. Null and
NaN are not interchangeable; the actual family decides which nonfinite or
unavailable values are applicable. `SourceRows` supplies diagnostic identities,
not compacted array indices. Keep row positions, source IDs and raw category keys
distinct. Do not filter one channel without aligning all associated channels.

Categories default to first-observed order. Explicit order must name every
rendered category once; value-derived order needs an unambiguous value per
category. `Labels::humanized()` changes presentation, `Labels::raw()` preserves
spelling, and aliases override display text. None changes scientific identity.
The complete domain is checked for ambiguous displayed names.

## Samples, estimates and supplied geometry

- `BarInput::samples` displays cell means and defaults to deterministic 95%
  percentile-bootstrap intervals. `ErrorBar::confidence_interval().level(...)?`
  can select a level and Student-t or explicit `Bootstrap` seed/resample count.
- `BarInput::estimates` and `estimates_with_error_bars` accept caller-owned
  estimates/bounds without inventing confidence or credible semantics.
- `HistogramInput` and `RaincloudInput` select samples or complete supplied
  geometry at construction; these sources cannot be mixed later.
- A sample histogram can retain its bins/rug while adding a caller-computed KDE
  through `HistogramComposite::KdeAndRug(HistogramKde::Precomputed(...))` and named
  `HistogramKdePoint` coordinates. `HistogramStat::Density` changes the displayed statistic.

The renderer may compute documented display statistics, but it does not select
hypothesis tests. Supply adjusted p-values as `Significance` display data. The
following complete example constructs explicit pairwise comparisons; it performs
no inferential test:

```rust
{{#include ../../../crates/lichtung-plot/examples/03_inject_stats.rs:example}}
```

`Significance::between` infers `Pairwise` placement; attached annotations have a
different typed placement. Family capabilities determine which placements exist.
See [family selection](families.md) and [text/annotations](text.md). Data-dependent
alignment, domain and annotation errors are checked before font work where possible.
