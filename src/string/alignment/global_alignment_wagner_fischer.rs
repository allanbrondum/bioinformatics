use crate::alphabet_model::CharT;
use crate::string::alignment::{AlignmentProperties, Edit, GlobalAlignment};
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Write};

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

#[cfg(test)]
mod test {
    use super::super::Edit::*;
    use super::*;
    use crate::ascii::ascii;
    use crate::string::alignment::{Edit, is_edit};
    use core::str::FromStr;

    fn edit(edits: &str) -> AString<Edit> {
        AString::from_str(edits).unwrap()
    }

    #[test]
    fn test_global_alignment() {
        let x = ascii("abcdabcd");
        let y = ascii("abcaadcd");
        let align = global_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, 2);
        assert_eq!(align.edits, edit("===X=X=="));
        assert!(is_edit(x, y, &align.edits));

        let x = ascii("abcdbc");
        let y = ascii("acdabcd");
        let align = global_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, 3);
        assert_eq!(align.edits, edit("=D==I==I"));
        assert!(is_edit(x, y, &align.edits));

        let x = ascii("bcdabcd");
        let y = ascii("abcdbbcd");
        let align = global_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, 2);
        assert_eq!(align.edits, edit("I===X==="));
        assert!(is_edit(x, y, &align.edits));
    }
}
