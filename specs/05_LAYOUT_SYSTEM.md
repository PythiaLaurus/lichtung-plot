# Layout System

**Status**: Approved D1-D6 target; non-current until Phase T6

## Purpose And Boundary

This is the single target layout architecture for `lichtung-plot`. The current
exact-size public API in `01_ARCHITECTURE.md` and `02_RENDERING_CONTRACT.md`
remains unchanged until the atomic Phase T6 migration in
`91_IMPLEMENTATION_PLAN.md`. Target names are not aliases, dormant public types,
or a second rendering route. This specification also defines the corrected T5a3
resource and workload target; historical global-search acceptance is superseded.

The D1-D6 user-visible behavior decisions are approved: bounded containing-frame
recovery, strict Unicode wrapping, three sizing modes, PNG-local output gates,
base-plus-mark font preference, and separate root title and caption alignment.
Approval is not implementation evidence. T5a0s in `91_IMPLEMENTATION_PLAN.md`
records the decisions and documentation review. Independent title/subtitle
alignment and narrow Plot/Canvas overrides are also user-approved T6 targets.
Broad public constraints remain deferred to DF6 review at T7 closure; remaining
broader text controls remain deferred to DF5 review at T6 entry. Neither checkpoint
is a promise to implement those extensions.

Current exact-root T5a6d implementation evidence is in `docs/t5a-closure.md` and
spec 91. T5a6d, T5a6, and parent T5a are COMPLETE after final review corrections
and the full gate in `/tmp/opencode/t5a6d-verify.log`: 1,024 library and 467
integration tests passed, 17 ignored, with strict Clippy, API docs, and package
verification passing. All ten release benchmark groups separately passed. The
20 native ordinary kinds, five specialized and two parent adapters, and deferred
Mark/Annotation consequences do not make this document's complete target current:
automatic sizing, native composition tracks, final specialized packing, and public
APIs remain in their later phases. T5b onward remain PENDING. Existing public
defaults remain exact 640x480 Plot and 960x640 Canvas until the atomic T6 switch.

Post-T5a3 accepted policies are recorded in `91_IMPLEMENTATION_PLAN.md`, T5a3p0
(PF1-PF7). The amendments below approve external conformance data, effective-
feedback fixed-point completion with dirty reuse, single producer capture, and
coarse resource accounting. P1 is complete; PF2/PF3 are implemented in `5c0acdf`
and PF6 in `23db99f`. PF4 is implemented and final primary verification and
independent recheck passed; PF5's evaluation/report is complete with unchanged
operation-local font freshness and same-Output reuse.
PF6 permits deduplication only with the exact selected tick list preserved; PF5
admits evaluation/reporting, not a global font cache. PF7 keeps current line
extension; alternative behavior is deferred to DF10 review at T7 closure.

Parents allocate space. A child returns ONE measured immutable local plan or a
typed fit miss under that allocation. Intrinsic summaries inform allocation but
are not collections of constrained layouts. Explicit, bounded local attempts
are allowed only where named below. There are no Cartesian child products,
global quality keys, complete wrap-transition sets, or exhaustive width search.
An automatic placement succeeds only when its affected containing layout fits,
not merely when the guide itself fits. Bounded parent recovery is distinct from
an exhaustive product of independently chosen child alternatives.

The guarantee is deterministic policy execution, not discovery of every feasible
layout or a globally best layout. A policy fit error can occur even if a different
unimplemented arrangement would fit. Exact roots neither shrink typography nor
omit content; automatic roots derive a natural preferred width once, then measure
content height. They do not start with a hidden fixed canvas and grow until fit.

The renderer remains standalone. It owns errors, annotation types, plans, font
policy, scene construction, and publication. No other `lichtung-*` crate,
network access, font provisioning, public layout plugin, or alternate renderer
is introduced.

## Private Protocol

```rust,ignore
type LayoutScalar = i64; // units of exactly 1/1024 CSS pixel

struct LengthConstraint {
    minimum: LayoutScalar,
    maximum: Option<LayoutScalar>,
}
struct LayoutConstraints {
    width: LengthConstraint,
    height: LengthConstraint,
}
struct LayoutSize { width: LayoutScalar, height: LayoutScalar }
struct MeasuredLayout<P> { size: LayoutSize, plan: P }
enum MeasurementOutcome<P> {
    Measured(MeasuredLayout<P>),
    FitMiss(FitMiss),
}
```

Intervals have nonnegative minima and optional maxima not below their minima.
`None` means no maximum, never infinity. Invalid constraints reject before
measurement; returned sizes must satisfy constraints without clamping. A fill
child receives an exact dimension; a nonfill child receives zero through its
assigned cell extent and is aligned by its parent. A `PanelBox` is an arranged
rectangle, not an incoming constraint or an intrinsic summary.

An intrinsic summary contains traced hard requirements, natural unwrapped text
width, preferred lengths, and width-dependence flags. It does not promise that
its minimum pair is jointly sufficient for every wrapping or packing outcome.
It is separate from constrained measurement and contains no plans at all widths.
Parents obtain summaries in canonical address order, allocate columns/widths,
measure width-dependent content, allocate rows/heights, and finalize geometry.
Named assigned-geometry calls receive exact containing size, projection identity,
and attachment targets. Each successful call returns just one plan.

Plans contain complete local child boxes, baselines, ink, text lines and glyphs,
ticks, guide flow, projections, and geometry. A parent may translate an immutable
child plan into its assigned box; it cannot mutate its decisions. Changed inputs
require another keyed call. Inline records belong to their measured owner and do
not manufacture anonymous child calls or duplicate topology addresses.

`FitReason` is `(kind, component_address, local_attempt_ordinal)`. Kinds sort as
`constraint, text, tick, legend_or_guide, track_or_residual, overlay_collision,
specialized_geometry, assigned_child`. Ordinals are zero-based in the owner's
declared local attempt order; a call with no alternatives uses zero. The original
child kind/address survives propagation; `assigned_child` is only a last-resort
mapping when no more specific reason exists. Reasons within an exhausted local
policy sort by that tuple and deduplicate; the least is returned.

A fit miss is either terminal or provisional with concrete named feedback
requests and enough provisional geometry to derive those requests. It is not an
empty successful plan. Only axis-gutter, shared-alignment, owned-ink reserve,
and automatic-domain feedback can make a miss provisional. A request must
actually change its consumed input and identify the predicate it can resolve.
An oversized unbreakable string, insufficient exact total after known minima,
or sibling collision without a named remedy is terminal. Resource, arithmetic,
font, and invariant errors are fatal immediately, never fit misses.

When child boxes can be constructed, siblings run in canonical order even after
a fit miss, so a later fatal error wins; otherwise the least terminal reason
wins, then provisional requests aggregate. Local attempts continue only after a
terminal fit miss that the explicitly named alternative can address. A placement
owner may change an automatic guide edge or its declared flow mode and remeasure
all children whose allocated constraints change. A child failure attributable to
that allocation rejects the containing placement attempt, even if the guide fits.
This does not authorize independent sibling-choice products, changing explicit
caller policies, or treating font/resource errors as reasons to retry.
The first provisional local attempt also stops that owner's alternatives and
returns its named requests to the root; stale parent inputs cannot justify
silently choosing a less-preferred tick source or guide edge.

## Canonical Numeric Domain

`LayoutScalar` is signed two's-complement `i64`. Lengths are `0..=i64::MAX`;
positions and ink edges are `i64::MIN..=i64::MAX`. Box edges are ordered and each
difference must be nonnegative and representable as a length. Negative zero
normalizes to zero. Convert binary64 input from its exact binary rational value
multiplied mathematically by 1024, not a rounded intermediate binary64 product.

The exhaustive spatial rounding rules are:

1. Lower length constraints round toward positive infinity.
2. Upper length constraints round toward negative infinity.
3. Hard required lengths, reserves, and line occupancy round upward.
4. Lower ink/collision/containing edges round toward negative infinity.
5. Upper ink/collision/containing edges round toward positive infinity.
6. Font sizes, advances, offsets, baselines, preferences, alignment guides,
   spacing, stroke centers/widths, and derived positions round nearest-even.
7. Exact public integral CSS dimensions use checked multiplication by 1024.

Painted stroke edges use rules 4 and 5; hard extents including spacing use rule 3.
Spatial sums/differences use checked `i128` intermediates and checked conversion
back to `LayoutScalar`. Counts, products, areas, squared distances, byte counts,
and ratio products use checked `u128`; signed differences are taken in checked
wide signed arithmetic before taking their unsigned magnitude. Failure aborts
the root, never saturates, wraps, drops one attempt, or retries a smaller value.

One `em` is the normalized resolved role font size. One `ch` is the normalized
shaped advance of U+0030 using that role's stack, selected face, weight, size,
and shaping profile. Required U+0030 failure is a font error. Multiples such as
`6ch`, `66ch`, and `72ch` use checked arithmetic. Font metrics use exact rational
`design_value * font_size_quanta / units_per_em`, rounded by their spatial role.
Advances/offsets are retained precisely enough to round the complete run's hard
advance upward once, rather than summing rounded glyph estimates.

Nonspatial binary64 inputs to geometry, state, or identity must be finite and
normalize negative zero. For bits `b`, the unsigned total-order key is `!b` when
the sign bit is set, otherwise `b ^ 0x8000000000000000`. Equality is normalized
bit equality. Computation uses nearest-even binary64 without FMA substitution.
Strings compare by UTF-8 bytes; integers numerically; `None` precedes `Some`;
sequences compare elementwise then by length. No pointer or hash iteration order
participates in a layout decision.

Layout `log10` uses the correctly rounded ties-to-even exact-integer interval
algorithm in `src/numeric.rs`, with pinned `num-bigint 0.4.8`. It bounds the
atanh series with outward fixed-point arithmetic, starting at 128 fractional
bits and doubling precision until both rational bounds round identically.
The current root's operation ledger is mandatory: each input, exact-power
comparison, refinement, series setup/iteration, and final conversion is admitted
before work. Checked upper bounds on first-party BigUint temporaries are charged
before their algebra, including limb rounding and capacity growth. The existing
shared 1,000,000-visit and 512 MiB cumulative-byte limits bound this work and
scratch; there is no separate precision cap or unbudgeted production entry point.
Refusal and precision/scratch arithmetic failure are fatal and propagate through
Forest values, references, ticks, limits, and nice-range consumers.
Exactly representable powers `10^0..10^22` and exceptional inputs are handled
explicitly; rationality proves termination for all remaining binary64 inputs.
This replaces the previously prescribed CORE-MATH port, not its correctly
rounded result contract. Evidence includes MPFR 4.2.1 rounding vectors, platform
libm misses, every binary exponent, and near-one cancellation. Platform libm and
approximate fallbacks are not admitted. `decimal_power(e)` is
the correctly rounded binary64 mathematical `10^e`, using exact integer/rational
arithmetic for the pinned exponent table `-323..=308`, not `powf`.

