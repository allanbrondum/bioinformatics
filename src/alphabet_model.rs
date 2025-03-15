use generic_array::ArrayLength;
use std::fmt::{Debug, Display};

pub trait CharT: Copy + Display {
    type N: ArrayLength + Debug;

    fn index(self) -> usize;
}
