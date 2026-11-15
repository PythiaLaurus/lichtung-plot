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
rebuilt peer guides have passed. Python package code now exists; its bounded
verification and remaining delivery gates are recorded below. Package publication,
site deployment, Git push and history rewriting are not authorized by this work.

Python architecture, the four input routes, constructor/binding/explicit-selector
rules, one-runtime-entry/overload-signature model, explicit row-position/column-name
modes and numeric discovery, flat named geometry arguments, strict signed positional
selection, exact pandas column keys, MaskedArray validity, immutable configuration
values, text-entry/omission/clear/precedence rules and object-lifecycle semantics
below are settled implementation directions, supported by bounded prototypes and
explicit API discussion. The current package implements only the subset recorded
below; native configuration sharing is delivered and the Python subset has bounded
installed verification, not complete interface delivery.
The [Python constructor inventory](92_PYTHON_CONSTRUCTORS_PLAN.md) maps all 25
families and records the selected call forms. The
[scientific/numeric interface plan](94_PYTHON_SCIENTIFIC_INTERFACE_PLAN.md) records
the complete scientific interface contract and its remaining alpha/layer/result
work. The
[columnar geometry refactor](93_COLUMNAR_GEOMETRY_PLAN.md) specifies the prerequisite
core/adapter migration. Its flat core inputs, coupled consumers and adapter column
extraction, native Text/slot clearing and reusable native ownership facilities are
delivered. All 25 Python constructors, their alternatives and exact-family typing,
base data/facet/configuration methods, complete Theme/Text and numeric policies,
13 scientific/appearance/position values and eight callback slots are implemented.
Alpha scopes/mappers, independent layers/results and final distribution/content/
cost closure still need implementation and verification.
The three input-boundary choices below are settled, not outstanding product decisions.

### Delivery Status

| Objective | Current status |
| --- | --- |
| Mechanical workspace and policy migration | Delivered; workspace verification, representative example execution and standalone Cargo package verification passed. |
| Peer-language Book/example/gallery foundation | Delivered: one gallery-first tutorial page per language, six scientific showcase workflows, two numeric/text reference workflows, same-page image-to-code walkthroughs, shared runnable-source includes and zero-argument examples with optional output redirection. The helper lives under docs/; full per-family gallery coverage remains for integrated content closure. |
| ndarray and Polars adapters | Delivered: all 25 families, source/direct views and fields, wide/grid origins, native configuration, independent layers, statuses, alpha and significance results. Cross-adapter native handoffs, focused/workspace checks, joint offline archives and peer guides passed; the [current contract](09_DATA_ADAPTERS.md) owns the API. |
| Columnar distribution geometry prerequisite | Delivered: flat core Histogram/Raincloud geometry and sample-owned external KDE, direct/multiple-source adapter column extraction, raw-key membership/original-point-row alpha and primary-only retained lookup. Focused core/adapter checks, the cross-adapter consumer, strict Clippy, API/Book checks and all three independently unpacked archives passed. The [refactor plan](93_COLUMNAR_GEOMETRY_PLAN.md) owns fields and evidence. |
| Native Text/slot clearing | Delivered: independent style/layout removal on Text, Plot/Canvas retained slots, PointLayer legend labels and NumericExplanation; both adapters forward applicable operations. Presence/default-reset/omission semantics, value independence and current Theme/Canvas inheritance passed focused tests and strict Clippy; peer guides are synchronized. |
| Native Canvas region overwrite | Delivered: set_panel/set_panel_span share strict insertion's native footprint/occupancy authority, remove complete intersecting Plot/TextPanel owners and prepare validation/resources before mutation. Focused topology/atomicity, composed rendering, doctest, strict Clippy and peer Book checks passed. Borrowed failure-safe placement now supplies reusable receiver ownership too. |
| Native reusable ownership | Delivered: borrowed Plot/Canvas/adapter exports, shared frozen Output handles with failure-safe borrowed publication, Canvas Clone with shared placements/Plot payloads, and borrowed placement primitives used by the consuming builders. Plot Clone shares the authored description's payload; Plot exports borrow that description directly, Canvas placement moves it, and one operation-local Root/occurrence registry supplies measurement factories without temporary working Plots. Recorded native placement, render/replay/failure, composed/shared-guide and cross-adapter checks passed. This is not a zero-copy/constant-cost guarantee or Python lifecycle evidence. |
| Native lifecycle read access | Implemented with actual Python consumers: canonical Canvas plot-anchor membership and TextLayout/TextOffset/TextBounds readers support callback-root cleanup and immutable derivation. Three focused native checks and bounded installed consumer checks passed. |
| Source-free adapter metadata | Delivered: each adapter splits/restores its exact-family native Plot and opaque binding metadata without retaining lookup. Native checks and independent archives passed; Python Scatter now preserves the matching native/origin version through copies, policy edits and stale-safe input commits. |
| Bound-only adapter projections | Implemented and consumed by Python: ndarray preserves original coordinates and stable raw identities; Polars carries complete names with selected or direct-local Columns and shared numeric discovery. Focused native naming/shape/order checks, external consumers and installed Scatter input checks passed. |
| Python lifecycle decision and complete binding | All 25 constructors, exact-family base configuration, complete Canvas configuration and eight callback slots have bounded installed verification below. Alpha, layers/results and final distribution/content/cost checks remain open; this is not complete binding delivery. |
| Python input boundaries | Strict signed positional selection, exact complete pandas keys and direct Series identities, MaskedArray validity, bound-only eager Polars Arrow import, fresh source lookup and failure-safe paired native/key-root commits are implemented. Shared transport has the earlier Scatter matrix/independent-sdist evidence; current all-family constructors, geometry and edits have separate focused wheel evidence below, not a repeated transport matrix or current sdist certification. |
| Python Theme/Text values | Implemented: all eight Theme components and thirteen TextLayout fields, 54 Theme descendants and nine Text value additions, with 28 consumer-required native readers. Six family appearance policies and two opaque text-position values are now delivered too; focused installed evidence is separated below. |
| Python owning numeric values | Implemented: twelve native-only policy additions, complete NumericFormat inputs/compiled specifiers and TickGeneration locators, shared TextIntent results, full-family forwarding and all eight owner attachment slots. Focused installed checks are recorded below. |
| Integrated content/package/site closure | Rust adapter examples, peer guides/site and all three source archives are verified. Bounded Python wheel/unpacked-sdist verification passed; complete per-family gallery and broader platform/distribution coverage remain open. No publication or deployment was performed. |

### Current Python Implementation Scope

`bindings/python/` is the sole binding package. The following implemented subset
has the bounded installed verification described below:

- All 25 exact families accept their selected native/NumPy/pandas/eager-Polars
  forms. Constructors include supplied estimates, wide/grid and discovery modes,
  and complete flat Histogram/Raincloud geometry. Shared selectors, exact pandas
  keys, direct Series names and validity use owned binding-time snapshots.
  Retained main sources supply fresh role lookup without refreshing old bindings;
  auxiliary geometry contexts remain independent and call-local.
  Mutable Plot/Canvas owners support aliases, independent copies, insertion-time
  captures, keyword-only spans/overwrite and failure-safe configuration edits.
- All five native terminal exports and frozen Output reuse use the same core path.
  NumericFormat value/set callbacks, TickGeneration function callbacks, scoped
  contexts, reentry/exception ownership and GC hooks are covered by those bounded
  installed checks.
- Eighty-four immutable appearance/text/PNG classes cover the original 21 plus
  54 Theme descendants and nine Text values. Theme exposes all eight components;
  TextLayout exposes all thirteen authored fields, including fit, orientation,
  anchor, overflow and visibility. Native values remain the sole representation;
  immutable copies preserve omission, real None and closed-branch presence.
  The 28 pure Text readers are delivered with their actual Python consumers.
- Immutable Labels exposes mode/aliases/copy/resolve and complete Plot/Canvas
  replacement. It holds the native policy; source/key roots are physical GC edges,
  not a second Labels store. Native `get_labels`/mode/aliases readers serve these
  actual consumers.
- Twelve owning numeric values complete precision/field, locale, prefix/factor,
  unit and explanation configuration. NumericFormat exposes all eighteen policy
  inputs, native specifier compilation and seven existing native readers.
  TickGeneration includes interval, multiplicative and log-decade factories,
  singleton/cap derivation and existing native parameter readers. Old
  fractional_digits/field_width shorthand is removed and callers migrated, not
  retained as a second API. All eight physical slots are connected: x/y formats,
  continuous color, Heatmap values, Coefficient estimate labels, Waterfall totals
  and x/y tick generation. Hidden Heatmap values retire only their own roots.
- Base data edits, all 18 facet entries, native pure configuration, guides,
  annotations and complete Canvas configuration forward through exact
  capabilities. Thirteen additional frozen values hold five statistical policies,
  six appearance policies and two opaque text-position values. Eight pure native
  readers and exact Raincloud hue-title methods are delivered with these consumers.
  Matrix and generic facets keep the native standalone-publication restriction.
- The mixed package uses PyO3 =0.29.2 with abi3-py310, rust-numpy =0.29.0 and the
  existing ndarray/Polars adapters. The core remains the independent default
  workspace member. Eager Python Polars/runtime-32 =1.41.2 exports public Arrow
  C streams into polars-core =0.55.2 without PyArrow or a Rust ABI cast.
  One actual native stub, `bindings/python/python/lichtung_plot/_native.pyi`, public
  owner annotations and `bindings/python/python/lichtung_plot/py.typed` describe
  the implemented surface, including nominal four-route input roles and
  constructor-mode overloads and exact capability-gated methods for all 25 families.

Source owners are `bindings/python/src/input.rs`, `bindings/python/src/owner/`,
`bindings/python/src/output.rs`, `bindings/python/src/values/` and
`bindings/python/src/callbacks/`; the thin public owners live in
`bindings/python/python/lichtung_plot/_owners.py`. The earlier input-foundation
installed runtime/callback/GC checks used one identical wheel/native binary on
sampled Linux x86_64 ordinary CPython environments, before the Theme/Text additions:

| Interpreter | Input providers | Actual runtime checks |
| --- | --- | --- |
| 3.10.19 | NumPy 1.26.4, pandas 2.3.3, Polars/runtime-32 1.41.2 | 127 passes plus one focused finalizer-fixture retry; not a fresh full 128-case rerun. |
| 3.10.19 | NumPy 2.2.6, pandas 2.3.3, Polars/runtime-32 1.41.2 | 128 passes. |
| 3.13.11 | NumPy 2.2.6, pandas 3.1.1, Polars/runtime-32 1.41.2 | 128 passes. |

