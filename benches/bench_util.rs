use generic_array::typenum::U2;
use proptest_derive::Arbitrary;
use rosalind::alphabet_model::CharT;
use rosalind::enum_char;
use std::fmt::{Debug, Display, Formatter};

enum_char!(Char; A, B);
