use super::*;
use crate::{CategoryOrder, CategoryValues, NumericValues, ValueSort};

fn missing_key_style(guide: &crate::decorations::legend::LegendDrawPlan) -> &FillStyle {
    guide
        .calls
        .iter()
        .find_map(|call| match call {
            crate::decorations::legend::LegendCall::Fill(_, style) => Some(style),
            _ => None,
        })
        .unwrap()
}

fn gradient_stops(guide: &crate::decorations::legend::LegendDrawPlan) -> &[GradientStop] {
    guide
        .calls
        .iter()
        .find_map(|call| match call {
            crate::decorations::legend::LegendCall::Gradient(_, _, _, stops) => {
                Some(stops.as_slice())
            }
            _ => None,
        })
        .unwrap()
}

struct HeatmapValues {
    x: CategoryValues,
    y: CategoryValues,
    fill: NumericValues,
}

fn valid_data() -> HeatmapValues {
    HeatmapValues {
        x: CategoryValues::new("x", [" b ", "a", "b"]).unwrap(),
        y: CategoryValues::new("y", ["first", "first", " second  row "]).unwrap(),
        fill: NumericValues::new("fill", [1.0, 2.0, 3.0]),
    }
}

fn chart(data: &HeatmapValues) -> Plot<crate::kind::Heatmap> {
    Plot::heatmap(data.x.clone(), data.y.clone(), data.fill.clone())
}

fn prepare_chart(chart: &Plot<crate::kind::Heatmap>) -> Result<HeatmapData, Error> {
    let chart = chart.clone().erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    prepare(&chart, chart.data().with_axis_resources(&resources))
}

#[test]
fn custom_heatmap_outlines_and_cross_leave_selected_cells_and_missing_key_in_place() {
    let source = chart(&valid_data()).size(1200, 800).erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let panel = PanelBox::new(10.0, 850.0, 20.0, 620.0);
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let original = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    theme.marks.heatmap_cell_outline.paint = crate::theme::Color::rgba(1, 3, 5, 110);
    theme.marks.heatmap_cell_outline.width = 0.7;
    theme.marks.heatmap_missing_cross.paint = crate::theme::Color::rgba(7, 9, 11, 220);
    theme.marks.heatmap_missing_fill = crate::theme::Color::rgba(19, 23, 29, 144);
    theme.marks.heatmap_missing_key_outline.paint = crate::theme::Color::rgba(13, 15, 17, 90);
    theme.marks.heatmap_missing_key_outline.width = 2.5;
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    let extract = |result: &PlotRenderResult| {
        let crate::layout::interior::MarkPayload::Heatmap(payload) =
            &result.marks.as_ref().unwrap().payload
        else {
            panic!("selected Heatmap")
        };
        (
            payload
                .cells
                .iter()
                .map(|cell| cell.corners)
                .collect::<Vec<_>>(),
            payload
                .cells
                .iter()
                .find(|cell| cell.cross.is_none())
                .unwrap()
                .fill
                .clone(),
            payload.cross_style.clone(),
            payload
                .cells
                .iter()
                .find(|cell| cell.cross.is_some())
                .unwrap()
                .fill
                .color,
            missing_key_style(payload.colorbar.as_ref().unwrap()).clone(),
        )
    };
    let (old_cells, old_cell, old_cross, old_missing, old_key) = extract(&original);
    let (new_cells, new_cell, new_cross, new_missing, new_key) = extract(&selected);
    assert_eq!(old_cells, new_cells);
    assert_eq!(old_cell.color, new_cell.color);
    assert_eq!(new_cell.stroke.alpha(), 110);
    assert_eq!(new_cell.stroke_width.to_bits(), 0.7_f32.to_bits());
    assert_ne!(old_cross.color, new_cross.color);
    assert_ne!(old_missing, new_missing);
    assert_eq!(new_missing, theme.marks.heatmap_missing_fill);
    assert_eq!(new_key.color, new_missing);
    assert_ne!(new_key.color, old_key.color);
    assert_eq!(new_key.stroke.alpha(), 90);
    assert_eq!(new_key.stroke_width.to_bits(), 2.5_f32.to_bits());
    let prepared = &chart.prepared_heatmap.as_ref().unwrap().panels[0].1;
    let missing = prepared
        .guide_semantics(&theme, chart.data().with_axis_resources(&resources))
        .unwrap()
        .missing
        .unwrap();
    assert_eq!(missing.fill, new_missing);
    assert_eq!(missing.outline.paint, new_key.stroke);
    assert_eq!(
        missing.outline.width.to_bits(),
        new_key.stroke_width.to_bits()
    );
}

#[test]
fn missing_row_and_key_consume_one_plot_owned_fill_independent_of_the_gradient() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["known", "known", "absent", "absent"]).unwrap(),
        NumericValues::nullable("fill", [Some(0.0), None, None, None]),
    )
    .heatmap_null_row_annotation(
        crate::HeatmapNullRowAnnotation::new("absent", "Unavailable").unwrap(),
    )
    .size(1200, 800)
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.marks.heatmap_missing_fill = crate::theme::Color::rgba(30, 40, 50, 175);
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(50.0, 750.0, 40.0, 540.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("Heatmap missing mark payload")
    };
    let row = payload
        .cells
        .iter()
        .find(|cell| cell.text.is_some())
        .unwrap();
    let missing = payload
        .cells
        .iter()
        .find(|cell| cell.cross.is_some())
        .unwrap();
    let key = missing_key_style(payload.colorbar.as_ref().unwrap());
    assert_eq!(row.fill.color, theme.marks.heatmap_missing_fill);
    assert_eq!(missing.fill.color, row.fill.color);
    assert_eq!(key.color, row.fill.color);
    assert_ne!(
        payload
            .cells
            .iter()
            .find(|cell| cell.text.is_none() && cell.cross.is_none())
            .unwrap()
            .fill
            .color,
        row.fill.color
    );
}

