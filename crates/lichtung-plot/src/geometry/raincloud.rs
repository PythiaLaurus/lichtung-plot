use std::collections::HashMap;

use super::{aligned, category_value, dimension, finite};
use crate::data::source::{collect_source, reserve_source};
use crate::{CategoryValues, Error, RaincloudGeometry};

const MAX_GROUPS: usize = 512;
const MAX_DENSITY_PER_GROUP: usize = 2_048;
const MAX_DENSITY: usize = 1_000_000;
const MAX_POINTS: usize = 1_000_000;

#[derive(Debug)]
pub(crate) struct RaincloudRows {
    pub(crate) density: Vec<usize>,
    pub(crate) points: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct RaincloudMembership {
    pub(crate) range: (f64, f64),
    pub(crate) groups: Vec<RaincloudRows>,
}

fn group_key<'a>(
    category: &'a CategoryValues,
    hue: Option<&'a CategoryValues>,
    row: usize,
) -> Result<(&'a str, Option<&'a str>), Error> {
    Ok((
        category_value(category, row, "raincloud category")?,
        hue.map(|hue| category_value(hue, row, "raincloud hue"))
            .transpose()?,
    ))
}

fn retain_row(rows: &mut Vec<usize>, row: usize) -> Result<(), Error> {
    if rows.len() == rows.capacity() {
        reserve_source(rows, rows.len().max(1))?;
    }
    rows.push(row);
    Ok(())
}

impl RaincloudGeometry {
    pub(crate) fn membership(&self) -> Result<RaincloudMembership, Error> {
        self.validate_columns()?;
        let mut index = HashMap::new();
        index.try_reserve(self.category.len()).map_err(|error| {
            Error::Render(format!("raincloud group index reservation: {error}"))
        })?;
        let mut groups = collect_source((0..self.category.len()).map(|row| {
            let key = group_key(&self.category, self.hue.as_ref(), row)?;
            if index.insert(key, row).is_some() {
                return Err(Error::Data(format!(
                    "raincloud geometry contains duplicate category-hue identity {key:?}"
                )));
            }
            Ok(RaincloudRows {
                density: Vec::new(),
                points: Vec::new(),
            })
        }))?;
        for row in 0..self.density_category.len() {
            let key = group_key(&self.density_category, self.density_hue.as_ref(), row)?;
            let group = index.get(&key).ok_or_else(|| {
                Error::Data(format!(
                    "raincloud density row {row} names an unknown summary group {key:?}"
                ))
            })?;
            retain_row(&mut groups[*group].density, row)?;
        }
        for row in 0..self.point_category.len() {
            let key = group_key(&self.point_category, self.point_hue.as_ref(), row)?;
            let group = index.get(&key).ok_or_else(|| {
                Error::Data(format!(
                    "raincloud point row {row} names an unknown summary group {key:?}"
                ))
            })?;
            retain_row(&mut groups[*group].points, row)?;
        }
        let mut range = (f64::INFINITY, f64::NEG_INFINITY);
        for (row, group) in groups.iter().enumerate() {
            let support = self.validate_group(row, group)?;
            range.0 = range.0.min(support.0);
            range.1 = range.1.max(support.1);
        }
        Ok(RaincloudMembership { range, groups })
    }

