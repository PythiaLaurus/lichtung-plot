//! Shared data preparation for vertical and horizontal bar plots.

#![allow(clippy::cast_precision_loss)]

use crate::alpha::{
    marks::{fill_style, line_style},
    AlphaTarget, AlphaUnitKind, ResolvedAlpha,
};
use crate::data::{
    category::{CategoryKey, RawField},
    source::{sort_numeric_source, sort_source_by},
    PlotData,
};
use crate::Error;

use crate::decorations::legend::source::{FillRole, LiteralDomain};
use crate::decorations::legend::{GuideEntryIdentity, GuideSourceKey};
use crate::orchestrator::registration::CategoricalBinding;
use crate::render::backend::{FillStyle, PointStyle};
use crate::spec::{
    BarInput, BarSource, BarSpacing, ConfidenceIntervalAlgorithm, ErrorBarKind, PercentileInterval,
    StandardDeviation, StandardError,
};
use crate::theme::{AccentColors, CategoryColors};
use crate::{CategoryOrder, Plot, ValueSort};

#[cfg(test)]
thread_local! {
    static BOOTSTRAP_DRAWS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Bar arrangement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarMode {
    /// One aggregate bar per x category.
    Single,
    /// One bar per x category and series, dodged side-by-side.
    Grouped,
    /// One additive segment per x category and series.
    Stacked,
}

pub(crate) struct BarSignificance {
    pub(crate) appearance: crate::theme::annotation::EffectiveSignificance,
    mode: BarMode,
    group_span: f64,
}

impl BarSignificance {
    pub(crate) fn new(
        chart: &Plot,
        theme: &crate::text::TextRequestContext<'_, '_>,
        horizontal: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            appearance: theme
                .effective_significance(chart.description().significance_mark_style_override())?,
            mode: mode(chart.description()),
            group_span: theme
                .marks
                .get_extents()
                .get_categories()
                .group_span(horizontal),
        })
    }

    pub(crate) fn center(&self, mark: &PreparedBarSignificance, series_count: usize) -> f64 {
        mark.series_index
            .map_or(mark.category_index as f64 + 0.5, |series| {
                category_center(
                    mark.category_index,
                    Some(series),
                    series_count,
                    self.mode,
                    self.group_span,
                )
            })
    }
}

/// The same categorical occupancy grammar drives selected polygons, attached
/// points/significance anchors and pre-selection mark pitch in both directions.
pub(crate) fn category_center(
    category_index: usize,
    series_index: Option<usize>,
    series_count: usize,
    mode: BarMode,
    group_span: f64,
) -> f64 {
    let center = category_index as f64 + 0.5;
    if mode != BarMode::Grouped {
        return center;
    }
    let slot_width = group_span;
    let slot = slot_width / series_count.max(1) as f64;
    center - slot_width / 2.0 + slot * (series_index.unwrap_or(0) as f64 + 0.5)
}

pub(crate) fn category_bar_geometry(
    bar: &PreparedBar,
    series_count: usize,
    mode: BarMode,
    spacing: BarSpacing,
    horizontal: bool,
    geometry: &crate::theme::mark::CategoryMarkGeometry,
) -> (f64, f64) {
    let center = category_center(
        bar.category_index,
        bar.series_index,
        series_count,
        mode,
        geometry.group_span(horizontal),
    );
    let thickness = match mode {
        BarMode::Single | BarMode::Stacked => geometry.body_breadth(horizontal),
        BarMode::Grouped => {
            let slot_width = geometry.group_span(horizontal);
            let slot = slot_width / series_count.max(1) as f64;
            match spacing {
                BarSpacing::Comfort => slot * 0.82,
                BarSpacing::Compact => slot,
            }
        }
    };
    (center, thickness)
}

/// Preserve actual occupied slots on the categorical axis; absent series
/// combinations never become bars, while scientific zero segments still do.
#[expect(
    clippy::too_many_lines,
    reason = "one categorical scan accounts for bar bodies, uncertainty caps, and supplemental points in either orientation"
)]
pub(crate) fn intrinsic_body_minimum(
    chart: &Plot,
    registered: Option<&crate::orchestrator::registration::PlotRegistration<'_>>,
    theme: &crate::text::TextRequestContext<'_, '_>,
) -> Result<(f64, f64), Error> {
    let prepared = retained(registered.map(|registered| registered.family.as_ref()))?;
    let series = &prepared.series;
    let horizontal = matches!(
        chart.kind(),
        crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
    );
    let categories = &registered
        .expect("prepared Bar occurrence")
        .axes
        .categories(usize::from(horizontal))
        .expect("prepared Bar categories")
        .raw;
    if categories.is_empty() || prepared.bars.is_empty() {
        return Err(Error::Render(
            "bar intrinsic has no prepared category or segment".into(),
        ));
    }
    let mode = mode(chart.description());

    let body = theme
        .marks
        .get_fills()
        .get_bodies()
        .get_bar_body()
        .backend_style((0, 0, 0));
    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = 0.0_f64;

    for bar in &prepared.bars {
        let body = fill_style(bar.fill_alpha, body.clone());
        let body_ink = crate::render::geometry::rectangular_fill_ink_bounds(&body)?;
        let outline = crate::render::geometry::rectangular_fill_ink_outset(&body)?;
        numeric = numeric.max(if horizontal { outline.0 } else { outline.1 });
        let interval = crate::render::geometry::line_ink_width(&line_style(
            bar.line_alpha,
            theme
                .marks
                .get_strokes()
                .get_summaries()
                .get_uncertainty()
                .backend_style(),
        ))?;
        if bar.category_index >= categories.len()
            || bar.series_index.is_some_and(|index| index >= series.len())
        {
            return Err(Error::Render(
                "bar occupied slot exceeds prepared category/series".into(),
            ));
        }
        let (center, thickness) = category_bar_geometry(
            bar,
            series.len(),
            mode,
            chart.description().bar_spacing_spec(),
            horizontal,
            theme.marks.get_extents().get_categories(),
        );
        // Distinct source slot edges must survive the canonical pixel quantum
        // even when their paint is transparent or the scientific height is zero.
        pitch = pitch.max((1.0 / 1024.0) / thickness);
        let relative = center - (bar.category_index as f64 + 0.5);
        let half = thickness / 2.0;
        if let Some(ink) = body_ink {
            let (low, high) = if horizontal {
                (ink.bottom, ink.top)
            } else {
                (ink.left, ink.right)
            };
            let left_room = 0.5 + relative - half;
            let right_room = 0.5 - relative - half;
            pitch = pitch.max(crate::layout::interior::ink_pitch(
                (-low).max(0.0),
                left_room,
            )?);
            pitch = pitch.max(crate::layout::interior::ink_pitch(
                high.max(0.0),
                right_room,
            )?);
        }
        if bar.interval.is_some() && interval > 0.0 {
            let cap = thickness
                * theme
                    .marks
                    .get_extents()
                    .get_categories()
                    .get_uncertainty_cap_ratio()
                    .as_f64();
            pitch = pitch.max((1.0 / 1024.0) / (2.0 * cap));
            let left_room = 0.5 + relative - cap;
            let right_room = 0.5 - relative - cap;
            if left_room <= 0.0 || right_room <= 0.0 {
                return Err(Error::Render(
                    "bar interval cap exceeds its declared slot".into(),
                ));
            }
            pitch = pitch.max((interval / 2.0) / left_room);
            pitch = pitch.max((interval / 2.0) / right_room);
            numeric = numeric.max(interval);
        }
    }

    for point in &prepared.points {
        if point.category_index >= categories.len()
            || point
                .series_index
                .is_some_and(|index| index >= series.len())
        {
            return Err(Error::Render(
                "bar supplemental point exceeds prepared category/series".into(),
            ));
        }
        let center = category_center(
            point.category_index,
            point.series_index,
            series.len(),
            mode,
            theme
                .marks
                .get_extents()
                .get_categories()
                .group_span(horizontal),
        );
        let relative = center - (point.category_index as f64 + 0.5);
        if let Some(ink) = crate::render::geometry::point_ink_bounds(&point.style)? {
            let (low, high) = if horizontal {
                (ink.bottom, ink.top)
            } else {
                (ink.left, ink.right)
            };
            pitch = pitch.max((-low).max(0.0) / (0.5 + relative));
            pitch = pitch.max(high.max(0.0) / (0.5 - relative));
            numeric = numeric.max(if horizontal {
                ink.right - ink.left
            } else {
                ink.top - ink.bottom
            });
        }
    }
    let categories = f64::from(
        u32::try_from(categories.len())
            .map_err(|_| Error::Render("bar category count exceeds coordinate range".into()))?,
    );
    let categorical = pitch * categories;
    if !categorical.is_finite() {
        return Err(Error::Render("bar category hard span overflowed".into()));
    }
    let numeric = if (if horizontal {
        registered.expect("prepared occurrence").raw_windows()[0]
    } else {
        registered.expect("prepared occurrence").raw_windows()[1]
    })
    .is_none()
    {
        numeric
    } else {
        0.0
    };
    Ok(if horizontal {
        (numeric, categorical)
    } else {
        (categorical, numeric)
    })
}

