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
be a prerequisite for an adapter workflow. The existing native-value core API
remains useful to standalone consumers and as the internal rendering boundary.

Names below are concrete proposals to refine through real implementation, not
compiled examples or frozen compatibility commitments.

## What Seaborn Establishes

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

## Proposed User Workflows

### Polars: Dataset And Variable Bindings

```rust
use lichtung_plot_polars::{Facet, Plot};

let plot = Plot::data(&frame)
    .scatter("concentration", "response")?
    .hue("condition")?
    .source_rows("sample_id")?
    .facet(Facet::wrap("batch", 3))?
    .title("Response by condition");

let output = plot.svg("response")?;
```

`Plot::data(&DataFrame)` creates a source-binding context, not a converted table.
The family constructor returns an exact-family adapter Plot retaining that
context for later bindings. A plain string means a literal column name in that
context, never a parsing expression, constant string column or failed-lookup
fallback. Missing columns identify the source name and plot role in the error.

Direct Column and Series inputs also work without constructing a DataFrame or
requiring `.into_column()` merely to plot:

```rust
let plot = Plot::scatter(&concentration_series, &response_series)?
    .hue(&condition_series)?;
```

Data-bound plots can mix a named source variable with a direct vector. Record-form
bindings align by logical position and must satisfy their owning input's record
count; no implicit joins, broadcasting or source-index alignment occurs. A string
without a source context is an unresolved binding error, not a vector of
characters. Source names are inherited; explicit plot labels remain optional
overrides.

### ndarray: Direct Vectors And Table Columns

```rust
use lichtung_plot_ndarray::Plot;

let scatter = Plot::scatter(x.view(), y.view())?
    .hue(condition.view())?
    .x_label("Concentration (µM)")
    .y_label("Response");

let selected = Plot::data(measurements.view())
    .scatter(0, 2)?
    .hue(condition.view())?;
```

One-dimensional views are accepted directly, including supported nullable
elements. In a two-dimensional table context, integer selectors name logical
columns; rows are observations. Selection does not transpose, flatten or change
the source. Names are optional metadata/presentation, not mandatory conversion
arguments. Positional defaults must not invent scientific units or variable
meaning. Standalone vectors and selected columns use the same binding rules.

Logical strides, transposes, reversed axes, Fortran storage and already-broadcast
views remain supported. No contiguous intermediate is required from the caller.
An explicit validity argument, when needed, accompanies the borrowed input as a
small input descriptor; it does not produce a public native-value object. Keep
true-means-present validity and exact-shape checks distinct from NumPy mask
semantics. Ordinary views need no descriptor or wrapper ceremony.

### Matrices And Wide Data

```rust
let heatmap = lichtung_plot_ndarray::Plot::heatmap(signals.view())?;
let lines = lichtung_plot_ndarray::Plot::line_wide(series.view())?;
let distributions = lichtung_plot_polars::Plot::boxplot_wide(&frame)?;
```

Rust cannot overload one function by argument count. Explicit `_wide` and
cell/grid entry points distinguish genuinely different data interpretations;
they are not versions or compatibility aliases. A table-plus-mapping constructor
and a standalone direct-matrix constructor can live on different context types.

| Input operation | Defined meaning |
| --- | --- |
| ndarray `heatmap(view2)` | Cell `(r,c)` is fill, x is column position `c`, y is row position `r`. Labels/order may be supplied optionally. Never require user flattening or mandatory raw string keys. |
| Polars rectangular heatmap | Selected numeric columns form the grid in source/selection order; source column names label x. Rows retain positional identities; an optional row-label variable supplies display labels. The whole-frame form requires a numeric rectangle; an explicit grid selection can exclude metadata columns. |
| Table-bound heatmap with x/y/value variables | Record-form cells use those explicit bindings; duplicate-cell and native applicability rules remain intact. This is distinct from interpreting a rectangle. |
| Wide line/scatter | Each numeric column is a series, row position is x, cell value is y and column identity is grouping/color identity. Native collective-mark behavior applies. |
| Wide boxplot/violin/raincloud | Column identity is the categorical variable; cells are its observations. Only the native family computes its existing display statistics. |
| Scatter matrix | Columns are variables and rows are observations, passed to the core's native scatter-matrix constructor. This is not a heatmap. |

