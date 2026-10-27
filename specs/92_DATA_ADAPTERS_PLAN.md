# Data-Source Plotting Interfaces For ndarray And Polars

## Status And Objective

This is a replacement design, not an implemented interface. The current
`lichtung-plot-ndarray` and `lichtung-plot-polars` packages export conversion
functions. Their public design has been rejected, despite successful conversion
tests and packaging. The current request authorizes replanning only; do not
change Rust code, dependencies, examples or guides to pretend this interface
already exists. Publication, deployment and Git push remain unauthorized.

The objective is to plot foreign data directly. A caller describes the dataset
and its mapping to plot roles, or supplies vectors/a matrix directly. Constructing
NumericValues, CategoryValues, BooleanValues, SourceRows or HeatmapValues must not
be a prerequisite for a foreign-data workflow. The complete native-value core API
also remains available unchanged through either adapter, not only to standalone
consumers or as an internal rendering boundary.

This plan selects top-level plot-family constructors, adapter-owned configured
plots and adapter-side data conversion. The [complete compatibility inventory](93_ADAPTER_API_COMPATIBILITY.md)
maps native types, chainable methods and composition boundaries. Rust snippets
throughout both plans illustrate the proposed interfaces; they have not been
compiled and are not claims that those interfaces exist. Implementation must
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
mod frontend;
pub use frontend::{DataPlot, Line, Scatter};
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

Foreign input is selected at the adapter root through the plot family. Each family
has two constructor names, referring to input workflows rather than two renderers:

| Native API, unchanged | Additional foreign-data API |
| --- | --- |
| `adapter::Plot::scatter(native_x, native_y)` | `adapter::Scatter::from_vectors(foreign_x, foreign_y)?` |
| Native Plot constructors and methods | `adapter::Scatter::from_data(source, x, y)?`, followed by source-bound setters |
| `adapter::Bounds` | `adapter::Interval::new(lower, upper)`, a pair of input bindings, not a computed confidence interval |
| HistogramInput, RaincloudInput, BarInput/BarSamples, CoefficientInput | HistogramData, RaincloudData, BarData/BarSampleData, CoefficientData |
| `adapter::PointLayer::new(native_x, native_y)` | `adapter::PointLayerData::from_data(source, x, y)?` or `::from_vectors(x, y)?` |
| `adapter::Error`, `adapter::Result<T>` | `adapter::DataError`, `adapter::DataResult<T>` |

`from_data` receives a dataset and family-specific role bindings. `from_vectors`
receives direct foreign vectors, or the matrix/explicit record description specified
for that family. Exact argument lists are in the 25-family inventory below; there
is no Rust overload by argument count. Both constructors return an adapter-owned
`DataPlot` carrying the real exact-family core Plot. `from_data` retains the
source context for later bindings; `from_vectors` snapshots its inputs and retains
only necessary shape/origin metadata. Prepared layer frontends are DataPointLayer
and DataBarPointLayer. Users need not name these output types in inferred chains.

Native constructors stay on Plot. Family names, DataPlot, input descriptors and
DataError are additive and must not shadow any native export. There is no public
`data(source)` factory, root family free-function catalog, `build()` method,
mandatory binding namespace or required extension-trait import for this workflow.
An explicit standard conversion can hand a bound adapter object to an unchanged
core consumer. Adapter-side binding, conversion and forwarding are the selected
baseline; potential core composition changes are separately identified for review.

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
foreign-input descriptors, DataPlot frontends and input errors are adapter-specific. Applications
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
| [Plotly mixed-form][plotly-wide] `px.line(frame, x="time", y=["a", "b"])` | A real x column and multiple y columns need no user reshape. | Whole-table numeric discovery can accidentally include time/ID columns; generated labels cannot invent missing scientific meaning. | Support explicit single-x/multiple-y selection. Explicit `RowIndex` or `ColumnIdentity` represents an omitted coordinate/category inside the same two family constructors. |
| [ggplot2 layers][ggplot-layers] `geom_line(data=fit, ...) + geom_point(data=observations, ...)`; [Seaborn Objects][seaborn-layers] `.add(mark, data=other, ...)` | Layers can specify their own datasets and mappings; reuse is natural. | Inherited mappings can refer to absent fields; ggplot2 exposes inherit.aes and Seaborn Objects can index-join against constructor data. | Resolve supplemental layers in their own context. Do not inherit parent hue/facets, join indexes or require parent-sized arrays. Do not add an arbitrary-layer grammar. |
| [AlgebraOfGraphics][aog-input] `mapping(:time, direct(corrected))`; `mapping(:time, [:a, :b], color=dims(1))` | Direct vectors, mixed inputs and selected wide columns remain plot inputs. | `direct` and dimension selectors add concepts; its direct-matrix route is documented as experimental. | Rust types distinguish vectors from selectors without a mandatory Direct wrapper. Adopt ordered multi-column selection, not the complete algebra/dimension system. |
| [Altair field/datum/value][altair-encoding], [fold][altair-data] and [layers][altair-layers] | Field mappings, constants, display encodings and chart-local transforms are explicit. | Its record-oriented grammar makes ordinary matrix/vector input more elaborate; copying it would create a second graphics specification. | Keep bindings separate from literal style and labels. No field-expression strings, JSON round trip or mandatory user fold/melt. |
| [Matplotlib matrices][matplotlib-image] and [plotly.rs ndarray][plotly-rust] | Direct arrays can be converted internally; matrix orientation can be stated explicitly. | Direct acceptance is not zero-copy. plotly.rs' ndarray constructors copy into vectors and are not a table-mapping API. | Accept views directly, define row/column roles, retain private native snapshots and make their cost explicit. |

### Frontend Architecture Decision

| Candidate | Benefit | Cost | Decision |
| --- | --- | --- | --- |
| One fallible function binds all inputs, then returns the core Plot immediately | Minimal wrapper; core style methods need no forwarding. | Later `.hue("condition")`, named bounds and facets cannot resolve without another context or a large up-front options object. | Not the selected ordinary interface. |
| Top-level family constructors plus exact-family DataPlot | Fits existing consuming chains; later bindings retain source and expansion context; preserves family-specific capabilities. | Both adapters maintain configuration forwarding. Source-bound plots retain a lookup borrow after selected values have been copied. Existing native consumers need a conversion. | Selected for `Family::from_data` and `Family::from_vectors`; root Plot remains the native re-export. |
| General Mark/Scale/Stat grammar | Uniform arbitrary layers and reusable mappings. | Requires new composition, inference and type-capability contracts beyond data adaptation. | Not part of this replacement. |

The selected interface is Seaborn/Plotly-like in input handling, not a claim of
identical scientific defaults or implementation. Reuse the renderer instead of
adding reverse dependencies, a generic core data-access framework or a new shared
framework crate.

## Public Input Grammar

In each package, `Family::from_data(source, ...)` binds the arguments it needs
and retains a read-only descriptor of the source in its DataPlot. Independent
plots may borrow the same source. `Family::from_vectors(...)` binds direct
inputs without manufacturing an implicit table for later field lookup. Exact
generic/lifetime spelling is an implementation detail; the resulting family
remains the core's actual kind marker. Foreign constructors and descriptors below
are additional root exports with their own names. Root Plot always keeps its
original native-value signatures.

| Position | Polars | ndarray |
| --- | --- | --- |
| Dataset | `&DataFrame`, eager only | `ArrayView2<T>` or its Valid descriptor, observations in rows |
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
Transient input/descriptor lifetimes must not unnecessarily become the completed
DataPlot's lifetime. A missing Polars source for a string, or ndarray source for
an index, is an unresolved-binding error, never a literal/broadcast fallback.

Role chooses interpretation: numeric, categorical, Boolean and exact provenance
remain distinct. A trajectory's numeric/category x can follow an unambiguous
admitted dtype; an all-Null ambiguous x requires an explicit `Axis::numeric(...)`
or `Axis::categorical(...)` descriptor. No automatic numeric-category formatting,
string-number parsing or continuous hue is introduced.

## Concrete User Workflows

### Polars: Dataset And Variable Bindings

```rust
use lichtung_plot_polars::{Facet, Scatter};

let plot = Scatter::from_data(&frame, "concentration", "response")?
    .hue("condition")?
    .source_rows("sample_id")?
    .facet(Facet::wrap("batch", 3))?
    .title("Response by condition");

let output = plot.svg("response")?;
```

The family constructor returns an exact-family DataPlot retaining the source
context for later bindings. A string is never an expression such as `log(x)` or
an Altair-style field/type shorthand. Missing columns identify the source name
and plot role. Literal titles, Color/Theme instructions and display labels are
not resolved as fields. `hue("red")` selects a field named red, not a paint.

Direct Column and Series inputs also work without constructing a DataFrame or
requiring `.into_column()` merely to plot:

```rust
let plot = Scatter::from_vectors(&concentration_series, &response_series)?
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

let scatter = nd::Scatter::from_vectors(x.view(), y.view())?
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
When explicit validity is needed, `Valid::new(values.view(), present.view())`
is a borrowed descriptor accepted wherever that shape/role is admitted, including
selected-table and grid input. Construction describes input; the binding call
checks exact rank/shape and snapshots it. True means present; Option absence and
false validity both mean null. This is not a NumPy true-means-hidden mask or a
materialized native-value object. Ordinary views need no wrapper.

### Explicit X And Multiple Y Variables

```rust
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pp;

let lines = pp::Line::from_data(&frame, "time", ["sensor_a", "sensor_b"])?
    .facet(pp::Facet::wrap("batch", 3))?;

let selected = nd::Line::from_data(table.view(), 0, [1, 2])?;

let direct = nd::Line::from_vectors(time.view(), signals.view())?;
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
`from_data` has the dataset in addition to those roles; `from_vectors` does not.
No separate wide constructor is introduced.
Ordinary binding errors remain fallible; there is no general deferred transformation graph.
Keep the listed plain-name/index collection syntax working in the real public
consumers; do not replace it with a mandatory selector DSL during implementation.

| Multi-variable operation | Native meaning |
| --- | --- |
| `line(x, ys)` | Repeat x for each y variable; column identity supplies the default native group and hue. The core sorts each numeric series; no new estimator is installed. |
| `scatter(x, ys)` | Repeat x; column identity supplies default hue, not a fictitious connectivity group. |
| `boxplot(category, ys)`, `violin(category, ys)` | Repeat the category variable; column identity supplies the positioned series. Native sample summaries remain native. |
| `raincloud(RaincloudData::samples(category, ys))` | The same category/positioned-series expansion, using the native sample route rather than supplied geometry. |

Other constructor positions remain single-variable unless explicitly listed.
No arbitrary wide interpretation is added to interval estimates, trajectories,
stacked totals or supplied distribution geometry.

### Wide Input Without An Explicit Coordinate

```rust
let lines = pp::Line::from_data(&frame, pp::RowIndex, pp::Columns::numeric())?;
let chosen = pp::Line::from_data(&frame, pp::RowIndex, ["sensor_a", "sensor_b"])?;
let indexed = nd::Line::from_vectors(nd::RowIndex, signals.view())?;
let distributions = pp::Boxplot::from_data(
    &frame, pp::ColumnIdentity, pp::Columns::numeric(),
)?;
```

`RowIndex` explicitly chooses row position as x for Line and Scatter; callers
pass the actual x selector/vector when a real coordinate exists. `ColumnIdentity`
in the primary-category position of Boxplot, Violin and sample-based Raincloud
uses each selected column as a category and its cells as observations. These are
typed role choices within `from_data`/`from_vectors`, not additional family
constructors. `Columns::numeric()` explicitly requests supported numeric columns
of a Polars source in schema order. `Columns::all()` selects every logical column:
on a Polars source an explicitly selected nonnumeric column is an error; on an
ndarray source the element type is homogeneous. Direct ndarray matrix views supply
all columns.

Polars `Columns::numeric()` chooses supported numeric dtypes in schema order,
excluding unsupported semantic dtypes and untyped Null columns.
Explicit numeric selections can resolve Null through the requested numeric role.
Document automatic selection; do not claim the adapter can recognize a numeric
time/ID column as metadata. No usable selected variables is an error. Explicit
selections for either adapter preserve the caller's order.

After expansion, ordinary row-scoped hue/group/series/facet bindings repeat by
source row. Their default unit is fixed by the operation, not guessed from length.
`Cells(input)` explicitly binds an already-expanded foreign vector, including
cell-level facets/alpha or unique IDs. Generated series defaults are installed
through actual native roles; a later group/hue/series setter replaces that role
as it does in the core. Do not secretly restore a replaced grouping or infer a
new cross-product. Native applicability still governs the resulting plot.

`ColumnIdentity` is a plot-role selector for the generated wide-column dimension,
not a magic source field name. It can be used in a facet or alpha key, for example
`Facet::wrap(ColumnIdentity, 2)`. A literal source column named variable/series
continues to mean that source column. Generate collision-free private native
identities and retain their mapping; never expose a reserved-name parser. For
unnamed ndarray columns use stable positional display defaults; optional
`Columns::selected(indices).labels(labels)` supplies one label per selected column
without changing index selectors. Label counts and aliases retain native
collision/replacement rules.

### Matrix Heatmaps And Record Cells

```rust
use lichtung_plot_ndarray::{Grid, Heatmap, HeatmapCells};

let plain = Heatmap::from_vectors(signals.view())?;
let labeled = Heatmap::from_vectors(
    Grid::new(signals.view())
        .x_labels(["A", "B", "C"])
        .y_labels(["Sample 1", "Sample 2"]),
)?
.heatmap_highlight(highlights.view())?;

let records = Heatmap::from_vectors(HeatmapCells::new(
    columns.view(), rows.view(), values.view(),
))?;
```

Grid is an optional borrowed plot-input descriptor for axis metadata, not a
flattened cell table. Raw views work without it. Axis-label counts must match
their dimensions. Internally generated axis keys stay separate from labels;
labels neither merge positions nor become application provenance. Duplicate
display-label collisions retain the core's normal validation; do not promise
label behavior the native Labels contract cannot express.
Grid labels are installed through native Labels aliases at construction. A later
`.labels(...)` call replaces that complete policy, including those aliases, just
as it does for the core; do not keep an invisible overriding alias cache. Source
positions and their private identity map do not change when label policy changes.

