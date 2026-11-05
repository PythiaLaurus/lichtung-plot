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
tables. Their final joint delivery checks are in progress. Python bindings remain planned. Package publication,
site deployment, Git push and history rewriting are not authorized by this work.

Python object mutation/copy semantics are deliberately unresolved. Record the
settled scope first; analyze that choice separately and, if source inspection
cannot settle it, resolve it through analysis and measurements at the beginning
of the Python-binding work. It must not block the workspace, adapters or Book.

### Delivery Status

| Objective | Current status |
| --- | --- |
| Mechanical workspace and policy migration | Delivered; workspace verification, representative example execution and standalone Cargo package verification passed. |
| Peer-language Book/example/gallery foundation | Delivered: one gallery-first tutorial page per language, six scientific showcase workflows, two numeric/text reference workflows, same-page image-to-code walkthroughs, shared runnable-source includes and zero-argument examples with optional output redirection. The helper lives under docs/; full per-family gallery coverage remains for integrated content closure. |
| ndarray and Polars adapters | Implemented: all 25 families, source/direct views and fields, wide/grid origins, native configuration, independent layers, statuses, alpha and significance results. The [current contract](09_DATA_ADAPTERS.md) owns the delivered API; final joint archive/guide verification is in progress. |
| Python lifecycle decision and complete binding | Not implemented; the binding-time decision gate remains required. |
| Integrated content/package/site closure | Pending the corresponding implementations; actual publication remains separately authorized. |

## Confirmed Scope And Constraints

| Concern | Required outcome |
| --- | --- |
| Existing adapter/binding code | Replace the existing adapters' conversion-only public design, reusing sound private conversion support; Python is not implemented. No repository/history import is requested. |
| Renderer boundary | One independent `lichtung-plot` core, with adapters and bindings depending on it, never the reverse. |
| Adapter public entry points | Either adapter exposes every native public item/module at its original relative path and with the same definition/signatures. Foreign input uses top-level `Family::new` and `Family::from_data` with non-conflicting helper names, never native-name shadowing or a mandatory binding namespace. No separate direct core dependency is needed; both adapters can coexist and share native types through one compatible resolved core package. |
| Python coverage | Equivalent access to the complete deliberate Rust public capabilities, including all 25 plot families, native inputs, composition, themes, text adaptation and numeric/tick callbacks. |
| Python input | Native Python sequences, NumPy and Polars use the same native-value/rendering route. |
| Python compatibility | CPython 3.10 is a required minimum, not a version that may be excluded by adopting a moving support-window recommendation. |
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
pre-implementation audit or compatibility project. Python remains at its own
decision point below.

## Complete Python Interface

### Public Capability Mapping

