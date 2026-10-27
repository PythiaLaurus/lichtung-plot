# Adapter API Compatibility And Native Handoffs

## Status And Scope

This is the compatibility inventory for the proposed
[data-source plotting interface](92_DATA_ADAPTERS_PLAN.md), not an implemented
API. The two current adapter packages still expose conversion-only functions;
all `Scatter::new`/`from_data`, DataPlot, descriptor, annotation-data and
adapter-layer examples below are prospective and have not been compiled. The core definitions
described here are established by the current source. Do not add a direct core
dependency to examples that are intended to demonstrate adapter-only use.

The selected architecture places data lookup, conversion, origin tracking and
configuration forwarding in the adapters. Both adapters expose 25 top-level
families with `new` and `from_data`, returning an adapter-owned exact-family
DataPlot as `DataResult<Self>`. `new` binds direct inputs, including matrices;
`from_data` additionally binds a dataset for field lookup. Both are data inputs,
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
family constructor's result is a different, adapter-owned DataPlot.

## Adapter Plot Handoff

Each DataPlot owns its actual native `Plot<K>`, with the same exact family marker.
`from_data` additionally retains its own source reference and origin/shape metadata;
`new` retains required metadata but does not retain direct input views
after the corresponding binding. Public adapter types have private fields and may
be named by callers. DataPlot `Clone` preserves native clone behavior, shares or
clones its read-only context safely, and never changes source membership. Any
clones retaining the source keep their own live borrow until consumed or dropped.

For Rust's `new` convention and strict Clippy, the family entry point denotes the
actual exact-family frontend: `new` and `from_data` return `DataResult<Self>`,
not a distinct factory's unrelated result. An alias such as
`Scatter<'source> = DataPlot<'source, kind::Scatter>` is a candidate, subject
to a prototype for inference, source lifetimes and the documented direct input
shapes. `DataPointLayer` and `DataBarPointLayer` likewise own their constructors.
Do not keep a second factory or old constructor as an alias; do not suppress
`new_ret_no_self` merely to retain an unrelated factory return type.

The adapter implements `From<DataPlot<..., K>> for Plot<K>`. Binding methods
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
it does not change the target family's scientific capabilities. `DataPlot::erase`
for an exact family delegates to core erasure, releases the context and returns
the native `Plot<kind::Any>`; that erased value remains ineligible for Canvas
`panel` and `panel_span`, exactly as in the core. `kind()` is forwarded without
erasure. Source-independent `.svg`, `.svg_preserve_text`, `.png`, `.png_preview`
and `.png_with_options` consume DataPlot, delegate to native Plot and return the
native `Result<Output>`; the caller need not request a conversion first.

## Chainable Data Bindings

The adapter defines local, role-specific input protocols. On the adapter receiver,
the following fallible binding methods return `DataResult<Self>` for both native
and foreign inputs. The unchanged native methods still have their native return
types when the receiver is `Plot<K>`. A source selector on a plot created by `new`
without an appropriate source is an error. String field selectors, literal labels and
significance target values are interpreted by their distinct argument roles.

| Adapter method | Accepted input / conversion | Native capability and constraints |
| --- | --- | --- |
| `hue` | Source field/index, admitted category vector/view or native `CategoryValues`; bind to native category values. | `K: kind::Hue`; categorical identity, not numeric continuous color. |
| `group` | Same category forms; convert using the graphical-group role. | `K: kind::Group`; does not replace a constructor-owned group where the core lacks this setter. |
| `series` | Same category forms; convert using the positioned-series role. | `K: kind::Series`; native supplied-geometry restrictions remain. |
| `source_rows` | Exact integer field/view, per-cell `Cells(...)`, or native `SourceRows`. | `K: kind::SourceRows`; non-null, nonnegative, fitting and unique IDs; never repeat one ID across expanded cells. |
| `facet` | `Facet::row`, `column`, `grid`, `wrap` with field/vector bindings, or native `FacetSpec`. | `K: kind::Facet`; preserve row/column/wrap distinction, scale policy and native facet restrictions. |
| Line `band` | `Interval::new(lower, upper)` with independently supplied bindings, or native `Bounds`. | Only exact Line; many-y bounds require matching selected-column collections or matching shapes, with no implicit pair reuse. |
| `heatmap_highlight` | Boolean record vector/field, same-shape grid, explicit `Cells(...)`, or native `BooleanValues`. | Heatmap only; true adds a border, independently of validity. |
| `estimate_status` | Explicit `Status::boolean(binding)`, `Status::categorical(binding, mapping)`, or native `EstimateStatus` iterator. | PointInterval, Coefficient and Forest; never infer status from missing numeric values. |
| `point_alpha`, `line_alpha`, `fill_alpha` | `Alpha` plus `AlphaMap` descriptors, independently prepared adapter alpha, or native `AlphaEncoding`. | Corresponding `kind::PointAlpha`, `LineAlpha`, `FillAlpha`; retain observation, Single, keyed series/aggregate or bin units as admitted by the core. |
| DataPointLayer `labels` | Layer-local field/index, direct category vector or native `CategoryValues`. | Labels align with that independent layer, not the parent Plot. |
| DataBarPointLayer `series` | Layer-local field/index, direct category vector or native `CategoryValues`. | Optional positioned-series identity resolves against the parent bar's prepared cells. |

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

