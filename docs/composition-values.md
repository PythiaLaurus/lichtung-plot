# Complete private composition values

## Stored model and construction

`ResolvedTheme.composition` owns one immutable `CompositionTheme` from
`src/theme/composition.rs`, implementing §7 of
`specs/94_T6_FOUNDATION_CONSTRUCTION.md`. `CompositionAppearance`,
`RootTitleBlockAppearance`, and the appearance-source `ROOT_PADDING` constant
are removed. Repository consumers use the complete component and selected
typography roles. Its fields remain private; getters use field names and
consuming `with_field` edits return `Result`.

| Complete value | Retained fields |
| --- | --- |
| `RootTitleSpacing` | `title_subtitle_gap`, `title_block_to_content_clearance`, `caption_to_content_clearance` |
| `RootPadding` | `left`, `right`, `bottom`, `top` |
| `TextPanelSpacing` | `inset`, `gap` |
| `HeaderSpacing` | `column_header_padding`, `row_header_padding`, `row_title_horizontal_padding`, `row_title_vertical_padding` |
| `FacetStripSpacing` | `horizontal_inset`, `content_clearance` |
| `CompositionTheme` | Three root alignments, the five complete groups above, `panel_edge_clearance`, `facet_gap`, `canvas_grid_gap` |

All **18 relationships** are validated nonnegative `RelativeSpacing` values.
Zero is valid for every spacing field. Groups introduce no additional positivity
or sibling-ordering constraint; closed alignments have no invalid states.
Subtitle alignment alone is optional. Defaults are Center / None / Start for
title / subtitle / caption. Edits replace one field, preserve independent siblings,
and cannot mutate a cloned source. Constructors inspect neither fonts nor a data
domain. Production defaults construct ratios with `ratio_in` on the supplied
operation ledger before arbitrary-integer storage, without a new production
resource root.

## Exhaustive source and metric mapping

T/S/C/A are each preset's original title/subtitle/caption/axis-title CSS sizes:
Publication 20/15/15/17, Nature 19/14/14/16, Science 21/16/15/17, CHI 19/15/15/17.
E(role) is the positive integral font-size em of the complete selected role.
Each ratio below is constructed from mathematical integers, preserving its
original pixel source exactly at the original role metric. Resolution uses exact
products, never an `as_f64` round trip.

| Former source | Sole field | Construction basis | Consuming metric / cardinality |
| --- | --- | --- | --- |
| Root title/subtitle gap 0 | `root_title_block.title_subtitle_gap` | 0/1 | max(E(root.title), E(root.subtitle)); only adjacent present roles |
| Top block/content clearance 14 | `root_title_block.title_block_to_content_clearance` | 14/max(T,S) | Same complete-role maximum; top-block presence |
| Caption/content clearance 12 | `root_title_block.caption_to_content_clearance` | 12/C | E(root.caption); caption presence |
| Root and embedded prose left 16 | `root_padding.left` | 16/A | E(owner composition.body); once at left edge |
| Root and embedded prose right 16 | `root_padding.right` | 16/A | Same owner; once at right edge |
| Root and embedded prose bottom 8 | `root_padding.bottom` | 8/A | Same owner; once at bottom edge |
| Root and embedded prose top 12 | `root_padding.top` | 12/A | Same owner; once at top edge |
| Measured axis/empty-edge comfort 4 | `panel_edge_clearance` | 4/16 | E(owner axes.tick_label); once per measured edge |
| TextPanel inset 16 | `text_panel.inset` | 16/A | E(Canvas composition.body); independently at all four edges |
| TextPanel title/body gap 12 | `text_panel.gap` | 12/max(T,A) | max(E(text_panel_title), E(body)); one sequence gap |
| Facet topology gap 12 | `facet_gap` | 12/S | E(owning Plot facet_header) |
| Canvas/Matrix/Marginal grid gap 12 | `canvas_grid_gap` | 12/A | E(owning composition.body), independent of first child |
| Column header band addition 16 | `headers.column_header_padding` | 16/T | E(Canvas composition_header); total height addition once |
| Row header band addition 20 | `headers.row_header_padding` | 20/T | Same header role; total width addition once |
| Row title horizontal addition/subtraction 16 | `headers.row_title_horizontal_padding` | 16/T | Same header role; total span once, not per edge |
| Row title vertical reserve 8 | `headers.row_title_vertical_padding` | 8/T | Same header role; one title/body reserve |
| Facet strip width subtraction/addition 16 | `facet_strip.horizontal_inset` | 16/S | E(Plot facet_header); total span once |
| Strip/content clearance 8 | `facet_strip.content_clearance` | 8/S | Same facet role; add complete measured strip occupancy |

