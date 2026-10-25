<a id="polars"></a>
## Polars: eager columns, native snapshots

[简体中文](../zh-CN/index.html#polars)

![Grouped synthetic observations converted from a nullable Polars frame](gallery/polars/grouped_observations.svg)

`lichtung-plot-polars` accepts `&Column`, the value returned by modern DataFrame
column lookup. A standalone Series uses Polars' existing `IntoColumn` conversion.
The adapter depends directly on polars-core 0.55.2; a matching polars facade
reexports the same types. The core renderer does not depend on either framework.

This synthetic example appends two response chunks and leaves one response null.
Sample IDs identify original observations independently of their current slots:

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:chunked-values}}
```

Conversions preserve all input slots. Native scatter preparation decides which
observations are scientifically usable; the adapter does not filter or renumber
the source. All values are owned after conversion, so the example drops the frame
before publication and reuses the frozen native Output for the second SVG:

```rust
{{#include ../../../../crates/lichtung-plot-polars/examples/grouped_observations.rs:observation-plot}}
```

| Function / dtype | Meaning |
| --- | --- |
| `numeric(&Column)` | Signed/unsigned 8/16/32/64/128-bit integers and Float16/32/64; normal f64 rounding without an exactness gate. |
| `categories(&Column)` | String/Categorical/Enum; decode exact string identities, not dictionary codes. Native order is first-observed, not Enum schema order. |
| `booleans(&Column)` | Boolean true/false/null; no numeric/string coercion. |
| Untyped Null | Any of the three explicit value-channel functions returns an all-null snapshot of the same logical length. |
| `source_rows(&Column)` | Integer IDs checked exactly against usize, then native uniqueness; null, negative, out-of-range and noninteger IDs fail. |
| `numeric_columns(&DataFrame, names)` | Exactly the named numeric variables, in selection order. No regex, expression or automatic discovery; missing/wrong-type selections fail. |
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

Run `cargo run --locked --offline -p lichtung-plot-polars --example grouped_observations`.
No output argument is required; one optional directory redirects the SVGs and is
created if needed. The adapter requests no facade IO/lazy features, but upstream
polars-core still includes Tokio/async dependencies; it is not a runtime-free graph.
[Complete source](examples/lichtung-plot-polars/grouped_observations.rs) ·
[Conversion API](https://docs.rs/lichtung-plot-polars/latest/lichtung_plot_polars/index.html) ·
[Back to native data](#data)
