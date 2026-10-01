use crate::Error;

use super::sources::SourceKind;
use super::{cells, Axis, AxisSide, AxisSpec, AxisTick, Layout, Projection, TickSource};
use crate::layout::canvas::PanelRequirements;
use crate::layout::fit::{self, FitKind, FitScope, PlanError};
use crate::layout::{PhysicalTextAlignment, ResolvedTextPlan};
use crate::render::backend::{Backend, LineStyle, PlannedTextPlacement, Point2D, TextAnchor};
use crate::theme::ResolvedTheme;

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
        if let Some(interval) = self.interval {
            interval.validate()?;
        }
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
    sides: [[Point2D; 2]; 4],
    style: LineStyle,
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
    ticks: Vec<FrozenTick>,
    title: Option<(ResolvedTextPlan, PlannedTextPlacement)>,
    tick_style: Option<LineStyle>,
    gutter: f64,
    start_overhang: f64,
    end_overhang: f64,
}

impl PanelAxisPlan {
    pub(crate) fn geometry_snapshot(
        &self,
        theme: &ResolvedTheme,
    ) -> Result<crate::render::geometry::GeometrySnapshot, Error> {
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(theme)?;
        if let Some(frame) = &self.frame {
            for side in &frame.sides {
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
            for side in &frame.sides {
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
        }
        Ok(())
    }

    pub(crate) fn observe_feedback(
        &self,
        round: &mut crate::layout::kernel::FeedbackRound,
    ) -> Result<(), Error> {
        self.standard.observe_feedback(round)?;
        for input in self.semantics {
            input.observe_feedback(round)?;
        }
        for range in [self.projection.x_range(), self.projection.y_range()] {
            round.observe_number(range.0)?;
            round.observe_number(range.1)?;
        }
        for axis in [&self.x_axis, &self.y_axis] {
            round.observe_count(axis.source.ordinal)?;
            round.observe_count(axis.source.cap)?;
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
        theme
            .prepared_font_environment()?
            .claim_unicode_bytes(
                u64::try_from(std::mem::size_of::<PanelAxisPlan>())
                    .map_err(|_| Error::Render("axis plan storage overflow".into()))?,
            )
            .map_err(|error| Error::Render(format!("axis plan admission: {error}")))?;
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
        let frame = theme.axes.frame.as_ref().map(|frame| FramePlan {
            sides: [
                [Point2D::new(x0, y0), Point2D::new(x1, y0)],
                [Point2D::new(x1, y0), Point2D::new(x1, y1)],
                [Point2D::new(x1, y1), Point2D::new(x0, y1)],
                [Point2D::new(x0, y1), Point2D::new(x0, y0)],
            ],
            style: frame.stroke.backend_style(),
        });
        let half_stroke = frame
            .as_ref()
            .map_or(0.0, |frame| f64::from(frame.style.width) / 2.0);
        let grid = if let Some(grid) = &theme.axes.grid {
            let mut lines = Vec::new();
            let count = x_axis
                .ticks
                .len()
                .checked_add(y_axis.ticks.len())
                .ok_or_else(|| Error::Render("grid plan count overflow".into()))?;
            fit::reserve(theme, &mut lines, count, "grid geometry")?;
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
                style: grid.backend_style(),
            })
        } else {
            None
        };
        let mut requirements = PanelRequirements {
            left: y_axis.gutter.max(x_axis.start_overhang).max(half_stroke),
            right: x_axis.end_overhang.max(half_stroke),
            bottom: x_axis.gutter.max(y_axis.start_overhang).max(half_stroke),
            top: y_axis.end_overhang.max(half_stroke),
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
        const MAX_EXPLICIT_TICKS: usize = 10_000;
        if ticks.is_empty() {
            return Err(Error::Data(format!(
                "{axis} explicit numeric ticks must not be empty"
            )));
        }
        if ticks.len() > MAX_EXPLICIT_TICKS {
            return Err(Error::Data(format!(
                "{axis} explicit numeric tick count {} exceeds {MAX_EXPLICIT_TICKS}",
                ticks.len()
            )));
        }
        for (index, tick) in ticks.iter().enumerate() {
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
                    &theme.typography.tick_label,
                    PhysicalTextAlignment::End,
                    "y-axis exact tick label",
                    false,
                )?;
                width = f64::max(width, plan.width().max(plan.fit_width()));
            }
            Ok(width)
        } else {
            theme.intrinsic_tick_label_width(
                self.ticks.iter().map(|tick| tick.label.as_str()),
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
        theme
            .prepared_font_environment()?
            .claim_measurement_call()
            .map_err(|error| Error::Render(format!("axis call admission: {error:?}")))?;
        let categorical = self.source == TickSource::SUPPLIED;
        let cells = cells::assign(
            &self.ticks,
            categorical,
            self.assigned_cells.as_deref(),
            projection,
            side,
            theme,
            scope,
        )?;
        let panel = projection.panel_box();
        let vertical_width = if side == AxisSide::Left {
            self.vertical_label_width(panel, theme)?
        } else {
            0.0
        };
        let frame_half = theme.axes.frame.as_ref().map_or(0.0, |frame| {
            f64::from(frame.stroke.backend_style().width) / 2.0
        });
        let tick_appearance = theme.axes.ticks.as_ref().filter(|_| !self.ticks.is_empty());
        let tick_style = tick_appearance.map(|ticks| ticks.stroke.backend_style());
        let tick_length = tick_appearance.map_or(0.0, |ticks| f64::from(ticks.length));
        let outward = frame_half
            + tick_length
            + tick_style
                .as_ref()
                .map_or(0.0, |style| f64::from(style.width) / 2.0);
        let mut ticks = Vec::new();
        fit::reserve(theme, &mut ticks, self.ticks.len(), "frozen axis ticks")?;
        let mut label_extent = 0.0_f64;
        let (gap, title_gap, title_width, tick_role, title_role) = match side {
            AxisSide::Bottom => (
                theme.axes.spacing.x_tick_label_gap,
                theme.axes.spacing.x_title_tick_label_gap,
                panel.width(),
                "x-axis tick label",
                "x-axis title",
            ),
            AxisSide::Left => (
                theme.axes.spacing.y_tick_label_gap,
                theme.axes.spacing.y_title_tick_label_gap,
                panel.height(),
                "y-axis tick label",
                "y-axis title",
            ),
        };
        for (tick, cell) in self.ticks.iter().zip(cells) {
            let segment = match side {
                AxisSide::Bottom => [
                    Point2D::new(cell.center, panel.y0 - frame_half),
                    Point2D::new(cell.center, panel.y0 - frame_half - tick_length),
                ],
                AxisSide::Left => [
                    Point2D::new(panel.x0 - frame_half, cell.center),
                    Point2D::new(panel.x0 - frame_half - tick_length, cell.center),
                ],
            };
            let label = if tick.label.is_empty() {
                None
            } else {
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
                let perpendicular = match side {
                    AxisSide::Bottom => panel.y0 - outward - gap - plan.height() / 2.0,
                    AxisSide::Left => panel.x0 - outward - gap,
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
        if label_extent > 0.0 {
            gutter += gap + label_extent;
        }
        let title = if self.label.is_empty() {
            None
        } else {
            let plan = fit::measured(fit::text(
                theme,
                &self.label,
                &standard.axis_title,
                title_width,
                PhysicalTextAlignment::Center,
                title_role,
                scope,
            ))?;
            gutter += if label_extent > 0.0 {
                title_gap
            } else {
                match side {
                    AxisSide::Bottom => theme.axes.spacing.x_title_without_tick_labels_gap,
                    AxisSide::Left => title_gap,
                }
            };
            let mut placement = match side {
                AxisSide::Bottom => PlannedTextPlacement::middle(
                    Point2D::new(
                        f64::midpoint(panel.x0, panel.x1),
                        panel.y0 - gutter - plan.height() / 2.0,
                    ),
                    TextAnchor::Middle,
                    0.0,
                ),
                AxisSide::Left => PlannedTextPlacement::middle(
                    Point2D::new(
                        panel.x0 - gutter - plan.height() / 2.0,
                        f64::midpoint(panel.y0, panel.y1),
                    ),
                    TextAnchor::Middle,
                    -90.0,
                ),
            };
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
            gutter += thickness;
            Some((plan, placement))
        };
        if side == AxisSide::Left && title.is_some() {
            gutter += theme.axes.spacing.y_title_edge_padding;
        }
        let (start, end) = match side {
            AxisSide::Bottom => (panel.x0, panel.x1),
            AxisSide::Left => (panel.y0, panel.y1),
        };
        let half_tick = tick_style
            .as_ref()
            .map_or(0.0, |style| f64::from(style.width) / 2.0);
        let start_overhang = ticks
            .iter()
            .map(|tick| start - tick.cell.center + half_tick)
            .fold(0.0_f64, f64::max);
        let end_overhang = ticks
            .iter()
            .map(|tick| tick.cell.center + half_tick - end)
            .fold(0.0_f64, f64::max);
        Ok(FrozenAxisPlan {
            source: self.source,
            ticks,
            title,
            tick_style,
            gutter,
            start_overhang,
            end_overhang,
        })
    }
}