Default RGBA, opaque paints, font stacks, weights, scientific source preparation,
and mark/axis role choices are preserved. Matrix still consumes its independent
bottom/left cell frame; composition creates no plot-family theme or style.

## Complete spatial boundaries and ownership

Operation-local length groups retain exact operands until their spatial use:
nearest-even S for frozen positions/gaps, upward H for complete requirements.
The nominal root totals 32px horizontally and 20px vertically are derived from
the four fields, not independently stored constants. Natural sizing, constrained
root allocation, and hard intrinsic guards use the same owner relationships.
Root and embedded prose wrapping subtract exact horizontal reserves before the
final floored text capacity. Negative capacity remains a typed fit miss.

Embedded Plot prose uses its Plot theme's four padding fields only when any
nonempty title/subtitle/caption is present. Its prose span is inset horizontally;
the scientific body retains the full cell width. Vertical padding and measured
prose reserves constrain its body height. With absent prose, its span/body remains
the whole cell. Canvas padding never replaces an explicit child's Plot padding.

Root text combines present-role measured heights with exact adjacent gaps and
clearances before hard rounding. The top-block metric always uses both complete
title/subtitle roles, even when one content is absent. Baselines and centered
block positions retain the parent parity through the final nearest-even boundary;
they do not round a gap separately and subtract it later. Publication title's
retained binary64 1.3 still produces **26625 quanta**, not a substituted 26px pitch.

`RootTextReserves` retains exact top, bottom, and combined reserves. Natural
content sizing returns this value, not two independently ceiled pixel estimates.
Root, embedded, Canvas, facet, and standalone natural-height owners add body and
outer padding before the one complete H boundary. Frozen body edges and prose
baselines combine padding and their exact offsets against the original parent
coordinate, then select nearest-even once. The immutable exact reserve payload
is retained through an admitted Arc; its `ExactLength` operands remain non-clonable.
Selected standalone/facet body-height recomposition also returns to the retained
exact prose/padding aggregate before the H boundary. Added shared-axis bands remain
their actual canonical measured consequences; rendered inset coordinates are not
substituted for the exact appearance reserves.

TextPanel retains its selected 1.2 title/body roles and existing 66ch preference.
Its width/height capacities subtract complete exact per-edge insets and sequence
gap before integer Grid assignment. Each text is measured once at its assigned
capacity. Frozen positions use the exact inset/preceding-title/gap aggregate.
Subquantum capacity cannot acquire a one-pixel fallback; impossible unbroken text
keeps its TextPanel title/body address instead of becoming a generic Grid miss.
The sequence validates its actual content slack against its allocated tracks.

Column/row header and row-title additions retain their total-band cardinality.
All three header producers freeze positions in nearest-even layout quanta,
replacing whole-pixel `.round()` placement. Horizontal centering freezes the
whole block origin with the parent's parity before using the renderer's canonical
text half once; independently rounding a midpoint and an odd-width text half
would move an unpadded header's ink one quantum outside its band.
The obsolete separate supplied-heading/count branch and `PANEL_HEADER_GAP` are
removed; embedded prose already has the common root title producer. Measured
axis-edge comfort and natural-axis proposals consume `panel_edge_clearance`.
The standalone renderer retains its existing whole-pixel outward axis envelope
after the complete exact comfort plus canonical axis requirement.

### Declared tracks

Canvas declared-track preference is exactly its owning
`composition_header.font_size() * 1024`; facets use their owning
`facet_header.font_size() * 1024`. Both axes use that selected header em even with
no visible header/strip. Each otherwise empty declared track retains hard minimum
one quantum and preferred minimum one em. Body and TextPanel-title edits cannot
substitute their font metric; a header-font edit changes the actual preference.
The separately owned gaps can change with body/facet font edits. Sparse/trailing
tracks retain their addresses, shape, and gaps; em remains a preference that can
shrink to the declared one-quantum hard minimum.

### Facet strip measure/paint consistency

Strip width uses the owned total horizontal inset, with no `.max(1px)` escape.
Reserve and placement now use the same complete selected 1.2 text plan plus its
owned clearance. The former `8 + line_count * (facet_size + 6)` estimate and
independently reconstructed 14px first-line offset are removed. Placement freezes
the actual retained first baseline and full block extent. The obsolete
`PlacedText::from_deferred_first_line` whole-pixel adapter is removed; its rotated
PNG caller uses explicit middle placement.

