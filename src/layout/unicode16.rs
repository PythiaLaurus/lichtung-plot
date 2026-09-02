//! Unicode 16.0 default line breaking for exact text planning.

use std::cmp::Ordering;

mod data;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BreakOpportunity {
    Allowed,
    Mandatory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub(super) enum Class {
    AI,
    AK,
    AL,
    AP,
    AS,
    B2,
    BA,
    BB,
    BK,
    CB,
    CJ,
    CL,
    CM,
    CP,
    CR,
    EB,
    EM,
    EX,
    GL,
    H2,
    H3,
    HL,
    HY,
    ID,
    IN,
    IS,
    JL,
    JT,
    JV,
    LF,
    NL,
    NS,
    NU,
    OP,
    PO,
    PR,
    QU,
    RI,
    SA,
    SG,
    SP,
    SY,
    VF,
    VI,
    WJ,
    XX,
    ZW,
    ZWJ,
}

#[derive(Clone, Copy)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "UAX #14 rules query four independent Unicode binary properties"
)]
struct Scalar {
    offset: usize,
    codepoint: u32,
    class: Class,
    accepts_combining: bool,
    ends_with_zwj: bool,
    east_asian: bool,
    initial_punctuation: bool,
    final_punctuation: bool,
    unassigned_extended_pictographic: bool,
}

pub(super) fn linebreaks(value: &str) -> Vec<(usize, BreakOpportunity)> {
    let raw_scalars = value
        .char_indices()
        .map(|(offset, scalar)| {
            let codepoint = scalar as u32;
            let original = line_break_class(codepoint);
            let class = match original {
                Class::CJ => Class::NS,
                Class::SA if in_ranges(codepoint, data::MARK) => Class::CM,
                Class::AI | Class::SG | Class::XX | Class::SA => Class::AL,
                class => class,
            };
            Scalar {
                offset,
                codepoint,
                class,
                accepts_combining: false,
                ends_with_zwj: false,
                east_asian: in_ranges(codepoint, data::EAST_ASIAN),
                initial_punctuation: in_ranges(codepoint, data::INITIAL_PUNCTUATION),
                final_punctuation: in_ranges(codepoint, data::FINAL_PUNCTUATION),
                unassigned_extended_pictographic: in_ranges(
                    codepoint,
                    data::UNASSIGNED_EXTENDED_PICTOGRAPHIC,
                ),
            }
        })
        .collect::<Vec<_>>();

    // LB9 removes attached combining marks from all later rule matching. LB10
    // turns only marks without an eligible base into independent AL scalars.
    let mut scalars: Vec<Scalar> = Vec::with_capacity(raw_scalars.len());
    for mut scalar in raw_scalars {
        if matches!(scalar.class, Class::CM | Class::ZWJ) {
            if let Some(base) = scalars.last_mut().filter(|base| base.accepts_combining) {
                base.ends_with_zwj |= scalar.class == Class::ZWJ;
                continue;
            }
            scalar.ends_with_zwj = scalar.class == Class::ZWJ;
            scalar.class = Class::AL;
        } else {
            scalar.accepts_combining = !matches!(
                scalar.class,
                Class::BK | Class::CR | Class::LF | Class::NL | Class::SP | Class::ZW
            );
        }
        scalars.push(scalar);
    }

    let mut breaks = Vec::new();
    for boundary in 1..scalars.len() {
        if let Some(opportunity) = opportunity_at(&scalars, boundary) {
            breaks.push((scalars[boundary].offset, opportunity));
        }
    }
    breaks.push((value.len(), BreakOpportunity::Mandatory));
    breaks
}

