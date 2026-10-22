# Layout System

## Protocol And Guarantees

Parents allocate space and positions. A child receives constraints and returns
**one immutable measured plan or a typed fit miss**. Intrinsic summaries inform
allocation separately from constrained measurement; they are not collections of
plans at all widths. Named bounded local choices and root-coordinated feedback
do not authorize sibling Cartesian products, exhaustive wrap partitions, global
quality ranking or root-width search.

Execution is deterministic, not a guarantee to discover every feasible layout.
Exact roots preserve complete requested content and typography or fail. Automatic
roots choose a natural preferred width once, then measure content height; they do
not grow a hidden fixed canvas until something fits. Public inputs/lifecycle are
defined by [rendering](02_RENDERING_CONTRACT.md), local appearance metrics by
[Theme](03_THEME_SYSTEM.md), numerical sources by
[numbers](06_NUMERIC_PRESENTATION.md) and explanatory flow by
[guides](08_GUIDE_SYSTEM.md).

Private owners are `src/layout/kernel.rs`, `src/layout/kernel/tracks.rs`,
`src/layout/kernel/feedback.rs`, `src/layout/interior.rs`,
`src/decorations/plan.rs`, `src/canvas.rs` and `src/orchestrator/facets.rs`.

### Constraints And Results

A length constraint has nonnegative minimum and optional maximum>=minimum. None
means no maximum, not infinity. Width/height constraints are independent.
Invalid intervals reject before measurement; returned sizes satisfy them without
clamping. Fill receives an exact assigned dimension; nonfill receives zero through
the assigned extent and is aligned by the parent. An arranged PanelBox is not an
incoming constraint or intrinsic summary.

Summaries retain traced hard requirements, unwrapped natural text width, preferred
lengths and width-dependence. A minimum pair is not a promise of joint sufficiency
for every wrapping/packing outcome. In canonical address order, parents obtain
summaries, allocate columns/widths, measure width-dependent content, allocate rows/
heights and arrange. Assigned-geometry calls include exact containing box,
projection identity and attachment targets. Immutable plans retain selected boxes,
baselines, ink, text/glyphs, ticks, guide flow, projections and geometry. Translation
cannot change decisions; changed inputs require another keyed measurement.

FitReason is `(kind,component_address,local_attempt_ordinal)`. Kind order is
constraint, text, tick, legend_or_guide, track_or_residual, overlay_collision,
specialized_geometry, assigned_child. Ordinals are zero-based local attempt order.
Propagate original child kind/address; assigned_child is only a last-resort reason.
Exhausted reasons sort/deduplicate by that tuple and return the least.

A miss is terminal or provisional with concrete named feedback, a resolvable
predicate and enough geometry to derive the consequence. Only axis gutter,
shared alignment, owned ink, automatic domain and content height can make a miss
provisional. Unbreakable oversized text, insufficient exact hard total or collision
without a named remedy is terminal. Arithmetic, allocation, I/O, font and invariant
failures are fatal, never fit alternatives.

Where boxes can be constructed, siblings run canonically even after a miss: later
fatal error wins, otherwise least terminal reason, otherwise aggregate provisional
requests. Local alternatives continue only when the next declared choice can
address a terminal miss. The first provisional attempt stops local alternatives
and returns to the root. Stale gutters/domains must not select a less-preferred
tick source or guide edge.

## Canonical Geometry

LayoutScalar is signed i64 in exactly **1/1024 CSS pixel** quanta. Lengths are
0..=i64::MAX; positions/edges use the signed range. Ordered box differences must
be nonnegative representable lengths. Decode floating inputs as exact represented
binary rationals before multiplying by1024. Normalize negative zero.

| Operand | Rounding |
| --- | --- |
| Lower length constraint | Toward positive infinity |
| Upper length constraint | Toward negative infinity |
| Complete hard requirement, reserve, line occupancy | Upward once after exact aggregation |
| Lower/upper ink or collision edge | Outward, down/up respectively |
| Font size, advance, offset, baseline, preference, alignment, spacing, stroke center/width, position | Nearest-even |
| Integral public CSS dimensions | Checked multiplication by1024 |

