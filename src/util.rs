use std::collections::HashMap;
use std::sync::OnceLock;

use nalgebra::{SVector, Scalar};

use crate::{Result, WorldError};

pub fn parse_vector<T, E, const D: usize>(s: &str) -> Result<nalgebra::SVector<T, D>>
where
    T: std::str::FromStr<Err = E> + Scalar,
    WorldError: From<E>,
{
    let parts = s
        .splitn(D, ',')
        .map(|s| s.parse())
        .collect::<Result<Vec<T>, E>>()?;
    Ok(SVector::<T, D>::from_vec(parts))
}

// Port of Minecraft Pi: Reborn's character handling to Rust

const CP437_CHARACTERS: usize = 256;
/// Used to convert a CP437 character to a Unicode character.
#[rustfmt::skip]
static CP437_TO_STR: [char; CP437_CHARACTERS] = [
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
pub static CHAR_TO_CP437: OnceLock<HashMap<char, u8>> = OnceLock::new();

fn get_char_to_cp437() -> &'static HashMap<char, u8> {
    CHAR_TO_CP437.get_or_init(|| {
        let mut map = HashMap::new();
        for (i, &c) in CP437_TO_STR.iter().enumerate() {
            map.insert(c, i as u8);
        }
        map
    })
}

pub fn cp437_to_string(cp437: &[u8]) -> String {
    let mut s = String::new();
    for &c in cp437 {
        s.push(CP437_TO_STR[c as usize]);
    }
    s
}

pub fn str_to_cp437(s: &str) -> Option<Vec<u8>> {
    let map = get_char_to_cp437();
    s.chars().map(|c| map.get(&c).cloned()).collect()
}

pub fn str_to_cp437_lossy(s: &str) -> Vec<u8> {
    let map = get_char_to_cp437();
    const REPLACEMENT: u8 = b'?'; // utf-8 codepoint same as cp437
    s.chars()
        .map(|c| map.get(&c).cloned().unwrap_or(REPLACEMENT))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp437_to_string() {
        assert_eq!(cp437_to_string(&[0, 1, 2, 3]), "\0☺☻♥");
    }

    #[test]
    fn test_str_to_cp437() {
        assert_eq!(str_to_cp437("☺☻♥♦"), Some(vec![1, 2, 3, 4]));
        assert_eq!(str_to_cp437("☺\r"), None);
    }

    #[test]
    fn test_str_to_cp437_lossy() {
        assert_eq!(str_to_cp437_lossy("☺☻♥♦"), vec![1, 2, 3, 4]);
        assert_eq!(str_to_cp437_lossy("☺☻♥♦\r"), vec![1, 2, 3, 4, b'?']);
    }
}