#[expect(
    clippy::too_many_lines,
    reason = "one ordered function keeps UAX #14 rule precedence directly auditable"
)]
fn opportunity_at(scalars: &[Scalar], boundary: usize) -> Option<BreakOpportunity> {
    let left = scalars[boundary - 1];
    let right = scalars[boundary];

    // LB4-LB6: mandatory breaks and their protected leading side.
    if left.class == Class::CR && right.class == Class::LF {
        return None;
    }
    if matches!(left.class, Class::BK | Class::CR | Class::LF | Class::NL) {
        return Some(BreakOpportunity::Mandatory);
    }
    if matches!(right.class, Class::BK | Class::CR | Class::LF | Class::NL) {
        return None;
    }

    // LB7-LB8a: explicit opportunities and join suppression.
    if matches!(right.class, Class::SP | Class::ZW) {
        return None;
    }
    if previous_non_space(scalars, boundary).is_some_and(|index| scalars[index].class == Class::ZW)
    {
        return Some(BreakOpportunity::Allowed);
    }
    if left.ends_with_zwj {
        return None;
    }

    // LB11-LB14: joiners, glue, and opening/closing punctuation.
    if left.class == Class::WJ || right.class == Class::WJ {
        return None;
    }
    if left.class == Class::GL {
        return None;
    }
    if right.class == Class::GL && !matches!(left.class, Class::SP | Class::BA | Class::HY) {
        return None;
    }
    if matches!(right.class, Class::CL | Class::CP | Class::EX | Class::SY) {
        return None;
    }
    if previous_non_space(scalars, boundary).is_some_and(|index| scalars[index].class == Class::OP)
    {
        return None;
    }

    // LB15a-LB15d: context-sensitive initial/final and decimal punctuation.
    if let Some(quote) = previous_non_space(scalars, boundary) {
        if scalars[quote].class == Class::QU && scalars[quote].initial_punctuation {
            let starts_context = quote == 0
                || matches!(
                    scalars[quote - 1].class,
                    Class::BK
                        | Class::CR
                        | Class::LF
                        | Class::NL
                        | Class::OP
                        | Class::QU
                        | Class::GL
                        | Class::SP
                        | Class::ZW
                );
            if starts_context {
                return None;
            }
        }
    }
    if right.class == Class::QU
        && right.final_punctuation
        && (boundary + 1 == scalars.len()
            || matches!(
                scalars[boundary + 1].class,
                Class::SP
                    | Class::GL
                    | Class::WJ
                    | Class::CL
                    | Class::QU
                    | Class::CP
                    | Class::EX
                    | Class::IS
                    | Class::SY
                    | Class::BK
                    | Class::CR
                    | Class::LF
                    | Class::NL
                    | Class::ZW
            ))
    {
        return None;
    }
    if left.class == Class::SP
        && right.class == Class::IS
        && boundary + 1 < scalars.len()
        && scalars[boundary + 1].class == Class::NU
    {
        return Some(BreakOpportunity::Allowed);
    }
    if right.class == Class::IS {
        return None;
    }

    // LB16-LB18: nonstarters, paired dashes, and ordinary spaces.
    if right.class == Class::NS
        && previous_non_space(scalars, boundary)
            .is_some_and(|index| matches!(scalars[index].class, Class::CL | Class::CP))
    {
        return None;
    }
    if right.class == Class::B2
        && previous_non_space(scalars, boundary)
            .is_some_and(|index| scalars[index].class == Class::B2)
    {
        return None;
    }
    if left.class == Class::SP {
        return Some(BreakOpportunity::Allowed);
    }

    // LB19-LB19a: unresolved quotation marks and East Asian context.
    if right.class == Class::QU && !right.initial_punctuation {
        return None;
    }
    if left.class == Class::QU && !left.final_punctuation {
        return None;
    }
    let right_quote_lacks_east_asian_context = right.class == Class::QU
        && (!left.east_asian || boundary + 1 == scalars.len() || !scalars[boundary + 1].east_asian);
    let left_quote_lacks_east_asian_context = left.class == Class::QU
        && (!right.east_asian || boundary == 1 || !scalars[boundary - 2].east_asian);
    if right_quote_lacks_east_asian_context || left_quote_lacks_east_asian_context {
        return None;
    }

    // LB20-LB22: conditional breaks, word-initial hyphens, and inseparables.
    if left.class == Class::CB || right.class == Class::CB {
        return Some(BreakOpportunity::Allowed);
    }
    if right.class == Class::AL
        && (left.class == Class::HY || left.codepoint == 0x2010)
        && (boundary == 1
            || matches!(
                scalars[boundary - 2].class,
                Class::BK
                    | Class::CR
                    | Class::LF
                    | Class::NL
                    | Class::SP
                    | Class::ZW
                    | Class::CB
                    | Class::GL
            ))
    {
        return None;
    }
    if matches!(right.class, Class::BA | Class::HY | Class::NS) || left.class == Class::BB {
        return None;
    }
    if right.class != Class::HL
        && boundary >= 2
        && scalars[boundary - 2].class == Class::HL
        && (left.class == Class::HY || (left.class == Class::BA && !left.east_asian))
    {
        return None;
    }
    if left.class == Class::SY && right.class == Class::HL {
        return None;
    }
    if right.class == Class::IN {
        return None;
    }

    // LB23-LB25: alphanumerics, affixes, and numeric expressions.
    if (is_alphabetic(left.class) && right.class == Class::NU)
        || (left.class == Class::NU && is_alphabetic(right.class))
    {
        return None;
    }
    if (left.class == Class::PR && matches!(right.class, Class::ID | Class::EB | Class::EM))
        || (matches!(left.class, Class::ID | Class::EB | Class::EM) && right.class == Class::PO)
    {
        return None;
    }
    if (matches!(left.class, Class::PR | Class::PO) && is_alphabetic(right.class))
        || (is_alphabetic(left.class) && matches!(right.class, Class::PR | Class::PO))
    {
        return None;
    }
    if matches!(right.class, Class::PO | Class::PR) && numeric_prefix_before(scalars, boundary) {
        return None;
    }
    if matches!(left.class, Class::PO | Class::PR) && numeric_suffix_after(scalars, boundary) {
        return None;
    }
    if matches!(left.class, Class::HY | Class::IS) && right.class == Class::NU {
        return None;
    }
    if right.class == Class::NU && numeric_prefix_before(scalars, boundary) {
        return None;
    }

    // LB26-LB27: Korean syllable blocks.
    if (left.class == Class::JL
        && matches!(right.class, Class::JL | Class::JV | Class::H2 | Class::H3))
        || (matches!(left.class, Class::JV | Class::H2)
            && matches!(right.class, Class::JV | Class::JT))
        || (matches!(left.class, Class::JT | Class::H3) && right.class == Class::JT)
        || (is_hangul(left.class) && right.class == Class::PO)
        || (left.class == Class::PR && is_hangul(right.class))
    {
        return None;
    }

    // LB28-LB29: alphabetic and Brahmic orthographic syllables.
    if is_alphabetic(left.class) && is_alphabetic(right.class) {
        return None;
    }
    if (left.class == Class::AP && is_brahmic_base(right))
        || (is_brahmic_base(left) && matches!(right.class, Class::VF | Class::VI))
        || (left.class == Class::VI
            && boundary >= 2
            && is_brahmic_base(scalars[boundary - 2])
            && (right.class == Class::AK || right.codepoint == 0x25CC))
        || (is_brahmic_base(left)
            && is_brahmic_base(right)
            && boundary + 1 < scalars.len()
            && scalars[boundary + 1].class == Class::VF)
    {
        return None;
    }
    if left.class == Class::IS && is_alphabetic(right.class) {
        return None;
    }

    // LB30-LB30b: parentheses, regional indicators, and emoji modifiers.
    if ((is_alphabetic(left.class) || left.class == Class::NU)
        && right.class == Class::OP
        && !right.east_asian)
        || (left.class == Class::CP
            && !left.east_asian
            && (is_alphabetic(right.class) || right.class == Class::NU))
    {
        return None;
    }
    if left.class == Class::RI && right.class == Class::RI {
        let preceding = scalars[..boundary]
            .iter()
            .rev()
            .take_while(|scalar| scalar.class == Class::RI)
            .count();
        if preceding % 2 == 1 {
            return None;
        }
    }
    if right.class == Class::EM
        && (left.class == Class::EB || left.unassigned_extended_pictographic)
    {
        return None;
    }

    // LB31: break everywhere else.
    Some(BreakOpportunity::Allowed)
}

