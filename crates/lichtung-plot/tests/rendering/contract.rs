use lichtung_plot::{BarInput, Bounds, CategoryValues, NumericValues, Plot};

struct IntervalData {
    category: CategoryValues,
    value: NumericValues,
    lower: NumericValues,
    upper: NumericValues,
}

fn identity_interval_plot(data: &IntervalData, horizontal: bool) -> Plot {
    if horizontal {
        Plot::horizontal_bar(
            data.category.clone(),
            BarInput::estimates_with_error_bars(
                data.value.clone(),
                Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        )
        .erase()
    } else {
        Plot::bar(
            data.category.clone(),
            BarInput::estimates_with_error_bars(
                data.value.clone(),
                Bounds::new(data.lower.clone(), data.upper.clone()),
            ),
        )
        .erase()
    }
}

#[test]
fn identity_bar_rejects_invalid_endpoint_order_during_preflight() {
    let data = IntervalData {
        category: CategoryValues::new("category", ["a"]).unwrap(),
        value: NumericValues::new("value", [2.0]),
        lower: NumericValues::new("lower", [3.0]),
        upper: NumericValues::new("upper", [4.0]),
    };
    let error = identity_interval_plot(&data, false)
        .preflight()
        .unwrap_err();

    assert!(error.to_string().contains("lower <= value <= upper"));
}
