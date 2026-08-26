# Visual Property Ownership Ledger

**Status**: Canonical T2 migration ledger; T2a and T2b COMPLETE

This ledger records current production sources and approved target ownership. It
does not make the target APIs in `03_THEME_SYSTEM.md` current, expose them early,
or supersede the implemented contracts in `01_ARCHITECTURE.md` and
`02_RENDERING_CONTRACT.md`. Target names below are migration assignments only
until Phase T6 performs the atomic public migration.

The ledger is append-only during T2 except for corrections and T2h
reconciliation. Tests, examples, and snapshots are evidence consumers, never
independent property sources.

## Progress

| Phase | Scope | Status |
| --- | --- | --- |
| T2a | Ledger protocol and renderer foundations | COMPLETE |
| T2a1 | Current theme and font foundations | COMPLETE |
| T2a2 | Private style carriers and backend protocol | COMPLETE |
| T2a3 | Serialization, geometry recording, and scene conversion | COMPLETE |
| T2a4 | Lifecycle, output, and artifact foundations | COMPLETE |
| T2b | Public controls and policies | COMPLETE |
| T2c | Layout, typography, axes, and grid | PENDING |
| T2d | Legends, Canvas composition, and decorations | PENDING |
| T2e | Ordinary plot marks and local styles | PENDING |
| T2f | Specialized marks and annotations | PENDING |
| T2g | Data scales and semantic domains | PENDING |
| T2h | Cross-source reconciliation and migration map | PENDING |

## Record Protocol

One property record may be physically split across the keyed current-state,
target-state, and migration/evidence tables below. The same unique `Property`
key joins every required field from the T2 schema in
`91_IMPLEMENTATION_PLAN.md`: source and value, consumers, current behavior,
layout path, semantic role, target class and owner, target resolution, target
validation, public surface, relationships, audit evidence, disposition and
phase, and visual evidence. A key present in only some physical tables is an
incomplete record.

The closed target classes are exactly `Theme component`, `Theme data scale`,
`Plot-local data scale`, `Exact element style`, `Explicit policy`, and
`Renderer invariant`. Completed T2a sections use no unresolved values and
introduce no future public names beyond the T1 types.

The named relationship set `theme-preset-property-keys` contains exactly these
property keys:

`color-scales.qualitative.current-theme-colors`,
`marks.accent.current-theme-colors`,
`typography.plot.font-stack.current-palette-field`,
`typography.annotation.font-stack.current-palette-field`,
`typography.legend-guide.font-stack.current-palette-field`,
`typography.facet-header.font-stack.current-palette-field`,
`typography.canvas-foundation.font-stack.current-palette-field`,
`typography.canvas-composition.font-stack.current-palette-field`,
`surface.canvas.current-background`,
`surface.panel.current-background`,
`axes.chrome.current-foreground`,
`typography.plot.current-foreground-text`,
`typography.annotation.current-foreground`,
`typography.legend-guide.current-foreground-text`,
`typography.facet-header.current-foreground-text`,
`typography.canvas-foundation.current-foreground-text`,
`typography.canvas-composition.current-foreground-text`,
`marks.neutral.current-foreground`,
`annotations.stroke.current-foreground`,
`axes.stroke-width.current-axis-width`,
`marks.interval-width.current-axis-width`,
`marks.general-line-width.current-dormant-field`,
`annotations.significance-bracket-width.current-field`,
`axes.frame-sides.current-show-top-right-spines`,
`axes.frame.current-show-frame`,
`axes.grid.current-show-grid`,
`axes.ticks.current-show-ticks`,
`axes.x-tick-label-presence.current-theme-flag`,
`axes.y-tick-label-presence.current-theme-flag`,
`axes.x-title-presence.current-theme-flag`,
`axes.y-title-presence.current-theme-flag`,
`content.title-presence.current-theme-flag`,
`content.subtitle-presence.current-theme-flag`,
`content.caption-presence.current-theme-flag`,
`axes.tick-length.current-field`,
`typography.tick-label-size.current-palette-field`,
`typography.detail-status-size.current-field`,
`typography.annotation-size.current-detail-field`,
`typography.legend-detail-size.current-detail-field`,
`typography.axis-title-size.current-label-field`,
`typography.body-size.current-label-field`,
`typography.plot-title-size.current-field`,
`typography.canvas-title-size.current-title-field`,
`typography.canvas-composition-title-size.current-title-field`,
`typography.plot-subtitle-size.current-field`,
`typography.facet-header-size.current-subtitle-field`,
`typography.canvas-subtitle-size.current-field`,
`typography.plot-caption-size.current-field`,
`typography.canvas-caption-size.current-field`,
`axes.grid-color.current-field`, and
`axes.grid-width.current-field`.

## T2a1 Source Manifest

| ID | Production source | Owned context | Audit disposition |
| --- | --- | --- | --- |
| M1 | `src/theme.rs:1-6` | Module boundary and re-exports | `Theme` is public; `ThemePalette`, `TICK_FONT_SIZE`, and `rgb_to_hex` are private plumbing. No independent visual literal. |
| M2 | `src/theme/palette.rs:9-40` | Tick-size constant, public preset enum generation, private preset routing | Records `preset.routing.current-theme-enum` and `typography.tick-label-size.current-global-constant`. |
| M3 | `src/theme/palette.rs:42-124` | Every `ThemePalette` field responsibility | Records from `color-scales.qualitative.current-theme-colors` through `axes.grid-width.current-field`. |
| M4 | `src/theme/palette.rs:126-130` | RGB-to-SVG conversion | Record `backend.svg.rgb-hex-serialization`. |
| M5 | `src/theme/palette.rs:132-173` | Publication preset construction | Every initializer is admitted through the keyed records and preset reference `PV`. |
| M6 | `src/theme/palette.rs:175-212` | Nature preset construction | Every initializer is admitted through the keyed records and preset reference `PV`. |
| M7 | `src/theme/palette.rs:214-251` | Science preset construction | Every initializer is admitted through the keyed records and preset reference `PV`. |
| M8 | `src/theme/palette.rs:253-290` | CHI preset construction | Every initializer is admitted through the keyed records and preset reference `PV`. |
| M9 | `src/theme/palette.rs:292-378` | Test module | Excluded as a property source; retained as evidence only. |
| M10 | `src/font.rs:3-17` | Public font family constants and ordered family list | Records from `typography.builtin-font-stack.current-public-constant` through `font-resolution.current-global-family-order`. |
| M11 | `src/font.rs:19-50` | Host-font discovery, selection, fallback, and errors | Records from `font-resolution.load-system-fonts` through `font-resolution.global-sans-serif-remap`. |

The manifest is exact for T2a1. Production consumers outside these three source
files are traces, not property sources owned by this subphase.

## Preset Value Reference

`P`, `N`, `S`, and `C` mean Publication, Nature, Science, and CHI. RGB values are
opaque `(u8, u8, u8)` tuples. Widths and lengths are current SVG/CSS pixels.
Font-size documentation says points, but current layout treats each `u32` as a
pixel-like value and SVG emits an unqualified `font-size`; the records preserve
that current unit mismatch.

| Ref | Field and exact source | Current type/unit | P | N | S | C |
| --- | --- | --- | --- | --- | --- | --- |
| PV.colors | `ThemePalette::colors`; `palette.rs:59,135-144,177-183,216-222,255-261` | `Vec<(u8,u8,u8)>`, opaque RGB | `[(0,114,178),(213,94,0),(0,158,115),(204,121,167),(86,180,233),(230,159,0),(240,228,66),(0,0,0)]` | `[(40,86,130),(130,160,188),(200,215,230),(120,130,90),(180,175,130)]` | `[(255,0,0),(0,0,255),(0,175,0),(255,175,0),(160,32,240)]` | `[(171,199,232),(243,169,175),(174,224,174),(244,213,152),(197,176,213)]` |
| PV.font | `ThemePalette::font_family`; `palette.rs:61,147,186,225,264` | `String`, CSS family expression | `FSTACK` | `FSTACK` | `FSTACK` | `FSTACK` |
| PV.background | `ThemePalette::background`; `palette.rs:63,148,187,226,265` | `(u8,u8,u8)`, opaque RGB | `(255,255,255)` | `(255,255,255)` | `(255,255,255)` | `(255,255,255)` |
| PV.foreground | `ThemePalette::foreground`; `palette.rs:65,149,188,227,266` | `(u8,u8,u8)`, opaque RGB | `(0,0,0)` | `(40,40,40)` | `(0,0,0)` | `(60,60,60)` |
| PV.axis-width | `ThemePalette::axis_width`; `palette.rs:67,150,189,228,267` | `f32`, px | `0.8` | `0.8` | `1.0` | `0.8` |
| PV.line-width | `ThemePalette::line_width`; `palette.rs:69,151,190,229,268` | `f32`, px | `1.5` | `1.0` | `1.5` | `1.5` |
| PV.bracket-width | `ThemePalette::bracket_width`; `palette.rs:71,152,191,230,269` | `f32`, px | `1.0` | `0.8` | `1.0` | `0.8` |
| PV.top-right | `ThemePalette::show_top_right_spines`; `palette.rs:76,153,192,231,270` | `bool` | `false` | `false` | `true` | `false` |
| PV.show-frame | `ThemePalette::show_frame`; `palette.rs:80,154,193,232,271` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-grid | `ThemePalette::show_grid`; `palette.rs:83,155,194,233,272` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-ticks | `ThemePalette::show_ticks`; `palette.rs:86,156,195,234,273` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-tick-labels | `ThemePalette::show_tick_labels`; `palette.rs:89,157,196,235,274` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-x-label | `ThemePalette::show_x_label`; `palette.rs:92,158,197,236,275` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-y-label | `ThemePalette::show_y_label`; `palette.rs:96,159,198,237,276` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-title | `ThemePalette::show_title`; `palette.rs:99,160,199,238,277` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-subtitle | `ThemePalette::show_subtitle`; `palette.rs:102,161,200,239,278` | `bool` | `true` | `true` | `true` | `true` |
| PV.show-caption | `ThemePalette::show_caption`; `palette.rs:105,162,201,240,279` | `bool` | `true` | `true` | `true` | `true` |
| PV.tick-length | `ThemePalette::tick_length`; `palette.rs:107,163,202,241,280` | `u32`, px | `5` | `5` | `5` | `5` |
| PV.tick-size | `ThemePalette::tick_font_size`; `palette.rs:109,164,203,242,281` | `u32`, current pixel-like font unit | `16` | `16` | `16` | `16` |
| PV.detail-size | `ThemePalette::detail_font_size`; `palette.rs:111,165,204,243,282` | `u32`, current pixel-like font unit | `14` | `14` | `14` | `14` |
| PV.label-size | `ThemePalette::label_font_size`; `palette.rs:113,166,205,244,283` | `u32`, current pixel-like font unit | `17` | `16` | `17` | `17` |
| PV.title-size | `ThemePalette::title_font_size`; `palette.rs:115,167,206,245,284` | `u32`, current pixel-like font unit | `20` | `19` | `21` | `19` |
| PV.subtitle-size | `ThemePalette::subtitle_font_size`; `palette.rs:117,168,207,246,285` | `u32`, current pixel-like font unit | `15` | `14` | `16` | `15` |
| PV.caption-size | `ThemePalette::caption_font_size`; `palette.rs:119,169,208,247,286` | `u32`, current pixel-like font unit | `15` | `14` | `15` | `15` |
| PV.grid-color | `ThemePalette::grid_color`; `palette.rs:121,170,209,248,287` | `(u8,u8,u8)`, opaque RGB | `(220,220,220)` | `(230,230,230)` | `(200,200,200)` | `(220,220,220)` |
| PV.grid-width | `ThemePalette::grid_width`; `palette.rs:123,171,210,249,288` | `f32`, px | `0.5` | `0.5` | `0.5` | `0.5` |

`FSTACK` is exactly `"Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif"` from `src/font.rs:7-8`.

## Current Resolution And Consumer References

The compact codes in the record table expand as follows and are part of each
record that cites them.

| Code | Exact current behavior or production consumers |
| --- | --- |
| B-PAL | `Plot` stores mandatory `Theme`, defaults to `Theme::Publication` at `builder.rs:455,542`, and replaces it through `builder/configuration.rs:321-325`. `Plot::palette()` at `builder/layout.rs:8-13` rebuilds an owned `ThemePalette`, including `Vec` and `String`, on every read. Canvas has no theme and reconstructs Publication at `canvas.rs:584,935`; headers/row titles may instead use the first Plot palette at `canvas.rs:657-662,936-941`. No palette construction validation or fallback exists. |
| B-TEXT | Deterministic measurement consumes text and numeric size, not family or installed faces. Preserve-text SVG emits the family expression. Outlined SVG and PNG resolve host fonts only after canonical SVG layout. `TextStyle` validates rotation only, not family or size. |
| B-WIDTH | A used line/bracket width is checked only at the backend boundary for finite nonnegative values. Palette construction and dormant widths are unchecked. |
| C-route | Direct production consumers are `theme.rs:5`, `builder/layout.rs:11-13`, mandatory/default/setter storage at `builder.rs:455,542` and `builder/configuration.rs:323-325`, and hidden Canvas calls at `canvas.rs:584,935`; all field consumers below are transitive consumers. |
| C-colors | `builder/accessors.rs:642-657`; `plots/bar_data.rs:641-659`; `bar.rs:153`; `boxplot.rs:122,172,264`; `density.rs:70,104`; `horizontal_bar.rs:149-156`; `line.rs:137,200,362-363,411,444`; `marginal.rs:415`; `matrix.rs:649`; `scatter.rs:82,97`; `violin.rs:470`; `violin/precomputed.rs:206`; `histogram.rs:74,148`; `interval_estimate.rs:511`; `point_interval.rs:150`; `qq.rs:76`; `regression.rs:95,122-123`; `waterfall.rs:127-129`; generated mark and legend styles consume the selected colors. |
| C-font-plot | Foundational Plot text only: axis titles/ticks at `layout/panel/axis_frame_grid.rs:133,141`; Plot title block through `decorations/apply.rs:44` and `decorations/title_block.rs:104-166`; matrix ticks/axes/title at `plots/matrix.rs:185,193,476`; composed Plot panel title/subtitle at `canvas.rs:1269,1277`; shared facet axes at `orchestrator/facets/rendering.rs:147`. All reach final SVG text serialization at `render/svg_backend.rs:612-661`. |
| C-font-annotation | Annotation text only: `significance.rs:868`; `decorations/panel_text.rs:35`; `plots/bar.rs:248`; `heatmap.rs:665,718`; `horizontal_bar.rs:264`; `interval_estimate.rs:201,569`; `waterfall.rs:166`. Bracket labels reach `render/svg_backend.rs:675`; other text reaches `render/svg_backend.rs:647,657`. |
| C-font-legend-guide | Continuous color-guide text only: `plots/beeswarm.rs:201` and `heatmap.rs:812`, reaching final SVG text serialization. |
| C-font-facet-header | Facet strip/header text only: `orchestrator/facets/rendering.rs:114`. Shared facet axes are in C-font-plot. |
| C-font-canvas-foundation | Canvas root title/subtitle/caption only: `canvas.rs:1010` passes hidden-Publication family to `TitleBlock`, which draws through `decorations/title_block.rs:104-166`. |
| C-font-canvas-composition | Canvas-owned shared headers, row titles, and text panels only: `canvas.rs:1121,1155,1191,1199`. Composed Plot panel headers are in C-font-plot. |
| C-bg | No `ThemePalette::background` production read. The current root counterpart is hardcoded `#FFFFFF` at `render/svg_backend.rs:731`; composed panels do not paint their resolved background. |
| C-fg-axis | Axis/tick/frame paint only: `layout/panel/axis_frame_grid.rs:127,357`; `plots/matrix.rs:174,179`; marginal-axis ticks at `plots/marginal.rs:160`. Text in those renderers is exclusively C-fg-plot-text. |
| C-fg-foundation-titleblock | The shared read at `decorations/title_block.rs:106` has disjoint call-context ownership: Plot entry through `decorations/apply.rs:44` belongs to C-fg-plot-text, while Canvas-root entry through `canvas.rs:1010` belongs to C-fg-canvas-foundation. |
| C-fg-plot-text | Foundational Plot text only: axis/tick text at `layout/panel/axis_frame_grid.rs:132,140`; matrix axis text at `plots/matrix.rs:184,192`; composed Plot panel title at `canvas.rs:1276`; shared facet axes at `orchestrator/facets/rendering.rs:146`; Plot title uses `foreground` at `decorations/title_block.rs:106`, while subtitle and caption use separate hardcoded muted colors. |
| C-fg-annotation-text | Annotation text only: significance label arm of the combined carrier at `significance.rs:866`; `decorations/panel_text.rs:34`; null-row and value/status labels at `plots/heatmap.rs:717`, `bar.rs:247`, `horizontal_bar.rs:263`, `interval_estimate.rs:200,568`, and `waterfall.rs:165`. Heatmap automatic-contrast cell text reads `font_family` but not `foreground`. |
| C-fg-legend-guide | Continuous color-guide text only: `plots/beeswarm.rs:200` and `heatmap.rs:811`. |
| C-fg-facet-header | Facet strip/header text only: `orchestrator/facets/rendering.rs:113`. Shared facet axes remain exclusively C-fg-plot-text. |
| C-fg-canvas-foundation | Canvas root title uses the hidden-Publication `foreground` through `canvas.rs:1010` and `decorations/title_block.rs:106`; subtitle and caption use separate hardcoded muted colors. |
| C-fg-canvas-composition | Canvas shared-header, row-title, and text-panel text only: `canvas.rs:1120,1154,1190,1198`. Composed Plot panel title at `canvas.rs:1276` remains exclusively C-fg-plot-text. |
| C-fg-marks | Non-text neutral marks only: `beeswarm.rs:117`; `bar_data.rs:653`; `point_interval.rs:86,96`; `dumbbell.rs:71`; `horizontal_bar.rs:137,167,174`; `histogram.rs:75,104,107,149,176,178`; `boxplot.rs:125,128,175,180,227`; `bar.rs:154,161`; `violin.rs:154,243,275,308,315,357,368`; `line.rs:351`; `waterfall.rs:140,150,159`; `regression.rs:104`; `interval_estimate.rs:492,505`; `violin/precomputed.rs:67,116,138,148`; and `qq.rs:72`. |
| C-fg-annotation-stroke | Heatmap highlight stroke at `plots/heatmap.rs:653`; the bracket-stroke arm of the combined carrier constructed at `significance.rs:866`, whose label arm belongs exclusively to C-fg-annotation-text. |
| C-axis-width | Axis/frame: `layout/panel/axis_frame_grid.rs:128,358`, `plots/matrix.rs:175,180`, `plots/marginal.rs:161`. Marks: `plots/bar.rs:162`, `boxplot.rs:125,181`, `horizontal_bar.rs:175`, `violin.rs:276`. |
| C-bracket | `significance.rs:863-875` creates `BracketStyle`; `render/svg_backend.rs:664-718` validates and draws the bracket. |
| C-top-right | `layout/panel/axis_frame_grid.rs:149-158`; the branch named for top/right sides actually draws bottom/left. |
| C-frame | `layout/panel/axis_frame_grid.rs:341-365`; matrix cell frames at `plots/matrix.rs:213-237`. |
| C-grid | `layout/panel/axis_frame_grid.rs:368-405`, called below marks through `decorations/apply.rs:13-24`. |
| C-ticks | Standard axis branches at `axis_frame_grid.rs:160-226`; matrix at `plots/matrix.rs:290-378`; marginal custom ticks ignore the flag. Canvas/layout still reserves tick length when marks are hidden. |
| C-x-tick-labels | X-label measurement/reserve in `layout/canvas.rs:94-132,204-231,509-524`; bottom-axis collision/draw in `axis_frame_grid.rs:49-96,161-191,231-278`; matrix bottom-row labels at `plots/matrix.rs:268-324`; Plot layout/preflight consumes the resulting extents. |
| C-y-tick-labels | Y-label measurement/reserve in `layout/canvas.rs:110-140,181-202,528-538`; left-axis collision/draw in `axis_frame_grid.rs:49-96,193-227,279-333`; matrix left-column labels at `plots/matrix.rs:334-398`; Plot layout/preflight consumes the resulting extents. Canvas category-label sharing is only a T2b consumer/counterpart. |
| C-axis-title-policy | `builder/layout.rs:58-103,172-203`; `layout/canvas.rs:141-231,518-538`; `orchestrator/facets.rs:33-35`; matrix at `plots/matrix.rs:312-398`. Standard `Axis::draw` ignores these flags once a label remains nonempty. |
| C-title-policy | `builder/layout.rs:56-103`; `decorations/apply.rs:39-67`; matrix title block; `canvas.rs:753-810,1234-1308`; `orchestrator/facets/planning.rs:26-31`. Canvas root content bypasses the palette flag. |
| C-tick-metrics | `orchestrator.rs:188-236`; `layout/canvas.rs:87-272,473-599`; `layout/panel/axis_frame_grid.rs`; `layout/panel/geometry.rs`; `builder/layout.rs:128-168`; custom matrix, marginal, horizontal-bar, heatmap, and interval-estimate planning/drawing. |
| C-detail-size | `decorations/panel_text.rs:33-84`; `plots/beeswarm.rs:64,202`; `heatmap.rs:63,813`; `interval_estimate.rs:187-202,533`; `waterfall.rs:98,167-179`. |
| C-label-size | Standard axis measurement/drawing; matrix row/column labels; facet shared axes; horizontal-bar adjustment; Canvas text-panel body at `canvas.rs:1197-1203` and its planning at `canvas.rs:868`. |
| C-title-size-plot | Plot title measurement/drawing in `layout/canvas.rs`, `decorations/title_block.rs`, matrix title, and composed Plot panel titles at `canvas.rs:1234-1308`. |
| C-title-size-canvas-foundation | Canvas root title measurement in `layout/canvas.rs:473-599,603-643` and drawing through `canvas.rs:1010-1020` plus `decorations/title_block.rs:104-123`. |
| C-title-size-canvas-composition | Canvas shared-header, row-title, and text-panel-title measurement/drawing at `canvas.rs:505-575,657-674,1119-1195`. |
| C-subtitle-size | Plot/Canvas subtitle measurement and drawing in `layout/canvas.rs`, `decorations/title_block.rs`, `canvas.rs`; facet strip/header use at `orchestrator/facets/rendering.rs:98-115`. |
| C-caption-size | Plot/Canvas caption measurement and drawing in `layout/canvas.rs:492-587,631-640` and `decorations/title_block.rs:148-166`. |
| C-rgb | Every SVG point, point outline, line, polygon fill/stroke, gradient stop, rounded-rectangle fill/stroke, text, and bracket paint at `render/svg_backend.rs:440-672`. |
| C-font-constants | `DEFAULT_FONT_FAMILY_STACK`: four presets, `decorations/legend.rs:677,694`, and dormant `TitleBlock::new` at `decorations/title_block.rs:41-45`. `DEFAULT_FONT_FAMILY`: point-layer labels at `plots/line.rs:243-254`. All three constants are re-exported at `lib.rs:52`. |
| C-host-fonts | `configure_system_fonts` is called only by outlined SVG at `render/svg_backend.rs:100-108` and PNG at `render/svg_backend.rs:125-138`; preserve-text SVG does not call it. |

## T2a1 Current State

| Property | Current source and value | Current consumers | Current behavior | Current layout path | Semantic role | Audit evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `preset.routing.current-theme-enum` | M1/M2 `define_themes!`; public `Theme::{Publication,Nature,Science,CHI}`; `Theme::palette() -> ThemePalette` exhaustive match | C-route | B-PAL; no unmatched branch or fallback; each call rebuilds the complete preset | Indirectly reaches every measure, reserve, geometry, legend, and draw consumer | Deterministic routing from one selected built-in identity to one complete preset | M1/M2; theme-selection and hidden-construction matches admitted |
| `color-scales.qualitative.current-theme-colors` | M3/M5-M8; PV.colors | C-colors categorical/hue/series/category subset and generated legend glyphs | B-PAL; no nonempty check; `index % len` cycles and empty usually panics; unkeyed scatter alone falls back to black | `none`; paint and legend construction only | Default categorical data-to-color mapping | M3/M5-M8; color tuples, indexing, modulo, and fallback admitted |
| `marks.accent.current-theme-colors` | M3/M5-M8; PV.colors | C-colors unkeyed primary, point-layer, regression, interval, histogram, QQ, waterfall-sign subset | B-PAL; shares categorical modulo sequence; exact local color can replace final paint after slot selection | `none`; mark/legend paint only | Non-data primary and supplemental accent sequence | M3/M5-M8; color/index matches admitted as split child |
| `typography.plot.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-plot | B-PAL and B-TEXT; foundational Plot roles share one unvalidated CSS expression | `none` in measurement; final title/axis text draw and post-SVG glyph conversion | Plot title/subtitle/caption, axis/tick, and self-managed axis-like typography stacks | M3/M5-M8; foundational Plot font consumers admitted |
| `typography.annotation.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-annotation | B-PAL and B-TEXT; annotation roles share the Plot stack without an independent owner | `none` in measurement; annotation text draw/postprocessing | Panel-text, significance, value-label, status, and variable-fill annotation stacks | M3/M5-M8; annotation font consumers admitted as split child |
| `typography.legend-guide.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-legend-guide | B-PAL and B-TEXT; color guides use the Plot stack rather than legend typography | `none` in measurement; guide text draw/postprocessing | Continuous color-guide typography stacks | M3/M5-M8; guide font consumers admitted as split child |
| `typography.facet-header.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-facet-header | B-PAL and B-TEXT; facet strips read the owning Plot palette repeatedly | `none` in measurement; facet-header text draw/postprocessing | Facet strip/header typography stack | M3/M5-M8; facet-header font consumer admitted as split child |
| `typography.canvas-foundation.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-canvas-foundation | B-PAL/B-TEXT; Canvas root title block uses hidden Publication | Measure/reserve ignore family; Canvas root text draw/postprocessing consumes it | Canvas root title/subtitle/caption typography stacks | M3/M5-M8; Canvas foundational font consumer admitted as split child |
| `typography.canvas-composition.font-stack.current-palette-field` | M3/M5-M8; PV.font and exact FSTACK | C-font-canvas-composition | B-PAL/B-TEXT; shared headers/text panels use Publication while row titles may use first Plot | Measure/reserve ignore family; composition text draw/postprocessing consumes it | Canvas shared-header, row-title, and text-panel typography stacks | M3/M5-M8; Canvas composition font consumers admitted as split child |
| `surface.canvas.current-background` | M3/M5-M8; PV.background | C-bg root counterpart only; field has no read | Dormant opaque RGB; backend independently paints white | `none`; no current draw path | Intended Canvas root paint | M3/M5-M8; dormant color source admitted |
| `surface.panel.current-background` | M3/M5-M8; PV.background | C-bg panel counterpart only; field has no read | Dormant and cannot distinguish Canvas from panel | `none`; no current draw path | Intended Plot panel paint | M3/M5-M8; dormant color source admitted as split child |
| `axes.chrome.current-foreground` | M3/M5-M8; PV.foreground | C-fg-axis | B-PAL; opaque RGB, no contrast check | `geometry` only through final frame/axis/tick strokes; no stroke reservation | Axis, tick, and frame paint | M3/M5-M8; axis color reads admitted |
| `typography.plot.current-foreground-text` | M3/M5-M8; PV.foreground | C-fg-plot-text | One opaque color supplies Plot title, axis, tick, and self-managed axis-like text; subtitle/caption use separate hardcoded gray | `none` for paint; final title/axis text draw | Plot title, axis/tick, and self-managed axis-like text paint | M3/M5-M8; foundational Plot text color reads admitted as split child |
| `typography.annotation.current-foreground` | M3/M5-M8; PV.foreground | C-fg-annotation-text | Annotation text shares the generic Plot foreground; significance also shares its color carrier with bracket stroke; automatic-contrast fills use separate colors | Size affects measure/fit elsewhere; paint reaches final annotation text draw | Panel-text, significance, value-label, and status text paint | M3/M5-M8; annotation text color reads admitted as split child |
| `typography.legend-guide.current-foreground-text` | M3/M5-M8; PV.foreground | C-fg-legend-guide | Continuous guides use generic Plot foreground instead of legend typography | `none` for paint; final guide text draw | Continuous color-guide text paint | M3/M5-M8; guide text color reads admitted as split child |
| `typography.facet-header.current-foreground-text` | M3/M5-M8; PV.foreground | C-fg-facet-header | Facet strips use generic Plot foreground; shared facet axes remain C-fg-plot-text | `none` for paint; final facet-header draw | Facet strip/header text paint | M3/M5-M8; facet-header color read admitted as split child |
| `typography.canvas-foundation.current-foreground-text` | M3/M5-M8; PV.foreground | C-fg-canvas-foundation | Canvas root title uses hidden-Publication foreground; root subtitle/caption use separate hardcoded gray | `none` for paint; Canvas root title-block draw | Canvas root title text paint | M3/M5-M8; Canvas foundational foreground context admitted as split child |
| `typography.canvas-composition.current-foreground-text` | M3/M5-M8; PV.foreground | C-fg-canvas-composition | Shared headers/text panels use Publication foreground while row titles may use first Plot | `none` for paint; Canvas composition text draw | Canvas shared-header, row-title, and text-panel text paint | M3/M5-M8; Canvas composition color reads admitted as split child |
| `marks.neutral.current-foreground` | M3/M5-M8; PV.foreground | C-fg-marks | Neutral mark roles share axis/text foreground and renderer-local widths | `none` for paint; final mark geometry/draw | Non-data neutral mark paint | M3/M5-M8; mark color reads admitted as split child |
| `annotations.stroke.current-foreground` | M3/M5-M8; PV.foreground | C-fg-annotation-stroke | Significance `BracketStyle.color` combines separately owned label and stroke meanings; heatmap highlight also reuses foreground | `geometry` through final bracket/highlight draw | Significance and highlight non-text paint | M3/M5-M8; annotation stroke matches admitted as split child |
| `axes.stroke-width.current-axis-width` | M3/M5-M8; PV.axis-width | C-axis-width axis/frame subset | B-WIDTH | `geometry` through final stroke; not reserved | Axis/frame/tick stroke width | M3/M5-M8; axis width reads admitted |
| `marks.interval-width.current-axis-width` | M3/M5-M8; PV.axis-width | C-axis-width mark subset | Accidental reuse of axis width; B-WIDTH | `geometry` through final interval/summary marks | Mark interval and summary stroke width | M3/M5-M8; mark width reads admitted as split child |
| `marks.general-line-width.current-dormant-field` | M3/M5-M8; PV.line-width | No production consumer | Dormant, never validated; renderers use independent constants | `none` | Nominal general series-line width without coherent current semantics | M3/M5-M8; dormant numeric source admitted |
| `annotations.significance-bracket-width.current-field` | M3/M5-M8; PV.bracket-width | C-bracket | B-WIDTH; no construction validation | `geometry` at final bracket draw; planning ignores stroke width | Significance bracket stroke width | M3/M5-M8; bracket width source admitted |
| `axes.frame-sides.current-show-top-right-spines` | M3/M5-M8; PV.top-right | C-top-right | `true` draws bottom/left while full Frame already draws all sides; Science duplicates strokes | `geometry`/draw only; no reserve | Frame-side presence | M3/M5-M8; visibility branch admitted; name/behavior contradiction recorded |
| `axes.frame.current-show-frame` | M3/M5-M8; PV.show-frame | C-frame | Standard Frame draws all four sides; matrix uses same flag for every cell outline | `geometry`/draw only | Nonsemantic frame and panel-outline presence | M3/M5-M8; visibility branches admitted |
| `axes.grid.current-show-grid` | M3/M5-M8; PV.show-grid | C-grid | Jointly gated by `grid_width != 0`; no validation | `geometry`/draw below marks; no reserve | Grid presence | M3/M5-M8; visibility and zero-width branches admitted |
| `axes.ticks.current-show-ticks` | M3/M5-M8; PV.show-ticks | C-ticks | Partially wired; marginal ignores it and layout reserves tick length even when hidden | Reserve remains; `geometry`/draw gate only | Tick-mark presence | M3/M5-M8; visibility branches admitted |
| `axes.x-tick-label-presence.current-theme-flag` | M3/M5-M8; PV.show-tick-labels | C-x-tick-labels | Suppresses x tick labels/reserve, but explicit or automatic labels may be generated or validated before the gate | `measure`, `reserve`, `collision`, `geometry`, draw | X-axis automatic-or-explicit tick-label content presence | M3/M5-M8; x visibility, measurement, and collision matches admitted as split child |
| `axes.y-tick-label-presence.current-theme-flag` | M3/M5-M8; PV.show-tick-labels | C-y-tick-labels | Suppresses y tick labels/reserve, but explicit or automatic labels may be generated or validated before the gate | `measure`, `reserve`, `collision`, `geometry`, draw | Y-axis automatic-or-explicit tick-label content presence | M3/M5-M8; y visibility, measurement, and collision matches admitted as split child |
| `axes.x-title-presence.current-theme-flag` | M3/M5-M8; PV.show-x-label | C-axis-title-policy x subset | Partially wired: reserve paths gate it, standard `Axis::draw` draws any remaining nonempty label | `measure`, `reserve`, `geometry`, draw mismatch | Caller-provided x-axis title presence | M3/M5-M8; policy/visibility matches admitted |
| `axes.y-title-presence.current-theme-flag` | M3/M5-M8; PV.show-y-label | C-axis-title-policy y subset | Same partial wiring as x; matrix gates draw directly | `measure`, `reserve`, `geometry`, draw mismatch | Caller-provided y-axis title presence | M3/M5-M8; policy/visibility matches admitted |
| `content.title-presence.current-theme-flag` | M3/M5-M8; PV.show-title | C-title-policy title subset | Gates Plot title reserve/draw; Canvas root title bypasses it | `measure`, `reserve`, `geometry`, draw | Semantic title content presence | M3/M5-M8; title presence branches admitted |
| `content.subtitle-presence.current-theme-flag` | M3/M5-M8; PV.show-subtitle | C-title-policy subtitle subset | Gates Plot/facet subtitle reserve/draw; Canvas root bypasses it | `measure`, `reserve`, `geometry`, draw | Semantic subtitle content presence | M3/M5-M8; subtitle presence branches admitted |
| `content.caption-presence.current-theme-flag` | M3/M5-M8; PV.show-caption | C-title-policy caption subset | Gates Plot caption reserve/draw; Canvas root bypasses it | `measure`, `reserve`, `geometry`, draw | Semantic caption content presence | M3/M5-M8; caption presence branches admitted |
| `axes.tick-length.current-field` | M3/M5-M8; PV.tick-length; docs say `4`, constructors use `5` | C-tick-metrics | No construction validation; often reserved regardless of `show_ticks` | `measure`, `reserve`, `geometry`, draw | Tick-mark length and tick/text offset | M3/M5-M8; numeric source admitted; documentation drift rejected as source |
| `typography.tick-label-size.current-palette-field` | M3/M5-M8; PV.tick-size | C-tick-metrics | B-PAL/B-TEXT; test-only positivity, no production size validation | `measure`, `reserve`, `collision`, `geometry`, draw | Tick-label text size | M3/M5-M8; size and font matches admitted |
| `typography.tick-label-size.current-global-constant` | M2 `TICK_FONT_SIZE: u32 = 16`; direct at `layout/panel/geometry.rs:89,98,145,150` | Automatic numeric tick generation plus all PV.tick-size initializers | Global bypass agrees with draw only because every preset is `16`; interval-estimate direct matches are tests | `measure`, `collision`, tick-density `geometry`; counterpart draws via palette field | Duplicate tick-label measurement size | M2; production constant uses admitted; interval-estimate matches rejected as tests |
| `typography.detail-status-size.current-field` | M3/M5-M8; PV.detail-size | C-detail-size status/detail subset | B-TEXT; no positive construction validation | `measure`, fit/collision, `geometry`, draw | Detail and status text size | M3/M5-M8; detail-size reads admitted |
| `typography.annotation-size.current-detail-field` | M3/M5-M8; PV.detail-size | C-detail-size panel/value annotation subset | Same field shares annotation, status, and guide roles | `measure`, fit/collision, `geometry`, draw | Annotation text size | M3/M5-M8; annotation-size reads admitted as split child |
| `typography.legend-detail-size.current-detail-field` | M3/M5-M8; PV.detail-size | C-detail-size heatmap/beeswarm guide subset | Plot palette supplies continuous-guide labels instead of a legend role | `measure`, guide `geometry`, draw | Continuous-guide label size | M3/M5-M8; guide-size reads admitted as split child |
| `typography.axis-title-size.current-label-field` | M3/M5-M8; PV.label-size | C-label-size axis/facet/matrix subset | B-TEXT; no positive construction validation | `measure`, `reserve`, fit/collision, `geometry`, draw | Axis-title text size | M3/M5-M8; label-size reads admitted |
| `typography.body-size.current-label-field` | M3/M5-M8; PV.label-size | C-label-size Canvas text-panel subset | Canvas body accidentally reuses axis-label size and hidden Publication | `measure`, `reserve`, `geometry`, draw | Canvas body text size | M3/M5-M8; body-size use admitted as split child |
| `typography.plot-title-size.current-field` | M3/M5-M8; PV.title-size | C-title-size-plot | B-TEXT; line height is separately hardcoded as size plus `6` | `measure`, `reserve`, wrap, `geometry`, draw | Plot title size | M3/M5-M8; Plot title-size reads admitted |
| `typography.canvas-title-size.current-title-field` | M3/M5-M8; PV.title-size | C-title-size-canvas-foundation | Hidden Publication supplies Canvas root title size | `measure`, `reserve`, wrap, `geometry`, draw | Canvas root title size | M3/M5-M8; Canvas root title-size reads admitted as split child |
| `typography.canvas-composition-title-size.current-title-field` | M3/M5-M8; PV.title-size | C-title-size-canvas-composition | Shared headers/text panels use Publication size while row titles may use first Plot | `measure`, `reserve`, wrap, `geometry`, draw | Canvas shared-header, row-title, and text-panel-title size | M3/M5-M8; Canvas composition title-size reads admitted as split child |
| `typography.plot-subtitle-size.current-field` | M3/M5-M8; PV.subtitle-size | C-subtitle-size Plot/panel subset | B-TEXT; hardcoded gray paint and size plus `6` line height | `measure`, `reserve`, wrap, `geometry`, draw | Plot subtitle size | M3/M5-M8; subtitle-size reads admitted |
| `typography.facet-header-size.current-subtitle-field` | M3/M5-M8; PV.subtitle-size | C-subtitle-size facet subset | Facet/header role accidentally tied to subtitle size | `measure`, `reserve`, wrap, `geometry`, draw | Facet/header size | M3/M5-M8; facet size reads admitted as split child |
| `typography.canvas-subtitle-size.current-field` | M3/M5-M8; PV.subtitle-size | C-subtitle-size Canvas subset | Hidden Publication; hardcoded gray paint | `measure`, `reserve`, wrap, `geometry`, draw | Canvas subtitle size | M3/M5-M8; Canvas size reads admitted as split child |
| `typography.plot-caption-size.current-field` | M3/M5-M8; PV.caption-size | C-caption-size Plot/matrix subset | B-TEXT; hardcoded gray and fixed extra reserve | `measure`, `reserve`, wrap, `geometry`, draw | Plot caption size | M3/M5-M8; caption-size reads admitted |
| `typography.canvas-caption-size.current-field` | M3/M5-M8; PV.caption-size | C-caption-size Canvas subset | Hidden Publication; hardcoded gray and fixed extra reserve | `measure`, `reserve`, wrap, `geometry`, draw | Canvas caption size | M3/M5-M8; Canvas caption-size reads admitted as split child |
| `axes.grid-color.current-field` | M3/M5-M8; PV.grid-color | C-grid | Opaque RGB; no alpha/contrast validation | `none`; final grid paint only | Grid-line paint | M3/M5-M8; grid color tuples admitted |
| `axes.grid-width.current-field` | M3/M5-M8; PV.grid-width; docs say `1`, constructors use `0.5` | C-grid | B-WIDTH; zero doubles as an absence sentinel | `geometry`/draw only; no reserve | Grid-line width | M3/M5-M8; width and zero-gate admitted; documentation drift rejected as source |
| `backend.svg.rgb-hex-serialization` | M4 `rgb_to_hex((u8,u8,u8)) -> String`; uppercase `#{:02X}{:02X}{:02X}` | C-rgb | Total deterministic spelling for typed RGB channels with no alpha or fallback; callers, not this helper, choose SVG paint properties and opacity placement | `none`; final private draw boundary | Canonical current SVG RGB color-token spelling | M4; format width and every call admitted |
| `typography.builtin-font-stack.current-public-constant` | M10 `DEFAULT_FONT_FAMILY_STACK: &str = FSTACK` | Four preset initializers; public re-export in C-font-constants | Opaque unvalidated CSS expression; B-TEXT | `none` in measure; draw/postprocessing through preset fields | Built-in ordered typography family preference | M10; font literal and preset uses admitted |
| `typography.legend-font-stack.current-direct-constant` | M10 `DEFAULT_FONT_FAMILY_STACK: &str = FSTACK` | `decorations/legend.rs:677,694`; per-Plot and shared legends | Bypasses Plot/Canvas palette and theme resolution | Legend measure ignores family; final text draw uses constant | Legend title/label stacks | M10; direct legend font uses admitted as split child |
| `typography.title-block-font-stack.current-dormant-fallback` | M10 `DEFAULT_FONT_FAMILY_STACK: &str = FSTACK` at `TitleBlock::new` | C-font-constants; every production caller immediately overrides it | Dormant hidden fallback that would mask a missing resolved style | `none` independently | Hidden title-block font fallback | M10; fallback admitted; overwrite trace recorded |
| `typography.point-layer-label-family.current-default-family` | M10 `DEFAULT_FONT_FAMILY: &str = "Noto Sans"` | Point-layer labels at `plots/line.rs:243-254`; public re-export | Bypasses palette stack; preserve-text SVG has no explicit later families | No measurement/collision; fixed offset then final text draw | Supplemental point-layer label family | M10; font literal/use admitted |
| `font-resolution.current-global-family-order` | M10 `SYSTEM_FONT_FAMILIES: &[&str] = ["Noto Sans","Noto Sans CJK SC","DejaVu Sans","Liberation Sans","Arial"]` | `configure_system_fonts`; public re-export | Global concrete-family order, independent of used CSS stacks; generic omitted | Post-layout font setup only | Generation-host fallback preference | M10; ordered font list admitted |
| `font-resolution.load-system-fonts` | M11 `database.load_system_fonts()` at `font.rs:24` | C-host-fonts | Loads generation-host fonts; no network/project cache; preserve-text bypasses it | Post-layout, before outlined tree parse/PNG rasterization | Host font discovery | M11; font-loading behavior admitted |
| `font-resolution.empty-database-failure` | M11 `database.is_empty()` and Render error at `font.rs:25-29` | C-host-fonts | Hard failure, no alternate renderer or fallback | Post-layout failure before outlined/PNG output | No-font host failure | M11; branch admitted; message text rejected as visual source |
| `font-resolution.named-family-probe` | M11 `SYSTEM_FONT_FAMILIES.iter().find` and `Query { Family::Name, ..Query::default() }` at `font.rs:31-41` | C-host-fonts generic sans-serif setup | First listed family with a normal style/stretch, weight-400 face; not per role or glyph | Post-layout font-selection behavior | Current primary generic-face selection | M11; order, query, and defaults admitted |
| `font-resolution.arbitrary-first-face-fallback` | M11 `.or_else(|| database.faces().find_map(first family))` at `font.rs:42-46` | C-host-fonts when named families miss | It is unconstrained by FSTACK and may select an out-of-stack family. | Post-layout; can change glyph geometry and output bytes | Out-of-stack font substitution | M11; fallback branch admitted |
| `font-resolution.no-family-name-failure` | M11 final `ok_or_else` at `font.rs:47` | C-host-fonts after nonempty but unusable database | Hard Render error | Post-layout failure | Unusable font database failure | M11; failure branch admitted; message text rejected as visual source |
| `font-resolution.global-sans-serif-remap` | M11 `database.set_sans_serif_family(selected)` at `font.rs:49` | usvg generic `sans-serif` resolution for C-host-fonts | Globally remaps generic sans-serif to one selected family; named entries still resolve independently | Post-layout final glyph selection | Generic-family host resolution | M11; generic remap admitted |

## T2a1 Target State

Validation phrases below are complete assignments: construction errors reject
the Theme/component before storage; Plot or Canvas preflight errors occur before
backend begin; geometry/backend checks reject rather than substitute.

| Property | Target classification and owner | Target resolution | Target validation | Public surface | Relationships |
| --- | --- | --- | --- | --- | --- |
| `preset.routing.current-theme-enum` | Renderer invariant; private complete-preset construction/routing responsibility | Complete Theme selected for Canvas root or resolved Plot before preflight | Built-in constructors produce structurally complete validated Themes; no routing fallback | Replaced by T6 `Theme` preset constructors | Split parent of the explicit `theme-preset-property-keys` set; only dispatch is invariant, not selected appearance; T3 pipeline and T4 ownership counterpart |
| `color-scales.qualitative.current-theme-colors` | Theme data scale; `ColorScales` qualitative mapping | Resolved Plot/facet default, replaceable by plot-local categorical scale | Construction requires nonempty opaque seeds; Plot/Canvas preflight validates capacity/mapping; no cycling/substitution | T6 `ColorScales` and `QualitativePalette` | Split sibling `marks.accent.current-theme-colors`; T2g domain/exact-map counterpart |
| `marks.accent.current-theme-colors` | Theme component; `MarkTheme` | Resolved Plot/facet; exact element paint wins | Construction requires nonempty extendable accent sequence; deterministic extension | T6 `MarkTheme` | Split sibling `color-scales.qualitative.current-theme-colors`; T2e/T2g accent-slot counterpart |
| `typography.plot.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Resolved Plot; shared facet axes inherit the Plot | Foundational Plot roles have validated nonempty `FontStack`; Plot preflight resolves face per glyph and fails if missing | T6 title, subtitle, caption, axis-title, and tick-label `TextTheme` stacks | Split siblings `typography.annotation.font-stack.current-palette-field`, `typography.legend-guide.font-stack.current-palette-field`, `typography.facet-header.font-stack.current-palette-field`, `typography.canvas-foundation.font-stack.current-palette-field`, and `typography.canvas-composition.font-stack.current-palette-field`; source counterpart `typography.builtin-font-stack.current-public-constant` |
| `typography.annotation.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Resolved Plot/facet | Annotation roles have validated nonempty stacks; Plot preflight resolves every annotation glyph and fails if missing | T6 annotation and automatic-contrast `TextTheme` stacks | Split child of current `font_family`; annotation metrics counterparts in T2f |
| `typography.legend-guide.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Resolved Plot for per-Plot guide; Canvas root for shared guide | Legend-role stacks validated at construction; Plot/Canvas preflight resolves glyphs and guide fit | T6 legend-title and legend-label `TextTheme` stacks | Split child of current `font_family`; `LegendTheme` owns guide chrome, not font stack |
| `typography.facet-header.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Facet inherited from resolved Plot | Facet/header stack validated at construction; Plot preflight resolves glyphs before facet measurement | T6 facet/header `TextTheme` stack | Split child of current `font_family`; shared facet axes remain `typography.plot.font-stack.current-palette-field` |
| `typography.canvas-foundation.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Canvas root | Canvas preflight resolves root title/subtitle/caption stacks and glyphs before measurement | T6 Canvas title/subtitle/caption `TextTheme` stacks | Split child; hidden-Publication cluster; T4 ownership counterpart |
| `typography.canvas-composition.font-stack.current-palette-field` | Theme component; `TypographyTheme` | Canvas root | Canvas preflight resolves shared-header, row-title, and text-panel stacks/glyphs before measurement | T6 facet/header and body `TextTheme` stacks | Split child; hidden-Publication and first-Plot fallback cluster; composition counterpart T2d |
| `surface.canvas.current-background` | Theme component; `SurfaceTheme` | Canvas root or standalone Publication root | `Color` construction; RGBA is complete and no backend white fallback is allowed | T6 `SurfaceTheme` | Split sibling `surface.panel.current-background`; T2a3 hardcoded-root-surface counterpart |
| `surface.panel.current-background` | Theme component; `SurfaceTheme` | Resolved Plot; facet inherited from Plot | `Color` construction; compositing order checked by Canvas preflight/backend protocol | T6 `SurfaceTheme` | Split sibling `surface.canvas.current-background`; panel-over-Canvas counterpart |
| `axes.chrome.current-foreground` | Theme component; `AxisTheme` | Resolved Plot/facet | `Color` construction; built-in chrome/surface contrast has regression evidence | T6 `AxisTheme` | Split siblings `typography.plot.current-foreground-text`, `typography.annotation.current-foreground`, `typography.legend-guide.current-foreground-text`, `typography.facet-header.current-foreground-text`, `typography.canvas-foundation.current-foreground-text`, `typography.canvas-composition.current-foreground-text`, `marks.neutral.current-foreground`, and `annotations.stroke.current-foreground`; P/Science duplicate-value cluster |
| `typography.plot.current-foreground-text` | Theme component; `TypographyTheme` | Resolved Plot; shared facet axes inherit the Plot | Complete title, axis-title, and tick-label `TextTheme` construction and built-in contrast evidence | T6 title, axis-title, and tick-label `TextTheme` colors | Split child of current `foreground`; hardcoded Plot subtitle/caption muted-paint counterpart |
| `typography.annotation.current-foreground` | Theme component; `TypographyTheme` | Resolved Plot/facet | Complete annotation `TextTheme`; glyph/fit preflight for present annotation text | T6 annotation `TextTheme` color | Split sibling `annotations.stroke.current-foreground`; significance current carrier combines separately owned text and stroke meanings; automatic-contrast color sources remain T2f counterparts |
| `typography.legend-guide.current-foreground-text` | Theme component; `TypographyTheme` | Resolved Plot for per-Plot guide; Canvas root for shared guide | Complete legend-label `TextTheme`; Plot/Canvas guide contrast and fit preflight | T6 legend-label `TextTheme` color | Split child of current `foreground`; `LegendTheme` owns guide chrome, not text paint |
| `typography.facet-header.current-foreground-text` | Theme component; `TypographyTheme` | Facet inherited from resolved Plot | Complete facet/header `TextTheme` and built-in contrast evidence | T6 facet/header `TextTheme` color | Split child of current `foreground`; shared facet axes remain `typography.plot.current-foreground-text` |
| `typography.canvas-foundation.current-foreground-text` | Theme component; `TypographyTheme` | Canvas root | Complete Canvas title `TextTheme` construction and built-in contrast evidence | T6 Canvas title `TextTheme` color | Split child; hidden-Publication and hardcoded subtitle/caption muted-paint cluster; T4 counterpart |
| `typography.canvas-composition.current-foreground-text` | Theme component; `TypographyTheme` | Canvas root | Complete shared-header/body roles and built-in contrast evidence | T6 facet/header and body `TextTheme` colors | Split child; hidden-Publication and first-Plot cluster; composition counterpart T2d |
| `marks.neutral.current-foreground` | Theme component; `MarkTheme` | Resolved Plot/facet; exact element paint wins | Complete `Color` construction; final mark geometry/backend validation | T6 `MarkTheme` | Split child; T2e/T2f neutral reference/outline/missing-role counterparts |
| `annotations.stroke.current-foreground` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Complete paint construction plus annotation geometry/backend validation | T6 `AnnotationTheme` | Split sibling `typography.annotation.current-foreground`; T2f significance/highlight counterparts |
| `axes.stroke-width.current-axis-width` | Theme component; `AxisTheme` | Resolved Plot/facet | Finite positive width at construction | T6 `AxisTheme` | Split sibling `marks.interval-width.current-axis-width`; draw counterpart in T2c |
| `marks.interval-width.current-axis-width` | Theme component; `MarkTheme` | Resolved Plot | Finite positive role width at construction; final geometry validation | T6 `MarkTheme` | Split sibling `axes.stroke-width.current-axis-width`; hardcoded mark-width cluster |
| `marks.general-line-width.current-dormant-field` | Theme component; `MarkTheme` line-role defaults | Resolved Plot | Actual target role widths finite and positive at construction | No preservation of the current general field; T6 coherent `MarkTheme` roles | Dormant field; T2e/T2f renderer-local line-width counterparts own target values |
| `annotations.significance-bracket-width.current-field` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Finite positive width at construction plus bracket geometry validation | T6 `AnnotationTheme` | Measurement-to-draw counterpart in significance planning/T2f |
| `axes.frame-sides.current-show-top-right-spines` | Theme component; `AxisTheme` | Resolved Plot/facet | Structurally complete frame-side selection at construction | T6 `AxisTheme` | Contradiction/duplicate cluster with `axes.frame.current-show-frame` |
| `axes.frame.current-show-frame` | Theme component; `AxisTheme` | Resolved Plot/facet | Frame-side/style construction; absent frame represented structurally | T6 `AxisTheme` | Overlaps `axes.frame-sides.current-show-top-right-spines`; matrix-panel frame counterpart in T2c/T2f |
| `axes.grid.current-show-grid` | Theme component; `AxisTheme` | Resolved Plot/facet | Optional complete grid style; no zero-width absence sentinel | T6 `AxisTheme` | Coupled to `axes.grid-color.current-field` and `axes.grid-width.current-field` |
| `axes.ticks.current-show-ticks` | Theme component; `AxisTheme` | Resolved Plot/facet | Optional complete tick style; reserve and draw consume one resolved value | T6 `AxisTheme` | Coupled to `axes.tick-length.current-field`; marginal bypass counterpart |
| `axes.x-tick-label-presence.current-theme-flag` | Explicit policy; unnamed x-axis tick-content policy whose absence selects renderer-owned automatic ticks and whose explicit current input is `Plot::x_ticks` | Explicit policy owner | Plot preflight validates generated or retained exact x ticks, collisions, and fit | Retained `Plot::x_ticks`; remove theme visibility field | Policy split from `typography.tick-label-size.current-palette-field`; x automatic-generation counterpart in T2c |
| `axes.y-tick-label-presence.current-theme-flag` | Explicit policy; unnamed y-axis tick-content policy whose absence selects renderer-owned automatic ticks and whose explicit current input is `Plot::y_ticks` | Explicit policy owner | Plot preflight validates generated or retained exact y ticks, collisions, and fit | Retained `Plot::y_ticks`; remove theme visibility field | Policy split from `typography.tick-label-size.current-palette-field`; `Canvas::category_labels` is a T2b counterpart only |
| `axes.x-title-presence.current-theme-flag` | Explicit policy; retained `Plot::x_label` | Explicit policy owner | Plot preflight validates content and fit | Retained `Plot::x_label`; no theme visibility field | Typography counterpart `typography.axis-title-size.current-label-field`; partial-wiring cluster with `axes.y-title-presence.current-theme-flag` |
| `axes.y-title-presence.current-theme-flag` | Explicit policy; retained `Plot::y_label` | Explicit policy owner | Plot preflight validates content and fit | Retained `Plot::y_label`; no theme visibility field | Typography counterpart `typography.axis-title-size.current-label-field`; partial-wiring cluster with `axes.x-title-presence.current-theme-flag` |
| `content.title-presence.current-theme-flag` | Explicit policy; retained `Plot::title` | Explicit policy owner | Plot preflight validates present title content and layout fit | Retained `Plot::title`; no theme visibility field | Typography counterpart `typography.plot-title-size.current-field`; `Canvas::title` is a T2b counterpart only because Canvas bypasses this flag |
| `content.subtitle-presence.current-theme-flag` | Explicit policy; retained `Plot::subtitle` | Explicit policy owner | Plot preflight validates present subtitle content and fit | Retained `Plot::subtitle`; no theme visibility field | Typography counterpart `typography.plot-subtitle-size.current-field`; `Canvas::subtitle` is a T2b counterpart only because Canvas bypasses this flag |
| `content.caption-presence.current-theme-flag` | Explicit policy; retained `Plot::caption` | Explicit policy owner | Plot preflight validates present caption content and fit | Retained `Plot::caption`; no theme visibility field | Typography counterpart `typography.plot-caption-size.current-field`; `Canvas::caption` is a T2b counterpart only because Canvas bypasses this flag |
| `axes.tick-length.current-field` | Theme component; `AxisTheme` | Resolved Plot/facet | Finite nonnegative length; mark absence represented separately | T6 `AxisTheme` | Coupled to `axes.ticks.current-show-ticks`, `axes.x-tick-label-presence.current-theme-flag`, and `axes.y-tick-label-presence.current-theme-flag`; documentation drift; measure-to-draw counterpart `typography.tick-label-size.current-palette-field` |
| `typography.tick-label-size.current-palette-field` | Theme component; `TypographyTheme` tick-label role | Resolved Plot/facet | Finite positive size and valid line height at construction; glyph/collision/fit at preflight | T6 tick-label `TextTheme` | Measurement-to-draw duplicate/counterpart `typography.tick-label-size.current-global-constant` |
| `typography.tick-label-size.current-global-constant` | Theme component; `TypographyTheme` tick-label role | Resolved Plot/facet before tick generation | Same validation as `typography.tick-label-size.current-palette-field`; no global bypass | Private-only after migration | Duplicate cluster and measurement-to-draw counterpart `typography.tick-label-size.current-palette-field` |
| `typography.detail-status-size.current-field` | Theme component; `TypographyTheme` detail role | Resolved Plot/facet | Finite positive size/line height; glyph and fit at Plot preflight | T6 detail `TextTheme` | Split siblings `typography.annotation-size.current-detail-field` and `typography.legend-detail-size.current-detail-field` |
| `typography.annotation-size.current-detail-field` | Theme component; `TypographyTheme` annotation role | Resolved Plot/facet; exact annotation style may win | Finite positive size/line height; annotation fit at preflight | T6 annotation `TextTheme` | Split child; current exact significance controls remain higher precedence |
| `typography.legend-detail-size.current-detail-field` | Theme component; `TypographyTheme` legend-label role | Resolved Plot for per-Plot guide; Canvas root for shared guide | Finite positive size; guide fit at Plot/Canvas preflight | T6 legend-label `TextTheme` | Split child; `LegendTheme` owns guide chrome/geometry, not text metrics |
| `typography.axis-title-size.current-label-field` | Theme component; `TypographyTheme` axis-title role | Resolved Plot/facet | Finite positive size/line height; glyph/wrap/fit at preflight | T6 axis-title `TextTheme` | Split sibling `typography.body-size.current-label-field`; policy counterparts `axes.x-title-presence.current-theme-flag` and `axes.y-title-presence.current-theme-flag` |
| `typography.body-size.current-label-field` | Theme component; `TypographyTheme` body role | Canvas root | Finite positive size/line height; Canvas glyph/fit preflight | T6 body `TextTheme` | Split child; hidden-Publication cluster; T4 ownership counterpart |
| `typography.plot-title-size.current-field` | Theme component; `TypographyTheme` title role | Resolved Plot/facet | Finite positive size and valid line height; glyph/wrap/fit preflight | T6 title `TextTheme` | Split siblings `typography.canvas-title-size.current-title-field` and `typography.canvas-composition-title-size.current-title-field`; current hardcoded line-height counterpart in T2c |
| `typography.canvas-title-size.current-title-field` | Theme component; `TypographyTheme` title role | Canvas root | Positive size/valid line height; Canvas root title wrap/fit preflight | T6 title `TextTheme` | Split siblings `typography.plot-title-size.current-field` and `typography.canvas-composition-title-size.current-title-field`; hidden-Publication cluster; T4 counterpart |
| `typography.canvas-composition-title-size.current-title-field` | Theme component; `TypographyTheme` facet/header role | Canvas root | Positive size/valid line height; Canvas composition wrap/fit preflight | T6 facet/header `TextTheme` | Split siblings `typography.plot-title-size.current-field` and `typography.canvas-title-size.current-title-field`; hidden-Publication/first-Plot cluster; T2d counterpart |
| `typography.plot-subtitle-size.current-field` | Theme component; `TypographyTheme` subtitle role | Resolved Plot | Finite positive size/line height; glyph/wrap/fit preflight | T6 subtitle `TextTheme` | Split siblings `typography.facet-header-size.current-subtitle-field` and `typography.canvas-subtitle-size.current-field`; hardcoded-muted-paint counterpart |
| `typography.facet-header-size.current-subtitle-field` | Theme component; `TypographyTheme` facet/header role | Facet inherited from Plot | Finite positive size/line height; facet wrap/fit preflight | T6 facet/header `TextTheme` | Split from subtitle semantics; T2d facet chrome counterpart |
| `typography.canvas-subtitle-size.current-field` | Theme component; `TypographyTheme` subtitle role | Canvas root | Finite positive size/line height; Canvas glyph/wrap/fit preflight | T6 subtitle `TextTheme` | Hidden-Publication and hardcoded-muted-paint cluster; T4 counterpart |
| `typography.plot-caption-size.current-field` | Theme component; `TypographyTheme` caption role | Resolved Plot | Finite positive size/line height; glyph/wrap/fit preflight | T6 caption `TextTheme` | Split sibling `typography.canvas-caption-size.current-field`; policy counterpart `content.caption-presence.current-theme-flag` |
| `typography.canvas-caption-size.current-field` | Theme component; `TypographyTheme` caption role | Canvas root | Finite positive size/line height; Canvas glyph/wrap/fit preflight | T6 caption `TextTheme` | Hidden-Publication cluster; T4 ownership counterpart |
| `axes.grid-color.current-field` | Theme component; `AxisTheme` | Resolved Plot/facet | Complete `Color` construction and built-in contrast evidence | T6 `AxisTheme` | P/CHI duplicate-value cluster; coupled to `axes.grid.current-show-grid` and `axes.grid-width.current-field` |
| `axes.grid-width.current-field` | Theme component; `AxisTheme` | Resolved Plot/facet | Optional grid has finite positive width; absence is structural | T6 `AxisTheme` | Coupled to `axes.grid.current-show-grid` and `axes.grid-color.current-field`; documentation drift |
| `backend.svg.rgb-hex-serialization` | Renderer invariant; private deterministic color-channel/token spelling responsibility | Final private SVG color-token boundary | Typed color channels produce one canonical deterministic token spelling; this owner explicitly excludes paint-property placement and alpha routing | Private-only | Token-spelling counterpart for every paint; paint-property placement and alpha routing belong exclusively to `backend.svg.incomplete-per-paint-alpha-serialization` |
| `typography.builtin-font-stack.current-public-constant` | Theme component; `TypographyTheme` role stacks | Canvas root/resolved Plot/facet by role | `FontFamily` names trimmed/control-free; nonempty `FontStack`; preflight face/glyph resolution | Replaced by T6 `FontStack` and `FontFamily` | Source counterparts `typography.plot.font-stack.current-palette-field`, `typography.annotation.font-stack.current-palette-field`, `typography.legend-guide.font-stack.current-palette-field`, `typography.facet-header.font-stack.current-palette-field`, `typography.canvas-foundation.font-stack.current-palette-field`, and `typography.canvas-composition.font-stack.current-palette-field`; duplicate prefix with `font-resolution.current-global-family-order` |
| `typography.legend-font-stack.current-direct-constant` | Theme component; `TypographyTheme` legend-title and legend-label roles | Resolved Plot for per-Plot legend; Canvas root for shared legend | Validated stacks; Plot/Canvas face/glyph and legend-fit preflight | T6 legend text `TextTheme` roles | Direct-bypass cluster with `typography.builtin-font-stack.current-public-constant`; `LegendTheme` measurement counterpart |
| `typography.title-block-font-stack.current-dormant-fallback` | Theme component; `TypographyTheme` title/subtitle/caption roles | Already-resolved Plot or Canvas owner; no late fallback | Typed stacks validated before title measurement | Private-only; no fallback surface | Dormant duplicate of `typography.plot.font-stack.current-palette-field`, `typography.canvas-foundation.font-stack.current-palette-field`, and `typography.builtin-font-stack.current-public-constant` |
| `typography.point-layer-label-family.current-default-family` | Theme component; `TypographyTheme` annotation role | Resolved Plot; exact element text style would have precedence | Validated `FontStack`; Plot preflight face/glyph availability | Replaced local source; public constant retired by T6 | Duplicate first family of `typography.builtin-font-stack.current-public-constant` and `font-resolution.current-global-family-order`; T2e point-label metrics counterpart |
| `font-resolution.current-global-family-order` | Renderer invariant; private per-used-`FontStack` host face resolution | Before measurement/preflight for each used stack | Resolve only supplied stack members; fail on unavailable face/glyph | Replaced by T6 typed stacks; no public global list | Duplicate prefix of `typography.builtin-font-stack.current-public-constant`; parent of `font-resolution.named-family-probe`, `font-resolution.arbitrary-first-face-fallback`, and `font-resolution.global-sans-serif-remap` |
| `font-resolution.load-system-fonts` | Renderer invariant; generation-host font database discovery | Before measurement and scene creation | Host discovery failure rejects Plot/Canvas preflight; no network/cache fallback | Private-only | Lifecycle mismatch with B-TEXT; parent of `font-resolution.empty-database-failure`, `font-resolution.named-family-probe`, `font-resolution.arbitrary-first-face-fallback`, `font-resolution.no-family-name-failure`, and `font-resolution.global-sans-serif-remap` |
| `font-resolution.empty-database-failure` | Renderer invariant; font availability preflight | Before measurement | Plot/Canvas preflight error identifies used stack/owner; no alternate renderer | Private-only error behavior | Lifecycle counterpart `font-resolution.load-system-fonts` |
| `font-resolution.named-family-probe` | Renderer invariant; first-installed-face resolution for each used `FontStack` | Before measurement/scene creation | Normal style and normal stretch are fixed private invariants; only approved `TextTheme` weight is role-controlled; resolve each glyph through stack order and fail if unavailable | Private-only | Child of `font-resolution.current-global-family-order`; weight counterpart in `TextTheme` |
| `font-resolution.arbitrary-first-face-fallback` | Renderer invariant; strict supplied-`FontStack` resolution | Before measurement | Reject when supplied stack cannot resolve; never select an outside family | Private-only | Forbidden fallback; child of `font-resolution.current-global-family-order` |
| `font-resolution.no-family-name-failure` | Renderer invariant; font stack availability preflight | Before measurement | Exact owner/stack preflight error; no substitution | Private-only | Failure terminus after `font-resolution.named-family-probe` and `font-resolution.arbitrary-first-face-fallback` |
| `font-resolution.global-sans-serif-remap` | Renderer invariant; per-stack generic `FontFamily` host resolution | Before measurement/scene creation | Generic entry resolves on generation host within each supplied stack; later entries provide glyph fallback | Private-only behind T6 `FontFamily` | Child of `font-resolution.current-global-family-order` and `font-resolution.named-family-probe`; measurement-to-draw mismatch |

## T2a1 Migration And Evidence

| Property | Disposition and owning phase | Visual evidence |
| --- | --- | --- |
| `preset.routing.current-theme-enum` | `replace`, T6; T3 builds one private pipeline and T4 establishes ownership first | No output change expected; exact preserve-text SVG snapshots for all four presets and standalone/Canvas resolution assertions |
| `color-scales.qualitative.current-theme-colors` | `replace`, T5d | Curated prefix unchanged for existing domains; over-capacity behavior is intentional redesign; categorical/facet/Canvas/legend evidence |
| `marks.accent.current-theme-colors` | `replace`, T5c | Existing prefix unchanged; deterministic noncycling extension is intentional redesign; point-layer slot and legend snapshots |
| `typography.plot.font-stack.current-palette-field` | `replace`, T5a | Measurement may intentionally correct when actual faces differ; long-text, axis, title, and shared-facet-axis snapshots |
| `typography.annotation.font-stack.current-palette-field` | `replace`, T5c | Possible intentional annotation text-geometry correction; panel-text, significance, value/status, and variable-fill snapshots |
| `typography.legend-guide.font-stack.current-palette-field` | `replace`, T5b | Intentional guide/legend typography unification; beeswarm, heatmap, standalone, and shared-guide snapshots |
| `typography.facet-header.font-stack.current-palette-field` | `replace`, T5b | Possible intentional text-geometry correction; facet strip/header snapshots |
| `typography.canvas-foundation.font-stack.current-palette-field` | `replace`, T5a; Canvas scope connected in T4 | Intentional hidden-Publication ownership/measurement correction; Canvas root title-block snapshots |
| `typography.canvas-composition.font-stack.current-palette-field` | `replace`, T5b | Intentional hidden-Publication/first-Plot ownership correction; shared-header, row-title, and text-panel snapshots |
| `surface.canvas.current-background` | `replace`, T5a | No built-in color change expected; transparent SVG/PNG and Canvas surface snapshots |
| `surface.panel.current-background` | `replace`, T5a | Built-in white unchanged; panel-over-Canvas compositing and transparency snapshots |
| `axes.chrome.current-foreground` | `replace`, T5a | No intended color change; all-preset axis/chrome snapshots |
| `typography.plot.current-foreground-text` | `replace`, T5a | No ordinary paint change expected; Plot title, axis/tick, self-managed-axis, and shared-facet-axis text assertions; muted subtitle/caption sources remain separate T2c records |
| `typography.annotation.current-foreground` | `replace`, T5c | No ordinary paint change expected; panel-text, significance, and value/status annotation snapshots; automatic-contrast colors remain separate T2f records |
| `typography.legend-guide.current-foreground-text` | `replace`, T5b | Intentional guide/legend typography unification; guide contrast and SVG text assertions |
| `typography.facet-header.current-foreground-text` | `replace`, T5b | No intended paint change; facet strip/header snapshots |
| `typography.canvas-foundation.current-foreground-text` | `replace`, T5a; scope connected in T4 | Intentional hidden-Publication ownership correction; Canvas root title snapshot, with muted subtitle/caption sources retained as separate T2c records |
| `typography.canvas-composition.current-foreground-text` | `replace`, T5b | Intentional hidden-Publication/first-Plot ownership correction; shared-header, row-title, and text-panel snapshots |
| `marks.neutral.current-foreground` | `replace`, T5c | No extraction change expected; renderer-family and exact legend-glyph snapshots |
| `annotations.stroke.current-foreground` | `replace`, T5c | No extraction change expected; significance/highlight SVG snapshots |
| `axes.stroke-width.current-axis-width` | `replace`, T5a | No intended width change; per-preset axis snapshots |
| `marks.interval-width.current-axis-width` | `replace`, T5c | Preserve current family outputs; interval/summary mark and legend snapshots |
| `marks.general-line-width.current-dormant-field` | `remove`, T5c | No output change; source/consumer search must remain empty while actual role widths retain evidence |
| `annotations.significance-bracket-width.current-field` | `move`, T5c | No intended change; significance layout and exact stroke SVG assertions |
| `axes.frame-sides.current-show-top-right-spines` | `replace`, T5a | Intentional correction; exact per-preset frame-side snapshots, including Science duplicate removal |
| `axes.frame.current-show-frame` | `replace`, T5a | Frame correction may intentionally change snapshots; standard and matrix evidence |
| `axes.grid.current-show-grid` | `replace`, T5a | No built-in change expected; grid layer/order snapshots |
| `axes.ticks.current-show-ticks` | `replace`, T5a | Intentional reserve/draw synchronization; standard, marginal, and matrix snapshots |
| `axes.x-tick-label-presence.current-theme-flag` | `remove`, T5a | No current preset change; renderer-owned automatic x ticks, retained `Plot::x_ticks`, collision, and fit tests |
| `axes.y-tick-label-presence.current-theme-flag` | `remove`, T5a | No current preset change; renderer-owned automatic y ticks, retained `Plot::y_ticks`, collision, and fit tests; Canvas category sharing remains T2b evidence |
| `axes.x-title-presence.current-theme-flag` | `remove`, T5a | No current preset change; x-title reserve/draw parity assertions |
| `axes.y-title-presence.current-theme-flag` | `remove`, T5a | No current preset change; multiline/composed y-title assertions |
| `content.title-presence.current-theme-flag` | `remove`, T5a | No current preset change; title layout and Canvas/panel snapshots |
| `content.subtitle-presence.current-theme-flag` | `remove`, T5a | No current preset change; Plot/facet/Canvas subtitle positioning snapshots |
| `content.caption-presence.current-theme-flag` | `remove`, T5a | No current preset change; caption reservation/order and Canvas snapshots |
| `axes.tick-length.current-field` | `move`, T5a | No built-in value change; tick-edge-gap and composed-axis tests |
| `typography.tick-label-size.current-palette-field` | `move`, T5a | No intended size change; scatter snapshot, auto-tick density, and collision tests |
| `typography.tick-label-size.current-global-constant` | `deduplicate`, T5a | No output change; theme-specific tick-size test proves one value reaches generation and draw |
| `typography.detail-status-size.current-field` | `replace`, T5a | No intended value change; interval-status fit assertions |
| `typography.annotation-size.current-detail-field` | `replace`, T5c | No intended value change; panel-text collision and value-label snapshots |
| `typography.legend-detail-size.current-detail-field` | `replace`, T5b | No intended value change; beeswarm/heatmap guide snapshots |
| `typography.axis-title-size.current-label-field` | `replace`, T5a | No intended size change; multiline and composed-axis tests |
| `typography.body-size.current-label-field` | `replace`, T5b; Canvas scope connected in T4 | No intended size change; Canvas text-panel snapshots |
| `typography.plot-title-size.current-field` | `replace`, T5a | No intended size change; all-preset title and matrix fit snapshots |
| `typography.canvas-title-size.current-title-field` | `replace`, T5a; scope connected in T4 | Intentional hidden-Publication ownership correction; Canvas root title snapshots |
| `typography.canvas-composition-title-size.current-title-field` | `replace`, T5b | Intentional hidden-Publication/first-Plot ownership correction; Canvas shared-header, row-title, and text-panel-title snapshots |
| `typography.plot-subtitle-size.current-field` | `replace`, T5a | No intended size change; subtitle position/color snapshots |
| `typography.facet-header-size.current-subtitle-field` | `replace`, T5b | No intended size change; facet strip/header snapshots |
| `typography.canvas-subtitle-size.current-field` | `replace`, T5a; scope connected in T4 | No intended size change; Canvas subtitle snapshots |
| `typography.plot-caption-size.current-field` | `replace`, T5a | No intended size change; existing caption reservation test and snapshots |
| `typography.canvas-caption-size.current-field` | `replace`, T5a; scope connected in T4 | No intended size change; Canvas caption snapshots |
| `axes.grid-color.current-field` | `move`, T5a | No intended paint change; exact all-preset grid snapshots |
| `axes.grid-width.current-field` | `replace`, T5a | No intended built-in width change; exact stroke-width snapshots |
| `backend.svg.rgb-hex-serialization` | `replace`, T5a; scale callers complete in T5d | RGB channel/token spelling remains exact; paint-property and alpha assertions belong to `backend.svg.incomplete-per-paint-alpha-serialization` |
| `typography.builtin-font-stack.current-public-constant` | `replace`, T5a; public constant retired T6 | Font measurement/resolution correction may be intentional; Chinese stack and all-role font tests |
| `typography.legend-font-stack.current-direct-constant` | `connect`, T5b | Intentional mixed-theme/shared-legend correction; legend size and Canvas snapshots |
| `typography.title-block-font-stack.current-dormant-fallback` | `remove`, T5a | No output change; construction assertion/search proves every title block receives resolved styles |
| `typography.point-layer-label-family.current-default-family` | `replace`, T5c; public constant retired T6 | Intentional stack/glyph correction; point-layer label and non-Latin SVG tests |
| `font-resolution.current-global-family-order` | `replace`, T5a; public list retired T6 | Intentional selection-contract correction; ordered-stack tests |
| `font-resolution.load-system-fonts` | `move`, T5a | Possible intentional layout correction; preserve-text/outlined/PNG host-parity tests |
| `font-resolution.empty-database-failure` | `move`, T5a | No successful-output change; focused empty-fontdb preflight error assertion |
| `font-resolution.named-family-probe` | `replace`, T5a | Intentional role/weight/glyph correction; ordered family, weight, and missing-glyph tests |
| `font-resolution.arbitrary-first-face-fallback` | `remove`, T5a | Intentional correction from silent substitution to error; isolated fontdb assertion |
| `font-resolution.no-family-name-failure` | `replace`, T5a | No successful-output change; unusable-fontdb owner/stack error assertion |
| `font-resolution.global-sans-serif-remap` | `replace`, T5a | Intentional per-stack generic resolution; generic, non-Latin, outlined SVG, and PNG parity tests |

## Rejected T2a1 Matches

| Source match | Disposition and reason |
| --- | --- |
| `src/theme/palette.rs:16-18` compile-fail snippet | Rustdoc test of `palette()` privacy, not a production property source; useful API-boundary evidence. |
| `src/theme/palette.rs:292-378` entire `#[cfg(test)]` module | Test-only. Theme arrays, RGB examples, palette lengths, positive-size checks, flag assertions, `14`, and `16` are evidence, not independent sources. |
| `src/theme/palette.rs:317-319` RGB tuples and hex strings | Test duplication of `backend.svg.rgb-hex-serialization`. |
| `src/theme/palette.rs:323-352` flag and positive-size assertions | Test evidence for the visibility and typography records; no independent defaults. |
| `src/theme/palette.rs:356-376` uniform `16` and `14` assertions | Test evidence for tick/detail typography; no independent defaults. |
| Field documentation literals such as default `4`, `14`, `17`, `20`, or `1` | Documentation, not executable defaults. Mismatches for tick length, grid width, and preset-specific font sizes are recorded as drift. |
| `src/theme.rs` and `src/font.rs` prose mentioning themes, colors, or fonts | Documentation only; no independent visual value. |
| Font error-message text at `src/font.rs:27,47` | Wording is non-visual. The failure behaviors themselves are `font-resolution.empty-database-failure` and `font-resolution.no-family-name-failure`. |
| Enum derives, `Hash`, `Eq`, `Clone`, `Copy`, `deny(missing_docs)` | Language/API mechanics that do not alter visible output, occupied space, selection, or paint. |
| `Query::default()` non-family fields | Not rejected: normal style/stretch and weight-400 selection are included in `font-resolution.named-family-probe`. |
| `SYSTEM_FONT_FAMILIES.iter().find`, `faces().find_map`, and `families.first()` | Not rejected: each changes font selection and is included in `font-resolution.named-family-probe`, `font-resolution.arbitrary-first-face-fallback`, `font-resolution.no-family-name-failure`, or `font-resolution.global-sans-serif-remap`. |
| Scientific or statistical data search class | No matches in the T2a1 manifest. |

## Reconciliation

- The completed T2a1 count is exactly **65**. Every key appears once in each of
  the current-state, target-state, and migration/evidence tables.
- Independent ownership review found that five of the initial 57 rows combined
  distinct target owners: tick-label policy, palette font stack, palette text
  foreground, and Canvas title-size contexts. Replacing those five rows with 13
  exact-owner rows increased the reconciled count by eight without discovering
  another production source.
- The public preset selector remains a classification boundary. The ledger
  classifies only its deterministic one-preset-to-one-complete-Theme dispatch as
  a private renderer invariant. Every selected visual value remains exclusively
  in the split component or scale records.
- `ThemePalette::colors`, `font_family`, `background`, `foreground`,
  `axis_width`, and the multi-role typography sizes remain physically separate
  records where their target semantics, resolution scopes, or owners differ.
- `background` and `line_width` are dormant; `show_top_right_spines` is miswired;
  `show_ticks`, `show_tick_labels`, `show_x_label`, and `show_y_label` are
  partially wired. Their dispositions preserve rather than conceal those facts.
- Equal preset literals are shared only through `PV`; they are not treated as
  evidence that independent target roles have one owner.
- Every direct palette font-stack read and every palette-foreground read is
  partitioned by target role. Shared renderer reads use disjoint Plot/Canvas or
  text/stroke call contexts rather than duplicate target ownership.
- No target owner, public surface, relationship, validation, phase, or evidence
  field is `TBD`, `unknown`, or `other`. No unapproved future public type name is
  introduced.

## Existing Evidence

Useful evidence consumers include the theme tests in M9,
`tests/visual_regression`, `tests/layout/layout_and_legends.rs`,
`tests/rendering/public_contracts.rs:168-185`, and
`tests/annotations/panel_text.rs:39-69`. They do not own ledger records. Exact
full-preset snapshots for Nature, Science, and CHI remain required by the
migration evidence above; T2a1 does not create or update them.

## T2a2: Private Style Carriers And Backend Protocol

This section owns only foundational declarations, defaults, mutators,
validation, and protocol behavior declared in `src/render/backend.rs`. Values
selected by plots, layouts, legends, annotations, or scales and merely carried
through these types remain consumer references owned by later T2 subphases.

### T2a2 Source Manifest

| ID | Production source | Owned context | Audit disposition |
| --- | --- | --- | --- |
| A2-M1 | `src/render/backend.rs:17-35` | Projected point carrier and constructor | Record `backend.coordinates.projected-point-carrier`. |
| A2-M2 | `src/render/backend.rs:37-128` | Point style, outline carrier, defaults, and mutators | Records from `backend.point.fill-paint-carrier` through `backend.point.outline-width-resolution`. |
| A2-M3 | `src/render/backend.rs:130-192` | Line, fill, and gradient-stop carriers and constructors | Records from `backend.line.stroke-paint-carrier` through `backend.gradient.stop-carrier`. |
| A2-M4 | `src/render/backend.rs:194-294` | Text anchor/style, bracket/layer carriers, significance metrics, and bracket style | Records from `backend.text.anchor-default` through `backend.bracket.label-gap-carrier`. |
| A2-M5 | `src/render/backend.rs:296-404` | Backend coordinate, lifecycle, layer, clip, gradient, and rounded-rectangle protocol | Records from `backend.protocol.coordinate-convention` through `backend.protocol.rounded-rectangle-fallback`. |
| A2-M6 | `src/render/backend.rs:406-615` | Geometry and style validation | Records from `backend.validation.clip-region` through `backend.validation.significance-bracket`. |
| A2-M7 | `src/render/backend.rs:617-666` | Balanced draw-scope helpers and error precedence | Record `backend.protocol.scope-cleanup-precedence`. |
| A2-M8 | `src/render/backend.rs:668-729` | Test module | Excluded as a property source; retained as evidence only. |

The manifest is exact for T2a2. Production sources outside
`src/render/backend.rs` are consumer or evidence traces, not records owned by
this section.

### T2a2 Consumer And Evidence References

The compact codes below expand into each record that cites them.

| Code | Exact production consumers or evidence |
| --- | --- |
| A2-C-POINT | Point styles are assembled by `builder/accessors.rs:917-918`, `layers.rs:197-200,292-301`, `plots/distribution_sample.rs:164-165`, and `plots/bar_data.rs`; drawn by `plots/bar.rs`, `beeswarm.rs`, `boxplot.rs`, `dumbbell.rs`, `horizontal_bar.rs`, `interaction.rs`, `interval_estimate.rs`, `line.rs`, `marginal.rs`, `matrix.rs`, `point_interval.rs`, `qq.rs`, `regression.rs`, `scatter.rs`, `violin.rs`, and `violin/precomputed.rs`; copied/compared/drawn by `decorations/legend.rs`; and validated, measured, or serialized by `orchestrator/backend.rs`, `render/geometry.rs`, and `render/svg_backend.rs`. |
| A2-C-LINE | Line styles are assembled or drawn by `layers.rs`, `spec/estimate.rs`, `layout/panel/axis_frame_grid.rs`, `orchestrator/layers.rs`, `decorations/legend.rs`, and `plots/bar.rs`, `beeswarm.rs`, `boxplot.rs`, `density.rs`, `dumbbell.rs`, `heatmap.rs`, `histogram.rs`, `horizontal_bar.rs`, `interaction.rs`, `interval_estimate.rs`, `line.rs`, `marginal.rs`, `matrix.rs`, `point_interval.rs`, `qq.rs`, `regression.rs`, `violin.rs`, `violin/precomputed.rs`, and `waterfall.rs`; terminal consumers are the deferred, null, geometry, and SVG backends. |
| A2-C-FILL | Fill styles are assembled or drawn by `decorations/legend.rs` and `plots/bar.rs`, `bar_data.rs`, `boxplot.rs`, `density.rs`, `heatmap.rs`, `histogram.rs`, `horizontal_bar.rs`, `line.rs`, `marginal.rs`, `matrix.rs`, `regression.rs`, `violin.rs`, `violin/precomputed.rs`, and `waterfall.rs`; terminal consumers are the deferred, null, geometry, and SVG backends. |
| A2-C-TEXT | Text styles are assembled or drawn by `canvas.rs`, `layout/panel/axis_frame_grid.rs`, `decorations/legend.rs`, `decorations/panel_text.rs`, `decorations/title_block.rs`, `orchestrator/facets/rendering.rs`, and `plots/bar.rs`, `beeswarm.rs`, `heatmap.rs`, `horizontal_bar.rs`, `interval_estimate.rs`, `line.rs`, `matrix.rs`, and `waterfall.rs`; terminal consumers are the deferred, null, geometry, and SVG backends. |
| A2-C-GRADIENT | Gradient stops and geometry are produced only by `plots/beeswarm.rs` and `plots/heatmap.rs`, then forwarded, validated, recorded, or serialized by `orchestrator/backend.rs`, `render/geometry.rs`, and `render/svg_backend.rs`. |
| A2-C-BRACKET | `significance.rs` plans brackets, derives metrics, assembles `BracketStyle`, and draws; `decorations/plan.rs` records their collision geometry; deferred, null, geometry, and SVG backends consume them. |
| A2-C-SCOPE | `with_clip`, `with_layer`, and `with_layer_role` are used by `canvas.rs`, `decorations/apply.rs`, `decorations/panel_text.rs`, `orchestrator/facets/rendering.rs`, `orchestrator/layers.rs`, and clipped plot renderers. `orchestrator/backend.rs`, `render/geometry.rs`, and `render/svg_backend.rs` implement the protocol. |
| A2-E-UNIT | `src/render/backend.rs:668-729` proves accepted complete gradients and rejects invalid bounds, vectors, endpoints, stop counts, offsets, and ordering. Test literals are evidence only. |
| A2-E-PAINT | `tests/rendering/public_contracts.rs`, `tests/lines/numeric_layers.rs`, `tests/lines/effect_bands.rs`, `tests/interactions/styles.rs`, `tests/categorical/grouping.rs`, and `tests/marginal_matrix/contracts.rs` assert point outlines, radii, opacity, fill opacity, and dash behavior. |
| A2-E-TEXT | `tests/rendering/labels.rs`, `tests/rendering/public_contracts.rs`, `tests/annotations/panel_text.rs`, and `tests/annotations/significance_layout.rs` assert font-family, weight omission, anchors, repeated spaces, rotation, and annotation text. |
| A2-E-SCOPE | `src/orchestrator/tests/lifecycle.rs`, `tests/composition/canvases.rs`, and clipping-focused integration tests assert begin/end counts, cleanup error preservation, clipping, and grid-before-marks behavior. |
| A2-E-GEOMETRY | `src/render/geometry.rs:267-361` and `src/decorations/plan/tests.rs` assert local point/line bounds, gradient bounds, bracket overhang, and collision-driven legend placement. |

### T2a2 Current State

| Property | Current source and value | Current consumers | Current behavior | Current layout path | Semantic role | Audit evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `backend.coordinates.projected-point-carrier` | A2-M1 `Point2D { x: f64, y: f64 }`; projected canvas pixels for non-text primitives; `new` copies both values | A2-C-POINT, A2-C-LINE, A2-C-FILL, A2-C-GRADIENT | Constructor accepts every `f64`; individual draw paths later call `validate_points`; text uses the separate integer positions in `backend.protocol.coordinate-convention` | `geometry`; reaches every projected non-text primitive and final draw | Canonical private projected-point carrier | A2-M1; coordinate declarations admitted; data coordinates remain consumer inputs |
| `backend.point.fill-paint-carrier` | A2-M2 `PointStyle.color: (u8,u8,u8)` opaque RGB plus separate `opacity: f64` nominally `[0,1]` | A2-C-POINT | No construction validation; backend validates only opacity; alpha applies only to fill and is not a color channel | `none`; final point paint only | Incomplete current point fill-paint carrier | A2-M2; RGB and opacity matches admitted |
| `backend.point.geometry-carrier` | A2-M2 `radius: u32` px and `shape: PointShape` | A2-C-POINT | Radius zero is accepted; closed shape is interpreted at draw; no fallback variant | `geometry`, then collision recording and draw | Point extent and marker geometry carrier | A2-M2; radius and shape matches admitted |
| `backend.point.outline-carrier` | A2-M2 `outline: Option<PointOutline>` | A2-C-POINT | `None` removes the stroke; `Some` carries RGB and radius-relative width; no outline alpha | `geometry`; final SVG draws it but current geometry recording omits its width | Independent point-outline carrier | A2-M2; option and outline matches admitted |
| `marks.point.outlined-constructor-default` | A2-M2 `PointStyle::new`: supplied RGB/radius plus `Some(PointOutline::default())`, `PointShape::Circle`, opacity `0.82` | `plots/distribution_sample.rs:164-165`; A2-C-POINT downstream | Late unvalidated complete default; no theme resolution in the constructor | `geometry`, collision, draw | Default outlined scatter-like point appearance | A2-M2; constructor defaults admitted; duplicate cluster with the two other point constructors |
| `marks.point.plain-constructor-default` | A2-M2 `PointStyle::plain`: supplied RGB/radius plus `None`, `Circle`, opacity `0.82` | `layers.rs:197-200,292-301`; A2-C-POINT downstream | Layer conversion overwrites opacity, making this `0.82` inert there; circle and no-outline persist | `geometry`, collision, draw | Default plain supplemental-point appearance | A2-M2; constructor defaults and overwrite trace admitted |
| `marks.point.explicit-outline-constructor-default` | A2-M2 `PointStyle::with_outline`: supplied RGB/radius/outline plus `Circle`, opacity `0.82` | `builder/accessors.rs:917-918`; A2-C-POINT downstream | Passed outline wins; shape/opacity remain late defaults; validation deferred to draw | `geometry`, collision, draw | Primary point defaults around an explicit outline setting | A2-M2; constructor defaults admitted |
| `backend.point.outline-assembly` | A2-M2 `with_outline` copies one `Option<PointOutline>` into the final carrier | `builder/accessors.rs:917-918`; A2-C-POINT | Current Plot outline policy is passed through exactly; no merge or fallback | `geometry`; outline draw and currently incomplete bounds | Private exact point-outline assembly | A2-M2; assignment branch admitted; public policy source excluded to T2b |
| `backend.point.local-mutator-replacement` | A2-M2 `with_shape` and `with_opacity` replace their fields on the consumed style | `plots/interaction.rs:167,197`; `layers.rs:198-200,296-301` | Last called setter wins; opacity replaces rather than multiplies; no immediate validation | Shape affects `geometry`; opacity is paint-only | Final private point-style replacement semantics | A2-M2; mutator branches admitted |
| `marks.point.outline-publication-default` | A2-M2 `PointOutline::default`: color `(255,255,255)` RGB and `width_ratio: 0.1` | `PointStyle::new`; duplicate construction at `builder.rs:552` | Opaque white and unchecked ratio; selected before backend validation | `geometry`; final outline draw, current bounds omission | Built-in point-outline appearance default | A2-M2; white tuple and ratio admitted; builder source is a duplicate consumer reference |
| `backend.point.outline-width-resolution` | A2-M2 `width_for_radius = f64::from(radius) * width_ratio`, yielding px | Point validation and SVG point drawing through A2-C-POINT | No clamp; finite/nonnegative ratio and finite product checked only at draw | `geometry`; final outline width | Radius-relative point-outline width rule | A2-M2; multiplication and conversion admitted |
| `backend.line.stroke-paint-carrier` | A2-M3 `LineStyle.color: (u8,u8,u8)` opaque RGB; no alpha field | A2-C-LINE | Supplied unchanged; no color validation or fallback | `none`; final stroke paint | Incomplete line-stroke paint carrier | A2-M3; RGB field admitted |
| `backend.line.stroke-width-carrier` | A2-M3 `LineStyle.width: f32` px | A2-C-LINE | Finite nonnegative validation occurs only at draw; zero is allowed | `geometry`, collision, draw | Line stroke-width carrier | A2-M3; width field admitted |
| `backend.line.dash-string-carrier` | A2-M3 `dash: String`; empty means solid; examples use backend string `"4 2"` | A2-C-LINE | Nonempty strings are neither parsed nor validated here; SVG later emits them directly | `none`; changes stroke pattern only | Backend-specific current dash encoding | A2-M3; string, empty sentinel, and dash documentation admitted |
| `backend.line.pattern-constructor-branches` | A2-M3 `solid` stores `String::new()`; `dashed` copies `pattern.to_string()` | Every A2-C-LINE constructor path | Branch is exact and has no fallback; color/width are copied unchanged | `none`; final solid-versus-dashed selection | Private line-pattern assembly | A2-M3; constructor branches admitted |
| `backend.fill.paint-carrier` | A2-M3 `FillStyle.color: (u8,u8,u8)` plus separate `opacity: f64` | A2-C-FILL | Opacity validated `[0,1]` only at draw; no fill alpha channel or color validation | `none`; polygon/rectangle fill paint | Incomplete fill-paint carrier | A2-M3; RGB and opacity fields admitted |
| `backend.fill.stroke-carrier` | A2-M3 `stroke: (u8,u8,u8)` and `stroke_width: f32` px; zero documented as no outline | A2-C-FILL | Width finite/nonnegative at draw; stroke is always opaque and still serialized at zero width | Width affects `geometry`; color is paint-only | Fill-outline stroke carrier | A2-M3; RGB, width, and zero sentinel admitted |
| `backend.gradient.stop-carrier` | A2-M3 `GradientStop { offset: f64, color: (u8,u8,u8) }`; constructor copies both | A2-C-GRADIENT | RGB-only stops; complete ordering/domain validation deferred to gradient draw | Gradient rectangle has `geometry`; stop is paint-only | Final private gradient-stop carrier | A2-M3; offsets and RGB fields admitted |
| `backend.text.anchor-default` | A2-M4 `TextAnchor::{Start,Middle,End}` with derived default `Start` | A2-C-TEXT | Closed alignment; most producers set explicitly; no fallback outside Start default | `geometry`, collision, draw | Horizontal text anchoring | A2-M4; enum default and variants admitted |
| `backend.text.paint-carrier` | A2-M4 `TextStyle.color: (u8,u8,u8)` opaque RGB; no alpha | A2-C-TEXT | Copied directly; no paint validation | `none`; final text paint | Incomplete text-paint carrier | A2-M4; RGB field admitted |
| `backend.text.typography-carrier` | A2-M4 `font_family: String`, `font_size: u32` documented points, `weight: TextWeight`; current SVG/geometry treat size as pixel-like | A2-C-TEXT | Family may be empty and size zero; geometry ignores family/weight; only later host conversion resolves fonts | `geometry` and collision; upstream measurement duplicates source metrics | Private unresolved text typography carrier | A2-M4; font, size, weight, and unit mismatch admitted |
| `backend.text.placement-carrier` | A2-M4 `anchor: TextAnchor`, `rotation: f64` clockwise degrees; docs state Start and `0.0` defaults but the struct has no constructor default | A2-C-TEXT | Rotation alone is checked finite; producers supply both values | `geometry`, collision, draw | Text placement and orientation carrier | A2-M4; anchor/rotation fields admitted; documentation defaults are not executable sources |
| `backend.bracket.geometry-carrier` | A2-M4 `SignificanceBracket`: three `f64` anchors, `String` label, and `BracketOrientation::{Vertical,Horizontal}` with documented coordinate overloading | A2-C-BRACKET | Values derive elsewhere; backend owns only fixed interpretation and distinct-anchor validation | `reserve`, `geometry`, collision, draw | Private significance-bracket geometry/orientation protocol | A2-M4; carrier and orientation behavior admitted; content/data excluded |
| `backend.layer.semantic-role` | A2-M4 `LayerRole::{Generic,Grid}` with derived default `Generic` | A2-C-SCOPE | Generic preserves call order; Grid is advisory to backends that can reorder; no other role | `none`; z-order only | Semantic backend layer-order role | A2-M4; enum/default/role matches admitted |
| `annotations.significance.tick-height-default` | A2-M4 `SIGNIFICANCE_BRACKET_METRICS.tick_height = 7.0 f64` px | A2-C-BRACKET | Fixed private value reused by planning, SVG drawing, and geometry recording | `reserve`, `geometry`, collision, draw | Significance bracket end-tick extent | A2-M4; metric literal admitted |
| `annotations.significance.label-gap-default-copy` | A2-M4 metrics `label_gap = 5.0 f64` px | `significance.rs::bracket_metrics` and style assembly | Production style conversion overwrites this field, so the constant copy has no independent successful-output effect | `none` independently | Duplicate significance line-to-label gap default | A2-M4; literal and overwrite trace admitted |
| `typography.significance.font-size-default-copy` | A2-M4 metrics `label_font_size = 12 u32` px | `significance.rs::bracket_metrics`; tests read the constant | Production conversion overwrites it from current significance style | `none` independently | Duplicate significance-label text-size default | A2-M4; literal and overwrite trace admitted |
| `annotations.significance.data-gap-default` | A2-M4 metrics `data_gap = 6.0 f64` px | `significance.rs::place_lines` and `data_fits_below_ceiling` | Fixed planning clearance; no local validation | `reserve`; affects automatic ranges and bracket placement | Data-to-bracket clearance | A2-M4; metric literal admitted |
| `annotations.significance.level-step-default` | A2-M4 metrics `level_step = 26.0 f64` px | `significance.rs::significance_level_spacing` | Lower bound combined with label extent and panel padding | `reserve`; separates annotation levels | Minimum significance-level separation | A2-M4; metric literal admitted |
| `annotations.significance.panel-padding-default` | A2-M4 metrics `panel_padding = 2.0 f64` px | Significance placement and fit functions | Fixed clearance from annotation ceiling/panel edge; no local validation | `reserve`; can reject or expand range | Significance annotation panel-edge clearance | A2-M4; metric literal admitted |
| `backend.bracket.combined-color-stroke-meaning` | A2-M4 `BracketStyle.color: (u8,u8,u8)` as consumed by bracket path stroke | A2-C-BRACKET | One opaque RGB field supplies both path and label; this record owns only the stroke meaning | `none`; final bracket-line paint | Current bracket non-text paint arm | A2-M4; combined RGB field admitted as split parent/child |
| `backend.bracket.combined-color-text-meaning` | Same A2-M4 `BracketStyle.color`, as consumed by bracket label fill | A2-C-BRACKET | Same opaque RGB cannot vary independently from bracket stroke | `none`; final annotation text paint | Current bracket label-text paint arm | A2-M4; combined RGB field admitted as split sibling |
| `backend.bracket.stroke-width-carrier` | A2-M4 `BracketStyle.width: f32` px | A2-C-BRACKET | Finite nonnegative validation at draw; geometry recording ignores stroke overhang | `geometry`, collision, draw | Bracket stroke-width carrier | A2-M4; width field admitted |
| `backend.bracket.typography-carrier` | A2-M4 family `String`, size `u32` px, `TextWeight` | A2-C-BRACKET | Size must be positive at draw; family/weight are not validated; geometry estimates size and ignores face/weight | `reserve`, `geometry`, collision, draw | Significance-label typography carrier | A2-M4; font/size/weight fields admitted |
| `backend.bracket.label-gap-carrier` | A2-M4 `BracketStyle.label_gap: f64` px | A2-C-BRACKET | Finite nonnegative validation at draw; exact style value comes from significance style conversion | `reserve`, `geometry`, collision, draw | Bracket-line-to-label spacing carrier | A2-M4; gap field admitted; current style source remains a consumer reference |
| `backend.protocol.coordinate-convention` | A2-M5 trait docs: x rightward, y upward from canvas bottom; non-text methods receive `f64` geometry while `draw_text` receives `u32` x/y math-space pixels | Every Backend implementation and draw caller; geometry converts text positions exactly to `f64`, while SVG subtracts the integer y from canvas height before integer output | Text positions are quantized before the backend boundary and cannot represent negative coordinates; data scaling/panel placement are excluded from backends; no alternate coordinate convention | `geometry`; governs every primitive and fixes text-position precision | Canonical backend coordinate-space and text-position protocol | A2-M5; protocol documentation, method geometry, and integer text-position signature admitted |
| `backend.protocol.begin-end-lifecycle` | A2-M5 `begin(width,height)` before draws and `end()` after; no draws after end until next begin | A2-C-SCOPE and output orchestration | Trait states sequencing; concrete implementations enforce differing amounts | `none`; lifecycle can gate all output | Balanced canvas rendering lifecycle | A2-M5; begin/end requirements admitted |
| `backend.protocol.ordinary-layer-fallbacks` | A2-M5 default `begin_layer` and `end_layer` return `Ok(())` | Geometry/null/future backends and A2-C-SCOPE | Ordinary grouping may be ignored; no stack validation in defaults | `none`; grouping/order only | Minimum ordinary-layer backend fallback | A2-M5; default method branches admitted |
| `backend.protocol.grid-role-delegation` | A2-M5 default `begin_layer_with_role(name, _role)` delegates to `begin_layer(name)` | Geometry/null backends; SVG overrides | Non-reordering backends ignore Grid semantics by design | `none`; grid z-order only | Optional grid-reordering protocol fallback | A2-M5; delegation branch admitted |
| `backend.protocol.clip-scope-contract` | A2-M5 `begin_clip(PanelBox)`/`end_clip`; scopes may nest with layers and require LIFO | A2-C-SCOPE | No trait default; every backend must implement; invalid balance is backend error | `geometry`; clips mark output and collision geometry | Nested rectangular clipping contract | A2-M5; clip methods and LIFO requirement admitted |
| `backend.protocol.gradient-contract` | A2-M5 `draw_linear_gradient`; math-space bounds/vector; stops begin 0, end 1, strictly increase | A2-C-GRADIENT | No fallback implementation; invalid protocol rejects | `geometry`; gradient rectangle and final paint | Native linear-gradient draw protocol | A2-M5; gradient method requirements admitted |
| `backend.protocol.rounded-rectangle-fallback` | A2-M5 default `draw_rounded_rect` builds four `Point2D` corners and calls `draw_polygon`, ignoring radius | No current concrete backend executes the default: Deferred forwards and SVG, Geometry, and Null override it; an extension backend that omits the method would inherit it | Always succeeds/fails as polygon draw; visible fallback silently loses rounding and performs no rounded-rectangle validation itself | `geometry`; rectangle remains, corner radius is dropped | Dormant lossy rounded-rectangle protocol fallback | A2-M5; fallback geometry, ignored radius, and concrete-implementation overrides admitted |
| `backend.validation.clip-region` | A2-M6 all four coordinates finite; derived width/height finite and `>0` | Geometry, SVG, and null clip implementations | `Error::Render`; no clamp/substitution | `geometry`; rejects invalid clip before drawing | Clip-region geometry validation | A2-M6; finite/positive branches admitted |
| `backend.validation.primitive-points` | A2-M6 each supplied point x/y finite; the function accepts empty slices | Scatter and gradient validate unconditionally; SVG, Geometry, and Null line/polygon paths return `Ok` before validation for fewer than two/three points, and Deferred delegates those semantics | Indexed `Error::Render` for consumed points; degenerate line/polygon inputs, including nonfinite points or invalid styles, are explicit current no-ops | `geometry`; rejects invalid consumed primitive geometry but bypasses degenerate primitives | Primitive-coordinate validation and degenerate-primitive boundary | A2-M6 finite loop admitted; concrete arity branches are T2a3/T2a4 consumer traces |
| `backend.validation.point-style` | A2-M6 opacity finite `[0,1]`; optional outline ratio finite/nonnegative and product finite; radius/color/shape otherwise accepted | A2-C-POINT terminal backends | Draw-time `Error::Render`; no normalization | `geometry`; outline width and point draw | Point-style backend validation | A2-M6; opacity/range/outline branches admitted |
| `backend.validation.line-style` | A2-M6 width finite and `>=0`; color/dash unchecked | A2-C-LINE terminal backends after their fewer-than-two-points early return | Draw-time `Error::Render` only for consumed lines; degenerate lines bypass style validation; zero width accepted | `geometry`; consumed line draw and collision bounds | Line-style backend validation | A2-M6 width branch admitted; concrete arity branches are T2a3/T2a4 consumer traces |
| `backend.validation.fill-style` | A2-M6 stroke width finite/nonnegative and opacity finite `[0,1]`; colors unchecked | A2-C-FILL terminal backends; polygons call it only after their fewer-than-three-points early return, while rounded rectangles call it after geometry validation | Draw-time `Error::Render` only for consumed fills; degenerate polygons bypass style validation; zero stroke and zero opacity accepted | `geometry`; consumed polygon/rectangle draw and bounds | Fill-style backend validation | A2-M6 width/opacity branches admitted; concrete arity branches are T2a3/T2a4 consumer traces |
| `backend.validation.linear-gradient` | A2-M6 finite positive bounds; finite distinct vector with signed-zero equivalence; at least two stops; finite offsets `[0,1]`; exact bitwise 0/1 ends; strict increase | A2-C-GRADIENT terminal backends | First failure returns `Error::Render`; no sorting, endpoint insertion, or fallback fill | `geometry`; rejects malformed gradient before draw/record | Complete private gradient protocol validation | A2-M6; every branch admitted; A2-E-UNIT |
| `backend.validation.text-style` | A2-M6 only rotation must be finite | A2-C-TEXT terminal backends | Empty family and zero size pass; no font/paint validation | `geometry`; text draw/collision may proceed with incomplete style | Minimal current text-style validation | A2-M6; rotation branch admitted |
| `backend.validation.rounded-rectangle` | A2-M6 x/y/width/height/radius finite; dimensions/radius nonnegative; far edges finite; zero dimensions accepted | SVG, geometry, and null rounded-rectangle paths | `Error::Render`; no radius clamp to half extent | `geometry`; rounded-container draw/bounds | Rounded-rectangle geometry validation | A2-M6; finite/nonnegative/far-edge branches admitted |
| `backend.validation.significance-bracket` | A2-M6 finite coordinates; distinct anchors with signed-zero equivalence; width finite/nonnegative; font size positive; gap finite/nonnegative | A2-C-BRACKET terminal backends | `Error::Render`; label content/family/weight/color unchecked | `reserve`, `geometry`, collision, draw | Bracket geometry/style protocol validation | A2-M6; all bracket branches admitted |
| `backend.protocol.scope-cleanup-precedence` | A2-M7 `with_clip`, `with_layer`, `with_layer_role`, `finish_scope`; closure always attempted; primary and cleanup errors combined deterministically | A2-C-SCOPE | Primary error retained; cleanup-only failure replaces successful body; double failure reports both | `none`; affects completion, not geometry | Deterministic balanced-scope cleanup semantics | A2-M7; all result branches admitted; A2-E-SCOPE |

### T2a2 Target State

| Property | Target classification and owner | Target resolution | Target validation | Public surface | Relationships |
| --- | --- | --- | --- | --- | --- |
| `backend.coordinates.projected-point-carrier` | Renderer invariant; private canonical projected-coordinate responsibility | Final private draw boundary | Geometry validation requires finite coordinates before any backend serialization | Private-only | Current-to-target counterpart `backend.protocol.coordinate-convention`; validation counterpart `backend.validation.primitive-points` |
| `backend.point.fill-paint-carrier` | Renderer invariant; private complete point-paint transport | Final private draw boundary after resolved mark/scale/exact-style precedence | Typed final `Color`; backend protocol accepts one RGBA paint and never multiplies alpha | Private-only | Default counterparts `marks.point.outlined-constructor-default`, `marks.point.plain-constructor-default`, and `marks.point.explicit-outline-constructor-default`; serialization counterpart `backend.svg.rgb-hex-serialization` |
| `backend.point.geometry-carrier` | Renderer invariant; private point-geometry transport | Final private draw boundary | Plot preflight validates role radius; geometry validation checks representable shape extent | Private-only; semantic shape/radius owners remain later T2 records | Defaults `marks.point.outlined-constructor-default`, `marks.point.plain-constructor-default`, and `marks.point.explicit-outline-constructor-default` |
| `backend.point.outline-carrier` | Renderer invariant; private independent point-outline paint/width transport | Final private draw boundary | Complete `Color` and finite nonnegative resolved width; bounds include the outline | Private-only | Default `marks.point.outline-publication-default`; assembly `backend.point.outline-assembly`; width rule `backend.point.outline-width-resolution` |
| `marks.point.outlined-constructor-default` | Theme component; `MarkTheme` | Resolved Plot/facet; exact point style wins | Complete point-role construction with valid color, alpha, radius, shape, and optional outline | T6 `MarkTheme` | Duplicate-default siblings `marks.point.plain-constructor-default` and `marks.point.explicit-outline-constructor-default`; outline default `marks.point.outline-publication-default` |
| `marks.point.plain-constructor-default` | Theme component; `MarkTheme` | Resolved Plot/facet supplemental point role; exact layer style wins | Complete role construction; no transient overwritten alpha default | T6 `MarkTheme` | Duplicate-default siblings `marks.point.outlined-constructor-default` and `marks.point.explicit-outline-constructor-default` |
| `marks.point.explicit-outline-constructor-default` | Theme component; `MarkTheme` | Resolved Plot/facet before retained outline policy or exact style | Complete role construction; explicit outline replacement validated once | T6 `MarkTheme` | Duplicate-default siblings `marks.point.outlined-constructor-default` and `marks.point.plain-constructor-default`; assembly `backend.point.outline-assembly` |
| `backend.point.outline-assembly` | Renderer invariant; private final point-outline assembly responsibility | Final private draw boundary after resolved Plot and exact-style precedence | Final optional outline is complete; no late default/fallback | Private-only; retained outline policy remains T2b-owned | Carrier `backend.point.outline-carrier`; constructor counterpart `marks.point.explicit-outline-constructor-default` |
| `backend.point.local-mutator-replacement` | Renderer invariant; private final point-style replacement semantics | Exact element then final private draw boundary | Shape and alpha replacement validate independently; alpha replacement never multiplies | Private-only; retained current controls remain owned by their exact styles | Carrier counterparts `backend.point.fill-paint-carrier` and `backend.point.geometry-carrier` |
| `marks.point.outline-publication-default` | Theme component; `MarkTheme` | Resolved Plot/facet | Complete outline `Color` and finite positive width when present; outline absence is structural | T6 `MarkTheme` | Carrier `backend.point.outline-carrier`; width rule `backend.point.outline-width-resolution`; duplicate current source in `marks.point.outlined-constructor-default` |
| `backend.point.outline-width-resolution` | Renderer invariant; private resolved point-outline geometry | Final private draw boundary | Finite nonnegative width and finite extent; geometry and drawing consume the same result | Private-only | Carrier `backend.point.outline-carrier`; default `marks.point.outline-publication-default`; validation `backend.validation.point-style` |
| `backend.line.stroke-paint-carrier` | Renderer invariant; private complete line-paint transport | Final private draw boundary after theme or exact-style resolution | Typed final `Color`; one stroke alpha, no element-wide opacity multiplication | Private-only | Width `backend.line.stroke-width-carrier`; dash `backend.line.dash-string-carrier`; serialization `backend.svg.rgb-hex-serialization` |
| `backend.line.stroke-width-carrier` | Renderer invariant; private line-width transport | Final private draw boundary | Finite nonnegative backend width; semantic owners enforce positive widths where absence is not allowed | Private-only | Paint `backend.line.stroke-paint-carrier`; validation `backend.validation.line-style` |
| `backend.line.dash-string-carrier` | Renderer invariant; private typed line-pattern encoding responsibility | Final private draw boundary | Closed pattern converts to a validated backend encoding; arbitrary backend strings are not accepted | Private-only | Constructor `backend.line.pattern-constructor-branches`; validation `backend.validation.line-style` |
| `backend.line.pattern-constructor-branches` | Renderer invariant; private solid/dashed pattern assembly | Final private draw boundary | Closed pattern is structurally valid; no empty-string sentinel exposed beyond the boundary | Private-only | Carrier `backend.line.dash-string-carrier` |
| `backend.fill.paint-carrier` | Renderer invariant; private complete fill-paint transport | Final private draw boundary | Typed final `Color`; alpha represented exactly once | Private-only | Stroke `backend.fill.stroke-carrier`; serialization `backend.svg.rgb-hex-serialization`; validation `backend.validation.fill-style` |
| `backend.fill.stroke-carrier` | Renderer invariant; private complete independent fill-stroke transport | Final private draw boundary | Typed final stroke `Color` and finite nonnegative width; structural absence replaces zero-width sentinels where owned upstream | Private-only | Fill `backend.fill.paint-carrier`; validation `backend.validation.fill-style` |
| `backend.gradient.stop-carrier` | Renderer invariant; private final gradient-stop transport | Final private draw boundary after theme or plot-local scale resolution | Ordered complete stops; each final stop has one RGBA paint, with opaque data-scale colors yielding alpha 255 | Private-only | Protocol `backend.protocol.gradient-contract`; validation `backend.validation.linear-gradient`; serialization `backend.svg.rgb-hex-serialization` |
| `backend.text.anchor-default` | Renderer invariant; private text-anchor responsibility | Final private draw boundary | Closed Start/Middle/End mapping; no invalid state | Private-only | Placement carrier `backend.text.placement-carrier` |
| `backend.text.paint-carrier` | Renderer invariant; private complete text-paint transport | Final private draw boundary after resolved typography/exact style | Typed final `Color`; automatic contrast candidates already resolved | Private-only | Source counterparts `typography.plot.current-foreground-text`, `typography.annotation.current-foreground`, `typography.legend-guide.current-foreground-text`, `typography.facet-header.current-foreground-text`, `typography.canvas-foundation.current-foreground-text`, and `typography.canvas-composition.current-foreground-text`; serialization `backend.svg.rgb-hex-serialization` |
| `backend.text.typography-carrier` | Renderer invariant; private complete resolved text-draw carrier | Final private draw boundary after preflight font resolution | Nonempty validated stack, positive size/line height, admitted weight, and resolved face per glyph | Private-only | Placement `backend.text.placement-carrier`; validation `backend.validation.text-style`; lifecycle counterparts `font-resolution.current-global-family-order`, `font-resolution.load-system-fonts`, and `font-resolution.global-sans-serif-remap` |
| `backend.text.placement-carrier` | Renderer invariant; private text anchoring and rotation | Final private draw boundary | Rotation finite; transformed bounds and final drawing share the same anchor/orientation | Private-only | Anchor `backend.text.anchor-default`; validation `backend.validation.text-style` |
| `backend.bracket.geometry-carrier` | Renderer invariant; private significance-bracket coordinate/orientation protocol | Final private draw boundary after annotation planning | Finite distinct rendered anchors and representable orientation geometry | Private-only | Metrics `annotations.significance.tick-height-default`, `annotations.significance.data-gap-default`, `annotations.significance.level-step-default`, and `annotations.significance.panel-padding-default`; validation `backend.validation.significance-bracket` |
| `backend.layer.semantic-role` | Renderer invariant; private layer-order role | Final private draw boundary | Closed Generic/Grid role; balanced layer protocol rejects invalid nesting | Private-only | Protocol counterparts `backend.protocol.ordinary-layer-fallbacks` and `backend.protocol.grid-role-delegation` |
| `annotations.significance.tick-height-default` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Finite nonnegative significance geometry at construction and representable final bounds | T6 `AnnotationTheme` | Draw carrier `backend.bracket.geometry-carrier`; spacing siblings `annotations.significance.data-gap-default`, `annotations.significance.level-step-default`, and `annotations.significance.panel-padding-default` |
| `annotations.significance.label-gap-default-copy` | Theme component; `AnnotationTheme` | Resolved Plot/facet from one canonical default | Finite nonnegative gap at construction | T6 `AnnotationTheme` | Duplicate of final carrier source `backend.bracket.label-gap-carrier`; current duplicate is removed |
| `typography.significance.font-size-default-copy` | Theme component; `TypographyTheme` annotation text role | Resolved Plot/facet | Finite positive size and valid line height; face/glyph preflight | T6 annotation `TextTheme` | Duplicate of final carrier source `backend.bracket.typography-carrier`; current duplicate is removed |
| `annotations.significance.data-gap-default` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Finite nonnegative spacing; Plot preflight validates fit/range expansion | T6 `AnnotationTheme` | Siblings `annotations.significance.tick-height-default`, `annotations.significance.level-step-default`, and `annotations.significance.panel-padding-default` |
| `annotations.significance.level-step-default` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Finite nonnegative spacing; Plot preflight validates all annotation levels fit | T6 `AnnotationTheme` | Siblings `annotations.significance.data-gap-default` and `annotations.significance.panel-padding-default` |
| `annotations.significance.panel-padding-default` | Theme component; `AnnotationTheme` | Resolved Plot/facet | Finite nonnegative spacing; Plot preflight validates panel fit | T6 `AnnotationTheme` | Siblings `annotations.significance.data-gap-default` and `annotations.significance.level-step-default` |
| `backend.bracket.combined-color-stroke-meaning` | Renderer invariant; private bracket-stroke paint transport | Final private draw boundary after resolved annotation appearance | Complete bracket-stroke `Color`; independent from label text | Private-only | Split sibling `backend.bracket.combined-color-text-meaning`; semantic source counterpart `annotations.stroke.current-foreground` |
| `backend.bracket.combined-color-text-meaning` | Renderer invariant; private bracket-label text-paint transport | Final private draw boundary after resolved annotation typography | Complete annotation text `Color`; face/glyph/fit validation remains with the typography carrier | Private-only | Split sibling `backend.bracket.combined-color-stroke-meaning`; semantic source counterpart `typography.annotation.current-foreground` |
| `backend.bracket.stroke-width-carrier` | Renderer invariant; private bracket-width transport | Final private draw boundary | Finite nonnegative final width; bounds include stroke extent | Private-only | Theme source counterpart `annotations.significance-bracket-width.current-field`; validation `backend.validation.significance-bracket` |
| `backend.bracket.typography-carrier` | Renderer invariant; private resolved bracket-label text carrier | Final private draw boundary | Complete resolved text style; positive size and glyph availability before drawing | Private-only | Default `typography.significance.font-size-default-copy`; text paint `backend.bracket.combined-color-text-meaning` |
| `backend.bracket.label-gap-carrier` | Renderer invariant; private final bracket-label spacing transport | Final private draw boundary | Finite nonnegative resolved spacing, identical in planning and drawing | Private-only | Duplicate default `annotations.significance.label-gap-default-copy`; validation `backend.validation.significance-bracket` |
| `backend.protocol.coordinate-convention` | Renderer invariant; canonical backend coordinate and text-position responsibility | Before geometry discovery and final private drawing | Non-text and text geometry use one finite signed bottom-up representation with sufficient precision for admitted geometry; signed off-canvas text remains representable, and measurement/drawing consume the same canonical post-conversion values | Private-only | Carrier `backend.coordinates.projected-point-carrier`; text placement `backend.text.placement-carrier`; concrete conversion `backend.svg.coordinate-conversion-and-precision` |
| `backend.protocol.begin-end-lifecycle` | Renderer invariant; balanced backend canvas lifecycle | Final private draw boundary | Exactly one successful begin before draw and one attempted end; no draw outside an open canvas | Private-only | Scope cleanup `backend.protocol.scope-cleanup-precedence`; layer `backend.protocol.ordinary-layer-fallbacks`; clip `backend.protocol.clip-scope-contract` |
| `backend.protocol.ordinary-layer-fallbacks` | Renderer invariant; balanced named-layer protocol | Final private draw boundary | Backends must preserve balance even if grouping has no native representation | Private-only | Role `backend.layer.semantic-role`; grid delegation `backend.protocol.grid-role-delegation`; cleanup `backend.protocol.scope-cleanup-precedence` |
| `backend.protocol.grid-role-delegation` | Renderer invariant; grid-behind-marks ordering responsibility | Final private draw boundary | Canonical backend enforces grid order; non-output planning backends may ignore paint order while preserving scopes | Private-only | Role `backend.layer.semantic-role`; ordinary layers `backend.protocol.ordinary-layer-fallbacks` |
| `backend.protocol.clip-scope-contract` | Renderer invariant; nested clipping and LIFO responsibility | Final private draw boundary | Finite positive regions; every active clip participates; imbalance rejects | Private-only | Validation `backend.validation.clip-region`; cleanup `backend.protocol.scope-cleanup-precedence` |
| `backend.protocol.gradient-contract` | Renderer invariant; native gradient draw protocol | Final private draw boundary | Complete validated vector, bounds, and stops; no fallback interpolation/backend selection | Private-only | Stop carrier `backend.gradient.stop-carrier`; validation `backend.validation.linear-gradient` |
| `backend.protocol.rounded-rectangle-fallback` | Renderer invariant; explicit rounded-rectangle backend responsibility | Final private draw boundary | Every admitted backend implements rounded rectangles explicitly; unsupported behavior fails backend admission rather than degrading to a polygon at draw time | Private-only | Validation `backend.validation.rounded-rectangle`; fill carriers `backend.fill.paint-carrier` and `backend.fill.stroke-carrier` |
| `backend.validation.clip-region` | Renderer invariant; clip-region protocol validation | Final private draw boundary | Finite coordinates and strictly positive finite extents; reject with no substitution | Private-only | Protocol `backend.protocol.clip-scope-contract` |
| `backend.validation.primitive-points` | Renderer invariant; primitive-coordinate and arity validation | Final private draw boundary | Scatter may be empty; lines with fewer than two points and polygons with fewer than three are explicit no-ops that consume neither coordinates nor style; every consumed coordinate is finite | Private-only | Carrier `backend.coordinates.projected-point-carrier`; style counterparts `backend.validation.line-style` and `backend.validation.fill-style` |
| `backend.validation.point-style` | Renderer invariant; final point-style protocol validation | Final private draw boundary | Complete RGBA paint, representable geometry, finite nonnegative outline; target owners validate stronger role constraints earlier | Private-only | Carriers `backend.point.fill-paint-carrier`, `backend.point.geometry-carrier`, and `backend.point.outline-carrier` |
| `backend.validation.line-style` | Renderer invariant; final line-style protocol validation | Final private draw boundary | Every consumed nondegenerate line has complete paint, finite nonnegative width, and a valid closed pattern; reject rather than normalize | Private-only | Carriers `backend.line.stroke-paint-carrier`, `backend.line.stroke-width-carrier`, and `backend.line.dash-string-carrier`; arity policy `backend.validation.primitive-points` |
| `backend.validation.fill-style` | Renderer invariant; final fill/stroke protocol validation | Final private draw boundary | Every consumed nondegenerate fill has independent complete RGBA paints and finite nonnegative stroke geometry | Private-only | Carriers `backend.fill.paint-carrier` and `backend.fill.stroke-carrier`; arity policy `backend.validation.primitive-points` |
| `backend.validation.linear-gradient` | Renderer invariant; complete gradient protocol validation | Final private draw boundary | Positive finite bounds, distinct finite vector, at least two exact ordered complete stops spanning 0..1; reject without repair | Private-only | Carrier `backend.gradient.stop-carrier`; protocol `backend.protocol.gradient-contract` |
| `backend.validation.text-style` | Renderer invariant; complete resolved text protocol validation | Final private draw boundary after Plot/Canvas font preflight | Complete paint, nonempty resolved stack, positive metrics, admitted weight, finite rotation; no late fallback | Private-only | Carriers `backend.text.paint-carrier`, `backend.text.typography-carrier`, and `backend.text.placement-carrier` |
| `backend.validation.rounded-rectangle` | Renderer invariant; rounded-rectangle geometry validation | Final private draw boundary | Finite geometry, nonnegative dimensions/radius, finite far edges, and radius no greater than half either extent; reject oversized radii rather than relying on backend normalization | Private-only | Explicit implementation requirement `backend.protocol.rounded-rectangle-fallback` |
| `backend.validation.significance-bracket` | Renderer invariant; bracket geometry/style protocol validation | Final private draw boundary after Plot annotation preflight | Finite distinct anchors, complete stroke/text styles, positive text metrics, finite nonnegative gap; no fallback | Private-only | Carrier `backend.bracket.geometry-carrier`; style carriers `backend.bracket.stroke-width-carrier`, `backend.bracket.typography-carrier`, and `backend.bracket.label-gap-carrier` |
| `backend.protocol.scope-cleanup-precedence` | Renderer invariant; deterministic balanced-scope cleanup responsibility | Final private draw boundary | Always attempt one matching close; preserve primary and cleanup failures without alternate rendering | Private-only | Lifecycle `backend.protocol.begin-end-lifecycle`; layers `backend.protocol.ordinary-layer-fallbacks`; clips `backend.protocol.clip-scope-contract` |

### T2a2 Migration And Evidence

| Property | Disposition and owning phase | Visual evidence |
| --- | --- | --- |
| `backend.coordinates.projected-point-carrier` | `retain`, T3 | No output change expected; A2-E-SCOPE projected-coordinate and clipping assertions |
| `backend.point.fill-paint-carrier` | `replace`, T6 | Intentional RGBA/alpha correction; exact point SVG paint and PNG alpha assertions in A2-E-PAINT |
| `backend.point.geometry-carrier` | `retain`, T3 | No output change expected; radius/shape SVG and A2-E-GEOMETRY bounds assertions |
| `backend.point.outline-carrier` | `replace`, T6 | Intentional independent outline-alpha and complete-bounds correction; default/disabled outline assertions |
| `marks.point.outlined-constructor-default` | `move`, T5c | No built-in output change; raw-overlay and default-outline snapshots |
| `marks.point.plain-constructor-default` | `replace`, T5c | No output change; supplemental point-layer default and opacity assertions |
| `marks.point.explicit-outline-constructor-default` | `move`, T5c | No built-in output change; primary point and legend-glyph snapshots |
| `backend.point.outline-assembly` | `connect`, T5c | No output change; retained outline-policy precedence assertions |
| `backend.point.local-mutator-replacement` | `retain`, T6 | No output change; exact shape and alpha replacement-order tests |
| `marks.point.outline-publication-default` | `deduplicate`, T5c | No output change; one resolved white-outline source and A2-E-PAINT assertions |
| `backend.point.outline-width-resolution` | `connect`, T5c | Intentional bounds correction; outline width SVG and collision assertions |
| `backend.line.stroke-paint-carrier` | `replace`, T6 | Intentional line-alpha support; exact stroke paint/alpha assertions |
| `backend.line.stroke-width-carrier` | `retain`, T3 | No output change; width and local-bounds assertions |
| `backend.line.dash-string-carrier` | `replace`, T3 | No output change; closed solid/dashed SVG and malformed-pattern rejection tests |
| `backend.line.pattern-constructor-branches` | `replace`, T3 | No output change; exact absence/presence of `stroke-dasharray` in A2-E-PAINT |
| `backend.fill.paint-carrier` | `replace`, T6 | Intentional exact RGBA correction; polygon/rounded-rectangle fill-alpha assertions |
| `backend.fill.stroke-carrier` | `replace`, T6 | Intentional independent stroke-alpha support; fill-outline SVG/PNG assertions |
| `backend.gradient.stop-carrier` | `replace`, T6 | No output change for opaque built-ins and intentional stop-alpha support; A2-E-UNIT plus exact RGBA heatmap/beeswarm gradient evidence; T5d owns scale connection |
| `backend.text.anchor-default` | `retain`, T3 | No output change; Start/Middle/End assertions in A2-E-TEXT |
| `backend.text.paint-carrier` | `replace`, T6 | Intentional text-alpha support; exact preserve-text SVG and PNG assertions |
| `backend.text.typography-carrier` | `replace`, T5a | Intentional measurement/draw synchronization; font-stack, glyph, weight, and long-text evidence |
| `backend.text.placement-carrier` | `retain`, T3 | No ordinary output change; anchor/rotation and transformed-bounds tests |
| `backend.bracket.geometry-carrier` | `retain`, T3 | No output change; vertical/horizontal bracket and clipping assertions |
| `backend.layer.semantic-role` | `retain`, T3 | No output change; grid-before-marks and balanced-layer evidence in A2-E-SCOPE |
| `annotations.significance.tick-height-default` | `move`, T5c | No built-in output change; exact bracket path and collision-overhang assertions |
| `annotations.significance.label-gap-default-copy` | `deduplicate`, T5c | No output change; one resolved gap reaches planning and draw |
| `typography.significance.font-size-default-copy` | `deduplicate`, T5a | No output change; one resolved annotation text size reaches measurement and draw |
| `annotations.significance.data-gap-default` | `move`, T5c | No intended output change; automatic-range and inside-legend significance tests |
| `annotations.significance.level-step-default` | `move`, T5c | No intended output change; multi-level significance layout snapshots |
| `annotations.significance.panel-padding-default` | `move`, T5c | No intended output change; panel-ceiling and fit/rejection tests |
| `backend.bracket.combined-color-stroke-meaning` | `replace`, T5c | No built-in paint change; resolved AnnotationTheme paint reaches an independent private bracket-stroke carrier |
| `backend.bracket.combined-color-text-meaning` | `replace`, T5a | No built-in paint change; resolved annotation text paint reaches an independent private bracket-label carrier |
| `backend.bracket.stroke-width-carrier` | `connect`, T5c | Intentional conservative-bounds correction if stroke overhang matters; exact width and collision tests |
| `backend.bracket.typography-carrier` | `replace`, T5a | Intentional font-metric correction; significance glyph/weight/layout evidence |
| `backend.bracket.label-gap-carrier` | `connect`, T5c | No intended output change; planning-to-draw gap equality assertion |
| `backend.protocol.coordinate-convention` | `replace`, T3 | Ordinary output remains stable; signed text-position, subprecision representability, shared Geometry/SVG post-conversion values, and y-orientation assertions |
| `backend.protocol.begin-end-lifecycle` | `retain`, T3 | No output change; exactly-one begin/end and failure-path tests in A2-E-SCOPE |
| `backend.protocol.ordinary-layer-fallbacks` | `connect`, T3 | No output change; balanced no-op backend and canonical group evidence |
| `backend.protocol.grid-role-delegation` | `retain`, T3 | No output change; Canvas-wide grid-before-marks assertions |
| `backend.protocol.clip-scope-contract` | `retain`, T3 | No output change; nested LIFO, invalid-region, and panel-clipping evidence |
| `backend.protocol.gradient-contract` | `retain`, T3 | No output change; A2-E-UNIT and native-gradient snapshots |
| `backend.protocol.rounded-rectangle-fallback` | `remove`, T3 | No current output change; compile-time explicit implementations for Deferred, Geometry, Null, and SVG plus exact rounded SVG evidence |
| `backend.validation.clip-region` | `retain`, T3 | No successful-output change; finite/nonpositive clip rejection tests |
| `backend.validation.primitive-points` | `retain`, T3 | No successful-output change; nonfinite consumed-primitive rejection and explicit degenerate no-op tests |
| `backend.validation.point-style` | `replace`, T6 | No ordinary output change; RGBA, alpha-bound, and outline-width rejection tests |
| `backend.validation.line-style` | `connect`, T3 | No successful-output change; degenerate bypass, NaN/negative width, and invalid closed-pattern tests |
| `backend.validation.fill-style` | `replace`, T6 | No ordinary output change; degenerate polygon bypass plus RGBA, width, and alpha rejection tests |
| `backend.validation.linear-gradient` | `retain`, T3 | No output change; every branch covered by A2-E-UNIT |
| `backend.validation.text-style` | `replace`, T5a | Intentional earlier rejection and font-resolution correction; empty/zero/missing-glyph tests |
| `backend.validation.rounded-rectangle` | `replace`, T3 | Intentional invalid-input correction; finite/nonnegative/far-edge and oversized-radius rejection tests, with ordinary rounded-container snapshots unchanged |
| `backend.validation.significance-bracket` | `retain`, T3 | No successful-output change; invalid-anchor/style tests and significance snapshots |
| `backend.protocol.scope-cleanup-precedence` | `retain`, T3 | No output change; primary-plus-cleanup and cleanup-only error assertions in A2-E-SCOPE |

### Rejected T2a2 Matches

| Source match | Disposition and reason |
| --- | --- |
| `src/render/backend.rs:1-16` module prose, `deny(missing_docs)`, and imports | Documentation/language plumbing only; coordinate semantics begin at A2-M1. |
| Non-`Default` derives and visibility on carrier declarations | Language/API mechanics; no independent paint, geometry, default, or behavior. The `Default` derives and `#[default]` variants on `TextAnchor` and `LayerRole` are admitted by their keyed records. |
| Documentation examples such as dash `"4 2"`, empty-string prose, opacity ranges, and bracket glyph examples | Not executable defaults. Executable empty/dashed branches, ranges, and label transport are recorded separately. |
| `src/render/backend.rs:541` `MAGNITUDE_BITS = 0x7fff_ffff_ffff_ffff` | Non-visual bit mask implementing signed-zero equivalence; the behavior is included in `backend.validation.linear-gradient` and `backend.validation.significance-bracket`. |
| Error-message strings and interpolated indices/values in A2-M6/A2-M7 | Diagnostic wording is non-visual; each failure branch and cleanup precedence is recorded. |
| `Vec` allocation and four fallback polygon points in `draw_rounded_rect` | The allocation is non-visual; the four-corner no-rounding fallback is `backend.protocol.rounded-rectangle-fallback`. |
| Plot, layout, legend, annotation, and scale colors, widths, radii, fonts, labels, bounds, and dash choices passed into backend carriers | Consumer references owned by T2b-T2g. T2a2 owns only the carrier, defaults declared in A2-M2/A2-M4, validation, and protocol behavior. |
| `src/render/backend.rs:668-729` entire `#[cfg(test)]` module | Test-only. Black/white/gray RGB tuples, offsets, panel bounds, NaN, and stop arrays are A2-E-UNIT evidence, not property sources. |
| Scientific/statistical data search class | No production declaration in the T2a2 manifest; projected coordinates and annotation labels are carried data, not appearance defaults. |

### T2a2 Reconciliation

- The completed T2a2 count is exactly **52**. Every descriptive key appears
  once in each keyed current-state, target-state, and migration/evidence table.
- T2a1 and T2a2 contain exactly **117 completed T2a records**. T2a3 brought the
  completed count to 162; T2a4 source reconciliation corrects and completes the
  four-part T2a plan at **222 records**.
- Source reconciliation required no net split correction to the initial 52.
  The multi-owner `BracketStyle::color` source is already represented by
  `backend.bracket.combined-color-stroke-meaning` and
  `backend.bracket.combined-color-text-meaning`. Other grouped declarations have
  one target class, owner, and owning migration phase.
- RGB-plus-separate-opacity fields remain explicit current defects rather than
  being mistaken for RGBA. Point, line, fill, text, outline, bracket, and
  gradient carriers each have one final private-boundary responsibility.
- No plot, layout, legend, annotation-content, or scale value merely passed
  through a carrier is assigned to T2a2. Significance metric literals are owned
  only because A2-M4 declares those executable defaults.
- No target field is unresolved and no future public type name is introduced.
  Component owners are limited to T1-approved types; private responsibilities
  use descriptive renderer-invariant names.

## T2a3: Serialization, Geometry Recording, And Scene Conversion

This section owns concrete SVG serialization, PNG paint/alpha serialization,
geometry-obstacle recording, immutable scene reuse, and scene conversion-limit
sources in `src/render.rs`, `src/render/svg_backend.rs`,
`src/render/geometry.rs`, and `src/scene.rs`. T2a2 remains the sole owner of
abstract backend carriers, validation, and protocol declarations. T2a4 owns PNG
sizing, options, transforms, compression, DPI metadata, format-local output
timing, and artifact publication even where those contexts live in
`svg_backend.rs`.

### T2a3 Source Manifest

| ID | Production source | Owned context | Audit disposition |
| --- | --- | --- | --- |
| A3-M1 | `src/render.rs:1-8` | Render module documentation and module declarations | Rejected: no visual declaration, literal, fallback, or behavior branch. |
| A3-M2 | `src/render/svg_backend.rs:18-98` | SVG construction, lifecycle helpers, coordinate conversion, buffer routing, escaping, and group state | Records `backend.svg.concrete-canvas-lifecycle`, `backend.svg.xml-escaping`, `backend.svg.coordinate-conversion-and-precision`, and part of `backend.svg.layer-buffering-and-grid-order`. |
| A3-M3 | `src/render/svg_backend.rs:99-123` | Preserve-text SVG to outlined-SVG conversion | Record `backend.svg.text-outline-conversion`. |
| A3-M4 | `src/render/svg_backend.rs:125-299` | Mixed PNG conversion and encoding | Record `backend.png.rgba8-paint-serialization`; sizing, options, transforms, compression, DPI, timing, and publication contexts are deferred to T2a4. |
| A3-M5 | `src/render/svg_backend.rs:301-435` | Concrete SVG state machine, layers, and clips | Completes `backend.svg.concrete-canvas-lifecycle` and `backend.svg.layer-buffering-and-grid-order`; records `backend.svg.clip-scope-serialization`. |
| A3-M6 | `src/render/svg_backend.rs:436-583` | Scatter, line, polygon, and gradient serialization | Records from `backend.svg.scatter-glyph-serialization` through `backend.svg.linear-gradient-serialization`, plus their paint contexts in `backend.svg.incomplete-per-paint-alpha-serialization`. |
| A3-M7 | `src/render/svg_backend.rs:585-718` | Rounded rectangle, generic text, and bracket serialization | Records from `backend.svg.rounded-rectangle-serialization` through `backend.svg.significance-bracket-label-serialization`, plus their paint contexts in `backend.svg.incomplete-per-paint-alpha-serialization`. |
| A3-M8 | `src/render/svg_backend.rs:719-758` | SVG finalization, root surface, and document framing | Completes `backend.svg.concrete-canvas-lifecycle` and `backend.svg.layer-buffering-and-grid-order`; records `surface.canvas.backend-white-root` and `backend.svg.document-framing`. |
| A3-M9 | `src/render/svg_backend.rs:759-890` | Test module | Excluded as a property source; retained as evidence only. |
| A3-M10 | `src/render/geometry.rs:1-81` | Collision weight, obstacle calculations, and recorder state | Records from `geometry.collision.annotation-weight` through `geometry.obstacle-recording-order`. |
| A3-M11 | `src/render/geometry.rs:83-157` | Geometry lifecycle, clip stack, and primitive bounds | Records from `geometry.backend-lifecycle` through `geometry.gradient-bounds`. |
| A3-M12 | `src/render/geometry.rs:159-265` | Text, rounded-rectangle, bracket bounds, and end lifecycle | Records `geometry.text-bounds`, `geometry.rounded-rectangle-bounds`, `geometry.significance-bracket-path-bounds`, and `geometry.significance-bracket-label-bounds`; completes `geometry.backend-lifecycle`. |
| A3-M13 | `src/render/geometry.rs:267-361` | Test module | Excluded as a property source; retained as evidence only. |
| A3-M14 | `src/scene.rs:1-17` | Conversion-limit declarations and immutable scene carrier | Records `scene.canonical-immutable-storage` and the declaration portions of the four maximum-limit records. |
| A3-M15 | `src/scene.rs:19-28` | `Debug` implementation | Rejected: diagnostic representation only. |
| A3-M16 | `src/scene.rs:30-105` | Scene construction, terminal routes, and conversion-input limits | Records from `scene.canonical-immutable-storage` through `scene.conversion-byte-limit`. |
| A3-M17 | `src/scene.rs:107-124` | Computed PNG target validation | Records target branches of `scene.export-dimension-limit` and `scene.target-pixel-work-limit`, plus `scene.png-target-positive-finite-limit`. |

The manifest is exact for T2a3. Mixed expressions are partitioned by semantic
responsibility: for example, transparent pixmap paint and RGBA8 bytes are T2a3,
while their target dimensions, transform, compression, metadata, and publication
are T2a4.

### T2a3 Consumer And Evidence References

The compact codes below expand into each record that cites them.

| Code | Exact production consumers or evidence |
| --- | --- |
| A3-C-SVG-LIFECYCLE | `src/orchestrator/output.rs:11-19` and `src/canvas.rs:421-430` construct an in-memory `SvgBackend`, render one canvas, finish it, and consume its bytes; every concrete SVG draw call passes through the same begun-state and finalization state machine. |
| A3-C-SVG-DRAW | T2a2 references A2-C-POINT, A2-C-LINE, A2-C-FILL, A2-C-TEXT, A2-C-GRADIENT, A2-C-BRACKET, and A2-C-SCOPE identify every producer; `render/svg_backend.rs` is their final concrete SVG serializer. |
| A3-C-OUTLINE | `RenderedScene::svg_with_text_as_paths` at `scene.rs:56-62` is called by `Output::svg` at `output.rs:21-25`; T2a1 C-host-fonts identifies the host-font consumer path. |
| A3-C-PNG | `RenderedScene::png_bytes` at `scene.rs:70-78` is called by `Output::png_with_options` at `output.rs:50-59`; PNG sizing/options are T2a4 consumers while SVG paint and alpha reach the RGBA8 pixmap/encoder path recorded here. |
| A3-C-GEOMETRY | `decorations/plan.rs:136-346,636,720` constructs fresh default `GeometryBackend` values, draws directly without calling `begin` or `end`, reads ordered bounds, scores overlap, and selects legend placement; the concrete lifecycle methods are dormant in production, while A2-C-POINT through A2-C-BRACKET identify primitive producers. |
| A3-C-SCENE | `orchestrator/output.rs:11-19` and `canvas.rs:421-430` construct `RenderedScene`; `output.rs:13-59` reuses it for outlined SVG, exact SVG, and PNG terminal calls. |
| A3-E-SVG | `render/svg_backend.rs:759-890`, `tests/rendering/contract.rs`, `tests/rendering/labels.rs`, `tests/rendering/public_contracts.rs`, and visual-regression snapshots cover outlining, clipping, group balance, deterministic gradients, geometry, text, and paint attributes. |
| A3-E-GEOMETRY | `render/geometry.rs:267-361`, `decorations/plan/tests.rs`, and `tests/layout/layout_and_legends.rs` cover local obstacles, gradient rectangles, bracket overhang, clipping, overlap, and collision-driven legend placement. |
| A3-E-OUTPUT | `tests/output/multiformat.rs`, `tests/output/png.rs`, `tests/composition/canvases.rs`, and exact SVG snapshots cover one-scene reuse, preserve/outlined SVG, PNG RGBA output, dimensions, and conversion failures. |
| A3-E-ALPHA | Exact SVG paint assertions in `tests/rendering/public_contracts.rs`, `tests/lines/effect_bands.rs`, `tests/interactions/styles.rs`, and PNG tests cover current fill opacity and target per-paint alpha behavior. |

### T2a3 Current State

| Property | Current source and value | Current consumers | Current behavior | Current layout path | Semantic role | Audit evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `backend.svg.concrete-canvas-lifecycle` | A3-M2/A3-M5/A3-M8: `in_memory`, `into_bytes`, `ensure_begun`, `SvgBackend` lifecycle fields, `begin`, and begun-state portions of `end`; dimensions are `u32`, buffers/IDs reset on begin, but `begin` does not clear a prior successful `output` | A3-C-SVG-LIFECYCLE | Rejects extraction before the first finish, reentrant begin, zero dimensions, operations outside begin/end, and end before begin; after one successful session, `into_bytes` can return stale prior bytes during a second open session or after a failed second begin/render because only successful `end` replaces `output` | `none`; gates all final output | Concrete SVG serializer state machine | A3-M2/A3-M5/A3-M8; stale-output and every lifecycle branch admitted; diagnostic wording rejected |
| `backend.svg.xml-escaping` | A3-M2 `escape_text` maps `&`, `<`, `>`, `"`, and `'` to XML entities and copies every other character | A3-C-SVG-DRAW | Applied to layer IDs, text payloads, font-family attributes, and bracket labels; no alternate escaping or fallback | `none`; emitted text/attributes only | XML-safe SVG textual serialization | A3-M2; all character branches admitted; A3-E-SVG |
| `backend.svg.coordinate-conversion-and-precision` | A3-M2/A3-M5-A3-M7: `y_mark = height - y`, `fmt_points` emits `.3` x/y pairs, gradient stops use `.6`, other method-local coordinates/extents use fixed `.1`/`.2`/`.3`, and text y is rounded then cast to `u32` | A3-C-SVG-DRAW | Final formatting can collapse distinct points, bracket anchors, gradient endpoints or stops, and positive clip/gradient/rectangle extents into equal or zero serialized values; when `y_pixel > height`, negative text y saturates to `0` on the `u32` cast instead of preserving an off-canvas position | `geometry`; affects every SVG primitive and text position | Canonical math-space-to-SVG coordinate and precision rule | A3-M2/A3-M5-A3-M7; T2a2 coordinate carrier is prior-owned; post-format collapse and saturation admitted; A3-E-SVG |
| `backend.svg.text-outline-conversion` | A3-M3 `convert_svg_text_to_paths`: parse exact canonical bytes with configured host fonts, require parsed dimensions equal supplied CSS dimensions, serialize the converted tree, reject remaining `<text>` | A3-C-OUTLINE | Font discovery occurs after canonical SVG layout; parse, dimension, or residual-text failure returns `Error::Render`; no alternate converter | `geometry`; glyph paths can differ from prior estimated text layout | Preserve-text SVG to generation-host glyph paths | A3-M3; T2a1 owns font discovery; A3-E-OUTPUT |
| `backend.png.rgba8-paint-serialization` | A3-M4 paint aspect of transparent `Pixmap::new` plus `resvg::render`; encoder uses `png::ColorType::Rgba`, `BitDepth::Eight`, and `take_demultiplied` | A3-C-PNG | Raster paint composites into an RGBA8 pixmap and writes straight-alpha bytes; current hardcoded white SVG root normally makes root pixels opaque | `none`; paint/alpha only | Canonical PNG RGBA8 paint and alpha serialization | A3-M4; sizing/publication portions deferred; A3-E-OUTPUT/A3-E-ALPHA |
| `backend.svg.layer-buffering-and-grid-order` | A3-M2/A3-M5/A3-M8: `buf`, `GroupState::{Grid,Layer}`, layer methods, grid/top-level branches, unclosed-scope rejection, and grid buffer emission before body | A3-C-SVG-LIFECYCLE, A3-C-SVG-DRAW | Ordinary layers emit in call order; Grid must be top-level and is buffered before all body content; layer closure is LIFO and rejects Clip state or missing opens | `none`; changes z-order/grouping only | Concrete SVG layer balance and grid-before-marks order | A3-M2/A3-M5/A3-M8; T2a2 owns abstract layer roles/protocol; A3-E-SVG |
| `backend.svg.clip-scope-serialization` | A3-M5 clip fields and `begin_clip`/`end_clip`: monotonically increasing `clip-{id}`, `.3` rectangle geometry, `<clipPath>` defs, linked `<g>`, and LIFO state branches | A3-C-SVG-DRAW | Valid clips may nest with layers; IDs reset per begin; closure rejects Layer/Grid state or no matching clip | `geometry`; clips visible output | Deterministic concrete SVG clipping | A3-M5; validation is prior-owned by T2a2; A3-E-SVG |
| `backend.svg.scatter-glyph-serialization` | A3-M6 structural `draw_scatter` contexts: Circle emits `<circle>`; TriangleUp uses radius, half-width factor `0.866_025_403_784_438_6`, and three vertices; optional outline adds stroke width formatted `.1`, while triangle polygons omit `stroke-linejoin` and `stroke-miterlimit` | A3-C-SVG-DRAW | One element per supplied point; shape match is closed; outlined triangles inherit SVG's implicit miter join and miter limit `4`, while circles have no corner join; paints are covered by the shared alpha record | `geometry`; final point shape, join, and extent | Concrete SVG point-glyph geometry | A3-M6; transported style values are prior/later-owned; implicit triangle join admitted; A3-E-SVG |
| `backend.svg.line-degenerate-noop` | A3-M6 independent `draw_line` branch `points.len() < 2` returns `Ok(())` before coordinate/style validation | A3-C-SVG-DRAW | Degenerate lines, including invalid carried values, produce no output and no error | `geometry`; explicit no-op | Concrete SVG line arity boundary | A3-M6; concrete branch admitted separately from T2a2 abstract validation and polygon branch |
| `backend.svg.line-structure-and-pattern-serialization` | A3-M6 nondegenerate `draw_line`: `<polyline>`, `fill="none"`, supplied width, round cap/join, and empty dash versus emitted `stroke-dasharray` | A3-C-SVG-DRAW | Empty backend string selects solid; every nonempty string is emitted directly; no parsing or normalization | `geometry` for path/width; pattern is paint-only | Concrete SVG line structure and pattern | A3-M6; line carrier/pattern source is T2a2; A3-E-SVG |
| `backend.svg.polygon-degenerate-noop` | A3-M6 independent `draw_polygon` branch `points.len() < 3` returns `Ok(())` before coordinate/style validation | A3-C-SVG-DRAW | Degenerate polygons, including invalid carried values, produce no output and no error | `geometry`; explicit no-op | Concrete SVG polygon arity boundary | A3-M6; equal line behavior remains a separate independent source |
| `backend.svg.polygon-structure-serialization` | A3-M6 nondegenerate `draw_polygon`: one `<polygon>` with canonical points and supplied stroke width, without `stroke-linejoin` or `stroke-miterlimit` | A3-C-SVG-DRAW | Uses one aggregate polygon element and inherits SVG's implicit miter join and miter limit `4`; paint/alpha attributes are assigned to the shared serializer record | `geometry`; final polygon join and stroke extent | Concrete SVG polygon structure | A3-M6; fill/style values are prior/later-owned; implicit join admitted; A3-E-SVG |
| `backend.svg.linear-gradient-serialization` | A3-M6 `draw_linear_gradient`: monotonically increasing IDs, `userSpaceOnUse`, `.3` vector/bounds, `.6` stop offsets, defs closure, and one gradient-filled rectangle | A3-C-SVG-DRAW | IDs reset per canvas; exact validated stop order is preserved; stop colors are emitted without stop alpha | `geometry` for rectangle/vector; stop paint does not affect layout | Native deterministic SVG gradient serialization | A3-M6; validation/stops are T2a2-owned carriers; A3-E-SVG |
| `backend.svg.rounded-rectangle-serialization` | A3-M7 structural `draw_rounded_rect`: bottom-left math coordinates become SVG top-left, dimensions use `.2`, and `rx`/`ry` use the same radius at `.1` | A3-C-SVG-DRAW | Emits a native rounded `<rect>` with supplied stroke width; no fallback polygon in this implementation | `geometry`; container bounds and corners | Native SVG rounded-rectangle serialization | A3-M7; validation/fallback protocol is T2a2-owned; A3-E-SVG |
| `backend.svg.text-element-serialization` | A3-M7 structural `draw_text`: Start/Middle/End mapping, omitted normal weight, emitted non-normal weight, `xml:space="preserve"` only when `text.contains("  ")`, family/size, middle baseline, and optional `.1` rotation | A3-C-SVG-DRAW | Preserve-text SVG does not request whitespace preservation for a single leading/trailing ASCII space, tabs, or newlines, and only incidentally preserves those when the same text also contains two consecutive ASCII spaces; rotation uses `abs() > EPSILON`; escaping and paint are separate records | `geometry`; final text anchoring/rotation and whitespace may disagree with estimated bounds/content | Generic preserve-text SVG element serialization | A3-M7; text carrier is T2a2-owned; exact whitespace limitations admitted; A3-E-SVG |
| `backend.svg.significance-bracket-path-serialization` | A3-M7 path portions of `draw_bracket`: Vertical/Horizontal path branches, fixed tick geometry from shared metrics, `.3` coordinates, round stroke caps, and supplied width | A3-C-SVG-DRAW | One path contains span plus two end ticks; horizontal overloads bracket coordinates per T2a2 protocol; stroke overhang is not separately planned here | `geometry`; final annotation path | Significance-bracket path serialization | A3-M7; label and paint contexts split; A3-E-SVG |
| `backend.svg.significance-bracket-label-serialization` | A3-M7 label portions of `draw_bracket`: family, size, omitted/explicit weight, gap placement, midpoint, Middle/Start anchors, and horizontal middle baseline; neither orientation emits `xml:space` | A3-C-SVG-DRAW | Label text is serialized separately from generic `draw_text`, never requests whitespace preservation, and uses carried size/family without a measured face | `geometry`; final label placement, whitespace, and extent | Significance-label preserve-text serialization | A3-M7; path is T5c sibling and text metrics are T5a; exact whitespace limitation admitted; A3-E-SVG |
| `backend.svg.incomplete-per-paint-alpha-serialization` | A3-M6/A3-M7 paint-attribute contexts: point and fill `fill-opacity`, polygon/rounded opacity formatted `.2`, and no independent alpha for outlines, lines, fill strokes, gradient stops, text, or bracket path/label | A3-C-SVG-DRAW | RGB comes from prior-owned `rgb_to_hex`; point/fill opacity is emitted once on fills, all other paints are opaque, and no element-wide opacity multiplies it | `none`; final paint only | Incomplete current per-paint SVG alpha serialization | A3-M6/A3-M7; one final serializer responsibility, not duplicate carrier rows; A3-E-ALPHA |
| `surface.canvas.backend-white-root` | A3-M8 literal `<rect width="100%" height="100%" fill="#FFFFFF"/>` | A3-C-SVG-LIFECYCLE, A3-C-PNG | Every completed SVG receives an opaque white root before grid/body, independently of dormant `ThemePalette::background`; PNG consequently starts visually opaque | `none`; root paint only | Backend-owned Canvas/root surface | A3-M8; T2a1 surface counterpart; A3-E-OUTPUT |
| `backend.svg.document-framing` | A3-M8 final assembly: root SVG namespace, width/height/viewBox, `role="img"`, non-scaling-stroke stylesheet, defs order, content order, and closing tag | A3-C-SVG-LIFECYCLE | Emits one canonical document with gradients before clips in defs, then root surface/grid/body; no alternate document form | `none`; framing/order and external scaling behavior | Canonical SVG document serialization | A3-M8; root paint and lifecycle state are separate; A3-E-SVG |
| `geometry.collision.annotation-weight` | A3-M10 `ANNOTATION_COLLISION_WEIGHT: f64 = 100.0`, assigned to generic text and bracket obstacles | A3-C-GEOMETRY | Text/brackets contribute 100 times their overlap area; ordinary primitives use independently declared weight `1.0` | `collision` | Fixed annotation-obstacle priority | A3-M10; declaration and all uses admitted; A3-E-GEOMETRY |
| `geometry.obstacle.point-envelope` | A3-M10 `GeometryBox::from_points`: infinity sentinels, extrema over points, symmetric caller padding, empty-to-`None`, and independent `weight: 1.0` | A3-C-GEOMETRY | Produces one axis-aligned envelope for nonempty input; no finite check here because draw methods validate first | `geometry`, collision | Conservative point-set obstacle envelope | A3-M10; independent weight source admitted; A3-E-GEOMETRY |
| `geometry.obstacle.clip-intersection` | A3-M10 `GeometryBox::clipped` plus `record` use of only `self.clips.last()` | A3-C-GEOMETRY | No active clip preserves the box; otherwise intersects inclusively with only the innermost clip and drops disjoint boxes | `geometry`, collision | Collision-obstacle clipping | A3-M10; current innermost-only behavior explicit; A3-E-GEOMETRY |
| `geometry.obstacle.overlap-score` | A3-M10 `overlap_area`: margin-expanded overlap width/height clamped at zero, then area multiplied by `self.weight` | A3-C-GEOMETRY | Returns finite/nonfinite arithmetic as produced; expands the compared obstacle by margin and applies only the receiver weight | `collision` | Weighted collision score | A3-M10; complete formula admitted; A3-E-GEOMETRY |
| `geometry.obstacle-recording-order` | A3-M10 `GeometryBackend.bounds`, immutable `bounds()`, and successful `bounds.push` in `record` | A3-C-GEOMETRY | Surviving obstacles append in draw order and remain exposed as one immutable slice; no deduplication or union | `geometry`, collision | Ordered collision-obstacle accumulation | A3-M10; clip calculation split to its own record; A3-E-GEOMETRY |
| `geometry.backend-lifecycle` | A3-M11/A3-M12 derived empty `Default`; `begin` and `end` both unconditionally return `Ok(())` | A3-C-GEOMETRY | Production decoration planning constructs fresh default backends, draws directly, and never calls `begin` or `end`, so both methods are dormant; if called, begin ignores dimensions/does not clear bounds or clips and end accepts active clips | `none`; dormant lifecycle currently has no production effect | Concrete Geometry backend lifecycle | A3-M11/A3-M12; dormant concrete lifecycle and direct-draw production path admitted; A3-E-GEOMETRY |
| `geometry.clip-stack-lifecycle` | A3-M11 `begin_clip` validates then pushes; `end_clip` pops or returns an unbalanced-scope error | A3-C-GEOMETRY | Nested clips are stored, but obstacle clipping consumes only the last; unmatched close fails | `geometry`, collision | Geometry clip-scope state | A3-M11; abstract clip protocol/validation prior-owned; A3-E-GEOMETRY |
| `geometry.scatter-radius-bounds` | A3-M11 `draw_scatter`: one square `from_points` obstacle per point padded only by `style.radius` | A3-C-GEOMETRY | Records even when fill opacity is zero, radius is zero, and no positive-width outline paints; uses the same square for Circle and TriangleUp, overestimates triangle fill horizontally, and omits every outline and acute triangle miter extent | `geometry`, collision | Point collision extent | A3-M11; paint, shape, outline, and miter discrepancies explicit; A3-E-GEOMETRY |
| `geometry.line-segment-bounds` | A3-M11 `draw_line`: independent `<2` no-op, one obstacle per `windows(2)` segment, and half-width padding | A3-C-GEOMETRY | Current line paint has no alpha and is opaque when width is positive, but every nondegenerate segment records even at zero stroke width; visible dashed lines conservatively occupy each full segment envelope rather than dash fragments, and the half-width envelope covers the SVG round caps/joins | `geometry`, collision | Polyline collision extent | A3-M11; concrete arity, zero-width, dash, and bounds behavior admitted without re-owning T2a2 validation; A3-E-GEOMETRY |
| `geometry.polygon-bounds` | A3-M11 `draw_polygon`: independent `<3` no-op, one aggregate point envelope, and half-stroke padding | A3-C-GEOMETRY | Records even when fill opacity is zero and the otherwise opaque stroke has zero width; half-width padding models neither the implicit miter join nor acute-corner miter extension and does not calculate fill-only versus stroked shape-specific corner extent | `geometry`, collision | Polygon collision extent | A3-M11; paint and miter discrepancies explicit and independent from the equal line branch; A3-E-GEOMETRY |
| `geometry.gradient-bounds` | A3-M11 `draw_linear_gradient`: exact `PanelBox` coordinates and independent `weight: 1.0`; vector/stops only validated | A3-C-GEOMETRY | Current RGB-only stops are always opaque, so every validated positive gradient rectangle is painted and recorded; vector and stop geometry do not alter its obstacle | `geometry`, collision | Gradient rectangle collision extent | A3-M11; current opaque paint behavior and independent weight source admitted; A3-E-GEOMETRY |
| `geometry.text-bounds` | A3-M12 `draw_text`: `estimated_text_width`, height `font_size * 1.2`, anchor offsets, exact `rotation == 0.0`, and absolute sine/cosine rotated projection centered on x/y | A3-C-GEOMETRY | Current text paint is opaque, but empty/whitespace-only or zero-size text can still record a zero- or positive-height box with no painted glyph; unrotated bounds honor anchor, rotated bounds ignore Start/End, and face/weight are ignored | `measure`, `geometry`, collision | Estimated generic text collision bounds | A3-M12; paint-area and measurement-to-draw discrepancies explicit; A3-E-GEOMETRY |
| `geometry.rounded-rectangle-bounds` | A3-M12 `draw_rounded_rect`: half-stroke padding around the axis-aligned rectangle and independent `weight: 1.0` | A3-C-GEOMETRY | Always records even for zero width/height or when fill opacity is zero and the otherwise opaque stroke has zero width; otherwise radius does not reduce the conservative rectangle | `geometry`, collision | Rounded-container collision extent | A3-M12; paint-area, zero-extent, and independent weight behavior admitted; A3-E-GEOMETRY |
| `geometry.significance-bracket-path-bounds` | A3-M12 path-derived `draw_bracket` terms: orientation/anchor span, `tick_height` extents, and fused box recording; stroke-width overhang is omitted | A3-C-GEOMETRY | Current bracket paint is opaque, but path and label always become one weighted box even when path width is zero; path terms neither filter the unpainted zero-width path nor include positive stroke extent | `geometry`, collision | Significance-path collision geometry | A3-M12; expression-context and paint-area split from label terms; A3-E-GEOMETRY |
| `geometry.significance-bracket-label-bounds` | A3-M12 label-derived terms: estimated label width, midpoint, font size/half-height, label gap, and overhang min/max in both orientations | A3-C-GEOMETRY | Current label paint is opaque, but an empty/whitespace-only label can enlarge the fused box despite having no painted glyph; face/weight are ignored | `measure`, `geometry`, collision | Significance-label collision bounds | A3-M12; expression-context, paint-area, and typography split from path terms; A3-E-GEOMETRY |
| `scene.canonical-immutable-storage` | A3-M14/A3-M16 `RenderedScene { width_css_px, height_css_px, svg: Box<[u8]> }`; `new` boxes supplied canonical bytes without validation | A3-C-SCENE | One owned immutable byte sequence and dimensions are reused by terminal calls; constructor accepts zero/excessive/mismatched values | `none`; stores already-rendered layout | Canonical immutable rendered scene | A3-M14/A3-M16; Debug excluded; A3-E-OUTPUT |
| `scene.preserve-svg-routing` | A3-M16 `svg_bytes` borrows `&self.svg` exactly | A3-C-SCENE | No parse, copy, validation, font lookup, or fallback | `none` | Exact preserve-text SVG scene reuse | A3-M16; separate route/phase from outlined SVG and PNG; A3-E-OUTPUT |
| `scene.outlined-svg-routing` | A3-M16 `svg_with_text_as_paths` forwards exact bytes and stored CSS dimensions to `convert_svg_text_to_paths` | A3-C-SCENE, A3-C-OUTLINE | No conversion-input limit check occurs here; converter owns parse/dimension/residual-text failures | `geometry`; text becomes glyph paths after layout | Canonical scene to outlined-SVG routing | A3-M16; conversion implementation counterpart `backend.svg.text-outline-conversion`; A3-E-OUTPUT |
| `scene.png-routing` | A3-M16 `png_bytes` first calls `validate_conversion_input("PNG")`, then forwards exact bytes/dimensions/options to rasterization | A3-C-SCENE, A3-C-PNG | PNG alone receives source scene limits here; raster sizing/encoding failures propagate with no alternate output | `none` for routing; target sizing is T2a4 | Canonical scene to PNG routing | A3-M16; sizing/options are carried and deferred; A3-E-OUTPUT |
| `scene.source-dimensions-positive-limit` | A3-M16 PNG source branch rejects `width_css_px == 0 \|\| height_css_px == 0` | A3-C-PNG | Applied only through current PNG route; outlined/preserve SVG bypass it | `none`; PNG conversion gate | Positive PNG source-scene dimensions | A3-M16; independent from computed PNG target finite/positive branch; A3-E-OUTPUT |
| `scene.export-dimension-limit` | A3-M14/A3-M16/A3-M17 `MAX_EXPORT_DIMENSION: u32 = 32_768`; PNG source `u32` and computed PNG target `f64` per-axis branches | A3-C-PNG | Rejects either axis above 32,768 only for PNG source conversion input and computed PNG target; exact equality passes; preserve/outlined SVG bypass it | `none`; PNG resource gate | PNG per-axis conversion resource limit | A3-M14/A3-M16/A3-M17; one declaration and one PNG semantic cap; A3-E-OUTPUT |
| `scene.source-pixel-work-limit` | A3-M14/A3-M16 `MAX_EXPORT_PIXELS: u64 = 100_000_000`; checked PNG source area uses exact `u64` multiplication | A3-C-PNG | Rejects PNG source CSS pixel work above 100,000,000; exact equality passes; preserve/outlined SVG bypass it | `none`; PNG resource gate | PNG source-scene conversion-work limit | A3-M14/A3-M16; independent integer declaration; A3-E-OUTPUT |
| `scene.target-pixel-work-limit` | A3-M14/A3-M17 independent `MAX_EXPORT_PIXELS_F64: f64 = 100_000_000.0`; computed target branch uses `width * height` | A3-C-PNG | Rejects finite computed PNG target work above 100,000,000; exact equality passes; source integer limit is separate | `none`; resource gate | Computed PNG target-work limit | A3-M14/A3-M17; equal magnitude does not merge independent declaration; A3-E-OUTPUT |
| `scene.conversion-byte-limit` | A3-M14/A3-M16 `MAX_CONVERSION_SCENE_BYTES: usize = 64 * 1024 * 1024`; PNG branch compares canonical SVG byte length | A3-C-PNG | Rejects PNG parser input above 64 MiB; exact equality passes; preserve/outlined SVG bypass it | `none`; PNG parser resource gate | PNG canonical-SVG parser-input limit | A3-M14/A3-M16; distinct PNG byte resource; A3-E-OUTPUT |
| `scene.png-target-positive-finite-limit` | A3-M17 `validate_png_target_dimensions` requires both `f64` values finite and `> 0.0` | A3-C-PNG | Rejects NaN, infinity, zero, or negative computed dimensions before allocation; no clamp/substitution | `none`; conversion gate | Computed PNG dimension representability | A3-M17; sizing arithmetic deferred but limit source retained here; A3-E-OUTPUT |

### T2a3 Target State

| Property | Target classification and owner | Target resolution | Target validation | Public surface | Relationships |
| --- | --- | --- | --- | --- | --- |
| `backend.svg.concrete-canvas-lifecycle` | Renderer invariant; private concrete SVG serializer state machine | Final private SVG draw boundary | Begin clears/invalidates every prior output before validating or opening the new session; one successful begin, balanced scopes, and one successful finish are required, and extraction can refer only to bytes from that just-finished session | Private-only | Concrete implementation of `backend.protocol.begin-end-lifecycle`; T2a4 orchestration is a consumer, not a duplicate source |
| `backend.svg.xml-escaping` | Renderer invariant; private XML-safe SVG text/attribute serialization | Final private SVG draw boundary | Every admitted string is escaped exactly once for its XML context; no raw fallback | Private-only | Text counterpart `backend.svg.text-element-serialization`; bracket counterpart `backend.svg.significance-bracket-label-serialization`; layer counterpart `backend.svg.layer-buffering-and-grid-order` |
| `backend.svg.coordinate-conversion-and-precision` | Renderer invariant; private canonical post-conversion geometry and SVG coordinate spelling | Before geometry measurement/recording and final SVG drawing | Measurement and drawing consume one canonical post-conversion geometry; signed off-canvas positions remain signed, and any precision/representation step that collapses distinct anchors/endpoints/stops or positive extents rejects without clamp, saturation, fallback, or alternate rendering | Private-only | Concrete counterpart `backend.protocol.coordinate-convention`; the same post-conversion values feed all T2a3 Geometry bounds and SVG serializers |
| `backend.svg.text-outline-conversion` | Renderer invariant; private generation-host outlined-SVG conversion | After resolved font preflight and canonical scene creation | Parsed dimensions must match; every used glyph/face is resolved before measurement; conversion must leave no text elements | Private-only | Route `scene.outlined-svg-routing`; font counterparts `font-resolution.load-system-fonts` and `backend.text.typography-carrier` |
| `backend.png.rgba8-paint-serialization` | Renderer invariant; private PNG RGBA8 compositing and alpha serialization | Final private PNG conversion boundary from one canonical scene | Target allocation validated by T2a4; final pixels retain one straight RGBA alpha with no multiplication | Private-only | Surface counterpart `surface.canvas.backend-white-root`; SVG alpha counterpart `backend.svg.incomplete-per-paint-alpha-serialization`; sizing/publication deferred to T2a4 |
| `backend.svg.layer-buffering-and-grid-order` | Renderer invariant; private balanced SVG grouping and grid-before-marks order | Final private SVG draw boundary | Closed layer roles, top-level Grid, LIFO closure, and no unclosed scopes at finish | Private-only | Abstract counterparts `backend.layer.semantic-role`, `backend.protocol.ordinary-layer-fallbacks`, and `backend.protocol.grid-role-delegation` |
| `backend.svg.clip-scope-serialization` | Renderer invariant; private deterministic SVG clipping | Final private SVG draw boundary | Valid positive clip geometry, deterministic IDs, nested LIFO closure, and no dangling reference/scope | Private-only | Abstract counterparts `backend.protocol.clip-scope-contract` and `backend.validation.clip-region`; geometry counterpart `geometry.clip-stack-lifecycle` |
| `backend.svg.scatter-glyph-serialization` | Renderer invariant; private SVG point-glyph geometry plus fixed join serialization | Final private draw boundary after mark/exact-style resolution | Closed supported shape, explicit fixed `miter` join and miter limit `4.0` for outlined triangle corners, finite representable extent, and identical canonical geometry in SVG and collision recording | Private-only | Carrier `backend.point.geometry-carrier`; bounds counterpart `geometry.scatter-radius-bounds`; join sibling `backend.svg.polygon-structure-serialization`; alpha counterpart `backend.svg.incomplete-per-paint-alpha-serialization` |
| `backend.svg.line-degenerate-noop` | Renderer invariant; concrete line arity behavior | Final private draw boundary | Fewer than two points is an explicit no-op consuming neither coordinates nor style | Private-only | Abstract policy `backend.validation.primitive-points`; independent sibling `backend.svg.polygon-degenerate-noop`; geometry counterpart `geometry.line-segment-bounds` |
| `backend.svg.line-structure-and-pattern-serialization` | Renderer invariant; private SVG line/pattern serialization | Final private draw boundary after exact line style resolution | Complete paint, finite nonnegative width, and valid closed pattern; no arbitrary backend string | Private-only | Carriers `backend.line.stroke-width-carrier` and `backend.line.dash-string-carrier`; alpha counterpart `backend.svg.incomplete-per-paint-alpha-serialization` |
| `backend.svg.polygon-degenerate-noop` | Renderer invariant; concrete polygon arity behavior | Final private draw boundary | Fewer than three points is an explicit no-op consuming neither coordinates nor style | Private-only | Abstract policy `backend.validation.primitive-points`; independent sibling `backend.svg.line-degenerate-noop`; geometry counterpart `geometry.polygon-bounds` |
| `backend.svg.polygon-structure-serialization` | Renderer invariant; private SVG polygon geometry plus fixed join serialization | Final private draw boundary after fill/exact-style resolution | Finite nondegenerate points, complete independent fill/stroke styles, and explicit fixed `miter` join with miter limit `4.0`, shared with painted-bounds calculation | Private-only | Carriers `backend.fill.paint-carrier` and `backend.fill.stroke-carrier`; bounds `geometry.polygon-bounds`; join sibling `backend.svg.scatter-glyph-serialization`; alpha `backend.svg.incomplete-per-paint-alpha-serialization` |
| `backend.svg.linear-gradient-serialization` | Renderer invariant; private native deterministic SVG gradient serialization | Final private draw boundary after resolved scale/style | Complete validated vector/bounds/stops, deterministic IDs, exact stop order, and complete per-stop paint | Private-only | Protocol `backend.protocol.gradient-contract`; carrier `backend.gradient.stop-carrier`; bounds `geometry.gradient-bounds`; alpha `backend.svg.incomplete-per-paint-alpha-serialization` |
| `backend.svg.rounded-rectangle-serialization` | Renderer invariant; private native SVG rounded-rectangle serialization | Final private draw boundary after resolved container style | Finite geometry, admitted radius, and complete independent fill/stroke styles | Private-only | Protocol `backend.protocol.rounded-rectangle-fallback`; validation `backend.validation.rounded-rectangle`; bounds `geometry.rounded-rectangle-bounds` |
| `backend.svg.text-element-serialization` | Renderer invariant; private complete resolved preserve-text serialization | Final private draw boundary after font/glyph preflight | Complete resolved text style, finite placement, validated stack/metrics, and exact measured/drawn preservation of every admitted leading, trailing, repeated, tab, and newline whitespace sequence | Private-only | Carriers `backend.text.typography-carrier` and `backend.text.placement-carrier`; bounds `geometry.text-bounds`; alpha `backend.svg.incomplete-per-paint-alpha-serialization` |
| `backend.svg.significance-bracket-path-serialization` | Renderer invariant; private significance-path geometry serialization | Final private draw boundary after annotation planning | Finite distinct anchors, representable tick/path geometry, complete stroke, and path bounds including stroke extent | Private-only | Carrier `backend.bracket.geometry-carrier`; bounds `geometry.significance-bracket-path-bounds`; label sibling `backend.svg.significance-bracket-label-serialization` |
| `backend.svg.significance-bracket-label-serialization` | Renderer invariant; private complete resolved significance-label serialization | Final private draw boundary after typography preflight | Resolved stack/glyphs, positive metrics, finite gap, identical measured/drawn placement, and exact preservation of every admitted leading, trailing, repeated, tab, and newline whitespace sequence | Private-only | Carrier `backend.bracket.typography-carrier`; bounds `geometry.significance-bracket-label-bounds`; path sibling `backend.svg.significance-bracket-path-serialization` |
| `backend.svg.incomplete-per-paint-alpha-serialization` | Renderer invariant; private final RGBA-to-SVG paint-property and alpha serialization responsibility | Final private SVG draw boundary after all paint precedence | Every fill, stroke, text paint, bracket paint, and gradient stop supplies one complete RGBA value; place each paint on its exact SVG property, emit alpha once on the corresponding paint-specific property, and never multiply | Private-only | Sole paint-property/alpha owner; `backend.svg.rgb-hex-serialization` supplies deterministic token spelling only, while T2a2 paint carriers connect in T5c/T5d/T6 without duplicate source rows |
| `surface.canvas.backend-white-root` | Theme component; `SurfaceTheme` Canvas/root surface | Canvas root or standalone Publication root | Complete resolved `Color`; SVG/PNG preserve RGBA and no backend fallback root is inserted | T6 `SurfaceTheme` | Current theme counterpart `surface.canvas.current-background`; PNG alpha `backend.png.rgba8-paint-serialization`; panel surface remains `surface.panel.current-background` |
| `backend.svg.document-framing` | Renderer invariant; private canonical SVG document serialization | Final private SVG draw boundary | Exact root dimensions/viewBox, deterministic defs/content order, and fixed non-scaling-stroke behavior | Private-only | Lifecycle `backend.svg.concrete-canvas-lifecycle`; ordering `backend.svg.layer-buffering-and-grid-order`; surface `surface.canvas.backend-white-root` |
| `geometry.collision.annotation-weight` | Renderer invariant; private annotation-obstacle collision priority | Geometry-discovery/collision planning | Finite positive fixed weight and deterministic weighted overlap | Private-only | Applied by `geometry.text-bounds` and both significance-bound records; collision rule is not themeable |
| `geometry.obstacle.point-envelope` | Renderer invariant; private conservative point-set envelope construction | Geometry-discovery boundary after caller paint filtering | Nonempty finite points and finite nonnegative padding produce finite bounds; empty input produces no obstacle; callers, not this generic helper, own paint visibility, joins, and shape-specific extent | Private-only | Geometric helper used by `geometry.scatter-radius-bounds`, `geometry.line-segment-bounds`, and `geometry.polygon-bounds`; no paint-semantic ownership |
| `geometry.obstacle.clip-intersection` | Renderer invariant; private collision-obstacle clipping | Geometry-discovery boundary | Every active validated clip participates in the effective intersection; disjoint obstacles are discarded | Private-only | Scope `geometry.clip-stack-lifecycle`; abstract protocol `backend.protocol.clip-scope-contract`; SVG counterpart `backend.svg.clip-scope-serialization` |
| `geometry.obstacle.overlap-score` | Renderer invariant; private weighted collision scoring | Collision planning | Finite nonnegative margin and finite bounds produce deterministic nonnegative score | Private-only | Weight source `geometry.collision.annotation-weight`; ordered inputs `geometry.obstacle-recording-order` |
| `geometry.obstacle-recording-order` | Renderer invariant; private ordered collision-obstacle accumulation | Geometry-discovery boundary | Fresh lifecycle state, deterministic append order, and immutable read access after discovery | Private-only | Lifecycle `geometry.backend-lifecycle`; clipping `geometry.obstacle.clip-intersection` |
| `geometry.backend-lifecycle` | Renderer invariant; balanced Geometry backend lifecycle | Decoration geometry-discovery boundary | T3 decoration planning must call begin before direct drawing and end afterward; begin starts fresh state with valid dimensions, end rejects unclosed clips, and no stale bounds cross sessions | Private-only | Connects actual `decorations/plan.rs` discovery passes to abstract lifecycle `backend.protocol.begin-end-lifecycle` and scope `geometry.clip-stack-lifecycle` |
| `geometry.clip-stack-lifecycle` | Renderer invariant; nested Geometry clip-scope state | Geometry-discovery boundary | Valid regions, LIFO push/pop, all active clips applied, and imbalance rejected at close/end | Private-only | Abstract protocol `backend.protocol.clip-scope-contract`; intersection `geometry.obstacle.clip-intersection` |
| `geometry.scatter-radius-bounds` | Renderer invariant; private complete painted point collision extent | Geometry-discovery boundary after point-style resolution | Record only the union of nonzero-alpha, positive-extent fill and outline parts; calculate Circle and TriangleUp bounds from their actual shape, outline width, and the fixed miter join/limit, omitting a point with no painted area | Private-only | Fill carrier `backend.point.fill-paint-carrier`; outline carrier `backend.point.outline-carrier`; SVG `backend.svg.scatter-glyph-serialization`; T5c can resolve current fill opacity/geometry, while T6 completes fill/outline RGBA inputs |
| `geometry.line-segment-bounds` | Renderer invariant; private painted polyline collision extent | Geometry-discovery boundary | Explicit degenerate no-op; omit zero-alpha or zero-width strokes, otherwise record the union of positive-extent painted segments/caps/joins and conservatively retain full-segment envelopes for visible dashed lines | Private-only | SVG arity `backend.svg.line-degenerate-noop`; structure `backend.svg.line-structure-and-pattern-serialization`; validation `backend.validation.line-style`; T3 owns geometry/zero width and `backend.line.stroke-paint-carrier` connects zero alpha in T6 |
| `geometry.polygon-bounds` | Renderer invariant; private painted polygon collision extent | Geometry-discovery boundary | Explicit degenerate no-op; record only the union of nonzero-alpha positive-area fill and nonzero-alpha positive-width stroke, including shape-specific acute-corner extent under the fixed miter join/limit, and omit a polygon with no painted area | Private-only | SVG arity `backend.svg.polygon-degenerate-noop`; structure/join `backend.svg.polygon-structure-serialization`; fill/stroke carriers `backend.fill.paint-carrier` and `backend.fill.stroke-carrier`; T3 owns current opacity/geometry and T6 completes independent RGBA inputs |
| `geometry.gradient-bounds` | Renderer invariant; private painted gradient rectangle collision extent | Geometry-discovery boundary | Record the positive-area rectangle only when resolved gradient paint has nonzero alpha somewhere; omit an all-transparent gradient, while vector/stops do not otherwise alter occupied bounds | Private-only | SVG `backend.svg.linear-gradient-serialization`; validation `backend.validation.linear-gradient`; T3 retains current opaque bounds and `backend.gradient.stop-carrier` connects complete stop alpha in T6 |
| `geometry.text-bounds` | Renderer invariant; private resolved painted glyph collision bounds | After font resolution and before collision planning | Record only the union of nonzero-alpha, positive-area painted glyphs using the same faces, glyphs, size, weight, anchor, transform, and whitespace interpretation as drawing; omit text with no painted glyph area | Private-only | SVG `backend.svg.text-element-serialization`; typography carrier `backend.text.typography-carrier`; T5a owns glyph/metric and empty-text parity, while `backend.text.paint-carrier` connects transparent paint in T6; T2c measurement sources remain separate |
| `geometry.rounded-rectangle-bounds` | Renderer invariant; private painted rounded-container collision extent | Geometry-discovery boundary | Record only the union of nonzero-alpha positive-area fill and nonzero-alpha positive-width stroke for positive representable extents; omit a rectangle with no painted area, while radius does not reduce conservative outer bounds | Private-only | SVG `backend.svg.rounded-rectangle-serialization`; validation `backend.validation.rounded-rectangle`; T3 owns current opacity/geometry and T6 fill/stroke carriers complete independent RGBA inputs |
| `geometry.significance-bracket-path-bounds` | Renderer invariant; private painted significance-path collision geometry | After annotation planning and before collision selection | Record only nonzero-alpha, positive-width path/tick parts and include their full stroke extent; omit an unpainted path independently of any visible label | Private-only | SVG `backend.svg.significance-bracket-path-serialization`; carrier `backend.bracket.stroke-width-carrier`; label sibling `geometry.significance-bracket-label-bounds` |
| `geometry.significance-bracket-label-bounds` | Renderer invariant; private resolved painted significance-label collision bounds | After typography preflight and before collision selection | Record only the union of nonzero-alpha, positive-area painted glyphs using the same resolved face/glyph/size/weight/gap/anchor and whitespace interpretation as final label serialization; omit a label with no painted glyph area | Private-only | SVG `backend.svg.significance-bracket-label-serialization`; carrier `backend.bracket.typography-carrier`; path sibling `geometry.significance-bracket-path-bounds` |
| `scene.canonical-immutable-storage` | Renderer invariant; private immutable canonical-scene ownership | After one complete render and before terminal output selection | Construction validates positive canonical dimensions and valid completed SVG bytes exactly once before any terminal route | Private-only | Owns deduplicated positivity from `scene.source-dimensions-positive-limit`; routes `scene.preserve-svg-routing`, `scene.outlined-svg-routing`, and `scene.png-routing`; T2a4 artifact owners consume but do not rebuild it |
| `scene.preserve-svg-routing` | Renderer invariant; exact canonical preserve-text SVG reuse | Terminal exact-SVG request | Return the immutable canonical bytes without parsing, font lookup, mutation, or fallback | Private-only | Storage `scene.canonical-immutable-storage`; distinct outlined route `scene.outlined-svg-routing` |
| `scene.outlined-svg-routing` | Renderer invariant; canonical-scene-to-outlined-SVG routing | Terminal outlined-SVG request after font preflight | Route exact bytes/dimensions through the one validated converter; PNG maximum-dimension, pixel-work, and byte limits do not apply unless a later contract changes their scope | Private-only | Converter `backend.svg.text-outline-conversion`; storage `scene.canonical-immutable-storage` |
| `scene.png-routing` | Renderer invariant; canonical-scene-to-PNG routing | Terminal PNG request | Route exact scene and validated options once; no rerender or alternate rasterizer | Private-only | Storage `scene.canonical-immutable-storage`; paint `backend.png.rgba8-paint-serialization`; sizing/options/publication deferred to T2a4 |
| `scene.source-dimensions-positive-limit` | Renderer invariant; canonical-scene construction positivity | At `scene.canonical-immutable-storage` construction before any terminal route | Both canonical CSS dimensions are strictly positive once; reject without substitution, then remove the PNG-local duplicate check | Private-only | Deduplicated into `scene.canonical-immutable-storage`; computed PNG target counterpart `scene.png-target-positive-finite-limit` remains separate |
| `scene.export-dimension-limit` | Renderer invariant; fixed PNG per-axis source/target resource limit | Before PNG source conversion and computed PNG target allocation | Each PNG source/computed target axis is at most 32,768 and otherwise rejected; preserve/outlined SVG are outside this limit | Private-only | PNG pixel-work siblings `scene.source-pixel-work-limit` and `scene.target-pixel-work-limit`; T2a4 PNG sizing consumes this limit |
| `scene.source-pixel-work-limit` | Renderer invariant; fixed PNG source-scene conversion-work limit | Before PNG parser/raster conversion | Exact integer PNG source work is at most 100,000,000 CSS pixels; preserve/outlined SVG are outside this limit | Private-only | Independent PNG target sibling `scene.target-pixel-work-limit`; dimensions `scene.export-dimension-limit` |
| `scene.target-pixel-work-limit` | Renderer invariant; fixed computed PNG target-work limit | Before computed PNG target allocation | Finite computed PNG target work is at most 100,000,000 pixels; reject before allocation; no preserve/outlined SVG scope | Private-only | Independent PNG source sibling `scene.source-pixel-work-limit`; sizing arithmetic deferred to T2a4 |
| `scene.conversion-byte-limit` | Renderer invariant; fixed PNG canonical-SVG parser-input limit | Before PNG rasterization parses the canonical SVG | PNG parser input is at most 64 MiB; preserve/outlined SVG are outside this limit unless a later contract changes scope | Private-only | Storage `scene.canonical-immutable-storage`; independent from PNG dimension/pixel limits |
| `scene.png-target-positive-finite-limit` | Renderer invariant; computed PNG dimension representability | Before target size rounding/allocation | Both dimensions finite and strictly positive; reject without clamp/substitution | Private-only | Source counterpart `scene.source-dimensions-positive-limit`; maximums `scene.export-dimension-limit` and `scene.target-pixel-work-limit` |

### T2a3 Migration And Evidence

| Property | Disposition and owning phase | Visual evidence |
| --- | --- | --- |
| `backend.svg.concrete-canvas-lifecycle` | `replace`, T3 | No successful first-session output change; reuse, extraction-during-second-session, failed-second-begin/render, stale-byte invalidation, balanced-scope, and exactly-one finish assertions |
| `backend.svg.xml-escaping` | `retain`, T3 | No output change; exact special-character text, family, and layer-ID SVG assertions |
| `backend.svg.coordinate-conversion-and-precision` | `replace`, T3 | Ordinary built-ins remain stable; subprecision distinct-point/bracket-anchor/gradient-endpoint/stop, positive-subprecision extent, signed off-canvas text, and no-saturation/no-collapse rejection evidence |
| `backend.svg.text-outline-conversion` | `replace`, T5a | Intentional font-metric timing correction; outlined glyph, non-Latin, dimension-parity, and no-residual-text evidence |
| `backend.png.rgba8-paint-serialization` | `connect`, T5a | Transparent surface/paint alpha becomes observable after root-surface correction; exact RGBA8 PNG and compositing assertions, with sizing/publication evidence deferred to T2a4 |
| `backend.svg.layer-buffering-and-grid-order` | `retain`, T3 | No output change; top-level Grid, nested group balance, and grid-before-marks snapshots |
| `backend.svg.clip-scope-serialization` | `retain`, T3 | No output change; deterministic IDs, nested LIFO, invalid close, and panel clipping evidence |
| `backend.svg.scatter-glyph-serialization` | `connect`, T3 | Decoded built-ins remain stable; exact Circle/TriangleUp geometry plus explicit `miter`/`4.0` outlined-triangle join parity and acute-corner assertions |
| `backend.svg.line-degenerate-noop` | `retain`, T3 | No successful-output change; explicit short-line no-op and invalid-carrier bypass assertions |
| `backend.svg.line-structure-and-pattern-serialization` | `replace`, T3 | No intended visual change; closed solid/dashed serialization and round cap/join assertions |
| `backend.svg.polygon-degenerate-noop` | `retain`, T3 | No successful-output change; explicit short-polygon no-op and invalid-carrier bypass assertions |
| `backend.svg.polygon-structure-serialization` | `connect`, T3 | Decoded built-ins remain stable; exact polygon points/stroke width plus explicit `miter`/`4.0` join parity and acute-corner assertions |
| `backend.svg.linear-gradient-serialization` | `retain`, T3 | No output change; deterministic defs/IDs, vector/bounds, stop-order, and heatmap/beeswarm snapshots |
| `backend.svg.rounded-rectangle-serialization` | `retain`, T3 | No output change; exact native rounded-container geometry and no-fallback evidence |
| `backend.svg.text-element-serialization` | `replace`, T5a | Intentional measurement/draw synchronization; anchor, rotation, weight, family, glyph, and exact leading/trailing/repeated/tab/newline whitespace snapshots |
| `backend.svg.significance-bracket-path-serialization` | `connect`, T5c | No intended path change plus possible stroke-bounds correction; vertical/horizontal exact path and clipping evidence |
| `backend.svg.significance-bracket-label-serialization` | `replace`, T5a | Intentional resolved-font/whitespace correction; label anchor, weight, gap, orientation, glyph, and exact leading/trailing/repeated/tab/newline whitespace snapshots |
| `backend.svg.incomplete-per-paint-alpha-serialization` | `replace`, T5a | Intentional complete-alpha support; install the one RGBA-to-SVG serializer with exact per-paint alpha/no-multiplication assertions, then connect mark/scale/public carriers in T5c/T5d/T6 |
| `surface.canvas.backend-white-root` | `replace`, T5a | Built-in white remains visually stable; transparent SVG/PNG and Canvas/panel compositing snapshots prove removal of backend fallback |
| `backend.svg.document-framing` | `retain`, T3 | No output change; exact root/viewBox/style/defs/content ordering snapshots |
| `geometry.collision.annotation-weight` | `retain`, T3 | No output change; deterministic weighted legend-collision selection tests |
| `geometry.obstacle.point-envelope` | `retain`, T3 | No output change; empty and padded extrema unit evidence plus local-obstacle tests |
| `geometry.obstacle.clip-intersection` | `replace`, T3 | Intentional nested-clip correction if noncontained clips occur; all-active-clip and disjoint-obstacle assertions |
| `geometry.obstacle.overlap-score` | `retain`, T3 | No output change; exact margin, zero-overlap, weight, and legend-choice tests |
| `geometry.obstacle-recording-order` | `retain`, T3 | No output change; ordered obstacle and immutable-bounds assertions |
| `geometry.backend-lifecycle` | `replace`, T3 | No intended output change; production decoration discovery must exercise begin/end, fresh-state reuse, dimensions, direct-draw prohibition, and unclosed-clip end failure tests |
| `geometry.clip-stack-lifecycle` | `connect`, T3 | No ordinary output change; nested push/pop, unmatched close, end balance, and all-active-clip evidence |
| `geometry.scatter-radius-bounds` | `connect`, T5c; complete fill/outline RGBA connects in T6 | Intentional shape/current-opacity/outline-geometry correction in T5c; transparent fill/outline combinations, Circle versus TriangleUp extent, acute-triangle miter, radius/outline collision, and inside-legend evidence complete with T6 carriers |
| `geometry.line-segment-bounds` | `replace`, T3; zero-alpha paint connects in T6 | T3 corrects zero-width geometry while preserving per-segment locality, visible dashed full-segment conservatism, cap/join extent, and degenerate no-op behavior; T6 adds zero-alpha omission evidence |
| `geometry.polygon-bounds` | `replace`, T3; independent fill/stroke RGBA connects in T6 | T3 corrects current fill-opacity, zero-width, no-painted-area, and acute-miter geometry; T6 adds independent transparent fill/stroke combination evidence |
| `geometry.gradient-bounds` | `retain`, T3; complete stop alpha connects in T6 | Current opaque gradients retain exact rectangular obstacles and guide/legend behavior; T6 adds all-transparent omission evidence |
| `geometry.text-bounds` | `replace`, T5a; complete text paint connects in T6 | T5a corrects empty/whitespace-only and resolved face/weight/rotation-anchor glyph bounds; T6 adds transparent-paint omission evidence |
| `geometry.rounded-rectangle-bounds` | `replace`, T3; independent fill/stroke RGBA connects in T6 | T3 corrects zero extent, current transparent fill, zero-width stroke, no-painted-area, stroke union, and radius-independent outer bounds; T6 adds independent transparent fill/stroke evidence |
| `geometry.significance-bracket-path-bounds` | `connect`, T5c | Intentional paint/stroke correction; zero-alpha/zero-width path omission, exact positive stroke extent, and inside-legend collision assertions |
| `geometry.significance-bracket-label-bounds` | `replace`, T5a | Intentional paint/font/whitespace correction; transparent or no-painted-glyph omission, both orientations, exact whitespace, weight, long label, and legend-collision evidence |
| `scene.canonical-immutable-storage` | `connect`, T3 | No ordinary output change; one-render/multi-terminal byte reuse plus canonical-construction positive-dimension and invalid-scene rejection assertions |
| `scene.preserve-svg-routing` | `retain`, T3 | No output change; exact canonical-byte equality and no-font-lookup evidence |
| `scene.outlined-svg-routing` | `connect`, T5a | Intentional font-preflight timing correction; one-scene outlined output, glyph, and error-path assertions |
| `scene.png-routing` | `retain`, T3 | No output change; one-scene PNG routing and no-rerender evidence, with options/sizing/publication completed by T2a4 |
| `scene.source-dimensions-positive-limit` | `deduplicate`, T3 | No successful-output change; move zero-axis rejection to canonical scene construction, remove the PNG-local duplicate, and prove one exact positive-boundary check serves all routes |
| `scene.export-dimension-limit` | `retain`, T3 | No successful-output change; PNG source/target per-axis boundary and over-limit tests plus preserve/outlined SVG nonapplication evidence |
| `scene.source-pixel-work-limit` | `retain`, T3 | No successful-output change; exact integer PNG source-work boundary/rejection plus preserve/outlined SVG nonapplication evidence |
| `scene.target-pixel-work-limit` | `retain`, T3 | No successful-output change; exact floating PNG target-work boundary and preallocation rejection tests |
| `scene.conversion-byte-limit` | `retain`, T3 | No successful-output change; exact PNG parser-input byte boundary/rejection plus preserve/outlined SVG nonapplication evidence |
| `scene.png-target-positive-finite-limit` | `retain`, T3 | No successful-output change; NaN/infinite/zero/negative target-dimension rejection tests, with arithmetic cases owned by T2a4 |

### Deferred T2a4 Matches

| Source match | Deferred responsibility |
| --- | --- |
| `src/render/svg_backend.rs:132-133,155-181` | PNG option/sizing validity and positive DPI, scale, width, and height policy. |
| `src/render/svg_backend.rs:140-150,183-230` | Canonical source-size agreement, source-to-target sizing, zero-size failure, target allocation dimensions, and repeated target-limit calls; T2a3 retains only the independently declared Scene limits. |
| `src/render/svg_backend.rs:232-242` | Source-to-target raster transform arithmetic. |
| `src/render/svg_backend.rs:135-145` | PNG-local host-font setup timing, SVG parse timing, and canonical-dimension lifecycle check; T2a1 remains the font-discovery source owner. |
| `src/render/svg_backend.rs:244-299`, excluding `ColorType::Rgba`, `BitDepth::Eight`, and `take_demultiplied` | Encoder dimensions, compression mapping, DPI metadata, pixels-per-meter conversion/range checks, header/data/finish publication, and output-byte lifecycle. |
| Dimension/allocation meaning of `Pixmap::new` and transform meaning of `resvg::render` at `src/render/svg_backend.rs:147-151` | PNG target sizing, allocation, and scaling; T2a3 owns only the same calls' transparent paint/compositing meaning. |

### Prior-Owned And Rejected T2a3 Matches

| Source match | Disposition and reason |
| --- | --- |
| `src/render.rs:1-8` | Rejected: module documentation, declarations, and visibility only. |
| Calls to `validate_*` in `render/svg_backend.rs` and `render/geometry.rs` | Prior-owned by T2a2 validation records. Concrete line/polygon no-op branches and concrete serialization/bounds remain T2a3 sources. |
| Calls to `rgb_to_hex` | Prior-owned by T2a1 `backend.svg.rgb-hex-serialization`; emitted paint/alpha attribute behavior is `backend.svg.incomplete-per-paint-alpha-serialization`. |
| Calls to `configure_system_fonts` | Prior-owned font discovery/selection behavior from T2a1; outlined conversion is a T2a3 consumer and PNG-local timing is deferred to T2a4. |
| Colors, widths, radii, fonts, labels, points, gradient stops, and bounds supplied to backend methods | Transported values owned by T2a1/T2a2 or later semantic inventories, not duplicate T2a3 sources. |
| Module prose, imports, ordinary derives, comments, and visibility in the admitted files | Rejected language/documentation plumbing; executable `Default` behavior in Geometry lifecycle is admitted. |
| Error-message wording and interpolated diagnostic values | Rejected as non-visual; each behavior branch is admitted or deferred separately. |
| `RenderedScene` `Debug` implementation at `src/scene.rs:19-28` | Rejected diagnostic representation. |
| `src/render/svg_backend.rs:759-890` and `src/render/geometry.rs:267-361` | Test-only evidence, never production property sources. |
| Scientific/statistical data search class | No source in the T2a3 manifest; projected coordinates and labels are transported data. |

### T2a3 Reconciliation

- The completed T2a3 count is exactly **45**: 20 SVG records, including the PNG
  RGBA8 paint/alpha boundary, 15 Geometry records, and 10 Scene records. Every
  descriptive key appears once in each keyed table in the same order.
- The initial 36 became 45 without discovering a new production source: SVG
  changed from 22 to 20 (`-2`) after PNG sizing/output ownership moved to T2a4,
  Geometry changed from 11 to 15 (`+4`) after lifecycle and path/text bounds
  were separated, and Scene changed from 3 to 10 (`+7`) after routes, positive
  checks, and four independent limit declarations were separated, for a net
  correction of `+9`.
- T2a1, T2a2, and T2a3 complete exactly **162 records**. Their then-current
  202-record four-part estimate assigned 40 records to T2a4; T2a4 source
  reconciliation below supersedes that estimate with **222 records**: 65 T2a1,
  52 T2a2, 45 T2a3, and 60 T2a4.
- `backend.svg.incomplete-per-paint-alpha-serialization` is the sole private
  paint-property placement and alpha-routing responsibility and remains owned by
  T5a. T2a1 `backend.svg.rgb-hex-serialization` now owns only deterministic
  color-channel/token spelling; T5c, T5d, and T6 complete annotation, scale,
  carrier, and public inputs without duplicating either source.
- The concrete SVG lifecycle remains T2a3 because it is executable serializer
  state-machine behavior. T2a2 owns the abstract lifecycle contract, while T2a4
  owns orchestration and artifact publication.
- Independent review corrected post-format geometry collapse and stale SVG
  output, made point/polygon joins explicit, made collision bounds paint-aware,
  connected dormant Geometry lifecycle methods to decoration planning, scoped
  maximum Scene limits to PNG, deduplicated positive source dimensions into
  canonical scene construction, and required exact generic/bracket whitespace
  preservation without changing the key set.
- Bracket path serialization/bounds migrate in T5c; bracket label
  serialization/bounds migrate in T5a. Equal Scene limits with independent
  declarations remain independent records.
- Source reconciliation and identical key-count/order checks passed; final
  independent ownership review found no remaining semantic or schema issue.
  Tracked-diff checks and `just verify` passed on 2026-08-26. No production code,
  tests, snapshots, or other specifications changed in T2a3.

## T2a4: Lifecycle, Output, And Artifact Foundations

This completed section owns Plot lifecycle and sequencing, concrete
Deferred and Null backend behavior, terminal-output policy and timing, artifact
staging, and PNG options, sizing, transforms, compression, metadata, and encoder
lifecycle. It includes the exact deferred PNG expression contexts in
`src/render/svg_backend.rs` and the exact public PNG policy declarations in
`src/spec/output.rs`. T2a1 through T2a3 remain the sole owners of palette values,
abstract backend protocol and validation, SVG/PNG paint serialization, immutable
scene routes, and Scene resource-limit declarations.

### T2a4 Source Manifest

| ID | Production source | Owned context | Audit disposition |
| --- | --- | --- | --- |
| A4-M1 | `src/orchestrator.rs:27-39,56-74,76-81,84-112,136-142,189` | Public/internal preflight routes, selected-kind and closed renderer dispatch, panel-pass sequencing, and post-panel palette reconstruction | Records `lifecycle.plot.public-preflight-route` through `lifecycle.plot.post-panel-appearance-reconstruction`; mixed domain, label, legend, and text-fit contexts are deferred below. |
| A4-M2 | `src/orchestrator/lifecycle.rs:11-24,33,38-70,75-80,92-93` | Configuration-before-backend, one-canvas routing/finalization, outer-decoration palette reconstruction, plan/render/decor sequencing, category-order validation timing, and panel Null preflight | Records `lifecycle.plot.configuration-before-canvas` through `lifecycle.plot.panel-null-preflight-route`; data, category-order semantics, family-support, matrix-layout, and hue contexts are deferred or rejected below. |
| A4-M3 | `src/orchestrator/backend.rs:11-228` | Concrete Deferred state machine and each concrete Null terminal selection | Records `backend.deferred.lazy-canvas-state-machine` through `backend.null.bracket-validation-parity`; called validators and inherited trait defaults remain prior-owned. |
| A4-M4 | `src/orchestrator/output.rs:10-61` | Plot scene preflight/render and five consuming Plot terminal selections | Records `output.plot.preflight-before-scene-render` through `output.plot.explicit-png-terminal`. |
| A4-M5 | `src/output.rs:8-59` | Reusable scene handle, five consuming Output terminals, and conversion-before-staging timing | Records `output.reusable-scene-handle` through `output.png.explicit-publication-terminal`; `write_atomic` is assigned to A4-M6. |
| A4-M6 | `src/output.rs:62-68`; `src/orchestrator/artifacts.rs:8-135` | Staged write transaction, unique sibling reservation, platform publication, explicit cleanup, Drop retry, and missing-file fallback | Records `artifact.sibling-temp-reservation` through `artifact.missing-file-cleanup-fallback`. |
| A4-M7 | `src/spec/output.rs:3-52` | PNG sizing/compression fields, defaults, closed modes/presets, and CSS reference density | Records `output.png.options.sizing-carrier` through `output.png.compression.closed-presets`; `SVG_CSS_DPI` completes `backend.png.preparse-target-prediction`. Ordinary derives are rejected below. |
| A4-M8 | `src/render/svg_backend.rs:125-299`, limited to T2a3's Deferred T2a4 Matches | PNG option validation, format-local font timing, parse/dimension lifecycle, target prediction/sizing/allocation, transform, compression, metadata, and encoder lifecycle | Records `backend.png.option-validation` through `backend.png.encoder-output-lifecycle`; RGBA8 paint contexts and Scene limit implementations remain prior-owned. |

The manifest is exact for T2a4. It admits no `src/canvas.rs` source. Canvas
composition lifecycle is T2d-owned, while independent Canvas terminal selections
remain a T2h residual under the existing source allocation.

### T2a4 Consumer And Evidence References

| Code | Exact production consumers or evidence |
| --- | --- |
| A4-C-LIFECYCLE | `Plot::preflight` and every Plot terminal call enter `orchestrator.rs`/`orchestrator/lifecycle.rs`; facets call `preflight_panel`; Canvas calls the Deferred adapter as a consumer at `canvas.rs:390-399`. |
| A4-C-BACKEND | Every backend scope and primitive emitted during final Plot or Canvas rendering passes through `DeferredBackend`; full Plot preflight uses Deferred over `NullBackend`, while panel, facet-planning, and coordinated-panel passes use Null directly. T2a2 consumer references A2-C-POINT through A2-C-SCOPE enumerate the concrete producers. |
| A4-C-SCENE | `orchestrator/output.rs:11-19` constructs one `RenderedScene`; `Output` stores it and consuming terminal methods reuse it. T2a3 A3-C-SCENE owns the scene bytes and conversion routes. |
| A4-C-TERMINAL | Plot terminals at `orchestrator/output.rs:23-61` create the initial scene; Output terminals at `output.rs:21-59` select outlined SVG, exact SVG, default PNG, preview PNG, or explicit PNG without rerendering. |
| A4-C-ARTIFACT | Every Output publication reaches `write_atomic`, sibling reservation, platform publication, explicit cleanup, and Drop fallback. Chained Output calls perform independent transactions. |
| A4-C-PNG | `Output::png_with_options` calls `RenderedScene::png_bytes`; `rasterize_svg_bytes_to_png` validates sizing, resolves conversion fonts, parses the canonical scene, allocates/transforms, and encodes. |
| A4-E-LIFECYCLE | `src/orchestrator/tests/lifecycle.rs`, `src/orchestrator/tests.rs`, and `tests/composition/canvases.rs` cover begin/end counts, preflight/render parity, failure precedence, and composed lifecycle consumers. |
| A4-E-OUTPUT | `tests/output/multiformat.rs`, `tests/output/png.rs`, and `tests/rendering/contract.rs` cover one-scene reuse, terminal selection, dimensions, preserve/outlined SVG, and conversion failures. |
| A4-E-ARTIFACT | `src/orchestrator/tests/artifacts.rs` covers unique staging, destination preservation, cleanup, chained publication, and failure paths. |
| A4-E-PNG | `tests/output/png.rs`, `tests/output/multiformat.rs`, and `render/svg_backend.rs:759-890` cover sizing modes, compression mappings, metadata, RGBA output, limits, and deterministic encoding. |

### T2a4 Current State

| Property | Current source and value | Current consumers | Current behavior | Current layout path | Semantic role | Audit evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `lifecycle.plot.public-preflight-route` | A4-M1 `Plot<K>::preflight` clones and erases, then `preflight_erased` renders through a fresh `NullBackend` | A4-C-LIFECYCLE | Runs the complete validating render path without files; errors match that pass, but a later terminal call repeats it | `measure`, `reserve`, collision, `geometry` | Public full-Plot validation route | A4-M1; clone/erase and Null terminal admitted; A4-E-LIFECYCLE |
| `lifecycle.plot.selected-kind-backend-route` | A4-M1 `render_with_backend_erased` calls `render_kind_erased(self.kind(), backend)` | All actual Plot scene renders | Uses the stored selected kind with no alternate/fallback renderer | All rendering phases | Selected Plot-to-backend lifecycle entry | A4-M1; terminal kind selection admitted |
| `lifecycle.plot.closed-renderer-dispatch` | A4-M1 exhaustive `PlotKind` match to one plot renderer; grouped bar/horizontal-bar variants share implementations | Every panel render | Exactly one closed renderer arm executes; renderer errors propagate | `geometry`, draw | Closed plot-family render dispatch | A4-M1; each terminal arm admitted; domain assignment excluded |
| `lifecycle.plot.panel-post-render-order` | A4-M1 family render, then `layers::apply`, resolved labels, axis-text validation, and panel text | Every panel | Fixed call order; a failure stops later steps, while earlier backend calls remain in the deferred pass | `measure`, `geometry`, draw; z-order | Canonical inner panel-pass order | A4-M1; sequence admitted; semantic layer/label values deferred |
| `lifecycle.plot.post-panel-appearance-reconstruction` | A4-M1 `let palette = chart.palette()` at `orchestrator.rs:189` | Axis tick/title text-fit validation after every `render_panel` pass, including Null preflight and final scene rendering | Rebuilds an owned preset after panel rendering; current closed preset resolution is deterministic, so values equal other reconstructions even though allocation and lookup are independent | `measure` | Post-panel axis-text validation appearance lookup | A4-M1; values prior-owned by T2a1; every pass and reconstruction timing admitted |
| `lifecycle.plot.configuration-before-canvas` | A4-M2 `validate_configuration(kind)?` before Deferred construction | Every Plot backend render, including Null preflight | Configuration failure occurs before backend begin; no fallback or finalization is attempted | Before all layout/draw | Validation-before-canvas boundary | A4-M2; ordering admitted; validation internals later-owned |
| `lifecycle.plot.one-canvas-route-and-finalization` | A4-M2 captures `self.width()/height()`, selects facet/open route, then `backend.finish(rendered, "backend canvas")` | Every Plot backend render | One route runs; Deferred decides whether begin/end occur and combines render/end errors | Gates all output | One-canvas routing and finalization | A4-M2; dimensions, route branch, and finish call admitted; A4-E-LIFECYCLE |
| `lifecycle.plot.outer-decoration-appearance-reconstruction` | A4-M2 `let palette = self.palette()` at `lifecycle.rs:33` | Grid, axes, and Canvas text decoration calls in every unfaceted pass | Rebuilds deterministic appearance before planning in both Null preflight and final rendering; values equal other current preset reconstructions but the owned lookup remains independent | `geometry`, draw | Outer-decoration appearance lookup | A4-M2; values prior-owned; pass timing and reconstruction admitted |
| `lifecycle.plot.plan-before-draw` | A4-M2 `plan_panel(self, kind, cell, LegendRole::Panel)` completes before the first grid draw | Unfaceted Plot render | Planning can render through validation/geometry consumers before final drawing; no plan object is reused by public preflight | `measure`, `reserve`, collision, `geometry` | Plan-before-final-render sequencing | A4-M2; plan call/timing admitted |
| `lifecycle.plot.grid-before-panel-order` | A4-M2 Standard branch calls `apply_grid` before `render_panel` | Standard unfaceted plots | Grid is drawn before marks; SelfManaged skips this call | `geometry`, draw; z-order | Grid-before-panel ordering | A4-M2; ordering only; grid policy/value sources later-owned |
| `lifecycle.plot.planned-chart-handoff` | A4-M2 `plan.chart_for_render(self)` selects a render chart, then context and `render_panel`; outer palette remains from original `self` | Unfaceted Plot render | Planned chart changes can affect panel rendering while outer decoration appearance remains separately reconstructed from the original Plot | `measure`, `geometry`, draw | Plan-to-render Plot/appearance handoff | A4-M2; chart selection and sequencing admitted; data rows excluded |
| `lifecycle.plot.post-panel-decoration-order` | A4-M2 significance after panel, then Standard axes, legend, and canvas text; SelfManaged performs none | Unfaceted Plot render | Fixed post-mark order; first error stops later decorations | `geometry`, draw; z-order | Marks-before-decoration ordering | A4-M2; sequence admitted; annotation/axis/legend values later-owned |
| `lifecycle.plot.panel-null-preflight-route` | A4-M2 `preflight_panel` first calls `validate_category_order_configuration(kind)`, then builds one context and calls `render_panel(..., &mut NullBackend)` | Facet and coordination planning | Category-order validation precedes context construction and direct Null rendering; no canvas begin/end occurs, and the returned `PlotRenderResult` feeds later coordination | `measure`, `geometry` | Panel-level validating render route | A4-M2; validation timing and Null terminal admitted; category-order semantics are deferred to T2b/T2g and data transport is rejected |
| `backend.deferred.lazy-canvas-state-machine` | A4-M3 stores backend, configured width/height, and `begun=false`; every scope/draw lazily starts; explicit begin accepts caller dimensions; explicit `end` ensures start then forwards without clearing `begun`; `finish` conditionally starts/ends | A4-C-BACKEND | First operation starts once; successful empty render is started then ended; pre-start failure returns untouched; begun render normally attempts end and combines errors; explicit begin may use dimensions differing from stored values; explicit end permits later forwarding and can make `finish` call the underlying end again | Gates all backend output | Concrete lazy one-canvas adapter | A4-M3; constructor, every delegate, explicit-end defect, and finish branches admitted; A4-E-LIFECYCLE |
| `backend.null.canvas-lifecycle-parity` | A4-M3 Null `begin` and `end` unconditionally return `Ok(())` | Full Plot preflight through Deferred; direct panel preflight bypasses these lifecycle methods | Ignores dimensions and canvas balance; ordinary layer defaults are inherited no-ops | `none`; lifecycle only | Validating-null canvas lifecycle | A4-M3; concrete methods and direct-panel bypass admitted; trait defaults prior-owned |
| `backend.null.clip-validation-parity` | A4-M3 Null `begin_clip` calls `validate_clip_region`; `end_clip` always succeeds | Null render paths using clipping | Validates each opening region but stores no stack and accepts unmatched closure | `geometry` validation only | Validating-null clip behavior | A4-M3; concrete selection admitted; validator prior-owned |
| `backend.null.scatter-validation-parity` | A4-M3 Null scatter validates points then point style | Null render paths | Empty scatter is accepted; invalid consumed points/style fail in that order; no geometry is stored | `geometry` validation only | Validating-null scatter terminal | A4-M3; terminal selection/order admitted; validators prior-owned |
| `backend.null.line-degenerate-parity` | A4-M3 independent `<2` early `Ok(())`, else points then line-style validation | Null line calls | Degenerate lines bypass invalid coordinates/style exactly like concrete output backends | `geometry` validation/no-op | Validating-null line arity behavior | A4-M3; branch admitted separately; A4-E-LIFECYCLE |
| `backend.null.polygon-degenerate-parity` | A4-M3 independent `<3` early `Ok(())`, else points then fill-style validation | Null polygon calls | Degenerate polygons bypass invalid coordinates/style exactly like concrete output backends | `geometry` validation/no-op | Validating-null polygon arity behavior | A4-M3; branch admitted separately |
| `backend.null.gradient-validation-parity` | A4-M3 forwards complete bounds/vector/stops to `validate_linear_gradient` | Null gradient calls | Complete gradient protocol is checked; no geometry is stored | `geometry` validation only | Validating-null gradient terminal | A4-M3; terminal selection admitted; validator prior-owned |
| `backend.null.text-validation-parity` | A4-M3 ignores x/y/text and validates only current `TextStyle` | Null text calls | Only finite rotation is checked; font, glyph, size, content, and placement parity remain incomplete | `geometry` validation only | Validating-null text terminal | A4-M3; ignored inputs and validator selection admitted |
| `backend.null.rounded-rectangle-validation-parity` | A4-M3 validates rounded geometry then fill style | Null rounded-container calls | Accepts current zero extents and oversized radius according to prior validator; style runs only after geometry succeeds | `geometry` validation only | Validating-null rounded-container terminal | A4-M3; order admitted; validators prior-owned |
| `backend.null.bracket-validation-parity` | A4-M3 forwards bracket/style to `validate_bracket` | Null significance calls | Validates current geometry/style contract and stores no obstacle | `reserve`, `geometry` validation | Validating-null bracket terminal | A4-M3; terminal selection admitted; validator prior-owned |
| `output.plot.preflight-before-scene-render` | A4-M4 `render_scene` first calls `self.preflight_erased()?` | Initial Plot terminal calls | Executes one complete Null render before constructing the actual SVG backend; successful output therefore plans/renders twice | All preflight layout phases | Plot preflight-to-scene timing | A4-M4; distinct first pass admitted; A4-E-OUTPUT |
| `output.plot.canonical-scene-render` | A4-M4 constructs `SvgBackend::in_memory(width,height)`, renders again, and builds `RenderedScene` from repeated dimensions plus `into_bytes()` | Initial Plot terminal calls and subsequent Output reuse | Actual scene is a second complete render; scene construction itself accepts supplied dimensions/bytes under T2a3 behavior | All render phases, then `none` storage | Plot canonical-scene construction | A4-M4; actual pass/dimension reads admitted; scene carrier prior-owned |
| `output.plot.outlined-svg-terminal` | A4-M4 consuming `Plot::svg`: erase, render scene, wrap Output, select `Output::svg` | Public callers | Publishes immediately and returns reusable Output; no sticky format state | `none` after scene | Initial outlined-SVG terminal policy | A4-M4; independent terminal selection admitted |
| `output.plot.preserve-text-svg-terminal` | A4-M4 consuming `Plot::svg_preserve_text` selects `Output::svg_preserve_text` | Public callers | Same one-scene lifecycle, exact-text SVG selection | `none` after scene | Initial preserve-text SVG terminal policy | A4-M4; independent terminal selection admitted |
| `output.plot.default-png-terminal` | A4-M4 consuming `Plot::png` selects `Output::png` | Public callers | Same one-scene lifecycle, then default PNG policy | `none` after scene | Initial default-PNG terminal policy | A4-M4; independent terminal selection admitted |
| `output.plot.preview-png-terminal` | A4-M4 consuming `Plot::png_preview` selects `Output::png_preview` | Public callers | Same one-scene lifecycle, then preview PNG policy | `none` after scene | Initial preview-PNG terminal policy | A4-M4; independent terminal selection admitted |
| `output.plot.explicit-png-terminal` | A4-M4 consuming `Plot::png_with_options` passes exact `PngOptions` to Output | Public callers | Options are call-local and copied unchanged; no sticky state | `none` after scene | Initial explicit-PNG terminal policy | A4-M4; independent terminal selection admitted |
| `output.reusable-scene-handle` | A4-M5 `Output { scene: RenderedScene }`; `new` moves exactly one scene | Every chained Output call | Consuming methods return the same handle after success, so subsequent formats reuse the scene without rerendering | `none`; already-rendered layout | One-scene reusable output ownership | A4-M5; distinct ownership wrapper admitted; T2a3 owns inner scene |
| `output.outlined-svg.conversion-before-staging` | A4-M5 calls `svg_with_text_as_paths()` before `write_atomic` | `Output::svg` | Font/conversion failure occurs before any staging path is reserved | `geometry` conversion after layout | Derive-before-stage outlined-SVG timing | A4-M5; call order admitted; converter prior-owned |
| `output.outlined-svg.publication-terminal` | A4-M5 consuming `Output::svg` publishes converted bytes with staging extension `"svg"` and returns self | Public/chained callers | One independent outlined publication; prior successful files remain if this call fails | `none` | Reusable outlined-SVG terminal policy | A4-M5; terminal and format selection admitted |
| `output.preserve-text-svg.publication-terminal` | A4-M5 consuming `Output::svg_preserve_text` publishes borrowed canonical bytes with `"svg"` | Public/chained callers | No parse, font lookup, or conversion before staging | `none` | Reusable preserve-text SVG terminal policy | A4-M5; independent terminal selection admitted |
| `output.png.default-terminal-options` | A4-M5 `Output::png` delegates with `PngOptions::default()` | Public/chained callers | Selects 300-DPI Balanced on every call; no retained options | `none` | Reusable default-PNG policy selection | A4-M5; default call admitted; declaration is separate A4-M7 source |
| `output.png.preview-terminal-options` | A4-M5 literal `PngOptions { Source, Balanced }` | Public/chained callers | Source dimensions, Balanced compression, and no DPI metadata; independent of prior calls | `none` | Reusable preview-PNG policy selection | A4-M5; complete literal preset admitted |
| `output.png.conversion-before-staging` | A4-M5 calls `scene.png_bytes(&options)` before `write_atomic` | `Output::png_with_options` and delegating PNG terminals | Validation, fonts, parse, allocation, and encoding all finish before staging reservation | `none` after raster conversion | Derive-before-stage PNG timing | A4-M5; call order admitted; route prior-owned |
| `output.png.explicit-publication-terminal` | A4-M5 consuming `Output::png_with_options` publishes derived bytes with `"png"` and returns self | Public/chained callers | Exact call-local options; independent transaction; no fallback rasterizer | `none` | Reusable explicit-PNG terminal policy | A4-M5; independent terminal selection admitted |
| `artifact.sibling-temp-reservation` | A4-M6 counter starts `0`; sibling name appends `.lichtung-tmp-{pid}-{sequence}.{extension}`; `create_new`; collision retries; missing file name/other I/O fail | Every file publication | Reserves an empty unique sibling atomically; Relaxed sequence plus PID avoids process-local reuse; stale collisions loop | `none` | Unique same-directory staging reservation | A4-M6; declaration, naming, and all loop branches admitted; A4-E-ARTIFACT |
| `artifact.posix-rename-publication` | A4-M6 non-Windows `std::fs::rename(staged, final_path)` | Non-Windows publications | Ordinary same-directory rename replaces an existing file atomically where the filesystem supports it; errors propagate | `none` | POSIX-like staged destination replacement | A4-M6; cfg branch admitted |
| `artifact.windows-publication` | A4-M6 Windows: rename directly when absent; reject existing non-file; otherwise generate one unreserved sibling backup candidate with no collision retry, rename destination to it, rename staged to final, rollback on failure, and ignore backup-removal failure | Windows publications | Existing-file replacement is not atomic: the destination is absent between renames; an existing backup candidate makes the first rename fail; failed rollback leaves the destination absent and an untracked backup residual; successful backup cleanup failure is hidden and can also leave an untracked backup | `none` | Windows staged destination replacement | A4-M6; absent/non-file, one-shot backup collision, install/rollback, residual-backup, and ignored-cleanup branches admitted; A4-E-ARTIFACT |
| `artifact.staged-write-transaction` | A4-M6 `write_atomic` reserves, enters `with_temporary_artifacts`, writes bytes, publishes, then combines work/cleanup results | Every Output terminal | Conversion is already complete; write/publish failure triggers explicit cleanup; primary and cleanup failures are preserved by prior `finish_scope` behavior | `none` | Complete staged file transaction | A4-M6; output/helper contexts grouped by one role |
| `artifact.explicit-cleanup-failure-retention` | A4-M6 removes every tracked path; successes leave the list, failures append diagnostics and remain for Drop; empty failures succeed, otherwise aggregate | Staged transaction cleanup | Attempts every path; cleanup failure can turn successful work into error or join a primary error | `none` | Deterministic explicit temporary cleanup | A4-M6; retain branch and aggregate branch admitted |
| `artifact.drop-best-effort-retry` | A4-M6 `Drop` retries every retained path and discards each result | Cleanup-error and unwind fallback | No error can escape Drop; failed files may remain | `none` | Best-effort terminal cleanup fallback | A4-M6; ignored-result branch admitted |
| `artifact.missing-file-cleanup-fallback` | A4-M6 `remove_if_present`: success or `NotFound` is `Ok`; every other error remains | Explicit and Drop cleanup | Concurrent/already-completed removal is idempotent; permission/type failures are real cleanup failures | `none` | Idempotent temporary-file removal | A4-M6; all result branches admitted |
| `output.png.options.sizing-carrier` | A4-M7 public `PngOptions::sizing: PngSizing` | Plot/Output explicit PNG terminals and A4-C-PNG | Call-local copied closed mode; no construction validation | `none`; determines target dimensions | PNG target-sizing policy carrier | A4-M7; split field admitted |
| `output.png.options.compression-carrier` | A4-M7 public `PngOptions::compression: PngCompression` | A4-C-PNG encoder | Call-local copied preset; no sticky storage | `none`; bytes/encode work only | PNG compression policy carrier | A4-M7; split field admitted |
| `output.png.options.default-preset` | A4-M7 `PngOptions::default = { Dpi(300.0), Balanced }` | `Output::png` | One complete publication preset; 300 is `f32` DPI and writes matching metadata | `none`; target sizing/bytes | Default PNG publication policy | A4-M7; both literals share one preset role |
| `output.png.sizing.closed-modes` | A4-M7 `Source`, `Dpi(f32)`, `Scale(f32)`, `Width(u32)`, `Height(u32)` | A4-C-PNG | Closed mutually exclusive sizing selection; modes request source identity or aspect-preserving scaling, but final integer aspect is approximate after mode-specific rounding; raw payloads can be invalid | `none`; target dimensions | Closed call-local PNG sizing modes | A4-M7; every variant admitted |
| `output.png.compression.closed-presets` | A4-M7 `None`, `Fast`, default `Balanced`, `High` | A4-C-PNG | Closed high-level selection; derived `PngCompression::default` is independently Balanced | `none`; encoded bytes/work | Closed PNG compression presets | A4-M7; variants and executable default admitted |
| `backend.png.option-validation` | A4-M8 `validate_png_options`: Source succeeds; DPI/Scale require finite `>0`; Width/Height require `>0` | A4-C-PNG before fonts/parse | First invalid sizing error stops conversion; compression has no invalid state | `none`; conversion gate | Terminal PNG option validity | A4-M8; match and helper branches admitted |
| `backend.png.format-local-font-setup-timing` | A4-M8 `configure_system_fonts(options.fontdb_mut())` after target prevalidation and before PNG SVG parse | PNG only; outlined SVG has a separate call | Host discovery/selection occurs after canonical layout and per PNG conversion; preserve SVG bypasses it | Post-layout conversion | PNG-local generation-font timing | A4-M8; call timing admitted; algorithm prior-owned |
| `backend.png.svg-parse-and-dimension-check` | A4-M8 default usvg options, parse exact scene bytes, compare parsed integer size to supplied CSS width/height | A4-C-PNG | Parse or mismatch is `Error::Render`; no alternate parser or dimension substitution | Post-layout conversion gate | Canonical SVG-to-PNG parser boundary | A4-M8; parse/default/dimension branch admitted |
| `backend.png.preparse-target-prediction` | A4-M7 `SVG_CSS_DPI=96.0`; A4-M8 computes floating target for all modes and calls Scene target limits before parse and again from `png_target_size` | A4-C-PNG | DPI performs `f32` division by `96.0` before widening the factor to `f64`; Scale widens its `f32`; Width/Height use `f64` aspect arithmetic; this duplicated prediction can differ from later `f32` integer sizing | `none`; preallocation work gate | Preparse PNG target prediction | A4-M7/A4-M8; exact per-mode arithmetic, all arms, and repeated call timing admitted |
| `backend.png.integer-target-sizing` | A4-M8 `IntSize` Source/`scale_by`/`scale_to_width`/`scale_to_height`, zero-size failure, then integer dimensions sent to Scene limits | A4-C-PNG | Source is identity; DPI/Scale independently multiply each source axis in `f32` and round each product to nearest integer; Width/Height pin the requested axis and compute the other in `f32` with `ceil`; aspect is therefore approximate after integer rounding; `None` rejects zero-sized results and no clamp occurs | `none`; final raster dimensions | Canonical integer PNG target sizing | A4-M8; all mode formulas, round/ceil behavior, zero branch, and repeated limit call admitted; limits prior-owned |
| `backend.png.target-pixmap-allocation` | A4-M8 `Pixmap::new(size.width(), size.height())` and `None` allocation failure | A4-C-PNG | Allocates only after source/target checks; no lower-resolution fallback | `none`; raster resource allocation | Validated PNG target allocation | A4-M8; dimension/allocation meaning admitted; paint meaning prior-owned |
| `backend.png.source-target-transform` | A4-M8 independent x/y scale `target/source`, passed to `resvg::render` | A4-C-PNG | Exact integer target fills the pixmap; aspect-preserving modes normally give equal scales, but transform does not enforce equality | `geometry`; raster scaling | Canonical source-to-target raster transform | A4-M8; arithmetic/render transform meaning admitted |
| `backend.png.compression-mapping` | A4-M8 closed map to `NoCompression`, `Fast`, `Balanced`, `High` | PNG encoder | One direct mapping per preset; no adaptive or fallback compression | `none`; encoded bytes/work | PNG encoder compression adapter | A4-M8; every match arm admitted; A4-E-PNG |
| `backend.png.dpi-metadata` | A4-M8 DPI mode alone sets equal x/y pixels-per-meter with `png::Unit::Meter`; computes `f64::from(dpi)/0.0254`, checks finite/positive and `u32::MAX`, then rounds to `u32` | DPI PNG encoding | Source/Scale/Width/Height omit metadata; the `u32::MAX` helper branch is defensive but unreachable through production PNG conversion because Scene target-dimension limits reject such DPI first; no clamp | `none`; metadata/bytes | PNG physical-resolution metadata | A4-M8; option branch, Meter unit, conversion, defensive range branch, production ordering, and rounding admitted |
| `backend.png.encoder-output-lifecycle` | A4-M8 encoder uses pixmap dimensions, writes header, demultiplied image data, finishes, and returns accumulated bytes | A4-C-PNG and output conversion timing | Header/data/finish errors propagate; bytes publish only after complete finish; RGBA8/de-multiplication semantics are prior-owned | `none`; final bytes | Deterministic PNG encoder lifecycle | A4-M8; dimensions/error/output lifecycle admitted; paint contexts excluded |

### T2a4 Target State

| Property | Target classification and owner | Target resolution | Target validation | Public surface | Relationships |
| --- | --- | --- | --- | --- | --- |
| `lifecycle.plot.public-preflight-route` | Renderer invariant; private canonical Plot preflight entry | Resolved Plot before any backend canvas | Run the same validator, resolved appearance, planning, and validating draw path as scene creation; no files or alternate route | Retained `Plot::preflight` behavior | Null counterpart `backend.null.canvas-lifecycle-parity`; scene counterpart `output.plot.preflight-before-scene-render` |
| `lifecycle.plot.selected-kind-backend-route` | Renderer invariant; private selected-kind backend routing | Resolved Plot at one-canvas entry | Stored kind is selected exactly once and must have one closed renderer arm | Private-only | Dispatch `lifecycle.plot.closed-renderer-dispatch`; canvas route `lifecycle.plot.one-canvas-route-and-finalization` |
| `lifecycle.plot.closed-renderer-dispatch` | Renderer invariant; private closed Plot renderer dispatch | Resolved Plot/panel | Every admitted `PlotKind` has exactly one terminal renderer; no fallback arm | Private-only | Entry `lifecycle.plot.selected-kind-backend-route`; inner order `lifecycle.plot.panel-post-render-order` |
| `lifecycle.plot.panel-post-render-order` | Renderer invariant; private canonical inner panel-pass sequencing | Resolved Plot/panel after family renderer selection | One resolved appearance and immutable plan govern family render, supplemental layers, labels, text checks, and panel text; first error stops the pass | Private-only | Outer handoff `lifecycle.plot.planned-chart-handoff`; appearance `lifecycle.plot.post-panel-appearance-reconstruction` |
| `lifecycle.plot.post-panel-appearance-reconstruction` | Renderer invariant; private shared resolved Plot appearance | Before Plot/panel preflight measurement | Resolve once before planning; post-panel text validation in every pass borrows the same immutable value used by measurement and final drawing | Private-only | Split current counterpart `lifecycle.plot.outer-decoration-appearance-reconstruction`; T2a1 preset routing source |
| `lifecycle.plot.configuration-before-canvas` | Renderer invariant; private validation-before-backend boundary | Resolved Plot before begin | Complete data-independent and data-dependent configuration validation before backend begin; reject without fallback | Private-only | Canvas lifecycle `lifecycle.plot.one-canvas-route-and-finalization` |
| `lifecycle.plot.one-canvas-route-and-finalization` | Renderer invariant; private balanced one-canvas orchestration | One resolved Plot and canonical canvas dimensions | Exactly one selected route, one successful begin before draws, and one attempted end; preserve render/end failures | Private-only | Abstract lifecycle `backend.protocol.begin-end-lifecycle`; concrete adapter `backend.deferred.lazy-canvas-state-machine` |
| `lifecycle.plot.outer-decoration-appearance-reconstruction` | Renderer invariant; private shared resolved Plot appearance | Resolved once before plan, preflight, and final draw | Remove the independent outer-decoration reconstruction; planning, Null validation, and final decorations consume the exact shared value | Private-only | Split counterpart `lifecycle.plot.post-panel-appearance-reconstruction`; plan `lifecycle.plot.plan-before-draw` |
| `lifecycle.plot.plan-before-draw` | Renderer invariant; private immutable Plot render plan | After resolved appearance and before backend begin/draw | Plan once with complete resolved appearance; validate layout, collision, and representability before final drawing | Private-only | Handoff `lifecycle.plot.planned-chart-handoff`; output preflight `output.plot.preflight-before-scene-render` |
| `lifecycle.plot.grid-before-panel-order` | Renderer invariant; private grid-before-marks ordering | Final Plot draw | Standard grid remains before panel marks; SelfManaged ownership is explicit and validated | Private-only | T2a3 `backend.svg.layer-buffering-and-grid-order`; post-order `lifecycle.plot.post-panel-decoration-order` |
| `lifecycle.plot.planned-chart-handoff` | Renderer invariant; private plan-to-render Plot/appearance handoff | One resolved Plot and immutable plan | Planned data/range substitutions cannot change resolved appearance; panel and outer decorations consume one owner | Private-only | Plan `lifecycle.plot.plan-before-draw`; inner order `lifecycle.plot.panel-post-render-order` |
| `lifecycle.plot.post-panel-decoration-order` | Renderer invariant; private marks-before-decoration ordering | Final Plot draw | Closed decoration mode and deterministic significance/axis/legend/text order; one resolved style set | Private-only | Grid sibling `lifecycle.plot.grid-before-panel-order`; T2d/T2f semantic decoration owners |
| `lifecycle.plot.panel-null-preflight-route` | Renderer invariant; private panel-level validating route | Facet/coordination preflight from resolved Plot | Run the later-owned category-order validator at the canonical pre-panel point, then execute the same panel plan and terminal validation as final render without opening a canvas or mutating appearance | Private-only | Null terminals `backend.null.canvas-lifecycle-parity` through `backend.null.bracket-validation-parity`; category-order semantics remain T2b/T2g-owned |
| `backend.deferred.lazy-canvas-state-machine` | Renderer invariant; private lazy one-canvas adapter | Canonical Plot/Canvas dimensions at backend boundary | Stored dimensions are authoritative; reject mismatched explicit begin; start at most once, transition to an explicit ended state, reject every post-end operation, call the underlying end exactly once, and preserve primary/finalization errors | Private-only | Abstract lifecycle `backend.protocol.begin-end-lifecycle`; Plot owner `lifecycle.plot.one-canvas-route-and-finalization` |
| `backend.null.canvas-lifecycle-parity` | Renderer invariant; private validating-null canvas lifecycle | Preflight backend boundary | Enforce fresh begin/end balance and canonical dimensions without producing bytes; no draws outside an open validation canvas | Private-only | Deferred sibling `backend.deferred.lazy-canvas-state-machine`; abstract lifecycle prior-owned |
| `backend.null.clip-validation-parity` | Renderer invariant; private validating-null clip protocol | Preflight backend boundary | Validate regions and LIFO balance identically to final/geometry backends; reject unmatched close/end | Private-only | Prior owners `backend.protocol.clip-scope-contract` and `backend.validation.clip-region` |
| `backend.null.scatter-validation-parity` | Renderer invariant; private validating-null scatter parity | Final validating draw boundary | Match final consumed-point, complete RGBA point-style, outline, and extent validation | Private-only | Prior owners `backend.validation.primitive-points` and `backend.validation.point-style`; T6 carrier connection |
| `backend.null.line-degenerate-parity` | Renderer invariant; private validating-null line parity | Final validating draw boundary | Fewer than two points consumes neither coordinates nor style; every nondegenerate line receives final line validation | Private-only | Prior owner `backend.validation.line-style`; concrete SVG sibling `backend.svg.line-degenerate-noop` |
| `backend.null.polygon-degenerate-parity` | Renderer invariant; private validating-null polygon parity | Final validating draw boundary | Fewer than three points consumes neither coordinates nor style; every nondegenerate polygon receives complete fill/stroke validation | Private-only | Prior owner `backend.validation.fill-style`; concrete SVG sibling `backend.svg.polygon-degenerate-noop` |
| `backend.null.gradient-validation-parity` | Renderer invariant; private validating-null gradient parity | Final validating draw boundary | Apply the one complete gradient protocol and complete stop paints with no sorting, repair, or fallback | Private-only | Prior owners `backend.validation.linear-gradient` and `backend.gradient.stop-carrier` |
| `backend.null.text-validation-parity` | Renderer invariant; private validating-null resolved-text parity | After font/glyph preflight and before final draw | Validate complete paint, stack, glyphs, positive metrics, content interpretation, finite placement, and rotation | Private-only | Prior owners `backend.validation.text-style` and `backend.text.paint-carrier`; timing `backend.png.format-local-font-setup-timing` |
| `backend.null.rounded-rectangle-validation-parity` | Renderer invariant; private validating-null rounded-container parity | Final validating draw boundary | Match final finite geometry, nonnegative extents, admitted radius, complete fill/stroke, and far-edge checks | Private-only | Prior owners `backend.validation.rounded-rectangle`, `backend.validation.fill-style`, `backend.fill.paint-carrier`, and `backend.fill.stroke-carrier` |
| `backend.null.bracket-validation-parity` | Renderer invariant; private validating-null bracket parity | After annotation/font preflight | Match final distinct anchors, complete path stroke and label text, gap, and representability validation | Private-only | Prior owners `backend.validation.significance-bracket`, `backend.bracket.typography-carrier`, `backend.bracket.combined-color-stroke-meaning`, and `backend.bracket.combined-color-text-meaning` |
| `output.plot.preflight-before-scene-render` | Renderer invariant; private single-resolution preflight-to-scene lifecycle | Initial Plot terminal before canonical scene creation | Resolve/validate/plan once, then reuse the exact resolved values and immutable plan for the one final scene render | Private-only | Public preflight `lifecycle.plot.public-preflight-route`; scene render `output.plot.canonical-scene-render` |
| `output.plot.canonical-scene-render` | Renderer invariant; private one-render canonical-scene construction | After successful initial Plot preflight | One canonical dimension pair, one complete SVG backend pass, successful finalization, and validated scene construction | Private-only | T2a3 `scene.canonical-immutable-storage`; handle `output.reusable-scene-handle` |
| `output.plot.outlined-svg-terminal` | Explicit policy; retained call-local `Plot::svg` terminal selection | Initial consuming Plot call | Complete one scene, then one outlined publication; no sticky state or alternate format | Retained `Plot::svg` | Output counterpart `output.outlined-svg.publication-terminal` |
| `output.plot.preserve-text-svg-terminal` | Explicit policy; retained call-local `Plot::svg_preserve_text` selection | Initial consuming Plot call | Complete one scene, then exact preserve-text publication | Retained `Plot::svg_preserve_text` | Output counterpart `output.preserve-text-svg.publication-terminal` |
| `output.plot.default-png-terminal` | Explicit policy; retained call-local `Plot::png` selection | Initial consuming Plot call | Complete one scene, then apply the one default PNG preset | Retained `Plot::png` | Output counterpart `output.png.default-terminal-options` |
| `output.plot.preview-png-terminal` | Explicit policy; retained call-local `Plot::png_preview` selection | Initial consuming Plot call | Complete one scene, then Source/Balanced preview policy | Retained `Plot::png_preview` | Output counterpart `output.png.preview-terminal-options` |
| `output.plot.explicit-png-terminal` | Explicit policy; retained call-local `Plot::png_with_options` selection | Initial consuming Plot call | Complete one scene and apply exact validated call-local options | Retained `Plot::png_with_options` | Output counterpart `output.png.explicit-publication-terminal` |
| `output.reusable-scene-handle` | Renderer invariant; private one-scene reusable terminal ownership | After initial scene creation and between consuming publications | Store exactly one validated immutable scene; successful methods return ownership unchanged; no rerender or sticky options | Opaque retained `Output` | Inner storage `scene.canonical-immutable-storage`; five Output terminal siblings |
| `output.outlined-svg.conversion-before-staging` | Renderer invariant; private derive-before-stage outlined-SVG timing | Terminal outlined-SVG request | Complete the one validated conversion before artifact reservation; conversion failure creates no staging file | Private-only | Converter `backend.svg.text-outline-conversion`; publication sibling `output.outlined-svg.publication-terminal` |
| `output.outlined-svg.publication-terminal` | Explicit policy; retained call-local `Output::svg` selection | Reusable Output terminal | Publish one fully derived outlined SVG independently and return the exact scene handle | Retained `Output::svg` | Timing `output.outlined-svg.conversion-before-staging`; artifact transaction |
| `output.preserve-text-svg.publication-terminal` | Explicit policy; retained call-local `Output::svg_preserve_text` selection | Reusable Output terminal | Publish exact canonical bytes independently with no font/conversion step | Retained `Output::svg_preserve_text` | T2a3 `scene.preserve-svg-routing`; artifact transaction |
| `output.png.default-terminal-options` | Explicit policy; retained call-local default-PNG selection | Reusable Output terminal | Select only `output.png.options.default-preset`; no retained/sticky options | Retained `Output::png` | Declaration `output.png.options.default-preset` |
| `output.png.preview-terminal-options` | Explicit policy; retained call-local preview-PNG selection | Reusable Output terminal | Select Source sizing and Balanced compression with no density metadata | Retained `Output::png_preview` | Modes `output.png.sizing.closed-modes` and `output.png.compression.closed-presets` |
| `output.png.conversion-before-staging` | Renderer invariant; private derive-before-stage PNG timing | Terminal PNG request | Validate, size, rasterize, and finish bytes before artifact reservation; conversion failure creates no staging file | Private-only | T2a3 `scene.png-routing`; publication sibling `output.png.explicit-publication-terminal` |
| `output.png.explicit-publication-terminal` | Explicit policy; retained call-local `Output::png_with_options` selection | Reusable Output terminal | Apply exact validated options, publish one fully derived PNG independently, and return the scene handle | Retained `Output::png_with_options` | Timing `output.png.conversion-before-staging`; options carriers |
| `artifact.sibling-temp-reservation` | Renderer invariant; private unique same-directory staging reservation | Before file write/publication | Require a file name, reserve one nonexisting sibling exclusively, retry collisions, and propagate every other I/O error | Private-only | Transaction `artifact.staged-write-transaction`; cleanup `artifact.missing-file-cleanup-fallback` |
| `artifact.posix-rename-publication` | Renderer invariant; private atomic same-directory replacement | Final non-Windows publication boundary | One ordinary rename must atomically install staged bytes or leave the old destination unchanged; reject unsupported filesystem behavior | Private-only | Windows sibling `artifact.windows-publication`; staging reservation |
| `artifact.windows-publication` | Renderer invariant; private atomic same-directory replacement | Final Windows publication boundary | Use a true atomic replace operation: success installs staged bytes in one indivisible step; every failure preserves the old destination; no backup/absence window or rollback chain | Private-only | POSIX sibling `artifact.posix-rename-publication`; cleanup transaction |
| `artifact.staged-write-transaction` | Renderer invariant; private complete staged-publication transaction | One terminal file call | Reserve sibling, write complete bytes, atomically publish, always attempt explicit staging cleanup, report and aggregate failures, retain failed paths for best-effort Drop retry, and preserve primary plus cleanup errors | Private-only | Reservation and platform-publication records; prior `backend.protocol.scope-cleanup-precedence` |
| `artifact.explicit-cleanup-failure-retention` | Renderer invariant; private deterministic explicit temporary cleanup | After transaction success/failure | Attempt every path, aggregate failures deterministically, and retain only failed paths for Drop retry | Private-only | Drop `artifact.drop-best-effort-retry`; remover `artifact.missing-file-cleanup-fallback` |
| `artifact.drop-best-effort-retry` | Renderer invariant; private best-effort terminal cleanup fallback | Object drop after explicit cleanup failure/unwind | Retry retained paths without panic or error escape; no publication state is altered | Private-only | Explicit cleanup `artifact.explicit-cleanup-failure-retention` |
| `artifact.missing-file-cleanup-fallback` | Renderer invariant; private idempotent temporary-file removal | Explicit and Drop cleanup | Success/NotFound complete removal; all other failures remain observable to explicit cleanup | Private-only | Reservation `artifact.sibling-temp-reservation`; explicit cleanup record |
| `output.png.options.sizing-carrier` | Explicit policy; retained `PngOptions::sizing` | One PNG terminal call | Closed mode validated before conversion; no sticky storage or silent substitution | Retained `PngOptions::sizing` | Compression sibling `output.png.options.compression-carrier`; modes record |
| `output.png.options.compression-carrier` | Explicit policy; retained `PngOptions::compression` | One PNG terminal call | Closed preset maps exactly once; no adaptive substitution | Retained `PngOptions::compression` | Sizing sibling; mapping `backend.png.compression-mapping` |
| `output.png.options.default-preset` | Explicit policy; retained `PngOptions::default` | Plain PNG terminal | Exactly 300 DPI plus Balanced compression; construction yields a complete valid preset | Retained `PngOptions::default` | Terminal `output.png.default-terminal-options`; metadata record |
| `output.png.sizing.closed-modes` | Explicit policy; retained closed `PngSizing` | One PNG terminal call | Source or one finite-positive DPI/scale/positive dimension; deterministic mode-specific integer target plan with only documented rounding approximation; no fallback | Retained `PngSizing` | Carrier `output.png.options.sizing-carrier`; validation and target-sizing records |
| `output.png.compression.closed-presets` | Explicit policy; retained closed `PngCompression` | One PNG terminal call | Closed None/Fast/Balanced/High; default remains Balanced; exact encoder mapping | Retained `PngCompression` | Carrier and `backend.png.compression-mapping` |
| `backend.png.option-validation` | Explicit policy; retained PNG sizing validity responsibility | Before any PNG font/parser/allocation work | Source valid; DPI/Scale finite and strictly positive; Width/Height strictly positive; reject without clamp | Retained behavior of `PngSizing` | Modes `output.png.sizing.closed-modes`; planner `backend.png.preparse-target-prediction` |
| `backend.png.format-local-font-setup-timing` | Renderer invariant; resolved generation-font timing | Before text measurement and canonical scene creation | Resolve every used stack/glyph once during Plot/Canvas preflight; PNG conversion consumes the same resolved glyph scene and performs no late appearance lookup | Private-only | T2a1 font-resolution records; Null text parity; T2a3 outlined conversion |
| `backend.png.svg-parse-and-dimension-check` | Renderer invariant; private canonical-scene PNG parser boundary | After validated target plan, before allocation | Parse exact canonical bytes with closed options; parsed dimensions exactly equal stored CSS dimensions; reject without substitution | Private-only | T2a3 `scene.png-routing`; target plan and allocation records |
| `backend.png.preparse-target-prediction` | Renderer invariant; private one-time validated PNG target plan | Before font/parser/allocation work | Use fixed 96 CSS px/in for DPI, compute one integer target plan with the canonical mode formula, apply Scene limits once, and pass the same plan through parser agreement, allocation, transform, and metadata | Private-only | Actual sizing `backend.png.integer-target-sizing`; T2a3 Scene limit records |
| `backend.png.integer-target-sizing` | Renderer invariant; private canonical integer PNG target sizing | One validated target plan before allocation | Source is identity; DPI/Scale independently round each `f32` source-axis product to nearest; Width/Height pin the requested axis and ceil the `f32` aspect-derived axis; dimensions stay positive with no second formula, clamp, or divergent revalidation | Private-only | Prediction record; allocation `backend.png.target-pixmap-allocation` |
| `backend.png.target-pixmap-allocation` | Renderer invariant; private validated PNG target allocation | After exact target plan and parser agreement | Allocate exactly validated dimensions once; allocation failure is terminal with no lower-resolution fallback | Private-only | T2a3 RGBA8 paint serializer; integer target sizing |
| `backend.png.source-target-transform` | Renderer invariant; private canonical PNG raster transform | Final rasterization from canonical scene to allocated target | Finite positive source/target scales derived from the one plan; transform exactly fills target without crop/substitution | Private-only | Integer target sizing; allocation; T2a3 paint serializer |
| `backend.png.compression-mapping` | Renderer invariant; private closed PNG compression adapter | Final encoder setup | Each public preset maps exactly to one encoder preset; no adaptive fallback | Private-only | Public `output.png.compression.closed-presets` |
| `backend.png.dpi-metadata` | Renderer invariant; private exact PNG physical-resolution metadata | Final encoder setup for DPI mode only | Set equal axes with `Unit::Meter` and deterministic nearest-integer pixels-per-meter; production target-plan limits establish representability first, while the helper retains a defensive range rejection; non-DPI modes omit metadata | Private-only | Default preset; target prediction; encoder lifecycle |
| `backend.png.encoder-output-lifecycle` | Renderer invariant; private deterministic PNG encoder lifecycle | Final private PNG conversion boundary | Exact target dimensions and complete RGBA8 pixels; header, data, and finish must all succeed before bytes escape | Private-only | T2a3 `backend.png.rgba8-paint-serialization`; compression and metadata siblings; output timing |

### T2a4 Migration And Evidence

| Property | Disposition and owning phase | Visual evidence |
| --- | --- | --- |
| `lifecycle.plot.public-preflight-route` | `retain`, T3 | No output change; public preflight/no-file and terminal-error parity assertions |
| `lifecycle.plot.selected-kind-backend-route` | `retain`, T3 | No output change; every exact/erased family reaches its selected renderer once |
| `lifecycle.plot.closed-renderer-dispatch` | `retain`, T3 | No output change; closed family route and representative family snapshots |
| `lifecycle.plot.panel-post-render-order` | `retain`, T3 | No output change; layer, label, axis-text, and panel-text ordering assertions |
| `lifecycle.plot.post-panel-appearance-reconstruction` | `replace`, T3 | No intended built-in change; post-panel validation in Null and final passes shares one resolved appearance identity with planning and draw |
| `lifecycle.plot.configuration-before-canvas` | `retain`, T3 | No successful-output change; invalid configuration proves zero begin/end calls |
| `lifecycle.plot.one-canvas-route-and-finalization` | `connect`, T3 | No output change; one begin/end, empty-success, render error, end error, and double-error evidence |
| `lifecycle.plot.outer-decoration-appearance-reconstruction` | `replace`, T3 | No intended built-in change; pre-plan Null/final passes plus axis/grid/title consumers prove one resolved appearance |
| `lifecycle.plot.plan-before-draw` | `replace`, T3 | No intended output change; one plan reused by final render and no duplicate planning assertions |
| `lifecycle.plot.grid-before-panel-order` | `retain`, T3 | No output change; grid-before-marks snapshots and SelfManaged absence evidence |
| `lifecycle.plot.planned-chart-handoff` | `replace`, T3 | No intended output change; resolved appearance identity survives range/data plan substitution |
| `lifecycle.plot.post-panel-decoration-order` | `retain`, T3 | No output change; significance/axis/legend/title z-order snapshots |
| `lifecycle.plot.panel-null-preflight-route` | `connect`, T3 | No output change; facet/coordination preflight and final-panel error parity |
| `backend.deferred.lazy-canvas-state-machine` | `replace`, T3 | No intended output change; stored-dimension authority, lazy start, empty render, explicit-ended state, post-end rejection, and exactly-once underlying-end tests |
| `backend.null.canvas-lifecycle-parity` | `connect`, T3 | No successful-output change; canonical/invalid dimensions, fresh begin/end balance, unclosed-scope end rejection, and draw-outside-canvas evidence |
| `backend.null.clip-validation-parity` | `replace`, T3 | No successful-output change; invalid region, nested LIFO, unmatched close, and unclosed-at-end parity tests |
| `backend.null.scatter-validation-parity` | `connect`, T6 | No ordinary output change; complete point RGBA/outline validation parity after carrier migration |
| `backend.null.line-degenerate-parity` | `connect`, T3 | No output change; short-line bypass and nondegenerate invalid-style parity |
| `backend.null.polygon-degenerate-parity` | `connect`, T6 | No ordinary output change; short-polygon bypass and complete fill/stroke parity |
| `backend.null.gradient-validation-parity` | `retain`, T3; complete stop paint connects in T6 | No output change through T3; every gradient geometry/order branch matches final failure, and T6 adds complete RGBA stop-paint parity |
| `backend.null.text-validation-parity` | `replace`, T5a; complete text paint connects in T6 | Intentional earlier font/glyph/style failures in T5a with preflight/final content, placement, rotation, and glyph parity; T6 adds complete text-paint parity |
| `backend.null.rounded-rectangle-validation-parity` | `replace`, T3; complete fill/stroke connects in T6 | No ordinary output change in T3; zero/invalid extent, oversized-radius, far-edge, and style-order parity assertions, with complete RGBA fill/stroke parity in T6 |
| `backend.null.bracket-validation-parity` | `retain`, T3; resolved typography and complete label paint connect in T5a; complete path paint connects in T5c | No T3 output change; invalid anchors/path geometry/style and significance snapshots, T5a label glyph/metric/paint parity, and T5c complete path-paint parity |
| `output.plot.preflight-before-scene-render` | `replace`, T3 | No intended visual change; one resolution/plan plus one final render call-count evidence |
| `output.plot.canonical-scene-render` | `connect`, T3 | No output change; exact dimensions, completed bytes, and one-scene construction assertions |
| `output.plot.outlined-svg-terminal` | `retain`, T3 | No output change; immediate outlined publication and returned Output reuse |
| `output.plot.preserve-text-svg-terminal` | `retain`, T3 | No output change; exact preserve-text publication and returned Output reuse |
| `output.plot.default-png-terminal` | `retain`, T3 | No output change; 300-DPI/Balanced output and returned Output reuse |
| `output.plot.preview-png-terminal` | `retain`, T3 | No output change; source dimensions, no density metadata, and returned Output reuse |
| `output.plot.explicit-png-terminal` | `retain`, T3 | No output change; exact call-local sizing/compression and no sticky state |
| `output.reusable-scene-handle` | `connect`, T3 | No output change; one render across chained SVG/PNG calls and exact scene-byte reuse |
| `output.outlined-svg.conversion-before-staging` | `retain`, T3 | No output change; conversion failure leaves no staging artifact or destination mutation |
| `output.outlined-svg.publication-terminal` | `retain`, T3 | No output change; independent chained outlined publication and destination preservation |
| `output.preserve-text-svg.publication-terminal` | `retain`, T3 | No output change; exact canonical-byte publication and no font lookup |
| `output.png.default-terminal-options` | `retain`, T3 | No output change; repeated/chained plain PNG always selects the complete default preset |
| `output.png.preview-terminal-options` | `retain`, T3 | No output change; repeated/chained preview remains Source/Balanced and metadata-free |
| `output.png.conversion-before-staging` | `retain`, T3 | No output change; invalid sizing/font/parse/encode failure creates no staging artifact |
| `output.png.explicit-publication-terminal` | `retain`, T3 | No output change; exact options and independent chained publication evidence |
| `artifact.sibling-temp-reservation` | `retain`, T3 | No output change; collision retry, sibling placement, no-file-name, and other-I/O tests |
| `artifact.posix-rename-publication` | `retain`, T3 | No output change; successful replacement and old-destination preservation on rename failure |
| `artifact.windows-publication` | `replace`, T3 | Intentional contract correction; current-branch evidence covers absent/non-file targets, one-shot backup collision, install failure with successful rollback, failed rollback with absent destination and untracked backup, and ignored backup cleanup; replacement-platform evidence proves no absence window and old-file preservation on every failed atomic replace |
| `artifact.staged-write-transaction` | `retain`, T3 | No output change; write/publish/cleanup primary-plus-cleanup failure matrix |
| `artifact.explicit-cleanup-failure-retention` | `retain`, T3 | No output change; all-path attempt, aggregate failure, and retained-path retry evidence |
| `artifact.drop-best-effort-retry` | `retain`, T3 | No output change; unwind/drop retry without panic and permitted residual-file evidence |
| `artifact.missing-file-cleanup-fallback` | `retain`, T3 | No output change; present, already-missing, and non-NotFound removal branches |
| `output.png.options.sizing-carrier` | `retain`, T3 | No output change; compile/runtime evidence for exact call-local mode transport |
| `output.png.options.compression-carrier` | `retain`, T3 | No output change; exact call-local preset transport and no sticky state |
| `output.png.options.default-preset` | `retain`, T3 | No output change; exact 300-DPI/Balanced dimensions, metadata, and compression selection |
| `output.png.sizing.closed-modes` | `retain`, T3 | No output change; Source/DPI/Scale/Width/Height dimensions and documented integer aspect-rounding approximation tests |
| `output.png.compression.closed-presets` | `retain`, T3 | No output change; all four encoder mappings and both Balanced defaults |
| `backend.png.option-validation` | `retain`, T3 | No successful-output change; NaN/infinite/nonpositive DPI/scale and zero width/height rejection |
| `backend.png.format-local-font-setup-timing` | `replace`, T5a | Intentional font-timing correction; missing/non-Latin glyph parity across layout, outlined SVG, and PNG |
| `backend.png.svg-parse-and-dimension-check` | `retain`, T3 | No output change; malformed scene and exact dimension-mismatch rejection |
| `backend.png.preparse-target-prediction` | `replace`, T3 | No intended output change; one target plan, one limit call, `f32` DPI-division boundary, and no floating/integer formula divergence evidence |
| `backend.png.integer-target-sizing` | `connect`, T3 | No output change for admitted cases; Source identity, independent nearest rounding for DPI/Scale, pinned-axis plus ceil for Width/Height, approximate aspect, and no-zero target tests |
| `backend.png.target-pixmap-allocation` | `retain`, T3 | No successful-output change; validated dimensions and terminal allocation failure without fallback |
| `backend.png.source-target-transform` | `retain`, T3 | No output change; exact edge/corner scaling at every sizing mode |
| `backend.png.compression-mapping` | `retain`, T3 | No visual change; exact encoder discriminant and deterministic-byte evidence |
| `backend.png.dpi-metadata` | `retain`, T3 | No visual change; DPI-only equal-axis `Unit::Meter` pHYs values and rounding, direct helper test of the defensive `u32::MAX` branch, integration proof that production target sizing rejects first, and non-DPI metadata omission tests |
| `backend.png.encoder-output-lifecycle` | `retain`, T3 | No output change; dimensions/header/data/finish failures and complete deterministic bytes |

### Deferred T2a4 Matches

| Source match | Deferred responsibility |
| --- | --- |
| `src/orchestrator.rs:113-135` | T2g semantic x/y domain assignment. |
| `src/orchestrator.rs:146-176` | T2g categorical raw/display identity resolution and tick-key storage. |
| `src/orchestrator.rs:177-179` | T2d legend-entry display resolution. |
| `src/orchestrator.rs:191-235`, excluding the admitted `palette()` reconstruction at line 189 | T2c `0.9`, `0.4`, `max(1.0)`, text widths, and axis-text measurement policy. Palette values remain prior-owned by T2a1. |
| `src/orchestrator/lifecycle.rs:27-32,81-86` | T2b significance family-support validation. |
| `src/orchestrator/lifecycle.rs:34-37` | T2c/T2f exact Matrix-versus-ordinary panel geometry contexts; T2a4 owns only subsequent lifecycle timing. |
| `src/orchestrator/lifecycle.rs:80` | T2a4 owns validation-before-context timing in `lifecycle.plot.panel-null-preflight-route`; validator semantics are split between T2b public policy/family support and T2g categorical ordering/domain ownership. |
| `src/orchestrator/lifecycle.rs:95-115` | T2g coordinated hue identity collection. |
| `src/canvas.rs:390-430` | T2d Canvas composition lifecycle and composed-scene construction; these are T2a4 consumer counterparts, not source records. |
| `src/canvas.rs:432-464` | T2h residual independent Canvas terminal selections. T2a4's written source scope does not admit `canvas.rs`. |
| `src/spec.rs:25` | Crate-private transport of T2a4-owned `SVG_CSS_DPI`; not a T2b public source. |
| `src/spec.rs:26` and `src/lib.rs:75` | T2b public module/crate-root PNG type re-exports; the underlying `src/spec/output.rs:3-52` declarations remain exact T2a4 policy sources. |

### Prior-Owned T2a4 Matches

| Source match | Prior owner and reason |
| --- | --- |
| Palette fields consumed after `chart.palette()`/`self.palette()` | T2a1 owns every value; T2a4 records only independent reconstruction timing and lifecycle divergence. |
| `validate_*` calls in `src/orchestrator/backend.rs` | T2a2 owns validator declarations/branches; T2a4 owns each concrete Null terminal selection, ordering, and line/polygon no-op. |
| Null inherited ordinary-layer and Grid-role defaults | T2a2 `backend.protocol.ordinary-layer-fallbacks` and `backend.protocol.grid-role-delegation`. |
| `finish_scope` called by Deferred and temporary-artifact cleanup | T2a2 `backend.protocol.scope-cleanup-precedence`; T2a4 owns these concrete lifecycle call sites. |
| `RenderedScene` dimensions/bytes and `svg_bytes`, `svg_with_text_as_paths`, `png_bytes` route implementations | T2a3 Scene records; T2a4 owns Output's one-scene handle, terminal selections, and conversion-before-staging timing. |
| `configure_system_fonts` discovery/selection behavior | T2a1; T2a4 owns only PNG-local invocation timing. |
| Transparent-pixmap and compositing meanings at `src/render/svg_backend.rs:147-151`; `ColorType::Rgba`, `BitDepth::Eight`, and `take_demultiplied` | T2a3 `backend.png.rgba8-paint-serialization`; T2a4 owns target dimensions, allocation, transform, and encoder lifecycle. |
| Scene target-limit declarations/branches called at `src/render/svg_backend.rs:196-199,229` | T2a3 `scene.export-dimension-limit`, `scene.target-pixel-work-limit`, and `scene.png-target-positive-finite-limit`; T2a4 owns sizing arithmetic and call timing. |

### Rejected T2a4 Matches

| Source match | Disposition and reason |
| --- | --- |
| `src/orchestrator.rs:1-21` | Module/import/re-export plumbing without an independent lifecycle source. |
| `src/orchestrator.rs:22-25,41-53,82-83,239-241` | Test-only counters, backend entry points, and test module; evidence only. |
| `src/orchestrator/lifecycle.rs:47,49-52,87-91,117-129` data/source-row meanings | Data/context transport and integrity, not visual or output policy. Their surrounding call order is admitted where applicable; the separate category-order validation timing at line 80 is admitted rather than rejected. |
| `debug_assert_eq!` at `src/orchestrator/lifecycle.rs:71` | Debug-only consistency evidence, not production behavior. |
| Documentation comments, imports, visibility, and ordinary non-`Default` derives in A4-M1-A4-M8 | Language/documentation plumbing. Executable defaults and `#[default]` compression selection remain admitted. |
| Error-message wording, interpolated diagnostic values, and artifact scope/action strings | Non-visual diagnostics; every controlling failure branch remains admitted. |
| Paths, scene bytes, labels, points, styles, bounds, and caller option payloads merely transported through admitted functions | Data or prior/later-owned values, not duplicate T2a4 property sources. |

### T2a4 Reconciliation

- The completed T2a4 count is exactly **60**: 13 Plot lifecycle and sequencing
  records, 10 Deferred/null backend records, 15 output-routing and
  publication-timing records, 7 artifact records, and 15 PNG
  option/sizing/encoding records. Every descriptive key appears once in each
  keyed table in the same order.
- The corrected and completed T2a aggregate is exactly **222**: 65 T2a1, 52
  T2a2, 45 T2a3, and 60 T2a4 records.
- Independent review findings corrected the T2a4/T2b PNG declaration boundary,
  both reconstruction contexts, explicit Deferred end state, Null lifecycle and
  migration phasing, Windows backup failure paths, exact PNG rounding and
  metadata ordering, and panel category-order validation timing. These
  corrections changed no keyed-record count or order.
- The superseded 40-record estimate changed without new production scope.
  Source adjudication separated independent Plot/Output terminal selections,
  preflight/plan/render timing, Null terminal policies, conversion-before-stage
  timing, artifact failure/cleanup paths, and the five exact public PNG policy
  declarations. It retained one Deferred state-machine record because those
  contexts share one semantic role, target owner, and T3 phase.
- PNG sizing/options are expressly T2a4-owned even though their declarations are
  public. T2b expressly excludes `src/spec/output.rs:3-52`; only the public PNG
  type re-exports at `src/spec.rs:26` and `src/lib.rs:75` remain T2b-owned, while
  `src/spec.rs:25` is crate-private transport. This keeps the manifests disjoint.
- Windows existing-file publication is currently non-atomic because it renames
  the destination to one unreserved backup candidate before installing staged
  bytes, creating an absence window and depending on fallible rollback and
  hidden cleanup. The target contract is one true atomic same-directory
  replacement that preserves the old destination on every failure;
  `artifact.windows-publication` is therefore `replace`, T3.
- No Canvas source record is admitted. `canvas.rs:390-430` remains T2d-owned and
  `canvas.rs:432-464` remains a T2h residual under the existing written scopes.
- A post-correction independent rereview found no remaining semantic, schema,
  ownership, phase, count, or order issue. Tracked-diff checks and `just verify`
  passed on 2026-08-26. No production code, tests, snapshots, or other
  specifications changed in T2a4.

## T2b: Public Controls And Policies

**Status**: COMPLETE

This section inventories public controls, their exact builder paths, public
family matrices, fixed caller-geometry representation rules, and the T2b-owned
erased validation branches. The target API remains non-current until T6.

### T2b Source Manifest

| ID | Production source | Owned context | Audit disposition |
| --- | --- | --- | --- |
| B-M1 | `src/spec.rs:28-549,592-618` | Bar spacing, histogram controls, panel text, raw overlay, split heatmap default/local-scale controls, numeric labels, null-row annotations, dumbbell endpoints, waterfall labels, and Plot/Canvas legend policies; exact significance runtime family matrix | 32 records. Three storage-only presence controls are attributed to B-M10. `NumericLabelFormat::format`, legend defaults, unified-series routing, and private render-result transport are deferred below. |
| B-M2 | `src/spec/bar.rs:6-366` | Public bar source/error-bar policies, independent SD/SE scale controls, independent percentile/CI coverage controls, and shared validators/defaults | 12 records. Numeric payload transport and tests are rejected. |
| B-M3 | `src/spec/categorical.rs:5-88` | Closed category-order vocabulary and exact categorical color-map construction | 5 records together with exact Plot storage paths. Used-key completeness and mapping expressions are T2g. |
| B-M4 | `src/spec/estimate.rs:32-303` | Coefficient interval presence, split significance defaults/exact style fields, regression/Q-Q/path policies, point/line vocabularies, interaction exact styles, estimate status, and forest scale | 17 records. `Bounds` data and SVG conversion literals are excluded; generated regression legend text is deferred below. |
| B-M5 | `src/spec/facet.rs:5-132` | Facet layout and numeric sharing policies | 2 records. Facet planning and rendering are T2d. |
| B-M6 | `src/layers.rs:8-321` | Guide controls, PointLayer presence/content, optional exact-color precedence, size/alpha defaults and exact overrides, BarPointLayer presence/exact styles, and shared radius validation | 16 records. T2b owns the optional exact-color field/default/setter; private exact-over-selected-accent conversion is T2e, and stable accent-slot selection is T2g. |
| B-M7 | `src/labels.rs:7-200` | Humanized/raw selection, singular and bulk alias mutation, alias precedence, display-collision failure, and fixed humanization | 6 records. |
| B-M8 | `src/geometry.rs:7-405` | Five visible dimension identities, one shared nonempty validator, and fixed raincloud/histogram representation, topology, support, positivity, and resource rules | 39 records. Public coordinate payloads are data; fixed representation branches are records. |
| B-M9 | `src/kind.rs:86-184` | Twenty-three independent public capability matrices after splitting Attached and Pairwise significance and rejecting SourceRows | 23 records. Marker sealing/erasure is API mechanics. |
| B-M10 | `src/builder.rs:437-598,606-615,716-733`; `src/builder/configuration.rs:14-554`; `src/builder/accessors.rs:419-987`; `src/builder/facets.rs:5-21` | Plot storage/default/setter/accessor paths not owned by the declaration records: panel-text/null-row/waterfall presence, hue/group/series, color-guide title, highlight, estimate labels, complete significance style, seven content/tick controls, ranges, sign/zero/marker/outline/significance/size/domain/band/reference/connector controls, and histogram geometry precedence | 31 records. Theme, point-outline appearance default, data transport, layout, legend suppression, and T2g mapping contexts are prior-owned, rejected, or deferred. |
| B-M11 | `src/canvas.rs:44-388,466-501,689-704,1079-1094` | Canvas category-label policy, split TextPanel content/validators, grid/cell policies, legend gap, split header policies, range groups, size/content/labels, and independent compatibility branches | 28 records, including complete-row occupancy for shared category labels. Composition implementation and category-domain equality are T2c/T2d/T2g; terminal methods remain T2h residual. |
| B-M12 | `src/orchestrator/validation.rs:17-78,222-277,285-631,634-668` limited to the exact T2b branches below | Independent family, presence, compatibility, precedence, and failure branches not grouped with one exact declaration/control owner | 42 records, including the four Matrix configuration invariants. Data-dependent order/completeness/color-domain expressions are T2g; axis/layout and numeric data validity are excluded. |
| B-M13 | `src/orchestrator/layers.rs:13-69,227-259` | Split erased numeric-x/numeric-y matrices, one atomic supplemental/facet incompatibility, exact tick numeric-axis gates, and three guide-axis gates | 8 independent records. The two exact PointLayer family gates are grouped with their B-M6 presence controls. Tick projection/validation is T2c; guide projection/style conversion is T2e. |
| B-M14 | `src/spec.rs:14-26`; `src/lib.rs:49-77` | Public module/crate-root exposure | Evidence only. Re-exports are not property records; PNG declarations remain T2a4-owned. |

The manifest is exact for the T2b production sources. Source contexts assigned
to T2a1, T2a2, T2a4, T2c-T2g, or T2h appear only in the disposition tables
below and are not duplicate T2b records.

### T2b Consumer And Evidence References

| Code | Exact production consumers or evidence |
| --- | --- |
| B-C-PLOT | `builder.rs` storage/defaults, `builder/configuration.rs` setters, `builder/accessors.rs` reads, `orchestrator/validation.rs`, `orchestrator/layers.rs`, Plot preflight/rendering, facets, decorations, and plot renderers. |
| B-C-CANVAS | `canvas.rs` structure/preflight/planning and Canvas composition tests; implementation values remain T2c/T2d/T2g. |
| B-C-STYLE | `layers.rs`, significance, plot renderers, generated legends, and private backend carriers. Carrier declarations remain T2a2; final conversions remain T2e/T2f. |
| B-C-SCALE | T2g category/domain/mapping paths, plot renderers, facet coordination, legends, heatmap/beeswarm guides, and Canvas shared-domain planning. |
| B-C-GEOMETRY | `HistogramInput`/`RaincloudInput`, Plot validation, histogram/raincloud renderers, axis-domain planning, clipping, and final mark geometry. |
| B-C-LABELS | Axis/title inference, ticks, legends, annotations, facets, Canvas label overrides, and display-domain collision checks. |
| B-E-COMPILE | Existing compile-pass/compile-fail doctests and public-contract tests for exact family availability; missing matrices are required T6 evidence, not unresolved ownership. |
| B-E-VISUAL | Existing rendering, annotation, line/layer, heatmap, distribution, facet, Canvas, legend, and visual-regression suites. Focused evidence named per record remains required where absent. |
| B-L-N | Paint/content/policy only; no current measurement effect. |
| B-L-M | Measurement or wrapping. |
| B-L-R | Reservation/range expansion. |
| B-L-C | Collision planning. |
| B-L-G | Geometry, projection, or final occupied extent. |

### T2b Current State

| Property | Current source and value | Current consumers | Current behavior | Current layout path | Semantic role | Audit evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `bars.spacing.current-bar-spacing` | B-M1 `BarSpacing::{Comfort,Compact}`; default Comfort; Plot field/setters at `builder.rs:461,547`, `configuration.rs:420-450` | B-C-PLOT | Closed replacement; Bar/HBar only | B-L-G | Within-category series spacing | B-M1; declaration/default/setters grouped |
| `histogram.binning.current-selection` | B-M1 `HistogramBins::{Auto,Count,Width}` at `spec.rs:38-53`; Plot default/setter/read | B-C-PLOT | Closed statistical geometry policy; payload validity later | B-L-G | Histogram bin geometry selection | B-M1 |
| `histogram.normalization.current-stat` | B-M1 `HistogramStat::{Count,Probability,Density}` at `spec.rs:55-65`; default Count | B-C-PLOT | Closed y-value preparation policy | B-L-G | Histogram vertical normalization | B-M1 |
| `histogram.composite.kde-source` | B-M1 `HistogramKde::{Internal,Precomputed}` at `spec.rs:67-84` | B-C-PLOT | Selects generated or caller-computed curve | B-L-G | KDE geometry source | B-M1 |
| `histogram.composite.presence` | B-M1 `HistogramComposite::{HistogramOnly,KdeAndRug}` at `spec.rs:86-100`; default inert | B-C-PLOT | Closed optional composite policy | B-L-G | KDE/rug presence | B-M1; runtime family gate grouped |
| `annotations.panel-text.presence-control` | B-M10 Plot vector default empty; append at `configuration.rs:301-309` | B-C-PLOT | Append order retained; zero or more annotations | B-L-M,B-L-C,B-L-G | Panel-text annotation presence | B-M10 |
| `annotations.panel-text.corner` | B-M1 `PanelCorner` plus `PanelTextAnnotation.corner` at `spec.rs:102-144` | B-C-PLOT | Four closed anchors; no fallback | B-L-C,B-L-G | Exact panel-corner placement | B-M1 |
| `annotations.panel-text.content` | B-M1 text field/validation/accessor at `spec.rs:115-151` | B-C-PLOT | Nonempty trimmed single-line control-free text | B-L-M,B-L-C,B-L-G | Exact panel annotation content | B-M1 |
| `marks.raw-overlay.presence-layout` | B-M1 `RawOverlay::{Hidden,Points,Jittered}`; default Hidden | B-C-PLOT,B-C-STYLE | Closed presence/displacement policy | B-L-G | Original-observation overlay | B-M1; exact runtime gate grouped |
| `color-scales.continuous.heatmap-default` | B-M1 `HeatmapScale::default()` selects Sequential | B-C-SCALE | Every Heatmap stores this library default before any caller replacement | B-L-N | Default heatmap continuous mapping | B-M1; split from caller replacement |
| `color-scales.continuous.heatmap-policy` | B-M1 `HeatmapScale::{Sequential,Diverging{center}}`; `Plot::scale` replaces the stored value | B-C-SCALE | Current closed caller scale selector; center unchecked at construction | B-L-N | Plot-local heatmap continuous mapping policy | B-M1; split from default |
| `heatmap.cells.layout-policy` | B-M1 `HeatmapLayout` Cartesian/lower/upper with diagonal; default Cartesian | B-C-PLOT | Exact Heatmap-only replacement | B-L-G | Active-cell topology | B-M1; exact runtime gate grouped |
| `heatmap.labels.fit-policy` | B-M1 `HeatmapLabelFit::{Elide,Wrap}`; default Elide | B-C-PLOT | Exact Heatmap-only replacement | B-L-M,B-L-R,B-L-G | Dense category-label fit | B-M1; exact runtime gate grouped |
| `annotations.numeric-label.precision` | B-M1 `NumericLabelFormat.precision`; default 2; accepts 0..=12 | B-C-PLOT | Setter rejects above 12 | B-L-M,B-L-G | Fixed decimal precision | B-M1 |
| `annotations.numeric-label.suffix` | B-M1 optional suffix; default None; nonempty/control-free setter | B-C-PLOT | Literal append; formatting implementation deferred | B-L-M,B-L-G | Numeric annotation suffix | B-M1 |
| `annotations.heatmap.cell-value-presence` | B-M1 `HeatmapAnnotations::{Hidden,Values}`; default Hidden | B-C-PLOT | Closed cell-label presence/format policy | B-L-M,B-L-G | Heatmap cell-value content | B-M1 |
| `annotations.heatmap.null-row-presence` | B-M10 empty vector plus append at `configuration.rs:76-84` | B-C-PLOT | Ordered zero-or-more complete annotations | B-L-M,B-L-G | Fully-null row annotation presence | B-M10 |
| `annotations.heatmap.null-row-target` | B-M1 `row_label` at `spec.rs:301-325` | B-C-PLOT | Exact normalized y-category target | B-L-G | Null-row target identity | B-M1 |
| `annotations.heatmap.null-row-text` | B-M1 `text` at `spec.rs:301-331` | B-C-PLOT | Verbatim wrapped content | B-L-M,B-L-G | Null-row annotation text | B-M1 |
| `annotations.heatmap.null-row-lexical-validation` | B-M1 shared branch `spec.rs:334-341` | B-C-PLOT | Both target and text must be nonempty, trimmed, control-free | B-L-N | Null-row lexical contract | B-M1; helper occurs once |
| `marks.dumbbell.left-endpoint-identity` | B-M1 left label at `spec.rs:345-365,389-393` | B-C-PLOT,B-C-STYLE | Caller-owned exact identity | B-L-M,B-L-G | Left endpoint label | B-M1 |
| `marks.dumbbell.right-endpoint-identity` | B-M1 right label at `spec.rs:345-365,401-405` | B-C-PLOT,B-C-STYLE | Caller-owned exact identity | B-L-M,B-L-G | Right endpoint label | B-M1 |
| `marks.dumbbell.endpoint-identities-nonempty` | B-M1 branch `spec.rs:366-370` | B-C-PLOT | Rejects either empty label | B-L-N | Endpoint identity presence | B-M1; independent failure |
| `marks.dumbbell.endpoint-identities-lexical-validity` | B-M1 branch `spec.rs:371-379` | B-C-PLOT | Rejects surrounding whitespace/control characters | B-L-N | Endpoint lexical validity | B-M1; independent failure |
| `marks.dumbbell.endpoint-identities-distinct` | B-M1 branch `spec.rs:381-385` | B-C-PLOT | Rejects equal endpoint identities | B-L-N | Endpoint identity compatibility | B-M1; independent failure |
| `marks.dumbbell.left-endpoint-color` | B-M1 left RGB at `spec.rs:345-365,395-399` | B-C-STYLE | Exact opaque RGB; no local fallback | B-L-N | Left endpoint paint | B-M1 |
| `marks.dumbbell.right-endpoint-color` | B-M1 right RGB at `spec.rs:345-365,407-411` | B-C-STYLE | Exact opaque RGB; no local fallback | B-L-N | Right endpoint paint | B-M1 |
| `annotations.waterfall.total-label-presence` | B-M10 optional field default None; setter/read at `configuration.rs:547-553`, `accessors.rs:667-671` | B-C-PLOT | Complete pair replaces absence | B-L-M,B-L-G | Waterfall total-label presence | B-M10 |
| `annotations.waterfall.base-label` | B-M1 base field/accessor `spec.rs:414-450` | B-C-PLOT | Caller-owned verbatim content | B-L-M,B-L-G | Base annotation label | B-M1 |
| `annotations.waterfall.terminal-label` | B-M1 terminal field/accessor `spec.rs:414-456` | B-C-PLOT | Caller-owned verbatim content | B-L-M,B-L-G | Terminal annotation label | B-M1 |
| `annotations.waterfall.labels-nonempty` | B-M1 branch `spec.rs:428-432` | B-C-PLOT | Rejects either empty label | B-L-N | Waterfall label presence | B-M1 |
| `annotations.waterfall.labels-lexical-validity` | B-M1 branch `spec.rs:433-442` | B-C-PLOT | Rejects surrounding whitespace/control characters | B-L-N | Waterfall lexical validity | B-M1 |
| `canvas.legend.composition-policy` | B-M1 `CanvasLegendPolicy::{PerPlot,Shared,Hidden}`; default PerPlot; Canvas setter | B-C-CANVAS | Closed Canvas legend behavior | B-L-M,B-L-R,B-L-C,B-L-G | Canvas legend sharing/presence | B-M1/B-M11 |
| `legend.plot.visibility-placement-policy` | B-M1 `LegendPolicy` plus 25 `LegendPosition` choices; defaults Automatic/Auto | B-C-PLOT | Hidden, automatic, or exact placement; hidden/automatic position fallback Auto | B-L-M,B-L-R,B-L-C,B-L-G | Plot legend presence/placement | B-M1; exact compile gate separate |
| `annotations.significance.erased-family-support` | B-M1 `PlotKind::supports_significance_annotations`; runtime checks in open/facet routes | B-C-PLOT | Eight-family erased union; placement matrix checked separately at compile time | B-L-N | Erased significance family support | B-M1; two runtime call sites grouped |
| `bars.input.estimate-source-and-bounds` | B-M2 `BarInput` samples/estimates/estimates_with_error_bars | B-C-PLOT | Selects generated means/errors or caller estimates/bounds | B-L-G | Bar estimate/error geometry source | B-M2 |
| `bars.error.generated-selection` | B-M2 `ErrorBarKind` Hidden/SD/percentile/SE/CI; default CI | B-C-PLOT | Closed generated-error taxonomy; BarSamples replacement | B-L-G | Generated error-bar presence/type | B-M2 |
| `bars.error.standard-deviation-scale` | B-M2 `StandardDeviation.scale`; default 1.0 | B-C-PLOT | Setter uses shared positive-finite validator | B-L-G | SD interval multiplier | B-M2 |
| `bars.error.standard-error-scale` | B-M2 `StandardError.scale`; default 1.0 | B-C-PLOT | Setter uses shared positive-finite validator | B-L-G | SE interval multiplier | B-M2 |
| `bars.error.spread-scale-validation` | B-M2 branch `spec/bar.rs:359-366` | B-C-PLOT | Rejects nonfinite/nonpositive spread multipliers | B-L-N | Shared spread-scale domain | B-M2; helper occurs once |
| `bars.error.percentile-interval-coverage` | B-M2 `PercentileInterval.level` | B-C-PLOT | Independent coverage control; shared default/domain | B-L-G | Original-observation percentile coverage | B-M2 |
| `bars.error.confidence-interval-coverage` | B-M2 `ConfidenceInterval.level` | B-C-PLOT | Independent coverage control; shared default/domain | B-L-G | Mean confidence coverage | B-M2 |
| `bars.error.shared-coverage-default-domain` | B-M2 `DEFAULT_LEVEL=0.95` and branch `350-357` | B-C-PLOT | Common default; finite strict (0,1) | B-L-N | Generated interval coverage default/domain | B-M2; declaration/validator each occur once |
| `bars.error.confidence-algorithm` | B-M2 Bootstrap or StudentT selector; bootstrap default | B-C-PLOT | Consuming replacement; no fallback | B-L-G | CI algorithm selection | B-M2 |
| `bars.error.bootstrap-resamples` | B-M2 default 1,000; range 1..=1,000,000 | B-C-PLOT | Setter rejects outside closed safety range | B-L-N | Bootstrap work/count policy | B-M2 |
| `bars.error.bootstrap-seed` | B-M2 default 0; arbitrary u64 replacement | B-C-PLOT | Deterministic root seed | B-L-G | Bootstrap deterministic geometry seed | B-M2 |
| `bars.error.bootstrap-units` | B-M2 optional CategoryValues; default None | B-C-PLOT | Optional multilevel resampling identity | B-L-G | Bootstrap sampling-unit policy | B-M2 |
| `categories.order.closed-selection` | B-M3 Observed/Explicit/Value and Ascending/Descending | B-C-PLOT,B-C-SCALE | Closed ordering vocabulary; default Observed | B-L-G | Categorical display-order selection | B-M3 |
| `categories.order.primary-control` | B-M10 primary field/default/setter/read; CategoryOrder capability | B-C-PLOT,B-C-SCALE | Exact replacement; family validation separate | B-L-G | Primary categorical order | B-M3/B-M10 |
| `categories.order.heatmap-x-control` | B-M10 heatmap x field/default/setter/read | B-C-PLOT,B-C-SCALE | Exact replacement | B-L-G | Heatmap horizontal order | B-M3/B-M10 |
| `categories.order.heatmap-y-control` | B-M10 heatmap y field/default/setter/read | B-C-PLOT,B-C-SCALE | Exact replacement | B-L-G | Heatmap vertical order | B-M3/B-M10 |
| `color-scales.categorical.exact-map-control` | B-M3 nonempty unique exact label/RGB entries; Plot optional storage/setter | B-C-SCALE | Construction validates keys; used-key completeness later | B-L-N | Exact plot-local categorical mapping | B-M3/B-M10 |
| `intervals.coefficient.presence` | B-M4 `CoefficientInput` estimates or estimates_with_intervals | B-C-PLOT | Optional complete bounds selected at construction | B-L-G | Coefficient interval presence | B-M4 |
| `typography.significance.font-size-default` | B-M4 `SignificanceMarkStyle::default().font_size = 16` | B-C-STYLE | Stored as part of every Plot even when the caller supplies no exact style | B-L-M,B-L-R,B-L-G | Default significance text size | B-M4; split from exact override |
| `annotations.significance.font-size` | B-M4 constructor/setter positive u32 replacement | B-C-STYLE | Caller exact field; constructor/setter validate the complete style; no current absence sentinel | B-L-M,B-L-R,B-L-G | Exact significance mark text size | B-M4; split from default |
| `typography.significance.weight-default` | B-M4 `SignificanceMarkStyle::default().weight = Normal` | B-C-STYLE | Stored as part of every Plot; current text-width estimation ignores weight | B-L-N | Default significance text weight | B-M4; split from exact override |
| `annotations.significance.weight` | B-M4 caller-selected `TextWeight::{Normal,Semibold,Bold}` | B-C-STYLE | Exact closed emphasis; current measurement still ignores the selected weight | B-L-N | Exact significance mark weight | B-M4; split from default |
| `annotations.significance.gap-default` | B-M4 `SignificanceMarkStyle::default().gap = 8.0` px | B-C-STYLE | Stored as part of every Plot even when the caller supplies no exact style | B-L-R,B-L-G | Default significance mark clearance | B-M4; split from exact override |
| `annotations.significance.gap` | B-M4 constructor/setter finite nonnegative f64 replacement | B-C-STYLE | Caller exact mark-to-data/label clearance; no current absence sentinel | B-L-R,B-L-G | Exact significance mark clearance | B-M4; split from default |
| `regression.band.presence-claim` | B-M4 PointwiseMean95/Hidden; default band; `RegressionBand::legend_label` at `estimate.rs:171-176` | B-C-PLOT,B-C-STYLE | Exact Regression-only replacement; policy also selects fixed generated legend claim text | B-L-G | Generated regression-band presence/claim | B-M4; legend construction deferred below |
| `qq.reference-line.policy` | B-M4 Quartiles/Identity/Hidden; default Quartiles | B-C-PLOT,B-C-STYLE | Exact Qq-only replacement | B-L-G | Q-Q reference line | B-M4 |
| `line-path.color-legend.policy` | B-M4 Neutral/Hue; default Neutral | B-C-PLOT,B-C-SCALE | Hue mode requires hue; family gate exact | B-L-N,B-L-G | Complete-path color/legend policy | B-M4 |
| `marks.point.closed-shape` | B-M4 Circle/TriangleUp; default Circle | B-C-STYLE | Closed public exact-style vocabulary | B-L-G | Point glyph shape | B-M4 |
| `marks.line.closed-pattern` | B-M4 Solid/Dashed; default Solid | B-C-STYLE | Closed public vocabulary; dash conversion deferred | B-L-N | Connector/guide pattern | B-M4 |
| `interaction.styles.group-selector` | B-M4 exact group String plus Plot style collection | B-C-PLOT,B-C-STYLE | One style selected by exact group identity | B-L-N | Interaction exact-style selector | B-M4/B-M10 |
| `interaction.styles.point-shape` | B-M4 InteractionStyle shape; default Circle | B-C-STYLE | Consuming replacement | B-L-G | Interaction point glyph | B-M4 |
| `interaction.styles.line-pattern` | B-M4 InteractionStyle pattern; default Solid | B-C-STYLE | Consuming replacement | B-L-N | Interaction connector pattern | B-M4 |
| `estimates.status.presence` | B-M4 Estimated/NotEstimable; Plot optional aligned storage | B-C-PLOT | Optional complete status vector; length validity rejected as data | B-L-M,B-L-G | Explicit unavailable-estimate content | B-M4/B-M10 |
| `axes.forest.scale-policy` | B-M4 Linear/Log10; default Linear; Forest-only setter | B-C-PLOT | Closed axis representation policy | B-L-M,B-L-G | Forest numeric scale | B-M4 |
| `facets.layout.current-policy` | B-M5 Row/Column/Grid/Wrap with constructor checks; Plot optional storage | B-C-PLOT | Closed layout; grid dimensions distinct, wrap columns positive | B-L-M,B-L-R,B-L-G | Generic facet layout | B-M5/B-M10 |
| `facets.scales.current-policy` | B-M5 Fixed/FreeX/FreeY/Free; default Fixed | B-C-PLOT | Exact replacement on FacetSpec | B-L-M,B-L-G | Facet numeric-axis sharing | B-M5 |
| `guides.exact.color` | B-M6 GuideStyle opaque RGB | B-C-STYLE | Exact caller paint | B-L-N | Guide stroke color | B-M6 |
| `guides.exact.width` | B-M6 finite positive f32 | B-C-STYLE | Constructor rejects invalid width | B-L-G | Guide stroke width | B-M6 |
| `guides.exact.pattern` | B-M6 closed LinePattern | B-C-STYLE | Exact caller pattern | B-L-N | Guide stroke pattern | B-M6 |
| `guides.geometry.presence-orientation` | B-M6 Horizontal/Vertical/Segment; Plot append vector | B-C-PLOT | Ordered guides; axis compatibility separate | B-L-G | Supplemental guide presence/orientation | B-M6/B-M10 |
| `layers.point.presence-coordinates` | B-M6 PointLayer x/y plus Plot append vector; exact Line-only gate B-M13 `orchestrator/layers.rs:58-62` | B-C-PLOT | Ordered zero-or-more strict layers; every other family rejects after erasure | B-L-G | Supplemental point-layer presence | B-M6/B-M13; exact family gate grouped |
| `layers.point.annotation-labels` | B-M6 optional labels; default None; consuming setter | B-C-PLOT | One label per point when present | B-L-M,B-L-G | PointLayer annotation content | B-M6 |
| `layers.point.legend-identity` | B-M6 optional legend_label; default None | B-C-PLOT | At most one entry per layer | B-L-M,B-L-R,B-L-G | PointLayer legend presence/identity | B-M6 |
| `layers.point.exact-color` | B-M6 `color: None` at `217` and `color(Some(rgb))` at `223-228`; private precedence conversion at `292-300` is T2e | B-C-STYLE | None selects the T2g-supplied automatic accent; Some selects exact caller paint | B-L-N | Optional PointLayer exact-paint override | B-M6; T2e precedence and T2g slot counterparts |
| `layers.point.default-radius` | B-M6 literal radius 4 at `218` | B-C-STYLE | Hardcoded public-constructor default | B-L-G | PointLayer default radius | B-M6 |
| `layers.point.exact-radius` | B-M6 radius setter plus shared positive validator | B-C-STYLE | Exact replacement; zero rejected | B-L-G | PointLayer exact radius | B-M6 |
| `layers.point.default-opacity` | B-M6 literal 0.8 at `219` | B-C-STYLE | Hardcoded public-constructor default | B-L-N | PointLayer default alpha | B-M6 |
| `layers.point.exact-opacity` | B-M6 setter accepts finite (0,1] | B-C-STYLE | Exact replacement | B-L-N | PointLayer exact alpha | B-M6 |
| `layers.bar-point.presence-positioning` | B-M6 required category/optional series/value plus Bar/HBar append; exact family gate B-M13 `orchestrator/layers.rs:63-68` | B-C-PLOT | Complete strict layer; only ordinary Bar/HBar; no legend | B-L-G | Bar-centered point-layer presence | B-M6/B-M13; exact family gate grouped |
| `layers.bar-point.exact-color` | B-M6 constructor-required RGB | B-C-STYLE | Exact caller paint | B-L-N | BarPointLayer paint | B-M6 |
| `layers.bar-point.exact-radius` | B-M6 constructor-required radius; shared positive validator | B-C-STYLE | Zero rejected | B-L-G | BarPointLayer radius | B-M6 |
| `layers.bar-point.opacity` | B-M6 constructor requires finite strict (0,1) | B-C-STYLE | Unlike PointLayer, 1.0 rejects | B-L-N | BarPointLayer alpha | B-M6 |
| `labels.inferred.mode-selection` | B-M7 Humanized/Raw; Labels default Humanized | B-C-LABELS | Closed automatic-label fallback mode | B-L-M,B-L-G | Inferred-label presentation mode | B-M7 |
| `labels.inferred.alias-insertion` | B-M7 `alias` inserts one raw/display pair | B-C-LABELS | Existing raw key is replaced | B-L-M,B-L-G | Singular alias mutation | B-M7 |
| `labels.inferred.aliases-bulk-extension` | B-M7 `aliases` extends from iteration | B-C-LABELS | Iterator order determines repeated-key replacement | B-L-M,B-L-G | Bulk alias mutation | B-M7 |
| `labels.inferred.alias-before-mode-precedence` | B-M7 alias lookup then humanized/raw fallback `72-80` | B-C-LABELS | Exact alias always wins | B-L-M,B-L-G | Alias resolution precedence | B-M7 |
| `labels.inferred.display-collision-failure` | B-M7 domain map branch `82-107` | B-C-LABELS | Distinct raw identities may not share display | B-L-M,B-L-C | Display-domain uniqueness | B-M7 |
| `labels.inferred.humanization-rule` | B-M7 fixed split/case/acronym functions `110-200` | B-C-LABELS | Deterministic ASCII-oriented transformation | B-L-M,B-L-G | Automatic humanization invariant | B-M7 |
| `geometry.raincloud.dimension-category` | B-M8 `RaincloudDimensions.category` | B-C-GEOMETRY,B-C-LABELS | Required raw string; shared nonempty validation | B-L-M,B-L-G | Inferred categorical x-title identity | B-M8 |
| `geometry.raincloud.dimension-value` | B-M8 `RaincloudDimensions.value` | B-C-GEOMETRY,B-C-LABELS | Required raw string; shared nonempty validation | B-L-M,B-L-G | Inferred continuous y-title identity | B-M8 |
| `geometry.raincloud.dimension-hue` | B-M8 optional `RaincloudDimensions.hue` | B-C-GEOMETRY,B-C-LABELS,B-C-SCALE | Optional raw legend/color identity | B-L-M,B-L-G | Raincloud hue semantic identity | B-M8 |
| `geometry.histogram.dimension-value` | B-M8 `HistogramDimensions.value` | B-C-GEOMETRY,B-C-LABELS | Required raw string; shared nonempty validation | B-L-M,B-L-G | Precomputed histogram x-title identity | B-M8 |
| `geometry.histogram.dimension-density` | B-M8 `HistogramDimensions.density` | B-C-GEOMETRY,B-C-LABELS | Required raw string; shared nonempty validation | B-L-M,B-L-G | Precomputed histogram y-title identity | B-M8 |
| `geometry.dimensions.nonempty-validation` | B-M8 `validate_dimension` at `400-405` | B-C-GEOMETRY | Rejects trim-empty; surrounding whitespace otherwise retained | B-L-N | Shared geometry-dimension presence rule | B-M8; one helper, five callers |
| `geometry.raincloud.maximum-groups` | B-M8 `MAX_RAINCLOUD_GROUPS=512` and branch `135-140` | B-C-GEOMETRY | Rejects group count above 512 | B-L-N | Raincloud group resource ceiling | B-M8 |
| `geometry.raincloud.maximum-density-points-per-group` | B-M8 constant 2,048 and branch `206-211` | B-C-GEOMETRY | Rejects one oversized density boundary | B-L-N | Per-group density resource ceiling | B-M8 |
| `geometry.raincloud.maximum-density-points-total` | B-M8 constant 1,000,000 and checked accumulation `148,173-183` | B-C-GEOMETRY | Rejects overflow or aggregate excess | B-L-N | Total raincloud density work ceiling | B-M8 |
| `geometry.raincloud.maximum-raw-points-total` | B-M8 constant 1,000,000 and checked accumulation `149,176-188` | B-C-GEOMETRY | Rejects overflow or aggregate excess | B-L-N | Total raincloud point work ceiling | B-M8 |
| `geometry.histogram.maximum-bins` | B-M8 `MAX_HISTOGRAM_BINS=1,000,000` and branch `285-290` | B-C-GEOMETRY | Rejects excess bins | B-L-N | Histogram bin resource ceiling | B-M8 |
| `geometry.histogram.maximum-kde-points` | B-M8 `MAX_HISTOGRAM_KDE_POINTS=2,048` and branch `291-296` | B-C-GEOMETRY | Rejects excess KDE points | B-L-N | Histogram KDE resource ceiling | B-M8 |
| `geometry.histogram.maximum-rugs` | B-M8 `MAX_HISTOGRAM_RUGS=1,000,000` and branch `297-302` | B-C-GEOMETRY | Rejects excess rug segments | B-L-N | Histogram rug resource ceiling | B-M8 |
| `geometry.raincloud.groups-present` | B-M8 branch `132-134` | B-C-GEOMETRY | At least one group required | B-L-G | Minimum raincloud topology | B-M8 |
| `geometry.raincloud.hue-presence-consistency` | B-M8 branches `141-146,156-161` | B-C-GEOMETRY,B-C-SCALE | Dimension hue and every group hue are all present or all absent | B-L-G | Complete raincloud hue schema | B-M8; two levels, one invariant |
| `geometry.raincloud.group-category-nonempty` | B-M8 branch `152-156` | B-C-GEOMETRY | Rejects trim-empty group category | B-L-N | Group categorical identity presence | B-M8 |
| `geometry.raincloud.group-hue-nonempty` | B-M8 branch `162-166` | B-C-GEOMETRY | Present hue may not be trim-empty | B-L-N | Group hue identity presence | B-M8 |
| `geometry.raincloud.group-identity-unique` | B-M8 set insertion branch `147,167-172` | B-C-GEOMETRY,B-C-SCALE | Rejects duplicate exact category/hue pair | B-L-G | One geometry group per semantic identity | B-M8 |
| `geometry.raincloud.density-minimum-points` | B-M8 branch `201-205` | B-C-GEOMETRY | Each boundary needs at least two points | B-L-G | Minimum density-polyline topology | B-M8 |
| `geometry.raincloud.density-point-domain` | B-M8 branch `212-220` | B-C-GEOMETRY | Finite y and finite normalized width in [0,1] | B-L-G | Renderable density boundary coordinates | B-M8 |
| `geometry.raincloud.density-y-order` | B-M8 branch `221-225` | B-C-GEOMETRY | y coordinates strictly increase | B-L-G | Non-self-reversing density topology | B-M8 |
| `geometry.raincloud.density-positive-width` | B-M8 branch `227-235` | B-C-GEOMETRY | At least one normalized width exceeds zero | B-L-G | Intelligible nonzero density body | B-M8 |
| `geometry.raincloud.summary-order` | B-M8 branch `236-245` | B-C-GEOMETRY | Finite q1 <= median <= q3 | B-L-G | Ordered summary geometry | B-M8 |
| `geometry.raincloud.summary-within-support` | B-M8 branch `246-252` | B-C-GEOMETRY | q1 and q3 lie within density support | B-L-G | Summary/body support compatibility | B-M8 |
| `geometry.raincloud.raw-points-present` | B-M8 branch `253-257` | B-C-GEOMETRY | Every group requires at least one raw point | B-L-G | Minimum raw-strip topology | B-M8 |
| `geometry.raincloud.raw-point-domain` | B-M8 branch `258-266` | B-C-GEOMETRY | Finite y and normalized strip in [0,1] | B-L-G | Renderable raw-point coordinates | B-M8 |
| `geometry.raincloud.raw-point-within-support` | B-M8 branch `267-271` | B-C-GEOMETRY | Every raw y lies within density support | B-L-G | Raw/body support compatibility | B-M8 |
| `geometry.histogram.bins-present` | B-M8 branch `280-284` | B-C-GEOMETRY | At least one bin required | B-L-G | Minimum histogram topology | B-M8 |
| `geometry.histogram.bin-domain` | B-M8 branch `303-313` | B-C-GEOMETRY | Finite lower < upper and finite nonnegative density | B-L-G | Renderable histogram bin geometry | B-M8 |
| `geometry.histogram.bins-contiguous` | B-M8 exact partial comparison `314-323` | B-C-GEOMETRY | Every lower edge exactly equals previous upper edge | B-L-G | Ordered contiguous bin topology | B-M8 |
| `geometry.histogram.positive-bin-density` | B-M8 branch `325-329` | B-C-GEOMETRY | At least one bin density exceeds zero | B-L-G | Intelligible nonzero histogram body | B-M8 |
| `geometry.histogram.kde-minimum-points` | B-M8 branch `337-342` | B-C-GEOMETRY | At least two KDE points required | B-L-G | Minimum KDE polyline topology | B-M8 |
| `geometry.histogram.kde-point-domain` | B-M8 branch `343-348` | B-C-GEOMETRY | Finite x and finite nonnegative y | B-L-G | Renderable KDE coordinates | B-M8 |
| `geometry.histogram.kde-within-bin-support` | B-M8 branch `349-354` | B-C-GEOMETRY | Every x lies in inclusive bin range | B-L-G | KDE/bin support compatibility | B-M8 |
| `geometry.histogram.kde-x-order` | B-M8 branch `355-359` | B-C-GEOMETRY | KDE x coordinates strictly increase | B-L-G | Non-self-reversing KDE topology | B-M8 |
| `geometry.histogram.positive-kde-density` | B-M8 branch `361-365` | B-C-GEOMETRY | At least one KDE y exceeds zero | B-L-G | Intelligible nonzero KDE | B-M8 |
| `geometry.histogram.rugs-present` | B-M8 branch `369-374` | B-C-GEOMETRY | At least one rug required | B-L-G | Minimum rug topology | B-M8 |
| `geometry.histogram.rug-domain` | B-M8 branch `375-385` | B-C-GEOMETRY | Finite x and 0 <= baseline < top | B-L-G | Renderable rug segment geometry | B-M8 |
| `geometry.histogram.rug-within-bin-support` | B-M8 branch `386-391` | B-C-GEOMETRY | Every rug x lies in inclusive bin range | B-L-G | Rug/bin support compatibility | B-M8 |
| `capabilities.hue.family-matrix` | B-M9 `kind::Hue`: Scatter, Density, Line, Trajectory, LinePath, Interaction, Marginal, Matrix | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability only | B-L-N | Categorical hue-control family matrix | B-M9 |
| `capabilities.group.family-matrix` | B-M9 `kind::Group`: Density, Line, Marginal, Matrix, Dumbbell | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability; constructor-owned groups differ | B-L-N | Graphical-group control matrix | B-M9 |
| `capabilities.series.family-matrix` | B-M9 `kind::Series`: Bar, HBar, Boxplot, Violin, Raincloud | B-C-PLOT,B-E-COMPILE | Sealed availability; precomputed raincloud rejected later | B-L-N | Positioned-series control matrix | B-M9 |
| `capabilities.continuous-color.family-matrix` | B-M9 `kind::ContinuousColor`: Heatmap, Beeswarm | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Continuous-color title/control matrix | B-M9 |
| `capabilities.numeric-x.family-matrix` | B-M9 `kind::NumericX` exact 17-family list `112-115` | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Numeric-x limit/tick matrix | B-M9 |
| `capabilities.numeric-y.family-matrix` | B-M9 `kind::NumericY` exact 15-family list `116-119` | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Numeric-y limit/tick matrix | B-M9 |
| `capabilities.guides.family-matrix` | B-M9 `kind::Guides` exact 23-family list `120-124` | B-C-PLOT,B-E-COMPILE | Sealed availability; orientation checked later | B-L-N | Supplemental-guide family matrix | B-M9 |
| `capabilities.heatmap-options.family-matrix` | B-M9 `kind::HeatmapOptions`: Heatmap | B-C-PLOT,B-E-COMPILE | Sealed singleton availability | B-L-N | Heatmap-specific control matrix | B-M9 |
| `capabilities.category-order.family-matrix` | B-M9 `kind::CategoryOrder`: PointInterval and four bar families | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Primary category-order matrix | B-M9 |
| `capabilities.signed-bars.family-matrix` | B-M9 `kind::SignedBars`: HorizontalBar | B-C-PLOT,B-E-COMPILE | Sealed singleton availability | B-L-N | Sign-color policy matrix | B-M9 |
| `capabilities.zero-line.family-matrix` | B-M9 `kind::ZeroLine`: PointInterval, HBar, StackedHBar, Beeswarm | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Zero-reference policy matrix | B-M9 |
| `capabilities.estimate-status.family-matrix` | B-M9 `kind::EstimateStatus`: PointInterval, Coefficient, Forest | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Estimate-status content matrix | B-M9 |
| `capabilities.markers.family-matrix` | B-M9 `kind::Markers`: Line, LinePath, Interaction | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Marker-visibility policy matrix | B-M9 |
| `capabilities.coefficient-options.family-matrix` | B-M9 `kind::CoefficientOptions`: Coefficient | B-C-PLOT,B-E-COMPILE | Sealed singleton availability | B-L-N | Coefficient-label control matrix | B-M9 |
| `capabilities.categorical-colors.family-matrix` | B-M9 `kind::CategoricalColors`: LinePath, Interaction, Violin, Raincloud | B-C-SCALE,B-E-COMPILE | Sealed compile-time availability | B-L-N | Exact categorical-map control matrix | B-M9 |
| `capabilities.panel-text.family-matrix` | B-M9 `kind::PanelText` exact 24-family list `143-147` | B-C-PLOT,B-E-COMPILE | Matrix excluded at compile time | B-L-N | Panel-text presence matrix | B-M9 |
| `capabilities.point-outline.family-matrix` | B-M9 `kind::PointOutline` exact 18-family list `148-151` | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Point-outline policy matrix | B-M9 |
| `capabilities.significance-style.family-matrix` | B-M9 `kind::SignificanceStyle` exact eight-family list `152-154` | B-C-STYLE,B-E-COMPILE | Sealed compile-time availability | B-L-N | Significance exact-style matrix | B-M9 |
| `capabilities.significance-attached.family-matrix` | B-M9 `Significance<Attached>`: Bar, HBar, Coefficient | B-C-PLOT,B-E-COMPILE | Placement-specific sealed implementations | B-L-N | Attached-significance matrix | B-M9; split from generic trait |
| `capabilities.significance-pairwise.family-matrix` | B-M9 `Significance<Pairwise>` exact seven-family list `166-172` | B-C-PLOT,B-E-COMPILE | Placement-specific sealed implementations | B-L-N | Pairwise-significance matrix | B-M9; split from generic trait |
| `capabilities.legend.family-matrix` | B-M9 `kind::Legend` exact 17-family list `173-176` | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Plot legend-policy matrix | B-M9 |
| `capabilities.raw-overlay.family-matrix` | B-M9 `kind::RawOverlay`: Boxplot, Violin | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Raw-overlay policy matrix | B-M9 |
| `capabilities.facet.family-matrix` | B-M9 `kind::Facet` exact 23-family list `180-184` | B-C-PLOT,B-E-COMPILE | Sealed compile-time availability | B-L-N | Generic-facet policy matrix | B-M9 |
| `channels.hue.optional-control` | B-M10 optional `Plot.hue`; setter `configuration.rs:25-33` | B-C-PLOT,B-C-SCALE | Absence means no separate categorical hue | B-L-M,B-L-G | Optional categorical color identity | B-M10; capability and erased validation separate |
| `channels.group.optional-control` | B-M10 optional `Plot.group`; setter `35-43` | B-C-PLOT | Absence means no separately configured grouping | B-L-G | Optional graphical grouping identity | B-M10 |
| `channels.series.optional-control` | B-M10 optional `Plot.series`; setter `45-54` | B-C-PLOT,B-C-SCALE | Absence means one positioned series; constructor-owned series differ | B-L-G | Optional positioned series identity | B-M10 |
| `legend.continuous.title-content` | B-M10 optional color label; setter/read `56-64`, `419-423` | B-C-PLOT,B-C-SCALE | Caller string replaces inferred title; no local lexical validation | B-L-M,B-L-R,B-L-G | Continuous color-guide title content | B-M10 |
| `heatmap.highlight.presence-values` | B-M10 optional BooleanValues; setter/read `86-94`, `467-469` | B-C-PLOT | Complete aligned mask enables highlighted borders | B-L-G | Heatmap highlighted-cell presence | B-M10; family gate grouped |
| `annotations.coefficient.estimate-label-presence` | B-M10 optional NumericLabelFormat; setter/read `96-104`, `471-475` | B-C-PLOT | Presence enables labels beside point-only estimates | B-L-M,B-L-R,B-L-G | Coefficient numeric-label presence | B-M10; family gate grouped |
| `content.plot.title` | B-M10 optional title; setter/read `116-121`, `686-690` | B-C-PLOT | Verbatim caller content; empty behaves absent in layout paths | B-L-M,B-L-R,B-L-G | Plot title content | B-M10 |
| `content.plot.subtitle` | B-M10 optional subtitle; setter/read `123-128`, `692-696` | B-C-PLOT | Verbatim caller content; empty behaves absent | B-L-M,B-L-R,B-L-G | Plot subtitle content | B-M10 |
| `content.plot.caption` | B-M10 optional caption; setter/read `130-135`, `698-702` | B-C-PLOT | Verbatim caller content; empty behaves absent | B-L-M,B-L-R,B-L-G | Plot caption content | B-M10 |
| `content.axis.x-title` | B-M10 optional x label; setter/read `137-142`, `704-708` | B-C-PLOT,B-C-LABELS | Exact content replaces family inference; empty suppresses drawn text | B-L-M,B-L-R,B-L-G | Caller-owned x-axis title | B-M10 |
| `content.axis.y-title` | B-M10 optional y label; setter/read `144-149`, `710-714` | B-C-PLOT,B-C-LABELS | Exact content replaces family inference; empty suppresses drawn text | B-L-M,B-L-R,B-L-G | Caller-owned y-axis title | B-M10 |
| `axes.x.exact-ticks-control` | B-M10 optional Vec<AxisTick>; setter/read `180-188`, `859-861` | B-C-PLOT | Presence replaces automatic ticks; layout validation is T2c | B-L-M,B-L-C,B-L-G | Exact numeric x-tick content | B-M10; numeric-axis gate B-M13 |
| `axes.y.exact-ticks-control` | B-M10 optional Vec<AxisTick>; setter/read `190-198`, `869-871` | B-C-PLOT | Presence replaces automatic ticks; layout validation is T2c | B-L-M,B-L-C,B-L-G | Exact numeric y-tick content | B-M10; numeric-axis gate B-M13 |
| `axes.x.explicit-limits-control` | B-M10 AxisSpec plus explicit flag; setter/read `158-167`, `972-978` | B-C-PLOT | Caller range distinguishes explicit from planned limits | B-L-G | Numeric x-domain policy | B-M10; AxisSpec validity T2c |
| `axes.y.explicit-limits-control` | B-M10 AxisSpec plus explicit flag; setter/read `169-178`, `980-986` | B-C-PLOT | Caller range distinguishes explicit from planned limits | B-L-G | Numeric y-domain policy | B-M10; AxisSpec validity T2c |
| `bars.sign-color.presence` | B-M10 false default; `color_by_sign()` sets true | B-C-PLOT,B-C-SCALE | One-way enable on HorizontalBar | B-L-N | Positive/negative data-color policy | B-M10 |
| `references.zero-line.presence` | B-M10 false default; `zero_line()` sets true | B-C-PLOT,B-C-STYLE | One-way enable on four exact families | B-L-G | Zero-reference presence | B-M10 |
| `marks.estimate-marker.visibility` | B-M10 true default; `markers(bool)` | B-C-PLOT,B-C-STYLE | Exact replacement; interaction compatibility later | B-L-G | Estimate marker presence | B-M10 |
| `marks.point-outline.visibility` | B-M10 default `Some(PointOutline::default())`; hide sets None | B-C-PLOT,B-C-STYLE | One-way removal; appearance default prior/later-owned | B-L-G | Point-outline presence | B-M10 |
| `annotations.significance.presence` | B-M10 empty erased vector; typed append setter `328-340` | B-C-PLOT | Ordered zero-or-more annotations; placement capability exact | B-L-R,B-L-C,B-L-G | Significance annotation presence | B-M10/B-M9 |
| `output.plot.pixel-size` | B-M10 default 640x480; `size(width,height)`; accessors | B-C-PLOT | Pair replaced without setter validation; lifecycle validates later | B-L-M,B-L-R,B-L-G | Plot output dimensions | B-M10 |
| `color-scales.continuous.heatmap-domain` | B-M10 optional `(min,max)` via Heatmap `color_limits` | B-C-SCALE | Exact saturation limits; validity/data mapping T2g | B-L-N | Plot-local heatmap numeric domain | B-M10 |
| `color-scales.continuous.beeswarm-domain` | B-M10 optional `(min,max)` via Beeswarm `color_limits` | B-C-SCALE | Exact saturation limits; validity/data mapping T2g | B-L-N | Plot-local beeswarm numeric domain | B-M10 |
| `intervals.line.band-presence` | B-M10 optional Bounds via Line `band` | B-C-PLOT,B-C-STYLE | Presence draws one caller-bounded band | B-L-G | Line interval-band presence | B-M10; exact family gate grouped |
| `references.coefficient.presence-value` | B-M10 optional f64 via Coefficient `reference_line` | B-C-PLOT,B-C-STYLE | Presence draws exact scientific reference | B-L-G | Coefficient reference-line policy | B-M10; family gate grouped |
| `references.forest.required-value` | B-M10 constructor `Plot::forest(...,reference)` stores f64; no default in public path | B-C-PLOT,B-C-STYLE | Required scientific reference; validity later | B-L-G | Forest reference-line value | B-M10 |
| `interaction.connectors.visibility` | B-M10 optional bool; None resolves visible, setter sets exact bool | B-C-PLOT,B-C-STYLE | Explicit presence/absence with compatibility branches | B-L-G | Interaction connector presence | B-M10; family gate grouped |
| `histogram.geometry.density-stat-precedence` | B-M10 `Plot::histogram` forces Density when input is precomputed geometry `606-615` | B-C-PLOT,B-C-GEOMETRY | Constructor correction precedes later user `stat` replacement, which may then fail preflight | B-L-G | Precomputed histogram normalization precedence | B-M10 |
| `canvas.category-labels.sharing-policy` | B-M11 EveryPlot/FirstColumnPerRow; default EveryPlot | B-C-CANVAS | Closed replacement; exact compatibility branches below | B-L-M,B-L-R,B-L-G | Composed categorical-label sharing | B-M11 |
| `canvas.text-panel.title-content` | B-M11 private title set by `TextPanel::new`; accessor | B-C-CANVAS | Required caller prose, measured and wrapped | B-L-M,B-L-R,B-L-G | Text-panel title content | B-M11 |
| `canvas.text-panel.body-content` | B-M11 private body set by `TextPanel::new`; accessor | B-C-CANVAS | Required caller prose, measured and wrapped | B-L-M,B-L-R,B-L-G | Text-panel body content | B-M11 |
| `canvas.text-panel.lexical-validation` | B-M11 shared role-aware helper `98-110`, called for title then body | B-C-CANVAS | Each value nonempty, trimmed, control-free | B-L-N | Text-panel lexical contract | B-M11; one helper, two ordered calls |
| `canvas.grid.cardinality` | B-M11 `Canvas::grid`; rows/columns positive and checked product | B-C-CANVAS | Rejects zero or allocation overflow | B-L-G | Canvas grid dimensions/cell count | B-M11 |
| `canvas.plot-panel.cell-range` | B-M11 `panel` bounds branch `221-226` | B-C-CANVAS | Rejects out-of-grid plot cell | B-L-N | Plot-panel cell addressing | B-M11 |
| `canvas.text-panel.cell-range` | B-M11 `text_panel` bounds branch `244-249` | B-C-CANVAS | Rejects out-of-grid text cell | B-L-N | Text-panel cell addressing | B-M11 |
| `canvas.cell.single-occupancy` | B-M11 identical plot/text occupancy branches `228-232,251-255` | B-C-CANVAS | A cell may contain at most one plot or text panel | B-L-G | Canvas cell occupancy invariant | B-M11; two setters, one rule |
| `canvas.legend.shared-gap` | B-M11 optional finite nonnegative f64 px; default None | B-C-CANVAS | Exact replacement; absence uses later legend-derived fallback | B-L-R,B-L-G | Shared legend-to-grid gap | B-M11; compatibility separate |
| `canvas.headers.column-content-range` | B-M11 `column_header` plus common header/range/lexical/overlap validators | B-C-CANVAS | Ordered nonoverlapping contiguous column spans | B-L-M,B-L-R,B-L-C,B-L-G | Shared column-header content/presence | B-M11 |
| `canvas.headers.row-content-range` | B-M11 `row_header` plus common validators | B-C-CANVAS | Ordered nonoverlapping contiguous row spans | B-L-M,B-L-R,B-L-C,B-L-G | Shared row-header content/presence | B-M11 |
| `canvas.headers.row-title-content` | B-M11 `row_title` as span-one header plus overlap validator | B-C-CANVAS | At most one validated title per row | B-L-M,B-L-R,B-L-C,B-L-G | Canvas row-title content/presence | B-M11 |
| `canvas.x-range-group.cell-range` | B-M11 branch `325-331` | B-C-CANVAS | Every requested cell must be in grid | B-L-N | Shared-x group cell addressing | B-M11 |
| `canvas.x-range-group.unique-membership` | B-M11 branch `333-342` | B-C-CANVAS | No duplicate within or across groups | B-L-G | One shared-x owner per panel | B-M11 |
| `canvas.x-range-group.nonempty` | B-M11 branch `345-349` | B-C-CANVAS | Each group contains at least one panel | B-L-N | Minimum shared-x group cardinality | B-M11 |
| `canvas.output.pixel-size` | B-M11 default 960x640; `size(width,height)` | B-C-CANVAS | Pair replaced; minimum checked at preflight | B-L-M,B-L-R,B-L-G | Canvas output dimensions | B-M11 |
| `content.canvas.title` | B-M11 optional title setter | B-C-CANVAS | Verbatim caller content | B-L-M,B-L-R,B-L-G | Canvas title content | B-M11 |
| `content.canvas.subtitle` | B-M11 optional subtitle setter | B-C-CANVAS | Verbatim caller content | B-L-M,B-L-R,B-L-G | Canvas subtitle content | B-M11 |
| `content.canvas.caption` | B-M11 optional caption setter | B-C-CANVAS | Verbatim caller content | B-L-M,B-L-R,B-L-G | Canvas caption content | B-M11 |
| `canvas.labels.override-policy` | B-M11 optional complete Labels via `Canvas::labels` | B-C-CANVAS,B-C-LABELS | When present, clones replacement into every configured Plot | B-L-M,B-L-G | Canvas-wide inferred-label policy | B-M11 |
| `validation.canvas.minimum-size` | B-M11 branch `466-469`: at least 160x120 | B-C-CANVAS | Preflight failure before rendering | B-L-N | Minimum composed-canvas representability | B-M11 |
| `validation.canvas.nonempty-content` | B-M11 branch `470-474` | B-C-CANVAS | Requires at least one plot or text panel | B-L-N | Minimum Canvas content presence | B-M11 |
| `validation.canvas.shared-gap-requires-shared-legend` | B-M11 branch `475-479` | B-C-CANVAS | Explicit gap rejects under PerPlot/Hidden | B-L-N | Legend-gap/policy compatibility | B-M11 |
| `validation.canvas.faceted-plot-incompatibility` | B-M11 branch `480-487` | B-C-CANVAS | Generic-faceted Plot cannot be nested | B-L-N | Canvas/facet ownership compatibility | B-M11 |
| `validation.canvas.row-title-panel-title-incompatibility` | B-M11 branch `489-500` | B-C-CANVAS | Row title rejects any nonempty Plot title/subtitle in row | B-L-M,B-L-C | Unambiguous row-header ownership | B-M11 |
| `validation.canvas.shared-category-label-complete-row` | B-M11 branches `689-704` | B-C-CANVAS | FirstColumnPerRow requires every cell in every row to contain a Plot, including the first-column label owner | B-L-N | Complete shared-label row occupancy | B-M11; domain comparison deferred to T2g |
| `validation.canvas.shared-category-label-family` | B-M11 branch `1079-1088` | B-C-CANVAS | Only horizontal bar families admitted | B-L-N | Shared category-label family compatibility | B-M11 |
| `validation.canvas.shared-category-label-axis` | B-M11 branch `1089-1094` | B-C-CANVAS | Requires categorical y-axis | B-L-N | Shared category-label axis compatibility | B-M11 |
| `validation.raincloud-geometry.series-incompatibility` | B-M12 branch `32-37` | B-C-PLOT,B-C-GEOMETRY | Precomputed geometry rejects configured series through the unified group path before channel validation | B-L-N | Self-contained raincloud/series compatibility | B-M12; ordered overlap with external-channel rejection |
| `validation.category-order.primary-family` | B-M12 branch `50-65` | B-C-PLOT | Non-Observed primary order limited to seven erased families | B-L-N | Erased primary-order family support | B-M12 |
| `validation.category-order.heatmap-axis-family` | B-M12 branch `66-73` | B-C-PLOT | Non-Observed x/y orders limited to Heatmap | B-L-N | Erased heatmap-axis-order support | B-M12 |
| `validation.category-order.value-facet-incompatibility` | B-M12 branch `74-78` | B-C-PLOT | Value-derived primary order rejects faceting | B-L-N | Value-order/facet compatibility | B-M12 |
| `validation.facets.self-managed-family` | B-M12 branch `222-227` | B-C-PLOT | Matrix and Marginal reject generic facets | B-L-N | Self-managed/facet compatibility | B-M12 |
| `validation.facets.panel-text-incompatibility` | B-M12 branch `228-233` | B-C-PLOT | Any panel-text annotation rejects faceting | B-L-N | Annotation panel ownership | B-M12 |
| `validation.significance.facet-owner-requires-facet` | B-M12 branch `237-246` | B-C-PLOT | Facet-cell annotation rejects unfaceted Plot | B-L-N | Significance facet ownership | B-M12 |
| `validation.intervals.ordinary-bar-line-band-incompatibility` | B-M12 branch `256-263` | B-C-PLOT | Ordinary bars reject generic interval-bound columns | B-L-N | Bar-bound source compatibility | B-M12 |
| `validation.intervals.stacked-bar-bounds-incompatibility` | B-M12 branch `264-270` | B-C-PLOT | Stacked bars reject interval bounds | B-L-N | Stacked-bar interval compatibility | B-M12 |
| `validation.panel-text.matrix-incompatibility` | B-M12 branch `286-291` | B-C-PLOT | Matrix rejects standard-panel text | B-L-N | Panel-text panel-shape compatibility | B-M12 |
| `validation.panel-text.unique-corners` | B-M12 set branch `292-300` | B-C-PLOT | At most one annotation per PanelCorner | B-L-C | Panel-text collision ownership | B-M12 |
| `validation.histogram-geometry.family` | B-M12 branch `302-307` | B-C-PLOT,B-C-GEOMETRY | Precomputed histogram geometry only under Histogram | B-L-N | Geometry/family compatibility | B-M12 |
| `validation.histogram-geometry.facet-incompatibility` | B-M12 branch `308-313` | B-C-PLOT,B-C-GEOMETRY | Precomputed histogram geometry rejects facets | B-L-N | Self-contained geometry/facet compatibility | B-M12 |
| `validation.histogram-geometry.density-stat` | B-M12 branch `314-319` | B-C-PLOT,B-C-GEOMETRY | Precomputed geometry requires Density stat | B-L-N | Geometry/axis normalization compatibility | B-M12 |
| `validation.histogram-geometry.sample-options-incompatibility` | B-M12 branch `320-327` | B-C-PLOT,B-C-GEOMETRY | Requires Auto bins and HistogramOnly options | B-L-N | Caller geometry/sample-policy precedence | B-M12 |
| `validation.histogram-geometry.external-channels-incompatibility` | B-M12 branch `328-337` | B-C-PLOT,B-C-GEOMETRY | Rejects x/y/hue/group channels | B-L-N | Self-contained histogram channel schema | B-M12 |
| `validation.histogram-geometry.source-rows-incompatibility` | B-M12 branch `338-343` | B-C-PLOT,B-C-GEOMETRY | Rejects source-row inputs | B-L-N | Self-contained histogram row schema | B-M12 |
| `validation.raincloud-geometry.family` | B-M12 branch `345-350` | B-C-PLOT,B-C-GEOMETRY | Precomputed geometry only under Raincloud | B-L-N | Geometry/family compatibility | B-M12 |
| `validation.raincloud-geometry.facet-incompatibility` | B-M12 branch `351-355` | B-C-PLOT,B-C-GEOMETRY | Precomputed raincloud geometry rejects facets | B-L-N | Self-contained geometry/facet compatibility | B-M12 |
| `validation.raincloud-geometry.external-channels-incompatibility` | B-M12 branch `356-365` | B-C-PLOT,B-C-GEOMETRY | Rejects external x/y/hue/group sample channels; configured series has already failed the earlier unified-group branch | B-L-N | Self-contained raincloud channel schema | B-M12; ordered overlap with series incompatibility |
| `validation.raincloud-geometry.source-rows-incompatibility` | B-M12 branch `366-370` | B-C-PLOT,B-C-GEOMETRY | Rejects source-row inputs | B-L-N | Self-contained raincloud row schema | B-M12 |
| `validation.raincloud-geometry.significance-incompatibility` | B-M12 branch `371-377` | B-C-PLOT,B-C-GEOMETRY | Rejects significance annotations | B-L-N | Precomputed geometry/annotation compatibility | B-M12 |
| `validation.line-family.explicit-group-required` | B-M12 branch `398-414` | B-C-PLOT | Trajectory and LinePath require explicit group and fail here before the later five-family erased check | B-L-N | Collective line identity presence | B-M12; ordered duplicate for two required-group families |
| `validation.line-path.hue-style-requires-hue` | B-M12 branch `415-426` | B-C-PLOT,B-C-SCALE | Hue style is LinePath-only and requires hue | B-L-N | LinePath style/channel compatibility | B-M12; family arm grouped with control |
| `validation.channels.hue-erased-family` | B-M12 matrix/branch `427-454` | B-C-PLOT | Rejects hue outside exact erased matrix; Histogram has focused diagnostic | B-L-N | Erased hue family support | B-M12 |
| `validation.channels.group-erased-family` | B-M12 matrix/branch `455-477` | B-C-PLOT | Rejects group outside exact erased matrix | B-L-N | Erased graphical-group support | B-M12 |
| `validation.channels.required-group-erased-family` | B-M12 branch `478-490` | B-C-PLOT | Five collective families require group after erasure; Trajectory/LinePath missing-group cases are already rejected earlier | B-L-N | Required graphical-group presence | B-M12; ordered duplicate with line-family check |
| `validation.interaction.styles-family-and-marker-compatibility` | B-M12 branches `491-502` | B-C-PLOT,B-C-STYLE | Styles are Interaction-only and require visible markers | B-L-N | Exact interaction-style compatibility | B-M12; two ordered failures, one configured object |
| `validation.interaction.connectors-family` | B-M12 branch `503-508` | B-C-PLOT | Explicit connector control is Interaction-only | B-L-N | Connector-policy family support | B-M12; exact control gate grouped for migration |
| `validation.interaction.hidden-connectors-style-incompatibility` | B-M12 branch `509-513` | B-C-PLOT,B-C-STYLE | `connectors(false)` rejects configured styles | B-L-N | Connector/style compatibility | B-M12 |
| `validation.interaction.visible-mark-required` | B-M12 branch `515-522` | B-C-PLOT | Markers and connectors cannot both be hidden | B-L-N | Minimum interaction intelligibility | B-M12 |
| `validation.distributions.redundant-hue-incompatibility` | B-M12 branch `523-531` | B-C-PLOT,B-C-SCALE | Intended to reject Boxplot/Violin redundant hue, but unified-series accessors compare internal `SERIES` with `X`, so the branch cannot observe equal raw identities | B-L-N | Dormant nonredundant categorical color identity check | B-M12; intentional T5d correction required |
| `validation.legend.continuous-title-family` | B-M12 branch `539-543` | B-C-PLOT | Explicit continuous title limited to continuous-color families | B-L-N | Color-guide title family support | B-M12 |
| `validation.estimate-status.family` | B-M12 branch `567-576` | B-C-PLOT | Status limited to PointInterval/Coefficient/Forest | B-L-N | Estimate-status family support | B-M12 |
| `validation.intervals.bound-family` | B-M12 matrix/branch `577-589` | B-C-PLOT | Generic bound columns limited to five families | B-L-N | Erased interval-bound support | B-M12 |
| `validation.matrix.variable-family` | B-M12 branch `595-601` | B-C-PLOT | Nonempty matrix-variable storage rejects non-Matrix kind | B-L-N | Matrix variable-list family support | B-M12 |
| `validation.matrix.external-channels-incompatibility` | B-M12 branch `605-610` | B-C-PLOT | Matrix rejects separately configured x or y channels | B-L-N | Self-contained Matrix channel schema | B-M12 |
| `validation.matrix.minimum-variables` | B-M12 branch `612-617` | B-C-PLOT | Matrix requires at least two ordered variables | B-L-G | Minimum Matrix panel topology | B-M12 |
| `validation.matrix.variable-name-nonempty` | B-M12 branch `618-624` | B-C-PLOT,B-C-LABELS | Rejects trim-empty variable names; surrounding whitespace and controls otherwise pass | B-L-M,B-L-G | Matrix variable identity presence | B-M12 |
| `validation.matrix.variable-name-unique` | B-M12 set/branch `618,625-629` | B-C-PLOT,B-C-LABELS | Rejects duplicate exact variable names | B-L-M,B-L-G | Unique Matrix panel identity | B-M12 |
| `validation.trajectory.categorical-x-limits-incompatibility` | B-M12 branch `634-650` | B-C-PLOT | Categorical trajectory x rejects numeric x limits | B-L-N | Axis-kind/limit compatibility | B-M12 |
| `validation.line-path.numeric-x-required` | B-M12 branch `653-668` | B-C-PLOT | LinePath x source must be numeric | B-L-N | Explicit path geometry axis invariant | B-M12 |
| `validation.supplemental.numeric-x-family-matrix` | B-M13 `numeric_axes` x arms `227-259` | B-C-PLOT | Closed erased runtime numeric-x classification | B-L-N | Supplemental numeric-x capability | B-M13; split from y |
| `validation.supplemental.numeric-y-family-matrix` | B-M13 `numeric_axes` y arms `227-259` | B-C-PLOT | Closed erased runtime numeric-y classification | B-L-N | Supplemental numeric-y capability | B-M13; split from x |
| `validation.supplemental.facet-incompatibility` | B-M13 atomic branch `13-24` | B-C-PLOT | Any exact ticks, guides, or point layers reject facets | B-L-N | Supplemental/facet compatibility | B-M13 |
| `validation.axes.x-exact-ticks-require-numeric` | B-M13 branch `27-31` | B-C-PLOT | Configured x ticks require runtime numeric x | B-L-N | Exact x-tick axis compatibility | B-M13 |
| `validation.axes.y-exact-ticks-require-numeric` | B-M13 branch `32-36` | B-C-PLOT | Configured y ticks require runtime numeric y | B-L-N | Exact y-tick axis compatibility | B-M13 |
| `validation.guides.horizontal-requires-numeric-y` | B-M13 match arm `39-43` | B-C-PLOT | Horizontal guide requires numeric y | B-L-N | Horizontal-guide axis compatibility | B-M13 |
| `validation.guides.vertical-requires-numeric-x` | B-M13 match arm `44-48` | B-C-PLOT | Vertical guide requires numeric x | B-L-N | Vertical-guide axis compatibility | B-M13 |
| `validation.guides.segment-requires-two-numeric-axes` | B-M13 match arm `49-53` | B-C-PLOT | Segment guide requires numeric x and y | B-L-N | Segment-guide axis compatibility | B-M13 |

### T2b Target State

| Property | Target classification and owner | Target resolution | Target validation | Public surface | Relationships |
| --- | --- | --- | --- | --- | --- |
| `bars.spacing.current-bar-spacing` | Explicit policy; retained within-category spacing role | Resolved Plot | Construction plus Plot preflight | Retained local control | Capability `capabilities.series.family-matrix` |
| `histogram.binning.current-selection` | Explicit policy; retained histogram binning role | Resolved Plot | Construction and data-dependent preflight | Retained local control | Geometry precedence `histogram.geometry.density-stat-precedence` |
| `histogram.normalization.current-stat` | Explicit policy; retained histogram normalization role | Resolved Plot | Construction and geometry compatibility preflight | Retained local control | Geometry validator `validation.histogram-geometry.density-stat` |
| `histogram.composite.kde-source` | Explicit policy; retained caller/internal KDE source role | Resolved Plot | Input construction and Plot preflight | Retained local control | Presence sibling `histogram.composite.presence` |
| `histogram.composite.presence` | Explicit policy; retained KDE/rug presence role | Resolved Plot | Construction and density-axis preflight | Retained local control | Source sibling `histogram.composite.kde-source` |
| `annotations.panel-text.presence-control` | Explicit policy; retained ordered annotation presence | Resolved Plot | Annotation construction and Plot preflight | Retained local control | Content/corner siblings; facet and corner validators |
| `annotations.panel-text.corner` | Explicit policy; retained exact panel-corner placement | Exact annotation | Closed construction | Retained `PanelTextAnnotation` field | Split parent `annotations.panel-text.presence-control` |
| `annotations.panel-text.content` | Explicit policy; retained caller annotation content | Exact annotation | Construction: nonempty, trimmed, control-free, and single-line, explicitly rejecting U+2028 and U+2029 | Retained `PanelTextAnnotation` field | Split parent `annotations.panel-text.presence-control` |
| `marks.raw-overlay.presence-layout` | Explicit policy; retained raw-observation presence/displacement | Resolved Plot | Closed construction and family preflight | Retained local control | Capability `capabilities.raw-overlay.family-matrix` |
| `color-scales.continuous.heatmap-default` | Theme data scale; `ColorScales` default sequential heatmap mapping | Resolved Plot/facet theme before local-scale replacement | Complete gradient construction and domain preflight | T6 `ColorScales` default | Local replacement `color-scales.continuous.heatmap-policy` |
| `color-scales.continuous.heatmap-policy` | Plot-local data scale; `ContinuousColorScale` kind and center | Plot-local scale before preflight | Construction plus domain/center preflight | Replaced by approved scale surface | Theme default; domain `color-scales.continuous.heatmap-domain` |
| `heatmap.cells.layout-policy` | Explicit policy; retained heatmap topology selection | Resolved Plot | Closed construction and family preflight | Retained local control | Category-order validators |
| `heatmap.labels.fit-policy` | Explicit policy; retained dense-label fit selection | Resolved Plot | Closed construction and layout-fit preflight | Retained local control | T2c label measurement counterpart |
| `annotations.numeric-label.precision` | Explicit policy; retained numeric-content formatting role | Exact annotation policy | Construction: 0..=12 | Retained local control | Suffix sibling; presence `annotations.coefficient.estimate-label-presence` |
| `annotations.numeric-label.suffix` | Explicit policy; retained numeric-content suffix role | Exact annotation policy | Construction: absent or nonempty/control-free | Retained local control | Precision sibling |
| `annotations.heatmap.cell-value-presence` | Explicit policy; retained heatmap annotation-content presence | Resolved Plot | Closed construction and Heatmap preflight | Retained local control | Numeric-label formatting siblings |
| `annotations.heatmap.null-row-presence` | Explicit policy; retained ordered null-row annotation presence | Resolved Plot | Annotation construction and Heatmap preflight | Retained local control | Target/text/lexical split children |
| `annotations.heatmap.null-row-target` | Explicit policy; retained exact row identity | Exact annotation | Construction lexical validation; data identity in preflight | Retained annotation field | Parent `annotations.heatmap.null-row-presence` |
| `annotations.heatmap.null-row-text` | Explicit policy; retained caller text content | Exact annotation | Construction lexical validation | Retained annotation field | Parent `annotations.heatmap.null-row-presence` |
| `annotations.heatmap.null-row-lexical-validation` | Renderer invariant; private null-row lexical contract | Exact annotation construction | Both fields nonempty, trimmed, control-free; reject | Private-only behavior | Target/text siblings |
| `marks.dumbbell.left-endpoint-identity` | Explicit policy; retained endpoint semantic label | Exact dumbbell input | Construction lexical validation | Retained input field | Right identity and compatibility validators |
| `marks.dumbbell.right-endpoint-identity` | Explicit policy; retained endpoint semantic label | Exact dumbbell input | Construction lexical validation | Retained input field | Left identity and compatibility validators |
| `marks.dumbbell.endpoint-identities-nonempty` | Renderer invariant; private endpoint identity validity | Input construction | Both identities nonempty; reject | Private-only behavior | Left/right identities |
| `marks.dumbbell.endpoint-identities-lexical-validity` | Renderer invariant; private endpoint lexical validity | Input construction | Both trimmed/control-free; reject | Private-only behavior | Left/right identities |
| `marks.dumbbell.endpoint-identities-distinct` | Renderer invariant; private endpoint compatibility | Input construction | Identities differ exactly; reject | Private-only behavior | Left/right identities |
| `marks.dumbbell.left-endpoint-color` | Exact element style; unnamed left-endpoint paint role | Exact dumbbell mark | `Color` construction | Replaced exact-style field | Right paint sibling; T6 RGBA carrier |
| `marks.dumbbell.right-endpoint-color` | Exact element style; unnamed right-endpoint paint role | Exact dumbbell mark | `Color` construction | Replaced exact-style field | Left paint sibling; T6 RGBA carrier |
| `annotations.waterfall.total-label-presence` | Explicit policy; retained complete label-pair presence | Resolved Plot | Pair construction and Waterfall preflight | Retained local control | Base/terminal label children |
| `annotations.waterfall.base-label` | Explicit policy; retained base content | Exact annotation input | Construction lexical validation | Retained input field | Parent `annotations.waterfall.total-label-presence` |
| `annotations.waterfall.terminal-label` | Explicit policy; retained terminal content | Exact annotation input | Construction lexical validation | Retained input field | Parent `annotations.waterfall.total-label-presence` |
| `annotations.waterfall.labels-nonempty` | Renderer invariant; private waterfall-label validity | Input construction | Both labels nonempty; reject | Private-only behavior | Base/terminal siblings |
| `annotations.waterfall.labels-lexical-validity` | Renderer invariant; private waterfall lexical validity | Input construction | Both trimmed/control-free; reject | Private-only behavior | Base/terminal siblings |
| `canvas.legend.composition-policy` | Explicit policy; approved `CanvasLegendPolicy` | Canvas root | Construction and shared-legend Canvas preflight | Retained named policy | Plot policy and shared-gap records |
| `legend.plot.visibility-placement-policy` | Explicit policy; retained Plot legend visibility/placement | Resolved Plot | Closed construction; Canvas shared compatibility preflight | Retained local control | Canvas policy; T2d legend geometry |
| `annotations.significance.erased-family-support` | Renderer invariant; private exact erased significance family support | Plot preflight after erasure | Runtime matrix equals public placement capabilities | Private-only behavior | Attached/pairwise capability records |
| `bars.input.estimate-source-and-bounds` | Explicit policy; retained bar statistical-input selection | Plot input | Construction and data/interval preflight | Retained input surface | Generated selection sibling |
| `bars.error.generated-selection` | Explicit policy; retained generated error-bar selection | Resolved Plot | Construction and BarSamples preflight | Retained input control | Four interval parameter siblings |
| `bars.error.standard-deviation-scale` | Explicit policy; retained SD multiplier | Exact generated-error policy | Construction: finite positive | Retained input field | Shared validator record |
| `bars.error.standard-error-scale` | Explicit policy; retained SE multiplier | Exact generated-error policy | Construction: finite positive | Retained input field | Shared validator record |
| `bars.error.spread-scale-validation` | Renderer invariant; private generated spread-scale validity | Policy construction | Finite and strictly positive; reject | Private-only behavior | SD/SE scale siblings |
| `bars.error.percentile-interval-coverage` | Explicit policy; retained percentile coverage | Exact generated-error policy | Construction: finite strict (0,1) | Retained input field | Shared coverage default/domain |
| `bars.error.confidence-interval-coverage` | Explicit policy; retained confidence coverage | Exact generated-error policy | Construction: finite strict (0,1) | Retained input field | Shared coverage default/domain |
| `bars.error.shared-coverage-default-domain` | Renderer invariant; private shared coverage default/validity | Policy construction | Default 0.95; finite strict (0,1) | Private-only behavior | Percentile/CI coverage siblings |
| `bars.error.confidence-algorithm` | Explicit policy; retained CI algorithm selection | Exact generated-error policy | Closed construction | Retained input field | Resample/seed/unit siblings |
| `bars.error.bootstrap-resamples` | Explicit policy; retained bootstrap work count | Exact generated-error policy | Construction: 1..=1,000,000 | Retained input field | Confidence algorithm sibling |
| `bars.error.bootstrap-seed` | Explicit policy; retained deterministic bootstrap seed | Exact generated-error policy | Any u64 | Retained input field | Confidence algorithm sibling |
| `bars.error.bootstrap-units` | Explicit policy; retained optional bootstrap unit identity | Exact generated-error policy | Alignment/data validation in preflight | Retained input field | Confidence algorithm sibling |
| `categories.order.closed-selection` | Explicit policy; retained category-order vocabulary | Resolved Plot | Construction plus identity/order preflight | Retained local policy | Three control records |
| `categories.order.primary-control` | Explicit policy; retained primary order | Resolved Plot | Family and data-dependent order preflight | Retained local control | Closed selection; primary-family validator |
| `categories.order.heatmap-x-control` | Explicit policy; retained heatmap x order | Resolved Plot | Heatmap and data-dependent order preflight | Retained local control | Closed selection; axis-family validator |
| `categories.order.heatmap-y-control` | Explicit policy; retained heatmap y order | Resolved Plot | Heatmap and data-dependent order preflight | Retained local control | Closed selection; axis-family validator |
| `color-scales.categorical.exact-map-control` | Plot-local data scale; approved `CategoricalColorMap` | Plot-local scale before preflight | Construction key validity; preflight completeness | Replaced by approved exact map | T5d domain/mapping counterpart |
| `intervals.coefficient.presence` | Explicit policy; retained coefficient interval presence | Plot input | Construction and interval/data preflight | Retained input surface | None |
| `typography.significance.font-size-default` | Theme component; `TypographyTheme` significance text role | Resolved Plot/facet before optional exact style | Theme construction: positive size and valid line height; glyph/fit preflight | T6 `TypographyTheme` field | Exact override `annotations.significance.font-size`; T2a2 duplicate `typography.significance.font-size-default-copy`; T2f measurement counterpart |
| `annotations.significance.font-size` | Exact element style; significance mark text-size override | Exact significance style after theme resolution | Construction: positive | Replaced by optional exact-style field | Theme default; weight/gap exact siblings; T2f measurement counterpart |
| `typography.significance.weight-default` | Theme component; `TypographyTheme` significance text role | Resolved Plot/facet before optional exact style | Closed validated text weight; resolved-face measurement | T6 `TypographyTheme` field | Exact override `annotations.significance.weight` |
| `annotations.significance.weight` | Exact element style; significance mark weight override | Exact significance style after theme resolution | Closed construction; resolved-face measurement uses weight | Replaced by optional exact-style field | Theme default; font-size/gap exact siblings |
| `annotations.significance.gap-default` | Theme component; `AnnotationTheme` significance clearance | Resolved Plot/facet before optional exact style | Theme construction: finite nonnegative; annotation-fit preflight | T6 `AnnotationTheme` field | Exact override `annotations.significance.gap`; T2a2 duplicate `annotations.significance.label-gap-default-copy` |
| `annotations.significance.gap` | Exact element style; significance mark clearance override | Exact significance style after theme resolution | Construction: finite nonnegative | Replaced by optional exact-style field | Theme default; font-size/weight exact siblings |
| `regression.band.presence-claim` | Explicit policy; retained generated-band presence/claim | Resolved Plot | Closed construction and Regression family preflight | Retained local control | T2e generated-band style counterpart |
| `qq.reference-line.policy` | Explicit policy; retained Q-Q reference selection | Resolved Plot | Closed construction and Qq family surface | Retained local control | T2e line-style counterpart |
| `line-path.color-legend.policy` | Explicit policy; retained path hue/legend behavior | Resolved Plot | LinePath family and hue compatibility preflight | Retained local control | Hue requirement validator; categorical scale |
| `marks.point.closed-shape` | Exact element style; retained closed point-shape vocabulary | Exact element | Closed construction | Retained local style vocabulary | Interaction point-shape field |
| `marks.line.closed-pattern` | Exact element style; retained closed line-pattern vocabulary | Exact element | Closed construction; private dash conversion | Retained local style vocabulary | Interaction/guide pattern fields |
| `interaction.styles.group-selector` | Exact element style; retained exact group selector | Exact interaction mark | Construction rejects empty, surrounding-whitespace, or control-bearing identities; preflight requires exactly one style for every observed group | Retained local style field with corrected validation | Shape/pattern siblings |
| `interaction.styles.point-shape` | Exact element style; retained interaction point shape | Exact interaction mark | Closed construction | Retained local style field | Group selector; point vocabulary |
| `interaction.styles.line-pattern` | Exact element style; retained interaction connector pattern | Exact interaction mark | Closed construction | Retained local style field | Group selector; line vocabulary |
| `estimates.status.presence` | Explicit policy; retained unavailable-estimate content | Resolved Plot | Alignment and exact family preflight | Retained local control | Family validator |
| `axes.forest.scale-policy` | Explicit policy; retained forest numeric-scale selection | Resolved Plot | Closed construction; positive-domain preflight | Retained local control | Forest reference and exact ticks |
| `facets.layout.current-policy` | Explicit policy; retained generic-facet layout | Facet inherited from Plot | Construction and complete facet preflight | Retained local control | Facet capability and Canvas incompatibility |
| `facets.scales.current-policy` | Explicit policy; retained facet range-sharing behavior | Facet inherited from Plot | Closed construction and facet planning | Retained local control | Canvas x-range groups are independent |
| `guides.exact.color` | Exact element style; retained guide stroke paint | Exact guide | `Color` construction | Replaced exact-style paint field | Width/pattern siblings; T6 RGBA carrier |
| `guides.exact.width` | Exact element style; retained guide stroke width | Exact guide | Construction: finite positive | Retained exact-style field | Color/pattern siblings |
| `guides.exact.pattern` | Exact element style; retained guide line pattern | Exact guide | Closed construction | Retained exact-style field | Color/width siblings |
| `guides.geometry.presence-orientation` | Explicit policy; retained ordered guide presence/orientation | Resolved Plot then exact guide | Family and axis compatibility preflight | Retained local control | Three guide gate records |
| `layers.point.presence-coordinates` | Explicit policy; retained ordered supplemental-layer presence | Resolved Plot then exact layer | Construction, alignment, family and geometry preflight | Retained local control | Point-layer style/content siblings |
| `layers.point.annotation-labels` | Explicit policy; retained point annotation content | Exact point layer | Alignment/content preflight | Retained layer field | Presence parent |
| `layers.point.legend-identity` | Explicit policy; retained layer legend content/presence | Exact point layer | Identity and shared-legend preflight | Retained layer field | Presence parent; T2d legend counterpart |
| `layers.point.exact-color` | Exact element style; unnamed supplemental-point paint role | Exact point layer | `Color` construction | Replaced exact-style field | Automatic color counterpart; T6 RGBA carrier |
| `layers.point.default-radius` | Theme component; `MarkTheme` supplemental point radius | Resolved Plot theme | Theme construction: positive finite | T6 theme component field | Exact radius override |
| `layers.point.exact-radius` | Exact element style; retained point-layer radius | Exact point layer | Construction: positive | Retained exact-style field | Theme default counterpart |
| `layers.point.default-opacity` | Theme component; `MarkTheme` supplemental point alpha | Resolved Plot theme | Theme construction: valid alpha | T6 theme component field | Exact opacity override |
| `layers.point.exact-opacity` | Exact element style; unnamed complete point-layer alpha role | Exact point layer | `Color`/alpha construction; no multiplication | Replaced exact-style field | Theme default; T6 alpha semantics |
| `layers.bar-point.presence-positioning` | Explicit policy; retained ordered bar-point-layer presence | Resolved Plot then exact layer | Construction, family and positioned-data preflight | Retained local control | Exact style siblings |
| `layers.bar-point.exact-color` | Exact element style; unnamed bar-point paint role | Exact bar-point layer | `Color` construction | Replaced exact-style field | Radius/alpha siblings |
| `layers.bar-point.exact-radius` | Exact element style; retained bar-point radius | Exact bar-point layer | Construction: positive | Retained exact-style field | Color/alpha siblings |
| `layers.bar-point.opacity` | Exact element style; unnamed complete bar-point alpha role | Exact bar-point layer | Valid alpha including opaque endpoint | Replaced exact-style field | Color/radius siblings; T6 alpha semantics |
| `labels.inferred.mode-selection` | Explicit policy; retained inferred-label mode | Resolved Plot or Canvas replacement | Closed construction | Retained local policy | Alias precedence and humanization siblings |
| `labels.inferred.alias-insertion` | Explicit policy; retained singular alias mutation | Policy construction | Exact strings are stored without a standalone lexical contract; use-site preflight rejects display collisions only | Retained local control | Bulk extension sibling |
| `labels.inferred.aliases-bulk-extension` | Explicit policy; retained bulk alias mutation | Policy construction | Deterministic iterator insertion | Retained local control | Singular insertion sibling |
| `labels.inferred.alias-before-mode-precedence` | Renderer invariant; private deterministic label resolution | Resolved Plot labels | Alias first, then one closed fallback mode | Private-only behavior | Mode and alias records |
| `labels.inferred.display-collision-failure` | Renderer invariant; private display-domain uniqueness | Plot/Canvas preflight | Reject distinct raw values resolving to one display label | Private-only behavior | Alias/mode records |
| `labels.inferred.humanization-rule` | Renderer invariant; private fixed humanization algorithm | Resolved Plot labels | Deterministic transformation; no locale fallback | Private-only behavior | Mode selection |
| `geometry.raincloud.dimension-category` | Explicit policy; retained geometry semantic-identity field | Exact caller geometry | Construction nonempty validation | Retained geometry field | Shared dimension validator |
| `geometry.raincloud.dimension-value` | Explicit policy; retained geometry semantic-identity field | Exact caller geometry | Construction nonempty validation | Retained geometry field | Shared dimension validator |
| `geometry.raincloud.dimension-hue` | Explicit policy; retained optional geometry hue identity | Exact caller geometry and Plot-local scale | Construction plus hue-schema preflight | Retained geometry field | Hue consistency invariant |
| `geometry.histogram.dimension-value` | Explicit policy; retained geometry semantic-identity field | Exact caller geometry | Construction nonempty validation | Retained geometry field | Shared dimension validator |
| `geometry.histogram.dimension-density` | Explicit policy; retained geometry semantic-identity field | Exact caller geometry | Construction nonempty validation | Retained geometry field | Shared dimension validator |
| `geometry.dimensions.nonempty-validation` | Renderer invariant; private semantic-dimension validity | Geometry construction/preflight | Reject empty, surrounding-whitespace, or control-bearing identities consistently | Private-only behavior | Five dimension identity records |
| `geometry.raincloud.maximum-groups` | Renderer invariant; private raincloud resource ceiling | Geometry preflight | At most 512 groups | Private-only behavior | Other raincloud ceilings |
| `geometry.raincloud.maximum-density-points-per-group` | Renderer invariant; private raincloud resource ceiling | Geometry preflight | At most 2,048 density points per group | Private-only behavior | Density topology records |
| `geometry.raincloud.maximum-density-points-total` | Renderer invariant; private raincloud resource ceiling | Geometry preflight | Checked aggregate at most 1,000,000 | Private-only behavior | Per-group ceiling |
| `geometry.raincloud.maximum-raw-points-total` | Renderer invariant; private raincloud resource ceiling | Geometry preflight | Checked aggregate at most 1,000,000 | Private-only behavior | Raw-point topology records |
| `geometry.histogram.maximum-bins` | Renderer invariant; private histogram resource ceiling | Geometry preflight | At most 1,000,000 bins | Private-only behavior | Bin topology records |
| `geometry.histogram.maximum-kde-points` | Renderer invariant; private histogram resource ceiling | Geometry preflight | At most 2,048 KDE points | Private-only behavior | KDE topology records |
| `geometry.histogram.maximum-rugs` | Renderer invariant; private histogram resource ceiling | Geometry preflight | At most 1,000,000 rugs | Private-only behavior | Rug topology records |
| `geometry.raincloud.groups-present` | Renderer invariant; private raincloud topology | Geometry preflight | At least one group | Private-only behavior | Group ceiling |
| `geometry.raincloud.hue-presence-consistency` | Renderer invariant; private raincloud hue schema | Geometry preflight before scale resolution | Dimension and every group hue all present or all absent | Private-only behavior | Hue identity field |
| `geometry.raincloud.group-category-nonempty` | Renderer invariant; private group-identity validity | Geometry preflight | Nonempty, trimmed, control-free category | Private-only behavior | Unique identity invariant |
| `geometry.raincloud.group-hue-nonempty` | Renderer invariant; private group-identity validity | Geometry preflight | Present hue nonempty, trimmed, control-free | Private-only behavior | Hue consistency invariant |
| `geometry.raincloud.group-identity-unique` | Renderer invariant; private raincloud identity topology | Geometry preflight | Exact category/hue pairs unique | Private-only behavior | Category/hue identity records |
| `geometry.raincloud.density-minimum-points` | Renderer invariant; private density topology | Geometry preflight | At least two points per group | Private-only behavior | Per-group maximum |
| `geometry.raincloud.density-point-domain` | Renderer invariant; private density coordinate validity | Geometry preflight | Finite y; normalized width in [0,1] | Private-only behavior | Order/positive-width siblings |
| `geometry.raincloud.density-y-order` | Renderer invariant; private density topology | Geometry preflight | Strictly increasing y | Private-only behavior | Point-domain sibling |
| `geometry.raincloud.density-positive-width` | Renderer invariant; private minimum visible geometry | Geometry preflight | At least one positive width | Private-only behavior | Point-domain sibling |
| `geometry.raincloud.summary-order` | Renderer invariant; private summary topology | Geometry preflight | Finite q1 <= median <= q3 | Private-only behavior | Support sibling |
| `geometry.raincloud.summary-within-support` | Renderer invariant; private summary/body compatibility | Geometry preflight | q1/q3 within density support | Private-only behavior | Summary-order sibling |
| `geometry.raincloud.raw-points-present` | Renderer invariant; private raw-strip topology | Geometry preflight | At least one raw point per group | Private-only behavior | Raw ceilings/domain |
| `geometry.raincloud.raw-point-domain` | Renderer invariant; private raw-point validity | Geometry preflight | Finite y; normalized strip in [0,1] | Private-only behavior | Support sibling |
| `geometry.raincloud.raw-point-within-support` | Renderer invariant; private raw/body compatibility | Geometry preflight | Raw y within density support | Private-only behavior | Raw-domain sibling |
| `geometry.histogram.bins-present` | Renderer invariant; private histogram topology | Geometry preflight | At least one bin | Private-only behavior | Bin ceiling/domain |
| `geometry.histogram.bin-domain` | Renderer invariant; private bin geometry validity | Geometry preflight | Finite lower < upper; finite nonnegative density | Private-only behavior | Contiguity/positivity siblings |
| `geometry.histogram.bins-contiguous` | Renderer invariant; private bin topology | Geometry preflight | Strict source order and exact shared edges | Private-only behavior | Bin-domain sibling |
| `geometry.histogram.positive-bin-density` | Renderer invariant; private minimum visible geometry | Geometry preflight | At least one positive density | Private-only behavior | Bin-domain sibling |
| `geometry.histogram.kde-minimum-points` | Renderer invariant; private KDE topology | Geometry preflight | At least two points | Private-only behavior | KDE ceiling |
| `geometry.histogram.kde-point-domain` | Renderer invariant; private KDE coordinate validity | Geometry preflight | Finite x and finite nonnegative y | Private-only behavior | Support/order/positivity siblings |
| `geometry.histogram.kde-within-bin-support` | Renderer invariant; private KDE/bin compatibility | Geometry preflight | Every x in inclusive bin support | Private-only behavior | KDE-domain sibling |
| `geometry.histogram.kde-x-order` | Renderer invariant; private KDE topology | Geometry preflight | Strictly increasing x | Private-only behavior | KDE-domain sibling |
| `geometry.histogram.positive-kde-density` | Renderer invariant; private minimum visible geometry | Geometry preflight | At least one positive y | Private-only behavior | KDE-domain sibling |
| `geometry.histogram.rugs-present` | Renderer invariant; private rug topology | Geometry preflight | At least one segment | Private-only behavior | Rug ceiling/domain |
| `geometry.histogram.rug-domain` | Renderer invariant; private rug geometry validity | Geometry preflight | Finite x and 0 <= baseline < top | Private-only behavior | Support sibling |
| `geometry.histogram.rug-within-bin-support` | Renderer invariant; private rug/bin compatibility | Geometry preflight | Every x in inclusive bin support | Private-only behavior | Rug-domain sibling |
| `capabilities.hue.family-matrix` | Explicit policy; retained sealed `Hue` availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Erased validator `validation.channels.hue-erased-family` |
| `capabilities.group.family-matrix` | Explicit policy; retained sealed `Group` availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Erased validator `validation.channels.group-erased-family` |
| `capabilities.series.family-matrix` | Explicit policy; retained sealed `Series` availability | Public compile-time Plot family | Exact compile-pass/fail matrix plus geometry preflight | Retained capability surface | Raincloud compatibility validator |
| `capabilities.continuous-color.family-matrix` | Explicit policy; retained sealed continuous-color availability | Public compile-time Plot family | Exact Heatmap/Beeswarm matrix | Retained capability surface | Continuous title/domain controls |
| `capabilities.numeric-x.family-matrix` | Explicit policy; retained sealed `NumericX` availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Runtime supplemental x matrix |
| `capabilities.numeric-y.family-matrix` | Explicit policy; retained sealed `NumericY` availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Runtime supplemental y matrix |
| `capabilities.guides.family-matrix` | Explicit policy; retained sealed `Guides` availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Three runtime guide gates |
| `capabilities.heatmap-options.family-matrix` | Explicit policy; retained sealed Heatmap-only availability | Public compile-time Plot family | Exact compile-pass/fail singleton | Retained capability surface | Heatmap controls |
| `capabilities.category-order.family-matrix` | Explicit policy; retained sealed primary-order availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Erased primary-order validator |
| `capabilities.signed-bars.family-matrix` | Explicit policy; retained sealed sign-color availability | Public compile-time Plot family | Exact compile-pass/fail singleton | Retained capability surface | Sign-color control |
| `capabilities.zero-line.family-matrix` | Explicit policy; retained sealed zero-line availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Zero-line control |
| `capabilities.estimate-status.family-matrix` | Explicit policy; retained sealed estimate-status availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Erased family validator |
| `capabilities.markers.family-matrix` | Explicit policy; retained sealed marker availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Marker control and interaction validators |
| `capabilities.coefficient-options.family-matrix` | Explicit policy; retained sealed coefficient-option availability | Public compile-time Plot family | Exact compile-pass/fail singleton | Retained capability surface | Estimate-label control |
| `capabilities.categorical-colors.family-matrix` | Explicit policy; generalized exact-map availability for every categorical data-color consumer | Public compile-time Plot family | Exact 15-family matrix: Scatter, Density, Line, Trajectory, LinePath, Interaction, Marginal, Matrix, Bar, HorizontalBar, StackedBar, StackedHorizontalBar, Boxplot, Violin, Raincloud | Replaced scale capability surface | Exact categorical map record; union of hue and positioned-series color semantics |
| `capabilities.panel-text.family-matrix` | Explicit policy; retained sealed panel-text availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Runtime facet/matrix validators |
| `capabilities.point-outline.family-matrix` | Explicit policy; retained sealed outline-policy availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Outline visibility control |
| `capabilities.significance-style.family-matrix` | Explicit policy; retained sealed significance-style availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Three exact-style fields |
| `capabilities.significance-attached.family-matrix` | Explicit policy; retained sealed Attached availability | Public compile-time Plot family/placement | Exact compile-pass/fail matrix | Retained capability surface | Erased support and Pairwise sibling |
| `capabilities.significance-pairwise.family-matrix` | Explicit policy; retained sealed Pairwise availability | Public compile-time Plot family/placement | Exact compile-pass/fail matrix | Retained capability surface | Erased support and Attached sibling |
| `capabilities.legend.family-matrix` | Explicit policy; retained sealed legend-policy availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Plot legend policy |
| `capabilities.raw-overlay.family-matrix` | Explicit policy; retained sealed raw-overlay availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Raw-overlay control |
| `capabilities.facet.family-matrix` | Explicit policy; retained sealed generic-facet availability | Public compile-time Plot family | Exact compile-pass/fail matrix | Retained capability surface | Facet layout and runtime incompatibility |
| `channels.hue.optional-control` | Explicit policy; retained optional hue channel | Resolved Plot | Compile-time capability, alignment, erased-family and scale preflight | Retained local control | Hue capability and erased validator |
| `channels.group.optional-control` | Explicit policy; retained optional group channel | Resolved Plot | Compile-time capability, alignment and erased-family preflight | Retained local control | Group capability/validators |
| `channels.series.optional-control` | Explicit policy; retained optional positioned-series channel | Resolved Plot | Compile-time capability and geometry compatibility preflight | Retained local control | Series capability |
| `legend.continuous.title-content` | Explicit policy; retained color-guide semantic title | Resolved Plot | Nonempty/trimmed/control-free construction; family preflight | Retained local control | Continuous-color capability and title validator |
| `heatmap.highlight.presence-values` | Explicit policy; retained highlighted-cell mask | Resolved Plot | Alignment and Heatmap-only preflight | Retained local control | Heatmap capability |
| `annotations.coefficient.estimate-label-presence` | Explicit policy; retained estimate-label presence | Resolved Plot | Coefficient-only and point-only preflight | Retained local control | Numeric format records |
| `content.plot.title` | Explicit policy; retained Plot title content | Resolved Plot | Nonempty, trimmed, control-free construction | Retained local control | T2c typography measurement/draw |
| `content.plot.subtitle` | Explicit policy; retained Plot subtitle content | Resolved Plot | Nonempty, trimmed, control-free construction | Retained local control | T2c typography measurement/draw |
| `content.plot.caption` | Explicit policy; retained Plot caption content | Resolved Plot | Nonempty, trimmed, control-free construction | Retained local control | T2c typography measurement/draw |
| `content.axis.x-title` | Explicit policy; retained exact x-title content | Resolved Plot | Nonempty, trimmed, control-free construction | Retained local control | Inferred-label and T2c axis counterparts |
| `content.axis.y-title` | Explicit policy; retained exact y-title content | Resolved Plot | Nonempty, trimmed, control-free construction | Retained local control | Inferred-label and T2c axis counterparts |
| `axes.x.exact-ticks-control` | Explicit policy; retained exact x-tick content | Resolved Plot | Construction plus numeric-axis/range/collision preflight | Retained local control | Numeric-x capability and runtime gate |
| `axes.y.exact-ticks-control` | Explicit policy; retained exact y-tick content | Resolved Plot | Construction plus numeric-axis/range/collision preflight | Retained local control | Numeric-y capability and runtime gate |
| `axes.x.explicit-limits-control` | Explicit policy; retained numeric x-domain limits | Resolved Plot | Finite strict range and axis-kind preflight | Retained local control | Numeric-x capability |
| `axes.y.explicit-limits-control` | Explicit policy; retained numeric y-domain limits | Resolved Plot | Finite strict range and axis-kind preflight | Retained local control | Numeric-y capability |
| `bars.sign-color.presence` | Explicit policy; retained sign-color behavior | Resolved Plot then data scale | HorizontalBar capability and value-domain preflight | Retained local control | SignedBars capability; T5d scale mapping |
| `references.zero-line.presence` | Explicit policy; retained zero-reference presence | Resolved Plot | Compile-time family and representability preflight | Retained local control | ZeroLine capability; T2e style counterpart |
| `marks.estimate-marker.visibility` | Explicit policy; retained marker presence | Resolved Plot | Compile-time family and interaction intelligibility preflight | Retained local control | Markers capability and interaction validators |
| `marks.point-outline.visibility` | Explicit policy; retained point-outline presence | Resolved Plot | Compile-time family preflight | Retained local control | PointOutline capability; MarkTheme outline appearance |
| `annotations.significance.presence` | Explicit policy; retained ordered significance content | Resolved Plot | Placement capability, annotation and facet preflight | Retained local control | Attached/Pairwise matrices and erased support |
| `output.plot.pixel-size` | Explicit policy; retained Plot output dimensions | Plot render owner | Construction or preflight positive/minimum/limit checks | Retained local control | T2a scene/output dimensions |
| `color-scales.continuous.heatmap-domain` | Plot-local data scale; `ContinuousColorScale` exact domain | Plot-local scale before preflight | Finite ordered domain; diverging center included | Replaced approved scale field | Heatmap policy; T5d mapping |
| `color-scales.continuous.beeswarm-domain` | Plot-local data scale; `ContinuousColorScale` exact domain | Plot-local scale before preflight | Finite ordered domain | Replaced approved scale field | T5d mapping |
| `intervals.line.band-presence` | Explicit policy; retained caller-band presence | Resolved Plot | Line family, alignment and bounds preflight | Retained local control | T2e band exact style/geometry |
| `references.coefficient.presence-value` | Explicit policy; retained coefficient reference presence/value | Resolved Plot | Coefficient family and finite representability preflight | Retained local control | T2e reference style |
| `references.forest.required-value` | Explicit policy; retained required forest reference value | Plot input | Finite and scale-domain preflight | Retained constructor argument | Forest scale and T2e reference style |
| `interaction.connectors.visibility` | Explicit policy; retained connector presence | Resolved Plot | Interaction family and intelligibility compatibility preflight | Retained local control | Markers/style validators |
| `histogram.geometry.density-stat-precedence` | Renderer invariant; private precomputed-geometry policy precedence | Plot construction/preflight | One canonical input-kind normalization; incompatible later replacement rejects | Private-only behavior | Binning/stat/composite validators |
| `canvas.category-labels.sharing-policy` | Explicit policy; retained Canvas category-label sharing | Canvas root | Closed construction and complete row compatibility preflight | Retained Canvas control | Two shared-category validators |
| `canvas.text-panel.title-content` | Explicit policy; retained caller text-panel title | Exact Canvas cell | Construction lexical validation; layout-fit preflight | Retained `TextPanel` field | Body and lexical siblings |
| `canvas.text-panel.body-content` | Explicit policy; retained caller text-panel body | Exact Canvas cell | Construction lexical validation; layout-fit preflight | Retained `TextPanel` field | Title and lexical siblings |
| `canvas.text-panel.lexical-validation` | Renderer invariant; private text-panel lexical validity | TextPanel construction | Both values nonempty, trimmed, control-free | Private-only behavior | Title/body records |
| `canvas.grid.cardinality` | Explicit policy; retained Canvas grid shape | Canvas root construction | Positive rows/columns and checked product | Retained constructor arguments | Cell-range records |
| `canvas.plot-panel.cell-range` | Renderer invariant; private Plot-cell address validity | Canvas mutation | Row/column inside fixed grid | Private-only behavior | Grid cardinality |
| `canvas.text-panel.cell-range` | Renderer invariant; private text-cell address validity | Canvas mutation | Row/column inside fixed grid | Private-only behavior | Grid cardinality |
| `canvas.cell.single-occupancy` | Renderer invariant; private Canvas cell occupancy | Canvas mutation | Reject second Plot/TextPanel in one cell | Private-only behavior | Both cell-range records |
| `canvas.legend.shared-gap` | Theme component; `LegendTheme` shared-legend spacing | Canvas root theme | Theme construction: finite nonnegative | T6 theme component field | Canvas legend policy; current explicit override removed |
| `canvas.headers.column-content-range` | Explicit policy; retained column-header content/span | Canvas root | Construction range/lexical/overlap validation | Retained Canvas control | `CompositionTheme` header appearance |
| `canvas.headers.row-content-range` | Explicit policy; retained row-header content/span | Canvas root | Construction range/lexical/overlap validation | Retained Canvas control | `CompositionTheme` header appearance |
| `canvas.headers.row-title-content` | Explicit policy; retained row-title content/presence | Canvas root | Construction plus title-ownership compatibility preflight | Retained Canvas control | `CompositionTheme` row-title appearance |
| `canvas.x-range-group.cell-range` | Renderer invariant; private shared-range address validity | Canvas mutation | Every cell inside grid | Private-only behavior | Group uniqueness/nonempty siblings |
| `canvas.x-range-group.unique-membership` | Renderer invariant; private shared-range ownership | Canvas mutation | A panel belongs to at most one group | Private-only behavior | Cell-range/nonempty siblings |
| `canvas.x-range-group.nonempty` | Renderer invariant; private shared-range cardinality | Canvas mutation | At least one panel per group | Private-only behavior | Cell-range/uniqueness siblings |
| `canvas.output.pixel-size` | Explicit policy; retained Canvas output dimensions | Canvas render owner | Preflight minimum and scene resource limits | Retained Canvas control | T2a scene/output dimensions |
| `content.canvas.title` | Explicit policy; retained Canvas title content | Canvas root | Nonempty, trimmed, control-free construction | Retained Canvas control | Canvas `TypographyTheme` title role |
| `content.canvas.subtitle` | Explicit policy; retained Canvas subtitle content | Canvas root | Nonempty, trimmed, control-free construction | Retained Canvas control | Canvas `TypographyTheme` subtitle role |
| `content.canvas.caption` | Explicit policy; retained Canvas caption content | Canvas root | Nonempty, trimmed, control-free construction | Retained Canvas control | Canvas `TypographyTheme` caption role |
| `canvas.labels.override-policy` | Explicit policy; retained Canvas-wide label replacement | Canvas root before Plot preflight | Complete policy construction and display-domain preflight | Retained Canvas control | Plot labels mode/alias records |
| `validation.canvas.minimum-size` | Renderer invariant; private Canvas representability | Canvas preflight | At least 160x120 before measurement | Private-only behavior | Canvas pixel-size policy |
| `validation.canvas.nonempty-content` | Renderer invariant; private Canvas content presence | Canvas preflight | At least one Plot or TextPanel | Private-only behavior | Grid/occupancy records |
| `validation.canvas.shared-gap-requires-shared-legend` | Renderer invariant; private legend-policy compatibility | Canvas preflight | Gap owner and shared policy must agree | Private-only behavior | Shared gap and Canvas legend policy |
| `validation.canvas.faceted-plot-incompatibility` | Renderer invariant; private composition ownership | Canvas preflight | Reject nested generic facet owner | Private-only behavior | Facet capability/layout |
| `validation.canvas.row-title-panel-title-incompatibility` | Renderer invariant; private title-ownership compatibility | Canvas preflight | Reject competing row and panel title/subtitle content | Private-only behavior | Row-title and Plot content records |
| `validation.canvas.shared-category-label-complete-row` | Renderer invariant; private shared-label occupancy | Canvas preflight | Every row has a first-column owner and a Plot in every compared cell | Private-only behavior | Sharing policy; family/axis validators |
| `validation.canvas.shared-category-label-family` | Renderer invariant; private category-sharing family support | Canvas preflight | Exact admitted family matrix | Private-only behavior | Sharing policy |
| `validation.canvas.shared-category-label-axis` | Renderer invariant; private category-sharing axis support | Canvas preflight | Every shared panel has a categorical y-axis | Private-only behavior | Sharing policy; separate T2g domain-equality owner |
| `validation.raincloud-geometry.series-incompatibility` | Renderer invariant; private self-contained geometry compatibility | Plot preflight | Reject external series on geometry input once, before generic channel validation | Private-only behavior | Series control/capability; external-channel sibling must not duplicate the failure |
| `validation.category-order.primary-family` | Renderer invariant; private erased primary-order support | Plot preflight | Exact runtime matrix equals supported current behavior | Private-only behavior | Primary control/capability |
| `validation.category-order.heatmap-axis-family` | Renderer invariant; private erased heatmap-order support | Plot preflight | Non-default x/y orders require Heatmap | Private-only behavior | Heatmap x/y controls |
| `validation.category-order.value-facet-incompatibility` | Renderer invariant; private ordering/facet compatibility | Plot preflight | Reject ambiguous value order under facets | Private-only behavior | Primary order and facet layout |
| `validation.facets.self-managed-family` | Renderer invariant; private facet family compatibility | Plot preflight | Reject generic facets for self-managed families | Private-only behavior | Facet capability |
| `validation.facets.panel-text-incompatibility` | Renderer invariant; private panel ownership compatibility | Plot preflight | Reject panel text under generic facets | Private-only behavior | Panel-text presence/facet layout |
| `validation.significance.facet-owner-requires-facet` | Renderer invariant; private annotation ownership compatibility | Plot preflight | Facet-cell selector requires facet owner | Private-only behavior | Significance presence/facet layout |
| `validation.intervals.ordinary-bar-line-band-incompatibility` | Renderer invariant; private interval-source compatibility | Plot preflight | Ordinary bars consume only bar-specific bounds | Private-only behavior | Bar input and line-band records |
| `validation.intervals.stacked-bar-bounds-incompatibility` | Renderer invariant; private interval-family compatibility | Plot preflight | Stacked bars reject generic bounds | Private-only behavior | Interval bound-family validator |
| `validation.panel-text.matrix-incompatibility` | Renderer invariant; private standard-panel compatibility | Plot preflight | Matrix rejects panel-corner text | Private-only behavior | PanelText capability |
| `validation.panel-text.unique-corners` | Renderer invariant; private annotation collision rule | Plot preflight | At most one annotation per corner | Private-only behavior | Panel-text presence/corner |
| `validation.histogram-geometry.family` | Renderer invariant; private geometry-family compatibility | Plot preflight | Exact Histogram owner | Private-only behavior | Geometry input |
| `validation.histogram-geometry.facet-incompatibility` | Renderer invariant; private self-contained geometry ownership | Plot preflight | Reject facets | Private-only behavior | Facet layout |
| `validation.histogram-geometry.density-stat` | Renderer invariant; private geometry normalization compatibility | Plot preflight | Require density axis | Private-only behavior | Histogram stat/precedence |
| `validation.histogram-geometry.sample-options-incompatibility` | Renderer invariant; private caller-geometry precedence | Plot preflight | Reject sample-derived bins/composite controls | Private-only behavior | Binning/composite records |
| `validation.histogram-geometry.external-channels-incompatibility` | Renderer invariant; private self-contained channel schema | Plot preflight | Reject external x/y/hue/group | Private-only behavior | Geometry dimensions |
| `validation.histogram-geometry.source-rows-incompatibility` | Renderer invariant; private self-contained row schema | Plot preflight | Reject source-row mapping | Private-only behavior | None |
| `validation.raincloud-geometry.family` | Renderer invariant; private geometry-family compatibility | Plot preflight | Exact Raincloud owner | Private-only behavior | Geometry input |
| `validation.raincloud-geometry.facet-incompatibility` | Renderer invariant; private self-contained geometry ownership | Plot preflight | Reject facets | Private-only behavior | Facet layout |
| `validation.raincloud-geometry.external-channels-incompatibility` | Renderer invariant; private self-contained channel schema | Plot preflight | Reject external x/y/hue/group sample channels without duplicating the series-specific failure | Private-only behavior | Geometry dimensions; series-incompatibility sibling |
| `validation.raincloud-geometry.source-rows-incompatibility` | Renderer invariant; private self-contained row schema | Plot preflight | Reject source-row mapping | Private-only behavior | None |
| `validation.raincloud-geometry.significance-incompatibility` | Renderer invariant; private geometry/annotation compatibility | Plot preflight | Reject significance on precomputed geometry | Private-only behavior | Significance presence |
| `validation.line-family.explicit-group-required` | Renderer invariant; private collective-line identity validity | Plot preflight | Trajectory/LinePath require group through one canonical required-group check | Private-only behavior | Group control/capability; deduplicate with erased required-group validator |
| `validation.line-path.hue-style-requires-hue` | Renderer invariant; private path-style compatibility | Plot preflight | Hue style requires LinePath and hue | Private-only behavior | LinePath policy/hue control |
| `validation.channels.hue-erased-family` | Renderer invariant; private erased hue support | Plot preflight | Exact runtime matrix; reject without fallback | Private-only behavior | Hue capability/control |
| `validation.channels.group-erased-family` | Renderer invariant; private erased group support | Plot preflight | Exact runtime matrix; reject without fallback | Private-only behavior | Group capability/control |
| `validation.channels.required-group-erased-family` | Renderer invariant; private required-group support | Plot preflight | Exact collective family matrix through one canonical check | Private-only behavior | Group control; line-family required-group subset |
| `validation.interaction.styles-family-and-marker-compatibility` | Renderer invariant; private exact-style compatibility | Plot preflight | Interaction-only; markers visible | Private-only behavior | Interaction style/marker records |
| `validation.interaction.connectors-family` | Renderer invariant; private connector-policy support | Plot preflight | Interaction-only | Private-only behavior | Connector control |
| `validation.interaction.hidden-connectors-style-incompatibility` | Renderer invariant; private connector/style compatibility | Plot preflight | Hidden connectors reject connector styles | Private-only behavior | Connector and style records |
| `validation.interaction.visible-mark-required` | Renderer invariant; private minimum interaction intelligibility | Plot preflight | Marker or connector remains visible | Private-only behavior | Marker/connector controls |
| `validation.distributions.redundant-hue-incompatibility` | Renderer invariant; private nonredundant scale identity | Plot preflight after raw semantic dimensions resolve | Boxplot/Violin positioned-series color identity differs from category identity | Private-only behavior | Series control and T5d domain; replaces dormant internal-key comparison |
| `validation.legend.continuous-title-family` | Renderer invariant; private continuous-title support | Plot preflight | Require continuous-color family | Private-only behavior | Color-guide title/capability |
| `validation.estimate-status.family` | Renderer invariant; private estimate-status support | Plot preflight | Exact three-family matrix | Private-only behavior | Status control/capability |
| `validation.intervals.bound-family` | Renderer invariant; private interval-bound support | Plot preflight | Exact five-family matrix | Private-only behavior | Interval controls |
| `validation.matrix.variable-family` | Renderer invariant; private matrix-list support | Plot preflight | Stored matrix list requires Matrix | Private-only behavior | Matrix constructor |
| `validation.matrix.external-channels-incompatibility` | Renderer invariant; private Matrix channel schema | Plot preflight | Reject separately configured x/y channels | Private-only behavior | Matrix constructor |
| `validation.matrix.minimum-variables` | Renderer invariant; private minimum Matrix topology | Plot preflight | At least two ordered variables | Private-only behavior | Matrix constructor and layout |
| `validation.matrix.variable-name-nonempty` | Renderer invariant; private Matrix identity validity | Plot preflight | Variable names are nonempty, trimmed, and control-free | Private-only behavior | Variable uniqueness and label resolution |
| `validation.matrix.variable-name-unique` | Renderer invariant; private Matrix identity topology | Plot preflight | Exact validated variable names are unique | Private-only behavior | Variable-name validity and panel layout |
| `validation.trajectory.categorical-x-limits-incompatibility` | Renderer invariant; private axis-kind/limit compatibility | Plot preflight | Categorical trajectory x rejects numeric limits | Private-only behavior | x-limit control |
| `validation.line-path.numeric-x-required` | Renderer invariant; private explicit-path axis validity | Plot preflight | Require numeric x | Private-only behavior | Numeric-x capability |
| `validation.supplemental.numeric-x-family-matrix` | Renderer invariant; private supplemental numeric-x classification | Plot preflight | Closed exact matrix, including data-dependent Trajectory x | Private-only behavior | Public NumericX capability |
| `validation.supplemental.numeric-y-family-matrix` | Renderer invariant; private supplemental numeric-y classification | Plot preflight | Closed exact matrix | Private-only behavior | Public NumericY capability |
| `validation.supplemental.facet-incompatibility` | Renderer invariant; private supplemental/facet compatibility | Plot preflight | Reject any admitted supplemental control under facets | Private-only behavior | Tick/guide/layer controls |
| `validation.axes.x-exact-ticks-require-numeric` | Renderer invariant; private exact-tick axis compatibility | Plot preflight | Require runtime numeric x | Private-only behavior | x-tick control and runtime matrix |
| `validation.axes.y-exact-ticks-require-numeric` | Renderer invariant; private exact-tick axis compatibility | Plot preflight | Require runtime numeric y | Private-only behavior | y-tick control and runtime matrix |
| `validation.guides.horizontal-requires-numeric-y` | Renderer invariant; private guide-axis compatibility | Plot preflight | Require runtime numeric y | Private-only behavior | Guide presence/orientation |
| `validation.guides.vertical-requires-numeric-x` | Renderer invariant; private guide-axis compatibility | Plot preflight | Require runtime numeric x | Private-only behavior | Guide presence/orientation |
| `validation.guides.segment-requires-two-numeric-axes` | Renderer invariant; private guide-axis compatibility | Plot preflight | Require runtime numeric x and y | Private-only behavior | Guide presence/orientation |

### T2b Migration And Evidence

The compact evidence codes below name required migration evidence. They do not
claim that a future implementation check already exists or passed.

| Code | Required visual or contract evidence |
| --- | --- |
| B-V-P | No output change expected; focused public-policy construction, default, setter, preflight, and representative snapshot assertions are required. |
| B-V-C | No output change expected; exact compile-pass and compile-fail family matrices, plus erased-runtime parity where applicable, are required. |
| B-V-G | No output change expected; focused valid geometry plus every independent invalid topology, support, positivity, and resource branch are required. |
| B-V-E | No ordinary valid-output change expected; focused exact-style precedence, validation, legend-glyph, and representative SVG assertions are required. |
| B-V-I | No output change expected; focused private invariant/preflight failure and successful render parity assertions are required. |
| B-V-D | Intentional custom-theme or scale redesign with unchanged built-in semantic intent; focused resolution, precedence, SVG, Canvas, and scale snapshots are required. |
| B-V-F | Intentional earlier or stricter validation correction; focused failure-timing and no-backend-output assertions are required. |

| Property | Disposition and owning phase | Visual evidence |
| --- | --- | --- |
| `bars.spacing.current-bar-spacing` | `retain`, T6 | B-V-P; grouped Bar/HBar spacing snapshots |
| `histogram.binning.current-selection` | `retain`, T6 | B-V-P; Auto/Count/Width geometry assertions |
| `histogram.normalization.current-stat` | `retain`, T6 | B-V-P; Count/Probability/Density axes and bars |
| `histogram.composite.kde-source` | `retain`, T6 | B-V-P; internal/precomputed source routing |
| `histogram.composite.presence` | `retain`, T6 | B-V-P; histogram-only and KDE/rug snapshots |
| `annotations.panel-text.presence-control` | `retain`, T6 | B-V-P; empty/multiple annotation order |
| `annotations.panel-text.corner` | `retain`, T6 | B-V-P; four-corner placement snapshots |
| `annotations.panel-text.content` | `retain`, T6 | B-V-P; exact content plus empty, untrimmed, control, U+2028, and U+2029 failures |
| `marks.raw-overlay.presence-layout` | `retain`, T6 | B-V-P; hidden/centered/jittered distribution snapshots |
| `color-scales.continuous.heatmap-default` | `move`, T5d; public theme connection T6 | B-V-D; built-in sequential default and custom-theme gradient snapshots |
| `color-scales.continuous.heatmap-policy` | `replace`, T5d; public connection T6 | B-V-D; local sequential/diverging replacement, center, and saturation snapshots |
| `heatmap.cells.layout-policy` | `retain`, T6 | B-V-P; Cartesian/lower/upper and diagonal topology |
| `heatmap.labels.fit-policy` | `retain`, T6 | B-V-P; elide/wrap measurement and draw parity |
| `annotations.numeric-label.precision` | `retain`, T6 | B-V-P; 0/12 boundaries and rendered labels |
| `annotations.numeric-label.suffix` | `retain`, T6 | B-V-P; absent/present suffix output and rejection |
| `annotations.heatmap.cell-value-presence` | `retain`, T6 | B-V-P; hidden/value cell annotations |
| `annotations.heatmap.null-row-presence` | `retain`, T6 | B-V-P; zero/multiple ordered null-row annotations |
| `annotations.heatmap.null-row-target` | `retain`, T6 | B-V-P; exact row targeting |
| `annotations.heatmap.null-row-text` | `retain`, T6 | B-V-P; wrapping and exact content |
| `annotations.heatmap.null-row-lexical-validation` | `retain`, T6 | B-V-I; every target/text lexical failure |
| `marks.dumbbell.left-endpoint-identity` | `retain`, T6 | B-V-P; endpoint label/legend identity output |
| `marks.dumbbell.right-endpoint-identity` | `retain`, T6 | B-V-P; endpoint label/legend identity output |
| `marks.dumbbell.endpoint-identities-nonempty` | `retain`, T6 | B-V-I; independent left/right empty failures |
| `marks.dumbbell.endpoint-identities-lexical-validity` | `retain`, T6 | B-V-I; whitespace/control failures |
| `marks.dumbbell.endpoint-identities-distinct` | `retain`, T6 | B-V-I; equal-identity rejection |
| `marks.dumbbell.left-endpoint-color` | `replace`, T6 | B-V-E; exact RGBA left endpoint and legend glyph |
| `marks.dumbbell.right-endpoint-color` | `replace`, T6 | B-V-E; exact RGBA right endpoint and legend glyph |
| `annotations.waterfall.total-label-presence` | `retain`, T6 | B-V-P; absent/present complete pair |
| `annotations.waterfall.base-label` | `retain`, T6 | B-V-P; exact base text snapshot |
| `annotations.waterfall.terminal-label` | `retain`, T6 | B-V-P; exact terminal text snapshot |
| `annotations.waterfall.labels-nonempty` | `retain`, T6 | B-V-I; independent empty failures |
| `annotations.waterfall.labels-lexical-validity` | `retain`, T6 | B-V-I; whitespace/control failures |
| `canvas.legend.composition-policy` | `retain`, T6 | B-V-P; PerPlot/Shared/Hidden Canvas snapshots |
| `legend.plot.visibility-placement-policy` | `retain`, T6 | B-V-P; hidden/automatic/all exact positions |
| `annotations.significance.erased-family-support` | `connect`, T6 | B-V-C; erased matrix equals attached/pairwise public matrices |
| `bars.input.estimate-source-and-bounds` | `retain`, T6 | B-V-P; samples/estimates/precomputed bounds |
| `bars.error.generated-selection` | `retain`, T6 | B-V-P; all five generated-error selections |
| `bars.error.standard-deviation-scale` | `retain`, T6 | B-V-P; independent SD scale geometry |
| `bars.error.standard-error-scale` | `retain`, T6 | B-V-P; independent SE scale geometry |
| `bars.error.spread-scale-validation` | `retain`, T6 | B-V-I; finite-positive boundary failures |
| `bars.error.percentile-interval-coverage` | `retain`, T6 | B-V-P; independent percentile coverage geometry |
| `bars.error.confidence-interval-coverage` | `retain`, T6 | B-V-P; independent CI coverage geometry |
| `bars.error.shared-coverage-default-domain` | `retain`, T6 | B-V-I; 0.95 default and strict-domain failures |
| `bars.error.confidence-algorithm` | `retain`, T6 | B-V-P; bootstrap/Student-t intervals |
| `bars.error.bootstrap-resamples` | `retain`, T6 | B-V-P; count boundaries and deterministic output |
| `bars.error.bootstrap-seed` | `retain`, T6 | B-V-P; same/different seed determinism |
| `bars.error.bootstrap-units` | `retain`, T6 | B-V-P; absent/present multilevel resampling |
| `categories.order.closed-selection` | `retain`, T6 | B-V-P; every order/direction combination |
| `categories.order.primary-control` | `retain`, T6 | B-V-P; supported families and order snapshots |
| `categories.order.heatmap-x-control` | `retain`, T6 | B-V-P; independent x order |
| `categories.order.heatmap-y-control` | `retain`, T6 | B-V-P; independent y order |
| `color-scales.categorical.exact-map-control` | `replace`, T5d; public connection T6 | B-V-D; exact raw-key map, completeness, facets, legends |
| `intervals.coefficient.presence` | `retain`, T6 | B-V-P; point-only and bounded coefficient snapshots |
| `typography.significance.font-size-default` | `move`, T5a; public theme connection T6 | B-V-D; built-in 16 px parity, custom-theme size, and measurement/draw identity |
| `annotations.significance.font-size` | `replace`, T6 | B-V-E; optional exact precedence, measurement/draw identity, and positive failure |
| `typography.significance.weight-default` | `move`, T5a; public theme connection T6 | B-V-D; built-in Normal parity and resolved-face measurement |
| `annotations.significance.weight` | `replace`, T6 | B-V-E; three exact weights in marks and resolved-font measurement |
| `annotations.significance.gap-default` | `move`, T5c; public theme connection T6 | B-V-D; built-in 8 px parity, custom-theme clearance, and planning/draw identity |
| `annotations.significance.gap` | `replace`, T6 | B-V-E; optional exact clearance precedence and invalid values |
| `regression.band.presence-claim` | `retain`, T6 | B-V-P; visible/hidden claim and legend output |
| `qq.reference-line.policy` | `retain`, T6 | B-V-P; quartile/identity/hidden snapshots |
| `line-path.color-legend.policy` | `retain`, T6 | B-V-P; neutral/hue marks and legend output |
| `marks.point.closed-shape` | `retain`, T6 | B-V-E; circle/triangle geometry and legends |
| `marks.line.closed-pattern` | `retain`, T6 | B-V-E; solid/dashed geometry and legends |
| `interaction.styles.group-selector` | `replace`, T6 | B-V-E/B-V-F; lexical construction failures and exact group matching/completeness |
| `interaction.styles.point-shape` | `retain`, T6 | B-V-E; per-group glyph snapshots |
| `interaction.styles.line-pattern` | `retain`, T6 | B-V-E; per-group connector/legend snapshots |
| `estimates.status.presence` | `retain`, T6 | B-V-P; estimated/not-estimable content |
| `axes.forest.scale-policy` | `retain`, T6 | B-V-P; linear/log ticks, ranges, references |
| `facets.layout.current-policy` | `retain`, T6 | B-V-P; row/column/grid/wrap snapshots |
| `facets.scales.current-policy` | `retain`, T6 | B-V-P; fixed/free axis-sharing snapshots |
| `guides.exact.color` | `replace`, T6 | B-V-E; exact RGBA guide stroke |
| `guides.exact.width` | `retain`, T6 | B-V-E; exact width and invalid boundaries |
| `guides.exact.pattern` | `retain`, T6 | B-V-E; solid/dashed guide snapshots |
| `guides.geometry.presence-orientation` | `retain`, T6 | B-V-P; all orientations and append order |
| `layers.point.presence-coordinates` | `retain`, T6 | B-V-P; zero/multiple supplemental layers |
| `layers.point.annotation-labels` | `retain`, T6 | B-V-P; absent/present aligned labels |
| `layers.point.legend-identity` | `retain`, T6 | B-V-P; layer legend and shared compatibility |
| `layers.point.exact-color` | `replace`, T6 | B-V-E; exact RGBA override and slot stability |
| `layers.point.default-radius` | `move`, T5c | B-V-D; resolved theme radius in layout/draw |
| `layers.point.exact-radius` | `retain`, T6 | B-V-E; exact precedence and positive failure |
| `layers.point.default-opacity` | `move`, T5c | B-V-D; resolved theme alpha and built-in parity |
| `layers.point.exact-opacity` | `replace`, T6 | B-V-E; replacement alpha, including opaque endpoint |
| `layers.bar-point.presence-positioning` | `retain`, T6 | B-V-P; Bar/HBar positioning and append order |
| `layers.bar-point.exact-color` | `replace`, T6 | B-V-E; exact RGBA paint |
| `layers.bar-point.exact-radius` | `retain`, T6 | B-V-E; exact radius and positive failure |
| `layers.bar-point.opacity` | `replace`, T6 | B-V-F; complete alpha including corrected opaque endpoint |
| `labels.inferred.mode-selection` | `retain`, T6 | B-V-P; Humanized/Raw labels across consumers |
| `labels.inferred.alias-insertion` | `retain`, T6 | B-V-P; insertion/replacement order |
| `labels.inferred.aliases-bulk-extension` | `retain`, T6 | B-V-P; bulk repeated-key determinism |
| `labels.inferred.alias-before-mode-precedence` | `retain`, T6 | B-V-I; alias wins in both modes |
| `labels.inferred.display-collision-failure` | `retain`, T6 | B-V-I; exact collision rejection before backend |
| `labels.inferred.humanization-rule` | `retain`, T6 | B-V-I; pinned token/case/acronym corpus |
| `geometry.raincloud.dimension-category` | `retain`, T6 | B-V-G; inferred x-title and label resolution |
| `geometry.raincloud.dimension-value` | `retain`, T6 | B-V-G; inferred y-title and label resolution |
| `geometry.raincloud.dimension-hue` | `retain`, T6 | B-V-G; absent/present hue legends and scales |
| `geometry.histogram.dimension-value` | `retain`, T6 | B-V-G; inferred x-title |
| `geometry.histogram.dimension-density` | `retain`, T6 | B-V-G; inferred density title |
| `geometry.dimensions.nonempty-validation` | `replace`, T6 | B-V-F; consistent empty/whitespace/control rejection |
| `geometry.raincloud.maximum-groups` | `retain`, T3 | B-V-G; 512/513 group boundary |
| `geometry.raincloud.maximum-density-points-per-group` | `retain`, T3 | B-V-G; 2,048/2,049 boundary |
| `geometry.raincloud.maximum-density-points-total` | `retain`, T3 | B-V-G; aggregate ceiling and checked overflow |
| `geometry.raincloud.maximum-raw-points-total` | `retain`, T3 | B-V-G; aggregate ceiling and checked overflow |
| `geometry.histogram.maximum-bins` | `retain`, T3 | B-V-G; bin ceiling boundary |
| `geometry.histogram.maximum-kde-points` | `retain`, T3 | B-V-G; KDE ceiling boundary |
| `geometry.histogram.maximum-rugs` | `retain`, T3 | B-V-G; rug ceiling boundary |
| `geometry.raincloud.groups-present` | `retain`, T3 | B-V-G; empty-group failure |
| `geometry.raincloud.hue-presence-consistency` | `retain`, T5d | B-V-G; dimension/group all-or-none cases |
| `geometry.raincloud.group-category-nonempty` | `replace`, T6 | B-V-F; normalized lexical identity failures |
| `geometry.raincloud.group-hue-nonempty` | `replace`, T6 | B-V-F; normalized lexical hue failures |
| `geometry.raincloud.group-identity-unique` | `retain`, T5d | B-V-G; duplicate category/hue failure |
| `geometry.raincloud.density-minimum-points` | `retain`, T3 | B-V-G; zero/one/two-point boundary |
| `geometry.raincloud.density-point-domain` | `retain`, T3 | B-V-G; finite and [0,1] boundaries |
| `geometry.raincloud.density-y-order` | `retain`, T3 | B-V-G; equal/decreasing y failures |
| `geometry.raincloud.density-positive-width` | `retain`, T3 | B-V-G; all-zero width failure |
| `geometry.raincloud.summary-order` | `retain`, T3 | B-V-G; every nonfinite/order branch |
| `geometry.raincloud.summary-within-support` | `retain`, T3 | B-V-G; lower/upper support failures |
| `geometry.raincloud.raw-points-present` | `retain`, T3 | B-V-G; empty raw strip failure |
| `geometry.raincloud.raw-point-domain` | `retain`, T3 | B-V-G; finite and [0,1] boundaries |
| `geometry.raincloud.raw-point-within-support` | `retain`, T3 | B-V-G; both support-edge failures |
| `geometry.histogram.bins-present` | `retain`, T3 | B-V-G; empty bins failure |
| `geometry.histogram.bin-domain` | `retain`, T3 | B-V-G; every finite/order/density branch |
| `geometry.histogram.bins-contiguous` | `retain`, T3 | B-V-G; gap/overlap/order failures |
| `geometry.histogram.positive-bin-density` | `retain`, T3 | B-V-G; all-zero bin failure |
| `geometry.histogram.kde-minimum-points` | `retain`, T3 | B-V-G; zero/one/two-point boundary |
| `geometry.histogram.kde-point-domain` | `retain`, T3 | B-V-G; every finite/nonnegative branch |
| `geometry.histogram.kde-within-bin-support` | `retain`, T3 | B-V-G; both support-edge failures |
| `geometry.histogram.kde-x-order` | `retain`, T3 | B-V-G; equal/decreasing x failures |
| `geometry.histogram.positive-kde-density` | `retain`, T3 | B-V-G; all-zero KDE failure |
| `geometry.histogram.rugs-present` | `retain`, T3 | B-V-G; empty rugs failure |
| `geometry.histogram.rug-domain` | `retain`, T3 | B-V-G; every finite/baseline/top branch |
| `geometry.histogram.rug-within-bin-support` | `retain`, T3 | B-V-G; both support-edge failures |
| `capabilities.hue.family-matrix` | `retain`, T6 | B-V-C; every admitted and representative rejected family |
| `capabilities.group.family-matrix` | `retain`, T6 | B-V-C; every admitted and representative rejected family |
| `capabilities.series.family-matrix` | `retain`, T6 | B-V-C; exact five-family matrix |
| `capabilities.continuous-color.family-matrix` | `retain`, T6 | B-V-C; Heatmap/Beeswarm only |
| `capabilities.numeric-x.family-matrix` | `retain`, T6 | B-V-C; exact 17-family matrix |
| `capabilities.numeric-y.family-matrix` | `retain`, T6 | B-V-C; exact 15-family matrix |
| `capabilities.guides.family-matrix` | `retain`, T6 | B-V-C; exact 23-family matrix |
| `capabilities.heatmap-options.family-matrix` | `retain`, T6 | B-V-C; Heatmap-only controls |
| `capabilities.category-order.family-matrix` | `retain`, T6 | B-V-C; exact five-family matrix |
| `capabilities.signed-bars.family-matrix` | `retain`, T6 | B-V-C; HorizontalBar-only control |
| `capabilities.zero-line.family-matrix` | `retain`, T6 | B-V-C; exact four-family matrix |
| `capabilities.estimate-status.family-matrix` | `retain`, T6 | B-V-C; exact three-family matrix |
| `capabilities.markers.family-matrix` | `retain`, T6 | B-V-C; exact three-family matrix |
| `capabilities.coefficient-options.family-matrix` | `retain`, T6 | B-V-C; Coefficient-only control |
| `capabilities.categorical-colors.family-matrix` | `replace`, T6 | B-V-C/B-V-D; approved exact 15-family hue/positioned-series matrix |
| `capabilities.panel-text.family-matrix` | `retain`, T6 | B-V-C; exact 24-family matrix |
| `capabilities.point-outline.family-matrix` | `retain`, T6 | B-V-C; exact 18-family matrix |
| `capabilities.significance-style.family-matrix` | `retain`, T6 | B-V-C; exact eight-family matrix |
| `capabilities.significance-attached.family-matrix` | `retain`, T6 | B-V-C; exact three-family placement matrix |
| `capabilities.significance-pairwise.family-matrix` | `retain`, T6 | B-V-C; exact seven-family placement matrix |
| `capabilities.legend.family-matrix` | `retain`, T6 | B-V-C; exact 17-family matrix |
| `capabilities.raw-overlay.family-matrix` | `retain`, T6 | B-V-C; Boxplot/Violin only |
| `capabilities.facet.family-matrix` | `retain`, T6 | B-V-C; exact 23-family matrix |
| `channels.hue.optional-control` | `retain`, T6 | B-V-P/B-V-C; absent/present and erased parity |
| `channels.group.optional-control` | `retain`, T6 | B-V-P/B-V-C; absent/present and required-group cases |
| `channels.series.optional-control` | `retain`, T6 | B-V-P/B-V-C; absent/present and raincloud geometry rejection |
| `legend.continuous.title-content` | `retain`, T6 | B-V-F; inferred/exact title and lexical failures |
| `heatmap.highlight.presence-values` | `retain`, T6 | B-V-P; absent/present borders and alignment |
| `annotations.coefficient.estimate-label-presence` | `retain`, T6 | B-V-P; point-only labels and interval suppression |
| `content.plot.title` | `retain`, T6 | B-V-F; content validation and title snapshots |
| `content.plot.subtitle` | `retain`, T6 | B-V-F; content validation and subtitle snapshots |
| `content.plot.caption` | `retain`, T6 | B-V-F; content validation and caption snapshots |
| `content.axis.x-title` | `retain`, T6 | B-V-F; exact/inferred/empty rejection and layout |
| `content.axis.y-title` | `retain`, T6 | B-V-F; exact/inferred/empty rejection and layout |
| `axes.x.exact-ticks-control` | `retain`, T6 | B-V-P; exact labels/positions and automatic replacement |
| `axes.y.exact-ticks-control` | `retain`, T6 | B-V-P; exact labels/positions and automatic replacement |
| `axes.x.explicit-limits-control` | `retain`, T6 | B-V-P; explicit/planned precedence and range failures |
| `axes.y.explicit-limits-control` | `retain`, T6 | B-V-P; explicit/planned precedence and range failures |
| `bars.sign-color.presence` | `connect`, T5d; public control retained T6 | B-V-D; positive/negative scale colors and legends |
| `references.zero-line.presence` | `retain`, T6 | B-V-P; absent/present reference snapshots |
| `marks.estimate-marker.visibility` | `retain`, T6 | B-V-P; visible/hidden marks and legends |
| `marks.point-outline.visibility` | `retain`, T6 | B-V-P; visible/hidden outline and legend glyphs |
| `annotations.significance.presence` | `retain`, T6 | B-V-P/B-V-C; placement append order and family matrix |
| `output.plot.pixel-size` | `retain`, T6 | B-V-P; defaults, exact dimensions, limit failures |
| `color-scales.continuous.heatmap-domain` | `replace`, T5d; public connection T6 | B-V-D; exact/saturated/diverging-domain snapshots |
| `color-scales.continuous.beeswarm-domain` | `replace`, T5d; public connection T6 | B-V-D; exact/saturated-domain snapshots |
| `intervals.line.band-presence` | `retain`, T6 | B-V-P; absent/present band and bounds validation |
| `references.coefficient.presence-value` | `retain`, T6 | B-V-P; absent/present finite reference |
| `references.forest.required-value` | `retain`, T6 | B-V-P; linear/log reference and domain failures |
| `interaction.connectors.visibility` | `retain`, T6 | B-V-P; default/visible/hidden compatibility |
| `histogram.geometry.density-stat-precedence` | `retain`, T6 | B-V-I; constructor/user-call order and preflight failure timing |
| `canvas.category-labels.sharing-policy` | `retain`, T6 | B-V-P; EveryPlot/FirstColumnPerRow rows |
| `canvas.text-panel.title-content` | `retain`, T6 | B-V-P; wrapping/fit and exact title |
| `canvas.text-panel.body-content` | `retain`, T6 | B-V-P; wrapping/fit and exact body |
| `canvas.text-panel.lexical-validation` | `retain`, T6 | B-V-I; independent title/body lexical failures |
| `canvas.grid.cardinality` | `retain`, T6 | B-V-P; positive grid and overflow failure |
| `canvas.plot-panel.cell-range` | `retain`, T6 | B-V-I; every row/column boundary |
| `canvas.text-panel.cell-range` | `retain`, T6 | B-V-I; every row/column boundary |
| `canvas.cell.single-occupancy` | `retain`, T6 | B-V-I; Plot/TextPanel collision permutations |
| `canvas.legend.shared-gap` | `move`, T5b; public theme connection T6 | B-V-D; resolved LegendTheme spacing and Canvas snapshots |
| `canvas.headers.column-content-range` | `retain`, T6 | B-V-P; span/range/overlap/content and layout |
| `canvas.headers.row-content-range` | `retain`, T6 | B-V-P; span/range/overlap/content and layout |
| `canvas.headers.row-title-content` | `retain`, T6 | B-V-P; one-per-row content/wrapping/layout |
| `canvas.x-range-group.cell-range` | `retain`, T6 | B-V-I; in/out-of-grid group cells |
| `canvas.x-range-group.unique-membership` | `retain`, T6 | B-V-I; duplicate within/across groups |
| `canvas.x-range-group.nonempty` | `retain`, T6 | B-V-I; empty/nonempty groups |
| `canvas.output.pixel-size` | `retain`, T6 | B-V-P; default/exact/minimum/limit dimensions |
| `content.canvas.title` | `retain`, T6 | B-V-F; lexical validation and Canvas title snapshots |
| `content.canvas.subtitle` | `retain`, T6 | B-V-F; lexical validation and Canvas subtitle snapshots |
| `content.canvas.caption` | `retain`, T6 | B-V-F; lexical validation and Canvas caption snapshots |
| `canvas.labels.override-policy` | `retain`, T6 | B-V-P; absent/replacement label policy across panels |
| `validation.canvas.minimum-size` | `retain`, T3 | B-V-I; 159/119 failures and exact minima |
| `validation.canvas.nonempty-content` | `retain`, T3 | B-V-I; empty versus Plot/TextPanel content |
| `validation.canvas.shared-gap-requires-shared-legend` | `replace`, T5b | B-V-D; theme-owned gap removes invalid local pairing |
| `validation.canvas.faceted-plot-incompatibility` | `retain`, T5b | B-V-I; nested facet rejection before draw |
| `validation.canvas.row-title-panel-title-incompatibility` | `retain`, T5b | B-V-I; title/subtitle conflict permutations |
| `validation.canvas.shared-category-label-complete-row` | `retain`, T5b | B-V-I; missing first and later cells in every row position |
| `validation.canvas.shared-category-label-family` | `retain`, T5b | B-V-I; exact admitted/rejected families |
| `validation.canvas.shared-category-label-axis` | `retain`, T5b | B-V-I; categorical versus noncategorical y-axis cases |
| `validation.raincloud-geometry.series-incompatibility` | `retain`, T6 | B-V-I; samples accept and geometry rejects series |
| `validation.category-order.primary-family` | `retain`, T6 | B-V-C/B-V-I; compile/runtime matrix parity |
| `validation.category-order.heatmap-axis-family` | `retain`, T6 | B-V-C/B-V-I; Heatmap-only x/y controls |
| `validation.category-order.value-facet-incompatibility` | `retain`, T6 | B-V-I; faceted value-order rejection |
| `validation.facets.self-managed-family` | `retain`, T6 | B-V-C/B-V-I; Matrix/Marginal rejection |
| `validation.facets.panel-text-incompatibility` | `retain`, T6 | B-V-I; panel-text/facet rejection |
| `validation.significance.facet-owner-requires-facet` | `retain`, T6 | B-V-I; facet selector ownership cases |
| `validation.intervals.ordinary-bar-line-band-incompatibility` | `retain`, T6 | B-V-I; generic versus bar-bound source |
| `validation.intervals.stacked-bar-bounds-incompatibility` | `retain`, T6 | B-V-I; both stacked orientations |
| `validation.panel-text.matrix-incompatibility` | `retain`, T6 | B-V-C/B-V-I; Matrix compile/runtime exclusion |
| `validation.panel-text.unique-corners` | `retain`, T6 | B-V-I; each duplicate-corner case |
| `validation.histogram-geometry.family` | `retain`, T6 | B-V-I; erased wrong-family rejection |
| `validation.histogram-geometry.facet-incompatibility` | `retain`, T6 | B-V-I; geometry/facet rejection |
| `validation.histogram-geometry.density-stat` | `retain`, T6 | B-V-I; density requirement and call-order case |
| `validation.histogram-geometry.sample-options-incompatibility` | `retain`, T6 | B-V-I; bins/composite independent failures |
| `validation.histogram-geometry.external-channels-incompatibility` | `retain`, T6 | B-V-I; every external channel branch |
| `validation.histogram-geometry.source-rows-incompatibility` | `retain`, T6 | B-V-I; source-row rejection |
| `validation.raincloud-geometry.family` | `retain`, T6 | B-V-I; erased wrong-family rejection |
| `validation.raincloud-geometry.facet-incompatibility` | `retain`, T6 | B-V-I; geometry/facet rejection |
| `validation.raincloud-geometry.external-channels-incompatibility` | `deduplicate`, T6 | B-V-I; every external channel branch with one series/group failure path |
| `validation.raincloud-geometry.source-rows-incompatibility` | `retain`, T6 | B-V-I; source-row rejection |
| `validation.raincloud-geometry.significance-incompatibility` | `retain`, T6 | B-V-I; annotation rejection |
| `validation.line-family.explicit-group-required` | `deduplicate`, T6 | B-V-I; one Trajectory/LinePath missing-group failure path |
| `validation.line-path.hue-style-requires-hue` | `retain`, T6 | B-V-I; wrong family and missing hue failures |
| `validation.channels.hue-erased-family` | `connect`, T6 | B-V-C; exact public/runtime matrix parity |
| `validation.channels.group-erased-family` | `connect`, T6 | B-V-C; exact public/runtime matrix parity |
| `validation.channels.required-group-erased-family` | `deduplicate`, T6 | B-V-I; exact five-family presence matrix with one failure per family |
| `validation.interaction.styles-family-and-marker-compatibility` | `retain`, T6 | B-V-I; wrong family and hidden-marker failures |
| `validation.interaction.connectors-family` | `retain`, T6 | B-V-C/B-V-I; Interaction-only parity |
| `validation.interaction.hidden-connectors-style-incompatibility` | `retain`, T6 | B-V-I; hidden connector/style rejection |
| `validation.interaction.visible-mark-required` | `retain`, T6 | B-V-I; all marker/connector combinations |
| `validation.distributions.redundant-hue-incompatibility` | `replace`, T5d | B-V-F; raw redundant versus distinct positioned-series identities and pre-backend failure |
| `validation.legend.continuous-title-family` | `retain`, T6 | B-V-C/B-V-I; exact continuous-color matrix |
| `validation.estimate-status.family` | `connect`, T6 | B-V-C; exact public/runtime three-family parity |
| `validation.intervals.bound-family` | `retain`, T6 | B-V-I; exact five-family runtime matrix |
| `validation.matrix.variable-family` | `retain`, T6 | B-V-I; Matrix/non-Matrix erased cases |
| `validation.matrix.external-channels-incompatibility` | `retain`, T6 | B-V-I; independent x/y channel rejection |
| `validation.matrix.minimum-variables` | `retain`, T6 | B-V-I; zero/one/two-variable boundary |
| `validation.matrix.variable-name-nonempty` | `replace`, T6 | B-V-F; empty, surrounding-whitespace, and control-character failures |
| `validation.matrix.variable-name-unique` | `retain`, T6 | B-V-I; duplicate exact identity rejection |
| `validation.trajectory.categorical-x-limits-incompatibility` | `retain`, T6 | B-V-I; categorical/numeric x and limits combinations |
| `validation.line-path.numeric-x-required` | `retain`, T6 | B-V-I; categorical rejection/numeric success |
| `validation.supplemental.numeric-x-family-matrix` | `connect`, T6 | B-V-C; public/runtime matrix including Trajectory |
| `validation.supplemental.numeric-y-family-matrix` | `connect`, T6 | B-V-C; public/runtime matrix parity |
| `validation.supplemental.facet-incompatibility` | `retain`, T6 | B-V-I; each tick/guide/layer presence under facets |
| `validation.axes.x-exact-ticks-require-numeric` | `retain`, T6 | B-V-I; exact runtime axis matrix |
| `validation.axes.y-exact-ticks-require-numeric` | `retain`, T6 | B-V-I; exact runtime axis matrix |
| `validation.guides.horizontal-requires-numeric-y` | `retain`, T6 | B-V-I; admitted/rejected axis families |
| `validation.guides.vertical-requires-numeric-x` | `retain`, T6 | B-V-I; admitted/rejected axis families |
| `validation.guides.segment-requires-two-numeric-axes` | `retain`, T6 | B-V-I; all one/two-axis combinations |

### T2b Public Exposure

Public exposure is evidence for the keyed declaration and policy records, not a
second property source.

| Source surface | Exposed T2b identities | Disposition |
| --- | --- | --- |
| `src/spec.rs:14-17` and `src/lib.rs:65-73` | `BarInput`, `BarSamples`, `Bootstrap`, `ConfidenceInterval`, `ErrorBar`, `PercentileInterval`, `StandardDeviation`, `StandardError` | Accounted by the 12 B-M2 bar/error records or rejected as numeric payload below. |
| `src/spec.rs:19` and `src/lib.rs:65-73` | `CategoricalColors`, `CategoryOrder`, `ValueSort` | Accounted by the five B-M3 category/scale records and their T2g consumer deferrals. |
| `src/spec.rs:20-23` and `src/lib.rs:65-73` | `AxisScale`, `Bounds`, `CoefficientInput`, `EstimateStatus`, `InteractionStyle`, `LinePathStyle`, `LinePattern`, `PointShape`, `QqLine`, `RegressionBand`, `SignificanceMarkStyle`, `TextWeight` | Accounted by B-M4/B-M10 records; `Bounds` numeric payload is rejected below. |
| `src/spec.rs:24` and `src/lib.rs:65-73` | `FacetScales`, `FacetSpec` | Accounted by the two B-M5 records and facet capability/control traces. |
| `src/spec.rs:26` and `src/lib.rs:75` | `PngCompression`, `PngOptions`, `PngSizing` | Public exposure only; all five declaration/behavior records are prior-owned by T2a4. |
| `src/lib.rs:50` | `Canvas`, `CanvasCategoryLabels`, `TextPanel` | Accounted by B-M11 and the prior/deferred Canvas contexts below. |
| `src/lib.rs:53-56` | `HistogramBin`, `HistogramDimensions`, `HistogramGeometry`, `HistogramKdePoint`, `HistogramRug`, `RaincloudDensityPoint`, `RaincloudDimensions`, `RaincloudGeometry`, `RaincloudGroup`, `RaincloudPoint` | Accounted by the 39 B-M8 semantic-identity and fixed-representation records; coordinate payload declarations are not separate properties. |
| `src/lib.rs:57` | `AxisValues`, `HistogramInput`, `RaincloudInput` | Public exposure only. `AxisValues` is an input carrier whose axis-kind gates are accounted by B-M12/B-M13; Histogram/Raincloud sample-versus-geometry selection is accounted by the geometry precedence and compatibility records. |
| `src/lib.rs:58-60` | `Labels`, `BarPointLayer`, `GuideSpec`, `GuideStyle`, `PointLayer`, `AxisTick` | Labels and layer declarations are accounted by B-M6/B-M7; `AxisTick` layout representation is T2c-owned. |
| `src/lib.rs:62-64` | `Attached`, `FacetCell`, `Pairwise`, `Significance`, `SignificanceMark`, `SignificanceTarget` | Attached/Pairwise family matrices and annotation presence are T2b-owned; exact annotation geometry/marks are T2f-owned. |
| `src/lib.rs:65-73` remaining direct `spec.rs` identities | `BarSpacing`, `CanvasLegendPolicy`, `DumbbellEndpoints`, `HeatmapAnnotations`, `HeatmapLabelFit`, `HeatmapLayout`, `HeatmapNullRowAnnotation`, `HeatmapScale`, `HistogramBins`, `HistogramComposite`, `HistogramKde`, `HistogramStat`, `LegendPolicy`, `LegendPosition`, `NumericLabelFormat`, `PanelCorner`, `PanelTextAnnotation`, `PlotKind`, `RawOverlay`, `WaterfallLabels` | Accounted by B-M1/B-M9/B-M10 records; private legend-default behavior and numeric formatting are deferred below. |
| `src/lib.rs:77` | `BooleanValues`, `CategoryValues`, `NumericValues`, `SourceRows` | Scientific/data carriers, not visual properties; T2b records only controls whose semantic presence consumes them. |

### Deferred T2b Source Contexts

| Source context | Owning phase | Reason it is not a T2b record |
| --- | --- | --- |
| `src/spec.rs:542-548,620-639` | T2d | Automatic/exact legend placement fallback and renderer-owned legend-clearance selection are legend construction/layout behavior, not the public visibility/placement declaration. |
| `src/spec.rs:592-604`; `src/builder/accessors.rs:39-89` | T2g | Unified-series routing merges constructor and optional series into hue/group semantic domains; it is one data-scale/domain mapping surface, not an additional public control. |
| `NumericLabelFormat::format` in `src/spec.rs` | T2f | Fixed string production is the annotation rendering consumer of T2b precision/suffix controls. |
| `TextWeight::svg_value` and `LinePattern::line_style` in `src/spec/estimate.rs:79-87,221-230` | T2e/T2f | Private final style conversion; public closed vocabularies remain T2b-owned. |
| `RegressionBand::legend_label` in `src/spec/estimate.rs:171-176` | T2d | Fixed generated legend claim is selected by the T2b band policy, but legend content construction is T2d-owned. |
| `GuideStyle::line_style`, `PointLayer::point_style`, and `BarPointLayer::point_style` in `src/layers.rs:39-41,197-200,292-301` | T2e | Exact-style conversion, including exact PointLayer paint precedence over one already selected automatic accent, and final draw carriers are mark/guide rendering sources. |
| Automatic PointLayer accent selection in `src/plots/line.rs:165-203` | T2g, from the T2a1 accent owner | T2g owns stable accent-slot selection and supplies that resolved paint to the T2e conversion; T2b owns the optional exact-color field/default/setter. |
| `src/builder/accessors.rs:642-657,716-851` | T2g | Categorical color lookup, palette fallback, semantic label/domain inference, and unified legend identities are scale/domain consumers. |
| `Plot::with_suppressed_legend` in `src/builder/accessors.rs:940-943` | T2d | Canvas composition suppresses a panel legend through an internal copy; this is a consumer of T2b policies, not another public visibility policy. |
| `src/canvas.rs:390-430` | T2d | Canvas preflight-to-render composition lifecycle, planning, and canonical composed-scene construction. |
| `src/canvas.rs:432-464` | T2h | Five Canvas terminal selections remain the explicitly admitted cross-source residual for final lifecycle reconciliation. |
| `src/canvas.rs:505-1077` except the B-M11 compatibility branches | T2c/T2d/T2g | Typography measurement, grid allocation, shared headers/titles, shared legends, domain collection, plan/render order, and hidden appearance lookup are later semantic owners. In particular, shared-category domain equality at `705-708` is T2g-owned. |
| `src/orchestrator/validation.rs:80-190,672-719` | T2g | Faceted explicit-order completeness, triangular family equality, normalization, and data-dependent category identities. |
| `src/orchestrator/validation.rs:533-538,544-551` | T2g | Continuous color-channel family/domain validation and beeswarm limit validity. |
| `src/orchestrator/layers.rs:10,74-225` | T2c/T2e | Tick count/projection/range/collision replacement is T2c; guide coordinate projection, style conversion, clipping, and drawing are T2e. |

### Prior-Owned T2b Contexts

| Source context | Prior owner | T2b treatment |
| --- | --- | --- |
| `Theme` storage/default/setter and every `Plot::palette()` reconstruction in `builder.rs`/`builder/`/`canvas.rs` | T2a1 and T2a4 | Consumer traces only; no duplicate theme or lifecycle record. |
| `PointOutline` carrier/default appearance and private point/line/fill/text styles | T2a2 | T2b records only the public outline-presence policy and exact public style fields. |
| `src/spec/output.rs:3-52` | T2a4 | All PNG declaration, mode, default, validation, and mapping records are complete there; B-M14 records exposure only. |
| Plot/Canvas scene rendering, Output publication, PNG/SVG terminals outside `canvas.rs:432-464`, and backend lifecycle | T2a3/T2a4 | Lifecycle/output consumers only. |
| `AxisSpec`, `AxisTick`, resolved axis ranges, exact-tick projection/collision, and axis text measurement | T2c allocation | T2b owns public content/policy controls and family gates only. |
| `Legend`, `LegendEntry`, legend metrics, shared-entry merging, and generated glyphs | T2d allocation | T2b owns only visibility, placement, sharing, title content, and exact layer legend identity. |
| Public significance placement/target geometry and bracket generation | T2f allocation | T2b owns annotation presence, exact text-style fields, and family/compatibility matrices only. |

### Rejected T2b Matches

| Source match | Disposition and reason |
| --- | --- |
| Imports, module declarations, ordinary derives, documentation comments, and public re-export statements | Language/API plumbing. Public exposure is accounted above and does not create duplicate properties. |
| `kind.rs` marker declarations, sealing, `ExactFamily`, `Any`, and erasure implementation | Typed API mechanics. The 23 independent admitted capability matrices are the policy sources; `SourceRows` is data-provenance availability and is rejected. |
| `Bounds`, coordinate/value vectors, category/boolean arrays, source-row maps, bootstrap unit payloads, and exact caller numeric positions | Scientific/data payload. Presence, fixed representation, and visible policy branches remain admitted where applicable. |
| `RaincloudDensityPoint`, `RaincloudPoint`, `HistogramBin`, `HistogramKdePoint`, and `HistogramRug` field declarations by themselves | Caller coordinate payload. Their fixed finite/order/support/topology/resource constraints are the admitted B-M8 renderer-invariant records. |
| `PlotInput`, row counts, aligned-column contents, matrix variable numeric payload, and `validate_value_alignment` | Data transport/integrity rather than visual policy. Independent visible presence/family branches remain admitted. |
| Private `PlotRenderContext`, `PlotRenderResult`, `CanvasPreflight`, plan structs, internal column-key constants, and accessor forwarding | Transport/plumbing; no independent visual value. |
| Error message prose and interpolated diagnostic values | Non-visual diagnostics. Every admitted controlling failure branch has one record. |
| Unit tests, doctest bodies, compile-fail examples, examples, snapshots, and test-only helpers/counters | Evidence only, never independent property sources. |
| `LinePattern` backend dash literal, point-layer automatic palette lookup, guide projection, and renderer-owned widths/colors supplied after public conversion | Deferred T2e/T2f/T2g implementation sources, not duplicate public declarations. |

### T2b Reconciliation

- The completed T2b inventory is exactly **261** records. Primary source
  attribution is 32 B-M1, 12 B-M2, 5 B-M3, 17 B-M4, 2 B-M5, 16 B-M6, 6
  B-M7, 39 B-M8, 23 B-M9, 31 B-M10, 28 B-M11, 42 B-M12, and 8 B-M13.
  The canonical table order is independent of those attribution totals.
- Each descriptive key occurs once in Current State, once in Target State, and
  once in Migration And Evidence, with the same 261-key order. Public exposure,
  deferred, prior-owned, and rejected tables are non-keyed audit dispositions
  and do not add records.
- Closed target-class totals are exactly **119 Explicit policy**, **112 Renderer
  invariant**, **19 Exact element style**, **4 Plot-local data scale**, **6 Theme
  component**, and **1 Theme data scale** records. The sum is 261, and no row
  uses an unclosed class or occupies more than one class.
- The corrected count preserves the binding splits: independent SD/SE scales,
  percentile/CI coverage, significance theme defaults versus exact overrides,
  heatmap theme default versus Plot-local replacement, PointLayer size/alpha
  defaults versus exact overrides, panel text/null-row/dumbbell/waterfall
  content and validity, five semantic geometry dimensions, complete Canvas row
  occupancy, four Matrix invariants, Attached versus Pairwise significance,
  numeric-x versus numeric-y runtime matrices, and all 42 configuration plus 8
  independent supplemental gates.
- `SourceRows` remains rejected; the five `canvas.rs:432-464` terminal methods
  remain T2h residuals; unified-series routing remains T2g-owned. No T2b record
  duplicates those contexts.
- T2a contributes 222 complete records. The ledger therefore inventories
  exactly **483** records through completed T2b. This is not a final-T2
  estimate; T2c through T2h remain source-driven and pending.
- Independent review corrected default-versus-override ownership, missing Canvas
  and Matrix invariants, capability counts, source dispositions, and exact
  current behavior without leaving a count, order, schema, or ownership finding.
  Tracked-diff checks and `just verify` passed on 2026-08-26. No production code,
  tests, snapshots, generated output, or other specifications changed in T2b.
