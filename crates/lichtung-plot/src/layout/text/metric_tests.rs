use super::*;
use crate::render::backend::{Backend, PlannedTextPlacement, Point2D, TextAnchor};
use crate::render::svg_backend::SvgBackend;
use canonical::MetricSource;

/// Change only generated fonts, through the existing retained-font test entry.
pub(crate) fn font(
    family: &str,
    coverage: &[char],
    typo: Option<(i16, i16)>,
    hhea: (i16, i16),
) -> Vec<u8> {
    let generated = PreparedFontEnvironment::synthetic_faces(&[(family, coverage)]);
    let mut database = fontdb::Database::new();
    generated.populate_database(&mut database).unwrap();
    let id = database.faces().next().unwrap().id;
    database
        .with_face_data(id, |bytes, index| {
            let raw = ttf_parser::RawFace::parse(bytes, index).unwrap();
            let tables = raw
                .table_records
                .into_iter()
                .filter_map(|record| {
                    let tag = record.tag.0.to_be_bytes();
                    let mut data = raw.table(record.tag).unwrap().to_vec();
                    let set = |data: &mut [u8], offset, value: i16| {
                        data[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
                    };
                    match &tag {
                        b"OS/2" => {
                            let (a, d) = typo?;
                            set(&mut data, 68, a);
                            set(&mut data, 70, d);
                        }
                        b"hhea" => {
                            set(&mut data, 4, hhea.0);
                            set(&mut data, 6, hhea.1);
                        }
                        _ => {}
                    }
                    Some((tag, data))
                })
                .collect();
            super::shaping_tests::sfnt(tables)
        })
        .unwrap()
}

fn plan(
    environment: &PreparedFontEnvironment,
    source: &str,
    names: &[&str],
) -> Result<ResolvedTextPlan, TextPlanError> {
    let stack = canonical::named_stack(names);
    let mut input = super::tests::input(source, i64::MAX, PhysicalTextAlignment::Start);
    input.stack = &stack;
    environment.plan_text(input)
}

fn scene(
    environment: &PreparedFontEnvironment,
    plan: &ResolvedTextPlan,
) -> crate::scene::RenderedScene {
    let mut backend = SvgBackend::in_memory_with_fonts(160, 120, environment);
    backend.begin(160, 120).unwrap();
    backend
        .draw_planned_text(
            PlannedTextPlacement::baseline(Point2D::new(20.0, 50.0), TextAnchor::Start, 0.0),
            plan,
        )
        .unwrap();
    backend.end().unwrap();
    crate::scene::RenderedScene::new(160, 120, backend.into_output()).unwrap()
}

fn alter(bytes: &[u8], mut edit: impl FnMut([u8; 4], &mut Vec<u8>)) -> Vec<u8> {
    let raw = ttf_parser::RawFace::parse(bytes, 0).unwrap();
    super::shaping_tests::sfnt(
        raw.table_records
            .into_iter()
            .map(|record| {
                let tag = record.tag.0.to_be_bytes();
                let mut data = raw.table(record.tag).unwrap().to_vec();
                edit(tag, &mut data);
                (tag, data)
            })
            .collect(),
    )
}

fn check_output(environment: &PreparedFontEnvironment, measured: &ResolvedTextPlan) {
    let before = canonical::planning_counts();
    let scene = scene(environment, measured);
    let outlined = scene.svg_with_text_as_paths().unwrap();
    let png = scene
        .png_bytes(&crate::PngOptions {
            sizing: crate::PngSizing::Source,
            ..crate::PngOptions::default()
        })
        .unwrap();
    let actual = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
    // The same frozen paths, not an independently shaped text scene, must be
    // exactly what PNG rasterizes.
    let tree = resvg::usvg::Tree::from_data(&outlined, &resvg::usvg::Options::default()).unwrap();
    let mut expected = resvg::tiny_skia::Pixmap::new(160, 120).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut expected.as_mut(),
    );
    assert!(
        actual.data() == expected.data(),
        "PNG pixels must match the frozen outlined scene"
    );
    assert_eq!(canonical::planning_counts(), before);
}

