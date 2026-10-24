# Theme System

## Complete Appearance And Construction

A Theme is one complete immutable appearance value. It owns visual defaults,
not scientific data/statistics, semantic content, output routing, numeric spelling,
tick generation or guide composition. Those concerns retain their owners in
[rendering](02_RENDERING_CONTRACT.md), [property ownership](04_VISUAL_PROPERTY_OWNERSHIP.md),
[layout](05_LAYOUT_SYSTEM.md), [numbers](06_NUMERIC_PRESENTATION.md) and
[guides](08_GUIDE_SYSTEM.md).

`Theme::new` consumes eight complete components in order: surfaces, typography,
axes, legend, composition, marks, annotations and color_scales. Corresponding
readonly getters borrow those components. `publication()`, `nature()`, `science()`
and `chi()` return ordinary complete validated values, without retained selector
identity. No public resolved-theme snapshot or backend style record is exposed.

`Theme::customize()` clones a complete working copy; `ThemeBuilder::new(Theme)`
consumes one. Builder `get_component()` readers borrow working components,
`component(value)` replaces one complete component and `build()` revalidates the
complete result. Theme's read-only `component()` readers retain their names.
`font_stack` fallibly replaces stacks across all eighteen text roles once
during construction; later role/component edits have ordinary latest-edit meaning.
The builder is construction-only and is never stored as a runtime override.

| Component | Complete responsibility | Source |
| --- | --- | --- |
| SurfaceTheme | Canvas root and Plot panel RGBA surfaces | `src/theme/resolved.rs` |
| TypographyTheme | Eighteen complete text roles | `src/theme/typography.rs` |
| AxisTheme | Ordinary/cell frames, ticks, grid and local gaps | `src/theme/axis.rs` |
| LegendTheme | One guide's frame, handles, gradient preferences and local gaps | `src/theme/guide.rs` |
| CompositionTheme | Root alignment/padding, prose relationships, Grid/header/strip chrome | `src/theme/composition.rs` |
| MarkTheme | Point/fill/line/stroke/cell/extents/neutral/accent roles | `src/theme/mark.rs` |
| AnnotationTheme | Significance, cell/value/detail, panel text and shelf geometry; contrast backdrop | `src/theme/annotation.rs` |
| ColorScales | Default qualitative, sequential and diverging mappings | `src/theme/scale_values.rs` |

SurfaceTheme consumes `canvas_root(value)` and `plot_panel(value)` edits, with
paired `get_canvas_root()` and `get_plot_panel()` readers. Both edits retain
complete surface construction and admit transparent paints.

### Uniform Value Contract

Component and nested group fields are private. Fully typed `new` constructors
consume every retained field; no string-keyed map or partially valid record is
admitted. Values implement Clone/Debug/PartialEq, and Eq where normalized storage
permits it. Complete Theme equality excludes operation context, fonts discovered
at render time and caches.

Each editable field `property` has one consuming `property(value)` edit and its
paired `get_property()` reader, including macro-generated mark and annotation
groups. Read-only and write-only properties keep bare names. Theme's read-only
components are distinct from ThemeBuilder's editable construction components;
there are no retained `with_*` configuration aliases.

Inspection returns a Copy value, a borrowed owned value, `Option<T>` for optional
Copy values or `Option<&T>` for optional owned values. Collections expose borrowed
slices/iterators, not mutable access. Consuming field edits replace only that
field, validate the resulting complete value and return Result. Cross-field
failures reject immediately. ThemeBuilder's
complete component replacements return Self; its font-stack edit and build return
Result. Closed-choice factory values use their documented factories and replace
whole choices, rather than exporting unchecked variants.

Construction performs no font discovery, data-domain resolution or rendering.
It validates all data-independent invariants. Preparation checks used fonts,
capacity, raw keys, final domains, Shared compatibility, fit and representability
after owner selection. No delayed error strings or sentinel defaults are stored.

### Units And Exact Local Relationships

Font sizes, radii and tick lengths are positive integral CSS pixels. Present
stroke widths are finite positive f32 CSS pixels; zero is not absence. Optional
frames/outlines/grid/ticks use structural None. Alpha is an inclusive u8 byte.
Normalize signed zero in nonnegative floating quantities/domains/offsets; reject
nonfinite inputs.

