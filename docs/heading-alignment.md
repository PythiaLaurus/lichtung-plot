# Independent Heading Instructions

## Public Scope And Resolution

Common Plot configuration exposes `title_alignment(TitleAlignment)` and
`subtitle_alignment(TitleAlignment)`, including on erased Plots. Canvas exposes
`panel_title_alignment(TitleAlignment)` and
`panel_subtitle_alignment(TitleAlignment)` for embedded Plot prose. These four
setters store independent optional inputs; repeated calls replace only their own
input, and complete Theme attachment order is independent.

Title resolves Plot instruction, Canvas panel default when composed, then the
resolved Plot theme's concrete root-title default. Subtitle resolves its Plot
instruction, Canvas panel subtitle default, the optional Plot-theme subtitle
default, then the final effective title alignment. Absence contributes no
instruction; it does not bypass a lower subtitle default. Explicit Plot Theme
replacement does not erase Canvas panel instructions.

Caption independently uses `CompositionTheme::root_caption_alignment`. Canvas's
own title, subtitle and caption use only its complete root theme. The two unused
Canvas root instruction fields are deleted; root sizing and assigned planning
pass actual absence rather than retain a dormant second alignment owner.
Headers, row titles, facet strips and TextPanel text retain their distinct roles.

Start and End mean physical left and right after the owning content insets,
independent of text direction. Both block placement and per-line alignment use
the same selected value. The existing measured root producer resolves concrete
alignments before measurement and retains them through canonical capture/replay;
drawing does not rerun optional precedence.

```rust
use lichtung_plot::{Canvas, NumericValues, Plot, Theme, TitleAlignment};

let plot = Plot::scatter(
    NumericValues::new("x", [1.0, 2.0]),
    NumericValues::new("y", [2.0, 1.0]),
)
.title("Plot instruction wins")
.subtitle("Independent subtitle")
.title_alignment(TitleAlignment::End)
.subtitle_alignment(TitleAlignment::Start)
.theme(Theme::science());
let canvas = Canvas::grid(1, 1)?
    .panel_title_alignment(TitleAlignment::Start)
    .panel_subtitle_alignment(TitleAlignment::End)
    .panel(0, 0, plot)?;
# let _ = canvas;
# Ok::<(), lichtung_plot::Error>(())
```

## Focused Acceptance Evidence

`canvas::lifecycle_tests::heading_instructions_are_role_local_order_independent_and_frozen`
replaces the redundant closed-route classification inventory
`route_census_covers_every_plot_family`. Production dispatch remains exhaustive
and the existing real all-family lifecycle owner remains. The new three-child
scene proves inherited/complete-replaced/explicit-instruction precedence, Canvas
root independence, distinct captions, complete multiline retention, opposite
setter orders and repeated replacement. Equivalent construction orders publish
identical canonical SVG, and replay does no data/font/projection/measurement work.

Existing standalone and Matrix frozen-root fixtures now attach complete public
Themes and use public Plot instructions. The standalone fixture also constructs
caption alignment through the public component/builder path for all four presets.
The outer-facet fixture uses the same public setters and preserves centered
strips. An existing composition integration uses both Canvas instructions and
checks independent physical heading positions while retaining its vertical
clearance contract.

Validation passed:

- **Five focused library tests**: the replacement, standalone, Matrix,
  outer-facet and independently justified physical-end ink-span contracts.
- **Eight existing external contracts**: embedded heading placement, independent
  root/Plot captions and six default/inherited/replaced/isolated theme owners.
- **86 doctests**, including the two new public heading examples.
- Strict all-target Clippy, formatting, whitespace checks, warning-denied API
  documentation and successful locked/offline package staging compilation.
- Library discovery remains **671**, including **17 ignored**, with no moved
  unit cases, full library/integration suite or repinned scene golden.

Execution logs are `/tmp/opencode/heading-alignment-*.log`. The following public
foundation closure supplies custom-owner, font, contrast, palette and scale
precedence evidence over the completed construction and instruction surfaces.