Installed strict mypy checked five explicit shipped-module/fixture roots with all
50 negative guards: Python 3.10 language targets for both 3.10 providers and 3.13
for the 3.13 provider. The actual pandas source-follow configuration, without
pandas-stubs or substitute declarations, verifies nominal roles only in those
providers. Recursive single-root stubtest passed for all three shipped modules
without an allowlist. Dependency checks passed and original third-party pins
remained unchanged. The four-route example ran outside the checkout; its actual
PNG was inspected.

The PEP 517 source producer is `bindings/python/_build_backend.py`; its derived-lock
policy is defined in `specs/90_DEVELOPMENT.md`. A newly produced expanded adapter/
NumPy/Arrow sdist was independently unpacked and built with the installed Rust
1.96 toolchain, locked/offline release settings and a separate target directory.
Its own cp310-abi3 wheel was installed in a fresh fourth CPython 3.10 prefix:
all 128 archived runtime cases, dependency checks and original third-party pin
checks passed. All 498 unpacked archived files remained byte-unchanged through
the build, including manifests and derived lock. The retained lock packages kept
their pins/fields and dependency-edge subsets; only the development Book member
was pruned. This is new expanded-closure evidence, not reuse of the earlier
native-data sdist's 38 passing cases. That archive predates the Theme/Text
implementation and is not source-package execution evidence for those additions.

For the earlier input dependency change, the full Rust gate passed on its repaired
product source: 1,075 named cases, 15 ignored/not executed, and 144 doctests; strict
workspace all-target Clippy, API/Book checks and all three native archives passed.
Only the finalizer fixture changed afterward: it records type/message rather than
retaining a traceback that keeps its own self alive. Product sources and the wheel
binary did not change. Historical compiler, loader, cold-import, runtime-fixture
and invocation failures remain recorded, not reclassified as successful commands.
In particular, the earlier native-data full-command failure was not rerun or
retroactively changed; that dependency-closure gate is separate evidence. These
full-suite counts are not a rerun on the current Theme/Text source.

For the Theme/Text additions, strict workspace all-target Clippy and the one
affected private native layout-copy case passed. A release cp310-abi3 wheel passed
the focused new/affected 17-case selection on CPython 3.13; CPython 3.10 passed
16 cases plus one exact title-anchor fixture retry, not a new full selection.
Installed strict mypy checked six explicit shipped/fixture roots against the 3.10
provider and language target, retaining 50 input, eight lifecycle and 28 new value
negative guards. This is declaration evidence for that provider, not a repeat of
the entire input runtime matrix.

Recursive stubtest initially failed because 13 actual None defaults were displayed
as Ellipsis by macro-generated signature metadata. Only the macro's default-token
capture changed; a newly built and installed wheel passed recursive single-root
stubtest without an allowlist. Both 3.10 and 3.13 passed the two affected constructor
cases, and inspection confirmed all 13 signature defaults are None. The new
wheel's shipped declarations are byte-identical to those checked by mypy.
The successful 17-case evidence remains tied to the preceding wheel; it was not
repeated or relabeled as a new-wheel full selection. The original metadata and
title-fixture failures remain recorded as failures.

The complete-appearance example ran outside the checkout on the repaired wheel;
its actual PNG was inspected. Installed dependency checks passed without changing
third-party pins. Final full-interface source-package build/install/runtime and
cost/content closure remain open; earlier archives do not certify later additions.

For the owning numeric closure, strict workspace all-target Clippy and one exact
private compiled-policy/emitter-identity case passed. The new release cp310-abi3
wheel passed 15 focused runtime cases on both CPython 3.10 and 3.13: nine new
drivers and six existing affected cases, including both parameterized replacement
finalizer branches. After review-driven test-only changes, the two affected
Explanation/singleton cases passed focused retries on both interpreters; the
successful 15-case selections were not repeated. The only later Rust change was
inside cfg(test); all preceding production bytes remained identical to the wheel
source, and strict Clippy/the one native case passed again.
Installed strict mypy checked seven explicit shipped/fixture
roots on the 3.10 provider/language target with all 107 targeted negative guards;
recursive single-root stubtest passed for all three shipped modules without an
allowlist. No full Rust suite or input runtime matrix was repeated.
The numeric example ran outside the checkout and its actual PNG was inspected;
dependency checks preserved the original provider pins. Initial Clippy warnings
and the separate prelaunch selection-count assertion remain recorded, not
reclassified as successful invocations. That evidence predates the family/eight-slot
closure below; it does not certify later distribution/content/cost or budget work.

For the exact-family closure, strict locked/offline workspace all-target Clippy
passed. Four new private Raincloud title/cache/census/error cases, one new public
case containing two publications and one affected existing shared-guide case
passed. No full Rust suite was repeated. The release cp310-abi3 family wheel3
selected 36 runtime cases on each of CPython 3.10.19 (NumPy 1.26.4/pandas 2.3.3)
and 3.13.11 (NumPy 2.2.6/pandas 3.1.1), both with Polars/runtime-32 1.41.2:
each had 34 passes and two failures. The two new fixtures incorrectly required
unabridged default Heatmap labels and combined Interaction styles with hidden
connectors; native rendering and error classification were not changed.

After those fixture repairs, a new family wheel4 passed exactly the two affected
cases on both interpreters. The earlier 34 passes remain wheel3 evidence, not a
wheel4 full-36 rerun. The only intervening Rust production edit changed a scalar
macro's default-token capture, fixing three constructor signature defaults;
the Python production edit represented six equivalent Literal aliases as unions.
Core, adapters, owner, input and callback Rust sources were byte-identical.
Strict Clippy and release build passed again; the unchanged six native cases
were not repeated. Both installed prefixes matched wheel4/native identities:
SHA-256 `70eff2a55c9c6d066f2fc96bbbe9b182929d1f8b7e9819d4dc513f04935cd6b3`
and `42a77df443a7a48ffad4177c39a920e65df5282fba9f59c4d194ea73ac1f57c2`.

Installed strict mypy passed eight explicit roots (three shipped modules and five
fixtures) with all 152 negative guards, targeting the 3.10 provider/language.
Recursive single-root stubtest passed all three shipped modules without an
allowlist. Actual default signatures and values passed on both interpreters.
Dependency checks passed and explicit before/after comparisons preserved all 17
third-party pins in each prefix. The four-panel scientific example ran outside
the checkout; its actual PNG was inspected for complete titles, axes/units,
unavailable status, Heatmap values/highlights and the synthetic-data caveat.
The initial compiler/lint, typing, metadata and fixture failures remain recorded.

These are focused family/owner checks, not a new all-container matrix. The third
3.10/NumPy-2 prefix remains on the earlier input wheel. There is no independently
unpacked/built/installed family sdist evidence yet; earlier input archives and the
numeric archive inventory do not certify this source closure. Alpha scopes and
foreign-loan-safe mappers, independent layers/results, final distribution/content/
cost and budget certification remain open.

This verifies sampled ordinary Linux CPython builds, not manylinux, multiplatform,
PyPy, free-threaded CPython, subinterpreters or native-worker-thread callbacks.
Repository-wide test-budget review identified excess module-integration coverage;
the limits have not been certified. Test deduplication remains a final closure
task after the complete planned interface is implemented, rather than blocking
the remaining implementation stages. Overall plan acceptance is not complete;
the complete Python binding is not delivered.

Alpha scopes/mappers and independent layers/results still need implementation,
their exact typing and real consumers, followed by final distribution/content/cost
observations. Constructor closure is not complete-interface closure.
The 2026-11-06 prototypes below are not runs of this package.

## Confirmed Scope And Constraints

| Concern | Required outcome |
| --- | --- |
| Existing adapter/binding code | Replace the existing adapters' conversion-only public design, reusing sound private conversion support; Python's bounded verification and remaining delivery gates are recorded above. No repository/history import is requested. |
| Renderer boundary | One independent `lichtung-plot` core, with adapters and bindings depending on it, never the reverse. |
| Adapter public entry points | Either adapter exposes every native public item/module at its original relative path and with the same definition/signatures. Foreign input uses top-level `Family::new` and `Family::from_data` with non-conflicting helper names, never native-name shadowing or a mandatory binding namespace. No separate direct core dependency is needed; both adapters can coexist and share native types through one compatible resolved core package. |
| Adapter foreign-container boundary | ndarray accepts its own views/validity/column selections; Polars accepts eager DataFrame/Series/Column and field selections. Preserve existing core-value pass-through and native re-exports. No cross-adapter foreign-container conversion or dependency; Python owns four-source unification. |
| Python coverage | Equivalent scientific/rendering capabilities across all 25 plot families, composition, themes, text adaptation and numeric/tick callbacks. Translate Rust scientific payload inputs into the four Python data routes, not a literal re-export of data classes. |
| Python input | Exactly four data-source families: Python native data, NumPy arrays, pandas DataFrame/Series and eager Polars DataFrame/Series. Convert internally to core values; do not expose NumericValues/CategoryValues/AxisValues, Bounds, closed source inputs or geometry payload classes as another public data-input route. |
| Python construction | For source-plus-variable calls, data is the only positional argument; variables use keywords. Common optional bindings such as Scatter hue also have chainable setters using the same input/native validation path. Support mixed selectors/direct values, not type-dependent positional reinterpretation. |
| Python overloads | One runtime callable per family checks explicit scientific keyword combinations and selects native paths. Shipped typing.overload signatures describe distinct legal call forms for editors/type checkers; they do not perform runtime dispatch or validation. |
| Python typing and constructor inventory | Use concise ordinary aliases for supported containers/selectors and meaningful overloads, not exhaustive scalar-width signatures or untyped kwargs. All 25 constructor/base-owner declarations, including explicit modes, discovery and named geometry, are delivered; exact alpha/layer/result declarations remain to be completed. |
| Python explicit modes and discovery | by_row_position=True on Scatter/Line generates zero-based row x and excludes explicit x; by_column_name=True on Boxplot/Violin/sample Raincloud makes each column a category and excludes explicit category. numeric_only=True explicitly discovers supported numeric columns in source order for those modes and Matrix, including numeric IDs; it excludes manual y/value/variables. No omission/schema/value-based inference. |
| Supplied distribution geometry | Flat named Histogram bins/KDE/rug and Raincloud summary/density/point columns, not nested payloads, geometry maps/TypedDict protocols or required wrappers. Keep native sample/geometry exclusivity and complete-geometry restrictions. Bind independent component sources and preserve raw-key membership/original point rows; labels are display text, not scientific keys. |
| Python column selection | Use col[key] for one exact field and cols[...] for an ordered column collection or signed-step positional slice. Slices address logical column positions on NumPy, pandas and eager Polars tables; explicit out-of-bounds endpoints fail rather than clip. NumPy positional keys support negative indices, while pandas integer keys remain literal labels. Bare list/tuple inputs remain direct data; retain native projections, without parallel selector aliases or schema guessing. |
| Python pandas column keys | Accept legal hashable complete labels that resolve to exactly one actual column, not a scalar-type allowlist. Keep original lookup keys, collision-free native identity and display text distinct; no partial matching, query execution or str(key)-based identity. Column-label admission does not broaden supported data dtypes. |
| Python MaskedArray | Support within the NumPy route by snapshotting bound data and mask into existing native validity. Preserve aligned slots, shape, logical order and unmasked NaN; no compression, filling or silent mask loss. Native roles still decide whether missing values are applicable. |
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