Polars supports a whole numeric rectangle, selected grid columns, and explicit
cell records without forcing one interpretation on all three:

```rust
use lichtung_plot_polars::{Grid, Heatmap, HeatmapCells};

let rectangle = Heatmap::from_data(&numeric_frame, Grid::all())?;
let selected = Heatmap::from_data(&frame,
    Grid::columns(["sensor_a", "sensor_b"]).row_labels("sample"),
)?;
let records = Heatmap::from_data(&cell_frame,
    HeatmapCells::new("column", "row", "value"),
)?;
```

`Heatmap::from_data` has exactly two arguments: the source and one grid-or-cell
descriptor. `Grid::all()` selects the full rectangle; `Grid::columns(...)`
selects ordered Polars names or ndarray indices. `HeatmapCells::new(x, y, fill)`
marks record-oriented roles and also accepts direct foreign vectors inside the
single `from_vectors` argument. `Cells(...)` is a separate aligned-input descriptor,
not the record-heatmap constructor. A plain ndarray matrix view is accepted as a
direct grid without a wrapper. The whole-frame heatmap requires a supported numeric
rectangle (Null is resolved as numeric fill); it
does not silently discard metadata columns. Record cells preserve native
categorical coordinate and duplicate-cell rules; `px.density_heatmap`-style
binning/aggregation is not what this matrix operation means.

| Input operation | Defined meaning |
| --- | --- |
| Rectangular heatmap | Cell `(r,c)` is fill, x is column `c`, y is row `r`; row zero is displayed at the top and columns left-to-right, consistent with the current native heatmap. Explicit native ordering may reorder presentation, not source identity. |
| Grid highlights | A same-shape Boolean ndarray view, Boolean Polars rectangle/ordered column selection, or `Cells(vector)` aligns with cells. True highlights a cell; it does not hide it or define its validity. |
| Grid source IDs | Exact same-shape integer values/selected integer columns, or `Cells(vector)`, provide one unique ID per expanded cell. A row-length ID field is not repeated across columns. |
| Record heatmap | One x/y/fill record per slot; highlights and IDs align with those records. |
| Scatter matrix | `Matrix::from_vectors(source2)` or `Matrix::from_data(source, columns)` uses columns as variables and rows as observations, with optional positional labels for unnamed arrays. This is never heatmap inference. |

Expansion order is logical row-major: `(r,c)` occupies slot `r*ncols+c` using
checked arithmetic. Retain original row and selected source-column identity
privately for diagnostics and aligned expansion. No transposition to match storage
layout, implicit compaction, imputation or visible string-key construction is
required from callers. Null grid cells retain coordinates; they are not removed.

### Supplied Bounds And Independent Supplemental Layers

```rust
use lichtung_plot_polars::{Color, Interval, Line, PointLayerData};

let points = PointLayerData::from_data(&observations, "time", "response")?
    .labels("sample")?
    .color(Color::rgb(90, 90, 90))
    .legend_label("Observed");

let fitted = Line::from_data(&fit, "time", "estimate")?
    .band(Interval::new("lower", "upper"))?
    .point_layer(points)
    .title("Supplied fit and observations");
```

Here fit may have 30 rows and observations 200. The interval binds to fit's estimate
records; the point layer's labels bind to its own 200 points. A layer never looks
up missing fields in the parent source, inherits parent hue/group/facets, or gains
an independent SourceRows map. There is no index join between the datasets.

`PointLayerData::from_data(source, x, y)` and `::from_vectors(x, y)` snapshot
coordinates fallibly and return a DataPointLayer. The data form retains only its
own lookup context for later labels. Paint/radius/alpha/text setters preserve
native semantics. `point_layer(layer)` on a Line DataPlot accepts a native
PointLayer or a fully bound DataPointLayer through an adapter-local generic Into
bound and returns Self, not Result. `From<DataPointLayer> for PointLayer` drops
the local lookup context; it does not change native Plot::point_layer's signature.

`BarPointLayerData::from_data(source, category, value, color, radius)` and
`::from_vectors(category, value, color, radius)` return a DataBarPointLayer;
an optional `.series(binding)?` resolves only in that layer context. The core still resolves
its categories/series against prepared parent bar centers, not parent row count.
Only Bar/HorizontalBar accept it. Ordinary PointLayer remains Line-only and
rejects generic faceting; no per-point hue/size mapping or arbitrary Mark layering
is added. Reuse the native types for style, endpoints and supplied geometry.

## Architecture: Adapt The Interface, Reuse The Renderer

