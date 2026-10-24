use super::*;

#[test]
fn marginal_legend_omits_hues_from_discarded_nonfinite_rows() {
    Plot::marginal(
        NumericValues::new("x", [1.0, 1.2, 1.4, f64::NAN]),
        NumericValues::new("y", [2.0, 2.1, 2.3, 4.0]),
    )
    .hue(CategoryValues::new("group", ["kept", "kept", "kept", "\u{10FFFF}"]).unwrap())
    .preflight()
    .unwrap();
}

#[test]
fn marginal_same_name_validation_ignores_dropped_rows() {
    Plot::marginal(
        NumericValues::nullable("x", [Some(1.0), Some(2.0), Some(3.0), None]),
        NumericValues::new("y", [2.0, 3.0, 5.0, 8.0]),
    )
    .group(CategoryValues::new("identity", ["kept", "kept", "kept", "dropped group"]).unwrap())
    .hue(CategoryValues::new("identity", ["kept", "kept", "kept", "dropped hue"]).unwrap())
    .preflight()
    .unwrap();
}

#[test]
fn marginal_reports_insufficient_retained_pair_with_group_and_hue() {
    let error = Plot::marginal(
        NumericValues::nullable("x", [Some(1.0), Some(2.0), Some(3.0), Some(4.0), None]),
        NumericValues::new("y", [2.0, 3.0, 5.0, 6.0, 7.0]),
    )
    .group(CategoryValues::new("group", ["g1", "g1", "g1", "g2", "g2"]).unwrap())
    .hue(CategoryValues::new("hue", ["h1", "h1", "h1", "h2", "h2"]).unwrap())
    .preflight()
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("x distribution for graphical group \"g2\" with hue \"h2\" requires at least 2"));
}

#[test]
fn matrix_pair_error_identifies_group_and_hue() {
    let error = Plot::matrix(vec![
        NumericValues::new("a", [1.0, 2.0, 3.0]),
        NumericValues::new("b", [2.0, 3.0, 4.0]),
        NumericValues::new("c", [3.0, 4.0, 5.0]),
    ])
    .group(CategoryValues::new("group", ["small", "valid", "valid"]).unwrap())
    .hue(CategoryValues::new("hue", ["red", "blue", "blue"]).unwrap())
    .preflight()
    .unwrap_err();
    assert!(error.to_string().contains("column \"a\" in graphical group \"group\"=\"small\" with hue \"hue\"=\"red\" requires at least 2 observations"));
}