#[test]
fn heatmap_intrinsic_cache_rechecks_contrast_role_metrics_after_theme_change() {
    let source = chart(&valid_data())
        .annotations(HeatmapAnnotations::Values(
            crate::NumericLabelFormat::default()
                .suffix(" long units")
                .unwrap(),
        ))
        .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let original = intrinsic_body_minimum(&chart, &theme).unwrap();
    assert_eq!(intrinsic_body_minimum(&chart, &theme).unwrap(), original);
    let mut custom = theme.clone();
    custom.typography.contrast_light.font_size = 40;
    custom.typography.contrast_dark.font_size = 40;
    let changed = intrinsic_body_minimum(&chart, &custom).unwrap();
    assert!(
        changed.0 > original.0 && changed.1 >= original.1,
        "original={original:?} changed={changed:?}"
    );
    assert_eq!(intrinsic_body_minimum(&chart, &theme).unwrap(), original);
}

#[test]
fn intrinsic_width_accumulates_only_occupied_column_text_instead_of_repeating_the_longest() {
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let measure = |values: [f64; 3]| {
        let source = Plot::heatmap(
            CategoryValues::new("column", ["a", "b", "c"]).unwrap(),
            CategoryValues::new("row", ["one"; 3]).unwrap(),
            NumericValues::new("value", values),
        )
        .annotations(HeatmapAnnotations::Values(
            crate::NumericLabelFormat::default(),
        ))
        .size(1200, 800)
        .erase();
        let resources = crate::font::canonical::OperationGenerationResources::new();
        let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
        let chart = inputs.attach(source).unwrap();
        let minimum = intrinsic_body_minimum(&chart, &theme).unwrap();
        chart.preflight().unwrap();
        minimum
    };
    let sparse = measure([1.0, 2.0, 123_456.78]);
    let dense = measure([123_456.78; 3]);
    assert!(sparse.0 < dense.0, "sparse={sparse:?} dense={dense:?}");
    assert_eq!(sparse.1.to_bits(), dense.1.to_bits());
}

#[test]
fn assigned_columns_keep_each_value_and_categorical_tick_at_its_measured_cell_center() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "c"]).unwrap(),
        CategoryValues::new("row", ["one"; 3]).unwrap(),
        NumericValues::new("value", [1.0, 2.0, 123_456.78]),
    )
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::default(),
    ))
    .size(1200, 800)
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(50.0, 1050.0, 80.0, 680.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("selected Heatmap")
    };
    assert_eq!(payload.cells.len(), 3);
    let widths = payload
        .cells
        .iter()
        .map(|cell| cell.corners[1].x - cell.corners[0].x)
        .collect::<Vec<_>>();
    assert!(widths[2] > widths[0] && widths[2] > widths[1], "{widths:?}");
    for (tick, cell) in selected.layout.x_axis.ticks.iter().zip(&payload.cells) {
        let (projected, _) = selected.layout.projection.project(tick.position, 0.5);
        let center = f64::midpoint(cell.corners[0].x, cell.corners[1].x);
        assert!((projected - center).abs() <= 1.0 / 1024.0);
    }
    assert!(payload.cells.iter().all(|cell| cell.text.is_some()));
}

#[test]
fn missing_fill_and_backdrop_invalidate_cached_intrinsics_with_unequal_contrast_roles() {
    use crate::theme::Color;

    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"]).unwrap(),
        CategoryValues::new("row", ["absent", "absent"]).unwrap(),
        NumericValues::nullable("fill", [None, None]),
    )
    .color_limits(0.0, 1.0)
    .heatmap_null_row_annotation(
        crate::HeatmapNullRowAnnotation::new("absent", "A long unavailable measurement").unwrap(),
    )
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let marks = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let axes = &inputs.axes[0].1;
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.surfaces.plot_panel = Color::rgba(0, 0, 0, 0);
    theme.surfaces.canvas_root = Color::rgba(0, 0, 0, 0);
    theme.marks.heatmap_missing_fill = Color::rgba(20, 40, 60, 0);
    theme.annotations.planning_backdrop = Color::rgb(0, 0, 0);
    theme.typography.contrast_light.font_size = 40;
    theme.typography.contrast_dark.font_size = 9;
    let large = prepare_intrinsics(marks, axes, source.data(), &theme, None).unwrap();
    assert_eq!(
        theme
            .contrasting_text(theme.marks.heatmap_missing_fill)
            .unwrap()
            .font_size,
        40
    );
    assert!(axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches(marks, &theme));

    theme.annotations.planning_backdrop = Color::rgb(255, 255, 255);
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches(marks, &theme));
    assert_eq!(
        theme
            .contrasting_text(theme.marks.heatmap_missing_fill)
            .unwrap()
            .font_size,
        9
    );
    let small = prepare_intrinsics(marks, axes, source.data(), &theme, None).unwrap();
    assert!(
        small.0 < large.0 && small.1 <= large.1,
        "small={small:?} large={large:?}"
    );

    theme.marks.heatmap_missing_fill = Color::rgb(0, 0, 0);
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches(marks, &theme));
    assert_eq!(
        theme
            .contrasting_text(theme.marks.heatmap_missing_fill)
            .unwrap()
            .font_size,
        40
    );
    assert_eq!(
        prepare_intrinsics(marks, axes, source.data(), &theme, None).unwrap(),
        large
    );
}