/// A prepared bar or stacked-bar segment in data coordinates.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBar {
    /// Index into the owning categorical axis's raw domain.
    pub(crate) category_index: usize,
    /// Optional index into the owning policy's positioned group domain.
    pub(crate) series_index: Option<usize>,
    /// Raw color identity, independent of local or parent palette order.
    pub(crate) hue_key: Option<CategoryKey>,
    /// Start value on the numeric axis.
    pub(crate) start: f64,
    /// End value on the numeric axis.
    pub(crate) end: f64,
    /// Whether the aggregate is nonzero before cumulative stack addition.
    pub(crate) nonzero: bool,
    /// Optional explicit lower and upper interval endpoints around `end`.
    pub(crate) interval: Option<(f64, f64)>,
    pub(crate) line_alpha: Option<ResolvedAlpha>,
    pub(crate) fill_alpha: Option<ResolvedAlpha>,
}

/// One attached significance label resolved to a rendered bar or category cluster.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBarSignificance {
    pub(crate) source_index: usize,
    pub(crate) category_index: usize,
    pub(crate) series_index: Option<usize>,
    pub(crate) edge: f64,
    pub(crate) positive: bool,
    pub(crate) label: String,
}

/// One strict supplemental point resolved to an ordinary bar anchor.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBarPoint {
    pub(crate) category_index: usize,
    pub(crate) series_index: Option<usize>,
    pub(crate) value: f64,
    pub(crate) style: PointStyle,
}

/// Prepared bar data and metadata shared by both orientations.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBars {
    pub(crate) series: Vec<String>,
    /// Prepared bars/segments.
    pub(crate) bars: Vec<PreparedBar>,
    /// Numeric value range including zero and CI/stack extents.
    pub(crate) value_range: (f64, f64),
    /// Supplemental points at resolved category/series centers.
    pub(crate) points: Vec<PreparedBarPoint>,
    /// Significance labels attached to exact bar/category anchors.
    pub(crate) significance: Vec<PreparedBarSignificance>,
}

impl PreparedBars {
    pub(crate) fn alpha_guide(
        &self,
        chart: &crate::builder::PlotDescription,
        categories: &[String],
        series: &[String],
        theme: &crate::theme::ResolvedTheme,
        guide: &mut crate::decorations::legend::alpha::AlphaGuideBuilder<'_>,
    ) -> crate::Result<()> {
        use crate::decorations::legend::alpha::{AlphaGlyphRole, AlphaUnit};
        let fill = theme.marks.get_fills().get_bodies().get_bar_body();
        for bar in &self.bars {
            let unit = AlphaUnit::aggregate(
                chart,
                &categories[bar.category_index],
                bar.series_index.map(|index| series[index].as_str()),
            )?;
            guide.fill(
                AlphaGlyphRole::Fill,
                AlphaUnitKind::Aggregate,
                unit.clone(),
                bar.fill_alpha,
                fill.backend_style((0, 0, 0)),
                fill.get_outline()
                    .is_some_and(|outline| outline.mapped_width().is_some()),
            )?;
            if bar.interval.is_some()
                && !matches!(
                    chart.kind(),
                    crate::PlotKind::StackedBar | crate::PlotKind::StackedHorizontalBar
                )
            {
                guide.line(
                    AlphaGlyphRole::Line,
                    AlphaUnitKind::Aggregate,
                    unit,
                    bar.line_alpha,
                    theme
                        .marks
                        .get_strokes()
                        .get_summaries()
                        .get_uncertainty()
                        .backend_style(),
                    false,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn observed_hue_levels<'a>(
        &'a self,
        hues: impl Iterator<Item = &'a str> + 'a,
    ) -> impl Iterator<Item = &'a str> {
        hues.filter(|hue| {
            self.bars.iter().any(|bar| {
                bar.hue_key
                    .as_ref()
                    .is_some_and(|key| key.as_ref().value == *hue)
            })
        })
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.bars.len() as u128
    }
}

pub(crate) fn resolved_colors(
    chart: &crate::builder::PlotDescription,
    axes: &crate::orchestrator::registration::PreparedAxesOwner,
    binding: &CategoricalBinding,
    theme: &crate::theme::ResolvedTheme,
    resources: &crate::font::canonical::OperationContext,
) -> Result<(CategoryColors, AccentColors), Error> {
    let colors = if chart.colors_by_sign() {
        binding.resolve(std::iter::empty(), theme, resources)?
    } else if let Some(categories) = binding.axis_categories(axes) {
        binding.resolve(categories.iter().map(String::as_str), theme, resources)?
    } else if binding.local_len() == 0 {
        let categories = &axes
            .categories(usize::from(matches!(
                chart.kind(),
                crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
            )))
            .expect("prepared Bar categories")
            .raw;
        binding.resolve(categories.iter().map(String::as_str), theme, resources)?
    } else {
        binding.resolve(binding.values(), theme, resources)?
    };
    let accents = AccentColors::resolve(
        theme.marks.get_accents(),
        if chart.colors_by_sign() { 2 } else { 0 },
    )?;
    Ok((colors, accents))
}

pub(crate) fn bar_color(
    chart: &Plot,
    categories: &[String],
    bar: &PreparedBar,
    colors: &CategoryColors,
    accents: &AccentColors,
) -> Result<(u8, u8, u8), Error> {
    if chart.description().colors_by_sign() {
        accents.color(usize::from(bar.end < bar.start))
    } else if let Some(hue) = &bar.hue_key {
        colors.color(hue.as_ref().value)
    } else {
        colors.color(&categories[bar.category_index])
    }
}

#[derive(Debug)]
pub(crate) struct BarShape {
    pub(crate) polygon: [crate::render::backend::Point2D; 4],
    pub(crate) fill: FillStyle,
    pub(crate) interval: Option<[[crate::render::backend::Point2D; 2]; 3]>,
    pub(crate) interval_style: crate::render::backend::LineStyle,
}

#[derive(Debug)]
pub(crate) struct BarPayload {
    pub(crate) clip: crate::layout::PixelRect,
    pub(crate) domain_expansion: Option<crate::layout::interior::DomainExpansion>,
    pub(crate) zero: Option<[crate::render::backend::Point2D; 2]>,
    pub(crate) zero_style: crate::render::backend::LineStyle,
    pub(crate) bars: Vec<BarShape>,
    pub(crate) text: Vec<crate::render::backend::PlacedText>,
    pub(crate) points: Vec<(crate::render::backend::Point2D, PointStyle)>,
}

/// One numeric-axis request from the selected complete bar, interval and
/// supplemental-point paints, merged with its existing significance domain.
pub(crate) fn automatic_numeric_ink_expansion(
    registered: &crate::orchestrator::registration::PlotRegistration<'_>,
    prepared: &PreparedBars,
    categories: &[String],
    payload: &BarPayload,
    projection: &crate::layout::Projection,

    scope: &crate::layout::fit::FitScope,
) -> Result<Option<crate::layout::interior::DomainExpansion>, crate::layout::fit::PlanError> {
    let horizontal = matches!(
        registered.source.kind(),
        crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
    );
    let automatic = if horizontal {
        registered.raw_windows()[0].is_none()
    } else {
        registered.raw_windows()[1].is_none()
    };
    if !automatic {
        return Ok(payload.domain_expansion);
    }
    let mut ink = crate::render::geometry::GeometrySnapshot::new();

    for bar in &payload.bars {
        ink.polygon(&bar.polygon, &bar.fill, None)?;
        if let Some(interval) = &bar.interval {
            for segment in interval {
                ink.line(segment, &bar.interval_style, None)?;
            }
        }
    }

    for (point, style) in &payload.points {
        ink.scatter_with_clip(std::slice::from_ref(point), style, None)?;
    }
    let Some(bounds) = ink
        .bounds()
        .iter()
        .copied()
        .reduce(crate::render::geometry::GeometryBox::union)
    else {
        return Ok(payload.domain_expansion);
    };
    let count = u32::try_from(categories.len())
        .map_err(|_| Error::Render("bar category count exceeds coordinate range".into()))?;
    let mut numeric = prepared.value_range;

    for point in &prepared.points {
        numeric.0 = numeric.0.min(point.value);
        numeric.1 = numeric.1.max(point.value);
    }
    let source = if horizontal {
        [numeric, (0.0, f64::from(count))]
    } else {
        [(0.0, f64::from(count)), numeric]
    };
    let proposals = crate::layout::interior::automatic_absolute_ink_expansions(
        source,
        projection,
        bounds,
        if horizontal {
            [true, false]
        } else {
            [false, true]
        },
        scope,
    )?;
    let merged = crate::layout::interior::merge_mark_domain_expansions(
        [payload.domain_expansion, None],
        proposals,
    );
    Ok(merged.into_iter().flatten().next())
}

impl BarPayload {
    pub(crate) fn draw(
        &self,
        backend: &mut dyn crate::render::backend::Backend,
    ) -> Result<(), Error> {
        crate::render::backend::with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.zero {
                backend.draw_line(line, &self.zero_style)?;
            }
            for bar in &self.bars {
                backend.draw_polygon(&bar.polygon, &bar.fill)?;
                for line in bar.interval.iter().flatten() {
                    backend.draw_line(line, &bar.interval_style)?;
                }
            }
            for text in &self.text {
                text.draw(backend)?;
            }
            for (point, style) in &self.points {
                backend.draw_scatter(std::slice::from_ref(point), style)?;
            }
            Ok(())
        })
    }