The Polars whole-frame wide form chooses supported numeric columns in schema
order, explicitly documented as part of that operation. It must not silently
discard a column the caller explicitly selected. Explicit selection preserves
caller order and errors on missing/unsupported columns. A matrix-wide operation
with no usable variables fails clearly rather than returning an unrelated plot.
Do not assign an undefined wide meaning to every family: use explicit variable
bindings when a family's roles cannot be inferred unambiguously.

Expansion order is logical row-major: `(r,c)` occupies slot `r*ncols+c`, with
checked shape arithmetic. Track source row/column origins privately for aligned
bindings and diagnostics. A row-scoped facet variable repeats by source row;
column identity supplies the default wide-series/category role. Other wide
bindings must state whether they address source rows, columns or expanded cells;
do not guess an axis from equal lengths. A grid highlight accepts a same-shape
Boolean matrix directly, such as `.heatmap_highlight(mask.view())?`, and uses the
same logical cell order. Record-form highlights remain one value per record.

## Architecture: Adapt The Interface, Reuse The Renderer

```text
DataFrame + column bindings / foreign vectors / matrices
        -> adapter source and role resolution
        -> private native snapshots and native input constructors
        -> existing Plot<K>, scientific preparation, Canvas and Output
```

- Keep the two existing adapter packages and dependency direction. Each exports
  plotting entry points and a small typed Plot frontend, not just value converters.
  `lichtung_plot_polars::Plot` and `lichtung_plot_ndarray::Plot` are the proposed
  foreign-input imports; the standalone `lichtung_plot::Plot` remains unchanged.
- The adapter owns argument interpretation, source lookup, shape/dtype diagnostics
  and binding metadata. Reuse current private conversion code where sound; never
  require users to call it before plotting or rebuild a whole dataframe in a
  project-specific public format.
- Native storage/normalization is an internal implementation choice, admitted
  here. Do not add a generic core data-access trait, generic core source parameter,
  reverse optional dependency or external-buffer rendering path without a concrete
  unmet consumer. No such core refactor is needed to begin this replacement.
- The frontend wraps the actual core Plot with its existing exact family marker.
  Forward appearance, numerical callbacks, text, guides and terminal methods;
  delegate scientific validation, layout, font policy, capture and publication.
  Do not copy those algorithms or maintain a second scientific configuration model.
- Preserve `kind` capability bounds. A wrapper must not expose unsupported setters
  by erasing every family. Do not rely on Deref for consuming setters: that would
  lose the binding context or return a different builder halfway through a chain.
  Small package-local forwarding macros are acceptable; a new shared framework
  crate or public universal schema is not the objective.
- A `build()` handoff may consume the frontend and return its real core Plot<K>
  for mixed native/adapter Canvas composition. It is a plot-builder completion,
  not manual data conversion. Normal adapter SVG/PNG output needs no handoff.
  Start with this existing-core-compatible path, rather than assuming Canvas or
  every core constructor needs an interoperability refactor.

### Ownership And Failure Timing

Prefer fallible binding operations returning adapter Result<Plot<...>>. Resolve
and snapshot the variables a constructor/data setter actually binds at that call;
do not materialize every unused source column, render eagerly or accumulate a
hidden first-error state to avoid Rust's `?` syntax. Ordinary non-data setters
keep the underlying core method's shape and behavior.

A data-bound frontend retains a read-only source context while later source-name
bindings remain possible. Its lifetime must be visible in Rust, not hidden behind
an unsafe pointer. `build()` drops this lookup context; the completed core Plot
and Output own their data independently. Direct-vector inputs are not retained
after snapshotting. These are Rust ownership decisions, not a claim that Seaborn
is immutable or that the future Python object lifecycle has been settled.

Lookup/unsupported dtype/shape failures belong to the binding call, with role,
source variable and logical slot/cell context where relevant. Native scientific
errors retain their existing preflight/render timing and original error sources;
do not duplicate every core check in the frontend or parse core error strings.

## Coverage Across The Existing API

All 25 families need direct source-bound/vector input, not just a scatter facade:

| Family group | Input responsibilities |
| --- | --- |
| Scatter, Line, Regression, Marginal | Coordinate variables; Line's supplied band and supplemental point data bind through the same frontend. |
| Histogram, Density, Qq | Direct sample variables; preserve the native histogram samples-versus-supplied-geometry distinction. |
| Trajectory, LinePath, Interaction | Constructor-owned group/condition/estimate variables and supplied interaction bounds. Numeric/categorical axes follow native applicability, not arbitrary dtype inference. |
| Boxplot, Violin, Raincloud | Category and sample variables; native summaries/raw overlays and supplied raincloud geometry remain available. |
| Bar, HorizontalBar, StackedBar, StackedHorizontalBar | Category/value/positioned-series variables; explicit sample versus supplied-estimate/bounds choices, not an added analysis policy. |
| PointInterval, Forest, Coefficient | Category/term, estimate and explicit lower/upper bindings; keep reference values and availability semantics separate from foreign dtype conversion. |
| Dumbbell | Category and paired endpoint variables; endpoint labels are display metadata. |
| Heatmap | Rectangle and record input, optional axis labels, Boolean highlights and null-cell semantics. |
| Matrix | Ordered numeric variable selection; retain native matrix-specific composition/faceting restrictions. |
| Beeswarm | Category, value and continuous-color variables. |
| Waterfall | Category and delta variables; base, totals and their display rules remain native. |

Adapt every data-bearing position: hue/group/series, provenance, complete facet
specifications, bounds, status bindings, highlights, point/bar-point layers,
labels and data-alpha mappings as admitted by the underlying family. Small
adapter-owned Bounds/Facet/layer/input specifications can hold names or foreign
vectors; they express plot semantics, not materialized replacement dataframes.
Core-only style/policy/geometry values are reused directly rather than renamed.
Do not require a native-value escape hatch to complete an ordinary adapter plot.

Alignment follows each owning semantic input, not one blanket DataFrame height.
Bounds and status align with their estimate records. Supplemental layers may use
their own source/direct vectors and record counts, and keyed alpha may use a
separate selector table. These inputs use explicit local contexts without changing
the parent plot's lookup source or joining datasets implicitly.

Alpha must still bind to its actual native unit (observations, records, series,
aggregates, bins or Single). Do not turn every source column into observation alpha
or reduce arbitrary rows to a collective value. Likewise, do not infer estimate
availability from nulls when the native contract requires an explicit status.

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

1. **Replace Polars' public entry point first.** Implement the whole-frame/name
   binding path and direct Series/Column input with a real grouped/faceted plot.
   Extend that same frontend across the data-bearing capability inventory, not a
   separate facade for each example. Reuse the existing private dtype converters.
2. **Replace ndarray's public entry point.** Direct views, selected table columns,
   matrix heatmaps and defined wide operations use the same native rendering
   boundary. Cover interval/layer bindings too, not only a dense-grid shortcut.
3. **Complete migration and verification.** Replace the adapter examples/guide
   excerpts with direct plotting calls, remove public numeric/categories/booleans/
   source_rows/numeric_columns/heatmap_values conversion exports and native-output
   helper bundles, and keep only private conversion support with real consumers.
   Migrate their public tests; do not retain aliases, deprecations or dual catalogs.

Migrate each package's callers with its working replacement, not only at the very
end. Core-native input constructors remain unchanged; removing adapter conversion
exports is not authorization to delete the standalone value API. Update the
current adapter contract after implementation and remove this completed plan.

These are implementation steps after separate direction, not preflight projects.
No dependency upgrade, toolchain change or Python bridge decision is needed for
the input-interface replacement. Retain the established working package metadata
and Cargo joint offline archive verification; prior registry publication is not
required. Do not reintroduce the previous false packaging blocker.

### Focused Evidence And Documentation

During implementation, verify the actual new contracts: named lookup and mixed
direct vectors; metadata/position alignment; missing-name/dtype errors; mapping
across hue/facets/bounds/layers; correct wide/grid origin semantics; source lifetime
and owned completion; preserved family restrictions; and native composition/Output
reuse. Existing stride/chunk/rounding tests still matter for private conversion.
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
