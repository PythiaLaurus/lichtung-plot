use super::*;
use crate::decorations::legend::discrete::LegendComposition;

fn empty_block() -> TextBlock {
    let theme = crate::Theme::publication().resolve().unwrap();
    TextBlock::from_resolved(
        fit::measured(fit::text(
            &theme,
            "",
            &theme.typography.guides.legend_label,
            0.0,
            PhysicalTextAlignment::Start,
            "fixed guide flow fixture",
            &FitScope::root(),
        ))
        .unwrap(),
    )
}

/// Font-independent, post-measurement operands from spec08's worked examples.
/// Empty retained plans are never drawn by these allocation/packing fixtures.
fn section(title: u32, widths: &[f64], heights: &[u32]) -> MeasuredSection {
    assert_eq!(widths.len(), heights.len());
    let block = empty_block();
    MeasuredSection {
        title: block.clone(),
        title_ascent: 0,
        title_width: ExactLength::pixels(10.0).unwrap(),
        title_occupancy: ExactLength::pixels(f64::from(title)).unwrap(),
        entries: widths
            .iter()
            .zip(heights)
            .map(|(&width, &height)| {
                let height = i64::from(height) * 1024;
                MeasuredEntry {
                    width: ExactLength::pixels(width).unwrap(),
                    key_origin: 0,
                    chrome: length(0).unwrap(),
                    ascent: height * 3 / 4,
                    descent: height / 4,
                    block: block.clone(),
                }
            })
            .collect(),
    }
}

fn metrics(row: f64, column: f64, title: f64) -> LegendMetrics {
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let spacing = theme
        .legend
        .spacing()
        .clone()
        .with_padding(crate::RelativeSpacing::ratio(0, 1).unwrap())
        .unwrap()
        .with_glyph_edge_padding(crate::RelativeSpacing::ratio(0, 1).unwrap())
        .unwrap()
        .with_row_gap(crate::RelativeSpacing::from_pixels(row, 11).unwrap())
        .unwrap()
        .with_column_gap(crate::RelativeSpacing::from_pixels(column, 11).unwrap())
        .unwrap()
        .with_title_gap(crate::RelativeSpacing::from_pixels(title, 12).unwrap())
        .unwrap();
    theme.legend = theme
        .legend
        .clone()
        .with_spacing(spacing)
        .unwrap()
        .with_frame(None)
        .unwrap();
    LegendMetrics::from_theme(&theme).unwrap()
}

#[test]
fn horizontal_sections_stack_actual_rows_and_compact_remeasures_can_be_taller() {
    let metrics = metrics(4.0, 5.0, 4.0);
    let scope = FitScope::root()
        .child(AddressRole::OutsideGuideAuto, 0)
        .unwrap();
    let normal = || {
        vec![
            section(12, &[60.0, 50.0], &[10, 10]),
            section(12, &[90.0], &[20]),
        ]
    };
    let assignment = Assignment::new((100.0, Some(76.0)), &metrics, &scope).unwrap();
    let layout = stack_sections(normal(), &assignment, true, &metrics, 0.0, &scope).unwrap();
    assert_eq!(layout.height, 76 * 1024);
    assert_eq!(
        layout
            .sections
            .iter()
            .map(|section| section.height)
            .collect::<Vec<_>>(),
        [36 * 1024, 32 * 1024]
    );
    assert_eq!(layout.sections[1].top, 44 * 1024);
    assert!(layout
        .sections
        .iter()
        .all(|section| section.capacity.is_none()));
    let smaller = Assignment::new((100.0, Some(76.0 - 1.0 / 1024.0)), &metrics, &scope).unwrap();
    assert!(matches!(
        stack_sections(normal(), &smaller, true, &metrics, 0.0, &scope),
        Err(PlanError::Fit(_))
    ));

    // These are separately supplied compact measurements, not normal heights
    // reused after the entry width changes to C2 = (100 - 5) / 2 = 47.5.
    let compact = || {
        vec![
            section(12, &[47.5, 47.5], &[20, 20]),
            section(12, &[47.5], &[30]),
        ]
    };
    assert!(matches!(
        stack_sections(compact(), &assignment, true, &metrics, 0.0, &scope),
        Err(PlanError::Fit(_))
    ));
    let taller = Assignment::new((100.0, Some(82.0)), &metrics, &scope).unwrap();
    let layout = stack_sections(compact(), &taller, true, &metrics, 0.0, &scope).unwrap();
    assert_eq!(layout.height, 82 * 1024);
    assert_eq!(layout.sections[0].columns, 2);
    assert_eq!(
        layout.sections[0].entries[0].key[1],
        layout.sections[0].entries[1].key[1]
    );
}

