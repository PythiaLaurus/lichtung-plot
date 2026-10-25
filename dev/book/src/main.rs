//! Assemble peer-language mdBooks from runnable public examples.

mod content;
mod gallery;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const LANGUAGES: [&str; 2] = ["en", "zh-CN"];
const MDBOOK_VERSION: &str = "mdbook v0.5.4";

fn fail(message: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(message.into()).into()
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(fail(format!("command failed ({status}): {command:?}")));
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        } else {
            return Err(fail(format!(
                "unsupported source entry: {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn stage(root: &Path) -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let stage = root.join(format!("target/book/work/{}-{nonce}", std::process::id()));
    fs::create_dir_all(&stage)?;
    copy_tree(
        &root.join("crates/lichtung-plot/examples"),
        &stage.join("crates/lichtung-plot/examples"),
    )?;
    for language in LANGUAGES {
        let book = stage.join("docs").join(language);
        copy_tree(&root.join("docs").join(language), &book)?;
        copy_tree(
            &root.join("crates/lichtung-plot/examples"),
            &book.join("src/examples"),
        )?;
    }
    Ok(stage)
}

fn require_mdbook() -> Result<()> {
    let output = Command::new("mdbook").arg("--version").output().map_err(|error| {
        fail(format!("mdbook is required ({error}); install it with cargo install mdbook --version 0.5.4 --locked"))
    })?;
    if !output.status.success() || String::from_utf8_lossy(&output.stdout).trim() != MDBOOK_VERSION
    {
        return Err(fail(format!(
            "expected {MDBOOK_VERSION}; install with cargo install mdbook --version 0.5.4 --locked"
        )));
    }
    Ok(())
}

fn build(root: &Path, stage: &Path) -> Result<()> {
    let destination = root.join("target/book/site");
    for language in LANGUAGES {
        run(Command::new("mdbook")
            .arg("build")
            .arg(stage.join("docs").join(language))
            .arg("--dest-dir")
            .arg(destination.join(language)))?;
    }
    fs::write(destination.join("index.html"),
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>lichtung-plot</title><h1>lichtung-plot</h1><nav><a lang=\"en\" href=\"en/introduction.html\">English</a> · <a lang=\"zh-CN\" href=\"zh-CN/introduction.html\">简体中文</a></nav></html>\n")?;
    println!("Books built: {}", destination.join("index.html").display());
    Ok(())
}

fn main() -> Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let [action] = arguments.as_slice() else {
        return Err(fail(
            "usage: cargo run -p lichtung-plot-book -- <check|gallery|build>",
        ));
    };
    if !["check", "gallery", "build"].contains(&action.as_str()) {
        return Err(fail(format!("unknown book command: {action}")));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let cases = content::check(&root)?;
    if action == "check" {
        return Ok(());
    }
    if action == "build" {
        require_mdbook()?;
    }
    let stage = stage(&root)?;
    gallery::assemble(&root, &stage, &cases)?;
    if action == "build" {
        build(&root, &stage)?;
    } else {
        println!("Fresh gallery sources and outputs: {}", stage.display());
    }
    Ok(())
}
