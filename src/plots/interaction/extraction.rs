use super::*;
use crate::{Bounds, CategoryValues, NumericValues};

pub(crate) fn source() -> Plot<crate::kind::Interaction> {
    Plot::interaction(
        CategoryValues::new("condition", ["b", "a", "c", "b", "c"]).unwrap(),
        NumericValues::new("estimate", [2.0, 3.0, 4.0, 5.0, 6.0]),
        CategoryValues::new("group", ["one", "one", "one", "two", "two"]).unwrap(),
        Bounds::new(
            NumericValues::new("lower", [1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericValues::new("upper", [3.0, 4.0, 5.0, 6.0, 7.0]),
        ),
    )
}
