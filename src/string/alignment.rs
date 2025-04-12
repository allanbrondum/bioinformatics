use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Formatter, Write};
use generic_array::typenum::U4;

mod global_alignment_simple;
mod global_alignment_hirschberg;

#[derive(Copy, Clone)]
pub struct AlignmentProperties {
    pub gap_penalty: usize,
    pub mismatch_penalty: usize,
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
}

impl Default for AlignmentProperties {
    fn default() -> Self {
        Self {
            gap_penalty: 1,
            mismatch_penalty: 1,
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


