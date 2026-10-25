use std::io::{self, ErrorKind};
use std::path::PathBuf;

/// Examples default to the working directory; one optional argument redirects output.
pub fn directory() -> io::Result<PathBuf> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments.next();
    if arguments.next().is_some() || directory.as_ref().is_some_and(|value| value.is_empty()) {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "expected at most one nonempty output directory: cargo run --example <name> -- [directory]",
        ));
    }
    let directory = match directory {
        Some(directory) => PathBuf::from(directory),
        None => std::env::current_dir()?,
    };
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}
