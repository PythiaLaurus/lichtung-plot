//! One immutable opaque mapping over prepared raw semantic identities.
//! Prepared family and guide consumers share this opaque mapping.

use super::{
    AccentPalette, CategoricalColorScale, ColorDomain, ColorStop, OpaqueColor, PaletteCapacity,
    QualitativePalette, SequentialGradient,
};
use crate::font::canonical::OperationGenerationResources;
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
        resources: &OperationGenerationResources,
    ) -> Result<Vec<OpaqueColor>, Error> {
        resources.claim_calls(1).map_err(work_error)?;
        if requested <= self.colors().len() {
            return super::color::extend_palette(self.colors(), requested, resources);
        }
        let mut cache = resources
            .palette_cache
            .lock()
            .map_err(|_| Error::Render("resolved palette cache synchronization failed".into()))?;
        for entry in &mut cache.entries {
            resources.claim_visits(1).map_err(work_error)?;
            if entry.seeds.len() != self.colors().len() {
                continue;
            }
            let mut matching = true;
            for (cached, current) in entry.seeds.iter().zip(self.colors()) {
                resources.claim_visits(1).map_err(work_error)?;
                if cached != current {
                    matching = false;
                    break;
                }
            }
            if !matching {
                continue;
            }
            if requested <= entry.colors.len() {
                return super::color::extend_palette(&entry.colors, requested, resources);
            }
            let result = super::color::extend_palette(self.colors(), requested, resources)?;
            let mut retained = Vec::new();
            reserve(resources, &mut retained, result.len())?;
            retained.extend_from_slice(&result);
            entry.colors = retained;
            return Ok(result);
        }
        let result = super::color::extend_palette(self.colors(), requested, resources)?;
        let mut seeds = Vec::new();
        reserve(resources, &mut seeds, self.colors().len())?;
        seeds.extend_from_slice(self.colors());
        let mut colors = Vec::new();
        reserve(resources, &mut colors, result.len())?;
        colors.extend_from_slice(&result);
        resources.claim_cache_entries(1).map_err(work_error)?;
        reserve(resources, &mut cache.entries, 1)?;
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
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        resources
            .claim_visits(identities.len() as u128)
            .map_err(work_error)?;
        // Selection retains the current immutable map/palette, never another key table.
        resources.claim_visits(1).map_err(work_error)?;
        let selected = exact.map_or_else(
            || CategoricalColorScale::from_palette(palette.clone()),
            |map| CategoricalColorScale::from_map(map.clone()),
        );
        let colors = if let Some(exact) = selected.map() {
            let mut colors = Vec::new();
            reserve(resources, &mut colors, identities.len())?;
            for identity in identities {
                resources.claim_visits(1).map_err(work_error)?;
                colors.push(exact.color_in(identity, resources)?.ok_or_else(|| {
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
                    reserve(resources, &mut colors, identities.len())?;
                    colors.extend_from_slice(&seeds[..identities.len()]);
                    colors
                }
                PaletteCapacity::Extendable => {
                    palette.extend_categorical_palette(identities.len(), resources)?
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
    pub(crate) fn resolve(
        palette: &AccentPalette,
        slots: usize,
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        Ok(Self {
            colors: super::color::extend_palette(palette.seeds(), slots, resources)?,
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
    pub(crate) fn share(&self, resources: &OperationGenerationResources) -> Result<Self, Error> {
        resources.claim_visits(1).map_err(work_error)?;
        Ok(self.clone())
    }

    pub(crate) fn same(
        &self,
        other: &Self,
        resources: &OperationGenerationResources,
    ) -> Result<bool, Error> {
        resources.claim_visits(1).map_err(work_error)?;
        if self.domain != other.domain {
            return Ok(false);
        }
        self.gradient.same_in(&other.gradient, resources)
    }

    pub(crate) fn resolve(
        domain: (f64, f64),
        stops: &[ColorStop],
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        Ok(Self {
            domain: ColorDomain::exact(domain.0, domain.1)?,
            gradient: SequentialGradient::new_in(stops.iter().copied(), resources)?,
        })
    }

    pub(crate) fn with_domain(
        &self,
        domain: (f64, f64),
        resources: &OperationGenerationResources,
    ) -> Result<Self, Error> {
        Self::resolve(domain, self.stops(), resources)
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

    pub(crate) fn color(
        &self,
        value: f64,
        resources: &OperationGenerationResources,
    ) -> Result<OpaqueColor, Error> {
        let fraction = Self::normalized_fraction(self.domain(), value)?;
        for pair in self.stops().windows(2) {
            resources.claim_visits(1).map_err(work_error)?;
            if fraction <= pair[1].offset() {
                let local = (fraction - pair[0].offset()) / (pair[1].offset() - pair[0].offset());
                let rgb = super::color::interpolate(
                    pair[0].color().as_rgb(),
                    pair[1].color().as_rgb(),
                    local.clamp(0.0, 1.0),
                    resources,
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
    pub(crate) fn guide_stops(
        &self,
        observed: &[f64],
        data: crate::data::PlotData<'_>,
    ) -> Result<Vec<GradientStop>, Error> {
        use crate::render::coordinates::{CanonicalCoordinate, CoordinatePrecision};

        let resources = data.source_resources()?;
        let domain = self.domain();
        let constant = domain.0.total_cmp(&domain.1).is_eq();
        let midpoint = constant
            .then(|| self.color(domain.0, resources))
            .transpose()?;
        let capacity = if constant {
            2
        } else {
            self.stops()
                .len()
                .checked_add(observed.len())
                .ok_or_else(|| Error::Render("continuous guide stop count overflowed".into()))?
        };
        let mut candidates = Vec::new();
        data.reserve_source(&mut candidates, capacity)?;
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
                data.claim_source_visits(1)?;
                push(stop.backend_stop())?;
            }
            for &value in observed {
                data.claim_source_visits(1)?;
                let offset = Self::normalized_fraction(domain, value)?;
                push(GradientStop::new(
                    offset,
                    self.color(value, resources)?.as_rgb(),
                ))?;
            }
        }
        crate::data::source::sort_source_by(&mut candidates, Some(data), |left, right| {
            left.0.cmp(&right.0)
        })?;
        let mut selected = Vec::new();
        data.reserve_source(&mut selected, candidates.len())?;
        let mut previous = None;
        for (units, stop) in candidates {
            data.claim_source_visits(1)?;
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

    fn stop(offset: f64, (r, g, b): Rgb) -> ColorStop {
        ColorStop::new(offset, OpaqueColor::rgb(r, g, b)).unwrap()
    }

    #[test]
    fn sequential_scale_preserves_endpoints_and_saturates_exact_domains() {
        let stops = [stop(0.0, (49, 130, 189)), stop(1.0, (222, 45, 38))];
        let resources = OperationGenerationResources::new();
        let scale = ContinuousColors::resolve((0.0, 1.0), &stops, &resources).unwrap();
        assert_eq!(scale.stops(), stops);
        assert_eq!(scale.color(-1.0, &resources).unwrap(), stops[0].color());
        assert_eq!(scale.color(2.0, &resources).unwrap(), stops[1].color());
        assert_eq!(
            scale.color(0.5, &resources).unwrap().as_rgb(),
            (157, 107, 124)
        );
        let constant = scale.with_domain((3.0, 3.0), &resources).unwrap();
        assert_eq!(
            constant.color(3.0, &resources).unwrap().as_rgb(),
            (157, 107, 124)
        );
        let extreme = scale
            .with_domain((-f64::MAX, f64::MAX), &resources)
            .unwrap();
        assert_eq!(
            extreme.color(0.0, &resources).unwrap().as_rgb(),
            (157, 107, 124)
        );
        let three = ContinuousColors::resolve(
            (0.0, 4.0),
            &[stops[0], stop(0.25, (10, 20, 30)), stops[1]],
            &resources,
        )
        .unwrap();
        assert_eq!(three.color(1.0, &resources).unwrap().as_rgb(), (10, 20, 30));
    }

    #[test]
    fn sequential_scale_rejects_invalid_stops_values_and_storage_before_use() {
        let resources = OperationGenerationResources::new();
        let stops = [stop(0.0, (0, 0, 0)), stop(1.0, (255, 255, 255))];
        for invalid in [[stops[1], stops[0]], [stops[0], stops[0]]] {
            assert!(ContinuousColors::resolve((0.0, 1.0), &invalid, &resources).is_err());
        }
        assert!(ColorStop::new(f64::NAN, OpaqueColor::rgb(0, 0, 0)).is_err());
        assert!(ContinuousColors::resolve((2.0, 1.0), &stops, &resources).is_err());
        let scale = ContinuousColors::resolve((0.0, 1.0), &stops, &resources).unwrap();
        assert!(scale.color(f64::NAN, &resources).is_err());
        let limited = OperationGenerationResources::with_test_limits(0, 0);
        assert!(ContinuousColors::resolve((0.0, 1.0), &stops, &limited).is_err());
    }

    #[test]
    fn sampled_guide_preserves_mark_colors_and_refuses_collapsed_distinct_stops() {
        let plot = crate::Plot::scatter(
            crate::NumericValues::new("x", [1.0, 2.0]),
            crate::NumericValues::new("y", [2.0, 3.0]),
        )
        .erase();
        let stops = [stop(0.0, (49, 130, 189)), stop(1.0, (222, 45, 38))];
        let run = |limits: Option<(u128, u64)>| {
            let resources = OperationGenerationResources::new();
            if let Some((visits, bytes)) = limits {
                resources.set_test_limits(visits, bytes);
            }
            let data = plot.data().with_axis_resources(&resources);
            let result = ContinuousColors::resolve((0.0, 1.0), &stops, &resources)
                .and_then(|scale| scale.guide_stops(&[0.5, 0.5, 1.0], data));
            (resources, result)
        };
        let (resources, selected) = run(None);
        let selected = selected.unwrap();
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[1], GradientStop::new(0.5, (157, 107, 124)));
        let visits = resources.usage().unwrap().1;
        let bytes = resources.byte_usage().unwrap();
        assert_eq!(run(Some((visits, bytes))).1.unwrap(), selected);
        assert!(run(Some((visits - 1, bytes))).1.is_err());
        assert!(run(Some((visits, bytes - 1))).1.is_err());

        let resources = OperationGenerationResources::new();
        let close = [
            stop(0.0, (0, 0, 0)),
            stop(0.500_000_4, (0, 0, 0)),
            stop(0.500_000_49, (255, 255, 255)),
            stop(1.0, (255, 255, 255)),
        ];
        let scale = ContinuousColors::resolve((0.0, 1.0), &close, &resources).unwrap();
        crate::font::canonical::FONT_DISCOVERY_CALLS.with(|calls| calls.set(0));
        assert!(matches!(
            scale.guide_stops(&[], plot.data().with_axis_resources(&resources)),
            Err(Error::Data(message)) if message.contains("collapse")
        ));
        assert_eq!(
            crate::font::canonical::FONT_DISCOVERY_CALLS.with(std::cell::Cell::get),
            0
        );
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
        let map = CategoryColors::resolve(
            &keys,
            &QualitativePalette::fixed([OpaqueColor::rgb(0, 0, 0)]).unwrap(),
            Some(&exact),
            &resources,
        )
        .unwrap();
        assert_eq!(
            map.colors(),
            &[OpaqueColor::rgb(10, 20, 30), OpaqueColor::rgb(10, 20, 30)]
        );
        assert_eq!(map.color("before", &resources).unwrap(), (10, 20, 30));
        assert!(map.color("Before", &resources).is_err());
        let missing = ["before".into(), "different".into()];
        assert!(
            matches!(CategoryColors::resolve(&missing, &QualitativePalette::fixed([OpaqueColor::rgb(0, 0, 0)]).unwrap(), Some(&exact),
            &OperationGenerationResources::new()), Err(Error::Data(message))
            if message.contains("different"))
        );
    }

    #[test]
    fn fixed_capacity_and_accent_slots_do_not_cycle_or_consume_hidden_overrides() {
        let keys = ["first".into(), "second".into(), "third".into()];
        assert!(
            matches!(CategoryColors::resolve(&keys, &QualitativePalette::fixed([OpaqueColor::rgb(1, 2, 3)]).unwrap(),
            None, &OperationGenerationResources::new()), Err(Error::Data(message))
            if message.contains("fixed categorical palette"))
        );
        let seeds = [(0, 114, 178), (230, 159, 0)];
        let palette =
            AccentPalette::new(seeds.iter().map(|&(r, g, b)| OpaqueColor::rgb(r, g, b))).unwrap();
        let prefix =
            AccentColors::resolve(&palette, 3, &OperationGenerationResources::new()).unwrap();
        let extended =
            AccentColors::resolve(&palette, 4, &OperationGenerationResources::new()).unwrap();
        assert_eq!(prefix.color(2).unwrap(), extended.color(2).unwrap());
        assert!(extended.color(4).is_err());
        assert_eq!(extended.color(0).unwrap(), seeds[0]);
    }

    #[test]
    fn extended_palette_is_reused_only_within_one_operation_and_same_seed_prefix() {
        let mut theme = crate::Theme::Publication
            .resolve(&OperationGenerationResources::new())
            .unwrap();
        let keys = (0..18)
            .map(|index| format!("hue-{index}"))
            .collect::<Vec<_>>();
        let resolve = |theme: &crate::theme::ResolvedTheme,
                       resources: &OperationGenerationResources| {
            CategoryColors::resolve(&keys, theme.color_scales.qualitative(), None, resources)
        };
        let resources = OperationGenerationResources::new();
        let first = resolve(&theme, &resources).unwrap();
        let bytes = resources.byte_usage().unwrap();
        let visits = resources.usage().unwrap().1;
        let cache_entries = resources.call_cache_usage().1;
        assert_eq!(cache_entries, 1);
        let again = resolve(&theme, &resources).unwrap();
        assert_eq!(first.colors(), again.colors());
        assert_eq!(resources.call_cache_usage().1, cache_entries);
        assert!(resources.byte_usage().unwrap() - bytes < bytes);
        assert!(resources.usage().unwrap().1 - visits < visits);

        let other = OperationGenerationResources::new();
        assert_eq!(resolve(&theme, &other).unwrap().colors(), first.colors());
        assert_eq!(other.call_cache_usage().1, 1);
        let colors = theme.color_scales.qualitative().colors();
        let changed = QualitativePalette::extendable(
            std::iter::once(OpaqueColor::rgb(10, 20, 30)).chain(colors[1..].iter().copied()),
        )
        .unwrap();
        theme.color_scales = theme
            .color_scales
            .clone()
            .with_qualitative(changed)
            .unwrap();
        let changed = resolve(&theme, &resources).unwrap();
        assert_eq!(changed.colors()[0], OpaqueColor::rgb(10, 20, 30));
        assert_ne!(changed.colors(), first.colors());
        assert_eq!(resources.call_cache_usage().1, 2);
    }
}
