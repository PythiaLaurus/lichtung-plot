use crate::{HistogramInput, NumericValues, Plot};

thread_local! {
    pub(crate) static STAGES: std::cell::Cell<[[usize; 4]; 4]> = const { std::cell::Cell::new([[0; 4]; 4]) };
}

pub(crate) fn count(family: usize, stage: usize) {
    STAGES.with(|counts| {
        let mut next = counts.get();
        next[family][stage] += 1;
        counts.set(next);
    });
}

pub(crate) fn sources() -> [Plot; 4] {
    sources_faceted(None)
}

fn sources_faceted(facet: Option<&crate::FacetSpec>) -> [Plot; 4] {
    fn finish<K: crate::kind::ExactFamily + crate::kind::Facet>(
        mut plot: Plot<K>,
        facet: Option<&crate::FacetSpec>,
    ) -> Plot {
        if let Some(facet) = facet {
            plot = plot.facet(facet.clone());
        }
        plot.erase()
    }
    let x = NumericValues::new("sample", [1.0, 2.0, 4.0, 5.0, 7.0, 8.0]);
    [
        finish(
            Plot::histogram(HistogramInput::samples(x.clone()))
                .bins(crate::HistogramBins::Count(8))
                .stat(crate::HistogramStat::Density)
                .composite(crate::HistogramComposite::KdeAndRug(
                    crate::HistogramKde::Internal,
                )),
            facet,
        ),
        finish(
            Plot::density(x.clone())
                .hue(crate::CategoryValues::new("series", ["a", "a", "a", "b", "b", "b"]).unwrap()),
            facet,
        ),
        finish(Plot::qq(x.clone()), facet),
        finish(
            Plot::regression(
                x,
                NumericValues::new("response", [1.0, 4.0, 2.0, 8.0, 6.0, 9.0]),
            ),
            facet,
        ),
    ]
}