Stroke edges remain outward even when width/center operands use nearest-even.
Spatial sums/differences use checked i128; counts/products/areas/squared distances/
ratio products use checked u128 and checked conversion. Signed magnitude is taken
after wide subtraction. Failure aborts, not saturates, wraps, skips a candidate or
retries a smaller value. There is no epsilon equality or float geometry fallback.

One em is the resolved role font size. One ch is its shaped U+0030 advance with
actual stack/face/weight/size/profile; missing required0 is a font error. Design
metrics use exact `design_value*font_size_quanta/units_per_em`, rounded by spatial
role. Round complete hard run advance upward once, not per glyph.

Nonspatial binary64 identity inputs are finite and normalize zero. For bits b,
total-order key is `!b` for negative, otherwise `b^0x8000000000000000`; equality
is normalized bits. Computation uses nearest-even without FMA substitution.
Strings compare UTF-8 bytes, integers numerically, None before Some, sequences
elementwise then length. Pointers/hash iteration never select layouts.

Renderer log10 in `src/numeric.rs` is correctly rounded binary64 nearest-even:
exact-integer outward atanh intervals start at 128 fractional bits and double until
both endpoints round identically. Exact powers10^0..10^22/exceptional inputs are
handled explicitly; rationality proves remaining termination. decimal_power uses
exact integer/rational conversion for exponents-323..308, not platform powf.
Checked precision/storage/refusal remains fatal. Mathematical Forest display
powers have the distinct [certified decimal contract](07_LOG10_DISPLAY.md).

Integer ceil division is `n/d + (n%d != 0)` for positive d; nearest-even compares
twice remainder with denominator and ties to even. Integer floor/ceil square roots
are exact. Numeric projection preserves endpoints0/1 and uses `(v-lo)/(hi-lo)`
when direct subtractions are finite; otherwise scaled opposite-extreme arithmetic
with `scale=max(abs(v),abs(lo),abs(hi))`. In-domain fractions are finite[0,1].
For D=2^64-1, key is exact nearest-even p*D; center is exact nearest-even
`origin+span*key/D`, including origin parity. Marks/ticks/grid share it.
Categorical edges/centers use exact integer ratios i/n and(2i+1)/(2n), including
origin; off-center/jittered observations retain continuous positions.

## Root Sizing And Preferences

| Request | Policy |
| --- | --- |
| Omitted | Choose natural preferred outer width once, measure at it, derive height |
| width(w) | Exact positive integral CSS width, measured automatic height |
| size(w,h) | Exact positive pair, fit or fail without changing either |

Latest sizing call replaces the entire request; width clears earlier exact height.
Zero rejects at preflight. Canvas ignores embedded standalone requests, including
zero; they are not cell preferences. No public generic bounds, exact-height/auto-
width or maximum-width mode exists.

Natural preference composes traced summaries, insets, gaps, guide and scientific
requirements bottom-up. Prose prefers
`max(hard_unbroken_width,min(natural_unwrapped_width,66ch))`; 72ch is a readability
diagnostic, not a cap. Automatic outside guides initially contribute their Right
one-column width+gap and cross-axis height. Explicit horizontal guides contribute
a width maximum, vertical guides add width. Multiple guides compose canonically.
This initial preference is not a placement commitment or permission to rechoose
root width during feedback.

For a standard panel with m prepared primary marks, including transparent and
coincident marks, use tick-label typography:

```text
mark_pitch = max(1em, maximum outward primary-mark width or height)
density_side = ceil_sqrt(max(1,m))*mark_pitch
preferred_width = max(hard_width,66ch,density_side)
preferred_height = max(hard_height,ceil_div(3*preferred_width,4),density_side)
```

Empty mark maximum is0. Assigned-width automatic height instead uses that assigned
width in the same4:3 preference. Hard body dimensions are maxima of one quantum
and actual representability/mark/interval/cell/annotation/text requirements, not
a universal panel floor. Categorical hard pitch is max(one quantum,occupied slot
span), preferred pitch max(hard,2em); multiply by semantic slots without compaction.
Occupancy excludes outside axes/guides. Heatmap/Matrix prefer square cells;
standard/categorical/Beeswarm/Waterfall/Marginal central prefer4:3. These are soft,
not exact-aspect requirements. No hidden640x480/960x640 default exists.

## Primitive Allocation And Arrangement

Closed primitives are Leaf, VerticalSequence, HorizontalSequence, Grid and Overlay.
Coordinates start upper-left, x right/y down. Gaps/insets resolve from owned local
roles, not root scale. Columns precede width-dependent text and rows.

