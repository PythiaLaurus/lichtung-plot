use super::*;

fn scatter() -> Plot<lichtung_plot::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [1.0, 2.0, 3.0]),
        NumericValues::new("y", [1.0, 3.0, 2.0]),
    )
}

fn text_line<'a>(svg: &'a str, text: &str) -> &'a str {
    svg.lines()
        .find(|line| line.contains(&format!(">{text}</text>")))
        .expect("text is rendered")
}

#[test]
fn explicit_plot_theme_completely_replaces_only_its_inherited_root() {
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .theme(Theme::nature())
            .panel(
                0,
                0,
                scatter().theme(Theme::science()).title("Explicit Science"),
            )
            .unwrap()
            .panel(0, 1, scatter().title("Inherited Nature"))
            .unwrap(),
        "theme-complete-plot-replacement",
    )
    .unwrap();

    assert_eq!(
        attribute(text_line(&svg, "Explicit Science"), "font-size"),
        "21"
    );
    assert_eq!(
        attribute(text_line(&svg, "Inherited Nature"), "font-size"),
        "19"
    );
    assert!(svg.contains("#FF0000"));
    assert!(svg.contains("#285682"));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "a real composed scene constructs all eight theme components and distinguishes root-owned text roles"
)]
fn mixed_children_keep_distinct_complete_canvas_roles_on_the_root_theme() {
    use lichtung_plot::{
        AccentPalette, Color, FontFamily, FontStack, GenericFontFamily, QualitativePalette,
        TextTheme, TextWeight,
    };
    let base = Theme::nature();
    let profile = base.typography().get_root().get_title().get_stack().clone();
    let role = |family, color, size, weight, leading| {
        TextTheme::new(
            FontStack::new(
                [FontFamily::generic(family)]
                    .into_iter()
                    .chain(profile.families().iter().cloned()),
            )
            .unwrap(),
            color,
            size,
            weight,
            leading,
        )
        .unwrap()
    };
    let title = role(
        GenericFontFamily::Monospace,
        Color::rgb(113, 31, 59),
        25.0,
        TextWeight::Semibold,
        1.4,
    );
    let root = base.typography().get_root().clone().title(title).unwrap();
    let header = role(
        GenericFontFamily::Serif,
        Color::rgb(43, 79, 127),
        17.0,
        TextWeight::Bold,
        1.25,
    );
    let panel_title = role(
        GenericFontFamily::Monospace,
        Color::rgb(97, 53, 131),
        20.0,
        TextWeight::Semibold,
        1.5,
    );
    let body = role(
        GenericFontFamily::SansSerif,
        Color::rgb(29, 83, 47),
        12.0,
        TextWeight::Normal,
        1.5,
    );
    let composition = base
        .typography()
        .get_composition()
        .clone()
        .composition_header(header)
        .unwrap()
        .text_panel_title(panel_title)
        .unwrap()
        .body(body)
        .unwrap();
    let typography = base
        .typography()
        .clone()
        .root(root)
        .unwrap()
        .composition(composition)
        .unwrap();
    let surfaces = base
        .surfaces()
        .clone()
        .canvas_root(Color::rgb(23, 43, 69))
        .unwrap()
        .plot_panel(Color::rgb(242, 225, 192))
        .unwrap();
    let marks = base
        .marks()
        .clone()
        .accents(AccentPalette::new([OpaqueColor::rgb(29, 131, 47)]).unwrap())
        .unwrap();
    let scales = base
        .color_scales()
        .clone()
        .qualitative(QualitativePalette::fixed([OpaqueColor::rgb(233, 129, 41)]).unwrap())
        .unwrap();
    let custom = Theme::new(
        surfaces,
        typography,
        base.axes().clone(),
        base.legend().clone(),
        base.composition().clone(),
        marks,
        base.annotations().clone(),
        scales,
    )
    .unwrap();
    let body_source = "Custom body remains complete while the narrow owning cell wraps these independently styled words across measured lines with the selected role leading and font preferences.";
    let svg = render_canvas(
        Canvas::grid(2, 2)
            .unwrap()
            .theme(custom)
            .panel(0, 0, scatter().theme(Theme::science()))
            .unwrap()
            .panel(0, 1, scatter())
            .unwrap()
            .text_panel(
                1,
                0,
                lichtung_plot::TextPanel::new("Context", body_source).unwrap(),
            )
            .unwrap()
            .title("Custom canvas")
            .column_header(0, 2, "Custom columns")
            .unwrap()
            .row_title(0, "Custom row")
            .unwrap(),
        "theme-canvas-owned-consumers",
    )
    .unwrap();

    for (text, size, paint, family, weight) in [
        ("Custom canvas", "25", "#711F3B", "monospace", "600"),
        ("Custom columns", "17", "#2B4F7F", "serif", "700"),
        ("Custom row", "17", "#2B4F7F", "serif", "700"),
        ("Context", "20", "#613583", "monospace", "600"),
    ] {
        let line = text_line(&svg, text);
        assert_eq!(attribute(line, "font-size"), size);
        assert_eq!(attribute(line, "fill"), paint);
        assert!(attribute(line, "font-family").starts_with(&format!("{family}, Noto Sans, ")));
        assert_eq!(attribute(line, "font-weight"), weight);
    }
    let document = roxmltree::Document::parse(&svg).unwrap();
    let body = document
        .descendants()
        .filter(|node| node.has_tag_name("text") && node.attribute("fill") == Some("#1D532F"))
        .collect::<Vec<_>>();
    assert!(body.len() >= 2);
    for line in &body {
        assert_eq!(line.attribute("font-size"), Some("12"));
        assert!(line
            .attribute("font-family")
            .unwrap()
            .starts_with("sans-serif, Noto Sans, "));
    }
    let rendered = body
        .iter()
        .flat_map(|line| line.text().unwrap().split_whitespace())
        .collect::<Vec<_>>();
    assert_eq!(rendered, body_source.split_whitespace().collect::<Vec<_>>());
    assert!(
        body[1].attribute("y").unwrap().parse::<f64>().unwrap()
            - body[0].attribute("y").unwrap().parse::<f64>().unwrap()
            >= 18.0 - 1.0 / 1024.0
    );
    assert!(svg.contains("fill=\"#172B45\"") && svg.contains("fill=\"#F2E1C0\""));
    assert!(svg.contains("fill=\"#1D832F\"") && svg.contains("fill=\"#FF0000\""));
    assert!(
        !svg.contains("fill=\"#E98129\""),
        "unkeyed marks use accents, not the fixed categorical palette"
    );
}

