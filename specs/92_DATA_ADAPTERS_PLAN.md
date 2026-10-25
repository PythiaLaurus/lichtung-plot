# ndarray And Polars Data Adapters

## Status And Scope

This is a design and delivery proposal, not an implemented API. The current user
request authorizes planning, not adapter implementation, dependency installation,
package publication, site deployment or Git push. No adapter workspace members
exist yet. Do not add manifests, empty packages or core feature flags in this
planning change.

The [workspace/interface plan](91_WORKSPACE_AND_INTERFACES_PLAN.md) owns the broader
delivery order and Python decision gate. Current native behavior remains defined
by [architecture](01_ARCHITECTURE.md), [rendering](02_RENDERING_CONTRACT.md) and
[development](90_DEVELOPMENT.md). Promote working behavior into current contracts
when delivered; proposed names below are not compatibility commitments.

| Basis | Established fact or proposed decision |
| --- | --- |
| Existing core | `crates/lichtung-plot/src/values.rs` already exports named nullable NumericValues, CategoryValues, BooleanValues and exact unique SourceRows. |
| Existing consumers | `input.rs`, `builder.rs` and `spec/categorical.rs` provide the family constructors and explicit category-order policy; adapters need not replace them. |
| Upstream ndarray | Logical iteration differs from memory-order slices; owned arrays and noncontiguous views share the ArrayView interface. |
| Upstream Polars | DataFrame columns are Column values, including repeated ScalarColumns; Series::cast uses NonStrict conversion. |
| Proposed architecture | Two independent conversion crates, adapter-owned free functions and owned native snapshots. No foreign Plot builders, shared adapter framework or new core reader API. |
| Proposed first scope | Typed one-dimensional channels, explicit provenance, ndarray named columns and labeled two-dimensional heatmaps, and selected Polars numeric columns. |

Refine the proposed signatures through real public consumers during implementation;
no separate prototype or pre-implementation acceptance test is required. Do not
silently reduce the promised conversion semantics to fit an inconvenient upstream API.

## Dependency Direction And Ownership

```text
ndarray arrays/views -> lichtung-plot-ndarray --+
                                             +-> native values -> lichtung-plot
Polars Column/frame -> lichtung-plot-polars ---+
```

- Add `crates/lichtung-plot-ndarray/` and `crates/lichtung-plot-polars/` only with
  working conversion consumers. Neither adapter depends on the other or Python.
- The renderer remains the default member. A core-only consumer must not compile
  ndarray, Polars, PyO3 or adapter dependencies, including through dev-dependencies.
- Adapters borrow foreign inputs only during conversion, then return the existing
  owned native values. Later source edits, rechunking or destruction cannot edit
  an already constructed native snapshot or Plot.
- No retained foreign buffer, lazy expression, source-container cache, JSON
  serialization, render dispatch or statistical analysis belongs in either adapter.
  Copying is intentional; zero-copy and exact allocation-count claims are not made.
- Use ordinary public constructors. Numeric/category/Boolean payload readers and
  SourceRows storage are private today; do not expose them merely for adapter tests.
- Keep errors, conversion modules, tests, deterministic fixtures, examples, README
  and licenses in each owning package. Use `lib.rs` plus private `numeric.rs`,
  `category.rs`, `boolean.rs`, `provenance.rs` and relevant table/grid modules,
  not `mod.rs` or a public universal data model.

Returning a small `HeatmapValues { x, y, fill }` bundle of native fields is admitted
for an explicit grid expansion. It is not a retained matrix abstraction, another
Plot type, or permission to bypass native family validation.

## Shared Conversion Semantics

### Numeric Values, Validity And Categories

