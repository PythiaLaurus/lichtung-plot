use super::*;
use crate::layout::fit::ROOT_SCOPE;
use crate::render::backend::*;

#[test]
fn cache_hit_admits_the_last_call_even_with_no_cache_entries_left() {
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.1, 300.1, 0.1, 300.1));
    let mut cached = None;
    let lookup = |cached: &mut Option<layout::NumericTickPlan>| {
        matrix_ticks(
            cached,
            (0.0, 3.0),
            &projection,
            layout::AxisSide::Bottom,
            &theme,
            &ROOT_SCOPE,
        )
        .map(|plan| plan.source)
    };
    let source = lookup(&mut cached).unwrap();
    let (calls, entries) = resources.call_cache_usage();
    resources.claim_calls(1_000_000 - calls - 1).unwrap();
    resources.claim_cache_entries(1_000_000 - entries).unwrap();
    let before = resources.usage().unwrap();
    let bytes = resources.byte_usage().unwrap();
    assert_eq!(lookup(&mut cached).unwrap(), source);
    assert_eq!(resources.call_cache_usage(), (1_000_000, 1_000_000));
    assert_eq!(resources.usage().unwrap(), (before.0, before.1 + 1));
    assert_eq!(resources.byte_usage().unwrap(), bytes);
    let before = resources.usage().unwrap();
    for slot in [&mut cached, &mut None] {
        let error = lookup(slot).unwrap_err();
        assert!(matches!(error, layout::fit::PlanError::Fatal(_)));
        assert!(error
            .into_error()
            .to_string()
            .contains("matrix tick call admission"));
        assert_eq!(resources.usage().unwrap(), before);
        assert_eq!(resources.byte_usage().unwrap(), bytes);
    }
    assert_eq!(cached.unwrap().source, source);
}

#[test]
fn cache_miss_reuses_its_call_ticket_and_admits_insertion_independently() {
    for available_entries in [0, 1] {
        let theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let resources = theme
            .prepared_font_environment()
            .unwrap()
            .generation_resources();
        let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.1, 300.1, 0.1, 300.1));
        let select = || {
            layout::numeric_tick_plan(
                (0.0, 3.0),
                None,
                false,
                &projection,
                layout::AxisSide::Bottom,
                &theme,
                false,
                &ROOT_SCOPE,
            )
        };
        select().unwrap();
        let calls = resources.call_cache_usage().0;
        let expected = select().unwrap();
        let (used_calls, used_entries) = resources.call_cache_usage();
        let source_calls = used_calls - calls;
        resources
            .claim_calls(1_000_000 - used_calls - source_calls)
            .unwrap();
        resources
            .claim_cache_entries(1_000_000 - used_entries - available_entries)
            .unwrap();
        let mut cached = None;
        let result = matrix_ticks(
            &mut cached,
            (0.0, 3.0),
            &projection,
            layout::AxisSide::Bottom,
            &theme,
            &ROOT_SCOPE,
        );
        if available_entries == 0 {
            let error = result.unwrap_err();
            assert!(matches!(error, layout::fit::PlanError::Fatal(_)));
            assert!(error
                .into_error()
                .to_string()
                .contains("matrix tick cache admission"));
            assert!(cached.is_none());
        } else {
            let actual = result.unwrap();
            assert_eq!(actual.source, expected.source);
            assert_eq!(actual.ticks, expected.ticks);
        }
        assert_eq!(resources.call_cache_usage(), (1_000_000, 1_000_000));
    }
}

#[derive(Default)]
struct AxisRecorder {
    lines: Vec<[Point2D; 2]>,
    text: Vec<PlacedText>,
}

impl Backend for AxisRecorder {
    fn begin(&mut self, _: u32, _: u32) -> Result<(), Error> {
        Ok(())
    }
    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn begin_clip(&mut self, _: PanelBox) -> Result<(), Error> {
        unreachable!()
    }
    fn end_clip(&mut self) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_surface(
        &mut self,
        _: PanelBox,
        _: crate::theme::Color,
        _: SurfaceRole,
    ) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_scatter(&mut self, _: &[Point2D], _: &PointStyle) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_line(&mut self, points: &[Point2D], _: &LineStyle) -> Result<(), Error> {
        self.lines.push(points.try_into().unwrap());
        Ok(())
    }
    fn draw_polygon(&mut self, _: &[Point2D], _: &FillStyle) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_linear_gradient(
        &mut self,
        _: PanelBox,
        _: Point2D,
        _: Point2D,
        _: &[GradientStop],
    ) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_text(&mut self, _: Point2D, _: &str, _: &TextStyle) -> Result<(), Error> {
        panic!("axis replay must not rebuild text")
    }
    fn draw_planned_text(
        &mut self,
        placement: PlannedTextPlacement,
        plan: &layout::ResolvedTextPlan,
    ) -> Result<(), Error> {
        self.text.push(PlacedText {
            placement,
            plan: plan.clone(),
        });
        Ok(())
    }
    fn draw_rounded_rect(
        &mut self,
        _: f64,
        _: f64,
        _: f64,
        _: f64,
        _: f64,
        _: &FillStyle,
    ) -> Result<(), Error> {
        unreachable!()
    }
    fn draw_bracket(&mut self, _: &SignificanceBracket, _: &BracketStyle) -> Result<(), Error> {
        unreachable!()
    }
}

