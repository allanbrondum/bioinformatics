use crate::alphabet_model::{ AlphabetT};
use generic_array::typenum::{Add1, B1};
use generic_array::{ArrayLength, GenericArray};
use std::ops::Add;

// struct Node<A: AlphabetExtendableWithTerminatortT>
// where
//     A::N: Add<B1>,
//     Add1<A::N>: ArrayLength,
// {
//     char: <WithTerminator<A> as AlphabetT>::Char,
//     children: GenericArray<Option<Box<Node<A>>>, <WithTerminator<A> as AlphabetT>::N>,
// }

// pub fn build<A: AlphabetT>(&[Char]) where
//     A::N: Add<B1>,
//     Add1<A::N>: ArrayLength,{
//
// }

#[cfg(test)]
mod test {
    use crate::alphabet_model::AlphabetT;
    use generic_array::typenum::{U2, UInt};

    enum Char {
        A,
        B,
    }

    struct Alphabet;

    // impl AlphabetT for Alphabet {
    //     type N = U2;
    //     type IdxType = u8;
    //     type Char = Char;
    // }
}