1. Normalize ordinary integer/floating values to f64 with normal rounding, as in
   [Matplotlib][matplotlib-conversion] and [Seaborn][seaborn-conversion]. For example,
   `9007199254740993_u64` becomes `9007199254740992.0`. Do not introduce a per-value
   exactness check, losslessness gate or public strictness switch. [NumPy casting
   categories][numpy-casting] are not per-value representability proofs.
   These precedents justify numeric normalization, not copying their filtering or
   masked-value-to-NaN policies into the native nullable boundary.
2. Preserve present NaN, infinities, signed zero and row positions. `None`/Polars
   null becomes native null; present NaN stays present NaN. Scientific filtering,
   nonfinite rejection and applicability remain with the actual native consumer.
3. Validity is not a row-selection filter. An invalid slot remains a slot; do not
   compact other channels, fill missing values or replace conversion failure with
   null/zero. Empty and all-null supported inputs return correspondingly empty or
   all-null native values; the plot decides whether they are scientifically usable.
4. Preserve names and string keys without trimming, humanization, normalization,
   sorting or number/Boolean stringification. Native Labels and explicit category
   order remain separate presentation choices.
5. Native categories use first-observed non-null raw-key order. Foreign dictionary
   codes and unobserved levels are not native category identities or extra rows.
   In particular, converting Enum values does not automatically install its declared
   order as a Plot instruction; document how the caller applies native CategoryOrder
   explicitly using the actual rendered categories. Do not pretend schema ordering
   metadata is carried inside CategoryValues.

The scope of an adapter is conversion, not a preliminary scientific-validation
pass. Do not duplicate native finite/bounds/facet/geometry validation in it.

### Provenance And Errors

SourceRows is an explicit, plot-owned, aligned diagnostic map. Integer IDs must
convert exactly to usize, be nonnegative and non-null, and satisfy the native
uniqueness constructor. This is an ID-domain check, not the numeric losslessness
gate rejected above. Never send IDs through f64, dictionary codes or generated
pointer/row-number identities. After attachment to a Plot, native input validation
checks length alignment with the other channels; the builder setter itself does
not validate it.

A slice/transposition/chunk layout does not reveal original application row IDs.
Adapter errors identify the logical input slot, or `(row, column)` for a grid;
they must not mislabel that position as a caller's SourceRows ID. Callers keep their
ID column aligned while performing any explicit upstream filtering/reordering.

Each adapter owns an Error and Result type. Cover structural shape/name/validity
mismatches, unsupported Polars dtype/channel combinations, invalid ID conversion,
and genuinely fallible conversion/allocation operations. Preserve underlying native
errors and PolarsError through `Error::source`, with relevant channel/name/slot
context, rather than reducing them to unrelated core Render strings. Native
constructor errors and later plot/render errors remain distinguishable.

Check real length products and reserve adapter-owned expanded buffers fallibly.
Do not promise recovery from every allocation: existing native NumericValues and
BooleanValues constructors and upstream operations include infallible allocations.
Do not add budgets, generic resource accounting or a new core buffer representation.

## ndarray Interface

### Proposed Public Surface

Use `ArrayView1<'_, T>` / `ArrayView2<'_, T>` arguments by value. Owned, Arc-backed,
borrowed and mutable arrays can supply a read-only `.view()`; slices and transposes
need no contiguous intermediate. There is one conversion route, not parallel
owned/view/layout-specific APIs. Dynamic-rank callers explicitly use ndarray's
dimension conversion; scalar or higher-rank inputs are not silently flattened.

The following are proposed adapter-owned free functions, not current signatures:

