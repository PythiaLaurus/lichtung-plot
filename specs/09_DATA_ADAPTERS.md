# Data Adapter Contracts

## Native Identity And Package Boundary

`lichtung-plot-ndarray` and `lichtung-plot-polars` are independent consumers of
the public standalone renderer. The core never depends on either adapter,
including through development dependencies. Neither adapter depends on the other.
They own lookup, conversion, origin metadata and configuration forwarding, not
another renderer, statistical model, runtime, network or asset provisioner.

Both roots use actual unrestricted `pub use lichtung_plot::*` re-exports. Every
public native relative path, including descendants of `kind` and `stats`, retains
its original definition, methods, bounds, errors and usage after only a crate-prefix
substitution. Root Plot, Bounds, AxisValues, closed scientific inputs, native
layers, Canvas, Theme, Output, Error and Result are not adapter wrappers. The
native API is not a curated allowlist; future root-name collisions require a
coordinated API decision, never silently shadowing the native item.

A caller needs only its chosen adapter as a direct renderer dependency. With one
compatible resolved core package, both adapter paths name identical native types;
their PlotAdapter and AdapterError types remain distinct. Different core versions
or sources can instead produce distinct types. Compatible versioned local core
dependencies and the root lockfile preserve the shared package here. Adapter
archives contain their own source/dependency declarations, not embedded core
sources. Joint Cargo archive verification does not merge packages or authorize
publication.

## Family Frontends And Ownership

Each adapter adds 25 root family aliases of its actual
`PlotAdapter<'source, kind::Family>`. `Family::new(...)` and
`Family::from_data(source, ...)` return the same exact-family frontend as standard
`Result<Self, AdapterError>`. `new` snapshots direct inputs without installing an
implicit table; `from_data` additionally retains a read-only source for subsequent
name/index lookup and may mix selectors and direct inputs. Direct input lifetimes
do not unnecessarily become the completed frontend's lifetime.

Each binding resolves and snapshots its own inputs at that call. It does not
materialize unused columns, render, defer the first error, clone a complete table,
join indexes or fall back from a missing field to a literal. Clone preserves
native configuration and safely shares the read-only context. Conversion
`From<PlotAdapter<K>> for Plot<K>` moves the already-owned native plot, preserves
its family and ends lookup without rendering or scientific preparation. Direct
inputs may expire after binding; retained sources may be edited/dropped after
handoff, subject to other live borrows. Frozen Output never reopens a source.

There is no public conversion-function catalog, source wrapper, data-role wrapper,
`data(source)` factory, `build`, `into_native`, Deref escape or adapter Canvas.
Sealed input protocols exist only for well-formed public signatures; callers pass
ordinary supported views, tuples, selections and native policies without importing
or implementing those traits. Production modules remain private.

Canvas and concrete application consumers still take native Plot values. Explicit
`.into()` works, including inferred `panel(row, column, adapted.into())` with the
current native signature; an annotated native intermediate is also valid. This is
not automatic argument coercion. Already-native values cross adapter paths without
conversion. No new core acceptance protocol or extension-trait import is required.
`erase` releases lookup and returns native `Plot<kind::Any>`, which remains
ineligible for exact-family Canvas insertion.

### Fixed Constructor And Scientific Modes

The following arguments apply to `new`; `from_data` adds the source first. Each
additional semantic mode also has an `_from_data` form and returns the same family.
No sample/estimate, axis, grid/record or omitted-coordinate meaning is inferred
from lengths or a similar upstream plotting signature.

