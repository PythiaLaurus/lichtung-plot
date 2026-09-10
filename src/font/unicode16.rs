//! Unicode 16.0 normalization for font family matching only, never rendered text.

use super::canonical::FontError;
use std::cmp::Ordering;

#[cfg(test)]
mod admission;
mod casefold_data;
#[cfg(test)]
mod conformance;
mod nfc;
mod nfc_data;
#[cfg(test)]
mod regeneration;

use casefold_data::NFKC_CASEFOLD;

pub(super) fn nfkc_casefold(
    value: &str,
    claim: &mut impl FnMut(u64, u64) -> Result<(), FontError>,
) -> Result<String, FontError> {
    u32::try_from(value.len()).map_err(|_| FontError::Arithmetic("Unicode input length"))?;
    claim(0, value.chars().count() as u64)?;
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
            append(&mut mapped, NFKC_CASEFOLD[index].2, claim)?;
        } else {
            append(&mut mapped, scalar.encode_utf8(&mut [0; 4]), claim)?;
        }
    }
    nfc::normalize(&mapped, claim)
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

fn reserve<T>(
    values: &mut Vec<T>,
    additional: usize,
    claim: &mut impl FnMut(u64, u64) -> Result<(), FontError>,
) -> Result<(), FontError> {
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
        let bytes = target
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(FontError::Arithmetic("Unicode vector bytes"))?;
        // Admit the complete replacement allocation, including coexistence.
        claim(
            u64::try_from(bytes).map_err(|_| FontError::Arithmetic("Unicode vector bytes"))?,
            0,
        )?;
        values
            .try_reserve_exact(target - values.len())
            .map_err(|_| FontError::Allocation("Unicode vector"))?;
    }
    Ok(())
}

fn append(
    output: &mut String,
    value: &str,
    claim: &mut impl FnMut(u64, u64) -> Result<(), FontError>,
) -> Result<(), FontError> {
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
        claim(target as u64, 0)?;
        output
            .try_reserve_exact(target - output.len())
            .map_err(|_| FontError::Allocation("Unicode string"))?;
    }
    output.push_str(value);
    Ok(())
}

#[cfg(test)]
pub(super) fn folded_for_test(value: &str) -> String {
    let resources = super::canonical::OperationFontResources::new();
    nfkc_casefold(value, &mut |bytes, work| {
        resources.claim_bytes(bytes)?;
        resources.claim_visits(work)
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{
        folded_for_test as nfkc_casefold,
        nfc_data::{CANONICAL_COMBINING_CLASS, CANONICAL_COMPOSITION, CANONICAL_DECOMPOSITION},
        NFKC_CASEFOLD,
    };

    fn hash_len(digest: &mut Sha256, len: usize) {
        digest.update(u64::try_from(len).unwrap().to_be_bytes());
    }

    #[test]
    fn unicode_16_casefold_sentinels_are_exact() {
        assert_eq!(nfkc_casefold("\u{AB70}"), "\u{13A0}");
        assert_eq!(nfkc_casefold("a\u{034F}\u{200B}\u{2060}\u{FEFF}b"), "ab");
        assert_eq!(nfkc_casefold("\u{A7F1}"), "\u{A7F1}");
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

    #[test]
    fn casefold_mapping_receives_unicode_16_nfc_composition() {
        assert_eq!(nfkc_casefold("A\u{030A}"), "\u{00E5}");
    }

    #[test]
    fn generated_normalization_tables_have_expected_digest() {
        // Hash semantic entries so Rust source formatting does not affect the digest.
        let mut digest = Sha256::new();
        digest.update(b"NFKC_CASEFOLD");
        hash_len(&mut digest, NFKC_CASEFOLD.len());
        for &(start, end, replacement) in NFKC_CASEFOLD {
            digest.update(start.to_be_bytes());
            digest.update(end.to_be_bytes());
            hash_len(&mut digest, replacement.len());
            digest.update(replacement.as_bytes());
        }
        digest.update(b"CANONICAL_COMBINING_CLASS");
        hash_len(&mut digest, CANONICAL_COMBINING_CLASS.len());
        for &(start, end, class) in CANONICAL_COMBINING_CLASS {
            digest.update(start.to_be_bytes());
            digest.update(end.to_be_bytes());
            digest.update([class]);
        }
        digest.update(b"CANONICAL_DECOMPOSITION");
        hash_len(&mut digest, CANONICAL_DECOMPOSITION.len());
        for &(codepoint, decomposition) in CANONICAL_DECOMPOSITION {
            digest.update(codepoint.to_be_bytes());
            hash_len(&mut digest, decomposition.len());
            digest.update(decomposition.as_bytes());
        }
        digest.update(b"CANONICAL_COMPOSITION");
        hash_len(&mut digest, CANONICAL_COMPOSITION.len());
        for &(first, second, composition) in CANONICAL_COMPOSITION {
            digest.update(first.to_be_bytes());
            digest.update(second.to_be_bytes());
            digest.update(composition.to_be_bytes());
        }
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "56796172967f43285a9e98b49d28ecbb0eacb4ed8cf2a9c9d65ba9513cbb8835"
        );
    }
}
