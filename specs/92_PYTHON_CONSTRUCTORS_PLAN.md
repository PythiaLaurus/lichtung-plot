# Python Constructor API Plan

## Status And Scope

This is a constructor inventory and API draft, not an implemented Python package,
stub file or change to the Rust API. The selected architecture, selectors, binding
snapshots and object lifecycle remain in the
[workspace/interface plan](91_WORKSPACE_AND_INTERFACES_PLAN.md). Current scientific
behavior remains defined by the [rendering contract](02_RENDERING_CONTRACT.md) and
[adapter contract](09_DATA_ADAPTERS.md).

The inventory covers all 25 existing families, their primary input alternatives
and a bounded set of optional categorical bindings. Straightforward mappings are
drafted below; the three outstanding constructor decisions are explicitly listed
under [Decisions Requiring Confirmation](#decisions-requiring-confirmation).
Draft spellings are not evidence of user approval or working bindings.

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
- A constructor's optional `hue`, `group` and `series` bindings default to no extra
  binding (`None`). They are also available through their admitted chainable
  setters and use the same binding path. In expanded inputs, no extra binding
  preserves the generated role; it is not a request to clear that generated role.
- Other configuration remains on deliberate methods: themes/text, dimensions,
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
| Scatter | `scatter(x, y)` | `hue` | Numeric x and one or multiple numeric y variables. Actual x values are preserved. Explicit indexed forms remain to be spelled below. |
| Histogram | `histogram(samples)` | None | Numeric raw samples. Core bin/stat/composite defaults remain unchanged. Alternative `histogram(geometry=...)` requires the supplied-geometry schema below; it is not a samples alias. |
| Density | `density(value)` | `hue`, `group` | Numeric samples; the core owns density estimation and group/hue restrictions. |
| Qq | `qq(value)` | None | Numeric samples against the native reference policy; no precomputed quantile mode is invented. |
| PointInterval | `point_interval(category, estimate, lower, upper)` | None | Categorical labels and supplied estimates with mandatory drawable endpoints. Availability remains an explicit status operation. |
| Line | `line(x, y)` | `hue`, `group` | Numeric x, sorted by the native Line semantics; one or multiple numeric y variables. Optional supplied bands use `.band(lower, upper)`. Indexed forms remain to be spelled below. |
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
| Boxplot | `boxplot(category, value)` | `series` | Categorical groups and one or multiple numeric sample variables. Explicit column-category forms remain to be spelled below. |
| Violin | `violin(category, value)` | `series` | Categorical groups and one or multiple numeric sample variables. Native density/statistical rules remain unchanged. Column-category forms remain to be spelled below. |
| Raincloud | `raincloud(category, value)` | `series` in sample forms only | Grouped samples, including multiple sample variables; explicit column-category forms remain to be spelled below. Alternative `raincloud(geometry=...)` requires a complete supplied-geometry schema and does not accept sample-only series. |
| Regression | `regression(x, y)` | None | One numeric x/y pair; native OLS and RegressionBand policies, not a Python estimator. |
| Marginal | `marginal(x, y)` | `hue`, `group` | One numeric x/y pair with native composed views. No generic facets are added. |
| Matrix | `matrix(variables=...)` | `hue`, `group` | An explicit ordered numeric variable collection or numeric rectangle, with rows as observations. Explicit numeric discovery remains to be spelled below. No generic facets are added. |
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
Matrix variables remain ordered columns, not rows or automatically discovered
numeric fields. Do not derive coordinates/IDs from a pandas index.

## Decisions Requiring Confirmation

These are proposed spellings/representations, not selected public capabilities.
They must preserve the existing scientific operations and the one-callable rule.

### Explicit Indexed And Column-Category Modes

Rust has explicit `indexed` Scatter/Line constructors and `columns`
Boxplot/Violin/Raincloud constructors. Omitting x/category in an ordinary form
must not silently choose these meanings.

Recommended Python spellings keep the existing variable roles and use narrow
Boolean mode keywords only where they actually disambiguate the scientific input:

```python
scatter(df, y=cols["a", "b"], indexed=True)
line(df, y=cols["a", "b"], indexed=True)
boxplot(df, value=cols["a", "b"], by_column=True)
violin(df, value=cols["a", "b"], by_column=True)
raincloud(df, value=cols["a", "b"], by_column=True)
```

`indexed=True` generates zero-based row x and excludes explicit x.
`by_column=True` uses ordered variable identities as categories and excludes
explicit category. Ordinary forms still require x/category. Neither flag means
"select several columns" in a role that does not otherwise support it. Direct
rectangles/variable collections remain possible in these same named roles.

Confirm these keyword spellings before producing their overloads. Do not add
separate same-capability plotting functions, infer modes from omitted inputs or
introduce parallel `x_cols`/`y_cols` selectors.

### Explicit Numeric-Column Discovery

Rust exposes numeric discovery separately from ordinary all-column selection.
The Python entry must also make discovery explicit rather than silently dropping
string/metadata/unsupported fields from a requested collection.

Recommended spelling is `numeric_only=True`, restricted to Matrix's variable
discovery and the indexed/column-category forms above:

```python
matrix(df, numeric_only=True)
scatter(df, indexed=True, numeric_only=True)
boxplot(df, by_column=True, numeric_only=True)
```

Under this proposal, discovery requires a source and replaces an omitted variable
collection; combining it with explicit `variables`, `y` or `value` is an argument
error. It is not an instruction to filter an explicitly selected collection.
Preserve schema order and the native admitted numeric dtype rules; Polars Null
does not join discovery just because an explicit numeric role could bind it.
Ordinary explicit selections must still reject unsupported selected fields.

Confirm this spelling/scope rather than assume `matrix(df)` means discovery.
The baseline explicit rectangle calls remain `matrix(variables=rectangle)` and
`heatmap(grid=rectangle)`; source-plus-selector calls retain their separate lookup
context. No implicit whole-table shortcut is selected by this inventory.

### Supplied Histogram And Raincloud Geometry

These are real existing scientific capabilities, not optional omissions from
Python coverage. With Rust payload classes excluded from Python data inputs,
their public data representation needs an explicit schema.

Recommended entry forms are `histogram(geometry=payload)` and
`raincloud(geometry=payload)`, using Python-native mappings/ordered records with
TypedDict documentation rather than new runtime geometry classes. Component-table
support from NumPy/pandas/Polars needs the same explicit fields and source scope;
do not guess geometry from coincident column names.

The schema must preserve at least these existing fields:

| Geometry | Required information |
| --- | --- |
| Histogram | Semantic value/density dimension names; ordered contiguous bins with lower/upper/density; ordered KDE x/y points; explicit rug x/baseline/top segments. Core minima and complete-geometry requirements still apply. |
| Raincloud | Semantic category/value and optional hue dimension names; ordered category/hue groups; each group's ordered density y/normalized-width points, q1/median/q3 and complete raw y/normalized-strip points. |

Confirm the mapping/record schema, component-table forms and metadata spelling
before publishing constructor overloads for geometry. Keep geometry mutually
exclusive with raw samples, grouped/column sample modes and sample-only error
policies. It is self-contained data, not a request to estimate missing components.
The core continues to reject sample-only facets/source IDs and supplied-Raincloud
series at its existing validation boundary. Supplied Histogram geometry selects
the native density-axis behavior; it is not arbitrary precomputed count bars.

Sample-owned Histogram with a supplied KDE remains a distinct composite option,
not this complete-geometry mode. Its ordinary-data spelling belongs to the same
follow-up data-bearing policy inventory, not a public Rust point/geometry class.

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
