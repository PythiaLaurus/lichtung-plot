# Retained Mark Geometry Resolution

The complete mark extent values now drive their actual consumers. This is the
private consumer migration preceding public component construction, not a second
geometry or theme representation.

## Selected Geometry And Owners

- Vertical/horizontal Bar single and stacked bodies select their own breadth;
  grouped bodies select their own group span and retain the existing
  renderer-owned Comfort conversion. Bodies, error caps, supplemental points,
  attached marks and pairwise anchors use the same selected centers.
- Waterfall independently selects its body breadth. One derivation computes
  `free = 1 - breadth`, `low = free / 2`, `high = 1 - low` and connector room
  `free`. Intrinsic pitch, scientific and physical body edges, and each adjacent
  connector use these operands in the operation order specified by spec94.
  The default binary64 operands retain their original bits.
- Required outward ink divided by actual containing room supplies a hard pitch.
  Positive ink without room refuses; absent/transparent ink contributes zero
  without evaluating `0/0`. Constructible breadth one is therefore not rejected
  solely because it touches the category boundary. Final canonical-span and
  complete-ink validation remain active.
- PointInterval intrinsic occupancy and drawn caps use the same selected integral
  cap half-height.
- Sampled Histogram rug height selects the owning theme's relationship to the
  prepared scientific maximum. `PreparedReference` and facet preparation carry
  the complete owner before fonts; Heatmap still selects that owner's color
  scales. Supplied Histogram geometry remains authoritative and unchanged.
- Marginal top/right strips and Matrix diagonal densities select their respective
  local density fractions. KDE, observations, source domains, stroke insets and
  assignment remain their existing owners.

The immutable full mark value remains part of resolved appearance identity used
by measured child reuse. Captured mark plans retain final draw operands; replay
does not reconstruct geometry from presets.

## Meaningful Test Replacement

Two tests replace two old library fixtures; total discovery stays **671**, with
the existing 17 ignored tests unchanged:

- The Waterfall selected-payload test replaces the local Axis clone-identity
  fixture and removes its private clone helper. It checks unchanged scientific
  endpoints, selected body breadth, actual connector attachment, and positive-ink
  refusal at breadth one.
- The sampled-rug owner test replaces a KDE maximum self-confirmation fixture
  and removes the test-only duplicate maximum implementation. It checks unchanged
  bins/KDE, complete original observation order including duplicates, the new rug
  height and range, and a successful actual preflight with the custom owner.

## Verification

Strict `just lint`, `just fmt` and `git diff --check` pass. Focused evidence:

- 12 library tests: categorical preferences, complete mark/annotation values,
  exact Waterfall shelf reserves and final missing-row text requirements.
- 3 library tests: selected Waterfall geometry, Histogram independent bin/overlay
  ink and owner-selected sampled rugs.
- 1 library test: facet producer capture retains plans without reshaping.
- 12 integration tests across annotations, categorical marks, distributions,
  estimates and Marginal/Matrix: grouped spans/gaps, exact attached marks,
  supplied/sample Histogram contracts, cap geometry and scientific completeness.
- 8 composition integrations: shared Heatmap guide, faceted extreme stacked
  values and all six explicit/inherited/default theme-owner contracts.
- `cargo test --locked --offline --lib -- --list`: 671 discovered tests.

These bounded checks cover the changed call paths. No whole-scene golden was
repinned and no complete-suite run was needed for this leaf.
