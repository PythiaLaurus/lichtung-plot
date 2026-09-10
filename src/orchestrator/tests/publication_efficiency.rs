use super::{test_directory, CategoryValues, NumericValues, Plot};
use crate::orchestrator::measurements::{self, STAGES};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Instant;

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
fn measure<R>(
    name: &str,
    construct: impl Fn() -> R,
    publish: impl Fn(R, &Path) -> crate::Output,
    separate: bool,
) {
    let directory = test_directory(name);
    let mut samples = Vec::new();
    let mut expected = None;
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
        println!(
            "admission,{name},{run},{},{},{}",
            observation.records, observation.visits, observation.bytes
        );
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
#[ignore = "release baseline; run explicitly with --release --ignored --nocapture --test-threads=1"]
#[expect(
    clippy::too_many_lines,
    reason = "one ordered baseline keeps the first-process workload explicit"
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
    println!("admission_header,case,run,records,processing_visits,font_source_scratch_bytes");
    // Keep scatter first: run 0 is the first font environment in this process.
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
        let theme = crate::Theme::Publication.resolve();
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
                    &theme.typography.caption,
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
