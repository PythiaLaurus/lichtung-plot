use super::Plot;
use crate::alpha::{AlphaEncoding, AlphaInputs, AlphaTarget, AlphaUnitKind, PreparedAlpha};

impl<K: crate::kind::Marker> Plot<K> {
    pub(crate) fn alpha_inputs(&self) -> &AlphaInputs {
        &self.alpha
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private attachment precedes sealed public alpha cutover"
        )
    )]
    pub(crate) fn with_alpha(mut self, target: AlphaTarget, encoding: AlphaEncoding) -> Self {
        self.alpha.set(target, encoding);
        self
    }

    pub(crate) fn prepare_alpha(
        &self,
        target: AlphaTarget,
        unit: AlphaUnitKind,
        required_fields: &[&str],
    ) -> Result<PreparedAlpha, crate::Error> {
        let mut facets = Vec::new();
        if let Some(facet) = &self.facet {
            crate::data::source::reserve_source(&mut facets, 3)?;
            facets.extend(facet.values().map(crate::CategoryValues::name));
        }
        let original_len = if let Some(geometry) = self.raincloud_geometry_input() {
            geometry.groups.iter().try_fold(0_usize, |count, group| {
                count.checked_add(group.points.len()).ok_or_else(|| {
                    crate::Error::Data("supplied raincloud alpha point count overflow".into())
                })
            })?
        } else {
            self.row_count()
        };
        self.alpha
            .get(target)
            .prepare(unit, original_len, required_fields, &facets)
    }
}
