use crate::alphabet_model::CharT;
use crate::string::alignment::{AlignmentProperties, Edit, LocalAlignment};
use crate::string_model::{AStr, AString};
use core::fmt::{Display, Write};
use std::iter;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FreeMode {
    Start,
    End,
}

fn alignment_penalty<'a, C: CharT>(
    mut x: &'a AStr<C>,
    mut y: &'a AStr<C>,
    props: AlignmentProperties,
    free_mode: FreeMode,
) -> isize {
    let mut c = vec![vec![0; 2]; y.len() + 1];

    let pen = Penalties {
        x,
        y,
        props,
        free_mode,
    };

    for j in 0..=y.len() {
        c[j][0] = (j * props.gap_penalty) as isize;
    }

    for i in 1..=x.len() {
        for j in 1..=y.len() {
            c[j][i % 2] = pen
                .diag(&c, i, j)
                .min(pen.vert(&c, i, j).min(pen.left(&c, i, j)));
        }
    }

    c[y.len()][x.len() % 2]
}

struct Penalties<'s, C> {
    x: &'s AStr<C>,
    y: &'s AStr<C>,
    props: AlignmentProperties,
    free_mode: FreeMode,
}

impl<C: PartialEq> Penalties<'_, C> {
    fn vert(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[j][(i + 1) % 2] + self.props.gap_penalty as isize
    }

    fn left(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[j - 1][i % 2] + self.props.gap_penalty as isize
    }

    fn diag(&self, c: &Vec<Vec<isize>>, i: usize, j: usize) -> isize {
        c[j - 1][(i + 1) % 2]
            + if self.x[self.x_index(i)] == self.y[self.y_index(j)] {
                -(self.props.match_score as isize)
            } else {
                self.props.mismatch_penalty as isize
            }
    }

    fn x_index(&self, i: usize) -> usize {
        match self.free_mode {
            FreeMode::Start => i - 1,
            FreeMode::End => self.x.len() - i,
        }
    }

    fn y_index(&self, j: usize) -> usize {
        match self.free_mode {
            FreeMode::Start => j - 1,
            FreeMode::End => self.y.len() - j,
        }
    }
}

