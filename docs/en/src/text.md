# Text and annotations

[简体中文](../zh-CN/text.html)

`Text` retains original source and optional complete TextTheme/TextLayout
instructions. Plain-string replacement retains previous instructions; a present
style or layout replaces that entire instruction. An explicit default layout
resets to owner defaults. Titles, subtitles, captions, axis titles, color-guide
titles, Canvas headers, TextPanel title/body and panel notes share this model.

Authored whitespace and newlines are preserved, including blank lines; CRLF is one
break. Empty authored prose occupies one empty line. Tabs and unsupported controls
fail. Exact AxisTick empty labels instead suppress text while retaining the tick.
Automatic names use Labels humanization/raw/aliases without changing raw identity.

Plot title/subtitle alignment is independent. Canvas panel defaults apply only
when the child has no explicit instruction; caption alignment is independent.
Start/Center/End heading instructions are physical after content insets. Text
layout also distinguishes logical Start/End from physical Left/Right in its own
line-alignment program. Panel instructions do not change Canvas-owned prose,
headers or TextPanel content.

Each TextPanel title/body can choose its own complete style and layout; omission
uses the independent Canvas roles. Local sizes drive their own padding/gaps.
Axis-title style also owns its relative clearances, including shared facet titles.
Shared color-guide titles require complete equal Text requests, including explicit resets.

## References, anchors and local adaptation

PanelTextAnnotation::new anchors to a corner. `at` instead accepts typed
Data/Panel/Figure references; TextLayout's block anchor is a separate choice.
The data reference is projected by the actual panel, not inferred from screen
coordinates. `position()` reads the typed reference. TextOffset chooses Screen
or TextLocal CSS offsets. Headers remain centered on their declared spans while
local alignment controls individual lines.

TextLayout controls bounds, wrapping, visibility, fractional font fitting,
rotation and overflow. A fixed angle or horizontal replacement is distinct from an
explicit ordered automatic tick-angle list. AlignAfterRotation aligns the rotated
box; the default rotates around its selected anchor. Typed data directions use the
actual projection rather than an unscaled numeric slope.

Overflow can refuse, hide, clip, remain visible or explicitly elide Start/Middle/
End with a configured marker and failure policy. The marker is unnecessary when
complete text fits. Original source remains in nonpainting SVG accessibility
metadata. Heatmap's dense three-dot default uses this engine; an explicit default
tick TextLayout restores complete wrapping rather than silently deleting categories.

`TextFit::shrink_to` permits a local fractional floor; range also allows bounded
growth. ContentFirst prefers complete content; PreferredSizeFirst may try allowed
ellipsis at the preferred size before shrinking. Uniform grouping applies only to
the eligible actual role group, not unrelated axes or independent title/body text.
Visibility every/greedy selects text without deleting marks, ticks or grids.

HeatmapNullRowAnnotation retains a raw row key and a Text explanation. WaterfallLabels
owns independent base/terminal names, whose styles apply to each complete
name-plus-number annotation. PointLayer::legend_label has independent Text/style
from point annotations; repeated layer identities must agree on complete label
requests and mark keys. None of these changes scientific membership.

```rust
{{#include ../../../crates/lichtung-plot/examples/12_text_layout.rs:example}}
```

The [text-layout gallery case](gallery-text-layout.md) shows the exact data anchor
beside an independently adapted TextPanel heading. See [themes](themes.md) for
role ownership and [output](output.md) for outlined versus selectable text.
