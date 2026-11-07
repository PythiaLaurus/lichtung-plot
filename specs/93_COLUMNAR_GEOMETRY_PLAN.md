# Columnar Distribution Geometry Refactor Plan

## Status And Authority

This is a selected refactor direction and implementation plan, not delivered Rust
or Python behavior. Current scientific and adapter behavior remains defined by the
[rendering contract](02_RENDERING_CONTRACT.md) and
[adapter contract](09_DATA_ADAPTERS.md). Update those current contracts with working
code, not in anticipation of it.

The [workspace/interface plan](91_WORKSPACE_AND_INTERFACES_PLAN.md) owns overall
delivery and lifecycle. The [Python constructor plan](92_PYTHON_CONSTRUCTORS_PLAN.md)
owns Python call forms. This document owns the prerequisite core/adapter changes.
No binding, new dependency, network access or asset provisioning is authorized by
recording this plan.

## Selected Scope

Replace public Histogram/Raincloud supplied geometry with flat named columns,
following the same input principle as core x/y. Adapters bind those columns from
ordinary direct inputs and explicit independent sources. Callers do not assemble
Dimensions/Group/Bin/Point payload trees or fixed-string-key maps.

- Keep one current public representation. Migrate repository callers and remove
  replaced types/signatures, without aliases, version selectors, fallback chains
  or deprecations.
- Keep `HistogramInput`/`RaincloudInput` as the mutually exclusive sample/geometry
  source choice and retain exact native family results. Do not change statistics,
  sample layouts, completeness requirements or applicability restrictions.
- Private prepared/render results may contain groups, row indices or mark records.
  They are derived operation results, not another public scientific input model.
  Do not reconstruct the old public payload tree as the new validation authority.
- Do not redesign unrelated Bar, x/y, Theme/Text, Canvas or Output APIs. In
  particular, flattening scientific input does not flatten the selected nested
  immutable Theme model.
- Each Rust adapter still handles only its corresponding foreign containers:
  ndarray views/validity/index selection, or eager Polars DataFrame/Series/Column
  and field selection. Preserve existing per-column core-value pass-through and
  complete native re-exports. Neither adapter depends on or converts the other's
  foreign containers; the core depends on neither.
- Four-source unification belongs to the Python binding. Python still does not
  expose Rust scientific payload classes as another data-input route.

## Flat Core Inputs

Retain the two top-level geometry names and replace their fields with the following
planned shape. Reuse existing owned `NumericValues`/`CategoryValues`, including
their names, validity and immutable payload sharing; do not add geometry-column
wrappers or a public selector framework.

```rust
pub struct HistogramGeometry {
    pub value_dimension: String,
    pub density_dimension: String,

    pub bin_lower: NumericValues,
    pub bin_upper: NumericValues,
    pub bin_density: NumericValues,

    pub kde_x: NumericValues,
    pub kde_y: NumericValues,

    pub rug_x: NumericValues,
    pub rug_baseline: NumericValues,
    pub rug_top: NumericValues,
}

pub struct RaincloudGeometry {
    pub category_dimension: String,
    pub value_dimension: String,
    pub hue_dimension: Option<String>,

    pub category: CategoryValues,
    pub hue: Option<CategoryValues>,
    pub q1: NumericValues,
    pub median: NumericValues,
    pub q3: NumericValues,

    pub density_category: CategoryValues,
    pub density_hue: Option<CategoryValues>,
    pub density_y: NumericValues,
    pub normalized_width: NumericValues,

    pub point_category: CategoryValues,
    pub point_hue: Option<CategoryValues>,
    pub point_y: NumericValues,
    pub normalized_strip: NumericValues,
}
```

### Semantic Dimensions And Display Labels

Move existing semantic dimension names to scalar top-level fields instead of
discarding them with the Dimensions structs. Raincloud category/hue dimension
names currently participate in keyed alpha matching as well as default prose.
Display aliases or authored axis titles must not replace those scientific keys.

