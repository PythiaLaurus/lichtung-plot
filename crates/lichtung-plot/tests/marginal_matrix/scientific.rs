use super::*;

fn svg_points(node: roxmltree::Node<'_, '_>) -> Vec<[f64; 2]> {
    node.attribute("points")
        .unwrap()
        .split_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            [x.parse().unwrap(), y.parse().unwrap()]
        })
        .collect()
}

fn attribute(node: roxmltree::Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name).unwrap().parse().unwrap()
}

fn axis_mapping(axes: roxmltree::Node<'_, '_>, side: usize) -> [(f64, f64); 2] {
    let ticks = axes
        .children()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|label| {
            let value = label.text()?.parse::<f64>().ok()?;
            let line = label
                .prev_siblings()
                .find(|node| node.has_tag_name("polyline"))?;
            let points = svg_points(line);
            let vertical = points[0][0].to_bits() == points[1][0].to_bits();
            (vertical == (side == 0)).then_some((value, points[0][side]))
        })
        .collect::<Vec<_>>();
    assert!(ticks.len() >= 2);
    [ticks[0], *ticks.last().unwrap()]
}

fn project(value: f64, mapping: [(f64, f64); 2]) -> f64 {
    let [(first, start), (last, end)] = mapping;
    start + (value - first) / (last - first) * (end - start)
}

