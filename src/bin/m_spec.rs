#![feature(btree_cursors)]

use bioinformatics::polymers::{protein_aa_mass, ProteinAa};
use bioinformatics::string_model::AString;
use bioinformatics::util::lines_file;
use itertools::Itertools;
use ordered_float::NotNan;
use std::collections::BTreeMap;
use std::ops::Bound;

fn main() {
    let prefix_spec = lines_file("src/bin/m_spec_data.txt")
        .map(|line| line.parse::<f64>().unwrap())
        .collect_vec();

    let mass_to_aa_map: BTreeMap<_, _> = ProteinAa::all().iter().copied().map(|aa| {
        (NotNan::new(protein_aa_mass(aa)).unwrap(), aa)
    }
    ).collect();

    let protein:AString<_> = prefix_spec.iter().tuple_windows::<(_, _)>().map(|(m1, m2)| {
        let aa_mass = NotNan::new(m2 - m1).unwrap();
        let lower = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).prev();
        let upper = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).next();

        // println!("{} {:?} {:?}", aa_mass, lower, upper);

        match (lower, upper) {
            (Some(lower), None) => lower.1,
            (None, Some(upper)) => upper.1,
            (Some(lower),Some(upper)) => {
                if (aa_mass - lower.0).abs() < (aa_mass - upper.0).abs() {
                    lower.1
                } else {
                    upper.1
                }
            },
            (None, None) => panic!(),

        }
    }).collect();;

    println!("{}", protein);
}

