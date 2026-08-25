# Implementation Plan

**Status**: IN PROGRESS

## Purpose

This document records the active theme-system design and implementation
sequence. The completed API-boundary and output-contract remediation remains
available in Git history through commit `0dcff20`; it is not a second current
plan.

`01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` remain authoritative for the
implemented renderer. `03_THEME_SYSTEM.md` is the approved replacement
architecture, not a second current API. Its custom themes, colors, scales, and
inheritance behavior remain non-current until Phase T6 performs the atomic public
migration. Later implementation phases do not begin merely because they are
listed here.

Status values are `COMPLETE`, `IN PROGRESS`, `PENDING`, and `BLOCKED`. Update a
phase when it starts, completes, changes scope, or discovers a blocker. Complete
and commit each admitted phase as one logical change before starting the next
phase. Split a broad phase into smaller approved phases before implementation
rather than combining unrelated migrations in one commit.

## Confirmed Direction

The following decisions are accepted starting points for the architecture
discussion:

- Publication is the replaceable library default for a standalone plot. It must
  not be forced or repeated as hidden renderer or Canvas state.
- Canvas supplies the root theme for composed output. A plot without an explicit
  theme inherits that Canvas theme; an explicitly themed plot may replace it.
- The first public customization milestone targets coherent full foundations,
  not a color-only API and not an exhaustive property surface.
- Theme values that affect text measurement, layout, legend size, or collision
  planning must resolve before preflight and remain identical during final
  rendering.

Phase T1 settled the exact public types, replacement semantics, component
boundaries, validation, and scope of local overrides in `03_THEME_SYSTEM.md`.

## Reference Philosophy

The approved architecture combines three relevant models:

- ggplot2 supplies the plotting-specific separation between non-data theme
  appearance, data-to-aesthetic scales, and exact layer aesthetics.
- Flutter supplies immutable complete theme data, organized component themes,
  and explicit parent-scope inheritance.
- Vega-Lite supplies typed component configuration and deterministic precedence
  from broad defaults to exact mark settings.

The design should not copy Flutter's widget-context lookup, reactive rebuilding,
animation, or dynamic theme extensions. It should not copy Matplotlib's mutable
process-global runtime configuration or CSS's selector-driven open cascade.

## Target Contract Rules

Phase T1 confirmed these rules for the approved target:

- A theme owns visual defaults. It does not own scientific data, statistical
  behavior, semantic labels, or output routing.
- Data palettes and scales map values to aesthetics. Exact element styles
  configure one caller-owned guide, layer, annotation, or mark.
- Visibility and placement policies remain distinct from appearance when they
  determine whether caller-provided content exists.
- Theme resolution is explicit and immutable. There is no mutable process-global
  theme, thread-local theme, environment-selected theme, or late backend lookup.
- The backend continues to receive complete concrete point, line, fill, text,
  gradient, and bracket styles rather than a public theme object.
- One resolved internal theme is shared by preflight, measurement, collision
  planning, legend construction, and final rendering.
- Public customization remains typed and validated. Do not add arbitrary CSS,
  backend dash strings, string-keyed property maps, dynamic style plugins,
  unvalidated mutable palettes, or silent fallback chains.
- Keep one current API and representation. Replaced theme APIs and private
  representations are migrated atomically without aliases or deprecations.

## Phase Map

| Phase | Objective | Status |
| --- | --- | --- |
| T1 | Approved theme architecture contract | COMPLETE |
| T2 | Complete visual-property ownership inventory | PENDING |
| T3 | One private resolved-theme pipeline | PENDING |
| T4 | Explicit Canvas-to-Plot theme inheritance | PENDING |
| T5a | Surfaces, typography, titles, axes, and grid | PENDING |
| T5b | Legends, facets, and Canvas composition chrome | PENDING |
| T5c | Mark defaults, annotations, and local styles | PENDING |
| T5d | Categorical and continuous data scales | PENDING |
| T6 | Public immutable theme and scale API | PENDING |
| T7 | Contract evidence, documentation, and final audit | PENDING |

## Phase T1: Approved Theme Architecture

**Status**: COMPLETE

Decision record: `03_THEME_SYSTEM.md` defines the complete immutable Theme,
construction-only builder, role-based components, explicit Canvas inheritance,
complete Plot replacement, typed font stacks, RGBA paints, opaque data-scale
colors, deterministic extendable palettes, exact categorical maps, OKLab/OKLCH
color math, transparent surfaces, content-policy boundaries, and exact
shared-legend compatibility. No production code changed in this phase.

Verification: independent architecture review found no remaining specification
contradictions, retired spec-path searches were empty, and `just verify` passed
on 2026-08-26.

Define the model before exposing or moving implementation state.

Scope:

