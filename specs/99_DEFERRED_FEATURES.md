# Deferred Features

## Scope And Review Rules

This specification records user-requested deferred reviews beyond the current
renderer contracts, with explicit review checkpoints.
It grants no implementation authorization, reserves no API/storage/callback and
does not establish a fallback route. Current behavior is defined by the system
specifications, particularly [Theme](03_THEME_SYSTEM.md),
[layout](05_LAYOUT_SYSTEM.md), [numbers](06_NUMERIC_PRESENTATION.md),
[property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md) and
[guides](08_GUIDE_SYSTEM.md).

DF11 is the only deferred item, with its prerequisite-based checkpoint below.
Renderer exclusions, possible extensions and agent suggestions do not by themselves
create deferred work. Discussion itself does not admit or reject a capability.

At review, present concrete workflows, unmet requirements, alternatives and
semantic/performance tradeoffs. Obtain user approval to admit a bounded scope,
reject it with a lasting boundary, or re-defer with a new explicit checkpoint.
No retained item may have an expired or vague checkpoint. IDs are not reused.
When scope enters an implementation plan, remove it from this inventory in the
same change, including any admitted subset of a broader item. Update current
contracts when implemented. A plan is not implementation evidence. Record lasting
requirements in the owning contract, not a permanent decision ledger.

## DF11: Scientific Plotting For AI-Authored Workflows

Review the library's positioning and capabilities for AI-authored deeply customized
figures: bespoke drawing scripts, low-level raster/vector libraries, generated-SVG
editing, custom annotations/shapes and manually composed multi-panel output.
A fixed plot catalog's value must be evaluated against those actual workflows,
including how established scientific libraries support them.

Include **fully caller-decided legends** independent of automatic contributions:
titles/labels/selection/order, point/line/patch symbols, exact RGBA/geometry,
continuous/discrete explanations and append/replacement on Plot/Canvas. Compare
declarative symbol-and-label inputs with caller-drawn graphics; no interface is
preselected. Custom alpha functions with caller-provided explanation belong here,
not automatic inference from opaque functions or observed bytes.

**Checkpoint:** after **all five** prerequisites are complete:

1. Refactor into a Rust workspace.
2. Complete the polars adapter.
3. Complete the ndarray adapter.
4. Complete the Python calling interface.
5. Publish the first version.

Review begins only after the last prerequisite. These name future timing and do
not authorize those projects or alter the standalone renderer boundary.

Evaluate representative single/composed scientific workflows, low-level drawing,
SVG postediting, added graphics and fully authored legends. Compare automatic
explanations with independent declarations, Plot/Canvas ownership, measurement/
placement/fonts/complete ink, coordinate access, source identity, immutable output,
discoverability and composability. Identify reliable scientific/statistical/text/
layout value versus work forced into external reconstruction. Present concrete
alternatives and their determinism/resource/format consequences before any API
proposal. No plugin, draw callback, mutable scene or SVG-editing route is reserved.