A track is flexible when an occupying child fills its dimension, except declared
Canvas/facet tracks are always noncollapsing flexible. Start hard minima from
topology, then single-track requirements in canonical child order. Process spans
by increasing span length then address, raising covered flexible tracks by equal
capped surplus until their requirement including internal gaps is met. No headroom
misses. Inflexible tracks keep required size. Build preferences using the same
procedure from the hard vector; explicit maxima cap only soft preferences.

Subtract insets/gaps/inflexible extents from available total with checked arithmetic.
If remaining A is below flexible hard sum, miss. For minima m and preferences p,
if A<sum(p), apply one proportional shrink:

```text
C = sum(p[i]-m[i]); deficit = sum(p)-A
shrink[i] = floor(deficit*(p[i]-m[i])/C)
```

Remaining shrink quanta go by descending fractional remainder, ties ascending
track index, skipping exhausted capacity. This exactly uses A without crossing
minima. If A>=sum(p), distribute surplus equally among flexible tracks with
headroom, quotient then index-ordered remainder; cap/remove exhausted tracks and
redistribute only remaining surplus. Each step finishes or removes a capped track.
All-max remainder is trailing parent slack unless full coverage is required, which
misses. Recheck spans after shrink; failure does not invoke another allocator.
Natural dimension uses preferred/content vector, not surplus up to a bound.

Grid edges cumulatively add insets/tracks/gaps. Spans include internal gaps.
Declared Canvas/facet empty/trailing tracks have hard1 quantum and preferred one
owner-local font-size em: Canvas composition_header, Plot facet_header. Occupied/
spanning requirements can raise them. Other unoccupied tracks have no implicit
floor; empty-slot children stay zero-size. No repacking. Natural n-track extent
is `sum(final_preferences)+(n-1)*gap`, with actual bands/gutters added once.

Vertical sequence natural size is `(l+max(w)+r,t+sum(h)+sum(gaps)+b)`.
Horizontal size is `(l+sum(w)+sum(gaps)+r,t+max(N,BA+BD)+b)`, where BA/BD are
maximum exposed baseline/height below, N maximum nonbaseline height. Empty maxima
are0. Absent optional children keep addresses without sequence-gap adjacency;
present empty text stays present. Grid gaps belong to topology even beside holes.

Cross-axis Start/End uses corresponding edge, Center nearest-even half residual,
baseline `top+BA-child_baseline`. Key center aligns to label first baseline.
Root title/subtitle/caption use resolved physical alignment across post-inset
content. Headers/row titles/strips center in their spanned cells. Other nonfill
children start unless a named role specifies otherwise.

Frames/bodies/panels/interiors/marks/Canvas cells/surfaces fill both axes. Horizontal
axes and top/bottom guide tracks fill width; vertical axes and left/right tracks
fill height. Text/glyphs/entries/individual guides/overlays/empty slots are nonfill.
The root body is the sole flexible main-axis child; root prose stays inflexible.

## Text And Unicode

Text retains original bytes without normalization. Use coherent Unicode 16.0.0:
UAX#29 extended graphemes, default UAX#14 legal breaks without locale tailoring,
UAX#9 paragraph bidi, script/Script_Extensions, mark General_Category and font-name
NFKC-casefold. All-neutral paragraphs default LTR. Mandatory BK/CR/LF/NL breaks
produce b+1 paragraphs including leading/interior/trailing empties, each with at
least one line; CRLF is one break. GL/WJ behavior remains intact.

Unicode tables/algorithms must have reproducible version/hash/provenance and
compatible shaping classification; a dependency version alone is insufficient.
Focused independent defaults/runtime tables remain local. Full conformance is
opt-in against explicitly supplied hash-verified official16 data; missing/mismatch
fails, with no downloads or hidden cache lookup. Development policy defines
regeneration and corpus-history publication boundaries.

Use locked rustybuzz0.20.1, language und, default variations, no caller features,
required/default OpenType features and compatible Unicode classification.
Resolve Common/Inherited scripts with Script_Extensions, surrounding strong
script, paired punctuation and inherited base context; prefer compatible preceding
strong script, otherwise following, leaving unresolved neutrals Common. Do not
artificially split punctuation/marks or change font selection for script/fit.
Runs follow resolved script, bidi direction and selected face; shape complete lines
with paragraph context. Retain face/index/glyph IDs/source clusters/offsets/advances/
directions/scripts. Selected-face/shaper/metric failure is fatal.