```text
DataFrame + column bindings / foreign vectors / matrices
        -> adapter source and role resolution
        -> private native snapshots and native input constructors
        -> existing Plot<K>, scientific preparation, Canvas and Output
```

- Keep the two existing adapter packages and dependency direction. Each fully
  re-exports the native public API and adds top-level family constructors and
  typed DataPlot frontends, not just value converters. Both adapter roots and the
  standalone core expose the same unchanged native Plot definition.
- The adapter owns argument interpretation, source lookup, shape/dtype diagnostics
  and binding metadata. Reuse current private conversion code where sound; never
  require users to call it before plotting or rebuild a whole dataframe in a
  project-specific public format.
- Native storage/normalization is an internal implementation choice, admitted
  here. Do not add a generic core data-access trait, generic core source parameter,
  reverse optional dependency or external-buffer rendering path without a concrete
  unmet consumer. No such core refactor is needed to begin this replacement.
- DataPlot wraps the actual core Plot with its existing exact family marker.
  Forward appearance, numerical callbacks, text, guides and terminal methods;
  delegate scientific validation, layout, font policy, capture and publication.
  Do not copy those algorithms or maintain a second scientific configuration model.
- Preserve `kind` capability bounds. A wrapper must not expose unsupported setters
  by erasing every family. Do not rely on Deref for consuming setters: that would
  lose the binding context or return a different builder halfway through a chain.
  Small package-local forwarding macros are acceptable; a new shared framework
  crate or public universal schema is not the objective.
- Implement `From<DataPlot<..., K>> for crate::Plot<K>` in the originating
  adapter, preserving the exact family and moving the already-bound core value.
  The conversion drops the source context and performs no scientific
  preflight/rendering. Independently bound DataPointLayer and DataBarPointLayer
  have analogous conversions to their native layer types. All binding failures
  precede these infallible moves; native scientific validation retains its timing.
  Do not add `build`, `into_native`, `as_core` or a parallel adapter Canvas.

On DataPlot, forward core-only methods with their existing names, capability
bounds, consuming shape, callbacks and return behavior. Data-bearing methods
resolve the foreign inputs listed here; their fallible signatures do not replace
the root native methods. Keep one package-local forwarding inventory and use it
for review of added/changed core configuration; macros do not remove maintenance
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
    nd::Heatmap::from_vectors(matrix.view())?.into();

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
kind, implemented by native Plot and adapter DataPlot, used by `panel` and
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

Binding operations return DataResult values. Resolve and snapshot the
variables a constructor/data setter actually binds at that call;
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
allow a temporary direct vector/descriptor to expire after its binding call while
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
matching and return types. DataError owns structural/lookup/dtype context and
original upstream/native error sources; `DataResult<T>` uses that error. It must
provide lossless `From<crate::Error>` independently of context-specific binding
variants, so a foreign-data workflow can use `?` for both binding and rendering:

```rust
use lichtung_plot_polars::{DataResult, Output, Scatter};

fn render(frame: &polars_core::prelude::DataFrame) -> DataResult<Output> {
    Ok(Scatter::from_data(frame, "time", "response")?
        .svg("response")?)
}
```

DataPlot terminal methods keep native Result<Output> behavior, as do the native
Canvas and Output. Native-only functions keep returning root `Result<T>` with no
new adapter wrapping. A function combining two adapters handles their distinct
DataError types in its own application error enum or a standard boxed error; this
does not require importing or directly depending on the core. Do not stringify
binding errors into native Error::Data, widen the core error for foreign-library
concerns, or pretend native Result can propagate a binding error without an
explicit application-side conversion. Moving the current adapter error types
to the non-conflicting DataError export and adding native From support is part
of replacement, not behavior already provided by the conversion-only packages.

## Coverage Across The Existing API

All 25 families need source-bound and direct input. Each row below defines the
fixed arguments of `Family::from_data(source, ...)` and `Family::from_vectors(...)`.
In `from_data`, role bindings are literal Polars names, ndarray indices or direct
vectors. In `from_vectors`, data roles are direct vectors/views and no implicit
source-name lookup is available. Either constructor also accepts an already-owned
corresponding native value in a documented data role, including mixtures of native
and foreign roles; native values keep native slot semantics. Literal reference
values, native policies and explicit semantic descriptors stay literal. Both entry points return an
exact-family DataPlot in a DataResult. Shapes admitted in a role are specified by
the input grammar above, not by Rust function overloading.

