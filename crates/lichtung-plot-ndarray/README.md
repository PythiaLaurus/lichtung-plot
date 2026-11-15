# lichtung-plot-ndarray

Plot logical ndarray views directly, with the complete native renderer API
re-exported unchanged. The standalone renderer does not depend on this adapter.
Strided, reversed, transposed, Fortran and broadcast views need no contiguous
intermediate.

```rust
use lichtung_plot_ndarray as nd;
use ndarray::array;

fn main() -> Result<(), nd::AdapterError> {
    let table = array![[0., 1., 4.], [5., 2., 5.], [20., 3., 6.]];
    let plot = nd::Line::from_data(table.view(), 0, [1, 2])?
        .column_labels(["Control", "Treated"])?
        .x_label("Time (h)");
    let _native: nd::Plot<nd::kind::Line> = plot.into();
    Ok(())
}
```

All 25 families have direct `new` and source-bound `from_data` constructors.
The latter retains a safe read-only array descriptor for later logical-column
lookup; `.into()` releases it and moves the owned native plot. Direct vectors and
matrices are snapshotted immediately and do not become implicit lookup sources.

- `ColumnProjection::new(original_shape, &[(original_index, &one_column_source)])`
  borrows only supplied columns while preserving original bounds and selections.
  Optional `.with_identities(...)` transports raw field identities independently
  of positions and display labels; identity-only entries do not supply values.
- One-dimensional views bind variables; `usize` selects a logical source column.
- Explicit many-y matrices/index collections preserve actual x. `indexed` and
  column-distribution methods deliberately choose omitted-coordinate meanings.
- `Heatmap::new(view2)` binds a grid; `records(x, y, fill)` binds explicit cells.
  `Matrix::new(view2)` instead treats columns as scatter-matrix variables.
- `(values.view(), present.view())` binds exact-shape validity: **true = present**.
  Highlights and unique per-cell IDs are separate same-shape inputs.
- Row/cell/generated-column facets and roles, supplied bounds, bar sample/estimate
  modes, independent point layers, statuses, native alpha mappings and independent
  significance results bind directly without public data-role wrappers.
- Histogram/Raincloud `geometry` binds complete flat columns, with
  `geometry_with_hue` for hued Rainclouds. Source forms prefix bins/summary plus
  two optional independent component sources (`Option<&dyn ArraySource>`).
  Each domain aligns independently; only the primary source remains for lookup.
  Sample Histogram `composite_precomputed_kde(x, y)` and its independent-source
  form configure KDE+rug without changing samples, bins or stat.

Numeric primitives round normally to f64. Null is not NaN; raw categories remain
strings and exact IDs never pass through f64. Binding does not sort, drop,
aggregate, impute or infer statistics. Native families retain their scientific
policies and errors. Root `Error`/`Result` are native; binding returns standard
`Result<_, AdapterError>`, with lossless native-error conversion for `?`.

Run the self-contained masked, strided synthetic heatmap:

```sh
cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap
# Optional: -- path/to/output-directory
```

This package is not yet published. See the peer
[English](../../docs/en/src/sections/ndarray.md) and
[Chinese](../../docs/zh-CN/src/sections/ndarray.md) guides and the shared
[adapter contract](../../specs/09_DATA_ADAPTERS.md).