    fn measure_attached_text_ink(
        &mut self,
        projection: &crate::layout::Projection,

        registered: &crate::orchestrator::registration::PlotRegistration<'_>,
        prepared: &PreparedBars,
        categories: &[String],

        scope: &crate::layout::fit::FitScope,
    ) -> Result<(), crate::layout::fit::PlanError> {
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        let horizontal = matches!(
            registered.source.kind(),
            crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
        );
        let numeric_axis = usize::from(!horizontal);
        let automatic = if horizontal {
            registered.raw_windows()[0].is_none()
        } else {
            registered.raw_windows()[1].is_none()
        };
        if self.text.len() != prepared.significance.len() {
            return Err(Error::Render(
                "selected Bar annotation count differs from prepared targets".into(),
            )
            .into());
        }
        let mut full_text = crate::render::geometry::GeometrySnapshot::new();

        for (index, text) in self.text.iter().enumerate() {
            let mut ink = crate::render::geometry::GeometrySnapshot::new();
            ink.text(text.placement, &text.plan, None)?;
            for bounds in ink.bounds() {
                let categorical_overflow = if horizontal {
                    bounds.bottom < clip.y0 || bounds.top > clip.y1
                } else {
                    bounds.left < clip.x0 || bounds.right > clip.x1
                };
                let numeric_overflow = if horizontal {
                    bounds.left < clip.x0 || bounds.right > clip.x1
                } else {
                    bounds.bottom < clip.y0 || bounds.top > clip.y1
                };
                if categorical_overflow || (numeric_overflow && !automatic) {
                    return Err(scope
                        .child(crate::layout::AddressRole::Annotation, index)?
                        .failure(
                            crate::layout::fit::FitKind::OverlayCollision,
                            "required Bar annotation ink exceeds its assigned plot panel",
                        )
                        .into());
                }
            }
            full_text.append(&ink)?;
        }
        if automatic {
            if let Some(bounds) = full_text
                .bounds()
                .iter()
                .copied()
                .reduce(crate::render::geometry::GeometryBox::union)
            {
                let first = prepared.significance.first().ok_or_else(|| {
                    Error::Render("Bar annotation ink has no prepared target".into())
                })?;
                let mut extrema = (first.edge, first.edge);
                for target in &prepared.significance {
                    extrema.0 = extrema.0.min(target.edge);
                    extrema.1 = extrema.1.max(target.edge);
                }
                let category_range = (0.0, categories.len() as f64);
                let source = if horizontal {
                    [extrema, category_range]
                } else {
                    [category_range, extrema]
                };
                let requests = crate::layout::interior::automatic_absolute_ink_expansions(
                    source,
                    projection,
                    bounds,
                    if horizontal {
                        [true, false]
                    } else {
                        [false, true]
                    },
                    scope,
                )?;
                self.domain_expansion = crate::layout::interior::merge_mark_domain_expansions(
                    if horizontal {
                        [self.domain_expansion, None]
                    } else {
                        [None, self.domain_expansion]
                    },
                    requests,
                )[numeric_axis];
            }
        }
        Ok(())
    }

    pub(crate) fn retain(
        mut self,
        mut result: crate::spec::PlotRenderResult,
        context: crate::spec::PlotRenderContext<'_, '_>,
        chart: &Plot,
        prepared: &PreparedBars,
    ) -> Result<crate::spec::PlotRenderResult, crate::layout::fit::PlanError> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        self.measure_attached_text_ink(
            &result.layout.projection,
            context.registered.expect("prepared Bar occurrence"),
            prepared,
            &context
                .registered
                .expect("prepared Bar occurrence")
                .axes
                .categories(usize::from(matches!(
                    chart.kind(),
                    crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
                )))
                .expect("prepared Bar categories")
                .raw,
            context.fit_scope(),
        )?;
        if let Some(line) = &self.zero {
            geometry.line(line, &self.zero_style, Some(clip))?;
        }
        for bar in &self.bars {
            geometry.polygon(&bar.polygon, &bar.fill, Some(clip))?;
            for line in bar.interval.iter().flatten() {
                geometry.line(line, &bar.interval_style, Some(clip))?;
            }
        }
        for text in &self.text {
            geometry.text(text.placement, &text.plan, Some(clip))?;
        }
        for (point, style) in &self.points {
            geometry.scatter(std::slice::from_ref(point), style, clip)?;
        }
        if let Some(significance) = &result.significance {
            for bracket in &significance.brackets {
                geometry.bracket(crate::render::backend::PlannedSignificanceCall {
                    bracket: &bracket.geometry,
                    style: &bracket.style,
                    label: &bracket.label,
                })?;
            }
        }
        result.marks = Some(crate::data::source::source_arc(
            crate::layout::interior::SelectedMarkPlan {
                payload: crate::layout::interior::MarkPayload::Bars(self),
                geometry,
            },
        )?);
        Ok(result)
    }
}

pub(crate) fn mode(chart: &crate::builder::PlotDescription) -> BarMode {
    if matches!(
        chart.kind(),
        crate::PlotKind::StackedBar | crate::PlotKind::StackedHorizontalBar
    ) {
        BarMode::Stacked
    } else if chart.group_col().is_some() {
        BarMode::Grouped
    } else {
        BarMode::Single
    }
}

pub(crate) fn retained(
    family: Option<crate::orchestrator::registration::FamilyRef<'_>>,
) -> Result<&PreparedBars, Error> {
    family
        .and_then(|family| match family {
            crate::orchestrator::registration::FamilyRef::Bars { marks, .. } => Some(marks),
            _ => None,
        })
        .ok_or_else(|| Error::Render("bar measurement requires root-prepared semantics".into()))
}

pub(crate) fn domain_expansion(
    range: (f64, f64),
    marks: (f64, f64),
    brackets: (f64, f64),
    axis: crate::layout::interior::ClearanceAxis,
) -> Result<Option<crate::layout::interior::DomainExpansion>, Error> {
    let limits = (marks.0.min(brackets.0), marks.1.max(brackets.1));
    if limits == range {
        return Ok(None);
    }
    // Bracket bisection gives the minimum for this projection's pixel clearances.
    // Reuse the bar's anchored automatic padding so reprojected attached marks
    // do not chase that boundary asymptotically across parent feedback rounds.
    let expanded = crate::layout::anchored_auto_range(limits, None, 0.0)?;
    Ok(Some(crate::layout::interior::DomainExpansion {
        axis,
        limits: (range.0.min(expanded.0), range.1.max(expanded.1)),
    }))
}

pub(crate) fn attached_significance_text_plans(
    context: crate::spec::PlotRenderContext<'_, '_>,
    marks: &[PreparedBarSignificance],
    appearance: &crate::theme::TextTheme,
    alignment: impl Fn(&PreparedBarSignificance) -> crate::layout::PhysicalTextAlignment,
    owner: &str,
    exact_override: bool,
) -> Result<Vec<crate::layout::ResolvedTextPlan>, Error> {
    let eligible =
        crate::data::source::collect_source(marks.iter().map(|mark| Ok(mark.source_index)))?;
    let mut plans = crate::data::source::collect_source(marks.iter().map(|mark| {
        crate::layout::fit::measured(crate::layout::text::request::measure_occurrence(
            context.theme,
            &mark.label,
            crate::text::TextAppearance {
                theme: appearance,
                exact_override,
                layout: context.theme.text_layouts.significance.as_ref(),
            },
            i64::MAX,
            alignment(mark),
            owner,
            context.fit_scope(),
            (mark.source_index, &eligible),
        ))
        .map_err(crate::layout::fit::PlanError::into_error)
    }))?;
    crate::layout::text::font_fit::uniform(context.theme, &mut plans, context.fit_scope())
        .map_err(crate::layout::fit::PlanError::into_error)?;
    Ok(plans)
}