`ceil_div(n,d) = n/d + u128::from(n%d != 0)` for positive `d`.
Exact-rational nearest-even compares twice the remainder with the denominator
and resolves equality to an even integer. `ceil_sqrt` and `floor_sqrt` are exact
integer operations, not floating approximations.

## Root Sizing And Preferences

| Target request | Policy |
| --- | --- |
| No sizing call | Derive natural preferred outer width once; measure at that width and derive content height. |
| `width(w)` | Allocate exactly width `w`, reflow at that width, then derive content height. |
| `size(w,h)` | Allocate exactly the normalized pair; return fit/error without changing either dimension. |

Plot and Canvas expose consuming `width(self, width: u32) -> Self` and retain
`size(self, width: u32, height: u32) -> Self`. One stored sizing request is
completely replaced by each sizing call: latest call wins, and `width` after
`size` clears the explicit height. No call means automatic width and height.
Dimensions must be positive integral CSS pixels; zero rejects at preflight under
the existing builder validation pattern, not by panicking in a setter.
All final boxes and backend dimensions are finite. Canvas owns composed
dimensions; an embedded Plot's standalone sizing request does not become a cell
preference. Facets belong to their one owning Plot. Constraint intervals remain
private implementation machinery, not additional public sizing modes.

Natural preference is bottom-up composition of summaries with traced insets,
gaps, guide requirements, and scientific panel preferences. For prose, preferred
width is `max(hard_unbroken_width,min(natural_unwrapped_width,66ch))` in its text
role. `72ch` is a readability diagnostic, not a cap or alternative width. Root
width is fixed from that initial composition; feedback reallocates within it and
can change content-derived height, but does not restart width selection.

For initial intrinsic composition, an automatic outside Plot guide or Canvas
shared guide contributes its preferred RIGHT-side configuration: preferred
one-column guide width plus the adjacent gap adds to the panel/body width, and
its natural height participates in the cross-axis maximum. Explicit top/bottom
guides instead contribute a width maximum; explicit left/right guides add width.
Process multiple guides in canonical order. This is one sizing preference, not
a placement commitment: later bounded edge attempts remain within the chosen
root width and do not restart intrinsic width selection.

For standard panels, with `m` prepared primary marks including coincident and
transparent marks, use tick-label typography:

```text
mark_pitch = max(1em, maximum outward width or height of any primary mark)
density_side = ceil_sqrt(max(1,m)) * mark_pitch
preferred_width = max(hard_width,66ch,density_side)
preferred_height = max(hard_height,ceil_div(3*preferred_width,4),density_side)
```

The empty mark maximum is zero. Hard panel dimensions are the maximum of one
quantum and traced representability, mark, interval, cell, annotation, and text
requirements, not an `80x60` floor. At assigned width, natural standard height
uses `max(hard_height,ceil_div(3*assigned_width,4),density_side)`. The `4:3`
preference and density are soft inputs: exact allocation may depart from them.
There is no hidden `640x480`, `960x640`, or fixed per-cell root default.

For a categorical dimension, `hard_pitch=max(1 quantum,maximum occupied slot
span)`, `preferred_pitch=max(hard_pitch,2em)`, and spans are semantic slot count
times pitch. Occupancy includes marks, intervals, value labels, and inside-band
annotations, not outside axes/guides. Heatmap/Matrix prefer `1:1` cells; standard,
categorical, Beeswarm, Waterfall, and Marginal central panels prefer `4:3`.
Current production families impose no exact aspect requirement. No additional
hard floor, aspect, preference, or root-size percentage may be invented locally.

## Tracks And Composition

The closed primitives are `Leaf`, `VerticalSequence`, `HorizontalSequence`,
`Grid`, and `Overlay`. Coordinates start upper-left, x rightward, y downward.
All gaps/insets resolve from adjacent role typography, stroke, glyph geometry,
and owned `CompositionTheme` relationships, never a root scale factor.

Track allocation has one deterministic policy per dimension. Columns precede
rows. A track is flexible exactly when at least one occupying child fills that
dimension; otherwise it is inflexible. Declared Canvas/facet coordinate tracks
have the noncollapsing flexible exception below. Initialize hard minima from
topology, then single-track requirements in
canonical child order. Process spanning requirements by increasing span length,
then canonical address. Raise covered flexible tracks by equal capped surplus
distribution until the span requirement, including its internal gaps, is met.
No headroom means a track fit miss. Inflexible tracks retain their required size.
Prepare preferred tracks with the same procedure starting at the hard vector and
using preferred requirements; cap only soft preferences at explicit maxima.

For available total `T`, subtract checked insets, gaps, and inflexible extents
before allocation. If subtraction would be negative or the flexible hard sum
exceeds the remainder, fail. Let flexible minima be `m[i]` and preferred sizes
`p[i]>=m[i]`, with `p[i]` no greater than any explicit maximum. If available
flexible total `A` is below `sum(p)`, perform ONE proportional shrink:

```text
C = sum(p[i]-m[i]); deficit = sum(p)-A
shrink[i] = floor(deficit*(p[i]-m[i])/C)
```

Give remaining shrink quanta to descending fractional-remainder numerators,
ties by ascending track index, skipping exhausted shrink capacity. This exactly
uses `A` without crossing minima. If `A>=sum(p)`, start at `p` and distribute
surplus equally among flexible tracks with headroom: quotient to each, one extra
to the first remainder tracks in index order, cap at maxima, remove capped
tracks, and redistribute only the undistributed remainder. Each redistribution
either finishes or removes a capped track; it is not a layout feedback loop.

When all maxima are reached, remaining space is explicit trailing parent slack,
not silently added to an inflexible child. A parent requiring full flexible
coverage rather than slack returns a typed miss. Spanning hard requirements are
rechecked against the allocated vector; proportional shrink does not authorize
violating one. A failure does not invoke another allocation search. A natural
dimension uses its preferred/content vector, not surplus up to a caller bound.

Grid lines are cumulative insets, tracks, and gaps. Spanned cells include all
covered tracks and internal gaps. Canvas/facet declared rows and columns,
including wholly empty and trailing tracks, are noncollapsing flexible tracks:
hard minimum one quantum, preferred minimum `1em` of the owning composition
role. Empty-slot children remain zero-size. Other unoccupied Grid tracks have
no implicit minimum. Sparse coordinates are never repacked.

Sequence natural sizes for children `(w[i],h[i])` and insets `(l,t,r,b)` are
`(l+max(w)+r,t+sum(h)+sum(gaps)+b)` vertically and
`(l+sum(w)+sum(gaps)+r,t+max(N,BA+BD)+b)` horizontally. `BA` is maximum exposed
baseline, `BD` maximum height below it, and `N` maximum non-baseline child height.
Empty maxima are zero. Absent optional children retain addresses but create no
sequence-gap adjacency; present empty text remains present. Grid gaps instead
belong to declared topology even beside empty tracks.

Cross-axis start/end use the corresponding edge; center uses nearest-even half
residual; baseline uses `t+BA-child_baseline`. A legend key exposes nearest-even
half its height and aligns to the label's first baseline. Other horizontal
sequences baseline-align children exposing a baseline and start-align the rest.
Root title and subtitle use independently resolved concrete `TitleAlignment`
values under the precedence in `03_THEME_SYSTEM.md`; root caption independently
uses the owning theme's `CompositionTheme::root_caption_alignment`, also
`TitleAlignment`. All four built-in presets use concrete `root_title_alignment`
`Center`, optional `root_subtitle_alignment` `None`, and caption `Start`.
Each effective value controls both line translation within its text block and
nonfill block placement across the owner's post-inset local root content span,
not the data-panel span. Embedded Plot titles/subtitles retain these same Plot
semantics within their allocated cell. Start/End are physical left/right,
independent of bidi direction. Headers, row titles, and strips center in their spanned cells.
Other nonfill Grid/sequence children start-align unless explicitly specified.

Fill policy is closed: frames, bodies, panels, interiors, mark layers, Canvas Plot
and TextPanel cells, and surfaces fill both axes. Horizontal axes and top/bottom
guide tracks fill width; vertical axes and left/right tracks fill height. Text,
glyphs, entries, individual guides, and overlays are nonfill. Empty slots are
nonfill. A primitive fills exactly the dimensions assigned as fill by its owner.
Root frame/body is the sole flexible main-axis child; root text stays inflexible.

## Text And Unicode

Text preserves original source bytes and is not normalized. Segmentation follows
Unicode 16.0.0 UAX #29 extended grapheme clusters and UAX #14 default line breaking
without locale tailoring, including BK/CR/LF/NL and GL/WJ rules. Paragraph bidi
uses Unicode 16 UAX #9, with an all-neutral paragraph defaulting left-to-right.
Mandatory breaks partition present text into `b+1` paragraphs, including empty
leading, interior, and trailing paragraphs; each produces at least one line.
CRLF is one mandatory break as required by the profile.

Wrapping uses only these legal Unicode opportunities. Failure does not authorize
emergency breaks, fit-driven rotation, typography shrinking, or content omission.
The fixed axis-title quarter-turns below are role geometry, not fit recovery.
New public controls for alternative text-fit behavior are deferred.

Pin one coherent Unicode-bearing dependency graph: segmentation, line breaking,
bidi, script/Script_Extensions, General_Category for mark classification,
normalization/casefolding for family names, and
shaping classification must agree with this release. A Cargo version alone is
not evidence of data compatibility. Record versions, data releases, checksums,
generation commands where applicable, and conformance fixtures. Pinned dependency
tables/algorithms are permitted; bespoke first-party Unicode algorithms are not
required. Any shaper whose internal tables differ must be patched or supplied a
tested compatible classification path before it satisfies this target.

PF1 supersedes full-corpus repository/package storage requirements: keep focused,
independently expected default tests, runtime tables, hashes, provenance, and
licenses. Full conformance uses explicitly invoked ignored tests against externally
supplied official Unicode 16 data, hash-verified before parsing; missing or mismatched
data fails. No automatic download, provisioning, or hidden cache lookup is added.
Historical full packaged runs remain true evidence, not fresh-checkout coverage.

