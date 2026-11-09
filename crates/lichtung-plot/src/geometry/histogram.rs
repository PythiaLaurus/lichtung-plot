use super::{aligned, dimension, finite};
use crate::{Error, HistogramGeometry, NumericValues};

const MAX_BINS: usize = 1_000_000;
const MAX_KDE_POINTS: usize = 2_048;
const MAX_RUGS: usize = 1_000_000;

impl HistogramGeometry {
    pub(crate) fn validate(&self) -> Result<(f64, f64), Error> {
        dimension(&self.value_dimension, "histogram value dimension")?;
        dimension(&self.density_dimension, "histogram density dimension")?;
        aligned(
            "histogram bins",
            self.bin_lower.len(),
            &[
                ("bin_upper", self.bin_upper.len()),
                ("bin_density", self.bin_density.len()),
            ],
        )?;
        aligned(
            "histogram rug",
            self.rug_x.len(),
            &[
                ("rug_baseline", self.rug_baseline.len()),
                ("rug_top", self.rug_top.len()),
            ],
        )?;
        if self.bin_lower.is_empty() || self.bin_lower.len() > MAX_BINS {
            return Err(Error::Data(format!(
                "histogram composite geometry requires 1..={MAX_BINS} bins"
            )));
        }
        if self.kde_x.len() > MAX_KDE_POINTS || self.rug_x.len() > MAX_RUGS {
            return Err(Error::Data(
                "histogram composite geometry exceeds its KDE/rug safety limit".into(),
            ));
        }
        let mut previous = None;
        let mut positive = false;
        for row in 0..self.bin_lower.len() {
            let lower = finite(&self.bin_lower, row, "histogram bin lower")?;
            let upper = finite(&self.bin_upper, row, "histogram bin upper")?;
            let density = finite(&self.bin_density, row, "histogram bin density")?;
            if lower >= upper || density < 0.0 {
                return Err(Error::Data(format!(
                    "histogram composite bin {row} requires lower < upper and nonnegative density"
                )));
            }
            if previous.is_some_and(|previous| {
                lower.partial_cmp(&previous) != Some(std::cmp::Ordering::Equal)
            }) {
                return Err(Error::Data(format!(
                    "histogram composite bins must be ordered and contiguous at bin {row}"
                )));
            }
            previous = Some(upper);
            positive |= density > 0.0;
        }
        if !positive {
            return Err(Error::Data(
                "histogram composite geometry requires a positive bin density".into(),
            ));
        }
        let range = (
            finite(&self.bin_lower, 0, "histogram bin lower")?,
            finite(
                &self.bin_upper,
                self.bin_upper.len() - 1,
                "histogram bin upper",
            )?,
        );
        validate_kde(&self.kde_x, &self.kde_y, range)?;
        if self.rug_x.is_empty() {
            return Err(Error::Data(
                "histogram composite geometry requires at least one rug segment".into(),
            ));
        }
        for row in 0..self.rug_x.len() {
            let x = finite(&self.rug_x, row, "histogram rug x")?;
            let baseline = finite(&self.rug_baseline, row, "histogram rug baseline")?;
            let top = finite(&self.rug_top, row, "histogram rug top")?;
            if !(range.0..=range.1).contains(&x) || baseline < 0.0 || baseline >= top {
                return Err(Error::Data(format!("histogram composite rug {row} requires x within bin support and 0 <= baseline < top")));
            }
        }
        Ok(range)
    }
}

pub(crate) fn validate_kde(
    x: &NumericValues,
    y: &NumericValues,
    range: (f64, f64),
) -> Result<(), Error> {
    aligned("histogram KDE", x.len(), &[("y", y.len())])?;
    if x.len() < 2 {
        return Err(Error::Data(
            "histogram KDE: a precomputed curve requires at least two points".into(),
        ));
    }
    let mut previous = None;
    let mut positive = false;
    for row in 0..x.len() {
        let x = finite(x, row, "histogram KDE finite x")?;
        let y = finite(y, row, "histogram KDE nonnegative finite density")?;
        if y < 0.0 {
            return Err(Error::Data(format!(
                "histogram KDE: precomputed point {row} requires nonnegative finite density"
            )));
        }
        if !(range.0..=range.1).contains(&x) {
            return Err(Error::Data(format!("histogram KDE: precomputed x {x} at point {row} lies outside histogram bin range {}..{}", range.0, range.1)));
        }
        if previous.is_some_and(|previous| x <= previous) {
            return Err(Error::Data(
                "histogram KDE: precomputed x values must be strictly increasing".into(),
            ));
        }
        previous = Some(x);
        positive |= y > 0.0;
    }
    if !positive {
        return Err(Error::Data(
            "histogram KDE: a precomputed density curve must contain a positive density".into(),
        ));
    }
    Ok(())
}