pub(crate) fn validate_attached_text(
    marks: &[PreparedBarSignificance],
    plans: &mut [crate::layout::ResolvedTextPlan],
    panel: crate::layout::PixelRect,
    scope: &crate::layout::fit::FitScope,
    vertical: bool,
    pending_domain: bool,
    place: impl Fn(
        &PreparedBarSignificance,
        &crate::layout::ResolvedTextPlan,
    ) -> Result<crate::render::backend::PlacedText, Error>,
) -> Result<(), crate::layout::fit::PlanError> {
    let mut text = crate::data::source::collect_source(
        marks
            .iter()
            .zip(plans.iter())
            .map(|(mark, plan)| place(mark, plan)),
    )?;
    let indices =
        crate::data::source::collect_source(marks.iter().map(|mark| Ok(mark.source_index)))?;
    crate::layout::text::visibility::apply(&mut text, &indices, panel, vertical, scope)?;
    for (index, label) in text.iter_mut().enumerate() {
        if !pending_domain {
            label.plan = crate::layout::text::placement::contain(
                label.plan.clone(),
                label.placement,
                panel,
                &scope.child(crate::layout::AddressRole::Annotation, indices[index])?,
            )?;
        }
    }
    crate::layout::text::visibility::require_nonoverlap(&mut text, scope)?;
    for (plan, text) in plans.iter_mut().zip(text) {
        *plan = text.plan;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct BarSummaryInput<'a> {
    units: Option<&'a [String]>,
    input: &'a BarInput,
    facet_identity: &'a FacetIdentity,
}

struct FacetIdentity(Vec<(String, String)>);

struct SingleBarCell {
    category_index: usize,
    hue_key: Option<CategoryKey>,
    values: Vec<f64>,
    units: Vec<String>,
}

struct GroupedBarCell {
    category_index: usize,
    series_index: usize,
    hue_key: Option<CategoryKey>,
    values: Vec<f64>,
    units: Vec<String>,
}

/// Build prepared bar data from category, value, and optional series channels.
pub(crate) fn prepare(
    chart: &crate::builder::PlotDescription,
    data: crate::data::PlotData<'_, '_>,
    mode: BarMode,
) -> Result<(PreparedBars, Vec<String>, Vec<String>), Error> {
    prepare_scoped(chart, data, mode, None, chart.significance_annotations())
}

pub(crate) fn prepare_scoped<'a>(
    chart: &crate::builder::PlotDescription,
    data: PlotData<'_, '_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
) -> Result<(PreparedBars, Vec<String>, Vec<String>), Error> {
    let (mut prepared, categories, series, hues) = prepare_core(chart, data, mode, facet)?;
    let category = data
        .column(
            chart
                .x_col()
                .ok_or_else(|| Error::Data("bar category column is missing".into()))?,
        )?
        .name();
    let mut fields = Vec::new();
    crate::data::source::reserve_source(&mut fields, 2)?;
    fields.push(category);
    fields.extend(chart.group_name());
    let fill_alpha = chart.prepare_scoped_alpha(
        AlphaTarget::Fill,
        AlphaUnitKind::Aggregate,
        &fields,
        data.facet_selector()?,
    )?;
    let line_alpha = if mode == BarMode::Stacked {
        None
    } else {
        Some(chart.prepare_scoped_alpha(
            AlphaTarget::Line,
            AlphaUnitKind::Aggregate,
            &fields,
            data.facet_selector()?,
        )?)
    };
    for bar in &mut prepared.bars {
        let key = (category, categories[bar.category_index].as_str());
        let series = chart
            .group_name()
            .zip(bar.series_index.map(|index| series[index].as_str()));
        bar.fill_alpha = fill_alpha.aggregate(key, series)?;
        if let Some(alpha) = line_alpha.as_ref().filter(|_| bar.interval.is_some()) {
            bar.line_alpha = alpha.aggregate(key, series)?;
        }
    }
    apply_attached_significance(annotations, &mut prepared, &categories, &series, mode)?;
    prepared.points = prepare_point_layers(chart, &prepared, mode, &categories, &series)?;
    for point in &prepared.points {
        prepared.value_range.0 = prepared.value_range.0.min(point.value);
        prepared.value_range.1 = prepared.value_range.1.max(point.value);
    }
    if let Some(exact) = chart
        .categorical_map_spec()
        .filter(|_| !chart.colors_by_sign())
    {
        let keys = if hues.is_empty() { &categories } else { &hues };
        for key in keys {
            if exact.color_in(key).is_none() {
                return Err(Error::Data(format!(
                    "bar: category {key:?} has no caller-owned color"
                )));
            }
        }
    }
    prepared.series = series;
    Ok((prepared, categories, hues))
}

#[expect(
    clippy::type_complexity,
    reason = "original build outputs move directly into their axis and policy owners without a retained wrapper"
)]
fn prepare_core(
    chart: &crate::builder::PlotDescription,
    data: PlotData<'_, '_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<(PreparedBars, Vec<String>, Vec<String>, Vec<String>), Error> {
    data.validate_row_source()?;
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;

    let categories = crate::data::strict_strings(data, x_col, "bar")?;
    let values = crate::data::strict_finite_f64(data, y_col, "bar")?;

    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&categories),
        "bar",
    )?;
    let group_order = crate::data::source::collect_source(
        facet
            .and_then(|sources| sources.groups)
            .unwrap_or(&channels.group_levels)
            .iter()
            .map(|s| crate::data::source::copy_source_string(s)),
    )?;
    let hue_order = crate::data::source::collect_source(
        facet
            .and_then(|sources| sources.hues)
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|s| crate::data::source::copy_source_string(s)),
    )?;
    let hue_field = chart.hue_field();
    let hues = hue_field.as_ref().zip(channels.hues.as_deref());
    let units = chart
        .bar_units_col()
        .map(|column| crate::data::strict_strings(data, column, "bar sampling units"))
        .transpose()?;
    let facet_identity = facet_identity(chart, data)?;
    let summary = || -> Result<_, Error> {
        Ok(BarSummaryInput {
            units: units.as_deref(),
            input: chart
                .bar_input()
                .ok_or_else(|| Error::Data("ordinary bar input is not configured".into()))?,
            facet_identity: &facet_identity,
        })
    };
    let (mut prepared, mut categories, series, hues) = match mode {
        BarMode::Single => prepare_single(categories, hues, values, hue_order, summary()?),
        BarMode::Grouped => {
            let groups = channels
                .groups
                .ok_or_else(|| Error::Data("clustered bar requires a series column".into()))?;
            prepare_grouped(
                categories,
                groups,
                hues,
                values,
                group_order,
                hue_order,
                summary()?,
            )
        }
        BarMode::Stacked => {
            let groups = channels
                .groups
                .ok_or_else(|| Error::Data("stacked bar requires a series column".into()))?;
            prepare_stacked(categories, groups, hues, values, group_order, hue_order)
        }
    }?;
    apply_category_order(
        &mut prepared,
        &mut categories,
        chart.category_order_spec(),
        mode,
        !data.is_full_plot(),
    )?;
    if let Some(bounds) = chart.bar_bound_cols() {
        apply_precomputed_bounds(chart, data, &mut prepared, &categories, &series, bounds)?;
    }
    Ok((prepared, categories, series, hues))
}

fn facet_identity(
    chart: &crate::builder::PlotDescription,
    data: crate::data::PlotData<'_, '_>,
) -> Result<FacetIdentity, Error> {
    let (row, column, wrap, _) = chart.facet_columns();
    let mut identity = Vec::new();
    for facet_column in [row, column, wrap].into_iter().flatten() {
        let values = crate::data::strict_strings(data, facet_column, "bar facet identity")?;
        let level = values
            .first()
            .ok_or_else(|| Error::Data("bar facet cell contains no observations".into()))?;

        if values.iter().any(|value| value != level) {
            return Err(Error::Data(
                "bar facet preparation mixed multiple levels in one panel".into(),
            ));
        }
        let name = data.column(facet_column)?.name();
        crate::data::source::reserve_source(&mut identity, 1)?;
        identity.push((
            crate::data::source::copy_source_string(name)?,
            crate::data::source::copy_source_string(level)?,
        ));
    }
    Ok(FacetIdentity(identity))
}

fn apply_attached_significance<'a>(
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
    prepared: &mut PreparedBars,
    categories: &[String],
    series_order: &[String],
    mode: BarMode,
) -> Result<(), Error> {
    for (source_index, annotation) in annotations.into_iter().enumerate() {
        let Some(target) = annotation.at_target() else {
            continue;
        };

        let category_index = categories
            .iter()
            .position(|category| category == target.category_name())
            .ok_or_else(|| {
                Error::Data(format!(
                    "bar significance references unknown category {:?}; available categories: {}",
                    target.category_name(),
                    categories.join(", ")
                ))
            })?;
        let series_index = target
            .series_name()
            .map(|series| {
                series_order
                    .iter()
                    .position(|candidate| candidate == series)
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "bar significance references unknown series {series:?}; available series: {}",
                            series_order.join(", ")
                        ))
                    })
            })
            .transpose()?;
        if target.series_name().is_some() && mode == BarMode::Single {
            return Err(Error::Data(
                "series-qualified significance target requires a configured bar series".into(),
            ));
        }
        if prepared
            .significance
            .iter()
            .any(|mark| mark.category_index == category_index && mark.series_index == series_index)
        {
            return Err(Error::Data(format!(
                "bar significance target {:?} is annotated more than once",
                target.category_name()
            )));
        }
        let bars = prepared.bars.iter().filter(|bar| {
            bar.category_index == category_index
                && series_index.is_none_or(|index| bar.series_index == Some(index))
        });
        let mut positive_edge = None::<f64>;
        let mut negative_edge = None::<f64>;
        for bar in bars {
            if bar.end >= 0.0 {
                let edge = bar
                    .interval
                    .map_or(bar.end, |(_, upper)| upper.max(bar.end));
                positive_edge = Some(positive_edge.map_or(edge, |current| current.max(edge)));
            } else {
                let edge = bar
                    .interval
                    .map_or(bar.end, |(lower, _)| lower.min(bar.end));
                negative_edge = Some(negative_edge.map_or(edge, |current| current.min(edge)));
            }
        }
        let (edge, positive) = positive_edge
            .map(|edge| (edge, true))
            .or_else(|| negative_edge.map(|edge| (edge, false)))
            .ok_or_else(|| Error::Data("bar significance target has no rendered bar".into()))?;
        crate::data::source::reserve_source(&mut prepared.significance, 1)?;
        prepared.significance.push(PreparedBarSignificance {
            source_index,
            category_index,
            series_index,
            edge,
            positive,
            label: crate::data::source::copy_source_string(annotation.mark().glyph()?)?,
        });
    }
    Ok(())
}

