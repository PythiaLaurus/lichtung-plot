# lichtung-plot-ndarray

Convert logical ndarray views into owned `lichtung-plot` values. The renderer
does not depend on this adapter. Arrays can be strided, reversed, transposed,
Fortran ordered or already broadcast; no contiguous copy is required as input.

- `numeric`, `categories`, `booleans`: one-dimensional primitive/nullable values.
- Matching `_with_validity` functions: **true = present**, exact-shape masks.
- `source_rows`: exact unique integer IDs, with checked `usize` conversion.
- `numeric_columns`: rows are observations; columns are explicitly named variables.
- `heatmap_values`: labeled grid cells expanded to native x/y/fill channels.

Numeric conversion rounds normally to `f64`. Null is not NaN. Nothing is sorted,
aggregated, imputed or dropped, and no foreign buffer is retained. Category keys
remain raw strings; apply native category ordering explicitly. Complex/semantic
dtypes need an explicit caller projection. This package is not yet published.

Run the self-contained synthetic heatmap example from any working directory:

```sh
cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap
# Optional: -- path/to/output-directory
```

See the peer [English](../../docs/en/src/sections/ndarray.md) and
[Chinese](../../docs/zh-CN/src/sections/ndarray.md) guides in the repository and the
shared [adapter contract](../../specs/09_DATA_ADAPTERS.md).
