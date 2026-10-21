use super::*;
use crate::alpha::{AlphaEncoding, AlphaValues};
use crate::{BarInput, CategoryValues, NumericValues};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one aggregate uncertainty contract follows actual facet/category/series units through both orientations and optional intervals"
)]
fn aggregate_uncertainty_alpha_uses_actual_category_series_and_facet_keys_before_fonts() {
    for horizontal in [false, true] {
        let category =
            CategoryValues::new("category", ["B", "B", "A", "A", "B", "B", "A", "A"]).unwrap();
        let values = NumericValues::new("value", [1.0, 3.0, 2.0, 4.0, 5.0, 7.0, 6.0, 8.0]);
        let input = BarInput::samples(values).error_bar(crate::ErrorBar::standard_deviation());
        let series =
            CategoryValues::new("cohort", ["u", "u", "v", "v", "u", "u", "v", "v"]).unwrap();
        let facet = crate::FacetSpec::column(
            CategoryValues::new("panel", ["L", "L", "L", "L", "R", "R", "R", "R"]).unwrap(),
        );
        let source = if horizontal {
            Plot::horizontal_bar(category, input)
                .series(series)
                .facet(facet)
                .erase()
        } else {
            Plot::bar(category, input)
                .series(series)
                .facet(facet)
                .erase()
        }
        .with_alpha(
            AlphaTarget::Line,
            AlphaEncoding::keyed(
                [
                    CategoryValues::new("panel", ["R", "L", "R", "L", "unused"]).unwrap(),
                    CategoryValues::new("category", ["A", "A", "B", "B", "\u{10ffff}"]).unwrap(),
                    CategoryValues::new("cohort", ["v", "v", "u", "u", "unused"]).unwrap(),
                ],
                AlphaValues::new("uncertainty confidence", [230, 64, 0, 128, 99]).unwrap(),
            )
            .unwrap(),
        )
        .with_alpha(
            AlphaTarget::Fill,
            AlphaEncoding::keyed(
                [
                    CategoryValues::new("category", ["A", "A", "B", "B", "\u{10ffff}"]).unwrap(),
                    CategoryValues::new("cohort", ["v", "v", "u", "u", "unused"]).unwrap(),
                    CategoryValues::new("panel", ["R", "L", "R", "L", "unused"]).unwrap(),
                ],
                AlphaValues::new("body confidence", [0, 230, 64, 0, 99]).unwrap(),
            )
            .unwrap(),
        );

        let mut theme = crate::Theme::publication().resolve().unwrap();
        let interval = theme
            .marks
            .strokes()
            .summaries()
            .uncertainty()
            .clone()
            .with_color(crate::Color::rgba(17, 41, 67, 0))
            .unwrap()
            .with_width(40.0)
            .unwrap();
        let summaries = theme
            .marks
            .strokes()
            .summaries()
            .clone()
            .with_uncertainty(interval)
            .unwrap();
        let strokes = theme
            .marks
            .strokes()
            .clone()
            .with_summaries(summaries)
            .unwrap();
        theme.marks = theme.marks.clone().with_strokes(strokes).unwrap();
        let body = theme
            .marks
            .fills()
            .bodies()
            .bar_body()
            .clone()
            .with_fill_alpha(0)
            .unwrap()
            .with_outline(Some(crate::DataOutlineTheme::mapped(2.375, 61).unwrap()))
            .unwrap();
        let bodies = theme
            .marks
            .fills()
            .bodies()
            .clone()
            .with_bar_body(body)
            .unwrap();
        let fills = theme.marks.fills().clone().with_bodies(bodies).unwrap();
        theme.marks = theme.marks.clone().with_fills(fills).unwrap();
        let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
        let panels = &chart.prepared_bars.as_ref().unwrap().panels;
        assert_eq!(panels.len(), 2);
        for ((scope, prepared), (expected, ends, fills)) in panels.iter().zip([
            ([128, 64], [2.0_f64, 3.0], [0, 230]),
            ([0, 230], [6.0, 7.0], [64, 0]),
        ]) {
            assert_eq!(
                prepared
                    .bars
                    .iter()
                    .map(|bar| bar.line_alpha.unwrap().alpha)
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(
                prepared
                    .bars
                    .iter()
                    .map(|bar| bar.fill_alpha.unwrap().alpha)
                    .collect::<Vec<_>>(),
                fills
            );
            assert_eq!(
                prepared
                    .bars
                    .iter()
                    .map(|bar| bar.end.to_bits())
                    .collect::<Vec<_>>(),
                ends.map(f64::to_bits)
            );
            assert!(prepared.bars.iter().all(|bar| bar.interval.is_some()));
            let chart = chart.clone().with_render_scope(scope.as_deref().unwrap());
            let minimum = intrinsic_body_minimum(&chart, &theme).unwrap();
            assert_eq!(
                (if horizontal { minimum.0 } else { minimum.1 }).to_bits(),
                40.0_f64.to_bits()
            );
            let reads = crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get);
            let selected = chart
                .measure_selected_interior(crate::spec::PlotRenderContext::new(
                    chart.data(),
                    crate::layout::PanelBox::new(20.0, 1000.0, 20.0, 800.0),
                    &theme,
                ))
                .unwrap();
            assert_eq!(
                crate::data::SOURCE_COLUMN_READS.with(std::cell::Cell::get),
                reads
            );
            let crate::layout::interior::MarkPayload::Bars(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("Bar payload")
            };
            assert_eq!(
                payload
                    .bars
                    .iter()
                    .map(|bar| bar.interval_style.color)
                    .collect::<Vec<_>>(),
                expected.map(|alpha| crate::Color::rgba(17, 41, 67, alpha))
            );
            assert_eq!(
                payload
                    .bars
                    .iter()
                    .map(|bar| bar.fill.color.alpha())
                    .collect::<Vec<_>>(),
                fills
            );
            assert!(payload.bars.iter().all(|bar| bar.fill.stroke.alpha() == 61));
        }
        let calls = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
        let incomplete = source.with_alpha(
            AlphaTarget::Line,
            AlphaEncoding::keyed(
                [
                    CategoryValues::new("category", ["B"]).unwrap(),
                    CategoryValues::new("cohort", ["u"]).unwrap(),
                    CategoryValues::new("panel", ["L"]).unwrap(),
                ],
                AlphaValues::new("uncertainty confidence", [0]).unwrap(),
            )
            .unwrap(),
        );
        assert!(matches!(
            incomplete.preflight().unwrap_err(),
            Error::Data(_)
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            calls
        );
    }

    let source = Plot::bar(
        CategoryValues::new("category", ["one"]).unwrap(),
        BarInput::estimates(NumericValues::new("value", [2.0])),
    )
    .erase();
    let absent = source.clone().with_alpha(
        AlphaTarget::Line,
        AlphaEncoding::keyed(
            [CategoryValues::new("category", std::iter::empty::<&str>()).unwrap()],
            AlphaValues::new("uncertainty confidence", []).unwrap(),
        )
        .unwrap(),
    );
    let prepared = prepare(&absent, absent.data(), BarMode::Single).unwrap();
    assert!(prepared.bars[0].interval.is_none());
    assert!(prepared.bars[0].line_alpha.is_none());
    let invalid = source.with_alpha(
        AlphaTarget::Line,
        AlphaEncoding::observations(AlphaValues::new("uncertainty confidence", [0]).unwrap()),
    );
    assert!(matches!(
        prepare(&invalid, invalid.data(), BarMode::Single).unwrap_err(),
        Error::Data(_)
    ));
}

#[test]
fn stacked_body_alpha_preserves_signed_accumulation_zero_segments_and_independent_outlines() {
    for horizontal in [false, true] {
        let category = CategoryValues::new("category", ["B", "A", "B", "A", "B", "A"]).unwrap();
        let values = NumericValues::new("value", [1.0, 2.0, -3.0, 4.0, 3.0, -2.0]);
        let series = CategoryValues::new("cohort", ["u", "u", "v", "v", "v", "w"]).unwrap();
        let source = if horizontal {
            Plot::stacked_horizontal_bar(category, values, series).erase()
        } else {
            Plot::stacked_bar(category, values, series).erase()
        }
        .with_alpha(
            AlphaTarget::Fill,
            AlphaEncoding::keyed(
                [
                    CategoryValues::new("cohort", ["w", "v", "v", "u", "u"]).unwrap(),
                    CategoryValues::new("category", ["A", "A", "B", "A", "B"]).unwrap(),
                ],
                AlphaValues::new("body confidence", [17, 64, 0, 230, 128]).unwrap(),
            )
            .unwrap(),
        );
        let theme = crate::Theme::publication().resolve().unwrap();
        let chart = crate::plots::shared_x::prepare_panel_for_test(&source, &theme).unwrap();
        let prepared = retained(&chart).unwrap();
        assert_eq!(prepared.primary_mark_count(), 5);
        assert_eq!(
            prepared
                .bars
                .iter()
                .map(|bar| (bar.start.to_bits(), bar.end.to_bits(), bar.nonzero))
                .collect::<Vec<_>>(),
            [
                (0.0_f64.to_bits(), 1.0_f64.to_bits(), true),
                (0.0_f64.to_bits(), 2.0_f64.to_bits(), true),
                (1.0_f64.to_bits(), 1.0_f64.to_bits(), false),
                (2.0_f64.to_bits(), 6.0_f64.to_bits(), true),
                (0.0_f64.to_bits(), (-2.0_f64).to_bits(), true),
            ]
        );
        let selected = chart
            .measure_selected_interior(crate::spec::PlotRenderContext::new(
                chart.data(),
                crate::layout::PanelBox::new(20.0, 1000.0, 20.0, 800.0),
                &theme,
            ))
            .unwrap();
        let crate::layout::interior::MarkPayload::Bars(payload) =
            &selected.marks.as_ref().unwrap().payload
        else {
            panic!("stacked Bar payload")
        };
        assert_eq!(
            payload
                .bars
                .iter()
                .map(|bar| bar.fill.color.alpha())
                .collect::<Vec<_>>(),
            [128, 230, 0, 64, 17]
        );
        assert!(payload.bars.iter().all(|bar| bar.interval.is_none()));
        assert!(payload.bars.iter().all(|bar| bar.fill.stroke.alpha()
            == theme
                .marks
                .fills()
                .bodies()
                .bar_body()
                .backend_style((0, 0, 0))
                .stroke
                .alpha()));
    }
}