An already-native input is forwarded unchanged according to native slot semantics;
the adapter does not silently reinterpret it as a source-row vector or expand it.
Foreign row, selected-column and `Cells(...)` bindings follow their explicit owning
operation's origin map. `Into` alone cannot provide the retained source or decide
whether an input is row-, cell-, observation-, series- or bin-aligned.

`Interval`, `Facet`, `Alpha`, `AlphaMap`, `Key`, `Status`, `Valid`, `Cells`,
`Grid`, `HeatmapCells`, `Columns`, `RowIndex`, `ColumnIdentity`, `Axis`,
`HistogramData`, `RaincloudData`, `BarData`/`BarSampleData` and `CoefficientData`
are adapter-side descriptors/markers; their definitions do not replace native
types. A descriptor containing unresolved names, symbolic plot-role identities or
fallible foreign conversions is resolved by the owning adapter operation. It can
implement a fallible conversion only when it carries all needed context. An
already materialized local value can implement `From` into its native counterpart,
but adding a public wrapper purely to expose that conversion is not required.

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

| Method(s) | DataPlot compatibility |
| --- | --- |
| `hue`, `group`, `series`, `source_rows`, `facet`, `heatmap_highlight`, `estimate_status`, `point_alpha`, `line_alpha`, `fill_alpha` | Use the role-specific adapter binding contracts above; native receiver remains unchanged. |
| `band` | Line accepts adapter Interval or native Bounds as a fallible data binding. Regression forwards native `RegressionBand` as a pure policy, with its original return shape. |
| `point_layer` | Line accepts native PointLayer or prepared DataPointLayer; Bar/HorizontalBar accept native BarPointLayer or prepared DataBarPointLayer. Adapter method consumes `impl Into<NativeLayer>` and returns Self. Native methods retain exact native arguments. |
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

Native `PointLayer` and `BarPointLayer` remain fully accessible. DataPointLayer
forwards `color`, `radius`, `alpha`, `legend_label`, `label_text_layout`,
`get_color`, `get_radius`, `get_alpha`, and adapts `labels`. DataBarPointLayer
preserves its native `category_values`, `series_values`, `values`, `color`, and
`radius` accessors, plus its adapter-bound `series`. Native BarPointLayer has no
series setter: the adapter must prepare/reconstruct the native layer through its
public constructor and getters, or retain prepared components, without accessing
private fields. Every fallible preparation occurs before an infallible layer
handoff. Point-layer x/y and labels remain independent of the primary line data.

### Auxiliary Input Constructors

These are distinct contracts, not a blanket requirement to duplicate every
method. Plain `new` is used for actual adapter-owned object construction; plain
operation names describe other actions. The `_from_data` spelling identifies an
explicit dataset argument, not a universal source lifetime or a claim that other
inputs are not data. The old `from_vectors`/factory spellings are not aliases.