Rust callers/adapters supply explicit scalar dimension metadata and preserve bound
column names for diagnostics. Python does not gain public `*_dimension` or
`*_name` keywords: its selected `value_label`, `density_label`, `category_label`
and `hue_label` keywords update the existing native text slots. The binding derives
internal dimensions from deliberate source metadata/documented role defaults,
without a second label store. Final label-default/None signatures remain a binding
design task; this refactor does not silently select new defaults.

### Remove Nested Public Payload Types

Remove the definitions, re-exports and all repository callers of:

- `HistogramDimensions`, `HistogramBin`, `HistogramKdePoint`, `HistogramRug`.
- `RaincloudDimensions`, `RaincloudGroup`, `RaincloudDensityPoint`, `RaincloudPoint`.

The adapters' unrestricted native re-exports expose the resulting current core
API automatically. Do not retain local shadow/compatibility types.

### Migrate Sample-Owned Precomputed KDE Too

`HistogramKdePoint` also serves the supplied KDE over sample-owned histogram bins
and rugs. Replace that public point-vector form atomically with:

```rust
pub enum HistogramKde {
    Internal,
    Precomputed {
        x: NumericValues,
        y: NumericValues,
    },
}
```

Preserve `HistogramComposite::KdeAndRug`, its Density-stat requirement and the
sample ownership of bins/rugs. Do not confuse this layer with complete supplied
geometry or retain a second tuple/point-vector KDE representation.

### Ownership, Equality And Existing Identity

Keep deliberate Clone/Debug/PartialEq capabilities on the geometry/KDE values.
NumericValues/CategoryValues currently do not implement PartialEq; replacing fields
cannot simply retain an uncompilable derive. Use small private comparison logic
near the affected types for actual names, validity and values, not Arc addresses
or a new global public comparison framework. Preserve the applicable numerical
identity/equality behavior rather than making invalid values reflexively equal by
an accidental bit comparison.

Geometry may still be held behind Arc by the closed source. Audit actual retained
preparation/cache inputs independently of public PartialEq; preserve existing
content/validity/operation identity rules without a broad storage redesign or
unmeasured zero-copy/performance claim.

## Alignment And Scientific Preparation

### Six Independent Alignment Domains

| Family | Domain | Equal-length columns within that domain |
| --- | --- | --- |
| Histogram | bins | bin_lower, bin_upper, bin_density |
| Histogram | KDE | kde_x, kde_y |
| Histogram | rugs | rug_x, rug_baseline, rug_top |
| Raincloud | summary | category, optional hue, q1, median, q3 |
| Raincloud | density | density_category, optional density_hue, density_y, normalized_width |
| Raincloud | points | point_category, optional point_hue, point_y, normalized_strip |

Different domains need not have the same length. Reject unequal lengths before
zipping instead of silently truncating. Do not assign one global sample-row count
to composite geometry or silently reinterpret generic row_count as summary/point
count. Binding/alpha code must use the applicable component scope.

Retain native completeness, safety limits and scientific rules: contiguous ordered
bins, finite bounds, nonnegative densities and required positive values, strictly
increasing KDE x, valid rug ranges, Raincloud quartiles/support and normalized
coordinates. Histogram still needs at least one bin, two KDE points and one rug;
Raincloud needs at least one group, with two density points and one raw point per
group. Keep nonempty dimension/category/hue identities and existing count limits.
Complete geometry still requires its existing components; do not estimate, impute,
filter, sort or normalize missing data to make it valid. Nullable columns retain
null separately from NaN, but consumed complete-geometry values must still satisfy
the native present/finite requirements.

Keep scientific failures at native preflight/preparation. Adapter extraction and
same-domain shape checks do not become a second scientific validator. Preserve
the restrictions on sample-only facets/SourceRows and supplied-Raincloud
series/significance; flat input is not a new sample mode.

### Raincloud Group Membership And Order