#[test]
fn constrained_merged_row_gets_its_own_height_and_keeps_all_cells_and_ticks() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "a", "b", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["first", "first", "second", "second", "absent", "absent"]).unwrap(),
        NumericValues::nullable("value", [Some(1.0), Some(2.0), Some(3.0), Some(4.0), None, None]),
    )
    .annotations(HeatmapAnnotations::Values(crate::NumericLabelFormat::default()))
    .heatmap_null_row_annotation(crate::HeatmapNullRowAnnotation::new(
        "absent",
        "This unavailable measurement requires several complete wrapped lines rather than clipping required annotation text.",
    ).unwrap())
    .size(600, 400)
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(
            chart.data(),
            PanelBox::new(50.0, 530.0, 50.0, 220.0),
            &theme,
        ))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &selected.marks.as_ref().unwrap().payload
    else {
        panic!("selected Heatmap")
    };
    assert_eq!(payload.cells.len(), 5);
    assert!(payload.cells.iter().all(|cell| cell.text.is_some()));
    let merged = &payload.cells[4];
    let height = |cell: &CellPayload| cell.corners[2].y - cell.corners[0].y;
    assert!(merged.text.as_ref().unwrap().plan.lines.len() > 1);
    assert!(height(merged) > height(&payload.cells[0]));
    assert!(height(merged) > height(&payload.cells[2]));
    assert_eq!(
        merged.corners[0].x.to_bits(),
        payload.cells[0].corners[0].x.to_bits()
    );
    assert_eq!(
        merged.corners[1].x.to_bits(),
        payload.cells[1].corners[1].x.to_bits()
    );
    assert_eq!(
        merged.corners[2].y.to_bits(),
        payload.cells[2].corners[0].y.to_bits()
    );
    assert_eq!(
        merged.corners[0].y.to_bits(),
        selected.layout.projection.panel_box().y0.to_bits()
    );
    for (row, cell) in [0, 2, 4].into_iter().enumerate() {
        let cell = &payload.cells[cell];
        let center = f64::midpoint(cell.corners[0].y, cell.corners[2].y);
        let (_, projected) = selected
            .layout
            .projection
            .project(0.0, selected.layout.y_axis.ticks[row].position);
        assert!((projected - center).abs() <= 1.0 / 1024.0);
        assert_eq!(
            selected.layout.y_axis.assigned_cells.as_ref().unwrap()[row],
            (cell.corners[0].y, cell.corners[2].y)
        );
    }
}

#[test]
fn native_heatmap_uses_traced_tracks_below_the_retired_parity_floor() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a"]).unwrap(),
        CategoryValues::new("row", ["b"]).unwrap(),
        NumericValues::new("value", [1.0]),
    )
    .x_label("")
    .y_label("")
    .erase()
    .with_suppressed_continuous_guide();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let chart = crate::plots::shared_x::prepare_plot(&source, &resources)
        .unwrap()
        .attach(source)
        .unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let hard = intrinsic_body_minimum(&chart, &theme).unwrap();
    assert!(hard.0 < 40.0 && hard.1 < 60.0);
    let panel = PanelBox::new(50.0, 83.0, 50.0, 74.0);
    let selected = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    assert!(selected.axes.is_some());
    assert_eq!(selected.layout.projection.panel_box(), panel);
    assert!(matches!(
        crate::layout::interior::InteriorRoute::classify(chart.kind()),
        crate::layout::interior::InteriorRoute::SpecializedGrid(
            crate::layout::interior::SpecializedGrid::Heatmap
        )
    ));
}

#[test]
fn heatmap_intrinsic_cache_rechecks_the_final_shared_color_domain() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"]).unwrap(),
        CategoryValues::new("row", ["one", "one"]).unwrap(),
        NumericValues::new("fill", [0.0, 1.0]),
    )
    .scale(crate::HeatmapScale::Diverging { center: 0.0 })
    .color_limits(-1.0, 1.0)
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::default()
            .suffix(" long units")
            .unwrap(),
    ))
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let mut inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.typography.contrast_light.font_size = 40;
    theme.typography.contrast_dark.font_size = 9;
    let axes = &inputs.axes[0].1;
    let prepared = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let initial = prepare_intrinsics(prepared, axes, source.data(), &theme, None).unwrap();
    let original = prepared.annotation_labels().last().unwrap().1;
    assert_eq!(theme.contrasting_text(original).unwrap().font_size, 40);
    let panels = std::sync::Arc::get_mut(inputs.heatmap.as_mut().unwrap()).unwrap();
    let prepared = &mut panels.panels[0].1;
    prepared
        .replace_domain(
            ColorDomain {
                minimum: -100.0,
                maximum: 100.0,
                center: Some(0.0),
            },
            source.data().with_axis_resources(&resources),
        )
        .unwrap();
    let remapped = prepared.annotation_labels().last().unwrap().1;
    assert_eq!(theme.contrasting_text(remapped).unwrap().font_size, 9);
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches(prepared, &theme));
    let changed = prepare_intrinsics(prepared, axes, source.data(), &theme, None).unwrap();
    assert!(changed.0 < initial.0);
}