Each adapter continues to accept only its corresponding external containers,
alongside its existing native-value pass-through. Polars does not gain ndarray
inputs and ndarray does not gain Polars inputs. The binding's four Python data
routes are not a reason to turn either Rust adapter into a universal converter.

Preserve ordinary f64 rounding, null-versus-NaN distinctions, logical row order,
category identity and explicit provenance. The core remains independent; adapters
reuse its scientific and rendering behavior. Migrate callers and remove the old
public conversion surface with the working replacement, without aliases or a
parallel legacy route. Replace Polars first to establish dataset/name bindings,
then ndarray's vector/table/matrix routes. Validate actual code, review counts
when changing tests, and verify working packages at delivery, not through a
pre-implementation audit or compatibility project. Python retains its own
implementation and verification gate below.

### Columnar Geometry Prerequisite

The adapters now bind complete geometry through ordinary direct/selected columns,
replacing their former native-payload frontend protocol as specified by the
[columnar geometry refactor](93_COLUMNAR_GEOMETRY_PLAN.md):

- The core now keeps top-level HistogramGeometry/RaincloudGeometry and the closed
  sample/geometry choice with flat public fields. Nested Dimensions/Group/Bin/Point
  types and their callers are removed; sample-owned precomputed KDE uses named x/y
  columns too, with no retained public point-type route.
- Both corresponding Rust adapters provide direct/multiple-source column binding.
  Histogram bins/KDE/rugs and Raincloud summary/density/points each have their own
  alignment domain. Only the main source remains for later lookup; auxiliary
  sources are used during the binding call, never fallback setter contexts.
- Core preparation matches actual category/hue strings, uses summary order for
  groups and retains relative child order plus original point rows for observation
  alpha. Keep scientific validation native and do not fill missing components.
- Preserve scalar scientific dimension metadata separately from authored display
  labels. Python uses the selected `*_label` text conveniences, not new public
  `*_dimension`/`*_name` keys, Rust data classes or a second binding-side label store.

This native migration precedes Python supplied-geometry bindings. It is not an
extension of accepted Rust foreign-container types or evidence of a Python bridge.

### Source-Free Metadata And Constructor Projections

Source-free metadata round-trip and constructor projections below are implemented
native adapter APIs, with separate acceptance boundaries. They support the selected
Python input routes without retaining borrowed Rust adapters or duplicating their
origin/shape algorithms. The core and the Python
public data-input surface do not change.

#### Source-Free Exact-Family Metadata

Each adapter exports its own opaque `BindingMetadata<K: kind::Marker>` at its
root. Move the existing seven metadata fields into that single owner:
`len`, `geometry_points`, `labels`, `facet`, `grid`, `expansion`, `matrix_keys`,
plus `PhantomData<K>`. `PlotAdapter` retains only its native Plot, its existing
source context and this metadata. Do not keep parallel copies of the old fields.
Metadata contains no Plot, source, foreign reference, schema or callback registry.
Actual family instances must naturally support Clone + Send + Sync + 'static,
without unsafe auto-trait implementations or new bounds on ordinary sources.

Both adapters provide:

```rust
impl<'source, K: kind::Marker> PlotAdapter<'source, K> {
    pub fn into_parts(self) -> (Plot<K>, BindingMetadata<K>);
}

impl<K: kind::Marker> PlotAdapter<'static, K> {
    pub fn from_parts(plot: Plot<K>, metadata: BindingMetadata<K>) -> Self;
}
```

ndarray additionally restores lookup through its existing validated context:

```rust
impl<K: kind::Marker> PlotAdapter<'static, K> {
    pub fn from_parts_with_data<'source>(
        plot: Plot<K>,
        metadata: BindingMetadata<K>,
        source: impl ArraySource + 'source,
    ) -> std::result::Result<PlotAdapter<'source, K>, AdapterError>;
}
```

Polars additionally restores its existing borrowed DataFrame context:

```rust
impl<K: kind::Marker> PlotAdapter<'static, K> {
    pub fn from_parts_with_data<'source>(
        plot: Plot<K>,
        metadata: BindingMetadata<K>,
        source: &'source polars_core::prelude::DataFrame,
    ) -> PlotAdapter<'source, K>;
}
```

Splitting/restoring moves existing state without scientific preflight or a
same-version validator. ndarray restoration retains the real `context(source)?`
structural validation; Polars must not invent a fallible operation. Restoration
does not require a new source height to equal the old plot's slots: subsequent
bindings retain their existing domain-specific checks. Existing `.into()` and
`erase()` remain deliberately metadata-discarding native handoffs.

The family marker prevents cross-family pairing, not mismatched versions of the
same family. Python must commit each native Plot/metadata candidate pair together.
Its stable configuration version owns this source-free pair; source objects and
callback holders remain separate, physically owned Python edges. Do not retain
an ndarray adapter containing Rc merely because its source currently equals None.
Later label/facet edits still go through the restored adapter so their existing
caches and native configuration remain synchronized.

Bounded acceptance covered the seven existing metadata responsibilities:
labels/facet replacement, expanded Line roles/bands, grid shape/origins, Matrix
keys and original geometry-point alpha alignment, plus source release,
source-free/restored lookup, actual-family auto-traits and cross-family
compile-fail boundaries. The external consumer restores both adapters before
native Canvas publication. These checks do not establish Python input support
or a new scientific-validation boundary.

#### ndarray Original-Coordinate Projection

ndarray exports one `ColumnProjection` from a private production module:

```rust
pub struct ColumnProjection<'columns> { /* private */ }

impl<'columns> ColumnProjection<'columns> {
    pub fn new(
        source_shape: (usize, usize),
        columns: &'columns [(usize, &'columns dyn ArraySource)],
    ) -> std::result::Result<Self, AdapterError>;

    pub fn with_identities(
        self,
        identities: &'columns [(usize, &'columns str)],
    ) -> std::result::Result<Self, AdapterError>;
}
```

Implement the existing sealed source protocol and `ArraySource`, not another
public trait or role-payload enum. The shape is the original rows/logical-column
count. Entries associate original logical indices with admitted one-column
two-dimensional views or existing values/present view pairs; different entries
may have different dtypes, strides and validity. The descriptor borrows columns
only for a binding call and does not own or copy observations.

Validate only descriptor structure: nonempty unique in-bounds indices, original
row counts, one-column shapes and the existing validity-shape checks. `shape()`
returns the original shape; each lookup delegates to physical column zero while
preserving the original index in identities, axis names and diagnostics.
Unsupported-element/source-row errors must remap physical zero to that index.
Use private sealed per-column axis classification and one existing role extraction,
not trial numeric/category conversion or a second scan.

In-bounds but unprojected fields fail explicitly. `..` still means all original
logical columns, not all supplied entries. Existing scalar/ordered selections,
row repetition and cell-shape checks remain authoritative. For example a Line
using x=4 and ys=[7,2] from an original ten-column table must keep that order and
those origins after importing only columns 4,7,2. A later hue lookup snapshots
only the current source's selected hue and restores the same metadata; it does
not refresh x/y or convert hue to native slots to bypass repetition.

Ordinary NumPy views need not use this descriptor. Concrete consumers are
per-column pandas conversion and selected MaskedArray data/validity snapshots;
mask inversion and pandas key/native-identity/display mapping remain binding
responsibilities. Test heterogeneous/independent-stride columns, original bounds,
missing projection entries, one-column collection meaning, null versus present
NaN and original-index diagnostics without a container/family cross-product.

`with_identities` supplies optional call-local raw field identities, needed when
a pandas key moves to a different logical position. A display alias cannot make
two positional `Column N` identities equal. Keep the actual source shape and
indices: the table is not a codec index space, selection list or value store.
Without this opt-in, ordinary views and projections keep the existing positional
name algorithm unchanged.

The borrowed table must contain unique in-bounds indices and unique exact raw
strings, covering every supplied value column. It may additionally cover
in-bounds unprojected columns: generated-column alpha can read only an integer
key variable whose values name other columns without reading those target
columns' observations. Such identity-only entries do not grant value lookup.
An explicit table completely replaces any prior table; missing identity entries
never fall back to positional names. Empty/whitespace raw strings are not
rejected here by a new native-name validator.

Validate table structure when supplied, without scanning observations. Resolve
integer targets only after the existing exact-integer role extraction; in this
explicit mode an out-of-bounds target is a column error and an in-bounds missing
identity is a binding error with its original index. Do not apply those new
descriptor checks to ordinary positional sources. Value lookup still checks the
real bounds and supplied value entries, and `..` still covers the original
logical domain. Preserve original-index diagnostics when delegating physical
column zero. Error text containing a native raw name may reflect the authored
identity; it must not pretend to remain a positional name.

Use one private borrowed metadata hook on the existing sealed protocol and one
source identity helper. Route scalar Numeric/Category/Boolean, both Axis branches,
alpha-byte names, non-local wide keys, generated-column integer targets and
Matrix keys through that same mapping. Do not change direct-local private keys,
grid positional keys, SourceRows values, order/repetition, masks or native-slot
pass-through. Retain single role extraction and existing native validation
boundaries, including checks already performed by native value constructors.

The four wide producers continue to choose their generated field names. Avoid
the existing other field and all supplied raw identities, including identity-only
entries, by extending the existing `Variable`/underscore algorithm. Without an
explicit identity table the result stays unchanged. Store only the already-owned
bound raw names/Expansion keys in metadata, never this borrowed table. Later
lookups interpret indices in the current source's coordinates, not frozen
constructor positions; independent-source operations use their explicit source,
not a hidden fallback to the main source.

Acceptance must additionally witness reordered stable identities, a fresh-role
binding that does not reread old observations, identity-only generated alpha
targets with native membership/coverage validation, shared scalar/axis/byte
naming, Matrix raw keys and unchanged grid/provenance authority. These Rust
witnesses are not pandas key-codec or FFI execution evidence.
The generated-field witness must include an unselected identity-only
`Variable*` entry, proving whole-table avoidance. With an identity-bearing
context, direct-local matrices and local column-alpha targets must still use
their existing private ordinals and positional display defaults.

