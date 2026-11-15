# Complete Python Scientific And Numeric Interface

## Status And Authority

This plan specifies the complete remaining scientific/numeric Python interface.
All 25 constructors/alternatives, exact-family base data/facet/configuration
methods, complete Canvas configuration, Theme/Text and numeric/locator policies,
13 scientific/appearance/position values, all eight callback slots and all 75 alpha
entries, independent layers and significance results are implemented. This closes
the selected scientific call surface, not final distribution/content/cost or
test-budget acceptance. Current bounded evidence and remaining gates
are recorded in `91_WORKSPACE_AND_INTERFACES_PLAN.md`. Constructor fields, modes,
presence and flat geometry are defined once in `92_PYTHON_CONSTRUCTORS_PLAN.md`.
Do not replace that inventory with a smaller family subset.

Scientific behavior, native errors and output/font policy remain governed by
`02_RENDERING_CONTRACT.md`, `04_VISUAL_PROPERTY_OWNERSHIP.md`,
`05_LAYOUT_SYSTEM.md`, `06_NUMERIC_PRESENTATION.md` and `09_DATA_ADAPTERS.md`.
The actual sealed capability sets in `crates/lichtung-plot/src/kind.rs` determine
which Python exact-family classes expose each method. A universal Plot with
runtime-rejected setters is not equivalent typing.

Implement and commit coherent consumer closures, with independent review and
focused real-wheel verification. Do not clean old baseline tests until the full
functionality/distribution/content/cost plan is complete. Existing test-budget
excess is neither a capability restriction nor a budget-pass claim.

## Family Ownership And Binding

Retain the existing `Config<K>` pair: one exact native `Plot<K>` and its matching
`Core`, ndarray metadata or Polars metadata origin. Extend the private NativePlot
enum to all 25 exact kinds; do not retain erased scientific state, foreign
buffers, borrowed adapters, constructor arguments or Python configuration mirrors.
Thin public owners mutate/return themselves and copy their exact family.

Use each adapter's existing direct/source, wide/indexed, columns/discovery, grid
and geometry constructors. Native scalar-column calls may use core constructors;
rectangles, generated identities and grid expansion must retain adapter authority.
All adapters/projections/capsules are call-local and split back into native parts.
The ordinary input path must retain `native_slots=false`; passing internal owned
NumericValues/CategoryValues must not bypass row repetition or per-dimension checks.

For state-dependent edits, snapshot the native Arc plus actual source/key refs,
prepare outside the owner borrow, restore the matching adapter and split again.
Commit configuration, origin and new roots together only if the expected Arc is
still current. A stale candidate must not overwrite reentrant edits or retry
automatically. Python calls, mapper execution, foreign loan endings and retired
ref drops remain outside owner borrows.

Construct complete scientific forms by presence before lookup. Preserve mandatory
None rejection, paired endpoints, sample/estimate exclusivity and native nullable
estimate/status preflight. Forest reference and Waterfall base are required
literal numbers; Dumbbell requires both distinct valid literal endpoint labels
and complete RGBA colors. Trajectory axis choice follows admitted dtype/elements,
including typed missing/empty columns, never numeric-to-string coercion or
constructor-failure fallback.

For Histogram and Raincloud geometry, preserve the three independent alignment
domains each, raw dimension identity, summary order and original point-row alpha
units. Auxiliary `kde_data`, `rug_data`, `density_data` and `point_data` are explicit
independent contexts: omission/None means direct-only, not main-source fallback.
Only the main source is retained. Bind auxiliary selectors in their own contexts
even when the main source is absent. Sample-owned supplied KDE is a separate
method and does not implicitly change Histogram stat to Density.

## Owner Methods And Appearance Policies

Expose all applicable methods in native builder configuration, alpha, facets,
text-layout and text-clear modules, not only constructors. Forward to existing
native/adapter producers without reproducing scientific algorithms or validators.

