use super::*;

#[test]
fn category_label_selection_keeps_twelve_bars_and_displays_six_month_names() {
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let plot = Plot::bar(
        CategoryValues::new("month", months).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0; 12])),
    )
    .labels(Labels::raw())
    .x_category_labels(lichtung_plot::CategoryLabelSelection::every(2).unwrap())
    .size(1800, 600);
    let output = unique_dir("selected-month-labels").join("months.svg");
    plot.clone().svg_preserve_text(&output).unwrap();
    let svg = std::fs::read_to_string(output).unwrap();
    assert_eq!(parse_polygon_bounds(&svg).len(), 12);
    let painted = parse_text_elements(&svg);
    for (index, month) in months.iter().enumerate() {
        assert_eq!(
            painted.iter().filter(|text| text.content == *month).count(),
            usize::from(index % 2 == 0)
        );
    }
    assert!(plot
        .labels(
            Labels::raw()
                .alias("January", "same")
                .alias("February", "same")
        )
        .preflight()
        .is_err());
}

#[test]
fn facet_category_selection_uses_final_local_domains_before_font_admission() {
    let plot = Plot::bar(
        CategoryValues::new("category", ["a", "\u{10fffe}", "c", "d", "\u{10ffff}", "e"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0; 6])),
    )
    .labels(Labels::raw())
    .x_category_labels(lichtung_plot::CategoryLabelSelection::every(2).unwrap())
    .facet(
        lichtung_plot::FacetSpec::column(
            CategoryValues::new(
                "facet",
                ["first", "first", "first", "second", "second", "second"],
            )
            .unwrap(),
        )
        .scales(lichtung_plot::FacetScales::FreeX),
    )
    .size(1000, 500);
    let path = unique_dir("facet-selected-categories").join("facets.svg");
    plot.svg_preserve_text(&path).unwrap();
    let svg = std::fs::read_to_string(path).unwrap();
    assert_eq!(parse_polygon_bounds(&svg).len(), 6);
    let text = parse_text_elements(&svg);
    for category in ["a", "c", "d", "e"] {
        assert_eq!(
            text.iter().filter(|text| text.content == category).count(),
            1
        );
    }
}

#[test]
fn shared_category_axes_compare_complete_selection_even_when_current_labels_agree() {
    let source = Plot::horizontal_bar(
        CategoryValues::new("category", ["a", "b", "c"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [1.0; 3])),
    );
    let make = |stride| {
        source
            .clone()
            .y_category_labels(lichtung_plot::CategoryLabelSelection::every(stride).unwrap())
    };
    let canvas = |right| {
        lichtung_plot::Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .category_labels(lichtung_plot::CanvasCategoryLabels::FirstColumnPerRow)
            .panel(0, 0, make(6))
            .unwrap()
            .panel(0, 1, make(right))
            .unwrap()
    };
    canvas(6).preflight().unwrap();
    assert!(matches!(
        canvas(7).preflight(),
        Err(lichtung_plot::Error::Data(_))
    ));
}

#[test]
fn bar_order_is_plot_owned_and_value_sort_runs_after_mean_aggregation() {
    let categories = CategoryValues::new("species", ["b", "b", "a", "a", "c", "c"]).unwrap();
    let values = NumericValues::new("value", [10.0, 0.0, 1.0, 1.0, 8.0, 8.0]);
    let dir = unique_dir("bar-plot-owned-order");
    let explicit_path = dir.join("explicit.svg");
    let sorted_path = dir.join("sorted.svg");

    Plot::bar(categories.clone(), BarInput::samples(values.clone()))
        .category_order(CategoryOrder::explicit(["c", "a", "b"]))
        .size(320, 240)
        .svg_preserve_text(&explicit_path)
        .unwrap();
    Plot::bar(categories, BarInput::samples(values))
        .category_order(ValueSort::Descending)
        .size(320, 240)
        .svg_preserve_text(&sorted_path)
        .unwrap();

    let explicit = std::fs::read_to_string(explicit_path).unwrap();
    let sorted = std::fs::read_to_string(sorted_path).unwrap();
    assert!(explicit.find(">C</text>") < explicit.find(">A</text>"));
    assert!(explicit.find(">A</text>") < explicit.find(">B</text>"));
    assert!(sorted.find(">C</text>") < sorted.find(">B</text>"));
    assert!(sorted.find(">B</text>") < sorted.find(">A</text>"));
}

#[test]
fn grouped_bar_rejects_ambiguous_value_order() {
    let category = CategoryValues::new("category", ["a", "a", "b", "b"]).unwrap();
    let group = CategoryValues::new("group", ["one", "two", "one", "two"]).unwrap();
    let value = NumericValues::new("value", [1.0, 2.0, 3.0, 4.0]);

    let error = Plot::bar(category, BarInput::estimates(value))
        .series(group)
        .category_order(ValueSort::Descending)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("requires one ungrouped bar per category"));
}
