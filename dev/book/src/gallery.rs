use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::content::Case;
use crate::{fail, run, Result, LANGUAGES};

fn page(case: &Case, language: usize) -> Result<String> {
    let peer = LANGUAGES[1 - language];
    let title = &case.titles[language];
    let mut page = format!(
        "# {title}\n\n[{}](../{peer}/gallery-{}.html)\n\n{}\n\n![{title}](gallery/{}/{})\n\n",
        if language == 0 {
            "简体中文"
        } else {
            "English"
        },
        case.id,
        case.purposes[language],
        case.id,
        case.artifacts[0]
    );
    let labels = if language == 0 {
        ["Related chapter", "API reference", "Complete example", "Output files", "Run from the workspace root; the directory is caller-selected. The complete source uses examples/support/output.rs, shipped alongside it."]
    } else {
        ["相关章节", "API 参考", "完整示例", "输出文件", "从 workspace 根目录运行；输出目录由调用者指定。完整源码使用随附的 examples/support/output.rs。"]
    };
    writeln!(page, "[{}]({}) · [{}](https://docs.rs/lichtung-plot/latest/lichtung_plot/{}) · [{}](examples/{}.rs) · [{}](examples/support/output.rs)\n", labels[0], case.chapter, labels[1], case.api, labels[2], case.example, if language == 0 { "Output helper" } else { "输出辅助代码" })?;
    writeln!(page, "{}\n\n```sh\ncargo run --locked --offline -p lichtung-plot --example {} -- target/examples/{}\n```\n", labels[4], case.example, case.id)?;
    writeln!(page, "```rust\n{{{{#include ../../../crates/lichtung-plot/examples/{}.rs:example}}}}\n```\n\n## {}\n", case.example, labels[3])?;
    for artifact in &case.artifacts {
        writeln!(page, "- [{artifact}](gallery/{}/{artifact})", case.id)?;
    }
    Ok(page)
}

fn artifacts(directory: &Path, case: &Case) -> Result<()> {
    let mut actual = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<_>>>()?;
    actual.sort();
    let mut expected = case.artifacts.clone();
    expected.sort();
    if actual != expected {
        return Err(fail(format!(
            "{} produced different artifact paths: {actual:?}, expected {expected:?}",
            case.id
        )));
    }
    for (index, artifact) in case.artifacts.iter().enumerate() {
        let svg = fs::read_to_string(directory.join(artifact))?;
        if !svg.contains("<svg") || !svg.contains("</svg>") || (index == 0 && svg.contains("<text"))
        {
            return Err(fail(format!(
                "{} must publish a complete outlined SVG for gallery display",
                case.id
            )));
        }
    }
    Ok(())
}

pub fn assemble(root: &Path, stage: &Path, cases: &[Case]) -> Result<()> {
    fs::create_dir(stage.join("artifacts"))?;
    for case in cases {
        let output = stage.join("artifacts").join(&case.id);
        fs::create_dir(&output)?;
        run(Command::new("cargo")
            .current_dir(root)
            .args([
                "run",
                "--locked",
                "--offline",
                "-p",
                "lichtung-plot",
                "--example",
                &case.example,
                "--",
            ])
            .arg(&output))?;
        artifacts(&output, case)?;
        for (index, language) in LANGUAGES.iter().enumerate() {
            let source = stage.join("docs").join(language).join("src");
            let assets = source.join("gallery").join(&case.id);
            fs::create_dir_all(&assets)?;
            for artifact in &case.artifacts {
                fs::copy(output.join(artifact), assets.join(artifact))?;
            }
            fs::write(
                source.join(format!("gallery-{}.md", case.id)),
                page(case, index)?,
            )?;
            let summary = source.join("SUMMARY.md");
            let mut text = fs::read_to_string(&summary)?;
            writeln!(text, "  - [{}](gallery-{}.md)", case.titles[index], case.id)?;
            fs::write(summary, text)?;
        }
    }
    let compiler = Command::new("rustc").arg("--version").output()?;
    if !compiler.status.success() {
        return Err(fail("failed to record Rust toolchain"));
    }
    let environment = format!("OS: {}\nArchitecture: {}\nRust: {}\nFonts: installed system fonts; Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif. Exact resolved files are not exposed by the renderer; output is not a cross-environment byte baseline.\n", std::env::consts::OS, std::env::consts::ARCH, String::from_utf8_lossy(&compiler.stdout).trim());
    fs::write(stage.join("generation-environment.txt"), environment)?;
    fs::copy(root.join("Cargo.lock"), stage.join("generation-Cargo.lock"))?;
    Ok(())
}
