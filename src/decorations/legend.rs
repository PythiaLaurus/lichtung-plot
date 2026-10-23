//! `lichtung-plot::decorations::legend` — plot legend rendering.
//!
//! Coordinates use the **math coordinate system** (y increases upward).
//! The SVG backend flips y internally via `y_mark()`. Therefore, to lay
//! out elements "top-to-bottom" visually, we *decrement* y_offset.
//!
//! ## Sizing philosophy
//!
//! Resolved legend text and chrome come from the owning theme. Measurement
//! responds to the assigned guide width and height, never a root-size font scale.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_precision_loss
)]

use crate::layout::TextBlock;
use crate::render::backend::{
    Backend, FillStyle, LineStyle, PlannedTextPlacement, Point2D, PointStyle, TextAnchor,
};
use crate::Error;

pub(crate) mod alpha;
mod continuous;
pub(crate) mod discrete;
mod identity;
mod measurement;
pub(crate) mod source;
pub(crate) use identity::GuideEntryIdentity;
pub(crate) use source::GuideSourceKey;
pub(crate) mod descriptor;
use discrete::PreparedGuide;
pub(crate) use measurement::GuideMeasurements;

#[derive(Debug)]
pub(crate) enum LegendCall {
    Background([f64; 5], FillStyle),
    Gradient(
        crate::layout::PanelBox,
        Point2D,
        Point2D,
        Vec<crate::render::backend::GradientStop>,
    ),
    Fill([Point2D; 4], FillStyle),
    Line([Point2D; 2], LineStyle),
    Point([Point2D; 1], PointStyle),
    Text(crate::render::backend::PlacedText),
}

impl LegendCall {
    fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
    ) -> Result<(), Error> {
        match self {
            Self::Background(rect, style) => geometry.rounded_rect(*rect, style, None),
            Self::Gradient(bounds, start, end, stops) => {
                geometry.linear_gradient(*bounds, *start, *end, stops, None)
            }
            Self::Fill(points, style) => geometry.polygon(points, style, None),
            Self::Line(points, style) => geometry.line(points, style, None),
            Self::Point(points, style) => geometry.scatter_with_clip(points, style, None),
            Self::Text(text) => geometry.text(text.placement, &text.plan, None),
        }
    }

    fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        match self {
            Self::Background([x, y, width, height, radius], style) => {
                backend.draw_rounded_rect(*x, *y, *width, *height, *radius, style)
            }
            Self::Gradient(bounds, start, end, stops) => {
                backend.draw_linear_gradient(*bounds, *start, *end, stops)
            }
            Self::Fill(points, style) => backend.draw_polygon(points, style),
            Self::Line(points, style) => backend.draw_line(points, style),
            Self::Point(points, style) => backend.draw_scatter(points, style),
            Self::Text(text) => text.draw(backend),
        }
    }
}

/// Assigned legend operands, including glyph and text placement.
#[derive(Debug)]
pub(crate) struct LegendDrawPlan {
    pub(crate) calls: Vec<LegendCall>,
}

impl LegendDrawPlan {
    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        for call in &self.calls {
            call.draw(backend)?;
        }
        Ok(())
    }

    pub(crate) fn geometry(
        &self,
        geometry: &mut crate::render::geometry::GeometrySnapshot,
    ) -> Result<(), Error> {
        for call in &self.calls {
            call.geometry(geometry)?;
        }
        Ok(())
    }
}

pub(crate) fn address_role(position: crate::LegendPosition) -> crate::layout::AddressRole {
    use crate::layout::AddressRole;
    use crate::LegendPosition::{
        Auto, OutsideBottom, OutsideBottomLeft, OutsideBottomRight, OutsideLeft, OutsideLeftBottom,
        OutsideLeftTop, OutsideRight, OutsideRightBottom, OutsideRightTop, OutsideTop,
        OutsideTopLeft, OutsideTopRight,
    };
    match position {
        Auto => AddressRole::OutsideGuideAuto,
        OutsideTopLeft | OutsideTop | OutsideTopRight => AddressRole::OutsideGuideTop,
        OutsideRightTop | OutsideRight | OutsideRightBottom => AddressRole::OutsideGuideRight,
        OutsideBottomLeft | OutsideBottom | OutsideBottomRight => AddressRole::OutsideGuideBottom,
        OutsideLeftTop | OutsideLeft | OutsideLeftBottom => AddressRole::OutsideGuideLeft,
        _ => AddressRole::InsideGuide,
    }
}

