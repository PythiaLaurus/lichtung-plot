//! Complete immutable typography construction and semantic role ownership.

use crate::{Error, SignificanceMarkStyle};

use super::TextTheme;

// Each input is retained once. Constructors and consuming edits validate the
// entire group, including nested siblings, without copying font payloads.
macro_rules! text_group {
    (
        $(#[$attribute:meta])* $name:ident {
            $($field:ident: $type:ty => $edit:ident),+ $(,)?
        }
        $(validate |$value:ident| $validation:block)?
    ) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $name {
            $(pub(crate) $field: $type),+
        }

        impl $name {
            pub(crate) fn new($($field: $type),+) -> Result<Self, Error> {
                let value = Self { $($field),+ };
                value.validate()?;
                Ok(value)
            }

            pub(crate) fn validate(&self) -> Result<(), Error> {
                $(self.$field.validate()?;)+
                $(let $value = self; $validation)?
                Ok(())
            }

            $(
                pub(crate) fn $field(&self) -> &$type {
                    &self.$field
                }

                #[cfg_attr(not(test), expect(dead_code, reason = "private construction edits await complete Theme exposure"))]
                pub(crate) fn $edit(mut self, value: $type) -> Result<Self, Error> {
                    self.$field = value;
                    self.validate()?;
                    Ok(self)
                }
            )+
        }
    };
}

text_group! {
    /// Independent root and embedded Plot headings.
    RootTextRoles {
        title: TextTheme => with_title,
        subtitle: TextTheme => with_subtitle,
        caption: TextTheme => with_caption,
    }
}

text_group! {
    /// Complete coordinate-axis typography.
    AxisTextRoles {
        axis_title: TextTheme => with_axis_title,
        tick_label: TextTheme => with_tick_label,
    }
}

text_group! {
    /// Complete discrete and continuous guide typography.
    GuideTextRoles {
        legend_title: TextTheme => with_legend_title,
        legend_label: TextTheme => with_legend_label,
    }
}

text_group! {
    /// Independent facet, composition, and embedded-prose text roles.
    CompositionTextRoles {
        facet_header: TextTheme => with_facet_header,
        composition_header: TextTheme => with_composition_header,
        text_panel_title: TextTheme => with_text_panel_title,
        body: TextTheme => with_body,
    }
}

text_group! {
    /// Complete opaque candidates selected before font planning.
    ContrastTextRoles {
        light: TextTheme => with_light,
        dark: TextTheme => with_dark,
    }
    validate |value| {
        if value.light.color().alpha() != 255 || value.dark.color().alpha() != 255 {
            return Err(Error::Data("contrast text candidates must be opaque".into()));
        }
    }
}

text_group! {
    /// Complete annotation text, independent of mark and axis chrome.
    AnnotationTextRoles {
        significance_label: TextTheme => with_significance_label,
        estimate_detail: TextTheme => with_estimate_detail,
        waterfall_total: TextTheme => with_waterfall_total,
        point_layer_label: TextTheme => with_point_layer_label,
        panel_text: TextTheme => with_panel_text,
        contrast: ContrastTextRoles => with_contrast,
    }
}

text_group! {
    /// One complete retained typography value with all eighteen semantic roles.
    TypographyTheme {
        root: RootTextRoles => with_root,
        axes: AxisTextRoles => with_axes,
        guides: GuideTextRoles => with_guides,
        composition: CompositionTextRoles => with_composition,
        annotations: AnnotationTextRoles => with_annotations,
    }
}

impl TypographyTheme {
    pub(crate) fn with_font_stack(
        mut self,
        stack: crate::font::canonical::FontStack,
    ) -> Result<Self, Error> {
        let roles = [
            &mut self.root.title,
            &mut self.root.subtitle,
            &mut self.root.caption,
            &mut self.axes.axis_title,
            &mut self.axes.tick_label,
            &mut self.guides.legend_title,
            &mut self.guides.legend_label,
            &mut self.composition.facet_header,
            &mut self.composition.composition_header,
            &mut self.composition.text_panel_title,
            &mut self.composition.body,
            &mut self.annotations.significance_label,
            &mut self.annotations.estimate_detail,
            &mut self.annotations.waterfall_total,
            &mut self.annotations.point_layer_label,
            &mut self.annotations.panel_text,
            &mut self.annotations.contrast.light,
            &mut self.annotations.contrast.dark,
        ];
        let stacks =
            std::iter::repeat_n(stack.clone(), roles.len() - 1).chain(std::iter::once(stack));
        for (role, stack) in roles.into_iter().zip(stacks) {
            *role = role.clone().with_stack(stack)?;
        }
        self.validate()?;
        Ok(self)
    }

    /// Stable traversal of each retained font handle, exactly once.
    pub(crate) fn roles(&self) -> [&TextTheme; 18] {
        [
            self.root().title(),
            self.root().subtitle(),
            self.root().caption(),
            self.axes().axis_title(),
            self.axes().tick_label(),
            self.annotations().significance_label(),
            self.annotations().estimate_detail(),
            self.annotations().waterfall_total(),
            self.annotations().point_layer_label(),
            self.annotations().panel_text(),
            self.annotations().contrast().light(),
            self.annotations().contrast().dark(),
            self.guides().legend_title(),
            self.guides().legend_label(),
            self.composition().facet_header(),
            self.composition().composition_header(),
            self.composition().text_panel_title(),
            self.composition().body(),
        ]
    }

    /// Apply exact significance size and weight over the selected complete role.
    pub(crate) fn effective_significance_label(
        &self,
        exact: Option<SignificanceMarkStyle>,
    ) -> Result<TextTheme, Error> {
        let mut effective = self.annotations().significance_label().try_copy()?;
        if let Some(exact) = exact {
            effective = effective
                .with_font_size(exact.font_size_spec())?
                .with_weight(exact.weight_spec())?;
        }
        Ok(effective)
    }
}

#[cfg(test)]
mod tests;