1. Summary row order determines display-group order. Category is unique without
   hue; `(category, hue)` is unique with hue. Preserve whole-group placement, not
   an implicit switch to sample-series dodge.
2. `hue_dimension`, `hue`, `density_hue` and `point_hue` are consistently present
   or consistently absent. Do not invent empty-string hue levels.
3. Density/point rows reference known summary groups, and every declared group
   has its required density/point components. Reject missing, unknown or ambiguous
   membership instead of discarding rows or filling components.
4. Match actual raw strings, not display aliases, normalized spelling, DataFrame
   indexes or dictionary integer codes from independently encoded columns.
5. Preserve relative input order within each group's density/point records, even
   when groups are interleaved. Check density monotonicity in that order; do not
   sort invalid input into validity.
6. Build a deterministic key-to-summary-row index with checked sizes and fallible
   first-party reservation. Hash iteration order never controls display; avoid
   scanning every group for every point. Reuse prepared membership within an
   operation rather than rebuilding it in each layout feedback pass.

### Original Point Rows And Alpha

Current supplied Raincloud preparation numbers points by concatenating nested
groups. The flat representation must instead retain the original points-domain
row index through grouping. Observation alpha/guide membership follows that row;
summary line/fill alpha remains keyed by canonical dimensions and group values.
Reordering summary groups must not reassign point alpha. Histogram bin alpha
continues to match exact lower/upper intervals, preserving rug duplicates/order.

Migrate old fixtures to equivalent group-contiguous columns to preserve their
scientific, guide and publication witnesses. Add bounded coverage for interleaved
rows and nonmatching dictionary codes; do not repin whole-scene goldens.

## Adapter Binding Design

Replace the current `geometry(native_geometry)` and
`geometry_from_data(source, native_geometry)` frontend pass-through signatures.
Rust consumers needing core construction still have the re-exported Plot API;
they do not need another native-payload frontend route.

| Complete input form | Direct columns | Source-bound columns |
| --- | --- | --- |
| Histogram | `geometry` | `geometry_from_data` |
| Raincloud without hue | `geometry` | `geometry_from_data` |
| Raincloud with hue | `geometry_with_hue` | `geometry_with_hue_from_data` |

These are planned method spellings. Numeric/category parameters use the existing
NumericInput/CategoryInput protocols in domain order, followed by literal scalar
dimension metadata. Do not introduce nested descriptor arguments, mandatory
native-value preprocessing or a bind namespace. The two Raincloud forms avoid
uninferrable `None` in `Option<impl CategoryInput>` and fake empty hue columns;
they are complete scientific field combinations, not legacy aliases.

### Independent Sources And Lifetimes

- Histogram's main source is bins; supplemental sources are KDE and rugs.
  Raincloud's main source is summary; supplemental sources are density and points.
- In Polars source-bound forms, retain the main `&'source DataFrame`; supplemental
  sources are independent `Option<&DataFrame>` borrows. In ndarray, keep the main
  `impl ArraySource + 'source`; supplemental `Option<&dyn ArraySource>` borrows
  avoid the None-inference problem of optional generic sources.
- Each variable resolves only in its named component context. None means that
  component is supplied directly; a selector without its source fails, never
  falls back to another component's source. Supported direct inputs can be mixed
  with selectors in every applicable field.
- Snapshot all bound columns during the call. Only the main source remains for
  later explicit lookups; supplemental sources/direct inputs do not constrain the
  completed frontend's lifetime. Conversion into the exact native Plot releases
  the main lookup context too.
- Later setters do not silently choose sources based on target or matching field
  names. Use a direct supplemental projection or an existing explicit independent
  source operation when binding later data from another table.
- Preserve each adapter's dtype, mask/validity, chunk/stride, naming and error
  contracts. No numeric-to-category formatting, implicit index joins, expression
  evaluation or cross-adapter foreign-container conversion is added.

