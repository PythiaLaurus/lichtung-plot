//! Producer-owned mapping provenance. Bindings describe actual selected inputs;
//! they are never inferred from equal RGB, glyphs or displayed labels.

use super::identity::{GuideEntryIdentity, RawCategory, RawField};
use super::ResolvedGuideKey;
use crate::alpha::{AlphaEncoding, AlphaTarget, AlphaUnitKind};
use crate::render::backend::{LineStyle, PointStyle};
use crate::{Error, Plot, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LiteralDomain {
    BarSign,
    DumbbellEndpoints,
    RegressionReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideEncoding {
    HueOnly { field: RawField },
    Joint { field: RawField },
    IndependentPair { group: RawField, hue: RawField },
    UnsupportedPair { group: RawField, hue: RawField },
    RowHue { field: RawField },
    Literal(LiteralDomain),
    SupplementalLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointRole {
    Primary,
    MandatoryStrip,
    Supplemental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FillRole {
    BarBody,
    BoxBody,
    ViolinBody,
    LineBand,
    RegressionBand,
    DensityArea,
    MarginalDensity,
    MatrixDensity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LineRole {
    SeriesLine,
    PathLine,
    DensityLine,
    InteractionConnector,
    SummaryStroke,
    RegressionFit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkPaintRole {
    PointFill(PointRole),
    PointOutline(PointRole),
    FillPaint(FillRole),
    FillOutline(FillRole),
    LineStroke(LineRole),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarkStyleRole {
    PointShape(PointRole),
    LinePattern(LineRole),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RgbBinding {
    Mapped { field: RawField, value: RawCategory },
    Role(MarkPaintRole),
    ExactElement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AlphaBinding {
    Role(MarkPaintRole),
    ExactElement(MarkPaintRole),
    Data {
        target: AlphaTarget,
        unit: AlphaUnitKind,
        input: AlphaEncoding,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AlphaBindings {
    pub(super) point_fill: Option<AlphaBinding>,
    pub(super) point_outline: Option<AlphaBinding>,
    pub(super) line_stroke: Option<AlphaBinding>,
    pub(super) fill_paint: Option<AlphaBinding>,
    pub(super) fill_outline: Option<AlphaBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AlphaExplanation {
    Constant,
    Neutralized,
    Explained,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AlphaExplanationMask {
    pub(super) point_fill: Option<AlphaExplanation>,
    pub(super) point_outline: Option<AlphaExplanation>,
    pub(super) line_stroke: Option<AlphaExplanation>,
    pub(super) fill_paint: Option<AlphaExplanation>,
    pub(super) fill_outline: Option<AlphaExplanation>,
}

impl AlphaBindings {
    pub(super) fn explanation(&self) -> AlphaExplanationMask {
        let selected = |binding: &Option<AlphaBinding>| {
            binding.as_ref().map(|binding| {
                if matches!(binding, AlphaBinding::Data { .. }) {
                    AlphaExplanation::Neutralized
                } else {
                    AlphaExplanation::Constant
                }
            })
        };
        AlphaExplanationMask {
            point_fill: selected(&self.point_fill),
            point_outline: selected(&self.point_outline),
            line_stroke: selected(&self.line_stroke),
            fill_paint: selected(&self.fill_paint),
            fill_outline: selected(&self.fill_outline),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StyleBinding {
    Role(MarkStyleRole),
    InteractionGroup { field: RawField, value: RawCategory },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PaintBindings {
    pub(super) point_fill: Option<RgbBinding>,
    pub(super) point_outline: Option<RgbBinding>,
    pub(super) line_stroke: Option<RgbBinding>,
    pub(super) fill_paint: Option<RgbBinding>,
    pub(super) fill_outline: Option<RgbBinding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StyleBindings {
    pub(super) point_shape: Option<StyleBinding>,
    pub(super) line_pattern: Option<StyleBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Projection {
    Complete,
    GroupStyle,
    HueColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RgbExplanation {
    Constant,
    ExplainedColor,
    NeutralizedColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StyleExplanation {
    Constant,
    ExplainedGroup,
    NeutralizedGroup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuideExplanationMask {
    pub(super) alpha: AlphaExplanationMask,
    pub(super) point_fill: Option<RgbExplanation>,
    pub(super) point_outline: Option<RgbExplanation>,
    pub(super) line_stroke: Option<RgbExplanation>,
    pub(super) fill_paint: Option<RgbExplanation>,
    pub(super) fill_outline: Option<RgbExplanation>,
    pub(super) point_shape: Option<StyleExplanation>,
    pub(super) line_pattern: Option<StyleExplanation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuideExplanation {
    Complete(AlphaExplanationMask),
    Projected(GuideExplanationMask),
}

impl Default for GuideExplanation {
    fn default() -> Self {
        Self::Complete(AlphaExplanationMask::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerLabelContext {
    Categorical,
    LiteralSeries,
}

impl LayerLabelContext {
    pub(crate) fn for_plot(plot: &Plot) -> Self {
        if plot.hue_dimension_key().is_some() || plot.interaction_styles_spec().is_some() {
            Self::Categorical
        } else {
            Self::LiteralSeries
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceMapping {
    pub(super) encoding: GuideEncoding,
    pub(super) paint_bindings: PaintBindings,
    pub(super) alpha_bindings: AlphaBindings,
    pub(super) style_bindings: StyleBindings,
    pub(super) layer_label_context: Option<LayerLabelContext>,
}

/// Transient complete producer key and its bindings. Only the final sectioned
/// descriptor retains resolved text and actual source occurrence records.
#[derive(Debug, Clone)]
pub(crate) struct GuideSourceKey {
    pub(crate) identity: GuideEntryIdentity,
    pub(crate) key: ResolvedGuideKey,
    pub(super) mapping: SourceMapping,
}

impl GuideSourceKey {
    fn new(identity: GuideEntryIdentity, encoding: GuideEncoding) -> Self {
        Self {
            identity,
            key: ResolvedGuideKey::default(),
            mapping: SourceMapping {
                encoding,
                paint_bindings: PaintBindings::default(),
                alpha_bindings: AlphaBindings::default(),
                style_bindings: StyleBindings::default(),
                layer_label_context: None,
            },
        }
    }

    pub(crate) fn hue_only(identity: GuideEntryIdentity) -> Result<Self> {
        let GuideEntryIdentity::Category { field, .. } = &identity else {
            return Err(invalid("hue-only encoding requires one raw category"));
        };
        let field = field.clone();
        Ok(Self::new(identity, GuideEncoding::HueOnly { field }))
    }

    pub(crate) fn row_hue(identity: GuideEntryIdentity) -> Result<Self> {
        let GuideEntryIdentity::Category { field, .. } = &identity else {
            return Err(invalid("row-hue encoding requires one raw category"));
        };
        let field = field.clone();
        Ok(Self::new(identity, GuideEncoding::RowHue { field }))
    }

    pub(crate) fn collective(
        identity: GuideEntryIdentity,
        plot: &Plot,
        group_styles: bool,
    ) -> Result<Self> {
        let encoding = match &identity {
            GuideEntryIdentity::Pair {
                group_field,
                hue_field,
                ..
            } => {
                if group_styles {
                    GuideEncoding::IndependentPair {
                        group: group_field.clone(),
                        hue: hue_field.clone(),
                    }
                } else {
                    GuideEncoding::UnsupportedPair {
                        group: group_field.clone(),
                        hue: hue_field.clone(),
                    }
                }
            }
            GuideEntryIdentity::Category { field, .. } => {
                if plot
                    .group_name()
                    .is_some_and(|group| Some(group) == plot.hue_name())
                    || (group_styles && plot.hue_name().is_none())
                {
                    GuideEncoding::Joint {
                        field: field.clone(),
                    }
                } else {
                    GuideEncoding::HueOnly {
                        field: field.clone(),
                    }
                }
            }
            _ => {
                return Err(invalid(
                    "collective encoding requires raw categorical identity",
                ))
            }
        };
        Ok(Self::new(identity, encoding))
    }

    pub(crate) fn literal(identity: GuideEntryIdentity, domain: LiteralDomain) -> Self {
        Self::new(identity, GuideEncoding::Literal(domain))
    }

    pub(crate) fn supplemental(identity: GuideEntryIdentity, context: LayerLabelContext) -> Self {
        let mut key = Self::new(identity, GuideEncoding::SupplementalLayer);
        key.mapping.layer_label_context = Some(context);
        key
    }

    fn rgb(&self, role: MarkPaintRole) -> RgbBinding {
        match &self.identity {
            GuideEntryIdentity::Category { field, value } => RgbBinding::Mapped {
                field: field.clone(),
                value: value.clone(),
            },
            GuideEntryIdentity::Pair {
                hue_field,
                hue_value,
                ..
            } => RgbBinding::Mapped {
                field: hue_field.clone(),
                value: hue_value.clone(),
            },
            _ => RgbBinding::Role(role),
        }
    }

    pub(crate) fn with_point(mut self, point: PointStyle, role: PointRole) -> Self {
        self.mapping.alpha_bindings.point_fill =
            Some(AlphaBinding::Role(MarkPaintRole::PointFill(role)));
        self.mapping.alpha_bindings.point_outline = point
            .outline
            .as_ref()
            .map(|_| AlphaBinding::Role(MarkPaintRole::PointOutline(role)));
        self.mapping.paint_bindings.point_fill = Some(self.rgb(MarkPaintRole::PointFill(role)));
        self.mapping.paint_bindings.point_outline = point
            .outline
            .as_ref()
            .map(|_| RgbBinding::Role(MarkPaintRole::PointOutline(role)));
        self.mapping.style_bindings.point_shape =
            Some(StyleBinding::Role(MarkStyleRole::PointShape(role)));
        self.key.point = Some(point);
        self.key.explanation =
            GuideExplanation::Complete(self.mapping.alpha_bindings.explanation());
        self
    }

    pub(crate) fn with_exact_point(self, point: PointStyle, role: PointRole) -> Self {
        let mut key = self.with_point(point, role);
        key.mapping.paint_bindings.point_fill = Some(RgbBinding::ExactElement);
        key.mapping.alpha_bindings.point_fill =
            Some(AlphaBinding::ExactElement(MarkPaintRole::PointFill(role)));
        key
    }

    pub(crate) fn with_line(mut self, line: LineStyle, role: LineRole) -> Self {
        self.mapping.alpha_bindings.line_stroke =
            Some(AlphaBinding::Role(MarkPaintRole::LineStroke(role)));
        self.mapping.paint_bindings.line_stroke = Some(self.rgb(MarkPaintRole::LineStroke(role)));
        self.mapping.style_bindings.line_pattern =
            Some(StyleBinding::Role(MarkStyleRole::LinePattern(role)));
        self.key.line = Some(line);
        self.key.explanation =
            GuideExplanation::Complete(self.mapping.alpha_bindings.explanation());
        self
    }

    pub(crate) fn with_role_line(self, line: LineStyle, role: LineRole) -> Self {
        let mut key = self.with_line(line, role);
        key.mapping.paint_bindings.line_stroke =
            Some(RgbBinding::Role(MarkPaintRole::LineStroke(role)));
        key
    }

    pub(crate) fn with_fill(
        mut self,
        fill: &crate::theme::DataFillTheme,
        color: (u8, u8, u8),
        role: FillRole,
    ) -> Self {
        self.mapping.paint_bindings.fill_paint = Some(self.rgb(MarkPaintRole::FillPaint(role)));
        self.mapping.paint_bindings.fill_outline = fill.outline().map(|outline| {
            if outline.mapped_width().is_some() {
                self.rgb(MarkPaintRole::FillOutline(role))
            } else {
                RgbBinding::Role(MarkPaintRole::FillOutline(role))
            }
        });
        self.key.fill = Some(fill.backend_style(color));
        self.mapping.alpha_bindings.fill_paint =
            Some(AlphaBinding::Role(MarkPaintRole::FillPaint(role)));
        self.mapping.alpha_bindings.fill_outline = fill
            .outline()
            .map(|_| AlphaBinding::Role(MarkPaintRole::FillOutline(role)));
        self.key.explanation =
            GuideExplanation::Complete(self.mapping.alpha_bindings.explanation());
        self
    }

    pub(crate) fn with_interaction_group_style(mut self) -> Result<Self> {
        let (field, value) = match &self.identity {
            GuideEntryIdentity::Category { field, value } => (field, value),
            GuideEntryIdentity::Pair {
                group_field,
                group_value,
                ..
            } => (group_field, group_value),
            _ => return Err(invalid("Interaction styles require a raw group identity")),
        };
        let binding = StyleBinding::InteractionGroup {
            field: field.clone(),
            value: value.clone(),
        };
        if self.key.point.is_some() {
            self.mapping.style_bindings.point_shape = Some(binding.clone());
        }
        if self.key.line.is_some() {
            self.mapping.style_bindings.line_pattern = Some(binding);
        }
        Ok(self)
    }

    pub(super) fn validate(&self, plot: &Plot) -> Result<()> {
        let paints = &self.mapping.paint_bindings;
        let styles = &self.mapping.style_bindings;
        if self.key.point.is_none() && self.key.line.is_none() && self.key.fill.is_none() {
            return Err(invalid("a source key must have a visible component"));
        }
        if self.key.point.is_some() != paints.point_fill.is_some()
            || self.key.point.is_some() != styles.point_shape.is_some()
            || self
                .key
                .point
                .as_ref()
                .is_some_and(|point| point.outline.is_some())
                != paints.point_outline.is_some()
            || self.key.line.is_some() != paints.line_stroke.is_some()
            || self.key.line.is_some() != styles.line_pattern.is_some()
            || self.key.fill.is_some() != paints.fill_paint.is_some()
            || self
                .key
                .fill
                .as_ref()
                .is_some_and(|fill| fill.stroke_width > 0.0)
                != paints.fill_outline.is_some()
        {
            return Err(invalid(
                "source bindings do not match actual key components",
            ));
        }
        self.validate_encoding()?;
        let alpha = &self.mapping.alpha_bindings;
        if paints.point_fill.is_some() != alpha.point_fill.is_some()
            || paints.point_outline.is_some() != alpha.point_outline.is_some()
            || paints.line_stroke.is_some() != alpha.line_stroke.is_some()
            || paints.fill_paint.is_some() != alpha.fill_paint.is_some()
            || paints.fill_outline.is_some() != alpha.fill_outline.is_some()
            || self.key.explanation != GuideExplanation::Complete(alpha.explanation())
        {
            return Err(invalid(
                "alpha bindings do not match actual complete key components",
            ));
        }
        if self
            .mapping
            .style_bindings
            .point_shape
            .iter()
            .chain(self.mapping.style_bindings.line_pattern.iter())
            .any(|binding| matches!(binding, StyleBinding::InteractionGroup { .. }))
            && (plot.kind() != crate::PlotKind::Interaction
                || plot.interaction_styles_spec().is_none())
        {
            return Err(invalid(
                "only configured Interaction styles map a group-style channel",
            ));
        }
        self.validate_bindings()
    }

    pub(super) fn bind_alpha(&mut self, plot: &Plot) -> Result<()> {
        let bindings = &mut self.mapping.alpha_bindings;
        bind_alpha(
            &mut bindings.point_fill,
            self.key.point.as_mut().map(|point| &mut point.color),
            plot,
        )?;
        bind_alpha(
            &mut bindings.line_stroke,
            self.key.line.as_mut().map(|line| &mut line.color),
            plot,
        )?;
        bind_alpha(
            &mut bindings.fill_paint,
            self.key.fill.as_mut().map(|fill| &mut fill.color),
            plot,
        )?;
        self.key.explanation = GuideExplanation::Complete(bindings.explanation());
        Ok(())
    }

    fn validate_encoding(&self) -> Result<()> {
        let matches = match (&self.mapping.encoding, &self.identity) {
            (
                GuideEncoding::HueOnly { field }
                | GuideEncoding::Joint { field }
                | GuideEncoding::RowHue { field },
                GuideEntryIdentity::Category { field: entry, .. },
            ) => field == entry,
            (
                GuideEncoding::IndependentPair { group, hue }
                | GuideEncoding::UnsupportedPair { group, hue },
                GuideEntryIdentity::Pair {
                    group_field,
                    hue_field,
                    ..
                },
            ) => group == group_field && hue == hue_field && group != hue,
            (
                GuideEncoding::Literal(LiteralDomain::BarSign),
                GuideEntryIdentity::BarSign { .. },
            )
            | (
                GuideEncoding::Literal(LiteralDomain::DumbbellEndpoints),
                GuideEntryIdentity::DumbbellEndpoint { .. },
            )
            | (
                GuideEncoding::Literal(LiteralDomain::RegressionReference),
                GuideEntryIdentity::RegressionObservation
                | GuideEntryIdentity::RegressionFit { .. },
            )
            | (GuideEncoding::SupplementalLayer, GuideEntryIdentity::SupplementalLayer { .. }) => {
                true
            }
            _ => false,
        };
        if !matches
            || matches!(self.mapping.encoding, GuideEncoding::SupplementalLayer)
                != self.mapping.layer_label_context.is_some()
        {
            return Err(invalid(
                "source encoding does not match its raw identity or label context",
            ));
        }
        Ok(())
    }

    fn validate_bindings(&self) -> Result<()> {
        let paints = &self.mapping.paint_bindings;
        let styles = &self.mapping.style_bindings;
        for (slot, binding) in [
            ("point fill", &paints.point_fill),
            ("point outline", &paints.point_outline),
            ("line stroke", &paints.line_stroke),
            ("fill paint", &paints.fill_paint),
            ("fill outline", &paints.fill_outline),
        ] {
            if let Some(RgbBinding::Mapped { field, value }) = binding {
                let valid = match &self.identity {
                    GuideEntryIdentity::Category {
                        field: expected,
                        value: raw,
                    } => field == expected && value == raw,
                    GuideEntryIdentity::Pair {
                        hue_field,
                        hue_value,
                        ..
                    } => field == hue_field && value == hue_value,
                    _ => false,
                };
                if !valid {
                    return Err(invalid(&format!(
                        "{slot} maps a different raw color identity"
                    )));
                }
            }
        }
        let mut group_style = false;
        for (slot, binding) in [
            ("point shape", &styles.point_shape),
            ("line pattern", &styles.line_pattern),
        ] {
            if let Some(StyleBinding::InteractionGroup { field, value }) = binding {
                let valid = match (&self.mapping.encoding, &self.identity) {
                    (
                        GuideEncoding::Joint { field: expected },
                        GuideEntryIdentity::Category { value: raw, .. },
                    ) => field == expected && value == raw,
                    (
                        GuideEncoding::IndependentPair { group, .. },
                        GuideEntryIdentity::Pair { group_value, .. },
                    ) => field == group && value == group_value,
                    _ => false,
                };
                if !valid {
                    return Err(invalid(&format!(
                        "{slot} maps a different raw group identity"
                    )));
                }
                group_style = true;
            }
        }
        if matches!(self.mapping.encoding, GuideEncoding::IndependentPair { .. }) && !group_style {
            return Err(invalid(
                "independent pairs require an actual group-style binding",
            ));
        }
        Ok(())
    }
}

fn bind_alpha(
    binding: &mut Option<AlphaBinding>,
    paint: Option<&mut crate::Color>,
    plot: &Plot,
) -> Result<()> {
    let Some(binding) = binding else {
        return Ok(());
    };
    let (AlphaBinding::Role(role) | AlphaBinding::ExactElement(role)) = binding else {
        return Err(invalid("data-alpha binding was installed more than once"));
    };
    let selected = match role {
        MarkPaintRole::PointFill(PointRole::Primary | PointRole::MandatoryStrip) => {
            Some((AlphaTarget::Point, AlphaUnitKind::Observation))
        }
        MarkPaintRole::LineStroke(role) => Some((
            AlphaTarget::Line,
            match role {
                LineRole::SeriesLine
                | LineRole::PathLine
                | LineRole::DensityLine
                | LineRole::InteractionConnector => AlphaUnitKind::Series,
                LineRole::SummaryStroke => AlphaUnitKind::Aggregate,
                LineRole::RegressionFit => AlphaUnitKind::Single,
            },
        )),
        MarkPaintRole::FillPaint(role) => Some((
            AlphaTarget::Fill,
            match role {
                FillRole::BarBody | FillRole::BoxBody | FillRole::ViolinBody => {
                    AlphaUnitKind::Aggregate
                }
                FillRole::LineBand
                | FillRole::DensityArea
                | FillRole::MarginalDensity
                | FillRole::MatrixDensity => AlphaUnitKind::Series,
                FillRole::RegressionBand => AlphaUnitKind::Single,
            },
        )),
        MarkPaintRole::PointFill(PointRole::Supplemental)
        | MarkPaintRole::PointOutline(_)
        | MarkPaintRole::FillOutline(_) => None,
    };
    if let Some((target, unit)) = selected {
        let input = plot.alpha_inputs().get(target);
        if !input.is_theme() {
            let paint = paint.ok_or_else(|| invalid("data-alpha binding has no actual paint"))?;
            *paint = paint.alpha(255);
            *binding = AlphaBinding::Data {
                target,
                unit,
                input: input.clone(),
            };
        }
    }
    Ok(())
}

fn invalid(detail: &str) -> Error {
    Error::Render(format!("invalid discrete guide provenance: {detail}"))
}
