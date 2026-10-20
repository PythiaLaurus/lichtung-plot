# Consumed parent guide routing

Actual Canvas and facet planning calls now enumerate the parent operands used by
their child and content-height keys. This completes the integration of the typed
consumption key described in `scoped-guide-dependencies.md`; its temporary
non-test dead-code expectation is removed.

## Canvas calls

`canvas/dependencies.rs` constructs canonical ordered dependencies for one cell.
Only configured x-range groups containing that cell are included, followed by the
original `CanvasSharedLegend[0]` slot when a Shared guide actually reserves space.
Other groups and unrelated siblings do not enter the snapshots or stability
checks. Range alignment records the exact consumed cell width. Shared-suppressed
children record the residual containing box and relevant Shared-owner feedback,
not the parent's complete descriptor, neutral paints or flow variant.

Final child planning, assigned gutter probes and category probes all receive that
explicit consumption view. The initial full-body Shared probes have no selected
guide allocation and retain an empty ancestor list. `canvas_probe` now takes the
actual context rather than reconstructing it from a panel and Theme, preserving
the supplied dependencies and fit scope.

The content-height track pass receives the same actual range-group operands and,
when the guide has been assigned, its residual width-only allocation. No fabricated
finite height is introduced into a width-dependent measurement. Initial natural
selection has no assigned Shared allocation; candidate and final passes carry
their actual residual width. Recorded PerPlot consequences use the matching
range-group dependencies from the selected child call. Height remains a
replacement consequence, including contraction; root width is still selected
once.

## Facet calls

An outside facet guide reserves one containing Grid allocation. Its suppressed
children receive allocation-only dependencies on the original root guide slot.
Routing an inside guide into an occupied cell retains its complete prepared
parent descriptor, exact assigned cell box, vertical normal variant and original
root `InsideGuide[0]` owner. The other inside-guide cells do not consume that
descriptor or a guide-reduced allocation, so their ancestor lists remain empty.
The locally drawn slot and scientific source contribution remain distinct.

The parent guide's existing measurement cache still keys full descriptor,
orientation/variant, canonical assignment and consumed appearance. This routing
does not introduce a parent-result cache, a sibling search or another measurement
path. Existing feedback replay and placement-attempt accounting are unchanged;
sections and rejected ancestor trials gain no additional attempts.

## Actual parent regression

`routed_facet_guides_invalidate_only_the_cells_consuming_their_descriptor` replaces
the duplicate isolated parent-track retry probe. It prepares real faceted Scatter
sources and final font requirements, then runs production `FacetPlan::new` through
stable child-cache hits. An alternate prepared parent guide has the same alias,
labels, keys and extents but different raw/source identity. At equal child bounds:

* The routed guide-owning cell misses; the independent cell still hits.
* Unchanged complete parent operands hit both cells.
* A rejected trial making the original root guide owner unstable invalidates only
  the routed consumer, even though that owner is outside the cell subtree.
* Cached and uncached complete roots produce byte-identical frozen SVG after final
  Null validation and retained replay.

The existing dirty-facet-neighbor test still proves selective free/fixed-scale
reuse. Complete child-key, height-key, coordinate, placement and frozen-capture
contracts remain. The library discovery count is **671, including 17 ignored**;
one test replaces one existing test and no integration tests are added.

## Verification

**38 distinct focused library tests** pass: five facet-parent tests, six child-cache
tests, six composed interval/provenance tests, sixteen native coordinate tests,
the height-key test, Shared effective-policy authority, frozen legend capture,
dirty-facet-neighbor reuse and rejected-ancestor attempt accounting.

**29 existing external tests** pass: ten root-sizing contracts (including actual
all-family content roots, mixed Canvas and range alignment), sixteen Shared
composition contracts, x-range refusal validation, explicit inside/outside guide
placement and inside-legend/significance collision. Strict all-target Clippy,
formatting and whitespace checks pass. Logs are
`/tmp/opencode/routed-guides-*.log`; the corrected placement filter executes one
test. No integrated broad-suite or release closure is claimed.

Public composition controls/examples and the full public Separate acceptance
matrix are the following work. No scientific source membership, color domains,
parent capability, Theme scope, placement budget or publication lifecycle is
changed by dependency routing.
