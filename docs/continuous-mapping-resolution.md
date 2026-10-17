# Complete Continuous Mapping Resolution

## One Mapping And Final Domain

`ContinuousColorScale` selects its complete matching owner gradient before
preparation. Its common domain calculation handles exact and observed sequential
domains and the checked symmetric-around-center diverging calculation, preserving
the explicit-domain remedy for unrepresentable automatic symmetry.

`ContinuousColors::selected` maps the full semantic gradient into the final
domain. Interior semantic stops retain spec94's exact named binary64 operation
order; endpoints and the unique neutral map exactly. Endpoint centers consume
only their noncollapsed semantic branch, and a constant-at-center domain consumes
neutral. Non-strict/nonfinite remapped offsets refuse rather than being repaired.
The exact center returns the retained neutral directly.

Heatmap preparation and Shared guide preparation both use this mapping. Their
three-stop-only restrictions and duplicate remapping implementations are removed.
Beeswarm preparation accepts the same complete validated selection, including
diverging gradients, exact/automatic domains and center semantics. Current public
inputs adapt once into that prepared value; public complete-scale attachments
remain the later atomic API cutover.

Shared installation always projects the semantic selection against the final
merged domain, not against previously projected local stops. Marks and guides
share the resulting opaque RGB mapping. Existing observed-color sampling and
six-decimal guide-stop collision refusal remain active. Point/cell alpha is a
separate role authority and does not become gradient-stop alpha.

## Labels And Defaults

One domain-label selector prepares the required low, optional interior center and
high values, or one label for a constant domain. Heatmap local guides, Beeswarm
local guides, Shared descriptors and their pre-font requirements use that order.
Beeswarm's two-string array and hardcoded absent center are removed.

All four built-ins now use their common `#F7FBFF -> #08306B` sequential gradient
for both Heatmap and Beeswarm. The old Beeswarm-only `#3182BD -> #DE2D26` source is
deleted. This implements the approved intentional Beeswarm color change; the
existing endpoint contract is updated to those independently specified colors.
The diverging default remains unchanged.

## Evidence

One actual prepared/Shared/selected-Beeswarm contract replaces the local
ColorScales component-edit inventory. It selects a five-stop gradient from its
owner, verifies every initial observation color, merges a compatible exact
asymmetric sibling domain, checks both extra branch-stop positions, installs the
final mapping and verifies actual selected points plus center-label presence.
Gradient samples retain the same RGB as each mark while mark alpha remains
independent. Discovered library count stays **671**, including the unchanged
17 ignored tests.

Strict `just lint`, formatting and whitespace checks pass. Focused checks:

- The new prepared/Shared/actual-mark contract.
- Twenty-five related library tests: complete typed scales, actual owner gradient,
  continuous guide fit/capture/cache, final-domain contrast, custom cell alpha and
  categorical mark preferences.
- The existing single lifecycle test exercises all 25 canonical families after
  changing pre-font continuous-owner transport.
- Eleven Heatmap integrations: endpoints, endpoint centers, constant domains,
  signed zero/subnormal/finite extremes, explicit saturation, all-missing exact
  domains, asymmetric guide/raster relationships, pre-font invalid choices and
  equal local/Shared numeric labels.
- Four Shared Heatmap/Beeswarm integrations plus the deterministic Beeswarm
  default/output contract.

The localized failing constant-domain check exposed a lost descriptive error
phrase. The shared typed constructor now reports that the exact domain must
contain center, and the focused check passes. No scene golden was repinned and
no full suite was used for this leaf.
