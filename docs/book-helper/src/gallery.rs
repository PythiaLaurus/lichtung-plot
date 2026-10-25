use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::content::Case;
use crate::{fail, run, Result, LANGUAGES};

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn cards(cases: &[Case], language: usize) -> Result<String> {
    let mut html = "<div class=\"gallery-grid\">\n".to_owned();
    for case in cases.iter().filter(|case| case.featured) {
        let title = escape(&case.titles[language]);
        writeln!(html, "<article class=\"gallery-card\"><a class=\"gallery-walkthrough\" href=\"#{}\"><img src=\"gallery/{}/{}\" alt=\"{title}\" loading=\"lazy\"><h3>{title}</h3><p>{}</p></a><p class=\"gallery-resources\"><a href=\"https://docs.rs/lichtung-plot/latest/lichtung_plot/{}\">API</a> · <a href=\"examples/{}.rs\">{}</a></p></article>",
            case.target, case.id, case.artifacts[0], escape(&case.purposes[language]), case.api, case.example,
            if language == 0 { "Complete source" } else { "完整源码" })?;
    }
    html.push_str("</div>\n");
    Ok(html)
}

pub fn landing(cases: &[Case]) -> Result<String> {
    let mut html = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>lichtung-plot — Gallery · 图库</title><link rel=\"stylesheet\" href=\"guide.css\"></head><body class=\"gallery-home\"><main><header><p class=\"gallery-eyebrow\">SCIENTIFIC FIGURES · 科学绘图</p><h1>lichtung-plot</h1><p class=\"gallery-intro\">Relationships. Distributions. Uncertainty.<br><span lang=\"zh-CN\">从关系、分布到不确定性，用完整图形理解数据。</span></p><nav><a href=\"en/index.html#gallery\" lang=\"en\">English guide →</a> · <a href=\"zh-CN/index.html#gallery\" lang=\"zh-CN\">中文指南 →</a></nav></header><section id=\"gallery\"><h2>Gallery · 图库</h2><p>Choose a figure, then explore its code. All datasets below are synthetic demonstrations.<br><span lang=\"zh-CN\">选择一张图，进入对应代码讲解。下列数据均为合成演示，不是研究结果。</span></p><div class=\"gallery-grid\">\n".to_owned();
    for case in cases.iter().filter(|case| case.featured) {
        writeln!(html, "<article class=\"gallery-card\"><img src=\"gallery/{}/{}\" alt=\"{} · {}\" loading=\"lazy\"><div class=\"gallery-caption\"><h3><span lang=\"en\">{}</span><br><span lang=\"zh-CN\">{}</span></h3><p><a href=\"en/index.html#{}\" lang=\"en\">Code walkthrough</a> · <a href=\"zh-CN/index.html#{}\" lang=\"zh-CN\">代码讲解</a></p></div></article>",
            case.id, case.artifacts[0], escape(&case.titles[0]), escape(&case.titles[1]), escape(&case.titles[0]), escape(&case.titles[1]), case.target, case.target)?;
    }
    let featured = cases.iter().filter(|case| case.featured).count();
    writeln!(html, "</div><p>{featured} selected scientific workflows; not yet an image catalog for all 25 plot families. Numeric and text-policy demonstrations live in the guide, not this showcase.<br><span lang=\"zh-CN\">当前精选 {featured} 个科学绘图场景，尚未完成全部 25 个图族的图像目录。数值与文本策略演示保留在指南正文，不作为首页展示图。</span></p></section></main></body></html>")?;
    Ok(html)
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
        let shared = stage.join("site/gallery").join(&case.id);
        fs::create_dir_all(&shared)?;
        for artifact in &case.artifacts {
            fs::copy(output.join(artifact), shared.join(artifact))?;
        }
        for language in LANGUAGES {
            let source = stage.join("docs").join(language).join("src");
            let assets = source.join("gallery").join(&case.id);
            fs::create_dir_all(&assets)?;
            for artifact in &case.artifacts {
                fs::copy(output.join(artifact), assets.join(artifact))?;
            }
        }
    }
    for (index, language) in LANGUAGES.iter().enumerate() {
        let generated = stage.join("docs").join(language).join("src/generated");
        fs::create_dir(&generated)?;
        fs::write(generated.join("gallery.md"), cards(cases, index)?)?;
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
