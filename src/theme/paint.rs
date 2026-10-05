//! Immutable byte-channel paint values shared by theme resolution and rendering.

use crate::{Error, Result};

/// Opaque RGB identity; there is no alpha-discarding conversion from `Color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpaqueColor {
    red: u8,
    green: u8,
    blue: u8,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "immutable validated color values expose borrowed getters"
)]
impl OpaqueColor {
    pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "typed hex construction awaits the public scale-input cutover"
        )
    )]
    pub(crate) fn hex(value: &str) -> Result<Self> {
        let [red, green, blue, _] = hex_channels(value, false)?;
        Ok(Self::rgb(red, green, blue))
    }

    pub(crate) const fn red(&self) -> u8 {
        self.red
    }

    pub(crate) const fn green(&self) -> u8 {
        self.green
    }

    pub(crate) const fn blue(&self) -> u8 {
        self.blue
    }

    /// Private adaptation to the current exact-paint/backend tuple inputs.
    pub(crate) const fn as_rgb(self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }
}

/// Complete RGBA paint with one opaque RGB identity and one byte alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Color {
    rgb: OpaqueColor,
    alpha: u8,
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "immutable validated color values expose borrowed getters"
)]
impl Color {
    pub(crate) const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 255)
    }

    pub(crate) const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            rgb: OpaqueColor::rgb(red, green, blue),
            alpha,
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "typed hex construction awaits the public exact-paint cutover"
        )
    )]
    pub(crate) fn hex(value: &str) -> Result<Self> {
        let [red, green, blue, alpha] = hex_channels(value, true)?;
        Ok(Self::rgba(red, green, blue, alpha))
    }

    pub(crate) const fn red(&self) -> u8 {
        self.rgb.red()
    }

    pub(crate) const fn green(&self) -> u8 {
        self.rgb.green()
    }

    pub(crate) const fn blue(&self) -> u8 {
        self.rgb.blue()
    }

    pub(crate) const fn alpha(&self) -> u8 {
        self.alpha
    }

    pub(crate) const fn with_alpha(self, alpha: u8) -> Self {
        Self { alpha, ..self }
    }
}

impl From<OpaqueColor> for Color {
    fn from(rgb: OpaqueColor) -> Self {
        Self { rgb, alpha: 255 }
    }
}

fn hex_channels(value: &str, alpha_allowed: bool) -> Result<[u8; 4]> {
    let bytes = value.as_bytes();
    if !(bytes.len() == 7 || (alpha_allowed && bytes.len() == 9))
        || bytes[0] != b'#'
        || !bytes[1..].iter().all(u8::is_ascii_hexdigit)
    {
        let syntax = if alpha_allowed {
            "#RRGGBB or #RRGGBBAA"
        } else {
            "#RRGGBB"
        };
        return Err(Error::Data(format!(
            "color must be ASCII hex in exactly {syntax} form; got {value:?}"
        )));
    }
    let mut channels = [0, 0, 0, 255];
    for (channel, digits) in channels.iter_mut().zip(bytes[1..].chunks_exact(2)) {
        *channel = (hex_digit(digits[0]) << 4) | hex_digit(digits[1]);
    }
    Ok(channels)
}

fn hex_digit(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => unreachable!("hex channels validate ASCII digits before decoding"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channels(color: Color) -> [u8; 4] {
        [color.red(), color.green(), color.blue(), color.alpha()]
    }

    #[test]
    fn encoded_hex_vectors_preserve_channel_order_case_and_alpha_endpoints() {
        for (encoded, expected) in [
            ("#000000", [0, 0, 0, 255]),
            ("#FFFFFF", [255, 255, 255, 255]),
            ("#01aBfE", [1, 171, 254, 255]),
            ("#12aBcD00", [18, 171, 205, 0]),
            ("#Ff8001ff", [255, 128, 1, 255]),
            ("#23456789", [35, 69, 103, 137]),
        ] {
            assert_eq!(
                channels(Color::hex(encoded).unwrap()),
                expected,
                "{encoded}"
            );
            if encoded.len() == 7 {
                assert_eq!(
                    channels(OpaqueColor::hex(encoded).unwrap().into()),
                    expected,
                    "{encoded}"
                );
            } else {
                assert!(OpaqueColor::hex(encoded).is_err(), "{encoded}");
            }
        }
    }

    #[test]
    fn hex_rejects_noncanonical_lengths_syntax_and_unicode_without_trimming() {
        for invalid in [
            "",
            "#",
            "#123",
            "#1234",
            "#12345",
            "#1234567",
            "#123456789",
            "123456",
            "0x123456",
            " #123456",
            "#123456 ",
            "#123456\n",
            "red",
            "rgb(1,2,3)",
            "#g12345",
            "#12345G",
            "#123456gg",
            "＃123456",
            "#１２３",
            "#é1234",
            "#1234é",
            "#123456é",
            "#12\0abc",
        ] {
            assert!(
                matches!(Color::hex(invalid), Err(Error::Data(_))),
                "{invalid:?}"
            );
            assert!(
                matches!(OpaqueColor::hex(invalid), Err(Error::Data(_))),
                "{invalid:?}"
            );
        }
        for length in 0..=12 {
            let encoded = format!("#{}", "a".repeat(length));
            assert_eq!(Color::hex(&encoded).is_ok(), length == 6 || length == 8);
            assert_eq!(OpaqueColor::hex(&encoded).is_ok(), length == 6);
        }
    }

    #[test]
    fn every_byte_channel_matches_independently_encoded_hex() {
        for value in 0..=u8::MAX {
            for expected in [
                [value, 1, 255, 0],
                [0, value, 1, 255],
                [255, 0, value, 1],
                [1, 255, 0, value],
            ] {
                let [red, green, blue, alpha] = expected;
                let encoded = format!("#{red:02X}{green:02x}{blue:02X}{alpha:02x}");
                let parsed = Color::hex(&encoded).unwrap();
                assert_eq!(channels(parsed), expected);
                assert_eq!(parsed, Color::rgba(red, green, blue, alpha));
                let rgb_encoded = format!("#{red:02x}{green:02X}{blue:02x}");
                assert_eq!(
                    OpaqueColor::hex(&rgb_encoded).unwrap(),
                    OpaqueColor::rgb(red, green, blue)
                );
                assert_eq!(
                    channels(Color::from(OpaqueColor::rgb(red, green, blue))),
                    [red, green, blue, 255]
                );
            }
        }
    }

    #[test]
    fn alpha_replacement_preserves_rgb_and_source_identity() {
        let source = Color::hex("#17a9E340").unwrap();
        for alpha in 0..=u8::MAX {
            assert_eq!(channels(source.with_alpha(alpha)), [23, 169, 227, alpha]);
            assert_eq!(
                source.with_alpha(0).with_alpha(alpha),
                source.with_alpha(alpha)
            );
        }
        assert_eq!(channels(source), [23, 169, 227, 64]);
        assert_ne!(source, source.with_alpha(255));
        assert_eq!(Color::rgb(23, 169, 227), source.with_alpha(255));
    }
}
