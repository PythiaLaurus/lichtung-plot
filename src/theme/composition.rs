//! Complete private composition chrome, with explicit owner-local metrics.

use super::spacing::ExactLength;
use super::{LocalMetric, RelativeSpacing, ResolvedTheme};
use crate::layout::kernel::SpatialRounding;
use crate::{Result, TextAlignment};

macro_rules! spacing_group {
    ($name:ident, $lengths:ident; $($field:ident => $get:ident: $metric:literal),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[allow(clippy::struct_field_names, reason = "field names follow the authoritative complete composition schema")]
        #[doc = concat!("Complete nonnegative owner-local `", stringify!($name), "` relationships.")]
        pub struct $name { $($field: RelativeSpacing),+ }
        impl $name {
            #[allow(clippy::unnecessary_wraps, reason = "all supplied immutable relationships already validate nonnegative spacing")]
            /// Construct all complete nonnegative local relationships.
            pub fn new($($field: RelativeSpacing),+) -> Result<Self> { Ok(Self { $($field),+ }) }
            $(
                #[doc = concat!("Borrow the exact `", stringify!($field), "` relationship.")]
                pub fn $get(&self) -> &RelativeSpacing { &self.$field }
                #[allow(clippy::unnecessary_wraps, reason = "immutable nonnegative relationships have no additional sibling constraint")]
                #[doc = concat!("Replace only the exact `", stringify!($field), "` relationship.")]
                pub fn $field(mut self, value: RelativeSpacing) -> Result<Self> {
                    self.$field = value; Ok(self)
                }
            )+
            fn relationships(&self) -> impl Iterator<Item = &RelativeSpacing> { [$(&self.$field),+].into_iter() }
            fn lengths(&self, metrics: &[LocalMetric]) -> Result<$lengths> {
                Ok($lengths { $($field: self.$get().product(metrics[$metric])),+ })
            }
        }
        #[derive(Debug)]
        #[allow(clippy::struct_field_names, reason = "measured products retain their authoritative source field names")]
        pub(crate) struct $lengths { $(pub(crate) $field: ExactLength),+ }
    };
}

spacing_group!(RootTitleSpacing, RootTitleLengths;
    title_subtitle_gap => get_title_subtitle_gap: 0,
    title_block_to_content_clearance => get_title_block_to_content_clearance: 0,
    caption_to_content_clearance => get_caption_to_content_clearance: 1);
spacing_group!(RootPadding, RootPaddingLengths;
    left => get_left: 0, right => get_right: 0, bottom => get_bottom: 0, top => get_top: 0);
spacing_group!(TextPanelSpacing, TextPanelLengths;
    inset => get_inset: 0, gap => get_gap: 1);
spacing_group!(HeaderSpacing, HeaderLengths;
    column_header_padding => get_column_header_padding: 0,
    row_header_padding => get_row_header_padding: 0,
    row_title_horizontal_padding => get_row_title_horizontal_padding: 0,
    row_title_vertical_padding => get_row_title_vertical_padding: 0);
spacing_group!(FacetStripSpacing, FacetStripLengths;
    horizontal_inset => get_horizontal_inset: 0, content_clearance => get_content_clearance: 0);

/// Complete owner-local composition, header, facet and root-prose appearance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionTheme {
    root_title_alignment: TextAlignment,
    root_subtitle_alignment: Option<TextAlignment>,
    root_caption_alignment: TextAlignment,
    root_title_block: RootTitleSpacing,
    root_padding: RootPadding,
    panel_edge_clearance: RelativeSpacing,
    text_panel: TextPanelSpacing,
    facet_gap: RelativeSpacing,
    canvas_grid_gap: RelativeSpacing,
    headers: HeaderSpacing,
    facet_strip: FacetStripSpacing,
}

macro_rules! owned_fields {
    ($($field:ident => $get:ident: $ty:ty),+ $(,)?) => { $(
        #[doc = concat!("Borrow the complete `", stringify!($field), "` relationship value.")]
        pub fn $get(&self) -> &$ty { &self.$field }
        #[allow(clippy::unnecessary_wraps, reason = "complete validated inputs have no additional cross-field constraint")]
        #[doc = concat!("Replace only `", stringify!($field), "`.")]
        pub fn $field(mut self, value: $ty) -> Result<Self> { self.$field = value; Ok(self) }
    )+ };
}
macro_rules! copy_fields {
    ($($field:ident => $get:ident: $ty:ty),+ $(,)?) => { $(
        #[doc = concat!("Inspect `", stringify!($field), "`.")]
        pub fn $get(&self) -> $ty { self.$field }
        #[allow(clippy::unnecessary_wraps, reason = "closed alignment inputs have no invalid states")]
        #[doc = concat!("Replace only `", stringify!($field), "`.")]
        pub fn $field(mut self, value: $ty) -> Result<Self> { self.$field = value; Ok(self) }
    )+ };
}

