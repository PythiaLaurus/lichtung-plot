//! Shared source syntax for authored and prepared display text.

mod fit;
pub(crate) mod layout;
pub use fit::{TextFit, TextFitGrouping, TextFitPriority};
pub(crate) mod orientation;
pub(crate) mod position;
pub use orientation::{TextOrientation, TextRotation, TextRotationMode};
pub(crate) mod roles;
pub use layout::{
    TextAnchor, TextBounds, TextDirection, TextElision, TextEllipsis, TextFailure, TextOffset,
    TextOffsetFrame, TextOverflow, TextOverlap, TextVisibility, TextWrapping,
};
pub use position::{DataCoordinate, DataPosition, TextCoordinate, TextPosition};
pub use roles::GuideTextRole;

/// Authored display text with optional complete appearance and layout instructions.
///
/// Plain strings carry no local instructions. Content validation happens
/// at the receiving constructor or preflight, without rewriting the source.
/// Clones share immutable source bytes and retain complete local instructions.
///
/// ```
/// use lichtung_plot::{NumericValues, DataAreaCorner, PlotTextAnnotation, Plot, Text, TextAlignment, TextLayout, TextPanel, Theme};
/// let style = Theme::publication().typography().get_composition()
///     .get_text_panel_title().clone().font_size(24.0)?;
/// let title = Text::new("Study\ncontext").style(style).alignment(TextAlignment::End);
/// let note = PlotTextAnnotation::new(DataAreaCorner::TopLeft, title.clone())?;
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
    source: Option<std::sync::Arc<String>>,
    style: Option<crate::TextTheme>,
    layout: Option<TextLayout>,
}

impl Text {
    /// Retain one verbatim display source.
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: Some(std::sync::Arc::new(source.into())),
            style: None,
            layout: None,
        }
    }

    /// Borrow the original source.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.source.as_ref().map_or("", |source| source.as_str())
    }

    /// Replace the complete exact appearance.
    #[must_use]
    pub fn style(mut self, style: crate::TextTheme) -> Self {
        self.style = Some(style);
        self
    }

    /// Remove local appearance without changing content or layout.
    /// The receiving owner's Theme role supplies appearance when this value is used.
    #[must_use]
    pub fn clear_style(mut self) -> Self {
        self.edit_clear_style();
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

    /// Remove the local layout request, rather than attach a present default.
    /// Passing this value to an existing slot still follows omission/preserve rules.
    #[must_use]
    pub fn clear_layout(mut self) -> Self {
        self.edit_clear_layout();
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

    pub(crate) const fn edit_alignment(&mut self, alignment: TextAlignment) {
        if self.layout.is_none() {
            // The replaced value is known None. Forget only that empty value so
            // this existing const edit does not require const Arc destruction.
            std::mem::forget(self.layout.replace(TextLayout::EMPTY));
        }
        if let Some(layout) = &mut self.layout {
            layout.alignment = Some(alignment);
        }
    }

    pub(crate) const fn alignment_request(&self) -> Option<TextAlignment> {
        match &self.layout {
            Some(layout) => layout.alignment,
            None => None,
        }
    }

    pub(crate) fn try_copy(&self) -> crate::Result<Self> {
        Ok(Self {
            source: self.source.clone(),
            style: self
                .style
                .as_ref()
                .map(crate::TextTheme::try_copy)
                .transpose()?,
            layout: self.layout.clone(),
        })
    }

    pub(crate) fn try_copy_with_source(&self, source: String) -> crate::Result<Self> {
        Ok(Self {
            source: Some(adopt_text_source_checked(source)?),
            style: self
                .style
                .as_ref()
                .map(crate::TextTheme::try_copy)
                .transpose()?,
            layout: self.layout.clone(),
        })
    }

    pub(crate) fn appearance<'a>(
        &'a self,
        role: impl Into<TextAppearance<'a>>,
    ) -> TextAppearance<'a> {
        let role = role.into();
        TextAppearance {
            theme: self.style.as_ref().unwrap_or(role.theme),
            exact_override: self.style.is_some() || role.exact_override,
            layout: self.layout.as_ref().or(role.layout),
        }
    }

    pub(crate) fn resolved_alignment(&self, default: TextAlignment) -> TextAlignment {
        self.alignment_request().unwrap_or(default)
    }
}