This consumes the strip consistency correction described with B2 in spec94 §4
while making the composition field fully active. It intentionally changes strip
reserve/baseline geometry, rather than storing a hidden historical leading field.
Prepared raw/display labels, all lines/glyphs, strip cardinality, scientific marks,
and declared topology remain intact; focused rendering and sparse-facet callers
certify those inputs.

## Admission, identity, and bounded root-frame reuse

Composition stores 18 immutable ratio handles. Complete theme copies now cost
**60 shared appearance handles**, up from 42. Full shared theme matching costs
**63 visits**, including the three axis/guide/composition descriptor witnesses.
Composition matching alone costs **19 visits** for shared equality: one alignment
descriptor plus each actual normalized relationship. Independent payloads use
admitted limb witnesses; scalar/relationship mismatches stop before later work.
Operational identity does not use unpriced aggregate component equality.
The terminating child tick-stack fixture now costs **61 visits** and retains
unexamined later font names/feedback tails. Sharing copies no ratio buffers.

The independent construction oracle uses the 18 explicit numerator/denominator
pairs, bounded native Euclidean steps, and actual native header/operand sizes.
Its work is `54 + gcd_steps`; bytes are
`18 * (ratio_header + 2 * admitted_operand)`. Preset and Canvas root/explicit-child
fixtures add this certificate to existing axis/guide/paint construction charges
and exercise zero, minus-one, and equal work/byte limits before font discovery
or backend begin. Inherited children share the root; explicit children construct
their own component.

The composition certificates are Publication **109**, Nature **106**, Science
**112**, CHI **117** visits (55/52/58/63 Euclidean steps); each costs **2880 bytes**
on this target. Complete preset construction becomes **549 / 533 / 546 / 551**
visits respectively. These figures are pinned independently in the constructor
fixture, not obtained by measuring a successful constructor and repinning its use.

Measured root composition uses canonical text-plan height/baseline quanta
directly through `ExactLength::quanta` / `add_quanta`. Those operations admit
integer operands before storage/work; they retain the same exact rational length
representation instead of decoding a pixel projection again.

The prepared standalone owner retains one successful root frame for its immutable
source, theme, font generation, and alignment instructions. Its key contains
root dimensions and traced body minima; changed constraints remeasure. Hits
charge a measurement call, input witness, and one actual immutable frame-Arc share.
New/replacement entries charge cache-entry, record, exact-reserve/frame Arc storage,
and handle retention before mutation. The cached frame shares its exact operands;
it neither clones those integers nor reconstructs rounded reserve copies. Canvas
candidate copies separately admit the reserve Arc and actual text-plan handles.
This cache cannot cross operations or reuse another Plot's template.
It adds no root-width restart, guide trial, output policy, or metric fallback.

The unchanged 2048-word caption fixture exposed a work-budget failure; it also
failed on the isolated `41dc420` baseline at cumulative claim 1000001. Direct
canonical-quanta sums and operation-owned root-frame reuse complete it under the
existing limit, without shortening the input or increasing any cap:

| Fixture | Source bytes | Visits | Bytes | Records | Calls | Cache entries | Proposal visits | Exact trials |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Three paragraphs | 1802 | 206420 | 29970141 | 24355 | 182 | 67 | 217 | 253 |
| 2048 `Measured` words | 18432 | 999771 | 56366327 | 234560 | 1152 | 477 | 2175 | 2403 |

The fixture verifies all 2048 words in actual preserved-text SVG. Root-frame reuse
has a separate actual-owner test for identical constraints, changed width,
payload sharing, no duplicate shaping, and work refusal before a cached copy.

## Default output evidence and focused verification

An isolated `41dc420` worktree and this leaf rendered the same default two-variable,
two-group/hue Matrix at 800×600 under every preset. Full preserved-text SVG bytes
were compared directly with `cmp`, not normalized hashes or trimmed structure:

| Preset | Baseline/current bytes | Comparison |
| --- | ---: | --- |
| Publication | 22590 / 22590 | Byte-identical |
| Nature | 22366 / 22366 | Byte-identical |
| Science | 22183 / 22183 | Byte-identical |
| CHI | 22590 / 22590 | Byte-identical |

The temporary Rust probe was removed. Artifacts remain outside the repository at
`/tmp/opencode/composition-parity-base-41dc420` and
`/tmp/opencode/composition-parity-current` for inspection. Matrix default paint,
source cardinality, density/point topology, guides, clips, and two-sided cell
frames therefore retain full output parity.

