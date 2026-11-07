# Python Constructor API Plan

## Status And Scope

This is a constructor inventory and API draft, not an implemented Python package,
stub file or change to the Rust API. The selected architecture, selectors, binding
snapshots and object lifecycle remain in the
[workspace/interface plan](91_WORKSPACE_AND_INTERFACES_PLAN.md). Current scientific
behavior remains defined by the [rendering contract](02_RENDERING_CONTRACT.md) and
[adapter contract](09_DATA_ADAPTERS.md). The selected
[columnar geometry refactor](93_COLUMNAR_GEOMETRY_PLAN.md) owns the prerequisite
core/adapter migration, not an already implemented API.

The inventory covers all 25 existing families, their primary input alternatives
and a bounded set of optional categorical bindings. Row-position/column-name modes,
explicit numeric discovery and flat named geometry inputs are selected below.
Exact annotated overloads, remaining label defaults and method signatures still
need completion; recording a selected direction does not deliver a binding.

## Shared Public Surface

- Data comes from exactly four input families: Python native data, NumPy, pandas
  and eager Polars. DataFrame/Series and array/vector/matrix forms are admitted in
  their appropriate roles, not interchangeably in every parameter.
- Do not expose Rust scientific payload classes as another Python data route.
  Numeric/category/axis values, bounds, closed sample/estimate inputs and supplied
  geometry are constructed internally from the four input families. This does not
  remove the separately selected public Theme/Text and configuration-value APIs.
- Use one snake_case callable per family. `data` is the only positional argument
  and supplies lookup context; primary variable roles are keyword-only. Direct
  values can be mixed with selectors. Direct projections do not install a parent
  lookup source.
- Keep the accepted `col[...]`/`cols[...]` grammar. Scalar field names, NumPy column
  indices and explicit single-column selectors resolve one variable. `cols[...]`
  resolves an ordered collection only where the table admits multiple variables.
  Bare list/tuple data never becomes a field selection by matching its contents.
- Use ordinary concise type aliases and meaningful `typing.overload` declarations
  in eventual stubs. Aliases describe the supported Python containers/selectors;
  they are not runtime data wrappers. Do not put an exhaustive NumPy scalar-width
  union into each signature or erase parameters to unrestricted Any/kwargs.
  Dtype, rank, validity, row-count and source-dependent selector checks still run.
- A constructor's ordinary optional `hue`, `group` and `series` bindings default to
  no extra binding (`None`). They are also available through their admitted
  chainable setters and use the same binding path. In expanded inputs, no extra
  binding preserves the generated role; it is not a request to clear that generated
  role. Supplied Raincloud's coordinated hue-key columns are instead part of its
  complete scientific input form, not an independently replaceable hue mapping.
- Apart from the deliberate supplied-geometry label conveniences below, other
  configuration remains on deliberate methods: themes/text, dimensions,
  facets, source IDs, alpha, status, highlights, layers and family policies. For
  example, sample Histogram bin/stat/composite selection, Qq reference lines,
  Regression bands and LinePath styling retain named methods. Do not duplicate
  every method into a constructor. A constructor-owned policy such as raw Bar
  error generation remains a constructor argument.

The table's role patterns omit the common `data=None, *` prefix and append the
optional-binding column as keywords defaulting to None. For example, the Scatter
row expands to `scatter(data=None, *, x, y, hue=None)`, not a positional x/y call.
Each pattern lists the required roles for that form; optional endpoint pairs and
alternative forms are stated explicitly. This is a parameter inventory, not the
final annotated `.pyi` text.

## Inventory Of All 25 Families

