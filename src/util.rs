use regex::Regex;

pub fn chars(s: &str) -> impl DoubleEndedIterator<Item = char> {
    s.trim().chars()
}

pub fn lines(s: &str) -> impl Iterator<Item = &str> {
    s.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
}

pub fn words(s: &str) -> impl Iterator<Item = &str> {
    s.split_whitespace()
}

#[derive(Debug, Clone, Default)]
pub struct FastaEntry {
    pub description: String,
    pub polymer: String,
}

impl FastaEntry {
    fn new(description: String) -> Self {
        Self {
            description,
            polymer: String::default(),
        }
    }
}

pub fn fasta_polymers(s: &str) -> impl Iterator<Item = FastaEntry> {
    let mut res = Vec::new();

    let mut entry = None;
    for line in lines(s) {
        if let Some(descr) = line.strip_prefix(">") {
            if let Some(entry) = entry.take() {
                res.push(entry);
            }
            entry = Some(FastaEntry::new(descr.to_string()));
        } else if let Some(entry) = &mut entry {
            entry.polymer.push_str(line);
        } else {
            panic!("invalid format");
        }
    }

    if let Some(entry) = entry.take() {
        res.push(entry);
    }

    res.into_iter()
}

pub fn positions<T: PartialEq>(s: &[T], t: &[T]) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = find(&s[offset..], t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn find<T: PartialEq>(s: &[T], t: &[T]) -> Option<usize> {
    'outer: for i in 0..s.len() {
        for j in 0..t.len() {
            if i + j >= s.len() || s[i + j] != t[j] {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

pub fn positions_str(s: &str, t: &str) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(index) = s[offset..].find(t) {
        offset += index + 1;
        res.push(offset);
    }

    res.into_iter()
}

pub fn positions_regex(s: &str, regex: &Regex) -> impl Iterator<Item = usize> {
    let mut res = Vec::new();

    let mut offset = 0;
    while let Some(mtch) = regex.find_at(s, offset) {
        offset = mtch.start() + 1;
        res.push(offset);
    }

    res.into_iter()
}
