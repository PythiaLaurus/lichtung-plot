use std::cmp::Ordering;

use super::nfc_data::{CANONICAL_COMBINING_CLASS, CANONICAL_COMPOSITION, CANONICAL_DECOMPOSITION};
use super::{append, reserve, search, FontError};

const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

pub(super) fn normalize(value: &str) -> Result<String, FontError> {
    u32::try_from(value.len()).map_err(|_| FontError::Arithmetic("NFC input length"))?;
    let mut decomposed = Vec::new();
    for scalar in value.chars() {
        decompose(scalar, &mut decomposed)?;
    }
    canonical_order(&mut decomposed);
    recompose(&mut decomposed);
    let mut output = String::new();
    for scalar in decomposed {
        append(&mut output, scalar.encode_utf8(&mut [0; 4]))?;
    }
    Ok(output)
}

fn decompose(scalar: char, output: &mut Vec<char>) -> Result<(), FontError> {
    let codepoint = scalar as u32;
    if (S_BASE..S_BASE + S_COUNT).contains(&codepoint) {
        let syllable = codepoint - S_BASE;
        reserve(output, 3)?;
        output.push(char::from_u32(L_BASE + syllable / N_COUNT).expect("valid Hangul jamo"));
        output.push(
            char::from_u32(V_BASE + (syllable % N_COUNT) / T_COUNT).expect("valid Hangul jamo"),
        );
        let trailing = syllable % T_COUNT;
        if trailing != 0 {
            output.push(char::from_u32(T_BASE + trailing).expect("valid Hangul jamo"));
        }
        return Ok(());
    }

    if let Ok(index) = search(CANONICAL_DECOMPOSITION, |&(candidate, _)| {
        candidate.cmp(&codepoint)
    }) {
        for part in CANONICAL_DECOMPOSITION[index].1.chars() {
            decompose(part, output)?;
        }
    } else {
        reserve(output, 1)?;
        output.push(scalar);
    }
    Ok(())
}

fn canonical_order(scalars: &mut [char]) {
    for index in 1..scalars.len() {
        let class = canonical_combining_class(scalars[index]);
        if class == 0 {
            continue;
        }

        let mut insertion = index;
        while insertion > 0 {
            let preceding_class = canonical_combining_class(scalars[insertion - 1]);
            if preceding_class == 0 || preceding_class <= class {
                break;
            }
            scalars.swap(insertion - 1, insertion);
            insertion -= 1;
        }
    }
}

fn recompose(decomposed: &mut Vec<char>) {
    let mut written = 0;
    let mut starter = None;
    let mut last_class = 0;

    for index in 0..decomposed.len() {
        let scalar = decomposed[index];
        let class = canonical_combining_class(scalar);
        if let Some(starter_index) = starter {
            if last_class == 0 || last_class < class {
                if let Some(composite) = compose(decomposed[starter_index], scalar) {
                    decomposed[starter_index] = composite;
                    continue;
                }
            }
        }

        if class == 0 {
            starter = Some(written);
        }
        last_class = class;
        decomposed[written] = scalar;
        written += 1;
    }

    decomposed.truncate(written);
}

fn canonical_combining_class(scalar: char) -> u8 {
    let codepoint = scalar as u32;
    search(CANONICAL_COMBINING_CLASS, |&(start, end, _)| {
        if end < codepoint {
            Ordering::Less
        } else if start > codepoint {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    })
    .map_or(0, |index| CANONICAL_COMBINING_CLASS[index].2)
}

fn compose(first: char, second: char) -> Option<char> {
    if let Some(composed) = compose_hangul(first, second) {
        Some(composed)
    } else {
        let pair = (first as u32, second as u32);
        search(CANONICAL_COMPOSITION, |&(first, second, _)| {
            (first, second).cmp(&pair)
        })
        .ok()
        .and_then(|index| char::from_u32(CANONICAL_COMPOSITION[index].2))
    }
}

fn compose_hangul(first: char, second: char) -> Option<char> {
    let first = first as u32;
    let second = second as u32;
    if (L_BASE..L_BASE + L_COUNT).contains(&first) && (V_BASE..V_BASE + V_COUNT).contains(&second) {
        let syllable = S_BASE + (first - L_BASE) * N_COUNT + (second - V_BASE) * T_COUNT;
        return char::from_u32(syllable);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&first)
        && (first - S_BASE).is_multiple_of(T_COUNT)
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&second)
    {
        return char::from_u32(first + second - T_BASE);
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordering_preserves_equal_class_marks() {
        let mut scalars = ['\u{0315}', '\u{0300}', '\u{0300}'];
        super::canonical_order(&mut scalars);
        assert_eq!(scalars, ['\u{0300}', '\u{0300}', '\u{0315}']);
    }

    fn normalize(value: &str) -> String {
        super::normalize(value).unwrap()
    }

    #[test]
    fn canonically_decomposes_before_recomposition() {
        assert_eq!(normalize("\u{212B}"), "\u{00C5}");
    }

    #[test]
    fn unicode_17_combining_mark_is_a_unicode_16_starter() {
        assert_eq!(normalize("\u{0315}\u{1ACF}"), "\u{0315}\u{1ACF}");
    }
}