/// Exact visible components of one legend glyph.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResolvedGuideKey {
    point: Option<PointStyle>,
    line: Option<LineStyle>,
    fill: Option<FillStyle>,
    explanation: source::GuideExplanation,
}

impl PartialEq for ResolvedGuideKey {
    fn eq(&self, other: &Self) -> bool {
        optional_point_style_eq(self.point.as_ref(), other.point.as_ref())
            && optional_line_style_eq(self.line.as_ref(), other.line.as_ref())
            && optional_fill_style_eq(self.fill.as_ref(), other.fill.as_ref())
            && self.explanation == other.explanation
    }
}
impl Eq for ResolvedGuideKey {}

impl ResolvedGuideKey {
    fn mismatched_component(&self, other: &Self) -> &'static str {
        if !optional_point_style_eq(self.point.as_ref(), other.point.as_ref()) {
            "point component"
        } else if !optional_line_style_eq(self.line.as_ref(), other.line.as_ref()) {
            "line component"
        } else if !optional_fill_style_eq(self.fill.as_ref(), other.fill.as_ref()) {
            "fill component"
        } else {
            "explanation mask"
        }
    }

    #[cfg(test)]
    pub(crate) fn point(point: PointStyle) -> Self {
        let alpha = source::AlphaExplanationMask {
            point_fill: Some(source::AlphaExplanation::Constant),
            point_outline: point
                .outline
                .as_ref()
                .map(|_| source::AlphaExplanation::Constant),
            ..source::AlphaExplanationMask::default()
        };
        Self {
            point: Some(point),
            explanation: source::GuideExplanation::Complete(alpha),
            ..Self::default()
        }
    }

    fn glyph_ink(
        &self,
        metrics: &LegendMetrics,
    ) -> Result<crate::render::geometry::GeometryBox, Error> {
        let half_width = metrics.handle_w / 2.0;
        let half_height = metrics.font_size * 0.32;
        let mut ink = crate::render::geometry::GeometrySnapshot::new();
        // Alpha never removes a component's structural key occupancy. Geometry
        // and selected drawing share the same grammar and canonical materializer.
        let mut occupied = self.clone();
        if let Some(fill) = &mut occupied.fill {
            fill.color = fill.color.alpha(255);
            fill.stroke = fill.stroke.alpha(255);
        }
        if let Some(line) = &mut occupied.line {
            line.color = line.color.alpha(255);
        }
        if let Some(point) = &mut occupied.point {
            point.color = point.color.alpha(255);
            if let Some(outline) = &mut point.outline {
                outline.color = outline.color.alpha(255);
            }
        }
        occupied.visit_calls(metrics, Point2D::new(0.0, 0.0), |call| {
            call.geometry(&mut ink)
        })?;

        Ok(ink.bounds().iter().copied().fold(
            crate::render::geometry::GeometryBox {
                left: -half_width,
                right: half_width,
                bottom: -half_height,
                top: half_height,
                weight: 1.0,
            },
            crate::render::geometry::GeometryBox::union,
        ))
    }

    fn visit_calls(
        &self,
        metrics: &LegendMetrics,
        center: Point2D,
        mut emit: impl FnMut(LegendCall) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let half_width = metrics.handle_w / 2.0;
        if let Some(fill) = &self.fill {
            let half_height = metrics.font_size * 0.32;
            emit(LegendCall::Fill(
                [
                    Point2D::new(center.x - half_width, center.y - half_height),
                    Point2D::new(center.x + half_width, center.y - half_height),
                    Point2D::new(center.x + half_width, center.y + half_height),
                    Point2D::new(center.x - half_width, center.y + half_height),
                ],
                fill.clone(),
            ))?;
        }
        if let Some(line) = &self.line {
            emit(LegendCall::Line(
                [
                    Point2D::new(center.x - half_width, center.y),
                    Point2D::new(center.x + half_width, center.y),
                ],
                line.clone(),
            ))?;
        }
        if let Some(point) = &self.point {
            emit(LegendCall::Point([center], point.clone()))?;
        }
        Ok(())
    }
}

