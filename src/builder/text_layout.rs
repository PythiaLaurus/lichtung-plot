//! Typed attachments at real text owners, retained through explicit erasure.

use super::Plot;
use crate::{GuideTextRole, TextLayout};

macro_rules! role {
    ($name:ident,$bound:ident,$field:ident) => {impl<K:crate::kind::$bound> Plot<K> {
        #[doc=concat!("Replace the `",stringify!($field),"` text owner's complete layout default.")]
        #[must_use]
        pub fn $name(mut self,layout:TextLayout) -> Self {self.text_layouts.$field=Some(layout);self}
    }};
    ($name:ident,$bound:ident,$field:ident,$index:expr) => {impl<K:crate::kind::$bound> Plot<K> {
        #[doc=concat!("Replace the `",stringify!($field),"` text owner's complete layout default.")]
        #[must_use]
        pub fn $name(mut self,layout:TextLayout) -> Self {self.text_layouts.$field[$index]=Some(layout);self}
    }};
}
role!(x_tick_text_layout, XAxisText, ticks, 0);
role!(y_tick_text_layout, YAxisText, ticks, 1);
role!(facet_row_text_layout, Facet, facet, 0);
role!(facet_column_text_layout, Facet, facet, 1);
role!(estimate_detail_text_layout, EstimateDetailText, estimate);
role!(significance_text_layout, SignificanceStyle, significance);

impl<K: crate::kind::Legend> Plot<K> {
    /// Replace one actual guide role's default; authored whole requests take precedence.
    #[must_use]
    pub fn guide_text_layout(mut self, role: GuideTextRole, layout: TextLayout) -> Self {
        self.text_layouts.guide[role.index()] = Some(layout);
        self
    }
}
impl Plot<crate::kind::Matrix> {
    /// Configure the actual left-edge variable titles, not cell ticks.
    #[must_use]
    pub fn matrix_row_text_layout(mut self, layout: TextLayout) -> Self {
        self.text_layouts.matrix[0] = Some(layout);
        self
    }
    /// Configure the actual bottom-edge variable titles, not cell ticks.
    #[must_use]
    pub fn matrix_column_text_layout(mut self, layout: TextLayout) -> Self {
        self.text_layouts.matrix[1] = Some(layout);
        self
    }
}
impl Plot<crate::kind::Heatmap> {
    /// Configure active finite cell value text, retaining independent contrast roles.
    #[must_use]
    pub fn value_text_layout(mut self, layout: TextLayout) -> Self {
        self.text_layouts.value = Some(layout);
        self
    }
    /// Configure fully-null row text; authored Text requests replace this default.
    #[must_use]
    pub fn null_row_text_layout(mut self, layout: TextLayout) -> Self {
        self.text_layouts.null_row = Some(layout);
        self
    }
}
impl Plot<crate::kind::Waterfall> {
    /// Configure eligible total labels without changing their masks or scientific anchors.
    #[must_use]
    pub fn total_text_layout(mut self, layout: TextLayout) -> Self {
        self.text_layouts.total = Some(layout);
        self
    }
}
