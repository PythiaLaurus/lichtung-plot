use super::*;

struct LineData {
    grid_x: NumericValues,
    estimate: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
    panel: CategoryValues,
}

struct PointValues {
    x: NumericValues,
    y: NumericValues,
}

fn line_data() -> LineData {
    LineData {
        grid_x: NumericValues::new("grid_x", [0.0, 1.0, 2.0]),
        estimate: NumericValues::new("estimate", [1.0, 2.0, 3.0]),
        lower: NumericValues::new("lower", [0.5, 1.5, 2.5]),
        upper: NumericValues::new("upper", [1.5, 2.5, 3.5]),
        panel: CategoryValues::new("panel", ["a", "a", "a"]).unwrap(),
    }
}

fn observations() -> PointValues {
    PointValues {
        x: NumericValues::new("observed_x", [0.0, 0.5, 1.0, 1.5, 2.0]),
        y: NumericValues::new("observed_y", [0.8, 1.7, 2.0, 2.8, 3.2]),
    }
}

fn line_plot(data: &LineData) -> Plot<lichtung_plot::kind::Line> {
    Plot::line(data.grid_x.clone(), data.estimate.clone())
        .band(Bounds::new(data.lower.clone(), data.upper.clone()))
}

fn svg<K: lichtung_plot::kind::Marker>(plot: Plot<K>) -> String {
    render_svg(plot, "numeric-layers").unwrap()
}

fn attribute(line: &str, name: &str) -> f64 {
    let prefix = format!("{name}=\"");
    line.split_once(&prefix)
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn text_line<'a>(svg: &'a str, text: &str) -> &'a str {
    svg.lines()
        .find(|line| line.contains(&format!(">{text}</text>")))
        .unwrap()
}

fn style(color: lichtung_plot::Color, pattern: LinePattern) -> GuideStyle {
    GuideStyle::new(color, 1.25, pattern).unwrap()
}

fn point_layer(values: &PointValues, radius: u32, alpha: u8) -> PointLayer {
    PointLayer::new(values.x.clone(), values.y.clone())
        .color(lichtung_plot::Color::rgb(123, 45, 200))
        .radius(radius)
        .unwrap()
        .alpha(alpha)
}

fn supplemental_point(x: f64, y: f64) -> PointLayer {
    PointLayer::new(
        lichtung_plot::NumericValues::new("supplemental x", [x]),
        lichtung_plot::NumericValues::new("supplemental y", [y]),
    )
}

#[test]
fn automatic_point_layers_follow_an_independent_accent_sequence() {
    use lichtung_plot::{
        AccentPalette, CategoricalColorScale, OpaqueColor, QualitativePalette, Theme,
    };
    let base = Theme::publication();
    let marks = base
        .marks()
        .clone()
        .accents(
            AccentPalette::new([OpaqueColor::rgb(19, 43, 71), OpaqueColor::rgb(127, 59, 31)])
                .unwrap(),
        )
        .unwrap();
    let theme = base.customize().marks(marks).build().unwrap();
    let palette = CategoricalColorScale::from_palette(
        QualitativePalette::fixed([
            OpaqueColor::rgb(29, 137, 59),
            OpaqueColor::rgb(173, 41, 109),
        ])
        .unwrap(),
    );
    let plot = Plot::line(
        lichtung_plot::NumericValues::new("x", [0.0, 1.0, 0.0, 1.0]),
        lichtung_plot::NumericValues::new("y", [1.0, 2.0, 2.0, 3.0]),
    )
    .theme(theme)
    .categorical_color_scale(palette)
    .hue(lichtung_plot::CategoryValues::new("cohort", ["a", "a", "b", "b"]).unwrap())
    .markers(false)
    .point_layer(supplemental_point(0.25, 1.5))
    .point_layer(supplemental_point(0.75, 2.5));
    let output = svg(plot);
    let points = output
        .lines()
        .filter(|line| line.starts_with("<circle "))
        .collect::<Vec<_>>();

    assert_eq!(points.len(), 2);
    assert!(points[0].contains("fill=\"#132B47\"") && points[1].contains("fill=\"#7F3B1F\""));
    assert!(output.contains("stroke=\"#1D893B\"") && output.contains("stroke=\"#AD296D\""));
}