| Rust family | Python constructor role pattern | Optional categorical bindings | Native meaning and additional forms |
| --- | --- | --- | --- |
| Scatter | `scatter(x, y)` | `hue` | Numeric x and one or multiple numeric y variables. Actual x values are preserved. Explicit `by_row_position=True` is the alternative below. |
| Histogram | `histogram(samples)` or `histogram(bin_lower, bin_upper, bin_density, kde_x, kde_y, rug_x, rug_baseline, rug_top)` | None | Numeric raw samples with unchanged native bin/stat/composite defaults, or the complete flat density-axis geometry below. The forms are mutually exclusive. |
| Density | `density(value)` | `hue`, `group` | Numeric samples; the core owns density estimation and group/hue restrictions. |
| Qq | `qq(value)` | None | Numeric samples against the native reference policy; no precomputed quantile mode is invented. |
| PointInterval | `point_interval(category, estimate, lower, upper)` | None | Categorical labels and supplied estimates with mandatory drawable endpoints. Availability remains an explicit status operation. |
| Line | `line(x, y)` | `hue`, `group` | Numeric x, sorted by the native Line semantics; one or multiple numeric y variables. Optional supplied bands use `.band(lower, upper)`. Explicit `by_row_position=True` is the alternative below. |
| Trajectory | `trajectory(x, y, group)` | `hue` | Required categorical graphical group; x is numeric or categorical according to supported type/dtype. y is one numeric variable. Native within-group ordering and always-visible observation markers remain unchanged. |
| LinePath | `line_path(x, y, group)` | `hue` | Numeric x/y and required categorical graphical group. Preserve source-row order and repeated x; do not use ordinary Line sorting. |
| Interaction | `interaction(condition, estimate, group, lower, upper)` | `hue` | Categorical condition/group and supplied estimates/bounds. No raw-sample aggregation mode. |
| Coefficient | `coefficient(term, estimate)` or `coefficient(term, estimate, lower, upper)` | None | Categorical terms with supplied point estimates, optionally with a complete endpoint pair. No automatically generated intervals. |
| Forest | `forest(term, estimate, lower, upper, reference)` | None | Categorical terms, supplied estimates/bounds and a required literal numeric reference. Do not invent a default reference of zero or one. |
| Dumbbell | `dumbbell(category, left, right, left_label, left_color, right_label, right_color)` | `group` | Categorical labels and two numeric endpoints. The four literal label/paint keywords draft a flattened Python spelling for native endpoint configuration, preserving distinct required labels and complete RGBA paints. |
| Heatmap | `heatmap(grid=...)` or `heatmap(x, y, fill)` | None | A numeric rectangle, or categorical x/y record coordinates with numeric fill. `grid` can be direct data or an admitted ordered selection. The two complete forms are mutually exclusive. |
| Bar | `bar(category, samples)` or `bar(category, estimate)` or `bar(category, estimate, lower, upper)` | `series` | Raw-sample means with optional sample-only `error_bar` policy, or supplied estimates with optional supplied endpoints. Detailed combination rules follow. |
| HorizontalBar | `horizontal_bar(category, samples)` or `horizontal_bar(category, estimate)` or `horizontal_bar(category, estimate, lower, upper)` | `series` | Same scientific input alternatives as Bar, with native horizontal/sign/reference capabilities. |
| StackedBar | `stacked_bar(category, value, series)` | None | Supplied numeric values and required categorical category/series. Preserve native sums; not the ordinary Bar sample-mean mode. |
| StackedHorizontalBar | `stacked_horizontal_bar(category, value, series)` | None | Same required stack channels and native sums as StackedBar, with horizontal geometry. |
| Boxplot | `boxplot(category, value)` | `series` | Categorical groups and one or multiple numeric sample variables, or the explicit `by_column_name=True` form below. |
| Violin | `violin(category, value)` | `series` | Categorical groups and one or multiple numeric sample variables, or explicit `by_column_name=True`. Native density/statistical rules remain unchanged. |
| Raincloud | `raincloud(category, value)` or `raincloud(category, q1, median, q3, density_category, density_y, normalized_width, point_category, point_y, normalized_strip)` | `series` in sample forms only | Grouped samples, including multiple variables and explicit `by_column_name=True`, or complete flat geometry. With geometry hue, require coordinated `hue`, `density_hue`, `point_hue` columns. No sample-only series in geometry forms. |
| Regression | `regression(x, y)` | None | One numeric x/y pair; native OLS and RegressionBand policies, not a Python estimator. |
| Marginal | `marginal(x, y)` | `hue`, `group` | One numeric x/y pair with native composed views. No generic facets are added. |
| Matrix | `matrix(variables=...)` or `matrix(numeric_only=True)` | `hue`, `group` | An explicit ordered numeric variable collection/rectangle, or explicit discovery from a source as below. Rows are observations. No generic facets are added. |
| Beeswarm | `beeswarm(category, value, color)` | None | Required categorical positions, numeric observations and constructor-owned numeric continuous color. `color` is not optional categorical hue. |
| Waterfall | `waterfall(category, delta, base)` | None | Categorical labels, ordered numeric deltas and a required literal numeric base. The core performs its ordered cumulative pass; no default zero base is introduced. |