`RelativeSpacing::new(f64)` requires finite nonnegative input and retains its exact
binary64 rational. `ratio(u32,u32)` requires positive denominator, permits zero
numerator and retains the reduced mathematical ratio; zero is 0/1. Both use one
representation. `as_f64()` is nearest-even inspection, not a spatial or identity
operand. Ratios may exceed one unless a specific field narrows its range.

Resolve a relationship by exact rational multiplication with its declared local
metric. S operands (positions/preferences) round nearest-even once; H operands
(hard occupancy/reserves) round upward once. Complete hard sums round after exact
aggregation, not by adding individually rounded gaps. Lower/upper ink edges round
outward. All arithmetic and spatial conversions are checked. No root-size scale,
epsilon rescue or imposed new public minimum may hide nonzero geometry collapse.
`src/theme/spacing.rs` owns this representation and resolution.

Built-in local relationships preserve exact represented source lengths divided
exactly by their original local metrics, not a rounded binary64 division or a
nearby decimal ratio. For example exact ratio 6/14 against 14px yields 6144 quanta;
`new(6.0/14.0)` is a different represented value. This rule is independent of named
nonspatial scientific fractions below, whose prescribed binary64 arithmetic is
retained before canonicalizing the resulting geometry.

## Ownership And Resolution

Standalone omission selects Publication. Canvas owns one complete Publication
root, replaceable with `Canvas::theme`. A composed Plot without `Plot::theme`
inherits that root; explicit Plot Theme completely replaces it. Facets have no
additional scope. No process-global, thread-local, environment-selected,
backend-selected or sticky appearance exists.

Canvas surfaces, root prose, shared headers, row titles, TextPanel and Shared-guide
chrome use Canvas. Plot marks, axes, annotations, panel surfaces and prose use
the selected Plot appearance. An embedded panel never repaints the Canvas root.
Resolve once before validation/measurement and retain the same complete value
through scientific preparation, fonts, collision checks and frozen operands.

Exact styles and Plot scales apply only to their owned properties. Root heading
instructions resolve separately as specified in property ownership. Shared
`TextAlignment::{Start, Center, End}` is owned by the private text module and used
by Plot/Canvas heading instructions and CompositionTheme. Composition alignment
controls block placement and each line across the physical post-inset
content span, not the data-panel span or bidi-logical edges. All built-ins use
Center title, None subtitle (follow the effective title) and independent Start
caption. Root rotation remains fixed.

## Colors, Paints And Typography

`OpaqueColor` is Copy/Eq RGB bytes; `Color` is Copy/Eq RGBA bytes. `rgb` is
infallible, with Color alpha 255; `rgba` supplies all four channels. Color's const
`alpha(byte)` replaces alpha and its paired const `get_alpha()` inspects it.
Read-only RGB queries remain `red()`, `green()` and `blue()`. Strict hex accepts
exactly ASCII `#RRGGBB` for OpaqueColor and `#RRGGBB`/`#RRGGBBAA` for Color, with
case-insensitive hex digits, no trimming,
shorthand, CSS expression or floating channel. OpaqueColor-to-Color sets 255;
there is no conversion silently dropping Color alpha into a data scale.

Mapped palettes/maps/gradients use OpaqueColor. Role or exact paints supply one
alpha, with replacement precedence in property ownership. Surfaces may be
transparent; no hardcoded white root is inserted. SVG and PNG preserve surface
alpha and composed document order.

### Typed Font Preferences

`FontFamily::named` retains one nonempty already-trimmed control-free literal
identity, including punctuation, interior spaces and Unicode bytes. A CSS-looking
name remains one quoted literal, never an expression or implicit generic.
`FontFamily::generic` accepts only Serif/SansSerif/Monospace.
`FontStack::new` requires nonempty ordered entries and retains duplicates. No raw
CSS input parser or global default-font sentinel is public.

Every built-in role uses the typed order Noto Sans, Noto Sans CJK SC, DejaVu Sans,
Liberation Sans, Arial, generic SansSerif. Callers obtain a built-in stack through
its complete text role. Matching/candidate order, same-byte retention, used-role
census, base-plus-mark preference and installed-host freshness are governed by
the layout font contract, not Theme construction. Unused roles do not require
font availability.