#[test]
fn a_custom_gradient_invalidates_the_cached_contrast_role_at_the_same_numeric_domain() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b"]).unwrap(),
        CategoryValues::new("row", ["one", "one"]).unwrap(),
        NumericValues::new("fill", [0.0, 1.0]),
    )
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::default()
            .suffix(" long units")
            .unwrap(),
    ))
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let mut inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let axes = &inputs.axes[0].1;
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.typography.contrast_light.font_size = 40;
    theme.typography.contrast_dark.font_size = 9;
    let initial = prepare_intrinsics(
        &inputs.heatmap.as_ref().unwrap().panels[0].1,
        axes,
        source.data(),
        &theme,
        None,
    )
    .unwrap();
    let panels = std::sync::Arc::get_mut(inputs.heatmap.as_mut().unwrap()).unwrap();
    let marks = &mut panels.panels[0].1;
    marks.color = PreparedHeatmapColor::resolve(
        marks.color.domain,
        &[
            GradientStop::new(0.0, (255, 255, 255)),
            GradientStop::new(1.0, (255, 255, 255)),
        ],
        &resources,
    )
    .unwrap();
    marks.fills = source
        .data()
        .with_axis_resources(&resources)
        .collect_source(marks.cells.iter().map(|value| {
            value
                .map(|value| marks.color.scale.color(value, &resources))
                .transpose()
        }))
        .unwrap();
    assert!(!axes
        .heatmap_intrinsics
        .get()
        .unwrap()
        .matches(marks, &theme));
    assert!(marks
        .annotation_labels()
        .all(|(_, fill)| { theme.contrasting_text(fill).unwrap().font_size == 9 }));
    let changed = prepare_intrinsics(marks, axes, source.data(), &theme, None).unwrap();
    assert!(
        changed.0 < initial.0,
        "changed={changed:?} initial={initial:?}"
    );
}

#[test]
fn heatmap_cell_and_missing_cross_ink_supply_preselection_track_bounds() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "c", "d", "e"]).unwrap(),
        CategoryValues::new("row", ["u", "v", "w", "x", "y"]).unwrap(),
        NumericValues::nullable("fill", [Some(1.0), None, None, None, None]),
    )
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let baseline = chart.intrinsic_body_minimum(&theme).unwrap();
    theme.marks.heatmap_cell_outline.width = 100.0;
    let outlined = chart.intrinsic_body_minimum(&theme).unwrap();
    assert!(outlined.0 >= 500.0 && outlined.1 >= 500.0);
    assert!(outlined.0 > baseline.0 && outlined.1 > baseline.1);
    theme.marks.heatmap_cell_outline.width = 0.5;
    theme.marks.heatmap_missing_cross.width = 24.0;
    let crossed = chart.intrinsic_body_minimum(&theme).unwrap();
    assert!(crossed.0 >= 120.0 && crossed.1 >= 120.0);
    assert!(crossed.0 < outlined.0 && crossed.1 < outlined.1);
}

#[test]
fn highlighted_cell_outline_invalidates_only_its_matching_intrinsic_input() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "c"]).unwrap(),
        CategoryValues::new("row", ["one", "one", "one"]).unwrap(),
        NumericValues::new("fill", [1.0, 2.0, 3.0]),
    )
    .heatmap_highlight(crate::BooleanValues::new("selected", [true, false, false]))
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let baseline = chart.intrinsic_body_minimum(&theme).unwrap();
    let mut changed = theme.clone();
    changed.marks.heatmap_highlight_outline.width = 100.0;
    let minimum = chart.intrinsic_body_minimum(&changed).unwrap();
    assert!(minimum.0 >= 300.0 && minimum.0 > baseline.0);
    assert_eq!(chart.intrinsic_body_minimum(&theme).unwrap(), baseline);
}

#[test]
fn wide_missing_cell_cross_retains_complete_ink_within_its_cell() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "c", "d", "e"]).unwrap(),
        CategoryValues::new("row", ["u", "v", "w", "x", "y"]).unwrap(),
        NumericValues::nullable("fill", [Some(1.0), None, None, None, None]),
    )
    .size(1200, 900)
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let chart = inputs.attach(source).unwrap();
    let mut theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    theme.marks.heatmap_missing_cross.width = 24.0;
    let panel = PanelBox::new(50.0, 1000.0, 50.0, 800.0);
    let result = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    let crate::layout::interior::MarkPayload::Heatmap(payload) =
        &result.marks.as_ref().unwrap().payload
    else {
        panic!("selected Heatmap")
    };
    let mut observed = 0;
    for cell in &payload.cells {
        let Some(cross) = &cell.cross else { continue };
        observed += 1;
        let (left, right) = (cell.corners[0].x, cell.corners[1].x);
        let (bottom, top) = (cell.corners[0].y, cell.corners[2].y);
        let mut geometry = crate::render::geometry::GeometrySnapshot::new(&theme).unwrap();
        for segment in cross {
            geometry.line(segment, &payload.cross_style, None).unwrap();
        }
        for bounds in geometry.bounds() {
            assert!(bounds.left >= left && bounds.right <= right);
            assert!(bounds.bottom >= bottom && bounds.top <= top);
        }
    }
    assert!(observed > 0);
}

#[test]
fn census_borrows_retained_formatted_values_in_render_order() {
    let source = chart(&valid_data())
        .annotations(HeatmapAnnotations::Values(
            crate::NumericLabelFormat::new(2)
                .unwrap()
                .suffix(" units")
                .unwrap(),
        ))
        .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &crate::font::canonical::OperationGenerationResources::new(),
    )
    .unwrap();
    let source = inputs.attach(source).unwrap();
    let marks = retained(&source).unwrap();
    let sources: Vec<_> = marks
        .guide_labels()
        .chain(marks.annotation_labels().map(|(label, _)| label))
        .chain(marks.missing_guide_label())
        .collect();
    assert_eq!(
        sources,
        [
            "1",
            "3",
            "1.00 units",
            "2.00 units",
            "3.00 units",
            "Missing"
        ]
    );
}

