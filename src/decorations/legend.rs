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

use crate::layout::{PhysicalTextAlignment, TextBlock};
use crate::render::backend::{
    Backend, FillStyle, LineStyle, PlannedTextPlacement, Point2D, PointStyle, TextAnchor,
};
use crate::Error;

mod continuous;
pub(crate) mod descriptor;
pub(crate) use continuous::MeasuredContinuousGuide;

/// The semantic kind of a contributing guide, determined before font discovery.
/// Entry and glyph compatibility is resolved by the prepared descriptor owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuideKind {
    Discrete,
    Continuous,
}

impl GuideKind {
    pub(crate) fn contributing_prepared(
        plot: &crate::Plot,
        inputs: &crate::layout::interior::PreparedInteriorInputs,
    ) -> Result<Option<Self>, Error> {
        use crate::PlotKind;
        if matches!(plot.kind(), PlotKind::Heatmap | PlotKind::Beeswarm) {
            return Ok(Some(Self::Continuous));
        }
        if plot.legend_policy_spec() == crate::LegendPolicy::Hidden {
            return Ok(None);
        }
        let missing = || {
            Error::Render(format!(
                "{:?} guide has no prepared semantic input",
                plot.kind()
            ))
        };
        let contributes = match plot.kind() {
            PlotKind::Scatter => inputs
                .scatter
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .hue_levels()
                .next()
                .is_some(),
            PlotKind::Line | PlotKind::Trajectory | PlotKind::LinePath => inputs
                .line
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .has_guide_entries(plot),
            PlotKind::Bar
            | PlotKind::HorizontalBar
            | PlotKind::StackedBar
            | PlotKind::StackedHorizontalBar => !inputs
                .bars
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .hues
                .is_empty(),
            PlotKind::Boxplot | PlotKind::Violin | PlotKind::Raincloud => !inputs
                .distributions
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .hue_levels()
                .is_empty(),
            PlotKind::Density => inputs
                .density
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .hue_levels()
                .next()
                .is_some(),
            PlotKind::Regression => {
                matches!(plot.legend_policy_spec(), crate::LegendPolicy::Visible(_))
            }
            PlotKind::Dumbbell => true,
            PlotKind::Interaction => inputs
                .interaction
                .as_ref()
                .and_then(|p| p.panels.first())
                .ok_or_else(missing)?
                .1
                .has_guide_entries(plot),
            PlotKind::Marginal => inputs
                .marginal
                .as_ref()
                .ok_or_else(missing)?
                .hue_levels()
                .next()
                .is_some(),
            PlotKind::Histogram
            | PlotKind::Qq
            | PlotKind::PointInterval
            | PlotKind::Coefficient
            | PlotKind::Forest
            | PlotKind::Waterfall => false,
            PlotKind::Heatmap | PlotKind::Beeswarm => {
                unreachable!("continuous guides handled above")
            }
            PlotKind::Matrix => {
                return Err(Error::Render(
                    "Matrix cannot contribute a Canvas guide".into(),
                ))
            }
        };
        Ok(contributes.then_some(Self::Discrete))
    }
}

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

/// Raw semantic identity for one renderer-owned legend entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LegendIdentity {
    Single {
        field: String,
        value: String,
    },
    Pair {
        group_field: String,
        group_value: String,
        hue_field: String,
        hue_value: String,
    },
    Literal {
        title: String,
        value: String,
    },
    /// Renderer-owned sign domain, distinct from caller category/series keys.
    BarSign {
        decreasing: bool,
    },
}

impl LegendIdentity {
    pub(crate) fn single(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Single {
            field: field.into(),
            value: value.into(),
        }
    }

    pub(crate) fn pair(
        group_field: impl Into<String>,
        group_value: impl Into<String>,
        hue_field: impl Into<String>,
        hue_value: impl Into<String>,
    ) -> Self {
        Self::Pair {
            group_field: group_field.into(),
            group_value: group_value.into(),
            hue_field: hue_field.into(),
            hue_value: hue_value.into(),
        }
    }

    pub(crate) fn literal(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Literal {
            title: title.into(),
            value: value.into(),
        }
    }

    pub(crate) const fn bar_sign(decreasing: bool) -> Self {
        Self::BarSign { decreasing }
    }

