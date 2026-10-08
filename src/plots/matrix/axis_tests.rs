use super::*;
use crate::layout::fit::ROOT_SCOPE;
use crate::render::backend::*;

#[test]
fn cache_hits_and_fresh_selection_keep_the_same_tick_source() {
    let theme = crate::Theme::Publication.resolve().unwrap();

    let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.1, 300.1, 0.1, 300.1));
    let mut cached = None;
    let lookup = |cached: &mut Option<MatrixTickCache>| {
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

    assert_eq!(lookup(&mut cached).unwrap(), source);

    for slot in [&mut cached, &mut None] {
        assert_eq!(lookup(slot).unwrap(), source);
    }
    assert_eq!(cached.unwrap().plan.source, source);
}

#[test]
fn cache_miss_retains_the_selected_numeric_ticks() {
    {
        let theme = crate::Theme::Publication.resolve().unwrap();

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

        let expected = select().unwrap();

        let mut cached = None;
        let result = matrix_ticks(
            &mut cached,
            (0.0, 3.0),
            &projection,
            layout::AxisSide::Bottom,
            &theme,
            &ROOT_SCOPE,
        );
        {
            let actual = result.unwrap();
            assert_eq!(actual.source, expected.source);
            assert_eq!(actual.ticks, expected.ticks);
        }
    }
}

#[test]
fn matrix_selection_cache_cannot_cross_operation_or_selected_span() {
    let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 300.0, 0.0, 300.0));
    let first = crate::Theme::Publication.resolve().unwrap();
    let second = crate::Theme::Publication.resolve().unwrap();
    let mut cached = None;
    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &first,
        &ROOT_SCOPE,
    )
    .unwrap();
    let address = cached.as_ref().unwrap().plan.ticks.as_ptr();
    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &second,
        &ROOT_SCOPE,
    )
    .unwrap();
    assert_ne!(cached.as_ref().unwrap().plan.ticks.as_ptr(), address);
    assert!(cached
        .as_ref()
        .unwrap()
        .operation
        .same_operation(&second.prepared_font_environment().unwrap().operation()));
    let address = cached.as_ref().unwrap().plan.ticks.as_ptr();
    let smaller = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 200.0, 0.0, 300.0));
    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &smaller,
        layout::AxisSide::Bottom,
        &second,
        &ROOT_SCOPE,
    )
    .unwrap();
    assert_ne!(cached.as_ref().unwrap().plan.ticks.as_ptr(), address);
}

#[test]
fn cell_frame_identity_is_in_the_cached_axis_descriptor() {
    let mut theme = crate::Theme::Publication.resolve().unwrap();

    let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 300.0, 0.0, 300.0));
    let mut cached = None;
    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &theme,
        &ROOT_SCOPE,
    )
    .unwrap();
    let address = cached.as_ref().unwrap().plan.ticks.as_ptr();
    let ordinary = theme.axes.frame().cloned();
    theme.axes = theme.axes.clone().with_cell_frame(None).unwrap();
    assert_eq!(theme.axes.frame(), ordinary.as_ref());

    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &theme,
        &ROOT_SCOPE,
    )
    .unwrap();
    assert_ne!(cached.as_ref().unwrap().plan.ticks.as_ptr(), address);
    assert!(cached
        .as_ref()
        .unwrap()
        .standard
        .axes
        .cell_frame()
        .is_none());
}

#[test]
fn cached_matrix_selection_rechecks_axis_relationships() {
    let mut theme = crate::Theme::Publication.resolve().unwrap();

    let projection = layout::Projection::new((0.0, 3.0), (0.0, 3.0), (0.0, 300.0, 0.0, 300.0));
    let mut cached = None;
    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &theme,
        &ROOT_SCOPE,
    )
    .unwrap();
    let address = cached.as_ref().unwrap().plan.ticks.as_ptr();

    theme.axes = theme
        .axes
        .clone()
        .with_spacing(
            theme
                .axes
                .spacing()
                .clone()
                .with_y_title_edge_padding(crate::theme::RelativeSpacing::ratio(1, 1).unwrap())
                .unwrap(),
        )
        .unwrap();

    matrix_ticks(
        &mut cached,
        (0.0, 3.0),
        &projection,
        layout::AxisSide::Bottom,
        &theme,
        &ROOT_SCOPE,
    )
    .unwrap();
    assert_ne!(cached.as_ref().unwrap().plan.ticks.as_ptr(), address);
}

