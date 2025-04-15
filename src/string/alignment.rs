use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Formatter, Write};
use generic_array::typenum::U4;
use std::ops::Range;

mod global_alignment_hirschberg;
mod global_alignment_wagner_fischer;
mod local_alignment_wagner_fischer;

#[derive(Debug, Copy, Clone)]
pub struct AlignmentProperties {
    pub gap_penalty: usize,
    pub mismatch_penalty: usize,
    pub match_score: usize,
}

impl AlignmentProperties {
    pub fn gap_penalty(mut self, gap_penalty: usize) -> Self {
        self.gap_penalty = gap_penalty;
        self
    }

    pub fn mismatch_penalty(mut self, mismatch_penalty: usize) -> Self {
        self.mismatch_penalty = mismatch_penalty;
        self
    }

    pub fn match_score(mut self, match_score: usize) -> Self {
        self.match_score = match_score;
        self
    }
}

impl Default for AlignmentProperties {
    fn default() -> Self {
        Self {
            gap_penalty: 1,
            mismatch_penalty: 1,
            match_score: 1,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Edit {
    Match,
    Mismatch,
    Insert,
    Delete,
}

impl Display for Edit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_char(self.to_char())
    }
}

impl CharT for Edit {
    type AlphabetSize = U4;

    fn index(self) -> usize {
        self as usize
    }

    fn from_char(ch: char) -> Option<Self> {
        match ch {
            '=' => Some(Edit::Match),
            'X' => Some(Edit::Mismatch),
            'I' => Some(Edit::Insert),
            'D' => Some(Edit::Delete),
            _ => None,
        }
    }

    fn to_char(self) -> char {
        match self {
            Edit::Match => '=',
            Edit::Mismatch => 'X',
            Edit::Insert => 'I',
            Edit::Delete => 'D',
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GlobalAlignment {
    pub penalty: usize,
    pub edits: AString<Edit>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalAlignment {
    pub penalty: isize,
    pub range: Range<usize>,
    pub edits: AString<Edit>,
}

fn is_edit<C: PartialEq>(x: &AStr<C>, y: &AStr<C>, edits: &AStr<Edit>) -> bool {
    let mut i = 0;
    let mut j = 0;

    for edit in edits.iter().copied() {
        match edit {
            Edit::Match => {
                if x.get(i) != y.get(j) {
                    return false;
                }
                i += 1;
                j += 1;
            }
            Edit::Mismatch => {
                if x.get(i) == y.get(j) {
                    return false;
                }
                i += 1;
                j += 1;
            }
            Edit::Insert => {
                j += 1;
            }
            Edit::Delete => {
                i += 1;
            }
        }
    }

    i == x.len() && j == y.len()
}
