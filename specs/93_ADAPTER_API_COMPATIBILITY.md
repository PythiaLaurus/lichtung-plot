# Adapter API Compatibility And Native Handoffs

## Status And Scope

This is the compatibility inventory for the proposed
[data-source plotting interface](92_DATA_ADAPTERS_PLAN.md), not an implemented
API. The two current adapter packages still expose conversion-only functions;
all `Scatter::new`/`from_data`, PlotAdapter, direct-binding and
adapter-layer examples below are prospective and have not been compiled. The core definitions
described here are established by the current source. Do not add a direct core
dependency to examples that are intended to demonstrate adapter-only use.

The selected architecture places data lookup, conversion, origin tracking and
configuration forwarding in the adapters. Both adapters expose 25 top-level
families with `new` and `from_data`, returning an adapter-owned exact-family
PlotAdapter as `std::result::Result<Self, AdapterError>`. `new` binds direct inputs,
including matrices; `from_data` additionally binds a dataset for field lookup. Both are data inputs,
not competing categories of containers or two renderers. Their exact arguments
and shape/semantic contracts are listed in
the [family inventory](92_DATA_ADAPTERS_PLAN.md#coverage-across-the-existing-api).
Native `Plot` and every other core export remain the original definitions.

## Complete Native Surface

Both adapters must use actual unrestricted native re-exports, including every
public descendant of `kind` and `stats`. For every public `lichtung_plot::path`,
substituting the adapter prefix alone retains the original definition, type
identity, method signatures, trait bounds, return types and behavior for that core
version. No native root item is shadowed. These groups describe the current
surface, not an allowlist: future public native exports are included as well.

| Group | Current native exports and compatibility |
| --- | --- |
| Plots, output and errors | `Plot`, `Canvas`, `CanvasCategoryLabels`, `TextPanel`, `Output`, `Error`, `Result`; all native constructors, methods, variants and ownership remain usable directly. |
| Modules and family contracts | `kind` and `stats` and their public descendants, including exact-family markers, capability bounds and public statistics; actual module identities are preserved. |
| Data inputs | `NumericValues`, `CategoryValues`, `BooleanValues`, `SourceRows`, `AxisValues`, `HistogramInput`, `RaincloudInput`, `Bounds`, `BarInput`, `BarSamples`, `CoefficientInput`; no wrapper or alias replaces a native name. |
| Layers and annotations | `GuideSpec`, `GuideStyle`, `PointLayer`, `BarPointLayer`, `Attached`, `Pairwise`, `Significance`, `SignificanceMark`, `SignificanceTarget`, `FacetCell`, `Labels`, `AxisTick`; preserve placement types and all constructors. |
| Alpha and geometry | `AlphaEncoding`, `AlphaSemantics`, `AlphaValues`, `HistogramBin`, `HistogramDimensions`, `HistogramGeometry`, `HistogramKdePoint`, `HistogramRug`, `RaincloudDensityPoint`, `RaincloudDimensions`, `RaincloudGeometry`, `RaincloudGroup`, `RaincloudPoint`. |
| Plot policies | `AxisScale`, `BarSpacing`, `Bootstrap`, `CanvasLegendPolicy`, `CategoricalColorMap`, `CategoryLabelSelection`, `CategoryOrder`, `ConfidenceInterval`, `DumbbellEndpoints`, `ErrorBar`, `EstimateStatus`, `FacetScales`, `FacetSpec`, `HeatmapAnnotations`, `HeatmapLayout`, `HeatmapNullRowAnnotation`, `HistogramBins`, `HistogramComposite`, `HistogramKde`, `HistogramStat`, `InteractionStyle`, `LabelEndpoints`, `LegendComposition`, `LegendPolicy`, `LegendPosition`, `LinePathStyle`, `LinePattern`, `PanelCorner`, `PanelTextAnnotation`, `PercentileInterval`, `PlotKind`, `PointShape`, `QqLine`, `RawOverlay`, `RegressionBand`, `SignificanceMarkStyle`, `StandardDeviation`, `StandardError`, `TextWeight`, `TickContext`, `TickGeneration`, `TickPositions`, `TickRequirement`, `TickSingleton`, `TickSpace`, `ValueSort`, `WaterfallLabels`, `PngCompression`, `PngOptions`, `PngSizing`. |
| Numeric contracts | `CoordinateAxis`, `NumericDomain`, `NumericFactors`, `NumericIdentity`, `NumericLabels`, `NumericOperand`, `NumericOwner`, `NumericPurpose`, `NumericRangeOrigin`, `NumericRole`, `NumericSetContext`, `NumericValue`, `NumericValueContext`, `NegativeZero`, `NumericNotation`, `NumericRadix`, `NumericRounding`, `TrailingZeros`, `NumericAlignment`, `NumericField`, `NumericRate`, `NumericSign`, `NumericSymbol`, `NumericFormat`, `NumericPrecision`, `NumericCurrency`, `NumericGroupRepeat`, `NumericGrouping`, `NumericLocale`, `NumericSymbols`, `NumericCompactRule`, `NumericFactor`, `NumericPrefixes`, `NumericPromotion`, `NumericSelection`, `SiPrefix`, `NumericExplanation`, `NumericUnit`, `NumericUnitPlacement`. |
| Text, fonts and backend style | `FontFamily`, `FontStack`, `GenericFontFamily`, `DataCoordinate`, `DataPosition`, `GuideTextRole`, `Text`, `TextAlignment`, `TextAnchor`, `TextBounds`, `TextCoordinate`, `TextDirection`, `TextElision`, `TextEllipsis`, `TextFailure`, `TextFit`, `TextFitGrouping`, `TextFitPriority`, `TextLayout`, `TextOffset`, `TextOffsetFrame`, `TextOrientation`, `TextOverflow`, `TextOverlap`, `TextPosition`, `TextRotation`, `TextRotationMode`, `TextVisibility`, `TextWrapping`, `StrokePattern`. |
| Theme and paint | `AccentPalette`, `AnnotationTextRoles`, `AnnotationTheme`, `AxisSpacing`, `AxisTextRoles`, `AxisTheme`, `BandFillRoles`, `BodyFillRoles`, `CategoricalColorScale`, `CategoryMarkGeometry`, `CellAnnotationGeometry`, `CellMarkRoles`, `Color`, `ColorDomain`, `ColorScales`, `ColorStop`, `CompositionTextRoles`, `CompositionTheme`, `ConnectorStrokeRoles`, `ContinuousColorScale`, `ContinuousScaleKind`, `ContrastTextRoles`, `CurveStrokeRoles`, `DataFillTheme`, `DataLineRoles`, `DataLineTheme`, `DataOutlineTheme`, `DensityFillRoles`, `DivergingGradient`, `EstimateAnnotationGeometry`, `FacetStripSpacing`, `FillRoles`, `FixedFillTheme`, `FrameSides`, `FrameTheme`, `GradientLengths`, `GuideTextRoles`, `HeaderSpacing`, `LegendFrameTheme`, `LegendSpacing`, `LegendTheme`, `MarkExtentTheme`, `MarkTheme`, `NeutralPaintRoles`, `OpaqueColor`, `OutlineTheme`, `PaletteCapacity`, `PointOutlineTheme`, `PointRadii`, `PointRoles`, `PointTheme`, `QualitativePalette`, `ReferenceStrokeRoles`, `RelativeSpacing`, `RootPadding`, `RootTextRoles`, `RootTitleSpacing`, `SequentialGradient`, `SignificanceGeometry`, `SizedPointTheme`, `SolidStrokeTheme`, `StrokeRoles`, `StrokeTheme`, `SummaryStrokeRoles`, `SupplementalPointTheme`, `SurfaceTheme`, `TextPanelSpacing`, `TextTheme`, `Theme`, `ThemeBuilder`, `TickTheme`, `TypographyTheme`, `ValueShelfSpacing`. |

All associated functions, methods, variants, callbacks and nested public module
items of these types remain available through the actual re-export. A core-only
workflow needs only a crate-prefix substitution, without a new `?`, trait import,
conversion or wrapper. The original `Plot` type remains available even though a
family constructor's result is a different, adapter-owned PlotAdapter.

## Adapter Plot Handoff

Each PlotAdapter owns its actual native `Plot<K>`, with the same exact family marker.
`from_data` additionally retains its own source reference and origin/shape metadata;
`new` retains required metadata but does not retain direct input views
after the corresponding binding. Public adapter types have private fields and may
be named by callers. PlotAdapter `Clone` preserves native clone behavior, shares or
clones its read-only context safely, and never changes source membership. Any
clones retaining the source keep their own live borrow until consumed or dropped.

For Rust's `new` convention and strict Clippy, the family entry point denotes the
actual exact-family frontend: `new` and `from_data` return
`std::result::Result<Self, AdapterError>`, not a distinct factory's unrelated result. An alias such as
`Scatter<'source> = PlotAdapter<'source, kind::Scatter>` is a candidate, subject
to a prototype for inference, source lifetimes and the documented direct input
shapes. `PointLayerAdapter` and `BarPointLayerAdapter` likewise own their constructors.
Do not keep a second factory or old constructor as an alias; do not suppress
`new_ret_no_self` merely to retain an unrelated factory return type.

The adapter implements `From<PlotAdapter<..., K>> for Plot<K>`. Binding methods
materialize the input they attach before returning, so the conversion moves the
native Plot and releases lookup context without data lookup, rendering or fallible
scientific preparation. Native preparation may still reject a configured plot.
No `build` method or new native-name wrapper is provided.

```rust
use lichtung_plot_polars as pp;

let adapted = pp::Scatter::from_data(&frame, "time", "response")?
    .hue("condition")?
    .title("Response");

// The annotation makes the target type explicit at the native boundary.
let native: pp::Plot<pp::kind::Scatter> = adapted.into();
let canvas = pp::Canvas::grid(1, 1)?.panel(0, 0, native)?;
```

An implementation must test whether `canvas.panel(0, 0, adapted.into())`
infers the exact family with the current native signature. The annotated version
is the baseline until proved otherwise; neither is an implicit argument conversion.
The original source can be dropped or mutated after the adapter value has been
consumed, subject to other outstanding borrows. Native Plot, Canvas and Output
retain their existing owned/frozen behavior.

The same conversion serves user functions and collections that explicitly require
native Plot. Converting consumes the adapter object and ends source-bound chaining;
it does not change the target family's scientific capabilities. `PlotAdapter::erase`
for an exact family delegates to core erasure, releases the context and returns
the native `Plot<kind::Any>`; that erased value remains ineligible for Canvas
`panel` and `panel_span`, exactly as in the core. `kind()` is forwarded without
erasure. Source-independent `.svg`, `.svg_preserve_text`, `.png`, `.png_preview`
and `.png_with_options` consume PlotAdapter, delegate to native Plot and return the
native `Result<Output>`; the caller need not request a conversion first.

## Chainable Data Bindings

The adapter defines local, role-specific input protocols. Fallible data-binding
methods on the adapter receiver return `std::result::Result<Self, AdapterError>`.
Their argument lists need not match native methods on `Plot<K>`; those original
methods and return types remain available through the real re-export. A source
selector on a plot created by `new` without an appropriate source is an error.
String field selectors, literal labels and significance targets retain distinct roles.

| Adapter method | Accepted input / conversion | Native capability and constraints |
| --- | --- | --- |
| `hue` | Source field/index, admitted category vector/view or native `CategoryValues`; bind to native category values. | `K: kind::Hue`; categorical identity, not numeric continuous color. |
| `group` | Same category forms; convert using the graphical-group role. | `K: kind::Group`; does not replace a constructor-owned group where the core lacks this setter. |
| `series` | Same category forms; convert using the positioned-series role. | `K: kind::Series`; native supplied-geometry restrictions remain. |
| `source_rows`, `source_rows_cells` | Exact integer field/view or an already native `SourceRows` in the same role; `_cells` explicitly selects expanded slots. | `K: kind::SourceRows`; non-null, nonnegative, fitting and unique IDs; never repeat one ID across expanded cells. |
| `facet_row`, `facet_column`, `facet_grid`, `facet_wrap`, `facet_scales` | Direct fields/vectors plus native `FacetScales`; applicable cell/column variants in the redesign. | `K: kind::Facet`; preserve row/column/wrap distinction, scale policy and native facet restrictions. Existing `facet(FacetSpec)` can forward unchanged. |
| Line `band` | `.band(lower, upper)` with independently supplied fields/vectors. | Only exact Line; many-y bounds require matching collections/shapes, with no implicit pair reuse. No `band_bounds` compatibility method. |
| `heatmap_highlight`, `heatmap_highlight_cells` | Boolean record vector/field, same-shape grid or explicitly expanded values. | Heatmap only; true adds a border, independently of validity. |
| `estimate_status_boolean`, `estimate_status_categorical`, `estimate_status` | Foreign Boolean/categorical inputs with explicit mapping, or native `EstimateStatus` iterator. | PointInterval, Coefficient and Forest; never infer status from missing numeric values. |
| `point_alpha`, `line_alpha`, `fill_alpha` and their unit/mapping methods | Direct u8 vectors, foreign fields/views or already-native `AlphaEncoding` in the same one-argument slot; explicit single values, mappings, keys and bins use their semantic operations. | Corresponding `kind::PointAlpha`, `LineAlpha`, `FillAlpha`; retain observation, Single, keyed series/aggregate or bin units as admitted by the core. |
| PointLayerAdapter `labels` | Layer-local field/index, direct category vector or native `CategoryValues`. | Labels align with that independent layer, not the parent Plot. |
| BarPointLayerAdapter `series` | Layer-local field/index, direct category vector or native `CategoryValues`. | Optional positioned-series identity resolves against the parent bar's prepared cells. |

The current exact-family data capabilities are listed here so that accepting
foreign inputs never accidentally broadens native scientific capabilities:

| Capability | Native exact families |
| --- | --- |
| `source_rows` | All 25 families, with native data-dependent applicability. |
| `hue` | Scatter, Density, Line, Trajectory, LinePath, Interaction, Marginal, Matrix. |
| `group` | Density, Line, Marginal, Matrix, Dumbbell. |
| `series` | Bar, HorizontalBar, Boxplot, Violin, Raincloud; supplied Raincloud geometry has an additional runtime restriction. |
| `facet` | All 25 except Marginal and Matrix, subject to the core's additional data/layer restrictions. |
| `heatmap_highlight` | Heatmap. |
| `estimate_status` | PointInterval, Coefficient, Forest. |
| `point_alpha` | Scatter, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell, Boxplot, Violin, Raincloud, Regression, Marginal, Matrix, Beeswarm. |
| `line_alpha` | Histogram, Density, Qq, PointInterval, Line, Trajectory, LinePath, Interaction, Coefficient, Forest, Dumbbell, Bar, HorizontalBar, Boxplot, Violin, Raincloud, Regression. |
| `fill_alpha` | Histogram, Density, Line, Heatmap, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud, Regression, Marginal, Matrix, Waterfall. |
| Attached `significance` | Bar, HorizontalBar, Coefficient. |
| Pairwise `significance` | Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud. |
| Line `band` / independent PointLayer | Line only; Regression `band` is a different native policy. |
| BarPointLayer | Bar and HorizontalBar only. |

An already-native input accepted by the same adapter method retains its native
slot semantics; the adapter does not silently reinterpret it as a source-row
vector or expand it. Foreign row, selected-column and explicitly cell-scoped
bindings follow their owning operation's origin map. `Into` alone cannot provide
the retained source or decide whether input is row-, cell-, observation-, series-
or bin-aligned.

Do not add parallel public Interval, Facet, Alpha/AlphaMap, Key, Status, Axis,
Valid, Cells, Grid, HeatmapCells, Columns, RowIndex, ColumnIdentity, HistogramData,
RaincloudData, BarData/BarSampleData or CoefficientData types. Their meaningful
roles instead use adapter methods, ordinary tuples/views and the native types
listed above. The 25-family constructor and semantic-mode inventory specifies
fixed arity and explicit sample/estimate, grid/records and indexed/columns modes.
The implementation must prototype local trait coherence, `new` inference,
lifetimes and role-disambiguation; do not guess an input meaning from lengths.

An adapter may define a local input trait for foreign Series/Column/array types;
it cannot implement the foreign standard `From`/`TryFrom` trait directly between
two foreign types, such as Polars Series and core CategoryValues. `Into` is not
a fallible lookup protocol, does not automatically convert function arguments,
and does not recursively convert collection elements. A `From` implementation
and a separate fallible `TryFrom` implementation for the same type pair also
conflict with the standard blanket conversion. See the [Rust coherence rules][coherence].

## Full Plot Method Forwarding Inventory

The following covers all current public instance methods of native `Plot<K>`
from `builder.rs`, `builder/configuration.rs`, `builder/facets.rs`,
`builder/alpha.rs`, `builder/text_layout.rs`, `builder/accessors.rs` and
`orchestrator/output.rs`. Forward exact-family methods and the original `kind`
capability restrictions, including compile-time versus preflight distinctions.
Keep this inventory synchronized with the core when implementation begins.

| Method(s) | PlotAdapter compatibility |
| --- | --- |
| `hue`, `group`, `series`, `source_rows`, `heatmap_highlight`, `estimate_status`, `point_alpha`, `line_alpha`, `fill_alpha` | Use the role-specific direct adapter binding methods above; native receiver remains unchanged. |
| `facet` | Native FacetSpec can use the existing same-name forwarding method; adapter field/vector workflows use direct `facet_row`, `facet_column`, `facet_grid` and `facet_wrap` methods plus explicit scale/cell/column operations. |
| `band` | Adapter Line binds `.band(lower, upper)?` directly; native Line retains `.band(Bounds)`. No adapter `band_bounds` method is required. Regression forwards native `RegressionBand` as a separate pure policy, with its original return shape. |
| `point_layer` | Line accepts native PointLayer or prepared PointLayerAdapter; Bar/HorizontalBar accept native BarPointLayer or prepared BarPointLayerAdapter. Adapter method consumes `impl Into<NativeLayer>` and returns Self. Native methods retain exact native arguments. |
| `significance` | Forward the native `IntoIterator<Item = Significance<P>>` contract and `K: kind::Significance<P>` bound, returning Self. Use native annotations or the data preparation described below. |
| `title`, `subtitle`, `caption`, `x_label`, `y_label`, `title_alignment`, `subtitle_alignment`, `labels` | Forward the actual native text, layout, replacement and label-policy behavior; strings here are literal text. |
| `color_label`, `color_label_format`, `heatmap_null_row_annotation`, `estimate_labels`, `significance_mark_style`, `panel_text_annotation` | Forward native typed values with their actual capability bounds. |
| `x_category_labels`, `y_category_labels`, `category_order`, `x_category_order`, `y_category_order` | Forward native label-selection and category-order policies, including `Into<CategoryOrder>` where the core accepts it. |
| `x_limits`, `y_limits`, `x_ticks`, `y_ticks`, `x_tick_generation`, `y_tick_generation`, `x_tick_format`, `y_tick_format`, `guide` | Forward native numeric settings, exact ticks, callbacks and operation timing; respect axis and guide bounds. |
| `categorical_color_scale`, `continuous_color_scale`, `color_by_sign`, `zero_line`, `hide_point_outline`, `markers`, `theme` | Forward native scale/paint and appearance settings with native capability bounds. |
| `width`, `max_width`, `size`, `legend`, `legend_composition`, `raw_overlay` | Forward native complete sizing, legend, composition and raw-observation policies. |
| `layout`, `annotations` | Heatmap-specific native `HeatmapLayout` and `HeatmapAnnotations`. |
| `scale` | Forest-specific native `AxisScale`. |
| `spacing` | Bar/HorizontalBar native `BarSpacing`. |
| `bins`, `stat`, `composite` | Histogram native bin selection, statistical normalization and composite. |
| `reference_line` | Qq's native `QqLine`; Coefficient's native numeric reference. These are different exact-family methods. |
| `style` | LinePath native `LinePathStyle`. |
| `connectors`, `styles` | Interaction native visibility and `InteractionStyle` sequence. |
| `total_format`, `total_labels` | Waterfall native numeric presentation and labels. |
| `x_tick_text_layout`, `y_tick_text_layout`, `facet_row_text_layout`, `facet_column_text_layout`, `estimate_detail_text_layout`, `significance_text_layout`, `guide_text_layout` | Forward native `TextLayout` and role-specific capability bounds, including the macro-generated methods. |
| `matrix_row_text_layout`, `matrix_column_text_layout`, `value_text_layout`, `null_row_text_layout`, `total_text_layout` | Forward the corresponding Matrix, Heatmap and Waterfall exact-family text owners. |
| `kind`, `erase`, `clone` | Preserve native kind and clone semantics; erasure returns the native erased Plot as specified above. |
| `svg`, `svg_preserve_text`, `png`, `png_preview`, `png_with_options` | Delegate the actual native terminal behavior and return native `Result<Output>`. |

Native `PointLayer` and `BarPointLayer` remain fully accessible. PointLayerAdapter
forwards `color`, `radius`, `alpha`, `legend_label`, `label_text_layout`,
`get_color`, `get_radius`, `get_alpha`, and adapts `labels`. BarPointLayerAdapter
preserves its native `category_values`, `series_values`, `values`, `color`, and
`radius` accessors, plus its adapter-bound `series`. Native BarPointLayer has no
series setter: the adapter must prepare/reconstruct the native layer through its
public constructor and getters, or retain prepared components, without accessing
private fields. Every fallible preparation occurs before an infallible layer
handoff. Point-layer x/y and labels remain independent of the primary line data.

### Auxiliary Input Constructors

These are distinct contracts, not a blanket requirement to duplicate every
method. Plain `new` constructs an actual adapter-owned object; plain semantic
operation names express independent scientific choices. `_from_data` identifies
an explicit dataset argument, not a universal source lifetime or a claim that
other inputs are not data. Do not retain old wrappers, factories or aliases.

| Surface | Direct/default input | Explicit dataset | Result and source context |
| --- | --- | --- | --- |
| Each of 25 exact-family frontends | `Family::new(...)` | `Family::from_data(source, ...)` | `std::result::Result<Self, AdapterError>`; the data form retains its own source for subsequent field bindings. |
| `PointLayerAdapter` | `PointLayerAdapter::new(x, y)` | `PointLayerAdapter::from_data(source, x, y)` | `std::result::Result<Self, AdapterError>`; only the data form retains the layer's source for later labels. |
| `BarPointLayerAdapter` | `BarPointLayerAdapter::new(category, value, color, radius)` | `BarPointLayerAdapter::from_data(source, category, value, color, radius)` | `std::result::Result<Self, AdapterError>`; only the data form retains the layer's source for later series binding. |
| Significance result-table binding | `plot.significance_pairwise(...)` / `significance_attached(...)` | Corresponding `_from_data(source, ...)` methods. | The adapter binds directly and appends native Significance values. Original `plot.significance(native_items)` is preserved. |
| Alpha keyed / bins | `line_alpha_keyed(...)`, `fill_alpha_bins(...)` and admitted target variants. | Corresponding `_from_data(source, ...)` methods. | Bind keys, alpha values and intervals directly; an independent table is resolved at the call and not retained. |
| Semantic modes | Family methods such as Heatmap `records`, Bar `estimates`, Line `indexed` and their `_from_data` forms. | The named operation makes grid/record, sample/estimate or omitted-coordinate meaning explicit. | Retain native validation and scientific behavior without public input-wrapper types. |

When `new` is used on a plot or layer, direct inputs and their required shape/
origin metadata are bound without installing a shared lookup source. The actual
values may have come from a table. When an explicit data context is present,
source selectors and direct inputs can be mixed. An unsuffixed adapter setter can
resolve a selector in its receiver's existing source; a `_from_data` method instead
binds the explicit independent source for that operation.
The original native Plot/layer constructors, significance types and methods keep
their unchanged definitions and signatures.

## Significance And Independent Result Tables

Core `Significance<Attached>` represents a mark at one target;
`Significance<Pairwise>` represents a bracket between two targets. Targets can
be a category or a category-plus-series identity, and optional `FacetCell`
selects a single or grid facet. These are raw target identities, not column
selectors or already-formatted display labels. Native validation checks p-value,
target and facet applicability at its established time. The renderer does not
calculate hypothesis tests or adjust p-values.

Native annotations pass straight through an adapter Plot without a new wrapper:

```rust
use lichtung_plot_polars as pp;

let comparison = pp::Significance::between(
    "control", "treated", pp::SignificanceMark::PValue(0.006),
);
let plot = pp::Boxplot::from_data(&observations, "group", "value")?
    .significance([comparison])
    .title("Treatment comparison");
```

For foreign results, adapter Plot methods bind data directly, without a parallel
annotation descriptor or factory type:

```rust
let plot = pp::Boxplot::from_data(&observations, "group", "value")?
    .significance_pairwise_from_data(&test_results, "left", "right", "adjusted_p")?;

let direct = nd::Boxplot::new(groups.view(), observations.view())?
    .significance_pairwise(left.view(), right.view(), p_values.view())?;
```

`significance_pairwise(left, right, marks)` and
`significance_attached(target, marks)` accept direct foreign variables and may
resolve selectors against the receiver's existing source. Their
`_from_data(source, ...)` counterparts resolve selectors against that explicit
independent source, not the parent Plot's source. Pairwise endpoints are independent:
each target is a category binding, a `(category, series)` pair of bindings, or an explicitly typed
sequence of native `SignificanceTarget`; the two endpoints may use different
shapes. Bare numeric mark bindings represent supplied p-values. A typed native
`SignificanceMark` sequence is forwarded as marks; a scalar native mark explicitly
broadcasts. `(categorical_binding, mapping)` supplies an explicit mapping to
native marks, rejecting missing/unmapped values; arbitrary strings are never
guessed as stars. A typed native target scalar explicitly broadcasts. All
non-scalar inputs must agree on annotation count, independently of the parent
plot. A native scalar broadcasts to that count; if all inputs are scalars, they
form one annotation. Null targets or marks cannot construct native items and fail at binding. Plain
strings in data roles are selectors, not constant target identities.

`significance_pairwise_in_facet(left, right, marks, facet)` and
`significance_attached_in_facet(target, marks, facet)` add one facet-level
binding. Their `_in_grid` counterparts add `facet_row, facet_column`; every
operation also has an `_from_data(source, ...)` form. Facet bindings are direct
variables or source selectors as appropriate; an explicit native `FacetCell`
in the facet argument can represent a constant single or grid target facet.
Selectors refer to source columns only in these data roles. Native
`Significance::between("control", "treated", mark)`
continues to interpret its strings as literal target identities.

For genuinely reusable results, adapter root functions with the same semantic
operation names and arguments, e.g. `significance_pairwise_from_data(source,
left, right, marks)`, return `std::result::Result<Vec<Significance<Pairwise>>,
AdapterError>`. Attached, direct, single-facet and grid-facet forms return the
corresponding native collection. A root function without `_from_data` has no plot
context and accepts direct inputs, not unresolved selectors. These functions
are not compatibility aliases or prerequisites for the Plot methods: they let a
caller bind once and reuse the same native items across different plots, including
native plots. They share binding logic with the methods and retain no foreign source:

```rust
let comparisons = pp::significance_pairwise_from_data(
    &test_results, "left", "right", "adjusted_p",
)?;
let first = pp::Boxplot::from_data(&observations, "group", "value")?
    .significance(comparisons.clone());
let second = native_plot.significance(comparisons);
```

Both adapter and native `.significance(...)` append, rather than replace, their
existing annotations. Do not add PairwiseAnnotations, AttachedAnnotations,
PairwiseData, AttachedData, AnnotationTarget or AnnotationMarks wrapper/factory
types. An ndarray direct call may use separate string target views and numeric
p-value views; a homogeneous numeric ndarray table does not become a string table.

An independent result table's row count is its number of annotations, not the
main plot's observation count. Its `_from_data` operation resolves its own fields
and preserves supplied order; there is no fallback to the parent source, implicit
join, statistical inference, correction or automatic star-string interpretation.
Preserve exact facet/target and type restrictions. Binding errors are reported when reading the source;
native p-value, target and facet validation retains its core preflight timing.
Representative implementation tests must include attached versus pairwise,
category/series targets, facet ownership, foreign p-values, native annotations,
and rejected unsupported family/placement combinations without a test cross-product.

## Conversion And Consumer Boundaries

| Source | Target / consumer | Selected baseline | Core change for a shorter spelling? |
| --- | --- | --- | --- |
| Fully bound PlotAdapter | Native `Plot<K>` | Adapter implements infallible `From`; explicit `.into()` in a concrete native context. | No. |
| PlotAdapter single-image terminal | Native `Output` | Adapter terminal delegates to the native Plot and returns its `Result`. | No. |
| PlotAdapter | Native `Canvas::panel` / `panel_span` | Convert into an exact native Plot first; current arguments are concrete `Plot<K>`. | Yes, if the exact same `.panel(..., adapted)` syntax must work without a conversion. A generic acceptance protocol or new method requires separate review. |
| PlotAdapter | Optional adapter-local Canvas extension | A separately reviewed `CanvasDataExt::panel_data`/`panel_span_data` could perform the same `Into` inside an extension trait on the real native Canvas. It needs a trait import and new method names. | No, if those tradeoffs are acceptable; inferred exact-family generics must be prototyped. |
| PlotAdapter | User function/collection requiring `Plot<K>` | Explicit conversion at the application boundary. | A core change cannot alter an application-owned concrete signature; the application may choose its own generic bound. |
| Two bounds / preassembled native Bounds | Adapter Line / native Line | Adapter `.band(lower, upper)?` binds foreign inputs. Native `.band(bounds)` remains unchanged on Plot; convert explicitly if using an already assembled Bounds. No `band_bounds` compatibility method. | No. |
| PointLayerAdapter / BarPointLayerAdapter | Native layer | Binding finishes in the layer's own context; adapter implements `From` to the appropriate native layer. | No. |
| Prepared adapter layer | PlotAdapter `point_layer` | Adapter method accepts `impl Into<NativeLayer>`; already-native layers work via identity conversion. | No. |
| Prepared adapter layer | Native Plot `point_layer` | Convert explicitly to the exact native layer. | Yes, only if the unchanged native method must accept the wrapper without explicit conversion. |
| External fields/vectors | PlotAdapter `hue`, `facet`, `band`, alpha, status, IDs, highlights | Adapter-local role traits and context-aware fallible binding. | No. |
| External fields/vectors | Original native Plot `hue` and other concrete-input methods | Prepare native values outside that call, or use a PlotAdapter. | Yes, if the original native same-name method itself must directly accept foreign values/field names while retaining original use and error behavior. |
| Native significance iterator / prepared result table | Adapter or native `significance` | Native items are forwarded directly; the data helper returns the correctly typed native collection. | No. |
| Adapter binding error | Native `Error` | Keep `AdapterError` separate; preserve upstream sources and support lossless propagation from native errors into AdapterError. | A lossless shared native error representation for arbitrary foreign errors would need a deliberate core error contract change; stringifying is not equivalent. |
| Native Theme, Color, Significance, Plot, Canvas, Output and other re-exports | Either adapter path | Same definition when both adapters resolve one compatible core. | No. |

An adapter-local Canvas extension would leave the original methods unchanged:

```rust
// Optional design for review; neither trait nor methods exist yet.
use lichtung_plot_polars::CanvasDataExt as _;

let canvas = pp::Canvas::grid(1, 2)?
    .panel_data(0, 0, polars_plot)?
    .panel_data(0, 1, ndarray_plot)?;
```

Its methods would be implemented for the real native Canvas and accept any
`impl Into<Plot<K>>` with `K: kind::ExactFamily`; one imported adapter trait can
therefore accept both adapters' wrappers and native Plots when they share the core.
The method names differ because a same-named extension cannot overload the native
inherent `panel` by parameter type. Importing both adapters' equivalent extension
traits at once would require disambiguation. This remains an optional, uncompiled
convenience; verify inference, trait visibility, errors and original native calls.

For the exact direct `.panel(..., adapted)` spelling, a core-defined `IntoPlot`
trait with an associated exact family or a generic `Into<Plot<K>>` parameter are
possible designs, not decisions. Changing `panel`/`panel_span` may affect original
method signatures, type inference, explicit generic arguments, function items or
`.into()` calls. Do not claim complete native parity without actual compilation.
A separately named native consumer would leave the original signature untouched
but would change the user-facing call. Present the concrete comparison and obtain
direction before implementing a core change. Adapter-owned conversion remains the
baseline.

## Representative Complete Workflows

### Both adapter input forms and original native API

```rust
use lichtung_plot_ndarray as nd;
use lichtung_plot_polars as pp;

let polars_plot = pp::Scatter::from_data(&frame, "time", "response")?
    .hue("condition")?
    .theme(pp::Theme::science());
let polars_vectors = pp::Scatter::new(&times, &responses)?
    .hue(&conditions)?;
let ndarray_plot = nd::Scatter::new(time.view(), response.view())?
    .hue(condition.view())?;
let ndarray_table = nd::Scatter::from_data(measurements.view(), 0, 1)?
    .hue(condition.view())?;

let original = pp::Plot::scatter(
    pp::NumericValues::new("x", [0.0, 1.0]),
    pp::NumericValues::new("y", [1.0, 2.0]),
).hue(pp::CategoryValues::new("condition", ["A", "B"])?);
```

`measurements` is a numeric ndarray, and `condition` is a separate string-category
view. A numeric ndarray column is not implicitly interpreted as string hue.
PlotAdapter setters accept native values too, e.g. `.hue(native_categories)?`; that
fallible adapter method is separate from the unchanged native `.hue(...) -> Self`.
Single-image output consumes the adapter Plot directly, and later publications
reuse the same native Output:

```rust
let output = polars_vectors.svg("scatter.svg")?
    .svg_preserve_text("scatter-editable.svg")?
    .png("scatter.png")?;
```

### Independent fit and observation layer

```rust
let observations = pp::PointLayerAdapter::from_data(&observed, "time", "response")?
    .labels("sample")?
    .legend_label("Observed");

let fit = pp::Line::from_data(&fitted, "time", "estimate")?
    .band("lower", "upper")?
    .point_layer(observations)
    .title("Fit and observations");
```

The 30-row fit/bounds and 200-row observation layer may coexist. Each resolves
against its own source; the adapter layer can also be explicitly converted into
native `PointLayer` for a native Line Plot or another adapter.

### Cross-adapter composition with unchanged Canvas

```rust
let left: pp::Plot<pp::kind::Scatter> = polars_plot.into();
let right: nd::Plot<nd::kind::Scatter> = ndarray_plot.into();
let theme: nd::Theme = pp::Theme::science();

let output = pp::Canvas::grid(1, 2)?
    .theme(theme)
    .panel(0, 0, left)?
    .panel(0, 1, right)?
    .svg("comparison.svg")?;
```

`pp::Plot<K>` and `nd::Plot<K>` are identical native types when Cargo resolves
the same compatible core package. The two PlotAdapter wrappers remain distinct.
Compatible dependency requirements are part of delivery; incompatible core
versions or sources may produce distinct types.

## Evidence Required During Implementation

Use focused compile consumers and representative semantic integration tests:

- Verify complete public native paths and identity through either adapter,
  including callbacks, significance placement markers, `kind`/`stats`, native
  errors and both adapters together. Check name collisions against all root exports.
- Cover all 25 fixed-arity family shapes and their native capability mapping via
  meaningful existing consumers, without 25 mechanical rendering smoke tests.
  Include histogram/raincloud geometry, bar sample versus estimate modes, explicit
  x/many-y, row-index/column-identity wide, grid versus cell heatmaps and mixed
  source/direct inputs.
- Prototype the actual family type representation so `Family::new` returns
  `std::result::Result<Self, AdapterError>` under strict Clippy, with inferred source lifetimes and
  concrete family methods. Test direct and source-bound layer constructors on
  the real frontend types. No old wrapper/factory aliases or compatibility-only
  `band_bounds` method may remain; exercise direct semantic operations and native
  result collection reuse.
- Verify field versus direct vector versus native inputs, explicit `?` timing,
  null/dtype/shape/strides/chunks, masked tuple coherence in composite roles,
  all relevant origin units and exact IDs. Include the independent line/point-layer
  and alpha-key contexts, mapped wide-alpha declarations/callbacks and label/facet
  replacement through native and adapter methods.
- Verify native and foreign annotations, attached/pairwise family restrictions,
  category-series and faceted targets, independent result counts, source errors,
  and native validation timing without accidentally performing statistics.
- Exercise all forwarded method families and callback behavior through focused
  existing witnesses; check unsupported methods remain unavailable. Test explicit
  native conversion, exact-family preservation, source-borrow release, native
  Canvas insertion, same-output replay and cross-adapter native identity.
- If a core Canvas acceptance alternative is later authorized, first compile
  representative existing native callers including explicit generic arguments,
  function items and `.into()` inference. Report any incompatible native use.
- Respect the repository-wide test ceilings and focused-test policy. Code changes
  require strict workspace Clippy (`cargo clippy --locked --offline --workspace
  --all-targets -- -D warnings`); planning-only documentation does not require
  compiling proposed snippets or running tests. Verify documentation references
  and `git diff --check` without presenting hypothetical examples as executable.

[coherence]: https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence
