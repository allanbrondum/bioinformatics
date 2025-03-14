use generic_array::ArrayLength;
use generic_array::typenum::{Add1, B1, Unsigned};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Add;

pub trait AlphabetT {
    type N: ArrayLength;
    type Char: CharT;
}

pub trait CharT {
    fn index(self) -> usize;
}

pub struct WithTerminator<A> {
    alphabet: PhantomData<A>,
}

impl<A: AlphabetT> AlphabetT for WithTerminator<A>
where
    A::N: Add<B1>,
    Add1<A::N>: ArrayLength,
{
    type N = Add1<A::N>;
    type Char = CharOrTerminator<A>;
}

pub enum CharOrTerminator<A: AlphabetT> {
    Char(A::Char),
    Terminator,
}

impl<A: AlphabetT> CharT for CharOrTerminator<A> {
    fn index(self) -> usize {
        match self {
            CharOrTerminator::Char(ch) => ch.index(),
            CharOrTerminator::Terminator => A::N::to_usize(),
        }
    }
}