| Family | Arguments | Additional meaning/modes |
| --- | --- | --- |
| Scatter | x, y-or-ys | Actual numeric coordinates; explicit indexed modes. |
| Histogram | samples | Native samples; geometry accepts native HistogramGeometry. |
| Density | value | Native sample density. |
| Qq | value | Native sample/reference policies. |
| PointInterval | category, estimate, lower, upper | Required supplied bounds; explicit status is separate. |
| Line | x, y-or-ys | Native sorted numeric lines; band(lower, upper) and independent PointLayer. |
| Trajectory | x, y, group | Constructor-owned group; explicit numeric-x/categorical-x constructors resolve ambiguous inputs. |
| LinePath | x, y, group | Numeric x and original source-order paths. |
| Interaction | condition, estimate, group, lower, upper | Constructor-owned group and supplied bounds. |
| Coefficient | term, estimate | Point-only; with_intervals chooses supplied bounds. |
| Forest | term, estimate, lower, upper, reference | Required bounds and literal scientific reference. |
| Dumbbell | category, left, right, endpoints | Native DumbbellEndpoints policy. |
| Heatmap | grid / selected columns | Rectangle; records(x, y, fill) explicitly binds categorical cell records. |
| Bar | category, samples | Native raw-sample mean/default error-bar policy; explicit modes below. |
| HorizontalBar | category, samples | Same sample/estimate distinctions and native sign/reference capabilities. |
| StackedBar | category, value, series | Required positioned-series values and native sums. |
| StackedHorizontalBar | category, value, series | Same stack semantics and exact native family. |
| Boxplot | category, value-or-values | Native samples and positioned-series expansion; explicit columns modes. |
| Violin | category, value-or-values | Native sample/density policies and columns modes. |
| Raincloud | category, value-or-values | Samples/columns; geometry accepts native RaincloudGeometry. |
| Regression | x, y | Existing native OLS and RegressionBand policy. |
| Marginal | x, y | Native composed views and restrictions. |
| Matrix | variables / selected columns | Scatter-matrix columns; explicit numeric discovery, no generic facets. |
| Beeswarm | category, value, color | Constructor-owned numeric color. |
| Waterfall | category, delta, base | Literal base and original ordered deltas. |

Bar/HorizontalBar `with_error_bar(category, samples, policy)` takes native
`impl Into<ErrorBar>`. `estimates(category, estimate)` uses supplied values without
generating intervals; `estimates_with_error_bars(category, estimate, lower, upper)`
uses supplied endpoints. There is no post-construction input/error-bar replay
setter. Histogram/Raincloud `geometry` and `geometry_from_data` preserve native
geometry order and preflight restrictions, including unsupported faceting and
supplied Raincloud series/significance combinations.

Native closed inputs remain directly available through root Plot. Adapter Line
binds `.band(lower, upper)?`; native Line still takes `.band(Bounds)`. There is no
`band_bounds` compatibility method. Regression's same-name setter remains its
different native RegressionBand policy.

## Foreign Input Grammar

| Role | Polars | ndarray |
| --- | --- | --- |
| Source | Borrowed eager DataFrame | Read-only ArrayView2 or (values, present) views; rows are observations |
| One selected variable | Exact literal name | Zero-based usize logical column index |
| One direct variable | Borrowed Series or Column, no IntoColumn ceremony | ArrayView1, including supported Option elements |
| Ordered selected variables | Name array or borrowed name slice | Index array or borrowed index slice |
| Direct many-y | Ordered borrowed Series/Column collection | ArrayView2 or matching validity pair |
| All columns | Explicit `..` in column-selection modes | Explicit `..` in column-selection modes |
| Rectangle | Numeric DataFrame/ordered columns | Numeric two-dimensional view/validity pair |

Selections are ordered, nonempty and unique; missing/out-of-range selectors fail
at binding. A one-element collection still expands a column dimension. A name
slice is not string observations, and an index slice is not numeric observations.
A selector without a source is an unresolved binding, not a literal or broadcast.
Other roles remain single-variable unless explicitly admitted above.

Native NumericValues, CategoryValues, BooleanValues, SourceRows, AxisValues and
AlphaEncoding can pass through their corresponding ordinary value-attachment
roles without re-expansion. Foreign status/result/selector metadata is resolved
from foreign variables; native typed statuses/annotations/encodings have their
unchanged direct attachment operations. No composite compatibility-only method
replaces the native closed-input API.

### Common Value Semantics

- Ordinary primitive integers/floats normalize to f64 with ordinary rounding,
  without a losslessness gate or general strictness switch. Null remains distinct
  from present NaN, infinity and signed zero.
- Binding does not compact, sort, impute, aggregate, infer tests, parse strings or
  invent temporal units. Native families retain their own statistics, validation
  and applicability. In particular, Line, Trajectory and LinePath keep their
  different native ordering meanings; ordinary sample bars still use native means.
- String keys remain exact. Native categories use first-observed non-null levels;
  explicit Labels/CategoryOrder remain native presentation policies. No implicit
  numeric-to-category formatting or continuous hue is introduced.
- SourceRows uses exact present nonnegative integer IDs fitting usize, followed by
  native uniqueness. IDs never pass through f64. Source slots, explicit IDs,
  category codes and generated positional identities remain distinct.
