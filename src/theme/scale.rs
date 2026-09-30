//! One immutable opaque mapping over prepared raw semantic identities.
//! Prepared family and guide consumers share this opaque mapping.

#![allow(
    dead_code,
    reason = "fixed private palette mode is exercised before the T6 configuration surface exists"
)]

use crate::font::canonical::OperationGenerationResources;
use crate::render::backend::GradientStop;
use crate::spec::CategoricalColors;
use crate::Error;

type Rgb = (u8, u8, u8);

#[derive(Debug, Clone, Copy)]
pub(crate) enum PaletteSource<'a> {
    Fixed(&'a [Rgb]),
    Extendable(&'a [Rgb]),
}

#[derive(Debug, Clone)]
pub(crate) struct CategoryColors {
    identities: Vec<String>,
    colors: Vec<Rgb>,
}

impl CategoryColors {
    pub(crate) fn resolve(
        identities: &[String],
        palette: PaletteSource<'_>,
        exact: Option<&CategoricalColors>,
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        resources
            .claim_visits(identities.len() as u128)
            .map_err(work_error)?;
        if let PaletteSource::Fixed(seeds) = palette {
            if seeds.len() < identities.len() && exact.is_none() {
                return Err(Error::Data(format!(
                    "fixed categorical palette has {} colors for {} used identities",
                    seeds.len(),
                    identities.len()
                )));
            }
        }
        let colors = if let Some(exact) = exact {
            let mut colors = Vec::new();
            reserve(resources, &mut colors, identities.len())?;
            for identity in identities {
                resources.claim_visits(1).map_err(work_error)?;
                colors.push(exact.color(identity, resources)?.ok_or_else(|| {
                    Error::Data(format!(
                        "categorical color map has no color for used raw key {identity:?}"
                    ))
                })?);
            }
            colors
        } else {
            match palette {
                PaletteSource::Fixed(seeds) => {
                    let mut colors = Vec::new();
                    reserve(resources, &mut colors, identities.len())?;
                    colors.extend_from_slice(&seeds[..identities.len()]);
                    colors
                }
                PaletteSource::Extendable(seeds) => {
                    super::color::extend_palette(seeds, identities.len(), resources)?
                }
            }
        };
        let mut retained = Vec::new();
        reserve(resources, &mut retained, identities.len())?;
        for (index, identity) in identities.iter().enumerate() {
            resources.claim_visits(index as u128).map_err(work_error)?;
            if identities[..index].contains(identity) {
                return Err(Error::Data(format!(
                    "resolved categorical domain repeats raw key {identity:?}"
                )));
            }
            resources
                .claim_bytes(
                    u64::try_from(identity.len())
                        .map_err(|_| Error::Render("categorical key storage overflowed".into()))?,
                )
                .map_err(|error| storage_error(&error))?;
            retained.push(identity.clone());
        }
        Ok(Self {
            identities: retained,
            colors,
        })
    }

    pub(crate) fn color(
        &self,
        raw_identity: &str,
        resources: &OperationGenerationResources,
    ) -> Result<Rgb, Error> {
        for (index, identity) in self.identities.iter().enumerate() {
            resources.claim_visits(1).map_err(work_error)?;
            if identity == raw_identity {
                return Ok(self.colors[index]);
            }
        }
        Err(Error::Data(format!(
            "used categorical color identity {raw_identity:?} is absent from the prepared domain"
        )))
    }

    pub(crate) fn colors(&self) -> &[Rgb] {
        &self.colors
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AccentColors {
    colors: Vec<Rgb>,
}

impl AccentColors {
    pub(crate) fn resolve(
        seeds: &[Rgb],
        slots: usize,
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        if seeds.is_empty() {
            return Err(Error::Data(
                "accent palette requires a nonempty seed prefix".into(),
            ));
        }
        Ok(Self {
            colors: super::color::extend_palette(seeds, slots, resources)?,
        })
    }

    pub(crate) fn color(&self, slot: usize) -> Result<Rgb, Error> {
        self.colors.get(slot).copied().ok_or_else(|| {
            Error::Render("undeclared accent slot is not part of this resolved plot".into())
        })
    }
}

/// One prepared numeric domain and ordered opaque stops, shared by marks and
/// their guide. Normalization happens before first-party OKLab interpolation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousColors {
    domain: (f64, f64),
    stops: Vec<GradientStop>,
}

impl ContinuousColors {
    pub(crate) fn resolve(
        domain: (f64, f64),
        stops: &[GradientStop],
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        if !domain.0.is_finite() || !domain.1.is_finite() || domain.0 > domain.1 {
            return Err(Error::Data(
                "continuous color domain must be finite and ordered".into(),
            ));
        }
        if stops.len() < 2
            || stops
                .first()
                .is_none_or(|stop| stop.offset.to_bits() != 0.0_f64.to_bits())
            || stops
                .last()
                .is_none_or(|stop| stop.offset.to_bits() != 1.0_f64.to_bits())
        {
            return Err(Error::Data(
                "continuous color stops must span exactly 0..1".into(),
            ));
        }
        let mut previous = -1.0;
        for stop in stops {
            resources.claim_visits(1).map_err(work_error)?;
            if !stop.offset.is_finite() || stop.offset <= previous || stop.offset > 1.0 {
                return Err(Error::Data(
                    "continuous color stops must increase strictly".into(),
                ));
            }
            previous = stop.offset;
        }
        let mut retained = Vec::new();
        reserve(resources, &mut retained, stops.len())?;
        retained.extend_from_slice(stops);
        Ok(Self {
            domain,
            stops: retained,
        })
    }

    pub(crate) fn with_domain(
        &self,
        domain: (f64, f64),
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        Self::resolve(domain, &self.stops, resources)
    }

    pub(crate) fn stops(&self) -> &[GradientStop] {
        &self.stops
    }

    pub(crate) fn into_stops(self) -> Vec<GradientStop> {
        self.stops
    }

    pub(crate) fn domain(&self) -> (f64, f64) {
        self.domain
    }

    /// One arithmetic owner for mark values and diverging center-stop offsets.
    pub(crate) fn normalized_fraction(domain: (f64, f64), value: f64) -> Result<f64, Error> {
        if !value.is_finite() || !domain.0.is_finite() || !domain.1.is_finite() {
            return Err(Error::Data(
                "continuous color coordinates must be finite".into(),
            ));
        }
        if domain.0 > domain.1 {
            return Err(Error::Data(
                "continuous color domain must be ordered".into(),
            ));
        }
        let (low, high) = domain;
        let fraction = if low.total_cmp(&high).is_eq()
            || (low.abs().to_bits() == 0 && high.abs().to_bits() == 0)
        {
            0.5
        } else if value <= low {
            0.0
        } else if value >= high {
            1.0
        } else {
            let span = high - low;
            if span.is_finite() {
                (value - low) / span
            } else {
                let scale = low.abs().max(high.abs());
                (value / scale - low / scale) / (high / scale - low / scale)
            }
        };
        if !fraction.is_finite() {
            return Err(Error::Data(
                "continuous color fraction is not finite".into(),
            ));
        }
        Ok(fraction.clamp(0.0, 1.0))
    }

    pub(crate) fn color(
        &self,
        value: f64,
        resources: &OperationGenerationResources,
    ) -> Result<Rgb, Error> {
        let fraction = Self::normalized_fraction(self.domain, value)?;
        for pair in self.stops.windows(2) {
            resources.claim_visits(1).map_err(work_error)?;
            if fraction <= pair[1].offset {
                let local = (fraction - pair[0].offset) / (pair[1].offset - pair[0].offset);
                return super::color::interpolate(
                    pair[0].color,
                    pair[1].color,
                    local.clamp(0.0, 1.0),
                    resources,
                );
            }
        }
        Err(Error::Render(
            "resolved continuous stops omit the color value".into(),
        ))
    }
}

fn reserve<T>(
    resources: &OperationGenerationResources,
    values: &mut Vec<T>,
    count: usize,
) -> Result<(), Error> {
    resources.claim_slots(count as u128).map_err(work_error)?;
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| Error::Render("color map storage arithmetic overflowed".into()))?;
    resources
        .claim_bytes(bytes)
        .map_err(|error| storage_error(&error))?;
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Render("cannot reserve resolved color map".into()))
}