| Family | `from_data` arguments after source | `from_vectors` arguments | Meaning retained |
| --- | --- | --- | --- |
| Scatter | `x, y_or_ys` | `x, y_or_ys` | Single coordinates or defined many-y expansion. |
| Histogram | `value_or_samples` | `value_or_input` | A bare value is samples; `HistogramData::samples(value)` is explicit; `HistogramData::geometry(native_geometry)` is a distinct direct-input mode. |
| Density | `value` | `value` | Native sample density. |
| Qq | `value` | `value` | Native sample/reference policies. |
| PointInterval | `category, estimate, interval` | `category, estimate, interval` | `Interval::new(lower, upper)` supplies required bounds; explicit status remains separate. |
| Line | `x, y_or_ys` | `x, y_or_ys` | Sorted numeric series, optional supplied band and PointLayer. |
| Trajectory | `x, y, group` | `x, y, group` | Constructor-owned group and numeric/categorical x. |
| LinePath | `x, y, group` | `x, y, group` | Numeric x and original source-order paths. |
| Interaction | `condition, estimate, group, interval` | `condition, estimate, group, interval` | Constructor-owned group and supplied `Interval` bounds. |
| Coefficient | `term, coefficient_data` | `term, coefficient_data` | `CoefficientData::estimates(value)` or `::estimates_with_intervals(value, interval)` selects point-only versus bounds. |
| Forest | `term, estimate, interval, reference` | `term, estimate, interval, reference` | Required bounds and literal scientific reference. |
| Dumbbell | `category, left, right, endpoints` | `category, left, right, endpoints` | Foreign paired values; native `DumbbellEndpoints` policy. |
| Heatmap | `grid_or_cells` | `grid_or_cells` | `Grid::all()`/`Grid::columns(...)` or `HeatmapCells::new(x,y,fill)`; a direct matrix view is a grid. One descriptor argument distinguishes records from rectangles. |
| Bar | `category, bar_data` | `category, bar_data` | `BarData::samples`, `::estimates`, or `::estimates_with_error_bars`; native mean/error-bar behavior. |
| HorizontalBar | `category, bar_data` | `category, bar_data` | Same closed sources and native sign/reference capabilities. |
| StackedBar | `category, value, series` | `category, value, series` | Required positioned-series values and native sum semantics. |
| StackedHorizontalBar | `category, value, series` | `category, value, series` | Same stack semantics and exact family. |
| Boxplot | `category, value_or_values` | `category, value_or_values` | Samples and positioned-series expansion; `ColumnIdentity` requests omitted-category wide input. |
| Violin | `category, value_or_values` | `category, value_or_values` | Native density/sample policies and multi-value series. |
| Raincloud | `raincloud_data` | `raincloud_data` | `RaincloudData::samples(category, values)`; `::geometry(native_geometry)` is a distinct direct-input mode. |
| Regression | `x, y` | `x, y` | Existing core OLS/band policy, not a new adapter estimator. |
| Marginal | `x, y` | `x, y` | Native composed views and applicability. |
| Matrix | `columns` | `variables` | `columns` is an ordered selection or `Columns::numeric()`/`Columns::all()`; `variables` is a 2D view or ordered Polars columns. No generic facets. |
| Beeswarm | `category, value, color` | `category, value, color` | Constructor-owned numeric color channel. |
| Waterfall | `category, delta, base` | `category, delta, base` | Literal base and original ordered delta semantics. |

`interval` denotes `Interval::new(lower, upper)` or an already prepared native
`Bounds`; the two bound roles may be source selectors or direct vectors. The
`*_data` arguments are the named adapter descriptors defined below or their
corresponding complete native inputs. `RowIndex` and `ColumnIdentity` are explicit
markers only in the roles and families described above. Grid `from_vectors` also
accepts ordered Polars numeric columns; `Grid::new(...)` supplies optional labels.
The full [method and type compatibility inventory](93_ADAPTER_API_COMPATIBILITY.md)
specifies native values, capability bounds, conversion and terminal behavior.

HistogramData, RaincloudData, BarData and CoefficientData mirror the core's closed
scientific choices while holding foreign bindings. `BarData::samples` returns
BarSampleData and retains `.error_bar(native_policy)` before attachment; native
BarSamples remains untouched. Supplied geometry types, references, themes and
policies are reused from the core. These are plot-semantic descriptors, not a
second public dataset representation. No implicit general Into<core input> escape
hatch is required to use foreign samples.

Adapt every data-bearing position: hue/group/series, provenance, complete facet
specifications, bounds, status bindings, highlights, point/bar-point layers,
labels and data-alpha mappings as admitted by the underlying family. Small
adapter-owned Interval/Facet/layer/data specifications can hold names or foreign
vectors; their names do not shadow native types, and they express plot semantics,
not materialized replacement dataframes. Core-only style/policy/geometry values
are reused from root re-exports directly
rather than renamed or duplicated. Do not require a native-value escape hatch to
complete an ordinary adapter plot.

Alignment follows each owning semantic input, not one blanket DataFrame height.
Bounds and status align with their estimate records. Supplemental layers may use
their own source/direct vectors and record counts, and keyed alpha may use a
separate selector table. These inputs use explicit local contexts without changing
the parent plot's lookup source or joining datasets implicitly.

