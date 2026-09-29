//! One immutable opaque mapping over prepared raw semantic identities.
//! T5d2 installs this before wiring the final family/Canvas consumers.

#![allow(
    dead_code,
    reason = "T5d2 mapping is wired across all family consumers before the old modulo paths are removed"
)]

use crate::font::canonical::OperationGenerationResources;
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
