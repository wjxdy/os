pub type Glyph = [u8; 8];

pub const GLYPH_A: Glyph = [
    0b0001_1000,
    0b0010_0100,
    0b0100_0010,
    0b0100_0010,
    0b0111_1110,
    0b0100_0010,
    0b0100_0010,
    0b0000_0000,
];

const GLYPH_R: Glyph = [
    0b0111_1100,
    0b0100_0010,
    0b0100_0010,
    0b0111_1100,
    0b0101_0000,
    0b0100_1000,
    0b0100_0100,
    0b0000_0000,
];

const GLYPH_U: Glyph = [
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0011_1100,
    0b0000_0000,
];

const GLYPH_S: Glyph = [
    0b0011_1100,
    0b0100_0010,
    0b0100_0000,
    0b0011_1100,
    0b0000_0010,
    0b0100_0010,
    0b0011_1100,
    0b0000_0000,
];

const GLYPH_T: Glyph = [
    0b0111_1110,
    0b0001_1000,
    0b0001_1000,
    0b0001_1000,
    0b0001_1000,
    0b0001_1000,
    0b0001_1000,
    0b0000_0000,
];

const GLYPH_O: Glyph = [
    0b0011_1100,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0100_0010,
    0b0011_1100,
    0b0000_0000,
];

const GLYPH_SPACE: Glyph = [0; 8];

pub const GLYPH_QUESTION: Glyph = [
    0b0011_1100,
    0b0100_0010,
    0b0000_0100,
    0b0000_1000,
    0b0001_0000,
    0b0000_0000,
    0b0001_0000,
    0b0000_0000,
];

pub fn glyph_for(character: char) -> Option<&'static Glyph> {
    match character {
        'A' | 'a' => Some(&GLYPH_A),
        'R' | 'r' => Some(&GLYPH_R),
        'U' | 'u' => Some(&GLYPH_U),
        'S' | 's' => Some(&GLYPH_S),
        'T' | 't' => Some(&GLYPH_T),
        'O' | 'o' => Some(&GLYPH_O),
        ' ' => Some(&GLYPH_SPACE),
        '?' => Some(&GLYPH_QUESTION),
        _ => None,
    }
}