impl Text {
    pub(crate) const UNSET: Self = Self {
        source: None,
        style: None,
        layout: None,
    };

    pub(crate) const fn is_provided(&self) -> bool {
        self.source.is_some()
    }

    pub(crate) const fn provided(&self) -> Option<&Self> {
        if self.is_provided() {
            Some(self)
        } else {
            None
        }
    }

    pub(crate) fn required_source(&self, role: &str) -> crate::Result<&str> {
        let source = self.source.as_ref().ok_or_else(|| {
            crate::Error::Render(format!("{role}: required text source is unset"))
        })?;
        validate_for(source, role)?;
        Ok(source)
    }

    pub(crate) fn edit_style(&mut self, style: crate::TextTheme) {
        self.style = Some(style);
    }
    pub(crate) fn edit_layout(&mut self, layout: TextLayout) {
        self.layout = Some(layout);
    }
    pub(crate) fn edit_clear_style(&mut self) {
        self.style = None;
    }
    pub(crate) fn edit_clear_layout(&mut self) {
        self.layout = None;
    }
    pub(crate) fn resolve_source(&self, source: String) -> crate::Result<Text> {
        if self.is_provided() {
            self.try_copy()
        } else {
            self.try_copy_with_source(source)
        }
    }
    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> crate::Result<()> {
        round.observe_count(usize::from(self.is_provided()))?;
        round.observe_authored_text(self)
    }

    pub(crate) fn replace_from(&mut self, incoming: Text) -> crate::Result<()> {
        if !incoming.is_provided() {
            return Err(crate::Error::Render(
                "text replacement: required text source is unset".into(),
            ));
        }
        self.source = incoming.source;
        if incoming.style.is_some() {
            self.style = incoming.style;
        }
        if incoming.layout.is_some() {
            self.layout = incoming.layout;
        }
        Ok(())
    }
}

fn adopt_text_source_checked(source: String) -> crate::Result<std::sync::Arc<String>> {
    crate::data::source::source_arc(source)
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
    pub(crate) alignment: Option<TextAlignment>,
    instructions: Option<std::sync::Arc<TextInstructions>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextInstructions {
    pub(crate) fit: Option<TextFit>,
    pub(crate) fit_priority: Option<TextFitPriority>,
    pub(crate) direction: Option<TextDirection>,
    pub(crate) orientation: Option<TextOrientation>,
    pub(crate) rotation_mode: Option<TextRotationMode>,
    pub(crate) anchor: Option<TextAnchor>,
    pub(crate) offset: Option<TextOffset>,
    pub(crate) bounds: Option<TextBounds>,
    pub(crate) wrapping: Option<TextWrapping>,
    pub(crate) overflow: Option<TextOverflow>,
    pub(crate) overlap: Option<TextOverlap>,
    pub(crate) visibility: Option<TextVisibility>,
}

impl TextLayout {
    pub(crate) fn hidden(&self) -> bool {
        self.instructions()
            .visibility
            .as_ref()
            .is_some_and(|visibility| visibility.hidden)
    }
    pub(crate) const EMPTY: Self = Self {
        alignment: None,
        instructions: None,
    };

    pub(crate) fn instructions(&self) -> &TextInstructions {
        static EMPTY: TextInstructions = TextInstructions {
            fit: None,
            fit_priority: None,
            direction: None,
            orientation: None,
            rotation_mode: None,
            anchor: None,
            offset: None,
            bounds: None,
            wrapping: None,
            overflow: None,
            overlap: None,
            visibility: None,
        };
        self.instructions.as_deref().unwrap_or(&EMPTY)
    }
    pub(crate) fn edit_instructions(&mut self) -> &mut TextInstructions {
        std::sync::Arc::make_mut(
            self.instructions
                .get_or_insert_with(|| std::sync::Arc::new(TextInstructions::default())),
        )
    }
    /// Set physical line alignment within the text block.
    #[must_use]
    pub const fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = Some(alignment);
        self
    }
}

