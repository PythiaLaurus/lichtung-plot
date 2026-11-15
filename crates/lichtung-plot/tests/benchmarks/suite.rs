use super::test_directory;
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
        println!(
            "{prefix}_gutter,{name},{run},owner={:?},consumed={:?},requested={:?}",
            gutter.owner, gutter.consumed, gutter.requested
        );
    }
    for intrinsic in &work.intrinsics {
        println!(
            "{prefix}_intrinsic,{name},{run},scope={:?},counter={},root_round={:?}",
            intrinsic.scope,
            intrinsic.counter.name(),
            intrinsic.root_round
        );
    }
    for child in &work.children {
        println!("{prefix}_child,{name},{run},{child:?}");
    }
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
        "unexpected outcome: {name}: {:?}; roots={:?}; guides={:?}",
        result.as_ref().err(),
        observations.roots,
        observations.guides,
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

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