fn apply_precomputed_bounds(
    chart: &crate::builder::PlotDescription,
    data: crate::data::PlotData<'_, '_>,
    prepared: &mut PreparedBars,
    category_order: &[String],
    series_order: &[String],
    (lower_column, upper_column): (&str, &str),
) -> Result<(), Error> {
    let category_column = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let categories = crate::data::strict_strings(data, category_column, "bar estimate bounds")?;
    let groups = chart
        .group_col()
        .map(|column| crate::data::strict_strings(data, column, "bar estimate bounds"))
        .transpose()?;
    let lower = crate::data::strict_finite_f64(data, lower_column, "bar estimate bounds")?;
    let upper = crate::data::strict_finite_f64(data, upper_column, "bar estimate bounds")?;
    if lower.len() != categories.len() || upper.len() != categories.len() {
        return Err(Error::Data(
            "bar estimate bounds: category/lower/upper length mismatch".into(),
        ));
    }
    for row in 0..categories.len() {
        let category_index = category_order
            .iter()
            .position(|category| category == &categories[row])
            .ok_or_else(|| {
                Error::Data("bar estimate bounds category disappeared during preparation".into())
            })?;
        let series_index = groups
            .as_ref()
            .map(|values| {
                series_order
                    .iter()
                    .position(|group| group == &values[row])
                    .ok_or_else(|| {
                        Error::Data(
                            "bar estimate bounds series disappeared during preparation".into(),
                        )
                    })
            })
            .transpose()?;
        let bar = prepared
            .bars
            .iter_mut()
            .find(|bar| bar.category_index == category_index && bar.series_index == series_index)
            .ok_or_else(|| {
                Error::Data("bar estimate bounds cell disappeared during preparation".into())
            })?;
        if lower[row] > bar.end || bar.end > upper[row] {
            return Err(Error::Data(format!(
                "bar estimate bounds row {row} requires finite lower <= value <= upper; got {} <= {} <= {}",
                lower[row], bar.end, upper[row]
            )));
        }
        bar.interval = Some((lower[row], upper[row]));
    }

    prepared.value_range = value_range(&prepared.bars)?;
    Ok(())
}

fn prepare_point_layers(
    source: &crate::builder::PlotDescription,
    prepared: &PreparedBars,
    mode: BarMode,
    categories: &[String],
    series_order: &[String],
) -> Result<Vec<PreparedBarPoint>, Error> {
    if source.bar_point_layers().is_empty() {
        return Ok(Vec::new());
    }
    if mode == BarMode::Stacked {
        return Err(Error::Data(
            "bar point layers are not supported by stacked bars".into(),
        ));
    }
    let has_series = source.group_col().is_some();
    let mut points = Vec::new();
    for (layer_index, layer) in source.bar_point_layers().iter().enumerate() {
        let context = format!("bar point layer {layer_index}");
        if layer.series_values().is_some() != has_series {
            return Err(Error::Data(format!(
                "{context}: layer series configuration must match the bar chart"
            )));
        }
        let category_values = |values: &crate::CategoryValues| {
            crate::data::source::collect_source((0..values.len()).map(
                |row| match values.get(row) {
                    Some(Some(value)) => crate::data::source::copy_source_string(value),
                    _ => Err(Error::Data(format!(
                        "{context}: value {:?} contains null at source row {row}",
                        values.name()
                    ))),
                },
            ))
        };
        let layer_categories = category_values(layer.category_values())?;
        let series = layer.series_values().map(category_values).transpose()?;
        let values = crate::data::source::collect_source(
            (0..layer.values().len())
                .map(|row| crate::data::strict_numeric_value(layer.values(), row, &context)),
        )?;
        if layer_categories.is_empty() || layer_categories.len() != values.len() {
            return Err(Error::Data(format!(
                "{context}: category and value columns must have equal non-zero lengths"
            )));
        }
        for row in 0..values.len() {
            let category_index = categories
                .iter()
                .position(|value| value == &layer_categories[row])
                .ok_or_else(|| {
                    Error::Data(format!(
                        "{context}: unknown category {:?} at row {row}",
                        layer_categories[row]
                    ))
                })?;
            let series_index = series
                .as_ref()
                .map(|series_values| {
                    series_order
                        .iter()
                        .position(|value| value == &series_values[row])
                        .ok_or_else(|| {
                            Error::Data(format!(
                                "{context}: unknown series {:?} at row {row}",
                                series_values[row]
                            ))
                        })
                })
                .transpose()?;
            if !prepared
                .bars
                .iter()
                .any(|bar| bar.category_index == category_index && bar.series_index == series_index)
            {
                return Err(Error::Data(format!(
                    "{context}: category/series at row {row} does not resolve to a prepared bar cell"
                )));
            }
            crate::data::source::reserve_source(&mut points, 1)?;
            points.push(PreparedBarPoint {
                category_index,
                series_index,
                value: values[row],
                style: layer.point_style(),
            });
        }
    }
    Ok(points)
}

/// Validate cardinality and strict values for typed ordinary-bar inputs.
#[expect(
    clippy::too_many_lines,
    reason = "one pass preserves source-row, facet, category, and series validation context"
)]
pub(crate) fn validate_cells(chart: &crate::builder::PlotDescription) -> Result<(), Error> {
    let column = |key: &str| {
        chart
            .data_column(key)
            .ok_or_else(|| Error::Data(format!("unknown semantic value {key:?}")))
    };
    let strings = |key: &str, context: &str| {
        let column = column(key)?;
        let crate::data::DataColumn::Category(values) = column else {
            return Err(Error::Data(format!(
                "{context}: value {:?} must be categorical",
                column.name()
            )));
        };
        crate::data::strict_source_strings(values, None, context)
    };
    let numbers = |key: &str, context: &str| {
        let column = column(key)?;
        let crate::data::DataColumn::Numeric(values) = column else {
            return Err(Error::Data(format!(
                "{context}: value {:?} must be numeric",
                column.name()
            )));
        };
        crate::data::strict_source_f64(values, None, context)
    };
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;
    let categories = strings(x_col, "bar")?;
    let values = numbers(y_col, "bar")?;
    let category_name = column(x_col)?.name();
    let facet_columns = {
        let (row, column, wrap, _) = chart.facet_columns();
        [row, column, wrap]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };
    let facet_values = facet_columns
        .iter()
        .map(|column| strings(column, "bar"))
        .collect::<Result<Vec<_>, _>>()?;
    let facet_names = facet_columns
        .iter()
        .map(|key| column(key).map(crate::data::DataColumn::name))
        .collect::<Result<Vec<_>, _>>()?;
    let mark_cells = (0..values.len())
        .map(|index| {
            let mut identity = String::new();
            for field in facet_values
                .iter()
                .map(|facet| facet[index].as_str())
                .chain(std::iter::once(categories[index].as_str()))
            {
                identity.push_str(&field.len().to_string());
                identity.push(':');
                identity.push_str(field);
            }
            identity
        })
        .collect::<Vec<_>>();
    let channels = crate::data::source_collective_channels(chart, Some(&mark_cells), "bar")?;
    if categories.len() != values.len()
        || channels
            .groups
            .as_ref()
            .is_some_and(|groups| groups.len() != values.len())
        || facet_values.iter().any(|facet| facet.len() != values.len())
    {
        return Err(Error::Data(
            "bar: category/value/series/facet length mismatch".into(),
        ));
    }

    let mut cells: Vec<(Vec<String>, usize)> = Vec::new();
    for index in 0..values.len() {
        let mut key = facet_values
            .iter()
            .map(|facet| facet[index].clone())
            .collect::<Vec<_>>();
        key.push(categories[index].clone());
        if let Some(groups) = &channels.groups {
            key.push(groups[index].clone());
        }
        if let Some((_, rows)) = cells.iter_mut().find(|(existing, _)| existing == &key) {
            *rows += 1;
        } else {
            cells.push((key, 1));
        }
    }
    if cells.is_empty() {
        return Err(Error::Data(
            "bar input requires at least one rendered category cell".into(),
        ));
    }
    let input = chart
        .bar_input()
        .ok_or_else(|| Error::Data("ordinary bar input is not configured".into()))?;
    if let Some(units_column) = chart.bar_units_col() {
        strings(units_column, "bar sampling units")?;
    }
    if let Some((lower_column, upper_column)) = chart.bar_bound_cols() {
        let lower = numbers(lower_column, "bar estimate bounds")?;
        let upper = numbers(upper_column, "bar estimate bounds")?;
        for row in 0..values.len() {
            if lower[row] > values[row] || values[row] > upper[row] {
                return Err(Error::Data(format!(
                    "bar estimate bounds row {row} requires finite lower <= value <= upper; got {} <= {} <= {}",
                    lower[row], values[row], upper[row]
                )));
            }
        }
    }
    for (key, rows) in cells {
        let mut labels = facet_names
            .iter()
            .zip(&key)
            .map(|(column, level)| format!("facet {column}={level}"))
            .collect::<Vec<_>>();
        let category_index = facet_columns.len();
        labels.push(format!("category {category_name}={}", key[category_index]));
        if let Some(series_name) = chart.group_name() {
            labels.push(format!("series {series_name}={}", key[category_index + 1]));
        }
        let cell = labels.join(", ");
        if !input.is_samples() && rows != 1 {
            return Err(Error::Data(format!(
                "bar estimate cell `{cell}` has {rows} finite observations; exactly one is required"
            )));
        }
    }
    Ok(())
}

