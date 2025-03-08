use itertools::Itertools;

macro_rules! char_identification {
    ($enum_ident:ident; $( $variant_ident:ident ),+ ) => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub enum $enum_ident {
            $(
                $variant_ident,
            )+
        }

        impl $enum_ident {
            pub fn from_char(ch: char) -> Self {
                let mut buffer = [0u8; 4];
                match ch.encode_utf8(&mut buffer) as &str {
                    $(
                        stringify!($variant_ident) => Self::$variant_ident,
                    )+
                    _ => panic!("unrecognized char '{}'", ch),
                }
            }

            pub fn to_char(self) -> char {
                match self {
                    $(
                        Self::$variant_ident => stringify!($variant_ident).chars().next().unwrap(),
                    )+
                }
            }

            pub fn all() -> &'static [Self] {
                &[$( Self::$variant_ident, )+]
            }
        }
    };
}

char_identification!(DnaNt; A, C, G, T);

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

char_identification!(RnaNt; A, C, G, U);

char_identification!(ProteinAa; A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y);

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
    match nts {
        [RnaNt::U, RnaNt::U, RnaNt::U] => Codon::Aa(ProteinAa::F),
        [RnaNt::C, RnaNt::U, RnaNt::U] => Codon::Aa(ProteinAa::L),
        [RnaNt::A, RnaNt::U, RnaNt::U] => Codon::Aa(ProteinAa::I),
        [RnaNt::G, RnaNt::U, RnaNt::U] => Codon::Aa(ProteinAa::V),
        [RnaNt::U, RnaNt::U, RnaNt::C] => Codon::Aa(ProteinAa::F),
        [RnaNt::C, RnaNt::U, RnaNt::C] => Codon::Aa(ProteinAa::L),
        [RnaNt::A, RnaNt::U, RnaNt::C] => Codon::Aa(ProteinAa::I),
        [RnaNt::G, RnaNt::U, RnaNt::C] => Codon::Aa(ProteinAa::V),
        [RnaNt::U, RnaNt::U, RnaNt::A] => Codon::Aa(ProteinAa::L),
        [RnaNt::C, RnaNt::U, RnaNt::A] => Codon::Aa(ProteinAa::L),
        [RnaNt::A, RnaNt::U, RnaNt::A] => Codon::Aa(ProteinAa::I),
        [RnaNt::G, RnaNt::U, RnaNt::A] => Codon::Aa(ProteinAa::V),
        [RnaNt::U, RnaNt::U, RnaNt::G] => Codon::Aa(ProteinAa::L),
        [RnaNt::C, RnaNt::U, RnaNt::G] => Codon::Aa(ProteinAa::L),
        [RnaNt::A, RnaNt::U, RnaNt::G] => Codon::Aa(ProteinAa::M),
        [RnaNt::G, RnaNt::U, RnaNt::G] => Codon::Aa(ProteinAa::V),
        [RnaNt::U, RnaNt::C, RnaNt::U] => Codon::Aa(ProteinAa::S),
        [RnaNt::C, RnaNt::C, RnaNt::U] => Codon::Aa(ProteinAa::P),
        [RnaNt::A, RnaNt::C, RnaNt::U] => Codon::Aa(ProteinAa::T),
        [RnaNt::G, RnaNt::C, RnaNt::U] => Codon::Aa(ProteinAa::A),
        [RnaNt::U, RnaNt::C, RnaNt::C] => Codon::Aa(ProteinAa::S),
        [RnaNt::C, RnaNt::C, RnaNt::C] => Codon::Aa(ProteinAa::P),
        [RnaNt::A, RnaNt::C, RnaNt::C] => Codon::Aa(ProteinAa::T),
        [RnaNt::G, RnaNt::C, RnaNt::C] => Codon::Aa(ProteinAa::A),
        [RnaNt::U, RnaNt::C, RnaNt::A] => Codon::Aa(ProteinAa::S),
        [RnaNt::C, RnaNt::C, RnaNt::A] => Codon::Aa(ProteinAa::P),
        [RnaNt::A, RnaNt::C, RnaNt::A] => Codon::Aa(ProteinAa::T),
        [RnaNt::G, RnaNt::C, RnaNt::A] => Codon::Aa(ProteinAa::A),
        [RnaNt::U, RnaNt::C, RnaNt::G] => Codon::Aa(ProteinAa::S),
        [RnaNt::C, RnaNt::C, RnaNt::G] => Codon::Aa(ProteinAa::P),
        [RnaNt::A, RnaNt::C, RnaNt::G] => Codon::Aa(ProteinAa::T),
        [RnaNt::G, RnaNt::C, RnaNt::G] => Codon::Aa(ProteinAa::A),
        [RnaNt::U, RnaNt::A, RnaNt::U] => Codon::Aa(ProteinAa::Y),
        [RnaNt::C, RnaNt::A, RnaNt::U] => Codon::Aa(ProteinAa::H),
        [RnaNt::A, RnaNt::A, RnaNt::U] => Codon::Aa(ProteinAa::N),
        [RnaNt::G, RnaNt::A, RnaNt::U] => Codon::Aa(ProteinAa::D),
        [RnaNt::U, RnaNt::A, RnaNt::C] => Codon::Aa(ProteinAa::Y),
        [RnaNt::C, RnaNt::A, RnaNt::C] => Codon::Aa(ProteinAa::H),
        [RnaNt::A, RnaNt::A, RnaNt::C] => Codon::Aa(ProteinAa::N),
        [RnaNt::G, RnaNt::A, RnaNt::C] => Codon::Aa(ProteinAa::D),
        [RnaNt::U, RnaNt::A, RnaNt::A] => Codon::Stop,
        [RnaNt::C, RnaNt::A, RnaNt::A] => Codon::Aa(ProteinAa::Q),
        [RnaNt::A, RnaNt::A, RnaNt::A] => Codon::Aa(ProteinAa::K),
        [RnaNt::G, RnaNt::A, RnaNt::A] => Codon::Aa(ProteinAa::E),
        [RnaNt::U, RnaNt::A, RnaNt::G] => Codon::Stop,
        [RnaNt::C, RnaNt::A, RnaNt::G] => Codon::Aa(ProteinAa::Q),
        [RnaNt::A, RnaNt::A, RnaNt::G] => Codon::Aa(ProteinAa::K),
        [RnaNt::G, RnaNt::A, RnaNt::G] => Codon::Aa(ProteinAa::E),
        [RnaNt::U, RnaNt::G, RnaNt::U] => Codon::Aa(ProteinAa::C),
        [RnaNt::C, RnaNt::G, RnaNt::U] => Codon::Aa(ProteinAa::R),
        [RnaNt::A, RnaNt::G, RnaNt::U] => Codon::Aa(ProteinAa::S),
        [RnaNt::G, RnaNt::G, RnaNt::U] => Codon::Aa(ProteinAa::G),
        [RnaNt::U, RnaNt::G, RnaNt::C] => Codon::Aa(ProteinAa::C),
        [RnaNt::C, RnaNt::G, RnaNt::C] => Codon::Aa(ProteinAa::R),
        [RnaNt::A, RnaNt::G, RnaNt::C] => Codon::Aa(ProteinAa::S),
        [RnaNt::G, RnaNt::G, RnaNt::C] => Codon::Aa(ProteinAa::G),
        [RnaNt::U, RnaNt::G, RnaNt::A] => Codon::Stop,
        [RnaNt::C, RnaNt::G, RnaNt::A] => Codon::Aa(ProteinAa::R),
        [RnaNt::A, RnaNt::G, RnaNt::A] => Codon::Aa(ProteinAa::R),
        [RnaNt::G, RnaNt::G, RnaNt::A] => Codon::Aa(ProteinAa::G),
        [RnaNt::U, RnaNt::G, RnaNt::G] => Codon::Aa(ProteinAa::W),
        [RnaNt::C, RnaNt::G, RnaNt::G] => Codon::Aa(ProteinAa::R),
        [RnaNt::A, RnaNt::G, RnaNt::G] => Codon::Aa(ProteinAa::R),
        [RnaNt::G, RnaNt::G, RnaNt::G] => Codon::Aa(ProteinAa::G),
    }
}
