use crate::{CategoryValues, NumericValues, Plot};

pub(crate) fn sources() -> [Plot; 3] {
    configured(false, false, None)
}

fn configured(annotated: bool, grouped: bool, limits: Option<(f64, f64)>) -> [Plot; 3] {
    fn finish<
        K: crate::kind::ExactFamily
            + crate::kind::Series
            + crate::kind::NumericY
            + crate::kind::Significance<crate::Pairwise>,
    >(
        mut plot: Plot<K>,
        annotated: bool,
        grouped: bool,
        limits: Option<(f64, f64)>,
    ) -> Plot {
        if grouped {
            plot = plot.series(
                CategoryValues::new(
                    "series",
                    [
                        "first", "first", "first", "first", "first", "first", "second", "second",
                    ],
                )
                .unwrap(),
            );
        }
        if let Some((lo, hi)) = limits {
            plot = plot.y_limits(lo, hi);
        }
        plot.significance(
            [crate::Significance::between(
                "after_visit",
                "before_visit",
                crate::SignificanceMark::PValue(0.0123),
            )]
            .into_iter()
            .filter(|_| annotated),
        )
        .erase()
    }
    let categories = CategoryValues::new(
        "category",
        [
            "after_visit",
            "after_visit",
            "after_visit",
            "after_visit",
            "before_visit",
            "before_visit",
            "after_visit",
            "after_visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [1.25, 1.25, 1.25, 9.75, -2.5, 3.125, 2.25, 4.5]);
    [
        finish(
            Plot::boxplot(categories.clone(), values.clone()),
            annotated,
            grouped,
            limits,
        ),
        finish(
            Plot::violin(categories.clone(), values.clone()),
            annotated,
            grouped,
            limits,
        ),
        finish(
            Plot::raincloud(crate::RaincloudInput::samples(categories, values)),
            annotated,
            grouped,
            limits,
        ),
    ]
}