Meaningful private fixtures cover every constructor/getter/edit, exact preset
basis, sibling independence, admission, and active typography metric selection.
Real boundary fixtures cover fractional root padding, odd-parent title parity,
one-pixel-role TextPanel shortest-digit capacity with exact one-third-quantum
reserves, complete ink containment, embedded prose presence/ownership, and Canvas
and facet declared em independence. Existing header/span, sparse topology,
root/facet/Canvas, range-group, replay, and output fixtures cover their callers.
Verification uses bounded locked/offline filters, strict `just lint`, formatting,
and whitespace checks; it does not use a full suite for regression discovery.

### Review boundary reproductions

All four real-owner regressions were observed failing before the review fixes:

| Regression | Observed pre-fix failure | Corrected contract |
| --- | --- | --- |
| Root frame with measured 1px-role title/caption, zero root padding, and two one-third-quantum clearances | Typed `root padding and complete content exceed the assigned root` miss | With canonical text heights T/C and body N−T−C−1, the exact requirement is N−1/3: H=N fits; one more body quantum misses |
| Actual embedded Plot title/caption at its scientific body minimum | `exact embedded clearances must leave the complete scientific body minimum` assertion | Same complete reserve sum fits and preserves the full body; one quantum less assigned height misses |
| Actual natural Canvas content-frame height | Chose 11px instead of 10px | Integral root selection occurs after the complete exact prose/body sum, selecting N=10px |
| Actual Canvas column/row/row-title paint at a legal 1px header role with zero band padding | Column ink reached y=4.482421875 outside the [3,4] band | All three complete selected text plans paint within their actual bands without clipping or a padding floor |

The header fixture exercises both even/odd quantum parent origins. A direct
midpoint-quanta fix exposed a remaining odd-width text-half error: row ink reached
x=0.5068359375 outside the [0.0009765625,0.505859375] band. Freezing the whole block
origin before the renderer's one canonical half fixes that leak. A separate real
spanning-header fixture retains complete Arabic/Hebrew multiline sources and
captures their unclipped glyph ink. Root/embedded regressions likewise capture
the complete title/caption primitives, not merely a direct arithmetic helper.

After these fixes the full four-preset 800×600 Matrix byte comparisons were
repeated against the same `41dc420` artifacts and remained identical. Root/Canvas
odd-size and Matrix/Marginal goldens pass existing expectations; none was repinned.

Representative reproducible checks:

```text
cargo test --locked --offline --lib theme::composition::tests
cargo test --locked --offline --lib preset_construction_
cargo test --locked --offline --lib decorations::title_block::plan::tests
cargo test --locked --offline --lib fractional_insets_and_gap_fit_once
cargo test --locked --offline --lib embedded_prose_uses_plot_owned_padding
cargo test --locked --offline --lib root_frame_combines_two_third_quantum
cargo test --locked --offline --lib embedded_frame_combines_two_third_quantum
cargo test --locked --offline --lib natural_root_height_combines_exact_prose
cargo test --locked --offline --lib one_pixel_headers_freeze_subpixel_centers
cargo test --locked --offline --lib unicode_spanning_headers_retain_complete_rtl
cargo test --locked --offline --lib canvas::tracks::tests
cargo test --locked --offline --lib orchestrator::facets::tracks::tests
cargo test --locked --offline --lib shared_multiline_titles_use_measured_clearance
cargo test --locked --offline --lib prepared_root_frame_reuses_complete_prose
cargo test --locked --offline --lib caption_publication_work_evidence_keeps_complete_required_sources -- --nocapture
cargo test --locked --offline --lib native_real_compositions_remeasure_gutters_and_select_guides_identically_without_cache
cargo test --locked --offline --test composition themes::
cargo test --locked --offline --test composition composed_horizontal_bars_support_shared_headers_labels_and_range_groups
cargo test --locked --offline --test visual-regression canvas_root_title_block_odd_size_snapshot
cargo test --locked --offline --test visual-regression matrix_clip_structure_snapshot
just lint
just fmt-check
git diff --check
```

Root/Canvas odd-size, Matrix/Marginal structure, native Bar-family and Beeswarm
snapshot filters pass their existing expectations. No snapshot was repinned for
this composition leaf. The strip correction is instead checked through complete
selected strip ink/source/line evidence and affected native/sparse facet callers.

## Remaining metric cutover boundary

The four legacy scalar typography carriers remain for their other B2 consumers;
composition spacing, declared tracks, and strips no longer read them. Remaining
generic legacy text adapters and non-composition line-pitch/producer metric
operations retain their current implementation until that owning cutover.
Root/embedded title pitch, composition bands/insets, TextPanel, and facet strip
measurement are already on the complete selected roles here. No hidden historical
strip field, family-specific component, public instruction setter, or new policy
was added. Public Shared-gap override behavior remains its existing B4 boundary.
