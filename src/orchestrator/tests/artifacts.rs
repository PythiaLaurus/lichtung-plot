use super::{assert_no_staging_files, test_directory, write_atomic, TEMP_RESERVATION_CALLS};

#[test]
fn render_failure_preserves_existing_svg_and_removes_staging_files() {
    TEMP_RESERVATION_CALLS.with(|calls| calls.set(0));
    let dir = test_directory("failed-svg-publication");
    let base = dir.join("canvas");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let chart = crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0]),
        crate::NumericValues::new("y", [1.0]),
    );

    assert!(chart.svg(&base).is_err());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_eq!(TEMP_RESERVATION_CALLS.with(std::cell::Cell::get), 0);
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn outlined_svg_conversion_failure_happens_before_staging_reservation() {
    TEMP_RESERVATION_CALLS.with(|calls| calls.set(0));
    let dir = test_directory("failed-outlined-conversion");
    let base = dir.join("canvas");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let scene = crate::scene::RenderedScene::new(
        10,
        10,
        Some(crate::render::svg_backend::FinalizedSvgOutput::from_test_bytes(10, 10, b"not svg")),
    )
    .unwrap();

    let Err(error) = crate::Output::new(scene).svg(&base) else {
        panic!("malformed canonical bytes unexpectedly converted")
    };

    assert!(error.to_string().contains("parse svg for text outlining"));
    assert_eq!(TEMP_RESERVATION_CALLS.with(std::cell::Cell::get), 0);
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rasterization_failure_preserves_existing_png_and_removes_staging_files() {
    use crate::spec::{PngCompression, PngOptions, PngSizing};

    let dir = test_directory("failed-png-publication");
    TEMP_RESERVATION_CALLS.with(|calls| calls.set(0));
    let base = dir.join("canvas");
    let output = dir.join("canvas.png");
    std::fs::write(&output, b"existing png artifact").unwrap();
    let options = PngOptions {
        sizing: PngSizing::Scale(0.0),
        compression: PngCompression::Balanced,
    };

    assert!(crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .erase()
    .png_with_options(&base, options)
    .is_err());
    assert_eq!(std::fs::read(&output).unwrap(), b"existing png artifact");
    assert_eq!(TEMP_RESERVATION_CALLS.with(std::cell::Cell::get), 0);
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn publication_replaces_existing_file_without_backup_artifacts() {
    let dir = test_directory("successful-replacement");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    write_atomic(&output, b"replacement artifact").unwrap();

    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "replacement artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_failed_atomic_replace_preserves_locked_destination_without_backup() {
    use std::io::{Read, Seek};
    use std::os::windows::fs::OpenOptionsExt;

    let dir = test_directory("windows-failed-replacement");
    let output = dir.join("canvas.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let mut locked = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&output)
        .unwrap();
    let result = write_atomic(&output, b"replacement artifact");

    assert!(result.is_err());
    locked.rewind().unwrap();
    let mut preserved = String::new();
    locked.read_to_string(&mut preserved).unwrap();
    assert_eq!(preserved, "existing artifact");
    drop(locked);
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_atomic_replace_supports_long_destination_paths() {
    use std::os::windows::ffi::OsStrExt;

    let root = test_directory("windows-long-replacement");
    let mut dir = root.clone();
    while dir.as_os_str().encode_wide().count() < 280 {
        dir.push("long publication 目录");
        std::fs::create_dir(&dir).unwrap();
    }
    let output = dir.join("canvas report ü.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    write_atomic(&output, b"replacement artifact").unwrap();

    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "replacement artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_atomic_replace_supports_component_limit_destination() {
    let dir = test_directory("windows-component-limit");
    let output = dir.join(format!("{}.svg", "d".repeat(251)));
    std::fs::write(&output, "existing artifact").unwrap();

    write_atomic(&output, b"replacement artifact").unwrap();

    assert_eq!(
        std::fs::read_to_string(output).unwrap(),
        "replacement artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(windows)]
#[test]
fn windows_reservation_rejects_reserved_device_name_without_retrying() {
    use std::os::windows::ffi::OsStringExt;

    TEMP_RESERVATION_CALLS.with(|calls| calls.set(0));
    let dir = test_directory("windows-reserved-name");

    let error = write_atomic(&dir.join("NUL.svg"), b"artifact").unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid or reserved Windows file name"));
    let error = write_atomic(&dir.join("COM1:data"), b"artifact").unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid or reserved Windows file name"));
    let non_unicode = std::ffi::OsString::from_wide(&[
        u16::from(b'N'),
        u16::from(b'U'),
        u16::from(b'L'),
        u16::from(b'.'),
        0xD800,
    ]);
    let error = write_atomic(&dir.join(non_unicode), b"artifact").unwrap_err();
    assert!(error
        .to_string()
        .contains("invalid or reserved Windows file name"));
    for leaf in ["trailing.", "trailing ", "wild*.svg", "control\u{1}.svg"] {
        let error = write_atomic(&dir.join(leaf), b"artifact").unwrap_err();
        assert!(error
            .to_string()
            .contains("invalid or reserved Windows file name"));
    }
    assert_eq!(TEMP_RESERVATION_CALLS.with(std::cell::Cell::get), 0);
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn chained_outputs_publish_each_file_independently() {
    let dir = test_directory("successful-chained-publication");
    let svg = dir.join("canvas.svg");
    let png = dir.join("canvas.png");
    crate::Plot::scatter(
        crate::NumericValues::new("x", [1.0, 2.0, 3.0, 4.0]),
        crate::NumericValues::new("y", [1.0, 3.0, 2.0, 5.0]),
    )
    .erase()
    .svg(&svg)
    .unwrap()
    .png_preview(&png)
    .unwrap();

    assert!(std::fs::read_to_string(&svg).unwrap().starts_with("<svg"));
    assert!(std::fs::read(&png)
        .unwrap()
        .starts_with(&[0x89, b'P', b'N', b'G']));
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}
