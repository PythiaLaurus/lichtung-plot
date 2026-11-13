//! Occurrence policy is separate from authored intent and prepared axis storage.

pub(crate) struct PlotScopePolicy {
    pub(crate) significance_indices: Vec<usize>,
    pub(crate) axis_text: AxisTextVisibility,
    pub(crate) coordinated: CoordinatedDomains,
}

#[derive(Debug, Default)]
pub(crate) struct CoordinatedDomains {
    pub(crate) colors: Option<std::sync::Arc<Vec<String>>>,
    pub(crate) groups: Option<std::sync::Arc<Vec<String>>>,
}

pub(crate) struct AxisTextVisibility {
    pub(crate) titles: [bool; 2],
}

impl PlotScopePolicy {
    pub(crate) fn for_source(
        source: &crate::builder::PlotDescription,
        facet: Option<&crate::FacetCell>,
    ) -> Self {
        Self {
            significance_indices: source
                .significance_annotations()
                .iter()
                .enumerate()
                .filter(|(_, annotation)| match facet {
                    Some(facet) => annotation.facet_cell() == Some(facet),
                    None => true,
                })
                .map(|(index, _)| index)
                .collect(),
            axis_text: AxisTextVisibility {
                titles: [!source.has_facets(); 2],
            },
            coordinated: CoordinatedDomains::default(),
        }
    }
}
