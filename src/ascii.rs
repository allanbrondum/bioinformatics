use crate::alphabet_model::CharT;
use crate::string_model::AStr;
use generic_array::typenum::U128;
use std::ascii;

impl CharT for ascii::Char {
    type AlphabetSize = U128;

    fn index(self) -> usize {
        self as usize
    }
}

pub fn ascii(s: &str) -> &AStr<ascii::Char> {
    AStr::from_slice(s.as_ascii().unwrap())
}