pub(crate) fn legend_text_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    if std::ptr::eq(left, right) {
        return true;
    }
    for index in 0..left.len() {
        if left.as_bytes()[index] != right.as_bytes()[index] {
            return false;
        }
    }
    true
}

fn optional_point_style_eq(left: Option<&PointStyle>, right: Option<&PointStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.radius == right.radius
                && left.shape == right.shape
                && match (&left.outline, &right.outline) {
                    (Some(left), Some(right)) => {
                        left.color == right.color && left.width_ratio == right.width_ratio
                    }
                    (None, None) => true,
                    _ => false,
                }
        }
        (None, None) => true,
        _ => false,
    }
}

fn optional_line_style_eq(left: Option<&LineStyle>, right: Option<&LineStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.width.to_bits() == right.width.to_bits()
                && left.pattern == right.pattern
        }
        (None, None) => true,
        _ => false,
    }
}

fn optional_fill_style_eq(left: Option<&FillStyle>, right: Option<&FillStyle>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => {
            left.color == right.color
                && left.stroke == right.stroke
                && left.stroke_width.to_bits() == right.stroke_width.to_bits()
        }
        (None, None) => true,
        _ => false,
    }
}

// ─── Local glyph metrics ──────────────────────────────────────────────────────

/// Resolved role-local metrics for legend layout and frozen drawing.
#[derive(Debug, Clone)]
pub(crate) struct LegendMetrics {
    /// Effective font size (px).
    pub font_size: f64,
    /// Border padding (px).
    pub pad: f64,
    /// Entry height (px) = font_size + label_spacing.
    pub entry_h: f64,
    /// Handle (marker) width (px).
    pub handle_w: f64,
    /// Handle-to-text gap (px).
    pub handle_text: f64,
    pub outside_gap: f64,
    pub shared_gap: f64,
    pub column_gap: f64,
    pub row_gap: f64,
    pub minimum: f64,
    pub preferred: f64,
    pub edge_pad: f64,
    pub frame: Option<LegendFrameMetrics>,
    exact: crate::theme::guide::LegendLengths,
    source: crate::theme::LegendTheme,
}

#[derive(Debug, Clone)]
pub(crate) struct LegendFrameMetrics {
    fill: crate::theme::Color,
    outline: Option<(crate::theme::Color, f32)>,
    radius: f64,
    inset: f64,
}

impl LegendFrameMetrics {
    fn style(&self) -> FillStyle {
        let (paint, width) = self
            .outline
            .unwrap_or((crate::theme::Color::rgba(0, 0, 0, 0), 0.0));
        FillStyle::new(self.fill, paint, width)
    }
}