fn numeric_prefix_before(scalars: &[Scalar], boundary: usize) -> bool {
    let mut index = boundary;
    if index > 0 && matches!(scalars[index - 1].class, Class::CL | Class::CP) {
        index -= 1;
    }
    while index > 0 && matches!(scalars[index - 1].class, Class::SY | Class::IS) {
        index -= 1;
    }
    index > 0 && scalars[index - 1].class == Class::NU
}

fn numeric_suffix_after(scalars: &[Scalar], boundary: usize) -> bool {
    let mut index = boundary;
    if index < scalars.len() && scalars[index].class == Class::OP {
        index += 1;
        if index < scalars.len() && scalars[index].class == Class::IS {
            index += 1;
        }
    }
    index < scalars.len() && scalars[index].class == Class::NU
}

fn previous_non_space(scalars: &[Scalar], boundary: usize) -> Option<usize> {
    (0..boundary)
        .rev()
        .find(|index| scalars[*index].class != Class::SP)
}

fn is_alphabetic(class: Class) -> bool {
    matches!(class, Class::AL | Class::HL)
}

fn is_hangul(class: Class) -> bool {
    matches!(
        class,
        Class::JL | Class::JV | Class::JT | Class::H2 | Class::H3
    )
}

fn is_brahmic_base(scalar: Scalar) -> bool {
    matches!(scalar.class, Class::AK | Class::AS) || scalar.codepoint == 0x25CC
}

