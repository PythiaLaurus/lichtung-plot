//! Shared finite coordinate normalization for immutable color and alpha mappings.

use crate::{Error, Result};

/// Exact endpoints, a midpoint for constant color domains, and overflow-safe
/// opposite-extreme normalization. Alpha validates a strictly ascending domain.
pub(crate) fn normalized_fraction(domain: (f64, f64), value: f64) -> Result<f64> {
    if !value.is_finite() || !domain.0.is_finite() || !domain.1.is_finite() {
        return Err(Error::Data(
            "continuous mapping coordinates must be finite".into(),
        ));
    }
    if domain.0 > domain.1 {
        return Err(Error::Data(
            "continuous mapping domain must be ordered".into(),
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
            "continuous mapping fraction is not finite".into(),
        ));
    }
    Ok(fraction.clamp(0.0, 1.0))
}