impl LegendMetrics {
    /// Complete role-local metrics do not depend on the outer root dimensions.
    pub(crate) fn from_theme(theme: &crate::theme::ResolvedTheme) -> Result<Self, Error> {
        use crate::layout::kernel::SpatialRounding::{Ceil, NearestEven};

        let exact = theme.legend.lengths(
            theme.typography.guides.legend_label.get_font_size(),
            theme.typography.guides.legend_title.get_font_size(),
        )?;
        let s = |value: &crate::theme::spacing::ExactLength| value.pixels_rounded(NearestEven);
        let font_size = f64::from(theme.typography.guides.legend_label.font_size);
        let pad = s(&exact.padding)?;
        let entry_h = s(&exact
            .row_gap
            .add_pixels(theme.typography.guides.legend_label.line_pitch_pixels())?)?;
        let handle_w = s(&exact.key_width)?;
        if handle_w <= 0.0 {
            return Err(Error::Render(
                "guide key width collapses in CSS quanta".into(),
            ));
        }
        let handle_text = s(&exact.key_label_gap)?;
        let frame = theme
            .legend
            .get_frame()
            .map(|frame| {
                let outline = frame
                    .get_outline()
                    .map(|value| (value.get_color(), value.get_width()));
                if let Some((_, width)) = outline {
                    let canonical = crate::layout::kernel::LayoutScalar::from_css_pixels(
                        f64::from(width),
                        NearestEven,
                    )?;
                    if canonical.quanta() == 0 {
                        return Err(Error::Render(
                            "guide frame outline collapses in CSS quanta".into(),
                        ));
                    }
                }
                Ok::<_, Error>(LegendFrameMetrics {
                    fill: frame.get_fill(),
                    outline,
                    radius: s(exact.radius.as_ref().expect("present frame radius"))?,
                    inset: crate::layout::kernel::required_length(
                        outline.map_or(0.0, |(_, width)| f64::from(width) / 2.0),
                    )?
                    .max(0.5),
                })
            })
            .transpose()?;
        let edge_pad = s(&exact.padding.add_length(&exact.glyph_edge_padding))?;
        Ok(Self {
            font_size,
            pad,
            entry_h,
            handle_w,
            handle_text,
            outside_gap: s(&exact.outside_gap)?,
            shared_gap: s(&exact.shared_gap)?,
            column_gap: s(&exact.column_gap)?,
            row_gap: s(&exact.row_gap)?,
            minimum: exact.minimum.pixels_rounded(Ceil)?,
            preferred: s(&exact.preferred)?,
            edge_pad,
            frame,
            exact,
            source: theme.legend.clone(),
        })
    }

    fn hard(value: &crate::theme::spacing::ExactLength) -> Result<f64, Error> {
        value.pixels_rounded(crate::layout::kernel::SpatialRounding::Ceil)
    }
    fn position(value: &crate::theme::spacing::ExactLength) -> Result<f64, Error> {
        value.pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)
    }
    fn allocation_insets(&mut self, minimum: f64) -> Result<f64, Error> {
        let frame_inset = self.frame.as_ref().map_or(0.0, |frame| frame.inset);
        let inset = crate::theme::spacing::ExactLength::pixels(minimum.max(frame_inset))?;
        if self.exact.padding.compare(&inset) == std::cmp::Ordering::Less {
            self.exact.padding = inset;
        }
        self.pad = Self::position(&self.exact.padding)?;
        self.edge_pad = Self::position(
            &self
                .exact
                .padding
                .add_length(&self.exact.glyph_edge_padding),
        )?;
        if let Some(radius) = &self.exact.radius {
            Self::hard(&radius.add_pixels(frame_inset)?.times(2))
        } else {
            Ok(0.0)
        }
    }
    fn ensure_key_gap(&mut self, minimum: f64) -> Result<(), Error> {
        let required = crate::theme::spacing::ExactLength::pixels(minimum)?;
        if self.exact.key_label_gap.compare(&required) == std::cmp::Ordering::Less {
            self.exact.key_label_gap = required;
        }
        self.handle_text = Self::position(&self.exact.key_label_gap)?;
        Ok(())
    }
    fn outer_width(&self, content: &crate::theme::spacing::ExactLength) -> Result<f64, Error> {
        Self::hard(&self.exact.padding.times(2).add_length(content))
    }
    fn outer_height(
        &self,
        content: &crate::theme::spacing::ExactLength,
        comfort: bool,
    ) -> Result<f64, Error> {
        let edge = if comfort {
            self.exact
                .padding
                .add_length(&self.exact.glyph_edge_padding)
        } else {
            self.exact.padding.times(1)
        };
        Self::hard(&edge.times(2).add_length(content))
    }
    fn edge_reserve(&self, comfort: bool) -> crate::theme::spacing::ExactLength {
        let edge = if comfort {
            self.exact
                .padding
                .add_length(&self.exact.glyph_edge_padding)
        } else {
            self.exact.padding.times(1)
        };
        edge.times(2)
    }
    fn background(
        &self,
        left: f64,
        bottom: f64,
        width: f64,
        height: f64,
    ) -> Option<([f64; 5], FillStyle)> {
        self.frame.as_ref().map(|frame| {
            (
                [
                    left + frame.inset,
                    bottom + frame.inset,
                    width - 2.0 * frame.inset,
                    height - 2.0 * frame.inset,
                    frame.radius,
                ],
                frame.style(),
            )
        })
    }
}

