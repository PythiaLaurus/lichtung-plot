# Deferred Features

## Scope And Review Rules

This specification lists capabilities outside
[the implementation plan](91_IMPLEMENTATION_PLAN.md), with explicit review checkpoints.
It grants no implementation authorization, reserves no API/storage/callback and
does not establish a fallback route. Current behavior is defined by the system
specifications, particularly [Theme](03_THEME_SYSTEM.md),
[layout](05_LAYOUT_SYSTEM.md), [numbers](06_NUMERIC_PRESENTATION.md),
[property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md) and
[guides](08_GUIDE_SYSTEM.md).

DF7, DF8, DF9 and DF10 are pending user disposition. Their checkpoint is the
next feature-scope discussion before their successor implementation planning.
DF11 has its separate prerequisite-based checkpoint below.
No additional general audit is a prerequisite. Discussion itself does not admit
or reject a capability.

At review, present concrete workflows, unmet requirements, alternatives and
semantic/performance tradeoffs. Obtain user approval to admit a bounded scope,
reject it with a lasting boundary, or re-defer with a new explicit checkpoint.
No retained item may have an expired or vague checkpoint. IDs are not reused.
When scope enters the implementation plan, remove it from this inventory in the
same change, including any admitted subset of a broader item. Update current
contracts when implemented. A plan is not implementation evidence. Record lasting
requirements in the owning contract, not a permanent decision ledger.

## DF7: Sparse Composition Compaction

Excluded: repacking occupied cells or collapsing empty rows/columns/trailing wrap
slots. Natural content sizing does not change declared topology. Empty Cartesian
combinations can carry scientific meaning; hand-authored Canvas can declare a
compact shape explicitly.

**Checkpoint:** next feature-scope discussion. Assess removing wholly empty tracks
separately from occupied-cell repacking/observed-combination galleries. Specify
source/destination identity, occupancy, metadata-only tracks, spans, row titles,
range groups/shared labels and partial final wrapped rows before claiming benefit.

## DF8: Caller-Selected Layout Objectives

Excluded: caller objectives for aspect, density, whitespace, guide flow, text measure
or Grid balance; scoring callbacks, strategy switches, string modes and layout plugins.

**Checkpoint:** next feature-scope discussion. Require an unmet workflow under
the deterministic private local policy, including exact-root failures. Compare
a small typed objective's complete semantics/cost without implying global search.

## DF9: Global Layout Optimization

Excluded: exhaustive whole-tree search, sibling Cartesian products, all text-break
partitions and root-wide dense-rank objectives. Bounded containing-frame guide
recovery is current and distinct.

**Checkpoint:** next feature-scope discussion alongside DF8. Require demonstrated
scientific workloads, measured comparison with current bounded recovery, semantic
completeness, determinism and a coherent replacement contract. No dormant solver
or runtime strategy selector is reserved.

## DF10: Stop At First Fitting Line Proposal

Excluded: removal of exact immediately-next-endpoint extension. Current wrapping
proposes forward, retreats to a fit, then extends until first next-endpoint nonfit.
No alternate strategy switch is provided.

**Checkpoint:** next feature-scope discussion. Compare actual release workloads,
line endpoints, caption heights, tight-root fit, bidi, soft hyphens and retained
glyph geometry. Changing this behavior requires user acceptance of the wrapping/
fit tradeoff, not simply a smaller internal trial count.

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