/// Local alignment of y inside x (deletes in x at start and end are "free")
pub fn local_alignment<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> LocalAlignment {
    assert!(props.mismatch_penalty <= 2 * props.gap_penalty);

    // if x.is_empty() {
    //     return LocalAlignment {
    //         penalty: y.len() * props.gap_penalty,
    //         range: Default::default(),
    //         edits: iter::repeat_n(Edit::Insert, y.len()).collect(),
    //     };
    // } else if y.is_empty() {
    //     return LocalAlignment {
    //         penalty: x.len() * props.gap_penalty,
    //         edits: iter::repeat_n(Edit::Delete, x.len()).collect(),
    //     };
    // } else if x.len() == 1 {
    //     return if let Some(match_idx) = y.iter().copied().position(|ch| ch == x[0]) {
    //         LocalAlignment {
    //             penalty: (y.len() - 1) * props.gap_penalty,
    //             edits: (0..y.len())
    //                 .map(|idx| {
    //                     if idx == match_idx {
    //                         Edit::Match
    //                     } else {
    //                         Edit::Insert
    //                     }
    //                 })
    //                 .collect(),
    //         }
    //     } else {
    //         LocalAlignment {
    //             penalty: (y.len() - 1) * props.gap_penalty + props.mismatch_penalty,
    //             edits: iter::repeat_n(Edit::Insert, y.len() - 1)
    //                 .chain(iter::once(Edit::Mismatch))
    //                 .collect(),
    //         }
    //     };
    // } else if y.len() == 1 {
    //     return if let Some(match_idx) = x.iter().copied().position(|ch| ch == y[0]) {
    //         LocalAlignment {
    //             penalty: (x.len() - 1) * props.gap_penalty,
    //             edits: (0..x.len())
    //                 .map(|idx| {
    //                     if idx == match_idx {
    //                         Edit::Match
    //                     } else {
    //                         Edit::Delete
    //                     }
    //                 })
    //                 .collect(),
    //         }
    //     } else {
    //         LocalAlignment {
    //             penalty: (x.len() - 1) * props.gap_penalty + props.mismatch_penalty,
    //             edits: iter::repeat_n(Edit::Delete, x.len() - 1)
    //                 .chain(iter::once(Edit::Mismatch))
    //                 .collect(),
    //         }
    //     };
    // };

    let mut penalty = isize::MAX;
    let mut split_at = usize::MAX;
    let y_mid = y.len() / 2;
    let y_1 = &y[..y_mid];
    let y_2 = &y[y_mid..];
    for i in 0..=x.len() {
        let x_1 = &x[..i];
        let x_2 = &x[i..];

        let cur_penalty =
            alignment_penalty(x_1, y_1, props, FreeMode::Start) + alignment_penalty(x_2, y_2, props, FreeMode::End);
        if cur_penalty < penalty {
            penalty = cur_penalty;
            split_at = i;
        }
    }

    let x_1 = &x[..split_at];
    let x_2 = &x[split_at..];
    let align_1 = local_alignment(x_1, y_1, props);
    let align_2 = local_alignment(x_2, y_2, props);

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
    let range = align_1.range.start..align_2.range.end;

    LocalAlignment { penalty, range, edits }
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
    fn test_alignment_penalty() {
        let align = alignment_penalty(
            ascii("aaaaaaaa"),
            ascii("aaaaaaaa"),
            AlignmentProperties::default(),
            FreeMode::Start,
        );
        assert_eq!(align, -8);

        let align = alignment_penalty(
            ascii("aaaaaaaa"),
            ascii("aaabbaaa"),
            AlignmentProperties::default(),
            FreeMode::Start,
        );
        assert_eq!(align, -4);

        let align = alignment_penalty(
            ascii("aaaabbbb"),
            ascii("aabbbb"),
            AlignmentProperties::default(),
            FreeMode::Start,
        );
        assert_eq!(align, -6);

        let align = alignment_penalty(
            ascii("aaaabbbb"),
            ascii("aaaabb"),
            AlignmentProperties::default(),
            FreeMode::Start,
        );
        assert_eq!(align, -4);

        let align = alignment_penalty(
            ascii("aaaaaaaa"),
            ascii("aaaaaaaa"),
            AlignmentProperties::default(),
            FreeMode::End,
        );
        assert_eq!(align, -8);

        let align = alignment_penalty(
            ascii("aaaaaaaa"),
            ascii("aaabbaaa"),
            AlignmentProperties::default(),
            FreeMode::End,
        );
        assert_eq!(align, -4);

        let align = alignment_penalty(
            ascii("aaaabbbb"),
            ascii("aabbbb"),
            AlignmentProperties::default(),
            FreeMode::End,
        );
        assert_eq!(align, -4);

        let align = alignment_penalty(
            ascii("aaaabbbb"),
            ascii("aaaabb"),
            AlignmentProperties::default(),
            FreeMode::End,
        );
        assert_eq!(align, -6);
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

    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(2000))]
    //
    //     #[test]
    //     fn prop_test_global_alignment(
    //         x in arb_astring::<Char>(0..20),
    //         y in arb_astring::<Char>(0..20),
    //         gap_penalty in 1..10usize,
    //         mismatch_penalty in 1..10usize)
    //     {
    //         let props = AlignmentProperties::default().gap_penalty(gap_penalty).mismatch_penalty(mismatch_penalty);
    //         prop_assume!(props.mismatch_penalty <= 2 * props.gap_penalty);
    //         let expected = global_alignment_wagner_fischer::global_alignment(&x, &y, props);
    //         let alignment = global_alignment(&x, &y, props);
    //         prop_assert_eq!(alignment.penalty, expected.penalty);
    //         prop_assert!(is_edit(&x, &y, &alignment.edits));
    //     }
    // }
}
