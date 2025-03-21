use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use generic_array::typenum::U128;
use std::ascii;

impl CharT for ascii::Char {
    type AlphabetSize = U128;

    fn index(self) -> usize {
        self as usize
    }

    fn from_char(ch: char) -> Option<Self> {
        ch.as_ascii()
    }

    fn to_char(self) -> char {
        self.to_char()
    }
}

pub fn ascii(s: &str) -> &AStr<ascii::Char> {
    AStr::from_slice(s.as_ascii().unwrap())
}