| Python mapping | Native meaning and boundary |
| --- | --- |
| `category_order(order)`, Heatmap `x_category_order` / `y_category_order` | `"observed"`, `"value_ascending"`, `"value_descending"` or ordered literal raw strings. Heatmap admits observed/explicit, not value sort. Do not invent group/hue/series/facet-order setters. |
| `facet_row`, `facet_column`, `facet_grid`, `facet_wrap`, `facet_scales` | Existing row/cell/generated forms, including mixed grid row/column-cell and column-generated forms. Scales are fixed/free_x/free_y/free; replacement facet resets to Fixed, scales-only edits retain bound facet observations. Do not reread the source or guess scope by length. |
| `hue`, `group`, `series`, corresponding `_cells` / `_columns`; `source_rows` / `_cells` | Exact capabilities only; ctor-owned channels do not gain replacement setters. Preserve checked integer IDs and metadata authority, not pandas index or f64 identity. None is not a universal scientific reset. |
| `column_labels`, Heatmap `x_labels` / `y_labels` / `row_labels` | Literal display collections versus categorical row-label input remain distinct. Aliases do not change scientific identities or grid shape. |
| Histogram `bins`, `stat`, `composite`, `composite_precomputed_kde[_from_data]` | Complete Auto/Count/Width bins; count/probability/density stat; histogram_only/kde_and_rug plus independently bound supplied KDE. No argument selects Auto bins; count and width are mutually exclusive. Native sample/geometry restrictions remain. |
| Qq `reference_line`, Regression `band`, LinePath `style` | quartiles/identity/hidden; pointwise_mean_95/hidden; neutral/hue respectively. Do not confuse these with Line's actual lower/upper data band. |
| Line `band(lower, upper)` | Existing single/many NumericSeriesInput and origin checks; a one-column collection remains many. |
| Forest `scale`, Coefficient `reference_line` / `estimate_labels` | linear/log10, literal numeric reference and complete NumericFormat respectively. No invented None reference reset or formatter capability on other estimate kinds. |
| `markers(bool)`, Interaction `connectors(bool)` / `styles(entries)` | Real reversible bool setters; styles consumes `Sequence[InteractionStyle]`, not triples. |
| `color_by_sign()`, `zero_line()`, `hide_point_outline()` | Native no-argument one-way enable/hide commands, only on their actual capabilities. |
| Boxplot/Violin `raw_overlay`, Bar/HorizontalBar `spacing` | hidden/points/jittered and comfort/compact; no mode switch or observation filtering. |
| Heatmap `layout`, `annotations`, `heatmap_null_row_annotation` | cartesian or lower_triangle/upper_triangle with explicit diagonal bool; annotations accepts `NumericFormat | Literal["hidden"]`, rejects None. Null-row annotations append literal raw row identity plus complete Text. |
| Waterfall `total_labels(base, terminal)`, `total_format` | Full replacement of two independent Text values and native numeric policy. Do not simulate independent retained text slots or alter total eligibility. |

Marginal has no extra options/bin/bandwidth object. Matrix has its real ordered
variables, display metadata, hue/group and point/fill alpha, but no generic facets,
numeric-axis setters, panel guides or text annotations. Preserve exact capabilities
even when a rendered result visually resembles another family.

Plot legend policy is automatic/hidden/visible; only visible admits position.
Visible with omitted position means Visible(Auto), not Automatic. Expose all 25
native LegendPosition variants as one-to-one snake_case literals, with no invented
center. Legend composition is combined/separate. Canvas retains per_plot/shared/
hidden policy and composition independently of captured Plot policies.

The six native-only immutable appearance values have been delivered:

| Value | Selected Python shape |
| --- | --- |
| CategoricalColorMap | Positional ordered `(str, OpaqueColor)` entries; readonly entries, `color(raw)` and keyword-only complete entries copy. Native duplicate-key admission and active-domain coverage remain. |
| CategoricalColorScale | Closed from_palette/from_map factories, readonly kind/palette/map, same-branch copy. Required payload None and wrong-branch edits reject. |
| ColorDomain | Positional minimum/maximum, readonly and complete copy; finite minimum <= maximum, including a constant. |
| ContinuousColorScale | Closed sequential with optional gradient/domain; diverging with required center and optional gradient/domain. Read kind/domain/center/sequential_gradient/diverging_gradient. Copy preserves branch and validates the final center/domain combination together; matching gradient/domain None means Theme/automatic, not a resolved snapshot. |
| InteractionStyle | `InteractionStyle(group, *, point_shape="circle", line_pattern="solid")`; readonly/copy all three. Native circle/triangle_up and solid/dashed choices; literal raw group identity. |
| SignificanceMarkStyle | Keyword-only font_size=16, weight="normal", gap=8; readonly/copy all three. Gap is finite nonnegative CSS pixels. Explicit default remains distinct from Theme inheritance. |

Attach these only through actual CategoricalScale, ContinuousColor, Interaction
and SignificanceStyle consumers. No native reader gap exists for these values.

Guides use flat owner entries `guide_horizontal`, `guide_vertical`, `guide_segment`
with literal numeric coordinates and explicit Color/width/solid-or-dashed pattern,
internally constructing native GuideStyle/GuideSpec and appending. Text annotations
use two additional factory-only, opaque frozen configuration classes, each holding
one native value, with this sole positioning grammar:

```text
TextCoordinate.data(value: DataCoordinate)
TextCoordinate.data_area_fraction(value: float)
TextCoordinate.figure_fraction(value: float)
TextPosition.coordinates(*, x: TextCoordinate, y: TextCoordinate)
TextPosition.corner(value: Literal["top_left", "top_right", "bottom_left", "bottom_right"])
text_annotation(position: TextPosition, text: str | Text, *, style=omitted, layout=omitted)
```

`coordinates` calls native TextPosition::new; x/y may independently mix coordinate
domains. Fractions are finite but not clamped to 0..1. Required arguments reject
None. No new reader, coordinate mirror, fake field-copy API or parallel flat
position grammar is needed. These authored scalar references are not scientific
column inputs. Do not add GuideSpec, layer or annotation builder hierarchies merely
for symmetry. Retained text slots, role-layout attachments and clears remain
capability-gated; new append-only annotations clear incoming Text instructions,
not earlier entries.

### Complete Canvas Configuration

Canvas also retains the native non-legend composition capabilities, not just
Plot placement. Expose `panel_title_alignment` / `panel_subtitle_alignment` with
start/center/end literals; these are embedded-Plot defaults, not edits of captured
Plot instructions or Canvas-owned prose.

Use `column_header(start_column, span, text, *, style=omitted, layout=omitted)`,
`row_header(start_row, span, text, *, style=omitted, layout=omitted)` and
`row_title(row, text, *, style=omitted, layout=omitted)`. Each appends one native
entry; the shared TextIntent path edits the incoming text, not earlier entries.
Native span, overlap and source validation stay fallible and failure-safe.

`text_panel(row, column, *, title: str | Text, body: str | Text)` constructs the
existing native TextPanel and uses strict single-cell placement. Each complete
Text independently carries its instructions. Do not add a TextPanel facade,
fake full-Text readers or spans/overwrite that the native text-panel API does not
provide. Existing Plot-panel overwrite may still replace whole intersecting
TextPanel owners through the native placement authority.

`category_labels` accepts every_plot/first_column_per_row. `x_range_group` accepts
an ordered sequence of literal `(row, column)` cells, not observation selectors.
Forward both to the original native methods, preserving range-group constraints
and placement/preflight timing. Keep canonical callback-root captures and native
topology as the only authorities; do not mirror headers, ranges or occupancy in
Python. These mappings need no new core API.

## Statistical Policy Values And Necessary Readers

Five frozen classes hold only their corresponding native values:

| Class, keyword-only construction | Readonly and derivation |
| --- | --- |
| StandardDeviation(scale=1), StandardError(scale=1) | scale; clone and native positive-finite scale edit. |
| PercentileInterval(level=.95) | level; clone and native strict (0,1) coverage edit. |
| Bootstrap(resamples=1000, seed=0, units=None, data=None) | resamples/seed only. Snapshot units from this call's explicit context, never a future Plot's source. Copy omitted units retains native data; units=None rebuilds without units while preserving current native resamples/seed. Retain no source or units mirror. |
| ConfidenceInterval(level=.95, bootstrap=omitted) | level/bootstrap; native omitted default is percentile bootstrap. Explicit None selects Student-t; Bootstrap selects that configuration. Copy omitted preserves algorithm and data. No second algorithm selector. |

Only raw-sample Bar/HorizontalBar constructors accept the four error-policy
values or `"hidden"`; omission retains the existing native bootstrap default.
Outer error_bar=None is invalid, and supplied-estimate forms do not accept it.
Bootstrap units are ordinary bound categorical observations, not another input
route or a persistent callback; alignment and multilevel semantics stay native.

Deliver exactly these public native readers with actual consumers:

| Owner | `&self` reader and consumer |
| --- | --- |
| StandardDeviation, StandardError | Each `get_scale() -> f64`, for frozen scale properties. |
| PercentileInterval, ConfidenceInterval | Each `get_level() -> f64`, for frozen level properties. |
| Bootstrap | `get_resamples() -> usize`, `get_seed() -> u64`, for properties and units=None copy. |
| ConfidenceInterval | `get_bootstrap() -> Option<&Bootstrap>`, for current algorithm-preserving copy/read. |
| `Plot<K: kind::Facet>` | `get_facet() -> Option<&FacetSpec>`, for Core-origin scales-only edits of the existing bound facet. |