Use the locked `rustybuzz 0.20.1` shaping profile, language `und`, default
variation coordinates, empty caller feature list, and required/default OpenType
features, subject to that coherent-data requirement. Resolve Common/Inherited
script using Script_Extensions, surrounding strong script within the paragraph,
paired punctuation context, and inherited combining-mark base context. Prefer a
compatible preceding strong script, otherwise following; unresolved neutrals
remain Common. Do not split punctuation and marks into artificial script runs
merely because their raw Script property differs. This script resolution never
changes the canonical face-selection policy defined below.

Run boundaries follow resolved script, bidi direction, and selected face. A
grapheme can cross face runs when the narrow base-plus-mark preference below
cannot cover its unit or does not apply. This is not general whole-grapheme
fallback. Shape with
paragraph context and exact selected line boundaries, preserving joining and
OpenType context rather than summing independently shaped words. Retain face
identity/index, glyph IDs, source clusters, offsets, advances, directions, and
resolved scripts. Font/shaper failure or invalid metrics is fatal, not a fit miss.

Controls have explicit semantics, not a blanket cmap requirement. Mandatory
separators and bidi controls affect their Unicode algorithms but require no
painted glyph. ZWJ/ZWNJ, variation selectors, and other default-ignorables remain
in shaping input and source/cluster mapping with their prescribed control
behavior; absence of a standalone cmap glyph for them is not missing visible
text. Associate nonpainting controls with the relevant surrounding face run (or
primary face for an otherwise empty run) without changing visible-unit or scalar
selection. The shaper suppresses nonpainting control glyphs; a discretionary
break at U+00AD paints U+002D only when selected, and that literal participates in
font census; the original U+00AD remains in source mapping. Other unselected
default-ignorables paint nothing. Tab and unsupported C0/C1 controls other than
the named mandatory separators reject during text preflight rather than silently
disappearing; no implicit tab-stop policy exists.
Ordinary visible scalars, including combining marks, require coverage under the
canonical base-plus-mark/scalar policy and valid shaped glyphs; missing visible
glyphs remain errors, with no missing-glyph boxes or font downloads.
These are explicit nonpainting-control exceptions to the glyph requirement in
`03_THEME_SYSTEM.md`. Primary selection and canonical candidate order remain
unchanged; the approved narrow whole-unit preference below is the only change
to visible face selection. Neither script resolution nor control association
adds a general cluster-wide policy.

Intrinsic text analysis shapes each full paragraph ONCE and each maximal sequence
without a legal internal break, retaining summary requirements and bounded reusable
data. Natural width is maximum full-paragraph outward ink span; the unbroken
requirement includes outward ink and hard advance. Retain the paragraph shaping
and one source-ordered shaping-cluster advance map for constrained proposals,
not an all-suffix substring map. Reuse requires identical text, font/profile,
and paragraph context. Summaries are not proof of contextual constrained fit.

Legal endpoints are UAX #14 opportunities between UAX #29 grapheme clusters plus
paragraph end. Mandatory paragraphs are processed separately, with no proposal
crossing a mandatory break. A shaping cluster may cover several graphemes or
share source coverage with fallback runs. Keep its source interval and exact
rational total advance along its run's inline axis. For proposals only, use the
nonnegative magnitude of that total; accumulate clusters in logical source order,
charging a cluster at the first legal endpoint at or after its source end. Equal
source endpoints use run then glyph order. A legal grapheme endpoint inside a
shaping cluster remains legal: it receives no partial cluster estimate and is
reshaped exactly if tried. These proposal prefixes are monotone bookkeeping,
not independently shaped word widths or final line geometry.

At assigned finite width, first check the already shaped complete paragraph's
upward-rounded advance and outward ink span. If both fit, directly retain that
exact unwrapped line, with its line-specific bidi treatment, without reshaping
or endpoint search. Empty paragraphs retain one line with primary metrics.
Otherwise apply this deterministic locally greedy policy from paragraph start:

1. Scan legal endpoints forward from the current line start, using differences
   of paragraph proposal prefixes. Stop at the first proposal exceeding width;
   propose the preceding endpoint, or the first endpoint when none preceded it.
   If no proposal exceeds width, propose paragraph end. Do not scan backward
   from paragraph end or precompute substrings from every possible line start.
2. Shape the complete proposed line exactly with paragraph context and the
   selected break's control behavior. Check upward-rounded complete-line advance
   AND outward ink span. If it fails, retreat one legal endpoint at a time and
   shape/check each complete line until one fits. Failure at the first legal
   endpoint returns a text miss, not an emergency grapheme split.
3. From that fitting endpoint, try only the immediately next legal endpoint,
   shaping/checking its complete line exactly. Extend while each next line fits;
   STOP at the first exact nonfit or paragraph end. Keep the last fitting line,
   then resume forward proposal scanning at its ending source boundary.

PF7 accepts keeping this extension policy unchanged. Removing extension is not
admitted; DF10 reviews it at T7 closure only if measured benefit justifies review.

Within one line-start trial sequence, reuse an already tested endpoint result;
in particular, a failed endpoint encountered during retreat stops later extension
without reshaping. A paragraph-fit failure can likewise be reused at that same
start. Release failed trial glyph payloads and obsolete line-start cache entries
as scanning advances; retain selected exact lines and the one paragraph map.
Any optional retained trial cache is admitted incrementally under the same
record/work limits, never a complete boundary-pair or all-suffix collection.

Paragraph advances only propose a break; they are never exact line-fit proof.
Final geometry uses retained contextual whole-line shaping, not sums of standalone
words, glyph estimates, or paragraph-map advances. This policy deliberately does
NOT promise a globally longest fit under pathological nonmonotonic contextual
widths: it never skips a failing next endpoint seeking a distant fit. Every scan
and exact trial is charged incrementally as specified below. Expensive text can
still fail the admitted budget explicitly, without an uncharged search phase.

Compute paragraph bidi levels once with full paragraph context, preserve them
across soft wraps, then apply UAX #9 line-specific reset/reordering to each
selected line. Do not re-resolve each soft line as an independent paragraph.
The final exact line shaping is retained, not repeated by a publication backend.

Accumulate design advances in exact rational layout units in visual run order.
Each absolute glyph origin is nearest-even rounding of the exact accumulated
pen plus that glyph's offset, once; do not accumulate individually rounded
advances or add separately rounded pen/offset values. Freeze those absolute
origins. Compute outward painted bounds from the same origins and scaled glyph
outlines consumed by collision checks and outlined/PNG publication. The exact
full-line advance used for wrapping is a separate hard measurement, not a reason
to use different glyph placements during output.

Ascender/descender use OS/2 `sTypoAscender/sTypoDescender` when present, otherwise
hhea; invalid metrics, including nonnegative descender, reject. Let `A` and `D`
contain the maximum scaled positive ascender/absolute descender and selected
glyph ink above/below baselines across fallback faces. For no glyphs use primary
metrics. Hard containment rounds outward. With resolved upward-rounded role
line height, `pitch=max(A+D,line_height)` and
`height=A+D+(line_count-1)*pitch`. Baseline `i` is `A+i*pitch`; first baseline
is the sole sequence baseline and the last remains available for ink contracts.

Returned nonfill width is maximum selected line ink span, not unused assigned
width. For edges `left,right`, `span=right-left`, block width `W`, line x is
`-left` for start, `round_even((W-span)/2)-left` for center, and `W-span-left`
for end. TextPanel, legend title/label, guide title, and standalone text start;
composition headers, axes, ticks, and guide numbers center. Panel/annotation
edge anchors start at left, end at right, otherwise center. These are fixed role
decisions except for the root title, subtitle, and caption alignment rules above.

Editable preserve-text SVG emits original content with explicit selected line
boundaries/anchors and validated ordered font stack/weight, preserving paragraph
bidi context in its text structure/direction metadata. Viewer shaping and installed
fonts remain viewer-dependent; exact retained glyph geometry is NOT promised for
editable text. Outlined SVG and PNG consume frozen generation-host glyph identities
and positioning exactly, using retained paths or same-byte format-local path
extraction as defined below, with no second wrapping, fallback, or text shaping.
Source text remains available for accessibility/metadata without painting it twice.

## Fonts And Admission

Resolve themes, scales, semantic domains, exact limits, categories, shared-guide
merges, and any domain-dependent generated text sources BEFORE their font census.
Later geometry-dependent formatters contribute a closed alphabet, not guessed
sample labels. Automatic numeric/Forest alphabet is ASCII `0123456789-.ekM^`;
other generated literals, including discretionary break glyphs, are declared by
their owner. All possible painted alphabet scalars require fallback coverage.

Prepared text requirements visit canonical component preorder, role order, and
local record order. Every present caller/display string, exact label, annotation,
status, legend/guide, header, and generated text source participates. Empty exact
tick labels retain identity but zero scalar requirements. CSS family names, SVG
syntax, and explicitly suppressed content are not painted text. First occurrence
assigns each structurally equal `(FontStack,weight)` pair its ordinal, retaining
stack-entry order. Census visits pair ordinal, record order, source scalars and
required unit spans, then generated alphabet in scalar order. Retain required
base-plus-mark source spans and their face selections as well as scalar coverage
requirements; a character union alone is insufficient. Identical requirements
may share immutable data, but actual consumed occurrences charge shared processing visits.
Include typography metric probes. Standalone generated ASCII and metric glyphs
retain first-covering-scalar selection.

Canonical matching follows `03_THEME_SYSTEM.md`: `fontdb 0.23.0` is explicitly
loaded from admitted files, with decoded ID-16 and ID-1 family occurrences from
`ttf-parser 0.25.1`; named matching uses Unicode-16 NFKC-casefold. Generic entries
use the resulting database's host generic family names, without project remap.
Occurrences sort by `(stack_entry_index,family_match_rank,style_distance,
stretch_distance,weight_distance,sha256_bytes,face_index)`.

Family ranks are named ID-16 zero, named ID-1 one, generic two. Style distances
are Normal/Oblique/Italic `0/1/2`; stretch distance is from width class 5;
weight distance is from requested Normal/Semibold/Bold `400/600/700`.
For duplicate `(SHA-256 complete file bytes,u32 face index)` identities keep the
least occurrence tuple, then sort. First is primary. For a narrow unit consisting
of one visible non-M base and its Unicode General_Category M marks (Mn, Mc, Me) within
one extended grapheme cluster, choose the first candidate in that SAME ordered
list covering the entire unit. Nonpainting controls are not coverage
requirements. If no candidate covers the entire unit, retain per-visible-scalar
first-covering fallback in the same list; absence of a whole-unit face alone is
not an error. Leading marks without a visible base use scalar fallback. This is
not general emoji, ZWJ-sequence, whole-grapheme, or whole-word fallback and does
not join multiple visible bases into a unit. Visible scalars outside such a unit
choose the first covering face in the same list.

