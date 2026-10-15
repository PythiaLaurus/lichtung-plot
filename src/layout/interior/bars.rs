use crate::{BarInput, CategoryValues, NumericValues, Plot};

pub(crate) fn sources() -> [Plot; 4] {
    configured_sources(false)
}

fn configured_sources(annotated: bool) -> [Plot; 4] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Significance<crate::Pairwise>>(
        plot: Plot<K>,
        annotated: bool,
    ) -> Plot {
        plot.significance(
            [crate::Significance::between(
                "after_visit",
                "before_visit",
                crate::SignificanceMark::OneStar,
            )]
            .into_iter()
            .filter(|_| annotated),
        )
        .erase()
    }
    let attached = || {
        [
            crate::Significance::at("after_visit", crate::SignificanceMark::PValue(0.0123)),
            crate::Significance::at("before_visit", crate::SignificanceMark::TwoStars),
        ]
        .into_iter()
        .filter(|_| annotated)
    };
    let categories = CategoryValues::new(
        "category",
        [
            "after_visit",
            "before_visit",
            "after_visit",
            "before_visit",
            "after_visit",
            "before_visit",
        ],
    )
    .unwrap();
    let values = NumericValues::new("value", [-2.0, 3.0, -4.0, 7.0, 5.0, -1.0]);
    let series = CategoryValues::new(
        "series",
        ["first", "first", "second", "second", "third", "third"],
    )
    .unwrap();
    [
        finish(
            Plot::bar(categories.clone(), BarInput::estimates(values.clone()))
                .series(series.clone())
                .significance(attached()),
            annotated,
        ),
        finish(
            Plot::horizontal_bar(categories.clone(), BarInput::estimates(values.clone()))
                .series(series.clone())
                .color_by_sign()
                .zero_line()
                .significance(attached()),
            annotated,
        ),
        finish(
            Plot::stacked_bar(categories.clone(), values.clone(), series.clone()),
            annotated,
        ),
        finish(
            Plot::stacked_horizontal_bar(categories, values, series),
            annotated,
        ),
    ]
}
