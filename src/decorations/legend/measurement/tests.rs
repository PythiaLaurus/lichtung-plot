use super::*;
use crate::LegendComposition;

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
        color: None,
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
        .get_spacing()
        .clone()
        .padding(crate::RelativeSpacing::ratio(0, 1).unwrap())
        .unwrap()
        .glyph_edge_padding(crate::RelativeSpacing::ratio(0, 1).unwrap())
        .unwrap()
        .row_gap(crate::RelativeSpacing::from_pixels(row, 11).unwrap())
        .unwrap()
        .column_gap(crate::RelativeSpacing::from_pixels(column, 11).unwrap())
        .unwrap()
        .title_gap(crate::RelativeSpacing::from_pixels(title, 12).unwrap())
        .unwrap();
    theme.legend = theme
        .legend
        .clone()
        .spacing(spacing)
        .unwrap()
        .frame(None)
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

fn factorized_plot(group_field: &str) -> crate::Plot {
    use crate::{CategoryValues, InteractionStyle, PointShape};
    crate::Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "c"]).unwrap(),
        crate::NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new(group_field, ["one", "one", "one", "two", "two"]).unwrap(),
        crate::Bounds::new(
            crate::NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            crate::NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
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
            .point_shape(PointShape::TriangleUp)
            .unwrap(),
        InteractionStyle::new("two")
            .unwrap()
            .point_shape(PointShape::Circle)
            .unwrap(),
    ])
    .labels(crate::Labels::raw().alias(group_field, "Group styles\nsecond line"))
    .erase()
}

