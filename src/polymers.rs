use crate::enum_char;
use crate::string_model::AString;
use itertools::Itertools;
use ordered_float::NotNan;
use std::collections::{BTreeMap, Bound};
use std::fmt::Formatter;
use std::sync::LazyLock;

enum_char!(DnaNt; A, C, G, T);

impl DnaNt {
    pub fn bonding_complement(self) -> Self {
        match self {
            DnaNt::A => DnaNt::T,
            DnaNt::C => DnaNt::G,
            DnaNt::G => DnaNt::C,
            DnaNt::T => DnaNt::A,
        }
    }

    pub fn transcribe(self) -> RnaNt {
        match self {
            DnaNt::A => RnaNt::A,
            DnaNt::C => RnaNt::C,
            DnaNt::G => RnaNt::G,
            DnaNt::T => RnaNt::U,
        }
    }
}

enum_char!(RnaNt; A, C, G, U);

enum_char!(ProteinAa; A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Codon {
    Start,
    Stop,
    Aa(ProteinAa),
}

pub fn all_codons() -> impl Iterator<Item = [RnaNt; 3]> {
    RnaNt::all()
        .iter()
        .cartesian_product(RnaNt::all().iter())
        .cartesian_product(RnaNt::all().iter())
        .map(|item| [*item.0.0, *item.0.1, *item.1])
}

pub fn to_codon(nts: [RnaNt; 3]) -> Codon {
    use RnaNt::*;

    match nts {
        [U, U, U] => Codon::Aa(ProteinAa::F),
        [C, U, U] => Codon::Aa(ProteinAa::L),
        [A, U, U] => Codon::Aa(ProteinAa::I),
        [G, U, U] => Codon::Aa(ProteinAa::V),
        [U, U, C] => Codon::Aa(ProteinAa::F),
        [C, U, C] => Codon::Aa(ProteinAa::L),
        [A, U, C] => Codon::Aa(ProteinAa::I),
        [G, U, C] => Codon::Aa(ProteinAa::V),
        [U, U, A] => Codon::Aa(ProteinAa::L),
        [C, U, A] => Codon::Aa(ProteinAa::L),
        [A, U, A] => Codon::Aa(ProteinAa::I),
        [G, U, A] => Codon::Aa(ProteinAa::V),
        [U, U, G] => Codon::Aa(ProteinAa::L),
        [C, U, G] => Codon::Aa(ProteinAa::L),
        [A, U, G] => Codon::Aa(ProteinAa::M),
        [G, U, G] => Codon::Aa(ProteinAa::V),
        [U, C, U] => Codon::Aa(ProteinAa::S),
        [C, C, U] => Codon::Aa(ProteinAa::P),
        [A, C, U] => Codon::Aa(ProteinAa::T),
        [G, C, U] => Codon::Aa(ProteinAa::A),
        [U, C, C] => Codon::Aa(ProteinAa::S),
        [C, C, C] => Codon::Aa(ProteinAa::P),
        [A, C, C] => Codon::Aa(ProteinAa::T),
        [G, C, C] => Codon::Aa(ProteinAa::A),
        [U, C, A] => Codon::Aa(ProteinAa::S),
        [C, C, A] => Codon::Aa(ProteinAa::P),
        [A, C, A] => Codon::Aa(ProteinAa::T),
        [G, C, A] => Codon::Aa(ProteinAa::A),
        [U, C, G] => Codon::Aa(ProteinAa::S),
        [C, C, G] => Codon::Aa(ProteinAa::P),
        [A, C, G] => Codon::Aa(ProteinAa::T),
        [G, C, G] => Codon::Aa(ProteinAa::A),
        [U, A, U] => Codon::Aa(ProteinAa::Y),
        [C, A, U] => Codon::Aa(ProteinAa::H),
        [A, A, U] => Codon::Aa(ProteinAa::N),
        [G, A, U] => Codon::Aa(ProteinAa::D),
        [U, A, C] => Codon::Aa(ProteinAa::Y),
        [C, A, C] => Codon::Aa(ProteinAa::H),
        [A, A, C] => Codon::Aa(ProteinAa::N),
        [G, A, C] => Codon::Aa(ProteinAa::D),
        [U, A, A] => Codon::Stop,
        [C, A, A] => Codon::Aa(ProteinAa::Q),
        [A, A, A] => Codon::Aa(ProteinAa::K),
        [G, A, A] => Codon::Aa(ProteinAa::E),
        [U, A, G] => Codon::Stop,
        [C, A, G] => Codon::Aa(ProteinAa::Q),
        [A, A, G] => Codon::Aa(ProteinAa::K),
        [G, A, G] => Codon::Aa(ProteinAa::E),
        [U, G, U] => Codon::Aa(ProteinAa::C),
        [C, G, U] => Codon::Aa(ProteinAa::R),
        [A, G, U] => Codon::Aa(ProteinAa::S),
        [G, G, U] => Codon::Aa(ProteinAa::G),
        [U, G, C] => Codon::Aa(ProteinAa::C),
        [C, G, C] => Codon::Aa(ProteinAa::R),
        [A, G, C] => Codon::Aa(ProteinAa::S),
        [G, G, C] => Codon::Aa(ProteinAa::G),
        [U, G, A] => Codon::Stop,
        [C, G, A] => Codon::Aa(ProteinAa::R),
        [A, G, A] => Codon::Aa(ProteinAa::R),
        [G, G, A] => Codon::Aa(ProteinAa::G),
        [U, G, G] => Codon::Aa(ProteinAa::W),
        [C, G, G] => Codon::Aa(ProteinAa::R),
        [A, G, G] => Codon::Aa(ProteinAa::R),
        [G, G, G] => Codon::Aa(ProteinAa::G),
    }
}

pub fn protein_aa_mass(aa: ProteinAa) -> f64 {
    use ProteinAa::*;

    match aa {
        A => 71.03711,
        C => 103.00919,
        D => 115.02694,
        E => 129.04259,
        F => 147.06841,
        G => 57.02146,
        H => 137.05891,
        I => 113.08406,
        K => 128.09496,
        L => 113.08406,
        M => 131.04049,
        N => 114.04293,
        P => 97.05276,
        Q => 128.05858,
        R => 156.10111,
        S => 87.03203,
        T => 101.04768,
        V => 99.06841,
        W => 186.07931,
        Y => 163.06333,
    }
}

pub fn protein_aa_with_mass(mass: NotNan<f64>) -> Option<ProteinAa> {
    const TOLERANCE: f64 = 0.01;
    let (aa_mass, aa) = protein_aa_with_mass_closest(mass);
    ((aa_mass - mass).abs() <= TOLERANCE).then_some(aa)
}

pub fn protein_aa_with_mass_closest(aa_mass: NotNan<f64>) -> (NotNan<f64>, ProteinAa) {
    static mass_to_aa_map: LazyLock<BTreeMap<NotNan<f64>, ProteinAa>> = LazyLock::new(|| {
        ProteinAa::all()
            .iter()
            .copied()
            .map(|aa| (NotNan::new(protein_aa_mass(aa)).unwrap(), aa))
            .collect()
    });

    // let aa_mass = NotNan::new(aa_mass).unwrap();

    let lower = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).prev();
    let upper = mass_to_aa_map.upper_bound(Bound::Included(&aa_mass)).next();

    match (lower, upper) {
        (Some(lower), None) => (*lower.0, *lower.1),
        (None, Some(upper)) => (*upper.0, *upper.1),
        (Some(lower), Some(upper)) => {
            if (aa_mass - lower.0).abs() < (aa_mass - upper.0).abs() {
                (*lower.0, *lower.1)
            } else {
                (*upper.0, *upper.1)
            }
        }
        (None, None) => panic!(),
    }
}

pub fn translate_rna(rna: impl IntoIterator<Item = RnaNt>) -> AString<ProteinAa> {
    rna.into_iter()
        .chunks(3)
        .into_iter()
        .map(|codon| to_codon(codon.collect_array().unwrap()))
        .take_while(|&codon| codon != Codon::Stop)
        .map(|codon| match codon {
            Codon::Start => todo!(),
            Codon::Stop => unreachable!(),
            Codon::Aa(aa) => aa,
        })
        .collect()
}