Audit frontend.rs/alpha.rs rather than assigning summary length to every geometry
binding. Point observations, summary-keyed values, bin-keyed values and Single
units retain their own scopes. Any necessary metadata is small and private, not
a public general alignment framework.

For flat entry signatures that genuinely exceed argument-count lint limits, use
narrow justified lint handling on those methods only. Do not disable strict Clippy
globally or replace typed parameters with an unrestricted argument bag.

### Ordinary Inputs For Sample-Owned KDE

Plan `composite_precomputed_kde(x, y)` for direct columns and
`composite_precomputed_kde_from_data(kde_source, x, y)` for an explicit independent
source. These configure the existing KDE+rug composite only: do not alter stat,
bins, samples or the main lookup source. Bind/snapshot the KDE at that call and
release its independent source. Keep ordinary `.composite(...)`/Internal choices
without another KDE representation or a native point wrapper requirement.

## Migration Owners

| Owner | Required migration |
| --- | --- |
| Core geometry/input/lib/spec | Flat fields, deleted exports, closed-source wiring, KDE variant, validation/membership and local equality. |
| Builder accessors and text requirements | Default axis/guide sources and summary-row category labels; preserve semantic names separately from authored text. |
| Histogram preparation/rendering | Consume columns in both complete geometry and sample-owned KDE; remove dependencies on deleted public point/bin types. |
| Supplied Raincloud preparation | Core key matching, stable group/row mappings, alpha, guides and marks. |
| Layout/distribution extraction fixtures | Migrate actual geometry consumers, not unrelated layout algorithms. |
| Both adapter constructor/context/alpha inventories | Direct/multiple-source forms, correct domains and lifetimes, independent KDE helpers and native errors. |
| Tests and peer guides | Replace old constructors and retain meaningful current scientific/publication witnesses. |

Start from the actual source owners:

- [geometry](../crates/lichtung-plot/src/geometry.rs),
  [closed inputs](../crates/lichtung-plot/src/input.rs),
  [exports](../crates/lichtung-plot/src/lib.rs),
  [KDE policy](../crates/lichtung-plot/src/spec.rs).
- [accessors](../crates/lichtung-plot/src/builder/accessors.rs),
  [text requirements](../crates/lichtung-plot/src/layout/text/requirements.rs),
  [Histogram](../crates/lichtung-plot/src/plots/histogram.rs),
  [supplied Raincloud](../crates/lichtung-plot/src/plots/violin/precomputed.rs),
  [layout fixtures](../crates/lichtung-plot/src/layout/interior/preference.rs),
  [distribution fixtures](../crates/lichtung-plot/src/plots/categorical_distribution/extraction.rs).
- [ndarray constructors](../crates/lichtung-plot-ndarray/src/frontend/constructors.rs)
  and [alpha](../crates/lichtung-plot-ndarray/src/frontend/alpha.rs);
  [Polars constructors](../crates/lichtung-plot-polars/src/frontend/constructors.rs)
  and [alpha](../crates/lichtung-plot-polars/src/frontend/alpha.rs).
- [distribution integration](../crates/lichtung-plot/tests/distributions/composites.rs),
  [degeneracy](../crates/lichtung-plot/tests/rendering/degeneracy.rs),
  [ndarray tests](../crates/lichtung-plot-ndarray/src/tests.rs),
  [Polars tests](../crates/lichtung-plot-polars/src/tests.rs) and the existing
  [cross-adapter consumer](../docs/book-helper/tests/native_adapters.rs).
- Peer [English](../docs/en/src/sections/data.md) and
  [Chinese](../docs/zh-CN/src/sections/data.md) data guides. Search the whole
  repository for further live examples/consumers before removing the old types.

Update affected current rendering/property/guide/adapter contracts when working
behavior is delivered. Keep Python plans marked as plans until the actual binding
and ownership facilities exist.

## Delivery And Verification

### Core API And Coupled Consumers