These eight readers do no validation/rendering or source lookup. Do not expose
private ErrorBarKind/ConfidenceIntervalAlgorithm, units contents, PlotDescription
or unused symmetric adapter readers. Both adapters already have facet caches and
scales edits and attached-facet/grid significance helpers.

## Supplied Raincloud Hue Title

The exact Raincloud native methods `hue_label(impl Into<Text>) -> Self`,
`clear_hue_label_style() -> Self` and `clear_hue_label_layout() -> Self` are delivered
with the real Python Raincloud consumer. No new capability trait, scientific hue
setter, public title reader or other-family categorical-title API is selected.

Store one private `raincloud_hue_label: Text` in SharedPayload, initially UNSET.
Use original replace_from/COW and independent instruction clears. Empty authored
text is present; clearing style/layout retains content/presence. Python uses the
existing TextIntent command path: setter followed by requested native clears,
so explicit None overrides incoming and retained instructions correctly.

Consume the slot only for Raincloud geometry with a hue dimension, never sample
series or no-hue geometry. Do not gate merely on hue_dimension_key, which also
covers sample series. Hidden legends do not create a guide. Stored unused text
still joins authored-source validation after existing slots, preserving previous
error order and Data classification for illegal controls; only reached layout
enters fonts/measurement. Do not introduce state-specific rejection or new fonts.

In discrete guide preparation, choose full authored Text before fallback only
for Combined(Field(raw_hue)) or HueColor(raw_hue). Otherwise retain the exact
raw-field -> effective Labels -> String -> Text fallback. Entry labels, raw
dimensions/keys, alpha membership, occurrence identity and color coordination
remain unchanged. Do not first consume an overridden alias or stringify Text.

Reuse prepared Arc<Text>, full Text equality, title collision/shared-guide merge,
existing appearance inheritance, text census, measurement and font policy.
Same text with different style or layout presence remains a semantic conflict,
not permission to take the first contributor. Extend ScopedGuideInputs capture
and source matching with the complete raw slot, including UNSET and presence.
The three necessary crate-private helpers are a geometry+hue-gated authored
borrow, fallible raw-slot copy, and full raw-slot equality, matching the existing
color-title cache pattern. No second editable title source or new public reader.

## Status, Alpha, Layers And Results

EstimateStatus owners expose literal estimated/not_estimable sequences plus
Boolean and categorical-mapping input forms. True/false are explicit statuses;
missing/unmapped and duplicate consumed keys retain native errors. Do not infer
availability from nullable estimates or attach this capability to Bar/Interaction.
Heatmap highlight is Boolean border input, not a validity mask.

For each admitted point/line/fill alpha target, retain native row, `_cells`,
`_single`, `_keyed`, `_columns` and Histogram fill-alpha-bins scopes and their
existing independent-source variants. Raw inputs are exact bytes; ordinary
one-element vectors do not become scalar broadcast. A no-argument `_theme()`
method resets through AlphaEncoding::theme, not an overloaded None/string input.

Mapped variants choose one complete native factory through mutually exclusive
linear=(domain, byte_range), log10=(domain, byte_range),
threshold=(boundaries, alpha_bytes), continuous=(domain, scalar_mapper), or
categorical mapper forms. Native AlphaValues retains declaration/guide
representatives, clamp, threshold and quantization semantics, not merely bytes.
Keyed fields use actual semantic raw identities through the existing key codec;
generated identities/ordinals come from metadata, not display labels or str(key).

Finish all needed Rust-owned scratch preparation and end every foreign ndarray,
Arrow/capsule and foreign-backed adapter loan before calling Python mappers.
Being outside an owner borrow or receiving owned mapper arguments alone is
insufficient. Resume the original producer with ordinary non-native-slot inputs
and original scope/validation order. Do not run later scientific checks merely
to prefetch keys. Do not promise identical foreign lookup timing to borrowed Rust.
Continuous/categorical mappers return u8 from float/raw-string arguments; native
factories determine their call domain, not a per-observation promise. Keep the
original PyErr in a call-local carrier and stop further mapper calls after failure.
A pending original PyErr takes precedence over subsequent native or stale errors;
only a successful producer without a PyErr proceeds to stale-check/paired commit.
Never commit fallback bytes. Borrow-free cleanup may still invoke finalizers.
No persistent callback slot or retained mapper is needed.

### Alpha Call Forms

These 75 entries are implemented with the bounded verification recorded in
`91_WORKSPACE_AND_INTERFACES_PLAN.md`: three targets each have 23 entries,
including their Theme reset, plus six Histogram bin entries.

