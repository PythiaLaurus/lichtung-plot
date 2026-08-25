# Implementation Plan

**Status**: Active remediation ledger

## Purpose

This document records the current API-boundary remediation sequence. The
completed implementation history remains available in Git through commit
`355155a`; it is not a second current plan.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative. This
ledger closes implementation gaps against those contracts and does not
authorize compatibility aliases, deprecated replacements, alternate data
representations, or dormant public extension points.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each phase as one logical change before starting the next phase.

## Contract Rules

- Keep one current API and migrate every repository caller atomically.
- Delete replaced methods, variants, modules, test shims, and report text. Do
  not retain aliases, compatibility wrappers, or private copies without a
  current renderer consumer.
- Public modules are deliberate extension points. A root re-export does not
  require its defining module to be public.
- Only an exact plot-family marker may cross the explicit erasure boundary.
  `kind::Any` is the result of erasure, not an input to erasure or Canvas
  insertion.
- Tests construct renderer-native values. They do not recreate a generic
  dataframe or string-indexed column conversion layer inside this repository.
- Run focused checks while iterating. Before completing each phase, inspect any
  snapshot changes and run `just verify`.

## Remediation Inventory

| Issue | Required outcome | Phase | Status |
| --- | --- | --- | --- |
| 1 | Internalize `decorations` and `TitleBlock` | R1 | PENDING |
| 2 | Delete overlay and regression report-provenance APIs | R1 | PENDING |
| 3 | Delete public kind and theme inventories and report naming | R1 | PENDING |
| 4 | Restrict erasure and Canvas insertion to exact families | R2 | PENDING |
| 5 | Remove duplicate default PNG compression identity | R3 | PENDING |
| 6 | Expose `Theme` only from the crate root | R1 | PENDING |
| 7 | Use `HistogramKdePoint` for every precomputed histogram KDE | R4 | PENDING |
| 10 | Replace dataframe-shaped test shims with native values | R5 | PENDING |

Issues 8, 9, and 11 require explicit contract decisions and are not authorized
implementation work in this plan.

## Phase R1: Public Surface Cleanup

**Status**: PENDING

Remove accidental module exposure and application/report-oriented
introspection while preserving renderer-owned behavior.

Scope:

- Make `decorations` crate-private. Make `title_block` and `TitleBlock`
  crate-private and retain them only as renderer implementation details.
- Make `theme` crate-private while retaining the root-level
  `lichtung_plot::Theme` export.
- Delete `RawOverlay::provenance()` and its fixed report strings. Retain
  `is_visible()` because rendering consumes it.
- Delete `RegressionBand::provenance()` and its fixed report strings. Retain the
  crate-private legend wording because the renderer consumes it.
- Delete `PlotKind::ALL`, `PlotKind::variant_name()`, `PlotKind::label()`, and
  `PlotKind::key_segment()`.
- Replace the validator's sole `PlotKind::key_segment()` use with diagnostic
  wording local to that validation concern. Do not introduce a general kind
  naming or report helper.
- Delete `Theme::ALL` and `Theme::variant_name()`. Rewrite internal theme tests
  to exercise the explicitly named built-in themes without restoring a
  runtime inventory API.
- Remove tests and documentation whose only purpose was to guarantee the
  deleted report, inventory, or public-module surface.

Acceptance:

- `lichtung_plot::Theme` remains usable, while `lichtung_plot::theme` and
  `lichtung_plot::decorations` are not public paths.
- No callable provenance method or fixed report-provenance text remains on
  `RawOverlay` or `RegressionBand`.
- No general inventory, variant-name, report-label, or generated-key API
  remains on `PlotKind` or `Theme`.
- All internal title, subtitle, caption, legend, and regression-band rendering
  behavior remains intact.
- Compile-fail API-boundary coverage and focused library, lifecycle, and theme
  tests pass.
- `just verify` passes.

Commit: `refactor(api)!: remove accidental introspection surface`

## Phase R2: Exact-Family Erasure

**Status**: PENDING

Make the architecture's exact-family boundary mechanically enforceable.

Scope:

- Add one sealed exact-family marker bound implemented by every concrete plot
  family and not implemented by `kind::Any`.
- Keep `kind::Marker` as the bound admitted by `Plot<K>` itself.
- Restrict `Plot::erase()` to exact family markers.
- Restrict `Canvas::panel()` to exact family markers and continue erasing only
  at Canvas's private heterogeneous-storage boundary.
- Do not add a second erasure method, an erased-panel insertion method, a
  runtime escape hatch, or an implementation for `kind::Any`.

Acceptance:

- Every concrete constructor result can call `erase()` and can be passed to
  `Canvas::panel()`.
- `Plot<kind::Any>` can do neither, including a second call to `erase()`.
- Erasure still moves all owned input and configuration without changing plot
  identity or render output.
- An erased plot retains its common `kind()`, `width()`, `height()`,
  `preflight()`, and terminal-output operations; compile-pass coverage exercises
  those operations after erasure.
- Compile-fail doctests cover both rejected erased-input operations.
- Focused builder and composition tests pass.
- `just verify` passes.

Commit: `feat(api)!: enforce exact-family erasure`

## Phase R3: Canonical PNG Compression

**Status**: PENDING

Remove duplicate public identities that encode the same current behavior
without deciding the separate SVG-only API question.

Scope:

- Delete `PngCompression::Default`.
- Make `PngCompression::Balanced` the enum default and make
  `PngOptions::default()` resolve to `Balanced`.
