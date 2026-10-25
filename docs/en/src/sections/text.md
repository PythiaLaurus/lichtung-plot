<a id="text"></a>
## Text and annotations

[简体中文](../zh-CN/index.html#text)

<a id="text-layout"></a>
### Walkthrough: a fixed data reference and an adaptable heading

![A data-anchored note beside a locally fitted TextPanel heading](gallery/text-layout/text_layout_outlined.svg)

The left panel's note refers to data coordinates (2,3), not necessarily an observed
sample. Its reference and block anchor are separate: the data position stays fixed
while `bottom_left` and the rotation determine how text is drawn around it. They
do not move scientific points:

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-reference}}
```

The right panel has a different responsibility. Its heading clones the Theme's
TextPanel title style at 14.25 CSS pixels, then explicitly permits local fitting
down to 10.25 and end ellipsis within the declared two-line bound. No other text
or scientific content receives that permission:

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-fitting}}
```

Canvas combines the Plot and TextPanel. The preserve-text publication and the
outlined gallery image come from one frozen scene, not separate layout runs:

```rust
{{#include ../../../../crates/lichtung-plot/examples/12_text_layout.rs:text-publication}}
```

Run `cargo run --locked --offline -p lichtung-plot --example 12_text_layout`.

[Complete source](examples/12_text_layout.rs) · [Shared output helper](examples/support/output.rs) ·
[Full-size SVG](gallery/text-layout/text_layout_outlined.svg) ·
[Selectable-text SVG](gallery/text-layout/text_layout.svg) ·
[TextLayout API](https://docs.rs/lichtung-plot/latest/lichtung_plot/struct.TextLayout.html) · [Back to gallery](#gallery)

<a id="text-options"></a>
### Text policies and ownership

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

### References, anchors and local adaptation

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

The [text-layout walkthrough](#text-layout) keeps the data reference and local
heading adaptation independent. See [themes](#themes) for role ownership and
[output](#output) for outlined versus selectable text.
