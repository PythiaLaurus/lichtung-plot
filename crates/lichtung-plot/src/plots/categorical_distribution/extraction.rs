use super::*;

use crate::RaincloudInput;

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