Retain complete ordered-list provenance for every first-use pair, required unit
source spans, whole-unit selections or scalar-fallback decisions, and selected
face identities. Paths, timestamps, database IDs, and discovery order are not
face identities. An empty list or uncovered visible scalar fails. A selected
face's shaping/metric failure is fatal, not a reason to try another face.

Use a bounded project-owned system-source walker, not unbounded
`Database::load_system_fonts`. Platform locations, regular-file/extension rules,
fontconfig roots, and generic extraction are specified against the locked
fontdb source with behavior fixtures; do not assert untested exact upstream parity.
Directory symlinks are not followed. Entries claim admission before processing,
canonical paths deduplicate, and admitted files order by source-location ordinal
then lossless native path units. Do not open all files simultaneously: open one
admitted regular file, obtain handle length, admit/reserve its bytes, read/hash/
decode that same bounded content, then close before the next file.

Retain admitted immutable font bytes through scene construction and any derived
glyph publication that needs them, or retain already-derived glyph geometry
that removes that need. Database loading, SHA identity, coverage, shaping, and
outlines must consume those same bytes; no selected-font reread or path fallback.
Short reads, changed lengths/content observed during admission, admitted-file I/O
failure, arithmetic/limit/reservation failure, and invalid selected faces are
fatal. Metadata admission follows the locked fontdb decoder: a file exposing no
decoded face contributes no candidate. That is not permission to discard a
selected face after shaping or metrics fail. No built-in-font, next-path,
network, or project-cache recovery is admitted.

Fontconfig configuration/includes and include-directory entries share discovery
and byte budgets with fonts. Includes are iterative, canonical-file deduplicated,
with sorted directory entries and bounded depth represented by admitted records.
Read an admitted configuration length before parsing; reject internal XML entity
declarations as syntax failure. Admit actual parser scalar input to the shared
processing ledger and set node limit `2*count('<')+1` before parsing; the prior
`2*count('<')+count('=')+1` synthetic parser-capacity price is superseded.
Relevant `dir`, `include`, `reset-dirs`, and generic-alias directives each claim
an entry before retained storage. Discard temporary configuration bytes after
bounded root/generic extraction; their cumulative charges remain.

For admitted inputs within the resource envelope, fontconfig directive and
fallback semantics follow locked `fontconfig-parser 0.5.8` as used by fontdb.
Missing optional paths are ignorable. A relevant-directive parse failure retains
previously accepted directives but also retains the failure status so fontdb's
global/default configuration fallback still occurs; partial results are not a
successful parse. Empty preference-family elements are skipped as upstream does.
Malformed XML contributes no directives. Resource or admitted-file I/O failures
remain fatal, not triggers for a smaller fallback environment. The bounded
adapter must prove these distinctions with differential fixtures for includes,
partial directives, reset-dirs, aliases, absent configuration, and default roots.
The explicit resource/entity restrictions above take precedence over unbounded
upstream parsing; matching the unrestricted parser is not claimed outside them.

## Axes And Ticks

PF6 implemented output-preserving deduplication of identical complete candidate
lists and repeated label measurements in `23db99f`; that optimization did not
authorize changing its greatest-count selection. The T5a5 numeric-source unit
intentionally supersedes that selector with the ordered first-fitting source
policy below. T5a5 also closes native axis geometry/text plans; its verification
and reviewed output changes are recorded in `91_IMPLEMENTATION_PLAN.md`.

Each axis measures at its one assigned panel-parallel span and returns one tick
plan plus required perpendicular gutter. The parent owns panel size. Initial
gutter estimates come from intrinsic title/label requirements; changed measured
gutters use only root-coordinated feedback. A source that fits the assigned
parallel span but needs a larger gutter is provisional, not a reason to coarsen
ticks merely because the previous round's gutter is stale.

Categorical axes retain one tick and present label per semantic category; no
stride, dropping, rotation, or empty-tick substitute. At span `S` and count `n`,
band edges are nearest-even exact `S*i/n` and `S*(i+1)/n`, centers
`S*(2*i+1)/(2*n)`. Measure each label once within its assigned band (swapping
parallel/perpendicular extents by orientation), using greedy text layout. Actual
outward intervals must remain in their bands with traced clearance.

A present exact list has `1..=10_000` entries, finite strictly ascending positions
within the final display domain. It is indivisible and overrides an interval
regardless of setter order; labels stay verbatim and need not be unique. Empty
exact labels intentionally paint no text, create no baseline/ink/gutter/collision,
and retain tick/grid positions. For nonempty numeric labels the parent assigns
nonoverlapping cells bounded by adjacent projected-center midpoints and panel
endpoints, with clearance removed. Endpoint labels use inward placement; interior
labels center. Measure once per cell and check actual intervals; exact failure
does not fall back to generated ticks.

Automatic range broadening is separate: the private nice-range seed count 5,
five-percent margin, and owned nice-step grammar affect domain snapping only.
They are neither generated-tick targets nor public count controls.

For no-interval linear generation, `pitch=max(1 quantum,6ch)` and
`upper_seed_count=1+min(9_999,max(1,S/pitch))`. Try desired seed counts from that
value down to 1, selecting the FIRST complete fitting source, then a terminal
`f64::midpoint(lo,hi)` singleton. There is no universal user tick-count target.
For each seed `k`, `raw_step=(hi-lo)/k`; if the subtraction is nonfinite use
`((hi/scale)-(lo/scale))*(scale/k)`, `scale=max(abs(lo),abs(hi))`.
Reject nonfinite/nonpositive raw steps locally.

Choose greatest decimal table exponent with `decimal_power(e)<=raw_step` and
test finite positive steps `10,5,2.5,2,1` times that magnitude. With no exponent,
test only raw_step. Select the admitted step minimizing `abs(count-k)`, ties
keeping the earlier larger step. If `lo.next_up()>=hi`, use `[lo,hi]`; a selected
vector with fewer than two positions likewise uses the validated endpoints.

Common linear-step generation computes binary64 `ceil(lo/step)` and
`floor(hi/step)`, requires exact integral indices in `[-2^53,2^53]`, calculates
count with checked wide arithmetic, and admits `1..=10_000` before allocation.
Positions are prescribed `(index as f64)*step`, finite, in the closed domain,
and strictly ascending. A too-dense/unrepresentable source rejects locally,
before vector materialization, without poisoning later coarser sources.

With `TickInterval::additive(step)`, try complete zero-aligned vectors for
`step*(1,2,3,5)`, then ten times those values, continuing by decades. Omit exact
duplicate steps. Stop after the first finite decade base greater than
`max(abs(lo),abs(hi))` or before the next nonfinite base. Requested interval is
first; first fitting complete source wins; only then coarsen; midpoint is last.
There are no interval subsequences or global joint-axis interval ranks.

Forest owns logarithmic source positions. Intersect its finite increasing
`[log_lo,log_hi]` with `[-323,308]`. Without interval, first try all integral
exponents when at least two exist, then evenly indexed subsequences in descending
size `2..=min(upper_seed_count,n)`, with index
`((i*(n-1))+((d-1)/2))/(d-1)`. If no integral source exists, use adaptive linear
sources over the logarithmic position domain. Try expression-labeled midpoint
last. Integral labels format exact decimal powers; adaptive/midpoint labels are
ASCII `10^` plus the formatted exponent, never `powf` or debug formatting.

With `TickInterval::log10_decades(d)`, try positive checked exponent steps
`d*(1,2,3,5)` by decades, each containing divisible integral exponents in the
intersection (at most 632). Stop after the first decade base beyond maximum
absolute clamped exponent or representability; no intersection charges one
requested miss then midpoint. Empty sources miss. No subsequences/adaptive
sources occur in interval mode. Always use the original domain for midpoint.

Generated labels use locale-free nearest-even decimal conversion: scientific
with two fractional digits for absolute value `>=1e12` or nonzero `<0.01`;
`>=1_000_000` uses millions with one fractional digit and `M`; `>=10_000` uses
thousands similarly and `k`; otherwise fixed precision 0 for `>=100`, 1 for
`>=1`, 2 below. Trim trailing fixed fractional zeros/point. Scientific notation
uses lowercase `e`, no positive sign, shortest exponent without leading zeros.
Normalize negative zero. If a complete list is not unique, retry the whole list
at scientific precisions `2..=17`, retaining first unique; otherwise source miss.

Numeric projection uses endpoints 0/1 and `(v-lo)/(hi-lo)` when both direct
subtractions are finite; otherwise it uses
`((v/scale)-(lo/scale))/((hi/scale)-(lo/scale))`, with
`scale=max(abs(v),abs(lo),abs(hi))`. In-domain fractions must be finite `[0,1]`.
With `D=2^64-1`, key is nearest-even exact `p*D`, and center nearest-even exact
`origin+S*key/D`, using normalized panel edges and exact integer ratio arithmetic
including origin parity. Marks, ticks, and grid lines share that projection;
out-of-domain clipped marks retain explicit extrapolation.
Categorical projections retain their semantic band count;
exact band edges/midpoints use integer ratios `i/n` and `(2*i+1)/(2*n)` including
the origin, rather than numeric keys. Off-center/jittered positions retain their
continuous projection and are never snapped to a category center. Cell midpoints
round nearest-even too. Keys must strictly ascend. Candidate and final axis label
placement share one `CssQuantum` anchor and actual-ink containment predicate.
Actual outward label intervals, clearance, tick geometry, and endpoint containment
determine fit, not width transitions.
Record source kind/ordinal, positions, label bytes, and retained glyph geometry.

Top/left axis order from outside is title, labels, ticks, stroke; bottom/right
from panel is stroke, ticks, labels, title. Labels are projected overlays, not
a parallel sequence. Gutter is complete perpendicular ink plus owned gaps,
ticks, stroke, and title. Horizontal titles have zero rotation; left/right titles
use exact -90/+90 quarter-turns around centered anchors, with transformed ink
and canonical rotation tokens stored, never trigonometric recomputation.

## Legends And Continuous Guides

