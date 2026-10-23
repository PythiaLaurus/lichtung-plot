# Deferred Features

## Scope And Review Rules

This specification identifies excluded capabilities and explicit review checkpoints.
It grants no implementation authorization, reserves no API/storage/callback and
does not establish a fallback route. Current behavior is defined by the system
specifications, particularly [Theme](03_THEME_SYSTEM.md),
[layout](05_LAYOUT_SYSTEM.md), [numbers](06_NUMERIC_PRESENTATION.md),
[property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md) and
[guides](08_GUIDE_SYSTEM.md).

DF7, DF8, DF9, DF10 and the residual scope in DF6 are pending user
disposition. Their checkpoint is the next feature-scope discussion **after
specification/documentation cleanup and before their successor implementation
planning**. Extended numeric presentation/tick generation, explicit category-label
selection, unified text controls with general anchoring/rotation/ellipsis/font fit,
and maximum-width/automatic-height sizing have been requested for planning as
described below. Public configuration naming follows the confirmed rule in
[the implementation plan](91_IMPLEMENTATION_PLAN.md).
No additional general audit is a prerequisite. Discussion itself does not admit
or reject a capability.

At review, present concrete workflows, unmet requirements, alternatives and
semantic/performance tradeoffs. Obtain user approval to admit a bounded scope,
reject it with a lasting boundary, or re-defer with a new explicit checkpoint.
No retained item may have an expired or vague checkpoint. IDs are not reused.
Remove admitted scope from this list when its plan is approved; update current
contracts when implemented. Admission is not implementation evidence. Record
lasting requirements in the owning contract, not a permanent decision ledger.

## DF2: Locale And Extended Numeric Formatting

NumericFormat already covers locale-free fixed/adaptive fractional/significant
precision, Scientific/Engineering notation, exact decimal-power scaling, literal
affixes and zero handling, including mathematical Forest powers. These are current,
not deferred scope. Existing omitted profiles already use prescribed k/M branches;
this is distinct from publicly configurable SI/compact rules.

**Requested planning:** support the complete extended formatting scope: explicit
named/custom locale digits/separators/grouping/currency, configurable per-value/
fixed/whole-set SI and compact rules, general display multipliers and rounding,
shared multiplier/unit explanations, D3/Plotly-style numeric specifiers and owned
single-value/complete-set functions while retaining automatic tick positions.
Keep structured NumericFormat and one exact/certified conversion route. Do not
reduce the requested capabilities to fit the current finite-font-alphabet model.
See [the implementation plan](91_IMPLEMENTATION_PLAN.md) for actual-source font
admission/final sealing, callback identity, measured explanations, composition and
verification. The new APIs are not implemented.

**Plan-review checkpoint:** review the integrated numeric proposal before its
implementation begins, including explicit locale provenance, D3 syntax versus
rounding semantics, mathematical Forest operands, final Shared domains, callback
materialization and font/geometry lifecycle. This is review of the requested full
scope, not a fresh undecided feature-scope discussion. Remove the admitted scope
from this list when its plan is approved and update current contracts as implemented.

## DF4: Further Automatic Tick Controls

Current: TickGeneration Automatic, preferred Additive/Log10Decades, approximate
Target and hard Maximum, plus verbatim exact ticks. Generated position selection
is separate from NumericFormat's spelling and formatting functions.

**Requested planning:** implement all five discussed controls through the existing
axis/formatting/root lifecycle, with concrete workflows in
[the implementation plan](91_IMPLEMENTATION_PLAN.md):

- Exact generated counts: four endpoint-including positions across [0,10], rather
  than an approximate Target or an upper bound.
- Relative automatic density: sparse/report versus dense/large-figure views while
  retaining span/typography-responsive automatic spacing.
- Anchored multiplicative intervals: [1,2,4,8,16,32,64] for ratio 2, including other
  caller-selected ratios and explicit anchor identity.
- Category-label stride/offset/endpoints: twelve month bars with six selected names,
  preserving all categories, marks, tick positions and grid geometry.
- Owned numeric position functions: include view endpoints and an in-range warning
  value 50 after final domain resolution; pass resulting positions to NumericFormat.

The plan covers one normalized generator/refinement policy, strict versus soft
source behavior, actual-axis capabilities, full category-domain/selected-text
ownership, callback identity/materialization, facets/Shared ranges, spanning Canvas
and frozen publication. Exact ticks retain precedence in either setter order.
The capabilities are not implemented; explicit category-label selection does not
authorize automatic fit-driven omission or removal of scientific categories.