#[test]
fn double_zero_nominal_metrics_can_render_from_actual_ink() {
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![font(
        "Zero",
        &[' ', 'A'],
        Some((0, 0)),
        (0, 0),
    )]);
    let measured = plan(&environment, "A", &["Zero"]).unwrap();
    let run = &measured.lines[0].runs[0];
    assert_eq!(run.metric_source, MetricSource::InkOnly);
    assert_eq!((run.ascender_quanta, run.descender_quanta), (0, 0));
    assert_eq!(measured.height_quanta, 7 * 1024);
    assert_eq!(measured.first_baseline_quanta, Some(7 * 1024));
    let scene = scene(&environment, &measured);
    let outlined = scene.svg_with_text_as_paths().unwrap();
    assert!(!outlined.windows(5).any(|bytes| bytes == b"<text"));
    assert!(outlined.windows(5).any(|bytes| bytes == b"<path"));
    let png = scene.png_bytes(&crate::PngOptions::default()).unwrap();
    let pixels = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
    assert!(pixels.pixels().iter().any(|pixel| pixel.alpha() != 0));
    check_output(&environment, &measured);
    for source in [" A ", "A\nA", "A\u{200d}", "A\n", "A\n\nA"] {
        let measured = plan(&environment, source, &["Zero"]).unwrap();
        assert!(measured.height_quanta > 0);
        if source == "A\n\nA" {
            assert_eq!(measured.height_quanta, 31 * 1024);
            assert_eq!(measured.first_baseline_quanta, Some(7 * 1024));
            assert_eq!(measured.last_baseline_quanta, Some(31 * 1024));
        }
        check_output(&environment, &measured);
    }
    for source in ["", " ", "\n", "\u{200d}"] {
        assert!(
            matches!(
                plan(&environment, source, &["Zero"]),
                Err(TextPlanError::NoTextHeight { .. })
            ),
            "{source:?}"
        );
    }
}

#[test]
fn frozen_metric_source_mismatch_is_an_output_error_not_font_fallback() {
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Frozen", &['A'], Some((800, 0)), (900, -300)),
        font("Fallback", &['A'], Some((800, -200)), (900, -300)),
    ]);
    let mut measured = plan(&environment, "A", &["Frozen", "Fallback"]).unwrap();
    measured.payload_mut().lines[0].runs[0].metric_source = MetricSource::Hhea;
    let scene = scene(&environment, &measured);
    let error = scene.svg_with_text_as_paths().unwrap_err();
    assert!(error.to_string().contains("metrics do not match"));
    assert!(scene.png_bytes(&crate::PngOptions::default()).is_err());
}

#[test]
fn zero_height_error_identifies_used_faces_not_unused_primary_or_fallback() {
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Primary", &['X'], Some((0, 0)), (0, 0)),
        font("Used", &[' '], Some((0, 0)), (0, 0)),
        font("Unused", &[' '], Some((800, -200)), (900, -300)),
    ]);
    let stack = canonical::named_stack(&["Primary", "Used", "Unused"]);
    let error = plan(&environment, " ", &["Primary", "Used", "Unused"]).unwrap_err();
    let pair = environment
        .prepared_pair(&stack, RequestedWeight::Normal)
        .unwrap()
        .unwrap();
    let TextPlanError::NoTextHeight { faces, .. } = error else {
        panic!("selected blank is not replaced by another font")
    };
    assert_eq!(faces, [pair.provenance.candidates[1]]);
    let error = plan(&environment, "", &["Primary", "Used", "Unused"]).unwrap_err();
    let TextPlanError::NoTextHeight { faces, .. } = error else {
        panic!("empty block uses primary metrics")
    };
    assert_eq!(faces, [pair.primary]);
}

