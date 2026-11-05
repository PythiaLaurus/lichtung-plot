<a id="polars"></a>
## Polars: direct fields and column variables

[简体中文](../zh-CN/index.html#polars)

![Grouped synthetic observations converted from a nullable Polars frame](gallery/polars/grouped_observations.svg)

`lichtung-plot-polars` accepts a DataFrame with literal field names, or borrowed
Series/Column variables directly. `Scatter::from_data(frame, "x", "y")` retains
that frame for later `.hue("condition")` lookup; `Scatter::new(&x, &y)` binds
variables without an implicit table. Both return an exact-family `PlotAdapter`.
All native renderer items are re-exported unchanged through the adapter. It depends
on polars-core 0.55.2; a matching facade reexports the same Polars types. The core
renderer does not depend on either framework.

This synthetic example appends two response chunks and leaves one response null.
Sample IDs identify original observations independently of their current slots:

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:chunked-values}}
```

Bindings preserve all input slots. Native scatter preparation decides which
observations are scientifically usable; the adapter does not filter or renumber
the source. Selected values are owned, but source-bound chaining keeps a frame
borrow. The example explicitly converts into native Plot to release that context,
drops the frame before publication and reuses the frozen Output for the second SVG:

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:observation-plot}}
```

| Input role / dtype | Meaning |
| --- | --- |
| Numeric coordinates, samples, estimates or bounds | Signed/unsigned 8/16/32/64/128-bit integers and Float16/32/64; normal f64 rounding without an exactness gate. |
| Categorical roles such as `.hue("condition")` | String/Categorical/Enum; decode exact string identities, not dictionary codes. Native order is first-observed, not Enum schema order. |
| Boolean heatmap highlights | Boolean true/false/null; no numeric/string coercion. |
| Untyped Null | The selected numeric/category/Boolean role produces an all-null snapshot of the same length; ambiguous trajectory x needs an explicit axis constructor. |
| `.source_rows("sample_id")` | Integer IDs checked exactly against usize, then native uniqueness; null, negative, out-of-range and noninteger IDs fail. |
| Heatmap/Matrix column selection | Ordered literal fields; `..` means all fields, not automatic numeric filtering. `Matrix::numeric_from_data` explicitly discovers supported numeric fields. |
| Temporal, Decimal, binary, nested, object and unknown dtypes | Unsupported implicit interpretation, including empty columns. Choose units or project explicitly upstream. |

All chunks and sliced validity offsets are honored. Repeated ScalarColumns use
their logical length, not the one-element backing Series; full materialization
may allocate and populate a Polars cache. Typed empty and all-null supported
columns remain empty/all-null. Present NaN, infinity and signed zero are not null.
Later source edits/rechunking cannot edit the returned snapshot.

Names and raw keys are not trimmed or humanized. Unobserved dictionary levels do
not become extra rows; use native `CategoryOrder::explicit` when you want an exact
order of rendered categories. Conversion never sorts, drops, fills, aggregates,
parses strings or executes LazyFrame/IO. Error sources retain Polars/native errors;
invalid ID diagnostics name logical slots, not guessed original IDs.

All 25 families support direct/source bindings, supplied intervals, distinct bar
sample/estimate modes and native supplied geometry. Independent `PointLayerAdapter`
and `BarPointLayerAdapter` use their own sources and counts: a fitted line/band
never inherits observation-layer labels, hue or facets.

| Explicit operation | Meaning |
| --- | --- |
| `Line::from_data(&frame, "time", ["b", "a"])`; equivalent Scatter/sample-distribution forms | Preserve actual x/category and selected variable order; column identity supplies native group/hue or positioned series. Direct many-y accepts ordered borrowed Series/Column collections. |
| `Line::indexed_from_data(&frame, ["a", "b"])`; equivalent Scatter methods | Explicit row-position x. `indexed_numeric_from_data` discovers supported numeric columns in schema order. |
| `Boxplot::columns_from_data(&frame, ["a", "b"])`; equivalent Violin/Raincloud methods | Column identity is category, cells are samples. `numeric_columns_from_data` explicitly discovers numeric columns. |
| `Heatmap::from_data(&frame, ["a", "b"])?.row_labels("sample")?` | Rectangle with separate row display metadata; `records_from_data` instead binds categorical cell records. |
| `Matrix::from_data(&frame, ["b", "a"])?.column_labels(["B", "A"])?` | Ordered scatter-matrix variables with independent display labels, not heatmap inference. |

Empty/duplicate/missing selections fail; selecting all does not silently exclude
metadata. Numeric discovery excludes untyped Null columns, while an explicit
numeric selection resolves Null as numeric. Actual source names remain identities.
Direct collection entries use local ordinals with supplied names as display
metadata; duplicate display names retain native collision validation, not silent
merging or renaming.

Wide/grid expansion follows logical row-major order. Ordinary hue/group/series,
facets and observation-alpha bindings repeat source-row values. `_cells` methods
explicitly bind expanded slots, even when lengths coincide. Generated-column role
and facet methods select the column dimension without magic reserved field names.
Unique per-cell IDs are never made by repeating row IDs. Grid highlights/ID
rectangles must match both dimensions; `heatmap_highlight_cells` and
`source_rows_cells` also accept explicit flattened vectors. True highlights add
borders independently of null validity. Many-y Line bounds require matching column
collections. Display label calls preserve one another; `.labels(...)` replaces the
complete native policy without resurrecting old aliases later.

Status methods explicitly map Boolean or categorical availability; missing estimates
never infer it. Alpha uses exact non-null u8 variables, explicit Single values,
native numeric/category mappings and actual keyed/bin units, not arbitrary
observation reduction. `_map` callbacks receive native values prepared by the
adapter, materialize once and preserve their declaration when row-expanded.
Generated-column alpha uses raw source names as keys, or zero-based local ordinals
for direct column collections. Independent keyed/bin `_from_data` tables resolve
at the call, do not replace the main source and are not retained.

`significance_pairwise_from_data` / `significance_attached_from_data` bind an
independent result table. Targets are category bindings, category/series pairs or
typed native targets; marks are supplied p-values, native marks or explicitly
mapped categories. Native scalars explicitly broadcast; non-scalars agree on result
count, not observation count. Single/grid-facet variants preserve exact ownership.
Root preparation functions return reusable native annotations; ordinary
`.significance(...)` appends them unchanged. No hypothesis tests or p-value
adjustments are performed.

Native Canvas composition uses explicit standard conversion into an exact native
Plot. With one compatible resolved core, native types from this adapter and the
ndarray adapter are identical; neither adapter depends on the other. Binding
failures use `AdapterError`; output keeps native `Error`/`Result`, with lossless
conversion for a combined `?` workflow. No public conversion-function catalog or
adapter result alias is required.

Run `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`.
No output argument is required; one optional directory redirects the SVGs and is
created if needed. The adapter requests no facade IO/lazy features, but upstream
polars-core still includes Tokio/async dependencies; it is not a runtime-free graph.
[Complete source](examples/lichtung-plot-polars/grouped_observations.rs) ·
[Plotting API](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[Back to native data](#data)