`TextTheme` owns stack, Color, positive finite f64 CSS font_size, closed
Normal/Semibold/Bold weight and finite positive f64 line_height. Leading is an
exact represented multiplier of the unrounded CSS font size, with one upward hard occupancy
rounding. It is not an absolute length or spacing setter. Every text role is
complete and independently editable.

Fractional sizes remain binary64 values through selected appearance, glyph scaling,
font-relative spacing and SVG/outline storage. Design-unit metrics use exact signed
rationals with wide-integer support for extreme dyadic denominators; final coordinate
rounding, not an intermediate integer font size, determines spatial representability.
SignificanceMarkStyle uses the same f64 size while retaining its independent pixel gap.

TextTheme implements complete Eq over its validated fields: positive finite size/leading
admits reflexive equality without NaN or signed-zero ambiguity. TextPanel's authored
Text may supply one complete exact TextTheme independently for title and body.
This local appearance replaces its selected Canvas role before the font census;
omission uses that complete role. Measurement, 66ch preferences and frozen plans
retain the selected stack, paint, size, weight, leading and exact provenance.
PanelTextAnnotation applies the same complete local appearance precedence over
the Plot's panel-text role; padding resolves against its selected font size.
Plot/Canvas title, subtitle and caption likewise resolve local exact appearance
over their complete root-text roles. Root title/subtitle gaps use the maximum of
the selected title/subtitle sizes, and caption clearance uses selected caption
size. Root padding remains tied to the root composition-body role.
X/y axis titles resolve independently over axis_title; their selected sizes own
the corresponding title clearances. Continuous guide titles resolve local Text
appearance over the consuming guide-title role, including Shared Canvas ownership.
Each measured section resolves title_gap against that title's selected size;
Frame, key and label spacing use the maximum actual selected guide-label size;
automatic entries and continuous numbers retain their guide-label role size.
Canvas headers and row titles independently replace composition_header with a
complete local Text style. Each named header padding uses that selected size;
shared band extents take the maximum complete text-plus-padding requirement.
The minimum for declared empty Canvas tracks still uses the owner's Theme metric.
Heatmap null-row Text may replace the selected missing-fill contrast appearance;
its chosen size supplies both null-row padding products. Ordinary value labels
continue to use their own final-cell contrast selection.

TypographyTheme and each nested text-role group use consuming `property(value)`
edits and paired borrowed `get_property()` readers. For example,
`theme.typography().get_root().get_title()` borrows the complete root title role;
editing that role and replacing its containing groups retains ordinary latest-edit
precedence without changing other text roles.

`TextTheme`'s consuming edits use `stack(value)`, `color(value)`, `font_size(value)`,
`weight(value)` and `line_height(value)`. Paired readers are `get_stack()`,
`get_color()`, `get_font_size()`, `get_weight()` and `get_line_height()`, preserving
the borrowed font stack and copy results. Each edit reconstructs and validates
the complete role, including retained size and leading constraints.

| Typography group | Retained complete roles |
| --- | --- |
| RootTextRoles | title, subtitle, caption |
| AxisTextRoles | axis_title, tick_label |
| GuideTextRoles | legend_title, legend_label |
| CompositionTextRoles | facet_header, composition_header, text_panel_title, body |
| AnnotationTextRoles | significance_label, estimate_detail, waterfall_total, point_layer_label, panel_text, contrast |
| ContrastTextRoles | light, dark |

Contrast light/dark colors must be opaque; their stacks, sizes, weights and
leading may differ. Select the complete role before fonts and measurement.

### Built-In Typography

Let F be preset foreground and T/S/C/A its title/subtitle/caption/axis-title sizes.
All built-in weights are Normal. Root leading is the exact binary64 evaluation
`(f64::from(size)+6.0)/f64::from(size)`; all other roles use represented 1.2.

| Preset | F RGB | T | S | C | A | Axis width f32 | Bracket width f32 | Grid RGB |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Publication | 0,0,0 | 20 | 15 | 15 | 17 | 0.8 | 1.0 | 220,220,220 |
| Nature | 40,40,40 | 19 | 14 | 14 | 16 | 0.8 | 0.8 | 230,230,230 |
| Science | 0,0,0 | 21 | 16 | 15 | 17 | 1.0 | 1.0 | 200,200,200 |
| CHI | 60,60,60 | 19 | 15 | 15 | 17 | 0.8 | 0.8 | 220,220,220 |

