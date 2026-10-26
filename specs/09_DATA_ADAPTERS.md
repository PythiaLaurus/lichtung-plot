# Data Adapter Contracts

## Current Implementation And Replacement

The conversion-only packages described below exist and have been verified, but
their public design has been rejected: callers should pass foreign data to
plotting interfaces, not first construct native values themselves. These sections
describe the current code, not the intended replacement or an accepted completion
of the adapter objective. The [plotting-interface redesign](92_DATA_ADAPTERS_PLAN.md)
owns the selected replacement design: typed variable/column-collection binding,
direct vectors, matrix/record heatmaps and local supplemental-layer contexts.
It compares the upstream input models and records lifecycle/forwarding costs;
this replanning change does not implement it.

## Native Boundary

Data adapters are separate workspace members consuming the deliberate public API
of `lichtung-plot`. The renderer never depends on an adapter or its foreign data
framework, including through development dependencies. The core remains the
default member. The current packages export conversion functions rather than
plotting entry points. They do not own scientific analysis, another rendering
path, a runtime or asset provisioning. The restriction against adapter plotting
frontends is not retained as a requirement for the replacement.

Conversion borrows a foreign input only while constructing existing owned native
NumericValues, CategoryValues, BooleanValues or SourceRows. No mutable foreign
buffer is retained; later source edits or destruction do not edit the snapshot.
Copying is intentional, not a zero-copy promise.

- Ordinary integer/floating numeric inputs normalize to f64 with normal rounding,
  without a losslessness gate or strictness mode. For example, u64
  `9007199254740993` rounds to `9007199254740992`.
- Null remains a missing slot, distinct from present NaN, infinity and signed zero.
  Conversion does not compact, sort, impute, aggregate or filter. Native consumers
  retain their scientific validation and applicability policies.
- Names and category keys preserve their bytes. Native categories use the first
  observed non-null keys; Labels and explicit CategoryOrder remain plot policies.
- SourceRows uses exact non-null nonnegative integer IDs fitting usize, followed
  by native uniqueness validation. IDs never pass through f64. Logical input
  positions, caller source IDs and category codes are distinct. A view cannot
  infer the application's original row IDs. Native plot validation checks map
  alignment after attachment; the builder setter itself does not check lengths.
- Adapter errors own structural/conversion context and preserve native/upstream
  error sources. Later scientific/plot/render errors remain core errors. Fallible
  adapter buffer reservations do not promise recovery from all allocations in
  native constructors or upstream libraries.

## ndarray

`crates/lichtung-plot-ndarray/src/lib.rs` exports adapter-owned free functions.
Accept ArrayView1/ArrayView2 by value; owned/mutable/Arc-backed arrays use read-only
`.view()`. Sealed NumericElement, CategoryElement, BooleanElement and
SourceRowElement bounds admit the concrete primitive families, not external scalar
plugins or a blanket Display conversion.

| Function | Result |
| --- | --- |
| numeric(name, view1) | Named NumericValues from f32/f64, signed/unsigned 8/16/32/64/128-bit integers, isize/usize and their Option forms. |
| categories(name, view1) | Named CategoryValues from String/&str and their Option forms. |
| booleans(name, view1) | Named BooleanValues from bool/Option<bool>, not implicit 0/1. |
| source_rows(view1) | SourceRows from integer primitives/Option forms, with missing/range/uniqueness errors. |
| numeric_columns(names, view2) | Ordered Vec<NumericValues>; rows are observations, columns are named variables. |
| heatmap_values(value_name, view2, x_keys, y_keys) | HeatmapValues with public owned native x/y/fill fields. |

All functions return adapter Result values. Every value/table/grid conversion has
a corresponding `_with_validity` function with the Boolean validity view as its
last argument. SourceRows has no validity overload: an absent ID is an error.
Each heatmap axis is `(channel_name: &str, raw_keys: &[&str])`.

### Iteration And Validity

Logical indices, not physical memory order, determine output order. Support safe
views with negative/positive strides, reversed axes, transposes, Fortran layout and
caller-created broadcasting. Do not require contiguous slices, inspect pointers,
flatten an unspecified rank, or broadcast between channels. Dynamic-rank callers
explicitly convert dimensionality upstream; scalar/higher-rank inputs are not
silently flattened.

Validity uses **true = present**, unlike NumPy masked-array true-means-missing
masks. Shapes must match in every dimension, not merely in flattened length.
Data and validity may have different strides. Option absence and false validity
both produce native null, retaining the slot and other channels' alignment.
Present NaN stays NaN. Complex/custom/decimal/half elements require explicit
upstream normalization or projection; no implicit string parsing is provided.

### Tables And Grids

`columns.rs` requires exactly one supplied name per logical column and preserves
their order and bytes. Duplicate/empty names are not silently repaired; the native
consumer owns any stronger variable-name or minimum-variable rule. Zero rows
produce empty named columns; zero columns produce an empty vector. This operation
can feed a native scatter matrix but does not interpret a table as a heatmap.

`grid.rs` requires x key count equal to ncols and y key count equal to nrows, with
unique raw keys within each supplied axis. Cell `(r,c)` becomes x key `c`, y key
`r`, fill `view[[r,c]]`, in slot `r*ncols+c` using checked size arithmetic. Invalid
cells retain their x/y keys and null fill. Duplicate axes are rejected, not
aggregated or renamed. Zero-sized valid shapes produce empty native fields, not
an invented unobserved category domain. Expanded provenance requires one unique
ID per cell, not repeated row IDs.