For `T = point_alpha | line_alpha | fill_alpha`, expose the existing adapter
method families on precisely the corresponding sealed capability sets:

| Scope | Python call shape |
| --- | --- |
| Row / expanded cells | `T(input)`, `T_cells(input)` |
| Native linear convenience | `T_linear(input, domain, range)` |
| Numeric mapping | `T_map(input, *, mapping)` and `T_cells_map(input, *, mapping)` |
| Categorical mapping | `T_map_categories(input, *, mapper)` and `T_cells_map_categories(input, *, mapper)` |
| Explicit broadcast | `T_single(*, value=omitted, input=omitted)`; exactly one is present |
| Mapped broadcast | `T_single_map(input, *, mapping)`, `T_single_map_categories(input, *, mapper)` |
| Semantic keys | `T_keyed(input, *, keys)` and corresponding `_map` / `_map_categories` forms |
| Generated columns | `T_columns(input, *, column_key, other_keys=())` and corresponding mapped forms |
| Reset | `T_theme()` |

In the table, `mapping` is notation for exactly one of the real keyword
parameters `linear=(domain, byte_range)`, `log10=(domain, byte_range)`,
`threshold=(boundaries, alpha_bytes)` or `continuous=(domain, scalar_mapper)`.
There is no `mapping=` argument or unrestricted keyword bag. Validate this
complete combination before source lookup. Categorical forms require the
keyword-only scalar `mapper`. Preserve the adapter's method names, including
`_cells_map_categories`, rather than adding interchangeable aliases.

Only keyed and generated-column scopes have independent-source variants, named
as in the adapters: `_keyed_from_data`, `_keyed_map_from_data`,
`_keyed_map_categories_from_data` and the analogous `_columns` names. Their
`data=None` is the sole positional argument; `input`, scope fields and mapper
keywords are keyword-only. None means direct-only, not retained-main-source
fallback. Ordinary forms use the retained main source when a selector needs it.

Histogram additionally exposes `fill_alpha_bins(input, *, lower, upper, facets=())`,
`fill_alpha_bins_map` and `fill_alpha_bins_map_categories`. Their `_from_data`
variants use the same source-first, remaining-keyword-only convention. Bounds and
facet inputs belong to this selector table, not the sample observations.

`keys`, `other_keys` and `facets` are ordered list/tuple collections of
`(semantic_field, categorical_input)` pairs; do not collapse duplicates into a
dict. A semantic field has two nonoverlapping canonical spellings: a plain string
means the literal native role/raw field; `col[original_key]` means the already-bound
pandas identity in the owner's existing key registry. The latter does not look
up observations or install a new identity; an unbound key is an error.
Do not resolve a plain string through that registry, stringify a key or use a
display alias. Thus raw `"hue"` and a pandas field addressed as `col["hue"]` can
coexist without priority rules or guessing from pair order/values. The independent
table changes where values are read, not which scientific fields they identify.
Generated `column_key` uses the existing
origin's logical column IDs/raw identities or local ordinals, not display labels;
Matrix display metadata does not invent an expansion.

Raw native list/tuple alpha members and explicit `value` are checked byte
integers (not bool, float opacity or truncating casts); foreign byte variables
retain the native UInt8/u8 dtype and missing-value checks. `input` remains an
ordinary variable/selector, including in `T_single`; only `value` selects a
literal scalar. None is neither a value nor a reset. Numeric/categorical mapped
inputs retain their existing ordinary role admission. These are call-local
inputs, not new public alpha payload/configuration classes.

### Necessary Identity-Only ndarray Projection

The required namespace-only branch is delivered by exactly
`ColumnProjection::from_identities(source_shape, identities)`, returning the
existing `Result<Self, AdapterError>` with the original descriptor representation.
It carries actual source shape and explicit raw identities, no value entries.
Restored named expansion with direct alpha/direct column-ID vectors therefore
does not import unrelated observations merely to satisfy `new()`'s unchanged
nonempty-value rule. The shared working contract is in `09_DATA_ADAPTERS.md`.

The real named-expansion Python alpha consumer and focused native checks accompany
this seam. Original bounds/identity validation, unprojected-value failures and
explicit-empty-identity no-fallback semantics remain. There is no parallel source
class, mode mirror, cache, new core reader or independent-None main-source fallback.

### Independent Layer And Result Call Forms