#[test]
fn canvas_absent_hue_levels_do_not_consume_accent_slots() {
    let left = Plot::line(
        lichtung_plot::NumericValues::new("left x", [0.0, 1.0]),
        lichtung_plot::NumericValues::new("left y", [1.0, 2.0]),
    )
    .hue(lichtung_plot::CategoryValues::new("cohort", ["local", "local"]).unwrap())
    .markers(false)
    .point_layer(supplemental_point(0.5, 1.5).legend_label("selection"));
    let right = Plot::line(
        lichtung_plot::NumericValues::new("right x", [0.0, 1.0]),
        lichtung_plot::NumericValues::new("right y", [2.0, 3.0]),
    )
    .hue(lichtung_plot::CategoryValues::new("cohort", ["absent", "absent"]).unwrap())
    .markers(false);
    let output = render_canvas(
        Canvas::grid(1, 2)
            .unwrap()
            .panel(0, 0, left)
            .unwrap()
            .panel(0, 1, right)
            .unwrap()
            .legend_policy(CanvasLegendPolicy::Shared),
        "point-layer-global-hue-slots",
    )
    .unwrap();
    let points = output
        .lines()
        .filter(|line| line.starts_with("<circle "))
        .collect::<Vec<_>>();

    assert_eq!(points.len(), 2);
    assert!(points
        .iter()
        .all(|point| point.contains("fill=\"#0072B2\"")));
}

#[test]
fn point_labels_and_whole_layer_legend_identity_remain_distinct() {
    use lichtung_plot::{Color, Text, TextAlignment, Theme};
    let frame = line_data();
    let style = Theme::publication()
        .typography()
        .get_guides()
        .get_legend_label()
        .clone()
        .font_size(19.0)
        .unwrap()
        .color(Color::rgb(25, 74, 117))
        .unwrap();
    let layer = PointLayer::new(
        lichtung_plot::NumericValues::new("selected x", [0.5, 1.5]),
        lichtung_plot::NumericValues::new("selected y", [1.5, 2.5]),
    )
    .color(lichtung_plot::Color::rgb(123, 45, 200))
    .labels(lichtung_plot::CategoryValues::new("annotations", ["first", "second"]).unwrap())
    .legend_label(
        Text::new("discarded")
            .style(style)
            .alignment(TextAlignment::End),
    )
    .legend_label("selected_coordinates");
    let base = line_plot(&frame)
        .markers(false)
        .point_layer(layer)
        .labels(lichtung_plot::Labels::raw().alias("selected_coordinates", "Chosen sample"));
    for composition in [
        lichtung_plot::LegendComposition::Combined,
        lichtung_plot::LegendComposition::Separate,
    ] {
        for hued in [false, true] {
            let plot = base.clone().legend_composition(composition);
            let plot = if hued {
                plot.hue(CategoryValues::new("cohort", ["main"; 3]).unwrap())
            } else {
                plot
            };
            let output = svg(plot);
            let legend = output
                .split_once("<g id=\"legend\">")
                .unwrap()
                .1
                .split_once("</g>")
                .unwrap()
                .0;
            let label = if hued {
                "Chosen sample"
            } else {
                "selected_coordinates"
            };
            assert_eq!(legend.matches(&format!(">{label}</text>")).count(), 1);
            let legend_text = text_line(&output, label);
            assert!(legend_text.contains("font-size=\"19\""));
            assert!(legend_text.contains("fill=\"#194A75\""));
            assert_eq!(legend.matches("<circle ").count(), 1);
            assert!(!legend.contains(">first</text>") && !legend.contains(">second</text>"));
            for label in ["first", "second"] {
                assert!(text_line(&output, label).contains("fill=\"#7B2DC8\""));
            }
            assert_eq!(output.matches("<circle ").count(), 3);
        }
        let collision = base
            .clone()
            .legend_composition(composition)
            .hue(CategoryValues::new("cohort", ["selected_coordinates"; 3]).unwrap());
        if composition == lichtung_plot::LegendComposition::Combined {
            let error = collision.preflight().unwrap_err();
            assert!(
                error.to_string().contains("distinct raw identities"),
                "{error}"
            );
        } else {
            let output = svg(collision);
            assert_eq!(output.matches(">Chosen sample</text>").count(), 2);
            assert_eq!(output.matches(">cohort</text>").count(), 1);
            assert_eq!(output.matches(">series</text>").count(), 1);
        }
    }
}

