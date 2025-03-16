use generic_array::ArrayLength;
use std::fmt::{Debug, Display};

pub trait CharT: Debug + Copy + Display + Eq + PartialEq + 'static {
    type AlphabetSize: ArrayLength + Debug;

    fn index(self) -> usize;
}
