//! Shared data preparation for vertical and horizontal bar plots.

#![allow(clippy::cast_precision_loss)]

use crate::data::{
    source::{sort_numeric_source, sort_source_by},
    PlotData,
};
use crate::Error;

use crate::decorations::legend::{LegendEntry, LegendIdentity};
use crate::render::backend::{FillStyle, PointStyle};
use crate::spec::{
    BarInput, BarSource, BarSpacing, ConfidenceIntervalAlgorithm, ErrorBarKind, PercentileInterval,
    StandardDeviation, StandardError,
};
use crate::theme::{AccentColors, CategoryColors, ResolvedTheme};
use crate::{CategoryOrder, Plot, ValueSort};

#[cfg(test)]
thread_local! {
    pub(crate) static ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
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

/// The same categorical occupancy grammar drives selected polygons, attached
/// points/significance anchors and pre-selection mark pitch in both directions.
pub(crate) fn category_center(
    category_index: usize,
    series_index: Option<usize>,
    prepared: &PreparedBars,
    mode: BarMode,
    horizontal: bool,
) -> f64 {
    let center = category_index as f64 + 0.5;
    if mode != BarMode::Grouped {
        return center;
    }
    let slot_width = if horizontal { 0.74 } else { 0.76 };
    let slot = slot_width / prepared.series.len().max(1) as f64;
    center - slot_width / 2.0 + slot * (series_index.unwrap_or(0) as f64 + 0.5)
}

pub(crate) fn category_bar_geometry(
    bar: &PreparedBar,
    prepared: &PreparedBars,
    mode: BarMode,
    spacing: BarSpacing,
    horizontal: bool,
) -> (f64, f64) {
    let center = category_center(
        bar.category_index,
        bar.series_index,
        prepared,
        mode,
        horizontal,
    );
    let thickness = match mode {
        BarMode::Single | BarMode::Stacked => {
            if horizontal {
                0.55
            } else {
                0.6
            }
        }
        BarMode::Grouped => {
            let slot_width = if horizontal { 0.74 } else { 0.76 };
            let slot = slot_width / prepared.series.len().max(1) as f64;
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
    theme: &ResolvedTheme,
) -> Result<(f64, f64), Error> {
    let prepared = retained(chart)?;
    if prepared.categories.is_empty() || prepared.bars.is_empty() {
        return Err(Error::Render(
            "bar intrinsic has no prepared category or segment".into(),
        ));
    }
    let horizontal = matches!(
        chart.kind(),
        crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
    );
    let mode = mode(chart);

    let body = theme
        .marks
        .fills()
        .bodies()
        .bar_body()
        .backend_style((0, 0, 0));
    let body_ink = crate::render::geometry::rectangular_fill_ink_bounds(&body)?;
    let outline = crate::render::geometry::rectangular_fill_ink_outset(&body)?;
    let interval = crate::render::geometry::line_ink_width(
        &theme
            .marks
            .strokes()
            .summaries()
            .uncertainty()
            .backend_style(),
    )?;
    let mut pitch = 1.0_f64 / 1024.0;
    let mut numeric = if horizontal { outline.0 } else { outline.1 };

    for bar in &prepared.bars {
        if bar.category_index >= prepared.categories.len()
            || bar
                .series_index
                .is_some_and(|index| index >= prepared.series.len())
        {
            return Err(Error::Render(
                "bar occupied slot exceeds prepared category/series".into(),
            ));
        }
        let (center, thickness) =
            category_bar_geometry(bar, prepared, mode, chart.bar_spacing_spec(), horizontal);
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
            if left_room <= 0.0 || right_room <= 0.0 {
                return Err(Error::Render(
                    "bar category body exceeds its declared slot".into(),
                ));
            }
            pitch = pitch.max((-low).max(0.0) / left_room);
            pitch = pitch.max(high.max(0.0) / right_room);
        }
        if bar.interval.is_some() && interval > 0.0 {
            let cap = thickness * 0.2;
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
        if point.category_index >= prepared.categories.len()
            || point
                .series_index
                .is_some_and(|index| index >= prepared.series.len())
        {
            return Err(Error::Render(
                "bar supplemental point exceeds prepared category/series".into(),
            ));
        }
        let center = category_center(
            point.category_index,
            point.series_index,
            prepared,
            mode,
            horizontal,
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
        u32::try_from(prepared.categories.len())
            .map_err(|_| Error::Render("bar category count exceeds coordinate range".into()))?,
    );
    let categorical = pitch * categories;
    if !categorical.is_finite() {
        return Err(Error::Render("bar category hard span overflowed".into()));
    }
    let numeric = if (if horizontal {
        chart.user_x_limits()
    } else {
        chart.user_y_limits()
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
    /// Index into [`PreparedBars::categories`].
    pub(crate) category_index: usize,
    /// Optional index into [`PreparedBars::series`].
    pub(crate) series_index: Option<usize>,
    /// Optional index into [`PreparedBars::hues`] for color only.
    pub(crate) hue_index: Option<usize>,
    /// Start value on the numeric axis.
    pub(crate) start: f64,
    /// End value on the numeric axis.
    pub(crate) end: f64,
    /// Whether the aggregate is nonzero before cumulative stack addition.
    pub(crate) nonzero: bool,
    /// Optional explicit lower and upper interval endpoints around `end`.
    pub(crate) interval: Option<(f64, f64)>,
}

/// One attached significance label resolved to a rendered bar or category cluster.
#[derive(Debug, Clone)]
pub(crate) struct PreparedBarSignificance {
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
    /// Primary category labels in observed order.
    pub(crate) categories: Vec<String>,
    /// Series labels in observed order. Empty for `Single` mode.
    pub(crate) series: Vec<String>,
    /// Color labels derived from positioned series in observed order.
    pub(crate) hues: Vec<String>,
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
    fn color_keys(&self) -> &[String] {
        if self.hues.is_empty() {
            &self.categories
        } else {
            &self.hues
        }
    }

    pub(crate) fn resolved_colors(
        &self,
        chart: &Plot,
        theme: &ResolvedTheme,
        data: PlotData<'_>,
    ) -> Result<(CategoryColors, AccentColors), Error> {
        let resources = data.source_resources()?;
        let keys = if self.hues.is_empty() {
            &self.categories[..]
        } else {
            chart.global_hue_levels().unwrap_or(&self.hues)
        };
        let colors = CategoryColors::resolve(
            if chart.colors_by_sign() { &[] } else { keys },
            theme.color_scales.qualitative(),
            if chart.colors_by_sign() {
                None
            } else {
                chart.categorical_colors_spec()
            },
            resources,
        )?;
        let accents = AccentColors::resolve(
            theme.marks.accents(),
            if chart.colors_by_sign() { 2 } else { 0 },
        )?;
        Ok((colors, accents))
    }

    pub(crate) fn bar_color(
        &self,
        chart: &Plot,
        bar: &PreparedBar,
        colors: &CategoryColors,
        accents: &AccentColors,
    ) -> Result<(u8, u8, u8), Error> {
        if chart.colors_by_sign() {
            accents.color(usize::from(bar.end < bar.start))
        } else if let Some(hue) = bar.hue_index {
            colors.color(&self.hues[hue])
        } else {
            colors.color(&self.categories[bar.category_index])
        }
    }

    pub(crate) fn primary_mark_count(&self) -> u128 {
        self.bars.len() as u128
    }
}

#[derive(Debug)]
pub(crate) struct PreparedBarInputs {
    pub(crate) panels: Vec<(Option<String>, PreparedBars)>,
}

#[derive(Debug)]
pub(crate) struct BarShape {
    pub(crate) polygon: [crate::render::backend::Point2D; 4],
    pub(crate) fill: FillStyle,
    pub(crate) interval: Option<[[crate::render::backend::Point2D; 2]; 3]>,
}

#[derive(Debug)]
pub(crate) struct BarPayload {
    pub(crate) clip: crate::layout::PanelBox,
    pub(crate) domain_expansion: Option<crate::layout::interior::DomainExpansion>,
    pub(crate) zero: Option<[crate::render::backend::Point2D; 2]>,
    pub(crate) zero_style: crate::render::backend::LineStyle,
    pub(crate) interval_style: crate::render::backend::LineStyle,
    pub(crate) bars: Vec<BarShape>,
    pub(crate) text: Vec<crate::render::backend::PlacedText>,
    pub(crate) points: Vec<(crate::render::backend::Point2D, PointStyle)>,
    #[cfg(test)]
    pub(crate) kind: crate::PlotKind,
}

/// One numeric-axis request from the selected complete bar, interval and
/// supplemental-point paints, merged with its existing significance domain.
pub(crate) fn automatic_numeric_ink_expansion(
    chart: &Plot,
    prepared: &PreparedBars,
    payload: &BarPayload,
    projection: &crate::layout::Projection,

    scope: &crate::layout::fit::FitScope,
) -> Result<Option<crate::layout::interior::DomainExpansion>, crate::layout::fit::PlanError> {
    let horizontal = matches!(
        chart.kind(),
        crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
    );
    let automatic = if horizontal {
        chart.user_x_limits().is_none()
    } else {
        chart.user_y_limits().is_none()
    };
    if !automatic {
        return Ok(payload.domain_expansion);
    }
    let mut ink = crate::render::geometry::GeometrySnapshot::new();

    for bar in &payload.bars {
        ink.polygon(&bar.polygon, &bar.fill, None)?;
        if let Some(interval) = &bar.interval {
            for segment in interval {
                ink.line(segment, &payload.interval_style, None)?;
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
    let count = u32::try_from(prepared.categories.len())
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
        #[cfg(test)]
        count(self.kind, 3);
        crate::render::backend::with_clip(backend, self.clip, |backend| {
            if let Some(line) = &self.zero {
                backend.draw_line(line, &self.zero_style)?;
            }
            for bar in &self.bars {
                backend.draw_polygon(&bar.polygon, &bar.fill)?;
                for line in bar.interval.iter().flatten() {
                    backend.draw_line(line, &self.interval_style)?;
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

        chart: &Plot,
        prepared: &PreparedBars,

        scope: &crate::layout::fit::FitScope,
    ) -> Result<(), crate::layout::fit::PlanError> {
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        let horizontal = matches!(
            chart.kind(),
            crate::PlotKind::HorizontalBar | crate::PlotKind::StackedHorizontalBar
        );
        let numeric_axis = usize::from(!horizontal);
        let automatic = if horizontal {
            chart.user_x_limits().is_none()
        } else {
            chart.user_y_limits().is_none()
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
                let category_range = (0.0, prepared.categories.len() as f64);
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
        context: crate::spec::PlotRenderContext<'_>,
        chart: &Plot,
        prepared: &PreparedBars,
    ) -> Result<crate::spec::PlotRenderResult, crate::layout::fit::PlanError> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        let clip = crate::render::backend::validate_clip_region(self.clip)?.panel();
        self.measure_attached_text_ink(
            &result.layout.projection,
            chart,
            prepared,
            context.fit_scope(),
        )?;
        if let Some(line) = &self.zero {
            geometry.line(line, &self.zero_style, Some(clip))?;
        }
        for bar in &self.bars {
            geometry.polygon(&bar.polygon, &bar.fill, Some(clip))?;
            for line in bar.interval.iter().flatten() {
                geometry.line(line, &self.interval_style, Some(clip))?;
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
                let label = &bracket.label.plan;
                let style = crate::render::backend::BracketStyle {
                    color: context
                        .theme
                        .annotations
                        .significance()
                        .bracket_stroke()
                        .color(),
                    width: context
                        .theme
                        .annotations
                        .significance()
                        .bracket_stroke()
                        .width(),
                    label_color: label.style.paint,
                    font_family: crate::data::source::copy_font_stack(&label.style.stack)?,
                    font_size: label.style.font_size,
                    line_height: label.line_height(),
                    weight: label.style.weight,
                    label_gap: chart.significance_mark_style_spec().gap_spec(),
                };
                geometry.bracket(crate::render::backend::PlannedSignificanceCall {
                    bracket: &bracket.geometry,
                    style: &style,
                    label: &bracket.label,
                })?;
            }
        }
        #[cfg(test)]
        count(self.kind, 2);
        result.marks = Some(crate::data::source::source_arc(
            crate::layout::interior::SelectedMarkPlan {
                payload: crate::layout::interior::MarkPayload::Bars(self),
                geometry,
            },
        )?);
        Ok(result)
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 4]> = const { std::cell::Cell::new([[0; 4]; 4]) };
}

#[cfg(test)]
pub(crate) fn count(kind: crate::PlotKind, stage: usize) {
    let family = match kind {
        crate::PlotKind::Bar => 0,
        crate::PlotKind::HorizontalBar => 1,
        crate::PlotKind::StackedBar => 2,
        crate::PlotKind::StackedHorizontalBar => 3,
        _ => unreachable!(),
    };
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn mode(chart: &Plot) -> BarMode {
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

pub(crate) fn retained(chart: &Plot) -> Result<&PreparedBars, Error> {
    chart
        .prepared_bars
        .as_ref()
        .and_then(|inputs| {
            inputs
                .panels
                .iter()
                .find(|(scope, _)| scope.as_deref() == chart.axis_scope())
        })
        .map(|(_, marks)| marks)
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
    let expanded =
        crate::layout::anchored_auto_range(limits, &crate::layout::AxisSpec::default(), 0.0)?;
    Ok(Some(crate::layout::interior::DomainExpansion {
        axis,
        limits: (range.0.min(expanded.0), range.1.max(expanded.1)),
    }))
}

pub(crate) fn attached_significance_text_plans(
    context: crate::spec::PlotRenderContext<'_>,
    marks: &[PreparedBarSignificance],
    appearance: &crate::theme::TextTheme,
    alignment: impl Fn(&PreparedBarSignificance) -> crate::layout::PhysicalTextAlignment,
    owner: &str,
    exact_override: bool,
) -> Result<Vec<crate::layout::ResolvedTextPlan>, Error> {
    #[cfg(test)]
    ATTACHED_SIGNIFICANCE_TEXT_PLANNING_CALLS.with(|calls| calls.set(calls.get() + 1));

    crate::data::source::collect_source(marks.iter().map(|mark| {
        context.theme.measure_text(
            &mark.label,
            appearance,
            alignment(mark),
            owner,
            exact_override,
        )
    }))
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
    hue_index: Option<usize>,
    values: Vec<f64>,
    units: Vec<String>,
}

struct GroupedBarCell {
    category_index: usize,
    series_index: usize,
    hue_index: Option<usize>,
    values: Vec<f64>,
    units: Vec<String>,
}

/// Build prepared bar data from category, value, and optional series channels.
pub(crate) fn prepare<'a>(
    chart: &Plot,
    data: impl Into<crate::data::PlotData<'a>>,
    mode: BarMode,
) -> Result<PreparedBars, Error> {
    let data = data.into();
    prepare_scoped(chart, data, mode, None, chart.significance_annotations())
}

pub(crate) fn prepare_scoped<'a>(
    chart: &Plot,
    data: PlotData<'_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
    annotations: impl IntoIterator<Item = &'a crate::significance::SignificanceAnnotation>,
) -> Result<PreparedBars, Error> {
    #[cfg(test)]
    count(chart.kind(), 0);
    let mut prepared = prepare_core(chart, data, mode, facet)?;
    apply_attached_significance(annotations, &mut prepared, mode)?;
    prepared.points = prepare_point_layers(chart, &prepared, mode)?;
    for point in &prepared.points {
        prepared.value_range.0 = prepared.value_range.0.min(point.value);
        prepared.value_range.1 = prepared.value_range.1.max(point.value);
    }
    if let Some(exact) = chart
        .categorical_colors_spec()
        .filter(|_| !chart.colors_by_sign())
    {
        for key in prepared.color_keys() {
            if exact.color_in(key).is_none() {
                return Err(Error::Data(format!(
                    "bar: category {key:?} has no caller-owned color"
                )));
            }
        }
    }
    Ok(prepared)
}

fn prepare_core(
    chart: &Plot,
    data: PlotData<'_>,
    mode: BarMode,
    facet: Option<crate::data::FacetAxisSources<'_>>,
) -> Result<PreparedBars, Error> {
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
            .map_or_else(|| chart.global_group_levels(), |sources| sources.groups)
            .unwrap_or(&channels.group_levels)
            .iter()
            .map(|s| crate::data::source::copy_source_string(s)),
    )?;
    let hue_order = crate::data::source::collect_source(
        facet
            .map_or_else(|| chart.global_hue_levels(), |sources| sources.hues)
            .unwrap_or(&channels.hue_levels)
            .iter()
            .map(|s| crate::data::source::copy_source_string(s)),
    )?;
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
    let mut prepared = match mode {
        BarMode::Single => prepare_single(
            categories,
            channels.hues.as_deref(),
            values,
            hue_order,
            summary()?,
        ),
        BarMode::Grouped => {
            let groups = channels
                .groups
                .ok_or_else(|| Error::Data("clustered bar requires a series column".into()))?;
            prepare_grouped(
                categories,
                groups,
                channels.hues.as_deref(),
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
            prepare_stacked(
                categories,
                groups,
                channels.hues.as_deref(),
                values,
                group_order,
                hue_order,
            )
        }
    }?;
    apply_category_order(
        &mut prepared,
        chart.category_order_spec(),
        mode,
        !data.is_full_plot(),
    )?;
    if let Some(bounds) = chart.bar_bound_cols() {
        apply_precomputed_bounds(chart, data, &mut prepared, bounds)?;
    }
    Ok(prepared)
}

fn facet_identity(chart: &Plot, data: crate::data::PlotData<'_>) -> Result<FacetIdentity, Error> {
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
    mode: BarMode,
) -> Result<(), Error> {
    for annotation in annotations {
        let Some(target) = annotation.at_target() else {
            continue;
        };

        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == target.category_name())
            .ok_or_else(|| {
                Error::Data(format!(
                    "bar significance references unknown category {:?}; available categories: {}",
                    target.category_name(),
                    prepared.categories.join(", ")
                ))
            })?;
        let series_index = target
            .series_name()
            .map(|series| {
                prepared
                    .series
                    .iter()
                    .position(|candidate| candidate == series)
                    .ok_or_else(|| {
                        Error::Data(format!(
                            "bar significance references unknown series {series:?}; available series: {}",
                            prepared.series.join(", ")
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
    chart: &Plot,
    data: crate::data::PlotData<'_>,
    prepared: &mut PreparedBars,
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
        let category_index = prepared
            .categories
            .iter()
            .position(|category| category == &categories[row])
            .ok_or_else(|| {
                Error::Data("bar estimate bounds category disappeared during preparation".into())
            })?;
        let series_index = groups
            .as_ref()
            .map(|values| {
                prepared
                    .series
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
    chart: &Plot,
    prepared: &PreparedBars,
    mode: BarMode,
) -> Result<Vec<PreparedBarPoint>, Error> {
    if chart.bar_point_layers().is_empty() {
        return Ok(Vec::new());
    }
    if mode == BarMode::Stacked {
        return Err(Error::Data(
            "bar point layers are not supported by stacked bars".into(),
        ));
    }
    let has_series = chart.group_col().is_some();
    let mut points = Vec::new();
    for (layer_index, layer) in chart.bar_point_layers().iter().enumerate() {
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
        let categories = category_values(layer.category_values())?;
        let series = layer.series_values().map(category_values).transpose()?;
        let values = crate::data::source::collect_source(
            (0..layer.values().len())
                .map(|row| crate::data::strict_numeric_value(layer.values(), row, &context)),
        )?;
        if categories.is_empty() || categories.len() != values.len() {
            return Err(Error::Data(format!(
                "{context}: category and value columns must have equal non-zero lengths"
            )));
        }
        for row in 0..values.len() {
            let category_index = prepared
                .categories
                .iter()
                .position(|value| value == &categories[row])
                .ok_or_else(|| {
                    Error::Data(format!(
                        "{context}: unknown category {:?} at row {row}",
                        categories[row]
                    ))
                })?;
            let series_index = series
                .as_ref()
                .map(|series_values| {
                    prepared
                        .series
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
                style: layer.point_style()?,
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
pub(crate) fn validate_cells(chart: &Plot, data: crate::data::PlotData<'_>) -> Result<(), Error> {
    let x_col = chart
        .x_col()
        .ok_or_else(|| Error::Data("bar: x column not set".into()))?;
    let y_col = chart
        .y_col()
        .ok_or_else(|| Error::Data("bar: y column not set".into()))?;
    let categories = crate::data::strict_strings(data, x_col, "bar")?;
    let values = crate::data::strict_finite_f64(data, y_col, "bar")?;
    let category_name = data.column(x_col)?.name();
    let facet_columns = {
        let (row, column, wrap, _) = chart.facet_columns();
        [row, column, wrap]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };
    let facet_values = facet_columns
        .iter()
        .map(|column| crate::data::strict_strings(data, column, "bar"))
        .collect::<Result<Vec<_>, _>>()?;
    let facet_names = facet_columns
        .iter()
        .map(|column| data.column(column).map(crate::data::DataColumn::name))
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
    let channels = crate::data::collective_channels(
        data,
        chart.group_col(),
        chart.hue_col(),
        Some(&mark_cells),
        "bar",
    )?;
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
        crate::data::strict_strings(data, units_column, "bar sampling units")?;
    }
    if let Some((lower_column, upper_column)) = chart.bar_bound_cols() {
        let lower = crate::data::strict_finite_f64(data, lower_column, "bar estimate bounds")?;
        let upper = crate::data::strict_finite_f64(data, upper_column, "bar estimate bounds")?;
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
    order: &CategoryOrder,
    mode: BarMode,
    is_subset: bool,
) -> Result<(), Error> {
    match order {
        CategoryOrder::Observed => Ok(()),
        CategoryOrder::Explicit(requested) => {
            let ordered = crate::data::explicit_category_order(
                &prepared.categories,
                requested,
                "bar category order",
                is_subset,
            )?;
            let old_categories = std::mem::replace(&mut prepared.categories, ordered);
            for bar in &mut prepared.bars {

                let label = &old_categories[bar.category_index];
                bar.category_index = prepared
                    .categories
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
            sort_single_values(prepared, *sort)
        }
        CategoryOrder::Value(_) => Err(Error::Data(
            "bar: value-derived category order requires one ungrouped bar per category; use an explicit category order for grouped or stacked bars"
                .into(),
        )),
    }
}

fn sort_single_values(prepared: &mut PreparedBars, sort: ValueSort) -> Result<(), Error> {
    let mut order = crate::data::source::collect_source((0..prepared.categories.len()).map(Ok))?;
    sort_source_by(&mut order, |left, right| {
        let left_value = prepared.bars[*left].end;
        let right_value = prepared.bars[*right].end;
        (match sort {
            ValueSort::Ascending => left_value.total_cmp(&right_value),
            ValueSort::Descending => right_value.total_cmp(&left_value),
        })
        .then_with(|| left.cmp(right))
    });
    let categories = crate::data::source::collect_source(
        order
            .iter()
            .map(|index| crate::data::source::copy_source_string(&prepared.categories[*index])),
    )?;
    let bars = crate::data::source::collect_source(order.iter().enumerate().map(
        |(category_index, old_index)| {
            let mut bar = prepared.bars[*old_index].clone();
            bar.category_index = category_index;
            Ok(bar)
        },
    ))?;
    prepared.categories = categories;
    prepared.bars = bars;
    Ok(())
}

/// Legend entries for positioned series.
pub(crate) fn legend_entries(
    prepared: &PreparedBars,
    chart: &Plot,
    theme: &ResolvedTheme,

    colors: &CategoryColors,
    accents: &AccentColors,
) -> Result<Vec<LegendEntry>, Error> {
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
                    LegendEntry::fill(
                        LegendIdentity::bar_sign(decreasing),
                        theme
                            .marks
                            .fills()
                            .bodies()
                            .bar_body()
                            .backend_style(accents.color(usize::from(decreasing))?),
                    )
                    .without_hue_level(),
                );
            }
        }
        return Ok(entries);
    }

    crate::data::source::collect_source(prepared.hues.iter().map(|label| {
        Ok(LegendEntry::fill(
            chart.point_legend_identity(label)?,
            theme
                .marks
                .fills()
                .bodies()
                .bar_body()
                .backend_style(colors.color(label)?),
        ))
    }))
}

fn prepare_single(
    categories: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    mut hue_levels: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<PreparedBars, Error> {
    let mut category_levels = Vec::new();
    let mut groups: Vec<SingleBarCell> = Vec::new();
    for (row, (category, value)) in categories.into_iter().zip(values).enumerate() {
        let category_index = level_index(&mut category_levels, category)?;
        let hue_index = hues
            .map(|hues| {
                level_index(
                    &mut hue_levels,
                    crate::data::source::copy_source_string(&hues[row])?,
                )
            })
            .transpose()?;
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
                hue_index,
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
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
        });
    }

    let value_range = value_range(&bars)?;
    Ok(PreparedBars {
        categories: category_levels,
        series: Vec::new(),
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
}

fn prepare_grouped(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
    summary: BarSummaryInput<'_>,
) -> Result<PreparedBars, Error> {
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
        let hue_index = hues
            .as_ref()
            .map(|hues| {
                level_index(
                    &mut hue_levels,
                    crate::data::source::copy_source_string(&hues[row])?,
                )
            })
            .transpose()?;
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
                hue_index,
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
            hue_index: cell.hue_index,
            start: 0.0,
            end: value,
            nonzero: value != 0.0,
            interval,
        });
    }

    let value_range = value_range(&bars)?;

    Ok(PreparedBars {
        categories: category_levels,
        series: series_levels,
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "admitted grouping and ordered signed stack accumulation share one scientific owner"
)]
fn prepare_stacked(
    categories: Vec<String>,
    series: Vec<String>,
    hues: Option<&[String]>,
    values: Vec<f64>,
    series_order: Vec<String>,
    hue_order: Vec<String>,
) -> Result<PreparedBars, Error> {
    if categories.len() != series.len() || categories.len() != values.len() {
        return Err(Error::Data(
            "bar: category/value/series length mismatch".into(),
        ));
    }
    let mut category_levels = Vec::new();
    let mut series_levels = series_order;
    let mut hue_levels = hue_order;
    let mut groups: Vec<(usize, usize, Option<usize>, Vec<f64>)> = Vec::new();

    for (row, ((category, series_label), value)) in
        categories.into_iter().zip(series).zip(values).enumerate()
    {
        let category_index = level_index(&mut category_levels, category)?;
        let series_index = level_index(&mut series_levels, series_label)?;
        let hue_index = hues
            .as_ref()
            .map(|hues| {
                level_index(
                    &mut hue_levels,
                    crate::data::source::copy_source_string(&hues[row])?,
                )
            })
            .transpose()?;
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
                hue_index,
                crate::data::source::collect_source(std::iter::once(Ok(value)))?,
            ));
        }
    }
    if groups.is_empty() {
        return Err(Error::Data("bar: no groups".into()));
    }
    let sums = crate::data::source::collect_source(groups.into_iter().map(
        |(category_index, series_index, hue_index, values)| {
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
                |sum| Ok((category_index, series_index, hue_index, sum)),
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
            let (_, _, hue_index, value) = &sums[index];
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
                hue_index: *hue_index,
                start,
                end,
                nonzero: *value != 0.0,
                interval: None,
            });
        }
    }

    let value_range = value_range(&bars)?;
    Ok(PreparedBars {
        categories: category_levels,
        series: series_levels,
        hues: hue_levels,
        value_range,
        bars,
        points: Vec::new(),
        significance: Vec::new(),
    })
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