| Role | Built-in color / CSS size |
| --- | --- |
| title / subtitle / caption | F/T; 80,80,80/S; 80,80,80/C, all opaque |
| axis_title / tick_label | opaque F/A; F/16 |
| legend_title / legend_label | opaque 60,60,60/12; 40,40,40/11 |
| significance_label / estimate_detail / waterfall_total | opaque F/16; F/14; F/14 |
| point_layer_label / panel_text | opaque F/10; F/14 |
| contrast.light / contrast.dark | opaque white/11; black/11 |
| facet_header / composition_header / text_panel_title / body | opaque F/S; F/T; F/T; F/A |

Publication title leading 1.3 has bits `0x3ff4cccccccccccd`; its exact product with
20px rounds hard occupancy to 26625 quanta, not rational 26px/26624. Preserve raw
leading and actual selected glyph occupancy rather than printed decimal estimates.

## Compound Paint And Mark Components

Compound values are typed appearance, not writable backend records. StrokePattern
is closed Solid/ShortDash/MediumDash/LongDash with dash grammar solid/`3 2`/`4 3`/
`5 3`. Exact LinePattern retains its narrower Solid/Dashed (LongDash) grammar.

| Value | Retained fields / construction |
| --- | --- |
| StrokeTheme | Color, positive width, StrokePattern |
| SolidStrokeTheme / OutlineTheme | Color, positive width |
| PointOutlineTheme | Color, strictly positive RelativeSpacing width_ratio against final point radius |
| DataOutlineTheme | `fixed(OutlineTheme)` or `mapped(positive width, alpha)`; private choice |
| DataFillTheme | fill_alpha, optional DataOutlineTheme |
| FixedFillTheme | Color, optional OutlineTheme |
| PointTheme | fill_alpha, PointShape, optional PointOutlineTheme |
| SizedPointTheme | complete PointTheme appearance, positive radius |
| SupplementalPointTheme | positive radius, fill_alpha; plain Circle/no outline |
| DataLineTheme | positive width, alpha, StrokePattern |

DataLineTheme consuming edits are `width(value)`, `alpha(value)` and
`pattern(value)`, with paired `get_width()`, `get_alpha()` and `get_pattern()`
readers. Every edit reconstructs and validates the complete width/alpha/pattern
value, including retained sibling constraints.

StrokeTheme, SolidStrokeTheme, OutlineTheme, PointOutlineTheme, DataFillTheme,
FixedFillTheme and PointTheme follow the same property naming: consuming
`property(value)` edits pair with `get_property()` readers. Borrowed optional
outlines remain optional references. DataOutlineTheme's closed-choice factories
and read-only `fixed_stroke()`, `mapped_width()` and `mapped_alpha()` keep their
names and structural choice semantics.

Mapped outlines receive mapped RGB and their own alpha; fixed outlines retain
complete Color. Point-outline ratios resolve against the final selected point
radius, with outward hard ink independent of nearest-even width/center operands.

| MarkTheme group | Complete nested fields |
| --- | --- |
| accents | Independent nonempty AccentPalette |
| points: PointRoles | primary, raw_overlay, mandatory_strip, radii, supplemental |
| points.radii: PointRadii | scatter, path, trajectory, regression, qq, marginal, matrix, interval_estimate, dumbbell, interaction, beeswarm, box_flier |
| fills: FillRoles | bodies, bands, densities |
| fills.bodies: BodyFillRoles | bar_body, box_body, violin_body, inner_summary, histogram_bin, waterfall_bar |
| fills.bands: BandFillRoles | line_band, regression_band |
| fills.densities: DensityFillRoles | density_area, marginal_density, matrix_density |
| data_lines: DataLineRoles | series_line, path_line, density_line, interaction_interval, interaction_connector |
| neutral_paints: NeutralPaintRoles | Independent opaque path and box_flier RGB, with no second alpha |
| strokes: StrokeRoles | references, connectors, summaries, curves |
| strokes.references: ReferenceStrokeRoles | bar_zero, waterfall_reference, beeswarm_reference, point_interval_reference, estimate_reference, qq_reference |
| strokes.connectors: ConnectorStrokeRoles | waterfall_connector, dumbbell_connector |
| strokes.summaries: SummaryStrokeRoles | uncertainty, summary, estimate_interval, median |
| strokes.curves: CurveStrokeRoles | histogram_curve, histogram_rug, regression_fit |
| cells: CellMarkRoles | fill_alpha, optional cell_outline, missing_fill, optional missing_row_outline, optional missing_key_outline, missing_cross, missing_cross_inset |
| extents: MarkExtentTheme | categories, waterfall_body_category_breadth, point_interval_cap_half_height, sampled_rug_height, marginal_density_fraction, matrix_density_fraction |
| extents.categories: CategoryMarkGeometry | vertical/horizontal body_breadth, vertical/horizontal group_span, uncertainty_cap_ratio |

