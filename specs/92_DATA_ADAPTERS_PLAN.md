# Data-Source Plotting Interfaces For ndarray And Polars

## Status And Objective

Implementation of this replacement is underway. Polars now has complete native
re-exports, exact-family PlotAdapter constructors for single-variable record
inputs across all 25 families, rectangular/record heatmaps, Matrix selection,
native configuration forwarding and independently source-bound layers. Its old
public conversion catalog has been removed and its shared executable example and
peer guides use direct plotting. The ndarray package still has the conversion-only
API and has not yet migrated.

Polars also implements multi-y/indexed/distribution-column modes, explicit
row/cell/generated-column bindings, status and native alpha mappings, and
independent significance result tables. The ndarray replacement and final
cross-adapter/documentation/package closure remain. The inventory below remains the full delivery target,
not a claim that every proposed call already compiles. Implementation is authorized;
publication, deployment and Git push remain separate decisions.

The objective is to plot foreign data directly. A caller describes the dataset
and its mapping to plot roles, or supplies vectors/a matrix directly. Constructing
NumericValues, CategoryValues, BooleanValues, SourceRows or HeatmapValues must not
be a prerequisite for a foreign-data workflow. The complete native-value core API
also remains available unchanged through either adapter, not only to standalone
consumers or as an internal rendering boundary.

This plan selects top-level plot-family constructors, adapter-owned configured
plots and adapter-side data conversion. The [complete compatibility inventory](93_ADAPTER_API_COMPATIBILITY.md)
maps native types, chainable methods and composition boundaries. Rust snippets
throughout both plans illustrate the delivery target; implemented subsets are
identified above, and proposed workflows outside them are not compiled witnesses.
Implementation must
validate their inferred types, trait coherence and lifetimes, and report a concrete
incompatibility rather than silently replace these workflows with conversion helpers.

## Complete Native API And Direct Plotting Entry Points

Either adapter is a complete public entry point to the same renderer. For every
publicly usable `lichtung_plot::path`, replacing only the crate prefix with
`lichtung_plot_polars::path` or `lichtung_plot_ndarray::path` must preserve the
original definition and usage for the resolved core version/configuration. This
includes all root exports, public modules and their descendants (`kind` and
`stats` today), constructors, associated methods, traits, generic/capability
bounds, error variants and result aliases. Do not add adapter-specific allowlists
or feature gates that hide native items available in that core configuration.
Private core modules remain private.

Each adapter re-exports the complete native API and explicitly adds its own
non-conflicting root family constructors. An illustrative export excerpt is:

```rust
pub use lichtung_plot::*;
mod error;
mod frontend;
pub use error::AdapterError;
pub use frontend::{BarPointLayerAdapter, Line, PlotAdapter, PointLayerAdapter, Scatter};
```

The native exports are actual [Rust re-exports][rust-reexports], not copied
definitions, newtype wrappers or a curated subset. In particular, root Plot,
Bounds, AxisValues, HistogramInput, RaincloudInput, BarInput, BarSamples, CoefficientInput, FacetSpec,
PointLayer, BarPointLayer, Error and Result remain the native items, just like
Canvas, Theme, Color and Output. No native path moves to a nested core/prelude
module. Do not add adapter-local definitions or explicit exports that shadow a
native item. Check added root names against the native exports; a future collision
requires a coordinated API decision, never silently masking a native export.
Full native export parity is the contract; the short list here is not its limit.
The excerpt is not the complete additional family list: every family in the
constructor inventory below needs its documented top-level entry point. Production
modules remain private; users do not navigate an implementation namespace.

The native workflow needs only a prefix substitution, with no additional import,
extension trait, new `?`, changed arguments or `build()` requirement:

```rust
use lichtung_plot_polars::{kind, Bounds, NumericValues, Plot};

let native: Plot<kind::Line> = Plot::line(
    NumericValues::new("time", [0.0, 5.0, 20.0]),
    NumericValues::new("response", [1.0, 2.0, 3.0]),
)
.band(Bounds::new(
    NumericValues::new("lower", [0.5, 1.5, 2.5]),
    NumericValues::new("upper", [1.5, 2.5, 3.5]),
));
```

The same code works with the ndarray adapter prefix. Imports such as
`use lichtung_plot_polars::stats::distribution` and native `Error::Data` or
`Result<T>` also retain their original meaning. A declaration of native Plot is
not a declaration of an adapter frontend with a lookalike name.

Foreign input is selected at the adapter root through the plot family. `new` is
the default direct-input constructor; `from_data` adds a dataset and role bindings.
These are two ways to configure the same exact-family adapter plot, not two
renderers or a division between “data” and “vectors”:

| Native API, unchanged | Additional foreign-data API |
| --- | --- |
| `adapter::Plot::scatter(native_x, native_y)` | `adapter::Scatter::new(foreign_x, foreign_y)?` |
| Native Plot constructors and methods | `adapter::Scatter::from_data(source, x, y)?`, followed by source-bound setters |
| Native `Plot<kind::Line>::band(Bounds)` | Adapter Line `.band(lower, upper)?`; each input is resolved by the adapter |
| Native HistogramInput, RaincloudInput, BarInput/BarSamples, CoefficientInput | Adapter family constructors bind direct inputs and expose meaningful sample/estimate/geometry modes without parallel input types |
| `adapter::PointLayer::new(native_x, native_y)` | `adapter::PointLayerAdapter::new(x, y)?` or `::from_data(source, x, y)?` |
| `adapter::Error`, `adapter::Result<T>` | Adapter binding returns `std::result::Result<T, adapter::AdapterError>` |

`new` receives the family's direct inputs: foreign variables or matrices admitted
in its roles. It binds them and retains necessary origin metadata, but does not
install a dataset for future name/index lookup. Those inputs may themselves have come from a dataframe or
array table. `from_data` receives an explicit dataset plus family-specific role
bindings and retains that dataset as the plot's lookup context. It can also mix
field selectors and directly supplied values. Neither name partitions inputs by
container type: both consume data. Exact fixed argument lists are in the 25-family
inventory below; there is no Rust overload by argument count.

Both constructors return `std::result::Result<Self, AdapterError>` for the
adapter-owned exact-family PlotAdapter carrying the real core Plot. They differ in
the installed context, not the native rendering model. Independent prepared layers are constructed on the
actual `PointLayerAdapter` and `BarPointLayerAdapter` frontend types. Users need not
write these output types in inferred chains. Name selectors without a context
produce a binding error; literal titles and style values remain literal.

### Constructor Types And Rust `new`

`new` must construct the family frontend itself, not be an unrelated factory
returning a different type while suppressing Clippy's [`new_ret_no_self`][clippy-new]. One
candidate is a public exact-family alias of the actual PlotAdapter, for example
`type Scatter<'source> = PlotAdapter<'source, kind::Scatter>`, with family-specific
constructors implemented on that specialization. The two constructors then return
`std::result::Result<Self, AdapterError>`; the same approach applies to each of the
25 families. The exact generic and lifetime representation is not yet validated
or fixed.

Under that candidate, both `Scatter::new(...)` and `Scatter::from_data(...)`
return `std::result::Result<Scatter<'source>, AdapterError>`, with the appropriate
inferred source lifetime. After `?`, both are the same exact-family PlotAdapter,
not native Plot values or different types for the two entry points. Only
`from_data` installs a source for subsequent field lookup; direct input borrows
must not unnecessarily become the PlotAdapter's source lifetime.

The implementation prototype must prove ordinary `Scatter::new(...)` and
`Scatter::from_data(...)` inference, family-specific methods, native Plot
conversion, shared-source borrowing, and that direct input views are released
after binding. A private source-context representation may type-erase supported
borrowed datasets safely, but cannot use unsafe pointers, add an adapter dependency
to the core, or require users to create a source wrapper. Do not retain the old
`from_vectors` family methods as aliases or add a separate factory type solely to
make `new` compile. Report an actual incompatibility before changing the public
grammar. Strict workspace Clippy applies to the eventual implementation.

Native constructors stay on Plot. Family names, PlotAdapter, PointLayerAdapter,
BarPointLayerAdapter and AdapterError are root-level additions and must not shadow
any native export. Do not retain the previous planned names as compatibility
aliases. External inputs are resolved by
adapter objects and methods, not by a parallel catalog of public input-wrapper
types. Ordinary tuples, slices, views and existing native semantic policies are
used where they express the actual role; explicit methods distinguish genuinely
different data meanings. There is no public
`data(source)` factory, root family free-function catalog, `build()` method,
mandatory binding namespace or required extension-trait import for this workflow.
An explicit standard conversion can hand a bound adapter object to an unchanged
core consumer. Adapter-side binding, conversion and forwarding are the selected
baseline; potential core composition changes are separately identified for review.

