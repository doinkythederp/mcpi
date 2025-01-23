use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::sync::LazyLock;

use derive_more::derive::AsRef;
use nalgebra::{Point, Scalar};

use crate::{Result, WorldError};

pub fn parse_point<T, E, const D: usize>(s: &str) -> Result<nalgebra::Point<T, D>>
where
    T: std::str::FromStr<Err = E> + Scalar,
    WorldError: From<E>,
{
    let parts = s
        .splitn(D, ',')
        .map(|s| s.parse())
        .collect::<Result<Vec<T>, E>>()?;
    Ok(Point::<T, D>::from_slice(&parts))
}

// Port of Minecraft Pi: Reborn's character handling to Rust

const CP437_CHARACTERS: usize = 256;
/// Used to convert a CP437 character to a Unicode character.
#[rustfmt::skip]
pub static CP437_TO_STR: [char; CP437_CHARACTERS] = [
    '\0', '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '\n', '♂', '♀', '♪', '♫', '☼',
    '►', '◄', '↕', '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼',
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
    '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '⌂',
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
	'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', '©'
];

pub static CHAR_TO_CP437: LazyLock<HashMap<char, u8>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, &c) in CP437_TO_STR.iter().enumerate() {
        map.insert(c, i as u8);
    }
    map
});

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, AsRef)]
#[as_ref(forward)]
pub struct Cp437String<'a>(pub Cow<'a, [u8]>);

impl<'a> Cp437String<'a> {
    #[must_use]
    pub fn from_utf8(s: &str) -> Option<Self> {
        let converted_bytes: Option<Vec<u8>> =
            s.chars().map(|c| CHAR_TO_CP437.get(&c).cloned()).collect();
        Some(Self(Cow::Owned(converted_bytes?)))
    }

    #[must_use]
    pub fn from_utf8_lossy(s: &str) -> Self {
        let replacement = CHAR_TO_CP437[&'?'];
        let converted_bytes = s
            .chars()
            .map(|c| CHAR_TO_CP437.get(&c).cloned().unwrap_or(replacement))
            .collect();
        Self(Cow::Owned(converted_bytes))
    }

    #[must_use]
    pub fn into_inner(self) -> Cow<'a, [u8]> {
        self.0
    }
}

impl<'a> Display for Cp437String<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &c in self.0.iter() {
            f.write_char(CP437_TO_STR[c as usize])?;
        }
        Ok(())
    }
}

impl From<Vec<u8>> for Cp437String<'static> {
    fn from(value: Vec<u8>) -> Self {
        Self(Cow::Owned(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp437_to_string() {
        assert_eq!(
            Cp437String(Cow::Borrowed(&[0, 1, 2, 3])).to_string(),
            "\0☺☻♥"
        );
    }

    #[test]
    fn test_str_to_cp437() {
        assert_eq!(
            Cp437String::from_utf8("☺☻♥♦"),
            Some(Cp437String(Cow::Owned(vec![1, 2, 3, 4])))
        );
        assert_eq!(Cp437String::from_utf8("☺\r"), None);
    }

    #[test]
    fn test_str_to_cp437_lossy() {
        assert_eq!(Cp437String::from_utf8_lossy("☺☻♥♦").as_ref(), &[1, 2, 3, 4]);
        assert_eq!(
            Cp437String::from_utf8_lossy("☺☻♥♦\r").as_ref(),
            &[1, 2, 3, 4, 63] // last char is CP437 "?" symbol
        );
    }
}