### Controls And Legal Endpoints

Mandatory separators/bidi controls affect algorithms without standalone painted
glyphs. ZWJ/ZWNJ, variation selectors/default-ignorables remain in shaping/source
mapping; absent standalone cmap is not missing visible text. Associate controls
with relevant surrounding run or primary face in an otherwise empty run. The
shaper suppresses nonpainting control glyphs. Selected U+00AD break paints declared
U+002D with original source mapping; otherwise it paints nothing. Tabs and
unsupported C0/C1 other than named mandatory separators reject; no implicit tab
policy. Ordinary visible scalars/marks require coverage and valid shaped glyphs.

Legal endpoints are UAX#14 opportunities between UAX#29 graphemes plus paragraph
end. No proposal crosses mandatory breaks. No emergency identifier/grapheme split,
fit-driven rotation, font shrink, omission or general ellipsis is allowed.

Unbroken words/identifiers remain intact regardless of assigned width; even a
short technical name such as `LightGBM` is not internally split to fit a compact
cell. Callers choose explicit mandatory breaks or prepare display labels before
layout. Label humanization is a content-preparation policy independent of width,
not a response to fit failure. No additional internal-break rule or automatic
rotation-versus-word-splitting policy is provided.

### Intrinsics And Constrained Wrapping

Shape each full paragraph once plus maximal sequences with no legal internal
break for intrinsic analysis. Natural width is maximum full-paragraph outward
ink span; hard unbroken requirement includes ink and advance. Retain paragraph
shaping and one source-ordered cluster advance map, not every suffix/boundary
pair. Reuse requires identical text, font/profile and paragraph context.

For proposals only, accumulate nonnegative magnitudes of exact shaping-cluster
inline advances in logical source order, assigning a cluster at the first legal
endpoint at/after its source end. Equal endpoints use run then glyph order.
An internal legal grapheme endpoint receives no partial cluster estimate and
remains legal for exact reshaping. Prefix monotonicity is bookkeeping, not fit proof.

At finite width, if the already shaped full paragraph's complete upward advance
and outward ink fit, retain it directly with line-specific bidi, no new endpoint
search. Empty paragraphs retain one line with primary metrics. Otherwise, from
each current line start:

1. Scan legal endpoints forward using paragraph prefix differences until first
   proposal exceeding width. Propose previous endpoint, or first if none precedes;
   if none exceeds, propose paragraph end. No scan backward from paragraph end.
2. Shape/check that complete line with paragraph context and selected controls.
   Both upward complete advance and outward ink must fit. On failure retreat one
   legal endpoint at a time, checking each; failure at first returns text miss.
3. From a fit, test only its immediately next legal endpoint and extend while it
   fits. Stop at first exact nonfit or paragraph end, retain last fitting line and
   resume from its source boundary. Never skip a nonfit to seek a distant fit.

Reuse endpoint results only for the same line-start trial. A retreat failure
also stops later extension without reshaping. Release failed glyph payloads and
obsolete start-trial entries as scanning advances; retain selected lines and one
paragraph map. Optional trial storage grows fallibly. Contextual nonadditivity
means this is not a globally longest-line guarantee.

Compute paragraph bidi once, retain across soft wraps, then apply UAX#9
line-specific reset/reorder per selected line. Do not re-resolve each as a new
paragraph. Retain final contextual shaping for publication.

### Glyph Geometry And Text Boxes

Accumulate design advances in exact rational layout units in visual run order.
Each absolute glyph origin rounds the exact accumulated pen+offset once nearest-
even, not individually rounded advance sums. Outward painted bounds use those
same origins/scaled outlines as collisions and outlined/PNG output. Complete-line
hard advance is separate from glyph placement.

Use OS/2 sTypoAscender/sTypoDescender when present, otherwise hhea; invalid metrics,
including nonnegative descender, reject. A/D contain maximum positive ascent/
absolute descent and selected ink above/below baseline across fallback faces;
without glyphs use primary metrics. With upward role leading,
`pitch=max(A+D,line_height)`, `height=A+D+(line_count-1)*pitch`, baseline i=A+i*pitch.
First baseline is sequence baseline; retain last for ink checks.

