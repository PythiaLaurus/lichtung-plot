//! Canvas-root prose shares TextSlot's independent instruction removal.

use super::Canvas;

macro_rules! clear_slot {
    ($style:ident, $layout:ident, $field:ident) => {
        impl Canvas {
            #[doc = concat!("Remove the local root `", stringify!($field), "` appearance override without changing content/layout or child plots.")]
            #[must_use]
            pub fn $style(mut self) -> Self {
                self.$field.clear_style();
                self
            }
            #[doc = concat!("Remove the local root `", stringify!($field), "` layout request without changing content/style or child plots.")]
            #[must_use]
            pub fn $layout(mut self) -> Self {
                self.$field.clear_layout();
                self
            }
        }
    };
}

clear_slot!(clear_title_style, clear_title_layout, title);
clear_slot!(clear_subtitle_style, clear_subtitle_layout, subtitle);
clear_slot!(clear_caption_style, clear_caption_layout, caption);