These call forms are implemented, with bounded installed verification recorded in
`91_WORKSPACE_AND_INTERFACES_PLAN.md`.
Only Line exposes
`point_layer(data=None, *, x, y, labels=None, color=None, radius=None, alpha=None, legend_label=None, legend_label_style=omitted, legend_label_layout=omitted, label_text_layout=None)`.
Only Bar/HorizontalBar expose
`point_layer(data=None, *, category, value, color, radius, series=None)`.
The latter has no labels, legend, layout or independent alpha parameters: its
required Color already owns complete RGBA. Radius is an integer native u32;
Line's optional explicit alpha is a checked byte, not floating opacity.

Each call appends one independently bound layer. None means direct-only data,
never parent lookup; optional None instructions are absent on this new layer.
For Line, a present legend label (str or Text, including empty authored text)
uses the existing TextIntent rules with the two explicit legend instruction
keywords. Instruction omission retains incoming Text instructions; explicit None
clears that instruction. Supplying either instruction without a legend label is
an incomplete form, rejected before data lookup. `label_text_layout` controls
column-backed point labels, not the complete-layer legend label.

Expose exactly these six method names for each placement, replacing `P` with
`attached` or `pairwise`:

| Context | Python method |
| --- | --- |
| Main/direct results | `significance_P(*, result_fields)` |
| Independent results | `significance_P_from_data(data=None, *, result_fields)` |
| One facet | `significance_P_in_facet(*, result_fields, facet_fields)` |
| Independent one facet | `significance_P_in_facet_from_data(data=None, *, result_fields, facet_fields)` |
| Grid facet | `significance_P_in_grid(*, result_fields, grid_fields)` |
| Independent grid facet | `significance_P_in_grid_from_data(data=None, *, result_fields, grid_fields)` |

The table names groups of real keyword parameters, not `result_fields=`,
`facet_fields=`, `grid_fields=` or arbitrary keyword bags. Attached capability
is exactly Bar, HorizontalBar, Coefficient; pairwise capability is exactly Bar,
HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud.
All result variables are keyword-only. Ordinary methods use fresh main-source
lookup when needed; `_from_data(None, ...)` is direct-only.

For attached placement, require exactly one of `target` or `target_value`.
For pairwise placement, require exactly one of `left`/`left_value` and exactly
one of `right`/`right_value`. A column choice accepts an ordinary categorical
input and optional corresponding `target_series`/`left_series`/`right_series`;
series None is absent, and a series argument is not a scalar-address modifier.
A `_value` choice is a literal raw str or `(category, series)` string tuple.
Never guess literal/scalar status from an ordinary input's length or tuple shape.

Marks have exactly three complete forms: numeric `marks` with mapping omitted;
categorical `marks` with required `mapping`; or literal `mark` with both `marks`
and `mapping` omitted. A literal/mapped mark is a numeric p-value or one of
`"ns"`, `"*"`, `"**"`, `"***"`, `"****"`. Mapping accepts a Mapping or ordered
list/tuple of `(raw_category, mark)` pairs; keep duplicate pairs for native
consumed-key checks, not dict normalization. Explicit mapping=None is invalid,
not numeric-mode omission. Native p-range/nonfinite/glyph checks retain their
existing timing; Python does not calculate a test or adjustment.

One-facet methods require either categorical `facet` or literal string
`facet_value`. Grid methods require both categorical `facet_row` and
`facet_column`, or literal `facet_value=(row, column)` strings, never a mixture.
Unfaceted methods accept none of these keywords. Validate complete presence
combinations before any source lookup, then retain the original producer's
scientific check order, scalar flags and broadcasting. Grid literal FacetCell
may use the existing scalar-capable `_in_facet` producer internally; ordinary
grid columns use the existing `_in_grid` producer. Do not emulate broadcasting
or add a native facet/grid helper.

Layers are flat append-only owner calls with an independent `data` context:
Line point_layer accepts x/y, optional labels/color/radius/alpha/legend_label/
label_text_layout; Bar/HorizontalBar accept category/value, optional series and
required complete Color/radius, with no independent alpha parameter. Use existing
PointLayerAdapter/BarPointLayerAdapter, retaining their optional Theme instructions
and native membership/radius rules. Auxiliary None never borrows parent lookup;
do not retain the auxiliary source when no public reusable layer needs it.