    fn same_title(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Single { field: left, .. }, Self::Single { field: right, .. })
            | (Self::Literal { title: left, .. }, Self::Literal { title: right, .. }) => {
                legend_text_eq(left, right)
            }
            (Self::BarSign { .. }, Self::BarSign { .. }) => true,
            (
                Self::Pair {
                    group_field: left_group,
                    hue_field: left_hue,
                    ..
                },
                Self::Pair {
                    group_field: right_group,
                    hue_field: right_hue,
                    ..
                },
            ) => legend_text_eq(left_group, right_group) && legend_text_eq(left_hue, right_hue),
            _ => false,
        }
    }

    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        if !self.same_title(other) {
            return false;
        }
        match (self, other) {
            (Self::Single { value: left, .. }, Self::Single { value: right, .. })
            | (Self::Literal { value: left, .. }, Self::Literal { value: right, .. }) => {
                legend_text_eq(left, right)
            }
            (Self::BarSign { decreasing: left }, Self::BarSign { decreasing: right }) => {
                left == right
            }
            (
                Self::Pair {
                    group_value: left_group,
                    hue_value: left_hue,
                    ..
                },
                Self::Pair {
                    group_value: right_group,
                    hue_value: right_hue,
                    ..
                },
            ) => legend_text_eq(left_group, right_group) && legend_text_eq(left_hue, right_hue),
            _ => false,
        }
    }

    pub(crate) fn resolved_text(&self, plot: &crate::Plot) -> Result<(String, String), Error> {
        let join = |mut first: String, separator: &str, second: String| -> Result<String, Error> {
            let additional = separator
                .len()
                .checked_add(second.len())
                .ok_or_else(|| Error::Render("legend label length overflow".into()))?;
            crate::data::source::reserve_source_string(&mut first, additional)?;
            first.push_str(separator);
            first.push_str(&second);
            Ok(first)
        };
        match self {
            Self::Single { field, value } => {
                Ok((plot.resolve_label(field)?, plot.resolve_label(value)?))
            }
            Self::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => Ok((
                join(
                    plot.resolve_label(group_field)?,
                    " × ",
                    plot.resolve_label(hue_field)?,
                )?,
                join(
                    plot.resolve_label(group_value)?,
                    ", ",
                    plot.resolve_label(hue_value)?,
                )?,
            )),
            Self::Literal { title, value } => Ok((
                crate::data::source::copy_source_string(title)?,
                crate::data::source::copy_source_string(value)?,
            )),
            Self::BarSign { decreasing } => Ok((
                crate::data::source::copy_source_string("Change")?,
                crate::data::source::copy_source_string(if *decreasing {
                    "Decreasing"
                } else {
                    "Nondecreasing"
                })?,
            )),
        }
    }

    pub(crate) fn hue_value_for(&self, field: &str) -> Option<&str> {
        match self {
            Self::Single {
                field: entry_field,
                value,
            } if entry_field == field => Some(value),
            Self::Pair {
                hue_field,
                hue_value,
                ..
            } if hue_field == field => Some(hue_value),
            _ => None,
        }
    }
}

/// Exact visible components of one legend glyph.
#[derive(Debug, Clone, Default)]
struct LegendGlyph {
    point: Option<PointStyle>,
    line: Option<LineStyle>,
    fill: Option<FillStyle>,
}

impl LegendGlyph {
    fn exact_eq(&self, other: &Self) -> bool {
        optional_point_style_eq(self.point.as_ref(), other.point.as_ref())
            && optional_line_style_eq(self.line.as_ref(), other.line.as_ref())
            && optional_fill_style_eq(self.fill.as_ref(), other.fill.as_ref())
    }
}

/// One private renderer-owned legend identity and its exact visible glyph.
#[derive(Debug, Clone)]
pub(crate) struct LegendEntry {
    identity: LegendIdentity,
    title: String,
    label: String,
    glyph: LegendGlyph,
    contributes_hue_level: bool,
}

impl LegendEntry {
    pub(crate) fn display_label(&self) -> &str {
        &self.label
    }

    pub(crate) fn same_child_input(&self, other: &Self) -> bool {
        self.exact_eq(other) && self.contributes_hue_level == other.contributes_hue_level
    }

    fn new(identity: LegendIdentity, glyph: LegendGlyph) -> Self {
        Self {
            identity,
            title: String::new(),
            label: String::new(),
            glyph,
            contributes_hue_level: true,
        }
    }

    pub(crate) fn point(identity: LegendIdentity, point: PointStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn line(identity: LegendIdentity, line: LineStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                line: Some(line),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn fill(identity: LegendIdentity, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                fill: Some(fill),
                ..LegendGlyph::default()
            },
        )
    }

    pub(crate) fn point_line(identity: LegendIdentity, point: PointStyle, line: LineStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: Some(line),
                fill: None,
            },
        )
    }

    pub(crate) fn point_fill(identity: LegendIdentity, point: PointStyle, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: None,
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn line_fill(identity: LegendIdentity, line: LineStyle, fill: FillStyle) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: None,
                line: Some(line),
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn point_line_fill(
        identity: LegendIdentity,
        point: PointStyle,
        line: LineStyle,
        fill: FillStyle,
    ) -> Self {
        Self::new(
            identity,
            LegendGlyph {
                point: Some(point),
                line: Some(line),
                fill: Some(fill),
            },
        )
    }

    pub(crate) fn identity(&self) -> &LegendIdentity {
        &self.identity
    }

    pub(crate) fn without_hue_level(mut self) -> Self {
        self.contributes_hue_level = false;
        self
    }

    pub(crate) fn coordinated_hue_value_for(&self, field: &str) -> Option<&str> {
        self.contributes_hue_level
            .then(|| self.identity.hue_value_for(field))
            .flatten()
    }

    fn exact_eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.title == other.title
            && self.label == other.label
            && self.glyph.exact_eq(&other.glyph)
    }

    fn glyph_ink(
        &self,
        metrics: &LegendMetrics,
    ) -> Result<crate::render::geometry::GeometryBox, Error> {
        let half_width = metrics.handle_w / 2.0;
        let half_height = metrics.font_size * 0.32;
        let mut ink = crate::render::geometry::GeometrySnapshot::new();
        if let Some(fill) = &self.glyph.fill {
            ink.polygon(
                &[
                    Point2D::new(-half_width, -half_height),
                    Point2D::new(half_width, -half_height),
                    Point2D::new(half_width, half_height),
                    Point2D::new(-half_width, half_height),
                ],
                fill,
                None,
            )?;
        }
        if let Some(line) = &self.glyph.line {
            ink.line(
                &[
                    Point2D::new(-half_width, 0.0),
                    Point2D::new(half_width, 0.0),
                ],
                line,
                None,
            )?;
        }
        if let Some(point) = &self.glyph.point {
            ink.scatter_with_clip(&[Point2D::new(0.0, 0.0)], point, None)?;
        }

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
}