All radii/cap half-height are positive integral CSS pixels. Category breadths/group
spans, Waterfall breadth and density fractions are in (0,1]; cap ratio in (0,1/2];
sampled rug height is nonnegative; missing-cross inset is in [0,1/2).
Containment/representability of actual strokes, caps, spans and connectors remains
a preflight requirement, not a construction-time invented panel floor.

Category/rug scientific fractions retain named binary64 operation order using
nearest-even inspection of the normalized relationship, then canonicalize resulting
positions. Defaults are vertical/horizontal breadth 0.6/0.55, group span 0.76/0.74,
cap ratio 0.2, sampled rug 0.035 of prepared maximum, local marginal/matrix density
fractions 0.90/0.85 and missing-cross inset 0.16. Waterfall independently uses 0.6;
derive `free=1-b`, `low=free/2`, `high=1-low`, edge room=low, connector room=free,
without FMA or replacing b by high-low. Positive ink with zero available room
misses; zero ink does not evaluate 0/0. All consuming edges/intrinsics/connectors
use that same prepared breadth.

Built-in primary points use alpha 209/Circle/white outline ratio `new(0.1)`.
Independent raw/mandatory-strip roles both start at radius 2 with that appearance;
supplemental is radius 4/alpha 204. Radii in the table's order are
3,3,4,3,3,3,2,4,4,4,3,2. Body alpha is 255 for bars/boxes/Waterfall, 178 for violins,
209 for Histogram; bands 46/64 and density areas 56/115/102 in their listed order.
Inner summary is opaque white. Present neutral/fixed strokes and cell/missing
paints are constructed explicitly in `src/theme/resolved.rs`, not inferred late
from another equal-valued component. Point/fill/stroke data-alpha attachments and
their actual component census are specified in property ownership.

## Axis, Guide And Composition Components

FrameSides, FrameTheme, TickTheme, AxisSpacing and AxisTheme use consuming
`property(value)` edits paired with `get_property()` readers. LegendSpacing,
GradientLengths, LegendFrameTheme and LegendTheme follow the same rule, including
`LegendTheme::shared_gap(value)` and `get_shared_gap()`. CompositionTheme and its
root/header/facet/text-panel spacing groups also use paired property names.
Inspection retains optional borrowed chrome, borrowed exact relationships and
copy alignment/length/side values. Renaming changes neither complete validation
nor which selected local text metric resolves a relationship.

FrameSides requires a nonempty subset of top/right/bottom/left. No frame is None.
FrameTheme owns those sides and StrokeTheme; TickTheme owns stroke and positive
length. AxisTheme owns independent optional frame, cell_frame, ticks, grid and
complete AxisSpacing. Built-ins use four-side ordinary frames, bottom/left cell
frames, length 5 ticks, preset axis width and preset RGB grid width 0.5/MediumDash.
Custom cell sides are not masked by Matrix. Labels/exact ticks/titles remain
semantic content even when chrome is absent.

Let E(role) be its positive font-size em. Each spacing below uses S/H unless
specified otherwise; stroke/ink occupancy is added separately.

| AxisSpacing field | Local metric | Built-in exact relationship |
| --- | --- | --- |
| x_tick_label_gap / y_tick_label_gap | tick_label em | 8/16; 6/16 |
| x_title_tick_label_gap / x_title_without_tick_labels_gap | axis_title em | 8/A; 16/A |
| y_title_tick_label_gap / y_title_edge_padding | axis_title em | 16/A; 12/A |

Empty exact labels retain positions with no text gap/ink. Absent ticks contribute
no length/stroke, not absent requested label content.