fn audit_density_values(
    densities: &[roxmltree::Node<'_, '_>],
    mappings: [[(f64, f64); 2]; 2],
    clips: [[f64; 4]; 2],
    values: &[[f64; 2]],
    groups: usize,
) {
    for side in 0..2 {
        let mut expected = Vec::new();
        let mut actual = Vec::new();
        for group in 0..groups {
            let points = svg_points(densities[2 * group + side]);
            let grid = points[1..81]
                .iter()
                .map(|point| {
                    project(
                        point[side],
                        mappings[side].map(|(value, pixel)| (pixel, value)),
                    )
                })
                .collect::<Vec<_>>();
            let samples = values
                .chunks(values.len() / groups)
                .nth(group)
                .unwrap()
                .iter()
                .map(|value| value[side])
                .collect::<Vec<_>>();
            // Independent public statistical input: original samples and the
            // grid recovered from emitted scientific tick positions.
            let mut curve = lichtung_plot::stats::distribution::kde(&samples, &grid).unwrap();
            curve[0] = 0.0;
            curve[79] = 0.0;
            expected.push(curve);
            actual.push(
                points[1..81]
                    .iter()
                    .map(|point| (point[1 - side] - points[0][1 - side]).abs())
                    .collect::<Vec<_>>(),
            );
        }
        let maximum =
            |curves: &[Vec<f64>]| curves.iter().flatten().copied().fold(0.0_f64, f64::max);
        let expected_maximum = maximum(&expected);
        let actual_maximum = maximum(&actual);
        let strip_span = clips[side][3 - side];
        assert!(actual_maximum > 0.0 && actual_maximum <= strip_span);
        for (expected, actual) in expected.iter().flatten().zip(actual.iter().flatten()) {
            // Inverting quantized ticks/grid and emitting curve vertices each
            // loses subpixel precision; 0.03% bounds that combined error here.
            assert!((expected / expected_maximum - actual / actual_maximum).abs() < 0.0003);
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "audit source values, density curves, linked axes and guide content in each scientific scene"
)]
fn audit_scientific_svg(svg: &str, name: &str, palette: [&str; 2], case: usize) {
    let document = roxmltree::Document::parse(svg).unwrap();
    let root = document.root_element();
    assert_eq!(root.attribute("viewBox"), Some("0 0 800 600"));
    let groups = if case == 0 { 1 } else { 2 };
    let clips = document
        .descendants()
        .filter(|node| node.has_tag_name("clipPath"))
        .map(|clip| {
            let rect = clip
                .children()
                .find(|node| node.has_tag_name("rect"))
                .unwrap();
            ["x", "y", "width", "height"].map(|name| attribute(rect, name))
        })
        .collect::<Vec<_>>();
    assert_eq!(clips.len(), 3 * groups);
    let [central, top, right] = [clips[0], clips[1], clips[2]];
    assert_eq!((central[0], central[2]), (top[0], top[2]));
    assert_eq!((central[1], central[3]), (right[1], right[3]));
    assert!(right[0] >= central[0] + central[2]);
    assert!(central[1] >= top[1] + top[3]);
    if groups == 2 {
        assert_eq!(&clips[..3], &clips[3..]);
    }
    let axes = document
        .descendants()
        .find(|node| node.attribute("id") == Some("axes"))
        .unwrap();
    let mappings = [axis_mapping(axes, 0), axis_mapping(axes, 1)];
    for name in ["X", "Y"] {
        assert_eq!(
            axes.children()
                .filter(|node| node.has_tag_name("text") && node.text() == Some(name))
                .count(),
            1
        );
    }
    if case == 3 {
        for (value, position) in [(0.5, central[0]), (4.0, central[0] + central[2])] {
            assert!((project(value, mappings[0]) - position).abs() < 2.0 / 1024.0);
        }
        for (value, position) in [(1.0, central[1] + central[3]), (5.0, central[1])] {
            assert!((project(value, mappings[1]) - position).abs() < 2.0 / 1024.0);
        }
    }
    let scientific = |tag| {
        document.descendants().filter(move |node| {
            node.has_tag_name(tag)
                && node
                    .parent()
                    .is_some_and(|parent| parent.attribute("clip-path").is_some())
        })
    };
    let marks = scientific("circle").collect::<Vec<_>>();
    assert_eq!(marks.len(), 6);
    let values = [
        [1.0, 2.0],
        [1.2, 2.1],
        [1.4, 2.3],
        [3.0, 4.0],
        [3.2, 4.2],
        [3.4, 4.3],
    ];
    for (index, (mark, [x, y])) in marks.iter().zip(values).enumerate() {
        let color = palette[usize::from(case > 0 && index >= 3)];
        assert_eq!(mark.attribute("fill"), Some(color));
        assert_eq!(mark.attribute("r"), Some("3"));
        assert_eq!(mark.attribute("fill-opacity"), Some("0.8196078431372549"));
        assert_eq!(mark.attribute("stroke"), Some("#FFFFFF"));
        assert_eq!(mark.attribute("stroke-width"), Some("0.2998046875"));
        assert!((attribute(*mark, "cx") - project(x, mappings[0])).abs() < 2.0 / 1024.0);
        assert!((attribute(*mark, "cy") - project(y, mappings[1])).abs() < 2.0 / 1024.0);
    }
    let densities = scientific("polygon").collect::<Vec<_>>();
    assert_eq!(densities.len(), 2 * groups);
    let mut data_grids = [Vec::new(), Vec::new()];
    for (index, density) in densities.iter().enumerate() {
        let side = index % 2;
        let clip = if side == 0 { top } else { right };
        let points = svg_points(*density);
        assert_eq!(
            points.len(),
            82,
            "complete 80-sample curve plus two closures"
        );
        let color = palette[index / 2];
        assert_eq!(density.attribute("fill"), Some(color));
        assert_eq!(density.attribute("stroke"), Some(color));
        assert_eq!(density.attribute("stroke-width"), Some("1"));
        assert_eq!(
            density.attribute("fill-opacity"),
            Some("0.45098039215686275")
        );
        let data = points.iter().map(|point| point[side]).collect::<Vec<_>>();
        if index < 2 {
            data_grids[side].clone_from(&data);
        } else {
            assert_eq!(data, data_grids[side]);
        }
        assert!(data.windows(2).all(|pair| if side == 0 {
            pair[0] <= pair[1]
        } else {
            pair[0] >= pair[1]
        }));
        let amplitude = 1 - side;
        let baseline = points[0][amplitude];
        assert_eq!(
            points.last().unwrap()[amplitude].to_bits(),
            baseline.to_bits()
        );
        assert!(points
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite()));
        assert!(points
            .iter()
            .any(|point| (point[amplitude] - baseline).abs() > 0.0));
        // Every sample remains in its automatic data window. Flat density
        // closures reserve the half-stroke on the amplitude dimension; source
        // endpoints retain their actual (direction-dependent) outline reserve.
        for point in &points {
            for dimension in 0..2 {
                if case != 3 || dimension != side {
                    let outset = if dimension == side { 0.0 } else { 0.5 };
                    assert!(point[dimension] - outset >= clip[dimension]);
                    assert!(point[dimension] + outset <= clip[dimension] + clip[dimension + 2]);
                }
            }
        }
    }
    audit_density_values(&densities, mappings, [top, right], &values, groups);
    let legend = document
        .descendants()
        .find(|node| node.attribute("id") == Some("legend"));
    assert_eq!(legend.is_some(), case == 1 || case == 2);
    if let Some(legend) = legend {
        let labels = legend
            .children()
            .filter(|node| node.has_tag_name("text"))
            .map(|node| node.text().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            if case == 1 {
                vec!["Hue", "B", "A"]
            } else {
                vec!["Group × Hue", "G 1, B", "G 2, A"]
            }
        );
        assert_eq!(
            legend
                .children()
                .filter(|node| node.has_tag_name("circle"))
                .count(),
            2
        );
        assert_eq!(
            legend
                .children()
                .filter(|node| node.has_tag_name("polygon"))
                .count(),
            2
        );
        let frame = legend
            .children()
            .find(|node| node.has_tag_name("rect"))
            .unwrap();
        assert!(attribute(frame, "x") > right[0] + right[2]);
        assert!(attribute(frame, "x") + attribute(frame, "width") + 0.5 <= 800.0);
        println!("{name}/{case}: central={central:?}, top={top:?}, right={right:?}, guide={}x{}, labels={labels:?}", attribute(frame, "width"), attribute(frame, "height"));
    }
}

