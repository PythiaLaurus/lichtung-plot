use std::path::Path;
#[cfg(any(feature = "png", windows))]
use std::path::PathBuf;

#[cfg(any(feature = "png", windows))]
use super::finish_committed_publication;
#[cfg(feature = "png")]
use super::reserve_staging_pair;
use super::{
    assert_no_staging_files, configured_chart, fixture, paths_resolve_to_same_destination,
    publish_staged, reserve_unique_sibling_temp_path, test_directory, with_temporary_artifacts,
    ComparisonAnnotation, Error, PlotKind,
};

#[test]
fn render_failure_preserves_existing_svg_and_removes_staging_files() {
    let dir = test_directory("failed-svg-publication");
    let output = dir.join("figure.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let data = fixture();
    let mut chart = configured_chart(&data, PlotKind::Scatter);
    let _ = chart.add_comparisons([ComparisonAnnotation::new("a", "b", 0.01)]);

    assert!(chart.render_to_svg(&output).is_err());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn output_identity_normalizes_relative_absolute_and_nonexistent_paths() {
    let current = std::env::current_dir().unwrap();
    let relative = Path::new("target/../normalized-output.svg");
    let absolute = current.join("normalized-output.svg");

    assert!(paths_resolve_to_same_destination(relative, &absolute).unwrap());
}

#[cfg(unix)]
#[test]
fn output_identity_resolves_existing_symlink_and_symlinked_parent() {
    use std::os::unix::fs::symlink;

    let dir = test_directory("output-symlink-identity");
    let real_parent = dir.join("real");
    std::fs::create_dir(&real_parent).unwrap();
    let real_file = real_parent.join("existing.svg");
    std::fs::write(&real_file, "existing").unwrap();
    let file_alias = dir.join("alias.svg");
    symlink(&real_file, &file_alias).unwrap();
    let parent_alias = dir.join("alias-parent");
    symlink(&real_parent, &parent_alias).unwrap();

    assert!(paths_resolve_to_same_destination(&real_file, &file_alias).unwrap());
    assert!(paths_resolve_to_same_destination(
        &real_parent.join("future.svg"),
        &parent_alias.join("future.svg")
    )
    .unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn output_identity_resolves_parent_after_symlink_in_filesystem_order() {
    use std::os::unix::fs::symlink;

    let dir = test_directory("output-symlink-parent-identity");
    let target_parent = dir.join("target");
    let target_child = target_parent.join("child");
    std::fs::create_dir_all(&target_child).unwrap();
    let parent_alias = dir.join("alias-parent");
    symlink(&target_child, &parent_alias).unwrap();

    assert!(paths_resolve_to_same_destination(
        &target_parent.join("future.svg"),
        &parent_alias.join("../future.svg")
    )
    .unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(all(feature = "png", unix))]
#[test]
fn combined_output_rejects_resolvable_symlink_aliases() {
    use std::os::unix::fs::symlink;

    let dir = test_directory("combined-symlink-alias");
    let output = dir.join("figure.svg");
    std::fs::write(&output, "existing").unwrap();
    let alias = dir.join("figure.png");
    symlink(&output, &alias).unwrap();
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);

    let error = chart
        .render_to_svg_and_png(&output, &alias)
        .unwrap_err()
        .to_string();

    assert!(error.contains("must be distinct"));
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "existing");
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(all(feature = "png", unix))]
#[test]
fn combined_output_rejects_parent_after_symlink_before_touching_destination() {
    use std::os::unix::fs::symlink;

    let dir = test_directory("combined-symlink-parent-alias");
    let target_parent = dir.join("target");
    let target_child = target_parent.join("child");
    std::fs::create_dir_all(&target_child).unwrap();
    let output = target_parent.join("figure.svg");
    std::fs::write(&output, "existing").unwrap();
    let parent_alias = dir.join("alias-parent");
    symlink(&target_child, &parent_alias).unwrap();
    let alias = parent_alias.join("../figure.svg");
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);

    let error = chart
        .render_to_svg_and_png(&output, &alias)
        .unwrap_err()
        .to_string();

    assert!(error.contains("must be distinct"));
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "existing");
    assert_no_staging_files(&target_parent);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn combined_output_rejects_lexically_equivalent_nonexistent_paths() {
    let dir = test_directory("combined-lexical-alias");
    let output = dir.join("figure.svg");
    let alias = dir.join(".").join("figure.svg");
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);

    let error = chart
        .render_to_svg_and_png(&output, &alias)
        .unwrap_err()
        .to_string();

    assert!(error.contains("must be distinct"));
    assert!(!output.exists());
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn finalization_failure_removes_staging_files_without_touching_destination() {
    let dir = test_directory("failed-finalization-cleanup");
    let output = dir.join("figure.svg");
    std::fs::write(&output, "existing artifact").unwrap();
    let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();

    let result: Result<(), Error> = with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, "partial staged artifact")?;
        Err(Error::Render("backend end failed".into()))
    });

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(&output).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn publication_rename_failure_removes_staging_files_and_preserves_destination() {
    let dir = test_directory("failed-rename-cleanup");
    let output = dir.join("figure.svg");
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("existing"), "existing artifact").unwrap();
    let staged = reserve_unique_sibling_temp_path(&output, "svg").unwrap();

    let result = with_temporary_artifacts(vec![staged.clone()], || {
        std::fs::write(&staged, "complete staged artifact")?;
        publish_staged(&staged, &output)
    });

    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(output.join("existing")).unwrap(),
        "existing artifact"
    );
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn rasterization_failure_preserves_existing_png_and_removes_staging_files() {
    use crate::spec::{PngCompression, PngOptions, PngSizing};

    let dir = test_directory("failed-png-publication");
    let output = dir.join("figure.png");
    std::fs::write(&output, b"existing png artifact").unwrap();
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let options = PngOptions {
        sizing: PngSizing::Scale(0.0),
        compression: PngCompression::Default,
    };

    assert!(chart.render_to_png_with_options(&output, &options).is_err());
    assert_eq!(std::fs::read(&output).unwrap(), b"existing png artifact");
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn combined_rasterization_failure_preserves_both_existing_artifacts() {
    use crate::spec::{PngCompression, PngOptions, PngSizing};

    let dir = test_directory("failed-combined-publication");
    let svg = dir.join("figure.svg");
    let png = dir.join("figure.png");
    std::fs::write(&svg, "existing svg").unwrap();
    std::fs::write(&png, b"existing png").unwrap();
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);
    let options = PngOptions {
        sizing: PngSizing::Scale(0.0),
        compression: PngCompression::Default,
    };

    assert!(chart
        .render_to_svg_and_png_with_options(&svg, &png, &options)
        .is_err());
    assert_eq!(std::fs::read_to_string(&svg).unwrap(), "existing svg");
    assert_eq!(std::fs::read(&png).unwrap(), b"existing png");
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "png")]
#[test]
fn successful_combined_publication_cleans_staging_and_backup_files() {
    let dir = test_directory("successful-combined-publication");
    let svg = dir.join("figure.svg");
    let png = dir.join("figure.png");
    std::fs::write(&svg, "old svg").unwrap();
    std::fs::write(&png, "old png").unwrap();
    let data = fixture();
    let chart = configured_chart(&data, PlotKind::Scatter);

    chart.render_to_svg_and_png(&svg, &png).unwrap();

    assert!(std::fs::read_to_string(&svg).unwrap().starts_with("<svg"));
    assert!(std::fs::read(&png)
        .unwrap()
        .starts_with(&[0x89, b'P', b'N', b'G']));
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(any(feature = "png", windows))]
#[test]
fn backup_cleanup_failure_does_not_reverse_publication_commit() {
    let backups = vec![(PathBuf::from("backup"), PathBuf::from("final"))];
    let mut attempts = 0;

    finish_committed_publication(&backups, |_| {
        attempts += 1;
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "simulated backup cleanup failure",
        ))
    });

    assert_eq!(attempts, 1);
}

#[cfg(feature = "png")]
#[test]
fn partial_staging_reservation_failure_cleans_first_path() {
    let dir = test_directory("failed-staging-reservation");
    let first = dir.join("figure.svg");
    let second = dir.join("missing-parent").join("figure.png");

    assert!(reserve_staging_pair(&first, "svg", &second, "png").is_err());
    assert_no_staging_files(&dir);
    std::fs::remove_dir_all(dir).unwrap();
}
