use crate::alphabet_model::CharT;
use generic_array::typenum::U2;
use proptest_derive::Arbitrary;
use std::fmt::{Debug, Display, Formatter};
use crate::enum_char;

enum_char!(Char; A, B);