`reference`, `base` and endpoint labels/paints are literal configuration, not
source-column roles or implicit broadcasts. Constructor-owned `group`, `series`,
`color` and `fill` channels remain constructor-owned; a universal setter must not
make them replaceable on families that do not admit replacement.

### Samples, Estimates And Supplied Intervals

The Bar/HorizontalBar draft uses the already discussed named alternatives:

```python
# Planned calls, not executable examples of a delivered binding.
bar(df, category="group", samples="value")
bar(df, category="group", samples="value", error_bar=policy)
bar(df, category="group", estimate="mean")
bar(df, category="group", estimate="mean", lower="lo", upper="hi")
```

- Require exactly one of `samples` and `estimate`. Report conflicting or incomplete
  calls rather than choosing a winner or trying another native constructor.
- `error_bar` configures generated error bars only for raw samples. Omission keeps
  the core policy: arithmetic cell means and deterministic two-sided 95%
  percentile-bootstrap intervals, 1,000 resamples and seed zero. A policy that
  hides errors is distinct from None or an estimate-only form.
- `lower`/`upper` belong only to supplied estimates and must occur together. They
  are actual drawable endpoints, not error lengths or an inferred confidence
  policy. Supplied estimates without the pair generate no interval.
- Coefficient has the same endpoint-pair completeness rule, but no sample mode.
  PointInterval, Interaction and Forest always require both endpoints.
- Primary inputs are required values, not nullable options just because the
  implementation needs an omission marker. None is not a missing-role substitute;
  supported missing entries inside a variable remain a separate data-validity issue.
- Keep the native status/validation boundary: binding nullable estimate variables
  does not infer availability or reject them before a later explicit status edit
  can make the complete configuration valid.

Use overload signatures for these genuinely different complete argument
combinations. Interval validation and all statistics remain native.

### Axis Types And Expanded Variables

Trajectory reuses the existing Rust distinction without Python native-value
wrappers or another axis-mode argument. Numeric values/dtypes select numeric x;
strings and admitted categorical dtypes select categorical x. A selected field is
classified by that field's dtype, not by the Python type of its lookup key.
String labels that look numeric stay strings; integer values do not silently
become category labels. Boolean input is not a numeric coordinate.

For an x input admitting both meanings, an untyped empty/all-missing sequence or
an untyped Null column cannot choose a meaning. Require the caller to prepare a
supported typed NumPy/pandas/Polars variable. Do not guess or reintroduce an exposed
Rust axis wrapper. This does not reject missing entries where the role/dtype is
already established or eagerly replace the core's useful-data validation.

Scatter/Line admit multiple y variables; Boxplot/Violin/sample Raincloud admit
multiple value variables. The distinction between a single variable and an
explicit collection, including a one-element `cols[...]`, follows the existing
adapter contract, not a Python length heuristic. Required x/category remains one
variable and is repeated by origin only in admitted expanded forms.

