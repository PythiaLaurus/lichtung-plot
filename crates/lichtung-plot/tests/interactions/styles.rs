use super::*;

#[test]
fn interaction_duplicate_identity_includes_condition_group_and_hue() {
    let condition = CategoryValues::new("condition", ["Before", "Before", "Before"]).unwrap();
    let group = CategoryValues::new("group", ["g1", "g1", "g1"]).unwrap();
    let hue = CategoryValues::new("hue", ["h1", "h2", "h1"]).unwrap();
    let estimate = NumericValues::new("estimate", [1.0, 2.0, 3.0]);
    let lower = NumericValues::new("lower", [0.8, 1.8, 2.8]);
    let upper = NumericValues::new("upper", [1.2, 2.2, 3.2]);
    let error = Plot::interaction(condition, estimate, group, Bounds::new(lower, upper))
        .hue(hue)
        .preflight()
        .unwrap_err()
        .to_string();

    assert!(error.contains("duplicate (condition, group, hue)") && error.contains("source row 2"));
}