fn apply_category_order(
    prepared: &mut PreparedBars,
    categories: &mut Vec<String>,
    order: &CategoryOrder,
    mode: BarMode,
    is_subset: bool,
) -> Result<(), Error> {
    match order {
        CategoryOrder::Observed => Ok(()),
        CategoryOrder::Explicit(requested) => {
            let ordered = crate::data::explicit_category_order(
                categories,
                requested,
                "bar category order",
                is_subset,
            )?;
            let old_categories = std::mem::replace(categories, ordered);
            for bar in &mut prepared.bars {

                let label = &old_categories[bar.category_index];
                bar.category_index = categories
                    .iter()
                    .position(|candidate| candidate == label)
                    .ok_or_else(|| {
                        Error::Render(format!(
                            "bar category {label:?} was lost while applying explicit order"
                        ))
                    })?;
            }
            Ok(())
        }
        CategoryOrder::Value(sort) if mode == BarMode::Single => {
            sort_single_values(prepared, categories, *sort)
        }
        CategoryOrder::Value(_) => Err(Error::Data(
            "bar: value-derived category order requires one ungrouped bar per category; use an explicit category order for grouped or stacked bars"
                .into(),
        )),
    }
}

fn sort_single_values(
    prepared: &mut PreparedBars,
    categories: &mut Vec<String>,
    sort: ValueSort,
) -> Result<(), Error> {
    let mut order = crate::data::source::collect_source((0..categories.len()).map(Ok))?;
    sort_source_by(&mut order, |left, right| {
        let left_value = prepared.bars[*left].end;
        let right_value = prepared.bars[*right].end;
        (match sort {
            ValueSort::Ascending => left_value.total_cmp(&right_value),
            ValueSort::Descending => right_value.total_cmp(&left_value),
        })
        .then_with(|| left.cmp(right))
    });
    let ordered = crate::data::source::collect_source(
        order
            .iter()
            .map(|index| crate::data::source::copy_source_string(&categories[*index])),
    )?;
    let bars = crate::data::source::collect_source(order.iter().enumerate().map(
        |(category_index, old_index)| {
            let mut bar = prepared.bars[*old_index].clone();
            bar.category_index = category_index;
            Ok(bar)
        },
    ))?;
    *categories = ordered;
    prepared.bars = bars;
    Ok(())
}

/// Legend entries for positioned series.
pub(crate) fn guide_sources(
    prepared: &PreparedBars,
    chart: &crate::builder::PlotDescription,
    binding: &CategoricalBinding,
    theme: &crate::theme::ResolvedTheme,

    colors: &CategoryColors,
    accents: &AccentColors,
) -> Result<Vec<GuideSourceKey>, Error> {
    if chart.colors_by_sign() {
        let mut present = [false; 2];
        for bar in &prepared.bars {
            present[usize::from(bar.end < bar.start)] = true;
        }
        let mut entries = Vec::new();
        crate::data::source::reserve_source(&mut entries, 2)?;
        for decreasing in [false, true] {
            if present[usize::from(decreasing)] {
                entries.push(
                    GuideSourceKey::literal(
                        GuideEntryIdentity::bar_sign(decreasing),
                        LiteralDomain::BarSign,
                    )
                    .with_fill(
                        theme.marks.get_fills().get_bodies().get_bar_body(),
                        accents.color(usize::from(decreasing))?,
                        FillRole::BarBody,
                    ),
                );
            }
        }
        return Ok(entries);
    }

    let mut entries = Vec::new();
    crate::data::source::reserve_source(&mut entries, binding.local_len())?;
    for label in prepared.observed_hue_levels(binding.local_values()) {
        entries.push(
            GuideSourceKey::hue_only(chart.point_legend_identity(label)?)?.with_fill(
                theme.marks.get_fills().get_bodies().get_bar_body(),
                colors.color(label)?,
                FillRole::BarBody,
            ),
        );
    }
    Ok(entries)
}

#[expect(
    clippy::type_complexity,
    reason = "original build outputs move directly into their axis and policy owners without a retained wrapper"
)]
fn prepare_single(
    categories: Vec<String>,
    hues: Option<(&RawField, &[String])>,
    values: Vec<f64>,
    mut hue_levels: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<(PreparedBars, Vec<String>, Vec<String>, Vec<String>), Error> {
    let mut category_levels = Vec::new();
    let mut groups: Vec<SingleBarCell> = Vec::new();
    for (row, (category, value)) in categories.into_iter().zip(values).enumerate() {
        let category_index = level_index(&mut category_levels, category)?;
        let hue_key = prepare_hue_key(
            hues,
            row,
            &mut hue_levels,
            groups.iter().filter_map(|cell| cell.hue_key.as_ref()),
        )?;
        if let Some(index) = crate::data::source::source_position(&groups, |cell| {
            cell.category_index == category_index
        }) {
            let cell = &mut groups[index];
            crate::data::source::reserve_source(&mut cell.values, 1)?;
            cell.values.push(value);
            if let Some(units) = summary.units {
                crate::data::source::reserve_source(&mut cell.units, 1)?;
                cell.units
                    .push(crate::data::source::copy_source_string(&units[row])?);
            }
        } else {
            crate::data::source::reserve_source(&mut groups, 1)?;
            groups.push(SingleBarCell {
                category_index,
                hue_key,
                values: crate::data::source::collect_source(std::iter::once(Ok(value)))?,
                units: summary
                    .units
                    .map(|units| {
                        crate::data::source::collect_source(std::iter::once_with(|| {
                            crate::data::source::copy_source_string(&units[row])
                        }))
                    })
                    .transpose()?
                    .unwrap_or_default(),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::new();
    crate::data::source::reserve_source(&mut bars, groups.len())?;
    for cell in &groups {
        let category = &category_levels[cell.category_index];
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            || format!("category {category:?}"),
            category,
            None,
            summary,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: None,
            hue_key: cell.hue_key.clone(),
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
            line_alpha: None,
            fill_alpha: None,
        });
    }

    let value_range = value_range(&bars)?;
    Ok((
        PreparedBars {
            series: Vec::new(),
            value_range,
            bars,
            points: Vec::new(),
            significance: Vec::new(),
        },
        category_levels,
        Vec::new(),
        hue_levels,
    ))
}

#[expect(
    clippy::type_complexity,
    reason = "original build outputs move directly into their axis and policy owners without a retained wrapper"
)]
fn prepare_grouped(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<(&RawField, &[String])>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<(PreparedBars, Vec<String>, Vec<String>, Vec<String>), Error> {
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data(
            "bar: category/value/series length mismatch".into(),
        ));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<GroupedBarCell> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category)?;
        let series_index = level_index(&mut series_levels, series_label)?;
        let hue_key = prepare_hue_key(
            hues,
            row,
            &mut hue_levels,
            groups.iter().filter_map(|cell| cell.hue_key.as_ref()),
        )?;
        if let Some(index) = crate::data::source::source_position(&groups, |cell| {
            cell.category_index == category_index && cell.series_index == series_index
        }) {
            let cell = &mut groups[index];
            crate::data::source::reserve_source(&mut cell.values, 1)?;
            cell.values.push(value);
            if let Some(units) = summary.units {
                crate::data::source::reserve_source(&mut cell.units, 1)?;
                cell.units
                    .push(crate::data::source::copy_source_string(&units[row])?);
            }
        } else {
            crate::data::source::reserve_source(&mut groups, 1)?;
            groups.push(GroupedBarCell {
                category_index,
                series_index,
                hue_key,
                values: crate::data::source::collect_source(std::iter::once(Ok(value)))?,
                units: summary
                    .units
                    .map(|units| {
                        crate::data::source::collect_source(std::iter::once_with(|| {
                            crate::data::source::copy_source_string(&units[row])
                        }))
                    })
                    .transpose()?
                    .unwrap_or_default(),
            });
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }

    let mut bars = Vec::new();
    crate::data::source::reserve_source(&mut bars, groups.len())?;
    for cell in groups {
        let context = || {
            format!(
                "category {:?}, series {:?}",
                category_levels[cell.category_index], series_levels[cell.series_index]
            )
        };
        let (value, interval) = summarize(
            &cell.values,
            summary.units.map(|_| cell.units.as_slice()),
            context,
            &category_levels[cell.category_index],
            Some(&series_levels[cell.series_index]),
            summary,
        )?;
        bars.push(PreparedBar {
            category_index: cell.category_index,
            series_index: Some(cell.series_index),
            hue_key: cell.hue_key,
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
            line_alpha: None,
            fill_alpha: None,
        });
    }

    let value_range = value_range(&bars)?;

    Ok((
        PreparedBars {
            series: Vec::new(),
            value_range,
            bars,
            points: Vec::new(),
            significance: Vec::new(),
        },
        category_levels,
        series_levels,
        hue_levels,
    ))
}

