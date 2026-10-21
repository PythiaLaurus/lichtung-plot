//! Actual alpha sections through parent/root modes and frozen publications.

use super::{legends::equal_output, measure_case, measure_pair, CaseMeasurement};
use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
use crate::orchestrator::measurements::Counter;
use crate::{
    AlphaEncoding, AlphaValues, Bounds, Canvas, CanvasLegendPolicy, CategoryValues, Labels,
    LegendPolicy, LegendPosition, NumericValues, Plot,
};

#[derive(Debug, Clone, Copy)]
enum Owner {
    Local,
    Facets,
    PerPlot,
    Shared,
    Hidden,
    Matrix,
    Marginal,
    Heatmap,
    Beeswarm,
    MixedShared,
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Automatic,
    Width,
    Exact,
}

fn single(name: &str) -> AlphaEncoding {
    AlphaEncoding::single(
        AlphaValues::linear(NumericValues::new(name, [0.5]), (0.0, 1.0), (0, 255)).unwrap(),
    )
    .unwrap()
}

fn line(count: u32) -> Plot<crate::kind::Line> {
    let group = |index| {
        if index < count / 2 {
            "control"
        } else {
            "treated"
        }
    };
    let values = (0..count)
        .map(|index| f64::from(index % 3) / 4.0 + 0.25)
        .collect::<Vec<_>>();
    Plot::line(
        NumericValues::new(
            "time",
            (0..count).map(|index| f64::from(index % (count / 2))),
        ),
        NumericValues::new("response", values.iter().copied()),
    )
    .hue(CategoryValues::new("series", (0..count).map(group)).unwrap())
    .markers(true)
    .band(Bounds::new(
        NumericValues::new("lower", values.iter().map(|value| value - 0.1)),
        NumericValues::new("upper", values.iter().map(|value| value + 0.1)),
    ))
    .point_alpha(AlphaEncoding::observations(
        AlphaValues::linear(
            NumericValues::new(
                "point confidence",
                (0..count).map(|i| f64::from(i) / f64::from(count - 1)),
            ),
            (0.0, 1.0),
            (0, 255),
        )
        .unwrap(),
    ))
    .line_alpha(
        AlphaEncoding::keyed(
            [CategoryValues::new("series", ["control", "treated"]).unwrap()],
            AlphaValues::threshold(
                NumericValues::new("line confidence", [0.25, 0.75]),
                [0.5],
                [0, 230],
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .fill_alpha(
        AlphaEncoding::keyed(
            [CategoryValues::new("series", ["control", "treated"]).unwrap()],
            AlphaValues::categorical(
                CategoryValues::new("fill confidence", ["limited", "strong"]).unwrap(),
                |value| if value == "limited" { 64 } else { 191 },
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .x_limits(0.0, f64::from(count / 2 - 1))
    .y_limits(0.0, 1.0)
    .labels(Labels::raw())
}

enum Root {
    Plot(Box<Plot>),
    Canvas(Box<Canvas>),
}

impl Root {
    fn new(owner: Owner, request: Request, count: u32) -> Self {
        let plot = || line(count);
        let root = match owner {
            Owner::Local => Self::Plot(Box::new(
                plot()
                    .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
                    .erase(),
            )),
            Owner::Facets => Self::Plot(Box::new(
                plot()
                    .facet(crate::FacetSpec::column(
                        CategoryValues::new(
                            "panel",
                            (0..count).map(|i| if i < count / 2 { "left" } else { "right" }),
                        )
                        .unwrap(),
                    ))
                    .legend(LegendPolicy::Visible(LegendPosition::OutsideRight))
                    .erase(),
            )),
            Owner::PerPlot | Owner::Shared | Owner::Hidden => Self::Canvas(Box::new(
                Canvas::grid(1, 2)
                    .unwrap()
                    .panel(0, 0, plot())
                    .unwrap()
                    .panel(0, 1, plot())
                    .unwrap()
                    .legend_policy(match owner {
                        Owner::PerPlot => CanvasLegendPolicy::PerPlot,
                        Owner::Shared => CanvasLegendPolicy::Shared,
                        _ => CanvasLegendPolicy::Hidden,
                    }),
            )),
            Owner::Matrix => Self::Plot(Box::new(
                Plot::matrix(
                    (0..4)
                        .map(|column| {
                            NumericValues::new(
                                format!("variable {column}"),
                                (0..count).map(|i| f64::from((i * (column * 12 + 7)) % 101)),
                            )
                        })
                        .collect(),
                )
                .point_alpha(single("point confidence"))
                .fill_alpha(single("fill confidence"))
                .labels(Labels::raw())
                .erase(),
            )),
            Owner::Marginal => Self::Plot(Box::new(
                Plot::marginal(
                    NumericValues::new("x", [0.0, 0.5, 1.0, 1.5, 2.0, 1.0]),
                    NumericValues::new("y", [0.5, 0.25, 1.0, 0.0, 0.75, 0.5]),
                )
                .point_alpha(single("point confidence"))
                .fill_alpha(single("fill confidence"))
                .labels(Labels::raw())
                .erase(),
            )),
            Owner::Heatmap => Self::Plot(Box::new(heatmap().erase())),
            Owner::Beeswarm => Self::Plot(Box::new(
                Plot::beeswarm(
                    CategoryValues::new("category", ["a", "a", "a", "b", "b", "b"]).unwrap(),
                    NumericValues::new("value", [0.0, 0.5, 1.0, 1.5, 2.0, 1.0]),
                    NumericValues::new("color", [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]),
                )
                .point_alpha(single("point confidence"))
                .labels(Labels::raw())
                .erase(),
            )),
            Owner::MixedShared => Self::Canvas(Box::new(
                Canvas::grid(1, 2)
                    .unwrap()
                    .panel(
                        0,
                        0,
                        Plot::scatter(
                            NumericValues::new("x", [0.0, 0.5, 1.0]),
                            NumericValues::new("y", [0.0, 1.0, 0.5]),
                        )
                        .point_alpha(single("point confidence")),
                    )
                    .unwrap()
                    .panel(0, 1, heatmap())
                    .unwrap()
                    .labels(Labels::raw())
                    .legend_policy(CanvasLegendPolicy::Shared),
            )),
        };
        match (root, request) {
            (Self::Plot(plot), Request::Automatic) => Self::Plot(plot),
            (Self::Canvas(canvas), Request::Automatic) => Self::Canvas(canvas),
            (Self::Plot(plot), Request::Width) => Self::Plot(Box::new((*plot).width(1600))),
            (Self::Canvas(canvas), Request::Width) => Self::Canvas(Box::new((*canvas).width(1600))),
            (Self::Plot(plot), Request::Exact) => Self::Plot(Box::new((*plot).size(1600, 1100))),
            (Self::Canvas(canvas), Request::Exact) => {
                Self::Canvas(Box::new((*canvas).size(1600, 1100)))
            }
        }
    }

    fn publish(self, path: &std::path::Path) -> crate::Result<crate::Output> {
        match self {
            Self::Plot(plot) => (*plot).svg(path),
            Self::Canvas(canvas) => (*canvas).svg(path),
        }
    }
}

fn heatmap() -> Plot<crate::kind::Heatmap> {
    Plot::heatmap(
        CategoryValues::new("column", ["a", "b", "a", "b"]).unwrap(),
        CategoryValues::new("row", ["one", "one", "two", "two"]).unwrap(),
        NumericValues::new("color", [0.0, 0.25, 0.75, 1.0]),
    )
    .fill_alpha(single("fill confidence"))
    .labels(Labels::raw())
}

fn case(owner: Owner, request: Request, count: u32, disabled: bool) -> CaseMeasurement {
    let previous = DISABLE_CHILD_REUSE.with(|flag| flag.replace(disabled));
    let result = std::panic::catch_unwind(|| {
        measure_case(
            &format!("alpha_{owner:?}_{request:?}_{count}_{disabled}"),
            || Root::new(owner, request, count),
            Root::publish,
            true,
            true,
        )
    });
    DISABLE_CHILD_REUSE.with(|flag| flag.set(previous));
    let sample = result.unwrap();
    assert_eq!(
        (sample.observations.calls[2], sample.observations.calls[3]),
        (1, 1)
    );
    assert_eq!(
        sample.observations.capture_counters,
        [0; Counter::ALL.len()]
    );
    assert!((1..=8).contains(&sample.observations.roots.len()));
    assert_output(&sample, owner, request);
    sample
}

fn assert_output(sample: &CaseMeasurement, owner: Owner, request: Request) {
    let bytes = sample.output.as_ref().unwrap();
    let document = roxmltree::Document::parse(
        std::str::from_utf8(bytes.preserved_svg.as_ref().unwrap()).unwrap(),
    )
    .unwrap();
    let root = document.root_element();
    let dimensions = (
        root.attribute("width").unwrap().parse::<u32>().unwrap(),
        root.attribute("height").unwrap().parse::<u32>().unwrap(),
    );
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
    if !matches!(request, Request::Automatic) {
        assert_eq!(dimensions.0, 1600);
    }
    if matches!(request, Request::Exact) {
        assert_eq!(dimensions.1, 1100);
    }
    let reader = png::Decoder::new(std::io::Cursor::new(&bytes.png))
        .read_info()
        .unwrap();
    assert_eq!((reader.info().width, reader.info().height), dimensions);
    let words = document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter_map(|node| node.text())
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>();
    let copies = if matches!(owner, Owner::PerPlot) {
        2
    } else {
        1
    };
    for (present, title) in [
        (
            !matches!(owner, Owner::Heatmap | Owner::Hidden),
            "point confidence (observation point alpha)",
        ),
        (
            matches!(
                owner,
                Owner::Local | Owner::Facets | Owner::PerPlot | Owner::Shared
            ),
            "line confidence (series line alpha)",
        ),
        (
            matches!(
                owner,
                Owner::Local | Owner::Facets | Owner::PerPlot | Owner::Shared
            ),
            "fill confidence (series fill alpha)",
        ),
        (
            matches!(owner, Owner::Matrix | Owner::Marginal),
            "fill confidence (series fill alpha)",
        ),
        (
            matches!(owner, Owner::Heatmap | Owner::MixedShared),
            "fill confidence (observation fill alpha)",
        ),
    ] {
        if present {
            let required = title.split_whitespace().collect::<Vec<_>>();
            assert_eq!(
                words
                    .windows(required.len())
                    .filter(|text| *text == required)
                    .count(),
                copies,
                "{owner:?}, {request:?}: {title}"
            );
        }
    }
    if matches!(owner, Owner::Hidden) {
        assert!(!words.contains(&"confidence"));
    }
    assert_eq!(
        document
            .descendants()
            .filter(|node| node.has_tag_name("linearGradient"))
            .count(),
        usize::from(matches!(
            owner,
            Owner::Heatmap | Owner::Beeswarm | Owner::MixedShared
        ))
    );
}

#[test]
fn alpha_owners_preserve_actual_sections_in_all_root_modes_and_cache_disabled_publications() {
    for owner in [
        Owner::Local,
        Owner::Facets,
        Owner::PerPlot,
        Owner::Shared,
        Owner::Hidden,
        Owner::Matrix,
        Owner::Marginal,
        Owner::Heatmap,
        Owner::Beeswarm,
        Owner::MixedShared,
    ] {
        for request in [Request::Automatic, Request::Width, Request::Exact] {
            equal_output(
                &case(owner, request, 6, false),
                &case(owner, request, 6, true),
            );
        }
    }
}

pub(super) fn measure() {
    println!("alpha_conditions,release,shared_vs_hidden,64_observations_per_panel,three_interleaved_runs,exact_1600x1100,three_explicit_targets");
    let shared = || case(Owner::Shared, Request::Exact, 64, false);
    let hidden = || case(Owner::Hidden, Request::Exact, 64, false);
    let samples = measure_pair(["alpha_shared", "alpha_hidden"], [&shared, &hidden]);
    for pair in &samples[1..] {
        equal_output(&samples[0][0], &pair[0]);
        equal_output(&samples[0][1], &pair[1]);
    }
    equal_output(
        &samples[0][0],
        &case(Owner::Shared, Request::Exact, 64, true),
    );
    equal_output(
        &samples[0][1],
        &case(Owner::Hidden, Request::Exact, 64, true),
    );
}
