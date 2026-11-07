# Workspace, Data Interfaces And Bilingual Documentation

## Status And Authority

This is an implementation plan with the delivered workspace baseline identified
below; it is not evidence that the remaining planned components exist.
Current behavior remains defined by [architecture](01_ARCHITECTURE.md),
[rendering](02_RENDERING_CONTRACT.md), [property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md),
[layout](05_LAYOUT_SYSTEM.md), [numbers](06_NUMERIC_PRESENTATION.md) and
[development](90_DEVELOPMENT.md). Update those contracts with working behavior,
not in anticipation of it.

The scope covers a workspace migration, new Polars/ndarray adapters, a complete
Python interface, and replacement of the loose guides with a serious Book-style
documentation system and gallery. The standalone renderer now lives in a virtual
workspace with a nonpublished Rust Book assembly command. The authorized Book,
example and gallery foundation covers existing Rust capabilities. Both ndarray
and Polars now expose complete direct plotting frontends and unchanged native
re-exports, including wide/grid bindings, independent layers and scientific result
tables. Workspace checks, joint offline archives, cross-adapter consumers and
rebuilt peer guides have passed. Python bindings remain planned. Package publication,
site deployment, Git push and history rewriting are not authorized by this work.

Python architecture, the four input routes, constructor/binding/explicit-selector
rules, one-runtime-entry/overload-signature model, immutable configuration values,
text-entry/omission/clear/precedence rules and object-lifecycle semantics below are
settled implementation directions, supported by bounded prototypes and explicit
API discussion, not delivered binding code. Configuration sharing is a preferred
implementation direction, not a completed experiment. The
[Python constructor inventory](92_PYTHON_CONSTRUCTORS_PLAN.md) now maps all 25
families and separates draft signatures from decisions requiring confirmation.
Required native ownership/text-clear facilities, open constructor forms and exact
stubs still need design and verification before expanding the binding surface.

### Delivery Status

| Objective | Current status |
| --- | --- |
| Mechanical workspace and policy migration | Delivered; workspace verification, representative example execution and standalone Cargo package verification passed. |
| Peer-language Book/example/gallery foundation | Delivered: one gallery-first tutorial page per language, six scientific showcase workflows, two numeric/text reference workflows, same-page image-to-code walkthroughs, shared runnable-source includes and zero-argument examples with optional output redirection. The helper lives under docs/; full per-family gallery coverage remains for integrated content closure. |
| ndarray and Polars adapters | Delivered: all 25 families, source/direct views and fields, wide/grid origins, native configuration, independent layers, statuses, alpha and significance results. Cross-adapter native handoffs, focused/workspace checks, joint offline archives and peer guides passed; the [current contract](09_DATA_ADAPTERS.md) owns the API. |
| Python lifecycle decision and complete binding | Four ordinary-data input routes without exposed Rust payload classes, binding-time snapshots, one runtime entry with overload signatures, explicit col/cols, mutable owners, immutable configuration values, text keyword/clear rules and span/overwrite semantics settled. All 25 constructors inventoried as a draft with open mode/discovery/geometry choices. Bounded experiments completed; no binding/selectors/stubs, native text-clear or region-overwrite implementation. Native API/ownership details and full verification are open. |
| Integrated content/package/site closure | Rust adapter examples, peer guides/site and all three source archives are verified. Python and complete per-family gallery coverage remain planned; no publication or deployment was performed. |

## Confirmed Scope And Constraints

| Concern | Required outcome |
| --- | --- |
| Existing adapter/binding code | Replace the existing adapters' conversion-only public design, reusing sound private conversion support; Python is not implemented. No repository/history import is requested. |
| Renderer boundary | One independent `lichtung-plot` core, with adapters and bindings depending on it, never the reverse. |
| Adapter public entry points | Either adapter exposes every native public item/module at its original relative path and with the same definition/signatures. Foreign input uses top-level `Family::new` and `Family::from_data` with non-conflicting helper names, never native-name shadowing or a mandatory binding namespace. No separate direct core dependency is needed; both adapters can coexist and share native types through one compatible resolved core package. |
| Python coverage | Equivalent scientific/rendering capabilities across all 25 plot families, composition, themes, text adaptation and numeric/tick callbacks. Translate Rust scientific payload inputs into the four Python data routes, not a literal re-export of data classes. |
| Python input | Exactly four data-source families: Python native data, NumPy arrays, pandas DataFrame/Series and eager Polars DataFrame/Series. Convert internally to core values; do not expose NumericValues/CategoryValues/AxisValues, Bounds, closed source inputs or geometry payload classes as another public data-input route. |
| Python construction | For source-plus-variable calls, data is the only positional argument; variables use keywords. Common optional bindings such as Scatter hue also have chainable setters using the same input/native validation path. Support mixed selectors/direct values, not type-dependent positional reinterpretation. |
| Python overloads | One runtime callable per family checks explicit scientific keyword combinations and selects native paths. Shipped typing.overload signatures describe distinct legal call forms for editors/type checkers; they do not perform runtime dispatch or validation. |
| Python typing and constructor inventory | Use concise ordinary aliases for supported containers/selectors and meaningful overloads, not exhaustive scalar-width signatures or untyped kwargs. The constructor inventory covers all 25 families and identifies pending indexed/column-category, numeric-discovery and supplied-geometry spellings. |
| Python column selection | Use col[key] for one exact field and cols[...] for an ordered column collection or explicit positional range. Bare list/tuple variable inputs remain direct data; retain native container indexing/projection inputs. Do not add x_cols/y_cols keywords, columns(...) aliases or schema-dependent selector guessing. |
| Python data ownership | All supported inputs use binding-time snapshots of the variables actually bound, not a frozen whole source. Retain the supplied source object only for later explicit lookups; rendering never refreshes existing bindings. |
| Python lifecycle | Mutable Plot/Canvas configuration, explicit configuration copies, insertion-time Canvas snapshots, non-consuming terminal renders and explicit frozen-Output reuse; sharing/copy-on-write is an implementation preference, not a second public mode. |
| Python configuration values | Theme, Text and their nested appearance/layout values expose read-only named attributes and keyword-only copy_with derivation. No direct field assignment, mutable child handles or duplicate getter/per-field-edit API; retain the native nested Theme representation and validation. |
| Python text entry points | Text constructors and applicable string/Text-taking slots accept common style/layout keywords. Explicit keywords, including None clearing, override incoming Text fields; absent keywords defer to native incoming-field/preserve rules. Use one native Text-slot path, not a separate Title style model. |
| Text clearing | Add narrow native operations to remove local style/layout from Text values and existing owner slots. Keep ordinary setters and explicit-default request presence unchanged; Python maps explicit None to the native capability, never a second style store or whole-Plot reconstruction. |
| Canvas placement | Python uses one panel method with optional keyword-only spans and overwrite=False. Rust keeps strict panel/panel_span and adds explicit set_panel/set_panel_span; region-overwrite ownership/validation belongs to the core, not a Python-only reconstruction. |
| Python compatibility | CPython 3.10 is the required minimum; start with abi3-py310. Actual platform/dependency release support still requires installed-wheel/sdist verification, not a moving support-window recommendation. |
| Numeric conversion | Ordinary numeric inputs convert to the core's binary64 representation with normal floating-point rounding, following established plotting practice; no new per-value losslessness gate or general strictness switch. |
| Book languages | Chinese and English have equal status, equivalent content and coverage; neither language is the source or reference edition for the other. |
| Guide navigation | Each language has one main tutorial page with in-page contents; real gallery images appear first and link directly to same-page code explanations. |
| Documentation tooling | Keep the Rust helper under docs/book-helper/, without a top-level dev/ directory or renderer dependency. |
| Test scale | Preserve the existing whole-repository ceilings: 800 unit tests, 300 module-integration tests and 80 independent core-E2E scenarios, including ignored cases. |
| Publication | Package/site build verification is in scope; actual publication, deployment and Git push require separate authorization. |

The [deferred-feature review](99_DEFERRED_FEATURES.md) remains separate. Building
its workspace/interface prerequisites does not admit the deferred drawing/legend
work or satisfy its first-publication checkpoint.

## Reference-Backed Architecture

### Workspace And Dependency Direction

Use a virtual workspace root, keeping the rendering crate intact rather than
splitting fonts, layout, scenes or backends into new public crates. The intended
organization is:

```text
Cargo.toml                         # virtual workspace
Cargo.lock
rust-toolchain.toml
Justfile
AGENTS.md
crates/
  lichtung-plot/                    # renderer src/, tests/, examples/
  lichtung-plot-polars/             # new Rust adapter and its public examples
  lichtung-plot-ndarray/            # new Rust adapter and its public examples
bindings/
  python/                          # Rust extension, Python package/tests/examples
docs/
  book-helper/                     # working nonpublished Rust assembly command
  en/book.toml
  en/src/index.md                  # gallery-first main tutorial page
  en/src/sections/                 # authoring fragments, not separate pages
  zh-CN/book.toml
  zh-CN/src/index.md
  zh-CN/src/sections/
  gallery.tsv
  theme/guide.css
specs/
```

Do not add empty workspace members before they have working consumers. The
nonpublished Rust package docs/book-helper provides gallery assembly alongside
the documentation, without a separate top-level dev/ directory. Keep root Justfile
recipes as the normal entry points and do not build a generic workflow framework
or add a `tools/` directory. This placement is a repository organization decision,
not a requirement imposed by mdBook or the reference projects.

