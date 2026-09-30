use super::*;
use crate::{CategoryOrder, CategoryValues, NumericValues, ValueSort};

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
    let mut theme = crate::Theme::Publication.resolve();
    let original = chart
        .measure_selected_interior(PlotRenderContext::new(chart.data(), panel, &theme))
        .unwrap();
    theme.marks.heatmap_cell_outline.paint = crate::render::backend::Rgba::new(1, 3, 5, 110);
    theme.marks.heatmap_cell_outline.width = 0.7;
    theme.marks.heatmap_missing_cross.paint = crate::render::backend::Rgba::new(7, 9, 11, 220);
    theme.marks.heatmap_missing_fill = crate::render::backend::Rgba::new(19, 23, 29, 144);
    theme.marks.heatmap_missing_key_outline.paint =
        crate::render::backend::Rgba::new(13, 15, 17, 90);
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
            payload
                .colorbar
                .as_ref()
                .unwrap()
                .missing
                .as_ref()
                .unwrap()
                .fill
                .clone(),
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
    let prepared = &chart.prepared_heatmap.as_ref().unwrap().panels[0].1;
    let missing = prepared
        .guide_semantics(&theme, chart.data().with_axis_resources(&resources))
        .unwrap()
        .missing
        .unwrap();
    assert_eq!(missing.fill, new_missing);
    assert_eq!(missing.outline, new_key.stroke);
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
    let mut theme = crate::Theme::Publication.resolve();
    theme.marks.heatmap_missing_fill = crate::render::backend::Rgba::new(30, 40, 50, 175);
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
    let key = payload.colorbar.as_ref().unwrap().missing.as_ref().unwrap();
    assert_eq!(row.fill.color, theme.marks.heatmap_missing_fill);
    assert_eq!(missing.fill.color, row.fill.color);
    assert_eq!(key.fill.color, row.fill.color);
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
    let theme = crate::Theme::Publication.resolve();
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
fn missing_fill_and_backdrop_invalidate_cached_intrinsics_with_unequal_contrast_roles() {
    use crate::render::backend::Rgba;

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
    let mut theme = crate::Theme::Publication.resolve();
    theme.surfaces.plot_panel = Rgba::new(0, 0, 0, 0);
    theme.surfaces.canvas_root = Rgba::new(0, 0, 0, 0);
    theme.marks.heatmap_missing_fill = Rgba::new(20, 40, 60, 0);
    theme.annotations.planning_backdrop = Rgba::rgb(0, 0, 0);
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

    theme.annotations.planning_backdrop = Rgba::rgb(255, 255, 255);
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

    theme.marks.heatmap_missing_fill = Rgba::rgb(0, 0, 0);
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
    let mut theme = crate::Theme::Publication.resolve();
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
    let mut theme = crate::Theme::Publication.resolve();
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
    let theme = crate::Theme::Publication.resolve();
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
    let mut theme = crate::Theme::Publication.resolve();
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
fn colorbar_display_preserves_signed_zero_thresholds_and_trimming() {
    for value in [
        -0.0,
        0.0,
        0.00001,
        0.0001,
        1.23,
        -999_999.999_999_9,
        1_000_000.0,
        f64::MAX,
    ] {
        let normalized = if value.abs().to_bits() == 0 {
            0.0
        } else {
            value
        };
        let expected = if normalized.abs() >= 1_000_000.0
            || (normalized.abs().to_bits() != 0 && normalized.abs() < 0.0001)
        {
            format!("{normalized:.3e}")
        } else {
            format!("{normalized:.6}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };
        assert_eq!(ColorbarValue(value).to_string(), expected);
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
    let sequential = PreparedHeatmapColor::resolve(sequential, &resources).unwrap();
    let diverging = PreparedHeatmapColor::resolve(diverging, &resources).unwrap();
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
    let theme = crate::Theme::Publication.resolve();
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
            .map(|fill| crate::render::backend::Rgba::from(fill.unwrap()))
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
    let theme = crate::Theme::Publication.resolve();
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
    let theme = crate::Theme::Publication.resolve();
    let source = chart(&valid_data()).erase();
    let resources = theme
        .prepared_font_environment()
        .unwrap()
        .generation_resources();
    let error = fitted_unique_labels(
        source.data().with_axis_resources(&resources),
        &theme,
        &["category".into()],
        1.0,
        "x",
        &crate::layout::fit::FitScope::root(),
    )
    .unwrap_err()
    .into_error();

    assert!(error.to_string().contains("no room for an ellipsis"));
}
