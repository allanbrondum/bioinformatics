use crate::alphabet_model::CharT;
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Formatter, Write};
use generic_array::typenum::U4;

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


/// Wagner-Fischer
pub fn global_alignment_simple<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> GlobalAlignment {
    let mut c = vec![vec![0; y.len() + 1]; x.len() + 1];

    let pen = Penalties { x, y, props };

    for i in 0..=x.len() {
        c[i][0] = i * props.gap_penalty;
    }

    for j in 0..=y.len() {
        c[0][j] = j * props.gap_penalty;
    }

    for i in 1..=x.len() {
        for j in 1..=y.len() {
            c[i][j] = pen
                .diag(&c, i, j)
                .min(pen.up(&c, i, j).min(pen.left(&c, i, j)));
        }
    }

    let mut edits = AString::with_capacity(x.len());
    let mut i = x.len();
    let mut j = y.len();
    while i != 0 || j != 0 {
        if i == 0 {
            edits.push(Edit::Insert);
            j -= 1;
        } else if j == 0 {
            edits.push(Edit::Delete);
            i -= 1;
        } else if pen.diag(&c, i, j) == c[i][j] {
            if x[i - 1] == y[j - 1] {
                edits.push(Edit::Match);
            } else {
                edits.push(Edit::Mismatch);
            }
            i -= 1;
            j -= 1;
        } else if pen.up(&c, i, j) == c[i][j] {
            edits.push(Edit::Delete);
            i -= 1;
        } else if pen.left(&c, i, j) == c[i][j] {
            edits.push(Edit::Insert);
            j -= 1;
        } else {
            unreachable!()
        }
    }

    edits.reverse();

    GlobalAlignment {
        penalty: c[x.len()][y.len()],
        edits,
    }
}

struct Penalties<'s, C> {
    x: &'s AStr<C>,
    y: &'s AStr<C>,
    props: AlignmentProperties,
}

impl<C: PartialEq> Penalties<'_, C> {
    fn up(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i - 1][j] + self.props.gap_penalty
    }

    fn left(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i][j - 1] + self.props.gap_penalty
    }

    fn diag(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i - 1][j - 1]
            + if self.x[i - 1] == self.y[j - 1] {
                0
            } else {
                self.props.mismatch_penalty
            }
    }
}

/// Hirschberg
pub fn global_alignment<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> GlobalAlignment {
    let mut c = vec![vec![0; y.len() + 1]; x.len() + 1];

    let pen = Penalties { x, y, props };

    for i in 0..=x.len() {
        c[i][0] = i * props.gap_penalty;
    }

    for j in 0..=y.len() {
        c[0][j] = j * props.gap_penalty;
    }

    for i in 1..=x.len() {
        for j in 1..=y.len() {
            c[i][j] = pen
                .diag(&c, i, j)
                .min(pen.up(&c, i, j).min(pen.left(&c, i, j)));
        }
    }

    let mut edits = AString::with_capacity(x.len());
    let mut i = x.len();
    let mut j = y.len();
    while i != 0 || j != 0 {
        if i == 0 {
            edits.push(Edit::Insert);
            j -= 1;
        } else if j == 0 {
            edits.push(Edit::Delete);
            i -= 1;
        } else if pen.diag(&c, i, j) == c[i][j] {
            if x[i - 1] == y[j - 1] {
                edits.push(Edit::Match);
            } else {
                edits.push(Edit::Mismatch);
            }
            i -= 1;
            j -= 1;
        } else if pen.up(&c, i, j) == c[i][j] {
            edits.push(Edit::Delete);
            i -= 1;
        } else if pen.left(&c, i, j) == c[i][j] {
            edits.push(Edit::Insert);
            j -= 1;
        } else {
            unreachable!()
        }
    }

    edits.reverse();

    GlobalAlignment {
        penalty: c[x.len()][y.len()],
        edits,
    }
}

#[cfg(test)]
mod test {
    use super::Edit::*;
    use super::*;
    use crate::ascii::{arb_ascii_astring, ascii};
    use crate::string_model::arb_astring;
    use core::str::FromStr;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};

    fn edit(edits: &str) -> AString<Edit> {
        AString::from_str(edits).unwrap()
    }

    #[test]
    fn test_global_alignment_simple() {
        let align = global_alignment_simple(
            ascii("abcdabcd"),
            ascii("abcaadcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);
        assert_eq!(align.edits, edit("===X=X=="));

        let align = global_alignment_simple(
            ascii("abcdbc"),
            ascii("acdabcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 3);
        assert_eq!(align.edits, edit("=D==I==I"));

        let align = global_alignment_simple(
            ascii("bcdabcd"),
            ascii("abcdbbcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);
        assert_eq!(align.edits, edit("I===X==="));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_global_alignment(
            x in arb_ascii_astring(0..20),
            y in arb_ascii_astring(0..20),
            gap_penalty in 1..10usize,
            mismatch_penalty in 1..10usize)
        {
            let props = AlignmentProperties::default().gap_penalty(gap_penalty).mismatch_penalty(mismatch_penalty);
            let expected = global_alignment_simple(&x, &y, props);
            let alignment = global_alignment(&x, &y, props);
            prop_assert_eq!(alignment, expected);
        }
    }
}
