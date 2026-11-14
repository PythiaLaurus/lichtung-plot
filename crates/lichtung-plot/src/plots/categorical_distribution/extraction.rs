use crate::{CategoryValues, NumericValues, Plot, RaincloudInput};

pub(crate) fn supplied() -> Plot {
    Plot::raincloud(RaincloudInput::geometry(crate::RaincloudGeometry {
        category_dimension: "category".into(),
        value_dimension: "value".into(),
        hue_dimension: Some("color".into()),
        category: CategoryValues::new("category", ["supplied"]).unwrap(),
        hue: Some(CategoryValues::new("color", ["one"]).unwrap()),
        q1: NumericValues::new("q1", [1.0]),
        median: NumericValues::new("median", [3.0]),
        q3: NumericValues::new("q3", [5.0]),
        density_category: CategoryValues::new("category", ["supplied"; 3]).unwrap(),
        density_hue: Some(CategoryValues::new("color", ["one"; 3]).unwrap()),
        density_y: NumericValues::new("y", [0.0, 3.0, 6.0]),
        normalized_width: NumericValues::new("width", [0.0, 0.8, 0.0]),
        point_category: CategoryValues::new("category", ["supplied"; 2]).unwrap(),
        point_hue: Some(CategoryValues::new("color", ["one"; 2]).unwrap()),
        point_y: NumericValues::new("y", [2.0, 4.0]),
        normalized_strip: NumericValues::new("strip", [0.25, 0.75]),
    }))
    .erase()
}
