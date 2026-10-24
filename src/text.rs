//! Shared source syntax for authored and prepared display text.

/// Authored display text with optional complete appearance and layout instructions.
///
/// Plain strings carry no local instructions. Content validation happens
/// at the receiving constructor or preflight, without rewriting the source.
///
/// ```
/// use lichtung_plot::{NumericValues, PanelCorner, PanelTextAnnotation, Plot, Text, TextAlignment, TextLayout, TextPanel, Theme};
/// let style = Theme::publication().typography().get_composition()
///     .get_text_panel_title().clone().font_size(24.0)?;
/// let title = Text::new("Study\ncontext").style(style).alignment(TextAlignment::End);
/// let note = PanelTextAnnotation::new(PanelCorner::TopLeft, title.clone())?;
/// let plot = Plot::scatter(NumericValues::new("x", [0.0, 1.0]),
///     NumericValues::new("y", [1.0, 0.0]))
///     .title(title.clone()).title("Updated heading")
///     .caption(Text::new("Centered caption").alignment(TextAlignment::Center));
/// let panel = TextPanel::new(title, Text::new("Independent body")
///     .layout(TextLayout::default().alignment(TextAlignment::Start)))?;
/// assert_eq!(panel.title(), "Study\ncontext");
/// assert_eq!(note.text(), panel.title());
/// # let _ = plot;
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    source: String,
    style: Option<crate::TextTheme>,
    layout: Option<TextLayout>,
}

impl Text {
    /// Retain one verbatim display source.
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            style: None,
            layout: None,
        }
    }

    /// Borrow the original source.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Replace the complete exact appearance.
    #[must_use]
    pub fn style(mut self, style: crate::TextTheme) -> Self {
        self.style = Some(style);
        self
    }

    /// Inspect local appearance presence.
    #[must_use]
    pub const fn get_style(&self) -> Option<&crate::TextTheme> {
        self.style.as_ref()
    }

    /// Supply a whole layout request, including an explicit default reset.
    #[must_use]
    pub fn layout(mut self, layout: TextLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Distinguish an omitted layout from a present request.
    #[must_use]
    pub const fn get_layout(&self) -> Option<&TextLayout> {
        self.layout.as_ref()
    }

    /// Edit alignment within this text's one layout request.
    #[must_use]
    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.edit_alignment(alignment);
        self
    }

    const fn edit_alignment(&mut self, alignment: TextAlignment) {
        if let Some(layout) = &mut self.layout {
            layout.alignment = Some(alignment);
        } else {
            self.layout = Some(TextLayout {
                alignment: Some(alignment),
            });
        }
    }

    pub(crate) const fn alignment_request(&self) -> Option<TextAlignment> {
        match &self.layout {
            Some(layout) => layout.alignment,
            None => None,
        }
    }

    pub(crate) fn try_copy(&self) -> crate::Result<Self> {
        self.try_copy_with_source(crate::data::source::copy_source_string(&self.source)?)
    }

    pub(crate) fn try_copy_with_source(&self, source: String) -> crate::Result<Self> {
        Ok(Self {
            source,
            style: self
                .style
                .as_ref()
                .map(crate::TextTheme::try_copy)
                .transpose()?,
            layout: self.layout.clone(),
        })
    }

    pub(crate) fn appearance<'a>(&'a self, role: &'a crate::TextTheme) -> TextAppearance<'a> {
        TextAppearance {
            theme: self.style.as_ref().unwrap_or(role),
            exact_override: self.style.is_some(),
        }
    }

    pub(crate) fn resolved_alignment(&self, default: TextAlignment) -> TextAlignment {
        self.alignment_request().unwrap_or(default)
    }
}

/// An optional authored owner can retain instructions before it receives content.
/// Its one Text request remains authoritative when later content replaces source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextSlot {
    text: Text,
    present: bool,
}

impl TextSlot {
    pub(crate) fn edit_style(&mut self, style: crate::TextTheme) {
        self.text.style = Some(style);
    }
    pub(crate) fn edit_layout(&mut self, layout: TextLayout) {
        self.text.layout = Some(layout);
    }
    pub(crate) fn resolve_source(&self, source: String) -> crate::Result<Text> {
        if self.present {
            self.text.try_copy()
        } else {
            self.text.try_copy_with_source(source)
        }
    }
    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> crate::Result<()> {
        round.observe_count(usize::from(self.present))?;
        round.observe_authored_text(&self.text)
    }
    pub(crate) const EMPTY: Self = Self {
        text: Text {
            source: String::new(),
            style: None,
            layout: None,
        },
        present: false,
    };

    pub(crate) fn replace(&mut self, incoming: Text) {
        self.text.source = incoming.source;
        if incoming.style.is_some() {
            self.text.style = incoming.style;
        }
        if incoming.layout.is_some() {
            self.text.layout = incoming.layout;
        }
        self.present = true;
    }

    pub(crate) const fn edit_alignment(&mut self, alignment: TextAlignment) {
        self.text.edit_alignment(alignment);
    }

