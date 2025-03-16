use crate::alphabet_model::CharT;
use crate::enum_char;
use generic_array::typenum::U2;
use proptest_derive::Arbitrary;
use std::fmt::{Debug, Display, Formatter};

enum_char!(Char; A, B);