The native adapter installs generated column hue for multi-y Scatter, hue and
group for multi-y Line, and positioned series for multi-value distributions.
Apply explicitly supplied optional constructor bindings afterwards through the
same path as chained setters. Row-scoped inputs repeat by origin; generated and
cell scopes remain explicit methods. Replacing a generated role never causes it
to be restored implicitly. Line bands must match the declared single/many-y shape.

Heatmap grid slots remain row-major with row zero at the top. Grid labels change
display, not identity; highlights and IDs retain their existing cell-shape rules.
Matrix variables remain ordered columns, not rows; discovery requires the explicit
option below. Do not derive coordinates/IDs from a pandas index.

## Selected Explicit Modes And Discovery

These selected Python spellings preserve existing native operations and the
one-callable rule. They do not rename the Rust adapter methods or claim binding
implementation.

### Row Position And Column Name

Rust has explicit `indexed` Scatter/Line constructors and `columns`
Boxplot/Violin/Raincloud constructors. Omitting x/category in an ordinary form
must not silently choose these meanings.

Keep the existing variable roles and use narrow Boolean mode keywords only where
they disambiguate the scientific input:

```python
scatter(df, y=cols["a", "b"], by_row_position=True)
line(df, y=cols["a", "b"], by_row_position=True)
boxplot(df, value=cols["a", "b"], by_column_name=True)
violin(df, value=cols["a", "b"], by_column_name=True)
raincloud(df, value=cols["a", "b"], by_column_name=True)

scatter(y=measurements_matrix, by_row_position=True)
boxplot(value=measurements_matrix, by_column_name=True)
```

`by_row_position=True` generates zero-based logical row x, never a pandas index,
and is mutually exclusive with explicit x. `by_column_name=True` makes each
ordered variable a category and is mutually exclusive with explicit category.
Use available column names for display; unnamed inputs receive default display
names without turning those labels into scientific identity/matching keys.
Ordinary forms still require x/category. Neither flag means "select several
columns" in a role that does not otherwise support it. Direct rectangles/ordered
variable collections retain their admitted shape in these same named roles.

Describe the complete forms with ordinary overloads, including the true mode
literal where needed. Do not add separate same-capability plotting functions,
infer modes from omitted inputs or introduce parallel `x_cols`/`y_cols` selectors.

### Explicit Numeric-Column Discovery

Rust exposes numeric discovery separately from ordinary all-column selection.
The Python entry must also make discovery explicit rather than silently dropping
string/metadata/unsupported fields from a requested collection.

Use `numeric_only=True`, restricted to Matrix's variable discovery and the
row-position/column-name forms above:

```python
matrix(df, numeric_only=True)
scatter(df, by_row_position=True, numeric_only=True)
line(df, by_row_position=True, numeric_only=True)
boxplot(df, by_column_name=True, numeric_only=True)
violin(df, by_column_name=True, numeric_only=True)
raincloud(df, by_column_name=True, numeric_only=True)
```

Discovery requires an admitted table source and replaces an omitted variable
collection; combining it with explicit `variables`, `y` or `value` is an argument
error. It is not an instruction to filter an explicitly selected collection.
Preserve source column order and supported numeric dtype rules. Numeric IDs are
included too: discovery does not infer measurement roles or exclude columns by
their names/values. Boolean/unsupported fields and untyped Polars Null do not join
discovery just because another explicit role could bind them. Ordinary explicit
selections must still reject unsupported selected fields.

`matrix(df)` does not mean discovery. The explicit rectangle calls remain
`matrix(variables=rectangle)` and `heatmap(grid=rectangle)`; source-plus-selector
calls retain their separate lookup context. No implicit whole-table shortcut is
selected.

## Flat Supplied Histogram And Raincloud Geometry

These are real existing scientific capabilities, not optional omissions from
Python coverage. Their selected Python representation is named column arguments,
not `geometry=payload`, fixed-string-key maps/TypedDict protocols, nested records
or required runtime geometry wrappers. Core/adapter changes are specified in the
[columnar geometry refactor](93_COLUMNAR_GEOMETRY_PLAN.md).

