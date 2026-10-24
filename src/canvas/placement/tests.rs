use super::*;

#[test]
fn complete_footprints_are_atomic_occupied_and_owned_only_at_their_anchor() {
    let mut placements = Placements::new(3, 3).unwrap();
    let text = || Payload::Text(TextPanel::new("Title", "Body").unwrap());
    let footprint = Footprint {
        row: 0,
        column: 1,
        row_span: 2,
        column_span: 2,
    };
    placements.insert(footprint, text()).unwrap();
    assert_eq!(placements.anchor(1, 2), Some(1));
    assert!(placements.footprint(5).is_none());
    assert_eq!(placements.texts().flatten().count(), 1);
    for invalid in [
        Footprint::cell(1, 2),
        Footprint {
            row: 1,
            column: 0,
            row_span: 2,
            column_span: 2,
        },
        Footprint {
            row: 0,
            column: 0,
            row_span: 0,
            column_span: 1,
        },
        Footprint {
            row: usize::MAX,
            column: 0,
            row_span: 2,
            column_span: 1,
        },
        Footprint {
            row: 0,
            column: 2,
            row_span: 1,
            column_span: 2,
        },
    ] {
        assert!(placements.insert(invalid, text()).is_err());
        assert_eq!(placements.texts().flatten().count(), 1);
        assert_eq!(placements.anchor(2, 0), None);
    }
}

#[test]
fn span_bands_require_anchor_ranges_and_matching_vertical_intervals_before_fonts() {
    let bar = || {
        crate::Plot::horizontal_bar(
            crate::CategoryValues::new("category", ["a", "b"]).unwrap(),
            crate::BarInput::estimates(crate::NumericValues::new("value", [1.0, 2.0])),
        )
    };
    crate::Canvas::grid(2, 3)
        .unwrap()
        .x_range_group([(0, 0), (0, 1)])
        .unwrap()
        .panel_span(0, 0, 2, 1, bar())
        .unwrap()
        .panel_span(0, 1, 2, 2, bar())
        .unwrap()
        .category_labels(crate::CanvasCategoryLabels::FirstColumnPerRow)
        .width(1200)
        .preflight()
        .unwrap();
    let calls = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let mismatched = crate::Canvas::grid(2, 2)
        .unwrap()
        .panel_span(0, 0, 2, 1, bar())
        .unwrap()
        .panel(0, 1, bar())
        .unwrap()
        .panel(1, 1, bar())
        .unwrap()
        .category_labels(crate::CanvasCategoryLabels::FirstColumnPerRow);
    assert!(matches!(mismatched.preflight(), Err(Error::Data(_))));
    let covered = crate::Canvas::grid(2, 2)
        .unwrap()
        .x_range_group([(0, 0), (1, 1)])
        .unwrap()
        .panel(0, 0, bar())
        .unwrap()
        .panel_span(0, 1, 2, 1, bar())
        .unwrap();
    assert!(matches!(covered.preflight(), Err(Error::Data(_))));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        calls
    );
    let text = || TextPanel::new("Note", "Body").unwrap();
    assert!(crate::Canvas::grid(2, 2)
        .unwrap()
        .panel_span(0, 0, 2, 2, bar())
        .unwrap()
        .text_panel(1, 1, text())
        .is_err());
    assert!(crate::Canvas::grid(2, 2)
        .unwrap()
        .text_panel(1, 1, text())
        .unwrap()
        .panel_span(0, 0, 2, 2, bar())
        .is_err());
}