- Empty vectors/zero-row matrices retain their structural meaning. Native consumers
  decide whether a useful plot exists; an explicitly empty variable selection
  still fails. Checked products and fallible adapter buffers do not promise
  recovery from all allocations in native constructors or foreign libraries.

### ndarray Views And Validity

`crates/lichtung-plot-ndarray/src/source.rs` retains safe borrowed descriptors
behind a sealed ArraySource context, never unsafe pointers or an eagerly copied
project-owned table. `input.rs` and `snapshot.rs` bind selected roles in logical
index order. Support transposes, reversed/strided axes, Fortran storage and
caller-created broadcast views without requiring contiguous storage. Scalar,
dynamic-rank and higher-rank inputs are not implicitly flattened.

Sealed primitive elements admit f32/f64, signed/unsigned integer widths through
128 bits, isize/usize, String/&str and bool with their supported Option forms.
Role support remains distinct; a homogeneous numeric table does not become a
categorical table. Complex/custom/semantic elements require upstream projection.

An ordinary `(values.view(), present.view())` pair accompanies its values at the
same binding operation. Exact rank/shape must match in every dimension, not just
flattened length; strides may differ. True means present, unlike NumPy's
true-means-hidden mask convention. Option absence and false validity both retain
null slots. Composite roles nest each masked variable intact, for example a
category/series target `((category, present), series)`; masks are never guessed as
a second categorical variable. IDs and alpha bytes still require present values.

The workspace pins ndarray 0.17.2 with defaults disabled and only std; no BLAS,
Rayon, serde or matrixmultiply threading feature is requested.

### Polars Logical Dtypes

`crates/lichtung-plot-polars/src/input.rs` resolves fields/direct variables;
private numeric/category/Boolean/provenance modules normalize only admitted logical
dtypes, including empty inputs, before materialization or physical interpretation.

| Role / dtype | Meaning |
| --- | --- |
| Numeric Int8/16/32/64/128, UInt8/16/32/64/128, Float16/32/64 | Ordinary f64 normalization; preserve null/nonfinite identity. |
| Category String, Categorical, Enum | Exact actual strings, never dictionary codes. Unobserved levels are not rows; Enum schema order is not an implicit CategoryOrder. |
| Boolean | True/false/null, not numeric 0/1. |
| Numeric/category/Boolean Null | Explicit role chooses an all-null snapshot with the original logical length. Ambiguous trajectory x requires new_numeric_x/new_categorical_x or source-bound counterparts. |
| IDs | Exact checked integer values, rejecting absent, negative, oversized and noninteger values by logical slot. |
| Alpha bytes | Exact non-null UInt8, not floating opacity or other integer widths. |
| Temporal, Decimal, binary, nested, object, extension or unknown dtype | Unsupported implicit interpretation, including empty columns; project/choose units upstream. |

Typed iterators span chunks and sliced validity without rechunking or editing
caller data. Repeated ScalarColumns use their full logical length, including null
and zero length; materialization may allocate/populate an upstream cache. Numeric
casts are ordinary typed primitive casts, not NonStrict fallible-cast-to-null.
Categorical decoding retains original strict-cast error sources.

Only eager inputs are admitted; no LazyFrame, expression evaluation, IO or joins
are executed. polars-core 0.55.2 is pinned directly with defaults disabled and
dtype-i8/i16/u8/u16/i128/u128/f16/categorical features. A matching facade re-exports
these same types. No facade IO/lazy/cloud/dtype-full/ndarray bridge is requested;
upstream core still includes Tokio/async/Rayon dependencies.

## Wide, Grid And Alignment Units

Scatter/Line actual-x many-y constructors repeat x and use column identity as
default hue; Line also installs native group. Boxplot/Violin/sample Raincloud
repeat category and use column identity as positioned series. Numeric x such as
`[0, 5, 20]` remains those actual coordinates, never row positions. Multi-y Line
bounds require matching row/column shapes and ordered variable collections; one
foreign pair is never reused for every series. Prepared native bounds retain
their original expanded slot semantics.

`indexed`/`indexed_from_data` on Scatter/Line explicitly choose zero-based row x.
`columns`/`columns_from_data` on Boxplot/Violin/Raincloud explicitly choose column
categories with cell samples. `indexed_numeric_from_data`,
`numeric_columns_from_data` and Matrix `numeric_from_data` explicitly discover
supported numeric columns in schema order; Polars excludes untyped Null from
discovery. Ordinary selection/`..` includes unsupported fields and fails rather
than silently discarding metadata. Explicit numeric selection can resolve Null.

