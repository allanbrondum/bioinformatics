pub mod suffix_trie_compact;
pub mod suffix_trie_simple;
pub mod suffix_trie_suffix_links;
mod superstring_petgraph;
mod superstring_rcrefcell;

use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use regex::Regex;
use std::ops::Sub;
pub use superstring_petgraph::scs as sc_supstr_petgraph;
pub use superstring_rcrefcell::scs;

pub fn indexes<C: CharT>(s: &AStr<C>, t: &AStr<C>) -> Vec<usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = find(AStr::from_slice(&s[offset..]), t) {
        res.push(offset + index);
        offset += index + 1;
    }

    res
}

pub fn find<C: CharT>(s: &AStr<C>, t: &AStr<C>) -> Option<usize> {
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

pub fn replace_all<C: CharT>(s: &AStr<C>, t: &AStr<C>, u: &AStr<C>) -> AString<C> {
    let mut res = AString::default();

    let mut i = 0;
    while let Some(idx) = find(&s[i..], t) {
        res.push_str(&s[i..i + idx]);
        res.push_str(u);
        i += idx + t.len();
    }
    res.push_str(&s[i..]);

    res
}

pub fn overlap<C: CharT>(a: &AStr<C>, b: &AStr<C>) -> usize {
    for i in (1..=a.len().min(b.len())).rev() {
        if a[a.len() - i..] == b[..i] {
            return i;
        }
    }

    0
}

pub fn lcp<'a, C: CharT>(a: &'a AStr<C>, b: &AStr<C>) -> &'a AStr<C> {
    let mut i = 0;
    while i < a.len() && i < b.len() && a[i] == b[i] {
        i += 1;
    }
    &a[0..i]
}

pub fn lcs<'a, C: CharT>(a: &'a AStr<C>, b: &AStr<C>) -> &'a AStr<C> {
    let mut substr: &[C] = &[];
    for i in 0..a.len() {
        if a.len() - i <= substr.len() {
            break;
        }
        for j in 0..b.len() {
            if b.len() - j <= substr.len() {
                break;
            }
            for k in 0..a.len().sub(i).min(b.len().sub(j)) {
                if a[i + k] != b[j + k] {
                    break;
                }
                if k + 1 > substr.len() {
                    substr = &a[i..i + k + 1];
                }
            }
        }
    }
    AStr::from_slice(substr)
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

pub fn indexes_regex(s: &str, regex: &Regex) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(mtch) = regex.find_at(s, offset) {
        offset = mtch.start() + 1;
        res.push(offset);
    }

    res.into_iter()
}

#[cfg(test)]
mod test {
    use crate::ascii::ascii;
    use crate::string::{find, indexes, lcp, lcs, overlap, replace_all};

    #[test]
    fn test_lcs() {
        assert_eq!(
            lcs(ascii("abcdefghijk"), ascii("uioefghijlm")),
            ascii("efghij")
        );
        assert_eq!(lcs(ascii("abcd"), ascii("abcd")), ascii("abcd"));
    }

    #[test]
    fn test_lcp() {
        assert_eq!(lcp(ascii("abcdefghijk"), ascii("abcd")), ascii("abcd"));
        assert_eq!(lcp(ascii("abcd"), ascii("abcdefghijk")), ascii("abcd"));
        assert_eq!(lcp(ascii("abcd"), ascii("defghijk")), ascii(""));
    }

    #[test]
    fn test_find() {
        assert_eq!(find(ascii("abcd"), ascii("ijk")), None);
        assert_eq!(find(ascii("abcdijk"), ascii("ijk")), Some(4));
        assert_eq!(find(ascii("abcdijk"), ascii("abc")), Some(0));
        assert_eq!(find(ascii("abcdijk"), ascii("cdi")), Some(2));
    }

    #[test]
    fn test_replace_all() {
        assert_eq!(
            replace_all(ascii("abcd"), ascii("ijk"), ascii("lmn")),
            ascii("abcd").to_owned()
        );
        assert_eq!(
            replace_all(ascii("abcdabcd"), ascii("bc"), ascii("lmn")),
            ascii("almndalmnd").to_owned()
        );
    }

    #[test]
    fn test_indexes() {
        assert_eq!(indexes(ascii("abcd"), ascii("ijk")), vec![]);
        assert_eq!(indexes(ascii("ijkabcdijk"), ascii("ijk")), vec![0, 7]);
    }

    #[test]
    fn test_overlap() {
        assert_eq!(overlap(ascii("uioefgh"), ascii("ijk"),), 0);
        assert_eq!(overlap(ascii("uioefghabcd"), ascii("abcdefghijk"),), 4);
        assert_eq!(overlap(ascii("abcd"), ascii("abcd"),), 4);
    }
}
