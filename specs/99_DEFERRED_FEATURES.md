# Deferred Features

**Status**: Deferred and non-current

## Contract

This document records only genuinely excluded scope, not approved work awaiting
implementation. It grants no implementation authorization and reserves no API,
storage, callback, or fallback. Approved boundaries are in `03_THEME_SYSTEM.md`,
`05_LAYOUT_SYSTEM.md`, T5a0s/D1-D6 in `91_IMPLEMENTATION_PLAN.md`, and the
user-approved T6 decisions in `93_T6_EXECUTION_PLAN.md` and the alpha/tick
extension admission in `97_ALPHA_AND_TICK_EXTENSION_PLAN.md`.

Every item has a stable ID and an explicit review checkpoint enforced by the
entry/closure gates or named successor milestones in `91_IMPLEMENTATION_PLAN.md`.
Review is not an implementation promise and does not automatically expand T6.
At its checkpoint, present the item's evidence and tradeoffs to the user and
record one user-approved outcome:
admit with an approved new phase, reject with a reason, or re-defer with a new
explicit milestone checkpoint. A vague future review does not close a gate.

When work is admitted into an approved implementation plan, remove the admitted
feature from this file immediately, retaining a broader heading only for related
excluded scope. Do not wait for code to land. Separately, update the current
architecture/rendering contracts atomically when implementation lands; planning
or removal here never makes an API current. Future updates must preserve explicit
checkpoints and evidence/decision records for every remaining item.

After a review, no retained item may keep an expired checkpoint. Re-deferral
updates its checkpoint; rejection removes it from this active review list and
records the reason and any lasting exclusion in the plan or canonical contract.
Stable IDs are not reused. Related unadmitted scope may retain its ID only with
an explicit next checkpoint.

## DF2: Locale And Extended Numeric Formatting

Locale-aware digits/separators/grouping, caller-selected compact/SI-prefix rules,
arbitrary display multipliers, configurable rounding modes, shared multiplier or
unit annotations, general format strings and callbacks remain excluded.

The user admitted the locale-free scientific core at T6 ENTRY: fixed fractional
and significant precision, scientific/engineering notation, exact decimal-power
display scaling including percentage, literal affixes, negative-zero/trailing-zero
policies and bounded adaptive precision across the actual numeric consumers.
Configured Forest Log10 ticks format the underlying mathematical value, as
explicitly selected by the user. This admitted scope is implemented by public
`NumericFormat` and its closed discriminants/attachments; evidence is in
`../docs/public-numeric-formatting.md`. It is separate from the residual deferred
capabilities above.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Use implemented T6 numeric policies on real scientific
figures, including scaling, units, close values, Forest logarithmic axes, shared
guides and exact-width wrapping, to identify unmet needs. For locale proposals,
specify pinned data, Unicode/bidi/font coverage and reproducibility rather than
ambient host settings. Compare the residual capabilities' semantics and bounded
costs, then obtain user-approved new-phase admission, reasoned rejection, or
re-deferral with an explicit new checkpoint.

## DF4: Further Automatic Tick Controls

Exact generated counts, visual-density scalars, arbitrary multiplicative ratios,
categorical stride/drop policies, and generation callbacks remain excluded.
The user admitted target and maximum generated numeric tick counts following the
T6 closure review. Their implementation phases and the distinction between a
target and a hard maximum are in `97_ALPHA_AND_TICK_EXTENSION_PLAN.md`; this
residual item does not defer that admitted work. The implemented preferred-interval
boundary remains in `05_LAYOUT_SYSTEM.md` until the owning public cutover.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Evaluate the admitted target/maximum controls alongside
preferred intervals and exact caller ticks on small panels, facets, and shared
Canvas dimensions. Identify needs beyond that combined surface. Review exact
counts separately from nice-step selection, and categorical label policies
separately from numeric generation. Compare precedence, semantic completeness,
and bounded costs without a second planning route; obtain user-approved new-phase
admission, reasoned rejection, or re-deferral with an explicit new checkpoint.

## DF5: Broader Text Layout Controls

TextPanel title/body horizontal alignment, explicit categorical-axis directions,
general per-role alignment/anchoring, arbitrary rotation, emergency identifier
breaking, general ellipsis and automatic font shrink remain excluded.

T6 ENTRY review found complete private heading precedence/typography, strict legal
wrapping, fixed role orientations, corner annotations and the current Heatmap
Wrap/Elide policy. The user approved completing the existing T6 public typography,
title/subtitle overrides, Canvas panel defaults and independent theme captions,
and re-deferred the additional controls. See `93_T6_EXECUTION_PLAN.md`; those
existing approved capabilities are not missing DF5 implementation.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Evaluate the implemented public typography and sizing
controls first. Review TextPanel alignment, bounded categorical orientations and
opt-in identifier breaking as distinct candidates, using narrow identifiers,
crowded labels and mixed-direction multiline text. Assess general anchors,
ellipsis and shrink separately, including source preservation, readability,
physical versus bidi-logical direction, measured ink, cache/feedback identity,
resource bounds and frozen SVG/PNG geometry. Obtain a user-approved disposition
for each admitted subset; default strict wrapping is not relaxed by this review.

## DF6: Broader Root Bounds And Maximum Width