`snapshot.rs` owns the production logical-iteration/validity conversion feeding
native constructors; tests can inspect those small intermediate buffers without
exposing core payload getters. `provenance.rs` reports invalid IDs by logical slot
and delegates uniqueness to SourceRows. Numeric scientific validation stays with
the native consumers.

### Dependencies And Consumers

The workspace pins ndarray 0.17.2 with default features disabled and only `std`.
Neither BLAS, rayon, serde nor matrixmultiply threading is requested by the
adapter. Versions/lints are inherited with a single root lockfile and a versioned
local core dependency.

`examples/strided_heatmap.rs` is a self-contained synthetic assay example using
alternate source columns. It runs with no arguments, or one nonempty output
directory created by the example if needed. Outlined and preserve-text SVGs reuse
the same frozen Output.
It is one public workflow shared by both guide editions, not separate language
or per-family rendering tests. Package README and license texts remain member-local.

## Polars

`crates/lichtung-plot-polars/src/lib.rs` exports numeric, categories, booleans,
source_rows and numeric_columns free functions. Single-column entry points accept
`&polars_core::prelude::Column` and use its source name. DataFrame column lookup
returns this type; standalone Series inputs use Polars' existing IntoColumn route.
The adapter accepts only eager inputs, never LazyFrame or query expressions.

| Entry point / input dtype | Conversion |
| --- | --- |
| numeric: Int8/16/32/64/128, UInt8/16/32/64/128, Float16/32/64 | Ordinary f64 normalization, preserving null and present nonfinite values. |
| categories: String | Exact present string keys and nulls. No numeric parsing or formatting. |
| categories: Categorical, Enum | Decode actual string identities, never physical dictionary codes. Rebuild first-observed native levels; unobserved levels are not extra observations, and Enum schema order is not an implicit CategoryOrder. |
| booleans: Boolean | True/false/null, not implicit numeric or string data. |
| numeric/categories/booleans: Null | The explicit entry point chooses an all-null destination channel of the same logical length. |
| source_rows: integer dtypes | Exact checked usize IDs, followed by native uniqueness validation. Null, negative and out-of-range IDs fail by logical slot; floating/string/categorical/Null dtypes are not coerced. |
| Temporal, Decimal, binary, nested, object, extension and unknown dtypes | Unsupported implicit interpretation, including zero-length inputs. Explicitly choose units/project upstream; physical storage does not establish scientific meaning. |

`numeric_columns(&DataFrame, names: &[&str])` returns ordered native columns from
exactly the literal names selected. No regex/expression/discovery, silent omission,
sorting or deduplication occurs. Missing/wrong-type selections fail with the named
column's context, without partially mutating the frame or publishing a destination.
Native consumers retain their own name/minimum-variable constraints.

### Logical Representation

The conversion modules dispatch on supported logical dtype before reserving or
materializing data; unsupported empty/semantic columns are not accepted through
a physical-type or all-null fallback. Typed iterators span all chunks, respecting
sliced validity offsets. Rechunking is not required and caller data is not edited.
Repeated ScalarColumns use full logical length, including null and zero length.
Calling as_materialized_series may allocate and populate an upstream internal cache;
as_materialized_series_maintain_scalar is not a logical full-column iterator.

Numeric conversion uses typed primitive casts, not the default NonStrict Polars
cast that can turn failure into null. Categorical/Enum decoding uses a dtype-bounded
fallible strict cast to String, with its original error source retained. This
operation introduces neither numeric exactness checking nor a general strict mode.
All channels preserve slot order and validity; snapshot buffers feed the actual
public native constructors and are inspectable by local unit tests without adding
core payload getters. Adapter errors and subsequent core scientific/render errors
remain distinct.

### Dependencies And Consumer

The workspace pins polars-core 0.55.2 directly with defaults disabled and the
dtype-i8, dtype-i16, dtype-u8, dtype-u16, dtype-i128, dtype-u128, dtype-f16 and
dtype-categorical features. A matching Polars facade reexports these same public
types. No facade default IO/temporal, dtype-full, lazy/cloud/query or ndarray
bridge features are requested. Upstream core still includes polars-async, Tokio
and Rayon dependencies; this is not an async/runtime-free graph. Foreign version
families are not bridged by adapters or compatibility modes.

`examples/grouped_observations.rs` builds a nullable two-chunk response column,
categorical conditions and explicit integer sample IDs from synthetic observations.
The frame is dropped before publication to demonstrate native snapshot ownership.
Outlined/preserve-text SVGs reuse one frozen native Output; the same execution
serves both guide editions. Like the ndarray example, zero arguments use the
working directory and one optional nonempty directory is created when needed.

## Documentation Ownership

The Book helper explicitly allows implemented member packages. Gallery inventory
identities include package and example; source includes, source downloads, API
links and cargo execution use that same owner. Downloads live under
`examples/<package>/...`; staging preserves package-local support without reaching
into another member's private example code. There is no legacy core-only catalog,
arbitrary command runner or dependency from the renderer to documentation tooling.

Follow [development](90_DEVELOPMENT.md) for focused tests, strict Clippy, whole-suite
closure and independent-package verification. Verify actual implementations as
they land, not speculative compatibility projects before writing conversion code.
Unverified package distribution is reported separately from conversion behavior;
no check authorizes publishing, deploying or pushing.