#[expect(
    clippy::too_many_lines,
    reason = "admitted grouping and ordered signed stack accumulation share one scientific owner"
)]
#[expect(
    clippy::type_complexity,
    reason = "original build outputs move directly into their axis and policy owners without a retained wrapper"
)]
fn prepare_stacked(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<(&RawField, &[String])>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
) -> Result<(PreparedBars, Vec<String>, Vec<String>, Vec<String>), Error> {
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data(
            "bar: category/value/series length mismatch".into(),
        ));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<(usize, usize, Option<CategoryKey>, Vec<f64>)> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category)?;
        let series_index = level_index(&mut series_levels, series_label)?;
        let hue_key = prepare_hue_key(
            hues,
            row,
            &mut hue_levels,
            groups.iter().filter_map(|(_, _, key, _)| key.as_ref()),
        )?;
        if let Some(index) = crate::data::source::source_position(&groups, |(c, s, _, _)| {
            *c == category_index && *s == series_index
        }) {
            let (_, _, _, values) = &mut groups[index];
            crate::data::source::reserve_source(values, 1)?;
            values.push(value);
        } else {
            crate::data::source::reserve_source(&mut groups, 1)?;
            groups.push((
                category_index,
                series_index,
                hue_key,
                crate::data::source::collect_source(std::iter::once(Ok(value)))?,
            ));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }
    let sums = crate::data::source::collect_source(groups.into_iter().map(
        |(category_index, series_index, hue_key, values)| {
            stable_sum(&values).map_or_else(
                |error| {
                    Err(match error {
                        Error::Data(reason) => Error::Data(format!(
                            "bar: stacked category {:?}, series {:?} {reason}",
                            category_levels[category_index], series_levels[series_index]
                        )),
                        error => error,
                    })
                },
                |sum| Ok((category_index, series_index, hue_key, sum)),
            )
        },
    ))?;

    let mut positive_offsets =
        crate::data::source::collect_source((0..category_levels.len()).map(|_| Ok(0.0)))?;
    let mut negative_offsets =
        crate::data::source::collect_source((0..category_levels.len()).map(|_| Ok(0.0)))?;
    let mut bars = Vec::new();
    crate::data::source::reserve_source(&mut bars, sums.len())?;
    for series_index in 0..series_levels.len() {
        for category_index in 0..category_levels.len() {
            let Some(index) = crate::data::source::source_position(&sums, |(c, s, _, _)| {
                *c == category_index && *s == series_index
            }) else {
                continue;
            };
            let (_, _, hue_key, value) = &sums[index];
            let (start, end) = if *value >= 0.0 {
                let start = positive_offsets[category_index];
                positive_offsets[category_index] += value;
                if !positive_offsets[category_index].is_finite() {
                    return Err(Error::Data(format!(
                        "bar: positive stack geometry for category {:?} is not representable",
                        category_levels[category_index]
                    )));
                }
                (start, positive_offsets[category_index])
            } else {
                let start = negative_offsets[category_index];
                negative_offsets[category_index] += value;
                if !negative_offsets[category_index].is_finite() {
                    return Err(Error::Data(format!(
                        "bar: negative stack geometry for category {:?} is not representable",
                        category_levels[category_index]
                    )));
                }
                (start, negative_offsets[category_index])
            };
            bars.push(PreparedBar {
                category_index,
                series_index: Some(series_index),
                hue_key: hue_key.clone(),
                start,
                end,
                nonzero: *value != 0.0,
                interval: None,
                line_alpha: None,
                fill_alpha: None,
            });
        }
    }

    let value_range = value_range(&bars)?;
    Ok((
        PreparedBars {
            series: Vec::new(),
            value_range,
            bars,
            points: Vec::new(),
            significance: Vec::new(),
        },
        category_levels,
        series_levels,
        hue_levels,
    ))
}

fn prepare_hue_key<'a>(
    hues: Option<(&RawField, &[String])>,
    row: usize,
    levels: &mut Vec<String>,
    mut existing: impl Iterator<Item = &'a CategoryKey>,
) -> Result<Option<CategoryKey>, Error> {
    hues.map(|(field, hues)| {
        let index = level_index(levels, crate::data::source::copy_source_string(&hues[row])?)?;
        let value = levels[index].as_str();
        Ok(existing
            .find(|key| key.as_ref().value == value)
            .cloned()
            .unwrap_or_else(|| CategoryKey::new(field.clone(), value)))
    })
    .transpose()
}

fn level_index(levels: &mut Vec<String>, label: String) -> Result<usize, Error> {
    if let Some(index) = crate::data::source::source_position(levels, |existing| existing == &label)
    {
        Ok(index)
    } else {
        let index = levels.len();
        crate::data::source::reserve_source(levels, 1)?;
        levels.push(label);
        Ok(index)
    }
}

fn summarize(
    values: &[f64],
    units: Option<&[String]>,
    context: impl Fn() -> String,
    category: &str,
    series: Option<&str>,
    summary: BarSummaryInput<'_>,
) -> Result<(f64, Option<(f64, f64)>), Error> {
    let contextualize = |error| match error {
        Error::Data(reason) => Error::Data(format!("bar: {} {reason}", context())),
        error => error,
    };
    match &summary.input.source {
        BarSource::Samples { error_bar, .. } => {
            let mean = stable_mean(values).map_err(&contextualize)?;
            let interval = generated_interval(
                values,
                units,
                mean,
                error_bar.kind(),
                summary.facet_identity,
                category,
                series,
            )
            .map_err(contextualize)?;
            Ok((mean, interval))
        }
        BarSource::Estimates { .. } => match values {
            [value] if value.is_finite() => Ok((*value, None)),
            _ => Err(Error::Data(format!(
                "bar: {} precomputed estimates require exactly one finite value, got {}",
                context(),
                values.len()
            ))),
        },
    }
}

fn generated_interval(
    values: &[f64],
    units: Option<&[String]>,
    mean: f64,
    error_bar: &ErrorBarKind,
    facet_identity: &FacetIdentity,
    category: &str,
    series: Option<&str>,
) -> Result<Option<(f64, f64)>, Error> {
    if values.len() == 1 || matches!(error_bar, ErrorBarKind::Hidden) {
        return Ok(None);
    }
    match error_bar {
        ErrorBarKind::Hidden => Ok(None),
        ErrorBarKind::StandardDeviation(StandardDeviation { scale }) => {
            symmetric_interval(mean, sample_standard_deviation(values)? * scale).map_err(stat_error)
        }
        ErrorBarKind::PercentileInterval(PercentileInterval { level }) => {
            central_quantiles(values, *level).map(Some)
        }
        ErrorBarKind::StandardError(StandardError { scale }) => symmetric_interval(
            mean,
            sample_standard_deviation(values)? / (values.len() as f64).sqrt() * scale,
        )
        .map_err(stat_error),
        ErrorBarKind::ConfidenceInterval(interval) => match &interval.algorithm {
            ConfidenceIntervalAlgorithm::Bootstrap(bootstrap) => {
                if let Some(units) = units {
                    if units
                        .first()
                        .is_some_and(|first| units.iter().all(|unit| unit == first))
                    {
                        return Ok(None);
                    }
                }
                // Charge the actual fields hashed into the deterministic cell stream.

                let seed = stable_cell_seed(bootstrap.seed, facet_identity, category, series);
                percentile_bootstrap(values, units, interval.level, bootstrap.resamples, seed)
                    .map(Some)
            }
            ConfidenceIntervalAlgorithm::StudentT => {
                let critical = crate::stats::student_t_critical(values.len() - 1, interval.level)
                    .map_err(stat_error)?;
                symmetric_interval(
                    mean,
                    critical * sample_standard_deviation(values)? / (values.len() as f64).sqrt(),
                )
                .map_err(stat_error)
            }
        },
    }
}