[Cargo workspaces][cargo-workspaces] and [Polars' workspace][polars-workspace]
provide direct precedents for a virtual root, shared lockfile/dependency policy,
member-local packages and explicit default members. Preserve the existing Rust
edition/toolchain during the mechanical move. Specify the resolver explicitly.
Share appropriate metadata, dependency versions and lints; do not equate sharing
metadata with defining their public APIs. The adapters' complete native re-exports
are an explicit user-facing contract, not an automatic workspace behavior.

Keep ordinary default development lightweight, with the core as the default
member. Full workspace validation must explicitly select all intended members;
default-member success is not adapter/binding verification. A core-only consumer
must not compile Polars, ndarray or Python, including through reverse optional
features or development helpers used by the core examples.

Adapters use public core constructors and values. Do not expose backend/layout
internals or introduce another general data model to make them possible. Internal
publishable dependencies need both a local path and a release version; an
in-checkout build is not proof that an independently unpacked package works.

### Repository Policy And Remaining Migration Rules

The root AGENTS/development rules now admit workspace inheritance and deliberate
inter-member dependencies while preserving core independence. Specifications
remain in one workspace-root specs directory, divided by responsibility rather
than duplicated under members. The mechanical move does not create adapter or
binding shells. Remaining additions must keep these boundaries:

- Permit workspace-managed dependencies and local member dependencies while
  preserving core independence and package verification.
- Permit Python package code, stubs, tests and examples within `bindings/python/`;
  do not turn that exception into Python-based renderer/development support
  throughout the repository.
- Retain no-network/no-asset-provisioning behavior, installed-font policy, modern
  Rust module layout and the single-current-API rule.
- Keep formal source-owner paths and repository consumers synchronized with real
  member paths. Do not leave old paths or introduce compatibility packages.
- Keep documentation/generation tooling out of production dependency paths.

Audit package-relative fixture/includes, `CARGO_MANIFEST_DIR`, examples, licenses,
readmes and explicit Cargo test targets. Move the core source, tests and examples
together so that the package remains self-contained.

## Data Adapter Contracts

The [current adapter contract](09_DATA_ADAPTERS.md) defines the implemented API:
foreign data sources plus variable bindings, direct vectors and family-specific
matrix/wide input. A scalar selector denotes one variable; an ordered selector
collection in an admitted many-y position denotes multiple variables with the
explicit x preserved. Explicit indexed/column-oriented family operations select
omitted-coordinate/category wide meanings without adapter marker types.
Grid and record-cell heatmaps are distinct operations. Supplemental layers bind
in their own source context without parent-mapping inheritance. Internal native
conversion is permitted, not an obligatory user preprocessing step.

Each adapter must re-export the entire native public API, including public module
paths, through `pub use lichtung_plot::*` without local root-name shadowing.
Replacing only `use lichtung_plot::...` with either adapter prefix preserves
original native signatures and behavior, including Plot, Bounds, closed inputs,
layers, Error and Result. Foreign-data entry points are additional top-level
`Scatter::from_data(source, x, y)`, `Scatter::new(x, y)` and equivalent
family constructors, not native-name replacements or required extension traits.
Adapter objects bind external inputs directly, using separate data parameters or
scientifically explicit methods rather than parallel input-descriptor types;
native Bounds and BarInput remain unchanged. `new` binds direct inputs without a
retained field lookup source; `from_data` installs the supplied dataset for later field binding
and can mix field selectors with direct values. Both are data workflows, not a
division by container type. Do not require a public bind/frontend namespace.

The PlotAdapter frontend retains native family bounds and owns the real core Plot.
Each family alias denotes the actual exact-family frontend and `new` returns
`std::result::Result<Self, AdapterError>`, not an unrelated factory result.
It forwards native configuration deliberately, without claiming zero maintenance
cost. Source-bound plots keep a lookup borrow; infallible standard `From`/`Into`
conversion releases it and yields the exact native Plot for current Canvas and
other concrete consumers. Direct inputs are released after binding. AdapterError
retains lossless native-error conversion for `?`; no additional result
alias is introduced, and root Error/Result remain native. The current contract
and each adapter's local forwarding inventory own semantic inputs and core boundaries. A native Canvas
acceptance protocol for `.panel(..., adapted)` is a separately reviewed option,
not part of the adapter-only baseline. No core data-access framework, general
Mark/Scale/Stat grammar, adapter Canvas or reverse dependency is prescribed.

Users need no direct core dependency for either foreign or native workflows through
an adapter. The two adapters may coexist without depending on each other; both
must resolve a compatible shared core for their native Plot/Canvas/Theme and
other exports to be identical. Different paths to a re-export do not copy types
or code. Cargo's adapter source archives contain the adapter sources and dependency
declarations, not embedded copies of core sources; joint verification does not
merge the packages. Distinct core versions/sources can introduce distinct types
and are not a supported substitute for the shared-core interoperability contract.

Preserve ordinary f64 rounding, null-versus-NaN distinctions, logical row order,
category identity and explicit provenance. The core remains independent; adapters
reuse its scientific and rendering behavior. Migrate callers and remove the old
public conversion surface with the working replacement, without aliases or a
parallel legacy route. Replace Polars first to establish dataset/name bindings,
then ndarray's vector/table/matrix routes. Validate actual code, review counts
when changing tests, and verify working packages at delivery, not through a
pre-implementation audit or compatibility project. Python retains its own
implementation and verification gate below.

## Complete Python Interface

### Public Capability Mapping

Use PyO3 and maturin with a mixed package layout, an internal native extension,
and shipped typing information. [Maturin's project layout][maturin-layout] and
[PyPA's source-layout guidance][python-layout] address accidental source-tree
imports and packaging boundaries. Keep Python glue thin; no Python-side renderer,
parallel scientific specification or JSON round trip is authorized.

Use one Python-facing entry point per family for direct values and source-bound
variables, dispatching deliberately on supported argument types and call forms.
Do not require separate core/NumPy/pandas/Polars plotting namespaces or different
plot object hierarchies for each data origin. These entry points reuse native
constructors and the appropriate adapters internally. Rust scientific payload
objects are internal conversion targets, not an additional Python input family
or obligatory user wrappers. Public Theme/Text and configuration values keep their
separately selected interfaces. A Python plot owns the same core configuration
regardless of its input route. The shared constructor/binding grammar, explicit
col/cols selection forms and single-runtime-entry/overload-signature model below are
selected. The [constructor inventory](92_PYTHON_CONSTRUCTORS_PLAN.md) covers each
family's required channels and optional categorical bindings; its explicitly open
forms, exact type stubs, remaining label types and slice boundaries still need
completion. Configuration mutation follows the selected lifecycle model, and
callback references have the separate external-state semantics below.

Inventory the core exports and actual family capabilities against Python-visible
constructors, methods, value types and exceptions. Preserve equivalent semantic
capabilities rather than literally reproducing Rust generics, const qualifiers
or borrowed lifetimes. [PyO3 class restrictions][pyo3-classes] require concrete
Python types and deliberate ownership. Erasing everything to Any and accepting
every setter is not equivalent capability coverage.

Numeric/tick callbacks are required capabilities, not optional simplifications.
Design their Python-reference ownership, garbage collection, operation identity,
context lifetime, exception propagation and reentrancy before exposing them.
Python contexts must not outlive borrowed Rust operations or expose private
renderer handles. Pure Rust work and Python callbacks need explicit interpreter
attachment/locking boundaries; see [PyO3 parallelism][pyo3-parallelism]. Do not
silently stringify and discard an application exception's meaningful context.

Preserve exact-tick precedence, once-per-reached-context materialization and frozen
Output replay. A new terminal render is distinct from another publication of the
same Output. Configuration edits must not perform rendering as a hidden cost.

The lifecycle model below selects the Python-visible behavior, not Rust method
signatures or a new standalone `render()` entry point. Identify and validate the
smallest deliberate public core ownership facilities needed to implement it;
neither a blanket Clone addition nor a broad storage refactor follows from the
capability inventory.

### Constructor Grammar And Optional Bindings

For source-plus-variable calls, allow `data` as the first and only positional
argument, or by keyword. Variable roles are keyword-only. The selected Scatter
shape is `scatter(data=None, *, x, y, hue=None)`, not its complete eventual
signature. Do not additionally interpret `scatter(a, b)` as x/y, or guess whether
the first positional object is a source or x based on its type.

The following examples specify planned call forms, not an implemented package:

```python
scatter(x=[1, 2], y=[3, 4])
scatter(df, x="time", y="value", hue="group")
scatter(data=df, x="time", y="value")
scatter(table_array, x=0, y=1)
scatter(df, x="time", y=adjusted_values)
```

- Accept direct supported Python data, arrays and Series in their admitted variable
  roles, including alongside source selectors. A scalar
  string selects an exact field name; a scalar integer selects a zero-based
  logical column in a two-dimensional NumPy source. Neither is a fallback literal
  or broadcast when source resolution fails. Explicit col/cols selectors below
  disambiguate structured field keys and multi-column selection without changing
  bare list/tuple inputs into selectors when their contents happen to match keys.
- Reuse the [adapter input contracts](09_DATA_ADAPTERS.md) for admitted wide/grid
  shapes, ordered selections, origins and alignment units. Bind corresponding
  observations in logical position order; do not join pandas indexes, silently
  broadcast/flatten inputs, infer x/IDs from an index or parse field names as
  expressions. Missing, ambiguous or out-of-range selectors fail at binding.
- Expose common optional data bindings such as Scatter `hue` both as constructor
  keywords and chainable setters. They use the same input-resolution/adaptation
  and native validation path; a setter replaces that role and preserves the
  original configuration on failure. Keep literal styling separate from variable
  mapping: a failed hue lookup must not turn its string into a fixed color.
- Constructor keywords and chained setters bind at their respective call times.
  With unchanged inputs, `scatter(df, x="time", y="value", hue="group")` and
  `scatter(df, x="time", y="value").hue("group")` have equivalent data bindings;
  they are not promised equivalent if the source changes between calls.
- Required scientific inputs remain required. Only expose bindings actually
  supported by that exact family; do not introduce an unrestricted keyword bag or
  duplicate every style/theme/layout setter into every constructor. The constructor
  inventory drafts the hue/group/series conveniences against actual capabilities;
  complete their exact stubs without mirroring every remaining method.

Uniformity concerns argument roles and binding behavior, not a universal x/y
signature or scientific inference. Wide/indexed, sample/estimate and grid/record
modes remain explicit through named inputs/options, never chosen by coincident
lengths, missing required roles or matching source keys. This does not prohibit
type-directed numeric/category adaptation within an already admitted role:
Trajectory x follows supported dtype/element-type semantics, without an added
axis-mode argument or automatic numeric-to-category formatting. Explicit matrix
inputs retain their declared rank; do not flatten them or guess another role.
[Seaborn's keyword-only migration][seaborn-keywords] and
[Plotly Express mixed inputs][plotly-inputs] provide interface precedents, not a
requirement to adopt either library's data lifetime or scientific defaults.

### Scientific Call Forms And Overload Signatures

Keep one runtime callable per family. Use [typing.overload][python-overload] in
the shipped `.pyi` files to describe materially different legal call forms of that
same callable, including their required keyword roles and actual return types.
An ordinary signature suffices when a callable has only one form; do not multiply
overloads into a family-by-input-container cross-product.

- Overload declarations serve editors and static type checkers only. They do not
  select an implementation, validate a call at runtime or require users to choose
  an overload. The binding supplies the actual callable separately from its stub;
  no runtime overload registry or decorator-driven dispatcher is selected.
- The actual callable checks explicitly provided scientific keyword combinations
  and selects the existing native constructor path. Parameters can be individually
  omissible in the implementation while a legal call still requires one complete
  combination. Reject conflicting, incomplete or unsupported combinations instead
  of assigning priority, ignoring extra arguments or trying successive constructors.
- Distinguish modes through meaningful named roles when these suffice, not a
  redundant `mode` keyword or additional same-capability public functions. Where
  identical roles admit different scientific meanings, retain an explicit typed
  scientific option rather than infer meaning from the input data or source schema.
  This does not add post-construction scientific-mode switching setters.
- Preserve argument presence where dispatch needs it, using the private omission
  mechanism rather than conflating omission and explicit None. None remains legal
  only where the individual role admits it; it is not a universal missing-argument
  token. Public stub types list actual accepted values, not the internal sentinel.
- Check the call combination before resolving selectors or extracting foreign
  variables. Then reuse the selected input route and native validation boundary;
  statistics, interval validation and other scientific rules remain native. Calls
  from untyped code must receive the same argument checks without a type checker.

Bar/HorizontalBar's existing raw-sample, supplied-estimate and supplied-interval
constructors motivate these distinct call forms. Overloads describe them; one
runtime family entry routes to them without reimplementing statistics in Python.
The [constructor inventory](92_PYTHON_CONSTRUCTORS_PLAN.md) records the full family
mapping and Bar draft combinations, with remaining mode/discovery/geometry choices
kept explicit. Exact stubs and policy/method signatures still need completion.
Neither that inventory nor this overload mechanism is delivered binding code.

Use ordinary shared type aliases to keep signatures readable; aliases describe
the four supported input families and selectors, not new runtime payload types.
Union types suffice for simple accepted-type alternatives with otherwise identical
call forms. Do not multiply overloads merely to distinguish numeric and category
containers; actual dtype/shape/validity validation remains at runtime.

### Explicit Column Selectors And Direct Container Indexing

The selected Python syntax uses shared, project-owned `col` and `cols` subscription
entry objects, not separate plotting namespaces or a source/container wrapper.
They are planned interfaces, not implemented bindings or existing Seaborn/Plotly
functions. Keep ordinary direct inputs and the container's own indexing available;
do not add parallel `x_cols`/`y_cols` keywords or a `columns(...)` call-form alias.

```python
# Planned call forms; ordinary row-count and family-role checks still apply.
scatter(matrix, x=0, y=[10, 20])  # Direct y values.
scatter(matrix, x=0, y=(10, 20))  # Direct y values too.
scatter(matrix, x=0, y=cols[1, 3])  # Select two logical columns.
scatter(matrix, x=0, y=cols[1:4])  # Select columns 1, 2, 3.
scatter(df, x="time", y=cols["a", "b"])  # Select two named columns.
scatter(x=matrix[:, 0], y=matrix[:, 1:4])  # Direct container projection.
```

- In ordinary data-variable positions, a bare list/tuple is direct data, with or
  without a source. Do not reinterpret it as column keys, a range, a mask pair or
  an implicit broadcast because of matching field names, values, length or shape.
  Policy and structured-data arguments retain their separately declared grammar;
  this is not permission to expose Rust closed-input payload classes in Python.
- `col[key]` denotes exactly one source field. Use literal DataFrame keys, including
  admitted pandas integer and complete tuple/MultiIndex labels, rather than guessing
  positional lookup or formatting a key to a string first. For a two-dimensional
  NumPy source, the admitted integer key denotes a zero-based logical column.
  Ordinary string field names and NumPy integer columns retain their shorthand
  forms such as `y="value"` and `y=1`; col is not mandatory for those common cases.
- `cols[1, 3]` on a NumPy table explicitly denotes two columns, not the interval
  between them. `cols[1:4]` denotes a half-open positional range, selecting 1, 2, 3.
  `cols["a", "b"]` denotes an ordered named selection. `cols` always represents a
  column collection, including `cols[1]`; never collapse a one-element collection
  into the single-variable meaning of `col[1]` or bare `1`.
- Admit a collection only in the family's existing multi-variable/wide/grid roles;
  selectors do not introduce multi-x/multi-hue support, infer indexed x, select
  sample/estimate modes or change observation/cell expansion and origin semantics.
  Share the grammar with subsequent source-bound setters where their roles admit
  it, rather than implement a separate per-family selector language.

Structured single-column labels must be explicit. For example, these have
different meanings regardless of the source schema:

```python
col[("measurement", "value")]       # One field with this complete tuple key.
cols["measurement", "value"]        # Two fields with separate string keys.

scatter(df, x=col[("time", "seconds")],
        y=col[("measurement", "value")])
```

Python passes `col["measurement", "value"]` the same tuple key as the parenthesized
form; do not split it inside the single-column selector. A collection of complete
tuple keys can use `cols[("measurement", "a"), ("measurement", "b")]`. A bare tuple
in a variable position remains direct data even if an identical tuple column label
exists. Full-key lookup must resolve uniquely: reject missing/ambiguous fields,
partial MultiIndex matches, duplicate selections, empty selections and invalid
indices at binding under the applicable adapter rules, never fall back to data.

The chosen examples establish positive half-open NumPy ranges and exact DataFrame
key collections. Finish precise slice normalization/admission, including boundary,
step and negative-index rules and any DataFrame positional-range support, before
implementation; do not silently import pandas label-slice or clipping conventions.
Remaining admitted label types and native identity/display-name mapping also need
verification without conflating exact lookup keys with their formatted labels.

These selectors are small immutable lookup descriptions, not data values, foreign
expressions, callbacks or retained plotting specifications. Creating one does not
capture/read a source, execute a Polars expression, perform I/O or render. Resolve
it against the supplied/retained lookup source during the actual binding call,
then snapshot only the selected variables through the existing input routes. A
selector without a source is an error. Source mutation can affect a later explicit
lookup but must never change the selector's classification or prior bound values.
No new public Rust selector framework or core-to-adapter dependency is required;
the Rust adapters keep their existing input grammar.

Direct NumPy views/slices, supported Series and other admitted projections remain
ordinary direct inputs with binding-time snapshot semantics. A direct many-y
matrix keeps its declared dimensional meaning and does not install an implicit
parent source. Preserve metadata actually carried by a projection, but do not
claim that values alone recover its original selected column identities/names or
that selection, foreign indexing or native binding is universally zero-copy.

### Binding-Time Snapshots And Retained Lookup Sources

Every admitted input route has the same value-lifetime contract: resolve and save
the variables bound by each constructor or setter during that call. Subsequent
external edits to a list, NumPy buffer/view, pandas Series/DataFrame or Polars
Series/DataFrame must not alter those already-bound values. Preserve the applicable
dtype, validity, category and identity contracts through the existing input routes;
do not add container-dependent live behavior or `live=True`/`copy=False` input
policy modes.

Snapshot means stable bound values, not a promise of the same physical copy for
every container. Initial conversion from mutable foreign buffers may copy or
normalize data. Internal immutable native payloads can be shared by configuration
copies and Canvas captures without repeating conversion. Any foreign-buffer
sharing must establish stable contents and ownership rather than assume that a
readonly view prevents its external owner from writing. No general zero-copy or
constant-cost construction guarantee follows from snapshot semantics.

When `data` supplies a lookup source, retain that supplied Python object for later
explicit selectors, separately from the already-owned native values. This can
extend the source's lifetime and retain its memory; it does not deep-copy/freeze
the complete table or convert unused columns. Mutating that same object can affect
future explicit bindings, but never refresh prior ones. Reassigning the Python
name `df` to a different object does not replace the retained source. Resolve later
lookups against its current contents, not a stale constructor-time native import.

Direct variable inputs do not install an implicit lookup source or infer a parent
DataFrame from Series metadata/indexes. Their foreign objects need not be retained
after successful owned binding unless independently supplied as the lookup source.
Configuration copies can share that external context while keeping their bound
values/configurations logically independent; copying is not whole-source freezing.

For example, this planned sequence deliberately binds at two different times:

```python
p = scatter(df, x="time", y="value")  # Save x and y now.
df["group"] = new_groups
p.hue("group")                       # Save the current group; keep prior x/y.
```

By contrast, passing `hue="group"` in the constructor binds x, y and hue during
that constructor call. After either form completes, further source mutation
changes none of those bound roles. Repeating `.hue("group")` explicitly replaces
only hue with a fresh snapshot; it does not reread x/y or other prior bindings.
Plot/Canvas export and frozen Output publication never reopen foreign inputs to
refresh them. These rules do not freeze a callback's separately specified external
state or turn each new render into frozen-Output replay.

Callers must keep source rows consistent across bindings when they want one
aligned dataset. Reordering/editing a source between calls may combine old
coordinates with a new mapping; matching lengths cannot prove row identity. Do
not mask that distinction with hidden index joins or whole-source copies. A
constructor reads its selected inputs during the call; this is not an atomic
whole-table snapshot under concurrent source mutation.
For a new complete dataset, reconstruct the Plot explicitly rather than expect
automatic refresh or add an unselected general data-rebinding API.

### Input Bridges And Compatible Dependencies

The four supported input classes use different boundary conversions, not different
renderers or scientific semantics:

| Python input | Implementation direction |
| --- | --- |
| Python native data | Extract role-appropriate values into internal native inputs; do not require Rust payload objects or force a NumPy/list/JSON round trip. Structured supplied geometry needs the explicit schema identified by the constructor inventory. |
| NumPy arrays | Use rust-numpy readonly ndarray views and the existing ndarray adapter, then release the Python-buffer borrow after binding owned native values. |
| pandas DataFrame/Series | Resolve only selected columns and extract by logical dtype. Numeric roles reuse the NumPy/ndarray route with explicit validity; category, Boolean and identity roles keep their native meanings. Do not require whole-table ndarray, Polars or PyArrow conversion. |
| Eager Polars DataFrame/Series | Bridge into the existing Polars adapter rather than first flattening the table into ndarray. The concrete Python-to-Rust bridge still needs native runtime verification. |

Preserve the [adapter value contracts](09_DATA_ADAPTERS.md): ordinary numeric f64
rounding, null distinct from present NaN, exact integer IDs, actual string category
keys, logical order, names and explicit provenance. Numeric dtype normalization is
not permission to cast IDs through f64, booleans to 0/1 or arbitrary objects to
display strings. Empty input retains its logical dtype and shape; unsupported
semantic dtypes must not become supported merely because they have no rows.

[rust-numpy][rust-numpy] is the NumPy bridge, not a renderer. Admit supported
strided, reversed, transposed, Fortran and broadcast views without a contiguous-only
restriction. Consume them into owned values before detached rendering. MaskedArray
requires deliberate mask handling or explicit rejection; raw ndarray extraction
can accept the subclass while silently discarding its mask. Adapter validity uses
true = present, the opposite of NumPy's true = hidden mask convention.

For pandas, use public dtype/array interfaces rather than private storage fields.
Keep per-column validity independently of numeric payloads; never use
`np.isnan(values)` as a universal null bitmap. NumPy-backed floating columns have
no independent null bitmap: preserve their NaNs consistently with the same NumPy
input. Nullable/Arrow-backed columns retain their actual null information. Do not
claim to recover None-versus-NaN distinctions already lost by pandas construction.
Whole-table `to_numpy()` can promote integer IDs to floating point and erase
heterogeneous column meanings. Only bound columns are converted, and their source
names must reach native identities/automatic labels rather than disappear into
generic x/y role names. Do not add implicit index joins, sorting or row compaction,
or infer x/source IDs from DataFrame.index. Ambiguous selected names fail instead
of choosing the first matching column. Exact composite-key selection follows the
col/cols grammar; remaining label-type admission and key-to-native identity mapping
require working extraction/typing verification. No Rust pandas adapter crate is
prescribed.

PyArrow is a Python Arrow implementation, not the Arrow protocol itself and not
a required intermediate dependency for all inputs. Ordinary pandas users must not
need Polars/PyArrow just to plot nullable columns; pandas Arrow-backed storage can
use its already-required Arrow support. Correctly configured pandas-to-Arrow or
pandas-to-Polars conversions can preserve the tested values, but their defaults
can turn ordinary NaNs into nulls. Reject those defaults as an implicit universal
route, not the libraries as inherently lossy.

Python Polars is not a Rust ABI object. Prefer reuse of the Polars adapter, while
comparing the official [pyo3-polars bridge][pyo3-polars-types] with direct
[Arrow PyCapsule][arrow-capsules] import for the native boundary. Published
pyo3-polars 0.28 uses Polars' SeriesExport FFI, not a PyArrow conversion; capsule
export likewise does not require the PyArrow Python package. Verify compatibility,
logical types/schema metadata, dependency footprint, reference/release safety and
adapter reuse before selecting one route, without fallback chains. Arrow exchange
does not by itself guarantee zero-copy conversion. A generic PyArrow consumer's
rejection of Polars' 128-bit formats does not disqualify a polars-arrow importer
that recognizes them; the actual Rust import still needs testing.

Choose a compatible dependency set, not independently latest package versions.
Published [pyo3-polars 0.28.0 dependencies][bridge-dependencies] require PyO3 0.29
and Polars 0.55; [numpy 0.29.0 dependencies][numpy-dependencies] also use PyO3 0.29
and admit ndarray 0.17. Start from PyO3 0.29 / rust-numpy 0.29 / ndarray 0.17 as the
tested minimal-extension baseline. The Polars bridge remains manifest-compatible
candidate evidence, not compiled/runtime verification. Final exact pins and Python
dependency bounds belong to the working binding, not speculative workspace edits.

### Python 3.10 And Distribution Verification

Set CPython 3.10 as the minimum. Do not adopt SPEC 0 as a reason to remove it.
Actual releases of [PyTorch][torch-release], [TensorFlow][tensorflow-release] and
[Polars][polars-python-release] declare Python 3.10 support; the interface must
serve that established machine-learning environment.

Use `abi3-py310` as the initial ordinary-CPython wheel design. A minimal extension
has run unchanged across the sampled interpreters/NumPy versions below, but that
does not prove the complete binding or Polars bridge can ship on every target.
Verify required APIs and actual release targets rather than silently dropping
3.10 or treating one local Linux wheel as a manylinux/platform certification.

Do not make latest NumPy the universal minimum. Both
[NumPy 1.26.4][numpy-one-release] and [NumPy 2.2.6][numpy-two-release] provide
CPython 3.10 wheels. Validate old-interpreter/current-compatible dependencies as
well as newer ordinary CPython combinations. Establish the supported dependency,
OS/architecture and wheel-ABI matrix from actual build/install tests, including
the oldest supported interpreter. Free-threaded/PyPy support is not implied by a
`requires-python` range and needs an explicit verified status.

Test installed wheels outside the source tree and build a wheel from the unpacked
sdist. [Maturin's distribution guidance][maturin-distribution] explicitly treats
that as a source-completeness check. Metadata, Rust compilation or an editable
installation alone cannot establish distribution compatibility. Preserve the
renderer font policy; successful installation does not provision fonts/assets.

### Bounded Prototype Evidence And Its Limits

External temporary prototypes were run on Linux x86_64 on 2026-11-06, without
adding binding code, dependencies or support scripts to this repository. They
used actual PyO3 entry points, strict Clippy and optimized builds; these are
design evidence, not retained automated tests or a delivered Python package.

- A PyO3 0.29.2 / rust-numpy 0.29.0 / ndarray 0.17.2 `cp310-abi3` wheel was
  installed outside its source tree and invoked on CPython 3.10.19 with NumPy
  1.26.4 and 2.2.6, and CPython 3.13.11 with NumPy 2.2.6. The loaded native
  binary was identical. This does not verify sdist completeness or other targets.
- pandas 2.3.3 on 3.10 and pandas 3.1.1 on 3.13 passed bounded column extraction
  checks. A small numeric entry point exercised the real ndarray adapter/native
  handoff, reversed views and independence after source mutation/release. Nullable
  floats, integer IDs, strings/categories and Boolean boundary rules were checked;
  not all of their native bindings or all 25 families were implemented. Ordinary
  pandas inputs also worked with Polars/PyArrow imports disabled.
- NumPy noncontiguous views and owned snapshots passed. Raw float64 extraction
  demonstrated the MaskedArray mask-loss hazard. pandas whole-table conversion
  demonstrated integer-ID rounding, while explicit column validity preserved
  null versus present NaN where the source retained that distinction.
- Python Polars 1.41.2 and PyArrow 25.0.1 exercised Arrow production/consumption
  on 3.10, including nulls, categories and source release. Schema metadata mattered
  for Enum identity; one stream export consolidated chunks and copied numeric
  buffers. These were Python consumers, not a tested Rust Polars importer.
- Ordinary title edits cost roughly 0.14 microseconds with a mutable owner versus
  0.4 microseconds for a whole-Plot-clone wrapper; a million shared input rows did
  not increase allocation traffic. With 2048 exact ticks the clone wrapper cost
  roughly 48-50 microseconds per edit. Retaining equivalent independent branches
  required equal Rust allocation traffic in both models. These are local workload
  observations, not universal mutability rankings or performance contracts.
- PyO3 callbacks demonstrated exclusive-borrow reentry failure, successful reentry
  after snapshotting/releasing the borrow, and preserved Python exception identity.
  Same-Output successful publication replay also passed. Numeric/tick callback GC,
  operation/context/thread rules, fallible reusable Canvas edits and Output reuse
  after I/O failure remain unverified.

Keep one native configuration/data representation. Shared observation data already
works, but whole-Plot cloning is not configuration-size independent. The selected
lifecycle prefers narrowly scoped immutable configuration sharing and separation
on edit where justified; it does not authorize persistent collections or a broad
storage refactor. That sharing design, native ownership facilities and remaining
compatibility/distribution checks have not been validated by these experiments.

## Equal-Status Chinese And English Books

Use mdBook as the user-guide system. Keep specs as architectural contracts and
rustdoc as the exact Rust API reference. README becomes an introduction/entry
point, not another full manual. The current main tutorial is one page per language;
src/index.md includes the authoring fragments in src/sections/, and SUMMARY.md
exposes only the assembled page. Do not maintain competing guide implementations
or restore separate pages for those fragments.

Build two peer language editions with matching language-neutral topic/case
identities and equivalent topics, capabilities, examples and release scope:

- Neither edition is an authoritative original, translation backlog or fallback.
- Existing English guides are input material for both editions, not the Chinese
  edition's semantic authority. Both derive from the same contracts and API.
- Use idiomatic prose in each language, without requiring sentence-by-sentence
  literal translation. Review semantic changes in both editions together.
- Missing topics/required examples are build/review failures, not permission to
  insert the other language's page. Language navigation links peer tutorial anchors.
- Mechanically check topic/case coverage, code includes and figure links. Do not claim
  that a structural check proves prose has equivalent meaning.

Start each edition with actual gallery images linking directly to its code
walkthroughs on the same page. Provide an in-page contents menu for the tutorial;
do not require repeated page changes between its main topics. The shared root
gallery offers equal links to both language walkthroughs rather than selecting a
preferred language for the reader. mdBook builds HTML; it does not require a
multi-page tutorial.

Organize the learning path around installation/first plot, native data and
missingness, plot families, axes/numeric formatting, themes/text/annotations,
facets/Canvas, output/fonts/errors, adapters, Python and gallery. Do not merely
wrap the existing file order in a sidebar.

Use [mdBook source includes and anchors][mdbook-includes] to share actual runnable
examples. Do not hand-copy code into both editions or substitute line-number
fragments for stable semantic anchors. Package executable examples with their
own crate/binding, using public APIs and self-contained source fixtures.

## Examples And Gallery As One Executable Content System

Examples own runnable code; guide sections explain it; gallery grids present its
actual output and link directly to those explanations. [Matplotlib's gallery][matplotlib-gallery] and
[Sphinx-Gallery's source/execution model][sphinx-gallery] establish this relationship;
they do not require replacing mdBook with Sphinx or adding another rendering stack.

Give each case a stable semantic identity and metadata linking its source, output
artifacts, purpose, tutorial anchor and API references. A program can produce
several figures and a figure can support several tutorial topics. Keep metadata about
navigation/artifacts, not a new serialized plotting language.

Distinguish scientific showcase cases from technical reference figures in the
same inventory and execution path. The opening grids feature complete scientific
questions, not sparse setter/formatting probes. Keep meaningful variables, units,
references and complete labels. Mark synthetic data and supplied illustrative
intervals honestly; do not imply real-study provenance or automatic inference.
The shared appearance lives in the renderer package's example support and uses
only public APIs, without changing core defaults. Review actual images at full
size and gallery scale, not just successful builds or syntactically valid SVG.
Use roomy responsive cards without cropping figure content.

Examples run without arguments, using the current working directory by default;
an optional directory argument redirects output. Gallery builds choose isolated
per-case paths internally, without requiring manual directory arguments. Keep fixtures local and deterministic,
with no data/font downloads. Produce outlined SVG for portable display and derive
optional PNG thumbnails from the same frozen Output. Preserve-text output remains
useful for relevant demonstrations, not an excuse for a different layout run.

Build generated pages, images and site output in build/staging directories. Do
not manually maintain generated scenes or silently substitute stale/placeholder
images when an example fails. Assemble both language editions into a fresh site,
validate rendered anchor uniqueness and source expansion, and replace the active
site only after both editions pass, without retaining obsolete chapter/case pages
there. Explicit site assembly coordinates rendering and Book assets; ordinary core
compilation must not generate a gallery in `build.rs`.
Use preinstalled, documented generation fonts and record the build environment
without bundling fonts or claiming cross-environment byte equality.

Make every existing plot family discoverable through a meaningful example, plus
representative cross-cutting composition/text/numeric cases. This is not a request
for a family-by-option image matrix. Reuse source and cases already exercising
the same contract rather than create duplicate rendering suites for each language
or each documentation entry point.

## Whole-Repository Verification And Delivery Gates

Keep the current ceilings across all crates, Python, doctests, executable Book
content and automated gallery workflows. Classify their actual contracts before
adding coverage. A new directory, foreign test runner, generated driver or loop
does not exempt independently exercised scenarios. Identify and remove genuinely
redundant coverage and obsolete support where necessary; do not delete distinct
contracts or raise the limits. Stop and report a real capacity conflict rather
than disguise it through packaging or relabeling.

| Work | Required bounded evidence |
| --- | --- |
| Mechanical move | Actual package paths, fixtures/includes, representative public consumers, examples and independent core package verification. |
| Data adapters | Complete native re-exports without shadowing, adapter-only renderer dependencies and cross-adapter native type identity; top-level family source/vector constructors, explicit-x/multiple-y versus omitted-coordinate wide, grid/cell semantics, independent layers, significance and other typed inputs, input/native errors and standard native handoffs. Preserve nullable/type/order and strided/chunked behavior through representative public workflows, not a separate compatibility project. |
| Python | Public capability/type mapping, source-first calls, overload signatures matching runtime scientific-combination checks, explicit col/cols versus direct values/projections, optional bindings, immutable values and shared Text entries with omission/None/value handling and keyword precedence; binding-time snapshots versus current-source lookup, lifecycle/error/callback ownership, input bridges and installed-wheel/sdist workflows. |
| Books/gallery | Peer single-page coverage/anchors/includes, direct image-to-code links, actual example execution, validated rendered HTML and fresh-site assembly without stale-source substitution. |

For code changes use focused tests and strict Clippy, never `cargo check`:

```sh
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
```

Update root recipes to select their intended scope explicitly. Respect the broad
closure requirements for dependency changes and the mechanical workspace/package
migration; state the concrete reason before a full-suite run. Do not repeatedly
rerun the entire suite after localized repairs. Documentation-only planning needs
reference/source consistency checks and `git diff --check`, not compiler tests.

Do not mistake `cargo test --all-targets` for executing every example main, or
`cargo test --doc` for checking Book Markdown. Use actual example execution and
the appropriately configured [mdBook test command][mdbook-test] where relevant.
Check isolated code and end-to-end workflows at their real semantic layer instead
of duplicating every imported fragment as a new test.

Cargo workspace success is not independently unpacked-package success. Verify
the renderer and adapters together through Cargo's temporary archive-backed
registry overlay, using distributable-library metadata rather than excluding the
core with `publish = false`. A prior registry publication is not required. Missing
third-party cache data is not a pass or a reason to substitute `--no-verify`. No
package contents or published ref may reintroduce the full Unicode corpus.

### Delivery Order

1. **Mechanical workspace and policy migration.** Move the core without API or
   algorithm changes, update real source paths/recipes/packaging rules, and verify
   independence. Do not add future-only interface crates.
2. **Book/example/gallery foundation.** Establish peer language books and shared
   runnable-source/output conventions using existing capabilities and examples.
3. **Polars plotting interface replacement.** Deliver complete native re-exports
   and top-level family whole-dataset/name, Series/Column, multi-variable and
   local-layer inputs with native capabilities. Migrate callers and peer guides
   to adapter-only renderer imports, removing the public conversion-only surface.
4. **ndarray plotting interface replacement.** Deliver the same complete native
   exports and top-level family constructors for direct views, selected table columns,
   explicit-x/view2 series and matrix/wide input. Verify shared native types when
   both adapters are used; migrate callers and peer guides with focused evidence,
   removing the old conversion surface.
5. **Python lifecycle implementation gate.** Implement the selected object
   semantics through deliberate public-core ownership and text-clear facilities;
   verify their failure/sharing/callback behavior before mass-producing bindings.
6. **Complete Python interface.** Deliver the capability mapping, callbacks, input
   bridges, typing and distribution verification through the same renderer.
7. **Integrated content and closure.** Complete bilingual content/gallery coverage
   and explicit workspace/package/site verification. Report unexecuted external
   checks honestly; actual release, deployment and push remain separately gated.

Commit each completed logical objective before unrelated work. Commit messages
describe enduring boundaries/behavior, not numbered delivery stages.

## Selected Python Object Lifecycle

The selected public model combines mutable Plot/Canvas owners with explicit
copies, immutable Theme/Text configuration values, insertion-time Canvas snapshots,
non-consuming renders and frozen Output reuse. This follows the bounded analysis
and API discussion, not an assumption that Rust's consuming `mut self -> Self`
means Python mutation. Both [mutation-and-return-self][plotly-updates] and
[new-configuration returns][altair-properties] have established precedents; the
choice here is project-specific, not a universal performance or idiom ranking.

### Mutable Configuration And Explicit Copies

- Plot and Canvas configuration operations edit the same Python object and return
  it for chaining. Ordinary assignment creates an alias, not an independent plot.
- `.copy()` creates a logically independent configuration handle. Later edits do
  not affect the original; immutable observation data and configuration versions
  may remain physically shared until separation is needed. Copying neither
  re-converts foreign inputs nor renders, and need not deep-copy every payload.
- A mutator called without assigning its return still applies the edit. Do not
  add per-setter `inplace` switches or parallel mutable/immutable API modes.
- Configure Plot/Canvas through their deliberate methods, not a parallel public
  attribute-assignment API such as `p.title = ...`. Reject attempts to replace
  configuration methods this way rather than silently shadowing them.
- Failed edits leave the original configuration valid and unchanged; existing
  snapshots remain valid too. A wrapper must not accidentally become empty after
  moving its Rust owner into a fallible consuming method.

### Immutable Configuration Values And Read-Only Fields

Theme, Text and their nested appearance/layout values are immutable Python values,
not mutable owners or live editing handles. Read declared fields through named
attributes and derive edited values with keyword-only `copy_with(...)`:

```python
# Planned Python interfaces, not implemented bindings.
title_style = theme.typography.root.title
size = title_style.font_size
large_style = title_style.copy_with(font_size=24, color=new_color)
```

- Unspecified fields retain the source value; supplied fields produce a logically
  independent result, leaving the original and its aliases unchanged. Chaining
  `copy_with` calls derives further values and requires retaining the result; it
  does not mutate the original. Reassigning a variable to the result is valid.
- Named attributes are read-only. Assignments such as `title_style.font_size = 24`
  or `theme.typography.root.title.font_size = 24` must raise `AttributeError`.
  Nested getters and exposed collections must not provide a mutation back door,
  a mutable parent-linked handle or a detached mutable copy that silently accepts
  field assignment.
- Do not duplicate an attribute as `get_font_size()` or an editable
  `.font_size(24)` method. Ordinary configuration-value updates use `copy_with`,
  rather than parallel per-field edit aliases, writable properties or modes.
  Preserve deliberate native factories/closed choices in their capability mapping;
  this is not permission to expose unchecked fields of every exported type.
- Constructors accept their supported typed configuration fields directly; users
  need not construct a default value and then derive each initial field. Both
  construction and derivation preserve native invariants and validation boundaries.
  A failed derivation leaves the original valid and does not store a partially
  valid record, render, discover fonts or reconvert scientific inputs.
- Copying is logical independence, not recursive deep copying. Unchanged immutable
  payloads may remain shared; do not promise constant-cost derivation or zero-copy
  attachment. Attaching an explicit Theme/Text value retains that value's content
  and instructions, without consuming its Python handle or linking later variants
  back into existing Plot/Canvas configurations.

Keep the eight-component nested representation from the
[native Theme contract](03_THEME_SYSTEM.md), not a flattened Python dictionary or
a separate style schema. For example, `theme.typography.root.title.font_size`
identifies title typography, while
`theme.composition.root_title_block.title_subtitle_gap` identifies a composition
relationship. `copy_with` replaces specified fields/components of a complete value;
it is not recursive dictionary merging or a string-keyed path-patching language.
Derive a child value and replace it in its parent when editing a nested component.

[Flutter ThemeData][flutter-theme-copy] and [TextStyle][flutter-text-copy] provide
precedents for copy-with updates, not a required object graph, null convention or
reactive UI framework. The existing Rust Theme uses `customize()`/`ThemeBuilder`
and its component/Text edits consume values. Python can use those public native
construction facilities without requiring new Rust `copy_with` aliases, exposing
private setters or storing a Python-side representation. A native construction-only
working copy is distinct from the completed immutable Theme and must never become
a retained runtime override or the mandatory Python value-edit workflow.

### Shared Text Constructors And Owner Slots

Text accepts source content and optional supported style/layout inputs at
construction, as well as `copy_with` derivation afterwards. The selected shape is
`Text(source, *, style=None, layout=None)`; additional convenience fields and full
per-type signatures still need their capability mapping. A fresh Text has no old
instructions to preserve: omitted or None style/layout means no corresponding
local instruction, not an eagerly materialized default Theme role.

Title is a semantic Text-taking slot, not a separate Title value class or a second
style system. Accept common Text configuration keywords directly on `title` and
the other applicable Text-taking operations, including subtitle/caption and axis
labels, while continuing to accept a plain string or an already-authored Text:

```python
p.title("Results", style=title_style, layout=title_layout)
p.title(Text("Results", style=title_style, layout=title_layout))
```

These are equivalent entry forms using the same Text preparation/native slot
update path, not two sets of validation or retained style fields. Keyword handling
must not mutate an incoming immutable Text. The Plot/Canvas operation still edits
and returns its owner; creating or deriving a Text does not. Explicit keyword
precedence and clearing follow the selected rules below.

Preserve the [native text ownership/precedence contract](04_VISUAL_PROPERTY_OWNERSHIP.md):

- Font stack, size, color, weight and line height use a complete local TextTheme
  when present, otherwise the consuming owner's selected Theme role. An unstyled
  Text may therefore look different in a title and a caption; attaching it does
  not eagerly freeze the role's effective appearance or disable Theme inheritance.
  A supplied local style replaces the complete role, not a sparse field merge.
- Alignment belongs to the common TextLayout request, not a distinct title-style
  field. Native title-alignment conveniences edit that same slot request in call
  order; any Python convenience must target it too. Do not introduce another
  alignment/style state or change the native owner/Canvas/default precedence.
- Title/subtitle gaps, title-block clearance and root padding remain composition
  Theme responsibilities. Do not move them into Text or add unselected title-local
  gap/margin controls merely because common Text keywords are accepted.
- Plain content replacement and omitted local instructions preserve the slot's
  existing style/layout. Present exact style and whole layout replace their
  respective instructions. A present default TextLayout resets the layout request
  to owner defaults; it is different from omission. Do not silently turn missing
  constructor/slot/copy-with keywords into clearing or default reset.

### Omission, Clearing And Explicit Keyword Precedence

For configuration-value derivation and the optional style/layout keywords of
Text-taking owner operations, distinguish argument presence from its value:

| Argument | Value derivation | Text-taking owner operation |
| --- | --- | --- |
| Omitted | Retain that field in the source value. | Apply the incoming Text's present instruction; if it has none, preserve the slot's existing instruction. |
| Explicit None | Remove the field's optional value where absence is admitted. | Remove the corresponding local slot instruction, even if the incoming Text supplies one. |
| Concrete value | Replace that field. | Replace the complete corresponding style or layout, overriding the incoming Text. |

None is not a universal reset token. Reject it for nonnullable fields such as font
size or a required Theme component instead of inventing a default. These rules
do not redefine data-family bindings or other APIs' separately typed None meanings.

Resolve style and layout independently: explicit keywords, including None, take
precedence over incoming Text fields, without mutating that Text or merging sparse
style fields. An absent keyword does not erase a present incoming instruction.
Resolve all keyword intentions and perform the call's required input/type/conversion
checks before committing an owner edit, preserving native validation boundaries;
a keyword/conversion failure leaves the prior configuration intact.

```python
# Planned interfaces; explicit None clearing is not implemented yet.
p.title("Changed")                             # Preserve existing instructions.
p.title("Changed", style=None)                  # Clear only the local style.
p.title(Text("Changed", style=a), style=b)       # Use complete style b.
p.title(Text("Changed", style=a), style=None)    # Clear despite incoming style a.
```

Clearing style means removing its local override, not hiding text, changing its
content/layout, modifying the Theme or attaching a copy of a default style. The
owner's effective Theme role then supplies appearance normally, including after a
later Theme replacement. Clearing layout removes the local request. By contrast,
`layout=TextLayout()` supplies a present default request: it resets prior layout
instructions but remains structurally distinct from no request, including in
native equality/cache inputs. Keep that existing distinction even when both happen
to render alike. No extra public RESET object is required.

A Text value describes content and optional instructions, not an operation history:
`plain = styled.copy_with(style=None)` produces a new unstyled Text without changing
`styled`. Passing `p.title(plain)` still follows native omission/preserve rules;
only `p.title(plain, style=None)` explicitly clears the owner's retained style.
Do not add persistent provided/cleared flags to Text to propagate that command.

Use a private unique omission sentinel at the Python argument-binding boundary,
compared by identity, for schematic update signatures such as
`title(text, *, style=_UNSET, layout=_UNSET)` and optional `copy_with` fields.
Callers need not import or pass it. Recognize explicit arguments even when their
values equal a default or the current value; do not guess presence by equality.
Preserve omitted/clear/replace intent across PyO3 rather than collapse omitted and
None into one Option value. This is transient call handling, not another retained
configuration representation or a new public generic update protocol.

### Native Text-Clear Capability

The core already distinguishes omitted instructions, concrete instructions and
present default layout requests through Text/TextSlot and their existing presence
rules. It already resolves Theme inheritance. The missing capability is a public
operation that removes an existing local override, not a new default-argument or
style-resolution system.

Add narrow consuming native clear operations while preserving ordinary setting
signatures and behavior. The proposed naming shape, not final implemented methods,
is:

```text
Text::clear_style(self) -> Self
Text::clear_layout(self) -> Self
Plot/Canvas::clear_title_style(self) -> Self
Plot/Canvas::clear_title_layout(self) -> Self
```

Inventory and consistently cover the actual retained text slots, including root
title/subtitle/caption and applicable axis/guide titles, through shared core
Text/TextSlot logic and existing owner/family boundaries, not a method implementation
per plot family. Final owner-method inventory/names belong to the native API design.
Adapters expose the resulting native capabilities without another style engine.

- Clear only the requested optional field. Preserve text content, content presence,
  the other instruction field, scientific data and the Theme. Clearing an already
  absent field is harmless. Do not render or discover fonts during these edits.
- Do not reinterpret the incoming None in existing native slot replacement as a
  clear command: ordinary `.title("Changed")` must continue retaining prior local
  instructions. Do not force normal Rust setters to take Some-wrapped values or
  replace explicit clearing with a present default layout/appearance.
- Keep the mutation and validation authority in the core. Python calls deliberate
  public facilities, not crate-private helpers, and must not clear by maintaining
  its own style state or reconstructing a whole Plot/Canvas. A public general
  `Update<T>` framework is not required; any small private call-intent representation
  must preserve the same single native storage and behavior.
- `Text.copy_with(...=None)` operates on an independent native value; owner keywords
  operate on the intended slot. Neither changes other configuration copies, Canvas
  captures or frozen Outputs. Preserve the selected Python failure-safe owner
  lifecycle through the deliberate ownership facilities, not accidental consumption.

Deliver the native capability first with focused contract checks and strict Clippy,
then map it in the planned Python binding and verify private-sentinel handling,
explicit keyword precedence and native errors. This does not authorize starting a
complete binding or a separate prototype merely to update this plan.

### Canvas Captures Configuration, Not A Live Child Alias

Placing a Plot captures its configuration at that call and leaves the external
Python Plot usable. Later edits to that Plot must not change any existing Canvas
capture. Each placement preserves native parent ownership, local instructions and
one scientific producer per owner; it is not a separate Python plotting spec.

Snapshot semantics do not require an immediate whole-Plot clone. Prefer narrow
immutable-configuration sharing with separation on later edits, where deliberate
native facilities make this possible. Inserting a temporary complex plot and
discarding its external handle should not inherently require an unused independent
configuration copy. `.copy()` may likewise initially share a configuration version
while returning a distinct Python owner. This is an implementation preference,
not an already-tested copy-on-write Canvas or a zero-copy guarantee. If it needs
disproportionate core refactoring, retain the simple cloning baseline with the
same public semantics rather than introduce another representation or mode.

A Canvas grid can be created before its Plot contents are decided; empty slots do
not require placeholder Plot objects or live-reference insertion. Updating occupied
regions uses the explicit overwrite rules below. An editable panel-access handle
is not selected by that decision; do not introduce live external Plot linkage to
make updates possible.

An external Plot remains independently renderable after insertion. That does not
promise an exact export/crop of its final Canvas panel: parent theme, shared guides
and composition sizing can affect that panel's result. Such a panel-export API is
not implied by the selected lifecycle.

### Unified Spans And Explicit Region Overwrite

The selected Python placement signature is:

```text
panel(row, column, plot, *, row_span=1, column_span=1, overwrite=False)
```

Single-cell and spanning placement are the same Python operation. Do not add
Python `panel_span` or `set_panel` variants for those choices. Spans and overwrite
are keyword-only; callers need not repeatedly pass the ordinary defaults. Both
successful forms mutate and return the same Canvas and capture the incoming
Plot's configuration without consuming its Python handle. `overwrite` selects
occupied-region behavior, not mutable versus immutable configuration semantics.

- With `overwrite=False`, every cell in the requested footprint must be empty;
  any overlap fails without changing the Canvas.
- With `overwrite=True`, an empty footprint behaves like ordinary insertion.
  Otherwise, remove every existing placement whose footprint intersects the new
  footprint, then place the incoming Plot at the requested row/column/span.
- Remove each intersecting owner as a whole, even if the new footprint covers
  only part of it or intersects several different owners. Release all cells of
  each removed footprint, including cells outside the new region. Do not leave a
  partially erased Plot, orphaned occupied cells or a stale anchor. Edge-touching
  footprints without shared cells do not overlap.
- Validate positive spans, checked extents and grid bounds, and prepare the
  fallible replacement resources before committing changes. Do not grow the grid
  implicitly. Failure leaves all prior placements intact, rather than deleting
  old panels before discovering an invalid replacement.

For example, replacing only cell `(1, 1)` in a two-row layout `A A B / A A C`
removes the whole 2-by-2 A owner, resulting in `. . B / . X C`. This potentially
clears cells outside the requested new region and must be documented explicitly;
it is region overwrite, not same-anchor replacement or an exact-footprint match.
Existing configuration copies and frozen Outputs remain unaffected. A later
Canvas export performs a new whole composition render; unchanged grid footprints
do not guarantee identical pixel dimensions/positions or incremental rendering.

Implement this as a native core Canvas capability, available to Rust consumers and
through the adapters' unchanged native Canvas re-exports. Preserve the ordinary
Rust insertion parameter lists and make the uncommon destructive intent explicit:

| Planned Rust method | Footprint | Occupied-region behavior |
| --- | --- | --- |
| `panel` | One cell | Reject overlap. |
| `panel_span` | Explicit row/column spans | Reject overlap. |
| `set_panel` | One cell | Remove all intersecting owners and insert; also valid on empty space. |
| `set_panel_span` | Explicit row/column spans | The same region-overwrite rule. |

These are not new aliases or alternative renderers. They express different
placement intent without forcing Rust callers to repeat an uncommon Boolean
argument. All routes use one native footprint/ownership validation and mutation
path, with an internal overlap policy if useful; Python only maps its arguments
and errors to that capability. Do not maintain a second panel list or simulate
overwrite by rebuilding Canvas in Python. Receiver/error ownership facilities
still need the implementation work below; the new Rust setters do not exist yet.

### Rendering Does Not Consume Python Configuration

Each Plot/Canvas terminal export preserves its Python configuration owner, performs
a new render operation and returns a frozen Output on success. The configuration
remains editable, copyable and renderable again. Rendering/publication failure must
not invalidate the configuration or an already existing Output.

Two calls to `p.svg(...)` mean two render operations even without intervening
setters. There is no implicit last-render reuse based only on configuration
equality or an edit counter. Reuse is explicit through the returned Output:
`out = p.svg(first_path)` followed by `out.svg(second_path)` or `out.png(path)`
publishes the same frozen scene without new layout, statistics or numeric/tick
callback materialization. Format conversion and I/O can still have real costs.
Later edits to the configuration do not affect that Output.

This selects non-consuming Python behavior, not a new public `render()` method
or a Rust signature change. Review the smallest public core facilities needed;
do not work around consuming methods by duplicating rendering/publication in the
binding. A terminal operation uses a stable configuration version even if an
allowed reentrant callback edits the Python owner for a later operation.

### Callback References Are Not Frozen Callback Results

Configuration copies and Canvas captures retain the configured callable and its
identity; they do not recursively clone or freeze Python globals, closures or
application objects reachable from it. The same callable can produce different
labels when external formatting options change, even if no Plot setter runs.
New render operations may evaluate that callable again. Frozen Output replay
retains the already-materialized result and must not reevaluate it.

Preserve exact-tick precedence and once-per-reached-context materialization within
each operation. Complete Python-reference ownership, [GC integration][pyo3-gc],
exception context/identity, context lifetime and interpreter/thread rules before
exposing callbacks. Do not hold a mutable PyO3 borrow across a Python callback that
needs reentrant access; neither immutable sharing nor explicit copies alone solve
reference cycles, invalid contexts or nested-operation ownership.

### Current Rust Ownership And Implementation Constraints

The existing native API is not yet the selected Python lifecycle:

| Current native fact | Binding implication |
| --- | --- |
| Plot observation data and attached Theme use Arc, but Plot Clone also copies owned text, ticks and layer/annotation vectors. | Share existing immutable payloads; do not claim whole-configuration clones or separation are constant-cost. |
| Theme has complete immutable components and a construction-only ThemeBuilder; Text/component field edits consume values. | Expose Python read-only attributes/copy_with through public native value construction and validation, not matching mutable setters or a new Rust alias family. |
| TextSlot replacement preserves omitted style/layout and accepts present default-layout reset, but has no public explicit local-clear route. | Reuse that presence/resolution machinery and add narrow value/owner clear facilities; Python omission detection alone cannot clear an existing native override. |
| Canvas placement takes Plot by value and owns a boxed erased Plot, without an automatic Clone. Public Canvas is not Clone. | Rust insertion moves ownership; it is neither a borrowed child nor the proposed shared-configuration facility. Preserve Python handles through a deliberate native design. |
| Both current panel and panel_span reject overlaps; no native region-overwrite setters exist. | Implement the selected overwrite capability in the core rather than imply it is already available or make it Python-only. |
| Canvas insertion consumes self and may return Err without returning either owner. | Do not take the sole Python-visible owner and assume error recovery can restore it. Avoid cloning an entire growing Canvas on every insertion. |
| Plot/Canvas public terminal methods consume self, while their private scene-rendering functions borrow self. | Non-consuming Python renders need a lawful public route; rendering does not inherently require consuming the configuration. Private helpers are not a binding API. |
| Output is not Clone and its fallible publishers consume self, returning it only on success. | Reusable Python Output must survive publication failure without rerendering or relying on an unavailable Clone. |
| Labels already uses Arc::make_mut. | Copy-on-write can still clone the complete shared map; Arc alone is not a complexity guarantee. |

Wrapping an entire Plot in Arc does not by itself make the existing Canvas accept
it, preserve placement-local ownership or recover fallible operations. Separation
may still clone the entire shared configuration; see [Arc behavior][arc-cow].
Do not infer sole ownership by guessing Python reference counts or silently
consume external handles. Keep one native representation/validation path and core
independence; no persistent collection framework, Python builder hierarchy or
general storage redesign is authorized by the preference for sharing.

### Remaining API And Implementation Gate

Public lifecycle semantics are selected; their complete native ownership/sharing
implementation is not. Before expanding the binding surface:

1. Resolve the explicit decisions in the
   [25-family constructor inventory](92_PYTHON_CONSTRUCTORS_PLAN.md), then complete
   exact overloads/runtime combination rules, policy/method signatures, remaining
   field-key admission and slice normalization/boundaries. Preserve the four ordinary
   data routes without Rust payload classes and conventional concise type aliases.
   Keep the selected single-runtime-entry/overload model. col/cols and
   direct-projection syntax are selected, not a choice between
   selector guessing and competing *_cols keywords. Finalize native text-clear
   method names/slot coverage and exact signatures/type stubs for the selected
   omission/None/value, explicit-default and keyword-precedence rules. Mutable
   owners, immutable values, shared Text entries, binding-time snapshots and Canvas
   placement/region-overwrite behavior are selected; do not reopen them as lifecycle
   modes or live aliases.
2. Identify the smallest public core facilities for configuration sharing/copy,
   explicit Text/slot clearing, shared strict/region-overwrite Canvas placement,
   stable non-consuming renders and failure-safe Output replay. Record justified
   native API/storage changes before implementing the full binding; migrate affected
   callers rather than retain compatibility alternatives.
3. Complete bounded real-PyO3 checks for aliases/copies, insertion followed by
   external edits, retained branches, temporary insertions, fallible Canvas edits,
   render/I/O failure and callback ownership/reentry. Representative input checks
   must distinguish unchanged-source constructor/chained equivalence, mutation or
   release after binding, changed-source later lookup, source reassignment versus
   mutation, and replacement of one binding without refreshing prior roles. Verify
   these boundaries across supported input routes without a family/type option
   cross-product; a stale bridge snapshot must not defeat current-source lookup.
   Overload verification must compare representative accepted/rejected typed calls
   with actual runtime call-combination checks, including calls without static type
   checking, required alternatives, conflicts, incomplete inputs and explicit None.
   Confirm native scientific paths remain distinct without a per-container replay
   matrix; do not rely on stubs as runtime validation.
   Selector coverage must distinguish direct list/tuple data even with matching
   schema keys, exact composite single keys, ordered discrete versus half-open
   selections, one-column versus one-element collection meaning, unsupported roles
   and missing/ambiguous/duplicate/empty/out-of-range/no-source failures. Verify
   selected and direct-projection value snapshots without assuming identical
   provenance/display names or requiring a per-family replay matrix.
   Shared value/text checks must cover unchanged originals/aliases after derivation,
   rejection of direct/nested writes, native validation failure, stable explicit
   value attachments, equivalent Text/keyword entry forms, ordinary role inheritance
   and content-only replacement retaining prior instructions. Add focused checks for
   omitted versus explicit None/value arguments, keyword precedence over authored
   Text, restoration of Theme control, independent/idempotent style/layout clearing,
   and absence versus present default requests. Reuse existing default-reset tests;
   verify failed updates and prior copies/captures/Outputs remain unaffected, not a
   family-by-option replay matrix or a separate renderer.
   Native placement coverage must include strict rejection, overwrite into empty
   space, partial/multiple owner overlap, complete old-footprint cleanup and failure
   atomicity; Python checks the unified argument mapping and lifecycle, not a
   second placement algorithm. Reuse the existing Plot and successful-replay
   evidence instead of repeating it as a full suite. Retained support/tests need
   strict Clippy and the shared semantic test ceilings.
4. Compare sharing/separation with the simple baseline on equivalent successful
   workloads where that implementation changes costs: ordinary configurations,
   shared large numeric inputs, growing tick/annotation/alias content, populated
   Canvas grids and retained independent branches. Do not reduce content or compare
   many retained branches with overwriting one result. Shared-state separation
   costs and whole-Canvas growth need explicit observation, not inferred savings.
5. Separate input conversion, Python/FFI/configuration, rendering/font discovery
   and frozen publication. Report absolute time, cumulative allocations and
   independently observed peak memory on CPython 3.10 and a newer interpreter.
   Allocation totals are not RSS; no arbitrary universal speedup threshold is
   required. Select the least complex implementation meeting the agreed semantics.

This is an implementation-closure gate, not an unresolved mutable/immutable product
choice or a feature deferred indefinitely. Polars bridge and wheel/sdist
verification remain separate pending checks. Do not publish Python examples as
working interfaces before the corresponding binding and ownership facilities exist.

## Reference Sources

References establish particular capabilities/behaviors, not blanket endorsement
of every upstream design. Published metadata establishes declared compatibility,
not successful builds of this repository. Recheck the selected versions when the
corresponding implementation begins.

[cargo-workspaces]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[polars-workspace]: https://github.com/pola-rs/polars/blob/rs-0.55.2/Cargo.toml
[maturin-layout]: https://www.maturin.rs/project_layout.html
[python-layout]: https://packaging.python.org/en/latest/discussions/src-layout-vs-flat-layout/
[python-overload]: https://docs.python.org/3.10/library/typing.html#typing.overload
[pyo3-classes]: https://pyo3.rs/v0.29.3/class.html
[pyo3-parallelism]: https://pyo3.rs/v0.29.3/parallelism.html
[rust-numpy]: https://github.com/PyO3/rust-numpy/tree/v0.29.0
[pyo3-polars-types]: https://github.com/pola-rs/polars/blob/rs-0.55.2/pyo3-polars/pyo3-polars/src/types.rs
[arrow-capsules]: https://arrow.apache.org/docs/format/CDataInterface/PyCapsuleInterface.html
[bridge-dependencies]: https://crates.io/api/v1/crates/pyo3-polars/0.28.0/dependencies
[numpy-dependencies]: https://crates.io/api/v1/crates/numpy/0.29.0/dependencies
[torch-release]: https://pypi.org/pypi/torch/2.14.1/json
[tensorflow-release]: https://pypi.org/pypi/tensorflow/2.21.0/json
[polars-python-release]: https://pypi.org/pypi/polars/2.0.0/json
[numpy-one-release]: https://pypi.org/pypi/numpy/1.26.4/json
[numpy-two-release]: https://pypi.org/pypi/numpy/2.2.6/json
[maturin-distribution]: https://www.maturin.rs/distribution.html
[mdbook-includes]: https://rust-lang.github.io/mdBook/format/mdbook.html
[matplotlib-gallery]: https://matplotlib.org/stable/gallery/index.html
[sphinx-gallery]: https://sphinx-gallery.github.io/stable/configuration.html
[mdbook-test]: https://rust-lang.github.io/mdBook/cli/test.html
[plotly-updates]: https://plotly.com/python/creating-and-updating-figures/#Chaining-Figure-Operations
[altair-properties]: https://github.com/vega/altair/blob/v5.5.0/altair/vegalite/v5/api.py
[arc-cow]: https://doc.rust-lang.org/std/sync/struct.Arc.html#method.make_mut
[pyo3-gc]: https://pyo3.rs/v0.29.3/class/protocols.html#garbage-collector-integration
[seaborn-keywords]: https://seaborn.pydata.org/whatsnew/v0.12.0.html#keyword-only-arguments
[plotly-inputs]: https://plotly.com/python/px-arguments/
[flutter-theme-copy]: https://api.flutter.dev/flutter/material/ThemeData/copyWith.html
[flutter-text-copy]: https://api.flutter.dev/flutter/painting/TextStyle/copyWith.html