pub(crate) fn resolve_entries(
    plot: &crate::Plot,
    entries: &mut Vec<LegendEntry>,
) -> Result<(), Error> {
    for entry in entries.iter_mut() {
        (entry.title, entry.label) = entry.identity.resolved_text(plot)?;
    }
    *entries = merge_exact_entries(std::mem::take(entries), "plot legend")?;
    Ok(())
}

pub(crate) fn merge_exact_entries(
    entries: impl IntoIterator<Item = LegendEntry>,
    context: &str,
) -> Result<Vec<LegendEntry>, Error> {
    let mut merged: Vec<LegendEntry> = Vec::new();
    for entry in entries {
        if let Some(first) = merged.first() {
            if !first.identity.same_title(&entry.identity)
                || !legend_text_eq(&first.title, &entry.title)
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        }
        let mut duplicate = None;
        for existing in &merged {
            if !existing.identity.same_identity(&entry.identity) {
                if legend_text_eq(&existing.label, &entry.label) {
                    return Err(Error::Render(format!(
                        "{context} has distinct raw identities resolving to display label {:?}",
                        entry.label
                    )));
                }
            } else if duplicate.is_none() {
                duplicate = Some(existing);
            }
        }
        if let Some(existing) = duplicate {
            if !legend_text_eq(&existing.title, &entry.title)
                || !legend_text_eq(&existing.label, &entry.label)
                || !existing.glyph.exact_eq(&entry.glyph)
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        } else {
            merged
                .try_reserve(1)
                .map_err(|error| Error::Render(format!("merged legend reservation: {error}")))?;
            merged.push(entry);
        }
    }
    Ok(merged)
}