LegendTheme owns LegendSpacing, GradientLengths and optional LegendFrameTheme.
The frame owns fill Color, optional OutlineTheme and nonnegative relative radius.
Gradient minimum/preferred and key_width are positive; preferred>=minimum.
Other gaps/padding/radius are nonnegative. LegendTheme `shared_gap` access/edit
delegates to spacing.shared_gap; it is the sole Shared-gap owner, valid with any
Canvas legend policy. No Canvas override or policy compatibility check exists.

Let P(p,m) denote exact represented binary64 p divided exactly by integer m.

| LegendSpacing field | Metric | Built-in ratio |
| --- | --- | --- |
| padding / key_label_gap / outside_gap | legend_label em | each P(8.8,11) |
| key_width | legend_label em | 1 |
| shared_gap | Canvas legend_label em | P(17.6,11) |
| row_gap / column_gap | legend_label em | P(3.3,11); P(13.2,11) |
| title_gap | legend_title em | P(7.8,12) |
| glyph_edge_padding | legend_label em | represented 0.2 |
| GradientLengths minimum / preferred | legend_label em | 40/11 hard; 160/11 preference |
| LegendFrameTheme radius | legend_label em | 2/11 |

Built-in frame fill is white/alpha 204 with opaque (200,200,200) outline width 1.
Horizontal edge inset is padding; vertical inset is padding+glyph_edge_padding
once per outer edge, never per section. Exact represented 8.8 plus represented 0.2
times 11 has placement 11264 and hard 11265 quanta. Do not round its composite via
binary64 first. Section gap is exact title_gap+row_gap under their own metrics.
Actual key glyph ink can exceed key_width and remains a hard requirement.

Continuous-guide maximum is derived from assigned capacity minus actual title,
gaps, missing block and endpoint overhangs, never a Theme cap. Given complete
hard requirement R, preferred P and finite maximum M, miss when M<R; otherwise
normal chooses `max(R,min(P,M))`, compact R. Unbounded natural height uses
`max(P,R)` as preference. Missing presence, exact swatch/label ink and one row gap
come from the descriptor; no family-local fixed title/gradient/missing reserve
reconstructs the accepted common plan.

CompositionTheme owns concrete root_title_alignment, optional
root_subtitle_alignment, concrete root_caption_alignment, RootTitleSpacing,
RootPadding, panel_edge_clearance, TextPanelSpacing, facet_gap, canvas_grid_gap,
HeaderSpacing and FacetStripSpacing. All relationships are nonnegative.

| Field | Local metric / cardinality | Built-in exact ratio |
| --- | --- | --- |
| root_title_block.title_subtitle_gap | max(title,subtitle) em; adjacent present roles | 0 |
| root_title_block.title_block_to_content_clearance | max(title,subtitle) em, even when one content is absent | 14/max(T,S) |
| root_title_block.caption_to_content_clearance | caption em; caption presence | 12/C |
| root_padding.left/right/bottom/top | owner body em, once per named edge | 16/A;16/A;8/A;12/A |
| panel_edge_clearance | Plot tick_label em, once per exposed axis edge | 4/16 |
| text_panel.inset | body em, once at every edge | 16/A |
| text_panel.gap | max(text_panel_title,body) em | 12/max(T,A) |
| facet_gap | Plot facet_header em | 12/S |
| canvas_grid_gap | owner body em, also Matrix/Marginal | 12/A |
| headers.column_header_padding | composition_header em, one addition to band height | 16/T |
| headers.row_header_padding | composition_header em, one addition to band width | 20/T |
| headers.row_title_horizontal_padding | composition_header em, total-span addition/subtraction | 16/T |
| headers.row_title_vertical_padding | composition_header em, one present-row reserve | 8/T |
| facet_strip.horizontal_inset | facet_header em, total-span addition/subtraction | 16/S |
| facet_strip.content_clearance | facet_header em, actual text occupancy separately | 8/S |

RootPadding applies to standalone/facet/Canvas root and embedded Plot prose under
its own Theme. Embedded prose insets its horizontal text span; the scientific
body retains the full cell horizontal span. Present prose constrains vertical
body space; without prose its local span remains the whole cell.