Each column admits the applicable ordinary direct data or single-column selector.
`data` remains the only positional argument; auxiliary sources are keyword-only.
Check the complete argument combination before extraction, then call the native
sample or geometry path. Do not guess mode from field matching, lengths or values.

### Histogram

| Component | Required named columns | Lookup source |
| --- | --- | --- |
| bins | `bin_lower`, `bin_upper`, `bin_density` | main `data` |
| KDE | `kde_x`, `kde_y` | `kde_data` |
| rugs | `rug_x`, `rug_baseline`, `rug_top` | `rug_data` |

Both sample and complete-geometry forms support direct data and source selection:

```python
# Sample-owned Histogram; existing bin/stat/composite defaults remain native.
histogram(samples=[0.25, 0.75, 1.25, 1.75])
histogram(observations, samples=col["measurement"])

# Complete supplied geometry: direct columns, not a public payload class.
histogram(
    bin_lower=[0.0, 1.0], bin_upper=[1.0, 2.0], bin_density=[0.5, 0.5],
    kde_x=[0.0, 1.0, 2.0], kde_y=[0.0, 0.5, 0.0],
    rug_x=[0.25, 1.75], rug_baseline=[0.0, 0.0], rug_top=[0.1, 0.1],
    value_label="Measurement", density_label="Density",
)

# The three component tables need not have equal row counts.
histogram(
    bins, bin_lower=col["lower"], bin_upper=col["upper"],
    bin_density=col["density"],
    kde_data=kde, kde_x=col["x"], kde_y=col["density"],
    rug_data=rugs, rug_x=col["x"], rug_baseline=col["baseline"],
    rug_top=col["top"], value_label="Measurement", density_label="Density",
)
```

All eight geometry columns are required together and exclude `samples`. Complete
geometry uses native density-axis behavior, not arbitrary precomputed count bars.
Its display keywords are `value_label` and `density_label`, not `*_name`.

### Raincloud

| Component | Required named columns | Additional columns in the hue form | Lookup source |
| --- | --- | --- | --- |
| summary | `category`, `q1`, `median`, `q3` | `hue` | main `data` |
| density | `density_category`, `density_y`, `normalized_width` | `density_hue` | `density_data` |
| points | `point_category`, `point_y`, `normalized_strip` | `point_hue` | `point_data` |

Both sample and complete-geometry forms support direct data and source selection:

```python
# Grouped samples; optional series uses the ordinary sample binding path.
raincloud(category=["a", "a", "b", "b"], value=[1.0, 2.0, 2.0, 3.0])
raincloud(observations, category=col["condition"], value=col["measurement"])

# Complete supplied geometry without hue, using direct columns.
raincloud(
    category=["a"], q1=[1.0], median=[2.0], q3=[3.0],
    density_category=["a", "a", "a"], density_y=[0.0, 2.0, 4.0],
    normalized_width=[0.0, 1.0, 0.0],
    point_category=["a", "a"], point_y=[1.0, 3.0],
    normalized_strip=[0.25, 0.75],
    category_label="Condition", value_label="Measurement",
)

# Complete hue form from three explicitly scoped component sources.
raincloud(
    summary, category=col["condition"], hue=col["treatment"],
    q1=col["q1"], median=col["median"], q3=col["q3"],
    density_data=density_frame, density_category=col["condition"],
    density_hue=col["treatment"], density_y=col["y"],
    normalized_width=col["width"],
    point_data=point_frame, point_category=col["condition"],
    point_hue=col["treatment"], point_y=col["y"],
    normalized_strip=col["strip"],
    category_label="Condition", value_label="Measurement", hue_label="Treatment",
)
```

Geometry requires every summary/density/point column in its complete form and
excludes sample `value`, `by_column_name`, `numeric_only` and `series` inputs.
Without hue, all three hue columns are absent/None; with hue, supply all three.
A missing entry within a supplied hue column is not the no-hue form. Geometry hue
keys do not introduce a sample `.hue(...)` setter. The display keywords are
`category_label`, `value_label` and, for the hue form, `hue_label`, not `*_name`.