#[test]
fn metric_pairs_preserve_zero_endpoints_and_use_one_same_face_source() {
    for (typo, hhea, expected, values) in [
        (
            Some((800, 0)),
            (900, -300),
            MetricSource::Typographic,
            (800, 0),
        ),
        (
            Some((0, -200)),
            (900, -300),
            MetricSource::Typographic,
            (0, -200),
        ),
        (Some((0, 0)), (900, -300), MetricSource::Hhea, (900, -300)),
        (
            Some((100, 200)),
            (900, -300),
            MetricSource::Hhea,
            (900, -300),
        ),
        (None, (900, -300), MetricSource::Hhea, (900, -300)),
    ] {
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![font(
            "Metric",
            &[' ', 'A'],
            typo,
            hhea,
        )]);
        let measured = plan(&environment, " ", &["Metric"]).unwrap();
        let run = &measured.lines[0].runs[0];
        assert_eq!(run.metric_source, expected);
        assert_eq!((run.design_ascender, run.design_descender), values);
        assert_eq!(
            measured.first_baseline_quanta,
            Some(i64::from(values.0.max(0)) * 1024 / 100)
        );
        check_output(&environment, &measured); // includes baseline=0
    }
    let generated = font("Metric", &['A'], Some((800, -200)), (900, -300));
    for (version, flags, truncate) in [
        (4_u16, 0_u16, false),
        (4, 0x80, false),
        (0, 0x80, false),
        (0, 0, true),
    ] {
        let bytes = alter(&generated, |tag, data| {
            if tag == *b"OS/2" {
                if version == 4 {
                    data.resize(96, 0);
                }
                data[..2].copy_from_slice(&version.to_be_bytes());
                data[62..64].copy_from_slice(&flags.to_be_bytes());
                if truncate {
                    data.truncate(68);
                }
            }
        });
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
        let measured = plan(&environment, "A", &["Metric"]).unwrap();
        let run = &measured.lines[0].runs[0];
        assert_eq!(
            run.metric_source,
            if truncate {
                MetricSource::Hhea
            } else {
                MetricSource::Typographic
            }
        );
        assert_eq!(run.design_ascender, if truncate { 900 } else { 800 });
    }
}

#[test]
fn metric_denominators_extremes_and_outward_quantization_remain_checked() {
    let generated = font(
        "Metric",
        &[' ', 'A'],
        Some((i16::MAX, i16::MIN)),
        (800, -200),
    );
    for em in [16_u16, 1001, 16384] {
        let bytes = alter(&generated, |tag, data| {
            if tag == *b"head" {
                data[18..20].copy_from_slice(&em.to_be_bytes());
            }
        });
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
        let stack = canonical::named_stack(&["Metric"]);
        for size in [f64::from_bits(1), 0.0001, 10.0] {
            let mut input = super::tests::input(" ", i64::MAX, PhysicalTextAlignment::Start);
            input.stack = &stack;
            input.font_size = size;
            let measured = environment.plan_text(input).unwrap();
            assert!(measured.height_quanta > 0 && measured.line_pitch_quanta > 0);
            let run = &measured.lines[0].runs[0];
            assert!(run.ascender_quanta >= 0 && run.descender_quanta <= 0);
        }
        let mut input = super::tests::input("A", i64::MAX, PhysicalTextAlignment::Start);
        input.stack = &stack;
        input.font_size = f64::MAX;
        input.line_height = TextLineHeight::ResolvedQuanta(1);
        let error = environment.plan_text(input).unwrap_err();
        assert!(
            matches!(
                error,
                TextPlanError::Arithmetic(_) | TextPlanError::InvalidLineHeight
            ),
            "{error:?}"
        );
    }
    for em in [0_u16, 15, 16385] {
        let bytes = alter(&generated, |tag, data| {
            if tag == *b"head" {
                data[18..20].copy_from_slice(&em.to_be_bytes());
            }
        });
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
        assert!(matches!(
            plan(&environment, "A", &["Metric"]),
            Err(TextPlanError::Font(FontError::Admission { .. }))
        ));
    }
    for pair in [(800, 200), (-200, -800)] {
        let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![font(
            "Metric",
            &[' '],
            Some(pair),
            (0, 0),
        )]);
        let measured = plan(&environment, " ", &["Metric"]).unwrap();
        assert_eq!(measured.height_quanta, 8 * 1024);
        assert_eq!(
            measured.first_baseline_quanta,
            Some(if pair.0 > 0 { 8 * 1024 } else { 0 })
        );
        check_output(&environment, &measured);
    }
}