Nonfill width is maximum selected line ink span. Given line edges left/right,
span=right-left and block W, line origin is -left for Start,
`round_even((W-span)/2)-left` for Center, W-span-left for End. TextPanel and guide
titles/labels start; headers/axes/ticks/guide numbers center; annotations follow
their typed edge anchor. Root prose has its resolved independent alignment.

Preserve-text SVG retains original content, chosen line boundaries/anchors,
validated stack/weight and paragraph bidi metadata. Viewer shaping/fonts may
differ. Outlined SVG/PNG use frozen generation glyphs/origins without second
wrapping/fallback/shaping. Source accessibility metadata must not paint text twice.

## Installed Fonts And Pre-Font Census

Resolve scientific domains, exact limits, final color maps/contrast roles and
Shared guide compatibility before census. Visit actual source strings in canonical
component preorder, role order and local record order. Empty exact labels retain
identity with zero scalar needs. CSS family names/SVG syntax/suppressed content
are not painted text. Include actual typography probes and discretionary hyphen.
Numeric late-generated sources contribute finite grammar/affix boundary units as
specified in numeric presentation, not guessed labels or isolated scalar unions.

Assign first-use ordinal to each structurally equal `(FontStack,weight)` pair,
preserving stack entries. Census pair ordinal, source record/scalars/unit spans,
then generated alphabet in scalar order. Retain complete base-plus-mark spans
and face selections; shared character sets do not prove equal unit requirements.

### Canonical Candidate Selection

Explicitly load discovered files into locked fontdb0.23.0; do not delegate an
unbounded load_system_fonts. Family occurrences are every ID16 typographic and
ID1 legacy record decoded by ttf-parser 0.25.1. Match named families with Unicode 16
NFKC-casefold. Generics use the resulting database's host family_name without
project global remap; unmatched generic contributes none.

Identity is `(SHA256(complete_file_bytes),face_index_u32)`, excluding path, mtime,
database ID and discovery order. Generate occurrence tuples and sort by
`(stack_entry_index,family_match_rank,style_distance,stretch_distance,
weight_distance,sha256_bytes,face_index)`. Ranks are named ID16=0, ID1=1,
generic=2. Style Normal/Oblique/Italic distances0/1/2; stretch distance abs(width
class-5) for classes1..9; weight distance abs(face weight-request400/600/700).
For duplicate identities retain least complete tuple then sort. First is primary;
fallback scans the same canonical list.

Within one EGC, a narrow selection unit is one visible non-Mark base plus its
General_Category Mn/Mc/Me marks. Prefer first candidate covering the complete unit;
otherwise select first coverage per visible scalar. Leading marks/outside scalars
use scalar fallback. Nonpainting controls are not coverage requirements. No emoji/
ZWJ/whole-word/general whole-grapheme fallback or multiple-base unit is implied.
Empty candidate list/uncovered visible scalar fails. Selected-face metrics/shaping
failure is not permission to try another candidate. Layout cannot change fonts
because another face fits better.

Retain canonical ordered lists, pair ordinals, unit spans, selected identities and
same immutable font bytes through geometry/path conversion. Discovery/hash/metadata/
coverage/shaping/outlining consume those bytes. No reread selected path, next-path
recovery, bundled font, network or project cache is allowed. Fresh operations
discover fresh installed state; Output reuse retains its original environment.

### Discovery And Fontconfig

Project-owned source walking retains platform roots, extension/regular-file rules
and canonical deduplication with lossless native path order, source-location first.
Directory symlinks are not followed. Grow traversal storage fallibly from actual
prefixes. Open one admitted regular file at a time, verify actual handle length,
reserve/read/hash/decode that content and close before next file. Short/changed
read, discovered-file I/O, capacity/length failure and invalid selected face are
fatal. No decoded face means no candidate, not a post-selection silent discard.

Fontconfig includes use iterative canonical-file deduplication and sorted directory
entries. Verify config length before parsing; reject internal XML entity declarations,
set input-derived node limit `2*count('<')+1`, extract directives then discard bytes.
Relevant dir/include/reset-dirs/generic aliases retain locked fontconfig-parser0.5.8
semantics. Missing optional paths are ignorable; discovered-file failure is not.
Partial relevant-directive parse failure retains earlier directives **and failure
status**, so global/default config fallback still runs. Empty family elements are
skipped; malformed XML contributes no directives. Real storage/I/O failure remains
fatal, not a smaller fallback environment. Do not claim unrestricted upstream parity
outside explicit XML/entity/length restrictions.

