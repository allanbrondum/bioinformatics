use generic_array::ArrayLength;
use std::fmt::{Debug, Display};

pub trait CharT: Copy + Display + Eq + PartialEq {
    type AlphabetSize: ArrayLength + Debug;

    fn index(self) -> usize;
}
