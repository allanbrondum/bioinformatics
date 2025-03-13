mod superstring;
mod superstring_petgraph;

use regex::Regex;
use std::ascii::Char;
use std::fmt::{Display, Formatter, Write};
use std::ops::Sub;

pub use superstring::sc_supstr;
pub use superstring_petgraph::sc_supstr as sc_supstr_petgraph;

pub fn positions_slice<T: PartialEq>(s: &[T], t: &[T]) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = find_slice(&s[offset..], t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn find_slice<T: PartialEq>(s: &[T], t: &[T]) -> Option<usize> {
    'outer: for i in 0..s.len() {
        for j in 0..t.len() {
            if i + j >= s.len() || s[i + j] != t[j] {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

pub fn positions_str(s: &str, t: &str) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = s[offset..].find(t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn positions_regex(s: &str, regex: &Regex) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(mtch) = regex.find_at(s, offset) {
        offset = mtch.start() + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn lc_substr<'a>(a: &'a [Char], b: &[Char]) -> &'a [Char] {
    let mut substr: &[Char] = &[];
    for i in 0..a.len() {
        if a.len() - i <= substr.len() {
            break;
        }
        for j in 0..b.len() {
            if b.len() - j <= substr.len() {
                break;
            }
            for k in 0..a.len().sub(i).min(b.len().sub(j)) {
                if a.as_bytes()[i + k] != b.as_bytes()[j + k] {
                    break;
                }
                if k + 1 > substr.len() {
                    substr = &a[i..i + k + 1];
                }
            }
        }
    }
    substr
}

pub struct AsciiStr<'a>(pub &'a [Char]);

impl Display for AsciiStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for char in self.0 {
            f.write_char(char.to_char())?;
        }
        Ok(())
    }
}

pub fn overlap_str(a: &str, b: &str) -> usize {
    for i in (1..=a.len().min(b.len())).rev() {
        if a[a.len() - i..] == b[..i] {
            return i;
        }
    }

    0
}

#[cfg(test)]
mod test {
    use crate::string::{lc_substr, overlap_str};

    #[test]
    fn test_lc_substr() {
        assert_eq!(
            lc_substr(
                "abcdefghijk".as_ascii().unwrap(),
                "uioefghijlm".as_ascii().unwrap()
            ),
            "efghij".as_ascii().unwrap()
        );
    }

    #[test]
    fn test_overlap_str() {
        assert_eq!(overlap_str("uioefghabcd", "abcdefghijk",), 4);
        assert_eq!(overlap_str("abcd", "abcd",), 4);
    }
}