- Migrate every repository caller, test, example, and document from `Default`
  to `Balanced`; do not retain an alias or decoder.
- Preserve the current exposure of PNG option types in SVG-only builds. Whether
  that exposure should change remains deferred issue 8 and is not part of this
  phase.

Acceptance:

- Exactly one public balanced PNG compression identity exists, and it is the
  default.
- All compression presets retain their existing encoder mappings apart from
  removal of the duplicate variant.
- The public availability of PNG option types is unchanged in minimal
  SVG-only and default PNG builds.
- Focused output tests pass in both the no-default-features and default-feature
  configurations.
- `just verify` passes.

Commit: `refactor(api)!: remove duplicate png compression`

## Phase R4: Canonical Histogram KDE Points

**Status**: PENDING

Use one named public point representation for caller-computed histogram KDE
geometry.

Scope:

- Change `HistogramKde::Precomputed` from `Vec<(f64, f64)>` to
  `Vec<HistogramKdePoint>`.
- Migrate validation, rendering, tests, examples, and documentation to named
  `x` and `y` fields. Do not retain tuple conversion as a second project-owned
  representation.
- Preserve the distinction between a precomputed KDE layer over sample-owned
  histogram input and complete self-contained `HistogramGeometry`.
- Do not change source-row or facet ownership. Complete precomputed histogram
  geometry continues to reject sample-only channels before backend begin.

Acceptance:

- All precomputed histogram KDE paths use the same public point type as
  `HistogramGeometry::kde`.
- Invalid KDE ordering, non-finite coordinates, negative density, and safety
  limits retain focused coverage through the named point representation.
- Sample-derived and complete-geometry histogram modes retain their existing
  validation and rendering behavior.
- Focused histogram, geometry, distribution, and facet tests pass.
- `just verify` passes.

Commit: `refactor(api)!: unify histogram kde points`

## Phase R5: Native Test Values

**Status**: PENDING

Remove test-only dataframe emulation so tests exercise the standalone
renderer boundary directly.

Scope:

- Delete the unit-test `Fixture`, `FixtureColumn`, `DataFrame`, `Series`,
  `FixtureField`, `IntoFixtureColumn`, and `test_data!` surface.
- Delete the integration-test equivalents and the `df!` macro from
  `tests/support/data.rs`.
- Construct `NumericValues`, `CategoryValues`, `BooleanValues`, `SourceRows`,
  and `AxisValues` directly at test call sites.
- Where repeated setup is substantial, use narrowly named family-specific
  structs or helpers whose fields are native values. Do not add generic named
  columns, dynamic type dispatch, conversion by column-name string, dataframe
  aliases, or a replacement table macro.
- Preserve intentional nullable values, source-row identities, category order,
  and mismatched-length cases explicitly rather than normalizing them in a
  fixture layer.
- Remove now-empty test support modules and imports.

Migration order:

1. Migrate crate unit tests by renderer family and delete `src/test_support.rs`.
2. Migrate integration suites by renderer family and delete
   `tests/support/data.rs` when its final consumer is gone.
3. Search the full repository for every retired shim name and macro before
   completing the phase.

Acceptance:

- No test-owned generic dataframe, series, fixture-column, dynamic column
  lookup, `df!`, or `test_data!` abstraction remains.
- Tests continue to distinguish numeric, categorical, boolean, axis, and
  source-row semantics through production native types.
- Existing invalid-input tests still construct the invalid state they claim to
  exercise rather than failing earlier in test setup.
- Preserve every existing behavioral case, assertion, deterministic snapshot,
  and exact-byte or hash check. Delete only coverage whose sole subject is a
  retired fixture abstraction.
- Focused suites pass as each family migrates.
- `just verify` passes with no unintended snapshot changes.

Commit: `refactor(tests)!: use native renderer values`

## Deferred Decisions

The following findings remain outside implementation scope until their current
contract is selected. Do not add placeholders for any option.

### Issue 8: PNG Options In SVG-Only Builds

Decide whether PNG-specific configuration is feature-gated, remains inert and
documented in SVG-only builds, or moves to a terminal PNG configuration
boundary. The decision must account for API consistency across feature sets and
must not create fallback output behavior.

### Issue 9: Supplemental Point-Layer Semantics

Decide whether `PointLayer` retains optional point labels and legend identity or
becomes fixed supplemental geometry without legend participation. Current
documentation that says the layer never creates a legend conflicts with
`legend_label()` and renderer behavior; implementation and documentation must
adopt one exact contract.

### Issue 11: Numeric Label Formatting

Decide whether `NumericLabelFormat` remains precision plus suffix, gains a
prefix, or adopts another closed deterministic formatting model. Arbitrary
callbacks are not assumed because they complicate ownership, cloning, and
deterministic rendering.

## Final Completion

**Status**: PENDING

The currently authorized remediation is complete only when:

- Phases R1 through R5 are `COMPLETE` with commit and verification evidence.
- Repository searches find no retired API names, report strings, compatibility
  aliases, tuple KDE representation, or dataframe-shaped test shims.
- Public-path compile-fail tests enforce private modules and exact-family
  erasure.
- `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` describe the implemented
  current contract without documenting deferred choices as current behavior.
- `just verify` passes from the completed worktree.

Completion of these phases does not close the overall API review. Issues 8, 9,
and 11 remain decision blockers; in particular, the issue 9 documentation and
renderer behavior conflict must be resolved before final review closure.
