//! Shared source syntax for authored and prepared display text.

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("display text contains unsupported control character U+{scalar:04X}")]
pub(crate) struct UnsupportedControl {
    pub(crate) scalar: u32,
}

/// Validate content without trimming, rewriting or treating empty text as absence.
pub(crate) fn validate(source: &str) -> Result<(), UnsupportedControl> {
    match source
        .chars()
        .find(|&scalar| is_unsupported_control(scalar))
    {
        Some(scalar) => Err(UnsupportedControl {
            scalar: scalar as u32,
        }),
        None => Ok(()),
    }
}

pub(crate) fn validate_for(source: &str, role: &str) -> crate::Result<()> {
    validate(source).map_err(|error| crate::Error::Data(format!("{role}: {error}")))
}

pub(crate) fn is_unsupported_control(scalar: char) -> bool {
    scalar.is_control()
        && !is_nonrendering_mandatory_break(scalar)
        && !is_nonpainting_control(scalar)
}

// UAX #14 BK, CR, LF, and NL controls partition paragraphs and do not paint.
// They remain in the census, but are not glyph-coverage requirements.
pub(crate) const fn is_nonrendering_mandatory_break(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

/// Unicode 16 Default_Ignorable_Code_Point plus bidi controls.
pub(crate) const fn is_nonpainting_control(scalar: char) -> bool {
    matches!(scalar as u32, 0x00AD | 0x034F | 0x061C | 0x115F..=0x1160
        | 0x17B4..=0x17B5 | 0x180B..=0x180F | 0x200B..=0x200F
        | 0x202A..=0x202E | 0x2060..=0x206F | 0x3164 | 0xFE00..=0xFE0F
        | 0xFEFF | 0xFFA0 | 0xFFF0..=0xFFF8 | 0x1BCA0..=0x1BCA3
        | 0x1D173..=0x1D17A | 0xE0000..=0xE0FFF)
}

#[cfg(test)]
mod tests {
    use crate::{
        AxisTick, Canvas, Labels, NumericValues, PanelCorner, PanelTextAnnotation, Plot, TextPanel,
    };

    #[test]
    fn unsupported_display_controls_fail_before_font_discovery() {
        let before = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let plot = || {
            Plot::scatter(
                NumericValues::new("x", [0.0, 1.0]),
                NumericValues::new("y", [1.0, 0.0]),
            )
        };
        let canvas = || {
            Canvas::grid(1, 1)
                .unwrap()
                .text_panel(0, 0, TextPanel::new("", "body").unwrap())
                .unwrap()
        };
        let errors = [
            PanelTextAnnotation::new(PanelCorner::TopLeft, "a\tb").unwrap_err(),
            TextPanel::new("title", "a\tb").unwrap_err(),
            canvas().column_header(0, 1, "a\tb").err().unwrap(),
            canvas().title("a\tb").preflight().unwrap_err(),
            plot().caption("a\tb").preflight().unwrap_err(),
            plot().x_label("a\tb").preflight().unwrap_err(),
            plot()
                .x_ticks(vec![AxisTick::new(0.0, "a\tb")])
                .preflight()
                .unwrap_err(),
            Labels::humanized()
                .alias("raw", "a\tb")
                .resolve("raw")
                .unwrap_err(),
        ];
        for error in errors {
            assert!(
                error
                    .to_string()
                    .contains("unsupported control character U+0009"),
                "{error}"
            );
        }
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            before
        );
    }
}
