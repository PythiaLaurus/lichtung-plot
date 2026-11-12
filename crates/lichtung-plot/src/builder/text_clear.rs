//! Remove local slot instructions without replacing content or its presence.

use super::Plot;

macro_rules! clear_slot {
    ($style:ident, $layout:ident, $field:ident $(, $bound:ident)?) => {
        impl<K: crate::kind::Marker> Plot<K> {
            #[doc = concat!("Remove the local `", stringify!($field), "` appearance override; retain content/layout and inherit the effective Theme role.")]
            #[must_use]
            pub fn $style(mut self) -> Self $(where K: crate::kind::$bound)? {
                self.$field.edit_clear_style();
                self
            }
            #[doc = concat!("Remove the local `", stringify!($field), "` layout request; retain content/style and restore owner inheritance.")]
            #[must_use]
            pub fn $layout(mut self) -> Self $(where K: crate::kind::$bound)? {
                self.$field.edit_clear_layout();
                self
            }
        }
    };
}

clear_slot!(clear_title_style, clear_title_layout, title);
clear_slot!(clear_subtitle_style, clear_subtitle_layout, subtitle);
clear_slot!(clear_caption_style, clear_caption_layout, caption);
clear_slot!(clear_x_label_style, clear_x_label_layout, x_label);
clear_slot!(clear_y_label_style, clear_y_label_layout, y_label);
clear_slot!(
    clear_color_label_style,
    clear_color_label_layout,
    color_label,
    ContinuousColor
);