Declared Canvas empty/trailing tracks have hard minimum1 quantum and preference
one **composition_header font-size em**, even without headers. Declared facet
tracks use the owning Plot's **facet_header font-size em**. Body, TextPanel title,
leading, glyph size and root dimensions are not substitutes. Occupied/spanning
requirements may raise minima/preferences. Both natural summaries and constrained
allocation retain that metric, topology and exact dependency identity. Grid gaps
still use body em independently. No universal root/panel floor is implied.

## Annotation Components

AnnotationTheme owns SignificanceGeometry, CellAnnotationGeometry,
EstimateAnnotationGeometry, point_label_gap, panel_text_padding,
ValueShelfSpacing waterfall_shelf and opaque planning_backdrop. Text belongs to
the complete typography roles; content/presence/targets remain policies.

| Field | Metric / cardinality | Built-in relationship |
| --- | --- | --- |
| significance.bracket_stroke | SolidStrokeTheme, positive width | F/preset bracket width |
| significance.label_gap/tick_height/data_gap/level_step/panel_padding/attached_extra_clearance | effective significance-label em | 8/16;7/16;6/16;26/16;2/16;2/16 |
| cells.highlight_outline | Independent positive OutlineTheme | F/2.5 |
| cells.value_horizontal_padding / value_vertical_padding | selected contrast em, total fit addition | 6/11;5/11 |
| cells.null_row_horizontal_padding / null_row_vertical_padding | selected contrast em, total fit addition | 12/11;4/11 |
| estimates.panel_inset / guide_clearance / numeric_label_gap | estimate_detail em; inset per edge, numeric gap center-relative | 6/14;6/14;7/14 |
| point_label_gap | point_layer_label em; nominal radius-relative offset | 2/10 |
| panel_text_padding | panel_text em, per edge | 8/14 |
| waterfall_shelf.padding/gap/row_gap | waterfall_total em; padding per edge | 4/14 each |
| planning_backdrop | OpaqueColor, calculation only | white |

Complete hard significance geometry stays exact through level occupancy/reserves
before one upward rounding; final positions have their own rounding. Half-stroke
plus one-quantum safety is a private containment invariant. Heatmap finite value
cells and fully-null merged rows have distinct source collections/padding:5 versus
4 vertical default pixels, 6 versus 12 total horizontal. All size/wrap/final fit
consumers use the matching collection and final selected contrast role.
Waterfall total names can supply independent complete Text styles. The retained
name-plus-number uses that style throughout census, measurement and drawing.
The shared shelf's padding/gap/row_gap use the maximum selected size of eligible
totals; each line block keeps its own font size and leading.
PointLayer legend Text can replace the guide-label appearance independently of
the layer's mark/point-annotation paint. Shared guide chrome resolves over Canvas;
an exact local label style remains complete. Shared label-relative metrics use
the maximum selected entry/number size without changing individual text styles.

## Data-Color Scales

QualitativePalette `fixed`/`extendable`, AccentPalette and CategoricalColorMap
require nonempty sequences/maps; repeated colors are valid. Fixed capacity must
cover the complete active raw domain before fonts, even with Hidden guides.
Extendable palettes preserve their exact seed prefix and deterministically extend
in OKLab/OKLCH by maximum minimum perceptual distance, fixed candidate/tie order
and deterministic sRGB gamut reduction, never cycling. Prefixes are independent
of final requested count. Accents are independently owned always-extendable
non-data colors; exact maps have no anonymous slots.

Map keys are nonempty already-trimmed control-free unique raw UTF-8 identities.
Preserve input inspection order and exact lookup. Do not trim, casefold,
Unicode-normalize or resolve display aliases to create keys. Every used key is
required; unused entries are valid. `CategoricalColorScale::from_palette/from_map`
selects one complete source, attached only to the fifteen CategoricalScale families
in `src/kind.rs`. Facets coordinate parent domain before slicing; Shared Canvas
coordinates raw fields in row-major order including visible marks with Hidden
entries. PerPlot keeps local domains.

ColorStop has finite normalized offset in[0,1] and OpaqueColor. SequentialGradient
requires at least two strictly ordered stops and exact 0/1 endpoints. DivergingGradient
requires at least three, exact 0/1 and one unique explicit neutral at 0.5, with stops
on both sides. Extra ordered stops are valid. No sorting, repair, epsilon merge,
transparent stop or arbitrary count cap occurs. `neutral()` derives from 0.5.
Interpolation is the fixed first-party OKLab model in `src/theme/color.rs`.

