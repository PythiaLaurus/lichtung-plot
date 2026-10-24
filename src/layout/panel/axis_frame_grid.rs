use crate::Error;

use super::sources::SourceKind;
use super::{cells, Axis, AxisSide, AxisSpec, AxisTick, Layout, Projection, TickSource};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::layout::{PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{Backend, LineStyle, PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::spacing::ExactLength;
use crate::theme::{AxisLengths, ResolvedTheme};

#[cfg(test)]
mod appearance_tests;
#[cfg(test)]
mod tests;

impl AxisSpec {
    /// Return a copy with finite, strictly ascending explicit limits.
    #[must_use]
    pub fn limits(mut self, min: f64, max: f64) -> Self {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::AxisSpecPreparation,
        );
        self.limits = Some((min, max));
        self.provenance = super::RangeProvenance::Configured;
        self
    }

    pub(crate) fn validate(&self, axis: &str) -> Result<(), Error> {
        if let Some((minimum, maximum)) = self.limits {
            if !minimum.is_finite() || !maximum.is_finite() || minimum >= maximum {
                return Err(Error::Data(format!(
                    "{axis} limits must be finite and strictly ascending; got {minimum}..{maximum}"
                )));
            }
        }
        if let Some(ticks) = &self.exact_ticks {
            Axis::validate_tick_source(ticks, self.limits, axis)?;
        }
        Ok(())
    }
}

impl AxisTick {
    /// Construct a tick with an unchanged display label.
    #[must_use]
    pub fn new(position: f64, label: impl Into<String>) -> Self {
        #[cfg(test)]
        crate::orchestrator::measurements::count(
            crate::orchestrator::measurements::Counter::AxisSourcePreparation,
        );
        Self {
            position,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PanelAxisPlan {
    projection: Projection,
    semantics: [super::AxisSemanticInputs; 2],
    standard: super::StandardPanelInputs,
    x_axis: FrozenAxisPlan,
    y_axis: FrozenAxisPlan,
    frame: Option<FramePlan>,
    grid: Option<GridPlan>,
    requirements: PanelRequirements,
}

#[derive(Debug, Clone)]
struct FramePlan {
    sides: [Option<[Point2D; 2]>; 4],
    style: LineStyle,
}

impl FramePlan {
    fn new(
        panel: crate::layout::PanelBox,
        frame: &crate::theme::FrameTheme,
    ) -> Result<Self, Error> {
        let selected = frame.get_sides().drawing_order();
        let segments = [
            [
                Point2D::new(panel.x0, panel.y0),
                Point2D::new(panel.x1, panel.y0),
            ],
            [
                Point2D::new(panel.x1, panel.y0),
                Point2D::new(panel.x1, panel.y1),
            ],
            [
                Point2D::new(panel.x1, panel.y1),
                Point2D::new(panel.x0, panel.y1),
            ],
            [
                Point2D::new(panel.x0, panel.y1),
                Point2D::new(panel.x0, panel.y0),
            ],
        ];
        Ok(Self {
            sides: std::array::from_fn(|index| selected[index].then_some(segments[index])),
            style: crate::theme::axis::stroke_style(frame.get_stroke())?,
        })
    }

    fn outward_edges(&self) -> [f64; 4] {
        let [bottom, right, top, left] = self.sides.each_ref().map(Option::is_some);
        [
            left || bottom || top,
            right || bottom || top,
            bottom || left || right,
            top || left || right,
        ]
        .map(|present| {
            if present {
                f64::from(self.style.width) / 2.0
            } else {
                0.0
            }
        })
    }
}

#[derive(Debug, Clone)]
struct GridPlan {
    lines: Vec<[Point2D; 2]>,
    style: LineStyle,
}

#[derive(Debug, Clone)]
struct FrozenTick {
    position: f64,
    cell: cells::LabelCell,
    segment: [Point2D; 2],
    label: Option<(ResolvedTextPlan, PlannedTextPlacement)>,
}

#[derive(Debug, Clone)]
struct FrozenAxisPlan {
    source: TickSource,
    eligible_labels: Vec<usize>,
    ticks: Vec<FrozenTick>,
    title: Option<(ResolvedTextPlan, PlannedTextPlacement)>,
    explanation: Option<(ResolvedTextPlan, PlannedTextPlacement)>,
    numeric: Option<std::sync::Arc<crate::numeric::set::FormattedSet>>,
    tick_style: Option<LineStyle>,
    gutter: f64,
    start_overhang: f64,
    end_overhang: f64,
}

impl PanelAxisPlan {
    pub(crate) fn geometry_snapshot(
        &self,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new();
        if let Some(frame) = &self.frame {
            for side in frame.sides.iter().flatten() {
                geometry.line(side, &frame.style, None)?;
            }
        }
        for axis in [&self.x_axis, &self.y_axis] {
            for tick in &axis.ticks {
                if let Some(style) = &axis.tick_style {
                    geometry.line(&tick.segment, style, None)?;
                }
                if let Some((plan, placement)) = &tick.label {
                    geometry.text(*placement, plan, None)?;
                }
            }
            if let Some((plan, placement)) = &axis.title {
                geometry.text(*placement, plan, None)?;
            }
            if let Some((plan, placement)) = &axis.explanation {
                geometry.text(*placement, plan, None)?;
            }
        }
        Ok(geometry)
    }
    pub(crate) fn range(&self, side: AxisSide) -> (f64, f64) {
        match side {
            AxisSide::Bottom => self.projection.x_range(),
            AxisSide::Left => self.projection.y_range(),
        }
    }

    pub(crate) fn requirements(&self) -> PanelRequirements {
        self.requirements
    }

    pub(crate) fn draw_grid(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        if let Some(grid) = &self.grid {
            for line in &grid.lines {
                backend.draw_line(line, &grid.style)?;
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        if let Some(frame) = &self.frame {
            for side in frame.sides.iter().flatten() {
                backend.draw_line(side, &frame.style)?;
            }
        }
        for axis in [&self.x_axis, &self.y_axis] {
            for tick in &axis.ticks {
                if let Some(style) = &axis.tick_style {
                    backend.draw_line(&tick.segment, style)?;
                }
                if let Some((plan, placement)) = &tick.label {
                    backend.draw_planned_text(*placement, plan)?;
                }
            }
            if let Some((plan, placement)) = &axis.title {
                backend.draw_planned_text(*placement, plan)?;
            }
            if let Some((plan, placement)) = &axis.explanation {
                backend.draw_planned_text(*placement, plan)?;
            }
        }
        Ok(())
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        self.standard.observe_feedback(round)?;
        for input in &self.semantics {
            input.observe_feedback(round)?;
        }
        for range in [self.projection.x_range(), self.projection.y_range()] {
            round.observe_number(range.0)?;
            round.observe_number(range.1)?;
        }
        for axis in [&self.x_axis, &self.y_axis] {
            if let Some(numeric) = &axis.numeric {
                numeric.observe_feedback(round)?;
            }
            round.observe_count(axis.source.ordinal)?;
            round.observe_count(axis.eligible_labels.len())?;
            for &index in &axis.eligible_labels {
                round.observe_count(index)?;
            }
            round.observe_count(axis.source.cap)?;
            round.observe_count(
                axis.source
                    .precision
                    .map_or(0, |digits| usize::from(digits) + 1),
            )?;
            let (kind, parameter) = match axis.source.kind {
                SourceKind::Supplied => (0, 0.0),
                SourceKind::Exact => (1, 0.0),
                SourceKind::Adaptive(seed) => (2, seed as f64),
                SourceKind::Additive(step) => (3, step),
                SourceKind::LogIntegral => (4, 0.0),
                SourceKind::LogSubset(count) => (5, count as f64),
                SourceKind::LogInterval(step) => {
                    round.observe_count((step >> 32) as usize)?;
                    round.observe_count((step & 0xffff_ffff) as usize)?;
                    (6, 0.0)
                }
                SourceKind::Midpoint => (7, 0.0),
                SourceKind::ExactCount => (8, 0.0),
                SourceKind::Multiplicative => (9, 0.0),
            };
            round.observe_count(kind)?;
            round.observe_number(parameter)?;
            for tick in &axis.ticks {
                round.observe_number(tick.position)?;
                round.observe_number(tick.cell.center)?;
                if tick.label.is_some() {
                    round.observe_number(tick.cell.start)?;
                    round.observe_number(tick.cell.end)?;
                    round.observe_count(usize::from(tick.cell.inward))?;
                }
            }
            for (plan, _) in axis
                .ticks
                .iter()
                .filter_map(|tick| tick.label.as_ref())
                .chain(axis.title.iter())
                .chain(axis.explanation.iter())
            {
                round.observe_text(plan)?;
            }
        }
        Ok(())
    }
}

impl Layout {
    #[cfg(test)]
    pub(crate) fn plan_axes(&self, theme: &ResolvedTheme) -> Result<PanelAxisPlan, Error> {
        self.measure_axes(theme, &FitScope::root(), self.test_semantics())
            .map_err(PlanError::into_error)
    }

    #[cfg(test)]
    pub(crate) fn test_semantics(&self) -> [super::AxisSemanticInputs; 2] {
        [&self.x_axis, &self.y_axis].map(|axis| {
            super::AxisSemanticInputs::new(
                &AxisSpec::default(),
                crate::spec::AxisScale::Linear,
                if axis.source == TickSource::SUPPLIED {
                    super::RangePolicy::CategorySlots
                } else {
                    super::RangePolicy::ContinuousNice
                },
            )
        })
    }

    pub(crate) fn measure_axes(
        &self,
        theme: &ResolvedTheme,
        scope: &FitScope,
        semantics: [super::AxisSemanticInputs; 2],
    ) -> Result<PanelAxisPlan, PlanError> {
        self.measure_axes_with_inputs(theme, scope, semantics, || {
            super::StandardPanelInputs::new(theme)
        })
    }

    pub(crate) fn measure_axes_with_inputs(
        &self,
        theme: &ResolvedTheme,
        scope: &FitScope,
        semantics: [super::AxisSemanticInputs; 2],
        standard: impl FnOnce() -> Result<super::StandardPanelInputs, Error>,
    ) -> Result<PanelAxisPlan, PlanError> {
        let standard = standard()?;
        // Evaluate both siblings: a later fatal error takes precedence over a fit miss.
        let x_axis = self.x_axis.measure(
            &self.projection,
            AxisSide::Bottom,
            theme,
            &standard,
            &scope.child(crate::layout::AddressRole::AxisBottom, 0)?,
        );
        let y_axis = self.y_axis.measure(
            &self.projection,
            AxisSide::Left,
            theme,
            &standard,
            &scope.child(crate::layout::AddressRole::AxisLeft, 0)?,
        );
        let (x_axis, y_axis) = match (x_axis, y_axis) {
            (Err(PlanError::Fatal(error)), _) | (_, Err(PlanError::Fatal(error))) => {
                return Err(error.into())
            }
            (Err(error), _) | (_, Err(error)) => return Err(error),
            (Ok(x), Ok(y)) => (x, y),
        };
        let (x0, x1, y0, y1) = self.projection.pixel_box();
        let frame = standard
            .axes
            .get_frame()
            .map(|frame| FramePlan::new(self.projection.panel_box(), frame))
            .transpose()?;
        let grid = if let Some(grid) = standard.axes.get_grid() {
            let mut lines = Vec::new();
            let count = x_axis
                .ticks
                .len()
                .checked_add(y_axis.ticks.len())
                .ok_or_else(|| Error::Render("grid plan count overflow".into()))?;
            fit::reserve(&mut lines, count, "grid geometry")?;
            for tick in &x_axis.ticks {
                lines.push([
                    Point2D::new(tick.segment[0].x, y0),
                    Point2D::new(tick.segment[0].x, y1),
                ]);
            }
            for tick in &y_axis.ticks {
                lines.push([
                    Point2D::new(x0, tick.segment[0].y),
                    Point2D::new(x1, tick.segment[0].y),
                ]);
            }
            Some(GridPlan {
                lines,
                style: crate::theme::axis::stroke_style(grid)?,
            })
        } else {
            None
        };
        let edges = frame.as_ref().map_or([0.0; 4], FramePlan::outward_edges);
        let mut requirements = PanelRequirements {
            left: y_axis.gutter.max(x_axis.start_overhang).max(edges[0]),
            right: x_axis.end_overhang.max(edges[1]),
            bottom: x_axis.gutter.max(y_axis.start_overhang).max(edges[2]),
            top: y_axis.end_overhang.max(edges[3]),
        };
        for extent in [
            &mut requirements.left,
            &mut requirements.right,
            &mut requirements.bottom,
            &mut requirements.top,
        ] {
            *extent = (*extent * 1024.0).ceil() / 1024.0;
        }
        Ok(PanelAxisPlan {
            projection: self.projection.clone(),
            semantics,
            standard,
            x_axis,
            y_axis,
            frame,
            grid,
            requirements,
        })
    }
}

impl Axis {
    pub(crate) fn label_capacities(
        &self,
        projection: &Projection,
        side: AxisSide,
        scope: &FitScope,
    ) -> Result<Vec<f64>, PlanError> {
        let cells = cells::assign(
            &self.ticks,
            self.source == TickSource::SUPPLIED,
            self.tick_labels_visible,
            self.assigned_cells.as_deref(),
            &self.category_labels,
            projection,
            side,
            scope,
        )?;
        Ok(crate::data::source::collect_source(
            cells.into_iter().map(|cell| Ok(cell.capacity())),
        )?)
    }

    /// Keep the title block centered on its semantic axis while aligning its lines
    /// in text-local space, before the existing horizontal/vertical transform.
    #[expect(
        clippy::cast_precision_loss,
        reason = "canonical text quanta convert at placement"
    )]
    pub(crate) fn title_placement(
        plan: &ResolvedTextPlan,
        center: Point2D,
        side: AxisSide,
    ) -> Result<PlannedTextPlacement, Error> {
        let half = crate::render::backend::nearest_even_text_half(plan.width_quanta)
            .ok_or_else(|| Error::Render("axis title half extent overflow".into()))?
            as f64
            / 1024.0;
        let (anchor, offset) = match plan.style.alignment {
            PhysicalTextAlignment::Start => (TextAnchor::Start, -half),
            PhysicalTextAlignment::Center => (TextAnchor::Middle, 0.0),
            PhysicalTextAlignment::End => (TextAnchor::End, plan.width() - half),
        };
        Ok(match side {
            AxisSide::Bottom => {
                PlannedTextPlacement::middle(Point2D::new(center.x + offset, center.y), anchor, 0.0)
            }
            AxisSide::Left => PlannedTextPlacement::middle(
                Point2D::new(center.x, center.y + offset),
                anchor,
                -90.0,
            ),
        })
    }

    pub(crate) fn validate_ticks(
        ticks: &[AxisTick],
        range: (f64, f64),
        axis: &str,
    ) -> Result<(), Error> {
        Self::validate_tick_source(ticks, Some(range), axis)
    }

    fn validate_tick_source(
        ticks: &[AxisTick],
        range: Option<(f64, f64)>,
        axis: &str,
    ) -> Result<(), Error> {
        if ticks.is_empty() {
            return Err(Error::Data(format!(
                "{axis} explicit numeric ticks must not be empty"
            )));
        }
        for (index, tick) in ticks.iter().enumerate() {
            crate::text::validate_for(&tick.label, axis)?;
            if !tick.position.is_finite() {
                return Err(Error::Data(format!(
                    "{axis} explicit tick {index} position must be finite; got {}",
                    tick.position
                )));
            }
            if let Some(range) = range {
                if tick.position < range.0 || tick.position > range.1 {
                    return Err(Error::Data(format!(
                        "{axis} explicit tick {index} position {} is outside resolved range {}..{}",
                        tick.position, range.0, range.1
                    )));
                }
            }
            if index > 0 && ticks[index - 1].position >= tick.position {
                return Err(Error::Data(format!(
                    "{axis} explicit tick positions must be strictly ascending"
                )));
            }
        }
        Ok(())
    }

    fn vertical_label_width(
        &self,
        panel: crate::layout::PanelBox,
        theme: &ResolvedTheme,
    ) -> Result<f64, Error> {
        if let Some(width) = self.assigned_tick_width {
            return Ok(width);
        }
        if self.source == TickSource::EXACT {
            let mut width = 1.0 / 1024.0;
            for tick in self.ticks.iter().filter(|tick| !tick.label.is_empty()) {
                let plan = theme.measure_text(
                    &tick.label,
                    &theme.typography.axes.tick_label,
                    PhysicalTextAlignment::End,
                    "y-axis exact tick label",
                    false,
                )?;
                width = f64::max(width, plan.width().max(plan.fit_width()));
            }
            Ok(width)
        } else {
            theme.intrinsic_tick_label_width(
                self.category_labels
                    .indices(self.ticks.len())
                    .map(|index| self.ticks[index].label.as_str()),
                panel.height(),
            )
        }
    }

    #[allow(clippy::too_many_lines)]
    fn measure(
        &self,
        projection: &Projection,
        side: AxisSide,
        theme: &ResolvedTheme,
        standard: &super::StandardPanelInputs,
        scope: &FitScope,
    ) -> Result<FrozenAxisPlan, PlanError> {
        let categorical = self.source == TickSource::SUPPLIED;
        let cells = cells::assign(
            &self.ticks,
            categorical,
            self.tick_labels_visible,
            self.assigned_cells.as_deref(),
            &self.category_labels,
            projection,
            side,
            scope,
        )?;
        let panel = projection.panel_box();
        let vertical_width = if side == AxisSide::Left && self.tick_labels_visible {
            self.vertical_label_width(panel, theme)?
        } else {
            0.0
        };

        let title_appearance = self.label.as_ref().map_or_else(
            || (&standard.axis_title).into(),
            |text| text.appearance(&standard.axis_title),
        );
        let lengths = standard.axes.get_spacing().lengths(
            standard.tick_label.get_font_size(),
            title_appearance.theme.get_font_size(),
        )?;
        let frame_half = standard
            .axes
            .get_frame()
            .filter(|frame| match side {
                AxisSide::Bottom => frame.get_sides().get_bottom(),
                AxisSide::Left => frame.get_sides().get_left(),
            })
            .map_or(0.0, |frame| f64::from(frame.get_stroke().get_width()) / 2.0);
        let tick_appearance = standard.axes.get_ticks().filter(|_| !self.ticks.is_empty());
        let tick_style = tick_appearance
            .map(|ticks| crate::theme::axis::stroke_style(ticks.get_stroke()))
            .transpose()?;
        let tick_length = tick_appearance.map_or(0.0, |ticks| f64::from(ticks.get_length()));
        let frame_offset = ExactLength::pixels(frame_half)?;
        let tick_end = frame_offset.add_pixels(tick_length)?;
        let outward = tick_end.add_pixels(
            tick_style
                .as_ref()
                .map_or(0.0, |style| f64::from(style.width) / 2.0),
        )?;
        let anchor = match side {
            AxisSide::Bottom => panel.y0,
            AxisSide::Left => panel.x0,
        };
        let segment_start = AxisLengths::before(&frame_offset, anchor)?;
        let segment_end = AxisLengths::before(&tick_end, anchor)?;
        let mut ticks = Vec::new();
        fit::reserve(&mut ticks, self.ticks.len(), "frozen axis ticks")?;
        let mut label_extent = 0.0_f64;
        let mut labels_present = false;
        let (gap, title_gap, title_width, tick_role, title_role) = match side {
            AxisSide::Bottom => (
                &lengths.x_tick_label_gap,
                &lengths.x_title_tick_label_gap,
                panel.width(),
                "x-axis tick label",
                "x-axis title",
            ),
            AxisSide::Left => (
                &lengths.y_tick_label_gap,
                &lengths.y_title_tick_label_gap,
                panel.height(),
                "y-axis tick label",
                "y-axis title",
            ),
        };
        let eligible_labels = crate::data::source::collect_source(
            self.category_labels.indices(self.ticks.len()).map(Ok),
        )?;
        for (index, (tick, cell)) in self.ticks.iter().zip(cells).enumerate() {
            let segment = match side {
                AxisSide::Bottom => [
                    Point2D::new(cell.center, segment_start),
                    Point2D::new(cell.center, segment_end),
                ],
                AxisSide::Left => [
                    Point2D::new(segment_start, cell.center),
                    Point2D::new(segment_end, cell.center),
                ],
            };
            let label = if !self.tick_labels_visible
                || !self.category_labels.includes(index, self.ticks.len())
                || (!categorical && tick.label.is_empty())
            {
                None
            } else {
                labels_present = true;
                let (width, alignment) = match side {
                    AxisSide::Bottom => (cell.capacity(), PhysicalTextAlignment::Center),
                    AxisSide::Left => (vertical_width, PhysicalTextAlignment::End),
                };
                let plan = fit::measured(fit::text(
                    theme,
                    &tick.label,
                    &standard.tick_label,
                    width,
                    alignment,
                    tick_role,
                    scope,
                ))?;
                let offset = outward.add_length(gap);
                let perpendicular = match side {
                    AxisSide::Bottom => {
                        AxisLengths::before(&offset.add_pixels(plan.height() / 2.0)?, panel.y0)?
                    }
                    AxisSide::Left => AxisLengths::before(&offset, panel.x0)?,
                };
                let placement = cell.placement(&plan, side, perpendicular)?.ok_or_else(|| {
                    scope.failure(
                        FitKind::Tick,
                        format!(
                            "{tick_role} ink or logical extent exceeds its assigned cell clearance"
                        ),
                    )
                })?;
                // Gutter extents are translation invariant. Subtracting large
                // absolute anchors can otherwise alternate a feedback quantum.
                if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
                    &plan,
                    Point2D::new(0.0, 0.0),
                    placement.anchor,
                    placement.rotation,
                    placement.origin,
                    placement.reverse_lines,
                ) {
                    label_extent = label_extent.max(match side {
                        AxisSide::Bottom => plan.height() / 2.0 - ink.bottom,
                        AxisSide::Left => -ink.left,
                    });
                }
                label_extent = label_extent.max(match side {
                    AxisSide::Bottom => plan.height(),
                    AxisSide::Left => plan.width(),
                });
                Some((plan, placement))
            };
            ticks.push(FrozenTick {
                position: tick.position,
                cell,
                segment,
                label,
            });
        }
        let mut gutter = outward;
        if labels_present {
            gutter = gutter.add_length(gap).add_pixels(label_extent)?;
        }
        let title = if let Some(label) = &self.label {
            let plan = fit::measured(fit::text(
                theme,
                label.as_str(),
                title_appearance,
                title_width,
                label
                    .resolved_alignment(crate::TextAlignment::Center)
                    .into(),
                title_role,
                scope,
            ))?;
            let separation = if labels_present {
                title_gap
            } else {
                match side {
                    AxisSide::Bottom => &lengths.x_title_without_tick_labels_gap,
                    AxisSide::Left => title_gap,
                }
            };
            gutter = gutter.add_length(separation);
            let position = AxisLengths::before(&gutter.add_pixels(plan.height() / 2.0)?, anchor)?;
            let center = match side {
                AxisSide::Bottom => Point2D::new(f64::midpoint(panel.x0, panel.x1), position),
                AxisSide::Left => Point2D::new(position, f64::midpoint(panel.y0, panel.y1)),
            };
            let mut placement = Self::title_placement(&plan, center, side)?;
            let canonical = crate::render::coordinates::canonical_text(
                placement.position,
                placement.rotation,
                crate::render::coordinates::CoordinatePrecision::CssQuantum,
            )?;
            placement.position = canonical.position.value();
            let mut thickness = plan.height();
            if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
                &plan,
                Point2D::new(0.0, 0.0),
                placement.anchor,
                placement.rotation,
                placement.origin,
                placement.reverse_lines,
            ) {
                thickness = thickness.max(match side {
                    AxisSide::Bottom => plan.height() / 2.0 - ink.bottom,
                    AxisSide::Left => plan.height() / 2.0 - ink.left,
                });
            }
            gutter = gutter.add_pixels(thickness)?;
            Some((plan, placement))
        } else {
            None
        };
        let explanation = if let Some(text) = self
            .numeric
            .as_ref()
            .and_then(|set| set.explanation.as_ref())
        {
            let plan = fit::measured(fit::text(
                theme,
                text.as_str(),
                text.appearance(&standard.tick_label),
                title_width,
                text.resolved_alignment(crate::TextAlignment::Center).into(),
                "numeric axis explanation",
                scope,
            ))?;
            gutter = gutter.add_length(title_gap);
            let position = AxisLengths::before(&gutter.add_pixels(plan.height() / 2.0)?, anchor)?;
            let center = match side {
                AxisSide::Bottom => Point2D::new(f64::midpoint(panel.x0, panel.x1), position),
                AxisSide::Left => Point2D::new(position, f64::midpoint(panel.y0, panel.y1)),
            };
            let placement = Self::title_placement(&plan, center, side)?;
            let mut thickness = plan.height();
            if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
                &plan,
                Point2D::new(0.0, 0.0),
                placement.anchor,
                placement.rotation,
                placement.origin,
                placement.reverse_lines,
            ) {
                thickness = thickness.max(match side {
                    AxisSide::Bottom => plan.height() / 2.0 - ink.bottom,
                    AxisSide::Left => plan.height() / 2.0 - ink.left,
                });
            }
            gutter = gutter.add_pixels(thickness)?;
            Some((plan, placement))
        } else {
            None
        };
        if side == AxisSide::Left && (title.is_some() || explanation.is_some()) {
            gutter = gutter.add_length(&lengths.y_title_edge_padding);
        }
        let (start, end) = match side {
            AxisSide::Bottom => (panel.x0, panel.x1),
            AxisSide::Left => (panel.y0, panel.y1),
        };
        let half_tick = tick_style
            .as_ref()
            .map_or(0.0, |style| f64::from(style.width) / 2.0);
        let mut start_overhang = ticks
            .iter()
            .map(|tick| start - tick.cell.center + half_tick)
            .fold(0.0_f64, f64::max);
        let mut end_overhang = ticks
            .iter()
            .map(|tick| tick.cell.center + half_tick - end)
            .fold(0.0_f64, f64::max);
        for (plan, placement) in title.iter().chain(explanation.iter()) {
            if let Some(ink) = crate::render::geometry::planned_text_ink_bounds(
                plan,
                placement.position,
                placement.anchor,
                placement.rotation,
                placement.origin,
                placement.reverse_lines,
            ) {
                let (low, high) = match side {
                    AxisSide::Bottom => (ink.left, ink.right),
                    AxisSide::Left => (ink.bottom, ink.top),
                };
                start_overhang = start_overhang.max(start - low);
                end_overhang = end_overhang.max(high - end);
            }
        }
        Ok(FrozenAxisPlan {
            source: self.source,
            eligible_labels,
            ticks,
            title,
            explanation,
            numeric: self.numeric.clone(),
            tick_style,
            gutter: AxisLengths::hard(&gutter)?,
            start_overhang,
            end_overhang,
        })
    }
}