Complete native API parity refers to the genuine re-exported native types and
methods, not identical signatures on the separate PlotAdapter receiver. In
particular, do not add `band_bounds` or another PlotAdapter method solely to accept a
preassembled native Bounds. A native Plot retains its original `.band(bounds)`;
an adapter Line uses `.band(lower, upper)?`. Standard conversion remains
available when an application deliberately hands off to a native Plot, at which
point the adapter's source lookup context ends. Pure native configuration types
such as Theme, Color, ErrorBar and Significance remain the original types.

### Direct Dependencies, Shared Types And Packaging

A caller needs only its chosen adapter as a direct renderer dependency, including
for native-only plots, composition, callbacks, type signatures and error handling.
The core remains a transitive dependency. Users still declare Polars/ndarray when
their own data-processing code uses those libraries; hiding those dependencies or
re-exporting their complete ecosystems is not this contract.

The two adapters are not alternatives selected by a global backend switch. They
can coexist, neither depends on the other, and both must resolve to the same
compatible core package to share native types:

```text
application -> lichtung-plot-polars  --+
            -> lichtung-plot-ndarray -+-> one resolved lichtung-plot package
```

Re-exporting a definition does not duplicate it. With that shared package,
`pp::Plot<K>` and `nd::Plot<K>` are the very same native type, as are Canvas,
Color, Theme, Output, Error, Result and the public kind/stats modules. The
PlotAdapter frontends and adapter errors are adapter-specific. Applications
normally use crate aliases rather than import both sets into the same unqualified
names; a duplicate-name import error is not a dependency or type-identity problem.

[Cargo resolution][cargo-resolution] reuses a shared dependency package where
constraints permit. Incompatible versions or distinct sources can instead resolve
to distinct packages whose native types are not interchangeable; matching a
package name alone is not enough. Keep compatible core dependency requirements
and treat core re-export compatibility as part of each adapter's public API.
The current workspace dependencies and lockfile already resolve one core package;
this is not evidence that the planned export/binding API is implemented.

An adapter's [Cargo source archive][cargo-package] contains its own package source
and dependency declarations, not a recursively embedded copy of the core or its
other dependencies. The core has its own archive. Joint offline archive
verification does not merge those archives. In one normal target/build context,
two dependency edges do not themselves produce two independent renderer crates.
Features generally combine for shared normal dependencies; different targets,
host/build contexts or build configurations may produce separate artifacts without
creating a second public native type universe in the application. Do not promise
one compiler invocation or exact final-binary size, and do not vendor/copy the
renderer into either adapter to claim standalone distribution.

## Comparative Evidence And Chosen Scope