macro_rules! layout_edits {
    ($($field:ident:$value:ty),+ $(,)?) => {impl TextLayout {$(
        #[doc = concat!("Replace this request's `",stringify!($field),"` instruction.")]
        #[must_use]
        pub fn $field(mut self,value:$value) -> Self {self.edit_instructions().$field=Some(value);self}
    )+}};
}
layout_edits! {anchor:TextAnchor,offset:TextOffset,bounds:TextBounds,wrapping:TextWrapping,overflow:TextOverflow,overlap:TextOverlap,visibility:TextVisibility,orientation:TextOrientation,rotation_mode:TextRotationMode,direction:TextDirection,fit:TextFit,fit_priority:TextFitPriority}
impl TextLayout {
    /// Replace orientation with one fixed screen angle.
    #[must_use]
    pub fn rotation(self, value: TextRotation) -> Self {
        self.orientation(TextOrientation::fixed(value))
    }
}

/// One immutable selected role, including exact-appearance provenance.
#[derive(Clone, Copy)]
pub(crate) struct TextAppearance<'a> {
    pub(crate) theme: &'a crate::TextTheme,
    pub(crate) exact_override: bool,
    pub(crate) layout: Option<&'a TextLayout>,
}

impl<'a> From<&'a crate::TextTheme> for TextAppearance<'a> {
    fn from(theme: &'a crate::TextTheme) -> Self {
        Self {
            theme,
            exact_override: false,
            layout: None,
        }
    }
}

impl TextAppearance<'_> {
    pub(crate) fn require_single(self) -> crate::Result<Self> {
        if !self.hidden()
            && self
                .layout
                .and_then(|request| request.instructions().orientation.as_ref())
                .is_some_and(|orientation| {
                    matches!(orientation.0, orientation::Orientation::Automatic(_))
                })
        {
            return Err(crate::Error::Data(
                "automatic orientation requires an actual tick group".into(),
            ));
        }
        if self
            .layout
            .and_then(|request| request.instructions().visibility.as_ref())
            .is_some_and(crate::TextVisibility::reduces)
        {
            return Err(crate::Error::Data("text visibility reduction requires an actual label group, not an independent authored slot".into()));
        }
        Ok(self)
    }
    pub(crate) fn includes(self, index: usize, eligible: &[usize]) -> bool {
        self.layout
            .and_then(|request| request.instructions().visibility.as_ref())
            .is_none_or(|policy| policy.includes(index, eligible))
    }
    pub(crate) fn hidden(self) -> bool {
        self.layout.is_some_and(|layout| {
            layout
                .instructions()
                .visibility
                .as_ref()
                .is_some_and(|visibility| visibility.hidden)
        })
    }
}

impl<'a> TextAppearance<'a> {
    pub(crate) fn layout(mut self, layout: Option<&'a TextLayout>) -> Self {
        self.layout = layout;
        self
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
        AxisTick, Canvas, DataAreaCorner, Labels, NumericValues, Plot, PlotTextAnnotation,
        TextPanel,
    };

