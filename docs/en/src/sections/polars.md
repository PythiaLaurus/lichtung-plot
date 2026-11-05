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

All 25 families currently support record-form single-variable bindings, along
with grid heatmaps, scatter matrices, supplied intervals, bar modes and independent
point layers. Multi-y/indexed/column-distribution modes, expanded row/cell metadata
and foreign alpha/status/significance mappings remain under implementation; native
policies and native annotations are already usable. Binding failures use
`AdapterError`; output keeps the native `Error`, with lossless conversion for a
combined workflow. The old public conversion functions are no longer exported.

Run `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`.
No output argument is required; one optional directory redirects the SVGs and is
created if needed. The adapter requests no facade IO/lazy features, but upstream
polars-core still includes Tokio/async dependencies; it is not a runtime-free graph.
[Complete source](examples/lichtung-plot-polars/grouped_observations.rs) ·
[Plotting API](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[Back to native data](#data)
