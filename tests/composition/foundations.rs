use super::*;
use lichtung_plot::{
    Color, ColorDomain, ColorStop, ContinuousColorScale, ContrastTextRoles, DivergingGradient,
    FontFamily, FontStack, GenericFontFamily, HeatmapAnnotations, NumericFormat, TextTheme,
    TextWeight,
};

fn stack(family: GenericFontFamily) -> FontStack {
    let profile = Theme::publication()
        .typography()
        .get_root()
        .get_title()
        .get_stack()
        .clone();
    FontStack::new(
        [FontFamily::generic(family)]
            .into_iter()
            .chain(profile.families().iter().cloned()),
    )
    .unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one mixed-family Shared scene verifies complete public gradient, alpha and chrome ownership"
)]
fn public_shared_gradient_drives_both_families_without_borrowing_mark_alpha_or_child_chrome() {
    let gradient = DivergingGradient::new([
        ColorStop::new(0.0, OpaqueColor::rgb(17, 53, 89)).unwrap(),
        ColorStop::new(0.25, OpaqueColor::rgb(41, 79, 113)).unwrap(),
        ColorStop::new(0.5, OpaqueColor::rgb(221, 223, 227)).unwrap(),
        ColorStop::new(0.75, OpaqueColor::rgb(193, 107, 71)).unwrap(),
        ColorStop::new(1.0, OpaqueColor::rgb(151, 47, 31)).unwrap(),
    ])
    .unwrap();
    let base = Theme::publication();
    let label = base
        .typography()
        .get_guides()
        .get_legend_label()
        .clone()
        .color(Color::rgb(19, 61, 113))
        .unwrap()
        .font_size(13)
        .unwrap();
    let guides = base
        .typography()
        .get_guides()
        .clone()
        .legend_label(label)
        .unwrap();
    let typography = base.typography().clone().guides(guides).unwrap();
    let root_theme = base.customize().typography(typography).build().unwrap();
    let primary = base
        .marks()
        .get_points()
        .get_primary()
        .clone()
        .fill_alpha(71)
        .unwrap();
    let points = base.marks().get_points().clone().primary(primary).unwrap();
    let cells = base.marks().get_cells().clone().fill_alpha(151).unwrap();
    let marks = base
        .marks()
        .clone()
        .points(points)
        .unwrap()
        .cells(cells)
        .unwrap();
    let label = base
        .typography()
        .get_guides()
        .get_legend_label()
        .clone()
        .color(Color::rgb(211, 79, 23))
        .unwrap()
        .font_size(7)
        .unwrap();
    let guides = base
        .typography()
        .get_guides()
        .clone()
        .legend_label(label)
        .unwrap();
    let typography = base.typography().clone().guides(guides).unwrap();
    let scales = base
        .color_scales()
        .clone()
        .diverging(gradient.clone())
        .unwrap();
    let child_theme = base
        .customize()
        .marks(marks)
        .typography(typography)
        .color_scales(scales)
        .build()
        .unwrap();
    let selection =
        ContinuousColorScale::diverging(0.0, None, Some(ColorDomain::exact(-2.0, 6.0).unwrap()))
            .unwrap();
    let heatmap = Plot::heatmap(
        CategoryValues::new("column", ["low", "lower", "center"]).unwrap(),
        CategoryValues::new("row", ["one"; 3]).unwrap(),
        NumericValues::new("response", [-2.0, -1.0, 0.0]),
    )
    .theme(child_theme.clone())
    .continuous_color_scale(selection.clone());
    let beeswarm = Plot::beeswarm(
        CategoryValues::new("group", ["one"; 3]).unwrap(),
        NumericValues::new("x", [0.0, 1.0, 2.0]),
        NumericValues::new("response", [0.0, 3.0, 6.0]),
    )
    .theme(child_theme)
    .continuous_color_scale(selection.diverging_gradient(Some(gradient)).unwrap());
    let svg = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .size(1201, 701)
            .theme(root_theme)
            .panel(0, 0, heatmap)
            .unwrap()
            .panel(0, 1, beeswarm)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "public-shared-foundations",
    )
    .unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let polygons = document
        .descendants()
        .filter(|node| {
            node.has_tag_name("polygon")
                && ["#113559", "#294F71", "#DDDFE3"]
                    .contains(&node.attribute("fill").unwrap_or_default())
        })
        .collect::<Vec<_>>();
    let points = document
        .descendants()
        .filter(|node| {
            node.has_tag_name("circle")
                && ["#DDDFE3", "#C16B47", "#972F1F"]
                    .contains(&node.attribute("fill").unwrap_or_default())
        })
        .collect::<Vec<_>>();
    assert_eq!(polygons.len(), 3);
    assert_eq!(points.len(), 3);
    for polygon in polygons {
        let alpha = polygon
            .attribute("fill-opacity")
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((alpha - 151.0 / 255.0).abs() < 0.000_001);
    }
    for point in points {
        let alpha = point
            .attribute("fill-opacity")
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!((alpha - 71.0 / 255.0).abs() < 0.000_001);
    }
    assert_eq!(svg.matches("<linearGradient ").count(), 1);
    for (offset, paint) in [
        ("0.000000", "#113559"),
        ("0.125000", "#294F71"),
        ("0.250000", "#DDDFE3"),
        ("0.625000", "#C16B47"),
        ("1.000000", "#972F1F"),
    ] {
        assert!(svg.contains(&format!(
            "<stop offset=\"{offset}\" stop-color=\"{paint}\"/>"
        )));
    }
    let guide = document
        .descendants()
        .find(|node| node.attribute("id") == Some("canvas-shared-legend"))
        .unwrap();
    let labels = guide
        .descendants()
        .filter(|node| {
            node.has_tag_name("text") && ["-2", "0", "6"].contains(&node.text().unwrap_or_default())
        })
        .collect::<Vec<_>>();
    assert_eq!(labels.len(), 3);
    for label in labels {
        assert_eq!(label.attribute("fill"), Some("#133D71"));
        assert_eq!(label.attribute("font-size"), Some("13"));
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one transparent public scene verifies complete contrast-role selection through SVG and decoded PNG"
)]
fn transparent_cells_select_complete_custom_contrast_roles_before_frozen_svg_and_png() {
    let base = Theme::publication();
    let light = TextTheme::new(
        stack(GenericFontFamily::Monospace),
        Color::rgb(255, 255, 255),
        17,
        TextWeight::Bold,
        1.5,
    )
    .unwrap();
    let dark = TextTheme::new(
        stack(GenericFontFamily::Serif),
        Color::rgb(0, 0, 0),
        9,
        TextWeight::Semibold,
        1.25,
    )
    .unwrap();
    let annotations = base
        .typography()
        .get_annotations()
        .clone()
        .contrast(ContrastTextRoles::new(light, dark).unwrap())
        .unwrap();
    let unused = base
        .typography()
        .get_composition()
        .get_body()
        .clone()
        .stack(
            FontStack::new([FontFamily::named("Unavailable unused body role").unwrap()]).unwrap(),
        )
        .unwrap();
    let composition = base
        .typography()
        .get_composition()
        .clone()
        .body(unused)
        .unwrap();
    let typography = base
        .typography()
        .clone()
        .annotations(annotations)
        .unwrap()
        .composition(composition)
        .unwrap();
    let surfaces = base
        .surfaces()
        .clone()
        .canvas_root(Color::rgba(19, 37, 53, 0))
        .unwrap()
        .plot_panel(Color::rgba(31, 47, 61, 0))
        .unwrap();
    let cells = base.marks().get_cells().clone().fill_alpha(128).unwrap();
    let marks = base.marks().clone().cells(cells).unwrap();
    let annotations = base
        .annotations()
        .clone()
        .planning_backdrop(OpaqueColor::rgb(0, 0, 0))
        .unwrap();
    let theme = base
        .customize()
        .typography(typography)
        .surfaces(surfaces)
        .marks(marks)
        .annotations(annotations)
        .build()
        .unwrap();
    let gradient = DivergingGradient::new([
        ColorStop::new(0.0, OpaqueColor::rgb(0, 0, 0)).unwrap(),
        ColorStop::new(0.5, OpaqueColor::rgb(128, 128, 128)).unwrap(),
        ColorStop::new(1.0, OpaqueColor::rgb(255, 255, 255)).unwrap(),
    ])
    .unwrap();
    let directory = tempdir("public-contrast-foundations");
    let first = directory.join("first.svg");
    let second = directory.join("second.svg");
    let png = directory.join("scene.png");
    Plot::heatmap(
        CategoryValues::new("column", ["low", "center", "high"]).unwrap(),
        CategoryValues::new("row", ["one"; 3]).unwrap(),
        NumericValues::new("response", [-1.0, 0.0, 1.0]),
    )
    .theme(theme)
    .size(700, 500)
    .continuous_color_scale(ContinuousColorScale::diverging(0.0, Some(gradient), None).unwrap())
    .annotations(HeatmapAnnotations::Values(NumericFormat::default()))
    .svg_preserve_text(&first)
    .unwrap()
    .png_preview(&png)
    .unwrap()
    .svg_preserve_text(&second)
    .unwrap();
    let svg = std::fs::read_to_string(&first).unwrap();
    assert_eq!(svg.as_bytes(), std::fs::read(second).unwrap());
    let document = roxmltree::Document::parse(&svg).unwrap();
    for (value, paint, family, size, weight) in [
        ("-1.00", "#FFFFFF", "monospace", "17", "700"),
        ("0.00", "#FFFFFF", "monospace", "17", "700"),
        ("1.00", "#000000", "serif", "9", "600"),
    ] {
        let text = document
            .descendants()
            .find(|node| node.has_tag_name("text") && node.text() == Some(value))
            .unwrap();
        assert_eq!(text.attribute("fill"), Some(paint));
        assert!(text
            .attribute("font-family")
            .unwrap()
            .starts_with(&format!("{family}, Noto Sans, ")));
        assert_eq!(text.attribute("font-size"), Some(size));
        assert_eq!(text.attribute("font-weight"), Some(weight));
    }
    let mut reader = png::Decoder::new(std::io::Cursor::new(std::fs::read(png).unwrap()))
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!((info.width, info.height), (700, 500));
    assert_eq!(info.color_type, png::ColorType::Rgba);
    assert_eq!(pixels[3], 0, "the planning backdrop is not painted");
    for (paint, expected) in [
        ("#000000", [0, 0, 0, 128]),
        ("#FFFFFF", [255, 255, 255, 128]),
    ] {
        let cell = document
            .descendants()
            .find(|node| node.has_tag_name("polygon") && node.attribute("fill") == Some(paint))
            .unwrap();
        let corners = cell
            .attribute("points")
            .unwrap()
            .split_whitespace()
            .map(|point| {
                let (x, y) = point.split_once(',').unwrap();
                (x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
            })
            .collect::<Vec<_>>();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "sample a finite positive point strictly inside a rendered cell, away from text and outline"
        )]
        let (x, y) = (
            (corners[0].0 * 0.75 + corners[1].0 * 0.25) as usize,
            (corners[0].1 * 0.75 + corners[2].1 * 0.25) as usize,
        );
        let offset = (y * info.width as usize + x) * 4;
        assert_eq!(&pixels[offset..offset + 4], &expected);
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn active_custom_font_roles_are_required_while_unused_preferences_are_not() {
    let base = Theme::publication();
    let title = base
        .typography()
        .get_root()
        .get_title()
        .clone()
        .stack(
            FontStack::new([FontFamily::named("Unavailable public heading role").unwrap()])
                .unwrap(),
        )
        .unwrap();
    let root = base.typography().get_root().clone().title(title).unwrap();
    let typography = base.typography().clone().root(root).unwrap();
    let theme = base.customize().typography(typography).build().unwrap();
    let plot = Plot::scatter(
        NumericValues::new("x", [1.0, 2.0]),
        NumericValues::new("y", [2.0, 1.0]),
    )
    .theme(theme);
    plot.preflight().unwrap();
    let error = plot
        .title("This role is now consumed")
        .preflight()
        .unwrap_err();
    assert!(matches!(error, lichtung_plot::Error::Render(_)), "{error}");
    assert!(
        error.to_string().contains("font candidate list is empty"),
        "{error}"
    );
}
