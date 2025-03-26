use generic_array::ArrayLength;

use generic_array::typenum::{Add1, B1, Unsigned};
use std::fmt::{Debug, Display, Formatter, Write};
use std::ops::Add;

pub trait CharT: Display + Copy + Eq + PartialEq + 'static {
    type AlphabetSize: ArrayLength + Debug;

    fn index(self) -> usize;

    fn from_char(ch: char) -> Option<Self>;

    fn to_char(self) -> char;
}



#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WithSpecial<C, const SPECIAL_CHAR: char, const SPECIAL_FIRST: bool> {
    Char(C),
    Special,
}

impl<C: Display, const SPECIAL_CHAR: char, const SPECIAL_FIRST: bool> Display for WithSpecial<C, SPECIAL_CHAR, SPECIAL_FIRST> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WithSpecial::Char(ch) => Display::fmt(&ch, f),
            WithSpecial::Special => f.write_char(SPECIAL_CHAR),
        }
    }
}

impl<C: CharT, const SPECIAL_CHAR: char, const SPECIAL_FIRST: bool> CharT for WithSpecial<C, SPECIAL_CHAR , SPECIAL_FIRST>
where
    C::AlphabetSize: Unsigned + Add<B1>,
    Add1<C::AlphabetSize>: Debug + ArrayLength,
{
    type AlphabetSize = Add1<C::AlphabetSize>;

    fn index(self) -> usize {
        if SPECIAL_FIRST {
            match self {
                WithSpecial::Char(ch) => ch.index() + 1,
                WithSpecial::Special => 0,
            }
        } else {
            match self {
                WithSpecial::Char(ch) => ch.index(),
                WithSpecial::Special => C::AlphabetSize::to_usize(),
            }
        }
    }

    fn from_char(ch: char) -> Option<Self> {
        if ch == SPECIAL_CHAR {
            Some(Self::Special)
        } else {
            C::from_char(ch).map(Self::Char)
        }
    }

    fn to_char(self) -> char {
        match self {
            WithSpecial::Char(c) => c.to_char(),
            WithSpecial::Special => SPECIAL_CHAR,
        }
    }
}

// macro_rules! replace_expr {
//     ($_t:tt $sub:expr) => {$sub};
// }
//
// macro_rules! count_items {
//     ($($tts:tt),*) => {0usize $(+ replace_expr!($tts 1usize))*};
// }
//
// macro_rules! count_items_typenum2 {
//     ($($tts:tt),*) => { $( replace_expr!($tts generic_array::typenum::operator_aliases::Add1) )*};
// }

// todo  macro
// macro_rules! count_items_typenum {
//     ($tt:tt, $($tts:tt),*) => { generic_array::typenum::operator_aliases::Add1<count_items_typenum!( $($tts),* )>};
//     ($tt:tt) => {generic_array::typenum::U1};
// }

#[macro_export]
macro_rules! enum_char {
    (@count_items_typenum, $tt:tt, $($tts:tt),*) => { generic_array::typenum::operator_aliases::Add1<enum_char!( @count_items_typenum, $($tts),* )>};
    (@count_items_typenum, $tt:tt) => {generic_array::typenum::U1};

    ($enum_ident:ident; $( $variant_ident:ident ),+ ) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, proptest_derive::Arbitrary)]
        pub enum $enum_ident {
            $(
                $variant_ident,
            )+
        }

        impl $crate::alphabet_model::CharT for $enum_ident {
            type AlphabetSize = enum_char!(@count_items_typenum, $( $variant_ident ),+ );

            fn index(self) -> usize {
                self as usize
            }

            fn from_char(ch: char) -> Option<Self> {
                let mut buffer = [0u8; 4];
                match ch.encode_utf8(&mut buffer) as &str {
                    $(
                        stringify!($variant_ident) => Some(Self::$variant_ident),
                    )+
                    _ => None,
                }
            }

            fn to_char(self) -> char {
                match self {
                    $(
                        Self::$variant_ident => stringify!($variant_ident).chars().next().unwrap(),
                    )+
                }
            }
       }

       impl std::fmt::Display for $enum_ident {
           fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
               write!(f, "{}", $crate::alphabet_model::CharT::to_char(*self))
           }
       }

       impl $enum_ident {
            pub fn all() -> &'static [Self] {
                &[$( Self::$variant_ident, )+]
            }
        }
    };
}
