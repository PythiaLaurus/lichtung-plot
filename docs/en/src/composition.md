# Facets and Canvas

[简体中文](../zh-CN/composition.html)

Facets split one scientific plot; Canvas owns separately authored Plot/TextPanel
owners. FacetSpec::row/column/grid/wrap attaches a complete layout; wrap requires
positive column count. Facets retain the owning Plot's theme and do not introduce
a new scope. Generic faceted plots cannot be embedded in Canvas. Matrix owns a
native variable grid; Marginal has its own nongeneric marginal layout.

Canvas::grid creates declared row-major topology. `panel` consumes an exact Plot
and erases its marker at heterogeneous storage, without weakening validation.
`panel_span(row,column,row_span,column_span,plot)` owns one positive rectangle,
anchored at its top-left. Covered slots are occupied, not additional producers.
Plot/TextPanel overlaps fail in either insertion order; range groups name anchors.

```rust
{{#include ../../../crates/lichtung-plot/examples/11_canvas_spans.rs:example}}
```

A spanning Plot has one continuous body and one set of axes/prose/guides.
Row titles cannot cut through its footprint. Shared categorical row bands require
matching vertical footprints and complete category/selection policies;
FirstColumnPerRow does not guess correspondence between unequal bands. Parent
geometry overrides embedded standalone sizing requests; declared tracks still
position headers independently of owner rectangles.

Canvas root defaults to Publication. Unthemed children inherit it; an explicit
child theme replaces it completely. Root prose, headers, TextPanel and Shared-guide
chrome use Canvas appearance, while scientific child marks/axes use Plot appearance.

```rust
{{#include ../../../crates/lichtung-plot/examples/04_canvas_themes.rs:example}}
```

Shared ranges and Shared guides are distinct relationships. Matching ranges need
not imply equal tick-generation or formatting choices. Shared guides retain raw
identity/paint compatibility and use Canvas composition; conflicting aliases or
styles fail rather than being merged heuristically. See [themes](themes.md).

Configure exact children before insertion. Canvas is not Clone, and Rust's
consuming methods are ownership transfers, not Python mutation semantics.