impl CompositionTheme {
    #[expect(
        clippy::too_many_arguments,
        reason = "the authoritative constructor accepts complete validated role groups and closed alignment values"
    )]
    /// Construct all root alignments and independently owned local relationships.
    pub fn new(
        root_title_alignment: TextAlignment,
        root_subtitle_alignment: Option<TextAlignment>,
        root_caption_alignment: TextAlignment,
        root_title_block: RootTitleSpacing,
        root_padding: RootPadding,
        panel_edge_clearance: RelativeSpacing,
        text_panel: TextPanelSpacing,
        facet_gap: RelativeSpacing,
        canvas_grid_gap: RelativeSpacing,
        headers: HeaderSpacing,
        facet_strip: FacetStripSpacing,
    ) -> Result<Self> {
        Ok(Self {
            root_title_alignment,
            root_subtitle_alignment,
            root_caption_alignment,
            root_title_block,
            root_padding,
            panel_edge_clearance,
            text_panel,
            facet_gap,
            canvas_grid_gap,
            headers,
            facet_strip,
        })
    }
    copy_fields!(root_title_alignment => get_root_title_alignment: TextAlignment,
        root_subtitle_alignment => get_root_subtitle_alignment: Option<TextAlignment>,
        root_caption_alignment => get_root_caption_alignment: TextAlignment);
    owned_fields!(root_title_block => get_root_title_block: RootTitleSpacing,
        root_padding => get_root_padding: RootPadding, panel_edge_clearance => get_panel_edge_clearance: RelativeSpacing,
        text_panel => get_text_panel: TextPanelSpacing, facet_gap => get_facet_gap: RelativeSpacing,
        canvas_grid_gap => get_canvas_grid_gap: RelativeSpacing, headers => get_headers: HeaderSpacing,
        facet_strip => get_facet_strip: FacetStripSpacing);

    pub(crate) fn builtin(t: u32, s: u32, c: u32, a: u32) -> Result<Self> {
        let ratio = |n, d| RelativeSpacing::ratio(n, d);
        Self::new(
            TextAlignment::Center,
            None,
            TextAlignment::Start,
            RootTitleSpacing::new(ratio(0, 1)?, ratio(14, t.max(s))?, ratio(12, c)?)?,
            RootPadding::new(ratio(16, a)?, ratio(16, a)?, ratio(8, a)?, ratio(12, a)?)?,
            ratio(4, 16)?,
            TextPanelSpacing::new(ratio(16, a)?, ratio(12, t.max(a))?)?,
            ratio(12, s)?,
            ratio(12, a)?,
            HeaderSpacing::new(ratio(16, t)?, ratio(20, t)?, ratio(16, t)?, ratio(8, t)?)?,
            FacetStripSpacing::new(ratio(16, s)?, ratio(8, s)?)?,
        )
    }
    pub(crate) fn retained_relationships(&self) -> impl Iterator<Item = &RelativeSpacing> {
        self.root_title_block
            .relationships()
            .chain(self.root_padding.relationships())
            .chain([&self.panel_edge_clearance])
            .chain(self.text_panel.relationships())
            .chain([&self.facet_gap, &self.canvas_grid_gap])
            .chain(self.headers.relationships())
            .chain(self.facet_strip.relationships())
    }
    pub(crate) fn same(&self, other: &Self) -> bool {
        if self.root_title_alignment != other.root_title_alignment
            || self.root_subtitle_alignment != other.root_subtitle_alignment
            || self.root_caption_alignment != other.root_caption_alignment
        {
            return false;
        }
        for (left, right) in self
            .retained_relationships()
            .zip(other.retained_relationships())
        {
            if !left.same(right) {
                return false;
            }
        }
        true
    }
}

pub(crate) fn hard(value: &ExactLength) -> Result<f64> {
    value.pixels_rounded(SpatialRounding::Ceil)
}
pub(crate) fn capacity(width: f64, reserve: &ExactLength) -> Result<f64> {
    Ok(ExactLength::pixels(width.max(0.0))?
        .subtract(reserve)
        .map(|value| value.pixels_rounded(SpatialRounding::Floor))
        .transpose()?
        .unwrap_or(-1.0))
}