fn factorized_fixture() -> (Arc<PreparedGuide>, ResolvedTheme) {
    use crate::PointShape;
    let plot = factorized_plot("group");
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let points = theme
        .marks
        .get_points()
        .clone()
        .radii(
            theme
                .marks
                .get_points()
                .get_radii()
                .clone()
                .interaction(23)
                .unwrap(),
        )
        .unwrap()
        .primary(
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
    theme.marks = theme.marks.clone().points(points).unwrap();
    let descriptor = PreparedGuide::prepare_for_test(&plot, &theme, LegendComposition::Separate)
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
    frameless_theme.legend = theme.legend.clone().frame(None).unwrap();
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

fn assert_frozen_sections(measured: &Legend, descriptor: &PreparedGuide, frames: usize) {
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
            .flat_map(|section| section.entries().iter()),
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

fn measure_fixture(
    descriptor: &Arc<PreparedGuide>,
    available: (f64, Option<f64>),
    variant: (bool, bool),
    theme: &ResolvedTheme,
    scope: &FitScope,
) -> Legend {
    fit::measured(Legend::measure_assigned(
        Arc::clone(descriptor),
        available,
        variant.0,
        variant.1,
        theme,
        &mut false,
        scope,
    ))
    .unwrap()
}

fn frozen_svg(measured: &Legend, theme: &ResolvedTheme) -> Vec<u8> {
    use crate::render::backend::Backend;
    let mut backend = crate::render::svg_backend::SvgBackend::in_memory_with_fonts(
        600,
        600,
        theme.prepared_font_environment().unwrap().clone(),
    );
    backend.begin(600, 600).unwrap();
    let (x, y) = measured.box_at(20.0, 30.0).anchor();
    measured.select(x, y).unwrap().draw(&mut backend).unwrap();
    backend.end().unwrap();
    backend.into_bytes().unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one actual scoped cache key proves complete consumed-input identity and same-operation uncached replay"
)]
fn scoped_section_measurements_reuse_values_and_invalidate_consumed_inputs() {
    let (descriptor, theme) = factorized_fixture();
    let scope = FitScope::root()
        .child(AddressRole::OutsideGuideAuto, 0)
        .unwrap();
    let original = measure_fixture(&descriptor, (420.0, None), (false, false), &theme, &scope);
    let equal_value = Arc::new(PreparedGuide {
        composition: descriptor.composition,
        counts: descriptor.counts,
        sections: Arc::clone(&descriptor.sections),
    });
    assert!(!Arc::ptr_eq(&equal_value, &descriptor));
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let repeated = measure_fixture(&equal_value, (420.0, None), (false, false), &theme, &scope);
    let duplicate_compact =
        measure_fixture(&equal_value, (420.0, None), (false, true), &theme, &scope);
    assert!(Arc::ptr_eq(&original.layout, &repeated.layout));
    assert!(Arc::ptr_eq(&original.layout, &duplicate_compact.layout));
    assert_eq!(
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        shapes
    );

    let uncached = fit::measured(measure_uncached(
        Arc::clone(&descriptor),
        (420.0, None),
        false,
        false,
        &theme,
        &mut false,
        &scope,
    ))
    .unwrap();
    assert!(!Arc::ptr_eq(&original.layout, &uncached.layout));
    assert_eq!(frozen_svg(&original, &theme), frozen_svg(&uncached, &theme));

    let different_source = PreparedGuide::prepare_for_test(
        &factorized_plot("other group"),
        &theme,
        LegendComposition::Separate,
    )
    .unwrap()
    .unwrap();
    let different = measure_fixture(
        &different_source,
        (420.0, None),
        (false, false),
        &theme,
        &scope,
    );
    assert!(!Arc::ptr_eq(&original.layout, &different.layout));
    assert_eq!(
        (original.layout.width, original.layout.height),
        (different.layout.width, different.layout.height)
    );
    assert_eq!(
        frozen_svg(&original, &theme),
        frozen_svg(&different, &theme)
    );

    let mut painted = theme.clone();
    painted.typography.guides.legend_label = painted
        .typography
        .guides
        .legend_label
        .clone()
        .color(crate::Color::rgba(11, 23, 37, 71))
        .unwrap();
    let new_paint = measure_fixture(&descriptor, (420.0, None), (false, false), &painted, &scope);
    assert!(!Arc::ptr_eq(&original.layout, &new_paint.layout));
    assert_eq!(
        (original.layout.width, original.layout.height),
        (new_paint.layout.width, new_paint.layout.height)
    );
    assert_ne!(
        frozen_svg(&original, &theme),
        frozen_svg(&new_paint, &painted)
    );
    let mut unrelated = theme.clone();
    unrelated.surfaces.plot_panel = crate::Color::rgb(1, 2, 3);
    let unrelated = measure_fixture(
        &descriptor,
        (420.0, None),
        (false, false),
        &unrelated,
        &scope,
    );
    assert!(Arc::ptr_eq(&original.layout, &unrelated.layout));

    let other_scope = FitScope::root()
        .child(AddressRole::OutsideGuideAuto, 1)
        .unwrap();
    let reowned = measure_fixture(
        &descriptor,
        (420.0, None),
        (false, false),
        &theme,
        &other_scope,
    );
    assert!(!Arc::ptr_eq(&original.layout, &reowned.layout));
    let finite = measure_fixture(
        &descriptor,
        (420.0, Some(500.0)),
        (false, false),
        &theme,
        &scope,
    );
    assert!(!Arc::ptr_eq(&original.layout, &finite.layout));
    assert!(finite
        .layout
        .sections
        .iter()
        .all(|section| section.capacity.is_some()));
    let horizontal = measure_fixture(&descriptor, (420.0, None), (true, false), &theme, &scope);
    assert!(!Arc::ptr_eq(&original.layout, &horizontal.layout));
    let wider = measure_fixture(
        &descriptor,
        (420.0 + 1.0 / 1024.0, None),
        (false, false),
        &theme,
        &scope,
    );
    assert!(!Arc::ptr_eq(&original.layout, &wider.layout));

    let miss = || match Legend::measure_assigned(
        Arc::clone(&descriptor),
        (1.0, None),
        false,
        false,
        &theme,
        &mut false,
        &scope,
    )
    .unwrap()
    {
        FitOutcome::Miss(failure) => failure,
        FitOutcome::Measured(_) => panic!("outer insets cannot fit a one-pixel assignment"),
    };
    let first = miss();
    assert_eq!(first, miss());
    assert_eq!(first.reason.address, scope.address);
    assert_color_measurement_cache(&theme);
    assert_alpha_measurement_cache(&theme);
}

fn assert_alpha_measurement_cache(theme: &ResolvedTheme) {
    let prepare = |different: bool, source_row| {
        let values = crate::alpha::AlphaValues::continuous(
            crate::NumericValues::new("quality", [0.25, 0.375, 0.75]),
            (0.0, 1.0),
            |value| {
                if different && value.to_bits() == 0.375_f64.to_bits() {
                    99
                } else {
                    0
                }
            },
        )
        .unwrap()
        .select([0])
        .unwrap();
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0]),
            crate::NumericValues::new("y", [1.0]),
        )
        .source_rows(crate::SourceRows::new([source_row]).unwrap())
        .with_alpha(
            crate::alpha::AlphaTarget::Point,
            crate::alpha::AlphaEncoding::observations(values),
        )
        .erase();
        PreparedGuide::prepare_for_test(&plot, theme, LegendComposition::Combined)
            .unwrap()
            .unwrap()
    };
    let descriptor = prepare(false, 100);
    assert_eq!(descriptor.sections[0].entries().len(), 5);
    let scope = FitScope::root()
        .child(AddressRole::OutsideGuideAuto, 0)
        .unwrap();
    let original = measure_fixture(&descriptor, (420.0, None), (false, false), theme, &scope);
    let equal = measure_fixture(
        &prepare(false, 100),
        (420.0, None),
        (false, false),
        theme,
        &scope,
    );
    assert!(Arc::ptr_eq(&original.layout, &equal.layout));
    for changed in [prepare(true, 100), prepare(false, 200)] {
        let changed = measure_fixture(&changed, (420.0, None), (false, false), theme, &scope);
        assert!(!Arc::ptr_eq(&original.layout, &changed.layout));
        assert_eq!(
            (original.layout.width, original.layout.height),
            (changed.layout.width, changed.layout.height)
        );
        assert_eq!(frozen_svg(&original, theme), frozen_svg(&changed, theme));
    }
    let uncached = fit::measured(measure_uncached(
        descriptor,
        (420.0, None),
        false,
        false,
        theme,
        &mut false,
        &scope,
    ))
    .unwrap();
    assert_eq!(frozen_svg(&original, theme), frozen_svg(&uncached, theme));
    let calls = original.select(20.0, 500.0).unwrap().calls;
    assert_eq!(
        calls
            .iter()
            .filter_map(
                |call| if let super::super::LegendCall::Point(_, point) = call {
                    Some(point.color.get_alpha())
                } else {
                    None
                }
            )
            .collect::<Vec<_>>(),
        [0; 5]
    );
}