#[test]
fn census_stops_on_first_admission_refusal_without_visiting_later_values() {
    let source = chart(&valid_data())
        .annotations(HeatmapAnnotations::Values(
            crate::NumericLabelFormat::default(),
        ))
        .erase();
    let inputs = crate::plots::shared_x::prepare_plot(
        &source,
        &crate::font::canonical::OperationGenerationResources::new(),
    )
    .unwrap();
    let source = inputs.attach(source).unwrap();
    crate::layout::plan_exact_root(
        320,
        240,
        |stage| {
            stage.font_resources().claim_visits(1_000_000).unwrap();
            let mut visits = 0;
            let marks = retained(&source).unwrap();
            let result = marks
                .guide_labels()
                .chain(marks.annotation_labels().map(|(label, _)| label))
                .chain(marks.missing_guide_label())
                .try_for_each(|source| {
                    visits += 1;
                    crate::font::canonical::format_census_source(
                        format_args!("{source}"),
                        Some(stage),
                    )
                    .map(|_| ())
                    .map_err(|error| Error::Render(error.to_string()))
                });
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("root processing visits"));
            assert_eq!(visits, 1);
            assert_eq!(stage.font_resources().usage().unwrap().font_bytes, 0);
            Ok(())
        },
        |(), assigned, _| Ok(crate::layout::MeasuredLayout::new(assigned, ())),
        |(), _, (), _| Ok(()),
    )
    .unwrap();
}

#[test]
fn guide_numbers_use_the_common_modern_grammar_and_normalize_signed_zero() {
    let source = chart(&valid_data()).erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    for (value, expected) in [
        (-0.0, "0"),
        (0.0, "0"),
        (0.00001, "1.00e-5"),
        (0.0001, "1.00e-4"),
        (1.23, "1.2"),
        (-999_999.999_999_9, "-1000k"),
        (1_000_000.0, "1M"),
        (f64::MAX, "1.80e308"),
    ] {
        let labels = crate::decorations::legend::descriptor::numeric_labels(
            &[value],
            source.data().with_axis_resources(&resources),
        )
        .unwrap();
        assert_eq!(labels, [expected]);
    }
}

#[test]
fn prepare_normalizes_categories_and_preserves_first_observed_axes() {
    let data = valid_data();

    let prepared = prepare_chart(&chart(&data)).unwrap();

    assert_eq!(
        (prepared.x_levels, prepared.y_levels),
        (
            vec!["b".to_string(), "a".to_string()],
            vec!["first".to_string(), "second row".to_string()]
        )
    );
}

#[test]
fn prepare_applies_independent_explicit_orders_to_shuffled_cells() {
    let data = HeatmapValues {
        x: CategoryValues::new("x", ["b", "a", "b", "a"]).unwrap(),
        y: CategoryValues::new("y", ["south", "north", "north", "south"]).unwrap(),
        fill: NumericValues::new("fill", [4.0, 1.0, 2.0, 3.0]),
    };
    let chart = chart(&data)
        .x_category_order(CategoryOrder::explicit([" a ", "b"]))
        .y_category_order(CategoryOrder::explicit([" north ", "south"]));

    let prepared = prepare_chart(&chart).unwrap();

    assert_eq!(prepared.x_levels, ["a", "b"]);
    assert_eq!(prepared.y_levels, ["north", "south"]);
    assert_eq!(prepared.cells, [Some(1.0), Some(2.0), Some(3.0), Some(4.0)]);
}