ColorScales owns qualitative, sequential and diverging values. Every built-in
sequential sequence is `0:#F7FBFF -> 1:#08306B`; diverging is
`0:(49,130,189), 0.5:(247,247,247), 1:(222,45,38)`. Built-in qualitative/independent
accent seeds are constructed in `src/theme/resolved.rs`, with stable exact prefixes.
Its consuming `qualitative(value)`, `sequential(value)` and `diverging(value)` edits
pair with borrowed `get_qualitative()`, `get_sequential()` and `get_diverging()`
readers. Read-only palette/stop/domain inspection retains its bare names.

ContinuousColorScale sequential/diverging factories own kind, optional matching
gradient, optional ColorDomain and finite diverging center. ColorDomain is finite
minimum<=maximum, allowing constants. An exact diverging domain contains center
inclusively. None gradient selects matching Theme default; None domain means
automatic from prepared finite active values. Wrong-kind gradient/center edits
reject immediately; reconstruct the complete kind to change it.

ContinuousColorScale edits use `domain(value)`, `center(value)`,
`sequential_gradient(value)` and `diverging_gradient(value)`, paired with
`get_domain()`, `get_center()`, `get_sequential_gradient()` and
`get_diverging_gradient()`. Optional domain/gradient readers remain borrowed;
the optional center is copied and the read-only `kind()` keeps its name.

Heatmap and Beeswarm share this complete attachment. Automatic sequential domain
is finite observed min/max; constant maps to midpoint. Automatic diverging domains
are symmetric about center when representable; otherwise reject with the explicit
domain remedy. Exact domains saturate outside values and need not contain every
observation. Constant-at-center paints neutral.

For an interior center fraction c in the final domain, semantic stop offsets
0/0.5/1 map exactly to 0/c/1; u<0.5 maps `(u*2)*c`, u>0.5 maps
`c+((u-0.5)*2)*(1-c)` in that binary64 order without FMA. Exact center returns
neutral. Endpoint centers use only the noncollapsed half; all extra branch stops
participate. Nonfinite/non-strict remapped offsets reject, not coalesce. Preserve
observed-color sampling and reject six-decimal serialized-stop collisions between
distinct colors. Required numeric labels remain endpoints plus interior center,
or one constant, independently of stop count.

Marks, guides and Heatmap contrast consume the same final-domain mapping. Shared
compatibility/domain installation is preparation before labels/contrast/fonts,
not feedback or a merge of already projected local stops. Numeric gradient stops
are opaque; they do not borrow point/cell alpha. Exact RGB source identity and
Shared compatibility are defined in the guide system.

## Automatic Contrast

General custom-theme construction does not enforce a universal contrast threshold.
For renderer-selected text over fills, select the complete light/dark candidate
using actual final fill RGBA, panel surface, Canvas surface and opaque planning
backdrop. The backdrop affects selection only and is never painted.

Use the checked-in 256-entry sRGB-to-linear binary64 table in
`src/theme/contrast.rs`, whose values correspond to nearest-even evaluation of
`(c/255)/12.92` for channels0..10, otherwise `((c/255+0.055)/1.055)^2.4` with its
defined binary64 operand order. No runtime platform pow determines the table.
Starting with opaque backdrop, composite Canvas, panel and fill back-to-front,
per channel in r/g/b order:

```text
one_minus_a = 1-a
source_term = source*a
destination_term = destination*one_minus_a
destination = source_term+destination_term
```

Each named operation rounds binary64 nearest-even without fused substitution.
Luminance is `0.2126*r+0.7152*g+0.0722*b` in that order; contrast is
`(max(l1,l2)+0.05)/(min(l1,l2)+0.05)`. Greater contrast wins; exact-bit tie selects
dark. Resolve before font census so only the selected complete role supplies
metrics. Transparent output cannot guarantee contrast on an unknown viewer
background.

## Extension Boundary

Theme construction does not admit CSS selectors, raw dash strings, arbitrary
property maps, runtime callbacks, dynamic theme extensions, process-global
defaults, font loading, blend modes, animation, one component per plot family or
new content-omission controls. Data alpha is a separate Plot-local value, not a
Theme field.
