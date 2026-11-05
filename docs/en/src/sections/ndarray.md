<a id="ndarray"></a>
## ndarray: preserve logical axes

[简体中文](../zh-CN/index.html#ndarray)

![A labeled response grid from alternate source columns](gallery/ndarray/strided_heatmap.svg)

`lichtung-plot-ndarray` accepts logical views directly. All 25 families have
`new(...)` for direct variables and `from_data(view2, ...)` for source-column
bindings; both return the same exact-family `PlotAdapter`. Native `Plot`, `Canvas`,
policies, `kind`, `stats`, `Error` and `Result` are also re-exported unchanged.
The standalone renderer does not depend on ndarray.

This synthetic assay interleaves QC columns. A strided view selects responses;
an independently laid-out validity grid marks one missing measurement. True means
present, while a separate true highlight adds a border. Cell IDs stay exact and
unique rather than repeating a source-row ID across columns:

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-values}}
```

Grid ID/highlight bindings also accept ordered source-column index collections;
their selected row/column shape must match the grid exactly.

Cell `(r,c)` maps to x position `c`, y position `r`, in logical row-major order;
row zero appears at the top. Private axis identities remain separate from display
labels. Successive x/y/row-label calls preserve each other, while `.labels(...)`
replaces the complete policy. Configure and publish directly:

```rust
{{#include ../../../../crates/lichtung-plot-ndarray/examples/strided_heatmap.rs:strided-plot}}
```

| Input operation | Meaning |
| --- | --- |
| `Scatter::new(x.view(), y.view())` | Direct one-dimensional variables; numeric primitives and their Option forms normalize to f64. |
| `Scatter::from_data(table.view(), 0, 2)` | Rows are observations; `usize` selectors address logical columns. Mix selectors and direct views in one call. |
| `Line::from_data(table.view(), 0, [2, 1])` | One actual x, ordered many y; preserve irregular coordinates, never substitute row positions. Direct y may be a 2D view. |
| `Line::indexed(view2)`; equivalent Scatter methods | Explicit zero-based row-position x. Source-bound variants select indices or `..`. |
| `Boxplot::columns(view2)`; equivalent Violin/Raincloud methods | Column identity is category, cells are samples. Ordinary category/many-y constructors instead use positioned series. |
| `Heatmap::new(view2)` / `records(x, y, fill)` | Explicit rectangle versus categorical cell records; no implicit binning. |
| `Matrix::new(view2)` | Columns are scatter-matrix variables, not heatmap cells; `column_labels` changes display metadata. |

Empty/duplicate/out-of-range selections fail at binding. Ordinary `..` means all
columns; numeric discovery has explicit names such as `indexed_numeric_from_data`.
An array has one element type: a numeric column is not automatically categorical
hue. Bind a separate string view for `.hue(...)`, `.group(...)`, `.series(...)` or
facets. Supported strings preserve raw keys; Boolean values are not numeric 0/1.

After wide expansion, ordinary categorical/facet and observation-alpha bindings
repeat by source row. `_cells` methods explicitly bind expanded slots, and
`hue_columns`, `group_columns`, `series_columns` and generated-column facet methods
select the generated dimension. Scope is never guessed from matching lengths.
Many-y Line bounds require matching collections/matrices; one pair is not reused.

`estimate_status_boolean` and `estimate_status_categorical` explicitly choose
availability. Alpha accepts exact non-null u8 variables, explicit Single values
(`64u8`, distinct from selector `0usize`), native mappings and keyed/bin units.
Numeric/category `_map` callbacks receive native values prepared by the adapter;
row mappings materialize once before expansion and preserve their declaration.
Generated-column alpha keys are zero-based source indices (local ordinals for a
direct matrix), not display labels. Independent `_from_data` selector/result
sources are resolved at the call and never joined to the main plot.

`significance_pairwise` / `significance_attached` bind supplied p-values or typed
marks; category/series target pairs may contain nested `(values, present)` inputs.
Facet/grid forms retain exact ownership. Native target/mark scalars explicitly
broadcast; other inputs agree on result count independently of observation count.
Reusable root functions return native annotation collections. No tests or p-value
adjustments are calculated. Independent `PointLayerAdapter` and
`BarPointLayerAdapter` likewise bind only their own source and record count.

Validity pairs require identical shape in every dimension, but may have different
strides. False validity and Option absence retain null slots; present NaN/infinity
is not converted to null. Transposes, reversals, Fortran storage and caller-created
broadcasts follow logical order. No sorting, filtering, imputation, numeric-to-string
conversion or inter-channel broadcasting is added. Scientific validation remains native.

Each binding snapshots only its inputs. `from_data` retains a read-only source
borrow for later selectors; direct views expire after binding. Standard `.into()`
yields `Plot<kind::Family>` and releases lookup before native Canvas insertion or
source mutation. Frozen Output never reopens the source. Binding uses `AdapterError`;
terminals return native results, with lossless native-error propagation into
`AdapterError`. There is no public conversion-function catalog or result alias.

Run `cargo run --locked --offline -p lichtung-plot-ndarray --example strided_heatmap`.
No output argument is required; one optional directory redirects the SVGs and is
created if needed. Both SVG publications reuse the same frozen native Output.
[Complete source](examples/lichtung-plot-ndarray/strided_heatmap.rs) ·
[Plotting API](https://docs.rs/lichtung-plot-ndarray/latest/lichtung_plot_ndarray/index.html) ·
[Back to native data](#data)