- Add a dedicated approved theme-system target specification, keep
  `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` unambiguous about the
  implemented API, and update `99_DEFERRED_FEATURES.md` for newly admitted and
  still-deferred boundaries.
- Define the exact meanings and boundaries of theme, preset, palette, scale,
  semantic color, component style, element style, visibility policy, renderer
  invariant, and resolved theme.
- Define standalone Plot ownership, Canvas inheritance, explicit Plot
  replacement, facets, text panels, shared headers, and shared-legend behavior.
- Define the complete precedence model across library defaults, Canvas theme,
  Plot theme, plot-specific scale or component configuration, and exact element
  styles.
- Decide whether public `Theme` is a complete immutable value, how callers derive
  a modified theme from a built-in preset, and whether any sparse override type
  has a justified consumer.
- Define which visibility decisions are appearance and which remain separate
  behavior or content policies.
- Define structural validation, palette capacity, missing mappings, fallback
  prohibition, color and alpha representation, contrast ownership, font policy,
  and deterministic SVG/PNG requirements.
- Define a role-based component taxonomy that avoids one independent theme block
  per plot family unless the component is genuinely distinct.
- Define the public-foundations milestone and explicitly defer properties that
  are not yet coherent enough to expose.

Acceptance:

- The approved target specification describes one unambiguous ownership and
  precedence model for standalone plots, Canvas, facets, and shared legends
  without presenting target types as current public API.
- Theme, data scale, exact style, policy, and renderer invariant are distinct
  concepts with no overlapping owner for the same property.
- The contract states when validation occurs and guarantees that preflight and
  final rendering use identical resolved values.
- Every remaining design choice is either decided or named as deferred without
  adding placeholder API.
- No production code changes in this phase.

## Phase T2: Visual-Property Ownership Inventory

**Status**: PENDING

Account for every current visual value before reorganizing implementation.

Scope:

- Inventory every `ThemePalette` field, hardcoded renderer color, font, size,
  width, opacity, dash, radius, spacing, visibility switch, and contrast rule.
- Inventory current public local controls including `CategoricalColors`,
  `GuideStyle`, `PointLayer`, `BarPointLayer`, `InteractionStyle`,
  `SignificanceMarkStyle`, `DumbbellEndpoints`, `HeatmapScale`, legend policies,
  point-outline policy, and sign coloring.
- Record each property's current source, consumers, semantic role, proposed
  owner, inheritance scope, validation, whether it affects layout, and intended
  public override surface.
- Classify data-aesthetic mappings separately from non-data chrome and exact
  element styling.
- Identify duplicated or inconsistent values across plot families, facets,
  legends, annotations, Canvas chrome, and output backends.
- Decide whether each dormant or incorrectly wired field is removed, connected,
  or replaced. Known examples include background, general line width, frame-side
  visibility, and axis-label visibility.
- Identify every intentional visual correction that requires separate snapshot
  evidence rather than hiding it inside structural refactoring.

Acceptance:

- Every visual constant and existing style API has exactly one proposed owner.
- Every layout-affecting style property is marked and traced into measurement.
- No hardcoded renderer value remains unclassified.
- Known wiring defects have explicit remediation phases and expected visual
  impact.
- The inventory agrees with the Phase T1 taxonomy and refines the later phase
  scopes before code migration begins.

## Phase T3: Private Resolved-Theme Pipeline

**Status**: PENDING

Create one internal path for built-in appearance without changing the public API
or intended rendered output.

Scope:

- Replace the current ad hoc `ThemePalette` flow with one complete private
  resolved-theme representation aligned with the approved component taxonomy.
- Make every built-in preset produce a complete validated resolved theme.
- Resolve the active theme once per rendering lifecycle and pass the same value
  through layout, preflight, geometry discovery, legend planning, and final
  rendering.
- Remove repeated owned palette construction and renderer calls back into
  `Plot::palette()`.
- Keep backend traits theme-agnostic and continue passing concrete styles at draw
  boundaries.
- Preserve current intended built-in visuals and snapshots. Handle known visual
  defects only in separately identified changes with explicit evidence.

Acceptance:

- One resolved theme instance governs both preflight and final rendering.
- No renderer independently reconstructs a built-in palette or hidden default.
- Backends remain independent of public theme types.
- Focused lifecycle, layout, theme, and visual-regression tests pass without
  unintended snapshot changes.
- `just verify` passes.

## Phase T4: Canvas Theme Inheritance

**Status**: PENDING

Make composition ownership explicit and consistent.

Scope:

- Give Canvas one replaceable root theme whose default is the same library
  Publication theme used by standalone plots.
- Represent the absence of an explicit Plot theme so a composed plot can inherit
  the Canvas theme without guessing whether Publication was caller-selected.