Use [Seaborn 0.13.2's input model][seaborn-data] as evidence for user-facing data
handling, not as a requirement to reproduce its entire plotting architecture:

| Public use | Established behavior |
| --- | --- |
| `scatterplot(data=frame, x="x", y="y", hue="condition")` | The dataset is supplied once; names bind source variables to plot roles. Users do not first extract and convert every column. |
| `scatterplot(x=x_array, y=y_array)` | Direct vectors are a first-class input, not a detour through a table or named native-value constructors. |
| `lineplot(data=matrix)` | Wide data has a family-specific mapping: row position/index, cell value and column identity have different plot roles. |
| `heatmap(matrix)` | The whole matrix is the heatmap input. Row/column labels are metadata or optional presentation, not a required user-built cell table. |

[PlotData][seaborn-plot-data] resolves keys/direct vectors and constructs an
internal pandas DataFrame; its foreign-dataframe route uses the exchange protocol.
[Wide-form assignment][seaborn-wide] reshapes internally, and
[heatmap preparation][seaborn-matrix] uses ndarray/pandas internally too. Therefore
direct acceptance does **not** imply no internal conversion, zero-copy storage,
or a need to replace the renderer's data model with foreign-buffer access traits.

Do not import unrelated Seaborn behavior automatically: pandas index alignment,
numeric hue inference, filtering, temporal scales, estimators and automatic
inferential intervals require their own renderer contracts. ndarray and Polars
do not acquire pandas indexes merely because Seaborn has them.

The additional research changes concrete API decisions, not just the reference
list. Examples in this table describe each library's interface, using its own
supported containers; they do not establish cross-language Polars compatibility.

| Task and reference | Advantage | Cost or limitation | Adapter decision |
| --- | --- | --- | --- |
| [Plotly Express][plotly-input] `px.scatter(frame, x="time", y="response", color="condition")`; `px.scatter(array, x=0, y=2)` | Dataset/name binding, direct vectors and integer ndarray column selection share one high-level route; current Python Plotly supports Polars through Narwhals. | A parameter can represent a field, vector or field collection; those interpretations need explicit rules. | Give Polars literal-name bindings and ndarray logical-index bindings, with typed foreign vectors as alternatives. Do not import Narwhals or a dataframe framework into the Rust renderer. |
| [Plotly mixed-form][plotly-wide] `px.line(frame, x="time", y=["a", "b"])` | A real x column and multiple y columns need no user reshape. | Whole-table numeric discovery can accidentally include time/ID columns; generated labels cannot invent missing scientific meaning. | Support explicit single-x/multiple-y selection. Dedicated indexed/column-oriented family operations explicitly represent an omitted coordinate/category. |
| [ggplot2 layers][ggplot-layers] `geom_line(data=fit, ...) + geom_point(data=observations, ...)`; [Seaborn Objects][seaborn-layers] `.add(mark, data=other, ...)` | Layers can specify their own datasets and mappings; reuse is natural. | Inherited mappings can refer to absent fields; ggplot2 exposes inherit.aes and Seaborn Objects can index-join against constructor data. | Resolve supplemental layers in their own context. Do not inherit parent hue/facets, join indexes or require parent-sized arrays. Do not add an arbitrary-layer grammar. |
| [AlgebraOfGraphics][aog-input] `mapping(:time, direct(corrected))`; `mapping(:time, [:a, :b], color=dims(1))` | Direct vectors, mixed inputs and selected wide columns remain plot inputs. | `direct` and dimension selectors add concepts; its direct-matrix route is documented as experimental. | Rust types distinguish vectors from selectors without a mandatory Direct wrapper. Adopt ordered multi-column selection, not the complete algebra/dimension system. |
| [Altair field/datum/value][altair-encoding], [fold][altair-data] and [layers][altair-layers] | Field mappings, constants, display encodings and chart-local transforms are explicit. | Its record-oriented grammar makes ordinary matrix/vector input more elaborate; copying it would create a second graphics specification. | Keep bindings separate from literal style and labels. No field-expression strings, JSON round trip or mandatory user fold/melt. |
| [Matplotlib matrices][matplotlib-image] and [plotly.rs ndarray][plotly-rust] | Direct arrays can be converted internally; matrix orientation can be stated explicitly. | Direct acceptance is not zero-copy. plotly.rs' ndarray constructors copy into vectors and are not a table-mapping API. | Accept views directly, define row/column roles, retain private native snapshots and make their cost explicit. |

[Seaborn scatterplot][seaborn-scatter] and [Plotly Express scatter][plotly-scatter]
each accept table/field mappings and direct vectors through one Python function;
they do not establish `new`/`from_data` as a universal naming convention. In
[plotly.rs 0.14.1][plotly-rust-constructors], `Scatter::new` takes two Vecs and
feature-gated `Scatter::from_array` takes two owned one-dimensional ndarrays;
both return `Box<Scatter>` and distinguish containers, not a retained dataset
context. Here `new`/`from_data` is an explicit project design: the former keeps
ordinary direct arguments short, the latter supplies a dataset for subsequent
field bindings. Rust's lack of argument-count overloading constrains the spelling
but does not itself prove two constructor names are necessary.

### Frontend Architecture Decision

| Candidate | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| One fallible function binds all inputs, then returns the core Plot immediately | Minimal wrapper; core style methods need no forwarding. | Later `.hue("condition")`, named bounds and facets cannot resolve without another context or a large up-front options object. | Not the selected ordinary interface. |
| Top-level family constructors plus exact-family PlotAdapter | Fits existing consuming chains; later bindings retain source and expansion context; preserves family-specific capabilities. | Both adapters maintain configuration forwarding. Source-bound plots retain a lookup borrow after selected values have been copied. Existing native consumers need a conversion. | Selected for `Family::new` and `Family::from_data`; root Plot remains the native re-export. |
| General Mark/Scale/Stat grammar | Uniform arbitrary layers and reusable mappings. | Requires new composition, inference and type-capability contracts beyond data adaptation. | Not part of this replacement. |

The selected interface is Seaborn/Plotly-like in input handling, not a claim of
identical scientific defaults or implementation. Reuse the renderer instead of
adding reverse dependencies, a generic core data-access framework or a new shared
framework crate.

## Public Input Grammar

In each package, `Family::from_data(source, ...)` binds the arguments it needs
and retains a read-only descriptor of the source in its PlotAdapter. Independent
plots may borrow the same source. `Family::new(...)` binds direct
inputs without manufacturing an implicit table for later field lookup. Exact
generic/lifetime spelling is an implementation detail; the resulting family
remains the core's actual kind marker. Family frontends are additional root
exports with their own names; source state is private. Role-specific input bounds
are sealed and exposed only as required for well-formed public Rust signatures;
ordinary callers need not import, implement or name them. Do not suppress a
private-bound warning or expose a general plugin framework. Root Plot always
keeps its original native-value signatures.

| Position | Polars | ndarray |
| --- | --- | --- |
| Dataset | `&DataFrame`, eager only | `ArrayView2<T>` or `(values, present)` views, observations in rows |
| One selected variable in a data-bound plot | `"response"`: exact literal column name | `2`: zero-based logical column index |
| One direct variable | `&Series` or `&Column`, without IntoColumn ceremony | `ArrayView1<T>`, including admitted Option elements |
| Ordered selected variables | `["a", "b"]` or borrowed name slice | `[1, 2]` or borrowed index slice |
| Direct many-y input | Ordered borrowed Series or Column collection | `ArrayView2<T>`, rows aligned with x and columns representing series |
| Whole grid | Numeric DataFrame rectangle | `ArrayView2<T>` |

Collections of selectors are admitted only in a documented multi-variable
position, not automatically in every argument. A Polars slice of names is not a
vector of string observations; use a Series/Column for those observations. An
ndarray index collection is not a numeric data vector; use a one-dimensional
view for values. This makes single-field, field-collection and direct-vector
interpretations type-directed, not a length/content guessing rule.

Use sealed adapter-local input bounds where Rust needs them; they admit supported
foreign shapes and primitive types, not an external scalar/plugin protocol. Do
not make callers implement traits or build a public project-specific dataframe.
Transient direct-input lifetimes must not unnecessarily become the completed
PlotAdapter's lifetime. A missing Polars source for a string, or ndarray source for
an index, is an unresolved-binding error, never a literal/broadcast fallback.

Role chooses interpretation: numeric, categorical, Boolean and exact provenance
remain distinct. A trajectory's numeric/category x can follow an unambiguous
admitted dtype. If the source is all-Null and ambiguous, the explicit
`new_numeric_x`/`new_categorical_x` or `from_data_numeric_x`/
`from_data_categorical_x` family constructor chooses that semantic role before
binding; there is no adapter Axis descriptor. Their other arguments and
return type match the ordinary Trajectory constructors. No automatic
numeric-category formatting, string-number parsing or continuous hue is introduced.

## Concrete User Workflows

### Polars: Dataset And Variable Bindings

```rust
use lichtung_plot_polars::Scatter;

let plot = Scatter::from_data(&frame, "concentration", "response")?
    .hue("condition")?
    .source_rows("sample_id")?
    .facet_wrap("batch", 3)?
    .title("Response by condition");

let output = plot.svg("response")?;
```

The family constructor returns an exact-family PlotAdapter retaining the source
context for later bindings. A string is never an expression such as `log(x)` or
an Altair-style field/type shorthand. Missing columns identify the source name
and plot role. Literal titles, Color/Theme instructions and display labels are
not resolved as fields. `hue("red")` selects a field named red, not a paint.

Direct Column and Series inputs also work without constructing a DataFrame or
requiring `.into_column()` merely to plot:

```rust
let plot = Scatter::new(&concentration_series, &response_series)?
    .hue(&condition_series)?;

let corrected = Scatter::from_data(&frame, "time", &corrected_response)?
    .hue("condition")?
    .y_label("Corrected response");
```

Data-bound plots can mix a named source variable with a direct vector. Record-form
bindings align by logical position and must satisfy their owning input's record
count; no implicit joins, broadcasting or source-index alignment occurs. A string
without a source context is an unresolved binding error, not a vector of
characters. Source names are inherited; explicit plot labels remain optional
overrides.

### ndarray: Direct Vectors And Table Columns

```rust
use lichtung_plot_ndarray as nd;

let scatter = nd::Scatter::new(x.view(), y.view())?
    .hue(condition.view())?
    .x_label("Concentration (µM)")
    .y_label("Response");

let selected = nd::Scatter::from_data(measurements.view(), 0, 2)?
    .hue(condition.view())?;
```

One-dimensional views are accepted directly, including supported nullable
elements. In a two-dimensional table context, integer selectors name logical
columns; rows are observations. Selection does not transpose, flatten or change
the source. Names are optional metadata/presentation, not mandatory conversion
arguments. Positional defaults must not invent scientific units or variable
meaning. Standalone vectors and selected columns use the same binding rules.
An ordinary ndarray has one element type: a numeric table's third column is not
implicitly a string/categorical hue. A separate string/category view can be bound
to a numeric table plot as shown; no number-to-category stringification is added.

Logical strides, transposes, reversed axes, Fortran storage and already-broadcast
views remain supported. No contiguous intermediate is required from the caller.
When explicit validity is needed, the ordinary pair `(values.view(), present.view())`
is accepted in ndarray input positions that admit that shape, including selected
tables and grids. It is bound as one input; the binding call checks exact
rank/shape and snapshots both views. True means present; Option absence and
false validity both mean null. This is not a NumPy true-means-hidden mask or a
materialized native-value object. Ordinary views need no wrapper. The mask must
accompany its values at the same binding operation, not be attached after earlier
conversion has already interpreted them. Role-specific input traits must resolve
tuple coherence without guessing from contents. In a composite role, nesting keeps
each masked variable intact: for example, a significance category/series target
uses `((category.view(), category_present.view()), series.view())`. Prototype
these forms together; do not silently interpret an ambiguous pair by its length.

### Explicit X And Multiple Y Variables

```rust
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pp;

let lines = pp::Line::from_data(&frame, "time", ["sensor_a", "sensor_b"])?
    .facet_wrap("batch", 3)?;

let selected = nd::Line::from_data(table.view(), 0, [1, 2])?;

let direct = nd::Line::new(time.view(), signals.view())?;
```

These are one x variable and an ordered collection/matrix of y variables. If x is
`[0, 5, 20]`, both series use those actual values, never `[0, 1, 2]`. Selection
preserves order and errors on an empty collection, duplicates, missing/out-of-range
selectors or an explicitly selected unsupported dtype. A one-element collection
still requests series expansion; it is not silently changed into scalar binding.
Selecting both x and y collections does not request a Cartesian product.
Selected variables retain raw source names/indices as identities. Direct
many-Series input retains supplied names as label metadata and uses stable local
ordinals to distinguish entries; unnamed arrays use column positions. Empty names
get positional defaults, not fabricated scientific descriptions. Nonempty duplicate
display names retain native collision validation rather than silent renaming.

The Scatter and Line constructors, and the listed distribution-sample positions,
accept a single value or many values because Rust can distinguish argument types.
`from_data` has the dataset in addition to those roles; `new` does not. The
explicit-x form does not require a separate wide constructor.
Ordinary binding errors remain fallible; there is no general deferred transformation graph.
Keep the listed plain-name/index collection syntax working in the real public
consumers; do not replace it with a mandatory selector DSL during implementation.

| Multi-variable operation | Native meaning |
| --- | --- |
| `line(x, ys)` | Repeat x for each y variable; column identity supplies the default native group and hue. The core sorts each numeric series; no new estimator is installed. |
| `scatter(x, ys)` | Repeat x; column identity supplies default hue, not a fictitious connectivity group. |
| `boxplot(category, ys)`, `violin(category, ys)` | Repeat the category variable; column identity supplies the positioned series. Native sample summaries remain native. |
| `Raincloud::new(category, ys)` | The same category/positioned-series expansion, using the native sample route rather than supplied geometry. |

Other constructor positions remain single-variable unless explicitly listed.
No arbitrary wide interpretation is added to interval estimates, trajectories,
stacked totals or supplied distribution geometry.

### Wide Input Without An Explicit Coordinate

```rust
let lines = pp::Line::indexed_numeric_from_data(&frame)?;
let chosen = pp::Line::indexed_from_data(&frame, ["sensor_a", "sensor_b"])?;
let indexed = nd::Line::indexed(signals.view())?;
let distributions = pp::Boxplot::numeric_columns_from_data(&frame)?;
```

`Line::indexed`/`indexed_from_data` and the equivalent Scatter operations
explicitly choose row position as x. Use `new`/`from_data` when a real coordinate
exists. `Boxplot::columns`/`columns_from_data` and the equivalent Violin and
sample-based Raincloud operations use each selected column's identity as category
and its cells as observations. `indexed_numeric_from_data` and
`numeric_columns_from_data` are the corresponding explicit Polars numeric-discovery
operations; ndarray has the same names for a homogeneous source. Ordinary
`indexed_from_data`/`columns_from_data` accept ordered names/indices or `..` to
select every logical column. Selecting all includes nonnumeric columns and errors
if they are unsupported; numeric discovery is a distinct operation. Direct
ndarray matrix views supply all columns. These named modes require no public
row-index, column-identity or column-selection wrapper type.

Polars numeric discovery chooses supported numeric dtypes in schema order,
excluding unsupported semantic dtypes and untyped Null columns.
Explicit numeric selections can resolve Null through the requested numeric role.
Document automatic selection; do not claim the adapter can recognize a numeric
time/ID column as metadata. No usable selected variables is an error. Explicit
selections for either adapter preserve the caller's order.

After expansion, ordinary row-scoped hue/group/series/facet bindings repeat by
source row. Their default unit is fixed by the operation, not guessed from length.
Explicit `hue_cells`, `group_cells`, `series_cells`, `source_rows_cells` and
applicable observation-alpha `_cells` methods bind already-expanded values;
they exist only where the corresponding native family capability applies.
`facet_row_cells`, `facet_column_cells`, `facet_wrap_cells`,
`facet_grid_row_cells`, `facet_grid_column_cells` and `facet_grid_cells` specify
which facet input is cell-scoped; ordinary facet methods use row scope. This
distinction remains explicit even if lengths coincide. Exact per-cell IDs must
be unique; do not repeat one row ID across expanded cells.

Generated series defaults are installed through actual native roles; a later
group/hue/series setter replaces that role as in the core. For the generated
column dimension, `hue_columns`, `group_columns`, `series_columns`,
`facet_columns_row`, `facet_columns_column` and `facet_columns_wrap` select it
explicitly where the native family admits the role. `facet_grid_columns_row`
and `facet_grid_columns_column` cross that dimension with the supplied row-scoped
other facet; corresponding `_cells` methods accept a cell-scoped other facet.
Keyed alpha has an explicit generated-column method below. These are semantic
operations, not magic source-field strings. A real source field named
`variable` or `series` remains a real field. Generate collision-free private
native identities and retain their mapping; never expose a reserved-name parser.
For unnamed ndarray columns use stable positional display defaults; the adapter
`column_labels(labels)?` assigns labels to the selected/generated columns without
changing their indices. It updates the current native Labels policy, and a later
`.labels(...)` replaces that policy as described for heatmaps below. Validate counts
and native alias/collision behavior.
Do not secretly restore a replaced grouping or infer a new cross-product.

### Matrix Heatmaps And Record Cells

```rust
let plain = nd::Heatmap::new(signals.view())?;
let masked = nd::Heatmap::new((signals.view(), present.view()))?;
let labeled = nd::Heatmap::new(signals.view())?
    .x_labels(["A", "B", "C"])?.y_labels(["Sample 1", "Sample 2"])?
    .heatmap_highlight(highlights.view())?;
let records = nd::Heatmap::records(columns.view(), rows.view(), values.view())?;

let rectangle = pp::Heatmap::from_data(&numeric_frame, ..)?;
let selected = pp::Heatmap::from_data(&frame, ["sensor_a", "sensor_b"])?.row_labels("sample")?;
let records = pp::Heatmap::records_from_data(&cell_frame, "column", "row", "value")?;
```

`Heatmap::new` accepts an ndarray grid or ordered Polars numeric columns;
`from_data` accepts ordered Polars names/ndarray indices or `..` for every logical
column. `records` and `records_from_data` explicitly select x/y/fill records,
without a wrapper or argument-count overloading. No record data are implicitly
binned. The whole-frame rectangle must be supported numeric input (Null resolves
as numeric fill); it does not silently discard metadata columns. Record cells
preserve native categorical-coordinate and duplicate-cell rules.

`x_labels` and `y_labels` take ordered literal display labels, not field names;
`row_labels` binds one source field/index or direct categorical variable as
display metadata. Grid axis-label counts must match dimensions. Internally
generated axis keys stay separate from labels; labels neither merge positions nor become provenance. The
adapter maintains x/y aliases as one native Labels policy: successive x/y/row
label calls preserve one another, while a later `.labels(...)` replaces the whole
policy including those aliases as in the core. A subsequent adapter label call
updates the current policy; it must not resurrect aliases replaced by `.labels`.
Native collision/replacement rules remain. Do not attach masks after snapshotting.

| Input operation | Defined meaning |
| --- | --- |
| Rectangular heatmap | Cell `(r,c)` is fill, x is column `c`, y is row `r`; row zero is displayed at the top and columns left-to-right, consistent with the current native heatmap. Explicit native ordering may reorder presentation, not source identity. |
| Grid highlights | A same-shape Boolean ndarray view, Boolean Polars rectangle/ordered column selection, or `heatmap_highlight_cells(vector)` aligns with cells. True highlights a cell; it does not hide it or define its validity. |
| Grid source IDs | Exact same-shape integer values/selected integer columns, or `source_rows_cells(vector)`, provide one unique ID per expanded cell. A row-length ID field is not repeated across columns. |
| Record heatmap | One x/y/fill record per slot; highlights and IDs align with those records. |
| Scatter matrix | `Matrix::new(source2)` or `Matrix::from_data(source, columns)` uses columns as variables and rows as observations, with optional positional labels for unnamed arrays. This is never heatmap inference. |

Expansion order is logical row-major: `(r,c)` occupies slot `r*ncols+c` using
checked arithmetic. Retain original row and selected source-column identity
privately for diagnostics and aligned expansion. No transposition to match storage
layout, implicit compaction, imputation or visible string-key construction is
required from callers. Null grid cells retain coordinates; they are not removed.

### Supplied Bounds And Independent Supplemental Layers

```rust
use lichtung_plot_polars::{Color, Line, PointLayerAdapter};

let points = PointLayerAdapter::from_data(&observations, "time", "response")?
    .labels("sample")?
    .color(Color::rgb(90, 90, 90))
    .legend_label("Observed");

let fitted = Line::from_data(&fit, "time", "estimate")?
    .band("lower", "upper")?
    .point_layer(points)
    .title("Supplied fit and observations");

let direct_points = PointLayerAdapter::new(&observed_time, &observed_response)?
    .labels(&sample_labels)?;
```

Here fit may have 30 rows and observations 200. The interval binds to fit's estimate
records; the point layer's labels bind to its own 200 points. A layer never looks
up missing fields in the parent source, inherits parent hue/group/facets, or gains
an independent SourceRows map. There is no index join between the datasets.

`PointLayerAdapter::from_data(source, x, y)` and `::new(x, y)` snapshot
coordinates fallibly and return `std::result::Result<Self, AdapterError>`. The data
form retains only its own lookup context for later labels. Paint/radius/alpha/text setters preserve
native semantics. `point_layer(layer)` on a Line PlotAdapter accepts a native
PointLayer or a fully bound PointLayerAdapter through an adapter-local generic Into
bound and returns Self, not Result. `From<PointLayerAdapter> for PointLayer` drops
the local lookup context; it does not change native Plot::point_layer's signature.

`BarPointLayerAdapter::from_data(source, category, value, color, radius)` and
`::new(category, value, color, radius)` return `std::result::Result<Self, AdapterError>`;
an optional `.series(binding)?` resolves only in that layer context. The core still resolves
its categories/series against prepared parent bar centers, not parent row count.
Only Bar/HorizontalBar accept it. Ordinary PointLayer remains Line-only and
rejects generic faceting; no per-point hue/size mapping or arbitrary Mark layering
is added. Reuse the native types for style, endpoints and supplied geometry. The
layer frontends themselves own these constructors: `PointLayerData` and
`BarPointLayerData` are not separate factory types or compatibility aliases.

```rust
let bars = pp::BarPointLayerAdapter::from_data(
    &bar_observations, "category", "value", pp::Color::rgb(90, 90, 90), 4,
)?
.series("treatment")?;

let direct_bars = pp::BarPointLayerAdapter::new(
    &categories, &values, pp::Color::rgb(90, 90, 90), 4,
)?
.series(&treatments)?;
```

## Architecture: Adapt The Interface, Reuse The Renderer

```text
DataFrame + column bindings / foreign vectors / matrices
        -> adapter source and role resolution
        -> private native snapshots and native input constructors
        -> existing Plot<K>, scientific preparation, Canvas and Output
```

- Keep the two existing adapter packages and dependency direction. Each fully
  re-exports the native public API and adds top-level family constructors and
  typed PlotAdapter frontends, not just value converters. Both adapter roots and the
  standalone core expose the same unchanged native Plot definition.
- The adapter owns argument interpretation, source lookup, shape/dtype diagnostics
  and binding metadata. Reuse current private conversion code where sound; never
  require users to call it before plotting or rebuild a whole dataframe in a
  project-specific public format.
- Native storage/normalization is an internal implementation choice, admitted
  here. Do not add a generic core data-access trait, generic core source parameter,
  reverse optional dependency or external-buffer rendering path without a concrete
  unmet consumer. No such core refactor is needed to begin this replacement.
- PlotAdapter wraps the actual core Plot with its existing exact family marker.
  Forward appearance, numerical callbacks, text, guides and terminal methods;
  delegate scientific validation, layout, font policy, capture and publication.
  Do not copy those algorithms or maintain a second scientific configuration model.
- Preserve `kind` capability bounds. A wrapper must not expose unsupported setters
  by erasing every family. Do not rely on Deref for consuming setters: that would
  lose the binding context or return a different builder halfway through a chain.
  Small package-local forwarding macros are acceptable; a new shared framework
  crate or public universal schema is not the objective.
- Implement `From<PlotAdapter<..., K>> for crate::Plot<K>` in the originating
  adapter, preserving the exact family and moving the already-bound core value.
  The conversion drops the source context and performs no scientific
  preflight/rendering. Independently bound PointLayerAdapter and BarPointLayerAdapter
  have analogous conversions to their native layer types. All binding failures
  precede these infallible moves; native scientific validation retains its timing.
  Do not add `build`, `into_native`, `as_core` or a parallel adapter Canvas.

On PlotAdapter, forward core-only methods with their existing names, capability
bounds, consuming shape, callbacks and return behavior. Data-bearing methods
resolve the foreign inputs listed here; their fallible signatures may differ
from the original native methods. They do not replace the root native methods
or require separate native-composite compatibility-only methods. Keep one
package-local forwarding inventory and use it for review of added/changed core configuration; macros do not remove maintenance
cost. Native export parity comes from re-exporting, not from this inventory or a
promise to emulate native methods on a wrapper. Do not hide most styles behind a
required `configure_native` closure, use Deref to escape the frontend, or create
repeated per-setter tests merely to census it.

```rust
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pp;

let left: pp::Plot<pp::kind::Scatter> =
    pp::Scatter::from_data(&frame, "time", "response")?
        .hue("condition")?
        .into();
let right: nd::Plot<nd::kind::Heatmap> =
    nd::Heatmap::new(matrix.view())?.into();

let theme: nd::Theme = pp::Theme::science();
let canvas = pp::Canvas::grid(1, 2)?
    .theme(theme)
    .panel(0, 0, left)?
    .panel(0, 1, right)?;
```

The explicit conversion is a handoff, not automatic Rust argument coercion.
The annotated target also avoids assuming generic `panel(..., adapted.into())`
inference before a prototype verifies it. Neither workflow needs a direct core
dependency. The same Canvas could be named `nd::Canvas`; native Plot values cross
the two paths directly, without conversion, erasure or serialization:

```rust
fn same_native_plot<K: pp::kind::Marker>(plot: pp::Plot<K>) -> nd::Plot<K> {
    plot
}
```

### Core Adaptation For Review, Not Selected Baseline

The selected baseline implements conversion, input protocols and forwarding in
the adapters. Existing Canvas and native Plot method signatures remain unchanged.
Consequently an adapter Plot needs an explicit standard conversion before a
concrete native consumer. No new renderer dependency, core generic input model,
wrapper Canvas or hidden `build()` is introduced to disguise that boundary.

If the desired ordinary syntax is specifically `.panel(row, column, adapted)`
with no explicit conversion, the existing native Canvas must gain an acceptance
protocol, or an additional consumer with a different API must be introduced.
An adapter-local `CanvasDataExt` trait is another reviewable option: after its
import, `panel_data`/`panel_span_data` could accept any exact-family input that
converts to native Plot, including the other adapter's wrapper, then call the
unchanged native Canvas methods. This avoids a core change at the cost of an
extension-trait import and different method names. Importing both adapters'
identically named traits would need disambiguation; one generic adapter trait is
enough for both when they share the core. Its generic inference also needs a
compiled witness. It is an optional convenience for review, not a second Canvas
or a requirement for the family constructors.
One core-only candidate is an `IntoPlot` trait with an associated exact native
kind, implemented by native Plot and PlotAdapter, used by `panel` and
`panel_span`. A standard `Into<Plot<K>>` bound is another candidate. Neither is
selected or authorized for implementation by this plan: both alter the native
method signatures and require a compatibility prototype. Review ordinary native
calls, explicit family turbofish, function items, target-inferred `.into()`,
`kind::Any` rejection, both adapters, and error handling. Preserve complete native
original usage; report any incompatibility instead of declaring the generic change
automatically compatible. A distinct new panel method avoids changing the old
signature but changes the call name and must also be reviewed explicitly.

No core change is needed for standalone adapter output, native annotations,
adapter-accepting-native layer setters or explicit conversion to a native Plot.
Native `Plot::point_layer` continues to take its exact native layer; a caller
using that receiver converts an adapter layer explicitly. User-owned functions
and typed collections accepting concrete `Plot<K>` remain concrete boundaries;
core cannot make those application signatures automatically accept wrappers.
Direct foreign inputs on the original native `.hue` or structured foreign errors
inside the original native `Error` would require different core contracts and
are not needed for the selected adapter workflow. The complete tradeoff and
illustrative examples are in the [compatibility inventory](93_ADAPTER_API_COMPATIBILITY.md).

### Ownership And Failure Timing

Binding operations return standard `Result<_, AdapterError>` values. Resolve and
snapshot the variables a constructor/data setter actually binds at that call;
do not materialize every unused source column, render eagerly or accumulate a
hidden first-error state to avoid Rust's `?` syntax. Ordinary non-data setters
keep the underlying core method's shape and behavior.

A data-bound frontend retains a read-only source context while later source-name
bindings remain possible. Its lifetime must be visible in Rust, not hidden behind
an unsafe pointer. Consuming conversion drops this lookup context; the completed core Plot
and Output own their data independently. Direct-vector inputs are not retained
after snapshotting. These are Rust ownership decisions, not a claim that Seaborn
is immutable or that the future Python object lifecycle has been settled.

Data-based heatmap/matrix/wide constructors retain source lookup context; a
directly bound vector or matrix view does not become a source for later name/index
lookup. Direct grids and expanded vectors retain their required shape/origin
metadata without retaining the input view. For example, plotting direct vectors
does not make integer zero a selector into an implicit invented table. Rust must
allow a temporary direct vector/view pair to expire after its binding call while
a separate DataFrame context remains borrowed.

DataFrame/array mutation or destruction is unavailable while a retained source
borrow remains live, even though selected values have been copied. A caller can
finish binding and convert into a native Plot to release it, then drop/edit foreign
inputs before native configuration, rendering or publication. Direct-vector plots
release borrowed inputs after binding. Frozen Output replay never reopens either source.

Copying selected values, wide/grid expansion and coexistence with a borrowed
source consume memory; no zero-copy or performance improvement is claimed. Do
not normalize unused columns or clone the complete DataFrame just to erase a
lifetime. Avoid obvious duplicate binding of the same field/role within one
construction by sharing native immutable payloads where possible; no global
cache, address-based identity or speculative persistent storage is required.

Lookup/unsupported dtype/shape failures belong to the binding call, with role,
source variable and logical slot/cell context where relevant. Native scientific
errors retain their existing preflight/render timing and original error sources;
do not duplicate every core check in the frontend or parse core error strings.
In particular, finite-value/bounds/status applicability cannot be rejected early
in a way that prevents a valid later explicit status binding. Bad array rank,
length, selection, validity shape and dtype are structural binding errors. Bad
native family/unit/facet/geometry combinations keep native preflight ownership.
Consuming failure follows Rust ownership: Err does not return the previous Plot.
Terminal methods return the actual native Output and preserve native errors; they
do not wrap publication in a new lifecycle or silently retry another input route.

### Native Errors And Binding Errors

The root Error/Result are actual core re-exports, preserving native variant
matching and return types. AdapterError owns structural/lookup/dtype context and
original upstream/native error sources; adapter bindings return
`std::result::Result<T, AdapterError>` without a public result alias. AdapterError must
provide lossless `From<crate::Error>` independently of context-specific binding
variants, so a foreign-data workflow can use `?` for both binding and rendering:

```rust
use lichtung_plot_polars::{AdapterError, Output, Scatter};

fn render(frame: &polars_core::prelude::DataFrame) -> std::result::Result<Output, AdapterError> {
    Ok(Scatter::from_data(frame, "time", "response")?
        .svg("response")?)
}
```

PlotAdapter terminal methods keep native Result<Output> behavior, as do the native
Canvas and Output. Native-only functions keep returning root `Result<T>` with no
new adapter wrapping. A function combining two adapters handles their distinct
AdapterError types in its own application error enum or a standard boxed error; this
does not require importing or directly depending on the core. Do not stringify
binding errors into native Error::Data, widen the core error for foreign-library
concerns, or pretend native Result can propagate a binding error without an
explicit application-side conversion. Do not add DataResult, AdapterResult or
another adapter result alias. Moving the current adapter error types to the
non-conflicting AdapterError export and adding native From support is part of
replacement, not behavior already provided by the conversion-only packages.

## Coverage Across The Existing API

All 25 families need source-bound and direct input. Each row defines fixed
arguments of `Family::from_data(source, ...)` and `Family::new(...)`; the latter
are the same roles without a source. Fields are literal Polars names or ndarray
indices in a data-bound operation, and direct foreign variables/views in either
form. Literal references and native style policies stay literal. Both constructors
return the exact-family frontend in `std::result::Result<Self, AdapterError>`.
The documented external inputs need no native-value construction; `new` does not
promise arbitrary Rust Vec/slice support by its name. Where a data role naturally
accepts an already-owned native value such as `NumericValues` or `CategoryValues`,
allow it alongside direct foreign inputs without adding a compatibility-only
method; native inputs retain their native slot semantics rather than being
silently row-expanded. Composite native inputs remain available through Plot.

| Family | Arguments after source / `new` arguments | Direct meaning and distinct semantic operations |
| --- | --- | --- |
| Scatter | `x, y_or_ys` | One x and one/multiple y; explicit indexed operations above. |
| Histogram | `samples` | Sample mode. `geometry(geometry)` and `geometry_from_data(source, geometry)` accept native supplied geometry. |
| Density | `value` | Native sample density. |
| Qq | `value` | Native sample/reference policies. |
| PointInterval | `category, estimate, lower, upper` | Required bounds are separate bindings; explicit status remains separate. |
| Line | `x, y_or_ys` | Native sorted numeric lines; `.band(lower, upper)?` and independent PointLayer. |
| Trajectory | `x, y, group` | Constructor-owned group; explicit numeric/categorical-x operations resolve ambiguous inputs. |
| LinePath | `x, y, group` | Numeric x and original source-order paths. |
| Interaction | `condition, estimate, group, lower, upper` | Constructor-owned group and supplied bounds. |
| Coefficient | `term, estimate` | Point-only; `with_intervals(term, estimate, lower, upper)` / `with_intervals_from_data(source, ...)` selects supplied bounds. |
| Forest | `term, estimate, lower, upper, reference` | Required bounds and literal scientific reference. |
| Dumbbell | `category, left, right, endpoints` | Foreign paired values and native `DumbbellEndpoints` policy. |
| Heatmap | `columns` / `grid` | Rectangle; `records(x, y, fill)` / `records_from_data(source, x, y, fill)` explicitly selects records. |
| Bar | `category, samples` | Raw samples, native default estimate/error-bar policy; explicit modes below. |
| HorizontalBar | `category, samples` | Same raw-sample modes and native sign/reference capabilities. |
| StackedBar | `category, value, series` | Required positioned-series values and native sum semantics. |
| StackedHorizontalBar | `category, value, series` | Same stack semantics and exact family. |
| Boxplot | `category, value_or_values` | Native samples, positioned-series expansion; explicit columns operations above. |
| Violin | `category, value_or_values` | Native density/sample policies and explicit columns operations. |
| Raincloud | `category, value_or_values` | Samples; `geometry(geometry)` / `geometry_from_data(source, geometry)` selects native supplied geometry. |
| Regression | `x, y` | Existing core OLS/band policy, not a new adapter estimator. |
| Marginal | `x, y` | Native composed views and applicability. |
| Matrix | `columns` / `variables` | Ordered selection/`..` or a 2D view/ordered columns. `numeric_from_data(source)` explicitly discovers Polars numeric columns. No generic facets. |
| Beeswarm | `category, value, color` | Constructor-owned numeric color channel. |
| Waterfall | `category, delta, base` | Literal base and original ordered delta semantics. |

All additional family constructors in this inventory return
`std::result::Result<Self, AdapterError>` as well; they bind only the inputs they
receive and leave scientific validation at the native stage. Their names select
different semantics, not compatibility-only aliases. The `geometry` operations
reuse `HistogramGeometry` or `RaincloudGeometry`,
not new adapter geometry types; the data form installs a source context for
subsequent supported bindings. Native restrictions remain: supplied composite
histogram and supplied raincloud geometry do not support faceting, and supplied
raincloud geometry retains its native series restrictions. Bind geometry in its
own supplied group/point order, not by implicitly joining another source.

Bar and HorizontalBar expose the following explicit additional modes; they are
scientifically distinct, not compatibility aliases:

| Direct inputs | Source-bound inputs | Native meaning |
| --- | --- | --- |
| `new(category, samples)` | `from_data(source, category, samples)` | Samples, arithmetic mean and core default error bar. |
| `with_error_bar(category, samples, policy)` | `with_error_bar_from_data(source, category, samples, policy)` | Samples with the supplied native `ErrorBar` policy or a native `impl Into<ErrorBar>`, as accepted by `BarSamples`. |
| `estimates(category, estimate)` | `estimates_from_data(source, category, estimate)` | Already-computed estimates, no generated error bar. |
| `estimates_with_error_bars(category, estimate, lower, upper)` | `estimates_with_error_bars_from_data(source, category, estimate, lower, upper)` | Already-computed estimates and explicit supplied bounds. |

Do not infer sample versus estimate from dtype or lengths, and do not invent an
adapter-only BarData/BarSampleData. Set a sample error-bar policy at construction:
the current core Plot has no public bar-input/error-bar replacement setter, so
an adapter `.error_bar(...)` after arbitrary forwarded configuration cannot be
claimed without storing/replaying configuration or changing the core. This plan
does neither. Native `BarInput`/`BarSamples` remain usable through native Plot.
The additional named operations express distinct science; no legacy alias or
constructor overload is retained.

The [compatibility inventory](93_ADAPTER_API_COMPATIBILITY.md) specifies native
values, capability bounds, conversion and terminal behavior. Adapt every
data-bearing position: hue/group/series, provenance, facet bindings, bounds,
status, highlights, independent layers, labels and alpha. Do not require user-
constructed native values or add public data-descriptor types for ordinary
foreign-data workflows. Native style/policy/geometry types stay unchanged.

Alignment follows each owning semantic input, not one blanket DataFrame height.
Bounds and status align with their estimate records. Supplemental layers may use
their own source/direct vectors and record counts, and keyed alpha may use a
separate selector table. These inputs use explicit local contexts without changing
the parent plot's lookup source or joining datasets implicitly.

Alpha must still bind to its actual native unit (observations, records, series,
aggregates, bins or Single). Do not turn every source column into observation alpha
or reduce arbitrary rows to a collective value. Likewise, do not infer estimate
availability from nulls when the native contract requires an explicit status.

### Facets, Status And Alpha: Direct Binding Surface

| Input | Adapter operation | Resolution and alignment |
| --- | --- | --- |
| Supplied bounds | Line `.band(lower, upper)?`; the interval-family constructors above take separate bounds. | Resolve together in the owning source. Many-y bounds require matching selected-column collections or same-shape matrices; one pair is not silently reused for every series. |
| Facets | `.facet_row(v)?`, `.facet_column(v)?`, `.facet_grid(row, column)?`, `.facet_wrap(v, columns)?`, `.facet_scales(native_policy)?` | Preserve native row/column/grid/wrap exclusivity, scale policy and family restrictions. `_cells` and generated-column variants are specified above. |
| Provenance | `.source_rows(binding)?`; `.source_rows_cells(binding)?` for expanded IDs. | Exact native IDs, never an inferred ID product. |
| Status | `.estimate_status_boolean(binding)?`; `.estimate_status_categorical(binding, mapping)?`; `.estimate_status(native_statuses)` forwards native status values. | Boolean true maps to Estimated, false to NotEstimable because the method explicitly chooses that mapping. Null/unknown keys fail; missing tuples never infer status. |
| Heatmap highlights | `.heatmap_highlight(binding)?`; `.heatmap_highlight_cells(binding)?` for expanded values. | Record Boolean vector or documented cell grid; no implicit numeric 0/1. |
| Observation/record alpha | `.point_alpha(binding)?` and `.line_alpha`/`.fill_alpha` where the target actually uses observation units bind exact u8 values; `.point_alpha_linear(binding, domain, range)?` chooses a core linear map. | Resolve at the actual admitted unit; wide row values follow origin, and explicit `_cells` methods bind expanded slots. Line stroke alpha, for example, is series-scoped, not observation-scoped. |
| Single alpha | `point_alpha_single(value)?`, `line_alpha_single(value)?`, `fill_alpha_single(value)?`, where admitted. | One explicit u8 or exactly one bound value; no inference from a constant column. |
| Keyed alpha | `line_alpha_keyed(keys, values)?` and `line_alpha_keyed_from_data(source, keys, values)?`, with corresponding admitted point/fill targets. | Explicit plot-field/source-binding pairs, exact actual series/aggregate keys and optional facet keys; unique tuples, no observation reduction or implicit join. |
| Bin alpha | `fill_alpha_bins(lower, upper, facet_keys, values)?` / `fill_alpha_bins_from_data(source, lower, upper, facet_keys, values)?` on Histogram. | Exact native histogram intervals and optional owning-facet keys; no sample-to-bin inference or approximate edge matching. |

Direct binding methods accept the role's documented Polars/ndarray data; where a
same-arity method naturally accepts a native `AlphaEncoding` or native status
iterator, it can forward it without an extra compatibility method. Native Plot's
original methods and return types remain unchanged. A PlotAdapter `.facet(native)`
forwards the already-existing native FacetSpec method; it is not a second
data-conversion API. Facet methods bind and snapshot their inputs, construct
native FacetSpec and retain its clone locally so `.facet_scales(policy)?` can
replace the complete spec using its public `.scales` method. Calling it without a
facet returns a binding error. Every new facet selection replaces the previous
complete facet and its scale with the core's Fixed default, just as a new native
FacetSpec does. Native `.facet(...)` replaces the cached spec too.

For mapped alpha, `{point,line,fill}_alpha_linear(binding, domain, range)?`
provides the common observation/record-unit linear operation where that unit is
admitted. `{target}_alpha_map(binding, mapper)?` resolves the numeric binding and
passes an owned native `NumericValues` to `mapper: FnOnce(NumericValues) ->
std::result::Result<AlphaValues, Error>`, with the re-exported native `Error`;
callers can use the native `linear`, `log10`,
`continuous` or `threshold` factory. `{target}_alpha_map_categories(binding,
mapper)?` supplies native `CategoryValues` for `AlphaValues::categorical`. The
adapter constructs the values; the caller is not asked to convert a foreign column.
The returned AlphaValues must align with the supplied source at that binding;
reject a mismatched mapper output as a structural binding error, without moving
native target/unit or other scientific validation out of its normal stage.
The corresponding explicit `_single_map`, `_keyed_map`, `_bins_map`, and
`_cells_map` forms, plus `_from_data` for independent keyed/bin sources, take
the same mapper after their unit-specific arguments; categorical variants use
`_map_categories`. These express only meaningful native units, not an unrestricted
method cross-product. Preserve the native distinction between target capability
checked at compile time and unit/schema applicability checked at preflight; do
not claim every target is observation-scoped. For row-scoped
mapped alpha on expanded wide input, materialize the native mapping once from the
row values and select/repeat its members by the origin map, preserving the complete
declaration and native callback timing. Explicit cell-scoped mapping instead
materializes from the cell values. Never invoke mapping callbacks once per expanded
copy or discard declaration metadata when expanding.

For keyed operations, `keys` is an ordered collection of `(plot_field,
source_binding)` pairs, each resolving to a category variable; direct operations
use direct variables, data-bound operations may use selectors. The independent
`_from_data` form resolves its own source and callbacks at the call, not the
parent source, and retains no foreign table. Bin `facet_keys` uses the same ordered
plot-field/source-binding pairs, and may be empty. Generated wide-column identity
uses `{target}_alpha_columns(column_key, other_keys, values)?` or its independent
`_from_data(source, ...)` form instead of a magic field string; `other_keys` names
any other actual series/facet keys, and mapped variants retain the same distinction.
Symbolic column identity resolves against the owning plot while binding. Retain
native duplicate-key and exact bin-edge rules: every consumed actual unit must
resolve, while unused pooled selector tuples do not themselves create marks.

Byte inputs remain exact non-null u8, not floating opacity. For ndarray, a literal
single-byte value and an integer column selector must remain distinguishable by
type (for example `64u8` versus `0usize`), not inferred from value or length.
Mapping delegates to native AlphaValues, with native clamping, validation, declaration and callback
timing; no adapter mapping algorithm, Alpha/AlphaMap/Key public type, callback
replay, implicit reduction or join is introduced. `AlphaEncoding::theme()` can
be supplied through the same one-argument alpha method to reset a target; no
compatibility-only reset wrapper is required.

```rust
let estimates = pp::PointInterval::from_data(
    &summary, "term", "estimate", "lower", "upper",
)?
.estimate_status_categorical("status", [
    ("ok", pp::EstimateStatus::Estimated),
    ("unavailable", pp::EstimateStatus::NotEstimable),
])?;

let lines = pp::Line::from_data(&frame, "time", "response")?
    .hue("condition")?
    .line_alpha_keyed_from_data(
        &series_alpha, [("condition", "group")], "stroke_alpha",
    )?;

let points = pp::Scatter::from_data(&frame, "time", "response")?
    .point_alpha_linear("quality", (0.0, 1.0), (64, 255))?;
```

The independent `series_alpha` has one unique `group` key per actual condition,
and `stroke_alpha` is an aligned u8 column. A table with repeated observation
rows is not silently reduced into a series-alpha table.

No broad new analysis or arbitrary channel mapping follows from these helpers.
For example, Scatter still has no scientific line_alpha method, Regression's
band setter still accepts native RegressionBand rather than two data bounds,
Matrix/Marginal still lack generic facets, and supplied geometry retains runtime
restrictions. Keep the core's compile-time versus data-dependent distinction.

## Scientific Meaning Is Not Copied From A Similar Signature

| Situation | Upstream behavior | Selected adapter behavior |
| --- | --- | --- |
| Repeated x in a line call | [Seaborn lineplot][seaborn-line] defaults to mean aggregation/CI; [Plotly lines][plotly-line] connect in supplied order; [ggplot2 line/path][ggplot-path] distinguish x order and source order. | Bind into the selected native Line, Trajectory or LinePath. Do not silently aggregate, add a CI or change ordering because input came from a dataframe. |
| Raw ordinary-bar samples | A plotting library may choose an estimator by default. | The Bar sample constructor retains the existing core BarInput::samples mean and default error-bar policy; this is not a blanket promise that adapters never trigger native statistics. Estimates/bounds use separate explicit modes. |
| Numeric hue | Seaborn/Plotly can map numeric color continuously. | Normal hue remains the core's categorical capability. Heatmap/Beeswarm retain their explicit continuous channels; a shared spelling does not imply equivalent channel coverage. |
| Constant style versus field | Altair separates field, datum and value; ggplot2 separates aes mappings from fixed geom parameters. | Literal text/Color/Theme/native policies never become source lookups. Field strings never fall back to constants on lookup failure. |
| Missingness/masking | Seaborn heatmap mask=true hides a cell. | Present=true means valid; highlight=true means bordered. Native missing-cell and nonfinite rules stay distinct and applicable to their family. |

## Data Semantics To Preserve

- Retain the currently supported primitive widths and nullable/Boolean/string/
  dictionary handling. Ordinary numeric values round normally to f64; no exactness
  gate, user strictness switch or fallible-cast-to-null fallback is introduced.
- Interpretation follows the requested plot role and admitted native capability.
  This redesign does not silently add temporal/Decimal/nested interpretation or
  numeric-hue/size semantics absent from the core. Unsupported roles/dtypes produce
  direct binding errors instead of asking callers to use native conversion helpers.
- Preserve null versus NaN, signed zero, raw string keys and logical slot order.
  Polars chunks, sliced validity and repeated ScalarColumn logical lengths remain
  correct. Do not execute lazy queries, IO, joins or analysis on behalf of binding.
- Source field identity is separate from its axis/legend label. Source names and
  explicitly supplied labels are preserved; unnamed arrays need useful positional
  defaults, not mandatory fabricated names. Do not use container addresses as row
  IDs or mistake Seaborn's variable-object identity metadata for source provenance.
- Numeric/categorical inference must not erase the core's explicit semantics.
  String/Categorical/Enum decoding preserves actual observations; Enum domain
  ordering is not silently installed as native CategoryOrder. Empty domains do
  not invent rows or unobserved observations.
- Explicit SourceRows remains exact, non-null, nonnegative, unique and aligned.
  Wide/grid expansion has a one-to-many relation to source rows: expanded slots
  have `(source row, source column)` origins, not repeated unique row IDs. Accept
  explicit per-expanded-record IDs where supported; reject repeating a row ID per
  series. Default positional cell/series identities are not application provenance.
  Private origin tracking does not claim that the native Output exposes a new
  two-dimensional provenance API.
- Empty vectors and zero-row matrices retain their structural meaning; the native
  consumer decides whether a useful plot exists. An empty variable selection still
  fails in operations that explicitly require selected variables. No implicit row
  removal or imputation is added merely to match Seaborn output.

## Replacement And Delivery

1. **Replace Polars' public entry point first.** Re-export the entire native API
   unchanged and implement top-level family whole-frame/name and direct Series/Column paths,
   including explicit-x/many-y and local supplemental layers. Extend that same
   PlotAdapter across the constructor/setter inventory, with real grouped/faceted
   consumers rather than a separate facade for each example. Reuse private dtype
   converters, keep input/context lifetimes separate, and provide AdapterError with
   native-error From support.
2. **Replace ndarray's public entry point.** Provide the same complete native
   re-exports and top-level family constructors. Direct views, selected table columns,
   direct x/view2 series, matrix heatmaps and explicit multi-column bindings use
   the same rendering boundary and input grammar, not a different adapter model.
   Cover validity, bounds and independent layers too, not only a grid shortcut.
3. **Complete migration and verification.** Replace the adapter examples/guide
   excerpts with direct plotting calls, remove public numeric/categories/booleans/
   source_rows/numeric_columns/heatmap_values conversion exports and native-output
   helper bundles, and keep only private conversion support with real consumers.
   Migrate their public tests and all adapter-local exports to the selected
   non-shadowing root surface; do not retain aliases, deprecations or dual catalogs.
   Examples import every renderer item through their adapter, not a direct core import.

Migrate each package's callers with its working replacement, not only at the very
end. Core-native input constructors remain unchanged; removing adapter conversion
exports is not authorization to delete the standalone value API. Update the
current adapter contract after implementation and remove this completed plan.

These are implementation steps, not preflight projects.
No dependency upgrade, toolchain change or Python bridge decision is needed for
the input-interface replacement. Retain the established working package metadata
and Cargo joint offline archive verification; prior registry publication is not
required. Commit each completed logical replacement after its focused checks;
publication/deployment/push remain separate authorization decisions.

### Focused Evidence And Documentation

During implementation, use these bounded semantic witnesses; this table does not
request one new E2E per row or mechanically parameterize all families/options.

| Contract | Required evidence |
| --- | --- |
| Complete native export parity | Re-export all native root/public module paths without adapter shadowing. Migrate a native consumer using only the crate-prefix substitution; include generic kind, native Bounds/input/layer and Error/Result usage and a public stats path without rendering a new scenario per item. |
| Dependency and native identity | Adapter-only renderer dependencies suffice for native configuration, callbacks, output and error types. In the existing composition witness, use both adapters' family constructors, explicit adapter-to-native conversions, an unchanged native Plot and cross-path Theme/Canvas types from the one resolved core; no adapter Canvas, erasure or conversion for already-native items. Use bounded compile consumers during implementation, not a separate compatibility project or per-export echo tests. |
| Source and direct-variable grammar | Whole Polars frame/name binding, direct Series and Column without IntoColumn, mixed named x/external y, and ndarray index/view binding; missing fields/indices and wrong selected dtype identify the role. |
| Explicit-x/many-y meaning | An irregular x such as `[0, 5, 20]` with selected a/b never becomes row positions or treats time as a third series; selection order and raw column identity survive, with no invented Cartesian product. |
| Omitted-coordinate wide input | Deliberate row-position/default-column-category semantics; typed numeric discovery does not silently omit an explicitly selected unsupported variable. Row versus explicit cell alignment is operation-defined, including when lengths happen to coincide. |
| Independent layers and bounds | A 30-record supplied line/band and 200-record point layer bind independently; labels resolve in the point source, no parent hue/facet inheritance or global DataFrame-height rule. Sample bars versus supplied estimates remain distinct. |
| Grid orientation and membership | A non-square strided matrix maps source `(r,c)` to the correct row/column; true-present validity and true-highlight Booleans stay distinct, missing cells keep coordinates, and per-cell exact IDs are not repeated row IDs. Use semantic source/axis relationships, not placement coordinates or SVG goldens. |
| Lifecycle and callbacks | Independent plots can borrow a source, direct input temporaries expire after binding, a named-source borrow ends at standard conversion or output, and native output replays without reopening sources. Mapped wide-row alpha preserves its declaration and native callback timing when expanded; cell mapping remains independently aligned. |
| Capabilities and error boundaries | Representative unsupported native/PlotAdapter methods remain unavailable, native data-dependent restrictions remain preflight errors, root Error/Result retain native identity, and standard `Result<_, AdapterError>` can propagate binding and original native failures with `?` without stringification. |

These examples guide focused unit/module coverage and migration of existing
public workflows. Prototype the selected `new` type/lifetime shape within the
implementation using bounded compile consumers, not a separate compatibility
project or a new pre-implementation gate.
Existing stride/chunk/rounding/dictionary tests still matter for private conversion.
Do not replace them with repeated per-family rendering smoke tests or keep tests
of removed public conversion functions just to preserve a historical count.

Use the existing two executable adapter workflows as documentation/rendering
witnesses rather than multiplying scenarios by language or entry point. The last
recorded baseline is 800 unit, 299 module-integration and 80 conservative core-E2E
scenarios, including adapter/gallery workflows; reclassify/count the affected
changes honestly under [development](90_DEVELOPMENT.md), not a new allowance.
If a genuine ceiling conflict occurs, identify the redundant witness or report
that concrete conflict; do not promise arbitrary deletions in advance.

Run focused changed tests and strict workspace Clippy for code changes:

```sh
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
```

Use required implementation-closure checks when due, not repeated whole-suite
discovery. Verify actual package archives, executable examples and generated guide
links. Implemented workflows require actual execution and strict Clippy; future
snippets remain design examples rather than passing compile evidence.

Keep documentation prose single-language and guide content in its language
directories. README entry documents remain English, with no language-specific
variants. Existing guide editions have equal status and equivalent scope, without
assuming all future languages form a binary pair. Preserve gallery-first,
single-page guides, direct same-page image-to-code links, shared executable-source
includes and zero-argument examples with optional output-directory redirection.
No language-tooling rewrite is part of adapter replanning.

## References

[seaborn-data]: https://seaborn.pydata.org/tutorial/data_structure.html
[seaborn-scatter]: https://seaborn.pydata.org/generated/seaborn.scatterplot.html
[seaborn-plot-data]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/_core/data.py
[seaborn-wide]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/_base.py
[seaborn-matrix]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/matrix.py
[seaborn-line]: https://seaborn.pydata.org/generated/seaborn.lineplot.html
[seaborn-layers]: https://seaborn.pydata.org/generated/seaborn.objects.Plot.add.html
[plotly-input]: https://plotly.com/python/px-arguments/
[plotly-scatter]: https://plotly.com/python-api-reference/generated/plotly.express.scatter.html
[plotly-wide]: https://plotly.com/python/wide-form/
[plotly-line]: https://plotly.com/python/line-charts/#Data-Order-in-Line-Charts
[ggplot-layers]: https://ggplot2.tidyverse.org/reference/geom_point.html
[ggplot-path]: https://ggplot2.tidyverse.org/reference/geom_path.html
[aog-input]: https://aog.makie.org/stable/tutorials/intro-v
[altair-data]: https://altair-viz.github.io/user_guide/data.html#converting-with-fold-transform
[altair-encoding]: https://altair-viz.github.io/user_guide/encodings/index.html#datum-and-value
[altair-layers]: https://altair-viz.github.io/user_guide/compound_charts.html
[matplotlib-image]: https://matplotlib.org/stable/api/_as_gen/matplotlib.pyplot.imshow.html
[plotly-rust]: https://docs.rs/plotly/0.14.1/src/plotly/traces/scatter.rs.html#305-381
[plotly-rust-constructors]: https://docs.rs/plotly/0.14.1/src/plotly/traces/scatter.rs.html#297-312
[clippy-new]: https://rust-lang.github.io/rust-clippy/master/index.html#new_ret_no_self
[rust-reexports]: https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility
[cargo-resolution]: https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility
[cargo-package]: https://doc.rust-lang.org/cargo/commands/cargo-package.html