#### Polars Names-Aware Selected-Column Projection

Polars exports its distinct descriptor from a private production module:

```rust
pub struct ColumnProjection<'columns> { /* private */ }

impl<'columns> ColumnProjection<'columns> {
    pub fn new(
        source_rows: usize,
        source_names: &'columns [&'columns str],
        columns: &'columns [polars_core::prelude::Column],
    ) -> std::result::Result<Self, AdapterError>;

    pub fn numeric_names(
        source: &polars_core::prelude::DataFrame,
    ) -> Vec<&str>;
}
```

Carry the complete ordered raw names namespace separately from bound-only native
Columns. Do not require a complete Rust SchemaRef: unrelated Decimal/Struct/Array/
Object/Extension fields may not be representable under current Polars features.
Do not enable those features, import their values, remove their names, fabricate
Null/Unknown/physical dtypes or reject them merely to construct the descriptor.

Construction checks only nonempty selections, unique namespace/selected names,
selected membership and original rows. Empty raw names remain legal. Imported
bound logical dtype fidelity belongs to the real bridge; existing role extractors
still decide scientific admission. Add only the existing sealed implementations
for `&ColumnProjection`: `ColumnsInput`, `NumericSeriesInput`, `BooleanInput` and
`SourceRowsInput`. Their fixed selected/non-native-slot origins preserve ordered
collection meaning, row-major cells and per-dimension grid checks. A 2x3 grid
must still reject 3x2 highlights/IDs despite equal products.

A private names-only scratch hook on the existing sealed protocol supplies the
complete namespace before an input is moved. The four existing wide producers
reuse their current Expansion naming algorithm against the union of main
DataFrame names and projection names. Scratch expires after construction; no
namespace/schema enters BindingMetadata or Plot. Binding code does not choose
generated field names or install a projection as the retained lookup source.

Correct the existing `multiple` helper narrowly: substitute `Column N` for an
empty name only when `local && empty`. Selected and projected `["", "Column 1"]`
must retain two distinct raw identities; direct-local ordinal/display defaults
remain unchanged. Keep current length/dtype/error ordering and native required
name/display-collision validation. Compare corrected native inputs, not updated
goldens that accept incorrectly merged groups.

`numeric_names` delegates to the existing discovery helper, sharing
`DataType::is_primitive_numeric()` with existing constructors. Python first maps
its supported concrete primitive logical dtype variants to actual native
DataTypes and calls that predicate to select candidate imports. Do not use dtype
name strings, physical storage, a generic `is_numeric()` including Decimal,
sample values or field-name heuristics. Cover all supported concrete primitive
variants; retain typed empty/all-null numeric columns and numeric IDs in source
order. Boolean, Null, temporal, nested and custom/Extension types do not become
primitive candidates through their storage.

After importing only candidates, call the shared native discovery reader and
construct the selected projection with the original complete names. Metadata
inconsistency in imported bound dtypes must fail visibly rather than silently
disappear through filtering. Explicit selections bypass discovery entirely.
Test explicit [a,b] versus discovered [b,a], unused Decimal/Struct Variable* name
avoidance, namespace union, empty selected names, zero rows and one-column
collections. The shared bound-only Arrow importer has the earlier Scatter
transport evidence and subsequent focused all-family binding evidence recorded
above. These Rust descriptor APIs and checks alone are not FFI evidence.

##### Direct-Local Namespace Transport

The input bridge consumes `from_direct` on the same descriptor to retain
direct-local identity while carrying complete main-source names. Ordinary column
references cannot supply those names; `new(...)` deliberately supplies selected
raw identities instead:

```rust
pub fn from_direct(
    source_names: &'columns [&'columns str],
    columns: &'columns [polars_core::prelude::Column],
) -> std::result::Result<Self, AdapterError>;
```

It borrows names/columns and checks only nonempty columns and unique namespace
names. Empty namespaces, zero rows and duplicate/empty direct column names remain
legal; there is no membership requirement or factory-time dtype/row-length scan.
A private selected/local mode controls both `ColumnsInput::local_identities` and
the existing `multiple(..., local, ...)`; native-slot status stays false. Direct
`cell_shape` calls the existing `columns_shape` at consumption, not construction.
Do not use the main source's rows for independent direct values.

Keep `new(...)`, existing direct inputs, the four wide producers and their
ordering unchanged. The numeric producer still checks slot count, then each
column's length and dtype/extraction in order; do not prescan all lengths.
Boolean/IDs retain row-major order, per-dimension checks and no repetition.
Namespaces remain call-local naming scratch, not lookup or metadata state.

The bridge must retain actual input origin before conversion loses it. Main
selected collections use `new(main_rows, main_names, selected_columns)`.
Anonymous native/NumPy rectangles and genuine direct-local collections use
`from_direct(main_names, columns)`. Independent Polars DataFrame collections keep
named identities through `new(actual_y_rows, ordered_unique_union(main_names,
y_names), y_columns)`; they are not all rewritten as local. This also implements
the newly admitted ordinary DataFrame collection: there is no existing
`NumericSeriesInput for &DataFrame` to claim as its old route.

Focused native checks witness unused `Variable*` namespace avoidance with duplicate/
empty local display names and private ordinals, one-column collection meaning,
unchanged delayed dtype/length ordering, real grid shape/ID authority, and named
independent-frame identity. Installed Scatter checks exercise the actual callers;
neither result certifies every future family binding.

These two descriptors are precise exceptions documented in the current adapter
contract, solely for preserving original coordinates and names while using
existing binding algorithms. They do not create a
general source framework, conversion catalog, selector DSL, Python Rust-payload
input route, cross-adapter dependency or another renderer.

#### Current Native Label Policy Access

The Python key codec preserves the sole native label policy when adding a default
display for a genuinely new bound key. Its borrowed
`Plot<K>::get_labels(&self) -> &Labels` reader delegates to the renamed private
PlotDescription reader; internal callers are migrated without a compatibility
name or public PlotDescription. Core-origin Plots need no adapter metadata.
`Labels::get_aliases(&self) -> Option<impl ExactSizeIterator<Item = (&str, &str)> + '_>`
serves both exact alias-presence checks and immutable PyLabels. No parallel point
reader, adapter reader or shadow map is needed.

The implemented immutable Python `Labels(*, mode="humanized", aliases=None)` holds
only the native policy. Its read-only mode and aliases and complete-replacement
`copy_with` use `Labels::is_humanized(&self) -> bool` and the same aliases reader;
omission preserves a field, aliases=None clears it. Modes are the closed
`humanized`/`raw` choices and aliases are string pairs. Rebuild through the native
factories without retaining a Python map; preserve native last-write-wins and
BTreeMap inspection order. Explicit `resolve(raw)` delegates to the existing
native method; ordinary inspection/copy must not resolve or validate display text.
Python `Plot.labels(Labels)` and `Canvas.labels(Labels)` consume this native policy.
Core-origin Plot and Canvas forward natively; ndarray/Polars-origin Plot restores
its matching adapter before replacement and splitting.

Clone the current native policy only for the candidate edit and apply it through
the core or matching adapter's existing `.labels(...)` method. Preserve the mode, unrelated aliases
and explicit aliases even when they equal the fallback text. Add defaults only
for newly bound identities; source reordering or complete policy replacement
must not resurrect old defaults from key bookkeeping. Reading aliases does not
resolve, humanize, validate or disambiguate display text. Native validation keeps
its existing timing, including empty aliases and display-domain collisions.

## Complete Python Interface

### Complete Theme And Text Values

The implemented immutable-value closure completes the existing Theme/Text roots,
not a new owner hierarchy or scientific-data route. It retains the original 21
appearance/text/PNG classes, adds the 54 Theme descendants and nine Text value
classes below, and extends the same Theme and TextLayout classes. Labels was
delivered with the input codec and is not implemented again. Owning numeric
policies are delivered separately; six additional appearance-policy roots with
family-specific callers still belong to their subsequent consumer phases.

Every wrapper holds only its complete native value. Ordinary records expose
keyword-only typed construction, getter-only named fields and keyword-only
`copy_with` with private omission defaults. Child reads clone public native
readers, not linked handles or Python mirrors. Optional fields admit None;
required fields do not. Resolve all edits and reconstruct a complete native value
once so a valid simultaneous sibling change is not rejected by an invalid
intermediate state. Preserve native validation/use-time boundaries and exact
spacing; never run fonts/rendering for value construction or inspection.

Theme retains Publication as its default and existing typography/surfaces keyword
order. Add axes, legend, composition, marks, annotations and color_scales to both
constructor and copy, with all eight components readable. None is invalid for any
component. Use existing complete native constructors/ThemeBuilder only within the
call; do not expose a Python builder, sparse dictionary, flattened margins or
effective-style cache.

| Native source under `crates/lichtung-plot/src/theme/` | New Python classes |
| --- | --- |
| `axis.rs` | AxisTheme, FrameSides, FrameTheme, TickTheme, AxisSpacing |
| `guide.rs` | LegendTheme, LegendSpacing, GradientLengths, LegendFrameTheme |
| `composition.rs` | CompositionTheme, RootTitleSpacing, RootPadding, TextPanelSpacing, HeaderSpacing, FacetStripSpacing |
| `mark_paint.rs` | StrokeTheme, SolidStrokeTheme, OutlineTheme, PointOutlineTheme, DataOutlineTheme, DataFillTheme, FixedFillTheme, PointTheme, DataLineTheme |
| `mark.rs` | MarkTheme, PointRoles, SizedPointTheme, SupplementalPointTheme, PointRadii, FillRoles, BodyFillRoles, BandFillRoles, DensityFillRoles, DataLineRoles, NeutralPaintRoles, StrokeRoles, ReferenceStrokeRoles, ConnectorStrokeRoles, SummaryStrokeRoles, CurveStrokeRoles, CellMarkRoles, CategoryMarkGeometry, MarkExtentTheme |
| `annotation.rs` | AnnotationTheme, SignificanceGeometry, CellAnnotationGeometry, EstimateAnnotationGeometry, ValueShelfSpacing |
| `scale_values.rs` | AccentPalette, QualitativePalette, ColorStop, SequentialGradient, DivergingGradient, ColorScales |

The complete public native `new` arguments/readers define ordinary record fields;
no Theme reader is missing. Retain the existing RelativeSpacing/Color/font/
typography types. Closed scalar policies use Literals, not wrapper inventories:
StrokePattern is solid/short_dash/medium_dash/long_dash; PointShape is circle/
triangle_up. Alpha fields here are literal appearance bytes, not observation
alpha encodings.