| Function shape | Native result and meaning |
| --- | --- |
| `numeric(name, view1)` | NumericValues; primitive numeric or nullable primitive elements. |
| `categories(name, view1)` | CategoryValues; String / `&str`, or their nullable forms. |
| `booleans(name, view1)` | BooleanValues; bool / Option<bool>. |
| `source_rows(view1)` | SourceRows; integer IDs, with null/range/uniqueness errors. |
| `numeric_with_validity(name, view1, validity1)` | NumericValues; explicit same-shape Boolean validity. |
| `categories_with_validity(name, view1, validity1)` | CategoryValues with the same validity convention. |
| `booleans_with_validity(name, view1, validity1)` | BooleanValues with the same validity convention. |
| `numeric_columns(names, view2)` | Vec<NumericValues>; rows are observations, columns are named variables. |
| `numeric_columns_with_validity(names, view2, validity2)` | The same column conversion with an explicit same-shape validity grid. |
| `heatmap_values(value_name, view2, x_keys, y_keys)` | HeatmapValues; each cell becomes one native `(x, y, fill)` slot. |
| `heatmap_values_with_validity(value_name, view2, x_keys, y_keys, validity2)` | The same grid expansion without dropping invalid cells. |

Use small sealed element bounds to accept the concrete families below, including
their Option forms, without blanket Display/ToString conversion, overlapping
blanket implementations or a new generic scalar hierarchy. The validity variants
are named data operations, not configuration modes or alternate representations.

| Element family | Planned interpretation |
| --- | --- |
| f32, f64; signed/unsigned 8/16/32/64/128-bit integers; isize, usize | Numeric conversion using ordinary f64 rounding. |
| Option of those numeric primitives | Same conversion, preserving null independently of NaN. |
| String, `&str`, and their Option forms | Exact native category keys. |
| bool, Option<bool> | Native Boolean channel, not implicit numeric 0/1. |
| Integer primitives and their Option forms in `source_rows` | Checked exact usize IDs; None is an error. |
| Complex, custom objects, strings-as-numbers, third-party decimal/half types | No implicit interpretation; callers explicitly normalize/project with the source library. This does not settle the later Python dtype table. |

### Logical Order And Explicit Shapes