pub(crate) fn resolved_title(entries: &[LegendEntry]) -> Result<String, Error> {
    let Some(first) = entries.first() else {
        return Ok(String::new());
    };

    for entry in entries {
        if !entry.identity.same_title(&first.identity)
            || !legend_text_eq(&entry.title, &first.title)
        {
            return Err(Error::Render(
                "legend entries require one compatible semantic title".into(),
            ));
        }
    }

    let mut title = String::new();
    title
        .try_reserve_exact(first.title.len())
        .map_err(|error| Error::Render(format!("legend title reservation: {error}")))?;
    title.push_str(&first.title);
    Ok(title)
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

/// Conservative half-height around the backend's middle-aligned text anchor.
const GLYPH_HALF_HEIGHT_FACTOR: f64 = 0.6;

/// Resolved role-local metrics for legend layout and frozen drawing.
#[derive(Debug)]
pub(crate) struct LegendMetrics {
    /// Effective font size (px).
    pub font_size: f64,
    /// Border padding (px).
    pub pad: f64,
    /// Entry height (px) = font_size + label_spacing.
    pub entry_h: f64,
    /// Title height (px).
    pub title_h: f64,
    /// Handle (marker) width (px).
    pub handle_w: f64,
    /// Handle-to-text gap (px).
    pub handle_text: f64,
    pub outside_gap: f64,
    pub shared_gap: f64,
    pub column_gap: f64,
    pub row_gap: f64,
    pub title_gap: f64,
    pub minimum: f64,
    pub preferred: f64,
    pub edge_pad: f64,
    pub frame: Option<LegendFrameMetrics>,
    exact: crate::theme::guide::LegendLengths,
    source: crate::theme::LegendTheme,
}

#[derive(Debug)]
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
            theme.typography.guides.legend_label.font_size(),
            theme.typography.guides.legend_title.font_size(),
        )?;
        let s = |value: &crate::theme::spacing::ExactLength| value.pixels_rounded(NearestEven);
        let font_size = f64::from(theme.typography.guides.legend_label.font_size);
        let pad = s(&exact.padding)?;
        let entry_h = s(&exact
            .row_gap
            .add_pixels(theme.typography.guides.legend_label.line_pitch_pixels())?)?;
        let title_h = s(&exact
            .title_gap
            .add_pixels(theme.typography.guides.legend_title.line_pitch_pixels())?)?;
        let handle_w = s(&exact.key_width)?;
        if handle_w <= 0.0 {
            return Err(Error::Render(
                "guide key width collapses in CSS quanta".into(),
            ));
        }
        let handle_text = s(&exact.key_label_gap)?;
        let frame = theme
            .legend
            .frame()
            .map(|frame| {
                let outline = frame.outline().map(|value| (value.color(), value.width()));
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
                    fill: frame.fill(),
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
            title_h,
            handle_w,
            handle_text,
            outside_gap: s(&exact.outside_gap)?,
            shared_gap: s(&exact.shared_gap)?,
            column_gap: s(&exact.column_gap)?,
            row_gap: s(&exact.row_gap)?,
            title_gap: s(&exact.title_gap)?,
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
    fn chrome(&self, key: f64) -> Result<crate::theme::spacing::ExactLength, Error> {
        self.exact.key_label_gap.add_pixels(key)
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
#[derive(Debug)]
pub(crate) struct Legend {
    /// The entries in the legend.
    entries: std::sync::Arc<[LegendEntry]>,
    /// Resolved theme-owned metrics retained for selection and frozen drawing.
    pub(crate) metrics: LegendMetrics,
    available_width: f64,
    available_height: f64,
    title_block: TextBlock,
    entry_blocks: Vec<TextBlock>,
    font_family: crate::font::canonical::FontStack,
    layout: LegendLayout,
}

#[derive(Debug)]
struct LegendLayout {
    width: f64,
    height: f64,
    columns: usize,
    title: PlannedTextPlacement,
    entries: Vec<LegendSlot>,
}

#[derive(Debug)]
struct LegendSlot {
    key: Point2D,
    label: PlannedTextPlacement,
    width: f64,
    height: f64,
}

struct MeasuredEntry {
    width: crate::theme::spacing::ExactLength,
    key: f64,
    chrome: crate::theme::spacing::ExactLength,
    ascent: f64,
    descent: f64,
}

struct EntryIntrinsics {
    minimum: crate::theme::spacing::ExactLength,
    preferred: crate::theme::spacing::ExactLength,
    key: f64,
    chrome: crate::theme::spacing::ExactLength,
    ascent: f64,
    descent: f64,
}

struct DiscreteGuideIntrinsics {
    entries: Vec<EntryIntrinsics>,
    preferred_column: crate::theme::spacing::ExactLength,
}

fn discrete_guide_intrinsics(
    entries: &[LegendEntry],
    theme: &crate::theme::ResolvedTheme,
    metrics: &LegendMetrics,
) -> Result<DiscreteGuideIntrinsics, Error> {
    let mut widths = Vec::new();
    crate::layout::fit::reserve(&mut widths, entries.len(), "guide width requirements")?;

    let mut preferred_column = crate::theme::spacing::ExactLength::pixels(0.0)?;
    for entry in entries {
        let (hard, natural) = crate::layout::fit::intrinsic_widths(
            theme,
            &entry.label,
            &theme.typography.guides.legend_label,
            "legend entry intrinsic",
        )?;
        let ink = entry.glyph_ink(metrics)?;
        let key = crate::layout::kernel::required_length(2.0 * (-ink.left).max(ink.right))?;
        let chrome_exact = metrics.chrome(key)?;
        let minimum = chrome_exact.add_pixels(hard)?;
        // A canonical preferred slot is an allocation choice, not the hard
        // minimum. Keep the latter exact so a tighter complete assignment can
        // select its unrounded remainder instead of adding independent ceilings.
        let preferred = crate::theme::spacing::ExactLength::pixels(LegendMetrics::hard(
            &chrome_exact.add_pixels(hard.max(natural))?,
        )?)?;
        if preferred_column.compare(&preferred) == std::cmp::Ordering::Less {
            preferred_column = preferred.times(1);
        }
        widths.push(EntryIntrinsics {
            minimum,
            preferred,
            key,
            chrome: chrome_exact,
            ascent: ink.top,
            descent: -ink.bottom,
        });
    }
    Ok(DiscreteGuideIntrinsics {
        entries: widths,
        preferred_column,
    })
}

fn entry_allocation(
    entry: &EntryIntrinsics,
    preferred_column: &crate::theme::spacing::ExactLength,
    cap: &crate::theme::spacing::ExactLength,
    horizontal: bool,
) -> crate::theme::spacing::ExactLength {
    let desired = if horizontal {
        &entry.preferred
    } else {
        preferred_column
    };
    let bounded = if desired.compare(cap) == std::cmp::Ordering::Greater {
        cap
    } else {
        desired
    };
    // Vertical entries share this total column allocation despite differing
    // key/chrome widths. A hard minimum above it rejects the trial; it cannot
    // silently enlarge one entry beyond the normal or compact cap.
    bounded.times(1)
}

struct MeasuredLegendFlow {
    slots: Vec<LegendSlot>,
    width: crate::theme::spacing::ExactLength,
    height: crate::theme::spacing::ExactLength,
    columns: usize,
}

/// Assigned outer bounds and their unrounded occupied chrome. Worked local-flow
/// fixtures use no outer reserve; production compares complete requirements.
#[derive(Clone, Copy)]
struct FlowCapacity<'a> {
    width: f64,
    height: f64,
    horizontal_reserve: Option<&'a crate::theme::spacing::ExactLength>,
    vertical_reserve: Option<&'a crate::theme::spacing::ExactLength>,
    title_offset: Option<&'a crate::theme::spacing::ExactLength>,
}

impl FlowCapacity<'_> {
    fn baseline(
        &self,
        consumed: &crate::theme::spacing::ExactLength,
        ascent: f64,
        anchor: f64,
    ) -> Result<f64, Error> {
        let offset = consumed.add_pixels(ascent)?;
        if let Some(title) = self.title_offset {
            title.add_length(&offset).before_pixels(anchor)
        } else {
            offset.before_pixels(anchor)
        }
    }
    fn fits(
        &self,
        value: &crate::theme::spacing::ExactLength,
        horizontal: bool,
    ) -> Result<bool, Error> {
        let (available, reserve) = if horizontal {
            (self.width, self.horizontal_reserve)
        } else {
            (self.height, self.vertical_reserve)
        };
        let hard = if let Some(reserve) = reserve {
            LegendMetrics::hard(&reserve.add_length(value))?
        } else {
            LegendMetrics::hard(value)?
        };
        Ok(hard <= available)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the assigned edge flow consumes one frozen set of measured entries"
)]
fn flow_assigned_entries(
    entries: &[MeasuredEntry],
    capacity: FlowCapacity<'_>,
    body_top: f64,
    metrics: &LegendMetrics,
    horizontal: bool,

    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::fit::FitOutcome<MeasuredLegendFlow>, Error> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::FitKind;
    use crate::theme::spacing::ExactLength;

    let mut slots: Vec<LegendSlot> = Vec::new();
    crate::layout::fit::reserve(&mut slots, entries.len(), "measured guide slots")?;

    let mut x = ExactLength::pixels(0.0)?;
    let mut y = ExactLength::pixels(0.0)?;
    let mut used_width = ExactLength::pixels(0.0)?;
    let mut used_height = ExactLength::pixels(0.0)?;
    let mut column_width = ExactLength::pixels(0.0)?;
    let mut columns = 1;
    let mut row_entries = 0;
    let mut row_start = 0;
    let mut row_ascent = 0.0_f64;
    let mut row_descent = 0.0_f64;
    for entry in entries {
        let entry_height = entry.ascent + entry.descent;
        if horizontal {
            if row_entries > 0 && !capacity.fits(&x.add_length(&entry.width), true)? {
                let baseline = capacity.baseline(&y, row_ascent, body_top)?;
                for slot in &mut slots[row_start..] {
                    slot.key.y = baseline;
                    slot.label.position.y = baseline;
                }
                y = y
                    .add_pixels(row_ascent)?
                    .add_pixels(row_descent)?
                    .add_length(&metrics.exact.row_gap);
                row_start = slots.len();
                row_entries = 0;
                row_ascent = 0.0;
                row_descent = 0.0;
                x = ExactLength::pixels(0.0)?;
            }
            row_entries += 1;
            columns = columns.max(row_entries);
            row_ascent = row_ascent.max(entry.ascent);
            row_descent = row_descent.max(entry.descent);
        } else if LegendMetrics::position(&y)? > 0.0
            && !capacity.fits(&y.add_pixels(entry_height)?, false)?
        {
            x = x
                .add_length(&column_width)
                .add_length(&metrics.exact.column_gap);
            column_width = ExactLength::pixels(0.0)?;
            y = ExactLength::pixels(0.0)?;
            columns += 1;
        }
        let entry_right = x.add_length(&entry.width);
        let entry_bottom = y.add_pixels(entry_height)?;
        if !capacity.fits(&entry_right, true)?
            || (!horizontal && !capacity.fits(&entry_bottom, false)?)
        {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend flow exceeds its assigned track",
            )));
        }
        let baseline = if horizontal {
            0.0
        } else {
            capacity.baseline(&y, entry.ascent, body_top)?
        };
        slots.push(LegendSlot {
            key: Point2D::new(
                x.after_pixels((entry.key - metrics.handle_w) / 2.0)?,
                baseline,
            ),
            label: PlannedTextPlacement::baseline(
                Point2D::new(
                    LegendMetrics::position(&x.add_length(&entry.chrome))?,
                    baseline,
                ),
                TextAnchor::Start,
                0.0,
            ),
            width: LegendMetrics::position(&entry.width)?,
            height: entry_height,
        });
        if used_width.compare(&entry_right) == std::cmp::Ordering::Less {
            used_width = entry_right;
        }
        if horizontal {
            x = x
                .add_length(&entry.width)
                .add_length(&metrics.exact.column_gap);
        } else {
            if column_width.compare(&entry.width) == std::cmp::Ordering::Less {
                column_width = entry.width.times(1);
            }
            if used_height.compare(&entry_bottom) == std::cmp::Ordering::Less {
                used_height = entry_bottom;
            }
            y = y
                .add_pixels(entry_height)?
                .add_length(&metrics.exact.row_gap);
        }
    }
    if horizontal {
        let baseline = capacity.baseline(&y, row_ascent, body_top)?;
        for slot in &mut slots[row_start..] {
            slot.key.y = baseline;
            slot.label.position.y = baseline;
        }
        used_height = y.add_pixels(row_ascent)?.add_pixels(row_descent)?;
        if !capacity.fits(&used_height, false)? {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend rows exceed their assigned height",
            )));
        }
    }
    Ok(FitOutcome::Measured(MeasuredLegendFlow {
        slots,
        width: used_width,
        height: used_height,
        columns,
    }))
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
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(LegendMetrics, f64), Error> {
        let mut metrics = LegendMetrics::from_theme(theme)?;
        let minimum = metrics.allocation_insets(0.0)?;
        Ok((metrics, minimum))
    }

    pub(crate) fn natural_preference(
        title: &str,
        entries: &[LegendEntry],
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<(f64, f64), Error> {
        let (metrics, frame_minimum) = Self::allocation_metrics(theme)?;
        let intrinsic = discrete_guide_intrinsics(entries, theme, &metrics)?;
        let mut width = crate::layout::interior::sizing::text_width(
            theme,
            title,
            &theme.typography.guides.legend_title,
        )?;
        for (entry, requirement) in entries.iter().zip(&intrinsic.entries) {
            width = width.max(
                crate::layout::interior::sizing::text_width(
                    theme,
                    &entry.label,
                    &theme.typography.guides.legend_label,
                )? + LegendMetrics::position(&requirement.chrome)?,
            );
        }
        let title_plan = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            title,
            &theme.typography.guides.legend_title,
            width,
            crate::layout::PhysicalTextAlignment::Start,
            "natural legend title",
            &crate::layout::fit::FitScope::root(),
        ))
        .map_err(crate::layout::fit::PlanError::into_error)?;

        let mut height = crate::theme::spacing::ExactLength::pixels(title_plan.height())?;
        let mut natural_width = crate::theme::spacing::ExactLength::pixels(
            title_plan.width().max(title_plan.fit_width()),
        )?;
        if !title.is_empty() {
            height = height.add_length(&metrics.exact.title_gap);
        }
        for (index, entry) in entries.iter().enumerate() {
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &entry.label,
                &theme.typography.guides.legend_label,
                width - LegendMetrics::position(&intrinsic.entries[index].chrome)?,
                crate::layout::PhysicalTextAlignment::Start,
                "natural legend entry",
                &crate::layout::fit::FitScope::root(),
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            width = width.max(LegendMetrics::position(&intrinsic.entries[index].minimum)?);
            let candidate = intrinsic.entries[index]
                .chrome
                .add_pixels(plan.width().max(plan.fit_width()))?;
            if natural_width.compare(&candidate) == std::cmp::Ordering::Less {
                natural_width = candidate;
            }
            let baseline = plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
            height = height
                .add_pixels(baseline.max(intrinsic.entries[index].ascent))?
                .add_pixels((plan.height() - baseline).max(intrinsic.entries[index].descent))?;
            if index + 1 < entries.len() {
                height = height.add_length(&metrics.exact.row_gap);
            }
        }
        Ok((
            metrics
                .outer_width(&natural_width.add_pixels(1.0 / 1024.0)?)?
                .max(frame_minimum),
            metrics.outer_height(&height, true)?.max(frame_minimum),
        ))
    }

    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "one assigned edge/flow measurement retains all entry geometry"
    )]
    pub(crate) fn measure_assigned(
        title: &str,
        entries: std::sync::Arc<[LegendEntry]>,
        available: (f64, impl Into<Option<f64>>),
        horizontal: bool,
        compact: bool,
        theme: &crate::theme::ResolvedTheme,
        compact_changes: &mut bool,
        scope: &crate::layout::fit::FitScope,
    ) -> Result<crate::layout::fit::FitOutcome<Self>, Error> {
        use crate::layout::fit::{self, FitOutcome};
        use crate::layout::{kernel::FitKind, AddressRole};

        let available = (available.0, available.1.into());
        theme.prepare_font_environment()?;

        if !compact {
            *compact_changes = false;
        }
        if !available.0.is_finite() || available.1.is_some_and(|height| !height.is_finite()) {
            return Err(Error::Render("nonfinite guide assignment".into()));
        }
        if entries.is_empty() {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "an empty guide has no assigned geometry",
            )));
        }
        let (metrics, frame_minimum) = Self::allocation_metrics(theme)?;
        let horizontal_reserve = metrics.edge_reserve(false);
        let Some(inner_exact) = crate::theme::spacing::ExactLength::pixels(available.0.max(0.0))?
            .subtract(&horizontal_reserve)
        else {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside guide has no positive content width",
            )));
        };
        let inner_width =
            inner_exact.pixels_rounded(crate::layout::kernel::SpatialRounding::Floor)?;
        let vertical_reserve = metrics.edge_reserve(true);
        let inner_height = available
            .1
            .map(|height| Ok::<_, Error>(height - LegendMetrics::hard(&vertical_reserve)?))
            .transpose()?;
        if inner_width <= 0.0 || inner_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside guide has no positive content extent",
            )));
        }
        let title_appearance = &theme.typography.guides.legend_title;
        let appearance = &theme.typography.guides.legend_label;
        let title_plan = match fit::text(
            theme,
            title,
            title_appearance,
            inner_width,
            PhysicalTextAlignment::Start,
            "outside legend title",
            &scope.child(AddressRole::LegendTitle, 0)?,
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let mut occupied_title = vertical_reserve.add_pixels(title_plan.height())?;
        if !title.is_empty() {
            occupied_title = occupied_title.add_length(&metrics.exact.title_gap);
        }
        let body_height = available
            .1
            .map(|height| Ok::<_, Error>(height - LegendMetrics::hard(&occupied_title)?))
            .transpose()?;
        if body_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend title leaves no entry height",
            )));
        }
        let intrinsic = discrete_guide_intrinsics(&entries, theme, &metrics)?;
        let preferred_column = &intrinsic.preferred_column;
        let compact_cap = inner_exact
            .subtract(&metrics.exact.column_gap)
            .map(|remaining| {
                let width = remaining
                    .divided_by(2.0)?
                    .pixels_rounded(crate::layout::kernel::SpatialRounding::Floor)?;
                crate::theme::spacing::ExactLength::pixels(width)
            })
            .transpose()?;
        if !compact {
            if let Some(cap) = &compact_cap {
                *compact_changes = false;
                for entry in &intrinsic.entries {
                    let regular =
                        entry_allocation(entry, preferred_column, &inner_exact, horizontal);
                    let reduced = entry_allocation(entry, preferred_column, cap, horizontal);
                    if regular.compare(&reduced) == std::cmp::Ordering::Greater {
                        *compact_changes = true;
                        break;
                    }
                }
            }
        }
        let cap = if compact {
            let Some(width) = compact_cap.as_ref() else {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::LegendOrGuide,
                    "compact guide has no column width",
                )));
            };
            width
        } else {
            &inner_exact
        };
        for (index, entry) in intrinsic.entries.iter().enumerate() {
            let allocated = entry_allocation(entry, preferred_column, cap, horizontal);
            if entry.minimum.compare(&allocated) == std::cmp::Ordering::Greater
                || metrics.outer_width(&entry.minimum)? > available.0
            {
                return Ok(FitOutcome::Miss(
                    scope.child(AddressRole::LegendEntry, index)?.failure(
                        FitKind::Text,
                        "guide entry allocation is below its hard unbroken requirement",
                    ),
                ));
            }
        }
        let mut entry_blocks = Vec::new();
        let mut measured = Vec::new();
        fit::reserve(&mut entry_blocks, entries.len(), "measured guide labels")?;
        fit::reserve(&mut measured, entries.len(), "measured guide entries")?;
        let title_half = GLYPH_HALF_HEIGHT_FACTOR * f64::from(title_appearance.font_size);
        let title_offset = if title.is_empty() {
            crate::theme::spacing::ExactLength::pixels(title_plan.height())?
        } else {
            metrics.exact.title_gap.add_pixels(title_plan.height())?
        };
        let body_top = title_half;
        for (index, (entry, width)) in entries.iter().zip(intrinsic.entries).enumerate() {
            let allocated = entry_allocation(&width, preferred_column, cap, horizontal);
            let Some(text_capacity) = allocated.subtract(&width.chrome) else {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::Text,
                    "guide entry has no text capacity after exact chrome",
                )));
            };
            let text_width =
                text_capacity.pixels_rounded(crate::layout::kernel::SpatialRounding::Floor)?;
            let plan = match fit::text(
                theme,
                &entry.label,
                appearance,
                if entry.label.is_empty() {
                    inner_width
                } else {
                    text_width
                },
                PhysicalTextAlignment::Start,
                "outside legend entry",
                &scope.child(AddressRole::LegendEntry, index)?,
            )? {
                FitOutcome::Measured(plan) => plan,
                FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
            };
            let baseline = plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
            let ascent = baseline.max(width.ascent);
            measured.push(MeasuredEntry {
                width: allocated,
                key: width.key,
                chrome: width.chrome,
                ascent,
                descent: (plan.height() - baseline).max(width.descent),
            });
            entry_blocks.push(TextBlock::from_resolved(plan));
        }
        let body_height = if let Some(height) = body_height {
            height
        } else {
            let count = u32::try_from(entries.len().saturating_sub(1))
                .map_err(|_| Error::Render("guide row count overflow".into()))?;
            let mut height = metrics.exact.row_gap.times(count);
            for entry in &measured {
                height = height.add_pixels(entry.ascent)?.add_pixels(entry.descent)?;
            }
            LegendMetrics::hard(&height)?
        };
        let flow = match flow_assigned_entries(
            &measured,
            FlowCapacity {
                width: available.0,
                height: available.1.unwrap_or(body_height),
                horizontal_reserve: Some(&horizontal_reserve),
                vertical_reserve: available.1.map(|_| &occupied_title),
                title_offset: Some(&title_offset),
            },
            body_top,
            &metrics,
            horizontal,
            scope,
        )? {
            FitOutcome::Measured(flow) => flow,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_baseline = title_plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
        let title_width = crate::theme::spacing::ExactLength::pixels(
            title_plan.width().max(title_plan.fit_width()),
        )?;
        let content_width = if flow.width.compare(&title_width) == std::cmp::Ordering::Less {
            title_width
        } else {
            flow.width
        };
        let mut content_height = flow.height.add_pixels(title_plan.height())?;
        if !title.is_empty() {
            content_height = content_height.add_length(&metrics.exact.title_gap);
        }
        let layout = LegendLayout {
            width: metrics.outer_width(&content_width)?.max(frame_minimum),
            height: metrics
                .outer_height(&content_height, true)?
                .max(frame_minimum),
            columns: flow.columns,
            title: PlannedTextPlacement::baseline(
                Point2D::new(0.0, title_half - title_baseline),
                TextAnchor::Start,
                0.0,
            ),
            entries: flow.slots,
        };
        if layout.width > available.0 || available.1.is_some_and(|height| layout.height > height) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "complete discrete guide frame exceeds its assigned track",
            )));
        }
        let legend = Self {
            entries,
            metrics,
            available_width: available.0,
            available_height: available.1.unwrap_or(layout.height),
            title_block: TextBlock::from_resolved(title_plan),
            entry_blocks,
            font_family: theme
                .typography
                .guides
                .legend_label
                .stack
                .copy_with_stage(None)
                .map_err(|error| Error::Render(error.to_string()))?,
            layout,
        };
        let bounds = legend.box_at(0.0, 0.0);
        let mut ink = crate::render::geometry::GeometrySnapshot::new();
        let (x, y) = bounds.anchor();
        legend.visit_calls(x, y, |call| call.geometry(&mut ink))?;
        for ink in ink.bounds() {
            if ink.left < bounds.left
                || ink.right > bounds.right
                || ink.bottom < bounds.bottom
                || ink.top > bounds.top
            {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::LegendOrGuide,
                    "complete discrete guide ink exceeds its assigned allocation",
                )));
            }
        }
        Ok(FitOutcome::Measured(legend))
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        self.metrics.source.observe_feedback(round)?;
        round.observe_number(self.compute_box_width())?;
        round.observe_number(self.compute_box_height())?;
        round.observe_count(self.column_count())?;
        for metric in [
            self.available_width,
            self.available_height,
            self.metrics.font_size,
            self.metrics.pad,
            self.metrics.entry_h,
            self.metrics.title_h,
            self.metrics.handle_w,
            self.metrics.handle_text,
            self.metrics.outside_gap,
            self.metrics.shared_gap,
            self.metrics.column_gap,
            self.metrics.row_gap,
            self.metrics.title_gap,
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
                for channel in [paint.red(), paint.green(), paint.blue(), paint.alpha()] {
                    round.observe_count(usize::from(channel))?;
                }
            }
            round.observe_count(usize::from(frame.outline.is_some()))?;
            if let Some((_, width)) = frame.outline {
                round.observe_number(f64::from(width))?;
            }
        }
        round.observe_count(self.entry_blocks.len())?;
        round.observe_font_stack(&self.font_family)?;
        for entry in self.entries.iter() {
            round.observe_legend_glyph(
                entry.glyph.point.as_ref(),
                entry.glyph.line.as_ref(),
                entry.glyph.fill.as_ref(),
            )?;
        }
        round.observe_text_block(&self.title_block)?;
        for entry in &self.entry_blocks {
            round.observe_text_block(entry)?;
        }
        round.observe_placement(self.layout.title)?;
        for slot in &self.layout.entries {
            round.observe_number(slot.key.x)?;
            round.observe_number(slot.key.y)?;
            round.observe_placement(slot.label)?;
            round.observe_number(slot.width)?;
            round.observe_number(slot.height)?;
        }
        Ok(())
    }
    /// Check the represented shared allocation before retaining measured entries.
    pub(crate) fn share_entries(
        entries: Vec<LegendEntry>,
    ) -> Result<std::sync::Arc<[LegendEntry]>, Error> {
        std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(
                std::alloc::Layout::array::<LegendEntry>(entries.len())
                    .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
            )
            .map_err(|_| Error::Render("legend entry Arc layout overflow".into()))?;
        Ok(entries.into())
    }

    /// Compute the total bounding box width based on entry labels.
    pub fn compute_box_width(&self) -> f64 {
        self.layout.width
    }

    /// Compute the total bounding box height.
    pub fn compute_box_height(&self) -> f64 {
        self.layout.height
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
            anchor_y: bottom + height - self.glyph_edge_pad() - self.title_half_height(),
        }
    }

    pub(crate) fn select(&self, x: f64, y: f64) -> Result<LegendDrawPlan, Error> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::LegendSelection,
        );

        let count = self.entries.iter().try_fold(
            1 + usize::from(self.metrics.frame.is_some()),
            |count, entry| {
                count
                    .checked_add(
                        1 + usize::from(entry.glyph.fill.is_some())
                            + usize::from(entry.glyph.line.is_some())
                            + usize::from(entry.glyph.point.is_some()),
                    )
                    .ok_or_else(|| Error::Render("legend call count overflow".into()))
            },
        )?;
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
        let title_block = &self.title_block;
        let entry_blocks = &self.entry_blocks;

        let box_w = self.compute_box_width();
        let box_h = self.compute_box_height();

        // ─── Background box ───
        // In math coords: top-left content is (x, y), box extends pad outward.
        let rx = x - m.pad;
        let ry_bot = y + self.glyph_edge_pad() + self.title_half_height() - box_h;

        if let Some((frame, style)) = m.background(rx, ry_bot, box_w, box_h) {
            emit(LegendCall::Background(frame, style))?;
        }

        // ─── Title ───
        let translate = |placement: PlannedTextPlacement| PlannedTextPlacement {
            position: Point2D::new(x + placement.position.x, y + placement.position.y),
            ..placement
        };
        emit(LegendCall::Text(crate::render::backend::PlacedText {
            placement: translate(self.layout.title),
            plan: title_block.plan().clone(),
        }))?;
        // ─── Entries (top to bottom = decreasing y) ───
        for (index, entry) in self.entries.iter().enumerate() {
            let slot = &self.layout.entries[index];
            let column_x = x + slot.key.x;
            let row_y = y + slot.key.y;
            // Renderer-owned fill, line, and point components share one handle.
            let marker_x = column_x + m.handle_w * 0.5;
            if let Some(fill_style) = &entry.glyph.fill {
                let half_height = self.entry_font_size() * 0.32;
                emit(LegendCall::Fill(
                    [
                        Point2D::new(column_x, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y - half_height),
                        Point2D::new(column_x + m.handle_w, row_y + half_height),
                        Point2D::new(column_x, row_y + half_height),
                    ],
                    fill_style.clone(),
                ))?;
            }
            if let Some(line_style) = &entry.glyph.line {
                emit(LegendCall::Line(
                    [
                        Point2D::new(column_x, row_y),
                        Point2D::new(column_x + m.handle_w, row_y),
                    ],
                    line_style.clone(),
                ))?;
            }
            if let Some(point_style) = &entry.glyph.point {
                emit(LegendCall::Point(
                    [Point2D::new(marker_x, row_y)],
                    point_style.clone(),
                ))?;
            }
            // Label text
            let block = &entry_blocks[index];
            emit(LegendCall::Text(crate::render::backend::PlacedText {
                placement: translate(slot.label),
                plan: block.plan().clone(),
            }))?;
        }

        Ok(())
    }

    fn title_font_size(&self) -> f64 {
        f64::from(self.title_block.plan().style.font_size)
    }

    fn entry_font_size(&self) -> f64 {
        self.metrics.font_size
    }

    fn title_half_height(&self) -> f64 {
        GLYPH_HALF_HEIGHT_FACTOR * self.title_font_size()
    }

    fn glyph_edge_pad(&self) -> f64 {
        self.metrics.edge_pad
    }

    fn column_count(&self) -> usize {
        self.layout.columns
    }
}
