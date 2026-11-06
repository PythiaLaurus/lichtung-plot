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

Python architecture, the four input routes and the object-lifecycle semantics
below are settled implementation directions, supported by bounded prototypes and
explicit API discussion, not delivered binding code. Configuration sharing is a
preferred implementation direction, not a completed experiment. The required
native ownership facilities and remaining API details still need design and
verification before expanding the binding surface.

### Delivery Status

| Objective | Current status |
| --- | --- |
| Mechanical workspace and policy migration | Delivered; workspace verification, representative example execution and standalone Cargo package verification passed. |
| Peer-language Book/example/gallery foundation | Delivered: one gallery-first tutorial page per language, six scientific showcase workflows, two numeric/text reference workflows, same-page image-to-code walkthroughs, shared runnable-source includes and zero-argument examples with optional output redirection. The helper lives under docs/; full per-family gallery coverage remains for integrated content closure. |
| ndarray and Polars adapters | Delivered: all 25 families, source/direct views and fields, wide/grid origins, native configuration, independent layers, statuses, alpha and significance results. Cross-adapter native handoffs, focused/workspace checks, joint offline archives and peer guides passed; the [current contract](09_DATA_ADAPTERS.md) owns the API. |
| Python lifecycle decision and complete binding | Architecture/input routes and mutable configuration/explicit copy/Canvas snapshot/non-consuming render/frozen Output semantics settled. Bounded experiments completed; no binding implementation. Native ownership facilities, remaining API details and full verification are open. |
| Integrated content/package/site closure | Rust adapter examples, peer guides/site and all three source archives are verified. Python and complete per-family gallery coverage remain planned; no publication or deployment was performed. |

## Confirmed Scope And Constraints

| Concern | Required outcome |
| --- | --- |
| Existing adapter/binding code | Replace the existing adapters' conversion-only public design, reusing sound private conversion support; Python is not implemented. No repository/history import is requested. |
| Renderer boundary | One independent `lichtung-plot` core, with adapters and bindings depending on it, never the reverse. |
| Adapter public entry points | Either adapter exposes every native public item/module at its original relative path and with the same definition/signatures. Foreign input uses top-level `Family::new` and `Family::from_data` with non-conflicting helper names, never native-name shadowing or a mandatory binding namespace. No separate direct core dependency is needed; both adapters can coexist and share native types through one compatible resolved core package. |
| Python coverage | Equivalent access to the complete deliberate Rust public capabilities, including all 25 plot families, native inputs, composition, themes, text adaptation and numeric/tick callbacks. |
| Python input | Native Python sequences, NumPy arrays, pandas DataFrame/Series and eager Polars DataFrame/Series converge on the same native values and renderer, through source-appropriate input routes. |
| Python lifecycle | Mutable Plot/Canvas configuration, explicit configuration copies, insertion-time Canvas snapshots, non-consuming terminal renders and explicit frozen-Output reuse; sharing/copy-on-write is an implementation preference, not a second public mode. |
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
constructors and the appropriate adapters internally; native value objects remain
valid direct inputs. A Python plot owns the same core configuration regardless of
its input route. Source lookup context, where retained, is separate from values
already snapshotted into the plot; rendering never reopens foreign containers to
refresh those values. Callback references have the separate external-state
semantics below. Exact constructor names/signatures remain an API design task;
configuration mutation follows the selected lifecycle model.

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

### Input Bridges And Compatible Dependencies

The four supported input classes use different boundary conversions, not different
renderers or scientific semantics:

| Python input | Implementation direction |
| --- | --- |
| Native sequences and exposed native value objects | Extract role-appropriate values into native inputs, or reuse already-native objects; do not force a NumPy/list/JSON round trip. |
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
of choosing the first matching column; the complete column-label syntax remains
part of public API design. No Rust pandas adapter crate is prescribed.

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
| Python | Public capability/type mapping, lifecycle/error/callback ownership, input bridges and installed-wheel/sdist workflows. |
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
   semantics through deliberate public-core ownership facilities and verify their
   failure/sharing/callback behavior before mass-producing bindings.
6. **Complete Python interface.** Deliver the capability mapping, callbacks, input
   bridges, typing and distribution verification through the same renderer.
7. **Integrated content and closure.** Complete bilingual content/gallery coverage
   and explicit workspace/package/site verification. Report unexecuted external
   checks honestly; actual release, deployment and push remain separately gated.

Commit each completed logical objective before unrelated work. Commit messages
describe enduring boundaries/behavior, not numbered delivery stages.

## Selected Python Object Lifecycle

The selected public model is mutable configuration with explicit copies,
insertion-time Canvas snapshots, non-consuming renders and frozen Output reuse.
This follows the bounded analysis and API discussion, not an assumption that
Rust's consuming `mut self -> Self` means Python mutation. Both
[mutation-and-return-self][plotly-updates] and
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
- Failed edits leave the original configuration valid and unchanged; existing
  snapshots remain valid too. A wrapper must not accidentally become empty after
  moving its Rust owner into a fallible consuming method.
- Policy/value objects such as Text and Theme need deliberate, consistent
  attachment/copy rules, but their exact public mutability is not selected merely
  by choosing mutable Plot/Canvas owners.

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
not require placeholder Plot objects or live-reference insertion. Explicit
replacement/editing of an already occupied panel, and any panel-access handle,
remain public API design questions. Do not silently change snapshot insertion into
live linkage to make those operations possible.

An external Plot remains independently renderable after insertion. That does not
promise an exact export/crop of its final Canvas panel: parent theme, shared guides
and composition sizing can affect that panel's result. Such a panel-export API is
not implied by the selected lifecycle.

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
| Canvas placement takes Plot by value and owns a boxed erased Plot, without an automatic Clone. Public Canvas is not Clone. | Rust insertion moves ownership; it is neither a borrowed child nor the proposed shared-configuration facility. Preserve Python handles through a deliberate native design. |
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

1. Resolve how users deliberately update occupied Canvas panels, followed by
   remaining constructor/column-label syntax and policy/value-object attachment
   rules. These details must not reintroduce live external Plot aliases or select
   competing lifecycle modes.
2. Identify the smallest public core facilities for configuration sharing/copy,
   safe Canvas edits, stable non-consuming renders and failure-safe Output replay.
   Record justified native API/storage changes before implementing the full binding;
   migrate affected callers rather than retain compatibility alternatives.
3. Complete bounded real-PyO3 checks for aliases/copies, insertion followed by
   external edits, retained branches, temporary insertions, fallible Canvas edits,
   render/I/O failure and callback ownership/reentry. Reuse the existing Plot and
   successful-replay evidence instead of repeating it as a full suite. Retained
   support/tests need strict Clippy and the shared semantic test ceilings.
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
