//! Complete generated-source selections through actual parents and frozen output.

use super::{legends::equal_output, measure_case, measure_pair, CaseMeasurement};
use crate::decorations::plan::children::DISABLE_CHILD_REUSE;
use crate::orchestrator::measurements::Counter;
use crate::{Canvas, CategoryValues, NumericFormat, NumericValues, Plot, TickGeneration};

#[derive(Debug, Clone, Copy)]
enum Fixture {
    Linear,
    Facets,
    RangeGroup,
    Forest,
    ForestCanvas,
}

impl Fixture {
    fn logarithmic(self) -> bool {
        matches!(self, Self::Forest | Self::ForestCanvas)
    }

    fn panels(self) -> usize {
        if matches!(self, Self::Facets | Self::RangeGroup | Self::ForestCanvas) {
            2
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Automatic,
    Width,
    Exact,
}

fn linear(generation: TickGeneration) -> Plot<crate::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("x", [0.0, 0.5, 1.0, 0.0, 0.5, 1.0]),
        NumericValues::new("y", [0.0, 0.25, 1.0, 0.5, 0.75, 0.0]),
    )
    .x_tick_generation(generation)
    .y_tick_generation(TickGeneration::maximum(3).unwrap())
    .x_tick_format(NumericFormat::default().prefix("X:").unwrap())
    .y_tick_format(NumericFormat::default().prefix("Y:").unwrap())
    .y_limits(0.0, 1.0)
}

fn forest(generation: TickGeneration) -> Plot<crate::kind::Forest> {
    Plot::forest(
        CategoryValues::new("term", ["a", "b"]).unwrap(),
        NumericValues::new("estimate", [0.5, 2.0]),
        crate::Bounds::new(
            NumericValues::new("lower", [0.1, 1.0]),
            NumericValues::new("upper", [1.0, 10.0]),
        ),
        1.0,
    )
    .scale(crate::AxisScale::Log10)
    .x_limits(0.001, 1000.0)
    .x_tick_generation(generation)
    .x_tick_format(
        NumericFormat::new(
            crate::NumericNotation::Scientific,
            crate::NumericPrecision::Significant(3),
        )
        .unwrap()
        .prefix("F:")
        .unwrap(),
    )
}

enum Root {
    Plot(Box<Plot>),
    Canvas(Box<Canvas>),
}