/// A plot legend.
#[derive(Debug, Clone)]
pub(crate) struct Legend {
    /// The sole prepared semantic guide retained by the measured plan.
    descriptor: std::sync::Arc<PreparedGuide>,
    /// Resolved theme-owned metrics retained for selection and frozen drawing.
    pub(crate) metrics: std::sync::Arc<LegendMetrics>,
    available_width: i64,
    available_height: Option<i64>,
    layout: std::sync::Arc<LegendLayout>,
}

#[derive(Debug)]
struct LegendLayout {
    width: i64,
    height: i64,
    sections: Vec<SectionLayout>,
    compact_changes: bool,
}

#[derive(Debug)]
struct SectionLayout {
    title: TextBlock,
    title_baseline: i64,
    top: i64,
    width: i64,
    height: i64,
    capacity: Option<i64>,
    columns: usize,
    entries: Vec<LegendSlot>,
    body_top: i64,
    color: Option<continuous::MeasuredColorBody>,
}

pub(crate) struct GuideWidths {
    pub(crate) title_minimum: f64,
    pub(crate) minimum: f64,
    pub(crate) preferred: f64,
    pub(crate) outside_gap: f64,
}

#[derive(Debug)]
struct LegendSlot {
    key: [i64; 2],
    label: [i64; 2],
    block: TextBlock,
    width: i64,
    height: i64,
}

/// A resolved legend rectangle in math-style pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LegendBox {
    /// Left edge.
    pub left: f64,
    /// Right edge.
    pub right: f64,
    /// Bottom edge.
    pub bottom: f64,
    /// Top edge.
    pub top: f64,
    anchor_x: f64,
    anchor_y: f64,
}

impl LegendBox {
    /// Return the anchor used to select immutable legend draw operands.
    #[must_use]
    pub fn anchor(self) -> (f64, f64) {
        (self.anchor_x, self.anchor_y)
    }

    pub(crate) fn translated(self, x: f64, y: f64) -> Self {
        Self {
            left: self.left + x,
            right: self.right + x,
            bottom: self.bottom + y,
            top: self.top + y,
            anchor_x: self.anchor_x + x,
            anchor_y: self.anchor_y + y,
        }
    }
}