#[test]
fn matrix_cells_freeze_selected_frame_sides_once_and_keep_scientific_marks_and_axis_text() {
    let plot = Plot::matrix(vec![
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [4.0, 3.0, 2.0, 1.0]),
    ])
    .erase();
    let plot = crate::plots::shared_x::prepare_plot(
        &plot,
        &crate::font::canonical::OperationContext::new(),
        None,
    )
    .unwrap()
    .attach(plot)
    .unwrap();
    for (sides, expected) in [
        (
            Some(crate::theme::FrameSides::new(true, true, true, true).unwrap()),
            4,
        ),
        (
            Some(crate::theme::FrameSides::new(false, false, false, true).unwrap()),
            1,
        ),
        (None, 0),
    ] {
        let mut theme = crate::Theme::Publication.resolve().unwrap();
        let stroke = crate::theme::StrokeTheme::new(
            crate::theme::Color::rgb(1, 2, 3),
            theme.axes.cell_frame().unwrap().stroke().width(),
            StrokePattern::Solid,
        )
        .unwrap();
        theme.axes = theme
            .axes
            .with_cell_frame(
                sides.map(|sides| crate::theme::FrameTheme::new(sides, stroke).unwrap()),
            )
            .unwrap()
            .with_frame(None)
            .unwrap();
        let selected = measure(
            &plot,
            PlotRenderContext::new(
                plot.data(),
                PanelBox::new(100.0, 700.0, 100.0, 700.0),
                &theme,
            ),
        )
        .unwrap();
        let crate::layout::interior::MarkPayload::Matrix(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("Matrix payload");
        };
        assert_eq!(payload.cells.len(), 4);
        let mut points = 0;
        for cell in &payload.cells {
            assert_eq!(cell.frame.iter().flatten().count(), expected);
            for mark in &cell.marks {
                if let MatrixPrimitive::Scatter(values, _, _) = mark {
                    points += values.len();
                }
            }
        }
        assert_eq!(points, 8);
        let mut backend = crate::render::svg_backend::SvgBackend::in_memory_with_fonts(
            800,
            800,
            theme.prepared_font_environment().unwrap().clone(),
        );
        backend.begin(800, 800).unwrap();
        payload.draw(&mut backend).unwrap();
        backend.end().unwrap();
        let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
        assert_eq!(svg.matches("stroke=\"#010203\"").count(), 4 * expected);
        assert!(svg.contains("<text ") && svg.contains("<circle "));
        assert_eq!(payload.frame_style.is_some(), expected != 0);
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
                &crate::font::canonical::OperationContext::new(),
                None,
            )
            .unwrap()
            .attach(plot)
            .unwrap();
            let theme = crate::Theme::Publication.resolve().unwrap();
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
                        cell.frame[0].unwrap()[0].x,
                        cell.frame[0].unwrap()[1].x,
                        cell.frame[1].unwrap()[0].y,
                        cell.frame[1].unwrap()[1].y,
                    ),
                );
                let mut actual = AxisRecorder::default();
                cell.axes.draw(&mut actual).unwrap();
                cell.axes.draw_grid(&mut actual).unwrap();
                let x_ticks = &axes.x_ticks[col].as_ref().unwrap().plan.ticks;
                let y_ticks = &axes.y_ticks[row].as_ref().unwrap().plan.ticks;
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
                reference.x_axis.source = axes.x_ticks[col].as_ref().unwrap().plan.source;
                reference.y_axis.source = axes.y_ticks[row].as_ref().unwrap().plan.source;
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
        let mut theme = crate::Theme::Publication.resolve().unwrap();
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
