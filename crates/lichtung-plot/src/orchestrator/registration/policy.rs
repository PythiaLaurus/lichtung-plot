//! Occurrence policy is separate from authored intent and prepared axis storage.

pub(crate) struct PlotScopePolicy {
    pub(crate) significance_indices: Vec<usize>,
    pub(crate) axis_text: AxisTextVisibility,
    pub(crate) gutter_owner: GutterOwner,
    pub(crate) coordinated: CoordinatedDomains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GutterOwner {
    Root,
    Composition,
}

#[derive(Debug, Default)]
pub(crate) struct CoordinatedDomains {
    pub(crate) colors: Option<std::sync::Arc<Vec<String>>>,
    pub(crate) groups: Option<std::sync::Arc<Vec<String>>>,
}

pub(crate) struct AxisTextVisibility {
    pub(crate) tick_labels: [bool; 2],
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
                tick_labels: [true; 2],
                titles: [!source.has_facets(); 2],
            },
            gutter_owner: if source.has_facets() {
                GutterOwner::Composition
            } else {
                GutterOwner::Root
            },
            coordinated: CoordinatedDomains::default(),
        }
    }
}