Explicit placement fixes its edge/typed inside anchor. The approved bounded
automatic policy tries `right,top,bottom,left` in that order. For each edge, the
owning Plot frame or Canvas frame allocates space and measures the entire affected
body, axes, labels, and guide. The first complete fitting attempt wins; guide-only
fit is insufficient. Success requires that containing layout to fit.
If the right legend fits but leaves insufficient plot-label space, the right
attempt fails and another supported arrangement is tried before final fit failure.

Each edge first uses the preferred flow below, then may use its one compact flow
variant. Explicit edges can reflow within their assigned edge but never move to
another edge; explicit inside anchors remain fixed. No variant shrinks fonts,
hides entries, shortens labels, or changes caller data/scale policies. Changed
child constraints cause fresh keyed local measurement; unchanged plans may be
reused. Provisional gutter/domain feedback follows the root protocol rather than
being treated as terminal failure or committed success on stale inputs.

The policy has at most eight edge/flow trials per automatic guide owner per
root round (two for an explicit outside edge), charged cumulatively across all
calls in that round, including calls caused by ancestor attempts. Nested frames
do not reset those budgets or enumerate combinations of guide choices. Multiple
guides follow canonical owner order; each trial holds other guide decisions fixed
at their current choices, initially their preferred placements. Exhaustion of
admitted work remains a resource error, not a hidden best-effort rendering route.
If every supported trial under the current constraints was evaluated and failed,
return fit failure. If ancestor-driven calls consume the shared budget before
those trials can finish, return an explicit planning-limit error instead. Reusing
an identical cached trial charges a call but not another edge/flow evaluation;
changed constraints are a new charged trial. Do not report a budget stop as proof
that the content cannot fit.
This limited recovery still does not promise every theoretically feasible figure.
These approved D1 behaviors are implemented by their assigned repair and
component phases. T6 adds the new public sizing/theme surface; it does not
postpone correctness fixes in already-active automatic placement paths.

Outside guides are normal flow; inside guides are typed collision overlays.
No root percentage chooses typography, edge, or flow.

Determine entry widths BEFORE greedy flow, using available inner WIDTH `W`,
never height as a text wrapping width. `W` is the assigned legend width after
horizontal chrome insets; measure the title at `W`. Available entry-body height
`H`, when finite, subtracts vertical chrome, measured title, and its active gap.
Negative available space misses. Entry preferred width `P[i]` is its intrinsic
label preferred width plus exact key width and key-to-label gap; its minimum
`M[i]` similarly includes the label's hard unbroken width. All arithmetic is
checked. Entry allocated width below `M[i]` misses before text measurement.

For top/bottom horizontal flow, fix each entry width to `E[i]=min(P[i],W)`.
Measure its label once at `E[i]-key_width[i]-key_gap[i]`, derive its baseline-
aligned entry height, and keep that allocated width even if actual label ink is
narrower. In semantic order, append entries left-to-right while the next entry
and inter-entry gap fit `W`; otherwise start the next row. Each row height is
the maximum entry height, with traced gaps between rows. Finite `H` must contain
the resulting total height. This is the preferred flow; only the compact variant
below may use a narrower entry width.

For left/right vertical flow, choose ONE uniform column width
`C=min(max(P[i]),W)`. Check every `M[i]<=C`, then measure each label once at
`C-key_width[i]-key_gap[i]`. Greedily append entries top-to-bottom in semantic
order while their heights and inter-entry gaps fit finite `H`; otherwise start
the next equal-width column with its traced column gap. An entry taller than
`H`, or columns whose total widths/gaps exceed `W`, misses. With automatic
height, use one column and derive its height. Do not mutate a completed flow;
the compact variant below is a separate bounded trial with newly measured labels.

For the compact trial, let `g` be the horizontal entry/column gap and
`C2=floor((W-g)/2)` in layout quanta. A nonpositive `C2` or an entry minimum above
its compact allocation rejects this variant. Horizontal entries use
`min(P[i],C2)` then the same ordered row flow. Vertical flow uses uniform width
`min(max(P[i]),C2)` then the same ordered column flow. If allocations duplicate
the preferred trial, reuse its outcome. There is no scan over all column counts
or widths, and exact label shaping still determines height and containment.

Worked horizontal example, in CSS pixels after chrome/title: `W=100`, `H=50`,
key-plus-gap 20 for each entry, and label preferred widths `[40,30,70]` give
fixed entry widths `[60,50,90]` and label widths `[40,30,70]`. With inter-entry
gap 5, each entry starts its own row. If measured entry heights are `[10,10,20]`
and row gap 4, total height is `10+4+10+4+20=48`, so it fits. Entries are not
shrunk in this preferred trial. If the containing frame fails, the compact trial
may assign narrower widths and remeasure; the title/plot must also fit before
either trial is accepted.

Worked vertical example, after chrome/title: `W=150`, `H=45`, preferred entry
widths `[60,70,50]`, minima at most 70, and key-plus-gap 20 give `C=70` and label
width 50 for every entry. With measured heights `[20,20,20]`, vertical gap 5,
and column gap 10, the first two entries fill column one exactly to 45; the
third occupies column two. Total width is `70+10+70=150`. At `W=140` the preferred
trial misses, but the compact trial uses width-65 columns. It fits only if minima
permit and the newly measured entries still fit the available height; it cannot
assume their old heights. With automatic height the preferred trial uses one
width-70 column of height `20+5+20+5+20=70`.

No entry permutations or additional row/column-major alternatives exist. Title, keys,
labels, baseline heights, gaps, padding, and border ink all participate in final
fit. Zero entries mean an absent legend, not a detached title. Flow slots and
allocated entry widths are plan payload, not semantic topology.

Outside suffix alignment is start/center/end along the panel-parallel track:
right/left Top/none/Bottom and top/bottom Left/none/Right respectively. Automatic
uses center. Track sets stack first canonical guide nearest panel, subsequent
guides outward; top/left reverse geometric assignment, not semantic traversal.

An automatic Plot guide stays a directly owned `outside_guide_auto` child after
the four explicit guide track sets. The selected side holds one non-owning routed
reference, never another child/address/call. Canvas shared guide uses the same
rule at its one `canvas_shared_legend` address. Edge is immutable plan data, not
reparenting. Explicit guides belong to their side's track set.

Canvas prepares one kind-tagged shared descriptor under
`canvas.legend.shared-entry-exact-merge`; compatibility, exact entry identity,
and continuous-domain merging occur before layout/font census. Shared labels are
formatted once under Canvas typography from the final merged domain, never copied
from arbitrary contributing Plot labels.

Continuous orientation is horizontal top/bottom, vertical left/right. Labels
are lo, an interior diverging center if present, hi; constant domains have one
midpoint label. Apply generated numeric uniqueness grammar. For outward extents
`(Ls,Rs),(Lc,Rc),(Le,Re)` and gap `g`, gradient length is at least
`max(Rs+g+Le,2*(Rs+g+Lc),2*(Rc+g+Le))`, omitting center terms without a center,
and at least traced minimum length. A traced maximum below requirement misses.
Constant-domain requirement is `2*max(Lc,Rc)+2*g`. Endpoint overhang is explicit
flow from `-Ls` through gradient end plus `Re`, not clipped.

Gradient thickness is traced key width. The inline plan is title, gradient with
projected labels (label column beside vertical gradient, row under horizontal),
then optional missing key/label block in reading order, with traced gaps/insets.
The gradient and guide text are inline owned records, not measured children or
independent orientation/length searches.

## Specialized Geometry And Overlays

Family ownership is exhaustive:

| Interior | PlotKinds / policy |
| --- | --- |
| Standard | Scatter, Histogram, Density, Qq, Line, Trajectory, LinePath, Regression; standard both axes. |
| Categorical | PointInterval, Coefficient, Forest, Dumbbell, HorizontalBar, StackedHorizontalBar; categorical height. |
| Categorical | Bar, StackedBar, Boxplot, Violin, Raincloud, Interaction; categorical width. |
| Beeswarm / Waterfall | Named specialized owner, categorical height and assigned standard width. |
| Heatmap / Matrix / Marginal | Named parent-owned track producers below. |

Heatmap retains every semantic row/column/cell, missing cell, highlight, merged
row, value label, and trailing slot. Column/row minima are maxima of complete
cell requirements; preferred pitch is at least `2em` with soft square preference.
Allocate columns, measure cell text at those widths, derive row requirements,
then allocate rows with the common track allocator. Matrix retains the declared
variable square including diagonal cells; preferred cell side is
`max(hard_width,hard_height,12em)`, aggregated by column/row maxima. Both store
final cell rectangles and mark geometry; neither enumerates track combinations.

Marginal is a two-by-two Grid: top `(0,0)`, empty `(0,1)`, central `(1,0)`, right
`(1,1)`, with owned gaps. Central/top share width, central/right share height.
Top preferred height is `max(top_hard_height,ceil_div(central_height,4))`;
right preferred width is `max(right_hard_width,ceil_div(central_width,4))`.
Derive these from the central natural preference once, then parent allocation
and exact shared child spans determine one layout, not coupled local iteration.

Beeswarm processes categories semantically and points by source-row order at
assigned numeric width. Radius `r[i]` contains complete stroked glyph half-extents.
Try zero offset then tangent offsets from previously placed same-category points:
`y[j] +/- ceil_sqrt((r[i]+r[j])^2-(x[i]-x[j])^2)` when x separation is less than
radius sum. Order offsets by absolute displacement, then negative before positive,
then source-row index; deduplicate equal offsets. Take first offset passing ALL
prior-point collision checks. Squared distance below squared radius sum overlaps;
equality is tangent. Charge each generated offset and tested pair before work.
No branching over prior placements or category products occurs. Derive occupied
category height from this one packing, then common categorical tracks. If assigned
height cannot contain it, miss rather than repack or hide points.

Waterfall stores one bar per category and connector per adjacent pair. Present
in-domain base then terminal labels use greedy shelves: visit in that order and
choose the lowest row whose final horizontal ink intervals satisfy traced gap;
otherwise append one row. No row-index vectors are enumerated. Row zero is
nearest body, increasing upward. `BA/BD` maxima give row height; from
`body_top-bottom_padding`, place each row upward, common baseline at row top+BA,
with inter-row gaps and final top padding. Labels center on exact shelf targets.
Category value labels remain inside their slots and contribute hard pitch. Store
rows, baselines, connectors, labels, and band reserve before drawing.