#[test]
fn prepare_rejects_inexact_explicit_axis_orders() {
    let data = valid_data();
    let orders = [vec!["b", "b"], vec!["b"], vec!["b", "a", "unknown"]];

    let errors = orders
        .into_iter()
        .map(|order| {
            prepare_chart(&chart(&data).x_category_order(CategoryOrder::explicit(order)))
                .unwrap_err()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert!(errors[0].contains("duplicate level"));
    assert!(errors[1].contains("omits observed level"));
    assert!(errors[2].contains("unknown level"));
}

#[test]
fn prepare_reorients_triangular_pairs_under_explicit_family_order() {
    let data = HeatmapValues {
        x: CategoryValues::new("x", ["a", "a", "b"]).unwrap(),
        y: CategoryValues::new("y", ["b", "c", "c"]).unwrap(),
        fill: NumericValues::new("fill", [1.0, 2.0, 3.0]),
    };
    let chart = chart(&data)
        .layout(HeatmapLayout::LowerTriangle { diagonal: false })
        .x_category_order(CategoryOrder::explicit(["c", "b", "a"]));

    let prepared = prepare_chart(&chart).unwrap();

    assert_eq!(prepared.x_levels, ["c", "b", "a"]);
    assert_eq!(prepared.y_levels, ["c", "b", "a"]);
    assert_eq!(prepared.cells[3], Some(3.0));
    assert_eq!(prepared.cells[6], Some(2.0));
    assert_eq!(prepared.cells[7], Some(1.0));
}

#[test]
fn prepare_builds_cartesian_grid_with_absent_and_null_cells() {
    let data = HeatmapValues {
        x: CategoryValues::new("x", ["a", "b", "a"]).unwrap(),
        y: CategoryValues::new("y", ["one", "one", "two"]).unwrap(),
        fill: NumericValues::nullable("fill", [Some(1.0), None, Some(3.0)]),
    };

    let prepared = prepare_chart(&chart(&data)).unwrap();

    assert_eq!(prepared.cells, [Some(1.0), None, Some(3.0), None]);
    assert!(prepared.has_missing);
}

#[test]
fn prepare_rejects_duplicate_normalized_pairs() {
    let data = HeatmapValues {
        x: CategoryValues::new("x", ["a b", " a  b "]).unwrap(),
        y: CategoryValues::new("y", ["one", "one"]).unwrap(),
        fill: NumericValues::new("fill", [1.0, 2.0]),
    };

    let error = prepare_chart(&chart(&data)).unwrap_err().to_string();

    assert!(error.contains("duplicate normalized"));
}

#[test]
fn prepare_rejects_null_and_empty_category_labels() {
    for x in [
        CategoryValues::nullable("x", [Some("a"), None]).unwrap(),
        CategoryValues::nullable("x", [Some("a"), Some(" \t ")]).unwrap(),
    ] {
        let data = HeatmapValues {
            x,
            y: CategoryValues::new("y", ["one", "two"]).unwrap(),
            fill: NumericValues::new("fill", [1.0, 2.0]),
        };
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_non_finite_and_all_null_fill() {
    for fill in [
        NumericValues::nullable("fill", [Some(1.0), Some(f64::NAN)]),
        NumericValues::nullable("fill", [Some(1.0), Some(f64::INFINITY)]),
        NumericValues::nullable("fill", [None, None]),
    ] {
        let data = HeatmapValues {
            x: CategoryValues::new("x", ["a", "b"]).unwrap(),
            y: CategoryValues::new("y", ["one", "one"]).unwrap(),
            fill,
        };
        assert!(prepare_chart(&chart(&data)).is_err());
    }
}

#[test]
fn prepare_rejects_cartesian_grid_above_safety_limit() {
    let levels = (0..=1_000)
        .map(|index| format!("v{index}"))
        .collect::<Vec<_>>();
    let data = HeatmapValues {
        x: CategoryValues::new("x", levels.clone()).unwrap(),
        y: CategoryValues::new("y", levels).unwrap(),
        fill: NumericValues::new("fill", vec![1.0; 1_001]),
    };

    let source = chart(&data).erase();
    let resources = crate::font::canonical::OperationGenerationResources::with_test_limits(
        10_000_000,
        100_000_000,
    );
    let error = prepare(&source, source.data().with_axis_resources(&resources))
        .unwrap_err()
        .to_string();

    assert!(error.contains("safety limit"), "{error}");
}

#[test]
fn color_domain_uses_exact_explicit_limits_and_validates_diverging_center() {
    let exact = color_domain(&[1.0], HeatmapScale::Sequential, Some((-2.0, 4.0))).unwrap();
    let error = color_domain(
        &[1.0],
        HeatmapScale::Diverging { center: 0.0 },
        Some((1.0, 2.0)),
    )
    .unwrap_err();

    assert_eq!(
        exact,
        ColorDomain {
            minimum: -2.0,
            maximum: 4.0,
            center: None
        }
    );
    assert!(error.to_string().contains("must contain center"));
}

#[test]
fn diverging_auto_domain_is_symmetric_and_handles_constant_signed_zero_and_subnormal() {
    let symmetric =
        color_domain(&[-2.0, 3.0], HeatmapScale::Diverging { center: 1.0 }, None).unwrap();
    let zero = color_domain(&[-0.0, 0.0], HeatmapScale::Diverging { center: 0.0 }, None).unwrap();
    let subnormal = f64::from_bits(1);
    let tiny = color_domain(
        &[0.0, subnormal],
        HeatmapScale::Diverging { center: 0.0 },
        None,
    )
    .unwrap();

    assert_eq!(
        symmetric,
        ColorDomain {
            minimum: -2.0,
            maximum: 4.0,
            center: Some(1.0)
        }
    );
    assert_eq!((zero.minimum, zero.maximum), (0.0, 0.0));
    assert_eq!((tiny.minimum, tiny.maximum), (-subnormal, subnormal));
}

#[test]
fn diverging_auto_domain_accepts_opposite_extremes_around_zero() {
    let domain = color_domain(
        &[-f64::MAX, f64::MAX],
        HeatmapScale::Diverging { center: 0.0 },
        None,
    )
    .unwrap();

    assert_eq!((domain.minimum, domain.maximum), (-f64::MAX, f64::MAX));
    assert!(crate::theme::ContinuousColors::normalized_fraction(
        (domain.minimum, domain.maximum),
        0.0
    )
    .unwrap()
    .is_finite());
}

#[test]
fn diverging_auto_domain_reports_unrepresentable_symmetry() {
    let error = color_domain(
        &[-f64::MAX],
        HeatmapScale::Diverging { center: f64::MAX },
        None,
    )
    .unwrap_err();

    assert!(error.to_string().contains("finite symmetric"));
}

#[test]
fn constant_domains_map_to_midpoint_colors_and_explicit_domains_saturate() {
    let sequential = ColorDomain {
        minimum: 2.0,
        maximum: 2.0,
        center: None,
    };
    let diverging = ColorDomain {
        minimum: -1.0,
        maximum: 1.0,
        center: Some(0.0),
    };

    let resources = crate::font::canonical::OperationGenerationResources::new();
    let sequential = PreparedHeatmapColor::resolve(
        sequential,
        &crate::theme::HEATMAP_SEQUENTIAL_STOPS,
        &resources,
    )
    .unwrap();
    let diverging = PreparedHeatmapColor::resolve(
        diverging,
        &crate::theme::HEATMAP_DIVERGING_STOPS,
        &resources,
    )
    .unwrap();
    assert_eq!(
        sequential.scale.color(2.0, &resources).unwrap(),
        (122, 146, 181)
    );
    assert_eq!(
        diverging.scale.color(0.0, &resources).unwrap(),
        (247, 247, 247)
    );
    assert_eq!(
        diverging.scale.color(-2.0, &resources).unwrap(),
        (49, 130, 189)
    );
    assert_eq!(
        diverging.scale.color(2.0, &resources).unwrap(),
        (222, 45, 38)
    );
}

#[test]
fn colorbar_gradient_stops_preserve_sequential_diverging_and_constant_scales() {
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let source = chart(&valid_data()).erase();
    let data = source.data().with_axis_resources(&resources);
    let sequential = PreparedHeatmapColor::resolve(
        ColorDomain {
            minimum: 0.0,
            maximum: 1.0,
            center: None,
        },
        &crate::theme::HEATMAP_SEQUENTIAL_STOPS,
        &resources,
    )
    .unwrap()
    .guide_stops(&[], data)
    .unwrap();
    let diverging = PreparedHeatmapColor::resolve(
        ColorDomain {
            minimum: -1.0,
            maximum: 3.0,
            center: Some(0.0),
        },
        &crate::theme::HEATMAP_DIVERGING_STOPS,
        &resources,
    )
    .unwrap()
    .guide_stops(&[], data)
    .unwrap();
    let constant = PreparedHeatmapColor::resolve(
        ColorDomain {
            minimum: 2.0,
            maximum: 2.0,
            center: None,
        },
        &crate::theme::HEATMAP_SEQUENTIAL_STOPS,
        &resources,
    )
    .unwrap()
    .guide_stops(&[], data)
    .unwrap();

    assert_eq!(
        sequential,
        [
            GradientStop::new(0.0, (247, 251, 255)),
            GradientStop::new(1.0, (8, 48, 107)),
        ]
    );
    assert_eq!(
        diverging,
        [
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(0.25, (247, 247, 247)),
            GradientStop::new(1.0, (222, 45, 38)),
        ]
    );
    assert_eq!(constant[0].color, constant[1].color);
}

#[test]
fn large_offset_diverging_center_uses_the_same_fraction_for_cell_and_guide() {
    let low = 1.0e300_f64;
    let center = f64::from_bits(low.to_bits() + 375);
    let high = f64::from_bits(low.to_bits() + 1000);
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let resolved = PreparedHeatmapColor::resolve(
        ColorDomain {
            minimum: low,
            maximum: high,
            center: Some(center),
        },
        &crate::theme::HEATMAP_DIVERGING_STOPS,
        &resources,
    )
    .unwrap();
    assert_eq!(
        resolved.scale.color(center, &resources).unwrap(),
        (247, 247, 247)
    );
    assert_eq!(resolved.scale.stops()[1].color, (247, 247, 247));
    assert_eq!(
        resolved.scale.stops()[1].offset.to_bits(),
        crate::theme::ContinuousColors::normalized_fraction((low, high), center)
            .unwrap()
            .to_bits()
    );
}

#[test]
fn private_plot_gradient_freezes_custom_sequential_and_diverging_marks_guides_before_fonts() {
    let sequential = [
        GradientStop::new(0.0, (0, 0, 0)),
        GradientStop::new(0.5, (255, 0, 0)),
        GradientStop::new(1.0, (255, 255, 255)),
    ];
    let diverging = [
        GradientStop::new(0.0, (0, 0, 255)),
        GradientStop::new(0.5, (0, 0, 0)),
        GradientStop::new(1.0, (255, 255, 0)),
    ];
    for (plot, expected, center) in [
        (
            Plot::heatmap(
                CategoryValues::new("column", ["low", "middle", "high"]).unwrap(),
                CategoryValues::new("row", ["one"; 3]).unwrap(),
                NumericValues::new("fill", [0.0, 5.0, 10.0]),
            )
            .color_limits(0.0, 10.0)
            .with_private_gradient(sequential.to_vec())
            .size(1200, 800)
            .erase(),
            [
                sequential[0].color,
                sequential[1].color,
                sequential[2].color,
            ],
            None,
        ),
        (
            Plot::heatmap(
                CategoryValues::new("column", ["low", "middle", "high"]).unwrap(),
                CategoryValues::new("row", ["one"; 3]).unwrap(),
                NumericValues::new("fill", [-2.0, 0.0, 6.0]),
            )
            .scale(HeatmapScale::Diverging { center: 0.0 })
            .color_limits(-2.0, 6.0)
            .with_private_gradient(diverging.to_vec())
            .theme(crate::Theme::Science)
            .size(1200, 800)
            .erase(),
            [diverging[0].color, diverging[1].color, diverging[2].color],
            Some(0.0),
        ),
    ] {
        let resources = crate::font::canonical::OperationGenerationResources::new();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let inputs = crate::plots::shared_x::prepare_plot(&plot, &resources).unwrap();
        let marks = &inputs.heatmap.as_ref().unwrap().panels[0].1;
        assert_eq!(marks.fills, expected.map(Some));
        let theme = plot
            .theme_spec()
            .unwrap_or(crate::Theme::Publication)
            .resolve(&crate::font::canonical::OperationGenerationResources::new())
            .unwrap();
        let semantics = marks
            .guide_semantics(&theme, plot.data().with_axis_resources(&resources))
            .unwrap();
        assert_eq!(semantics.center, center);
        assert_eq!(
            semantics
                .stops
                .iter()
                .map(|stop| stop.color)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
        let chart = inputs.attach(plot).unwrap();
        let selected = chart
            .measure_selected_interior(PlotRenderContext::new(
                chart.data(),
                PanelBox::new(50.0, 700.0, 50.0, 500.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Heatmap(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("custom Heatmap mark payload")
        };
        assert_eq!(
            payload
                .cells
                .iter()
                .map(|cell| cell.fill.color)
                .collect::<Vec<_>>(),
            expected.map(crate::theme::color_from_rgb)
        );
        assert_eq!(
            gradient_stops(payload.colorbar.as_ref().unwrap())[1].color,
            expected[1]
        );
    }

    let source = Plot::heatmap(
        CategoryValues::new("column", ["one", "two"]).unwrap(),
        CategoryValues::new("row", ["one", "one"]).unwrap(),
        NumericValues::new("fill", [0.0, 1.0]),
    )
    .with_private_gradient(vec![
        GradientStop::new(0.0, (0, 0, 0)),
        GradientStop::new(0.0, (255, 255, 255)),
    ])
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    assert!(crate::plots::shared_x::prepare_plot(
        &source,
        &crate::font::canonical::OperationGenerationResources::new()
    )
    .is_err());
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let wrong_center = Plot::heatmap(
        CategoryValues::new("column", ["one", "two"]).unwrap(),
        CategoryValues::new("row", ["one", "one"]).unwrap(),
        NumericValues::new("fill", [-1.0, 1.0]),
    )
    .scale(HeatmapScale::Diverging { center: 0.0 })
    .with_private_gradient(vec![
        GradientStop::new(0.0, (0, 0, 0)),
        GradientStop::new(0.4, (100, 100, 100)),
        GradientStop::new(1.0, (255, 255, 255)),
    ])
    .erase();
    assert!(matches!(
        crate::plots::shared_x::prepare_plot(
            &wrong_center,
            &crate::font::canonical::OperationGenerationResources::new()
        ),
        Err(Error::Data(message)) if message.contains("neutral center stop")
    ));
}

#[test]
fn private_gradient_survives_faceted_root_preparation_and_shared_raw_domain() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["one"; 4]).unwrap(),
        NumericValues::new("fill", [0.0, 5.0, 10.0, 5.0]),
    )
    .with_private_gradient(vec![
        GradientStop::new(0.0, (0, 0, 0)),
        GradientStop::new(0.5, (255, 0, 0)),
        GradientStop::new(1.0, (255, 255, 255)),
    ])
    .color_limits(0.0, 10.0)
    .facet(crate::FacetSpec::column(
        CategoryValues::new("panel", ["left", "left", "right", "right"]).unwrap(),
    ))
    .size(1200, 800)
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let panels = &inputs.heatmap.as_ref().unwrap().panels;
    assert_eq!(panels.len(), 2);
    for (_, marks) in panels {
        assert_eq!(marks.color.domain.minimum.to_bits(), 0.0_f64.to_bits());
        assert_eq!(marks.color.domain.maximum.to_bits(), 10.0_f64.to_bits());
        assert_eq!(marks.color.semantic_stops[1].color, (255, 0, 0));
        assert!(marks
            .fills
            .iter()
            .flatten()
            .any(|fill| *fill == (255, 0, 0)));
    }
    source.preflight().unwrap();
}

#[test]
fn prepared_cell_guide_and_value_label_share_final_diverging_colors() {
    let source = Plot::heatmap(
        CategoryValues::new("column", ["low", "middle", "high", "missing"]).unwrap(),
        CategoryValues::new("row", ["one"; 4]).unwrap(),
        NumericValues::nullable("fill", [Some(-2.0), Some(0.0), Some(2.0), None]),
    )
    .scale(HeatmapScale::Diverging { center: 0.0 })
    .annotations(HeatmapAnnotations::Values(
        crate::NumericLabelFormat::default(),
    ))
    .erase();
    let resources = crate::font::canonical::OperationGenerationResources::new();
    let inputs = crate::plots::shared_x::prepare_plot(&source, &resources).unwrap();
    let prepared = &inputs.heatmap.as_ref().unwrap().panels[0].1;
    let data = source.data().with_axis_resources(&resources);
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let guide = prepared.guide_semantics(&theme, data).unwrap();
    let stops = prepared
        .color
        .guide_stops(&prepared.guide_samples(data).unwrap(), data)
        .unwrap();
    assert_eq!(
        prepared.fills[..3],
        [
            Some(stops[0].color),
            Some(stops[1].color),
            Some(stops[2].color)
        ]
    );
    assert_eq!(stops, guide.stops);
    assert_eq!(prepared.fills[3], None);
    assert_eq!(
        guide.missing.as_ref().unwrap().fill,
        theme.marks.heatmap_missing_fill
    );
    assert_eq!(
        prepared
            .annotation_labels()
            .map(|(_, fill)| fill)
            .collect::<Vec<_>>(),
        prepared.fills[..3]
            .iter()
            .map(|fill| crate::theme::color_from_rgb(fill.unwrap()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn validate_options_rejects_heatmap_incompatible_configuration() {
    let data = valid_data();
    let charts = [
        chart(&data).x_category_order(ValueSort::Ascending).erase(),
        chart(&data)
            .scale(HeatmapScale::Diverging { center: f64::NAN })
            .erase(),
    ];

    assert!(charts.iter().all(|chart| validate_options(chart).is_err()));
}

#[test]
fn fitted_label_elides_only_at_extended_grapheme_boundaries() {
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let expected = "e\u{301}...";
    let maximum_width = theme
        .measure_text(
            expected,
            &theme.typography.tick_label,
            PhysicalTextAlignment::End,
            "heatmap category label",
            false,
        )
        .unwrap()
        .fit_width();

    let source = chart(&valid_data()).erase();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let fitted = fitted_label(
        source.data().with_axis_resources(&resources),
        &theme,
        "e\u{301}e\u{301}e\u{301}e\u{301}",
        maximum_width,
    )
    .unwrap();

    assert_eq!(fitted, expected);
    assert!(
        theme
            .measure_text(
                &fitted,
                &theme.typography.tick_label,
                PhysicalTextAlignment::End,
                "heatmap category label",
                false,
            )
            .unwrap()
            .width()
            <= maximum_width
    );
}

#[test]
fn fitted_unique_labels_rejects_space_too_narrow_for_an_ellipsis() {
    let theme = crate::Theme::Publication
        .resolve(&crate::font::canonical::OperationGenerationResources::new())
        .unwrap();
    let source = chart(&valid_data()).erase();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let error = fitted_unique_labels(
        source.data().with_axis_resources(&resources),
        &theme,
        &["category".into()],
        |_| 1.0,
        "x",
        &crate::layout::fit::FitScope::root(),
    )
    .unwrap_err()
    .into_error();

    assert!(error.to_string().contains("no room for an ellipsis"));
}
