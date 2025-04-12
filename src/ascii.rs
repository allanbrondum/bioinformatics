use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use generic_array::typenum::U128;
use proptest::collection::SizeRange;
use proptest::strategy::Strategy;
use proptest::{arbitrary, collection};
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

pub fn arb_ascii_astring(
    size: impl Into<SizeRange>,
) -> impl Strategy<Value = AString<ascii::Char>> {
    collection::vec(
        arbitrary::any::<char>().prop_filter_map("ascii", |ch| ch.as_ascii()),
        size,
    )
    .prop_map(AString::from)
}