Overlay anchors have closed order `projected,panel_center,panel_top,panel_right,
panel_bottom,panel_left,panel_top_right,panel_bottom_right,panel_bottom_left,
panel_top_left,attached_start,attached_end,bracket_shelf,waterfall_shelf`.
Projected cell/point labels have only projected; configured panel text/inside
legends keep their configured anchor; attached labels try start then end;
significance and Waterfall use their one shelf producer. Edges/corners align
outward ink after inset; single edges center the other axis. Attached start/end
place before/after the target on the declared x/y axis with traced gap, centered
orthogonally. Projected anchors use final projection plus traced offsets.

The overlay owner visits records canonically and selects first legal anchor
against static obstacles and already accepted earlier records. Parent supplies
the deterministic current obstacle context before the keyed call; no later
sibling backtracking or obstacle feedback loop occurs. Touching at required
clearance is allowed. A collision after listed attempts is a terminal miss.

After applying the closed source-owned reserve/domain rules, all remaining
painted ink outside its exact assigned containing box is a terminal fit miss,
even when it remains inside the root. Overlay ownership or collision-free
placement alone does not authorize overflow into neighboring axes or cells.

Significance brackets sort `(start,end,lower_target,upper_target,annotation)`;
occupied interval is span union centered label ink plus clearance. Assign the
smallest level with no intersecting prior occupied interval, charging each level
test. `level_step=max(traced_step,max_label_height+label_gap+bracket_tick_height)`.
Shelf is above minimum target ink top by data gap plus level*step; label baseline
is shelf minus label gap minus descent, centered over span. Horizontal-value
orientation uses exact axis swap/quarter-turn. Stroke/label union is owned ink.
There is no draw-time reserve, packing, shelf choice, or generic grow-until-fit.

## Topology And Ownership

Semantic content must occur exactly once unless an explicit owned caller policy
authorizes its omission/transformation. No implicit clipping, elision,
abbreviation, substitution, category dropping, scrolling, or scale-to-fit text.
Automatic numeric ticks are renderer representation; exact ticks are caller
content. Existing visibility/elision paths must be re-adjudicated before T6.
Unsupported Canvas/facet family nesting rejects in semantic preparation.

Structural address is a sequence of `(role_discriminant,slot_index)` pairs.
Preserve this closed role order:

```text
root, root_title, root_subtitle, root_body, root_caption,
plot_body, canvas_body, canvas_grid, facet_grid, canvas_shared_legend,
canvas_header, canvas_row_title, canvas_range_group, canvas_cell,
facet_header, facet_row_title, facet_range_group, facet_cell,
data_panel, panel_interior, specialized_interior, mark_layer, mark_item,
heatmap_cell, heatmap_merged_row, heatmap_value_label, heatmap_highlight,
beeswarm_category, beeswarm_point, marginal_central, marginal_top, marginal_right,
matrix_cell, matrix_mark, categorical_category, categorical_series_slot,
categorical_body_mark, categorical_interval, categorical_value_label,
waterfall_bar, waterfall_connector, waterfall_base_label, waterfall_terminal_label,
legend_title, legend_entry, legend_key, legend_label,
guide_title, guide_gradient, guide_label, guide_missing_block,
surface, text_panel_content, text_panel_title, text_panel_body,
inside_guide, annotation, annotation_target, other_overlay,
axis_top, axis_right, axis_bottom, axis_left, axis_title,
outside_guide_top, outside_guide_right, outside_guide_bottom,
outside_guide_left, outside_guide_auto, text, glyph
```

Closed measured types retain order `empty_slot,canvas,plot,plot_body,plot_frame,
canvas_body,canvas_grid,facet_grid,track_set,data_panel,standard_interior,axis,
discrete_legend,continuous_guide,composition_text,text_panel,text_panel_content,
legend_entry,annotation_layer,overlay_layer,heatmap_interior,beeswarm_interior,
marginal_interior,marginal_subpanel,matrix_interior,categorical_interior,
waterfall_interior,mark_layer,text,glyph`. PlotKind uses public declaration order.
Root is a role, not a third root type.

Traversal is preorder. Plot/Canvas roots are vertical title, subtitle, frame/body,
caption. Absent named roles are addressed empty slots. Plot frame is a five-by-five
Grid with rows outside-top, axis-top, panel, axis-bottom, outside-bottom and
columns outside-left, axis-left, panel, axis-right, outside-right. Visit Plot body
and its complete panel subtree, axes top/right/bottom/left, guide track sets in
that order, then direct automatic guides. Plot body transparently fills one data
panel or facet Grid. Data panel owns interior, ordered mark layers, inside guides,
annotations, other overlays. It is not repeated as a Plot sibling.

Canvas body uses that outer frame with Canvas Grid central and one optional shared
guide; hidden/per-Plot policies leave its shared slot empty. Canvas cells contain
Plot, TextPanel, or empty slot. TextPanel is a fill surface plus nonfill content
anchored top-left after chrome inset; content is required title then body text.
Extra exact space expands chrome, not text. Headers/row titles/strips are
`composition_text`; range groups are nonvisual records without measured boxes.

An embedded Plot retains its own title, subtitle, and caption exactly once in its
local root flow. Its allocated cell includes measured space for every present
text role; Canvas does not replace its titles with differently anchored headers
or omit its caption. The Plot caption remains local, with its resolved Plot theme's
independent caption alignment. A Canvas caption is separate Canvas-owned content,
not a replacement or a duplicate of panel captions. Insufficient allocated space
returns the applicable fit failure rather than omitting any of this content.

Canvas/facet coordinates are row-major, including declared empty/trailing slots.
Canvas shared-x groups contain configured Plot members with continuous automatic
x domains only; categorical axes, explicit x limits, and unconfigured members
reject during semantic preparation. Their union must have finite strictly
ascending endpoints and is installed before child measurement. Fixed facet
categorical axes require matching coordinated category identities; continuous
fixed axes use their coordinated domain union. Shared alignment never weakens
these data/scale compatibility checks.

Canvas/facet traversal retains every declared slot.
Spanning items occur once at minimum covered row/column; at one origin order is
header, row title, range group, cell, then declaration order. Spans remain in
topology. Facet cells are Plot frames under the owning Plot, not nested roots.
Root faceted frame axes are empty. Fixed facet dimensions use shared outer
axes/guides around the cell Grid and empty corresponding cell axes; free
dimensions use panel-local domains/axes and empty shared roles. Horizontal guides
and axes span central column only, vertical ones central row only.

Repeated slots use semantic category/entry order, source-row mark order,
declaration-order layers/annotations, target-address then declaration overlays,
and grapheme-order text; singletons use zero. No post-filter compaction or map
iteration assigns addresses. Heatmap traverses cells row-major, merged rows,
value labels, highlights; Beeswarm category then points; Marginal central/top/right;
Matrix row-major cells then source marks; categorical category/series then body,
interval,value label; Waterfall categories,bars,connectors,base,terminal,values.

These specialized items and generic mark items are addressed prepared records,
not measured children. Marginal subpanels ARE measured children. Discrete legend
title and entries are measured, each entry key then label. Continuous guide
title/gradient/labels/missing block are inline addressed records. Axis tick/label
vectors, text breaks, flow slots, packing offsets, and allocated track values are
owner payload without structural addresses. Each record occurs exactly once.

Every input has an owner-ledger or target-only key, canonical address, source
category, normalized value, consuming stages, and immutable plan field. Sources
are caller policies/content, resolved appearance/typography, font provenance and
glyph metrics, scales/ticks/guides, declared topology, scientific/collision geometry,
constraints/preferences/tracks, named feedback, resource/reservation counts, and
canonical serialization inputs. The trace is a design/test census, not a public
dynamic property map. Unowned layout inputs block implementation.

## Root Feedback And Cache

At most EIGHT root-coordinated rounds cover the entire Plot/Canvas tree, not eight
per child. No nested convergence loop is admitted. Round one consumes prepared
base domains, intrinsic gutter estimates, and empty reserve/alignment feedback.
Each round allocates and measures by the fixed policy and returns one provisional
tree or typed misses plus named consequences. Only these consequences transfer:

- Axis gutter: measured complete required thickness replaces the corresponding
  estimate; stale-gutter misses may proceed provisionally with that requirement.
- Shared alignment: declared Canvas range groups/fixed facet axes use exactly
  occupied members, leading `panel_start-cell_start` and trailing
  `cell_end-panel_end`, requiring nonnegative extents. Group coordinatewise maxima
  raise corresponding member leading/trailing track requirements.
- Owned ink: normal-flow overhang targets nearest containing sequence/Grid with
  outward edge differences as reserves, combined coordinatewise by maximum with
  already consumed cumulative reserves. Apply once as additional owned inset,
  not repeated padding for the same ink. All other unowned overflow is terminal.
- Automatic domain: projected semantic marks/data-anchored annotations requiring
  out-of-domain values target owning scale; union prepared base, prior cumulative
  envelope, and new requests. Exact caller-domain changes are terminal.

Requests identify source, target, and closed consequence kind; conflicting
nonaggregating identities are internal errors. Domain preparation must establish
all relevant text alphabets before font census; later envelope changes may
reformat only through those prepared alphabets and resolved face lists. Shared
guide semantic merging is preparation, never feedback. Collision, text breaks,
selected tick sources, legend edges, packing, and tracks do not transfer as
independent requests. Their consequences must use one named owner above.

State includes consumed/next feedback, root dimensions, constraints, assigned
contexts (including obstacles), topology, domains, preferences, resolved font
identities, every plan/miss and local decision, boxes, ink, baselines, text/glyphs,
ticks, guide slots, tracks, and reservations needed to interpret retained data.
It excludes cumulative operational counters and cache-hit statistics. Compare
exact normalized structural content, not hashes alone or epsilon distances.

PF2, implemented in `5c0acdf`, converges when effective outgoing feedback
equals consumed feedback, no misses remain, and the complete immutable plan has
passed semantic, geometry, containment, and root-ink validation. An empty request
list alone is insufficient. An unchanged provisional miss becomes its terminal
fit error. Retain exact state comparison for cycle detection: a nonadjacent
repeated state fails immediately; a nonconverged eighth round fails for exhaustion.
Successful convergence takes one through eight rounds, with no mandatory duplicate
measurement. Never publish a previous best plan.

Optional root-local caching stores one outcome per full normalized constraint,
assigned geometry/obstacle context, prepared input, and consumed feedback subtree,
including descendants. Changed font/domain/topology/text/alignment/ink inputs
invalidate reuse. Cache hits charge call attempts but not repeated producer work;
cached diagnostic identity is unchanged. Admit and fallibly grow entries as used,
not one million slots up front. No cache survives root completion/failure.