#[test]
fn marginal_preserves_scientific_values_and_linked_axes() {
    for (name, theme, palette) in [
        (
            "publication",
            lichtung_plot::Theme::publication(),
            ["#0072B2", "#D55E00"],
        ),
        (
            "science",
            lichtung_plot::Theme::science(),
            ["#FF0000", "#0000FF"],
        ),
    ] {
        for case in 0..4 {
            let mut plot = Plot::marginal(
                NumericValues::new("x", [1.0, 1.2, 1.4, 3.0, 3.2, 3.4]),
                NumericValues::new("y", [2.0, 2.1, 2.3, 4.0, 4.2, 4.3]),
            )
            .theme(theme.clone())
            .size(800, 600);
            if case > 0 {
                plot =
                    plot.hue(CategoryValues::new("hue", ["b", "b", "b", "a", "a", "a"]).unwrap());
            }
            if case > 1 {
                plot = plot.group(
                    CategoryValues::new("group", ["g1", "g1", "g1", "g2", "g2", "g2"]).unwrap(),
                );
            }
            if case == 3 {
                plot = plot
                    .x_limits(0.5, 4.0)
                    .y_limits(1.0, 5.0)
                    .legend(LegendPolicy::Hidden);
            }
            let directory = unique_dir("marginal-parity");
            let output = directory.join("plot.svg");
            plot.svg_preserve_text(&output).unwrap();
            let svg = std::fs::read(&output).unwrap();
            audit_scientific_svg(std::str::from_utf8(&svg).unwrap(), name, palette, case);
            if let Some(directory) = std::env::var_os("LICHTUNG_MARGINAL_AUDIT_DIR") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory).unwrap();
                std::fs::write(directory.join(format!("marginal-{name}-{case}.svg")), &svg)
                    .unwrap();
            }
            std::fs::remove_dir_all(directory).unwrap();
        }
    }
}
