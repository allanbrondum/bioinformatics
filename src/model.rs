#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum DnaNt {
    A,
    C,
    G,
    T,
}

impl DnaNt {
    pub fn from_char(ch: char) -> DnaNt {
        match ch {
            'A' => DnaNt::A,
            'C' => DnaNt::C,
            'G' => DnaNt::G,
            'T' => DnaNt::T,
            _ => panic!("unrecognized nucleotide '{}'", ch),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            DnaNt::A => 'A',
            DnaNt::C => 'C',
            DnaNt::G => 'G',
            DnaNt::T => 'T',
        }
    }

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


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RnaNt {
    A,
    C,
    G,
    U,
}

impl RnaNt {
    pub fn from_char(ch: char) -> RnaNt {
        match ch {
            'A' => RnaNt::A,
            'C' => RnaNt::C,
            'G' => RnaNt::G,
            'U' => RnaNt::U,
            _ => panic!("unrecognized nucleotide '{}'", ch),
        }
    }

    pub fn to_char(self) -> char {
        match self {
            RnaNt::A => 'A',
            RnaNt::C => 'C',
            RnaNt::G => 'G',
            RnaNt::U => 'U',
        }
    }
}