PF2 requires dirty dependency propagation and reuse of unchanged child outcomes;
remeasure affected owners when effective inputs change. Audit every invalidation
edge and mutable cache's output independence. Reuse is not a cache-purity assumption
derived from Rust traits or an unchanged request list.

## Resources And Lifecycle

PF4's accepted replacement contract below is implemented. Final primary
`just verify`, independent byte/variable-visit recheck, and all 27 baseline output
hash comparisons passed; evidence is in `docs/resource-admission.md` and T5a3p4
in `91_IMPLEMENTATION_PLAN.md`. These are static cumulative quotas, not CPU/RSS
guarantees. T5a3p closure does not start the later native layout phases.

| PF4 Resource | Inclusive Limit | Admission Point And Unit |
| --- | ---: | --- |
| Root processing visits, shared by font and text | `1_000_000` | Actual scalar inputs to variable text passes; normalization input/decomposition nodes and each actual preceding comparison in quadratic ordering; candidate, unit, endpoint, and list visits; collision pair tests. Admit known inputs in bulk before processing and variable visits before their guarded operation. |
| Retained planning records | `1_000_000` | Actual headers/elements, including glyphs, before first-party retention; immutable sharing counts once and copied values count again. |
| Prepared topology slots | `1_000_000` | Existing structural census before preparation. |
| Measurement/intrinsic call attempts | `1_000_000` | Before lookup, including hits. |
| Root-local cache entries | `1_000_000` | Before insertion. The exact text cache still optionally stops at `256` entries. |
| Font discovery entries | `1_000_000` | Before discovery admission. |
| Font/source/scratch bytes | `536_870_912` | Checked cumulative byte admission before read or first-party retention, including glyph output; no processing price for hashing, copying, or materialization. |
| Root feedback rounds / guide placement trials | `8` / `8` | Existing round/trial boundaries. |
| PNG axis / pixels / parser-input bytes | `32_768` / `100_000_000` / `67_108_864` | Existing PNG-local format boundaries only. |

One root processing-visits ledger exists from operation creation, before font
preparation. Font budgets, generation resources, and their clones share it.
Sharing resources must transfer consumed admissions or safely refuse a mismatched
started ledger; it must never reset or discard consumption. Nested replay owns
its independent record budget; copying child-cache values does not create a new
root quota. Claims are synchronized, not computed from racy usage snapshots.

Fixed Unicode-table binary-search comparisons and fixed `64`/`8`/`256` per-rule
allowances have no processing charge. Unicode's private callback stays a simple
`(bytes, visits)` admission interface; fixed searches need no callback, while
variable normalization ordering retains its actual-comparison guard. Reservations,
cache keys, producer capture, feedback, and glyph output claim actual records and
bytes, not a second processing price. Equality-based cache fallback remains valid;
pointer identity alone is not sufficient.

Shaping input is admitted at the canonical Rustybuzz call boundary, before buffer
construction and shaping, counting actual input plus at most five trailing
pre-context scalars and five leading post-context scalars, exactly as consumed by
locked Rustybuzz `0.20.1` (`CONTEXT_LENGTH = 5`). Do not scan or charge the rest of
either context string. Known output records and bytes are admitted in bulk
before first-party vectors or ink probes, avoiding per-glyph locks. Third-party
internals retain their bounded-input caveat. Guarded variable loops may batch only
without unpaid failure paths or concurrent oversubscription.

Failure precedence is checked representability, cumulative admission, fallible
reservation, then processing/retention. Existing typed arithmetic, resource,
reservation, and font refusals remain terminal; none triggers omission, a smaller
representation, or fit retry. Claims are cumulative and never refunded. The
million-visit magnitude protects actual variable and quadratic workloads without
artificial rule prices; a physically fitting 2,048-word workload is positive
evidence when its actual visits, records, and bytes fit. Successful glyphs, line
breaks, ticks, geometry, PF7 line filling, and script context remain unchanged.
Release evidence must report visits and records separately from CPU/wall time and
check all nine publication output hash sets against the preserved baseline. Fewer
accounting units are not evidence of speed. No global cache is introduced.

PNG dimension, pixel, and parser-input limits apply only at a PNG terminal, in
all three sizing modes. They are not shared layout or SVG admission gates.
Automatic SVG has no PNG-derived canonical byte cap either. All modes retain
the shared processing-visits and record caps, checked representability and byte
arithmetic, and fallible reservations; no replacement SVG byte cap is introduced.

Existing stricter semantic family limits remain. Structural slots are exactly
the addressed component/prepared-record census above, including sparse slots,
marks, cells, targets, categories, connectors, and entries. Planning records are
retained plan/state/cache/feedback headers and each retained line, glyph, track,
box, tick/label, flow slot, packing offset, or obstacle element; immutable shared
payload counts once, a copied payload again. Original semantic strings/font bytes
are admitted by their source census/byte rules, not counted as one record per byte.
No candidate-product or unused alternative-capacity counter exists.

Actual scalar inputs to paragraph partition, segmentation, line-break,
script-resolution, bidi, and shaping passes consume visits. Sorting and
deduplication bulk-admit their input record counts, not comparator prices.
Variable list searches charge examined entries rather than unused list tails.
Retained-face searches for shaping, metrics, and outlining, and guide-owner
searches for trials and ordinals, admit each entry before its comparison,
including the terminating match. Direct file-ordinal access has no list scan.
Storage-only traversals, copying, and hashing do not acquire a second processing
price. A cache hit does no producer processing; cache search still admits visited
entries, and retention still admits records/bytes before insertion.

Text proposal clusters, legal endpoints, and forward/retreat/extension steps
consume actual visits. An endpoint's exact advance and ink predicates together
count as one fit trial, not two; a reused failed result has a boundary visit but
no second shaping or predicate evaluation. The initial whole-paragraph fit is
one trial. PF7 line filling and early stopping at the first failed extension are
unchanged. Shaper output counts as actual retained records/bytes, not visits.

First-party paragraph/run headers, scalar-property/source-index metadata,
grapheme and legal-boundary entries, base-plus-mark unit spans/selections,
shaping-cluster/source intervals, proposal
prefix entries, trial-cache headers/results, and retained glyph/path-command
elements each count as one planning record per logical element. A fixed-field
record counts once; a variable payload counts its header plus each element,
not merely its outer vector. Shared identical immutable data counts once;
copies and repeated materialization consume new cumulative admissions. Glyph
outline commands and temporary trial metadata are not an unbounded exempt phase.
Variable UTF-8/byte payload capacities use checked byte admission and fallible
growth. Release does not refund cumulative record/visit/byte claims. No arbitrary first-million-slot
allocation replaces incremental admission of actually needed metadata.

Font processing uses the same root ledger: census scalar inputs, discovery/config
entry visits, decoded face/name inputs, candidate/unit/coverage visits, and actual
examined canonical-path deduplication records. Fontconfig parser input is admitted
as actual scalar input, not parser-capacity prices.
Font bytes admit each source read/hash pass before access; a single read that
hashes the same buffer is one byte stream, rereads are additional charges.
The 512 MiB byte bound includes font/source/scratch admissions and remains
independent of PNG backing-store viability, processing visits, and record counts.

All counters bound named admitted processing/storage, NOT allocator overhead,
RSS, wall time, CPU instructions, or third-party allocations. Large glyph
outlines and variable-length payloads still require checked byte arithmetic and
fallible admission. Do not promise fully controlled third-party OOM handling.
Third-party parsers/shapers receive already bounded inputs and configured limits
where supported; internally infallible allocations remain an explicit residual
risk, not a reason to bypass first-party admission.

Before any retained first-party allocation, check `u128` and `usize`, admit its
known records/bytes, then reserve fallibly. Exact known counts reserve their
needed capacity; unknown payloads may grow incrementally with checked admission
and fallible bounded growth BEFORE retaining the next chunk. Capacity growth
must fit the admitted class and checked byte extent. No exact whole-world glyph
or font payload census is required before parsing/shaping can discover lengths.
No million-entry discovery/cache allocation is mandatory. Resource/reservation
failure aborts, never retries a smaller representation or omits content.

Root lifecycle order is:

1. Validate call-local PNG options first when invoked through a PNG terminal;
   normalize numeric/root constraints and resolve theme/scale ownership.
2. Validate semantic/exact policies, domains and shared descriptors; count/admit
   topology and text requirements before retaining their bounded preparation.
3. Perform bounded font census/discovery/resolution and intrinsic summaries.
4. Choose root width once; execute root-coordinated constrained rounds, with
   admission before every retained local plan, state, cache, and feedback growth.
5. Converge or fail; validate semantic completeness, hard fit, finite positive
   dimensions, and checked canonical dimension conversion. Do not apply
   hypothetical PNG viability or PNG-derived byte gates to the shared plan.
6. Freeze root plan, glyph IDs, origins/positions, advances, bounds, face identities,
   and immutable same-byte face sources needed for later outline extraction.
   PF3, implemented in `5c0acdf`, captures each producer once with
   checked incremental admission and fallible bounded growth during planning,
   before final canonical storage reservation. Never rerun a producer to size it.
   Count from the captured immutable commands; capture is not final backend draw.
   Count canonical serialization tokens; reserve final plan/scene storage,
   canonical SVG buffer, and serializer scratch before any final validation
   backend begins. Eager extraction of every glyph path is not required.
7. Validate that frozen plan through one balanced Null lifecycle; for a terminal
   operation draw the SAME plan through one balanced concrete lifecycle.
8. Publish requested format with its call-local limits/reservations, without
   changing the scene, rerunning layout, or switching rendering routes.

Count-only operations create no Null/concrete backend and emit no artifact.
All first-party canonical drawing storage is reserved before Null begins. Final draw
does not measure, wrap, shape, choose ticks/edges, pack, resize, change domains,
or account new planning work. Arithmetic/resource/font errors abort at their
encountered stage. No partial scene or file is published on failure.

Derived outlined-SVG/PNG publication may extract paths later ONLY as deterministic
format conversion from the frozen glyph IDs and immutable retained face bytes,
using frozen origins and transforms. It may not reread font files, select faces,
reshape, wrap, change bounds, or perform new layout. Already extracted paths may
be reused. Derived conversion buffers and path-command payloads may be admitted
incrementally with checked byte/element counts and fallible growth under the
applicable format gates; they are not mandatory pre-Null canonical buffers or
new layout-planning trials. If extracted during planning, those path records
remain subject to the planning accounting above. Conversion failure remains
format-local and cannot mutate the canonical scene.