fn symmetric_interval(mean: f64, half_width: f64) -> Result<Option<(f64, f64)>, &'static str> {
    let lower = mean - half_width;
    let upper = mean + half_width;
    if !half_width.is_finite() || half_width < 0.0 || !lower.is_finite() || !upper.is_finite() {
        return Err("error-bar geometry is not numerically representable");
    }
    Ok(Some((lower, upper)))
}

fn stat_error(reason: &str) -> Error {
    Error::Data(reason.to_owned())
}

fn sample_standard_deviation(values: &[f64]) -> Result<f64, Error> {
    if values.len() < 2 {
        return Err(stat_error(
            "sample standard deviation requires at least two observations",
        ));
    }

    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err(stat_error("contains non-finite observations"));
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut mean = 0.0;
    let mut squared_deviations = 0.0;

    for (index, value) in values.iter().enumerate() {
        let normalized = value / scale;
        let count = (index + 1) as f64;
        let delta = normalized - mean;
        mean += delta / count;
        squared_deviations += delta * (normalized - mean);
    }
    let deviation = (squared_deviations / (values.len() - 1) as f64).sqrt() * scale;
    if deviation.is_finite() {
        Ok(deviation)
    } else {
        Err(stat_error(
            "sample standard deviation is not numerically representable",
        ))
    }
}

fn central_quantiles(values: &[f64], level: f64) -> Result<(f64, f64), Error> {
    let tail = (1.0 - level) / 2.0;

    if values.is_empty()
        || !tail.is_finite()
        || !(0.0..=1.0).contains(&tail)
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(stat_error(
            "could not compute the lower error-bar percentile",
        ));
    }
    let mut sorted = Vec::new();
    crate::data::source::reserve_source(&mut sorted, values.len())?;
    sorted.extend_from_slice(values);
    sort_numeric_source(&mut sorted);
    let lower = crate::stats::distribution::sorted_quantile(&sorted, tail);
    let upper = crate::stats::distribution::sorted_quantile(&sorted, 1.0 - tail);
    if lower.is_finite() && upper.is_finite() && lower <= upper {
        Ok((lower, upper))
    } else {
        Err(stat_error(
            "percentile error-bar geometry is not numerically representable",
        ))
    }
}

fn percentile_bootstrap(
    values: &[f64],
    units: Option<&[String]>,
    level: f64,
    resamples: usize,
    seed: u64,
) -> Result<(f64, f64), Error> {
    if units.is_some_and(|units| units.len() != values.len()) {
        return Err(stat_error("sampling units must align with observations"));
    }
    let mut rng = SplitMix64::new(seed);
    let mut means = Vec::new();
    crate::data::source::reserve_source(&mut means, resamples)?;
    if let Some(units) = units {
        let grouped = grouped_unit_values(values, units)?;
        let mut sample = Vec::new();
        for _ in 0..resamples {
            sample.clear();
            for _ in 0..grouped.len() {
                let unit = &grouped[rng.index(grouped.len())?].1;
                crate::data::source::reserve_source(&mut sample, unit.len())?;
                for _ in 0..unit.len() {
                    sample.push(unit[rng.index(unit.len())?]);
                }
            }
            means.push(stable_mean(&sample)?);
        }
    } else {
        let mut sorted = Vec::new();
        crate::data::source::reserve_source(&mut sorted, values.len())?;
        sorted.extend_from_slice(values);
        sort_numeric_source(&mut sorted);
        let mut sample = Vec::new();
        crate::data::source::reserve_source(&mut sample, sorted.len())?;
        sample.resize(sorted.len(), 0.0);
        for _ in 0..resamples {
            for value in &mut sample {
                *value = sorted[rng.index(sorted.len())?];
            }
            means.push(stable_mean(&sample)?);
        }
    }
    central_quantiles(&means, level)
}

fn grouped_unit_values<'a>(
    values: &[f64],
    units: &'a [String],
) -> Result<Vec<(&'a str, Vec<f64>)>, Error> {
    let mut grouped: Vec<(&str, Vec<f64>)> = Vec::new();
    for (value, unit) in values.iter().copied().zip(units) {
        if let Some((_, observations)) = grouped.iter_mut().find(|(label, _)| *label == unit) {
            crate::data::source::reserve_source(observations, 1)?;
            observations.push(value);
        } else {
            crate::data::source::reserve_source(&mut grouped, 1)?;
            let mut observations = Vec::new();
            crate::data::source::reserve_source(&mut observations, 1)?;
            observations.push(value);
            grouped.push((unit, observations));
        }
    }
    sort_source_by(&mut grouped, |left, right| left.0.cmp(right.0));
    for (_, observations) in &mut grouped {
        sort_numeric_source(observations);
    }
    Ok(grouped)
}

fn stable_cell_seed(
    seed: u64,
    facets: &FacetIdentity,
    category: &str,
    series: Option<&str>,
) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    hash_bytes(&mut hash, &seed.to_le_bytes());
    hash_bytes(
        &mut hash,
        &u64::try_from(facets.0.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for (name, level) in &facets.0 {
        hash_field(&mut hash, name);
        hash_field(&mut hash, level);
    }
    hash_field(&mut hash, category);
    if let Some(series) = series {
        hash_bytes(&mut hash, &[1]);
        hash_field(&mut hash, series);
    } else {
        hash_bytes(&mut hash, &[0]);
    }
    splitmix64(hash)
}

fn hash_field(hash: &mut u64, field: &str) {
    let length = u64::try_from(field.len()).unwrap_or(u64::MAX);
    hash_bytes(hash, &length.to_le_bytes());
    hash_bytes(hash, field.as_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        #[cfg(test)]
        BOOTSTRAP_DRAWS.with(|draws| draws.set(draws.get() + 1));
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, upper: usize) -> Result<usize, Error> {
        let upper = u64::try_from(upper).unwrap_or(u64::MAX);
        let limit = u64::MAX - u64::MAX % upper;
        loop {
            let value = self.next();
            if value < limit {
                return Ok(usize::try_from(value % upper).unwrap_or(0));
            }
        }
    }
}

fn value_range(bars: &[PreparedBar]) -> Result<(f64, f64), Error> {
    let mut range = (0.0_f64, 0.0_f64);
    for bar in bars {
        let (low, high) = bar.interval.unwrap_or((bar.end, bar.end));
        if [bar.start, bar.end, low, high]
            .iter()
            .any(|value| !value.is_finite())
        {
            return Err(Error::Data(
                "bar: derived statistics or geometry are not finite".into(),
            ));
        }
        range = (
            range.0.min(bar.start).min(low),
            range.1.max(bar.start).max(high),
        );
    }
    Ok(range)
}

fn stable_mean(values: &[f64]) -> Result<f64, Error> {
    if values.is_empty() {
        return Err(stat_error("cannot compute a mean from no observations"));
    }

    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if !scale.is_finite() {
        return Err(stat_error("contains non-finite observations"));
    }
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    let mut correction = 0.0;

    for value in values {
        let normalized = value / scale;
        let next = sum + normalized;
        correction += if sum.abs() >= normalized.abs() {
            (sum - next) + normalized
        } else {
            (normalized - next) + sum
        };
        sum = next;
    }
    let mean = ((sum + correction) / values.len() as f64) * scale;
    if mean.is_finite() {
        Ok(mean)
    } else {
        Err(stat_error("mean is not numerically representable"))
    }
}

fn stable_sum(values: &[f64]) -> Result<f64, Error> {
    if values.is_empty() {
        return Err(stat_error("cannot compute a sum from no observations"));
    }

    if values.iter().any(|value| !value.is_finite()) {
        return Err(stat_error("contains non-finite observations"));
    }
    if let Some(sum) = compensated_sum(values.iter().copied()) {
        return Ok(sum);
    }

    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }
    let sum = compensated_sum(values.iter().map(|value| value / scale))
        .ok_or_else(|| stat_error("sum is not numerically representable"))?
        * scale;
    if sum.is_finite() {
        Ok(sum)
    } else {
        Err(stat_error("sum is not numerically representable"))
    }
}

fn compensated_sum(mut values: impl ExactSizeIterator<Item = f64>) -> Option<f64> {
    let mut sum: f64 = 0.0;
    let mut correction: f64 = 0.0;
    for _ in 0..values.len() {
        let value = values.next().expect("exact sample length");
        let next = sum + value;
        if !next.is_finite() {
            return None;
        }
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        if !correction.is_finite() {
            return None;
        }
        sum = next;
    }
    let result = sum + correction;
    result.is_finite().then_some(result)
}

#[cfg(test)]
mod alpha_tests;
