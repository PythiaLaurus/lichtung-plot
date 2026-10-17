use crate::{Bounds, CategoryValues, CoefficientInput, DumbbellEndpoints, NumericValues, Plot};

pub(crate) fn sources() -> [Plot; 4] {
    let terms = CategoryValues::new("term", ["after_visit", "before_visit", "last_visit"]).unwrap();
    let estimate = NumericValues::new("estimate", [10.0, 20.0, 30.0]);
    let bounds = Bounds::new(
        NumericValues::new("lower", [9.0, 19.0, 29.0]),
        NumericValues::new("upper", [11.0, 21.0, 31.0]),
    );
    [
        Plot::point_interval(terms.clone(), estimate.clone(), bounds.clone())
            .zero_line()
            .erase(),
        Plot::coefficient(terms.clone(), CoefficientInput::estimates(estimate.clone())).erase(),
        Plot::forest(terms.clone(), estimate.clone(), bounds, 1.0).erase(),
        Plot::dumbbell(
            terms,
            estimate,
            NumericValues::new("right", [15.0, 18.0, 34.0]),
            DumbbellEndpoints::new(
                "Before",
                crate::Color::rgb(20, 80, 140),
                "After",
                crate::Color::rgb(180, 60, 40),
            )
            .unwrap(),
        )
        .erase(),
    ]
}
