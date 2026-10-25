# lichtung-plot-polars

Convert eager Polars `Column` inputs into owned `lichtung-plot` values. The renderer
does not depend on this adapter. All chunks, sliced validity and repeated scalar
column lengths are honored; no source buffer is retained. Scalar conversion may
populate Polars' internal full-Series materialization cache.

- `numeric`: primitive integers and Float16/32/64, with normal f64 rounding.
- `categories`: String/Categorical/Enum identities, not dictionary codes or schema order.
- `booleans`: true/false/null, without numeric coercion.
- `source_rows`: exact unique integer IDs, checked directly against usize.
- `numeric_columns(frame, names)`: explicitly selected variables, in caller order.

Null remains distinct from present NaN. No sorting, dropping, imputation,
aggregation, parsing, temporal/Decimal reinterpretation, nested flattening or
query execution is provided. Apply explicit native category ordering separately.
Use Polars' existing `IntoColumn` conversion for a standalone Series. Depend on
matching Polars 0.55 types; this package uses polars-core directly without facade
IO/lazy features. Upstream core still includes Tokio/async dependencies.

Run the self-contained nullable/chunked synthetic scatter example:

```sh
cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations
# Optional: -- path/to/output-directory
```

This package is not yet published. See the peer
[English](../../docs/en/src/sections/polars.md) and
[中文](../../docs/zh-CN/src/sections/polars.md) guides and the shared
[adapter contract](../../specs/09_DATA_ADAPTERS.md).