impl Legend {
    fn allocation_metrics(
        descriptor: &PreparedGuide,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(LegendMetrics, f64), Error> {
        let mut metrics = LegendMetrics::from_theme(theme)?;
        let inset = descriptor
            .continuous()
            .and_then(|color| color.missing.as_ref())
            .and_then(|key| key.outline.as_ref())
            .map_or(0.0, |outline| f64::from(outline.get_width()) / 2.0);
        let minimum = metrics.allocation_insets(inset)?;
        metrics.ensure_key_gap(inset)?;
        Ok((metrics, minimum))
    }

    pub(crate) fn natural_preference(
        descriptor: &PreparedGuide,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(f64, f64), Error> {
        measurement::natural_preference(descriptor, theme)
    }

    pub(crate) fn vertical_widths(
        descriptor: &PreparedGuide,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<GuideWidths, Error> {
        measurement::width_requirements(descriptor, theme)
    }

    pub(crate) fn measure_assigned(
        descriptor: std::sync::Arc<PreparedGuide>,
        available: (f64, impl Into<Option<f64>>),
        horizontal: bool,
        compact: bool,
        theme: &crate::theme::ResolvedTheme,
        compact_changes: &mut bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<Self>, Error> {
        measurement::measure_assigned(
            descriptor,
            (available.0, available.1.into()),
            horizontal,
            compact,
            theme,
            compact_changes,
            scope,
        )
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        round.observe_guide(&self.descriptor)?;
        self.metrics.source.observe_feedback(round)?;
        round.observe_number(self.compute_box_width())?;
        round.observe_number(self.compute_box_height())?;
        round.observe_count(self.layout.sections.len())?;
        round.observe_count(usize::from(self.available_height.is_some()))?;
        round.observe_number(measurement::pixels(self.available_width))?;
        if let Some(height) = self.available_height {
            round.observe_number(measurement::pixels(height))?;
        }
        for metric in [
            self.metrics.font_size,
            self.metrics.pad,
            self.metrics.entry_h,
            self.metrics.handle_w,
            self.metrics.handle_text,
            self.metrics.outside_gap,
            self.metrics.shared_gap,
            self.metrics.column_gap,
            self.metrics.row_gap,
            self.metrics.edge_pad,
        ] {
            round.observe_number(metric)?;
        }

        for spacing in [&self.metrics.exact.padding, &self.metrics.exact.row_gap] {
            round.observe_number(
                spacing.pixels_rounded(crate::layout::kernel::SpatialRounding::NearestEven)?,
            )?;
        }
        round.observe_count(usize::from(self.metrics.frame.is_some()))?;
        if let Some(frame) = &self.metrics.frame {
            round.observe_number(frame.radius)?;
            for paint in [Some(frame.fill), frame.outline.map(|(paint, _)| paint)]
                .into_iter()
                .flatten()
            {
                for channel in [paint.red(), paint.green(), paint.blue(), paint.get_alpha()] {
                    round.observe_count(usize::from(channel))?;
                }
            }
            round.observe_count(usize::from(frame.outline.is_some()))?;
            if let Some((_, width)) = frame.outline {
                round.observe_number(f64::from(width))?;
            }
        }
        for (section, layout) in self.descriptor.sections.iter().zip(&self.layout.sections) {
            round.observe_count(layout.columns)?;
            round.observe_count(layout.entries.len())?;
            round.observe_count(usize::from(layout.capacity.is_some()))?;
            for value in [
                layout.top,
                layout.width,
                layout.height,
                layout.title_baseline,
            ]
            .into_iter()
            .chain(layout.capacity)
            {
                round.observe_number(measurement::pixels(value))?;
            }
            round.observe_text_block(&layout.title)?;
            for (entry, slot) in section.entries().iter().zip(&layout.entries) {
                round.observe_legend_glyph(
                    entry.key.point.as_ref(),
                    entry.key.line.as_ref(),
                    entry.key.fill.as_ref(),
                )?;
                round.observe_text_block(&slot.block)?;
                for value in slot
                    .key
                    .into_iter()
                    .chain(slot.label)
                    .chain([slot.width, slot.height])
                {
                    round.observe_number(measurement::pixels(value))?;
                }
            }
            if let Some(color) = &layout.color {
                round.observe_number(measurement::pixels(layout.body_top))?;
                color.observe_feedback(round)?;
            }
        }
        Ok(())
    }
    /// Compute the total bounding box width based on entry labels.
    pub fn compute_box_width(&self) -> f64 {
        measurement::pixels(self.layout.width)
    }

    /// Compute the total bounding box height.
    pub fn compute_box_height(&self) -> f64 {
        measurement::pixels(self.layout.height)
    }

    /// Pick an inside-axes legend box by normalized x/y alignment.
    #[must_use]
    pub fn aligned_inside_box(
        &self,
        pixel_box: (f64, f64, f64, f64),
        x_align: f64,
        y_align: f64,
    ) -> LegendBox {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        let available_w = (x1 - x0 - inset * 2.0 - width).max(0.0);
        let available_h = (y1 - y0 - inset * 2.0 - height).max(0.0);
        self.box_from_rect(
            x0 + inset + available_w * x_align.clamp(0.0, 1.0),
            y0 + inset + available_h * y_align.clamp(0.0, 1.0),
            width,
            height,
        )
    }

    /// Build a legend box from its lower-left corner and total size.
    #[must_use]
    pub fn box_at(&self, left: f64, bottom: f64) -> LegendBox {
        self.box_from_rect(
            left,
            bottom,
            self.compute_box_width(),
            self.compute_box_height(),
        )
    }

    pub(crate) fn inside_candidates(&self, pixel_box: (f64, f64, f64, f64)) -> [LegendBox; 4] {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        // Tie-breaker order favors top placements when overlap scores are equal.
        [
            self.box_from_rect(x1 - inset - width, y1 - inset - height, width, height),
            self.box_from_rect(x0 + inset, y1 - inset - height, width, height),
            self.box_from_rect(x1 - inset - width, y0 + inset, width, height),
            self.box_from_rect(x0 + inset, y0 + inset, width, height),
        ]
    }

    fn box_from_rect(&self, left: f64, bottom: f64, width: f64, height: f64) -> LegendBox {
        let left = left.max(0.0);
        let bottom = bottom.max(0.0);
        LegendBox {
            left,
            right: left + width,
            bottom,
            top: bottom + height,
            anchor_x: left + self.metrics.pad,
            anchor_y: bottom + height - self.metrics.edge_pad,
        }
    }

    pub(crate) fn select(&self, x: f64, y: f64) -> Result<LegendDrawPlan, Error> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::LegendSelection,
        );

        let color_calls = self.descriptor.continuous().map_or(Ok(0), |color| {
            color
                .numeric
                .labels
                .len()
                .checked_add(1)
                .and_then(|count| count.checked_add(usize::from(color.missing.is_some()) * 2))
                .ok_or_else(|| Error::Render("legend call count overflow".into()))
        })?;
        let count = [
            u64::from(self.metrics.frame.is_some()),
            u64::from(self.descriptor.counts.sections),
            u64::from(self.descriptor.counts.entries),
            u64::from(self.descriptor.counts.points),
            u64::from(self.descriptor.counts.lines),
            u64::from(self.descriptor.counts.fills),
        ]
        .into_iter()
        .try_fold(0_u64, u64::checked_add)
        .and_then(|count| usize::try_from(count).ok())
        .and_then(|count| count.checked_add(color_calls))
        .ok_or_else(|| Error::Render("legend call count overflow".into()))?;
        let mut calls = Vec::new();
        crate::data::source::reserve_source(&mut calls, count)?;
        self.visit_calls(x, y, |call| {
            calls.push(call);
            Ok(())
        })?;
        Ok(LegendDrawPlan { calls })
    }

    fn visit_calls(
        &self,
        x: f64,
        y: f64,
        mut emit: impl FnMut(LegendCall) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let m = &self.metrics;

        let box_w = self.compute_box_width();
        let box_h = self.compute_box_height();

        // ─── Background box ───
        // In math coords: top-left content is (x, y), box extends pad outward.
        let rx = x - m.pad;
        let ry_bot = y + m.edge_pad - box_h;

        if let Some((frame, style)) = m.background(rx, ry_bot, box_w, box_h) {
            emit(LegendCall::Background(frame, style))?;
        }

        let translate = |position: [i64; 2]| Point2D {
            x: x + measurement::pixels(position[0]),
            y: y + measurement::pixels(position[1]),
        };
        for (section, layout) in self.descriptor.sections.iter().zip(&self.layout.sections) {
            let title = layout.title.plan();
            let (title_anchor, title_offset) = match title.style.alignment {
                crate::layout::PhysicalTextAlignment::Start => (TextAnchor::Start, 0),
                crate::layout::PhysicalTextAlignment::Center => (
                    TextAnchor::Middle,
                    crate::render::backend::nearest_even_text_half(title.width_quanta)
                        .ok_or_else(|| Error::Render("legend title half extent overflow".into()))?,
                ),
                crate::layout::PhysicalTextAlignment::End => (TextAnchor::End, title.width_quanta),
            };
            emit(LegendCall::Text(crate::render::backend::PlacedText {
                placement: PlannedTextPlacement::baseline(
                    translate([title_offset, layout.title_baseline]),
                    title_anchor,
                    0.0,
                ),
                plan: title.clone(),
            }))?;
            for (entry, slot) in section.entries().iter().zip(&layout.entries) {
                entry.key.visit_calls(m, translate(slot.key), &mut emit)?;
                emit(LegendCall::Text(crate::render::backend::PlacedText {
                    placement: PlannedTextPlacement::baseline(
                        translate(slot.label),
                        TextAnchor::Start,
                        0.0,
                    ),
                    plan: slot.block.plan().clone(),
                }))?;
            }
            if let Some(color) = &layout.color {
                color.visit_calls(x, y - measurement::pixels(layout.body_top), &mut emit)?;
            }
        }

        Ok(())
    }
}
