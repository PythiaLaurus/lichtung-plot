//! Occurrence policy is separate from authored intent and prepared axis storage.

pub(crate) struct PlotScopePolicy {
    pub(crate) labels: crate::Labels,
    pub(crate) significance_indices: Vec<usize>,
    pub(crate) axis_text: AxisTextVisibility,
    pub(crate) gutter_owner: GutterOwner,
    pub(crate) discrete_guide_use: crate::decorations::legend::discrete::scoped::GuideUse,
    pub(crate) continuous_guide_use: crate::decorations::legend::discrete::scoped::GuideUse,
    pub(crate) guide_composition: crate::LegendComposition,
    pub(crate) continuous_materialization: ContinuousMaterialization,
    pub(crate) facet_scales: Option<crate::FacetScales>,
    pub(crate) coordinated: CoordinatedDomains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GutterOwner {
    Root,
    Composition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContinuousMaterialization {
    Local,
    SharedFinal,
    Suppressed,
}

#[derive(Debug, Default)]
pub(crate) struct CoordinatedDomains {
    pub(crate) colors: Option<std::sync::Arc<Vec<String>>>,
    pub(crate) groups: Option<std::sync::Arc<Vec<String>>>,
    pub(crate) collectives: Option<std::sync::Arc<Vec<crate::data::CollectivePair>>>,
}

impl CoordinatedDomains {
    pub(crate) fn composed_groups(
        &self,
        source: &crate::builder::PlotDescription,
    ) -> Option<&[String]> {
        if matches!(
            source.kind(),
            crate::PlotKind::Bar
                | crate::PlotKind::HorizontalBar
                | crate::PlotKind::StackedBar
                | crate::PlotKind::StackedHorizontalBar
                | crate::PlotKind::Boxplot
                | crate::PlotKind::Violin
                | crate::PlotKind::Raincloud
        ) && (!source.has_facets() || source.group_col().is_none())
        {
            return None;
        }
        self.groups.as_deref().map(Vec::as_slice)
    }
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
        let guide_use = if source.legend_policy_spec() == crate::LegendPolicy::Hidden {
            crate::decorations::legend::discrete::scoped::GuideUse::Hidden
        } else {
            crate::decorations::legend::discrete::scoped::GuideUse::Local
        };
        Self {
            labels: source.labels_policy().clone(),
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
            discrete_guide_use: guide_use,
            continuous_guide_use: guide_use,
            guide_composition: source.legend_composition_spec(),
            continuous_materialization: ContinuousMaterialization::Local,
            facet_scales: source.has_facets().then(|| source.facet_scales_spec()),
            coordinated: CoordinatedDomains::default(),
        }
    }
}