impl Root {
    fn new(fixture: Fixture, request: Request, generation: TickGeneration) -> Self {
        let root = match fixture {
            Fixture::Linear => Self::Plot(Box::new(linear(generation).x_limits(0.0, 1.0).erase())),
            Fixture::Facets => Self::Plot(Box::new(
                linear(generation)
                    .x_limits(0.0, 1.0)
                    .facet(crate::FacetSpec::column(
                        CategoryValues::new("panel", ["a", "a", "a", "b", "b", "b"]).unwrap(),
                    ))
                    .erase(),
            )),
            Fixture::RangeGroup => Self::Canvas(Box::new(
                Canvas::grid(1, 2)
                    .unwrap()
                    .panel(0, 0, linear(generation))
                    .unwrap()
                    .panel(
                        0,
                        1,
                        linear(TickGeneration::maximum(3).unwrap())
                            .x_tick_format(NumericFormat::default().prefix("R:").unwrap()),
                    )
                    .unwrap()
                    .x_range_group([(0, 0), (0, 1)])
                    .unwrap(),
            )),
            Fixture::Forest => Self::Plot(Box::new(forest(generation).erase())),
            Fixture::ForestCanvas => Self::Canvas(Box::new(
                Canvas::grid(1, 2)
                    .unwrap()
                    .panel(0, 0, forest(generation))
                    .unwrap()
                    .panel(
                        0,
                        1,
                        forest(TickGeneration::maximum(3).unwrap())
                            .x_tick_format(NumericFormat::default().prefix("R:").unwrap()),
                    )
                    .unwrap(),
            )),
        };
        match (root, request) {
            (Self::Plot(plot), Request::Automatic) => Self::Plot(plot),
            (Self::Canvas(canvas), Request::Automatic) => Self::Canvas(canvas),
            (Self::Plot(plot), Request::Width) => Self::Plot(Box::new((*plot).width(1200))),
            (Self::Canvas(canvas), Request::Width) => Self::Canvas(Box::new((*canvas).width(1200))),
            (Self::Plot(plot), Request::Exact) => Self::Plot(Box::new((*plot).size(1200, 800))),
            (Self::Canvas(canvas), Request::Exact) => {
                Self::Canvas(Box::new((*canvas).size(1200, 800)))
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

fn assert_output(
    sample: &CaseMeasurement,
    fixture: Fixture,
    request: Request,
    generation: &TickGeneration,
) {
    let bytes = sample.output.as_ref().unwrap();
    let svg = std::str::from_utf8(bytes.preserved_svg.as_ref().unwrap()).unwrap();
    let document = roxmltree::Document::parse(svg).unwrap();
    let labels = |prefix: &str| {
        document
            .descendants()
            .filter(|node| node.has_tag_name("text"))
            .filter_map(|node| node.text())
            .filter(|text| text.starts_with(prefix))
            .count()
    };
    let primary = labels(if fixture.logarithmic() { "F:" } else { "X:" });
    let copies = if matches!(fixture, Fixture::Facets) {
        2
    } else {
        1
    };
    assert!(primary > 0);
    if let Some(maximum) = generation.maximum_count() {
        assert!(primary <= maximum as usize * copies);
    }
    if matches!(request, Request::Exact)
        && generation.target_count() == Some(if fixture.logarithmic() { 4 } else { 5 })
    {
        assert_eq!(
            primary,
            generation.target_count().unwrap() as usize * copies
        );
    }
    if matches!(fixture, Fixture::RangeGroup | Fixture::ForestCanvas) {
        assert!((1..=3).contains(&labels("R:")));
    }
    if !fixture.logarithmic() {
        assert!((1..=3 * fixture.panels()).contains(&labels("Y:")));
    }
    let points = document
        .descendants()
        .filter(|node| node.has_tag_name("circle"))
        .count();
    assert_eq!(
        points,
        match fixture {
            Fixture::Linear | Fixture::Facets => 6,
            Fixture::RangeGroup => 12,
            Fixture::Forest => 2,
            Fixture::ForestCanvas => 4,
        }
    );
    let root = document.root_element();
    let dimensions = (
        root.attribute("width").unwrap().parse::<u32>().unwrap(),
        root.attribute("height").unwrap().parse::<u32>().unwrap(),
    );
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
    if !matches!(request, Request::Automatic) {
        assert_eq!(dimensions.0, 1200);
    }
    if matches!(request, Request::Exact) {
        assert_eq!(dimensions.1, 800);
    }
    let reader = png::Decoder::new(std::io::Cursor::new(&bytes.png))
        .read_info()
        .unwrap();
    assert_eq!((reader.info().width, reader.info().height), dimensions);
}

fn case(
    fixture: Fixture,
    request: Request,
    generation: &TickGeneration,
    disabled: bool,
) -> CaseMeasurement {
    let previous = DISABLE_CHILD_REUSE.with(|flag| flag.replace(disabled));
    let result = std::panic::catch_unwind(|| {
        measure_case(
            &format!("ticks_{fixture:?}_{request:?}_{generation:?}_{disabled}"),
            || Root::new(fixture, request, generation.clone()),
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
    assert_output(&sample, fixture, request, generation);
    sample
}

#[test]
fn generated_tick_choices_survive_all_root_modes_parents_and_cache_disabled_replay() {
    for fixture in [
        Fixture::Linear,
        Fixture::Facets,
        Fixture::RangeGroup,
        Fixture::Forest,
        Fixture::ForestCanvas,
    ] {
        for generation in [
            TickGeneration::target(if fixture.logarithmic() { 4 } else { 5 }).unwrap(),
            TickGeneration::maximum(3).unwrap(),
        ] {
            for request in [Request::Automatic, Request::Width, Request::Exact] {
                let cached = case(fixture, request, &generation, false);
                let uncached = case(fixture, request, &generation, true);
                equal_output(&cached, &uncached);
            }
        }
    }
}

pub(super) fn measure() {
    println!("tick_conditions,release,target128_vs_maximum12,linear_6_observations,three_interleaved_runs,exact_1200x800");
    let target = || {
        case(
            Fixture::Linear,
            Request::Exact,
            &TickGeneration::target(128).unwrap(),
            false,
        )
    };
    let maximum = || {
        case(
            Fixture::Linear,
            Request::Exact,
            &TickGeneration::maximum(12).unwrap(),
            false,
        )
    };
    let samples = measure_pair(["ticks_target128", "ticks_maximum12"], [&target, &maximum]);
    for pair in &samples[1..] {
        equal_output(&samples[0][0], &pair[0]);
        equal_output(&samples[0][1], &pair[1]);
    }
    let uncached = case(
        Fixture::Linear,
        Request::Exact,
        &TickGeneration::maximum(12).unwrap(),
        true,
    );
    equal_output(&samples[0][1], &uncached);
}
