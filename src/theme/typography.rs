//! Complete immutable typography construction and semantic role ownership.

use crate::Error;

use super::TextTheme;

// Each input is retained once. Constructors and consuming edits validate the
// entire group, including nested siblings, without copying font payloads.
macro_rules! text_group {
    (
        $(#[$attribute:meta])* $name:ident {
            $($field:ident: $type:ty => $get:ident),+ $(,)?
        }
        $(validate |$value:ident| $validation:block)?
    ) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name {
            $(pub(crate) $field: $type),+
        }

        impl $name {
            /// Construct all complete text roles and validate their invariants.
            pub fn new($($field: $type),+) -> Result<Self, Error> {
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
                #[doc = concat!("Borrow the independent `", stringify!($field), "` text role.")]
                pub fn $get(&self) -> &$type {
                    &self.$field
                }

                #[doc = concat!("Replace only `", stringify!($field), "`, validating the complete role group.")]
                pub fn $field(mut self, value: $type) -> Result<Self, Error> {
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
        title: TextTheme => get_title,
        subtitle: TextTheme => get_subtitle,
        caption: TextTheme => get_caption,
    }
}

text_group! {
    /// Complete coordinate-axis typography.
    AxisTextRoles {
        axis_title: TextTheme => get_axis_title,
        tick_label: TextTheme => get_tick_label,
    }
}

text_group! {
    /// Complete discrete and continuous guide typography.
    GuideTextRoles {
        legend_title: TextTheme => get_legend_title,
        legend_label: TextTheme => get_legend_label,
    }
}

text_group! {
    /// Independent facet, composition, and embedded-prose text roles.
    CompositionTextRoles {
        facet_header: TextTheme => get_facet_header,
        composition_header: TextTheme => get_composition_header,
        text_panel_title: TextTheme => get_text_panel_title,
        body: TextTheme => get_body,
    }
}

text_group! {
    /// Complete opaque candidates selected before font planning.
    ContrastTextRoles {
        light: TextTheme => get_light,
        dark: TextTheme => get_dark,
    }
    validate |value| {
        if value.light.get_color().get_alpha() != 255 || value.dark.get_color().get_alpha() != 255 {
            return Err(Error::Data("contrast text candidates must be opaque".into()));
        }
    }
}

text_group! {
    /// Complete annotation text, independent of mark and axis chrome.
    AnnotationTextRoles {
        significance_label: TextTheme => get_significance_label,
        estimate_detail: TextTheme => get_estimate_detail,
        waterfall_total: TextTheme => get_waterfall_total,
        point_layer_label: TextTheme => get_point_layer_label,
        panel_text: TextTheme => get_panel_text,
        contrast: ContrastTextRoles => get_contrast,
    }
}

text_group! {
    /// One complete retained typography value with all eighteen semantic roles.
    TypographyTheme {
        root: RootTextRoles => get_root,
        axes: AxisTextRoles => get_axes,
        guides: GuideTextRoles => get_guides,
        composition: CompositionTextRoles => get_composition,
        annotations: AnnotationTextRoles => get_annotations,
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
            *role = role.clone().stack(stack)?;
        }
        self.validate()?;
        Ok(self)
    }

    /// Stable traversal of each retained font handle, exactly once.
    pub(crate) fn roles(&self) -> [&TextTheme; 18] {
        [
            self.get_root().get_title(),
            self.get_root().get_subtitle(),
            self.get_root().get_caption(),
            self.get_axes().get_axis_title(),
            self.get_axes().get_tick_label(),
            self.get_annotations().get_significance_label(),
            self.get_annotations().get_estimate_detail(),
            self.get_annotations().get_waterfall_total(),
            self.get_annotations().get_point_layer_label(),
            self.get_annotations().get_panel_text(),
            self.get_annotations().get_contrast().get_light(),
            self.get_annotations().get_contrast().get_dark(),
            self.get_guides().get_legend_title(),
            self.get_guides().get_legend_label(),
            self.get_composition().get_facet_header(),
            self.get_composition().get_composition_header(),
            self.get_composition().get_text_panel_title(),
            self.get_composition().get_body(),
        ]
    }
}

#[cfg(test)]
mod tests;