## Axes And Required Tick Geometry

Each axis measures at one assigned panel-parallel span and returns a frozen tick
plan plus complete perpendicular gutter. Initial gutters are intrinsic; changed
actual gutters return to root feedback. Parallel-fitting source with larger gutter
is provisional, not permission to coarsen on stale input. Source/format policies
and exact precedence are owned by numeric presentation.

Categorical axes retain every semantic tick/label. At span S/count n, equal band
edges are exact nearest-even S*i/n, centers S*(2i+1)/(2n), including origin. Native
Heatmap axes instead consume actual allocated producer cell edges, never rebuild
equal bands. Measure each label at its cell width with correct orientation and
complete outward interval/clearance. Only an explicit existing Heatmap label-fit
policy can transform that family's category text.

Numeric labels receive nonoverlapping cells bounded by adjacent projected-center
midpoints and panel endpoints minus clearance. Endpoints place inward, interior
center. Exact labels need not be unique; nonempty strings must fit whole. Empty
exact labels retain tick/grid positions without text baseline/ink/gutter/collision.
Candidate/final placement share canonical anchor and actual-ink predicate.

From outside, top/left order is title, labels, ticks, stroke; bottom/right from
panel is stroke, ticks, labels, title. Labels are projected overlays, not a parallel
sequence. Gutter contains full perpendicular ink and owned gaps/stroke/ticks/title.
Horizontal titles use0 rotation; left/right exact-90/+90 quarter-turns with stored
canonical transform/ink, no trigonometric recomputation.

## Guides, Specialized Geometry And Overlays

One guide owner handles Symbols/ContinuousColor/Alpha under one frame. Auto outside
tries Right,Top,Bottom,Left, each normal then effective compact, at most8 evaluations
per owner/root round; explicit outside at most2 at its fixed edge. Ancestor calls
cannot reset that budget. Identical cached trials do not consume a new evaluation;
changed constraints do. Exhausting evaluated fit choices gives fit failure;
stopping before choices finish due to the shared trial limit is an explicit
planning-limit error. Sections add no budget or independent placements. Each
success includes body/axes/labels/annotations, not guide-only fit.

Explicit inside anchors remain fixed collision overlays. One selected complete
guide is one obstacle. Outside suffix aligns Start/Center/End along the parallel
track; Auto centers. Multiple guide tracks put first canonical nearest panel;
top/left reverse geometric assignment, not semantic order. Auto retains its direct
owner/address with a routed side reference, never reparents or duplicates a child.
Detailed section/gradient allocation belongs to the guide system.

| Interior family | Native geometry policy |
| --- | --- |
| Standard | Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath, Regression; standard axes |
| Categorical height | PointInterval, Coefficient, Forest, Dumbbell, HorizontalBar, StackedHorizontalBar |
| Categorical width | Bar, StackedBar, Boxplot, Violin, Raincloud, Interaction |
| Heatmap | Column-local minima, width-constrained cell/merged-row text then row allocation; retain every cell/missing/highlight/trailing slot and actual unequal axis bands |
| Matrix | Declared variable square including diagonal/off-diagonal cells; preferred side max(hard width,hard height,12em), row/column maxima; reuse retained statistics and span-specific axes |
| Marginal | 2x2 Grid: top(0,0), empty(0,1), central(1,0), right(1,1); exact linked widths/heights |
| Beeswarm | Source-ordered greedy tangent packing at assigned numeric width, then occupied categorical tracks |
| Waterfall | Native category bodies/adjacent connectors and source-anchored greedy base/terminal shelves |

Heatmap/Beeswarm use measured native right guide track; Matrix its trailing track.
Heatmap column/row minima include actual final contrast text/stroke/cross/outline
and label requirements, not global max-label-width times all columns. Marginal top
height preference is max(hard,ceil(central natural height/4)); right width similarly
central natural width/4. Derive once, no alternating subpanel search. Domain changes
may rebuild render grids from retained samples, not recompute statistics.

