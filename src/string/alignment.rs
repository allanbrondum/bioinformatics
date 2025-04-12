use crate::alphabet_model::CharT;
use crate::string_model::AStr;

#[derive(Copy, Clone)]
pub struct AlignmentProperties {
    pub gap_penalty: usize,
    pub mismatch_penalty: usize,
}

impl AlignmentProperties {
    pub fn gap_penalty(mut self, gap_penalty: usize) -> Self {
        self.gap_penalty = self.gap_penalty;
        self
    }

    pub fn mismatch_penalty(mut self, mismatch_penalty: usize) -> Self {
        self.mismatch_penalty = self.mismatch_penalty;
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

#[derive(Clone)]
pub struct GlobalAlignment {
    pub penalty: usize,
}

pub fn global_alignment_simple<C: CharT>(
    x: &AStr<C>,
    y: &AStr<C>,
    props: AlignmentProperties,
) -> GlobalAlignment {
    let mut c = vec![vec![0; y.len() + 1]; x.len() + 1];

    for i in 0..=x.len() {
        c[i][0] = i * props.gap_penalty;
    }

    for j in 0..=y.len() {
        c[0][j] = j * props.gap_penalty;
    }

    for i in 1..=x.len() {
        for j in 1..=y.len() {
            let c0 = c[i - 1][j - 1]
                + if x[i - 1] == y[j - 1] {
                    0
                } else {
                    props.mismatch_penalty
                };
            let c1 = c[i - 1][j] + props.gap_penalty;
            let c2 = c[i][j - 1] + props.gap_penalty;
            c[i][j] = c0.min(c1.min(c2));
        }
    }

    GlobalAlignment {
        penalty: c[x.len()][y.len()],
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::ascii::ascii;

    #[test]
    fn test_global_alignment_simple() {
        let align = global_alignment_simple(
            ascii("abcdabcd"),
            ascii("abcaadcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);

        let align = global_alignment_simple(
            ascii("abcdbc"),
            ascii("acdabcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 3);

        let align = global_alignment_simple(
            ascii("bcdabcd"),
            ascii("abcdbbcd"),
            AlignmentProperties::default(),
        );
        assert_eq!(align.penalty, 2);
    }
}