impl RootPaddingLengths {
    pub(crate) fn zero() -> Result<Self> {
        let zero = || ExactLength::quanta(crate::layout::kernel::LayoutScalar::from_quanta(0));
        Ok(Self {
            left: zero()?,
            right: zero()?,
            bottom: zero()?,
            top: zero()?,
        })
    }
    pub(crate) fn horizontal(&self) -> ExactLength {
        self.left.add_length(&self.right)
    }
    pub(crate) fn vertical(&self) -> ExactLength {
        self.bottom.add_length(&self.top)
    }
    pub(crate) fn inset(&self, span: crate::layout::PanelBox) -> Result<crate::layout::PanelBox> {
        Ok(crate::layout::PanelBox::new(
            self.left.after_pixels(span.x0)?,
            self.right.before_pixels(span.x1)?,
            self.bottom.after_pixels(span.y0)?,
            self.top.before_pixels(span.y1)?,
        ))
    }
}

impl ResolvedTheme {
    pub(crate) fn panel_edge_length(&self) -> Result<ExactLength> {
        Ok(self
            .composition
            .get_panel_edge_clearance()
            .product(LocalMetric::css_pixels(
                self.typography.axes.tick_label.get_font_size(),
            )?))
    }
    pub(crate) fn embedded_minimum_height(&self, present: [bool; 3], body: f64) -> Result<f64> {
        let spacing = self.root_title_lengths()?;
        let mut required = ExactLength::pixels(body)?;
        for (active, role) in present.into_iter().zip([
            &self.typography.root.title,
            &self.typography.root.subtitle,
            &self.typography.root.caption,
        ]) {
            if active {
                required = required.add_length(&role.line_pitch_exact()?);
            }
        }
        if present[0] && present[1] {
            required = required.add_length(&spacing.title_subtitle_gap);
        }
        if present[0] || present[1] {
            required = required.add_length(&spacing.title_block_to_content_clearance);
        }
        if present[2] {
            required = required.add_length(&spacing.caption_to_content_clearance);
        }
        if present.into_iter().any(|active| active) {
            required = required.add_length(&self.root_padding_lengths()?.vertical());
        }
        hard(&required)
    }
    pub(crate) fn root_inner_width(&self, outer: f64) -> Result<f64> {
        capacity(outer, &self.root_padding_lengths()?.horizontal())
    }
    pub(crate) fn root_outer_width(&self, inner: f64) -> Result<f64> {
        hard(
            &self
                .root_padding_lengths()?
                .horizontal()
                .add_pixels(inner)?,
        )
    }
    pub(crate) fn root_outer_height(&self, inner: &ExactLength) -> Result<f64> {
        hard(&self.root_padding_lengths()?.vertical().add_length(inner))
    }
    pub(crate) fn root_padding_lengths(&self) -> Result<RootPaddingLengths> {
        self.composition
            .get_root_padding()
            .lengths(&[LocalMetric::css_pixels(
                self.typography.composition.body.get_font_size(),
            )?])
    }
    pub(crate) fn root_title_lengths(&self) -> Result<RootTitleLengths> {
        self.composition.get_root_title_block().lengths(&[
            LocalMetric::css_pixels(
                self.typography
                    .root
                    .title
                    .get_font_size()
                    .max(self.typography.root.subtitle.get_font_size()),
            )?,
            LocalMetric::css_pixels(self.typography.root.caption.get_font_size())?,
        ])
    }
    pub(crate) fn text_panel_lengths(&self, title: u32, body: u32) -> Result<TextPanelLengths> {
        self.composition.get_text_panel().lengths(&[
            LocalMetric::css_pixels(body)?,
            LocalMetric::css_pixels(body.max(title))?,
        ])
    }
    pub(crate) fn header_lengths(&self) -> Result<HeaderLengths> {
        self.composition
            .get_headers()
            .lengths(&[LocalMetric::css_pixels(
                self.typography
                    .composition
                    .composition_header
                    .get_font_size(),
            )?])
    }
    pub(crate) fn facet_strip_lengths(&self) -> Result<FacetStripLengths> {
        self.composition
            .get_facet_strip()
            .lengths(&[LocalMetric::css_pixels(
                self.typography.composition.facet_header.get_font_size(),
            )?])
    }
    pub(crate) fn composition_grid_gap(&self) -> Result<f64> {
        self.composition
            .get_canvas_grid_gap()
            .product(LocalMetric::css_pixels(
                self.typography.composition.body.get_font_size(),
            )?)
            .pixels_rounded(SpatialRounding::NearestEven)
    }
    pub(crate) fn composition_facet_gap(&self) -> Result<f64> {
        self.composition
            .get_facet_gap()
            .product(LocalMetric::css_pixels(
                self.typography.composition.facet_header.get_font_size(),
            )?)
            .pixels_rounded(SpatialRounding::NearestEven)
    }
}

impl super::TextTheme {
    pub(crate) fn line_pitch_exact(&self) -> Result<ExactLength> {
        Ok(RelativeSpacing::new(self.get_line_height())?
            .product(LocalMetric::css_pixels(self.get_font_size())?))
    }
}

#[cfg(test)]
mod tests;
