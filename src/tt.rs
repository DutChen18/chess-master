use crate::r#move::Move;

pub const TT_SIZE: usize = 1024 * 1024;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Entry {
    hash: u64,
    age: u32,
    r#move: Move,
    depth: u16,
    score: i16,
    bound: Bound,
}

pub struct TranspositionTable {
    table: Vec<Entry>,
}

impl Entry {
    pub fn null() -> Self {
        Self {
            hash: 0,
            age: 0,
            r#move: Move::null(),
            depth: 0,
            score: 0,
            bound: Bound::Exact,
        }
    }

    pub fn new(hash: u64, age: u32, r#move: Move, depth: u16, score: i16, bound: Bound) -> Self {
        Self {
            hash,
            age,
            r#move,
            depth,
            score,
            bound,
        }
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn r#move(&self) -> Move {
        self.r#move
    }

    pub fn depth(&self) -> u16 {
        self.depth
    }

    pub fn score(&self) -> i16 {
        self.score
    }

    pub fn bound(&self) -> Bound {
        self.bound
    }

    pub fn value(&self) -> u32 {
        self.age * 2 + self.depth as u32
    }
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: vec![Entry::null(); TT_SIZE],
        }
    }

    pub fn insert(&mut self, entry: Entry) {
        let table_entry = &mut self.table[entry.hash as usize % TT_SIZE];

        if entry.value() > table_entry.value() {
            *table_entry = entry;
        }
    }

    pub fn probe(&self, hash: u64) -> Option<Entry> {
        let entry = self.table[hash as usize % TT_SIZE];

        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }
}