Maximum-width sizing, minimum width/height, maximum height, exact-height/automatic-
width sizing, and public Flutter-like constraint objects remain excluded. The
three admitted sizing modes are defined in `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Collect publication-column and embedding workflows using
automatic, exact-width/automatic-height, and exact-pair output. Demonstrate unmet
needs and compare interactions with aspect, wrapping, legends, facets, Canvas,
and strict fit. Let the user choose a small justified extension, rejection, or
re-deferral; do not assume a general bounds surface is the destination.

## DF7: Sparse Composition Compaction

Packing occupied cells or collapsing empty rows, columns, or trailing wrap slots
remains excluded; preserved topology is specified in `05_LAYOUT_SYSTEM.md`.
At T6 ENTRY the user approved re-deferral: natural content sizing is implemented
but does not transform topology, hand-authored Canvas can declare compact grids,
and Cartesian holes can encode absent scientific combinations.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Compare real public automatic/exact-width sparse
figures after T6. Assess removal of wholly empty tracks separately from row-major
occupied-cell repacking or an observed-combinations facet gallery. Specify
occupancy, source/destination coordinate mapping, metadata-only tracks, header
spans, row titles, range-group membership, shared label ownership and diagnostic
identities. A partial wrapped final row is not solved by deleting wholly empty
tracks. Show the semantic cost and actual benefit before obtaining user-approved
new-phase admission, reasoned rejection or explicit re-deferral.

## DF8: Caller-Selected Layout Objectives

Caller objectives for aspect, density, whitespace, legend flow, text measure, or
grid balance remain excluded. No scoring callback, strategy switch, string mode,
or public layout plugin is reserved.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Review unmet scientific workflows against the completed
private local policies, including exact-constraint failures and reproducibility.
Compare a small typed objective with existing explicit controls and its bounded
costs; obtain the user-approved disposition without creating global search by
implication.

## DF9: Global Layout Optimization

Exhaustive whole-tree search, sibling Cartesian products, exhaustive text break
partitions, and root-wide dense-rank objectives remain excluded. Approved bounded
containing-frame placement recovery is not this feature; see `05_LAYOUT_SYSTEM.md`.

**Review checkpoint**: T7 CLOSURE, before successor planning, alongside DF8.

**Evidence and decision**: Require demonstrated scientific workloads, measured
performance/resource comparisons with bounded local recovery, semantic
completeness, determinism, and a coherent replacement contract. Obtain the
user-approved disposition; historical search achievements do not authorize a
dormant solver or runtime strategy switch.

## DF10: Stop At First Fitting Line Proposal

Removing exact next-endpoint line extension remains excluded. PF7 accepts KEEP
of the current extension policy; T5a3p7 is complete as a disposition only, not a
wrapping implementation change. No strategy switch or alternate wrapping mode
is reserved.

**Review checkpoint**: T7 CLOSURE, before successor planning.

**Evidence and decision**: Use post-p2/p3/p4/p6 release measurements to establish
whether avoiding extension has meaningful benefit. Compare exact line endpoints,
caption heights, tight-height fit failures, bidi, soft hyphens, and retained glyph
geometry. Revisit only if benefit justifies the wrapping/fit tradeoff; otherwise
record rejection or explicit re-deferral. Any admission requires user acceptance
of changed wrapping and a new approved phase under the process above.

## DF11: Scientific Plotting For AI-Authored Workflows

Reconsider the library's role and capabilities for LLM-authored, deeply customized
scientific figures. The motivating user observation is that LLMs can write bespoke
drawing scripts, use low-level libraries such as PIL directly, edit generated
vector SVGs, and add custom shapes or annotations to both individual plots and
manually composed multi-panel figures. A library limited to a closed catalog of
internally predefined plot kinds may not provide enough value in those workflows.
The review concerns the library's positioning as well as concrete capabilities;
its evidence should establish how scientific plotting libraries, including
matplotlib, fit these workflows and where a different interface adds value.

The user explicitly added **fully caller-decided legends** to this review. This
includes legends independent of automatic data/scale contributions: caller-owned
titles, labels, entry selection and order, point/line/patch symbols, exact RGBA
and geometry, continuous or discrete explanations, and addition to or replacement
of automatic legends on individual plots and composed Canvas figures. Review
declarative symbol-and-label construction as well as the broader possibility of
caller-drawn/custom graphics; neither interface is preselected. Include custom
alpha functions whose intended explanation is supplied by the caller, rather than
inferred from the function or observed alpha values. The user chose to consider
this capability at the AI-workflow review, not in the admitted alpha/tick work.

**Review checkpoint**: The post-first-release checkpoint in
`91_IMPLEMENTATION_PLAN.md`, after **all** of the following are complete:

1. Refactoring the project into a Rust workspace.
2. Completing the polars data adapter.
3. Completing the ndarray data adapter.
4. Completing the Python calling interface.
5. Publishing the first version.

The review begins only after the last prerequisite is satisfied. It is not a
T6/T7 entry or closure gate. These prerequisites identify the future review timing;
this item does not itself authorize their implementation or change today's
standalone renderer boundary.

**Evidence and decision**: Collect representative AI-authored workflows for
single scientific plots and custom Canvas-like compositions, including direct
low-level drawing, SVG post-editing, caller-added graphical elements, and fully
caller-authored legends. For legends, compare automatic scale explanations with
independent caller declarations, append/replacement and Plot/Canvas ownership,
symbol/label versus custom-drawing scope, measurement/placement, fonts, complete
ink, source identity, and immutable SVG/PNG output. Identify
where the library contributes reliable scientific transforms/statistics, axes,
text, layout and reproducible output, and where its fixed plot catalog or public
controls force external reconstruction. Evaluate what changes would better fit
LLM-generated scripts and iterative figure editing, including discoverability,
composability, custom graphics, coordinate/annotation access and retained semantic
identity. Compare concrete alternatives and their ownership, determinism,
measurement/resource and SVG/PNG consequences before recommending any API or
architectural change. Present the evidence and tradeoffs for a user-approved new
phase, reasoned rejection, or explicit re-deferral. No plugin, drawing callback,
mutable scene, SVG-editing API or alternate rendering path is reserved here.
