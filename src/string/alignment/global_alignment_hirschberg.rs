use crate::alphabet_model::CharT;
use crate::string::alignment::{AlignmentProperties, GlobalAlignment};
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Write};

fn global_alignment_penalty<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> usize {
    let mut c = vec![vec![0; 2]; x.len() + 1];

    let pen = Penalties { x, y, props };

    for i in 0..=x.len() {
        c[i][0] = i * props.gap_penalty;
    }

    for j in 1..=y.len() {
        c[0][j % 2] = j * props.gap_penalty;
        for i in 1..=x.len() {
            c[i][j % 2] = pen
                .diag(&c, i, j)
                .min(pen.up(&c, i, j).min(pen.left(&c, i, j)));
        }
    }

    c[x.len()][y.len() % 2]
}

struct Penalties<'s, C> {
    x: &'s AStr<C>,
    y: &'s AStr<C>,
    props: AlignmentProperties,
}

impl<C: PartialEq> Penalties<'_, C> {
    fn up(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i - 1][j % 2] + self.props.gap_penalty
    }

    fn left(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i][(j - 1) % 2] + self.props.gap_penalty
    }

    fn diag(&self, c: &Vec<Vec<usize>>, i: usize, j: usize) -> usize {
        c[i - 1][(j - 1) % 2]
            + if self.x[i - 1] == self.y[j - 1] {
                0
            } else {
                self.props.mismatch_penalty
            }
    }
}

pub fn global_alignment<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> GlobalAlignment {
    let mut edits = AString::with_capacity(x.len());

    let mut penalty = usize::MAX;
    let y_mid = y.len() / 2;
    let y_1 = &y[..y_mid];
    let y_2 = &y[y_mid..];
    for i in 0..=x.len() {
        let x_1 = &x[..i];
        let x_2 = &x[i..];

        let cur_penalty =
            global_alignment_penalty(x_1, y_1, props) + global_alignment_penalty(x_2, y_2, props);
        penalty = penalty.min(cur_penalty);
    }

    GlobalAlignment { penalty, edits }
}

#[cfg(test)]
mod test {
    use super::super::Edit::*;
    use super::*;
    use crate::ascii::{arb_ascii_astring, ascii};
    use crate::string::alignment::{Edit, global_alignment_wagner_fischer};
    use crate::string_model::arb_astring;
    use core::str::FromStr;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert_eq, proptest};

    fn edit(edits: &str) -> AString<Edit> {
        AString::from_str(edits).unwrap()
    }

    #[test]
    fn test_global_alignment_penalty() {
        let align = global_alignment_penalty(
            ascii("abcdabcd"),
            ascii("abcaadcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align, 2);

        let align = global_alignment_penalty(
            ascii("abcdbc"),
            ascii("acdabcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align, 3);

        let align = global_alignment_penalty(
            ascii("bcdabcd"),
            ascii("abcdbbcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align, 2);
    }

    #[test]
    fn test_global_alignment() {
        let align = global_alignment(
            ascii("abcdabcd"),
            ascii("abcaadcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);
        // assert_eq!(align.edits, edit("===X=X=="));

        let align = global_alignment(
            ascii("abcdbc"),
            ascii("acdabcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 3);
        // assert_eq!(align.edits, edit("=D==I==I"));

        let align = global_alignment(
            ascii("bcdabcd"),
            ascii("abcdbbcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);
        // assert_eq!(align.edits, edit("I===X==="));
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
            let expected = global_alignment_wagner_fischer::global_alignment(&x, &y, props);
            let alignment = global_alignment(&x, &y, props);
            prop_assert_eq!(alignment, expected);
        }
    }
}
