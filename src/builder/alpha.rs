use super::Plot;
use crate::alpha::{
    AlphaEncoding, AlphaInputs, AlphaTarget, AlphaUnitKind, PreparedAlpha, ResolvedAlpha,
};

pub(crate) struct ScopedAlpha<'a> {
    prepared: PreparedAlpha,
    facets: Vec<(&'a str, &'a str)>,
}

impl ScopedAlpha<'_> {
    pub(crate) fn bin(&self, bounds: (f64, f64)) -> crate::Result<Option<ResolvedAlpha>> {
        self.prepared.bin(bounds, &self.facets)
    }

    pub(crate) fn keyed(&self, unit: &[(&str, &str)]) -> crate::Result<Option<ResolvedAlpha>> {
        if self.prepared.selector_fields().is_empty() {
            return self.prepared.keyed(&[]);
        }
        let count = unit
            .len()
            .checked_add(self.facets.len())
            .ok_or_else(|| crate::Error::Data("alpha actual unit key count overflow".into()))?;
        let mut keys = Vec::new();
        crate::data::source::reserve_source(&mut keys, count)?;
        keys.extend_from_slice(unit);
        keys.extend_from_slice(&self.facets);
        self.prepared.keyed(&keys)
    }

    pub(crate) fn collective(
        &self,
        chart: &Plot,
        identity: &crate::data::CollectivePair,
    ) -> crate::Result<Option<ResolvedAlpha>> {
        let group = chart.group_name().zip(identity.group.as_deref());
        let hue = chart.hue_name().zip(identity.hue.as_deref());
        match (group, hue) {
            (Some(group), Some(hue)) => self.keyed(&[group, hue]),
            (Some(group), None) => self.keyed(&[group]),
            (None, Some(hue)) => self.keyed(&[hue]),
            (None, None) => self.keyed(&[]),
        }
    }

    pub(crate) fn aggregate(
        &self,
        category: (&str, &str),
        series: Option<(&str, &str)>,
    ) -> crate::Result<Option<ResolvedAlpha>> {
        if let Some(series) = series {
            self.keyed(&[category, series])
        } else {
            self.keyed(&[category])
        }
    }
}

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

    pub(crate) fn prepare_scoped_alpha<'a>(
        &'a self,
        target: AlphaTarget,
        unit: AlphaUnitKind,
        required_fields: &[&str],
        data: crate::data::PlotData<'a>,
    ) -> crate::Result<ScopedAlpha<'a>> {
        let prepared = self.prepare_alpha(target, unit, required_fields)?;
        let mut facets = Vec::new();
        if let Some(facet) = &self.facet {
            for values in facet.values() {
                if data.height() == 0
                    || !prepared
                        .selector_fields()
                        .iter()
                        .any(|field| field == values.name())
                {
                    continue;
                }
                let raw = values.get(data.row_index(0)).flatten().ok_or_else(|| {
                    crate::Error::Data(format!(
                        "alpha owning facet {:?} has no value at original slot {}",
                        values.name(),
                        data.row_index(0)
                    ))
                })?;
                if (1..data.height())
                    .any(|row| values.get(data.row_index(row)).flatten() != Some(raw))
                {
                    return Err(crate::Error::Data(format!(
                        "alpha owning facet {:?} does not identify one actual prepared scope",
                        values.name()
                    )));
                }
                crate::data::source::reserve_source(&mut facets, 1)?;
                facets.push((values.name(), raw));
            }
        }
        Ok(ScopedAlpha { prepared, facets })
    }

    pub(crate) fn prepare_collective_alpha<'a>(
        &'a self,
        target: AlphaTarget,
        unit: AlphaUnitKind,
        include_hue: bool,
        data: crate::data::PlotData<'a>,
    ) -> crate::Result<ScopedAlpha<'a>> {
        let mut fields = Vec::new();
        crate::data::source::reserve_source(&mut fields, 2)?;
        fields.extend(self.group_name());
        if include_hue {
            fields.extend(self.hue_name());
        }
        self.prepare_scoped_alpha(target, unit, &fields, data)
    }
}
