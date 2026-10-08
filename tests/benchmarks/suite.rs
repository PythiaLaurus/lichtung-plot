use super::{test_directory, CategoryValues, NumericValues, Plot};
use crate::orchestrator::measurements::{self, STAGES};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{Duration, Instant};

mod annotations;
mod fonts;
mod legends;
mod text;

pub(super) struct ArtifactBytes {
    pub svg: Vec<u8>,
    pub png: Vec<u8>,
    pub preserved_svg: Option<Vec<u8>>,
}

pub(super) struct CaseMeasurement {
    pub elapsed: Duration,
    pub setup: Duration,
    pub observations: measurements::Observations,
    pub shapes: usize,
    pub output: Option<ArtifactBytes>,
    pub error: Option<crate::Error>,
}

fn report_work(prefix: &str, name: &str, run: usize, work: &measurements::Observations) {
    for counter in measurements::Counter::ALL {
        println!(
            "{prefix}_counter,{name},{run},{},{}",
            counter.name(),
            work.counters[counter as usize]
        );
        println!(
            "{prefix}_capture_counter,{name},{run},{},{}",
            counter.name(),
            work.capture_counters[counter as usize]
        );
    }
    for root in &work.roots {
        println!("{prefix}_root,{name},{run},{root:?}");
    }
    for guide in &work.guides {
        println!("{prefix}_guide,{name},{run},{guide:?}");
    }
    for gutter in &work.gutters {
        println!("{prefix}_gutter,{name},{run},{gutter:?}");
    }
    for intrinsic in &work.intrinsics {
        println!("{prefix}_intrinsic,{name},{run},{intrinsic:?}");
    }
    for child in &work.children {
        println!("{prefix}_child,{name},{run},{child:?}");
    }
}