Use PyO3 and maturin with a mixed package layout, an internal native extension,
and shipped typing information. [Maturin's project layout][maturin-layout] and
[PyPA's source-layout guidance][python-layout] address accidental source-tree
imports and packaging boundaries. Keep Python glue thin; no Python-side renderer,
parallel scientific specification or JSON round trip is authorized.

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

No mutable/immutable Python configuration choice, public Clone addition, new
render entry point or configuration-storage refactor is authorized merely by this
capability inventory. Resolve the object-lifecycle design at its explicit gate.

### Input Bridges And Compatible Dependencies

Native sequences, NumPy and Polars converge on the same native data contracts.
[rust-numpy][rust-numpy] offers readonly ndarray views; consume them into owned
values instead of retaining a mutable Python buffer during detached rendering.
Do not force a contiguous-only route or introduce a list/JSON fallback that changes
missingness, dtype meaning or source order.

Python Polars is not a Rust ABI object. Compare the official
[pyo3-polars bridge][pyo3-polars-types] with the
[Arrow PyCapsule protocol][arrow-capsules], including compatibility, supported
types, dependency footprint, reference/release safety and adapter reuse. Select
one supported route rather than accumulate fallback chains.

Choose a compatible dependency set, not independently latest package versions.
Published [pyo3-polars 0.28.0 dependencies][bridge-dependencies] require PyO3 0.29
and Polars 0.55; [numpy 0.29.0 dependencies][numpy-dependencies] also use PyO3 0.29
and admit ndarray 0.17. This is a candidate manifest-compatible set, not compiled
or runtime verification and not a fixed version decision for implementation.

### Python 3.10 And Distribution Verification

Set CPython 3.10 as the minimum. Do not adopt SPEC 0 as a reason to remove it.
Actual releases of [PyTorch][torch-release], [TensorFlow][tensorflow-release] and
[Polars][polars-python-release] declare Python 3.10 support; the interface must
serve that established machine-learning environment.

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
5. **Python lifecycle decision gate.** Resolve the deferred object semantics and
   necessary public-core ownership facilities before mass-producing bindings.
6. **Complete Python interface.** Deliver the capability mapping, callbacks, input
   bridges, typing and distribution verification through the same renderer.
7. **Integrated content and closure.** Complete bilingual content/gallery coverage
   and explicit workspace/package/site verification. Report unexecuted external
   checks honestly; actual release, deployment and push remain separately gated.

Commit each completed logical objective before unrelated work. Commit messages
describe enduring boundaries/behavior, not numbered delivery stages.

## Open Python Object-Lifecycle Decision

No choice is made here between mutation-and-return-self and new-configuration
returns. Do not infer a Python decision from Rust's consuming `mut self -> Self`:
it moves ownership, whereas Python callers can retain aliases. Do not treat either
design as universally faster or more idiomatic without examining this project.

### What Source Inspection Establishes

Both [Plotly's mutation-and-return-self contract][plotly-updates] and
[Altair's new-configuration methods][altair-properties] have established
precedents. A choice of configuration semantics is separate from a choice of
physical storage or rendering schedule. It is not a contest between inevitably
deep-copying everything and inevitably mutating without copying.

The current native implementation provides the following concrete evidence:

| Owner | Established fact | Consequence, not a timing claim |
| --- | --- | --- |
| `crates/lichtung-plot/src/builder.rs`, `crates/lichtung-plot/src/values.rs` | Plot input and native numeric/category payloads use Arc; Theme attachments are shared too. | A new configuration need not copy all observation buffers or repeat foreign-input conversion. |
| `crates/lichtung-plot/src/builder.rs`, `crates/lichtung-plot/src/text.rs`, `crates/lichtung-plot/src/layout/panel.rs` | Plot's derived Clone also clones owned text, annotation/layer vectors and exact tick vectors/strings. | Whole-Plot cloning is not constant-cost in total configuration size, even with shared scientific data. |
| `crates/lichtung-plot/src/canvas.rs`, `crates/lichtung-plot/src/canvas/placement.rs` | Public Canvas is not Clone; its private placements own grid-sized vectors and boxed Plot payloads. | A reusable Python Canvas needs an explicit ownership design; blindly copying placements for every insertion can have quadratic cumulative cost when panel count grows with grid size. |
| `crates/lichtung-plot/src/labels.rs` | Alias maps already use Arc::make_mut. | Existing copy-on-write is useful, but repeated edits to shared maps can still copy the map; Arc alone is not a complexity guarantee. |
| `crates/lichtung-plot/src/output.rs` | Output is not Clone and its fallible publication methods consume self. | Both configuration models must address reusable Python Output handles and publication failure without rerendering the frozen scene. |

Public setters commonly consume `self`; some, such as Canvas placement, return
`Result<Self>`. On Err the caller does not receive the original Rust owner back.
A Python mutation wrapper cannot simply move the value out, call such a method,
and assume it can restore the original on failure. Retaining a candidate copy,
changing a deliberately public ownership facility, or choosing a different
documented failure protocol has real consequences. Private in-place helpers are
not automatically available to a separate binding crate. New-configuration
returns also need a lawful native ownership route; neither design is free.

Transient and retained versions have different memory behavior. In a linear
configuration chain, released predecessors need not all remain resident, but
cumulative allocation/copy traffic can still grow with every edit. Retaining many
branches retains their actual differing configuration. Conversely, a fair mutable
comparison must include the snapshots/copies needed to produce those same
independent branches, not compare branch construction with repeatedly overwriting
one result.

Wrapping an entire configuration in Arc and invoking make_mut can still clone
that whole configuration when shared; see [Arc's documented behavior][arc-cow].
Fine-grained sharing/persistent collections add indirection, reference-counting,
equality/dependency and maintenance costs. Do not prescribe them before measuring
the simple baseline, or claim that shared observation data proves negligible
configuration overhead. Do not use a guessed Python reference count as proof
that a caller cannot observe mutation.

### Compare Complete Observable Semantics

Evaluate these concerns together, not only setter timing:

| Concern | Required comparison |
| --- | --- |
| Aliases and variants | Whether an edit changes another name for the same Python object; how callers derive independent plots; whether chaining without assignment discards the result. |
| Exceptions and ownership | Invalid edits, invalid placement, callback failures and output I/O failures; retained handles must not accidentally become empty or refer to half-applied edits. |
| Composition | Whether inserting a Plot into Canvas captures a native configuration or keeps a live mutable child reference. Preserve parent ownership, local instructions and one scientific producer per owner rather than accidentally changing composition semantics. |
| Input independence | Both models use owned native snapshots; neither may turn later NumPy/Polars mutation into an unrequested edit of an existing plot. |
| Rendering and publication | A terminal render uses stable inputs; an already returned Output stays frozen across later configuration edits and subsequent publications. No implicit cross-operation layout cache or early render may hide construction cost. |
| Callbacks and threads | Python ownership/cycles, callback identity, exceptions, reentrancy and interpreter attachment. [PyO3 runtime borrowing][pyo3-classes] means holding a mutable object borrow across a callback can prevent reentrant access; immutability alone also does not solve [Python reference cycles][pyo3-gc]. |
| Native boundary and complexity | Whether the design uses only deliberate public core facilities and preserves one representation/validation path, versus introducing a parallel Python specification, duplicated validators or a general builder hierarchy. |
| User-facing consistency | Naming, typing, documentation and error behavior must consistently express the selected model across Plot, Canvas, Text/Theme policies and Output; these objects need not all have identical mutability, but differences must be deliberate. |

The mutable-configuration model can be attractive for interactive stepwise edits
and may avoid some candidate copies where ownership permits it. The
new-configuration model can make reusable templates and independent variants
clearer and may isolate configuration changes naturally. These are tradeoffs,
not proof of a universal performance winner or grounds to reduce API coverage.

### Binding-Time Analysis And Measurement Gate

**Decision: defer the configuration-semantics choice to the beginning of the
Python-binding work.** Source inspection identifies risks but cannot establish
actual PyO3/CPython construction latency, allocation behavior or acceptable
tradeoffs. No benchmark or Python prototype has been executed for this planning
change. Do not select either model or restructure the core speculatively now.

At that gate:

1. Write concrete alias/variant, edit-failure, composition and repeated-publication
   examples for each candidate. Specify native ownership and Python-reference
   lifetimes, including reentrant/throwing callbacks, before comparing speed.
2. Build bounded internal comparison prototypes through actual PyO3 entry points,
   covering Plot, fallible Canvas edits and frozen Output reuse. A Rust Clone
   microbenchmark alone cannot settle the Python interface. Apply strict Clippy
   and the existing semantic test ceilings to any retained support/tests.
3. Compare equivalent workloads: small ordinary configurations; large shared
   numeric inputs with many scalar edits; growing exact-tick/annotation/alias
   content; progressively populated Canvas grids; and retained branches derived
   from a common configuration. Include successful output/semantic checks so that
   reduced content or changed ownership cannot masquerade as an optimization.
4. Separate foreign-container conversion, Python/FFI/configuration construction,
   rendering/font discovery and frozen-output publication. Report absolute timing,
   scaling with input/configuration size, cumulative allocations and independently
   observed peak memory; do not substitute allocation totals for peak RSS or hide
   setter cost in total rendering time. Include required CPython 3.10 and a newer
   supported interpreter. Do not invent a speedup claim or arbitrary universal
   percentage threshold without evidence and a real workflow.
5. Choose the least complex design meeting the agreed observable semantics and
   measured workflow needs. If that evidence still leaves a product tradeoff,
   present the concrete results and examples to the user then, rather than ask
   for another abstract mutable/immutable preference now.
6. Record the selected single public model and justified native API/storage
   changes before delivering the full binding surface. Migrate affected callers
   and remove comparison-only alternatives. Do not introduce modes, per-setter
   inplace switches or competing representations merely to avoid the decision;
   an additional product capability would need its own established basis and
   user confirmation.

This is a required bounded decision gate, not indefinite uncertainty or a renderer
feature in the deferred inventory. Workspace, adapters and peer-language Book
infrastructure can proceed independently; Python-specific published examples and
method/lifecycle contracts must wait for the result.

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