#[test]
fn clipped_point_labels_do_not_require_unpainted_glyphs() {
    let outside = PointLayer::new(
        NumericValues::new("point x", [2.0]),
        NumericValues::new("point y", [2.0]),
    )
    .labels(CategoryValues::new("labels", ["\u{10FFFF}"]).unwrap());

    Plot::line(
        NumericValues::new("x", [0.0, 1.0]),
        NumericValues::new("y", [0.0, 1.0]),
    )
    .x_limits(0.0, 1.0)
    .y_limits(0.0, 1.0)
    .point_layer(outside)
    .preflight()
    .unwrap();
}

#[test]
fn conflicting_duplicate_point_layer_legend_identity_is_rejected() {
    let frame = line_data();
    let first = supplemental_point(0.5, 1.5)
        .color(lichtung_plot::Color::rgba(123, 45, 200, 71))
        .radius(5)
        .unwrap()
        .legend_label("Selected");
    let second = supplemental_point(1.5, 2.5)
        .color(lichtung_plot::Color::rgba(123, 45, 200, 71))
        .radius(5)
        .unwrap()
        .legend_label("Selected");
    for composition in [
        lichtung_plot::LegendComposition::Combined,
        lichtung_plot::LegendComposition::Separate,
    ] {
        for conflicting in [
            second
                .clone()
                .color(lichtung_plot::Color::rgba(0, 114, 178, 71)),
            second.clone().alpha(72),
            second.clone().radius(6).unwrap(),
            second.clone().legend_label(
                lichtung_plot::Text::new("Selected").layout(lichtung_plot::TextLayout::default()),
            ),
        ] {
            let error = line_plot(&frame)
                .point_layer(first.clone())
                .point_layer(conflicting)
                .legend_composition(composition)
                .preflight()
                .unwrap_err()
                .to_string();

            assert!(error.contains("compatible semantic titles, labels, and styles"));
        }
    }
}

#[test]
fn explicit_x_ticks_reject_projected_neighboring_label_collisions() {
    let frame = line_data();
    let error = line_plot(&frame)
        .x_limits(0.0, 2.0)
        .x_ticks(vec![
            AxisTick::new(1.0, "First exact label"),
            AxisTick::new(1.01, "Second exact label"),
            AxisTick::new(1.02, "Third exact label"),
        ])
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(
        error.contains("AxisBottom") && error.contains("assigned cell"),
        "{error}"
    );
}

#[test]
fn horizontal_vertical_and_diagonal_guides_render_closed_styles() {
    let frame = line_data();
    let diagonal = GuideStyle::new(lichtung_plot::Color::rgb(0, 0, 0), 2.0, LinePattern::Solid)
        .unwrap()
        .color(lichtung_plot::Color::rgba(34, 51, 204, 149))
        .unwrap()
        .width(1.25)
        .unwrap()
        .pattern(LinePattern::Dashed)
        .unwrap();
    assert_eq!(
        diagonal.get_color(),
        lichtung_plot::Color::rgba(34, 51, 204, 149)
    );
    assert_eq!(diagonal.get_width().to_bits(), 1.25_f32.to_bits());
    assert_eq!(diagonal.get_pattern(), LinePattern::Dashed);
    let output = svg(line_plot(&frame)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .guide(GuideSpec::horizontal(
            2.0,
            style(
                lichtung_plot::Color::rgba(200, 17, 34, 73),
                LinePattern::Dashed,
            ),
        ))
        .guide(GuideSpec::vertical(
            1.0,
            style(lichtung_plot::Color::rgb(17, 180, 51), LinePattern::Solid),
        ))
        .guide(GuideSpec::segment(0.0, 0.5, 2.0, 3.5, diagonal)));
    assert!(["#C81122", "#11B433", "#2233CC"]
        .iter()
        .all(|color| output.contains(color)));
    assert_eq!(output.matches("stroke-dasharray=\"5 3\"").count(), 2);
    let horizontal = output
        .lines()
        .find(|line| line.contains("stroke=\"#C81122\""))
        .unwrap();
    let diagonal = output
        .lines()
        .find(|line| line.contains("stroke=\"#2233CC\""))
        .unwrap();
    assert_eq!(
        attribute(horizontal, "stroke-opacity").to_bits(),
        (73.0_f64 / 255.0).to_bits()
    );
    assert_eq!(
        attribute(diagonal, "stroke-opacity").to_bits(),
        (149.0_f64 / 255.0).to_bits()
    );
}