#[test]
fn facets_use_their_owning_plot_theme_without_a_new_scope() {
    use lichtung_plot::{Color, FontFamily, FontStack, GenericFontFamily, TextWeight};
    let facets =
        || FacetSpec::column(CategoryValues::new("panel", ["first", "first", "second"]).unwrap());
    let omitted = render_svg(scatter().facet(facets()), "theme-facet-default").unwrap();
    let explicit = render_svg(
        scatter().theme(Theme::publication()).facet(facets()),
        "theme-facet-publication",
    )
    .unwrap();
    let base = Theme::nature();
    let profile = base.typography().get_root().get_title().get_stack().clone();
    let stack = FontStack::new(
        [FontFamily::generic(GenericFontFamily::Serif)]
            .into_iter()
            .chain(profile.families().iter().cloned()),
    )
    .unwrap();
    let header = base
        .typography()
        .get_composition()
        .get_facet_header()
        .clone()
        .color(Color::rgb(131, 47, 103))
        .unwrap()
        .font_size(23.0)
        .unwrap()
        .weight(TextWeight::Semibold)
        .unwrap()
        .stack(stack)
        .unwrap();
    let composition = base
        .typography()
        .get_composition()
        .clone()
        .facet_header(header)
        .unwrap();
    let typography = base.typography().clone().composition(composition).unwrap();
    let custom = base.customize().typography(typography).build().unwrap();
    let nature = render_svg(
        scatter().theme(custom).facet(facets()),
        "theme-facet-nature",
    )
    .unwrap();

    assert_eq!(omitted, explicit);
    assert_ne!(nature, omitted);
    assert!(nature.contains("#285682"));
    for source in ["Panel = First", "Panel = Second"] {
        let line = text_line(&nature, source);
        assert_eq!(attribute(line, "font-size"), "23");
        assert_eq!(attribute(line, "fill"), "#832F67");
        assert_eq!(attribute(line, "font-weight"), "600");
        assert!(attribute(line, "font-family").starts_with("serif, Noto Sans, "));
    }
}
