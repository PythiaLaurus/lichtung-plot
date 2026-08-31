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
fn omitted_theme_calls_select_publication_for_plot_and_canvas() {
    let omitted_plot = render_svg(scatter(), "theme-default-plot").unwrap();
    let explicit_plot = render_svg(
        scatter().theme(Theme::Publication),
        "theme-publication-plot",
    )
    .unwrap();
    assert_eq!(omitted_plot, explicit_plot);

    let omitted_canvas = render_canvas(
        Canvas::grid(1, 1).unwrap().panel(0, 0, scatter()).unwrap(),
        "theme-default-canvas",
    )
    .unwrap();
    let explicit_canvas = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .theme(Theme::Publication)
            .panel(0, 0, scatter())
            .unwrap(),
        "theme-publication-canvas",
    )
    .unwrap();
    assert_eq!(omitted_canvas, explicit_canvas);
}

#[test]
fn unthemed_canvas_child_inherits_the_root_theme() {
    let inherited = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .theme(Theme::Nature)
            .panel(0, 0, scatter())
            .unwrap(),
        "theme-inherited-nature",
    )
    .unwrap();
    let explicit = render_canvas(
        Canvas::grid(1, 1)
            .unwrap()
            .theme(Theme::Nature)
            .panel(0, 0, scatter().theme(Theme::Nature))
            .unwrap(),
        "theme-explicit-nature",
    )
    .unwrap();
    let publication = render_canvas(
        Canvas::grid(1, 1).unwrap().panel(0, 0, scatter()).unwrap(),
        "theme-inherited-publication",
    )
    .unwrap();

    assert_eq!(inherited, explicit);
    assert_ne!(inherited, publication);
    assert!(inherited.contains("#285682"));
}

#[test]
fn explicit_plot_theme_completely_replaces_only_its_inherited_root() {
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .theme(Theme::Nature)
            .panel(
                0,
                0,
                scatter().theme(Theme::Science).title("Explicit Science"),
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
fn mixed_children_keep_t4_canvas_consumers_on_the_root_theme() {
    let svg = render_canvas(
        Canvas::grid(2, 2)
            .unwrap()
            .theme(Theme::Nature)
            .panel(0, 0, scatter().theme(Theme::Science))
            .unwrap()
            .panel(0, 1, scatter())
            .unwrap()
            .text_panel(
                1,
                0,
                lichtung_plot::TextPanel::new("Context", "Nature body").unwrap(),
            )
            .unwrap()
            .title("Nature canvas")
            .column_header(0, 2, "Nature columns")
            .unwrap()
            .row_title(0, "Nature row")
            .unwrap(),
        "theme-canvas-owned-consumers",
    )
    .unwrap();

    for text in ["Nature canvas", "Nature columns", "Nature row", "Context"] {
        let line = text_line(&svg, text);
        assert_eq!(attribute(line, "font-size"), "19");
        assert_eq!(attribute(line, "fill"), "#282828");
    }
    let body = text_line(&svg, "Nature body");
    assert_eq!(attribute(body, "font-size"), "16");
    assert_eq!(attribute(body, "fill"), "#282828");
}

#[test]
fn facets_use_their_owning_plot_theme_without_a_new_scope() {
    let facets =
        || FacetSpec::column(CategoryValues::new("panel", ["first", "first", "second"]).unwrap());
    let omitted = render_svg(scatter().facet(facets()), "theme-facet-default").unwrap();
    let explicit = render_svg(
        scatter().theme(Theme::Publication).facet(facets()),
        "theme-facet-publication",
    )
    .unwrap();
    let nature = render_svg(
        scatter().theme(Theme::Nature).facet(facets()),
        "theme-facet-nature",
    )
    .unwrap();

    assert_eq!(omitted, explicit);
    assert_ne!(nature, omitted);
    assert!(nature.contains("#285682"));
}

#[test]
fn theme_owners_are_isolated_across_render_calls() {
    let canvas = |theme| {
        Canvas::grid(1, 1)
            .unwrap()
            .theme(theme)
            .panel(0, 0, scatter())
            .unwrap()
    };
    let first_nature =
        render_canvas(canvas(Theme::Nature), "theme-isolation-nature-first").unwrap();
    let science = render_canvas(canvas(Theme::Science), "theme-isolation-science").unwrap();
    let publication = render_canvas(
        Canvas::grid(1, 1).unwrap().panel(0, 0, scatter()).unwrap(),
        "theme-isolation-publication",
    )
    .unwrap();
    let second_nature =
        render_canvas(canvas(Theme::Nature), "theme-isolation-nature-second").unwrap();

    assert_eq!(first_nature, second_nature);
    assert_ne!(first_nature, science);
    assert_ne!(first_nature, publication);
}