- Make an explicit Plot theme replace the inherited Canvas theme for that plot.
- Apply the resolved Canvas theme consistently to canvas title blocks, text
  panels, shared headers, row titles, spacing, and other Canvas-owned chrome.
- Make header and text measurement use the same resolved values as rendering.
- Define and enforce exact shared-legend compatibility when panels intentionally
  use different themes.
- Preserve plot erasure and ownership boundaries without global or thread-local
  theme state.

Acceptance:

- Standalone and composed defaults are visually identical when no explicit theme
  is supplied.
- Canvas-owned chrome never falls back to hidden `Theme::Publication` calls.
- Unthemed plots inherit; explicitly themed plots replace; neither behavior is
  sticky outside its owner.
- Measurement and rendering agree for headers, titles, text panels, facets, and
  shared legends.
- Focused standalone, Canvas, facet, and shared-legend tests pass.
- `just verify` passes.

## Phase T5a: Surfaces, Typography, Titles, Axes, And Grid

**Status**: PENDING

Move foundational non-data appearance into the resolved theme.

Scope:

- Centralize canvas and panel surfaces, foreground and muted text roles,
  typography roles, title/subtitle/caption styles, axis lines, ticks, labels,
  frame sides, and grid appearance.
- Remove backend-owned background color and title-block hardcoded grays.
- Separate title or label content presence from theme appearance.
- Wire or remove dormant frame, label, background, and line properties according
  to the approved inventory.
- Ensure every font and spacing value used for measurement is the value used for
  drawing.

Acceptance:

- These components contain no unclassified visual constants.
- Theme changes cannot silently remove caller-provided semantic text.
- Layout and paint use identical typography and spacing.
- Built-in preset changes, if any, have reviewed snapshot evidence.
- Focused layout, axes, typography, output, and visual-regression tests pass.
- `just verify` passes.

## Phase T5b: Legends, Facets, And Canvas Composition Chrome

**Status**: PENDING

Give composed and explanatory chrome coherent theme ownership.

Scope:

- Centralize legend container, title, label, key, outline, opacity, padding, and
  spacing styles while keeping visibility and placement in legend policy.
- Apply role-based typography and surfaces to facet strips, panel headers, shared
  headers, row titles, text panels, and shared legends.
- Preserve exact legend glyph reproduction from resolved mark styles.
- Keep collision planning, outside reservation, and shared-legend compatibility
  synchronized with the resolved theme.
- Remove implicit Publication styling and duplicated fixed whites and grays from
  composition code.

Acceptance:

- Legend and composition measurement consume the exact final styles.
- Shared and per-plot legends use one component contract while retaining exact
  mark glyphs.
- Mixed explicit Plot themes have deterministic documented behavior.
- Focused legend, facet, Canvas, collision, and visual-regression tests pass.
- `just verify` passes.

## Phase T5c: Mark Defaults, Annotations, And Local Styles

**Status**: PENDING

Unify common mark roles without erasing typed local control.

Scope:

- Define common point, line, fill, interval, ribbon, uncertainty, reference,
  outline, and missing-mark defaults by semantic role rather than plot family.
- Classify family-specific constants as shared theme defaults, justified
  specialized component defaults, exact caller styles, or renderer invariants.
- Centralize panel text, significance, value annotation, missing-value, raw
  overlay, and supplemental-layer appearance.
- Integrate existing typed local APIs into the approved precedence model rather
  than adding parallel replacements.
- Preserve exact legend glyph agreement with the final resolved mark styles.

Acceptance:

- Ordinary renderers no longer duplicate unclassified widths, radii, opacities,
  outlines, fills, and neutral colors.
- Exact local styles override only their documented properties and scope.
- Renderer invariants are explicit and cannot be changed indirectly through a
  theme field.
- Focused renderer-family, layer, annotation, legend, and snapshot tests pass.
- `just verify` passes.

## Phase T5d: Categorical And Continuous Data Scales

**Status**: PENDING

Separate data-to-color mapping from non-data theme appearance.

Scope:

- Define typed qualitative, sequential, and diverging palette or scale values,
  including neutral and missing-value behavior where applicable.
- Make built-in themes supply default data scales without conflating those
  scales with foreground, surface, grid, and annotation colors.
- Define exact keyed categorical mappings, ordered palette mappings, continuous
  interpolation, domain ownership, capacity validation, and missing mappings.
- Replace hardcoded heatmap and beeswarm gradients with resolved scale values.
- Generalize exact categorical color control to every plot family that consumes
  the same semantic mapping, or document why a family has a distinct owner.
- Coordinate facets and Canvas global hue order without silently changing exact
  caller mappings.

Acceptance:

- Theme appearance and data scales are distinct types and resolution steps.
- Custom mappings never silently cycle, substitute, or fall back contrary to the
  Phase T1 contract.
- Legends, marks, gradients, colorbars, facets, and Canvas share the same
  resolved mappings.
- Focused categorical, continuous-scale, heatmap, beeswarm, facet, Canvas, and
  legend tests pass.
- `just verify` passes.

## Phase T6: Public Immutable Theme And Scale API

**Status**: PENDING

Expose the complete foundations only after every admitted consumer uses the
private resolved model.

Scope:

- Replace the closed public theme selector with the approved immutable complete
  `Theme` API and built-in constructors or presets.
- Expose only validated component theme, color, typography, and scale types
  required by the full-foundations milestone.
- Add or migrate explicit `Canvas::theme(...)` and `Plot::theme(...)` ownership
  without global state or compatibility aliases.
- Provide a typed way to derive a modified complete theme from a built-in base.
- Migrate current local style and categorical color callers to the one approved
  precedence model; delete replaced public and private representations.
- Keep backend styles private and avoid exposing renderer implementation details.
- Atomically update `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md` to the
  implemented theme contract, mark `03_THEME_SYSTEM.md` current, and reconcile
  `99_DEFERRED_FEATURES.md`; no target/current specification interval remains.
- Update examples and user documentation with standalone, Canvas inheritance,
  Plot replacement, palette, and exact local-style examples.

Acceptance:

- Callers can construct one complete custom theme covering the admitted full
  foundations without editing private renderer state.
- Built-in Publication, Nature, Science, and CHI behavior remains available
  through the single current API.
- The API has no ambiguous global default setter, generic property map, detached
  mutable palette, or alternate compatibility identity.
- Public constructors validate every structural invariant before rendering.
- Compile-pass and compile-fail evidence enforces the intended visibility and
  ownership boundaries.
- All canonical specifications describe the implemented replacement API before
  the phase is complete.
- `just verify` passes.

## Phase T7: Contract Evidence And Final Audit

**Status**: PENDING

Close the phase with exact behavioral evidence and no residual old model.

Scope:

- Add precedence tests for library default, Canvas inheritance, explicit Plot
  replacement, plot scale configuration, and exact element styles.
- Prove one resolved theme is used by preflight and final rendering.
- Cover shared legends, facets, text panels, mixed explicit Plot themes, palette
  capacity, missing mappings, fonts, contrast behavior, SVG, and PNG.
- Retain reviewed deterministic snapshots for every built-in preset and selected
  custom themes.
- Search for retired theme APIs, repeated hidden Publication defaults, dormant
  theme fields, unclassified hardcoded style values, and duplicate scale logic.
- Audit all specifications, README, guides, examples, and rustdoc for residual
  drift from the already-current implemented contract.

Acceptance:

- Phases T1 through T7 and all admitted T5 subphases are complete with commit and
  verification evidence.
- Every visual property has one implemented owner matching the approved
  inventory.
- No mutable global theme, hidden Canvas default, late theme lookup, compatibility
  alias, or unclassified renderer style constant remains.
- Snapshot changes are intentional, reviewed, and committed with their owning
  behavior change.
- `just verify` passes from the completed worktree.

## T1 Decisions

- `Theme` is one complete immutable value. Built-ins are constructors, and a
  construction-only builder produces custom complete themes.
- Canvas owns the root theme. Unthemed plots inherit it; explicit Plot themes
  replace it completely; facets add no scope.
- Components are surfaces, typography, axes, legend, composition, marks,
  annotations, and color scales. Plot-family theme blocks are not admitted.
- `OpaqueColor` owns RGB data-scale color. `Color` owns final RGBA paint with
  `u8` alpha. Alpha replacement is exact and never multiplicative.
- Built-in qualitative palettes preserve curated colors and extend them through
  deterministic perceptual generation. Custom palettes explicitly select fixed
  or extendable capacity and never cycle silently.
- `CategoricalColorMap` is a general exact mapping for raw categorical
  identities. Used keys are required; unused entries and repeated exact colors
  are allowed.
- Sequential and diverging interpolation and palette extension use fixed
  OKLab/OKLCH color math. Missing marks are not palette endpoints.
- Transparent Canvas and panel surfaces are supported in SVG and PNG.
- Built-in contrast is regression-tested; automatic variable-fill text selects
  the better theme candidate; general custom-theme contrast remains caller-owned.
- Fonts use typed ordered stacks and the first installed generation-host family.
- Themes may omit nonsemantic chrome but cannot hide caller-provided semantic
  content. Existing explicit policies own content presence and placement.
- Shared Canvas legends use Canvas chrome and exact panel glyphs; conflicting
  glyphs for one semantic identity reject.
