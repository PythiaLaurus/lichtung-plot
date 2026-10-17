//! Independent fixed-point root/product oracle. It calls no production range
//! reduction, logarithm/exponential, quantizer, formatter or acceptance helper.

use num_bigint::{BigInt, BigUint, Sign};

pub(super) struct Operand {
    pub(super) decade: BigInt,
    pub(super) fraction: BigUint,
    pub(super) denominator_bits: usize,
}

pub(super) fn decode(bits: u64, scale: i16) -> Operand {
    let stored = (bits >> 52) & 0x7ff;
    assert!(stored < 0x7ff);
    let fraction = bits & ((1_u64 << 52) - 1);
    let magnitude = if stored == 0 {
        BigUint::from(fraction)
    } else {
        BigUint::from(fraction + (1_u64 << 52)) << usize::try_from(stored - 1).unwrap()
    };
    let numerator = BigInt::from_biguint(
        if bits >> 63 == 0 {
            Sign::Plus
        } else {
            Sign::Minus
        },
        magnitude,
    );
    let unit = BigInt::from(1_u8) << 1074_usize;
    let numerator = numerator + &unit * scale;
    // Signed right shift is an independent floor construction. Check its
    // defining inequalities rather than trusting it as a quotient oracle.
    let decade = &numerator >> 1074_usize;
    assert!(&decade * &unit <= numerator);
    assert!((&decade + 1_u8) * &unit > numerator);
    let mut fraction = BigUint::try_from(numerator - &decade * unit).unwrap();
    let mut denominator_bits = 1074;
    while denominator_bits > 0 && !fraction.bit(0) {
        fraction >>= 1_usize;
        denominator_bits -= 1;
    }
    Operand {
        decade,
        fraction,
        denominator_bits,
    }
}

pub(super) fn bounds(
    fraction: &BigUint,
    denominator_bits: usize,
    precision: usize,
) -> (BigUint, BigUint) {
    assert!(precision >= 128);
    let unit = BigUint::from(1_u8) << precision;
    let mut root_lower = &unit * 10_u8;
    let mut root_upper = root_lower.clone();
    let mut lower = unit.clone();
    let mut upper = unit.clone();
    for bit in (0..denominator_bits).rev() {
        root_lower = square_roots(&(&root_lower * &unit)).0;
        root_upper = square_roots(&(&root_upper * &unit)).1;
        if fraction.bit(u64::try_from(bit).unwrap()) {
            lower = lower * &root_lower / &unit;
            let product = upper * &root_upper;
            upper = (&product + &unit - 1_u8) / &unit;
        }
    }
    assert!(lower >= unit);
    assert!(upper >= lower);
    assert!(&upper - &lower <= BigUint::from(90 * denominator_bits));
    (lower, upper)
}

fn square_roots(value: &BigUint) -> (BigUint, BigUint) {
    if value.bits() == 0 {
        return (BigUint::ZERO, BigUint::ZERO);
    }
    let half = usize::try_from(value.bits().div_ceil(2)).unwrap();
    let limit = (usize::BITS - (half + 1).leading_zeros()) as usize + 2;
    let mut current = BigUint::from(1_u8) << half;
    for _ in 0..limit {
        let next = (&current + value / &current) >> 1_usize;
        if next >= current {
            let square = &current * &current;
            let successor = &current + 1_u8;
            assert!(square <= *value);
            assert!(&successor * &successor > *value);
            let upper = if square == *value {
                current.clone()
            } else {
                successor
            };
            return (current, upper);
        }
        current = next;
    }
    panic!("independent integer square root exceeded its proved iteration guard")
}

/// Independently certify the literal K's physical rounding cell. Start at 256
/// bits and refine on undecided containment; never invoke the endpoint quantizer.
pub(super) fn certify_cell(bits: u64, scale: i16, quantum: i32, coefficient: &BigUint) -> usize {
    let operand = decode(bits, scale);
    let power = &operand.decade - quantum;
    let negative = power.sign() == Sign::Minus;
    assert!(!negative || power == BigInt::from(-1));
    let power = if negative {
        0
    } else {
        u32::try_from(power).unwrap()
    };
    let multiplier = BigUint::from(10_u8).pow(power);
    let lower_cell = if coefficient.bits() == 0 {
        BigUint::ZERO
    } else {
        coefficient * 2_u8 - 1_u8
    };
    let upper_cell = coefficient * 2_u8 + 1_u8;
    let even = !coefficient.bit(0);
    let mut precision = 256;
    loop {
        let (lower, upper) = bounds(&operand.fraction, operand.denominator_bits, precision);
        let unit = BigUint::from(1_u8) << precision;
        let (lower, upper, denominator) = if negative {
            (lower * 2_u8, upper * 2_u8, unit * 10_u8)
        } else {
            (lower * &multiplier * 2_u8, upper * &multiplier * 2_u8, unit)
        };
        let low = &lower_cell * &denominator;
        let high = &upper_cell * &denominator;
        if (lower > low || (even && lower == low)) && (upper < high || (even && upper == high)) {
            return precision;
        }
        assert!(
            lower < high && upper > low,
            "literal expected coefficient is outside the independent power interval"
        );
        precision = precision.checked_mul(2).unwrap();
    }
}

/// For half/quarter exponents, certify the same literal cell by an entirely
/// algebraic power comparison after cancelling the physical decade.
pub(super) fn certify_algebraic_cell(bits: u64, scale: i16, quantum: i32, coefficient: &BigUint) {
    let operand = decode(bits, scale);
    if operand.denominator_bits > 2 {
        return;
    }
    let denominator = 1_u32 << operand.denominator_bits;
    let power = BigInt::from(operand.fraction) + (&operand.decade - quantum) * denominator;
    let positive = power.sign() != Sign::Minus;
    let ten = BigUint::from(10_u8).pow(u32::try_from(power.magnitude()).unwrap());
    let unit = BigUint::from(2_u8).pow(denominator);
    let lower = if coefficient.bits() == 0 {
        BigUint::ZERO
    } else {
        coefficient * 2_u8 - 1_u8
    };
    let upper = coefficient * 2_u8 + 1_u8;
    if positive {
        let target = unit * ten;
        assert!(lower.pow(denominator) < target);
        assert!(upper.pow(denominator) > target);
    } else {
        assert!(lower.pow(denominator) * &ten < unit);
        assert!(upper.pow(denominator) * ten > unit);
    }
}