#[test]
fn separate_point_layers_preserve_row_count_radius_and_alpha() {
    let frame = line_data();
    let first = observations();
    let second = PointValues {
        x: NumericValues::new("second_x", [0.25, 1.75]),
        y: NumericValues::new("second_y", [1.2, 3.0]),
    };
    let output = svg(line_plot(&frame)
        .markers(false)
        .point_layer(point_layer(&first, 5, 89))
        .point_layer(point_layer(&second, 5, 89)));
    let count = output
        .lines()
        .filter(|line| {
            line.starts_with("<circle ")
                && line.contains("fill=\"#7B2DC8\"")
                && line.contains("r=\"5\"")
                && line.contains("fill-opacity=\"0.34901960784313724\"")
        })
        .count();
    assert_eq!(count, 7);
}

#[test]
fn point_layer_and_line_ribbon_share_one_projection() {
    let frame = line_data();
    let raw = PointValues {
        x: NumericValues::new("raw_x", [1.0]),
        y: NumericValues::new("raw_y", [2.0]),
    };
    let output = svg(line_plot(&frame)
        .markers(false)
        .point_layer(point_layer(&raw, 4, 102)));
    let point = output
        .lines()
        .find(|line| line.starts_with("<circle ") && line.contains("#7B2DC8"))
        .unwrap();
    let coordinate = format!("{},{}", attribute(point, "cx"), attribute(point, "cy"));
    let line = output
        .lines()
        .find(|line| {
            line.starts_with("<polyline ") && line.contains("stroke-width=\"1.7998046875\"")
        })
        .unwrap();
    let ribbon = output
        .lines()
        .find(|line| {
            line.starts_with("<polygon ") && line.contains("fill-opacity=\"0.1803921568627451\"")
        })
        .unwrap();
    let x_prefix = format!("{},", attribute(point, "cx"));
    assert!(line.contains(&coordinate) && ribbon.matches(&x_prefix).count() == 2);
}

#[test]
fn out_of_range_ticks_and_guides_are_rejected() {
    let frame = line_data();
    let tick = line_plot(&frame)
        .x_limits(0.0, 2.0)
        .x_ticks(vec![AxisTick::new(3.0, "outside")])
        .preflight()
        .unwrap_err()
        .to_string();
    let guide = line_plot(&frame)
        .x_limits(0.0, 2.0)
        .y_limits(0.0, 4.0)
        .guide(GuideSpec::segment(
            0.0,
            1.0,
            3.0,
            2.0,
            style(lichtung_plot::Color::rgb(0, 0, 0), LinePattern::Solid),
        ))
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(tick.contains("outside resolved range") && guide.contains("outside"));
}

#[test]
fn point_layers_reject_incomplete_or_non_finite_rows() {
    let frame = line_data();
    let invalid = PointLayer::new(
        lichtung_plot::NumericValues::nullable("raw_x", [Some(0.0), None, Some(2.0)]),
        lichtung_plot::NumericValues::new("raw_y", [1.0, 2.0, f64::NAN]),
    );
    let error = line_plot(&frame)
        .point_layer(invalid)
        .preflight()
        .unwrap_err()
        .to_string();
    assert!(error.contains("line point layer 0") && error.contains("source row 1"));
}

#[test]
fn supplemental_geometry_rejects_facets_and_incompatible_guides() {
    let frame = line_data();
    let observed = observations();
    let layer = point_layer(&observed, 3, 128);
    let faceted = line_plot(&frame)
        .facet(FacetSpec::column(frame.panel.clone()))
        .point_layer(layer)
        .preflight()
        .unwrap_err()
        .to_string();
    let segment = Plot::bar(
        frame.panel.clone(),
        lichtung_plot::BarInput::samples(frame.estimate.clone()),
    )
    .guide(GuideSpec::segment(
        0.0,
        1.0,
        1.0,
        2.0,
        style(lichtung_plot::Color::rgb(0, 0, 0), LinePattern::Solid),
    ))
    .preflight()
    .unwrap_err()
    .to_string();
    assert!(faceted.contains("not supported with faceting"));
    assert!(segment.contains("requires numeric x and y axes"));
}