Source names/indices remain raw selected identities. Direct collections/matrices
use stable local ordinals with supplied names or positional display defaults.
Duplicate nonempty display names retain native collision validation, not merging
or silent renaming. Generated field identities avoid source-name collisions and
do not introduce reserved selector strings. Later setters replace native roles;
the adapter never restores a replaced group/hue/series implicitly.

Ordinary hue/group/series/facet and observation-alpha bindings are row-scoped and
repeat by origin after expansion. `_cells` operations bind already-expanded slots,
never infer scope from lengths. Generated dimension operations are
`hue_columns`, `group_columns`, `series_columns`, `facet_columns_row`,
`facet_columns_column`, `facet_columns_wrap` and crossed
`facet_grid_columns_row`/`facet_grid_columns_column` with their `_cells` variants.
Cell facets include `facet_row_cells`, `facet_column_cells`, `facet_wrap_cells`,
`facet_grid_row_cells`, `facet_grid_column_cells` and `facet_grid_cells`.

Rectangular Heatmap cell `(r,c)` occupies slot `r*ncols+c`, x column c and y row r;
row zero displays at the top. Missing fill retains coordinates. Grid highlights
are exact same-shape Boolean rectangles/views, ordered source-column selections,
or explicit flattened `_cells`
variables; true adds a border independently of validity. Grid IDs are exact
same-shape integer cells or `source_rows_cells` variables, one unique ID per
expanded record. Row IDs are never repeated across columns. Shapes check every
dimension, including equal-product mismatches. Private metadata retains logical
rows and selected source-column origins independently of labels. Record heatmaps
retain native categorical-coordinate and duplicate-cell rules.

Grid `x_labels`, `y_labels` and field/view-bound `row_labels` change display only,
without merging positions or becoming provenance. `column_labels` assigns labels
to selected/generated wide or Matrix variables without changing identity. Counts
must match. Adapter aliases share one current native Labels policy; successive
calls preserve each other, `.labels(...)` replaces it completely, and subsequent
adapter calls do not resurrect replaced aliases. Native collision behavior remains.

## Facets, Status And Independent Layers

`facet_row`, `facet_column`, `facet_grid`, `facet_wrap` and explicit scope variants
snapshot categorical bindings and construct native FacetSpec. `facet(native)`
forwards and caches the complete supplied spec. `facet_scales(policy)` requires an
existing facet and replaces its scales via the native method. Each new facet
replaces the whole previous selection with native Fixed defaults. Capabilities
and data-dependent native restrictions remain unchanged; Matrix/Marginal have no
generic faceting.

PointInterval/Coefficient/Forest status is explicit: Boolean true/false maps to
Estimated/NotEstimable; categorical status uses an exact supplied mapping.
Null/unmapped status fails at binding; numeric absence never infers availability.
Native `estimate_status(iterator)` forwards unchanged. Binding null numeric tuples
must not reject a plot before a valid later availability setter can be applied.

PointLayerAdapter and BarPointLayerAdapter own direct `new` and independent
`from_data` constructors. Coordinates/category/value roles align only within that
layer, not with the main plot's count or source. Point-layer labels and bar-layer
series resolve locally, without inherited hue/facets or SourceRows. Exact native
paint, radius, alpha, legend identity, text layout and public accessors retain
native behavior. Bar series replacement reconstructs the native layer through
public constructors/getters. Layer conversion ends lookup and moves the native
layer; PlotAdapter `point_layer(impl Into<NativeLayer>)` also accepts native layers.
Native Plot setters keep their original exact native arguments and family bounds.

## Alpha And Independent Selector Tables

`point_alpha`, `line_alpha`, `fill_alpha` accept exact observation bytes or an
unchanged native AlphaEncoding on the corresponding native target capability.
Ordinary foreign values use row origins; `_cells` selects expanded slots.
`_single` explicitly broadcasts one u8/exactly one bound value; ndarray distinguishes
literal `64u8` from selector `0usize`. Constant columns are not inferred Single.
Native target/unit/schema applicability remains preflight-owned: e.g. Line stroke
alpha is series-scoped, not observation-scoped. No row-to-series reduction occurs.