### Component Binding, Metadata And Validation

- Each variable resolves only against its component's explicit source. An omitted/
  None auxiliary source requires direct columns there; it does not mean "use main
  data". Reuse one table by passing it explicitly for each desired context.
- Direct and selected columns can be mixed within any component. For example,
  `kde_data=kde, kde_x=col["x"], kde_y=adjusted_density` binds selected x and direct
  y, while all other required columns remain present. Source choice is never
  inferred from a matching name, data length or alpha target.
- Check lengths within bins/KDE/rugs and summary/density/points independently.
  Domains need not be equally long, and DataFrame indexes never join them.
- Snapshot only bound variables during this call. Retain only main `data` for
  later explicit source-bound setters; auxiliary sources do not become implicit
  lookup contexts or need to survive completed binding. Exports do not reread them.
- Core Raincloud preparation groups by raw category/hue strings, not independently
  encoded dictionary integers or display labels. Summary order controls group
  order; each group's child rows retain their relative input order. Original point
  row indices survive grouping for observation alpha. Unknown/duplicate/missing
  groups, incomplete hue, invalid ordering and incomplete components fail rather
  than being repaired or re-estimated.
- `*_label` keywords configure existing native axis/guide Text slots and remain
  available at construction, with later owner text methods for edits. They do not
  change scientific dimensions, raw group keys or diagnostic column names. No
  public Python `*_dimension`/`*_name` parameters or binding-side label store are
  added. Complete exact label-default/None types under the shared text contract.
- Preserve existing complete-geometry minima, limits and scientific validation.
  No missing component is computed. The core still rejects sample-only facets/
  source IDs and supplied-Raincloud series/significance at its existing boundary.
  Adapters do not duplicate this scientific validation.

Sample-owned Histogram with a supplied KDE remains a distinct composite option,
not this complete-geometry mode. The prerequisite plan replaces its shared public
KDE point vector with named x/y columns too. Final Python method signatures must
bind ordinary data at an explicit direct/independent-source call, without a public
Rust point class or a hidden lookup against the samples source.

## Remaining Work Outside This Constructor Inventory

Selector negative indices/steps/bounds, callback parameters/ownership, exception
mapping, full policy/method signatures and distribution verification remain in
the parent plan. In particular, Bootstrap sampling-unit data, independent layers,
status mappings, alpha mappings and supplied-KDE data must also accept ordinary
inputs rather than require Rust payload objects. Do not silently make an unbound
selector inside a policy read the main Plot source; specify its binding context
and snapshot call when that method inventory is completed.

Do not treat the table as a completed installed-stub or native-ownership gate.
Verify representative valid/invalid overload calls against actual runtime
combination checks when bindings exist, without a family/container cross-product.
This document-only inventory needs source/link consistency and whitespace checks,
not a Rust build or renderer test suite.

## Inventory Sources

- [Core semantic constructors](../crates/lichtung-plot/src/builder.rs) and
  [family capabilities](../crates/lichtung-plot/src/kind.rs).
- [ndarray record constructors](../crates/lichtung-plot-ndarray/src/frontend/constructors.rs),
  [wide/indexed forms](../crates/lichtung-plot-ndarray/src/frontend/wide.rs) and
  [grid forms](../crates/lichtung-plot-ndarray/src/frontend/grid.rs).
- [Polars record constructors](../crates/lichtung-plot-polars/src/frontend/constructors.rs),
  [wide/indexed/discovery forms](../crates/lichtung-plot-polars/src/frontend/wide.rs)
  and [grid forms](../crates/lichtung-plot-polars/src/frontend/grid.rs).
- [Bar source/error policies](../crates/lichtung-plot/src/spec/bar.rs),
  [estimate inputs](../crates/lichtung-plot/src/spec/estimate.rs) and
  [complete supplied geometry fields](../crates/lichtung-plot/src/geometry.rs).
