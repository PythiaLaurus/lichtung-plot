//! Unicode 16.0 normalization for font family matching only, never rendered text.

use super::canonical::FontError;
use std::cmp::Ordering;

mod casefold_data;
#[cfg(test)]
mod conformance;
mod nfc;
mod nfc_data;

use casefold_data::NFKC_CASEFOLD;

pub(super) fn nfkc_casefold(value: &str) -> Result<String, FontError> {
    u32::try_from(value.len()).map_err(|_| FontError::Arithmetic("Unicode input length"))?;
    let mut mapped = String::new();
    for scalar in value.chars() {
        let codepoint = scalar as u32;
        let replacement = search(NFKC_CASEFOLD, |(start, end, _)| {
            if *end < codepoint {
                Ordering::Less
            } else if *start > codepoint {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        if let Ok(index) = replacement {
            append(&mut mapped, NFKC_CASEFOLD[index].2)?;
        } else {
            append(&mut mapped, scalar.encode_utf8(&mut [0; 4]))?;
        }
    }
    nfc::normalize(&mapped)
}

fn search<T>(table: &[T], mut compare: impl FnMut(&T) -> Ordering) -> Result<usize, usize> {
    let (mut low, mut high) = (0, table.len());
    while low < high {
        let middle = low + (high - low) / 2;
        match compare(&table[middle]) {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(middle),
        }
    }
    Err(low)
}

fn reserve<T>(values: &mut Vec<T>, additional: usize) -> Result<(), FontError> {
    let required = values
        .len()
        .checked_add(additional)
        .ok_or(FontError::Arithmetic("Unicode vector length"))?;
    u32::try_from(required).map_err(|_| FontError::Arithmetic("Unicode vector length"))?;
    if required > values.capacity() {
        let target = required.max(
            values
                .capacity()
                .checked_mul(2)
                .ok_or(FontError::Arithmetic("Unicode vector capacity"))?,
        );
        target
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(FontError::Arithmetic("Unicode vector bytes"))?;
        values
            .try_reserve_exact(target - values.len())
            .map_err(|_| FontError::Allocation("Unicode vector"))?;
    }
    Ok(())
}

fn append(output: &mut String, value: &str) -> Result<(), FontError> {
    let required = output
        .len()
        .checked_add(value.len())
        .ok_or(FontError::Arithmetic("Unicode string length"))?;
    u32::try_from(required).map_err(|_| FontError::Arithmetic("Unicode string length"))?;
    if required > output.capacity() {
        let target = required.max(
            output
                .capacity()
                .checked_mul(2)
                .ok_or(FontError::Arithmetic("Unicode string capacity"))?,
        );
        output
            .try_reserve_exact(target - output.len())
            .map_err(|_| FontError::Allocation("Unicode string"))?;
    }
    output.push_str(value);
    Ok(())
}

#[cfg(test)]
pub(super) fn folded_for_test(value: &str) -> String {
    nfkc_casefold(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::folded_for_test as nfkc_casefold;

    #[test]
    fn checked_unicode_lengths_refuse_without_reservation() {
        let mut scalars = Vec::<char>::new();
        assert!(matches!(
            super::reserve(&mut scalars, usize::MAX),
            Err(super::FontError::Arithmetic(_))
        ));
        assert_eq!(scalars.capacity(), 0);
    }

    #[test]
    fn unicode_16_versions_are_exact() {
        assert_eq!(unicode_script::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_bidi::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_segmentation::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_properties::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_ccc::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_bidi_mirroring::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(nfkc_casefold("\u{0315}\u{1ACF}"), "\u{0315}\u{1ACF}");
    }
}