`_linear(binding, domain, range)` delegates to the native linear factory.
`_map(binding, mapper)` supplies native NumericValues to a FnOnce returning native
Result<AlphaValues>; `_map_categories` supplies native CategoryValues. Callers can
choose native linear/log10/continuous/threshold/categorical factories without
constructing converted foreign values themselves. Mapper output length must match
its bound variable. Wide row mappings materialize once before selection/repetition,
retaining the full declaration and callback timing; cell mappings materialize
from explicit cell values. No callback survives into rendering/replay.

`_keyed(keys, values)` and `_keyed_from_data(source, keys, values)` bind ordered
`(plot_field, source_binding)` category selectors for exact actual series/aggregate
and optional facet keys. Independent tables resolve at the call, retain no borrow
and do not replace the main source. Histogram `fill_alpha_bins(lower, upper,
facet_keys, values)` and its source form use exact native intervals, never sample
inference or approximate edge matching. Native tuple uniqueness/completeness rules
apply; unused pooled tuples do not create marks.

`_columns(column_key, other_keys, values)` and independent source forms resolve
the generated dimension without a magic field string. Keys are raw Polars selected
names, ndarray selected indices, or zero-based local ordinals for direct
collections/matrices. Numeric/category mapped `_single_map`, `_cells_map`,
`_keyed_map`, `_columns_map` and Histogram `_bins_map` variants retain these units;
independent keyed/column/bin forms end in `_from_data`. Native Theme reset remains
`AlphaEncoding::theme()` in the ordinary one-argument method, not a compatibility
reset wrapper. Alpha zero preserves scientific membership.

## Significance Results

`significance_pairwise(left, right, marks)` and `significance_attached(target,
marks)` append native typed annotations. Their independent `_from_data` forms
resolve their own source, never the parent as fallback. Each endpoint independently
accepts a category variable, category/series pair or typed native target sequence.
Bare numeric bindings are supplied p-values; typed native marks remain marks.
`(category_binding, mapping)` explicitly maps categories to marks, rejecting absent
or unmapped values. Native target/mark scalars explicitly broadcast; all non-scalar
inputs agree on result count independently of main observations. All-scalars forms
produce one annotation; empty sequences do not invent annotations.

`_in_facet` adds one field/view or typed native FacetCell; `_in_grid` binds row and
column fields/views. Both have source forms. Raw targets and facet identities are
not display aliases. Native p-value/target/facet/placement validation retains its
normal timing. No tests, p-value adjustment, star-string inference or joins occur.

Root functions with these same semantic names/arguments return standard
`Result<Vec<Significance<Placement>>, AdapterError>` for reusable native results,
without retaining foreign sources. Original native `significance(iterator)` is
forwarded and appends rather than replacing existing annotations. Attached is
admitted on Bar/HorizontalBar/Coefficient; Pairwise on Bar/HorizontalBar,
StackedBar/StackedHorizontalBar, Boxplot/Violin/Raincloud, with native runtime
restrictions for supplied geometry.

## Forwarding, Errors And Verification Ownership

Each adapter's `src/frontend/forwarding.rs` is its local native-configuration
inventory. Preserve original consuming shape, arguments, callbacks, return types
and kind bounds for text/labels, numeric formats/ticks/guides, categories/scales,
themes/paint, sizing, legends, raw overlays, family-specific policies and text
layout. Do not erase families, replay configuration or require a configure closure.
The separate data-bearing methods above bind fallibly; root native methods remain
unchanged. Unsupported target/family setters remain compile-time unavailable;
native data/unit/geometry/facet restrictions remain native preflight failures.

Binding returns standard `Result<_, AdapterError>`, without a public result alias.
AdapterError retains structural/lookup/dtype/shape/slot context and original native
or upstream sources, with lossless From<native Error> for combined `?` workflows.
Do not stringify into native Error::Data or parse native errors to duplicate
scientific validation. Root Error/Result remain native, and terminal `svg`,
`svg_preserve_text`, `png`, `png_preview`, `png_with_options` return native
Result<Output>, preserving operation/publication behavior.

The package-local examples use adapter-only renderer imports, default to the
working directory and accept one optional output directory. They share real source
anchors and frozen outputs with equivalent peer-language single-page guides.
The Book helper's test-only consumer checks cross-adapter native identity and
handoff; its production command does not depend on the renderer. Follow
[development](90_DEVELOPMENT.md) for semantic test ceilings, focused validation,
strict Clippy, independent-package verification and publication authorization.