fn assert_stable_work(
    name: &str,
    work: &measurements::Observations,
    first: &measurements::Observations,
) {
    assert_eq!(work.counters, first.counters, "unstable counters: {name}");
    assert_eq!(
        work.capture_counters, first.capture_counters,
        "capture: {name}"
    );
    assert_eq!(work.calls, first.calls, "lifecycle calls: {name}");
    assert_eq!(
        work.producers, first.producers,
        "producer invocations: {name}"
    );
    let roots = |work: &measurements::Observations| {
        work.roots
            .iter()
            .map(|root| {
                (
                    root.ordinal,
                    root.heatmap_prefix_trials,
                    root.guides.clone(),
                    root.successful,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(roots(work), roots(first), "root trace: {name}");
    // Durations are observations, not deterministic guide identity.
    let guides = |work: &measurements::Observations| {
        work.guides
            .iter()
            .map(|guide| {
                (
                    guide.owner.clone(),
                    guide.root_round,
                    guide.edge,
                    guide.compact,
                    guide.phase,
                    guide.outcome,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(guides(work), guides(first), "guide trace: {name}");
    let gutters = |work: &measurements::Observations| {
        work.gutters
            .iter()
            .map(|gutter| {
                (
                    gutter.owner.clone(),
                    gutter.consumed.map(f64::to_bits),
                    gutter.requested.map(f64::to_bits),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(gutters(work), gutters(first), "gutter trace: {name}");
    let intrinsics = |work: &measurements::Observations| {
        work.intrinsics
            .iter()
            .map(|event| (event.scope.clone(), event.counter, event.root_round))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        intrinsics(work),
        intrinsics(first),
        "intrinsic trace: {name}"
    );
    let children = |work: &measurements::Observations| {
        work.children
            .iter()
            .map(|event| {
                (
                    event.owner.clone(),
                    event.phase,
                    event.root_round,
                    event.counter,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(children(work), children(first), "child trace: {name}");
}

/// `publish` must use the root's public outlined SVG terminal.
/// Semantic checks can inspect retained bytes and the complete observations.
pub(super) fn measure_case<R>(
    name: &str,
    construct: impl Fn() -> R,
    publish: impl Fn(R, &Path) -> Result<crate::Output, crate::Error>,
    expected_success: bool,
    preserve_text: bool,
) -> CaseMeasurement {
    let directory = test_directory(name);
    let svg = directory.join("scene.svg");
    let png = directory.join("scene.png");
    let preserved_svg = directory.join("text.svg");
    crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
    measurements::start();
    let start = Instant::now();
    let root = construct();
    let setup = start.elapsed();
    let result = publish(root, &svg).and_then(|output| output.png_preview(&png));
    let elapsed = start.elapsed();
    let observations = measurements::finish();
    let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
    assert_eq!(
        result.is_ok(),
        expected_success,
        "unexpected outcome: {name}: {:?}",
        result.as_ref().err()
    );
    // Replay, reads, and caller assertions are outside both timing and observation.
    let (output, error) = match result {
        Ok(output) => {
            if preserve_text {
                output.svg_preserve_text(&preserved_svg).unwrap();
            }
            (
                Some(ArtifactBytes {
                    svg: std::fs::read(&svg).unwrap(),
                    png: std::fs::read(&png).unwrap(),
                    preserved_svg: preserve_text.then(|| std::fs::read(&preserved_svg).unwrap()),
                }),
                None,
            )
        }
        Err(error) => {
            assert!(!svg.exists() && !png.exists() && !preserved_svg.exists());
            (None, Some(error))
        }
    };
    super::assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
    CaseMeasurement {
        elapsed,
        setup,
        observations,
        shapes,
        output,
        error,
    }
}

/// Three interleaved runs, with alternating order and separate failure summaries.
/// Closures may use different root types and assert actual captured guide choices.
pub(super) fn measure_pair(
    names: [&str; 2],
    cases: [&dyn Fn() -> CaseMeasurement; 2],
) -> Vec<[CaseMeasurement; 2]> {
    let mut samples: Vec<[CaseMeasurement; 2]> = Vec::new();
    for run in 0..3 {
        let pair = if run % 2 == 0 {
            [cases[0](), cases[1]()]
        } else {
            let second = cases[1]();
            [cases[0](), second]
        };
        for (name, sample) in names.iter().zip(&pair) {
            let outcome = if sample.error.is_none() {
                "success"
            } else {
                "failure"
            };
            println!(
                "paired_sample,{name},{run},{outcome},{:.6},{:.6},{:?},{:?},{},{}",
                sample.elapsed.as_secs_f64() * 1000.0,
                sample.setup.as_secs_f64() * 1000.0,
                sample.observations.times,
                sample.observations.calls,
                sample.observations.producers,
                sample.shapes,
            );
            if let Some(error) = &sample.error {
                println!("paired_failure,{name},{run},{error:?}");
            }
            report_work("paired", name, run, &sample.observations);
            if let Some(bytes) = &sample.output {
                let mut reader = png::Decoder::new(std::io::Cursor::new(&bytes.png))
                    .read_info()
                    .unwrap();
                let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
                let info = reader.next_frame(&mut pixels).unwrap();
                println!(
                    "paired_hash,{name},{run},{},{},{}",
                    hash(&bytes.svg),
                    hash(&bytes.png),
                    hash(&pixels[..info.buffer_size()])
                );
                if let Some(svg) = &bytes.preserved_svg {
                    println!("paired_preserved_hash,{name},{run},{}", hash(svg));
                }
            }
        }
        if let Some(first) = samples.first() {
            for (index, sample) in pair.iter().enumerate() {
                assert_stable_work(
                    names[index],
                    &sample.observations,
                    &first[index].observations,
                );
                assert_eq!(sample.shapes, first[index].shapes);
            }
        }
        samples.push(pair);
    }
    for (index, name) in names.iter().enumerate() {
        for success in [true, false] {
            for (column, stage) in ["end_to_end", "constructor_first"]
                .into_iter()
                .chain(STAGES)
                .enumerate()
            {
                let mut values = samples
                    .iter()
                    .map(|pair| &pair[index])
                    .filter(|sample| sample.error.is_none() == success)
                    .map(|sample| match column {
                        0 => sample.elapsed,
                        1 => sample.setup,
                        _ => sample.observations.times[column - 2],
                    })
                    .collect::<Vec<_>>();
                values.sort_unstable();
                if let Some(min) = values.first() {
                    let middle = values.len() / 2;
                    let median = if values.len() % 2 == 0 {
                        (values[middle - 1] + values[middle]) / 2
                    } else {
                        values[middle]
                    };
                    let outcome = if success { "success" } else { "failure" };
                    println!(
                        "paired_summary,{name},{outcome},{stage},{},{:.6},{:.6},{:.6}",
                        values.len(),
                        min.as_secs_f64() * 1000.0,
                        median.as_secs_f64() * 1000.0,
                        values.last().unwrap().as_secs_f64() * 1000.0
                    );
                }
            }
        }
    }
    samples
}

#[test]
fn paired_measurement_contract() {
    let samples = measure_pair(
        ["helper_success", "helper_failure"],
        [
            &|| {
                measure_case(
                    "helper_success",
                    || {
                        Plot::scatter(
                            NumericValues::new("Predictor", [1.0, 2.0]),
                            NumericValues::new("Response", [2.0, 1.0]),
                        )
                        .size(320, 240)
                    },
                    |root, path| root.svg(path),
                    true,
                    true,
                )
            },
            &|| {
                measure_case(
                    "helper_failure",
                    || (),
                    |(), _| Err(crate::Error::Render("expected rejection".into())),
                    false,
                    false,
                )
            },
        ],
    );
    assert_eq!(samples.len(), 3);
    for [success, failure] in samples {
        let output = success.output.unwrap();
        assert!(!output.svg.is_empty() && !output.png.is_empty());
        assert!(String::from_utf8(output.preserved_svg.unwrap())
            .unwrap()
            .contains("Predictor"));
        assert!(failure.output.is_none());
        assert!(
            matches!(failure.error, Some(crate::Error::Render(message)) if message == "expected rejection")
        );
        assert_eq!(failure.observations.calls, [0; 8]);
    }
}

fn scatter() -> Plot<crate::kind::Scatter> {
    Plot::scatter(
        NumericValues::new("Predictor", (0..100).map(f64::from)),
        NumericValues::new("Response", (0..100).map(|i| f64::from((i * 37) % 101))),
    )
    .size(960, 720)
}

fn bar() -> Plot<crate::kind::HorizontalBar> {
    Plot::horizontal_bar(
        CategoryValues::new(
            "Treatment",
            [
                "Control population with standard observation",
                "Treatment population after extended exposure",
                "Recovery population with repeated measurements",
            ],
        )
        .unwrap(),
        crate::BarInput::estimates(NumericValues::new("Response", [12.0, 19.0, 15.0])),
    )
    .size(960, 720)
}

fn publish_plot<K: crate::kind::Marker>(plot: Plot<K>, path: &Path) -> crate::Output {
    plot.svg(path).unwrap()
}

fn caption(paragraphs: bool) -> String {
    let sentence = "Measured responses remained stable across independent trials under controlled conditions. ";
    let result = if paragraphs {
        [sentence.repeat(7), sentence.repeat(7), sentence.repeat(6)].join("\n")
    } else {
        sentence.repeat(20)
    };
    assert_eq!(result.split_whitespace().count(), 200);
    result
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

// Constructors stay generic so Plot, Canvas, and Matrix exercise their real roots.
#[expect(
    clippy::too_many_lines,
    reason = "one publication lifecycle keeps timing separate from work and artifact assertions"
)]
fn measure<R>(
    name: &str,
    construct: impl Fn() -> R,
    publish: impl Fn(R, &Path) -> crate::Output,
    separate: bool,
) {
    let directory = test_directory(name);
    let mut samples = Vec::new();
    let mut expected = None;
    let mut first_work = None;
    for run in 0..3 {
        let svg = directory.join("scene.svg");
        let png = directory.join("scene.png");
        crate::font::canonical::FONT_SHAPE_CALLS.with(|calls| calls.set(0));
        measurements::start();
        let start = Instant::now();
        let root = construct();
        let setup = start.elapsed();
        let mut output = publish(root, &svg);
        output = output.png_preview(&png).unwrap();
        // The first operation plus nine more, versus nine terminals on its Output.
        if name == "separate10" || name == "reuse10" {
            for _ in 1..10 {
                if separate {
                    output = publish(construct(), &svg);
                } else {
                    output = output.svg(&svg).unwrap();
                }
                output = output.png_preview(&png).unwrap();
            }
        }
        let elapsed = start.elapsed();
        let observation = measurements::finish();
        let shapes = crate::font::canonical::FONT_SHAPE_CALLS.with(std::cell::Cell::get);
        drop(output);
        let mut row = vec![elapsed.as_secs_f64() * 1000.0, setup.as_secs_f64() * 1000.0];
        row.extend(
            observation
                .times
                .iter()
                .map(|time| time.as_secs_f64() * 1000.0),
        );
        println!(
            "sample,{name},{run},{}",
            row.iter()
                .map(|value| format!("{value:.6}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        println!(
            "calls,{name},{run},{},{},{},{},{},{},{},{},{},{}",
            observation.calls[0],
            observation.calls[1],
            observation.calls[2],
            observation.calls[3],
            observation.calls[4],
            observation.calls[5],
            observation.calls[6],
            observation.calls[7],
            observation.producers,
            shapes
        );
        samples.push(row);
        report_work("publication", name, run, &observation);
        if let Some((first, first_shapes)) = &first_work {
            assert_stable_work(name, &observation, first);
            assert_eq!(shapes, *first_shapes, "unstable shapes: {name}");
        } else {
            first_work = Some((observation, shapes));
        }
        // File reads, SHA256, and PNG decode are deliberately outside timing.
        let svg_bytes = std::fs::read(&svg).unwrap();
        let png_bytes = std::fs::read(&png).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(&png_bytes))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        let hashes = (
            hash(&svg_bytes),
            hash(&png_bytes),
            hash(&pixels[..info.buffer_size()]),
        );
        if let Some(expected) = &expected {
            assert_eq!(&hashes, expected, "unstable bytes for {name}");
        } else {
            println!(
                "hash,{name},{},{},{},{},{},{},{},{:?},{:?}",
                hashes.0,
                hashes.1,
                hashes.2,
                svg_bytes.len(),
                png_bytes.len(),
                info.width,
                info.height,
                info.color_type,
                info.bit_depth
            );
            expected = Some(hashes);
        }
    }
    for (column, stage) in std::iter::once("end_to_end")
        .chain(std::iter::once("constructor_first"))
        .chain(STAGES)
        .enumerate()
    {
        let mut values = samples.iter().map(|row| row[column]).collect::<Vec<_>>();
        values.sort_by(f64::total_cmp);
        println!("summary,{name},{stage},{:.6},{:.6}", values[0], values[1]);
    }
    super::assert_no_staging_files(&directory);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn matrix_publication_feedback_produces_complete_output() {
    let sample = measure_case(
        "matrix4_feedback_storage",
        || {
            Plot::matrix(
                (0..4)
                    .map(|j| {
                        NumericValues::new(
                            format!("Variable {j}"),
                            (0..100).map(move |i| f64::from((i * (j * 12 + 7)) % 101)),
                        )
                    })
                    .collect(),
            )
            .size(1400, 1400)
        },
        |matrix, path| matrix.svg(path),
        true,
        false,
    );
    assert!(sample.error.is_none());
    assert!(sample.output.is_some());
}

#[test]
#[ignore = "release baseline; run explicitly with --release --ignored --nocapture --test-threads=1"]
#[expect(
    clippy::too_many_lines,
    reason = "one ordered baseline preserves the original workload sequence"
)]
fn publication_efficiency_measurements() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "baseline requires --release"
    );
    println!("conditions,release,three_runs,no_warmup,OS_cache_uncontrolled,PNG_source_balanced,SVG_outlined");
    println!(
        "sample_header,case,run,end_to_end_ms,constructor_first_ms,{}",
        STAGES.map(|stage| format!("{stage}_ms")).join(",")
    );
    println!("calls_header,case,run,roots,root_rounds,captures,null_validations,publications,svg_conversions,png_conversions,svg_replays,producer_invocations,font_shape_calls");
    println!("hash_header,case,svg_sha256,png_sha256,decoded_sha256,svg_bytes,png_bytes,width,height,color,depth");
    println!("summary_header,case,stage,min_ms,median_ms");
    // Scatter is first within this control, not necessarily within the test process.
    measure("scatter100", scatter, publish_plot, false);
    measure("long_label_bar", bar, publish_plot, false);
    measure(
        "caption200_1para",
        || scatter().caption(caption(false)).size(1200, 1000),
        publish_plot,
        false,
    );
    measure(
        "caption200_3para",
        || scatter().caption(caption(true)).size(1200, 1000),
        publish_plot,
        false,
    );
    measure(
        "facets2x2",
        || {
            scatter()
                .facet(
                    crate::FacetSpec::grid(
                        CategoryValues::new(
                            "Row",
                            (0..100).map(|i| if i % 2 == 0 { "A" } else { "B" }),
                        )
                        .unwrap(),
                        CategoryValues::new(
                            "Column",
                            (0..100).map(|i| if i % 4 < 2 { "C" } else { "D" }),
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                )
                .size(1200, 1000)
        },
        publish_plot,
        false,
    );
    measure(
        "mixed_canvas",
        || {
            crate::Canvas::grid(1, 2)
                .unwrap()
                .panel(0, 0, scatter())
                .unwrap()
                .panel(0, 1, bar())
                .unwrap()
                .size(1600, 800)
        },
        |canvas, path| canvas.svg(path).unwrap(),
        false,
    );
    measure(
        "matrix4",
        || {
            Plot::matrix(
                (0..4)
                    .map(|j| {
                        NumericValues::new(
                            format!("Variable {j}"),
                            (0..100).map(move |i| f64::from((i * (j * 12 + 7)) % 101)),
                        )
                    })
                    .collect(),
            )
            .size(1400, 1400)
        },
        |matrix, path| matrix.svg(path).unwrap(),
        false,
    );
    measure("separate10", scatter, publish_plot, true);
    measure("reuse10", scatter, publish_plot, false);

    for run in 0..3 {
        let start = Instant::now();
        let theme = crate::Theme::Publication.resolve().unwrap();
        theme.prepare_font_environment().unwrap();
        println!(
            "font_probe,{run},fresh_environment,{:.6}",
            start.elapsed().as_secs_f64() * 1000.0
        );
        for (label, text) in [
            ("first_query", "Measured responses"),
            ("new_query", "Independent trials"),
            ("repeat_query", "Independent trials"),
        ] {
            let start = Instant::now();
            theme
                .plan_text(
                    text,
                    &theme.typography.root.caption,
                    768.0,
                    crate::layout::PhysicalTextAlignment::Start,
                    "baseline font probe",
                    false,
                )
                .unwrap();
            println!(
                "font_probe,{run},{label},{:.6}",
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
}
