//! Bounded numeric samples join the existing publication release benchmark.

use crate::numeric::power::{certified::PowerFormatter, Log10Operand};
use crate::{NumericFormat, NumericNotation, NumericPrecision};
use std::time::Instant;

pub(super) fn measure() {
    println!("numeric_set_header,notation,run,members,distinct_members,precision,label_bytes,milliseconds");
    for (notation, count, distinct) in [
        (NumericNotation::Decimal, 32_u32, 32_u32),
        (NumericNotation::Decimal, 1024, 32),
        (NumericNotation::Decimal, 1024, 1024),
        (NumericNotation::Scientific, 1024, 1024),
    ] {
        let values: Vec<_> = (0..count)
            .map(|index| 1.0 + f64::from(index % distinct) / 1024.0)
            .collect();
        let precision = if notation == NumericNotation::Decimal {
            NumericPrecision::AdaptiveFractional { min: 0, max: 6 }
        } else {
            NumericPrecision::AdaptiveSignificant { min: 1, max: 8 }
        };
        let format = NumericFormat::new(notation, precision).unwrap();
        let mut previous = None;
        for run in 0..3 {
            let start = Instant::now();
            let set = format
                .format_set(std::hint::black_box(&values), false, true)
                .unwrap()
                .unwrap();
            let elapsed = start.elapsed();
            // Members 21/22 collide at fractional 3; dyadic spacing exceeds
            // 10^-4. Scientific values here have exactly one integral digit.
            assert_eq!(
                set.precision,
                if notation == NumericNotation::Decimal {
                    4
                } else {
                    5
                }
            );
            assert!(set
                .members
                .iter()
                .zip(&values)
                .all(|(operand, value)| operand.binary64().unwrap().to_bits() == value.to_bits()));
            let bytes: usize = set.labels.iter().map(String::len).sum();
            println!(
                "numeric_set_sample,{notation:?},{run},{count},{distinct},{},{bytes},{:.6}",
                set.precision,
                elapsed.as_secs_f64() * 1000.0
            );
            if let Some(previous) = &previous {
                assert_eq!(previous, &set);
            }
            previous = Some(set);
        }
    }
    println!("numeric_power_header,run,exponent_bits,precision,working_precisions,constant_hits,constant_misses,label_bytes,milliseconds");
    for run in 0..3 {
        let mut formatter = PowerFormatter::default();
        for (exponent, digits, expected) in [
            (0.5, 2, "3.16"),
            (-0.5, 2, "0.32"),
            (39.5, 0, "3162277660168379331998893544432718533720"),
            (f64::from_bits(1), 2, "1.00"),
        ] {
            let operand = Log10Operand::new(exponent, 0).unwrap();
            let format = NumericFormat::new(
                NumericNotation::Decimal,
                NumericPrecision::Fractional(digits),
            )
            .unwrap();
            let (_, old_hits, old_misses) = formatter.certification_work();
            let start = Instant::now();
            let result = formatter
                .format(&operand, &format.resolve_at(digits).unwrap())
                .unwrap();
            let elapsed = start.elapsed();
            assert_eq!(result.text, expected);
            let (attempts, hits, misses) = formatter.certification_work();
            if exponent.to_bits() == 39.5_f64.to_bits() {
                assert_eq!(attempts, [128, 256]);
            }
            println!(
                "numeric_power_sample,{run},{:016x},{digits},{attempts:?},{},{},{},{:.6}",
                exponent.to_bits(),
                hits - old_hits,
                misses - old_misses,
                result.text.len(),
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }
}
