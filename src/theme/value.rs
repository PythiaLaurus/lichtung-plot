//! Complete construction values, independent of per-operation resolution state.

use crate::font::canonical::FontStack;
use crate::Result;

use super::{
    AnnotationTheme, AxisTheme, ColorScales, CompositionTheme, LegendTheme, MarkTheme,
    ResolvedTheme, SurfaceTheme, TypographyTheme,
};

/// Complete immutable appearance, independent of fonts discovered during rendering.
///
/// Built-in constructors return ordinary complete values. Customize a cloned
/// value with [`Theme::customize`] and attach the built result to a Plot or Canvas.
///
/// ```
/// use lichtung_plot::{Color, Theme};
/// let base = Theme::publication();
/// let title = base.typography().root().title().clone()
///     .with_color(Color::hex("#194A75")?)?;
/// let root = base.typography().root().clone().with_title(title)?;
/// let typography = base.typography().clone().with_root(root)?;
/// let custom = base.customize().with_typography(typography).build()?;
/// assert_ne!(custom.typography(), base.typography());
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
///
/// Built-ins are complete values, not closed selector variants:
/// ```compile_fail
/// let _ = lichtung_plot::Theme::Publication;
/// ```
///
/// Resolution is private:
/// ```compile_fail
/// let _ = lichtung_plot::Theme::publication().resolve();
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub(crate) surfaces: SurfaceTheme,
    pub(crate) typography: TypographyTheme,
    pub(crate) axes: AxisTheme,
    pub(crate) legend: LegendTheme,
    pub(crate) composition: CompositionTheme,
    pub(crate) marks: MarkTheme,
    pub(crate) annotations: AnnotationTheme,
    pub(crate) color_scales: ColorScales,
}

/// Construction-only working copy of a complete [`Theme`].
#[derive(Clone, Debug)]
pub struct ThemeBuilder {
    value: Theme,
}

macro_rules! components {
    ($($field:ident: $ty:ty => $edit:ident),+ $(,)?) => {
        impl Theme {
            $(
                #[doc = concat!("Borrow the complete `", stringify!($field), "` component.")]
                pub fn $field(&self) -> &$ty { &self.$field }
            )+
        }
        impl ThemeBuilder {
            $(
                #[doc = concat!("Borrow the current complete `", stringify!($field), "` component.")]
                pub fn $field(&self) -> &$ty { self.value.$field() }
                #[doc = concat!("Replace the complete `", stringify!($field), "` construction input.")]
                #[must_use]
                pub fn $edit(mut self, value: $ty) -> Self {
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

impl Theme {
    #[expect(
        clippy::too_many_arguments,
        reason = "the complete constructor retains eight independently owned components"
    )]
    /// Construct all eight complete components without discovering fonts or data domains.
    pub fn new(
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

    /// Clean high-contrast publication appearance with generous whitespace.
    pub fn publication() -> Self {
        super::resolved::publication_theme().expect("validated publication constructor")
    }

    /// Muted blues and greens with bottom/left coordinate frame sides.
    pub fn nature() -> Self {
        super::resolved::nature_theme().expect("validated nature constructor")
    }

    /// High-saturation scientific palette and emphasized axes.
    pub fn science() -> Self {
        super::resolved::science_theme().expect("validated science constructor")
    }

    /// Accessible pastel appearance for HCI publications.
    pub fn chi() -> Self {
        super::resolved::chi_theme().expect("validated CHI constructor")
    }

    pub(crate) fn resolve(&self) -> Result<ResolvedTheme> {
        ResolvedTheme::new(self.clone())
    }

    /// Start a complete construction-only working copy, leaving this value unchanged.
    pub fn customize(&self) -> ThemeBuilder {
        ThemeBuilder::new(self.clone())
    }
}

impl ThemeBuilder {
    /// Consume a complete initial theme for construction edits.
    pub fn new(value: Theme) -> Self {
        Self { value }
    }

    /// Replace the stack in all eighteen roles once; later role edits take precedence normally.
    pub fn with_font_stack(mut self, stack: FontStack) -> Result<Self> {
        self.value.typography = self.value.typography.with_font_stack(stack)?;
        Ok(self)
    }

    /// Validate and return the complete immutable appearance.
    pub fn build(self) -> Result<Theme> {
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
        let source = Theme::publication();
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
        let local = crate::font::canonical::named_stack(&["Later title replacement"]);
        let title = builder
            .typography()
            .root()
            .title()
            .clone()
            .with_stack(local.clone())
            .unwrap();
        let root = builder
            .typography()
            .root()
            .clone()
            .with_title(title)
            .unwrap();
        let typography = builder.typography().clone().with_root(root).unwrap();
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