    #[test]
    fn content_replacement_retains_instructions_until_a_whole_request_replaces_them() {
        use super::{Text, TextAlignment, TextLayout};
        const BEFORE_CONTENT: Text = {
            let mut slot = Text::UNSET;
            slot.edit_alignment(TextAlignment::End);
            slot
        };
        let mut slot = BEFORE_CONTENT;
        assert!(slot.provided().is_none());
        let style = crate::Theme::publication()
            .typography()
            .get_root()
            .get_title()
            .clone();
        slot.replace_from(Text::new("first").style(style.clone()))
            .unwrap();
        slot.replace_from(Text::new("replacement")).unwrap();
        assert_eq!(
            slot.provided(),
            Some(
                &Text::new("replacement")
                    .style(style.clone())
                    .alignment(TextAlignment::End)
            )
        );
        slot.replace_from(Text::new("reset").layout(TextLayout::default()))
            .unwrap();
        assert_eq!(
            slot.provided(),
            Some(
                &Text::new("reset")
                    .style(style.clone())
                    .layout(TextLayout::default())
            )
        );
        slot.edit_alignment(TextAlignment::Start);
        assert_eq!(
            slot.provided(),
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
    fn clearing_instructions_preserves_content_presence_and_does_not_encode_a_slot_command() {
        use super::{Text, TextAlignment, TextLayout};
        let before = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let style = crate::Theme::publication()
            .typography()
            .get_root()
            .get_title()
            .clone();
        let styled = Text::new("source")
            .style(style.clone())
            .alignment(TextAlignment::End);
        let plain = styled.clone().clear_style().clear_layout();
        assert_eq!(plain, Text::new("source"));
        assert_ne!(plain, Text::new("source").layout(TextLayout::default()));
        assert_eq!(styled.get_style(), Some(&style));
        assert_eq!(styled.alignment_request(), Some(TextAlignment::End));

        let mut slot = Text::UNSET;
        slot.edit_style(style.clone());
        slot.edit_layout(TextLayout::default());
        slot.edit_clear_style();
        slot.edit_clear_layout();
        assert!(slot.provided().is_none());
        assert_eq!(
            slot.resolve_source("automatic".into()).unwrap(),
            Text::new("automatic")
        );
        slot.replace_from(styled.clone()).unwrap();
        slot.replace_from(plain).unwrap();
        assert_eq!(
            slot.provided(),
            Some(&styled),
            "an unstyled value is omission, not a clearing command"
        );
        let retained = slot.clone();
        slot.edit_clear_style();
        assert_eq!(slot.get_layout(), styled.get_layout());
        slot.edit_clear_layout();
        slot.edit_clear_style();
        slot.replace_from(Text::new("changed")).unwrap();
        assert_eq!(slot.provided(), Some(&Text::new("changed")));
        assert_eq!(retained.provided(), Some(&styled));

        let layer =
            crate::PointLayer::new(NumericValues::new("x", [0.]), NumericValues::new("y", [1.]))
                .legend_label(styled.clone());
        let cleared = layer
            .clone()
            .clear_legend_label_style()
            .clear_legend_label_layout();
        assert_eq!(cleared.legend_content(), Some(&Text::new("source")));
        assert_eq!(layer.legend_content(), Some(&styled));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            before
        );
    }

    #[test]
    fn source_sharing_keeps_unset_and_explicit_empty_distinct() {
        use super::Text;
        use std::sync::Arc;

        let unset = Text::UNSET.try_copy().unwrap();
        let empty = Text::new("");
        assert!(!unset.is_provided());
        assert!(empty.is_provided());
        assert_ne!(unset, empty);
        assert_eq!(unset.as_str(), empty.as_str());
        assert_eq!(
            unset.resolve_source("automatic".into()).unwrap().as_str(),
            "automatic"
        );
        assert_eq!(empty.resolve_source("automatic".into()).unwrap(), empty);

        let mut source = String::with_capacity(128);
        source.push_str("verbatim\r\nsource ");
        let (pointer, capacity) = (source.as_ptr(), source.capacity());
        let mut text = Text::new(source);
        let retained = text.try_copy().unwrap();
        assert_eq!(text.as_str().as_ptr(), pointer);
        assert_eq!(text.source.as_ref().unwrap().capacity(), capacity);
        assert!(Arc::ptr_eq(
            text.source.as_ref().unwrap(),
            retained.source.as_ref().unwrap()
        ));
        assert_eq!(retained, Text::from("verbatim\r\nsource "));
        text.replace_from(Text::from("replacement")).unwrap();
        assert_eq!(retained.as_str(), "verbatim\r\nsource ");
        assert_eq!(text.as_str(), "replacement");
    }

    #[test]
    fn required_reception_rejects_unset_before_mutating_instructions() {
        use super::{Text, TextAlignment};

        let mut text = Text::new("retained").alignment(TextAlignment::End);
        let previous = text.clone();
        assert!(matches!(
            text.replace_from(Text::UNSET),
            Err(crate::Error::Render(_))
        ));
        assert_eq!(text, previous);
        assert!(matches!(
            Text::UNSET.required_source("required"),
            Err(crate::Error::Render(_))
        ));
        assert_eq!(Text::new("").required_source("required").unwrap(), "");
        assert!(matches!(
            Text::new("invalid\tcontrol").required_source("required"),
            Err(crate::Error::Data(_))
        ));
        assert!(TextPanel::new(Text::UNSET, "body").is_err());
        assert!(PlotTextAnnotation::new(DataAreaCorner::TopLeft, Text::UNSET).is_err());
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
            PlotTextAnnotation::new(DataAreaCorner::TopLeft, "a\tb").unwrap_err(),
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
