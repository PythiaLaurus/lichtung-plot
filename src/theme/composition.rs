//! Complete private composition chrome, with explicit owner-local metrics.

use super::spacing::ExactLength;
use super::{LocalMetric, RelativeSpacing, ResolvedTheme, TitleAlignment};
use crate::font::canonical::OperationGenerationResources;
use crate::layout::kernel::SpatialRounding;
use crate::{Error, Result};

macro_rules! spacing_group {
    ($name:ident, $lengths:ident; $($field:ident => $edit:ident: $metric:literal),+ $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        #[allow(clippy::struct_field_names, reason = "field names follow the authoritative complete composition schema")]
        pub(crate) struct $name { $($field: RelativeSpacing),+ }
        impl $name {
            #[allow(clippy::unnecessary_wraps, reason = "all supplied immutable relationships already validate nonnegative spacing")]
            pub(crate) fn new($($field: RelativeSpacing),+) -> Result<Self> { Ok(Self { $($field),+ }) }
            $(
                pub(crate) fn $field(&self) -> &RelativeSpacing { &self.$field }
                #[cfg_attr(not(test), allow(dead_code, reason = "private complete component edits await theme exposure"))]
                #[allow(clippy::unnecessary_wraps, reason = "immutable nonnegative relationships have no additional sibling constraint")]
                pub(crate) fn $edit(mut self, value: RelativeSpacing) -> Result<Self> {
                    self.$field = value; Ok(self)
                }
            )+
            fn relationships(&self) -> impl Iterator<Item = &RelativeSpacing> { [$(&self.$field),+].into_iter() }
            fn lengths(&self, metrics: &[LocalMetric], resources: &OperationGenerationResources) -> Result<$lengths> {
                Ok($lengths { $($field: self.$field().product(metrics[$metric], resources)?),+ })
            }
        }
        #[derive(Debug)]
        #[allow(clippy::struct_field_names, reason = "measured products retain their authoritative source field names")]
        pub(crate) struct $lengths { $(pub(crate) $field: ExactLength),+ }
    };
}

spacing_group!(RootTitleSpacing, RootTitleLengths;
    title_subtitle_gap => with_title_subtitle_gap: 0,
    title_block_to_content_clearance => with_title_block_to_content_clearance: 0,
    caption_to_content_clearance => with_caption_to_content_clearance: 1);
spacing_group!(RootPadding, RootPaddingLengths;
    left => with_left: 0, right => with_right: 0, bottom => with_bottom: 0, top => with_top: 0);
spacing_group!(TextPanelSpacing, TextPanelLengths;
    inset => with_inset: 0, gap => with_gap: 1);
spacing_group!(HeaderSpacing, HeaderLengths;
    column_header_padding => with_column_header_padding: 0,
    row_header_padding => with_row_header_padding: 0,
    row_title_horizontal_padding => with_row_title_horizontal_padding: 0,
    row_title_vertical_padding => with_row_title_vertical_padding: 0);
