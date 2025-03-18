#[cfg(test)]
pub mod test_util;

use crate::alphabet_model::CharT;
use ::proptest::collection::SizeRange;
use ::proptest::strategy::Strategy;
use ::proptest::{arbitrary, collection};
use itertools::Itertools;
use proptest::arbitrary::Arbitrary;
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::{Add, Deref, Index, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use std::str::FromStr;
use smallvec::{SmallVec, ToSmallVec};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AStr<C: CharT>([C]);

impl<'a, C: CharT> IntoIterator for &'a AStr<C> {
    type Item = &'a C;
    type IntoIter = <&'a [C] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &C> {
        self.into_iter()
    }

    pub fn empty() -> &'static AStr<C> {
        AStr::from_slice(&[])
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


type AStringVec<C> = SmallVec<[C;5]>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AString<C: CharT>(AStringVec<C>);

impl<C: CharT> FromIterator<C> for AString<C> {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self(AStringVec::from_iter(iter))
    }
}

impl<C: CharT> Default for AString<C> {
    fn default() -> Self {
        Self(AStringVec::default())
    }
}

impl<C: CharT> IntoIterator for AString<C> {
    type Item = C;
    type IntoIter = <AStringVec<C> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, C: CharT> IntoIterator for &'a AString<C> {
    type Item = &'a C;
    type IntoIter = <&'a Vec<C> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<C: CharT> AString<C> {
    pub fn extend_from_slice(&mut self, slice: &[C]) {
        self.0.extend_from_slice(slice);
    }

    pub fn push_str(&mut self, str: &AStr<C>) {
        self.0.extend_from_slice(str.as_slice());
    }

    pub fn as_str(&self) -> &AStr<C> {
        AStr::from_slice(self.0.as_slice())
    }
}

impl<C: CharT> FromStr for AString<C> {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = s
            .chars()
            .map(|ch| C::from_char(ch).ok_or_else(|| format!("invalid char {}", ch)))
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self::from(vec))
    }
}

impl<C: CharT> From<Vec<C>> for AString<C> {
    fn from(value: Vec<C>) -> Self {
        Self(value.into())
    }
}

impl<C: CharT> From<AStringVec<C>> for AString<C> {
    fn from(value: AStringVec<C>) -> Self {
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
        AString(self.as_slice().to_smallvec())
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

#[cfg(test)]
proptest::prop_compose! {
    pub fn arb_astring2(size: impl Into<SizeRange>)
                (s in collection::vec(arbitrary::any::<crate::string_model::test_util::Char>(), size))
                -> AString<crate::string_model::test_util::Char> {
        AString::from(s)
    }
}

pub fn arb_astring<C: CharT + Arbitrary>(
    size: impl Into<SizeRange>,
) -> impl Strategy<Value = AString<C>> {
    collection::vec(arbitrary::any::<C>(), size).prop_map(AString::from)
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
