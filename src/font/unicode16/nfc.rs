use std::cmp::Ordering;

use super::nfc_data::{CANONICAL_COMBINING_CLASS, CANONICAL_COMPOSITION, CANONICAL_DECOMPOSITION};

const S_BASE: u32 = 0xAC00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11A7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

pub(super) fn normalize(value: &str) -> String {
    let mut decomposed = Vec::new();
    for scalar in value.chars() {
        decompose(scalar, &mut decomposed);
    }
    canonical_order(&mut decomposed);
    recompose(decomposed).into_iter().collect()
}

fn decompose(scalar: char, output: &mut Vec<char>) {
    let codepoint = scalar as u32;
    if (S_BASE..S_BASE + S_COUNT).contains(&codepoint) {
        let syllable = codepoint - S_BASE;
        output.push(char::from_u32(L_BASE + syllable / N_COUNT).expect("valid Hangul jamo"));
        output.push(
            char::from_u32(V_BASE + (syllable % N_COUNT) / T_COUNT).expect("valid Hangul jamo"),
        );
        let trailing = syllable % T_COUNT;
        if trailing != 0 {
            output.push(char::from_u32(T_BASE + trailing).expect("valid Hangul jamo"));
        }
        return;
    }

    if let Ok(index) =
        CANONICAL_DECOMPOSITION.binary_search_by_key(&codepoint, |&(codepoint, _)| codepoint)
    {
        for part in CANONICAL_DECOMPOSITION[index].1.chars() {
            decompose(part, output);
        }
    } else {
        output.push(scalar);
    }
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

fn recompose(decomposed: Vec<char>) -> Vec<char> {
    let mut composed = Vec::with_capacity(decomposed.len());
    let mut starter = None;
    let mut last_class = 0;

    for scalar in decomposed {
        let class = canonical_combining_class(scalar);
        if let Some(starter_index) = starter {
            if last_class == 0 || last_class < class {
                if let Some(composite) = compose(composed[starter_index], scalar) {
                    composed[starter_index] = composite;
                    continue;
                }
            }
        }

        if class == 0 {
            starter = Some(composed.len());
        }
        last_class = class;
        composed.push(scalar);
    }

    composed
}

fn canonical_combining_class(scalar: char) -> u8 {
    let codepoint = scalar as u32;
    CANONICAL_COMBINING_CLASS
        .binary_search_by(|&(start, end, _)| {
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
    compose_hangul(first, second).or_else(|| {
        let pair = (first as u32, second as u32);
        CANONICAL_COMPOSITION
            .binary_search_by_key(&pair, |&(first, second, _)| (first, second))
            .ok()
            .and_then(|index| char::from_u32(CANONICAL_COMPOSITION[index].2))
    })
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
    use super::normalize;

    #[test]
    fn composes_canonical_pairs_and_hangul() {
        assert_eq!(normalize("e\u{0301}"), "\u{00E9}");
        assert_eq!(normalize("\u{1100}\u{1161}\u{11A8}"), "\u{AC01}");
    }

    #[test]
    fn orders_marks_before_composing_and_honors_exclusions() {
        assert_eq!(normalize("a\u{0315}\u{0300}"), "\u{00E0}\u{0315}");
        assert_eq!(normalize("\u{0915}\u{093C}"), "\u{0915}\u{093C}");
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
