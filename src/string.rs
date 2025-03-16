mod superstring_rcrefcell;
mod superstring_petgraph;
pub mod suffix_trie_simple;
pub mod suffix_trie_compact;
pub mod suffix_trie_suffix_links;
#[cfg(test)]
mod test_util;

use regex::Regex;
use std::ascii::Char;
use std::fmt::{Display, Formatter, Write};
use std::ops::Sub;

pub use superstring_rcrefcell::scs;
pub use superstring_petgraph::scs as sc_supstr_petgraph;

pub fn indexes_slice<T: PartialEq>(s: &[T], t: &[T]) -> Vec<usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = find_slice(&s[offset..], t) {
        res.push(offset + index);
        offset += index + 1;
    }

    res
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

pub fn indexes_str(s: &str, t: &str) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = s[offset..].find(t) {
        res.push(offset + index);
        offset += index + 1;
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

// todo perf + reimpl using suffix trie

pub fn lcs<'a>(a: &'a [Char], b: &[Char]) -> &'a [Char] {
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
    use std::ascii::Char;
    use crate::string::{find_slice, indexes_slice, lcs, overlap_str};

    fn ascii(s: &str) -> &[Char] {
        s.as_ascii().unwrap()
    }

    #[test]
    fn test_lc_substr() {
        assert_eq!(
            lcs(
                ascii("abcdefghijk"),
                ascii("uioefghijlm")
            ),
            ascii("efghij")
        );
        assert_eq!(
            lcs(
                ascii("abcd"),
                ascii("abcd")
            ),
            ascii("abcd")
        );
    }

    #[test]
    fn test_find_slice() {
        assert_eq!(find_slice(ascii("abcd"), ascii("ijk")), None);
        assert_eq!(find_slice(ascii("abcdijk"), ascii("ijk")), Some(4));
        assert_eq!(find_slice(ascii("abcdijk"), ascii("abc")), Some(0));
        assert_eq!(find_slice(ascii("abcdijk"), ascii("cdi")), Some(2));
    }

    #[test]
    fn test_indexes_slice() {
        assert_eq!(indexes_slice(ascii("abcd"), ascii("ijk")), vec![]);
        assert_eq!(indexes_slice(ascii("ijkabcdijk"), ascii("ijk")), vec![0, 7]);
    }

    #[test]
    fn test_overlap_str() {
        assert_eq!(overlap_str("uioefgh", "ijk",), 0);
        assert_eq!(overlap_str("uioefghabcd", "abcdefghijk",), 4);
        assert_eq!(overlap_str("abcd", "abcd",), 4);
    }
}