Results use capability-gated significance_attached/significance_pairwise and
their existing independent-source/facet/grid producers. Column targets use
target/left/right plus optional corresponding series columns; explicit scalar
target_value/left_value/right_value accepts raw str or category/series tuple.
marks is a p-value column or categorical column with explicit mapping; mark is
the mutually exclusive literal p-value or ns/*/**/***/****. Facet columns use
facet or facet_row/facet_column; facet_value is an explicit literal level/tuple.
Check complete combinations before lookup, then use original scalar flags and
broadcasters. Length-one vectors stay vectors; preserve independent result count,
append order, target/facet membership and native p-value/glyph validation.
Do not calculate tests or adjustments, expose result payload classes, or add
already-existing attached-facet/grid native helpers.

## Complete Numeric Values And Callback Slots

The owning policy values, full-family forwarding and all eight physical slots
are implemented within the bounded checks recorded in the workspace/interface
plan. That evidence alone does not certify the separate scientific closures or
final distribution/content/cost and budget acceptance.

No new core numeric reader is required. Owning policies and existing scoped
callback views have distinct lifetimes; do not replace a scoped view by a detached
snapshot or expose private context fields. All unedited opaque native fields and
emitter identity survive clone/edit; do not reconstruct from Python defaults or
remember specifiers/constructor arguments.

NumericFormat constructor/copy and value_function/set_function factories accept
the full native policy: notation, precision, decimal_scale, prefix, suffix,
negative_zero, trailing_zeros, rounding, multiplier, rate, locale, grouping, sign,
symbol, field, prefixes, unit and explanation. Replace the old fractional_digits/
field_width shorthand with precision/field, migrate repository callers and remove
obsolete signatures/property rather than retaining a compatibility route.

Notation is decimal/scientific/engineering/general/integer_decimal/integer_binary/
integer_octal/integer_hex_lower/integer_hex_upper; negative_zero preserve/normalize;
trailing_zeros keep/trim; rounding nearest_even/nearest_away/nearest_toward_zero/
toward_zero/away_from_zero/floor/ceiling; rate value/percent/per_mille; sign
negative_only/always/space/accounting; symbol none/currency/radix_prefix.
Use native validators and defaults, not Python algorithms or a restricted default
profile. Canonical constructor defaults remain Decimal/Fractional(2).

Field/unit None are real clears. Prefix/suffix constructor absence means absent;
their native edits require nonempty strings, so copy None and empty strings are
not resets. Prefixes/explanation resets use explicit native value factories.
Format attachment setters accept NumericFormat, not None/default reconstruction.
Heatmap `"hidden"` clears only the HeatmapValues roots, never continuous color.

Notation/precision/symbol are coupled. Resolve all requested inputs first and
choose a valid native clone/edit order so legal simultaneous edits are accepted
without dropping opaque policy or emitter identity. General/Significant(3) to
hex/Fractional(0) is a required representative. A temporary Decimal bridge must
not invalidate retained radix-prefix policy; apply an explicitly changed
non-radix symbol first, and introduce radix_prefix only after legal integer state.
Final incompatibility still fails natively.

| Native-only Python value | Complete selected mapping |
| --- | --- |
| NumericPrecision | Closed fractional/significant/adaptive_fractional/adaptive_significant factories, readonly branch data and same-branch copy. u8 conversion at binding; native format attachment validates bounds/notation compatibility, including legal degenerate adaptive forms. |
| NumericField | Opaque `NumericField(width, *, fill=" ", alignment="right")`; fill is one Unicode scalar; left/right/center/after_sign. No invented field readers or shadow args. |
| NumericLocale | Closed invariant/named/named_ids/custom factories. custom decimal/digits and required grouping/symbols/currency; grouping may explicitly be None. Read/copy native token components; do not retain original named ID or use host locale. |
| NumericGrouping, NumericSymbols, NumericCurrency | Positional token records; grouping additionally required repeat=cycle/last/stop. Read/copy complete native fields. Preserve literal tokens and distinct empty-currency/nonempty-symbol admission. |
| SiPrefix | Closed from_exponent; readonly exponent/symbol, using the actual native table including 2022 extremes, not all integers in a range. |
| NumericCompactRule | Positional threshold/divisor, required keyword prefix/suffix; four readonly/copy fields. Native threshold need not be nonnegative; affixes may be empty. |
| NumericFactor | Closed automatic_decimal/from_value/power_of_ten; public enum branch inspection only. Exact arbitrary signed exponent text is not restricted through i64/f64. Native prefixes attachment owns normalization. |
| NumericPrefixes | Opaque closed none/si/decimal_compact/compact/factor. SI fixed selection uses SiPrefix; compact fixed uses integer index including zero. promotion is keep_threshold/promote_on_rounding. Required selection/promotion, original ordering and empty-table/native fixed rules remain; no invented inner getters or field mirror. |
| NumericUnit | Positional text, required per_label/shared placement, separator default space; native readonly/copy. Token checks remain at format attachment, not standalone construction. |
| NumericExplanation | Closed automatic/text(str-or-Text), common style/layout keywords and instruction-only copy using native style/layout/clear operations. Keep generated versus authored content; no fake empty Text or public inner-source reader. |

NumericPrecision kind is fractional/significant/adaptive_fractional/
adaptive_significant. Readonly digits is present only on the two exact branches;
minimum/maximum are present only on adaptive branches. Its keyword-only copy
accepts omitted digits/minimum/maximum and edits only the current branch.
Wrong-branch fields reject even when None; required payload None is not a clear.
Standalone construction/copy performs u8 conversion, not format validation.

NumericFactor kind is automatic_decimal/value/power_of_ten, with readonly optional
value/exponent properties derived from the actual public enum. Keyword-only copy
accepts omitted value/exponent and edits only the current payload branch.
Automatic permits no-argument copy but rejects any payload edit, including None.
Native factor attachment, not standalone copy, validates finite nonzero divisors
and exponent syntax/canonicalization.

NumericFormat readonly fields use its existing seven readers: notation, precision,
decimal_scale, prefix, suffix, negative_zero, trailing_zeros. Other inputs need
not gain opaque getters. from_specifier calls the native compiler once and retains
only normalized native policy. Factory-only classes must actually reject bare
construction, with accurate NoReturn typing rather than fictitious initializers.

Plain numeric policy copy clones the emitter and INCREFs the same physical holder.
Only an explicitly supplied callback key installs a new native emitter/holder.
At most one callback key may be present, including None; explicit None calls
builtin and retires only that copy's roots without resetting built-in policy.
Preserve borrow-free callback validation and DECREF/finalizer behavior.
NumericLabels explanation uses the shared TextIntent path: absent replacement
does not clear enclosing generated/configured explanation; empty authored text
is present. Style/layout alone must not invent content.

TickGeneration keeps automatic/additive/target/maximum/exact_count/function and
adds log10_decades, interval(step,origin,*,space,requirement) and multiplicative
(anchor,ratio,*,requirement). Expose existing additive_step/decade_step/
interval_parameters/multiplicative_parameters readers; do not invent a primary
or callback-handle mirror. Copy supports singleton and maximum_count, preserving
the complete native primary/context binding. Singleton is lower/midpoint/upper or
finite numeric position; only ExactCount admits it. maximum_count None clears cap;
automatic resets the whole request. Ratio below one and exact count zero remain
legal native choices. TickPositions retains unsorted/repeated data or
log10_exponents values; materialization owns projection/window/cap applicability.
Exact authored ticks retain their native precedence and validation.

Expand physical owner roots from four slots to exactly eight: XFormat, YFormat,
ColorFormat, HeatmapValues, EstimateLabels, TotalFormat, XTick and YTick. Color is
ContinuousColor, heatmap values exact Heatmap, estimate labels exact Coefficient,
totals exact Waterfall; x/y remain their sealed numeric capabilities. No shared
slot, callable deduplication or extra mapper slot. Each real strong ref is
traversed/pinned once; holder traverses its callable once. Snapshot input policy
and drop its PyRef before committing/releasing old roots. Capture/copy, replacement,
failure and operation-local pins reuse existing lifecycle semantics. Output
retains no callback roots, and frozen publication does not invoke callbacks.

## Verification And Completion

Each remaining constructor needs a real public exact-family consumer and accurate
typing; transport evidence is shared, not a family-by-container matrix. Distinct
sample/estimate/geometry, wide/grid, nullable-status and independent-context
contracts need bounded representatives, not getter or smoke inventories.
All new native APIs must be delivered with real consumers; new readers retain
pure-reader scope.

Cover final-state policy edits, omitted/None branches, Bootstrap source release,
core/adapted facet-scale preservation, geometry raw identities and Raincloud title
selection/merge/census/clear. Cover alpha declaration/scope/reset, mapper loan
ending/PyErr/reentrant stale safety, independent layer/results scalar semantics,
and all four new persistent slots with actual attachment/retirement/capture.
Reuse existing diagnostics; do not create general fault-injection or quota systems.

Run strict locked/offline workspace all-target Clippy, focused affected native
tests and actual outside-source installed-wheel runtime/typing/stubtest. After
the full interface is implemented, independently unpack/build/install its actual
sdist, complete representative content and same-source cost observations, then
deduplicate tests and certify the unchanged whole-repository budgets. Never update
goldens to accept scientific drift or claim static API presence as runtime proof.
