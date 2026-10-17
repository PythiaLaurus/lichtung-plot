use super::*;

#[test]
fn shared_numeric_guides_resolve_final_sets_and_reject_equal_text_policy_conflicts_before_fonts() {
    use crate::numeric::{
        decimal::{NegativeZero, NumericNotation, TrailingZeros},
        format::{NumericFormat, NumericPrecision},
    };
    let format = NumericFormat::new(NumericNotation::Decimal, NumericPrecision::Fractional(0))
        .unwrap()
        .with_suffix(" u")
        .unwrap();
    let plot = |values, format| {
        Plot::heatmap(
            crate::CategoryValues::new("x", ["a", "b"]).unwrap(),
            crate::CategoryValues::new("y", ["row", "row"]).unwrap(),
            crate::NumericValues::new("value", values),
        )
        .color_label_format(format)
    };
    let first = plot([1.01, 1.02], format.clone());
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    assert!(first
        .preflight()
        .unwrap_err()
        .to_string()
        .contains("not unique"));
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    let canvas = |second| {
        Canvas::grid(1, 2)
            .unwrap()
            .size(1000, 500)
            .legend_policy(CanvasLegendPolicy::Shared)
            .panel(0, 0, first.clone())
            .unwrap()
            .panel(0, 1, plot([3.1, 3.2], second))
            .unwrap()
    };
    let root = canvas(format.clone()).preflight_plan().unwrap();
    for source in ["1 u", "3 u"] {
        assert_eq!(
            root.plan()
                .retained_text()
                .iter()
                .filter(|(_, text)| text.source == source)
                .count(),
            1
        );
    }
    Canvas::grid(1, 1)
        .unwrap()
        .size(600, 400)
        .legend_policy(CanvasLegendPolicy::Hidden)
        .panel(
            0,
            0,
            plot(
                [1.01, 1.02],
                format.clone().with_suffix("\u{10ffff}").unwrap(),
            ),
        )
        .unwrap()
        .preflight_plan()
        .unwrap();
    for different in [
        format
            .clone()
            .with_notation(NumericNotation::Engineering)
            .unwrap(),
        format
            .clone()
            .with_precision(NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
            .unwrap(),
        format.clone().with_decimal_scale(2).unwrap(),
        format.clone().with_prefix("unit ").unwrap(),
        format.clone().with_suffix(" other").unwrap(),
        format
            .clone()
            .with_negative_zero(NegativeZero::Normalize)
            .unwrap(),
        format.with_trailing_zeros(TrailingZeros::Trim).unwrap(),
    ] {
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        let error = canvas(different)
            .preflight_plan()
            .err()
            .expect("incompatible complete policy");
        assert!(error.to_string().contains("numeric format"));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one composed scene verifies role-local precedence, construction order and frozen replay"
)]
fn heading_instructions_are_role_local_order_independent_and_frozen() {
    use crate::layout::PhysicalTextAlignment;
    use crate::TitleAlignment::{Center, End, Start};
    let base = Theme::publication();
    let composition = base
        .composition()
        .clone()
        .with_root_title_alignment(Start)
        .unwrap()
        .with_root_subtitle_alignment(None)
        .unwrap()
        .with_root_caption_alignment(End)
        .unwrap();
    let root_theme = base
        .customize()
        .with_composition(composition)
        .build()
        .unwrap();
    let composition = base
        .composition()
        .clone()
        .with_root_title_alignment(End)
        .unwrap()
        .with_root_subtitle_alignment(Some(Start))
        .unwrap()
        .with_root_caption_alignment(Center)
        .unwrap();
    let local_theme = base
        .customize()
        .with_composition(composition)
        .build()
        .unwrap();
    let scatter = |name: &str| {
        Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 1.0]),
        )
        .title(format!("{name} title\nx"))
        .subtitle(format!("{name} subtitle\ny"))
        .caption(format!("{name} caption\nz"))
    };
    let mut canonical = None;
    for reversed in [false, true] {
        let explicit = if reversed {
            scatter("Explicit")
                .title_alignment(Start)
                .title_alignment(End)
                .subtitle_alignment(End)
                .subtitle_alignment(Start)
                .theme(local_theme.clone())
        } else {
            scatter("Explicit")
                .theme(local_theme.clone())
                .subtitle_alignment(Start)
                .title_alignment(End)
        };
        let canvas = Canvas::grid(1, 3)
            .unwrap()
            .size(1201, 701)
            .title("Canvas title\nx")
            .subtitle("Canvas subtitle\ny")
            .caption("Canvas caption\nz")
            .panel(0, 0, scatter("Inherited"))
            .unwrap()
            .panel(0, 1, scatter("Replaced").theme(local_theme.clone()))
            .unwrap()
            .panel(0, 2, explicit)
            .unwrap();
        let canvas = if reversed {
            canvas
                .panel_title_alignment(End)
                .panel_title_alignment(Center)
                .panel_subtitle_alignment(Start)
                .panel_subtitle_alignment(End)
                .theme(root_theme.clone())
        } else {
            canvas
                .theme(root_theme.clone())
                .panel_subtitle_alignment(End)
                .panel_title_alignment(Center)
        };
        let root = canvas.preflight_plan().unwrap();
        let retained = root.plan().retained_text();
        for (source, expected) in [
            ("Canvas title\nx", PhysicalTextAlignment::Start),
            ("Canvas subtitle\ny", PhysicalTextAlignment::Start),
            ("Canvas caption\nz", PhysicalTextAlignment::End),
            ("Inherited title\nx", PhysicalTextAlignment::Center),
            ("Inherited subtitle\ny", PhysicalTextAlignment::End),
            ("Inherited caption\nz", PhysicalTextAlignment::End),
            ("Replaced title\nx", PhysicalTextAlignment::Center),
            ("Replaced subtitle\ny", PhysicalTextAlignment::End),
            ("Replaced caption\nz", PhysicalTextAlignment::Center),
            ("Explicit title\nx", PhysicalTextAlignment::End),
            ("Explicit subtitle\ny", PhysicalTextAlignment::Start),
            ("Explicit caption\nz", PhysicalTextAlignment::Center),
        ] {
            let matching = retained
                .iter()
                .filter(|(_, text)| text.source == source)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1, "{source}");
            assert_eq!(matching[0].1.style.alignment, expected, "{source}");
            assert_eq!(matching[0].1.lines.len(), 2, "{source}");
        }
        let counters = capture_counters();
        let mut backend = root.plan().take_svg_backend().unwrap();
        Canvas::render_preflighted(&root, &mut backend).unwrap();
        assert_eq!(capture_counters(), counters);
        let svg = backend.into_bytes().unwrap();
        if let Some(before) = &canonical {
            assert_eq!(before, &svg);
        } else {
            canonical = Some(svg);
        }
    }
}