#[test]
fn a_shared_baseline_reserves_the_independent_maximum_ascent_and_descent() {
    let metrics = metrics(0.0, 0.0, 0.0);
    let scope = FitScope::root();
    let measured = || {
        let mut section = section(0, &[40.0, 40.0], &[20, 20]);
        section.entries[0].ascent = 18 * 1024;
        section.entries[0].descent = 2 * 1024;
        section.entries[1].ascent = 5 * 1024;
        section.entries[1].descent = 15 * 1024;
        vec![section]
    };
    let assignment = Assignment::new((100.0, Some(33.0)), &metrics, &scope).unwrap();
    let layout = stack_sections(measured(), &assignment, true, &metrics, 0.0, &scope).unwrap();
    assert_eq!(layout.height, 33 * 1024);
    assert_eq!(layout.sections[0].entries[0].key[1], -18 * 1024);
    assert_eq!(layout.sections[0].entries[1].key[1], -18 * 1024);
    let smaller = Assignment::new((100.0, Some(33.0 - 1.0 / 1024.0)), &metrics, &scope).unwrap();
    assert!(matches!(
        stack_sections(measured(), &smaller, true, &metrics, 0.0, &scope),
        Err(PlanError::Fit(_))
    ));
}

#[test]
fn vertical_sections_shrink_once_in_quanta_and_never_redistribute_unused_capacity() {
    let metrics = metrics(5.0, 10.0, 5.0);
    let scope = FitScope::root()
        .child(AddressRole::OutsideGuideAuto, 0)
        .unwrap();
    let normal = || {
        vec![
            section(10, &[45.0; 3], &[20; 3]),
            section(10, &[90.0; 2], &[20; 2]),
        ]
    };
    assert_eq!(
        section_capacities(&normal(), 100 * 1024, &metrics, &scope).unwrap(),
        [50 * 1024, 40 * 1024]
    );
    let short = Assignment::new((100.0, Some(100.0)), &metrics, &scope).unwrap();
    let Err(PlanError::Fit(short_failure)) =
        stack_sections(normal(), &short, false, &metrics, 0.0, &scope)
    else {
        panic!("three fixed-width columns cannot fit W=100")
    };
    assert_eq!(
        short_failure.reason.address,
        scope.child(AddressRole::LegendSection, 0).unwrap().address
    );
    assert_eq!(
        section_capacities(&normal(), 120 * 1024, &metrics, &scope).unwrap(),
        [64853, 47787]
    );
    let assignment = Assignment::new((100.0, Some(120.0)), &metrics, &scope).unwrap();
    let Err(PlanError::Fit(normal_failure)) =
        stack_sections(normal(), &assignment, false, &metrics, 0.0, &scope)
    else {
        panic!("B's two 90px columns must miss")
    };
    assert_eq!(
        normal_failure.reason.address,
        scope.child(AddressRole::LegendSection, 1).unwrap().address
    );
    let compact = vec![
        section(10, &[45.0; 3], &[20; 3]),
        section(10, &[45.0; 2], &[20; 2]),
    ];
    let layout = stack_sections(compact, &assignment, false, &metrics, 0.0, &scope).unwrap();
    assert_eq!((layout.width, layout.height), (100 * 1024, 95 * 1024));
    assert_eq!(
        layout
            .sections
            .iter()
            .map(|section| section.capacity)
            .collect::<Vec<_>>(),
        [Some(64853), Some(47787)]
    );
    assert_eq!(
        layout
            .sections
            .iter()
            .map(|section| (section.height, section.columns))
            .collect::<Vec<_>>(),
        [(55 * 1024, 2), (30 * 1024, 2)]
    );
    assert_eq!(64853 - layout.sections[0].height, 8533);
    assert_eq!(47787 - layout.sections[1].height, 17067);
    let automatic = Assignment::new((100.0, None), &metrics, &scope).unwrap();
    let layout = stack_sections(normal(), &automatic, false, &metrics, 0.0, &scope).unwrap();
    assert_eq!(layout.height, 145 * 1024);
    assert!(layout
        .sections
        .iter()
        .all(|section| section.columns == 1 && section.capacity.is_none()));
}