fn line_break_class(codepoint: u32) -> Class {
    data::LINE_BREAK
        .binary_search_by(|(start, end, _)| compare_range(*start, *end, codepoint))
        .map_or(Class::XX, |index| data::LINE_BREAK[index].2)
}

fn in_ranges(codepoint: u32, ranges: &[(u32, u32)]) -> bool {
    ranges
        .binary_search_by(|(start, end)| compare_range(*start, *end, codepoint))
        .is_ok()
}

fn compare_range(start: u32, end: u32, codepoint: u32) -> Ordering {
    if end < codepoint {
        Ordering::Less
    } else if start > codepoint {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

#[cfg(test)]
mod tests {
    use super::{line_break_class, linebreaks, BreakOpportunity, Class};

    fn allowed_offsets(value: &str) -> Vec<usize> {
        linebreaks(value)
            .into_iter()
            .filter_map(|(offset, opportunity)| {
                (opportunity == BreakOpportunity::Allowed).then_some(offset)
            })
            .collect()
    }

    #[test]
    fn unicode_16_brahmic_syllables_follow_lb28a() {
        let akara = "\u{1B05}";
        let virama = "\u{1B44}";
        let pangolat = "\u{1BF2}";
        assert!(allowed_offsets(&format!("{akara}{virama}{akara}")).is_empty());
        assert!(allowed_offsets(&format!("{akara}{pangolat}")).is_empty());
        assert_eq!(
            allowed_offsets(&format!("{akara}{akara}")),
            vec![akara.len()]
        );
    }

    #[test]
    fn line_break_offsets_are_utf8_byte_offsets() {
        assert_eq!(allowed_offsets("a b"), vec![2]);
        assert_eq!(allowed_offsets("\u{4E00}\u{4E01}"), vec![3]);
    }

    #[test]
    fn unicode_16_quote_rules_and_version_boundary_are_exact() {
        assert!(allowed_offsets("\u{23E9} \u{00BB}").is_empty());
        assert_eq!(line_break_class(0xA7F1), Class::XX);
    }
}