Alpha must still bind to its actual native unit (observations, records, series,
aggregates, bins or Single). Do not turn every source column into observation alpha
or reduce arbitrary rows to a collective value. Likewise, do not infer estimate
availability from nulls when the native contract requires an explicit status.

### Facets, Status And Alpha: Concrete Binding Surface

| Input | Selected surface | Resolution and alignment |
| --- | --- | --- |
| Supplied bounds | `Interval::new(lower, upper)`; `.band(interval)?` on Line | Resolve together in the owning constructor/line context. Many-y bounds require matching selected-column collections or same-shape matrices; one pair is not silently reused for every series. |
| Facets | `Facet::row(v)`, `column(v)`, `grid(row, column)`, `wrap(v, columns)`, then `.scales(native_policy)` | Deferred small descriptor consumed by `.facet(spec)?`; row/grid/wrap exclusivity and native family restrictions remain. |
| Provenance | `.source_rows(binding)?` | Exact native IDs. Record inputs align with records; wide/grid inputs require Cells or same-shape/selected-column IDs, never an inferred ID product. |
| Status | `.estimate_status(Status::boolean(binding))?`; `Status::categorical(binding, explicit_map)`; direct native EstimateStatus sequence or ndarray status view | Boolean true maps to Estimated, false to NotEstimable only because this descriptor explicitly requests that mapping. Null/unknown keys fail; missing numeric tuples never infer status. |
| Heatmap highlights | `.heatmap_highlight(binding)?` | Record Boolean vector or the documented cell grid; no implicit numeric 0/1. |
| Observation alpha | `.point_alpha(Alpha::observations(map))?` and native-admitted target equivalents | Resolve numeric/category/byte sources and preserve actual original-slot membership. Wide row maps repeat by origin; Cells requests explicit expanded values. |
| Single alpha | `Alpha::single(map)`; `Alpha::theme()` | Single resolves exactly one explicit source value; it is not inferred by observing a constant column. Theme resets only that target. |
| Keyed alpha | `Alpha::keyed(keys, map)` in the plot source, or `Alpha::keyed_from_data(source, keys, map)?` in an independent selector table | Exact actual series/aggregate keys, optional facet keys, unique tuples; no observation reduction or implicit join. |
| Bin alpha | `Alpha::bins(bounds, facet_keys, map)` or `Alpha::bins_from_data(source, bounds, facet_keys, map)?` | Exact native histogram intervals and optional owning-facet keys; no sample-to-bin inference or approximate edge matching. |

AlphaMap holds one of `bytes`, `linear`, `log10`, `continuous`, `categorical` or
`threshold`, with the same source/domain/range/callback meaning as native
AlphaValues. Its source is a foreign binding, not a caller-built NumericValues.
Byte inputs remain exact non-null u8 values, not ordinary f64 rounding or floating
opacity. Mapping callbacks run at the binding call under native materialization
rules; no callback or borrowed selector table survives in the stored encoding.

Keys can use a source field directly when names coincide, or
`Key::new(plot_field, source_binding)` when an independent table uses a different
name. ColumnIdentity selects the private generated column field without naming
it as a source column. Key only describes native alpha selectors; it is not a
general dataframe schema. Independent `Alpha::*_from_data` calls eagerly resolve
their own source fields and callbacks, then return an owned adapter alpha input.
Later setters resolve any symbolic plot-field targets against parent role metadata
without retaining or reopening the independent selector table. Only a fully
resolved alpha input has an infallible conversion to native AlphaEncoding; a
symbolic input needs the owning Plot context at attachment.

```rust
use lichtung_plot_polars::{Alpha, AlphaMap, EstimateStatus, Interval, Line, PointInterval, Scatter, Status};

let estimates = PointInterval::from_data(
    &summary, "term", "estimate", Interval::new("lower", "upper"),
)?
    .estimate_status(Status::categorical("status", [
        ("ok", EstimateStatus::Estimated),
        ("unavailable", EstimateStatus::NotEstimable),
    ]))?;

let alpha = Alpha::keyed_from_data(
    &series_alpha, ["condition"], AlphaMap::bytes("stroke_alpha"),
)?;
let lines = Line::from_data(&frame, "time", "response")?
    .hue("condition")?
    .line_alpha(alpha)?;

let points = Scatter::from_data(&frame, "time", "response")?
    .point_alpha(Alpha::observations(
        AlphaMap::linear("quality", (0.0, 1.0), (64, 255)),
    ))?;
```

No broad new analysis or arbitrary channel mapping follows from these helpers.
For example, Scatter still has no scientific line_alpha method, Regression's
band setter still accepts native RegressionBand rather than a foreign Interval,
Matrix/Marginal still lack generic facets, and supplied geometry retains runtime
restrictions. Keep the core's compile-time versus data-dependent distinction.