fn factorized_fixture() -> (Arc<PreparedDiscreteGuide>, ResolvedTheme) {
    use crate::{CategoryValues, InteractionStyle, PointShape};
    let plot = crate::plots::interaction::extraction::source()
        .hue(
            CategoryValues::new(
                "hue",
                ["control", "control", "treated", "treated", "treated"],
            )
            .unwrap(),
        )
        .styles([
            InteractionStyle::new("one")
                .unwrap()
                .with_point_shape(PointShape::TriangleUp)
                .unwrap(),
            InteractionStyle::new("two")
                .unwrap()
                .with_point_shape(PointShape::Circle)
                .unwrap(),
        ])
        .labels(crate::Labels::raw().alias("group", "Group styles\nsecond line"))
        .erase();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let points = theme
        .marks
        .points()
        .clone()
        .with_radii(
            theme
                .marks
                .points()
                .radii()
                .clone()
                .with_interaction(23)
                .unwrap(),
        )
        .unwrap()
        .with_primary(
            crate::PointTheme::new(
                17,
                PointShape::Circle,
                Some(
                    crate::PointOutlineTheme::new(
                        crate::Color::rgba(3, 5, 7, 211),
                        crate::RelativeSpacing::ratio(5, 23).unwrap(),
                    )
                    .unwrap(),
                ),
            )
            .unwrap(),
        )
        .unwrap();
    theme.marks = theme.marks.clone().with_points(points).unwrap();
    let descriptor =
        PreparedDiscreteGuide::prepare_for_test(&plot, &theme, LegendComposition::Separate)
            .unwrap()
            .unwrap();
    (descriptor, theme)
}

#[test]
fn factorized_keys_and_multiline_titles_freeze_in_one_contained_frame() {
    let (descriptor, theme) = factorized_fixture();
    assert_eq!(
        (descriptor.counts.sections, descriptor.counts.entries),
        (2, 4)
    );
    let scope = FitScope::root()
        .child(AddressRole::CanvasSharedLegend, 0)
        .unwrap();
    let mut frameless_theme = crate::Theme::publication().resolve().unwrap();
    frameless_theme.legend = theme.legend.clone().with_frame(None).unwrap();
    for horizontal in [false, true] {
        let measured = fit::measured(Legend::measure_assigned(
            Arc::clone(&descriptor),
            (420.0, None),
            horizontal,
            false,
            &theme,
            &mut false,
            &scope,
        ))
        .unwrap();
        let frameless = fit::measured(Legend::measure_assigned(
            Arc::clone(&descriptor),
            (420.0, None),
            horizontal,
            false,
            &frameless_theme,
            &mut false,
            &scope,
        ))
        .unwrap();
        assert_eq!(
            (measured.layout.width, measured.layout.height),
            (frameless.layout.width, frameless.layout.height)
        );
        assert_frozen_sections(&measured, &descriptor, 1);
        assert_frozen_sections(&frameless, &descriptor, 0);
    }
}

fn assert_frozen_sections(measured: &Legend, descriptor: &PreparedDiscreteGuide, frames: usize) {
    assert_eq!(measured.layout.sections.len(), 2);
    assert_eq!(measured.layout.sections[0].title.plan().lines.len(), 2);
    let bounds = measured.box_at(20.0, 30.0);
    let selected = measured
        .select(bounds.anchor().0, bounds.anchor().1)
        .unwrap();
    assert_eq!(
        selected
            .calls
            .iter()
            .filter(|call| matches!(call, super::super::LegendCall::Background(_, _)))
            .count(),
        frames
    );
    let text = selected
        .calls
        .iter()
        .filter_map(|call| match call {
            super::super::LegendCall::Text(text) => Some(text.plan.source.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        text,
        [
            "Group styles\nsecond line",
            "one",
            "two",
            "hue",
            "control",
            "treated"
        ]
    );
    let points = selected
        .calls
        .iter()
        .filter_map(|call| match call {
            super::super::LegendCall::Point(_, style) => Some(style),
            _ => None,
        })
        .collect::<Vec<_>>();
    for (point, entry) in points.iter().zip(
        descriptor
            .sections
            .iter()
            .flat_map(|section| section.entries.iter()),
    ) {
        assert_eq!(
            crate::decorations::legend::ResolvedGuideKey::point((*point).clone()),
            crate::decorations::legend::ResolvedGuideKey::point(
                entry.key.point.as_ref().unwrap().clone()
            )
        );
    }
    let mut ink = crate::render::geometry::GeometrySnapshot::new();
    selected.geometry(&mut ink).unwrap();
    assert!(ink.bounds().iter().all(|ink| ink.left >= bounds.left
        && ink.right <= bounds.right
        && ink.bottom >= bounds.bottom
        && ink.top <= bounds.top));
    let shape_calls = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let replay = measured
        .select(bounds.anchor().0, bounds.anchor().1)
        .unwrap();
    let mut replay_ink = crate::render::geometry::GeometrySnapshot::new();
    replay.geometry(&mut replay_ink).unwrap();
    assert_eq!(ink.bounds(), replay_ink.bounds());
    assert_eq!(
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        shape_calls
    );
}
