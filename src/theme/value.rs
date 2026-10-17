//! Complete construction values, independent of per-operation resolution state.

use crate::font::canonical::FontStack;
use crate::Result;

use super::{
    AnnotationTheme, AxisTheme, ColorScales, CompositionTheme, LegendTheme, MarkTheme,
    ResolvedTheme, SurfaceTheme, TypographyTheme,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ThemeValue {
    pub(crate) surfaces: SurfaceTheme,
    pub(crate) typography: TypographyTheme,
    pub(crate) axes: AxisTheme,
    pub(crate) legend: LegendTheme,
    pub(crate) composition: CompositionTheme,
    pub(crate) marks: MarkTheme,
    pub(crate) annotations: AnnotationTheme,
    pub(crate) color_scales: ColorScales,
}

#[derive(Clone, Debug)]
pub(crate) struct ThemeBuilder {
    value: ThemeValue,
}

macro_rules! components {
    ($($field:ident: $ty:ty => $edit:ident),+ $(,)?) => {
        impl ThemeValue {
            $(
                #[allow(dead_code, reason = "complete private inspection precedes public Theme exposure")]
                pub(crate) fn $field(&self) -> &$ty { &self.$field }
            )+
        }
        #[allow(dead_code, reason = "private complete construction precedes the public customization cutover")]
        impl ThemeBuilder {
            $(
                pub(crate) fn $field(&self) -> &$ty { self.value.$field() }
                pub(crate) fn $edit(mut self, value: $ty) -> Self {
                    self.value.$field = value;
                    self
                }
            )+
        }
    };
}

components!(surfaces: SurfaceTheme => with_surfaces,
    typography: TypographyTheme => with_typography, axes: AxisTheme => with_axes,
    legend: LegendTheme => with_legend, composition: CompositionTheme => with_composition,
    marks: MarkTheme => with_marks, annotations: AnnotationTheme => with_annotations,
    color_scales: ColorScales => with_color_scales);

impl ThemeValue {
    #[expect(
        clippy::too_many_arguments,
        reason = "the complete constructor retains eight independently owned components"
    )]
    pub(crate) fn new(
        surfaces: SurfaceTheme,
        typography: TypographyTheme,
        axes: AxisTheme,
        legend: LegendTheme,
        composition: CompositionTheme,
        marks: MarkTheme,
        annotations: AnnotationTheme,
        color_scales: ColorScales,
    ) -> Result<Self> {
        let value = Self {
            surfaces,
            typography,
            axes,
            legend,
            composition,
            marks,
            annotations,
            color_scales,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        self.typography().validate()?;
        self.axes().validate()?;
        self.legend().validate_in()?;
        self.marks().validate()?;
        self.annotations().validate()
    }

    pub(crate) fn publication() -> Self {
        super::resolved::publication_theme().expect("validated publication constructor")
    }

    pub(crate) fn nature() -> Self {
        super::resolved::nature_theme().expect("validated nature constructor")
    }

    pub(crate) fn science() -> Self {
        super::resolved::science_theme().expect("validated science constructor")
    }

    pub(crate) fn chi() -> Self {
        super::resolved::chi_theme().expect("validated CHI constructor")
    }

    pub(crate) fn resolve(&self) -> Result<ResolvedTheme> {
        ResolvedTheme::new(self.clone())
    }

    #[allow(
        dead_code,
        reason = "private complete customization precedes public Theme exposure"
    )]
    pub(crate) fn customize(&self) -> ThemeBuilder {
        ThemeBuilder::new(self.clone())
    }
}

#[allow(
    dead_code,
    reason = "private complete construction precedes the public customization cutover"
)]
impl ThemeBuilder {
    pub(crate) fn new(value: ThemeValue) -> Self {
        Self { value }
    }

    pub(crate) fn with_font_stack(mut self, stack: FontStack) -> Result<Self> {
        self.value.typography = self.value.typography.with_font_stack(stack)?;
        Ok(self)
    }

    pub(crate) fn build(self) -> Result<ThemeValue> {
        self.value.validate()?;
        Ok(self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Color;

    #[test]
    fn construction_edits_preserve_source_and_bulk_font_is_not_a_runtime_override() {
        let source = ThemeValue::publication();
        let saved = source.clone();
        let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let stack = crate::font::canonical::named_stack(&["Complete construction font"]);
        let surfaces = source
            .surfaces()
            .clone()
            .with_canvas_root(Color::rgba(11, 23, 37, 53))
            .unwrap()
            .with_plot_panel(Color::rgba(41, 59, 73, 97))
            .unwrap();
        let builder = source
            .customize()
            .with_surfaces(surfaces)
            .with_font_stack(stack.clone())
            .unwrap();
        let mut typography = builder.typography().clone();
        let local = crate::font::canonical::named_stack(&["Later title replacement"]);
        typography.root.title = typography
            .root
            .title
            .clone()
            .with_stack(local.clone())
            .unwrap();
        let customized = builder.with_typography(typography).build().unwrap();
        assert_eq!(source, saved);
        assert_eq!(
            customized.surfaces().canvas_root(),
            Color::rgba(11, 23, 37, 53)
        );
        assert_eq!(
            customized.surfaces().plot_panel(),
            Color::rgba(41, 59, 73, 97)
        );
        for (index, (before, after)) in source
            .typography()
            .roles()
            .into_iter()
            .zip(customized.typography().roles())
            .enumerate()
        {
            assert_eq!(after.stack(), if index == 0 { &local } else { &stack });
            assert_eq!(after.color(), before.color());
            assert_eq!(after.font_size(), before.font_size());
            assert_eq!(after.weight(), before.weight());
            assert_eq!(
                after.line_height().to_bits(),
                before.line_height().to_bits()
            );
        }
        assert_eq!(customized.marks(), source.marks());
        let mut operation = customized.resolve().unwrap();
        operation.canvas_surface_override = Some(Color::rgb(1, 2, 3));
        operation.surfaces = source.surfaces().clone();
        assert_ne!(operation.surfaces, *customized.surfaces());
        assert_eq!(customized.typography().root.title.stack(), &local);
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            discoveries
        );
    }
}
