use crate::alphabet_model::CharT;
use crate::string::alignment::{AlignmentProperties, Edit, GlobalAlignment};
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Write};
use std::ops::Deref;
use std::{iter, mem};

fn global_alignment_penalty<'a, C: CharT>(
    mut x: &'a AStr<C>,
    mut y: &'a AStr<C>,
    props: AlignmentProperties,
) -> usize {
    if x.len() > y.len() {
        mem::swap(&mut x, &mut y);
    }

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
    assert!(props.mismatch_penalty <= 2 * props.gap_penalty);

    if x.is_empty() {
        return GlobalAlignment {
            penalty: y.len() * props.gap_penalty,
            edits: iter::repeat_n(Edit::Insert, y.len()).collect(),
        };
    } else if y.is_empty() {
        return GlobalAlignment {
            penalty: x.len() * props.gap_penalty,
            edits: iter::repeat_n(Edit::Delete, x.len()).collect(),
        };
    } else if x.len() == 1 {
        return if let Some(match_idx) = y.iter().copied().position(|ch| ch == x[0]) {
            GlobalAlignment {
                penalty: (y.len() - 1) * props.gap_penalty,
                edits: (0..y.len())
                    .map(|idx| {
                        if idx == match_idx {
                            Edit::Match
                        } else {
                            Edit::Insert
                        }
                    })
                    .collect(),
            }
        } else {
            GlobalAlignment {
                penalty: (y.len() - 1) * props.gap_penalty + props.mismatch_penalty,
                edits: iter::repeat_n(Edit::Insert, y.len() - 1)
                    .chain(iter::once(Edit::Mismatch))
                    .collect(),
            }
        };
    } else if y.len() == 1 {
        return if let Some(match_idx) = x.iter().copied().position(|ch| ch == y[0]) {
            GlobalAlignment {
                penalty: (x.len() - 1) * props.gap_penalty,
                edits: (0..x.len())
                    .map(|idx| {
                        if idx == match_idx {
                            Edit::Match
                        } else {
                            Edit::Delete
                        }
                    })
                    .collect(),
            }
        } else {
            GlobalAlignment {
                penalty: (x.len() - 1) * props.gap_penalty + props.mismatch_penalty,
                edits: iter::repeat_n(Edit::Delete, x.len() - 1)
                    .chain(iter::once(Edit::Mismatch))
                    .collect(),
            }
        };
    };

    let mut penalty = usize::MAX;
    let mut split_at = usize::MAX;
    let y_mid = y.len() / 2;
    let y_1 = &y[..y_mid];
    let y_2 = &y[y_mid..];
    for i in 0..=x.len() {
        let x_1 = &x[..i];
        let x_2 = &x[i..];

        let cur_penalty =
            global_alignment_penalty(x_1, y_1, props) + global_alignment_penalty(x_2, y_2, props);
        if cur_penalty < penalty {
            penalty = cur_penalty;
            split_at = i;
        }
    }

    let x_1 = &x[..split_at];
    let x_2 = &x[split_at..];
    let align_1 = global_alignment(x_1, y_1, props);
    let align_2 = global_alignment(x_2, y_2, props);

    debug_assert_eq!(
        align_1.penalty + align_2.penalty,
        penalty,
        "x_1: {}, y_1: {}, penalty_1: {}, x_2: {}, y_2: {}, penalty_2: {}, props: {:?}",
        x_1,
        y_1,
        align_1.penalty,
        x_2,
        y_2,
        align_2.penalty,
        props,
    );

    let edits = align_1.edits + align_2.edits.as_str();

    GlobalAlignment { penalty, edits }
}

#[cfg(test)]
mod test {
    use super::super::Edit::*;
    use super::*;
    use crate::ascii::{arb_ascii_astring, ascii};
    use crate::string::alignment::{Edit, global_alignment_wagner_fischer, is_edit};
    use crate::string_model::arb_astring;
    use crate::string_model::test_util::Char;
    use core::str::FromStr;
    use proptest::prelude::ProptestConfig;
    use proptest::{prop_assert, prop_assert_eq, prop_assume, proptest};

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

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(2000))]

        #[test]
        fn prop_test_global_alignment(
            x in arb_astring::<Char>(0..20),
            y in arb_astring::<Char>(0..20),
            gap_penalty in 1..10usize,
            mismatch_penalty in 1..10usize)
        {
            let props = AlignmentProperties::default().gap_penalty(gap_penalty).mismatch_penalty(mismatch_penalty);
            prop_assume!(props.mismatch_penalty <= 2 * props.gap_penalty);
            let expected = global_alignment_wagner_fischer::global_alignment(&x, &y, props);
            let alignment = global_alignment(&x, &y, props);
            prop_assert_eq!(alignment.penalty, expected.penalty);
            prop_assert!(is_edit(&x, &y, &alignment.edits));
        }
    }
}