spacing_group!(FacetStripSpacing, FacetStripLengths;
    horizontal_inset => with_horizontal_inset: 0, content_clearance => with_content_clearance: 0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CompositionTheme {
    root_title_alignment: TitleAlignment,
    root_subtitle_alignment: Option<TitleAlignment>,
    root_caption_alignment: TitleAlignment,
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
    ($($field:ident => $edit:ident: $ty:ty),+ $(,)?) => { $(
        pub(crate) fn $field(&self) -> &$ty { &self.$field }
        #[cfg_attr(not(test), allow(dead_code, reason = "private complete component edits await theme exposure"))]
        #[allow(clippy::unnecessary_wraps, reason = "complete validated inputs have no additional cross-field constraint")]
        pub(crate) fn $edit(mut self, value: $ty) -> Result<Self> { self.$field = value; Ok(self) }
    )+ };
}
macro_rules! copy_fields {
    ($($field:ident => $edit:ident: $ty:ty),+ $(,)?) => { $(
        pub(crate) fn $field(&self) -> $ty { self.$field }
        #[cfg_attr(not(test), allow(dead_code, reason = "private complete component edits await theme exposure"))]
        #[allow(clippy::unnecessary_wraps, reason = "closed alignment inputs have no invalid states")]
        pub(crate) fn $edit(mut self, value: $ty) -> Result<Self> { self.$field = value; Ok(self) }
    )+ };
}

impl CompositionTheme {
    #[expect(
        clippy::too_many_arguments,
        clippy::unnecessary_wraps,
        reason = "the authoritative constructor accepts complete validated role groups and closed alignment values"
    )]
    pub(crate) fn new(
        root_title_alignment: TitleAlignment,
        root_subtitle_alignment: Option<TitleAlignment>,
        root_caption_alignment: TitleAlignment,
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
    copy_fields!(root_title_alignment => with_root_title_alignment: TitleAlignment,
        root_subtitle_alignment => with_root_subtitle_alignment: Option<TitleAlignment>,
        root_caption_alignment => with_root_caption_alignment: TitleAlignment);
    owned_fields!(root_title_block => with_root_title_block: RootTitleSpacing,
        root_padding => with_root_padding: RootPadding, panel_edge_clearance => with_panel_edge_clearance: RelativeSpacing,
        text_panel => with_text_panel: TextPanelSpacing, facet_gap => with_facet_gap: RelativeSpacing,
        canvas_grid_gap => with_canvas_grid_gap: RelativeSpacing, headers => with_headers: HeaderSpacing,
        facet_strip => with_facet_strip: FacetStripSpacing);

    pub(crate) fn builtin(
        t: u32,
        s: u32,
        c: u32,
        a: u32,
        resources: &OperationGenerationResources,
    ) -> Result<Self> {
        let ratio = |n, d| RelativeSpacing::ratio_in(n, d, resources);
        Self::new(
            TitleAlignment::Center,
            None,
            TitleAlignment::Start,
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
    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool> {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("composition identity: {error:?}")))?;
        if self.root_title_alignment != other.root_title_alignment
            || self.root_subtitle_alignment != other.root_subtitle_alignment
            || self.root_caption_alignment != other.root_caption_alignment
        {
            return Ok(false);
        }
        for (left, right) in self
            .retained_relationships()
            .zip(other.retained_relationships())
        {
            if !left.same(right, resources)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub(crate) fn hard(value: &ExactLength, resources: &OperationGenerationResources) -> Result<f64> {
    value.pixels_rounded(SpatialRounding::Ceil, resources)
}
pub(crate) fn capacity(
    width: f64,
    reserve: &ExactLength,
    resources: &OperationGenerationResources,
) -> Result<f64> {
    Ok(ExactLength::pixels(width.max(0.0), resources)?
        .subtract(reserve, resources)?
        .map(|value| value.pixels_rounded(SpatialRounding::Floor, resources))
        .transpose()?
        .unwrap_or(-1.0))
}

impl RootPaddingLengths {
    pub(crate) fn zero(resources: &OperationGenerationResources) -> Result<Self> {
        let zero = || {
            ExactLength::quanta(
                crate::layout::kernel::LayoutScalar::from_quanta(0),
                resources,
            )
        };
        Ok(Self {
            left: zero()?,
            right: zero()?,
            bottom: zero()?,
            top: zero()?,
        })
    }
    pub(crate) fn horizontal(
        &self,
        resources: &OperationGenerationResources,
    ) -> Result<ExactLength> {
        self.left.add(&self.right, resources)
    }
    pub(crate) fn vertical(&self, resources: &OperationGenerationResources) -> Result<ExactLength> {
        self.bottom.add(&self.top, resources)
    }
    pub(crate) fn inset(
        &self,
        span: crate::layout::PanelBox,
        resources: &OperationGenerationResources,
    ) -> Result<crate::layout::PanelBox> {
        Ok(crate::layout::PanelBox::new(
            self.left.after_pixels(span.x0, resources)?,
            self.right.before_pixels(span.x1, resources)?,
            self.bottom.after_pixels(span.y0, resources)?,
            self.top.before_pixels(span.y1, resources)?,
        ))
    }
}

impl ResolvedTheme {
    pub(crate) fn panel_edge_length(&self) -> Result<ExactLength> {
        self.composition.panel_edge_clearance().product(
            LocalMetric::css_pixels(self.typography.axes.tick_label.font_size())?,
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn embedded_minimum_height(&self, present: [bool; 3], body: f64) -> Result<f64> {
        let resources = self.prepared_font_environment()?.generation_resources();
        let spacing = self.root_title_lengths()?;
        let mut required = ExactLength::pixels(body, &resources)?;
        for (active, role) in present.into_iter().zip([
            &self.typography.root.title,
            &self.typography.root.subtitle,
            &self.typography.root.caption,
        ]) {
            if active {
                required = required.add(&role.line_pitch_exact(&resources)?, &resources)?;
            }
        }
        if present[0] && present[1] {
            required = required.add(&spacing.title_subtitle_gap, &resources)?;
        }
        if present[0] || present[1] {
            required = required.add(&spacing.title_block_to_content_clearance, &resources)?;
        }
        if present[2] {
            required = required.add(&spacing.caption_to_content_clearance, &resources)?;
        }
        if present.into_iter().any(|active| active) {
            required = required.add(
                &self.root_padding_lengths()?.vertical(&resources)?,
                &resources,
            )?;
        }
        hard(&required, &resources)
    }
    pub(crate) fn root_inner_width(&self, outer: f64) -> Result<f64> {
        let resources = self.prepared_font_environment()?.generation_resources();
        capacity(
            outer,
            &self.root_padding_lengths()?.horizontal(&resources)?,
            &resources,
        )
    }
    pub(crate) fn root_outer_width(&self, inner: f64) -> Result<f64> {
        let resources = self.prepared_font_environment()?.generation_resources();
        hard(
            &self
                .root_padding_lengths()?
                .horizontal(&resources)?
                .add_pixels(inner, &resources)?,
            &resources,
        )
    }
    pub(crate) fn root_outer_height(&self, inner: &ExactLength) -> Result<f64> {
        let resources = self.prepared_font_environment()?.generation_resources();
        hard(
            &self
                .root_padding_lengths()?
                .vertical(&resources)?
                .add(inner, &resources)?,
            &resources,
        )
    }
    pub(crate) fn root_padding_lengths(&self) -> Result<RootPaddingLengths> {
        self.composition.root_padding().lengths(
            &[LocalMetric::css_pixels(
                self.typography.composition.body.font_size(),
            )?],
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn root_title_lengths(&self) -> Result<RootTitleLengths> {
        self.composition.root_title_block().lengths(
            &[
                LocalMetric::css_pixels(
                    self.typography
                        .root
                        .title
                        .font_size()
                        .max(self.typography.root.subtitle.font_size()),
                )?,
                LocalMetric::css_pixels(self.typography.root.caption.font_size())?,
            ],
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn text_panel_lengths(&self) -> Result<TextPanelLengths> {
        let body = self.typography.composition.body.font_size();
        self.composition.text_panel().lengths(
            &[
                LocalMetric::css_pixels(body)?,
                LocalMetric::css_pixels(
                    body.max(self.typography.composition.text_panel_title.font_size()),
                )?,
            ],
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn header_lengths(&self) -> Result<HeaderLengths> {
        self.composition.headers().lengths(
            &[LocalMetric::css_pixels(
                self.typography.composition.composition_header.font_size(),
            )?],
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn facet_strip_lengths(&self) -> Result<FacetStripLengths> {
        self.composition.facet_strip().lengths(
            &[LocalMetric::css_pixels(
                self.typography.composition.facet_header.font_size(),
            )?],
            &self.prepared_font_environment()?.generation_resources(),
        )
    }
    pub(crate) fn composition_grid_gap(&self) -> Result<f64> {
        self.composition
            .canvas_grid_gap()
            .product(
                LocalMetric::css_pixels(self.typography.composition.body.font_size())?,
                &self.prepared_font_environment()?.generation_resources(),
            )?
            .pixels_rounded(
                SpatialRounding::NearestEven,
                &self.prepared_font_environment()?.generation_resources(),
            )
    }
    pub(crate) fn composition_facet_gap(&self) -> Result<f64> {
        self.composition
            .facet_gap()
            .product(
                LocalMetric::css_pixels(self.typography.composition.facet_header.font_size())?,
                &self.prepared_font_environment()?.generation_resources(),
            )?
            .pixels_rounded(
                SpatialRounding::NearestEven,
                &self.prepared_font_environment()?.generation_resources(),
            )
    }
}

impl super::TextTheme {
    pub(crate) fn line_pitch_exact(
        &self,
        resources: &OperationGenerationResources,
    ) -> Result<ExactLength> {
        RelativeSpacing::new_in(self.line_height(), resources)?
            .product(LocalMetric::css_pixels(self.font_size())?, resources)
    }
}

#[cfg(test)]
mod tests;

/// Independent fixed-integer construction certificate: headers, two operands and Euclidean steps.
#[cfg(test)]
pub(crate) fn preset_resource_oracle(preset: super::Theme) -> (u128, u64) {
    let (t, s, c, a) = match preset {
        super::Theme::Publication => (20, 15, 15, 17),
        super::Theme::Nature => (19, 14, 14, 16),
        super::Theme::Science => (21, 16, 15, 17),
        super::Theme::CHI => (19, 15, 15, 17),
    };
    builtin_resource_oracle(t, s, c, a)
}

#[cfg(test)]
pub(crate) fn builtin_resource_oracle(t: u32, s: u32, c: u32, a: u32) -> (u128, u64) {
    let sources = [
        (0, 1),
        (14, t.max(s)),
        (12, c),
        (16, a),
        (16, a),
        (8, a),
        (12, a),
        (4, 16),
        (16, a),
        (12, t.max(a)),
        (12, s),
        (12, a),
        (16, t),
        (20, t),
        (16, t),
        (8, t),
        (16, s),
        (8, s),
    ];
    let mut steps = 0_u128;
    for (mut n, mut d) in sources {
        while d != 0 {
            (n, d) = (d, n % d);
            steps += 1;
        }
    }
    let header = (size_of::<RelativeSpacing>()
        + 2 * size_of::<num_bigint::BigUint>()
        + size_of::<f64>()
        + 2 * size_of::<usize>()) as u64;
    let operand = 16 + size_of::<num_bigint::BigUint>() as u64;
    (54 + steps, 18 * (header + 2 * operand))
}