#[test]
fn matrix_marks_ticks_and_reference_grid_share_large_fractional_cell_coordinates() {
    for origin in [0.1, 1_000_000_000.1] {
        for magnitude in [1.0, 1e150] {
            let plot = Plot::matrix(vec![
                crate::NumericValues::new("x", [-magnitude, 0.0, magnitude]),
                crate::NumericValues::new("y", [-magnitude, 0.0, magnitude]),
            ])
            .erase();
            let plot = crate::plots::shared_x::prepare_plot(
                &plot,
                &crate::font::canonical::OperationGenerationResources::new(),
                None,
            )
            .unwrap()
            .attach(plot)
            .unwrap();
            let theme = crate::Theme::Publication
                .resolve(&crate::font::canonical::OperationGenerationResources::new())
                .unwrap();
            let panel = PanelBox::new(
                origin,
                origin + 608.0 + 6.0 / 1024.0,
                origin,
                origin + 608.0 + 10.0 / 1024.0,
            );
            let selected =
                measure(&plot, PlotRenderContext::new(plot.data(), panel, &theme)).unwrap();
            let crate::layout::interior::MarkPayload::Matrix(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("matrix payload")
            };
            let axes = selected.matrix_axes.as_ref().unwrap();
            for (index, cell) in payload.cells.iter().enumerate() {
                let (row, col) = (index / 2, index % 2);
                let projection = layout::Projection::new(
                    axes.ranges[col],
                    axes.ranges[row],
                    (
                        cell.frame[0][0].x,
                        cell.frame[0][1].x,
                        cell.frame[1][0].y,
                        cell.frame[1][1].y,
                    ),
                );
                let mut actual = AxisRecorder::default();
                cell.axes.draw(&mut actual).unwrap();
                cell.axes.draw_grid(&mut actual).unwrap();
                let x_ticks = &axes.x_ticks[col].as_ref().unwrap().ticks;
                let y_ticks = &axes.y_ticks[row].as_ref().unwrap().ticks;
                assert_eq!(actual.lines.len(), x_ticks.len() + y_ticks.len());
                assert_eq!(
                    actual.text.len(),
                    usize::from(row == 1) * (x_ticks.len() + 1)
                        + usize::from(col == 0) * (y_ticks.len() + 1)
                );
                assert_eq!(
                    actual
                        .text
                        .iter()
                        .filter(|text| text.plan.style.provenance.owner == "x-axis title")
                        .count(),
                    usize::from(row == 1)
                );
                assert_eq!(
                    actual
                        .text
                        .iter()
                        .filter(|text| text.plan.style.provenance.owner == "y-axis title")
                        .count(),
                    usize::from(col == 0)
                );
                let mut reference = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
                reference.projection = projection.clone();
                reference.x_axis.ticks = x_ticks.clone();
                reference.y_axis.ticks = y_ticks.clone();
                reference.x_axis.source = axes.x_ticks[col].as_ref().unwrap().source;
                reference.y_axis.source = axes.y_ticks[row].as_ref().unwrap().source;
                let reference = reference.plan_axes(&theme).unwrap();
                let mut grid = AxisRecorder::default();
                reference.draw_grid(&mut grid).unwrap();
                assert_eq!(grid.lines.len(), actual.lines.len());
                for (tick, (line, grid)) in x_ticks
                    .iter()
                    .chain(y_ticks)
                    .zip(actual.lines.iter().zip(&grid.lines))
                {
                    let (px, py) = projection.project(tick.position, tick.position);
                    if line[0].x.to_bits() == line[1].x.to_bits() {
                        assert_eq!(line[0].x.to_bits(), px.to_bits());
                        assert_eq!(grid[0].x.to_bits(), px.to_bits());
                    } else {
                        assert_eq!(line[0].y.to_bits(), py.to_bits());
                        assert_eq!(grid[0].y.to_bits(), py.to_bits());
                    }
                }
                if row != col {
                    let MatrixPrimitive::Scatter(points, _, _) = &cell.marks[0] else {
                        panic!("scatter")
                    };
                    let expected = projection.project(0.0, 0.0);
                    assert_eq!((points[1].x, points[1].y), expected);
                    let x = x_ticks
                        .iter()
                        .position(|tick| tick.position == 0.0)
                        .unwrap();
                    let y = y_ticks
                        .iter()
                        .position(|tick| tick.position == 0.0)
                        .unwrap();
                    assert_eq!(actual.lines[x][0].x.to_bits(), points[1].x.to_bits());
                    assert_eq!(
                        actual.lines[x_ticks.len() + y][0].y.to_bits(),
                        points[1].y.to_bits()
                    );
                } else {
                    let MatrixPrimitive::Polygon(points, _) = &cell.marks[0] else {
                        panic!("density")
                    };
                    for (point, x) in points[1..=KDE_POINTS]
                        .iter()
                        .zip(&plot.prepared_matrix_axes.as_ref().unwrap().diagonals[row].positions)
                    {
                        assert_eq!(point.x.to_bits(), projection.project(*x, 0.0).0.to_bits());
                    }
                }
            }
        }
    }
}

