use super::*;
use crate::font::canonical::OperationContext;

fn stop(offset: f64, red: u8) -> ColorStop {
    ColorStop::new(
        offset,
        OpaqueColor::rgb(red, red.wrapping_add(3), red.wrapping_add(7)),
    )
    .unwrap()
}

#[test]
fn hidden_shared_plots_validate_the_final_palette_domain_before_fonts() {
    let base = crate::Theme::publication();
    let scales = base
        .color_scales()
        .clone()
        .qualitative(QualitativePalette::fixed([OpaqueColor::rgb(13, 29, 47)]).unwrap())
        .unwrap();
    let theme = base
        .customize()
        .color_scales(scales)
        .font_stack(
            crate::FontStack::new([crate::FontFamily::named(
                "Absent shared palette preflight font",
            )
            .unwrap()])
            .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
    let scatter = |key| {
        crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 1.0]),
        )
        .hue(crate::CategoryValues::new("raw", [key, key]).unwrap())
        .legend(crate::LegendPolicy::Hidden)
    };
    let canvas = crate::Canvas::grid(1, 2)
        .unwrap()
        .theme(theme)
        .panel(0, 0, scatter("A  B"))
        .unwrap()
        .panel(0, 1, scatter("A B"))
        .unwrap()
        .legend_policy(crate::CanvasLegendPolicy::Shared);
    let discoveries = crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get);
    let error = canvas.preflight().unwrap_err();
    assert!(matches!(error, crate::Error::Data(_)), "{error}");
    assert!(
        error.to_string().contains("1 colors for 2 used identities"),
        "{error}"
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        discoveries,
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "independent categorical, accent and native continuous consumers retain their original palette ownership"
)]
fn resolved_palette_edits_do_not_change_accents_and_heatmap_reads_its_actual_owner() {
    use crate::render::backend::Backend;
    let resources = OperationContext::new();
    let mut theme = crate::Theme::publication().resolve().unwrap();
    let original = theme.clone();
    let palette =
        QualitativePalette::fixed([OpaqueColor::rgb(11, 23, 37), OpaqueColor::rgb(41, 59, 73)])
            .unwrap();
    let gradient = SequentialGradient::new([stop(0.0, 83), stop(0.4, 97), stop(1.0, 109)]).unwrap();
    theme.color_scales = theme
        .color_scales
        .clone()
        .qualitative(palette.clone())
        .unwrap()
        .sequential(gradient.clone())
        .unwrap();
    assert_eq!(theme.marks.get_accents(), original.marks.get_accents());
    let keys = ["one", "two"];
    let field = crate::data::category::RawField::new("category").unwrap();
    let mapped = super::super::CategoryColors::resolve(
        Some(&field),
        keys.into_iter(),
        theme.color_scales.get_qualitative(),
        None,
        &resources,
    )
    .unwrap();
    assert_eq!(mapped.colors(), palette.colors());
    assert!(super::super::CategoryColors::resolve(
        Some(&field),
        ["one", "two", "three"].into_iter(),
        theme.color_scales.get_qualitative(),
        None,
        &resources
    )
    .is_err());
    let source = crate::Plot::heatmap(
        crate::CategoryValues::new("column", ["left", "right"]).unwrap(),
        crate::CategoryValues::new("row", ["one", "one"]).unwrap(),
        crate::NumericValues::new("value", [0.0, 1.0]),
    )
    .erase();
    crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
    let (inputs, _) =
        crate::plots::shared_x::prepare_plot(source.description(), &resources, &theme).unwrap();
    let text = crate::text::TextService::default();
    let theme = crate::text::TextRequestContext::new(&theme, &text);
    let Some(crate::orchestrator::registration::PreparedFamily::Heatmap(prepared)) =
        &inputs.panels[0].family
    else {
        panic!("prepared Heatmap")
    };
    let semantics = prepared.guide_semantics(&theme).unwrap();
    assert_eq!(
        semantics.selection.get_sequential_gradient(),
        Some(&gradient)
    );
    assert_eq!(
        semantics.selection.stops().unwrap().as_ptr(),
        gradient.stops().as_ptr()
    );
    assert_eq!(
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
        0
    );
    crate::orchestrator::with_prepared_plot_for_test(
        &source,
        &theme,
        |_| Ok(()),
        |chart, inputs, theme, _operation, round| {
            let measurement = inputs.assignment_factory().assign(
                chart
                    .sizing_request()
                    .assigned_size()
                    .unwrap_or(crate::layout::RootSize::new(1, 1)),
            );
            let factory = inputs
                .plot_assignment_factory(&measurement, &inputs.plots[0].scope.id)
                .unwrap();
            let selected = crate::layout::interior::measure_for_test(
                &factory,
                crate::layout::PixelRect::new(20.0, 640.0, 20.0, 480.0),
                round,
            )
            .unwrap();
            let crate::layout::interior::MarkPayload::Heatmap(payload) =
                &selected.marks.as_ref().unwrap().payload
            else {
                panic!("selected owner Heatmap")
            };
            let mut backend = crate::render::svg_backend::SvgBackend::in_memory_with_fonts(
                700,
                500,
                theme.text.prepared_font_environment().unwrap(),
            );
            backend.begin(700, 500).unwrap();
            payload.draw(&mut backend).unwrap();
            backend.end().unwrap();
            let svg = String::from_utf8(backend.into_bytes().unwrap()).unwrap();
            assert!(svg.contains("fill=\"#53565A\""));
            assert!(svg.contains("fill=\"#6D7074\""));
            assert_eq!(
                original.color_scales.get_qualitative().capacity(),
                PaletteCapacity::Extendable
            );
        },
    )
    .unwrap();
}
