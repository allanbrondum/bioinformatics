use generic_array::typenum::U2;
use proptest_derive::Arbitrary;
use rosalind::alphabet_model::CharT;
use std::fmt::{Debug, Display, Formatter};
use rosalind::enum_char;

enum_char!(Char; A, B);