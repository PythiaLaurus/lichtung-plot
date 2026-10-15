//! One immutable opaque mapping over prepared raw semantic identities.
//! Prepared family and guide consumers share this opaque mapping.

use super::{
    AccentPalette, CategoricalColorScale, ColorDomain, ColorStop, OpaqueColor, PaletteCapacity,
    QualitativePalette, SequentialGradient,
};
use crate::font::canonical::OperationContext;
use crate::render::backend::GradientStop;
use crate::spec::CategoricalColors;
use crate::Error;

type Rgb = (u8, u8, u8);

#[derive(Debug, Default)]
pub(crate) struct PaletteCache {
    entries: Vec<CachedPalette>,
}

#[derive(Debug)]
struct CachedPalette {
    seeds: Vec<OpaqueColor>,
    colors: Vec<OpaqueColor>,
}

impl QualitativePalette {
    fn extend_categorical_palette(
        &self,
        requested: usize,
        resources: &OperationContext,
    ) -> Result<Vec<OpaqueColor>, Error> {
        if requested <= self.colors().len() {
            return super::color::extend_palette(self.colors(), requested);
        }
        let mut cache = resources
            .palette_cache
            .lock()
            .map_err(|_| Error::Render("resolved palette cache synchronization failed".into()))?;
        for entry in &mut cache.entries {
            if entry.seeds.len() != self.colors().len() {
                continue;
            }
            let mut matching = true;
            for (cached, current) in entry.seeds.iter().zip(self.colors()) {
                if cached != current {
                    matching = false;
                    break;
                }
            }
            if !matching {
                continue;
            }
            if requested <= entry.colors.len() {
                return super::color::extend_palette(&entry.colors, requested);
            }
            let result = super::color::extend_palette(self.colors(), requested)?;
            let mut retained = Vec::new();
            reserve(&mut retained, result.len())?;
            retained.extend_from_slice(&result);
            entry.colors = retained;
            return Ok(result);
        }
        let result = super::color::extend_palette(self.colors(), requested)?;
        let mut seeds = Vec::new();
        reserve(&mut seeds, self.colors().len())?;
        seeds.extend_from_slice(self.colors());
        let mut colors = Vec::new();
        reserve(&mut colors, result.len())?;
        colors.extend_from_slice(&result);

        reserve(&mut cache.entries, 1)?;
        cache.entries.push(CachedPalette { seeds, colors });
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CategoryColors {
    identities: Vec<String>,
    colors: Vec<OpaqueColor>,
}

impl CategoryColors {
    pub(crate) fn resolve(
        identities: &[String],
        palette: &QualitativePalette,
        exact: Option<&CategoricalColors>,
        resources: &OperationContext,
    ) -> Result<Self, Error> {
        // Selection retains the current immutable map/palette, never another key table.

        let selected = exact.map_or_else(
            || CategoricalColorScale::from_palette(palette.clone()),
            |map| CategoricalColorScale::from_map(map.clone()),
        );
        let colors = if let Some(exact) = selected.map() {
            let mut colors = Vec::new();
            reserve(&mut colors, identities.len())?;
            for identity in identities {
                colors.push(exact.color_in(identity).ok_or_else(|| {
                    Error::Data(format!(
                        "categorical color map has no color for used raw key {identity:?}"
                    ))
                })?);
            }
            colors
        } else {
            let palette = selected.palette().expect("validated palette selection");
            match palette.capacity() {
                PaletteCapacity::Fixed => {
                    let seeds = palette.colors();
                    if seeds.len() < identities.len() {
                        return Err(Error::Data(format!(
                            "fixed categorical palette has {} colors for {} used identities",
                            seeds.len(),
                            identities.len()
                        )));
                    }
                    let mut colors = Vec::new();
                    reserve(&mut colors, identities.len())?;
                    colors.extend_from_slice(&seeds[..identities.len()]);
                    colors
                }
                PaletteCapacity::Extendable => {
                    palette.extend_categorical_palette(identities.len(), resources)?
                }
            }
        };
        let mut retained = Vec::new();
        reserve(&mut retained, identities.len())?;
        for (index, identity) in identities.iter().enumerate() {
            if identities[..index].contains(identity) {
                return Err(Error::Data(format!(
                    "resolved categorical domain repeats raw key {identity:?}"
                )));
            }

            retained.push(identity.clone());
        }
        Ok(Self {
            identities: retained,
            colors,
        })
    }

    pub(crate) fn color(&self, raw_identity: &str) -> Result<Rgb, Error> {
        for (index, identity) in self.identities.iter().enumerate() {
            if identity == raw_identity {
                return Ok(self.colors[index].as_rgb());
            }
        }
        Err(Error::Data(format!(
            "used categorical color identity {raw_identity:?} is absent from the prepared domain"
        )))
    }

    #[cfg(test)]
    pub(crate) fn colors(&self) -> &[OpaqueColor] {
        &self.colors
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AccentColors {
    colors: Vec<OpaqueColor>,
}

impl AccentColors {
    pub(crate) fn resolve(palette: &AccentPalette, slots: usize) -> Result<Self, Error> {
        Ok(Self {
            colors: super::color::extend_palette(palette.seeds(), slots)?,
        })
    }

    pub(crate) fn color(&self, slot: usize) -> Result<Rgb, Error> {
        self.colors
            .get(slot)
            .copied()
            .map(OpaqueColor::as_rgb)
            .ok_or_else(|| {
                Error::Render("undeclared accent slot is not part of this resolved plot".into())
            })
    }
}

/// One prepared numeric domain and ordered opaque stops, shared by marks and
/// their guide. Normalization happens before first-party OKLab interpolation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ContinuousColors {
    domain: ColorDomain,
    gradient: SequentialGradient,
}

impl ContinuousColors {
    pub(crate) fn same(&self, other: &Self) -> bool {
        if self.domain != other.domain {
            return false;
        }
        self.gradient.same_in(&other.gradient)
    }

    pub(crate) fn resolve(domain: (f64, f64), stops: &[ColorStop]) -> Result<Self, Error> {
        Ok(Self {
            domain: ColorDomain::exact(domain.0, domain.1)?,
            gradient: SequentialGradient::new(stops.iter().copied())?,
        })
    }

    pub(crate) fn with_domain(&self, domain: (f64, f64)) -> Result<Self, Error> {
        Self::resolve(domain, self.stops())
    }

    pub(crate) fn stops(&self) -> &[ColorStop] {
        self.gradient.stops()
    }

    pub(crate) fn domain(&self) -> (f64, f64) {
        self.domain.as_pair()
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

    pub(crate) fn color(&self, value: f64) -> Result<OpaqueColor, Error> {
        let fraction = Self::normalized_fraction(self.domain(), value)?;
        for pair in self.stops().windows(2) {
            if fraction <= pair[1].offset() {
                let local = (fraction - pair[0].offset()) / (pair[1].offset() - pair[0].offset());
                let rgb = super::color::interpolate(
                    pair[0].color().as_rgb(),
                    pair[1].color().as_rgb(),
                    local.clamp(0.0, 1.0),
                )?;
                return Ok(OpaqueColor::rgb(rgb.0, rgb.1, rgb.2));
            }
        }
        Err(Error::Render(
            "resolved continuous stops omit the color value".into(),
        ))
    }

    /// Freeze a backend-interpolated gradient that contains every representable
    /// observed mark color as an exact stop, under the backend's six-decimal
    /// offset spelling. Distinct colors at one spelled position must refuse.
    pub(crate) fn guide_stops(&self, observed: &[f64]) -> Result<Vec<GradientStop>, Error> {
        use crate::render::coordinates::{CanonicalCoordinate, CoordinatePrecision};

        let domain = self.domain();
        let constant = domain.0.total_cmp(&domain.1).is_eq();
        let midpoint = constant.then(|| self.color(domain.0)).transpose()?;
        let capacity = if constant {
            2
        } else {
            self.stops()
                .len()
                .checked_add(observed.len())
                .ok_or_else(|| Error::Render("continuous guide stop count overflowed".into()))?
        };
        let mut candidates = Vec::new();
        crate::data::source::reserve_source(&mut candidates, capacity)?;
        let mut push = |stop: GradientStop| -> Result<(), Error> {
            let canonical = CanonicalCoordinate::from_f64(
                stop.offset,
                CoordinatePrecision::Millionths,
                "continuous guide stop offset",
            )?;
            candidates.push((
                canonical.units(),
                GradientStop::new(canonical.value(), stop.color),
            ));
            Ok(())
        };
        if let Some(color) = midpoint {
            push(GradientStop::new(0.0, color.as_rgb()))?;
            push(GradientStop::new(1.0, color.as_rgb()))?;
        } else {
            for stop in self.stops() {
                push(stop.backend_stop())?;
            }
            for &value in observed {
                let offset = Self::normalized_fraction(domain, value)?;
                push(GradientStop::new(offset, self.color(value)?.as_rgb()))?;
            }
        }
        crate::data::source::sort_source_by(&mut candidates, |left, right| left.0.cmp(&right.0));
        let mut selected = Vec::new();
        crate::data::source::reserve_source(&mut selected, candidates.len())?;
        let mut previous = None;
        for (units, stop) in candidates {
            if let Some((last_units, last_color)) = previous {
                if last_units == units {
                    if last_color != stop.color {
                        return Err(Error::Data(
                            "distinct continuous mark colors collapse at one serialized guide offset".into(),
                        ));
                    }
                    continue;
                }
            }
            previous = Some((units, stop.color));
            selected.push(stop);
        }
        Ok(selected)
    }
}

fn reserve<T>(values: &mut Vec<T>, count: usize) -> Result<(), Error> {
    values
        .try_reserve_exact(count)
        .map_err(|_| Error::Render("cannot reserve resolved color map".into()))
}
