use super::*;

use crate::{CategoryValues, NumericValues, RaincloudInput};

pub(crate) fn sources(facet: Option<crate::FacetScales>) -> [Plot; 3] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet>(
        mut plot: Plot<K>,
        facet: Option<crate::FacetScales>,
    ) -> Plot {
        if let Some(scales) = facet {
            plot = plot.facet(
                crate::FacetSpec::column(
                    CategoryValues::new(
                        "panel",
                        [
                            "left", "left", "left", "left", "right", "right", "right", "right",
                        ],
                    )
                    .unwrap(),
                )
                .scales(scales),
            );
        }
        plot.size(1100, 640).erase()
    }
    let categories =
        CategoryValues::new("category", ["b", "b", "a", "a", "b", "b", "a", "a"]).unwrap();
    let values = NumericValues::new("value", [1.0, 4.0, 2.0, 7.0, 3.0, 6.0, 4.0, 9.0]);
    let hues = CategoryValues::new(
        "color",
        ["one", "one", "two", "two", "one", "one", "two", "two"],
    )
    .unwrap();
    [
        finish(
            Plot::boxplot(categories.clone(), values.clone())
                .series(hues.clone())
                .raw_overlay(crate::RawOverlay::Jittered),
            facet,
        ),
        finish(
            Plot::violin(categories.clone(), values.clone()).series(hues.clone()),
            facet,
        ),
        finish(
            Plot::raincloud(RaincloudInput::samples(categories, values)).series(hues),
            facet,
        ),
    ]
}

pub(crate) fn supplied() -> Plot {
    Plot::raincloud(RaincloudInput::geometry(crate::RaincloudGeometry {
        dimensions: crate::RaincloudDimensions {
            category: "category".into(),
            value: "value".into(),
            hue: Some("color".into()),
        },
        groups: vec![crate::RaincloudGroup {
            category: "supplied".into(),
            hue: Some("one".into()),
            density: vec![
                crate::RaincloudDensityPoint {
                    y: 0.0,
                    normalized_width: 0.0,
                },
                crate::RaincloudDensityPoint {
                    y: 3.0,
                    normalized_width: 0.8,
                },
                crate::RaincloudDensityPoint {
                    y: 6.0,
                    normalized_width: 0.0,
                },
            ],
            q1: 1.0,
            median: 3.0,
            q3: 5.0,
            points: vec![
                crate::RaincloudPoint {
                    y: 2.0,
                    normalized_strip: 0.25,
                },
                crate::RaincloudPoint {
                    y: 4.0,
                    normalized_strip: 0.75,
                },
            ],
        }],
    }))
    .erase()
}
