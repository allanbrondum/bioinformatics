#[cfg(test)]
pub mod test_util;

use crate::alphabet_model::CharT;
use itertools::Itertools;
use ::proptest::collection::SizeRange;
use ::proptest::strategy::Strategy;
use ::proptest::{arbitrary, collection};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::{Add, Deref, Index, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use proptest::arbitrary::Arbitrary;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AStr<C: CharT>([C]);

impl<C: CharT> Display for AStr<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().format_with("", |ch, f| f(ch)))
    }
}

impl<C: CharT> AStr<C> {
    pub fn from_slice(slice: &[C]) -> &Self {
        unsafe { mem::transmute(slice) }
    }

    pub fn as_slice(&self) -> &[C] {
        &self.0
    }
}

impl<C: CharT> Deref for AStr<C> {
    type Target = [C];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: CharT> AsRef<[C]> for AStr<C> {
    fn as_ref(&self) -> &[C] {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AString<C: CharT>(Vec<C>);

impl<C: CharT> From<Vec<C>> for AString<C> {
    fn from(value: Vec<C>) -> Self {
        Self(value)
    }
}

impl<C: CharT> Add<&AStr<C>> for AString<C> {
    type Output = Self;

    fn add(mut self, rhs: &AStr<C>) -> Self::Output {
        self.0.extend_from_slice(rhs.as_slice());
        self
    }
}

impl<C: CharT> Add<&[C]> for AString<C> {
    type Output = Self;

    fn add(mut self, rhs: &[C]) -> Self::Output {
        self.0.extend_from_slice(rhs);
        self
    }
}

impl<C: CharT> Display for AString<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().format_with("", |ch, f| f(ch)))
    }
}

impl<C: CharT> Deref for AString<C> {
    type Target = AStr<C>;

    fn deref(&self) -> &Self::Target {
        AStr::from_slice(self.0.as_slice())
    }
}

impl<C: CharT> AsRef<AStr<C>> for AString<C> {
    fn as_ref(&self) -> &AStr<C> {
        AStr::from_slice(self.0.as_slice())
    }
}

impl<C: CharT> Borrow<AStr<C>> for AString<C> {
    fn borrow(&self) -> &AStr<C> {
        AStr::from_slice(self.0.as_slice())
    }
}

impl<C: CharT> ToOwned for AStr<C> {
    type Owned = AString<C>;

    fn to_owned(&self) -> Self::Owned {
        AString(self.as_slice().to_vec())
    }
}

impl<C: CharT> Index<usize> for AStr<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<C: CharT> Index<Range<usize>> for AStr<C> {
    type Output = AStr<C>;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        AStr::from_slice(&self.as_slice()[index])
    }
}

impl<C: CharT> Index<RangeFrom<usize>> for AStr<C> {
    type Output = AStr<C>;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        AStr::from_slice(&self.as_slice()[index])
    }
}

impl<C: CharT> Index<RangeTo<usize>> for AStr<C> {
    type Output = AStr<C>;

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        AStr::from_slice(&self.as_slice()[index])
    }
}

impl<C: CharT> Index<RangeToInclusive<usize>> for AStr<C> {
    type Output = AStr<C>;

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        AStr::from_slice(&self.as_slice()[index])
    }
}

impl<C: CharT> Index<RangeInclusive<usize>> for AStr<C> {
    type Output = AStr<C>;

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        AStr::from_slice(&self.as_slice()[index])
    }
}

// prop_compose! {
//     pub fn arb_astring<C: CharT>(size: impl Into<SizeRange>)
//                 (id in arb_order_id(), item in "[a-z]*",
//                  quantity in 1..max_quantity)
//                 -> AString<C> {
//         Order { id, item, quantity }
//     }
// }

#[cfg(test)]
pub fn arb_astring<C: CharT + Arbitrary>(
    size: impl Into<SizeRange>,
) -> impl Strategy<Value = AString<C>> {
    collection::vec(arbitrary::any::<C>(), size).prop_map(|vec| AString::from(vec))
}

#[cfg(test)]
mod test {
    use crate::string_model::AStr;

    #[test]
    fn test_astr_from_slice() {
        use crate::string_model::test_util::Char::*;

        let slice = &[A, B];
        let astr = AStr::from_slice(slice);
        assert_eq!(astr.len(), 2);
        assert_eq!(astr.as_slice(), slice);
    }
}