**Plan-review checkpoint:** review the integrated tick proposal before implementing
it, including exact-count endpoints/scale space, density/cap combinations,
geometric boundary arithmetic, selected-label font/native-cell requirements and
raw-versus-exponent callback coordinates. This reviews the requested scope rather
than reopening its feature disposition. Remove admitted scope from this list when
its plan is approved and update current contracts as implemented.

## DF5: Broader Text Layout Controls

Current: complete role typography, Plot heading instructions, Canvas panel defaults,
independent captions, shared Unicode legal wrapping and explicit Heatmap Wrap/Elide.
Public `Labels::humanized()` is the existing default policy for inferred names;
the single-string resolver is private. Display-input validation and local styling/
alignment are not yet consistent across authored text slots.

**Requested planning:** unify authored plain-text input, explicit newlines, local
complete typography and horizontal alignment, including independent TextPanel
title/body controls. Expose single-string label resolution while preserving the
existing default humanization and Raw/alias precedence. Include all four additional
controls as one text-system extension:

- General anchoring: keep a two-line label's bottom-left corner at its data
  reference or center a text block in its assigned cell as content/size changes.
- Explicit rotation beyond category labels: a 30-degree point note, horizontal
  y title and numeric/exact/category ticks share one canonical transform.
- General ellipsis: a fixed-width guide shows `Treatment group…`, retaining full
  original source, identity and original-versus-displayed mappings.
- Automatic font shrink: a 140px title prefers 14px and permits a 10px floor;
  layout selects a fitting candidate while independent text keeps its own policy.

See [the implementation plan](91_IMPLEMENTATION_PLAN.md) for one TextLayout request
beside complete TextTheme appearance, owner-specific attachments, reference/anchor
geometry, Heatmap migration, Unicode source views, bounded size/group fitting,
actual font admission and frozen publication. Ellipsis/shrink require the owner's
explicit permission except for Heatmap's existing dense-grid elision default.
These APIs are not implemented; update current contracts when implemented and
remove admitted scope here when its plan is approved.

Emergency word/identifier/grapheme splitting and caller-specified internal-break
rules are rejected, rather than deferred candidates. Preserve ordinary Unicode
wrapping and caller-authored breaks. Complete explicit/raw names stay intact under
the complete-text policy; explicit ellipsis derives a display view without adding
internal-word line breaks or changing raw identity. Default humanization remains
preparation, never a fit alternative. The lasting no-internal-break boundary is
in [layout](05_LAYOUT_SYSTEM.md); the new controls remain planned, not current.

**Plan-review checkpoint:** review the integrated text proposal before implementing
it, including typed reference/anchor meaning, absolute versus inherited angles,
ellipsis Unicode/bidi/source provenance, complete-before-abbreviated font fitting,
local size bounds/grouping, existing exact annotation styles, font admission and
parent/root/cache/output consequences. This reviews the requested full scope, not
a new feature-disposition discussion. Remove admitted scope when its plan is
approved; automatic angle selection and emergency word splitting remain outside it.

## DF6: Broader Root Bounds

Excluded: maximum width, minimum width/height, maximum height, exact-height/automatic-
width and public general constraint objects. Automatic, exact-width/automatic-
height and exact-pair requests are current.

**Maximum-width planning:** the user requested a bounded `max_width(u32)` plan for
Plot and Canvas, including owning facets, with automatic height and a natural
preferred width capped once. See [the implementation plan](91_IMPLEMENTATION_PLAN.md).
The API is not implemented. Remove this subset from the deferred scope when its
plan is approved; update current contracts with the implemented behavior.

**Residual checkpoint:** next feature-scope discussion for the remaining bounds.
Compare real publication-column/embedding needs against current root modes and
the proposed width cap, including wrapping, aspect, guides, facets/Canvas and
strict fit. A general bounds surface is not the presumed outcome.

## DF7: Sparse Composition Compaction

Excluded: repacking occupied cells or collapsing empty rows/columns/trailing wrap
slots. Natural content sizing does not change declared topology. Empty Cartesian
combinations can carry scientific meaning; hand-authored Canvas can declare a
compact shape explicitly.

Explicit rectangular Canvas Plot spans are requested for separate bounded planning
in [the implementation plan](91_IMPLEMENTATION_PLAN.md). They retain caller-declared
rows/columns and one owner per placed Plot. This planned capability is not automatic
compaction or observed-cell repacking and does not admit the excluded scope here.

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
