//! Certificates for a finite traversal of quantized metric cells, not samples.

use crate::layout::TextPlanError;
use std::sync::Mutex;

/// Intersection of the downward stability intervals of every reached rounding.
/// Positive finite binary64 values have the same ordering as their bit patterns.
#[derive(Debug)]
pub(crate) struct Stability {
    pub(crate) size: f64,
    lower: Mutex<u64>,
}

impl Stability {
    pub(crate) fn new(size: f64, minimum: f64) -> Self {
        Self {
            size,
            lower: Mutex::new(minimum.to_bits()),
        }
    }
    pub(crate) fn lower(&self) -> Result<f64, TextPlanError> {
        self.lower
            .lock()
            .map(|value| f64::from_bits(*value))
            .map_err(|_| TextPlanError::Arithmetic("font fit certificate lock"))
    }
    /// `value` is a monotone rounded affine metric. Bisection finds the exact
    /// first represented size with the observed value, including tie ownership.
    pub(crate) fn observe(
        &self,
        observed: i64,
        mut value: impl FnMut(f64) -> Result<i64, TextPlanError>,
    ) -> Result<(), TextPlanError> {
        let mut lower = self
            .lower
            .lock()
            .map_err(|_| TextPlanError::Arithmetic("font fit certificate lock"))?;
        if value(f64::from_bits(*lower))? == observed {
            return Ok(());
        }
        let mut lo = *lower;
        let mut hi = self.size.to_bits();
        while hi - lo > 1 {
            let middle = lo + (hi - lo) / 2;
            if value(f64::from_bits(middle))? == observed {
                hi = middle;
            } else {
                lo = middle;
            }
        }
        *lower = hi;
        Ok(())
    }
}
