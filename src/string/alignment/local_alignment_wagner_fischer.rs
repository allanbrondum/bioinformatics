use crate::alphabet_model::CharT;
use crate::string::alignment::{AlignmentProperties, Edit, LocalAlignment};
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Write};

/// Local alignment of y inside x (deletes in x at start and end are "free")
pub fn local_alignment<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> LocalAlignment {
    let mut c = vec![vec![0; y.len() + 1]; x.len() + 1];

    let pen = Penalties { x, y, props };

    for j in 0..=y.len() {
        c[0][j] = (j * props.gap_penalty) as isize;
    }

    for i in 1..=x.len() {
        for j in 1..=y.len() {
            c[i][j] = pen
                .diag(&c, i, j)
                .min(pen.up(&c, i, j).min(pen.left(&c, i, j)));
        }
    }

    let mut i_end = 0;
    for i in 0..=x.len() {
        if c[i][y.len()] < c[i_end][y.len()] {
            i_end = i;
        }
    }
    let i_end = i_end;

    let mut edits = AString::with_capacity(x.len());
    let mut i = i_end;
    let mut j = y.len();
    while i != 0 || j != 0 {
        if j == 0 {
            break;
        } else if i == 0 {
            edits.push(Edit::Insert);
            j -= 1;
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

    let penalty = c[i_end][y.len()];
    let range = i..i_end;
    edits.reverse();

    LocalAlignment {
        penalty,
        range,
        edits,
    }
}

struct Penalties<'s, C> {
    x: &'s AStr<C>,
    y: &'s AStr<C>,
    props: AlignmentProperties,
}

impl<C: PartialEq> Penalties<'_, C> {
    fn up(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[i - 1][j] + self.props.gap_penalty as isize
    }

    fn left(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[i][j - 1] + self.props.gap_penalty as isize
    }

    fn diag(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[i - 1][j - 1]
            + if self.x[i - 1] == self.y[j - 1] {
                -(self.props.match_score as isize)
            } else {
                self.props.mismatch_penalty as isize
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
    fn test_local_alignment() {
        let x = ascii("abcdabcd");
        let y = ascii("cdab");
        let align = local_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, -4);
        assert_eq!(align.range, 2..6);
        assert_eq!(align.edits, edit("===="));
        assert!(is_edit(&x[align.range], y, &align.edits));

        let x = ascii("abcd");
        let y = ascii("abcd");
        let align = local_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, -4);
        assert_eq!(align.range, 0..4);
        assert_eq!(align.edits, edit("===="));
        assert!(is_edit(&x[align.range], y, &align.edits));

        let x = ascii("abcdabcd");
        let y = ascii("cdcbc");
        let align = local_alignment(x, y, AlignmentProperties::default());
        assert_eq!(align.penalty, -3);
        assert_eq!(align.range, 2..7);
        assert_eq!(align.edits, edit("==X=="));
        assert!(is_edit(&x[align.range], y, &align.edits));

        let x = ascii("abcdabcd");
        let y = ascii("dcdaba");
        let align = local_alignment(x, y, AlignmentProperties::default().mismatch_penalty(2));
        assert_eq!(align.penalty, -2);
        assert_eq!(align.range, 2..6);
        assert_eq!(align.edits, edit("I====I"));
        assert!(is_edit(&x[align.range], y, &align.edits));

        let x = ascii("abcdabcd");
        let y = ascii("cdbc");
        let align = local_alignment(x, y, AlignmentProperties::default().mismatch_penalty(2));
        assert_eq!(align.penalty, -3);
        assert_eq!(align.range, 2..7);
        assert_eq!(align.edits, edit("==D=="));
        assert!(is_edit(&x[align.range], y, &align.edits));
    }
}