Beeswarm visits categories semantically and points by original source order. Use
radius containing full stroked glyph. Try zero, then same-category prior-point
tangent offsets `yj +/- ceil_sqrt((ri+rj)^2-(xi-xj)^2)` where x distance<radius sum.
Order by absolute displacement, negative before positive, source-row tie; deduplicate
offsets. Take first passing all prior collision tests; strict squared-distance
inequality overlaps, equality is tangent. No backtracking or height-driven repack;
insufficient assigned height misses without hiding observations.

Waterfall visits eligible base then terminal total at exact scientific source
anchors. Choose lowest shelf row whose complete horizontal ink intervals satisfy
gap, otherwise append one. Row0 nearest body, increasing upward, with separate
maximum ascent/descent, common baselines, per-edge padding and row gaps. Store all
rows/connectors/labels/reserves before capture; do not clamp total labels horizontally.

Overlay anchor order is projected, panel_center, panel_top, panel_right,
panel_bottom, panel_left, panel_top_right, panel_bottom_right, panel_bottom_left,
panel_top_left, attached_start, attached_end, bracket_shelf, waterfall_shelf.
Projected labels only use projected; configured panel text/inside guides keep
their anchor; attached uses start then end; significance/Waterfall use their shelf.
Visit records canonically against static parent obstacles and accepted earlier
records. Touching at required clearance is valid. No sibling backtracking or
obstacle feedback loop; exhausted anchors terminally miss.

Significance brackets sort `(start,end,lower_target,upper_target,annotation)` and
choose first collision-free level for span union centered complete label/stroke
ink. Step=max(selected step,label height+label gap+tick height), with complete exact
requirements rounded once. Shelf clears target ink/data gap/half-stroke; horizontal
orientation uses exact axis swap/quarter-turn. Required annotations, panel text,
cell text and supplemental labels fit their actual containing boxes, not merely
somewhere inside root. Scientific windows are the explicit clipping exception in
rendering, not layout-overflow permission.

## Topology, Feedback And Reuse

Structural addresses are closed `(role_discriminant,slot_index)` sequences in
`src/layout/kernel.rs`. Preserve canonical preorder and stable optional slots.
Inline tick vectors, flow rows/columns, glyph/mark records, packing offsets and
track values do not create anonymous measured calls or duplicate children.

Plot/Canvas roots flow title,subtitle,body,caption. A Plot frame uses outside/axis/
panel/axis/outside rows/columns, visiting panel subtree, axes Top/Right/Bottom/Left,
explicit guide track sets then direct Auto. Data panel owns interior/marks/inside
guides/annotations/overlays once. Facets use Plot frames beneath one owner, not
new roots; fixed dimensions use outer shared axes with corresponding empty cell
axes, free dimensions local axes with empty outer slots.

Canvas cells are Plot/TextPanel/empty, row-major with all declared holes/trailing
slots. Spanning records occur once at minimum covered coordinate, ordered header,
row title, range group,cell, then declaration. Range groups are nonvisual.
Embedded Plot title/subtitle/caption remain local exactly once; Canvas prose is
independent. TextPanel fills its surface and places nonfill required title/body
top-left after inset; extra space expands chrome, not text. No topology repacking.
Specialized records retain semantic category/source/declaration order; guide
sections are stable descendants of their original guide slot.

### Root Coordination

At most **eight rounds cover the entire root**, with no nested convergence loop.
Round1 uses prepared base domains, intrinsic gutters and empty reserve/alignment.
Only these named consequences transfer:

| Consequence | Update |
| --- | --- |
| Axis gutter | Actual complete thickness replaces estimate |
| Shared alignment | Coordinatewise maxima of nonnegative member leading/trailing panel-to-cell extents raise corresponding tracks |
| Owned ink | Nearest containing sequence/Grid gains outward reserves by coordinatewise max with already consumed cumulative reserves, applied once |
| Automatic domain | Union prepared base, prior cumulative envelope and new source-backed request; exact-domain changes are terminal |
| ContentHeight | Replace the scoped guided-cell measured height, including contraction; never cumulative ink or width reselection |

Requests name source,target,kind and resolvable predicate; conflicting nonaggregating
identities are internal errors. Shared guide merging is preparation, not feedback.
Breaks/tick sources/edges/packing/track choices are payload, not new consequence
kinds. Later domains use already sealed text grammar/face lists.

