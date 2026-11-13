//! Occurrence policy is separate from authored intent and prepared axis storage.

pub(crate) struct PlotScopePolicy {
    pub(crate) axis_text: AxisTextVisibility,
}

pub(crate) struct AxisTextVisibility {
    pub(crate) titles: [bool; 2],
}

impl PlotScopePolicy {
    pub(crate) fn for_source(source: &crate::builder::PlotDescription) -> Self {
        Self {
            axis_text: AxisTextVisibility {
                titles: [!source.has_facets(); 2],
            },
        }
    }
}
