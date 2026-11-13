//! Short-lived text requests over operation-owned services and appearance.

use crate::theme::ResolvedTheme;

pub(crate) use crate::theme::TextService;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TextTrial<'a> {
    pub(crate) size_trace:
        Option<&'a std::sync::Arc<crate::layout::text::font_fit::stability::Stability>>,
    pub(crate) height: Option<i64>,
    pub(crate) padding: Option<&'a (crate::RelativeSpacing, crate::RelativeSpacing)>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextRequestContext<'v, 'src> {
    pub(crate) theme: &'v ResolvedTheme,
    pub(crate) text: &'v TextService,
    pub(crate) reference: Option<super::orientation::TextReferenceResolver<'v, 'src>>,
    pub(crate) trial: TextTrial<'v>,
}

impl<'v, 'src> TextRequestContext<'v, 'src> {
    pub(crate) fn new(theme: &'v ResolvedTheme, text: &'v TextService) -> Self {
        Self {
            theme,
            text,
            reference: None,
            trial: TextTrial::default(),
        }
    }

    pub(crate) fn with_text_projection<'a, 's>(
        &self,
        registered: &'a crate::orchestrator::registration::PlotRegistration<'s>,
        layout: &'a crate::layout::Layout,
        dimensions: crate::orchestrator::assignment::PlotMeasureDimensions,
    ) -> TextRequestContext<'a, 's>
    where
        'v: 'a,
    {
        TextRequestContext {
            theme: self.theme,
            text: self.text,
            trial: self.trial,
            reference: Some(super::orientation::TextReferenceResolver {
                registered,
                layout,
                dimensions,
            }),
        }
    }

    pub(crate) fn reproject_text<'a>(
        &self,
        layout: &'a crate::layout::Layout,
    ) -> TextRequestContext<'a, 'src>
    where
        'v: 'a,
    {
        self.reference.map_or(*self, |frame| {
            self.with_text_projection(frame.registered, layout, frame.dimensions)
        })
    }
}

impl std::ops::Deref for TextRequestContext<'_, '_> {
    type Target = ResolvedTheme;
    fn deref(&self) -> &Self::Target {
        self.theme
    }
}