[ndarray's iterator contract][ndarray-iteration] visits logical row-major order,
rightmost index fastest. `as_slice_memory_order()` explicitly promises something
different. Support positive/negative strides, reversed axes, Fortran layout,
transposition and caller-created broadcast views through safe logical iteration.
Do not require `as_slice()`, inspect raw pointers, infer storage origin, or add a
storage-order option. An already broadcast view is real logical input; the adapter
does not invent cross-channel broadcasting itself.

Validity uses **true = present**, false = native null. It must have the exact
logical shape of the corresponding view, not merely the same flattened length.
Each validity view may have its own strides. Option absence and false validity
both remain null; valid NaN stays NaN. Do not interpret this as NumPy masked-array
true-means-missing semantics. Any later binding must translate that convention
explicitly at its own input boundary.

For `numeric_columns`, require exactly one supplied name per logical column and
preserve caller order and bytes. Do not silently select variables, transpose,
rename duplicates or impute values. Return native columns even when rows are empty;
zero columns return an empty vector. Native `Plot::matrix` still decides its own
minimum-variable count, unique names and scientific validity. It is a scatter
matrix, not a synonym for an arbitrary ndarray numeric grid.

For `heatmap_values`, the naming proposal for each axis is a tuple
`(channel_name: &str, raw_keys: &[&str])`, so x/y identities and names are explicit
without another axis-schema object. Require `x_keys.len() == ncols` and
`y_keys.len() == nrows`, with nonduplicated keys within each supplied axis: duplicate
keys cannot identify distinct grid cells in the native categorical model. Do not
normalize keys to make them unique. Cell `(r,c)` reads `view[[r,c]]`, x key `c`,
y key `r`, and produces slot `r*ncols+c` using checked arithmetic. This slot is not
an invented SourceRows ID. A supplied cell-provenance map must have one ID per
expanded cell, not one repeated ID per original matrix row.

Zero-sized valid shapes produce empty native fields. They do not preserve an
unobserved categorical domain in a hidden schema object or promise an empty plot
will render successfully. The adapter neither fabricates data nor aggregates
duplicate observations to fill a heatmap.

### Intended Consumer Shape

Illustrative proposed calls, not compiled examples or implementation evidence:

```rust
use lichtung_plot::{Bounds, Plot};
use lichtung_plot_ndarray as nd;

let plot = Plot::line(
    nd::numeric("Time (h)", time.view())?,
    nd::numeric("Response", response.view())?,
).band(Bounds::new(
    nd::numeric("Lower", lower.view())?,
    nd::numeric("Upper", upper.view())?,
));

let cells = nd::heatmap_values(
    "Signal", signals.view(),
    ("Sample", &sample_keys), ("Program", &program_keys),
)?;
let heatmap = Plot::heatmap(cells.x, cells.y, cells.fill);
```

This is enough to reuse native bounds, layers, facets, grouping, themes, callbacks
and publication across applicable families. Do not add 25 ndarray Plot constructors.

## Polars Interface

### Proposed Public Surface

Prefer `&polars_core::prelude::Column` as the single-column entry boundary. It is
what modern DataFrame column lookup actually returns; [Column][polars-column]
includes both ordinary Series columns and repeated scalar columns. A standalone
Series uses Polars' existing `IntoColumn`/`From<Series>` conversion, not another
adapter API or an orphan-rule-invalid foreign TryFrom implementation.

| Proposed function | Meaning |
| --- | --- |
| `numeric(&Column)` | Named NumericValues from the supported numeric dtype set. |
| `categories(&Column)` | Named CategoryValues from String/Categorical/Enum/null data. |
| `booleans(&Column)` | Named BooleanValues from Boolean/null data. |
| `source_rows(&Column)` | Exact checked integer provenance; never a generated row index. |
| `numeric_columns(&DataFrame, names: &[&str])` | Ordered Vec<NumericValues> from exactly the named columns, using the same single-column conversion. |

The source column's name becomes the native name. Frame selectors are literal
names, not regexes, expressions or automatic numeric-column discovery. Preserve
selection order; do not sort or silently omit a wrong-type/missing column. This
helper is not another DataFrame wrapper. Duplicate requested names are not renamed;
the eventual native consumer owns any unique-variable-name constraint.

Illustrative proposed use, not a claim that these adapter crates exist:

```rust
use lichtung_plot::Plot;
use lichtung_plot_polars as pl;

let plot = Plot::scatter(
    pl::numeric(frame.column("concentration")?)?,
    pl::numeric(frame.column("response")?)?,
)
.hue(pl::categories(frame.column("condition")?)?)
.source_rows(pl::source_rows(frame.column("sample_id")?)?);
```

### Dtype Table

This table is explicit channel dispatch, not permission to cast every foreign
dtype to its physical storage. [Polars' dtype inventory][polars-dtypes] includes
Float16 and 128-bit integers as well as semantic and nested types.

| Input dtype | Planned result |
| --- | --- |
| Int8/16/32/64/128, UInt8/16/32/64/128 | `numeric`: normal f64 rounding; null retained. No per-value exact-integer gate. |
| Float16/32/64 | `numeric`: numerical f64 normalization; null distinct from present NaN/infinity. |
| String | `categories`: exact present strings; null retained. No parsing into numbers. |
| Categorical, Enum | `categories`: decode actual string identities with validity; rebuild the native first-observed dictionary. Never use physical codes as labels or IDs. |
| Boolean | `booleans`: true/false/null retained; no automatic numeric or string conversion. |
| Null | Requested numeric/category/Boolean channel becomes all-null of the same length, including length zero. The explicit entry function chooses the destination channel. |
| Integer dtypes in `source_rows` | Non-null nonnegative values that fit usize, then native uniqueness validation. Floating/string/categorical/null ID dtypes are not coerced. |
| Decimal, Date, Datetime with/without timezone, Duration, Time | Unsupported implicit conversion. Caller explicitly chooses units or numeric meaning upstream; never reinterpret physical integers as plotted values. |
| Binary/BinaryOffset, List, fixed Array, Struct, Object, extensions/unknown dtypes | Unsupported channel interpretation; no explode, parsing, flattening, Debug strings or JSON fallback. |

Supported empty typed columns remain supported; an unsupported dtype does not
become accepted merely because it has zero rows. Supported all-null typed columns
and untyped Null values retain their length. Null ID rejection and invalid ID range
are provenance failures, not ordinary numeric conversion failures.

### Chunks, Scalar Columns And Fallible Casts

- Iterate the complete logical column across all chunks and sliced validity
  offsets; never read only the first Arrow chunk or require a contiguous slice.
  Rechunking must not be a prerequisite or a hidden edit to caller data.
- Materializing a ScalarColumn into its full logical Series is permitted as part
  of snapshot conversion and must be documented as allocation, not zero-copy.
  Do not use `as_materialized_series_maintain_scalar()` as if its unit-length
  backing data had the full column length. Logical source values remain unchanged,
  although Polars may populate an internal materialization cache.
- [Polars distinguishes null from NaN][polars-missing]. No fill_null/fill_nan,
  drop_nulls, default value, sortedness trust or string formatting shortcut belongs
  in this boundary. Respect Categorical/Enum mappings even when physical category
  code width or global mapping differs.
- Prefer dtype-directed typed iteration into native construction. A bounded
  upstream cast for an allowed dtype is acceptable if it preserves validity and
  reports real failure. [Series::cast defaults to NonStrict][polars-series-source],
  so it is not a safe generic conversion fallback. If `strict_cast` is used
  internally, it prevents failed casts from becoming null; it must not add an
  exactness policy or reject normal integer-to-f64 rounding.
- Accept only already eager Column/DataFrame inputs. Do not accept LazyFrame,
  execute queries, read files/URLs, or treat upstream IO support as authorization
  for network access or asset provisioning.

## Dependencies And Package Verification

Candidate dependency lines for implementation are ndarray **0.17.2** and
polars-core **0.55.2**, compatible in declared version families with the previously
researched future NumPy/Polars bridges. [Published ndarray metadata][ndarray-meta]
declares Rust 1.64; [polars-core metadata][polars-meta] declares edition 2024 but no
rust-version. Neither published metadata nor source inspection proves that the
full dependency graph builds on the pinned Rust 1.96 toolchain. Resolve dependencies
in the actual adapter implementation and validate them through its required strict
Clippy run, not a separate compatibility probe before writing code. Do not upgrade
the toolchain or choose a different Python lifecycle as an incidental adapter change.

Proposed workspace-managed dependency configuration:

- ndarray with defaults disabled and only `std`; no BLAS, rayon, serde or
  matrixmultiply-threading feature requested by this adapter.
- Direct polars-core dependency with defaults disabled, using its public
  Column/Series/DataFrame types that the matching polars facade reexports. Enable
  the supported contract's `dtype-i8`, `dtype-i16`, `dtype-u8`, `dtype-u16`,
  `dtype-i128`, `dtype-u128`, `dtype-f16` and `dtype-categorical` features rather than
  exposing ordinary dtype support as user-selected adapter modes.
- Do not enable the facade's default CSV/Parquet/temporal stack, `dtype-full`,
  lazy/query/IO/cloud features or Polars' ndarray bridge. The [core feature
  manifest][polars-core-manifest] is narrower; facade `dtype-categorical` also
  reaches IO/ops dependencies. Do not promise an async/runtime-free graph:
  polars-core itself currently depends on polars-async and Tokio.
- One root Cargo.lock, inherited versions/lints, adapter-to-core local path plus
  version, and no adapter dependency in the renderer. Feature unification must not
  accidentally make the standalone consumer compile a foreign framework.
- Do not download dependencies or provision tools from renderer/build.rs/site
  recipes. Missing development dependency cache is a setup/verification blocker,
  not permission to replace locked/offline verification with an unreported mode.

Independent package verification belongs to delivery after working members and
consumers exist; it is not a prerequisite for conversion implementation. The core
is currently unpublished and has `publish = false`, so an in-workspace path
dependency build alone cannot prove that an unpacked adapter resolves its core
dependency. At package delivery, use [Cargo's supported packaging][cargo-package]
to verify the actual normalized archives and their packaged dependency sources,
not checkout paths, temporary manifest patches or fabricated registry substitutes.
If the needed registry version or stock-Cargo route is unavailable, report package
verification as blocked without blocking unrelated conversion work. `--no-verify`
or silently publishing the core is not a solution. Building coordinated local
archives is not evidence of registry publication.

## Documentation And Executable Consumers

Deliver both language sections with each working adapter, under the existing
single-page guides. Include the actual owning package's examples through semantic
anchors; keep examples self-contained when unpacked, with zero arguments by default
and one optional output directory. Do not reach into a sibling package's private
example support through relative paths.

The current docs helper is core-only: `content.rs` whitelists the renderer's
examples, `main.rs` stages that directory, and `gallery.rs` hardcodes its package
in cargo run. Extending those three owners is part of adapter delivery, not an
already available capability. Make case/source/download/API identities
package-aware using an explicit allowlist of implemented members, without arbitrary
commands or a generic workflow engine. Migrate existing links and inventory to one
current representation; do not retain compatibility catalogs or filename fallbacks.

Use one representative executable consumer per adapter initially: an ndarray
labeled heatmap from a genuinely strided view, and a Polars grouped observational
scatter from nullable/chunked input with explicit IDs. These are proposed examples,
not authorization to inflate gallery/E2E counts. Use meaningful variables and
truthful data provenance; inspect real full-size and thumbnail output. Reuse these
same executions/artifacts for the docs instead of another per-language or per-family
rendering suite. Rustdoc documents the conversion API; guides explain when to use it.

## Verification And Capacity

The last established census is **800 unit tests** (795 core + 5 Book helper),
**299 module-integration tests**, and a conservative **78 core-E2E scenarios**
(60 prior core + 18 current gallery root/constituent scenarios). This is planning
baseline evidence, not a new census run or a separate allowance for adapters.

When changing tests, classify the added contracts and review the affected layer's
count under the existing development policy. This is part of the code change,
not a separate repository-wide audit before implementation. At a ceiling, remove
only genuinely redundant/low-value coverage with explicit retained witnesses;
do not promise uninspected deletions or merge/parameterize/relabel scenarios to fit.
Count reference routes, doctests, composed producers, examples and automated gallery
scenarios by their actual contracts. Report a real capacity conflict in the affected
test changes rather than exceeding the ceilings or reducing adapter behavior.

| Evidence | Required semantic coverage, not a mechanical cross-product |
| --- | --- |
| Numeric conversion | Independently expected normal integer rounding, representative width/sign boundaries, present NaN/infinity/signed zero, null and all-null/empty distinctions. |
| ndarray layout | A transpose/Fortran view that exposes wrong memory-order traversal; a reversed/strided view; independent validity strides and a same-length/wrong-shape mismatch. |
| Grid/table roles | Names-to-columns alignment, explicit x/y mapping, checked expansion, null cells retained, raw axis-key collision and empty-shape behavior. |
| Polars representation | Cross-chunk sliced validity, repeated ScalarColumn length including null/zero length, decoded categorical/Enum identities, explicit selected-column order and unsupported logical dtype. |
| Provenance | Exact IDs beyond binary64 integer precision when usize permits, null/negative/out-of-range rejection, duplicate native IDs and retained logical source slots. |
| Ownership | Source mutation/destruction after conversion cannot edit the retained snapshot; failed conversion does not partially mutate a destination or logical input values. |
| Public consumer | Representative adapter-to-native preparation/publication and frozen Output reuse, using the same admitted workflows as executable documentation. No 25-family adapter dispatch matrix. |

Test production conversion helpers against small independent expected buffers where
needed; these must feed the real native constructors, not be a test-only alternate
conversion path. Native payloads being opaque is not grounds for public getter
inventories, renderer-private imports or copied validators. Do not pin SVG length,
scene hashes, automatic panel size or internal layout coordinates.

Once the relevant code exists, run affected conversion tests and public consumers,
source/peer guide checks and strict workspace validation:

```sh
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
```

`cargo check` and a successful non-linting build are not substitutes. At implementation
closure, satisfy the broad verification required by development policy for actual
dependency changes; state that reason before the run. Local fixes after that use
their focused regressions, not repeated whole-suite runs. At delivery, independently
unpacked packages and a standalone core-only consumer need their own evidence;
workspace success alone is insufficient.
This planning-only change needs reference/source consistency and diff checks, not
compiler tests, package builds or execution of the proposed snippets.

## Implementation And Delivery Order

Implementation still requires separate direction. Once requested, start production
conversion code directly; do not insert an audit, compatibility prototype or
packaging investigation as a preliminary project.

1. **Implement ndarray.** Add its actual dependency and scalar/nullable/validity
   conversions with explicit table/grid mappings and a working strided-view consumer.
   Validate the real implementation with focused tests and strict Clippy; review
   capacity alongside the affected test changes. Deliver both guide editions and
   package-aware docs support. Keep the renderer API and dependency graph independent.
2. **Implement Polars.** Add its actual dependency and implement the dtype table,
   Column/scalar/chunk/mapping paths, exact provenance and selected frame columns
   through the same native boundary. Validate its real code and consumer, and
   deliver peer documentation without a separate advance compatibility exercise.
3. **Verify delivery.** Check both adapters together and the core alone, one lockfile,
   source-complete archives, actual guide/gallery outputs and the resulting test
   census. Verify packaging with the working packages; record unavailable checks
   without claiming distribution readiness. Commit each completed logical objective
   before unrelated work; do not publish or push without separate authorization.

If actual implementation encounters dependency resolution failure, a test-capacity
conflict, unavailable package verification or a need for new core facilities,
report the concrete affected step and alternatives. A blocked distribution check
does not prohibit unrelated implementation. Do not quietly add a foreign data
model, fallback chain or reduced capability set.
Python object mutation/copy semantics and bridge selection remain at their existing
binding-time gate; these Rust adapters neither settle nor block that decision.

## Reference Sources

Sources establish the particular upstream behaviors cited, not blanket approval
of their architecture or proof that this repository compiled their candidate
versions. No proposed adapter or dependency combination was built for this plan.

[matplotlib-conversion]: https://github.com/matplotlib/matplotlib/blob/v3.11.2/lib/matplotlib/cbook.py
[seaborn-conversion]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/_base.py
[numpy-casting]: https://numpy.org/doc/stable/reference/generated/numpy.can_cast.html
[ndarray-iteration]: https://github.com/rust-ndarray/ndarray/blob/0.17.2/src/impl_methods.rs
[ndarray-meta]: https://crates.io/api/v1/crates/ndarray/0.17.2
[polars-column]: https://docs.rs/polars-core/0.55.2/polars_core/frame/column/enum.Column.html
[polars-dtypes]: https://docs.rs/polars/0.55.2/polars/prelude/enum.DataType.html
[polars-missing]: https://docs.pola.rs/user-guide/expressions/missing-data/
[polars-series-source]: https://github.com/pola-rs/polars/blob/rs-0.55.2/crates/polars-core/src/series/mod.rs
[polars-core-manifest]: https://github.com/pola-rs/polars/blob/rs-0.55.2/crates/polars-core/Cargo.toml
[polars-meta]: https://crates.io/api/v1/crates/polars-core/0.55.2
[cargo-package]: https://doc.rust-lang.org/cargo/commands/cargo-package.html
