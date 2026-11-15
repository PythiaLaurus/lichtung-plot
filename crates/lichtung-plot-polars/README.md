# lichtung-plot-polars

Plot eager Polars fields or borrowed `Series`/`Column` variables directly. The
standalone renderer does not depend on this adapter. The complete native API is
also re-exported unchanged: `Plot`, `Canvas`, `Theme`, `kind`, `stats`, native
`Error` and `Result` keep their original definitions and usage.

```rust
use lichtung_plot_polars::{AdapterError, Output, Scatter};

fn render(frame: &polars_core::prelude::DataFrame)
    -> std::result::Result<Output, AdapterError>
{
    Ok(Scatter::from_data(frame, "time", "response")?
        .hue("condition")?
        .svg("response")?)
}
```

`Scatter::new(&time, &response)` accepts direct Series or Columns without a
DataFrame or user conversion. Both constructors return the exact-family
`PlotAdapter`; only `from_data` borrows a DataFrame for later field lookup. Each
binding snapshots its selected values. Explicit conversion to `Plot<kind::Scatter>`
with `.into()` releases the source context for native Canvas/application consumers.
Output methods return native `Result<Output>`; `AdapterError` preserves native
errors when a combined workflow uses `?`. No public adapter result alias is added.

`into_parts()` releases lookup and returns the native Plot plus opaque,
source-free `BindingMetadata<K>`. Restore the pair with `from_parts`, or
`from_parts_with_data` to borrow a fresh DataFrame for later setters. Restoration
does not refresh values or validate matching versions; callers must keep each
same-family Plot/metadata candidate pair together.

`ColumnProjection::new(source_rows, source_names, columns)` borrows selected
Columns plus the complete ordered raw names namespace, including unimported
fields. It remains a collection even with one column and installs no lookup
source. Existing extractors retain dtype, cell-shape and exact-ID checks.
`ColumnProjection::numeric_names(&frame)` shares the existing primitive-numeric
discovery predicate. This Rust facility is not evidence of an Arrow/Python bridge.

Constructors are available for all 25 native families,
including supplied bounds, bar sample/estimate modes and flat supplied geometry.
`Heatmap::new(&numeric_frame)` binds a rectangle; `Heatmap::from_data(frame, ["a",
"b"])` selects columns and `Heatmap::records_from_data(frame, "x", "y", "value")`
binds categorical cell records. `Matrix` binds ordered numeric variables, with
explicit `numeric_from_data` discovery. Independent `PointLayerAdapter` and
`BarPointLayerAdapter` have their own direct/source-bound constructors.

Histogram/Raincloud `geometry` binds complete flat columns, with
`geometry_with_hue` for hued Rainclouds. Source forms prefix bins/summary plus
two optional independent component DataFrames. Each domain aligns independently;
only the primary source remains for subsequent lookup. Sample Histogram
`composite_precomputed_kde(x, y)` and its independent-source form configure KDE+rug
without changing samples, bins or stat. Dictionary keys match decoded raw strings,
and supplied Raincloud point alpha follows original point rows, not summary order.

`Line::from_data(&frame, "time", ["a", "b"])` preserves actual x with ordered
many-y series. `indexed` methods explicitly choose row position; distribution
`columns` methods explicitly choose column-category samples. Row, expanded-cell
and generated-column bindings have explicit scopes. Statuses, exact-byte/native
alpha mappings, keyed/bin alpha and independent significance result tables bind
directly. Mapping callbacks materialize once and preserve native declarations.
The previous public value-conversion functions are not a second API.

Null remains distinct from present NaN. No sorting, dropping, imputation,
parsing, temporal/Decimal reinterpretation, nested flattening or query execution
is performed by binding. Native families retain their own statistical behavior.
All chunks, sliced validity and repeated scalar-column lengths are honored;
scalar binding may populate Polars' materialization cache. Numeric primitives
round normally to f64; categorical/Enum values decode actual strings, not codes or
schema order. Boolean values are not numeric 0/1, and source IDs stay exact integers.
Depend on matching Polars 0.55 types; this package uses polars-core directly without
facade IO/lazy features. Upstream core still includes Tokio/async dependencies.

Run the self-contained nullable/chunked synthetic scatter example:

```sh
cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations
# Optional: -- path/to/output-directory
```

This package is not yet published. See the peer
[English](../../docs/en/src/sections/polars.md) and
[Chinese](../../docs/zh-CN/src/sections/polars.md) guides and the shared
[adapter contract](../../specs/09_DATA_ADAPTERS.md).
