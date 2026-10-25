use std::io::{self, ErrorKind};
use std::path::PathBuf;

/// Every example requires one caller-selected output directory.
pub fn directory() -> io::Result<PathBuf> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments.next().filter(|value| !value.is_empty());
    if arguments.next().is_some() || directory.is_none() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "supply exactly one output directory: cargo run --example <name> -- <directory>",
        ));
    }
    let directory = PathBuf::from(directory.expect("checked directory argument"));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}
