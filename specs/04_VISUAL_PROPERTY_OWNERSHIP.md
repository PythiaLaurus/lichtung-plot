# Visual Property Ownership Ledger

**Status**: Canonical T2 migration ledger; T2a umbrella IN PROGRESS

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
| T2a | Ledger protocol and renderer foundations | IN PROGRESS |
| T2a1 | Current theme and font foundations | COMPLETE |
| T2a2 | Private style carriers and backend protocol | COMPLETE |
| T2a3 | Serialization, geometry recording, and scene conversion | PENDING |
| T2a4 | Lifecycle, output, and artifact foundations | PENDING |
| T2b | Public controls and policies | PENDING |
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
| `backend.svg.rgb-hex-serialization` | M4 `rgb_to_hex((u8,u8,u8)) -> String`; uppercase `#{:02X}{:02X}{:02X}` | C-rgb | Total for typed RGB; no alpha or fallback; opacity remains in separate carriers | `none`; final private draw boundary | Canonical current SVG RGB paint serialization | M4; format width and every call admitted |
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
| `backend.svg.rgb-hex-serialization` | Renderer invariant; private final RGBA paint serialization responsibility | Final private draw boundary | Typed final RGBA; backend protocol emits exactly one alpha without multiplication | Private-only | Current-to-target counterpart for every paint; T2a2 style-alpha and T2a3 serialization records |
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
| `backend.svg.rgb-hex-serialization` | `replace`, T5a; scale callers complete in T5d | RGB spelling unchanged; exact paint serialization and alpha assertions |
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
| `backend.protocol.coordinate-convention` | Renderer invariant; canonical backend coordinate and text-position responsibility | Final private draw boundary | Non-text geometry uses finite `f64`; text positions use exact `u32` math-space pixels; both use one bottom-up space before backend conversion | Private-only | Carrier `backend.coordinates.projected-point-carrier`; text placement `backend.text.placement-carrier` |
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
| `backend.protocol.coordinate-convention` | `retain`, T3 | No output change; projected geometry, integer text-position, and y-orientation assertions across Geometry and SVG |
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
- T2a1 and T2a2 contain exactly **117 completed T2a records**. The reconciled
  four-part T2a plan remains **193 records**, with 76 pending in T2a3/T2a4.
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