Replace both payloads and KDE together; migrate every directly affected library,
test, example and document consumer of removed types so all targets still compile.
The existing adapter pass-through may temporarily accept the new core payload
until its own signature replacement, without an old/new core representation.
Run focused core checks and strict workspace Clippy, then commit this logical API
change before moving to the adapter input surface.

### Both Adapter Input Surfaces

Replace native-payload frontend signatures, implement direct/multiple-source
bindings and independent KDE inputs, and migrate their callers/guides/current
adapter contract together. Run focused adapter checks, the existing cross-adapter
consumer and strict workspace Clippy, then commit this logical change.

### Integrated Closure

Verify API docs, peer guide checks and distributable archives using `just docs`,
`just book-check` and `just package`; run changed runnable examples. Do not claim
Python wheel/Polars bridge/lifecycle verification from this Rust refactor. Keep
commit subjects scoped and about enduring behavior, not numbered plan stages.
Push/publication requires separate authorization.

Reuse and migrate existing tests. Add only distinct contracts within the shared
800 unit / 300 integration / 80 core-E2E limits, counting semantic scenarios, not
packaging. Do not add per-field accessor echoes, family/container cross-products
or scene/hash/automatic-size goldens. Important bounded witnesses include:

- Per-domain shape/null/finite rules and preserved scientific restrictions.
- Summary order versus interleaved child rows, unequal component sizes, distinct
  dictionary coding, unknown/duplicate/missing membership and incomplete hue.
- Original-row point alpha, summary-keyed alpha and shared-guide coordination.
- Same-operation sample/supplied Histogram equivalence and meaningful zero-span/
  scientific-collapse behavior.
- Direct/selected mixed adapter inputs, independent strided validity or chunked
  data, supplemental-source errors and source-independent owned handoff.

Start with explicitly filtered tests such as geometry::validation_tests, relevant
precomputed_/supplied distribution cases, the affected Histogram/Raincloud
degeneracy cases and categorical-preference witness. Add focused adapter geometry
tests and the existing native_adapters integration consumer; confirm filters
actually match tests. Include sample-owned precomputed-KDE neighbors.

Implementation-time starting commands for existing witnesses are:

```sh
cargo test --locked --offline -p lichtung-plot --lib geometry::validation_tests
cargo test --locked --offline -p lichtung-plot --test distributions precomputed_
cargo test --locked --offline -p lichtung-plot --test distributions supplied_histogram_geometry_matches_sample_labels_and_recorded_marks
cargo test --locked --offline -p lichtung-plot --test distributions embedded_raincloud_hues_coordinate_with_sample_series_in_shared_canvas_legend
cargo test --locked --offline -p lichtung-plot --test distributions histogram_composite_requires_density_before_backend_begin
cargo test --locked --offline -p lichtung-plot --test rendering histogram_spans_preserve_zero_density_and_reject_collapsed_width_or_positive_height
cargo test --locked --offline -p lichtung-plot --test rendering supplied_raincloud_preserves_zero_quartiles_but_not_collapsed_positive_body_or_box
cargo test --locked --offline -p lichtung-plot --lib sparse_collective_families_share_one_categorical_preference_carrier
cargo test --locked --offline -p lichtung-plot-book --test native_adapters
```

Add bounded filters for each adapter's new geometry/KDE tests once they exist;
there are no current flat-geometry adapter tests to claim as passing. Extend to
sample composite/rug neighbors only when their changed preparation path requires
it. These commands are a future verification plan, not execution evidence.

Compiler validation is always:

```sh
cargo clippy --locked --offline --workspace --all-targets -- -D warnings
```

Use `just lint` equivalently, never `cargo check`. A multi-module change alone does
not justify a full suite. Expand only for a specific uncovered interaction, an
explicit user request or a stated broad closure gate. Documentation-only updates
need source/link consistency and `git diff --check`, not Rust builds.

Closure requires no remaining public deleted-type references or old native-payload
frontend protocol, no compatibility re-exports and no private resurrection of the
old input model. No implementation or test execution is claimed by this plan.