    fn validate_columns(&self) -> Result<(), Error> {
        dimension(&self.category_dimension, "raincloud category dimension")?;
        dimension(&self.value_dimension, "raincloud value dimension")?;
        if let Some(hue) = &self.hue_dimension {
            dimension(hue, "raincloud hue dimension")?;
        }
        for hue in [&self.hue, &self.density_hue, &self.point_hue] {
            if hue.is_some() != self.hue_dimension.is_some() {
                return Err(Error::Data("raincloud hue dimension and all three hue columns must be present together or absent together".into()));
            }
        }
        aligned(
            "raincloud summary",
            self.category.len(),
            &[
                ("q1", self.q1.len()),
                ("median", self.median.len()),
                ("q3", self.q3.len()),
            ],
        )?;
        aligned(
            "raincloud density",
            self.density_category.len(),
            &[
                ("density_y", self.density_y.len()),
                ("normalized_width", self.normalized_width.len()),
            ],
        )?;
        aligned(
            "raincloud points",
            self.point_category.len(),
            &[
                ("point_y", self.point_y.len()),
                ("normalized_strip", self.normalized_strip.len()),
            ],
        )?;
        for (domain, len, hue) in [
            ("raincloud summary", self.category.len(), self.hue.as_ref()),
            (
                "raincloud density",
                self.density_category.len(),
                self.density_hue.as_ref(),
            ),
            (
                "raincloud points",
                self.point_category.len(),
                self.point_hue.as_ref(),
            ),
        ] {
            if let Some(hue) = hue {
                aligned(domain, len, &[(hue.name(), hue.len())])?;
            }
        }
        if self.category.is_empty() || self.category.len() > MAX_GROUPS {
            return Err(Error::Data(format!(
                "raincloud geometry requires 1..={MAX_GROUPS} summary groups"
            )));
        }
        if self.density_category.len() > MAX_DENSITY || self.point_category.len() > MAX_POINTS {
            return Err(Error::Data(
                "raincloud geometry exceeds its density/raw-point safety limit".into(),
            ));
        }
        Ok(())
    }

    fn validate_group(&self, row: usize, group: &RaincloudRows) -> Result<(f64, f64), Error> {
        if group.density.len() < 2 || group.density.len() > MAX_DENSITY_PER_GROUP {
            return Err(Error::Data(format!("raincloud geometry group {row} requires 2..={MAX_DENSITY_PER_GROUP} density points")));
        }
        let mut previous = None;
        let mut positive = false;
        for &point in &group.density {
            let y = finite(&self.density_y, point, "raincloud density y")?;
            let width = finite(&self.normalized_width, point, "raincloud density width")?;
            if !(0.0..=1.0).contains(&width) {
                return Err(Error::Data(format!(
                    "raincloud geometry group {row} requires normalized width in [0, 1]"
                )));
            }
            if previous.is_some_and(|previous| y <= previous) {
                return Err(Error::Data(format!("raincloud geometry group {row} density y coordinates must be strictly increasing")));
            }
            previous = Some(y);
            positive |= width > 0.0;
        }
        if !positive {
            return Err(Error::Data(format!(
                "raincloud geometry group {row} requires a positive density width"
            )));
        }
        let q1 = finite(&self.q1, row, "raincloud q1")?;
        let median = finite(&self.median, row, "raincloud median")?;
        let q3 = finite(&self.q3, row, "raincloud q3")?;
        let support = (
            finite(&self.density_y, group.density[0], "raincloud density y")?,
            finite(
                &self.density_y,
                group.density[group.density.len() - 1],
                "raincloud density y",
            )?,
        );
        if q1 > median || median > q3 || q1 < support.0 || q3 > support.1 {
            return Err(Error::Data(format!(
                "raincloud geometry group {row} requires q1 <= median <= q3 within density support"
            )));
        }
        if group.points.is_empty() {
            return Err(Error::Data(format!(
                "raincloud geometry group {row} requires at least one raw point"
            )));
        }
        for &point in &group.points {
            let y = finite(&self.point_y, point, "raincloud point y")?;
            let strip = finite(
                &self.normalized_strip,
                point,
                "raincloud normalized strip coordinate",
            )?;
            if !(0.0..=1.0).contains(&strip) {
                return Err(Error::Data(format!(
                    "raincloud geometry group {row} requires normalized strip coordinate in [0, 1]"
                )));
            }
            if !(support.0..=support.1).contains(&y) {
                return Err(Error::Data(format!(
                    "raincloud geometry group {row} raw point {point} lies outside density support"
                )));
            }
        }
        Ok(support)
    }
}
