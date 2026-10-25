//! Data anchoring and text-only adaptation share the normal composed lifecycle.

// ANCHOR: example
#[path = "support/output.rs"]
mod output;

use lichtung_plot::{
    Canvas, DataCoordinate, NumericValues, PanelTextAnnotation, Plot, Text, TextAlignment,
    TextAnchor, TextBounds, TextCoordinate, TextElision, TextEllipsis, TextFailure, TextFit,
    TextFitPriority, TextLayout, TextOverflow, TextPanel, TextPosition, TextRotation, Theme,
};

fn main() -> lichtung_plot::Result<()> {
    let directory = output::directory()?;
    let theme = Theme::publication();
    let heading = theme
        .typography()
        .get_composition()
        .get_text_panel_title()
        .clone()
        .font_size(14.25)?;
    // ANCHOR: text-reference
    let note = PanelTextAnnotation::at(
        TextPosition::new(
            TextCoordinate::data(DataCoordinate::numeric(2.0)?),
            TextCoordinate::data(DataCoordinate::numeric(3.0)?),
        ),
        Text::new("Fixed data reference").layout(
            TextLayout::default()
                .anchor(TextAnchor::bottom_left())
                .rotation(TextRotation::degrees(30.05)?),
        ),
    )?;
    // ANCHOR_END: text-reference
    let plot = Plot::scatter(
        NumericValues::new("x", [0.0, 1.0, 4.0]),
        NumericValues::new("y", [1.0, 0.0, 2.0]),
    )
    .x_limits(0.0, 4.0)
    .y_limits(0.0, 4.0)
    .panel_text_annotation(note)
    .title("Scientific coordinates stay unchanged");
    // ANCHOR: text-fitting
    let title = Text::new("Treatment group with extended follow-up")
        .style(heading)
        .layout(
            TextLayout::default()
                .bounds(TextBounds::new(Some(140.0), None, Some(2))?)
                .fit(TextFit::range(10.25, 14.25)?)
                .fit_priority(TextFitPriority::ContentFirst)
                .overflow(TextOverflow::ellipsis(
                    TextEllipsis::new(TextElision::End).max_lines(Some(2)),
                    TextFailure::Error,
                )),
        );
    // ANCHOR_END: text-fitting
    let panel = TextPanel::new(
        title,
        Text::new("Independent complete body. Only the heading permits shrinking or ellipsis.")
            .alignment(TextAlignment::Start),
    )?;
    // ANCHOR: text-publication
    let destination = directory.join("text_layout.svg");
    let output = Canvas::grid(1, 2)?
        .theme(theme)
        .panel(0, 0, plot)?
        .text_panel(0, 1, panel)?
        .max_width(1100)
        .svg_preserve_text(&destination)?
        .svg(destination.with_file_name("text_layout_outlined.svg"))?;
    // ANCHOR_END: text-publication
    println!(
        "text_layout.svg: {}×{} CSS pixels",
        output.width(),
        output.height()
    );
    Ok(())
}
// ANCHOR_END: example
