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
fn region_replacement_removes_whole_owners_and_commits_only_valid_topology() {
    let mut placements = Placements::new(2, 3).unwrap();
    let text = |name| Payload::Text(TextPanel::new(name, "Body").unwrap());
    placements
        .insert(
            Footprint {
                row: 0,
                column: 0,
                row_span: 2,
                column_span: 2,
            },
            text("A"),
        )
        .unwrap();
    placements.insert(Footprint::cell(0, 2), text("B")).unwrap();
    placements.insert(Footprint::cell(1, 2), text("C")).unwrap();
    let original = placements.clone();
    for invalid in [
        Footprint {
            row: 1,
            column: 1,
            row_span: 2,
            column_span: 1,
        },
        Footprint {
            row: 1,
            column: 1,
            row_span: 0,
            column_span: 1,
        },
        Footprint {
            row: 1,
            column: usize::MAX,
            row_span: 1,
            column_span: 2,
        },
    ] {
        assert!(placements.replace(invalid, text("invalid")).is_err());
        assert_eq!(placements.occupancy, original.occupancy);
        assert_eq!(placements.text(0), original.text(0));
        assert_eq!(placements.footprint(0), original.footprint(0));
    }
    let plot = || {
        Payload::Plot(Arc::new(
            crate::Plot::scatter(
                crate::NumericValues::new("x", [0., 1.]),
                crate::NumericValues::new("y", [1., 2.]),
            )
            .erase(),
        ))
    };
    placements.replace(Footprint::cell(1, 1), plot()).unwrap();
    assert_eq!(
        placements.occupancy,
        [None, None, Some(2), None, Some(4), Some(5)]
    );
    assert_eq!(
        placements.plot(4).unwrap().canvas_owner.as_deref(),
        Some(&[2, 3, 1, 1, 1, 1])
    );
    assert!(placements.text(0).is_none());
    assert_eq!(placements.text(2).unwrap().title(), "B");
    assert_eq!(placements.text(5).unwrap().title(), "C");
    assert_eq!(original.text(0).unwrap().title(), "A");

    placements
        .replace(
            Footprint {
                row: 0,
                column: 1,
                row_span: 2,
                column_span: 2,
            },
            plot(),
        )
        .unwrap();
    assert_eq!(
        placements.occupancy,
        [None, Some(1), Some(1), None, Some(1), Some(1)]
    );
    assert_eq!(placements.plots().flatten().count(), 1);
    assert_eq!(placements.texts().flatten().count(), 0);
    placements
        .insert(Footprint::cell(0, 0), text("edge neighbor"))
        .unwrap();
    assert_eq!(placements.anchor(0, 0), Some(0));
    assert_eq!(placements.anchor(1, 1), Some(1));
}

#[test]
fn borrowed_canvas_edits_preserve_failures_and_separate_shared_placement_snapshots() {
    let plot = || {
        crate::Plot::scatter(
            crate::NumericValues::new("x", [0., 1.]),
            crate::NumericValues::new("y", [1., 2.]),
        )
        .title("captured")
    };
    let source = plot();
    let mut canvas = crate::Canvas::grid(1, 3).unwrap();
    let unique = Arc::as_ptr(&canvas.placements);
    canvas
        .place_panel(0, 0, 1, 1, source.clone(), false)
        .unwrap();
    assert_eq!(
        Arc::as_ptr(&canvas.placements),
        unique,
        "unshared insertion does not clone the owner set"
    );
    let snapshot = canvas.clone();
    assert!(Arc::ptr_eq(&canvas.placements, &snapshot.placements));
    let source = source.title("external later edit");
    assert_eq!(
        snapshot.placements.plot(0).unwrap().prose()[0]
            .unwrap()
            .as_str(),
        "captured"
    );
    assert_eq!(source.prose()[0].unwrap().as_str(), "external later edit");

    assert!(canvas.place_panel(0, 0, 1, 1, plot(), false).is_err());
    assert!(canvas.place_panel(0, 1, 0, 1, plot(), true).is_err());
    assert!(canvas
        .place_panel(0, 1, 1, usize::MAX, plot(), true)
        .is_err());
    assert_eq!(canvas.placements.anchor(0, 0), Some(0));
    assert_eq!(canvas.placements.anchor(0, 1), None);
    canvas
        .place_text_panel(0, 1, TextPanel::new("Independent note", "Body").unwrap())
        .unwrap();
    assert!(!Arc::ptr_eq(&canvas.placements, &snapshot.placements));
    assert!(
        std::ptr::eq(
            canvas.placements.plot(0).unwrap(),
            snapshot.placements.plot(0).unwrap()
        ),
        "unchanged native plot payloads remain shared"
    );
    assert!(snapshot.placements.text(1).is_none());
    assert!(canvas
        .place_text_panel(0, 0, TextPanel::new("Rejected", "Body").unwrap())
        .is_err());
    canvas.place_panel(0, 1, 1, 2, source, true).unwrap();
    assert!(canvas.placements.text(1).is_none());
    assert_eq!(canvas.placements.anchor(0, 2), Some(1));
    assert_eq!(snapshot.placements.anchor(0, 2), None);
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