DataOutlineTheme is factory-only fixed(stroke)/mapped(width, alpha), with readonly
kind/fixed_stroke/mapped_width/mapped_alpha; copy edits only the current branch.
An optional branch inspection does not make its required payload nullable.
QualitativePalette is factory-only fixed/extendable with readonly colors/capacity;
copy may replace both complete inputs. Accent/palette/gradient sequences are
snapshotted into native values, preserve order and use native validation without
sorting or cycling. Gradient neutral is derived, not editable.
Specialized scalar/sequence constructors keep their positional payloads:
`ColorStop(offset, color)`, `AccentPalette(seeds)`, `SequentialGradient(stops)`,
`DivergingGradient(stops)` and `TextRotation(degrees)`. Their `copy_with` fields
remain keyword-only; the ordinary-record keyword-only rule does not replace these
selected call forms.

TextLayout adds fit, orientation, anchor, overflow and visibility to its existing
eight fields. Constructor defaults are None and copy defaults are omission;
explicit None removes only that instruction. Preserve all thirteen through
derivation. Rotation remains a fixed-orientation convenience, not another field.
The nine new classes are TextFit, TextRotation, DataCoordinate, DataPosition,
TextOrientation, TextAnchor, TextEllipsis, TextOverflow and TextVisibility:

- TextFit is factory-only fixed/shrink_to/range with grouping=independent/uniform.
  Read minimum/maximum/grouping; copy changes grouping while retaining the
  absolute authored interval. Fixed has absent bounds; equal explicit endpoints
  read the same number twice. Never resolve against a preferred font size.
- TextRotation(degrees) reads the normalized native degrees and derives through
  that constructor; retain the five named native preset factories.
- DataCoordinate is factory-only numeric/category/category_series, with readonly
  kind/value and same-branch value derivation. DataPosition has required x/y.
  These are authored scalar locations needed by data_direction, not columns,
  selectors, domain lookups or a fifth input route.
- TextOrientation is factory-only fixed(rotation), data_direction(from_, to) or
  automatic_ticks(angles). Read optional rotation/data_from/data_to/angles;
  copy edits only the present branch. Some(empty automatic angles) stays present
  and is not proactively rejected.
- TextAnchor is factory-only fractions/first_baseline/center_baseline plus the
  nine named box points. Read horizontal/vertical_fraction/baseline; copy changes
  horizontal while preserving the vertical choice. Fractions are not clamped.
- TextEllipsis(position, *, marker="…", max_lines=1, distinct=None) exposes these four
  fields and complete copy. Position is start/middle/end/left/right. None clears
  the actual optional max_lines/distinct; empty marker, zero cap and false remain
  concrete. Marker admission stays in the reached abbreviation operation.
- TextOverflow(*, failure="error", ellipsis=None) reads/copies those two fields.
  Failure is error/hide/clip/visible; None clears ellipsis, not failure.
- TextVisibility is factory-only all/hidden/every/greedy/parity, preserving native
  required endpoint choices. Read kind/stride/offset/endpoints/separation/
  boundary_tolerance. Copy changes only separation and boundary_tolerance on a
  clone. Even zero separation is only admitted for Greedy/Parity; signed finite
  boundary tolerance follows native rules. A present boundary tolerance,
  including zero, requests boundary reduction; None clears that optional field.
  Owner/context eligibility remains a use-time check.

#### Consumer-Required Text Readers

The following implemented public core readers serve those real Python properties and
copies. All take `&self`, allocate nothing natively and perform no normalization,
validation, projection or font/render work. Python child cloning need not be
zero-cost. Deliver readers and bindings together; no private enums, instruction
dump, placeholder fields or second retained state is authorized.

| Native owner | New reader signatures, return type after `->` |
| --- | --- |
| TextFit | `get_minimum -> Option<f64>`, `get_maximum -> Option<f64>`, `get_grouping -> TextFitGrouping` |
| TextRotation | `get_degrees -> f64` |
| DataCoordinate | `get_numeric_value -> Option<f64>`, `get_category_value -> Option<&str>`, `get_category_series_value -> Option<(&str, &str)>` |
| DataPosition | `get_x -> &DataCoordinate`, `get_y -> &DataCoordinate` |
| TextOrientation | `get_fixed_rotation -> Option<TextRotation>`, `get_data_direction -> Option<(&DataPosition, &DataPosition)>`, `get_automatic_angles -> Option<&[TextRotation]>` |
| TextAnchor | `get_horizontal -> f64`, `get_vertical_fraction -> Option<f64>`, `is_first_baseline -> bool` |
| TextEllipsis | `get_position -> TextElision`, `get_marker -> &str`, `get_max_lines -> Option<usize>`, `get_distinct -> Option<bool>` |
| TextOverflow | `get_failure -> TextFailure`, `get_ellipsis -> Option<&TextEllipsis>` |
| TextVisibility | `is_hidden -> bool`, `get_every -> Option<(usize, usize)>`, `is_greedy -> bool`, `is_parity -> bool`, `get_endpoints -> LabelEndpoints`, `get_separation -> f64`, `get_boundary_tolerance -> Option<f64>` |

These are 28 readers on nine types. Orientation branches are mutually exclusive;
Anchor inspects vertical_fraction before using the baseline query. Visibility is
All only when hidden/every/greedy/parity queries are all absent/false. Existing
TextLayout, TextOffset, TextBounds and the 54 Theme children need no extra reader.
Binding implementations, the one shipped stub/exports and actual immutable-copy/
native-attachment consumers agree within the bounded verification recorded above.
Use focused sibling-validation, nested independence, presence/clear and one
representative publication relationship rather than getter inventories or a
family/container/style matrix.

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
family's required channels, selected alternative forms and optional categorical
bindings. Signed positional selection, pandas key admission and MaskedArray
handling are selected below; exact type stubs and native capability mapping still
need completion. Configuration mutation follows the selected lifecycle
model, and callback references have the separate external-state semantics below.

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
signatures or a new standalone `render()` entry point. Reuse the delivered public
core ownership facilities below and verify their Python mapping; neither a blanket
Clone addition nor a broad storage refactor follows from the capability inventory.

### Constructor Grammar And Optional Bindings

For source-plus-variable calls, allow `data` as the first and only positional
argument, or by keyword. Variable roles are keyword-only. The selected Scatter
shape is `scatter(data=None, *, x, y, hue=None)`, not its complete eventual
signature. Do not additionally interpret `scatter(a, b)` as x/y, or guess whether
the first positional object is a source or x based on its type.

The following examples specify the selected call grammar and are implemented for
Scatter; other families retain the delivery scope recorded above:

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

The selected Python mode names are `by_row_position=True` for Scatter/Line and
`by_column_name=True` for Boxplot/Violin/sample Raincloud. They exclude explicit
x/category respectively. Row position is zero-based, not a pandas index; column
names/default display names never replace scientific column identities. Explicit
`numeric_only=True` discovers supported numeric columns in source order for those
modes and Matrix, including numeric IDs, and excludes a manual variable collection.
It does not filter an explicit selection or make `matrix(df)` an implicit discovery
call. Supplied Histogram/Raincloud instead take complete named geometry fields and
explicit component sources; no map/TypedDict or runtime geometry wrapper is selected.
The [constructor plan](92_PYTHON_CONSTRUCTORS_PLAN.md) records the complete fields,
direct/source-bound examples, label keywords and exclusion rules.

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
mapping, Bar combinations and selected mode/discovery/geometry forms. Exact stubs,
remaining label defaults and policy/method signatures still need completion.
Neither that inventory nor this overload mechanism is delivered binding code.

Use ordinary shared type aliases to keep signatures readable; aliases describe
the four supported input families and selectors, not new runtime payload types.
Union types suffice for simple accepted-type alternatives with otherwise identical
call forms. Do not multiply overloads merely to distinguish numeric and category
containers; actual dtype/shape/validity validation remains at runtime.

### Explicit Column Selectors And Direct Container Indexing

The selected Python syntax uses shared, project-owned `col` and `cols` subscription
entry objects, not separate plotting namespaces or a source/container wrapper.
They are wired to applicable families, with the earlier Scatter transport evidence
and separate focused family-wheel evidence above, not a family/container matrix.
These are not Seaborn/Plotly functions.
Keep ordinary direct inputs and the container's own indexing available;
do not add parallel `x_cols`/`y_cols` keywords or a `columns(...)` call-form alias.

