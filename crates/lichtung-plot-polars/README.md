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

Record-form single-variable constructors are available for all 25 native families,
including supplied bounds, bar sample/estimate modes and native supplied geometry.
`Heatmap::new(&numeric_frame)` binds a rectangle; `Heatmap::from_data(frame, ["a",
"b"])` selects columns and `Heatmap::records_from_data(frame, "x", "y", "value")`
binds categorical cell records. `Matrix` binds ordered numeric variables, with
explicit `numeric_from_data` discovery. Independent `PointLayerAdapter` and
`BarPointLayerAdapter` have their own direct/source-bound constructors.

The replacement is still in progress: multi-y/indexed/distribution-column modes,
expanded row/cell bindings, foreign alpha/status mappings and foreign significance
result-table binding are not implemented yet. Native alpha, status, significance
and all native-only workflows remain available. The previous public value-conversion
functions have been removed, not retained as a second API.

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
