//! Immutable byte-channel paint values shared by theme resolution and rendering.

use crate::{Error, Result};

/// Opaque RGB identity; there is no alpha-discarding conversion from `Color`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpaqueColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl OpaqueColor {
    /// Construct an exact opaque RGB identity for data scales.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Parse exactly ASCII `#RRGGBB`, accepting either hex case and no alpha.
    pub fn hex(value: &str) -> Result<Self> {
        let [red, green, blue, _] = hex_channels(value, false)?;
        Ok(Self::rgb(red, green, blue))
    }

    /// Inspect the red channel.
    pub const fn red(&self) -> u8 {
        self.red
    }

    /// Inspect the green channel.
    pub const fn green(&self) -> u8 {
        self.green
    }

    /// Inspect the blue channel.
    pub const fn blue(&self) -> u8 {
        self.blue
    }

    /// Private adaptation to the current exact-paint/backend tuple inputs.
    pub(crate) const fn as_rgb(self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }
}

/// Complete RGBA paint with one opaque RGB identity and one byte alpha.
///
/// Channels are exact bytes, including fully transparent and opaque alpha.
/// Use [`Color::alpha`] to replace alpha without multiplying it.
///
/// ```
/// use lichtung_plot::Color;
/// let paint = Color::hex("#12345680")?;
/// assert_eq!(paint.get_alpha(), 128);
/// assert_eq!(paint.alpha(255), Color::rgb(18, 52, 86));
/// # Ok::<(), lichtung_plot::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    rgb: OpaqueColor,
    alpha: u8,
}

impl Color {
    /// Construct an opaque paint from exact RGB channels.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 255)
    }

    /// Construct a paint from exact RGB and alpha channels.
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            rgb: OpaqueColor::rgb(red, green, blue),
            alpha,
        }
    }

    /// Parse exactly `#RRGGBB` or `#RRGGBBAA`, accepting either ASCII hex case.
    ///
    /// Six-digit input is opaque. Surrounding whitespace, shorthand, named
    /// colors and other CSS syntax are rejected.
    pub fn hex(value: &str) -> Result<Self> {
        let [red, green, blue, alpha] = hex_channels(value, true)?;
        Ok(Self::rgba(red, green, blue, alpha))
    }

    /// Inspect the red channel.
    pub const fn red(&self) -> u8 {
        self.rgb.red()
    }

    /// Inspect the green channel.
    pub const fn green(&self) -> u8 {
        self.rgb.green()
    }

    /// Inspect the blue channel.
    pub const fn blue(&self) -> u8 {
        self.rgb.blue()
    }

    /// Inspect alpha: zero is transparent and 255 is opaque.
    pub const fn get_alpha(&self) -> u8 {
        self.alpha
    }

    /// Replace alpha, retaining all RGB channels exactly.
    #[must_use]
    pub const fn alpha(self, alpha: u8) -> Self {
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
        [color.red(), color.green(), color.blue(), color.get_alpha()]
    }

    #[test]
    fn encoded_hex_vectors_preserve_channels_and_reject_nonliteral_syntax() {
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
        for invalid in [
            "",
            "#123",
            "#12345G",
            " #123456",
            "#123456 ",
            "#1234567",
            "red",
            "rgb(1,2,3)",
            "＃123456",
        ] {
            assert!(Color::hex(invalid).is_err(), "{invalid:?}");
            assert!(OpaqueColor::hex(invalid).is_err(), "{invalid:?}");
        }
    }
}
