use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::content::Case;
use crate::{example_extension, examples_path, fail, run, Result, LANGUAGES, PYTHON_PACKAGE};

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
        let api = if case.package == PYTHON_PACKAGE {
            "#python".to_owned()
        } else {
            format!(
                "https://docs.rs/{}/latest/{}/{}",
                case.package,
                case.package.replace('-', "_"),
                case.api
            )
        };
        writeln!(html, "<article class=\"gallery-card\"><a class=\"gallery-walkthrough\" href=\"#{}\"><img src=\"gallery/{}/{}\" alt=\"{title}\" loading=\"lazy\"><h3>{title}</h3><p>{}</p></a><p class=\"gallery-resources\"><a href=\"{api}\">API</a> · <a href=\"examples/{}/{}.{}\">{}</a></p></article>",
            case.target, case.id, case.artifacts[0], escape(&case.purposes[language]), case.package, case.example, example_extension(&case.package),
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
    let references = cases.iter().filter(|case| !case.featured).count();
    writeln!(html, "</div><p>{featured} scientific composition showcases. The guides include {references} reference workflows with runnable source and declared outputs.<br><span lang=\"zh-CN\">这里展示 {featured} 个科学合成绘图场景；指南另含 {references} 个参考流程，均提供可运行源码与声明的输出。</span></p></section></main></body></html>")?;
    Ok(html)
}

fn png_crc(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 == 0 { 0 } else { 0xedb8_8320 };
        }
    }
    !crc
}

pub(crate) fn png(bytes: &[u8]) -> bool {
    // Validate the header and chunk container, not decoded pixels or a golden image.
    let Some(header) = bytes.get(..33) else {
        return false;
    };
    let width = u32::from_be_bytes(header[16..20].try_into().expect("four-byte PNG width"));
    let height = u32::from_be_bytes(header[20..24].try_into().expect("four-byte PNG height"));
    if &header[..8] != b"\x89PNG\r\n\x1a\n"
        || &header[8..16] != b"\0\0\0\rIHDR"
        || !(1..=0x7fff_ffff).contains(&width)
        || !(1..=0x7fff_ffff).contains(&height)
        || !matches!(
            (header[25], header[24]),
            (0, 1 | 2 | 4 | 8 | 16) | (2 | 4 | 6, 8 | 16) | (3, 1 | 2 | 4 | 8)
        )
        || header[26] != 0
        || header[27] != 0
        || header[28] > 1
    {
        return false;
    }
    let mut offset = 8_usize;
    let mut image_data = false;
    while let Some(prefix) = bytes.get(offset..).and_then(|tail| tail.get(..8)) {
        let Ok(length) = usize::try_from(u32::from_be_bytes(
            prefix[..4].try_into().expect("four-byte PNG chunk length"),
        )) else {
            return false;
        };
        let Some(end) = offset
            .checked_add(12)
            .and_then(|base| base.checked_add(length))
        else {
            return false;
        };
        let Some(chunk) = bytes.get(offset..end) else {
            return false;
        };
        if png_crc(&chunk[4..chunk.len() - 4]).to_be_bytes() != chunk[chunk.len() - 4..] {
            return false;
        }
        match &prefix[4..8] {
            b"IHDR" if offset != 8 => return false,
            b"IDAT" => image_data |= length > 0,
            b"IEND" => return image_data && length == 0 && end == bytes.len(),
            _ => {}
        }
        offset = end;
    }
    false
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
        let file = directory.join(artifact);
        if !fs::symlink_metadata(&file)?.is_file() {
            return Err(fail(format!("artifact must be a regular file: {artifact}")));
        }
        let bytes = fs::read(&file)?;
        match file.extension().and_then(|ext| ext.to_str()) {
            Some("svg") => {
                let svg = std::str::from_utf8(&bytes)?;
                if !svg.contains("<svg")
                    || !svg.contains("</svg>")
                    || (index == 0 && svg.contains("<text"))
                {
                    return Err(fail(format!(
                        "{} must publish a complete outlined SVG for gallery display",
                        case.id
                    )));
                }
            }
            Some("png") if index > 0 && png(&bytes) => {}
            _ => {
                return Err(fail(format!(
                    "{} must publish complete SVG/PNG artifacts with an outlined SVG first",
                    case.id
                )));
            }
        }
    }
    Ok(())
}

fn selected_python(cases: &[Case]) -> Result<Option<PathBuf>> {
    if !cases.iter().any(|case| case.package == PYTHON_PACKAGE) {
        return Ok(None);
    }
    let selected = std::env::var_os("LICHTUNG_PYTHON").ok_or_else(|| {
        fail("Python gallery cases require LICHTUNG_PYTHON to select an absolute interpreter with an already-installed lichtung-plot wheel; book assembly never installs it")
    })?;
    let interpreter = PathBuf::from(selected);
    if !interpreter.is_absolute() || !interpreter.is_file() {
        return Err(fail(
            "LICHTUNG_PYTHON must be an absolute existing interpreter with an already-installed lichtung-plot wheel",
        ));
    }
    // Preserve the virtual environment's path, not its executable symlink target.
    // Package/prefix identity is verified separately by the calling workflow.
    Ok(Some(interpreter))
}

pub fn assemble(root: &Path, stage: &Path, cases: &[Case]) -> Result<()> {
    let python = selected_python(cases)?;
    fs::create_dir(stage.join("artifacts"))?;
    for case in cases {
        let output = stage.join("artifacts").join(&case.id);
        fs::create_dir(&output)?;
        if case.package == PYTHON_PACKAGE {
            let interpreter = python
                .as_ref()
                .ok_or_else(|| fail("Python gallery cases require LICHTUNG_PYTHON"))?;
            let script = root.join(examples_path(&case.package)).join(format!(
                "{}.{}",
                case.example,
                example_extension(&case.package)
            ));
            run(Command::new(interpreter)
                .args(["-I", "-B"])
                .arg(script)
                .arg(&output)
                .current_dir(&output))?;
        } else {
            run(Command::new("cargo")
                .current_dir(root)
                .args([
                    "run",
                    "--locked",
                    "--offline",
                    "-p",
                    &case.package,
                    "--example",
                    &case.example,
                    "--",
                ])
                .arg(&output))?;
        }
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
    let mut environment = format!("OS: {}\nArchitecture: {}\nRust: {}\nFonts: installed system fonts; Noto Sans, Noto Sans CJK SC, DejaVu Sans, Liberation Sans, Arial, sans-serif. Exact resolved files are not exposed by the renderer; output is not a cross-environment byte baseline.\n", std::env::consts::OS, std::env::consts::ARCH, String::from_utf8_lossy(&compiler.stdout).trim());
    if let Some(interpreter) = python {
        writeln!(
            environment,
            "Python interpreter: {} (-I -B; installed-package identity is a separate preflight)",
            interpreter.display()
        )?;
    }
    fs::write(stage.join("generation-environment.txt"), environment)?;
    fs::copy(root.join("Cargo.lock"), stage.join("generation-Cargo.lock"))?;
    Ok(())
}
