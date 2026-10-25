<a id="ndarray"></a>
## ndarray: preserve logical axes

[简体中文](../zh-CN/index.html#ndarray)

![A labeled response grid from alternate source columns](gallery/ndarray/strided_heatmap.svg)

`lichtung-plot-ndarray` borrows an `ArrayView1` or `ArrayView2` while converting,
then returns an owned native snapshot. The core renderer does not depend on
ndarray. This synthetic assay deliberately interleaves QC columns; a strided view
selects only the response columns without a contiguous intermediate.

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-values}}
```

Grid cell `(r,c)` maps to x key `c`, y key `r`, in logical row-major order even
after transposition or reversal. Axis keys must have the corresponding lengths
and be unique within each axis. Then use ordinary native plot configuration:

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-plot}}
```

| Conversion | Accepted data and meaning |
| --- | --- |
| `numeric(name, view1)` | f32/f64 and primitive integers through 128 bits, including isize/usize and Option forms; normal f64 rounding, no exactness gate. |
| `categories(name, view1)` | String / &str and their Option forms; exact raw keys, first-observed native order. |
| `booleans(name, view1)` | bool / Option<bool>, not implicit 0/1. |
| `source_rows(view1)` | Integer IDs with exact usize range and native uniqueness checks; null is an error. |
| `numeric_columns(names, view2)` | Observations in rows, named variables in columns; suitable for native scatter matrices, not an implicit heatmap. |
| `heatmap_values(name, view2, x_keys, y_keys)` | Explicit labeled grid expansion into native x/y/fill. |

Every value/table/grid conversion also has a `_with_validity` form. The Boolean
validity view must have exactly the same shape and uses **true = present** (the
opposite of NumPy masked-array masks). False validity and Option::None retain a
null slot; present NaN/infinity is not changed to null. No rows are dropped,
sorted, broadcast across channels or imputed. Empty supported views stay empty.
Later native consumers own scientific validity and channel-length alignment.

Complex values, custom objects and semantic dtypes require explicit upstream
projection. Names/keys are not trimmed or formatted. SourceRows never passes
through f64 or guesses original IDs from a slice; expanded heatmaps need one ID
per cell. Foreign buffers are not retained, so later source mutation is safe.
Adapter errors carry structural/slot context and preserve native error sources.

Run `cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap`.
No output argument is required; one optional directory redirects the SVGs and is
created if needed. Both publications reuse the same frozen native Output.
[Complete source](examples/lichtung-plot-ndarray/strided_heatmap.rs) ·
[Conversion API](https://docs.rs/lichtung-plot-ndarray/latest/lichtung_plot_ndarray/index.html) ·
[Back to native data](#data)
