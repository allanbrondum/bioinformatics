use generic_array::typenum::U2;
use proptest_derive::Arbitrary;
use rosalind::alphabet_model::CharT;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Arbitrary)]
pub enum Char {
    A,
    B,
}

impl CharT for Char {
    type AlphabetSize = U2;

    fn index(self) -> usize {
        match self {
            Char::A => 0,
            Char::B => 1,
        }
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
