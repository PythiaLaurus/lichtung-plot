//! Fixed synthetic observations, without a random dependency or downloaded data.

/// Fixed Box–Muller inputs, not measurements or a statistical resampling procedure.
pub fn normal(index: u32) -> f64 {
    let index = f64::from(index) + 1.0;
    let u = (index * 0.618_033_988_749_895).fract();
    let v = (index * 0.414_213_562_373_095).fract();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}