Converge when effective outgoing feedback equals consumed feedback, no miss remains
and complete immutable plan passes semantic/geometry/containment/root-ink validation.
Empty request list alone is insufficient. Unchanged provisional miss becomes its
terminal fit error. A nonadjacent repeated complete normalized state immediately
fails; changed/nonconverged eighth round fails exhaustion. One through eight rounds
may succeed without mandatory duplicate measurement. Never publish previous best.

Exact state includes consumed/next feedback, root request/dimensions, constraints,
assigned obstacles, topology/domains/preferences, full used role/font provenance,
plans/misses, local decisions, boxes/ink/baselines/text/glyphs/ticks/guides/tracks.
It excludes diagnostic counters/cache-hit statistics. No epsilon/hash-only equality.

### Scoped Caches And Height Consequences

Root-local reuse keys complete prepared immutable inputs, owner/scope, normalized
constraints, actual assigned geometry/obstacles and relevant consumed feedback
subtree/explicit ancestor dependencies. Propagate dirty inputs and reuse unchanged
outcomes; unchanged request list alone is not purity proof. No cache survives an
operation or failure. Optional exact text memo stops insertion at 256, continues
rendering and retained hits; it is not a rendering quota.

Content-height dependencies include prepared scientific sources, Labels, raw/color
domains/provenance, source-row identities, complete Theme/used fonts, both complete
TickGeneration choices, numeric policies/value sets, three alpha inputs/actual
members, guide semantics/visibility/composition, current width/capacity, local/
parent insets/gutters, scope and label visibility. Measurement/reuse/replacement
requests use the same identity. Equal width/height/output bytes do not authorize
reuse. Provisional height resolves only its named row/residual predicate under
the existing root driver; unchanged height cannot authorize overflow outside row.

Guide dependencies distinguish actual descriptor+allocation consumption from
allocation-only residual consumption by a Shared-suppressed child. Enumerate
dependencies at the actual call, not by scanning unrelated siblings. Owners
outside a child subtree still must be stable when their descriptor/allocation/
feedback is consumed. Complete scientific alpha identity persists even Hidden
or Undeclared. Detailed guide masks/dependencies belong to the guide specification.

## Storage, Freeze And Publication

Production has no cumulative accounting of visits, planning records, topology,
measurement calls, cache insertions, discovery events or estimated bytes. No
claim ticket, quota switch, fee-only error or replacement whole-operation score
is permitted. Retain independent operation/source identity, cache synchronization,
full equality, font census sealing, real length/capacity checks and fallible
first-party allocation. External allocator OOM is not promised recoverable.

Lifecycle order is semantic/option validation, scientific/domain/guide preparation,
complete text census and fonts, intrinsics, one width choice, bounded constrained
rounds, complete fit validation, immutable capture/count/reservation, one final
Null lifecycle, concrete same-plan replay and format-local publication. Capture
each selected producer once; never rerun it to determine size. Count serialization
from immutable commands, reserve canonical output and scratch before Null.
Count-only work opens no backend and publishes nothing. Final draw does not
measure/shape/format/read sources/choose policy or allocate new planning work.

Canonical dimension is positive u32 `ceil_div(logical_quanta,1024)`. Exact pair and
exact width reproduce requested dimensions; no clamping/retry. SVG root dimensions/
viewBox are integral, internal geometry remains quantized. For signed quantum q,
serialize magnitude whole `/1024` and remainder `%1024`; nonzero remainder times
9765625 gives ten zero-padded fractional digits, trim trailing zeros. Minus only
for nonzero, no exponent/locale. Bottom-up adapters transform checked root_height-y
exactly once. Null/concrete share coordinates. Counter and serializer share exact
token/escaping rules; emitted canonical bytes equal actual count.

Later outlined/PNG conversion may extract paths deterministically from frozen glyph
IDs/positions/transforms and immutable retained face bytes, with checked fallible
format buffers. It may not rediscover fonts, reshape, wrap, change bounds or
relayout. SVG has no PNG-derived dimension/pixel/parser-byte gate; PNG checks actual
outlined payload and its own target as specified in rendering. At300DPI, dimension
d becomes checked `((25*d)+4)/8`. A format failure does not mutate the scene or
choose another representation/layout. Output dimensions stay canonical CSS pixels.