pub(super) fn capture_counters() -> [usize; 6] {
    [
        crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
        crate::layout::Projection::test_projection_calls(),
        crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get),
        crate::plots::shared_x::AXIS_PREPARATIONS.with(std::cell::Cell::get),
        crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(std::cell::Cell::get),
        crate::orchestrator::SELECTED_INTERIOR_MEASUREMENTS.with(std::cell::Cell::get),
    ]
}

/// Parent interaction uses representative coordinate, categorical, grid and
/// distribution children. The caller separately exercises every family as a root.
pub(crate) fn mixed_family_parent_audit(plots: &[(PlotKind, Plot)]) {
    assert_eq!(plots.len(), 25);
    let mut canvas = Canvas::grid(2, 2)
        .unwrap()
        .size(1601, 1201)
        .title("Mixed Canvas families")
        .subtitle("Retained child scopes")
        .caption("Independent root caption")
        .labels(Labels::raw().alias("x", "Shared sample"));
    let mut index = 0;
    for (kind, plot) in plots {
        if *kind == PlotKind::Matrix {
            let mut rejected = Canvas::grid(1, 1).unwrap();
            rejected.panels[0] = Some(plot.clone());
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
            assert!(rejected.preflight_plan().is_err());
            assert_eq!(
                crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
                0
            );
            continue;
        }
        if !matches!(
            kind,
            PlotKind::Scatter | PlotKind::HorizontalBar | PlotKind::Heatmap | PlotKind::Raincloud
        ) {
            continue;
        }
        canvas.panels[index] = Some(plot.clone().title(format!("{kind:?}")));
        index += 1;
    }
    assert_eq!(index, 4);
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS.with(|calls| calls.set(0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| calls.set(0));
    crate::orchestrator::measurements::start();
    let cached = canvas.render_scene().unwrap();
    let work = crate::orchestrator::measurements::finish();
    assert_eq!(work.calls[2], 1);
    assert_eq!(work.calls[3], 1);
    assert_eq!(
        work.capture_counters,
        [0; crate::orchestrator::measurements::Counter::ALL.len()]
    );
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(true));
    let uncached = canvas.render_scene();
    crate::decorations::plan::children::DISABLE_CHILD_REUSE.with(|disabled| disabled.set(false));
    assert_eq!(cached.svg_bytes(), uncached.unwrap().svg_bytes());
    crate::render::geometry::GEOMETRY_BACKEND_CONSTRUCTIONS
        .with(|calls| assert_eq!(calls.get(), 0));
    crate::render::geometry::GEOMETRY_BACKEND_BEGINS.with(|calls| assert_eq!(calls.get(), 0));
}