fn assert_color_measurement_cache(theme: &ResolvedTheme) {
    let plot = |field: &str| {
        crate::Plot::beeswarm(
            crate::CategoryValues::new("category", ["A", "A"]).unwrap(),
            crate::NumericValues::new("position", [1.0, 2.0]),
            crate::NumericValues::new(field, [0.0, 1.0]),
        )
        .labels(crate::Labels::raw().alias(field, "Signal"))
        .erase()
    };
    let prepare = |field| {
        PreparedGuide::prepare_for_test(&plot(field), theme, LegendComposition::Combined)
            .unwrap()
            .unwrap()
    };
    let descriptor = prepare("signal");
    let equal = prepare("signal");
    let distinct = prepare("other signal");
    let scope = FitScope::root()
        .child(AddressRole::OutsideGuideRight, 0)
        .unwrap();
    let measured = measure_fixture(&descriptor, (420.0, None), (false, false), theme, &scope);
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    let repeated = measure_fixture(&equal, (420.0, None), (false, false), theme, &scope);
    assert!(!Arc::ptr_eq(&descriptor, &equal));
    assert!(Arc::ptr_eq(&measured.layout, &repeated.layout));
    assert_eq!(
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        shapes
    );
    let distinct = measure_fixture(&distinct, (420.0, None), (false, false), theme, &scope);
    assert!(!Arc::ptr_eq(&measured.layout, &distinct.layout));
    assert_eq!(
        (measured.layout.width, measured.layout.height),
        (distinct.layout.width, distinct.layout.height)
    );
    assert_eq!(frozen_svg(&measured, theme), frozen_svg(&distinct, theme));
    let uncached = fit::measured(measure_uncached(
        descriptor,
        (420.0, None),
        false,
        false,
        theme,
        &mut false,
        &scope,
    ))
    .unwrap();
    assert!(!Arc::ptr_eq(&measured.layout, &uncached.layout));
    assert_eq!(frozen_svg(&measured, theme), frozen_svg(&uncached, theme));
}