#[test]
fn only_actual_fallback_and_blank_lines_require_their_own_metrics() {
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Empty nominal", &[' '], Some((0, 0)), (0, 0)),
        font("Fallback", &[' ', 'A'], Some((800, 0)), (900, -300)),
    ]);
    canonical::FONT_METRIC_QUERIES.with(|queries| queries.borrow_mut().clear());
    let measured = plan(&environment, "A", &["Empty nominal", "Fallback"]).unwrap();
    canonical::FONT_METRIC_QUERIES
        .with(|queries| assert!(queries.borrow().is_empty(), "no eager primary metric query"));
    let primary = measured.pair_provenance.candidates[0];
    assert_eq!(
        measured.pair_provenance.candidates.len(),
        2,
        "do not filter the primary"
    );
    assert_eq!(
        measured.lines[0].runs[0].face,
        measured.pair_provenance.candidates[1]
    );
    check_output(&environment, &measured);
    for source in ["A\n", "A\n\nA"] {
        canonical::FONT_METRIC_QUERIES.with(|queries| queries.borrow_mut().clear());
        let measured = plan(&environment, source, &["Empty nominal", "Fallback"]).unwrap();
        canonical::FONT_METRIC_QUERIES
            .with(|queries| assert_eq!(queries.borrow().as_slice(), &[primary]));
        assert!(measured.height_quanta > 0);
        check_output(&environment, &measured);
    }
    for source in ["", " ", "\n"] {
        assert!(
            matches!(
                plan(&environment, source, &["Empty nominal", "Fallback"]),
                Err(TextPlanError::NoTextHeight { .. })
            ),
            "{source:?}"
        );
    }
    // The primary owns real blank lines; zero ascent is valid and baseline 0
    // remains accepted by placement and the concrete backend.
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![
        font("Blank", &[' '], Some((0, -200)), (900, -300)),
        font("Fallback", &['A'], Some((800, 0)), (900, -300)),
    ]);
    let measured = plan(&environment, "A\n\n \n", &["Blank", "Fallback"]).unwrap();
    assert_eq!(measured.lines.len(), 4);
    assert_eq!(measured.first_baseline_quanta, Some(8 * 1024));
    assert_eq!(measured.last_baseline_quanta, Some(44 * 1024));
    assert_eq!(measured.height_quanta, 46 * 1024);
    assert_eq!(measured.line_pitch_quanta, 12 * 1024);
    for (index, line) in measured.lines.iter().enumerate() {
        assert_eq!(
            line.translation_y_quanta,
            8 * 1024 + i64::try_from(index).unwrap() * 12 * 1024
        );
    }
    for source in ["", " ", "\n", "\u{200d}"] {
        let blank = plan(&environment, source, &["Blank", "Fallback"]).unwrap();
        assert_eq!(blank.first_baseline_quanta, Some(0));
        assert!(blank.height_quanta > 0);
        check_output(&environment, &blank);
    }
    check_output(&environment, &measured);
}

#[test]
fn real_positioned_ink_expands_zero_nominal_metrics_before_outline_and_png() {
    let bytes = super::shaping_tests::contextual_fonts().remove(0);
    let bytes = alter(&bytes, |tag, data| match &tag {
        b"OS/2" => data[68..72].fill(0),
        b"hhea" => data[4..8].fill(0),
        // Give all contextual glyphs the generated rectangle's real outline.
        b"glyf" => *data = data.repeat(8),
        b"loca" => {
            *data = [0_u16, 0, 17, 34, 51, 68, 85, 102, 119, 136]
                .into_iter()
                .flat_map(u16::to_be_bytes)
                .collect();
        }
        _ => {}
    });
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
    let measured = plan(&environment, "A\u{301}", &["First"]).unwrap();
    let glyph = &measured.lines[0].runs[0].glyphs[0];
    assert_eq!(
        (glyph.glyph_id, glyph.design_x_offset, glyph.design_y_offset),
        (4, -17, 23)
    );
    let ink = measured.lines[0].ink_quanta.unwrap();
    assert_eq!(measured.first_baseline_quanta, Some(ink.top));
    assert_eq!(ink.top, (723 * 1024 + 99) / 100);
    check_output(&environment, &measured);
    // Below-baseline ink must likewise expand a zero nominal descent.
    let bytes = alter(
        &font("Below", &['A'], Some((800, 0)), (900, -300)),
        |tag, data| {
            if tag == *b"glyf" {
                data[4..6].copy_from_slice(&(-200_i16).to_be_bytes());
                data[8..10].copy_from_slice(&500_i16.to_be_bytes());
                data[26..28].copy_from_slice(&(-200_i16).to_be_bytes());
            } else if tag == *b"head" {
                data[38..40].copy_from_slice(&(-200_i16).to_be_bytes());
                data[42..44].copy_from_slice(&500_i16.to_be_bytes());
            }
        },
    );
    let environment = PreparedFontEnvironment::synthetic_shaping_fonts(vec![bytes]);
    let measured = plan(&environment, "A", &["Below"]).unwrap();
    assert_eq!(measured.height_quanta, 10 * 1024);
    assert_eq!(measured.lines[0].ink_quanta.unwrap().bottom, -2 * 1024);
    check_output(&environment, &measured);
}