```python
# Implemented Scatter forms; ordinary row-count and family-role checks still apply.
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
- `col[key]` denotes exactly one source field. Use literal DataFrame keys rather
  than guessing positional lookup or formatting a key to a string first. For a
  two-dimensional NumPy source, an integer key denotes a logical column; negative
  indices count from the end. A pandas integer key remains a literal label, including negative
  labels; eager Polars single-column lookup retains literal string names.
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

#### Positional Indices And Slices

The selected grammar supports negative positional indices and positive/negative
slice steps. A `cols[start:stop:step]` slice always addresses logical column
positions on a supported two-dimensional NumPy source or pandas/eager Polars
DataFrame, never label ranges. Resolve it at binding against the current source
column count and preserve the resulting order. Thus pandas `col[-1]` looks up the
literal label -1, while `cols[-1:]` selects its last column as a collection.

- Positional integers use [Python's integer-index protocol][python-index], including
  supported NumPy integers, but reject Boolean positions/endpoints/steps. Do not
  apply that restriction to an actual Boolean pandas column label.
- With n columns, a single/discrete NumPy index must lie in `[-n, n)`; normalize a
  negative index by adding n. Check duplicates by resolved column, so `cols[0, -5]`
  is a duplicate selection on a five-column NumPy source.
- Slices are start-inclusive and stop-exclusive. Omitted step means 1; any nonzero
  integer step is admitted. Omitted endpoints follow the direction: forward defaults
  to 0/n, reverse defaults to n-1/the boundary before column zero.
- Normalize explicit negative endpoints relative to n without silently clipping.
  Forward normalized endpoints must lie in `[0, n]`, reverse endpoints in
  `[-1, n-1]`; these include the direction-appropriate exclusive boundary. Keep an
  omitted reverse stop distinct from an explicitly supplied -1, which refers to
  the last column before stop exclusion.
- Reject zero steps, out-of-bounds explicit endpoints, empty results and duplicate
  resolved columns. A one-column result remains a collection. Do not turn a string
  endpoint into a pandas label slice, or add expression/Boolean-mask selection.

For a five-column table these planned selections mean:

```python
cols[1:4]       # Positions 1, 2, 3, on any supported table source.
cols[-3:]       # Positions 2, 3, 4.
cols[::2]       # Positions 0, 2, 4.
cols[::-1]      # Positions 4, 3, 2, 1, 0.
cols[3:5]      # Positions 3, 4; the forward exclusive boundary is valid.
cols[3:8]      # Error: do not silently clip to positions 3, 4.
cols[-1:-1:-1] # Error: empty, not the meaning of cols[::-1].
cols["a":"d"]  # Error: no label-range query.
```

Strict selector bounds deliberately differ from
[Python's ordinary slice clipping][python-sequence-slices]. Container projections
such as `array[:, 3:8]` and `df.iloc[:, 3:8]` retain their container's clipping/indexing
rules and are ordinary direct data. Do not add a strictness flag or modify the
Rust adapters' public
selection grammar; the Rust/PyO3 boundary resolves the Python description into
the existing applicable selected-column inputs.

#### Exact pandas Column Keys

Accept legal hashable complete pandas column labels that resolve uniquely,
including floats, Timestamp labels and custom hashable objects as well as strings,
integers and tuples. Use a concise Hashable-based key type where appropriate, not
a growing scalar-type union or required runtime key wrapper. This is label lookup,
not permission to accept unsupported temporal/object column data in scientific
roles.

`col[key]` means exactly one complete actual column label. Do not execute a callable
key, parse an expression, accept partial MultiIndex/date matches or use Interval
containment queries. A legal callable used as an actual label is a literal lookup
key, not an operation to invoke. Missing/ambiguous keys fail;
[pandas lookup helpers][pandas-index-get-loc] returning a slice/mask or otherwise
admitting partial queries are not by themselves proof of exact unique lookup.
Normal pandas label equivalence applies; do not invent a type-identity comparison
that splits keys naming the same column.

Retain the original key for lookup. Preserve distinct actual columns such as the
integer label 1 and string label "1" without collapsing them through `str(key)` or
`repr(key)`. Formatting for display is permitted, but formatted text never becomes
the scientific identity. Internal native identities must remain collision-free
and consistent across applicable later bindings; display aliases/column-label
edits do not change them. Existing native display-collision validation still
applies. Encoding and lookup bookkeeping are private binding implementation work,
not a second scientific model, another user decision or a reason to require column
renaming. Users may still make complex pandas selections explicitly and pass the
result as direct data.

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

For complete geometry, Histogram main `data` is bins, with explicit `kde_data` and
`rug_data`; Raincloud main `data` is summary, with `density_data` and `point_data`.
Resolve each role only in its component context and check that domain's lengths,
not a global composite row count or DataFrame-index join. Omitted/None auxiliary
sources require direct inputs there, without falling back to main data. Snapshot
all bound components during the constructor, retaining only main data for later
explicit lookups. Auxiliary sources never become implicit setter contexts or need
to survive successful owned binding; exports do not reread any source.

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

#### Exact-Family Input Ownership And Error Timing

The implementation target retains one native Plot plus its construction origin
in each exact-family configuration version: Core (no adapter metadata), ndarray
metadata or Polars metadata. Store the pair together inside the existing native
Arc version; do not retain an adapter, foreign buffer or Python reference there.
Later argument containers do not switch origin. Expansion/grid bindings use the
origin's existing repetition, alignment and shape algorithms, not pre-expanded
native payloads that bypass them.

The existing native list/tuple one-variable Scatter remains on its original core
path: extract x, then y, then optional hue, construct the native Plot, and leave
cross-column alignment to native preflight. Its hue setter likewise retains its
existing delayed alignment. Do not replace it with an adapter constructor that
rejects length mismatches earlier, or try one constructor and fall back after a
validation error. New source/expanded/grid forms use the selected adapters.
Core-origin foreign roles reuse the existing role extractors before core
attachment, without duplicating scientific checks.

For foreign work depending on owner/source/key state, capture the native/origin
version and real Python source/key references before lookup or conversion can
reenter Python. Plain P1 input conversion with no consumed owner state may still
finish before capturing the current version, preserving its existing update path.
Build candidate data, key
bookkeeping and the matching Plot/metadata pair outside owner borrows. Commit
only if the expected native version is still current; stale work must fail
without overwriting a reentrant edit or automatically retrying user code.
Labels/facet edits affecting adapter caches restore and edit the matching adapter
before splitting again.

Source and original-key bookkeeping are separate physical Python GC edges, not a
second label store or hidden references inside Rust Arc payloads. Copies own
their actual references. Clear/replacement and failed candidates release Python
references only after PyO3 borrows/locks end. Canvas captures only the native
configuration and its existing callback roots, not the source owner/key registry.
Render operation pins and frozen Output retain their existing distinct lifetimes.
Traversal reports the binding's actual physical edges, not every object reachable
inside a foreign container. It cannot repair cycles hidden from CPython's GC by
foreign storage, such as arbitrary keys held in an untracked NumPy object buffer
inside a pandas Index. Such a cycle can remain alive even without a Plot. Do not
duplicate traversal visits or add shadow references to compensate, or claim that
all otherwise legal foreign-container cycles are automatically collectable.

### Input Bridges And Compatible Dependencies

The four supported input classes use different boundary conversions, not different
renderers or scientific semantics:

| Python input | Implementation direction |
| --- | --- |
| Python native data | Extract role-appropriate values into internal native inputs; do not require Rust payload objects or force a NumPy/list/JSON round trip. Supplied geometry uses explicit named columns from the constructor plan, not a geometry map or nested input tree. |
| NumPy arrays | Use rust-numpy readonly ndarray views and the existing ndarray adapter; recognize MaskedArray first and carry its mask into native validity. Release foreign borrows after binding owned values. |
| pandas DataFrame/Series | Resolve exact unique original column keys, preserve identity separately from display, and extract only bound columns by logical dtype. Numeric roles reuse the NumPy/ndarray route with explicit validity; category, Boolean and identity roles keep their native meanings. Do not require whole-table ndarray, Polars or PyArrow conversion. |
| Eager Polars DataFrame/Series | Bridge into the existing Polars adapter rather than first flattening the table into ndarray. The shared bridge has the earlier Scatter transport evidence and separate focused family-wheel evidence above; applicable family consumers are wired, without claiming a family/container verification matrix. |

Preserve the [adapter value contracts](09_DATA_ADAPTERS.md): ordinary numeric f64
rounding, null distinct from present NaN, exact integer IDs, actual string category
keys, logical order, names and explicit provenance. Numeric dtype normalization is
not permission to cast IDs through f64, booleans to 0/1 or arbitrary objects to
display strings. Empty input retains its logical dtype and shape; unsupported
semantic dtypes must not become supported merely because they have no rows.

[rust-numpy][rust-numpy] is the NumPy bridge, not a renderer. Admit supported
strided, reversed, transposed, Fortran and broadcast views without a contiguous-only
restriction. Consume them into owned values before detached rendering.

#### NumPy MaskedArray

[NumPy MaskedArray][numpy-masked-arrays] support is selected as part of the NumPy
input route, not a fifth source family, optional support mode or new project-owned
wrapper. Recognize it before ordinary rust-numpy extraction: ndarray extraction
can accept the subclass while silently discarding its mask.

- Bind data and mask together at the actual constructor/setter call. Preserve
  logical shape, aligned slots, order and source-row correspondence, including
  strided/reversed views and data/mask buffers with different strides. Convert
  only the variables used by this binding, not the entire masked table.
- NumPy `mask=True` means missing; native `validity=True` means present. Invert the
  mask into the existing ndarray/native validity route. Handle nomask/all-valid,
  partial and all-masked inputs without changing their declared shape or dtype.
- Never use `.compressed()`, `.filled()`, fill_value, row deletion or implicit
  imputation. A masked underlying value is not a present observation; an unmasked
  NaN remains present NaN, distinct from null.
- Keep normal dtype/rank restrictions even when every entry is masked. Supporting
  MaskedArray does not make structured/object/temporal data implicitly applicable,
  or infer the meaning of an otherwise ambiguous all-missing axis input.
- Missing-data applicability remains native: sample/coordinate consumers apply
  their existing aligned rules, while roles requiring present values, including
  IDs, alpha bytes and complete supplied geometry, retain their existing failures.
  Do not implement another missing-data/scientific policy in Python.
- Save stable owned bound values/validity. Later data or mask mutation cannot
  refresh them; retaining a main masked lookup source only permits a later explicit
  binding to take a fresh snapshot. Ordinary ndarray users need not supply a mask
  or change their existing input form.

#### DataFrame Extraction And Bridges

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
selected col/cols key contract, without a label-type whitelist or stringification
of lookup keys. Implement and verify the original-key/native-identity/display
mapping through the existing native label policy, not a second label store. No
Rust pandas adapter crate is prescribed.

PyArrow is a Python Arrow implementation, not the Arrow protocol itself and not
a required intermediate dependency for all inputs. Ordinary pandas users must not
need Polars/PyArrow just to plot nullable columns; pandas Arrow-backed storage can
use its already-required Arrow support. Correctly configured pandas-to-Arrow or
pandas-to-Polars conversions can preserve the tested values, but their defaults
can turn ordinary NaNs into nulls. Reject those defaults as an implicit universal
route, not the libraries as inherently lossy.

Python Polars is not a Rust ABI object. The selected implementation target uses
its public Series `__arrow_c_stream__()` and the matching polars-arrow
`ArrowArrayStreamReader`, then the safe Field-aware
`Series::try_from((&field, chunks))`. Preserve complete Field metadata, including
Categorical/Enum meaning; do not use a name-only conversion that discards it.
This requires no PyArrow or private writable-pointer export fallback. The
official [pyo3-polars bridge][pyo3-polars-types] remains reference evidence, not a
second route or Rust ABI cast.

Only genuinely bound Series are exported. Keep original rows and the complete
names namespace separate from imported Columns through the working projection.
For discovery, map supported concrete Python primitive dtype variants to actual
native DataTypes and their shared primitive predicate; after import verify logical
dtype fidelity and shared native discovery. Do not convert unused fields, invent
placeholder dtypes, strip extensions to physical storage or filter an imported
mismatch out of the result.

Collect Python metadata before a capsule pointer loan. The private binding seam
checks the `arrow_array_stream` capsule name and borrows its stream uniquely while
the local capsule reference stays alive. In that lexical loan, perform only Rust
work: no arbitrary Python/reentry, owner borrow, detach or worker access. Return
an explicit Rust Result; end the reader loan before capsule DECREF or Python error
conversion. The capsule owns stream release; imported schema/arrays own their
distinct release obligations. Do not move the capsule's Box or manually mirror
its C layout. Arrays are call-local conversion scratch, never native Plot state.

Guard Arrow logical Field/metadata before reading chunks or invoking unsupported
Polars conversions; dictionary/string shape alone does not validate categorical
metadata parsers containing infallible assumptions. A conforming genuine eager
producer is a safety precondition, not something proved by the GIL or capsule
name. Validate actual binary release behavior, 128-bit private format codes,
Float16, Enum/named Categorical metadata, chunks/offsets and typed empty streams.
No static interface match proves this compatibility or zero-copy behavior.

Choose a compatible dependency set, not independently latest package versions.
Published [pyo3-polars 0.28.0 dependencies][bridge-dependencies] require PyO3 0.29
and Polars 0.55; [numpy 0.29.0 dependencies][numpy-dependencies] also use PyO3 0.29
and admit ndarray 0.17. The next binding dependency target is PyO3 0.29.2 /
rust-numpy 0.29.0 / ndarray 0.17.2 and polars-core/polars-arrow 0.55.2, using the
existing adapter members. Add direct bridge dependencies only to the binding,
with shared exact pins and no new Polars features or reverse core dependency.
Use an ordinary offline lock update and verify preserved third-party pins; no
asset provisioning or downloads follow. The expanded source closure needs a new
independent sdist build/install verification. Actual bridge runtime compatibility
and final Python dependency bounds remain working-implementation gates, not
consequences of this target record.

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
| Columnar geometry prerequisite | Core/adapter fields and removed-type consumers, six independent alignment domains, raw-key grouping and original-point-row alpha, sample-owned supplied KDE, direct/multiple-source inputs and source-release lifetimes. Use the focused [refactor gates](93_COLUMNAR_GEOMETRY_PLAN.md#delivery-and-verification), then API-doc/peer-guide/archive closure. |
| Python | Public capability/type mapping, source-first calls, overload signatures matching runtime scientific-combination checks, selected row-position/column-name modes and numeric discovery, named geometry columns/component contexts, explicit col/cols versus direct values/projections, optional bindings, immutable values and shared Text entries with omission/None/value handling and keyword precedence; binding-time snapshots versus current-source lookup, lifecycle/error/callback ownership, input bridges and installed-wheel/sdist workflows. |
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
package contents may reintroduce the full Unicode corpus.

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
5. **Columnar core geometry and coupled consumers.** Replace both public geometry
   payloads and sample-owned precomputed KDE, migrating validation/preparation,
   alpha/labels and all removed-type consumers together. Verify focused native
   witnesses and strict workspace Clippy before committing this logical change.
6. **Columnar adapter binding.** Replace geometry payload pass-through with each
   adapter's direct/multiple-source column inputs and independent KDE helpers.
   Migrate callers/current contracts/peer guides; verify focused binding/lifetime
   cases, the shared-native consumer, strict Clippy and docs/package closure.
7. **Python lifecycle implementation gate.** Implement the selected object
   semantics through deliberate public-core ownership and text-clear facilities;
   verify their failure/sharing/callback behavior before mass-producing bindings.
8. **Complete Python interface.** Deliver the capability mapping, callbacks, input
   bridges, typing and distribution verification through the same renderer.
9. **Integrated content and closure.** Complete bilingual content/gallery coverage
   and explicit workspace/package/site verification. Report unexecuted external
   checks honestly; actual release, deployment and push require explicit authorization.

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
# Installed-verified value subset; complete interface remains planned.
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
# Installed-verified native-data owner subset.
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

The core distinguishes omitted instructions, concrete instructions and present
default layout requests through Text and its existing presence rules.
It now exposes explicit local-clear operations through that same storage and Theme
resolution, without a new default-argument or style-resolution system.

The delivered consuming native clear operations preserve ordinary setting
signatures and behavior:

```text
Text::clear_style(self) -> Self
Text::clear_layout(self) -> Self
Plot/Canvas::clear_title_style(self) -> Self
Plot/Canvas::clear_title_layout(self) -> Self
```

Plot/Canvas title/subtitle/caption and Plot x/y-label slots have paired
`clear_*_style` / `clear_*_layout` methods. Plot color-label methods retain the
ContinuousColor capability bound. PointLayer has corresponding legend-label
methods, while NumericExplanation has `clear_style` / `clear_layout`. All use shared
Text instruction removal, not a per-family implementation. Both adapters forward
applicable owner/layer methods without another style engine. Constructor-owned immutable Text
values remain editable through ordinary native value construction/clearing.

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

Native clearing passed focused value/slot/numeric-explanation tests, a composed
current-Theme/parent-alignment inheritance witness, strict workspace Clippy and
peer Book checks. Bounded installed checks now cover the current binding's
private-sentinel handling, explicit keyword precedence and failure-safe owners;
those Python behaviors are not established by native clear evidence alone.

### Canvas Captures Configuration, Not A Live Child Alias

Placing a Plot captures its configuration at that call and leaves the external
Python Plot usable. Later edits to that Plot must not change any existing Canvas
capture. Each placement preserves native parent ownership, local instructions and
one scientific producer per owner; it is not a separate Python plotting spec.

Snapshot semantics do not require copying every configuration collection. Native
Plot Clone now shares the authored description's payload, separating it on payload
edits; Canvas placement moves an owned Plot's description into its canonical owner.
A binding can clone the native Plot for a retained external Python handle without
duplicating the shared tick/layer/annotation vectors at capture time. `.copy()` can
likewise start from shared native payloads while returning a distinct Python owner.
Inline fields still clone, and separation can copy payload-owned content; these
facilities do not promise zero-copy or constant-cost capture. Current native-data
Python copy/capture independence has bounded installed verification, not a general
cost guarantee or another core representation.

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

| Current Rust method | Footprint | Occupied-region behavior |
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
overwrite by rebuilding Canvas in Python. These native setters now exist and
passed focused topology/atomicity and composed-rendering witnesses plus their
public example and strict Clippy. Borrowed native placement now preserves the Canvas
on error; the current Python owner and argument mapping passed the bounded installed
checks described above.

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

This Python lifecycle does not introduce a public `render()` method. The native
facilities below now provide borrowed terminal exports; the binding must reuse
them rather than duplicate rendering/publication. A terminal operation uses a
stable configuration version even if an allowed reentrant callback edits the
Python owner for a later operation.

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

The native facilities below are delivered, but do not by themselves implement the
selected Python lifecycle.

The authored source owners are `crates/lichtung-plot/src/builder.rs`,
`crates/lichtung-plot/src/builder/description.rs` and
`crates/lichtung-plot/src/builder/configuration.rs`; placement is owned by
`crates/lichtung-plot/src/canvas.rs` and
`crates/lichtung-plot/src/canvas/placement.rs`.

| Current native fact | Binding implication |
| --- | --- |
| `Plot<K>` contains only an authored PlotDescription and `PhantomData<K>`. The description has inline fields and one `Arc<SharedPayload>`; Plot Clone shares that payload, including exact ticks and layer/annotation vectors, while cloning inline fields such as title/subtitle. | Use public Plot Clone for independent handles without reconverting data. Payload edits use Arc::make_mut and may copy those collections; neither clones nor separation are promised constant-cost. |
| Exact-family/capability bounds, eleven public const configuration setters and const `kind()` are retained. | Map deliberate public family APIs; the private description/payload is not a binding extension point or another public mode. |
| Theme has complete immutable components and a construction-only ThemeBuilder; Text/component field edits consume values. | Expose Python read-only attributes/copy_with through public native value construction and validation, not matching mutable setters or a new Rust alias family. |
| Text preserves content absence, omitted style/layout and present default-layout reset without a separate slot wrapper; Text and retained owners expose independent explicit clear operations. | Map Python omitted/None/value intentions into the same native slots without another retained state; bounded installed checks cover precedence and failure safety for the current native-data subset. |
| Canvas placement takes an owned exact-family Plot and moves its description into the canonical placement payload, not an erased or temporary working Plot. Canvas Clone shares placements; separating them clones placement descriptions while retaining shared Plot payloads. | Capture configuration at insertion, not a live external child. Keep the Python Plot usable through a native clone when needed; inline fields, Canvas headers and range requests can still copy. Whole-Canvas copying and separation are not constant-cost promises. |
| Strict panel/panel_span and region-overwrite set_panel/set_panel_span share one native validation/placement path. | Map Python spans/overwrite to these current capabilities; preserve whole-owner removal and atomic resource preparation through reusable ownership. |
| Borrowed place_panel/place_text_panel preserve the Canvas on error. Consuming insertion/replacement builders delegate to these same native mutations. | Use borrowed placement without taking the sole Python owner or cloning the entire growing Canvas per insertion. Failed placement can detach shared storage but does not change logical configuration. |
| Plot/Canvas and both adapters' terminal exports borrow self. Plot exports pass a borrowed description directly to the native render lifecycle; one operation-local Root owns the occurrence registry, services and caches, with short-lived measurement factories instead of temporary working Plots. | Reuse the same public terminal route while fixing a stable configuration version before callbacks. Internal ownership removes the old working-Plot path, not all copies/allocations or Python reentry hazards. |
| Output is Clone and owns Arc<RenderedScene>; borrowed publishers return a shared frozen handle, including for chained publication. | Preserve existing Output handles on conversion/I/O failure without rerendering. Clone/replay does not rematerialize callbacks. |
| Labels already uses Arc::make_mut. | Copy-on-write can still clone the complete shared map; Arc alone is not a complexity guarantee. |

Wrapping an external Plot in Arc is not itself Canvas insertion or a placement-local
snapshot: the public boundary still accepts an owned exact-family Plot. Use its
public Clone when the external owner must remain usable; placement then moves that
clone's description. Shared-payload separation may still copy the complete payload;
see [Arc behavior][arc-cow]. Do not infer sole ownership by guessing Python reference
counts or silently consume external handles. Keep one native representation/validation
path and core independence. No persistent collection framework, Python builder
hierarchy or general storage redesign is authorized by the preference for sharing;
the delivered facilities suffice to start the binding.

### Remaining API And Implementation Gate

#### Current Native Reusable-Ownership Baseline

The delivered native implementation keeps the existing consuming configuration
and building forms and one scientific/configuration representation. It does not
add a Python builder mode or a second panel list:

- Plot Clone shares the description's single authored payload while cloning inline
  fields. Payload-changing setters separate through Arc::make_mut; inline
  title/subtitle and const-setter edits do not require payload separation. Shared
  payload separation can still copy ticks, text instructions and layer/annotation
  collections, so sharing alone does not establish cost or performance guarantees.
- Plot and Canvas terminal exports borrow the configuration and still return a
  newly rendered Output for each call. Plot passes its description directly, without
  cloning a Plot for marker erasure. Existing consuming publication spellings are
  replaced, not retained as parallel terminal methods; there is no public `render()`.
- Output owns an Arc of one immutable RenderedScene. Its publishers borrow the
  handle and return a clone of the same frozen handle, preserving chaining while
  retaining the original after conversion/I/O failure. They never rerender.
- Canvas implements Clone, sharing its immutable Placements until an edit;
  placement payloads own Plot descriptions whose authored payloads remain shared
  when placements separate. Headers/range requests may still copy, so neither
  whole-Canvas copying nor separation is claimed constant-cost.
- `Canvas::place_panel(&mut self, row, column, row_span, column_span, plot,
  overwrite)` is the failure-safe borrowed placement capability;
  `place_text_panel(&mut self, row, column, panel)` provides strict text insertion.
  The Plot argument remains owned and exact-family; its description is moved into
  placement storage without retaining a working Plot.
  Existing panel/panel_span/set_panel/set_panel_span/text_panel consuming builders
  delegate to these same native mutations. These differ in receiver/error ownership,
  not data interpretation or representation; there is no compatibility dispatcher.
  The ordinary Rust builders retain their parameter lists and do not require a
  repeated Boolean. Python exposes only its already selected single panel method.
- The borrowed operation validates/prepares before committing the placement state.
  Independent Canvas copies retain their prior placements; ordinary insertion on
  an unshared Canvas does not clone the growing owner set. Other fallible builder
  updates may prepare a native candidate before assigning it to the Python owner.
- Preparation/layout borrow descriptions through one canonical occurrence registry
  owned by the operation's Root, not temporary Plot copies. Root-issued factories
  create short-lived measurement contexts; services/caches remain operation-local,
  while owning results survive scene freezing/publication. These private owners
  are not additional Python APIs or a reason to redesign core storage.
- The binding must retain a stable native configuration version and release
  its PyO3 borrow before callbacks/rendering. Reentry edits a later version. Actual Python
  GC, exception/thread/context safety and wheel behavior remain separate real-PyO3
  verification gates, not consequences of these native signatures.

The terminal source is `crates/lichtung-plot/src/orchestrator/output.rs`; operation
ownership is in `crates/lichtung-plot/src/orchestrator/root.rs`,
`crates/lichtung-plot/src/orchestrator/registration.rs` and
`crates/lichtung-plot/src/orchestrator/assignment.rs`. Frozen publication is in
`crates/lichtung-plot/src/output.rs`. Both adapters forward these same public exports
through `crates/lichtung-plot-ndarray/src/frontend/forwarding.rs` and
`crates/lichtung-plot-polars/src/frontend/forwarding.rs`.

Native ownership checks cover shared placement separation and captured Plot
independence, strict/overwrite failure recovery, repeated render operations and
callback failure, shared Output replay after conversion/I/O failure, composed
shared guides and the cross-adapter native consumer. The output-target run passed
its other 21 tests; the new callback lifecycle witness was rerun successfully with
adequate explicit dimensions instead of weakening complete-function-tick fitting.
Strict workspace Clippy, formatting, API documentation and peer Book checks passed.
This is native evidence, not a real-PyO3 ownership/callback result.

Public lifecycle semantics are selected and the native ownership baseline is
delivered; the complete Python implementation is not. Before expanding the binding
surface:

1. Complete exact overloads/runtime combination rules for the selected
   [25-family constructor forms](92_PYTHON_CONSTRUCTORS_PLAN.md), policy/method
   signatures and native label/text mappings. Implement the selected signed-index/
   strict-slice rules, complete hashable pandas keys and MaskedArray validity;
   these are settled contracts, not remaining product choices. Preserve the four
   ordinary data routes without Rust payload classes and use conventional concise
   type aliases.
   Keep the selected single-runtime-entry/overload model. col/cols and
   direct-projection syntax are selected, not a choice between
   selector guessing and competing *_cols keywords. Row-position/column-name modes,
   numeric discovery and flat named geometry/component contexts are selected too,
    not open map-versus-wrapper choices. Map delivered native text-clear
    methods into exact signatures/type stubs for the selected
   omission/None/value, explicit-default and keyword-precedence rules. Mutable
   owners, immutable values, shared Text entries, binding-time snapshots and Canvas
   placement/region-overwrite behavior are selected; do not reopen them as lifecycle
   modes or live aliases.
2. Preserve the delivered native input surface of the prerequisite
   [geometry refactor](93_COLUMNAR_GEOMETRY_PLAN.md) during Python geometry binding.
   Keep one native representation, the sample/geometry
   scientific boundary, independent source domains, raw category/hue keys and
   original point-row alpha. Verify sample-owned external KDE as well as complete
   geometry. This does not extend either Rust adapter to the other's containers.
3. Use the delivered public core facilities for configuration sharing/copy,
   explicit Text/slot clearing, shared strict/region-overwrite Canvas placement,
   non-consuming terminal exports and failure-safe Output replay. Retain a stable
   native configuration version and release the PyO3 borrow before callbacks;
   reentry may edit a later version. Verify the Python mapping through that same
   native path, without a new core-storage task or compatibility alternative.

   These two consumer-required read-only public seams are implemented with their
   actual Python consumers, for delivery in the same logical product commit rather
   than as unused native APIs. Three focused native checks and the bounded installed
   Python consumer checks described above passed.

   - **Canvas callback roots.** The public
     `Canvas::contains_plot_anchor(&self, row: usize, column: usize) -> bool` in
     `crates/lichtung-plot/src/canvas.rs` reads the canonical placements in
     `crates/lichtung-plot/src/canvas/placement.rs`. Only an actual Plot owner's
     anchor returns true; covered non-anchor cells, TextPanel, empty and out-of-range
     coordinates, including `usize::MAX`, return false. The query allocates nothing
     and introduces no Result, validator or layout-time work. The Python consumer
     retains only an anchor-to-callback-Python-reference map: after successful
     placement, remove roots at anchors no longer present and explicitly replace
     old roots at the incoming anchor even when membership remains true. Failed
     placement leaves roots unchanged. Do not mirror spans/occupancy or maintain
     another native Plot inventory.
   - **Immutable layout derivation.** The following TextLayout readers are public in
     `crates/lichtung-plot/src/text.rs`. Each is a public method taking only `&self`:

     | Reader | Return type |
     | --- | --- |
     | `get_alignment` | `Option<TextAlignment>` |
     | `get_fit` | `Option<TextFit>` |
     | `get_fit_priority` | `Option<TextFitPriority>` |
     | `get_direction` | `Option<TextDirection>` |
     | `get_orientation` | `Option<&TextOrientation>` |
     | `get_rotation_mode` | `Option<TextRotationMode>` |
     | `get_anchor` | `Option<TextAnchor>` |
     | `get_offset` | `Option<TextOffset>` |
     | `get_bounds` | `Option<TextBounds>` |
     | `get_wrapping` | `Option<TextWrapping>` |
     | `get_overflow` | `Option<&TextOverflow>` |
     | `get_overlap` | `Option<TextOverlap>` |
     | `get_visibility` | `Option<&TextVisibility>` |

     Python `copy_with` reads all thirteen fields and rebuilds the complete layout
     through `TextLayout::default()` and existing setters, cloning borrowed values
     where needed. Omitted fields retain their source values; explicit None fields
     stay absent rather than being refilled. Preserve all other fields and the
     original value. `rotation` is only the fixed-orientation convenience, not an
     additional stored field.

     Python TextOffset/TextBounds read-only attributes and `copy_with` use these
     public `&self` readers in `crates/lichtung-plot/src/text/layout.rs`:

     | Owner | Reader | Return type |
     | --- | --- | --- |
     | TextOffset | `get_dx`, `get_dy` | `f64` |
     | TextOffset | `get_frame` | `TextOffsetFrame` |
     | TextBounds | `get_max_width`, `get_max_height` | `Option<f64>` |
     | TextBounds | `get_max_lines` | `Option<usize>` |

     Reconstruct derived TextOffset/TextBounds values through their existing
     `new()` constructors with the same checks. Do not expose an instructions dump,
     introduce a general clear/getter framework or retain shadow Python state.
     This does not admit an inner-field inventory for TextAnchor, TextFit or other
     opaque values; NumericFormat still requires its own demonstrated consumer.
     These read-only seams add no core storage representation or validation/layout
     timing change.

4. Complete bounded real-PyO3 checks for aliases/copies, insertion followed by
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
   Include explicit mode/excluded-role checks, numeric discovery in source order
   including numeric IDs versus explicit selections, and complete named geometry
   with direct/selected inputs, unequal component lengths and auxiliary-source
   release. An omitted auxiliary source must not read the main lookup context;
   display-label edits must not change scientific keys or point-alpha alignment.
   Selector coverage must distinguish direct list/tuple data even with matching
   schema keys, exact composite single keys, ordered discrete versus half-open
   selections, one-column versus one-element collection meaning, unsupported roles
   and missing/ambiguous/duplicate/empty/out-of-range/no-source failures. Verify
   selected and direct-projection value snapshots without assuming identical
   provenance/display names or requiring a per-family replay matrix.
   Add bounded boundary witnesses for signed steps/open endpoints, omitted reverse
   stop versus explicit -1, strict non-clipping bounds, duplicate normalized indices
   and pandas integer labels versus positional slices. Verify complete float/
   Timestamp/custom hashable keys, rejection of partial/query matches and distinct
   integer/string keys with equal display text. MaskedArray witnesses must preserve
   independent data/mask strides, null versus unmasked NaN, row alignment and stable
   snapshots after either buffer changes; include native present-only failures.
   Key encoding, mask conversion and the per-API Rust/PyO3 mapping are implementation
   tasks. Reuse native scientific/default/callback behavior; do not reopen them as
   Python product choices or implement another Python-side scientific model.
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
5. Compare sharing/separation with the simple baseline on equivalent successful
   workloads where that implementation changes costs: ordinary configurations,
   shared large numeric inputs, growing tick/annotation/alias content, populated
   Canvas grids and retained independent branches. Do not reduce content or compare
   many retained branches with overwriting one result. Shared-state separation
   costs and whole-Canvas growth need explicit observation, not inferred savings.
6. Separate input conversion, Python/FFI/configuration, rendering/font discovery
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
[python-index]: https://docs.python.org/3.10/reference/datamodel.html#object.__index__
[python-sequence-slices]: https://docs.python.org/3.10/library/stdtypes.html#common-sequence-operations
[pandas-index-get-loc]: https://pandas.pydata.org/docs/reference/api/pandas.Index.get_loc.html
[numpy-masked-arrays]: https://numpy.org/doc/1.26/reference/maskedarray.generic.html
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
