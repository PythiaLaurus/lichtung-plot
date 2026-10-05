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

use crate::font::canonical::OperationGenerationResources;
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

    fn same_title(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool, Error> {
        match (self, other) {
            (Self::Single { field: left, .. }, Self::Single { field: right, .. })
            | (Self::Literal { title: left, .. }, Self::Literal { title: right, .. }) => {
                legend_text_eq(left, right, resources)
            }
            (Self::BarSign { .. }, Self::BarSign { .. }) => Ok(true),
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
            ) => Ok(legend_text_eq(left_group, right_group, resources)?
                && legend_text_eq(left_hue, right_hue, resources)?),
            _ => Ok(false),
        }
    }

    pub(crate) fn same_identity(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool, Error> {
        if !self.same_title(other, resources)? {
            return Ok(false);
        }
        match (self, other) {
            (Self::Single { value: left, .. }, Self::Single { value: right, .. })
            | (Self::Literal { value: left, .. }, Self::Literal { value: right, .. }) => {
                legend_text_eq(left, right, resources)
            }
            (Self::BarSign { decreasing: left }, Self::BarSign { decreasing: right }) => {
                Ok(left == right)
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
            ) => Ok(legend_text_eq(left_group, right_group, resources)?
                && legend_text_eq(left_hue, right_hue, resources)?),
            _ => Ok(false),
        }
    }

    pub(crate) fn resolved_text(
        &self,
        plot: &crate::Plot,
        data: crate::data::PlotData<'_>,
    ) -> Result<(String, String), Error> {
        let join = |mut first: String, separator: &str, second: String| -> Result<String, Error> {
            let additional = separator
                .len()
                .checked_add(second.len())
                .ok_or_else(|| Error::Render("legend label length overflow".into()))?;
            data.reserve_source_string(&mut first, additional)?;
            first.push_str(separator);
            first.push_str(&second);
            Ok(first)
        };
        match self {
            Self::Single { field, value } => Ok((
                plot.resolve_source_label(field, data)?,
                plot.resolve_source_label(value, data)?,
            )),
            Self::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => Ok((
                join(
                    plot.resolve_source_label(group_field, data)?,
                    " × ",
                    plot.resolve_source_label(hue_field, data)?,
                )?,
                join(
                    plot.resolve_source_label(group_value, data)?,
                    ", ",
                    plot.resolve_source_label(hue_value, data)?,
                )?,
            )),
            Self::Literal { title, value } => Ok((
                data.copy_source_string(title)?,
                data.copy_source_string(value)?,
            )),
            Self::BarSign { decreasing } => Ok((
                data.copy_source_string("Change")?,
                data.copy_source_string(if *decreasing {
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

    pub(crate) fn admitted_clone(&self, data: crate::data::PlotData<'_>) -> Result<Self, Error> {
        let identity = match &self.identity {
            LegendIdentity::Single { field, value } => [Some(field), Some(value), None, None],
            LegendIdentity::Literal { title, value } => [Some(title), Some(value), None, None],
            LegendIdentity::BarSign { .. } => [None, None, None, None],
            LegendIdentity::Pair {
                group_field,
                group_value,
                hue_field,
                hue_value,
            } => [
                Some(group_field),
                Some(group_value),
                Some(hue_field),
                Some(hue_value),
            ],
        };
        for text in identity
            .into_iter()
            .flatten()
            .chain([&self.title, &self.label])
        {
            data.admit_clone_str(text)?;
        }
        Ok(self.clone())
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
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometryBox, Error> {
        let half_width = metrics.handle_w / 2.0;
        let half_height = metrics.font_size * 0.32;
        let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
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
        legend_visits(theme, ink.bounds().len())?;
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
    resources: &OperationGenerationResources,
) -> Result<(), Error> {
    let data = plot.data().with_axis_resources(resources);
    data.claim_source_visits(entries.len())?;
    for entry in entries.iter_mut() {
        (entry.title, entry.label) = entry.identity.resolved_text(plot, data)?;
    }
    *entries = merge_exact_entries(std::mem::take(entries), "plot legend", resources)?;
    Ok(())
}

pub(crate) fn merge_exact_entries(
    entries: impl IntoIterator<Item = LegendEntry>,
    context: &str,
    resources: &OperationGenerationResources,
) -> Result<Vec<LegendEntry>, Error> {
    let mut merged: Vec<LegendEntry> = Vec::new();
    for entry in entries {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))?;
        if let Some(first) = merged.first() {
            if !first.identity.same_title(&entry.identity, resources)?
                || !legend_text_eq(&first.title, &entry.title, resources)?
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        }
        let mut duplicate = None;
        for existing in &merged {
            resources
                .claim_visits(1)
                .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))?;
            if !existing
                .identity
                .same_identity(&entry.identity, resources)?
            {
                if legend_text_eq(&existing.label, &entry.label, resources)? {
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
            if !legend_text_eq(&existing.title, &entry.title, resources)?
                || !legend_text_eq(&existing.label, &entry.label, resources)?
                || !existing.glyph.exact_eq(&entry.glyph)
            {
                return Err(Error::Render(format!(
                    "{context} requires compatible semantic titles, labels, and styles; glyphs must match exactly"
                )));
            }
        } else {
            resources
                .claim_slots(1)
                .map_err(|error| Error::Render(format!("merged legend records: {error:?}")))?;
            resources
                .claim_bytes(
                    u64::try_from(std::mem::size_of::<LegendEntry>())
                        .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
                )
                .map_err(|error| Error::Render(format!("merged legend storage: {error}")))?;
            merged
                .try_reserve(1)
                .map_err(|error| Error::Render(format!("merged legend reservation: {error}")))?;
            merged.push(entry);
        }
    }
    Ok(merged)
}

pub(crate) fn resolved_title(
    entries: &[LegendEntry],
    resources: &OperationGenerationResources,
) -> Result<String, Error> {
    let Some(first) = entries.first() else {
        return Ok(String::new());
    };
    resources
        .claim_visits(entries.len() as u128)
        .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))?;
    for entry in entries {
        if !entry.identity.same_title(&first.identity, resources)?
            || !legend_text_eq(&entry.title, &first.title, resources)?
        {
            return Err(Error::Render(
                "legend entries require one compatible semantic title".into(),
            ));
        }
    }
    resources
        .claim_slots(1)
        .map_err(|error| Error::Render(format!("legend title record admission: {error:?}")))?;
    resources
        .claim_bytes(
            u64::try_from(first.title.len())
                .map_err(|_| Error::Render("legend title length overflow".into()))?,
        )
        .map_err(|error| Error::Render(format!("legend title storage admission: {error}")))?;
    let mut title = String::new();
    title
        .try_reserve_exact(first.title.len())
        .map_err(|error| Error::Render(format!("legend title reservation: {error}")))?;
    title.push_str(&first.title);
    Ok(title)
}

fn legend_visits(theme: &crate::theme::ResolvedTheme, count: usize) -> Result<(), Error> {
    theme
        .prepared_font_environment()?
        .claim_processing_visits(count as u128)
        .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))
}

pub(crate) fn legend_text_eq(
    left: &str,
    right: &str,
    resources: &OperationGenerationResources,
) -> Result<bool, Error> {
    resources
        .claim_visits(1)
        .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))?;
    if left.len() != right.len() {
        return Ok(false);
    }
    if std::ptr::eq(left, right) {
        return Ok(true);
    }
    for index in 0..left.len() {
        resources
            .claim_visits(1)
            .map_err(|error| Error::Render(format!("legend work admission: {error:?}")))?;
        if left.as_bytes()[index] != right.as_bytes()[index] {
            return Ok(false);
        }
    }
    Ok(true)
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

#[cfg(test)]
#[test]
fn point_outline_guide_identity_uses_the_exact_relationship() {
    let mut left = PointStyle::new((0, 0, 0), 3);
    let mut right = left.clone();
    left.outline.as_mut().unwrap().width_ratio =
        crate::theme::RelativeSpacing::ratio(3, 5).unwrap();
    right.outline.as_mut().unwrap().width_ratio = crate::theme::RelativeSpacing::new(0.6).unwrap();
    assert_eq!(
        left.outline
            .as_ref()
            .unwrap()
            .width_ratio
            .as_f64()
            .to_bits(),
        right
            .outline
            .as_ref()
            .unwrap()
            .width_ratio
            .as_f64()
            .to_bits()
    );
    assert!(!optional_point_style_eq(Some(&left), Some(&right)));
    right.outline.as_mut().unwrap().width_ratio =
        crate::theme::RelativeSpacing::ratio(6, 10).unwrap();
    assert!(optional_point_style_eq(Some(&left), Some(&right)));
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
/// Extra edge comfort retained from the previous bottom padding (x font_size).
const GLYPH_EDGE_PAD_FACTOR: f64 = 0.2;

/// Resolved role-local metrics for legend layout and frozen drawing.
#[derive(Debug, Clone, Copy)]
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
    pub frame_fill: crate::theme::Color,
    pub frame_outline: crate::theme::Color,
    pub frame_stroke_width: f32,
    pub frame_radius: f64,
}

impl LegendMetrics {
    /// Complete role-local metrics do not depend on the outer root dimensions.
    pub(crate) fn from_theme(theme: &crate::theme::ResolvedTheme) -> Self {
        let font_size = f64::from(theme.typography.legend_label.font_size);
        let pad = theme.legend.padding;
        let entry_h = theme.typography.legend_label.line_pitch_pixels() + theme.legend.row_gap;
        let title_h = theme.typography.legend_title.line_pitch_pixels() + theme.legend.title_gap;
        let handle_w = theme.legend.key_width;
        let handle_text = theme.legend.key_label_gap;
        Self {
            font_size,
            pad,
            entry_h,
            title_h,
            handle_w,
            handle_text,
            outside_gap: theme.legend.outside_gap,
            shared_gap: theme.legend.shared_gap,
            column_gap: theme.legend.column_gap,
            row_gap: theme.legend.row_gap,
            title_gap: theme.legend.title_gap,
            frame_fill: theme.legend.frame_fill,
            frame_outline: theme.legend.frame_outline,
            frame_stroke_width: theme.legend.frame_stroke_width,
            frame_radius: theme.legend.frame_radius,
        }
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
    width: f64,
    key: f64,
    chrome: f64,
    ascent: f64,
    descent: f64,
}

struct EntryIntrinsics {
    minimum: f64,
    preferred: f64,
    key: f64,
    chrome: f64,
    ascent: f64,
    descent: f64,
}

struct DiscreteGuideIntrinsics {
    entries: Vec<EntryIntrinsics>,
    preferred_column: f64,
}

fn discrete_guide_intrinsics(
    entries: &[LegendEntry],
    theme: &crate::theme::ResolvedTheme,
    metrics: &LegendMetrics,
) -> Result<DiscreteGuideIntrinsics, Error> {
    let mut widths = Vec::new();
    crate::layout::fit::reserve(
        theme,
        &mut widths,
        entries.len(),
        "guide width requirements",
    )?;
    let mut preferred_column = 0.0_f64;
    for entry in entries {
        legend_visits(theme, 1)?;
        let (hard, natural) = crate::layout::fit::intrinsic_widths(
            theme,
            &entry.label,
            &theme.typography.legend_label,
            "legend entry intrinsic",
        )?;
        let ink = entry.glyph_ink(metrics, theme)?;
        let key = crate::layout::kernel::required_length(2.0 * (-ink.left).max(ink.right))?;
        let chrome = crate::layout::kernel::required_length(key + metrics.handle_text)?;
        let preferred = crate::layout::kernel::required_length(hard.max(natural) + chrome)?;
        preferred_column = preferred_column.max(preferred);
        widths.push(EntryIntrinsics {
            minimum: crate::layout::kernel::required_length(hard + chrome)?,
            preferred,
            key,
            chrome,
            ascent: ink.top,
            descent: -ink.bottom,
        });
    }
    Ok(DiscreteGuideIntrinsics {
        entries: widths,
        preferred_column,
    })
}

struct MeasuredLegendFlow {
    slots: Vec<LegendSlot>,
    width: f64,
    height: f64,
    columns: usize,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the assigned edge flow consumes one frozen set of measured entries"
)]
fn flow_assigned_entries(
    entries: &[MeasuredEntry],
    width: f64,
    height: f64,
    body_top: f64,
    metrics: &LegendMetrics,
    horizontal: bool,
    theme: &crate::theme::ResolvedTheme,
    scope: &crate::layout::fit::FitScope,
) -> Result<crate::layout::fit::FitOutcome<MeasuredLegendFlow>, Error> {
    use crate::layout::fit::FitOutcome;
    use crate::layout::kernel::FitKind;

    let mut slots: Vec<LegendSlot> = Vec::new();
    crate::layout::fit::reserve(theme, &mut slots, entries.len(), "measured guide slots")?;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut used_width = 0.0_f64;
    let mut used_height = 0.0_f64;
    let mut columns = 1;
    let mut row_entries = 0;
    let mut row_start = 0;
    let mut row_ascent = 0.0_f64;
    let mut row_descent = 0.0_f64;
    for entry in entries {
        let entry_height = entry.ascent + entry.descent;
        if horizontal {
            if row_entries > 0 && x + entry.width > width {
                let baseline = body_top - y - row_ascent;
                for slot in &mut slots[row_start..] {
                    slot.key.y = baseline;
                    slot.label.position.y = baseline;
                }
                y += row_ascent + row_descent + metrics.row_gap;
                row_start = slots.len();
                row_entries = 0;
                row_ascent = 0.0;
                row_descent = 0.0;
                x = 0.0;
            }
            row_entries += 1;
            columns = columns.max(row_entries);
            row_ascent = row_ascent.max(entry.ascent);
            row_descent = row_descent.max(entry.descent);
        } else if y > 0.0 && y + entry_height > height {
            x += entry.width + metrics.column_gap;
            y = 0.0;
            columns += 1;
        }
        if x + entry.width > width || (!horizontal && y + entry_height > height) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend flow exceeds its assigned track",
            )));
        }
        let baseline = if horizontal {
            0.0
        } else {
            body_top - y - entry.ascent
        };
        slots.push(LegendSlot {
            key: Point2D::new(x + (entry.key - metrics.handle_w) / 2.0, baseline),
            label: PlannedTextPlacement::baseline(
                Point2D::new(x + entry.chrome, baseline),
                TextAnchor::Start,
                0.0,
            ),
            width: entry.width,
            height: entry_height,
        });
        used_width = used_width.max(x + entry.width);
        if horizontal {
            x += entry.width + metrics.column_gap;
        } else {
            used_height = used_height.max(y + entry_height);
            y += entry_height + metrics.row_gap;
        }
    }
    if horizontal {
        let baseline = body_top - y - row_ascent;
        for slot in &mut slots[row_start..] {
            slot.key.y = baseline;
            slot.label.position.y = baseline;
        }
        used_height = y + row_ascent + row_descent;
        if used_height > height {
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
        let mut metrics = LegendMetrics::from_theme(theme);
        let inset =
            crate::layout::kernel::required_length(f64::from(metrics.frame_stroke_width) / 2.0)?
                .max(0.5);
        metrics.pad = metrics.pad.max(inset);
        let minimum = 2.0 * (inset + metrics.frame_radius);
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
            &theme.typography.legend_title,
        )?;
        for (entry, requirement) in entries.iter().zip(&intrinsic.entries) {
            width = width.max(
                crate::layout::interior::sizing::text_width(
                    theme,
                    &entry.label,
                    &theme.typography.legend_label,
                )? + requirement.chrome,
            );
        }
        let title_plan = crate::layout::fit::measured(crate::layout::fit::text(
            theme,
            title,
            &theme.typography.legend_title,
            width,
            crate::layout::PhysicalTextAlignment::Start,
            "natural legend title",
            &crate::layout::fit::FitScope::root(),
        ))
        .map_err(crate::layout::fit::PlanError::into_error)?;
        let mut height = if title.is_empty() {
            0.0
        } else {
            title_plan.height() + metrics.title_gap
        };
        for (index, entry) in entries.iter().enumerate() {
            theme
                .prepared_font_environment()?
                .claim_processing_visits(1)
                .map_err(|error| Error::Render(format!("natural legend entry: {error:?}")))?;
            let plan = crate::layout::fit::measured(crate::layout::fit::text(
                theme,
                &entry.label,
                &theme.typography.legend_label,
                width - intrinsic.entries[index].chrome,
                crate::layout::PhysicalTextAlignment::Start,
                "natural legend entry",
                &crate::layout::fit::FitScope::root(),
            ))
            .map_err(crate::layout::fit::PlanError::into_error)?;
            width = width.max(intrinsic.entries[index].minimum);
            let baseline = plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
            height += baseline.max(intrinsic.entries[index].ascent)
                + (plan.height() - baseline).max(intrinsic.entries[index].descent);
            if index + 1 < entries.len() {
                height += metrics.row_gap;
            }
        }
        Ok((
            crate::layout::kernel::required_length(
                (width + 2.0 * metrics.pad + 1.0 / 1024.0).max(frame_minimum),
            )?,
            crate::layout::kernel::required_length(
                (height + 2.0 * (metrics.pad + GLYPH_EDGE_PAD_FACTOR * metrics.font_size))
                    .max(frame_minimum),
            )?,
        ))
    }
    #[cfg(test)]
    pub(crate) fn text_plans(&self) -> impl Iterator<Item = &crate::layout::ResolvedTextPlan> {
        std::iter::once(self.title_block.plan())
            .chain(self.entry_blocks.iter().map(TextBlock::plan))
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
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| {
                Error::Render(format!("legend measurement call admission: {error:?}"))
            })?;
        let available = (available.0, available.1.into());
        theme.prepare_font_environment()?;
        let resources = theme.prepared_font_environment()?.generation_resources();
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
        let font_size = metrics.font_size.round() as u32;
        let edge_pad = metrics.pad + GLYPH_EDGE_PAD_FACTOR * f64::from(font_size);
        let inner_width = available.0 - 2.0 * metrics.pad;
        let inner_height = available.1.map(|height| height - 2.0 * edge_pad);
        if inner_width <= 0.0 || inner_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside guide has no positive content extent",
            )));
        }
        let title_appearance = &theme.typography.legend_title;
        let appearance = &theme.typography.legend_label;
        let title_plan = match fit::text(
            theme,
            title,
            title_appearance,
            inner_width,
            PhysicalTextAlignment::Start,
            "outside legend title",
            &scope.child(AddressRole::LegendTitle, 0, &resources)?,
        )? {
            FitOutcome::Measured(plan) => plan,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_gap = if title.is_empty() {
            0.0
        } else {
            metrics.title_gap
        };
        let body_height = inner_height.map(|height| height - title_plan.height() - title_gap);
        if body_height.is_some_and(|height| height <= 0.0) {
            return Ok(FitOutcome::Miss(scope.failure(
                FitKind::LegendOrGuide,
                "outside legend title leaves no entry height",
            )));
        }
        let gap = metrics.column_gap;
        let intrinsic = discrete_guide_intrinsics(&entries, theme, &metrics)?;
        let preferred_column = intrinsic.preferred_column.min(inner_width);
        if !compact {
            if let Some(cap) = crate::layout::kernel::compact_half_width(inner_width, gap)? {
                *compact_changes = intrinsic.entries.iter().any(|entry| {
                    if horizontal {
                        entry.preferred.min(inner_width) > cap
                    } else {
                        preferred_column > cap
                    }
                });
            }
        }
        let cap = if compact {
            let Some(width) = crate::layout::kernel::compact_half_width(inner_width, gap)? else {
                return Ok(FitOutcome::Miss(scope.failure(
                    FitKind::LegendOrGuide,
                    "compact guide has no column width",
                )));
            };
            width
        } else {
            inner_width
        };
        for (index, entry) in intrinsic.entries.iter().enumerate() {
            let allocated = if horizontal {
                entry.preferred.min(cap)
            } else {
                preferred_column.min(cap)
            };
            if allocated < entry.minimum {
                return Ok(FitOutcome::Miss(
                    scope
                        .child(AddressRole::LegendEntry, index, &resources)?
                        .failure(
                            FitKind::Text,
                            "guide entry allocation is below its hard unbroken requirement",
                        ),
                ));
            }
        }
        let mut entry_blocks = Vec::new();
        let mut measured = Vec::new();
        fit::reserve(
            theme,
            &mut entry_blocks,
            entries.len(),
            "measured guide labels",
        )?;
        fit::reserve(
            theme,
            &mut measured,
            entries.len(),
            "measured guide entries",
        )?;
        let title_half = GLYPH_HALF_HEIGHT_FACTOR * f64::from(title_appearance.font_size);
        let body_top = title_half - title_plan.height() - title_gap;
        for (index, (entry, width)) in entries.iter().zip(intrinsic.entries).enumerate() {
            let allocated = if horizontal {
                width.preferred.min(cap)
            } else {
                preferred_column.min(cap)
            };
            let plan = match fit::text(
                theme,
                &entry.label,
                appearance,
                allocated - width.chrome,
                PhysicalTextAlignment::Start,
                "outside legend entry",
                &scope.child(AddressRole::LegendEntry, index, &resources)?,
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
        let body_height = body_height.unwrap_or_else(|| {
            measured
                .iter()
                .map(|entry| entry.ascent + entry.descent)
                .sum::<f64>()
                + metrics.row_gap * entries.len().saturating_sub(1) as f64
        });
        let flow = match flow_assigned_entries(
            &measured,
            inner_width,
            body_height,
            body_top,
            &metrics,
            horizontal,
            theme,
            scope,
        )? {
            FitOutcome::Measured(flow) => flow,
            FitOutcome::Miss(reason) => return Ok(FitOutcome::Miss(reason)),
        };
        let title_baseline = title_plan.first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
        let layout = LegendLayout {
            width: crate::layout::kernel::required_length(
                (flow
                    .width
                    .max(title_plan.width().max(title_plan.fit_width()))
                    + 2.0 * metrics.pad)
                    .max(frame_minimum),
            )?,
            height: crate::layout::kernel::required_length(
                (title_plan.height() + title_gap + flow.height + 2.0 * edge_pad).max(frame_minimum),
            )?,
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
                .legend_label
                .stack
                .copy_with_stage(
                    &theme
                        .prepared_font_environment()?
                        .font_resources()
                        .map_err(|error| Error::Render(error.to_string()))?,
                    None,
                )
                .map_err(|error| Error::Render(error.to_string()))?,
            layout,
        };
        let bounds = legend.box_at(0.0, 0.0);
        let mut ink = crate::render::geometry::GeometrySnapshot::new(theme)?;
        let (x, y) = bounds.anchor();
        legend.visit_calls(x, y, |call| {
            legend_visits(theme, 1)?;
            call.geometry(&mut ink)
        })?;
        for ink in ink.bounds() {
            legend_visits(theme, 1)?;
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
            f64::from(self.metrics.frame_stroke_width),
            self.metrics.frame_radius,
        ] {
            round.observe_number(metric)?;
        }
        for paint in [self.metrics.frame_fill, self.metrics.frame_outline] {
            for channel in [paint.red(), paint.green(), paint.blue(), paint.alpha()] {
                round.observe_count(usize::from(channel))?;
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
    /// Admit the exact slice allocation before converting measured entries to Arc.
    pub(crate) fn admit_entries(
        entries: Vec<LegendEntry>,
        theme: &crate::theme::ResolvedTheme,
    ) -> Result<std::sync::Arc<[LegendEntry]>, Error> {
        let allocation = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(
                std::alloc::Layout::array::<LegendEntry>(entries.len())
                    .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
            )
            .map_err(|_| Error::Render("legend entry Arc layout overflow".into()))?
            .0
            .pad_to_align();
        legend_visits(theme, entries.len())?;
        theme
            .prepared_font_environment()?
            .claim_unicode_bytes(
                u64::try_from(allocation.size())
                    .map_err(|_| Error::Render("legend entry storage overflow".into()))?,
            )
            .map_err(|error| Error::Render(format!("legend entry Arc admission: {error}")))?;
        // Stable Arc has no fallible slice constructor; admit its exact layout first.
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

    /// Pick the preferred right-inside-axes legend box.
    #[must_use]
    #[cfg(test)]
    pub fn right_box(&self, pixel_box: (f64, f64, f64, f64)) -> LegendBox {
        let (x0, x1, y0, y1) = pixel_box;
        let plot_short = (x1 - x0).min(y1 - y0);
        let inset = plot_short * 0.04;
        let width = self.compute_box_width();
        let height = self.compute_box_height();
        let bottom = f64::midpoint(y0, y1) - height / 2.0;
        self.box_from_rect(x1 - inset - width, bottom, width, height)
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

    pub(crate) fn select(
        &self,
        x: f64,
        y: f64,
        data: crate::data::PlotData<'_>,
    ) -> Result<LegendDrawPlan, Error> {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::LegendSelection,
        );
        data.claim_source_visits(self.entries.len())?;
        let count = self.entries.iter().try_fold(2_usize, |count, entry| {
            count
                .checked_add(
                    1 + usize::from(entry.glyph.fill.is_some())
                        + usize::from(entry.glyph.line.is_some())
                        + usize::from(entry.glyph.point.is_some()),
                )
                .ok_or_else(|| Error::Render("legend call count overflow".into()))
        })?;
        let mut calls = Vec::new();
        data.reserve_source(&mut calls, count)?;
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

        let bg_fill = FillStyle::new(m.frame_fill, m.frame_outline, m.frame_stroke_width);
        let inset =
            crate::layout::kernel::required_length(f64::from(m.frame_stroke_width) / 2.0)?.max(0.5);
        emit(LegendCall::Background(
            [
                rx + inset,
                ry_bot + inset,
                box_w - 2.0 * inset,
                box_h - 2.0 * inset,
                m.frame_radius,
            ],
            bg_fill,
        ))?;

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
        self.metrics.pad + GLYPH_EDGE_PAD_FACTOR * self.entry_font_size()
    }

    fn column_count(&self) -> usize {
        self.layout.columns
    }
}

#[cfg(test)]
pub(crate) fn test_assigned_legend(
    title: &str,
    entries: Vec<LegendEntry>,
    available: (f64, f64),
    theme: &crate::theme::ResolvedTheme,
) -> Result<Legend, Error> {
    let entries = Legend::admit_entries(entries, theme)?;
    match Legend::measure_assigned(
        title,
        entries,
        available,
        false,
        false,
        theme,
        &mut false,
        &crate::layout::fit::FitScope::root(),
    )? {
        crate::layout::fit::FitOutcome::Measured(legend) => Ok(legend),
        crate::layout::fit::FitOutcome::Miss(reason) => Err(reason.into_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_private_discrete_metrics_do_not_acquire_a_new_size_floor() {
        let mut theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme.typography.legend_title.font_size = 2;
        theme.typography.legend_label.font_size = 2;
        theme.legend.padding = 0.0;
        theme.legend.key_width = 1.0;
        theme.legend.key_label_gap = 0.0;
        theme.legend.title_gap = 0.0;
        theme.legend.row_gap = 0.0;
        theme.legend.frame_stroke_width = 0.0;
        theme.legend.frame_radius = 0.0;
        let mut key = LegendEntry::point(
            LegendIdentity::literal("i", "i"),
            PointStyle::plain((0, 0, 0), 1),
        );
        key.title = "i".into();
        key.label = "i".into();
        let guide = test_assigned_legend("i", vec![key], (10.0, 10.0), &theme).unwrap();
        assert!(guide.compute_box_width() < 10.0);
        assert!(guide.compute_box_height() < 10.0);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "local and merged guides must preserve every stroked glyph operand in both flows"
    )]
    fn local_and_shared_discrete_guides_allocate_complete_custom_glyph_and_frame_ink() {
        use crate::layout::fit::{self, FitScope};
        use crate::render::backend::{Backend, PointOutline};
        use crate::theme::Color;
        let source = crate::Plot::scatter(
            crate::NumericValues::new("x", [0.0, 1.0]),
            crate::NumericValues::new("y", [0.0, 1.0]),
        )
        .hue(crate::CategoryValues::new("group", ["a", "b"]).unwrap())
        .erase();
        let prepared =
            crate::plots::shared_x::prepare_plot(&source, &OperationGenerationResources::new())
                .unwrap();
        let plot = prepared.attach(source).unwrap();
        for shape in [crate::PointShape::Circle, crate::PointShape::TriangleUp] {
            let mut theme = crate::Theme::Publication
                .resolve(&OperationGenerationResources::new())
                .unwrap();
            theme.marks.primary_point.shape = shape;
            theme.marks.point_sizes.scatter = 6;
            theme.marks.primary_point.outline = PointOutline {
                color: Color::rgb(11, 22, 33),
                width_ratio: crate::theme::RelativeSpacing::ratio(4, 1).unwrap(),
            };
            theme.legend.frame_stroke_width = 30.0;
            theme.legend.frame_radius = 50.0;
            let result = plot
                .measure_selected_interior(crate::spec::PlotRenderContext::new(
                    plot.data(),
                    crate::layout::PanelBox::new(10.0, 700.0, 10.0, 700.0),
                    &theme,
                ))
                .unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let mut local = result.legend_entries;
            assert_eq!(local.len(), 2);
            let mut composite = LegendEntry::point_line_fill(
                LegendIdentity::single("group", "composite"),
                local[0].glyph.point.clone().unwrap(),
                LineStyle::solid(Color::rgb(20, 40, 60), 40.0),
                FillStyle::new(Color::rgba(10, 20, 30, 140), Color::rgb(40, 50, 60), 36.0),
            );
            composite.title.clone_from(&local[0].title);
            composite.label = "Composite".into();
            local.push(composite);
            let shared = merge_exact_entries(
                local.iter().cloned().chain(local.iter().cloned()),
                "shared test guide",
                &resources,
            )
            .unwrap();
            for entries in [local, shared] {
                let entries = Legend::admit_entries(entries, &theme).unwrap();
                let natural = Legend::natural_preference("Group", &entries, &theme).unwrap();
                for horizontal in [false, true] {
                    let guide = fit::measured(Legend::measure_assigned(
                        "Group",
                        entries.clone(),
                        (natural.0 + 300.0, natural.1 + 300.0),
                        horizontal,
                        false,
                        &theme,
                        &mut false,
                        &FitScope::root(),
                    ))
                    .unwrap();
                    let bounds = guide.box_at(20.0, 30.0);
                    let (x, y) = bounds.anchor();
                    let selected = guide
                        .select(x, y, plot.data().with_axis_resources(&resources))
                        .unwrap();
                    let mut ink = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
                    selected.geometry(&mut ink).unwrap();
                    assert!(ink.bounds().iter().all(|ink| ink.left >= bounds.left
                        && ink.right <= bounds.right
                        && ink.bottom >= bounds.bottom
                        && ink.top <= bounds.top));
                    let mut drawn = crate::render::geometry::GeometryBackend::new(&theme).unwrap();
                    drawn.begin(800, 800).unwrap();
                    selected.draw(&mut drawn).unwrap();
                    drawn.end().unwrap();
                    assert_eq!(ink.bounds(), drawn.bounds());
                    assert_eq!(selected.calls.iter().filter(|call| matches!(call, LegendCall::Point(_, style) if style.shape == shape)).count(), 3);
                    let LegendCall::Background(rect, style) = &selected.calls[0] else {
                        panic!("selected frame")
                    };
                    assert_eq!(rect[4].to_bits(), 50.0_f64.to_bits());
                    assert_eq!(style.stroke_width.to_bits(), 30.0_f32.to_bits());
                    assert!(matches!(
                        Legend::measure_assigned(
                            "Group",
                            entries.clone(),
                            (129.0, 600.0),
                            horizontal,
                            false,
                            &theme,
                            &mut false,
                            &FitScope::root()
                        )
                        .unwrap(),
                        fit::FitOutcome::Miss(_)
                    ));
                }
            }
        }
    }

    #[test]
    fn assigned_outside_frame_uses_the_owner_theme_gap_once() {
        let mut theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        theme.legend.outside_gap = 24.0;
        let legend =
            test_assigned_legend("group", vec![entry("one")], (300.0, 240.0), &theme).unwrap();
        let width = legend.compute_box_width();
        let crate::layout::fit::FitOutcome::Measured((panel, placed)) =
            crate::decorations::plan::place_outside_legend(
                legend,
                crate::LegendPosition::OutsideRight,
                crate::layout::PanelBox::new(0.0, 500.0, 0.0, 300.0),
                (40.0, 40.0),
                &crate::layout::fit::FitScope::root(),
            )
            .unwrap()
        else {
            panic!("right guide and containing panel must fit");
        };
        assert!((panel.x1 - (500.0 - width - 24.0)).abs() < 0.01);
        assert!((placed.bounds.left - panel.x1 - 24.0).abs() < 0.01);
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "horizontal, vertical, and compact worked cases share one measured-flow fixture"
    )]
    fn assigned_discrete_flow_matches_horizontal_and_vertical_worked_examples() {
        use crate::layout::fit::{FitOutcome, FitScope};
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let mut metrics = LegendMetrics::from_theme(&theme);
        metrics.column_gap = 5.0;
        metrics.row_gap = 4.0;
        let horizontal = [(60.0, 6.0, 4.0), (50.0, 6.0, 4.0), (90.0, 10.0, 10.0)].map(
            |(width, ascent, descent)| MeasuredEntry {
                width,
                key: metrics.handle_w,
                chrome: 20.0,
                ascent,
                descent,
            },
        );
        let FitOutcome::Measured(rows) = flow_assigned_entries(
            &horizontal,
            100.0,
            50.0,
            0.0,
            &metrics,
            true,
            &theme,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("worked rows must fit");
        };
        assert_eq!((rows.width, rows.height, rows.columns), (90.0, 48.0, 1));
        assert_eq!(
            rows.slots.iter().map(|slot| slot.key.x).collect::<Vec<_>>(),
            [0.0; 3]
        );
        assert_eq!(
            rows.slots.iter().map(|slot| slot.key.y).collect::<Vec<_>>(),
            [-6.0, -20.0, -38.0]
        );

        metrics.column_gap = 10.0;
        metrics.row_gap = 5.0;
        let vertical = [60.0, 70.0, 50.0].map(|_| MeasuredEntry {
            width: 70.0,
            key: metrics.handle_w,
            chrome: 20.0,
            ascent: 10.0,
            descent: 10.0,
        });
        let FitOutcome::Measured(columns) = flow_assigned_entries(
            &vertical,
            150.0,
            45.0,
            0.0,
            &metrics,
            false,
            &theme,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("worked columns must fit");
        };
        assert_eq!(
            (columns.width, columns.height, columns.columns),
            (150.0, 45.0, 2)
        );
        assert_eq!(
            columns
                .slots
                .iter()
                .map(|slot| slot.key.x)
                .collect::<Vec<_>>(),
            [0.0, 0.0, 80.0]
        );
        assert!(matches!(
            flow_assigned_entries(
                &vertical,
                140.0,
                45.0,
                0.0,
                &metrics,
                false,
                &theme,
                &FitScope::root()
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));
        let compact = vertical.map(|entry| MeasuredEntry {
            width: 65.0,
            ..entry
        });
        let FitOutcome::Measured(columns) = flow_assigned_entries(
            &compact,
            140.0,
            45.0,
            0.0,
            &metrics,
            false,
            &theme,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("compact columns must fit after remeasurement");
        };
        assert_eq!(
            (columns.width, columns.height, columns.columns),
            (140.0, 45.0, 2)
        );
    }

    #[test]
    fn entries_in_one_row_share_a_baseline_and_union_their_ink_heights() {
        use crate::layout::fit::{FitOutcome, FitScope};
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let metrics = LegendMetrics::from_theme(&theme);
        let entries = [(6.0, 4.0), (9.0, 1.0)].map(|(ascent, descent)| MeasuredEntry {
            width: 40.0,
            key: metrics.handle_w,
            chrome: 20.0,
            ascent,
            descent,
        });
        let FitOutcome::Measured(flow) = flow_assigned_entries(
            &entries,
            100.0,
            13.0,
            0.0,
            &metrics,
            true,
            &theme,
            &FitScope::root(),
        )
        .unwrap() else {
            panic!("both entries should share a row");
        };
        assert_eq!(flow.height.to_bits(), 13.0_f64.to_bits());
        assert_eq!(flow.columns, 2);
        assert_eq!(flow.slots[0].key.y.to_bits(), (-9.0_f64).to_bits());
        assert_eq!(
            flow.slots[1].label.position.y.to_bits(),
            (-9.0_f64).to_bits()
        );
    }

    #[test]
    fn legend_stack_copy_admits_literal_storage_and_work_before_copy() {
        use crate::font::canonical::{named_stack, FontFamily, OperationGenerationResources};
        let resources = OperationGenerationResources::new();
        let stack = named_stack(&["Group, condition"]);
        let bytes = (size_of::<FontFamily>() + "Group, condition".len()) as u64;
        let visits = stack.byte_work().unwrap() as u128;
        resources.set_test_limits(visits + 2, bytes);
        assert_eq!(stack.copy_with_generation(&resources).unwrap(), stack);
        assert_eq!(resources.usage().unwrap(), (1, visits));
        assert_eq!(resources.byte_usage().unwrap(), bytes);
        assert!(named_stack(&["x"])
            .copy_with_generation(&resources)
            .unwrap_err()
            .to_string()
            .contains("system-font bytes"));
        assert_eq!(resources.byte_usage().unwrap(), bytes);
    }

    #[test]
    fn empty_and_unusable_legend_measurements_admit_their_own_call() {
        use crate::layout::fit::{FitOutcome, FitScope};
        for empty in [false, true] {
            for remaining in [0, 1, 2] {
                let theme = crate::Theme::Publication
                    .resolve(&OperationGenerationResources::new())
                    .unwrap();
                theme.prepare_font_environment().unwrap();
                let resources = theme
                    .prepared_font_environment()
                    .unwrap()
                    .generation_resources();
                resources
                    .claim_calls(1_000_000 - resources.call_cache_usage().0 - remaining)
                    .unwrap();
                let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
                let entries: std::sync::Arc<[LegendEntry]> =
                    if empty { vec![] } else { vec![entry("group")] }.into();
                let mut compact_changes = true;
                let result = Legend::measure_assigned(
                    "",
                    entries,
                    (0.0, 0.0),
                    false,
                    false,
                    &theme,
                    &mut compact_changes,
                    &FitScope::root(),
                );
                if remaining == 0 {
                    assert!(result
                        .err()
                        .unwrap()
                        .to_string()
                        .contains("measurement/intrinsic call attempts"));
                    assert!(
                        compact_changes,
                        "refusal must precede trial output mutation"
                    );
                } else {
                    assert!(matches!(result.unwrap(), FitOutcome::Miss(_)));
                    assert!(!compact_changes);
                }
                assert_eq!(
                    resources.call_cache_usage().0,
                    1_000_000 - remaining + u128::from(remaining > 0)
                );
                assert_eq!(
                    crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
                    shapes
                );
            }
        }
    }

    #[test]
    fn repeated_legend_measurement_admits_calls_even_with_cached_text() {
        use crate::layout::fit::{self, FitScope};
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let entries: std::sync::Arc<[LegendEntry]> = vec![entry("first"), entry("second")].into();
        let measure = || {
            fit::measured(Legend::measure_assigned(
                "group",
                entries.clone(),
                (400.0, 400.0),
                false,
                false,
                &theme,
                &mut false,
                &FitScope::root(),
            ))
            .unwrap()
        };
        let original = measure();
        let before = resources.call_cache_usage().0;
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        let repeated = measure();
        // One guide call, one title query, and unassigned text + intrinsic +
        // assigned text queries per entry. Cached text admits all eight calls.
        assert_eq!(resources.call_cache_usage().0 - before, 8);
        assert_eq!(
            crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
            shapes
        );
        assert!(repeated
            .title_block
            .plan()
            .shares_payload(original.title_block.plan()));
        assert_eq!(
            repeated.layout.width.to_bits(),
            original.layout.width.to_bits()
        );
        assert_eq!(
            repeated.layout.height.to_bits(),
            original.layout.height.to_bits()
        );
    }

    #[test]
    fn assigned_guide_rejects_an_unbreakable_label_before_other_entries_wrap() {
        use crate::layout::fit::{FitOutcome, FitScope};
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let scope = FitScope::root();
        let entries: std::sync::Arc<[LegendEntry]> =
            vec![entry("first"), entry("M".repeat(40))].into();
        let FitOutcome::Miss(failure) = Legend::measure_assigned(
            "group",
            entries,
            (100.0, 400.0),
            true,
            false,
            &theme,
            &mut false,
            &scope,
        )
        .unwrap() else {
            panic!("the second unbroken label cannot fit its assigned width");
        };
        assert_eq!(failure.reason.kind, crate::layout::fit::FitKind::Text);
        assert_eq!(
            failure.reason.address,
            crate::layout::ComponentAddress::root()
                .child(
                    crate::layout::AddressRole::LegendEntry,
                    1,
                    &theme
                        .prepared_font_environment()
                        .unwrap()
                        .generation_resources()
                )
                .unwrap()
        );
    }

    #[test]
    fn assigned_guide_admits_long_label_arc_and_slots_before_text_work() {
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication
                .resolve(&OperationGenerationResources::new())
                .unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let entries = (0..16)
                .map(|index| {
                    entry(format!(
                        "Long measurement condition number {index} with repeated observations"
                    ))
                })
                .collect();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
            let legend = test_assigned_legend(
                "Long measurement conditions",
                entries,
                (1600.0, 1000.0),
                &theme,
            );
            (
                resources,
                legend,
                crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get) - shapes,
            )
        };
        let (resources, legend, _) = run(None);
        let legend = legend.unwrap();
        assert_eq!(legend.entry_blocks.len(), 16);
        assert_eq!(legend.layout.columns, 1);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for limits in [(visits - 1, bytes), (visits, bytes - 1)] {
            assert!(run(Some(limits)).1.is_err());
        }
        for limits in [(0, bytes), (visits, 0)] {
            let (_, result, shapes) = run(Some(limits));
            assert!(result.is_err());
            assert_eq!(shapes, 0, "entry admission must precede text planning");
        }
        let (_, exact, _) = run(Some((visits, bytes)));
        let exact = exact.unwrap();
        assert_eq!(exact.layout.columns, legend.layout.columns);
        assert_eq!(exact.layout.width.to_bits(), legend.layout.width.to_bits());
        assert_eq!(
            exact.layout.height.to_bits(),
            legend.layout.height.to_bits()
        );
    }

    #[test]
    fn resolved_pair_strings_merge_and_title_obey_exact_limits() {
        let plot = crate::Plot::matrix(vec![crate::NumericValues::new("x", [1.0, 2.0])]).erase();
        let run = |limits: Option<(u128, u64)>| {
            let theme = crate::Theme::Publication
                .resolve(&OperationGenerationResources::new())
                .unwrap();
            let resources = theme
                .prepared_font_environment()
                .unwrap()
                .generation_resources();
            let mut entries = (0..12)
                .map(|index| {
                    LegendEntry::point(
                        LegendIdentity::pair(
                            "long_group_field",
                            format!("measured_group_{index}"),
                            "long_hue_field",
                            "first_condition",
                        ),
                        PointStyle::new((0, 0, 0), 3),
                    )
                })
                .collect::<Vec<_>>();
            entries.push(entries[0].clone());
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let result = resolve_entries(&plot, &mut entries, &resources)
                .and_then(|()| resolved_title(&entries, &resources));
            (resources, entries, result)
        };
        let (resources, entries, title) = run(None);
        assert_eq!(entries.len(), 12);
        assert_eq!(title.unwrap(), "Long group field × Long hue field");
        assert_eq!(entries[0].label, "Measured group 0, First condition");
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        for limits in [
            (visits - 1, bytes),
            (visits, bytes - 1),
            (0, bytes),
            (visits, 0),
        ] {
            assert!(run(Some(limits)).2.is_err());
        }
        run(Some((visits, bytes))).2.unwrap();
    }

    fn entry(label: impl Into<String>) -> LegendEntry {
        let mut entry = LegendEntry::point(
            LegendIdentity::literal("group", "entry"),
            PointStyle::new((0, 0, 0), 3),
        );
        entry.title = "group".into();
        entry.label = label.into();
        entry
    }

    #[test]
    fn entry_compatibility_uses_operation_resources_without_font_discovery() {
        let prepared = |label: &str| {
            let mut entry = entry(label);
            entry.identity = LegendIdentity::literal("group", label);
            entry
        };
        let run = |limits: Option<(u128, u64)>| {
            let resources = OperationGenerationResources::new();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let result = merge_exact_entries(
                [prepared("first"), prepared("second"), prepared("first")],
                "prepared discrete guide",
                &resources,
            );
            (resources, result)
        };
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let (resources, result) = run(None);
        assert_eq!(result.unwrap().len(), 2);
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        assert!(visits > 0 && bytes > 0);
        assert_eq!(run(Some((visits, bytes))).1.unwrap().len(), 2);
        assert!(run(Some((visits - 1, bytes))).1.is_err());
        assert!(run(Some((visits, bytes - 1))).1.is_err());

        let resources = OperationGenerationResources::new();
        let mut conflicting = prepared("first");
        conflicting.glyph = LegendGlyph {
            point: Some(PointStyle::new((255, 0, 0), 3)),
            ..LegendGlyph::default()
        };
        assert!(matches!(
            merge_exact_entries([prepared("first"), conflicting], "shared guide", &resources),
            Err(Error::Render(message)) if message.contains("glyphs must match")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn signed_bar_identity_merges_across_panels_but_not_with_matching_literal_text() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let make = |identity| {
            LegendEntry::fill(
                identity,
                crate::render::backend::FillStyle::new(
                    crate::theme::Color::rgb(10, 20, 30),
                    crate::theme::Color::rgb(0, 0, 0),
                    1.0,
                ),
            )
        };
        let mut signs = vec![
            make(LegendIdentity::bar_sign(true)),
            make(LegendIdentity::bar_sign(true)),
            make(LegendIdentity::bar_sign(false)),
        ];
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        resolve_entries(&plot, &mut signs, &resources).unwrap();
        assert_eq!(signs.len(), 2);
        assert_eq!(signs[0].label, "Decreasing");
        assert_eq!(signs[1].label, "Nondecreasing");
        let mut literal = vec![make(LegendIdentity::literal("Change", "Decreasing"))];
        resolve_entries(&plot, &mut literal, &resources).unwrap();
        assert!(matches!(
            merge_exact_entries(
                [signs[0].clone(), literal.remove(0)],
                "shared guide",
                &resources,
            ),
            Err(Error::Render(message)) if message.contains("compatible semantic titles")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    #[test]
    fn semantic_entry_labels_resolve_before_font_discovery() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let resources = OperationGenerationResources::new();
        let mut entries = vec![LegendEntry::point(
            LegendIdentity::single("study_group", "control_group"),
            PointStyle::new((0, 0, 0), 3),
        )];
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        resolve_entries(&plot, &mut entries, &resources).unwrap();
        assert_eq!(entries[0].title, "Study group");
        assert_eq!(entries[0].label, "Control group");
        assert_eq!(resolved_title(&entries, &resources).unwrap(), "Study group");
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }

    fn legend() -> Legend {
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        test_assigned_legend(
            "group",
            vec![entry("first"), entry("second")],
            (640.0, 480.0),
            &theme,
        )
        .unwrap()
    }

    #[test]
    fn legend_text_roles_do_not_scale_with_assigned_size() {
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let smaller = test_assigned_legend(
            "group",
            vec![entry("one"), entry("two")],
            (640.0, 480.0),
            &theme,
        )
        .unwrap();
        let larger = test_assigned_legend(
            "group",
            vec![entry("one"), entry("two")],
            (1600.0, 1200.0),
            &theme,
        )
        .unwrap();
        assert_eq!(
            smaller.metrics.font_size.to_bits(),
            larger.metrics.font_size.to_bits()
        );
        assert_eq!(smaller.metrics.pad.to_bits(), larger.metrics.pad.to_bits());
        assert_eq!(
            smaller.title_block.plan().style.font_size,
            theme.typography.legend_title.font_size
        );
        assert!(smaller.entry_blocks.iter().all(|entry| {
            entry.plan().style.font_size == theme.typography.legend_label.font_size
        }));
        assert!(larger.entry_blocks.iter().all(|entry| {
            entry.plan().style.font_size == theme.typography.legend_label.font_size
        }));
    }

    #[test]
    fn compact_vertical_flow_remeasures_labels_and_fits_two_columns() {
        use crate::layout::fit::FitOutcome;
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let entries: std::sync::Arc<[LegendEntry]> =
            vec![entry("longer longer"), entry("a"), entry("b"), entry("c")].into();
        let probe = match Legend::measure_assigned(
            "group",
            std::sync::Arc::clone(&entries),
            (500.0, 1000.0),
            false,
            false,
            &theme,
            &mut true,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap()
        {
            FitOutcome::Measured(guide) => guide,
            FitOutcome::Miss(reason) => panic!("{reason}"),
        };
        let row_gap = probe.metrics.row_gap;
        let title_gap = probe.metrics.title_gap;
        let height = 2.0 * probe.glyph_edge_pad()
            + probe.title_block.height()
            + title_gap
            + 3.0 * probe.layout.entries[0].height
            + 2.0 * row_gap;
        let width = 2.0 * probe.layout.entries[0].width
            + probe.metrics.column_gap
            + 2.0 * probe.metrics.pad
            - 2.0;
        assert!(matches!(
            Legend::measure_assigned(
                "group",
                std::sync::Arc::clone(&entries),
                (width, height),
                false,
                false,
                &theme,
                &mut true,
                &crate::layout::fit::FitScope::root(),
            )
            .unwrap(),
            FitOutcome::Miss(_)
        ));
        let compact = match Legend::measure_assigned(
            "group",
            entries,
            (width, height),
            false,
            true,
            &theme,
            &mut true,
            &crate::layout::fit::FitScope::root(),
        )
        .unwrap()
        {
            FitOutcome::Measured(guide) => guide,
            FitOutcome::Miss(reason) => panic!("{reason}"),
        };
        assert_eq!(compact.layout.columns, 2);
        assert!(compact.entry_blocks[0].lines().len() > probe.entry_blocks[0].lines().len());
        assert!(compact.compute_box_width() <= width && compact.compute_box_height() <= height);
    }

    #[test]
    fn glyph_changes_invalidate_feedback_even_when_legend_extent_is_unchanged() {
        let mut legend = legend();
        let extent = [
            legend.compute_box_width().to_bits(),
            legend.compute_box_height().to_bits(),
        ];
        let mut calls = 0;
        let mut observations = None;
        crate::layout::plan_exact_root(
            640,
            480,
            |_| Ok(()),
            |(), assigned, round| {
                calls += 1;
                let owner = crate::layout::ComponentAddress::root();
                round.request_insets(
                    &owner,
                    &owner,
                    crate::layout::kernel::FeedbackKind::OwnedInk,
                    [1.0; 4],
                )?;
                if calls == 2 {
                    std::sync::Arc::make_mut(&mut legend.entries)[1]
                        .glyph
                        .point
                        .as_mut()
                        .unwrap()
                        .radius = 2;
                }
                assert_eq!(
                    extent,
                    [
                        legend.compute_box_width().to_bits(),
                        legend.compute_box_height().to_bits()
                    ]
                );
                let start = round.child_checkpoint();
                legend.observe_feedback(round)?;
                let current = round.retain_child_feedback(start)?.unwrap();
                if let Some(previous) = &observations {
                    assert_ne!(&current, previous);
                }
                observations = Some(current);
                Ok(crate::layout::MeasuredLayout::new(assigned, ()))
            },
            |(), _, (), _| Ok(()),
        )
        .unwrap();
        assert_eq!(calls, 2);
    }

    #[test]
    fn glyph_edge_padding_is_symmetric_and_retains_bottom_comfort() {
        let legend = legend();
        let bounds = legend.box_at(20.0, 30.0);
        let (_, title_center) = bounds.anchor();
        let title_top = title_center + legend.title_half_height();
        let last = legend.layout.entries.last().unwrap();
        let text = legend.entry_blocks.last().unwrap();
        let baseline = text.plan().first_baseline_quanta.unwrap_or(0) as f64 / 1024.0;
        let descent = (text.height() - baseline).max(
            -legend
                .entries
                .last()
                .unwrap()
                .glyph_ink(
                    &legend.metrics,
                    &crate::Theme::Publication
                        .resolve(&OperationGenerationResources::new())
                        .unwrap(),
                )
                .unwrap()
                .bottom,
        );
        let last_bottom = title_center + last.label.position.y - descent;
        let top_padding = bounds.top - title_top;
        let bottom_padding = last_bottom - bounds.bottom;
        let previous_bottom_comfort =
            legend.metrics.pad + GLYPH_EDGE_PAD_FACTOR * legend.entry_font_size();

        assert!((top_padding - bottom_padding).abs() <= 1.0 / 1024.0);
        assert!(top_padding >= previous_bottom_comfort);
    }

    #[test]
    fn eight_short_entries_use_one_column_when_the_assigned_height_fits() {
        let entries = (1..=8)
            .map(|index| entry(format!("item {index}")))
            .collect();
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let legend = test_assigned_legend("group", entries, (640.0, 480.0), &theme).unwrap();

        assert_eq!(legend.column_count(), 1);
    }

    #[test]
    fn overwide_legend_text_returns_the_planning_error() {
        let theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let error = test_assigned_legend(
            &"W".repeat(10_000),
            vec![entry("entry")],
            (160.0, 120.0),
            &theme,
        )
        .expect_err("overwide legend title must fail");

        assert!(error.to_string().contains("layout fit"));
        assert!(error.to_string().contains("LegendTitle"));
    }

    #[test]
    fn right_box_does_not_add_border_padding_twice() {
        let legend = legend();
        let bounds = legend.right_box((10.0, 410.0, 20.0, 300.0));

        assert!((bounds.right - bounds.left - legend.compute_box_width()).abs() <= 1.0e-9);
    }
}