## Scientific Meaning Is Not Copied From A Similar Signature

| Situation | Upstream behavior | Selected adapter behavior |
| --- | --- | --- |
| Repeated x in a line call | [Seaborn lineplot][seaborn-line] defaults to mean aggregation/CI; [Plotly lines][plotly-line] connect in supplied order; [ggplot2 line/path][ggplot-path] distinguish x order and source order. | Bind into the selected native Line, Trajectory or LinePath. Do not silently aggregate, add a CI or change ordering because input came from a dataframe. |
| Raw ordinary-bar samples | A plotting library may choose an estimator by default. | Explicit BarData::samples retains the existing core BarInput::samples mean and default error-bar policy; this is not a blanket promise that adapters never trigger native statistics. Estimates/bounds use the separate closed mode. |
| Numeric hue | Seaborn/Plotly can map numeric color continuously. | Normal hue remains the core's categorical capability. Heatmap/Beeswarm retain their explicit continuous channels; a shared spelling does not imply equivalent channel coverage. |
| Constant style versus field | Altair separates field, datum and value; ggplot2 separates aes mappings from fixed geom parameters. | Literal text/Color/Theme/native policies never become source lookups. Field strings never fall back to constants on lookup failure. |
| Missingness/masking | Seaborn heatmap mask=true hides a cell. | Valid=true means present; highlight=true means bordered. Native missing-cell and nonfinite rules stay distinct and applicable to their family. |

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
- Empty vectors/matrices retain their structural meaning; the native consumer
  decides whether a useful plot exists. No implicit row removal or imputation is
  added merely to match Seaborn output.

## Replacement And Delivery

1. **Replace Polars' public entry point first.** Re-export the entire native API
   unchanged and implement top-level family whole-frame/name and direct Series/Column paths,
   including explicit-x/many-y and local supplemental layers. Extend that same
   DataPlot across the constructor/setter inventory, with real grouped/faceted
   consumers rather than a separate facade for each example. Reuse private dtype
   converters, keep input/context lifetimes separate, and provide DataError with
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

These are implementation steps after separate direction, not preflight projects.
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
| Omitted-coordinate wide input | Deliberate row-position/default-column-category semantics; typed numeric discovery does not silently omit an explicitly selected unsupported variable. Row versus Cells alignment is operation-defined, including when lengths happen to coincide. |
| Independent layers and bounds | A 30-record supplied line/band and 200-record point layer bind independently; labels resolve in the point source, no parent hue/facet inheritance or global DataFrame-height rule. Sample bars versus supplied estimates remain distinct. |
| Grid orientation and membership | A non-square strided matrix maps source `(r,c)` to the correct row/column; true-present validity and true-highlight Booleans stay distinct, missing cells keep coordinates, and per-cell exact IDs are not repeated row IDs. Use semantic source/axis relationships, not placement coordinates or SVG goldens. |
| Lifecycle and callbacks | Independent plots can borrow a source, direct input temporaries expire after binding, a named-source borrow ends at standard conversion or output, native output replays without reopening sources, and alpha/numeric callbacks retain their native timing. |
| Capabilities and error boundaries | Representative unsupported native/DataPlot methods remain unavailable, native data-dependent restrictions remain preflight errors, root Error/Result retain native identity, and DataResult can propagate binding and original native failures with `?` without stringification. |

These examples guide focused unit/module coverage and migration of existing
public workflows, not separate compatibility prototypes before implementation.
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
links. This replanning-only change needs source/reference consistency and diff
checks, not compilation, benchmarks or execution of the proposed snippets.

Keep documentation prose single-language and guide content in its language
directories. README entry documents remain English, with no language-specific
variants. Existing guide editions have equal status and equivalent scope, without
assuming all future languages form a binary pair. Preserve gallery-first,
single-page guides, direct same-page image-to-code links, shared executable-source
includes and zero-argument examples with optional output-directory redirection.
No language-tooling rewrite is part of adapter replanning.

## References

[seaborn-data]: https://seaborn.pydata.org/tutorial/data_structure.html
[seaborn-plot-data]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/_core/data.py
[seaborn-wide]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/_base.py
[seaborn-matrix]: https://github.com/mwaskom/seaborn/blob/v0.13.2/seaborn/matrix.py
[seaborn-line]: https://seaborn.pydata.org/generated/seaborn.lineplot.html
[seaborn-layers]: https://seaborn.pydata.org/generated/seaborn.objects.Plot.add.html
[plotly-input]: https://plotly.com/python/px-arguments/
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
[rust-reexports]: https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility
[cargo-resolution]: https://doc.rust-lang.org/cargo/reference/resolver.html#semver-compatibility
[cargo-package]: https://doc.rust-lang.org/cargo/commands/cargo-package.html