| Surface | Direct/default input | Explicit dataset | Result and source context |
| --- | --- | --- | --- |
| Each of 25 exact-family frontends | `Family::new(...)` | `Family::from_data(source, ...)` | `DataResult<Self>`; the data form retains its own source for subsequent field bindings. |
| `DataPointLayer` | `DataPointLayer::new(x, y)` | `DataPointLayer::from_data(source, x, y)` | `DataResult<Self>`; only the data form retains the layer's own source for later labels. |
| `DataBarPointLayer` | `DataBarPointLayer::new(category, value, color, radius)` | `DataBarPointLayer::from_data(source, category, value, color, radius)` | `DataResult<Self>`; only the data form retains the layer's own source for later series binding. |
| `PairwiseAnnotations` / `AttachedAnnotations` | `::prepare(descriptor)` | `::from_data(source, descriptor)` | `DataResult<Vec<Significance<P>>>`; inputs are prepared eagerly and neither result retains a dataset. |
| `Alpha` keyed / bins | `Alpha::keyed(...)` / `Alpha::bins(...)` | `Alpha::keyed_from_data(source, ...)` / `Alpha::bins_from_data(source, ...)` | The unsuffixed descriptor resolves direct inputs or the owning plot's source at attachment; the `_from_data` form eagerly binds its independent table and retains no foreign table. Symbolic plot targets may still need the parent plot at attachment. |
| `Interval`, `Facet`, `Grid`, `HeatmapCells`, family input and mark descriptors | Role-specific `new`, `row`, `samples`, `geometry`, etc. | Names/indices are resolved by the owning adapter operation. | A descriptor is not a dataset binding operation; do not add `_from_data` variants mechanically. |

When `new` is used on a plot or layer, direct inputs and their required shape/
origin metadata are bound without installing a shared lookup source. The actual
values may have come from a table. When an explicit data context is present,
source selectors and direct inputs can be mixed. A plain semantic operation such
as `Alpha::keyed` can itself contain selectors resolved in an existing plot context;
the lack of a suffix does not universally mean every argument is already materialized.
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

For caller-computed tabular results, the following additive helper shapes are
planned for review. `PairwiseAnnotations::from_data(source, descriptor)` and
`::prepare(descriptor)` resolve a `PairwiseData` descriptor and return
`DataResult<Vec<Significance<Pairwise>>>`; `AttachedAnnotations` and
`AttachedData` do the same for `Attached`. They are adapter-owned names, not
replacements for native Significance. A prepared result can be passed to native
or adapter `.significance` directly because it contains the native items.
`prepare` describes producing a native collection; it does not introduce a new
annotation wrapper merely to use `new`. Both forms resolve their own inputs eagerly,
and the resulting collection retains no foreign source for future field lookups.

```rust
let comparisons = pp::PairwiseAnnotations::from_data(
    &test_results,
    pp::PairwiseData::new(
        pp::AnnotationTarget::category("left"),
        pp::AnnotationTarget::category("right"),
        pp::AnnotationMarks::p_values("adjusted_p"),
    ),
)?;

let plot = pp::Boxplot::from_data(&observations, "group", "value")?
    .significance(comparisons);
```

`PairwiseData::new(left, right, marks)` accepts two independent target
descriptors. `AttachedData::new(target, marks)` accepts one. Each target uses
`AnnotationTarget::category(binding)`, `::category_series(category, series)`,
or `::native(binding)` for an explicitly typed SignificanceTarget sequence.
The two pairwise endpoints may have different target shapes, as in the core.
Each annotation descriptor may add `facet_single(binding)` or
`facet_grid(row, column)`.
`AnnotationMarks::p_values(binding)` uses supplied numeric p-values;
`::native(binding)` accepts an explicitly typed native SignificanceMark sequence;
`::mapped(binding, explicit_mapping)` maps external categorical values to native
marks, rejecting missing or unmapped values; `::constant(mark)` is explicit
broadcast. Dataset selectors and direct vectors follow the adapter's normal role
grammar. The ndarray direct form can use separate string target views and numeric
p-value views; a homogeneous numeric ndarray table does not become a string table.
For example:

```rust
let comparisons = nd::PairwiseAnnotations::prepare(
    nd::PairwiseData::new(
        nd::AnnotationTarget::category(left.view()),
        nd::AnnotationTarget::category(right.view()),
        nd::AnnotationMarks::p_values(p_values.view()),
    ),
)?;
let plot = nd::Boxplot::new(groups.view(), observations.view())?
    .significance(comparisons);
```

`AnnotationMarks::native` accepts an actual native-mark sequence or typed ndarray
view; it does not imply that Polars stores arbitrary Rust enums in a column.

The same direct/result-table distinction applies to attached marks:

```rust
let attached = pp::AttachedAnnotations::prepare(pp::AttachedData::new(
    pp::AnnotationTarget::category(&target_series),
    pp::AnnotationMarks::p_values(&p_value_series),
))?;

let attached_from_table = pp::AttachedAnnotations::from_data(
    &test_results,
    pp::AttachedData::new(
        pp::AnnotationTarget::category("target"),
        pp::AnnotationMarks::p_values("adjusted_p"),
    ),
)?;
```

Both variables contain native `Significance<Attached>` values ready for an
admitted native or adapter Plot. Their independent inputs need not have the same
length as the observations used to construct that Plot.

The independent result table's row count is its number of annotations, not the
main plot's observation count. Resolve its fields and preserve supplied order;
no parent-source lookup, implicit join, statistical inference, correction or
automatic star-string interpretation is introduced. Preserve exact facet/target
and type restrictions. Binding errors are reported when reading the source;
native p-value, target and facet validation retains its core preflight timing.
Representative implementation tests must include attached versus pairwise,
category/series targets, facet ownership, foreign p-values, native annotations,
and rejected unsupported family/placement combinations without a test cross-product.

## Conversion And Consumer Boundaries

| Source | Target / consumer | Selected baseline | Core change for a shorter spelling? |
| --- | --- | --- | --- |
| Fully bound DataPlot | Native `Plot<K>` | Adapter implements infallible `From`; explicit `.into()` in a concrete native context. | No. |
| DataPlot single-image terminal | Native `Output` | Adapter terminal delegates to the native Plot and returns its `Result`. | No. |
| DataPlot | Native `Canvas::panel` / `panel_span` | Convert into an exact native Plot first; current arguments are concrete `Plot<K>`. | Yes, if the exact same `.panel(..., adapted)` syntax must work without a conversion. A generic acceptance protocol or new method requires separate review. |
| DataPlot | Optional adapter-local Canvas extension | A separately reviewed `CanvasDataExt::panel_data`/`panel_span_data` could perform the same `Into` inside an extension trait on the real native Canvas. It needs a trait import and new method names. | No, if those tradeoffs are acceptable; inferred exact-family generics must be prototyped. |
| DataPlot | User function/collection requiring `Plot<K>` | Explicit conversion at the application boundary. | A core change cannot alter an application-owned concrete signature; the application may choose its own generic bound. |
| DataPointLayer / DataBarPointLayer | Native layer | Binding finishes in the layer's own context; adapter implements `From` to the appropriate native layer. | No. |
| Prepared adapter layer | Adapter DataPlot `point_layer` | Adapter method accepts `impl Into<NativeLayer>`; already-native layers work via identity conversion. | No. |
| Prepared adapter layer | Native Plot `point_layer` | Convert explicitly to the exact native layer. | Yes, only if the unchanged native method must accept the wrapper without explicit conversion. |
| External fields/vectors | DataPlot `hue`, `facet`, `band`, alpha, status, IDs, highlights | Adapter-local role traits and context-aware fallible binding. | No. |
| External fields/vectors | Original native Plot `hue` and other concrete-input methods | Prepare native values outside that call, or use an adapter DataPlot. | Yes, if the original native same-name method itself must directly accept foreign values/field names while retaining original use and error behavior. |
| Native significance iterator / prepared result table | Adapter or native `significance` | Native items are forwarded directly; the data helper returns the correctly typed native collection. | No. |
| Adapter binding error | Native `Error` | Keep `DataError` separate; preserve upstream sources and support lossless native-error-to-DataError propagation. | A lossless shared native error representation for arbitrary foreign errors would need a deliberate core error contract change; stringifying is not equivalent. |
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
DataPlot setters accept native values too, e.g. `.hue(native_categories)?`; that
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
let observations = pp::DataPointLayer::from_data(&observed, "time", "response")?
    .labels("sample")?
    .legend_label("Observed");

let fit = pp::Line::from_data(&fitted, "time", "estimate")?
    .band(pp::Interval::new("lower", "upper"))?
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
the same compatible core package. The two DataPlot wrappers remain distinct.
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
  `DataResult<Self>` under strict Clippy, with inferred source lifetimes and
  concrete family methods. Test direct and source-bound layer constructors on
  the real frontend types. No old `from_vectors` or separate layer factory alias
  may remain; distinguish `new` from the annotation `prepare` collection operation.
- Verify field versus direct vector versus native inputs, explicit `?` timing,
  null/dtype/shape/strides/chunks, all relevant origin units and exact IDs.
  Include the independent line/point-layer and alpha-key contexts.
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