#[test]
fn selected_source_cells_keep_logical_and_ink_clearance_at_odd_quanta() {
    for side in [layout::AxisSide::Bottom, layout::AxisSide::Left] {
        let mut theme = crate::Theme::Publication
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        theme.axes.frame = None;
        theme.axes.grid = None;
        for span in [81_923, 204_803, 307_205] {
            let origin = 0.1;
            let end = origin + f64::from(span) / 1024.0;
            let projection =
                layout::Projection::new((0.0, 3.0), (0.0, 3.0), (origin, end, origin, end));
            let mut cached = None;
            let selected = matrix_ticks(
                &mut cached,
                (0.0, 3.0),
                &projection,
                side,
                &theme,
                &ROOT_SCOPE,
            )
            .unwrap();
            let mut layout = Layout::envelope(PanelBox::new(80.0, 770.0, 60.0, 570.0));
            layout.projection = projection.clone();
            let axis = match side {
                layout::AxisSide::Bottom => &mut layout.x_axis,
                layout::AxisSide::Left => &mut layout.y_axis,
            };
            axis.ticks = selected.ticks.clone();
            axis.source = selected.source;
            let axes = layout.plan_axes(&theme).unwrap();
            let mut actual = AxisRecorder::default();
            axes.draw(&mut actual).unwrap();
            assert_eq!(actual.text.len(), selected.ticks.len());
            let quanta = |value| layout::kernel::nearest_spatial_quanta(value).unwrap();
            let centers = selected
                .ticks
                .iter()
                .map(|tick| {
                    let (x, y) = projection.project(tick.position, tick.position);
                    quanta(match side {
                        layout::AxisSide::Bottom => x,
                        layout::AxisSide::Left => y,
                    })
                })
                .collect::<Vec<_>>();
            for (index, text) in actual.text.iter().enumerate() {
                let midpoint = |a: i64, b: i64| {
                    let sum = a + b;
                    let half = sum.div_euclid(2);
                    half + i64::from(sum.rem_euclid(2) != 0 && half % 2 != 0)
                };
                let start = if index == 0 {
                    quanta(origin)
                } else {
                    midpoint(centers[index - 1], centers[index])
                } + 2048;
                let end = if index + 1 == centers.len() {
                    quanta(end)
                } else {
                    midpoint(centers[index], centers[index + 1])
                } - 2048;
                let placement = text.placement;
                let ink = crate::render::geometry::planned_text_ink_bounds(
                    &text.plan,
                    placement.position,
                    placement.anchor,
                    placement.rotation,
                    placement.origin,
                    placement.reverse_lines,
                )
                .unwrap();
                let length = match side {
                    layout::AxisSide::Bottom => text.plan.width_quanta,
                    layout::AxisSide::Left => text.plan.height_quanta,
                };
                let half = nearest_even_text_half(length).unwrap();
                let (low, high, center) = match side {
                    layout::AxisSide::Bottom => (
                        quanta(ink.left),
                        quanta(ink.right),
                        quanta(placement.position.x),
                    ),
                    layout::AxisSide::Left => (
                        quanta(ink.bottom),
                        quanta(ink.top),
                        quanta(placement.position.y),
                    ),
                };
                let (logical_low, logical_high) = match side {
                    layout::AxisSide::Bottom => (center - half, center + length - half),
                    layout::AxisSide::Left => (center + half - length, center + half),
                };
                assert!(low.min(logical_low) >= start);
                assert!(high.max(logical_high) <= end);
                if index > 0 && index + 1 < centers.len() {
                    assert_eq!(center, centers[index]);
                }
            }
        }
    }
}