    pub(crate) const fn alignment_request(&self) -> Option<TextAlignment> {
        self.text.alignment_request()
    }

    pub(crate) const fn content(&self) -> Option<&Text> {
        if self.present {
            Some(&self.text)
        } else {
            None
        }
    }

    pub(crate) fn source(&self) -> Option<&str> {
        self.content().map(Text::as_str)
    }

    pub(crate) fn try_copy(&self) -> crate::Result<Self> {
        Ok(Self {
            text: self.text.try_copy()?,
            present: self.present,
        })
    }
}

impl From<String> for Text {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

impl From<&str> for Text {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<&String> for Text {
    fn from(source: &String) -> Self {
        Self::new(source)
    }
}

/// Optional text layout instructions. Omitted fields retain owner defaults.
///
/// A supplied default request is an explicit reset, distinct from no request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextLayout {
    alignment: Option<TextAlignment>,
}

impl TextLayout {
    /// Set physical line alignment within the text block.
    #[must_use]
    pub const fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }
}

/// One immutable selected role, including exact-appearance provenance.
#[derive(Clone, Copy)]
pub(crate) struct TextAppearance<'a> {
    pub(crate) theme: &'a crate::TextTheme,
    pub(crate) exact_override: bool,
}

impl<'a> From<&'a crate::TextTheme> for TextAppearance<'a> {
    fn from(theme: &'a crate::TextTheme) -> Self {
        Self {
            theme,
            exact_override: false,
        }
    }
}

/// Physical horizontal alignment within a text owner's local span.
///
/// Start and End mean physical left and right, including for right-to-left text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    /// Align to the physical left edge of the local span.
    Start,
    /// Align to the physical midpoint of the local span.
    Center,
    /// Align to the physical right edge of the local span.
    End,
}

impl TextAlignment {
    /// Resolve one physical x coordinate and backend anchor.
    pub(crate) fn resolve_physical(
        self,
        span_start: f64,
        span_end: f64,
    ) -> (f64, crate::render::backend::TextAnchor) {
        use crate::render::backend::TextAnchor;
        match self {
            Self::Start => (span_start, TextAnchor::Start),
            Self::Center => (
                span_start + (span_end - span_start) / 2.0,
                TextAnchor::Middle,
            ),
            Self::End => (span_end, TextAnchor::End),
        }
    }
}

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
    fn content_replacement_retains_instructions_until_a_whole_request_replaces_them() {
        use super::{Text, TextAlignment, TextLayout, TextSlot};
        const BEFORE_CONTENT: TextSlot = {
            let mut slot = TextSlot::EMPTY;
            slot.edit_alignment(TextAlignment::End);
            slot
        };
        let mut slot = BEFORE_CONTENT;
        assert!(slot.content().is_none());
        let style = crate::Theme::publication()
            .typography()
            .get_root()
            .get_title()
            .clone();
        slot.replace(Text::new("first").style(style.clone()));
        slot.replace(Text::new("replacement"));
        assert_eq!(
            slot.content(),
            Some(
                &Text::new("replacement")
                    .style(style.clone())
                    .alignment(TextAlignment::End)
            )
        );
        slot.replace(Text::new("reset").layout(TextLayout::default()));
        assert_eq!(
            slot.content(),
            Some(
                &Text::new("reset")
                    .style(style.clone())
                    .layout(TextLayout::default())
            )
        );
        slot.edit_alignment(TextAlignment::Start);
        assert_eq!(
            slot.content(),
            Some(
                &Text::new("reset")
                    .style(style)
                    .alignment(TextAlignment::Start)
            )
        );
    }

    #[test]
    fn whole_layout_reset_preserves_style_and_complete_text_panel_equality() {
        use super::{Text, TextAlignment, TextLayout};
        let style = crate::Theme::publication()
            .typography()
            .get_composition()
            .get_text_panel_title()
            .clone();
        let authored = Text::new("same source")
            .style(style.clone())
            .alignment(TextAlignment::End);
        let reset = authored.clone().layout(TextLayout::default());
        assert_eq!(
            reset.resolved_alignment(TextAlignment::Start),
            TextAlignment::Start
        );
        assert_eq!(reset.get_style(), Some(&style));
        assert_ne!(reset, Text::new("same source").style(style.clone()));
        assert_eq!(
            reset,
            Text::new(String::from("same source"))
                .style(style.clone())
                .layout(TextLayout::default())
        );
        let panel = TextPanel::new(authored, "body").unwrap();
        let changed = TextPanel::new(
            Text::new("same source")
                .style(style.font_size(37.0).unwrap())
                .alignment(TextAlignment::End),
            "body",
        )
        .unwrap();
        assert_ne!(
            panel, changed,
            "same text and alignment do not erase appearance identity"
        );
        assert_ne!(
            panel,
            TextPanel::new(reset, "body").unwrap(),
            "request reset participates in identity"
        );
    }

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