## Canonical Output And Viability

Canonical dimension is `ceil_div(logical_quanta,1024)`, positive and `u32`.
Exact-size roots reproduce both caller dimensions; exact-width roots reproduce
the caller width and derive height. Conversion never clamps or reruns policy. Root SVG
width/height/viewBox use these integral dimensions; internal geometry remains
quantized, not rounded to whole pixels.

For signed quantum `q`, format magnitude whole part `/1024` and remainder
`%1024`. A nonzero remainder becomes `remainder*9_765_625`, zero-padded to ten
decimal digits then trailing zeros trimmed. Use optional minus only for nonzero,
no exponent or locale separators. Stroke/path operands use the same spelling.
A bottom-up adapter, if needed, uses checked `root_height_quanta-y` exactly once;
Null and canonical SVG consume identical transformed geometry.

The nonrendering byte counter and serializer share token-length/emission rules,
including escaped original text and exact coordinates. Count is not a Null
lifecycle. Concrete canonical bytes must equal the count. Outlined SVG is a
derived publication of retained glyph geometry, not the canonical text byte count.

Preserve-text and outlined SVG in all three sizing modes have no inherited PNG
dimension, pixel, or 64 MiB byte gate. Shared processing-visits and record caps,
representability, checked byte counts, and fallible format allocation still
apply. No automatic-only canonical preserve-text byte limit remains.

Only a PNG terminal validates source geometry/axes/area, actual parser input
bytes, and rounded-positive target geometry/axes/area against the PNG limits
above, in every sizing mode. For default 300 DPI and source dimension `d`,
target is checked `((25*d)+4)/8`. Custom `png_with_options` validates options
before conversion/allocation and checks its actual requested target; preview
checks its own raster target. Because PNG consumes retained glyph outlines,
count/check the ACTUAL outlined SVG or equivalent parser payload against
`67_108_864` bytes, not merely the smaller preserve-text SVG. Reject known
dimension/product or payload limits before the corresponding allocation/parser
work; do not rasterize to discover a known limit violation.

A valid SVG can fail PNG dimension, pixel, parser-input, or reservation checks
in any sizing mode. That failure is format-local: it neither invalidates SVG nor
mutates the figure, canonical dimensions, or retained immutable font/path source.
It never chooses another width, coarsens labels, shrinks fonts, or retries another
layout. Chained publication preserves the canonical scene and same-byte path
extraction source. `Output::width/height` always report the frozen canonical
scene, never raster target dimensions.

## Target Public Surface

Phase T6 atomically installs automatic no-sizing behavior, Plot/Canvas
consuming `width(self, width: u32) -> Self` for exact width with automatic height,
retained exact `size(self, width: u32, height: u32) -> Self`, and final
`Output::width()` / `height()`, removing pre-render Plot dimension getters.
The target `width` builder replaces the current getter at T6, not an overload or
compatibility alias. Each sizing call completely replaces the one stored request;
`width` after `size` clears height, and zero dimensions fail preflight without a
setter panic. Until then the current exact API remains the sole public contract.
There is no public maximum-width builder. Broad public constraints are deferred
for T7 closure; internal intervals remain private.

At T6 only, `TitleAlignment::{Start,Center,End}` and `CompositionTheme` expose
concrete `root_title_alignment`, optional `root_subtitle_alignment`, and concrete
independent `root_caption_alignment`, with built-in defaults `Center`, `None`,
and `Start`. Caption uses the same `TitleAlignment`, not a new caption type.
The narrow public setters are `Plot::title_alignment`, `Plot::subtitle_alignment`,
`Canvas::panel_title_alignment`, and `Canvas::panel_subtitle_alignment`; each
owns an optional override input. Public `panel` naming denotes composed Plot
semantics; internal layout trees may still call their nodes children.

Title precedence is explicit Plot override > Canvas panel default > resolved Plot
theme title. Subtitle precedence is explicit Plot subtitle override > Canvas
panel subtitle default > resolved Plot theme optional subtitle > final effective
Plot title. Standalone Plots skip Canvas panel defaults. `None` means no
instruction, not forced title-following that bypasses another subtitle setting.
Theme/alignment call order is independent, and repeated setters replace only
their own input. Complete theme replacement is otherwise unchanged; there is no
general merging. Resolve optional inputs to concrete alignments before measuring
and retain them in plans for unchanged consumption by arrangement and drawing.
Canvas's own title/subtitle use its theme's root fields, with absent subtitle
falling back to its own title alignment, separately from Canvas panel settings.

`TickInterval` is an opaque immutable crate-root `Clone + Copy + Debug + PartialEq`
value. `additive(step:f64)->Result<Self>` requires finite positive step;
`log10_decades(n:u32)->Result<Self>` requires nonzero n. Plot
`x_tick_interval(self,interval)->Self` and `y_tick_interval` require NumericX/Y
respectively. Repeated setters replace prior interval; scale-kind mismatch is
semantic error; categorical axes do not admit intervals. Exact ticks override
regardless of setter order. An interval is a local first-tried preference, not
exact positions; exact `AxisTick` remains the strict control.

No target count, density control, tick callback, public constraint/measurement
operation, `SizeRequest`, plan, layout trait, quality scorer, or plugin is added.
Repository callers migrate atomically rather than retaining compatibility aliases.

## Validation And Repair Evidence

Implementation repair phases assign exact datasets, fonts, roots, expected local
decisions, counters, and snapshots before claiming acceptance. This specification
does not assert that those implementations or fixtures already exist. Measurable
workload acceptance includes the string `"A ".repeat(144)`, an approximately
200-word scientific caption in a SINGLE paragraph, the three-paragraph caption
baseline on ordinary automatic and exact-width/automatic-height Plot/Canvas output, and
multilingual paragraphs with Arabic joining/bidi, combining marks, CJK breaks,
and emoji controls under compatible pinned fonts. They must complete within the
stated logical budgets at adequate assigned widths/heights, with exact-size cases
given sufficient height for the selected wrapped lines. Record proposal visits,
exact trials, shaped input/output counts, and retained metadata for each fixture;
the repeated-word narrow-line regression must not reshape all remaining suffixes
per line. Also cover direct retention of a fitting unwrapped paragraph. There is
no all-width/all-boundary-pair enumeration. This is
not a promise that arbitrary text fits a small exact root or every font stack.

Evidence must cover numeric rounding/rational metrics, signed and `usize` limits,
canonical coordinate bytes, one-plan child ownership, proportional shrink and
capped surplus/remainder ties, span ordering/rechecks, sparse/trailing tracks,
all family mappings, shared axes/domains/guides, and original semantic identity.
Text evidence includes contextual nonadditivity, proposal/retreat/next-endpoint
extension, stopping at first nonfit despite a distant nonmonotonic fit, mandatory
empty paragraphs, retained paragraph bidi across soft wraps, Common/Inherited
resolution, default-ignorables/controls, missing visible glyphs, and viewer-text
versus exact outlined/PNG geometry. Run Unicode conformance against pinned data.
Unbreakable-label fixtures must return a typed fit miss without emergency
splitting, fit-driven rotation, shrinking, or omission; no new text-fit controls
are implied by the evidence requirements.

Tick evidence includes more than five automatic ticks at wide spans, exact list
0/1/10_000/10_001 boundaries, unlabeled exact ticks, interval first/coarsen/midpoint
ordering, `2^53` indices, Forest expressions, and stale-gutter provisional misses.
Guide/overlay evidence includes fixed edges, bounded edge/compact-flow trials,
entry-order flow, and rejection of guide-only fit when the affected plot fails.
Prove another supported arrangement succeeds before reporting final fit failure,
explicit placements remain fixed, nested owners share attempt/work budgets,
and deterministic packing/shelves preserve content. There is no exhaustive
sibling-product search or font/content change to force exact fit.

Font evidence covers canonical name occurrences, first-use stacks/weights,
SHA/face ordering and unchanged primary selection, first candidate covering a
base-plus-mark unit, scalar fallback when no candidate covers the unit, leading
marks, and nonpainting controls excluded from coverage. Include a stack where
the first scalar face lacks a mark but a later face covers the whole unit;
prove the later face wins for that unit while standalone generated ASCII and
metric probes retain scalar-first selection. Preserve unit spans/selections in
census and profile provenance tests: equal character unions with different unit
groupings must not erase distinct requirements. Prove no general emoji/ZWJ or
whole-word fallback is introduced. Also cover same-byte use, discovery permutation
invariance, bounded fontconfig malformed/partial directive and fallback behavior,
short/changing files, no simultaneous all-file handles, and coherent Unicode
dependency provenance. Reservation refusal is injected before preparation growth,
local/state growth, final freeze/Null, and format conversion.

Resource evidence exercises limit-minus-one/equal/plus-one for every independent
limit, cache hits/calls, unknown-payload growth, and actual processing visits rather
than claiming an RSS/CPU guarantee. Convergence covers rounds two/eight, changed
eight, nonadjacent repetition, terminal misses, and feedback-resolvable misses.
Sizing evidence covers all three modes on Plot and Canvas, exact width with
derived height, both sizing-call orders and repeated calls, and zero dimensions
rejecting at preflight without setter panic. Alignment evidence covers all four
presets and independent title/subtitle overrides, each precedence level, absent
subtitle instructions falling through to the final effective title, theme/alignment
call-order independence, and repeated-setter replacement. Also cover caption
independence, separate Canvas root versus panel settings, wrapped line and block
placement across each physical post-inset local root content span (not data-panel
span), and bidi without swapping Start/End. Embedded Plot evidence must preserve
titles, subtitles, and local captions exactly once with allocated space, keep the
Canvas caption separate, and fail rather than omit content when space is insufficient.

Output evidence covers `10_485` versus `10_486` source-axis default-target boundary,
`3_200x3_200` versus `3_200x3_201` target-area boundary, and actual outlined PNG
parser bytes. In all three sizing modes, cover valid SVG beyond PNG dimension,
pixel, and 64 MiB parser-input limits, including canonical preserve-text SVG over
64 MiB where shared admission permits it. PNG must reject safely without figure
mutation or loss of the retained path source; later SVG publication stays valid.
No hypothetical default-PNG gate may reject automatic or exact-width SVG.

Snapshots distinguish current explicit-size preservation from intentional target
automatic changes. Record chosen natural width, assigned tracks, local source
ordinals, feedback state, font provenance, and workload counters, not competing
global frontiers. Increased output size alone is not acceptance. No tests or
verification are implied by this documentation-only revision.