fn work_error(error: crate::font::canonical::TextGenerationClaimError) -> Error {
    Error::Render(format!("color map work admission: {error:?}"))
}

fn storage_error(error: &crate::font::canonical::FontError) -> Error {
    Error::Render(format!("color map storage admission: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_scale_preserves_endpoints_and_saturates_exact_domains() {
        let stops = [
            GradientStop::new(0.0, (49, 130, 189)),
            GradientStop::new(1.0, (222, 45, 38)),
        ];
        let resources = OperationGenerationResources::new();
        let scale = ContinuousColors::resolve((0.0, 1.0), &stops, &resources).unwrap();
        assert_eq!(scale.stops(), stops);
        assert_eq!(scale.color(-1.0, &resources).unwrap(), stops[0].color);
        assert_eq!(scale.color(2.0, &resources).unwrap(), stops[1].color);
        assert_eq!(scale.color(0.5, &resources).unwrap(), (157, 107, 124));
        let constant = scale.with_domain((3.0, 3.0), &resources).unwrap();
        assert_eq!(constant.color(3.0, &resources).unwrap(), (157, 107, 124));
        let extreme = scale
            .with_domain((-f64::MAX, f64::MAX), &resources)
            .unwrap();
        assert_eq!(extreme.color(0.0, &resources).unwrap(), (157, 107, 124));
        let three = ContinuousColors::resolve(
            (0.0, 4.0),
            &[stops[0], GradientStop::new(0.25, (10, 20, 30)), stops[1]],
            &resources,
        )
        .unwrap();
        assert_eq!(three.color(1.0, &resources).unwrap(), (10, 20, 30));
    }

    #[test]
    fn sequential_scale_rejects_invalid_stops_values_and_storage_before_use() {
        let resources = OperationGenerationResources::new();
        let stops = [
            GradientStop::new(0.0, (0, 0, 0)),
            GradientStop::new(1.0, (255, 255, 255)),
        ];
        for invalid in [
            [stops[1], stops[0]],
            [stops[0], stops[0]],
            [stops[0], GradientStop::new(f64::NAN, (0, 0, 0))],
        ] {
            assert!(ContinuousColors::resolve((0.0, 1.0), &invalid, &resources).is_err());
        }
        assert!(ContinuousColors::resolve((2.0, 1.0), &stops, &resources).is_err());
        let scale = ContinuousColors::resolve((0.0, 1.0), &stops, &resources).unwrap();
        assert!(scale.color(f64::NAN, &resources).is_err());
        let limited = OperationGenerationResources::with_test_limits(0, 0);
        assert!(ContinuousColors::resolve((0.0, 1.0), &stops, &limited).is_err());
    }

    #[test]
    fn exact_raw_keys_allow_unused_and_repeated_colors_but_require_used_coverage() {
        let keys = ["before".into(), "after".into()];
        let exact = CategoricalColors::new([
            ("after".into(), (10, 20, 30)),
            ("before".into(), (10, 20, 30)),
            ("unused".into(), (40, 50, 60)),
        ])
        .unwrap();
        let resources = OperationGenerationResources::new();
        let map =
            CategoryColors::resolve(&keys, PaletteSource::Fixed(&[]), Some(&exact), &resources)
                .unwrap();
        assert_eq!(map.colors(), &[(10, 20, 30), (10, 20, 30)]);
        assert_eq!(map.color("before", &resources).unwrap(), (10, 20, 30));
        assert!(map.color("Before", &resources).is_err());
        let missing = ["before".into(), "different".into()];
        assert!(
            matches!(CategoryColors::resolve(&missing, PaletteSource::Fixed(&[]), Some(&exact),
            &OperationGenerationResources::new()), Err(Error::Data(message))
            if message.contains("different"))
        );
    }

    #[test]
    fn fixed_capacity_and_accent_slots_do_not_cycle_or_consume_hidden_overrides() {
        let keys = ["first".into(), "second".into(), "third".into()];
        assert!(
            matches!(CategoryColors::resolve(&keys, PaletteSource::Fixed(&[(1, 2, 3)]),
            None, &OperationGenerationResources::new()), Err(Error::Data(message))
            if message.contains("fixed categorical palette"))
        );
        let seeds = [(0, 114, 178), (230, 159, 0)];
        let prefix =
            AccentColors::resolve(&seeds, 3, &OperationGenerationResources::new()).unwrap();
        let extended =
            AccentColors::resolve(&seeds, 4, &OperationGenerationResources::new()).unwrap();
        assert_eq!(prefix.color(2).unwrap(), extended.color(2).unwrap());
        assert!(extended.color(4).is_err());
        assert_eq!(extended.color(0).unwrap(), seeds[0]);
    }
}
