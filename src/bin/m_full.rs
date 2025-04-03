#![feature(btree_cursors)]

use bioinformatics::polymers::protein_aa_with_mass_closest;
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use itertools::Itertools;
use ordered_float::NotNan;
use std::collections::{BTreeMap, BTreeSet, Bound};

fn main() {
    let masses = lines_file("src/bin/m_full_data.txt")
        .map(|line| line.parse::<NotNan<f64>>().unwrap())
        .collect_vec();
    let (full_mass, prefix_suffix_spec) = masses.split_first().unwrap();

    const TOLERANCE: f64 = 0.01;

    let mut protein = AString::with_capacity(prefix_suffix_spec.len() / 2);

    let mut skip_m = BTreeSet::new();

    let (m_base, prefix_suffix_spec_rest) = prefix_suffix_spec.split_first().unwrap();
    let mut m_base = *m_base;
    skip_m.insert(full_mass - m_base);
    for &m in prefix_suffix_spec_rest {
        if let Some(closest_m) = closest(&skip_m, m) {
            if (m - closest_m).abs() <= TOLERANCE {
                println!("skip");
                continue;
            }
        }

        let potential_aa_m = m - m_base;
        let (aa_m, aa) = protein_aa_with_mass_closest(potential_aa_m);

        println!(
            "{} (m_base) {} (m) {} (aa_m) {} aa {} m_diff",
            m_base,
            m,
            aa_m,
            aa,
            (aa_m - potential_aa_m).abs()
        );

        if (aa_m - potential_aa_m).abs() <= TOLERANCE {
            println!("match");
            protein.push(aa);
            m_base = m;
            skip_m.insert(full_mass - m);
        }
    }

    // let aa_mass = NotNan::new(m2 - m1).unwrap();
    // let lower = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).prev();
    // let upper = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).next();

    println!("{}", protein);
}

pub fn closest(set: &BTreeSet<NotNan<f64>>, val: NotNan<f64>) -> Option<NotNan<f64>> {
    let lower = set.upper_bound(Bound::Included(&val)).prev();
    let upper = set.upper_bound(Bound::Included(&val)).next();

    Some(match (lower, upper) {
        (Some(&lower), None) => lower,
        (None, Some(&upper)) => upper,
        (Some(&lower), Some(&upper)) => {
            if (val - lower).abs() < (val - upper).abs() {
                lower
            } else {
                upper
            }
        }
        (None, None) => return None,
    })
}
