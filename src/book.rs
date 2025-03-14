use crate::{position::Position, r#move::Move};
use std::collections::HashMap;
use rand::distr::{weighted::WeightedIndex, Distribution};

pub struct Book {
    openings: HashMap<u64, (Vec<Move>, Vec<usize>)>,
}

impl Book {
    pub fn new() -> Self {
        let content = String::from_utf8_lossy(include_bytes!("../book"));
        // let content = String::from_utf8_lossy(include_bytes!("../gambit"));
        
        let mut openings = HashMap::new();
        let mut position = Position::new();
        
        let mut moves: Vec<Move> = Vec::new();
        let mut weights: Vec<usize> = Vec::new();

        for line in content.lines().map(|line| line.trim()).filter(|line| !line.is_empty()) {
            let mut words = line.split(" ");

            if line.starts_with("pos") {
                if !moves.is_empty() {
                    openings.insert(position.hash(), (std::mem::replace(&mut moves, Vec::new()), std::mem::replace(&mut weights, Vec::new())));
                }

                position = Position::parse(&words.skip(1).collect::<Vec<_>>());
            } else {
                let m = Move::from_str(words.next().unwrap());
                let mut weight = 1;

                if let Some(word) = words.next() {
                    weight = word.parse().unwrap();
                }

                moves.push(m);
                weights.push(weight);
            }

        }
            
        openings.insert(position.hash(), (moves, weights));

        Self { openings }
    }

    pub fn top(&self, position: &Position) -> Option<Move> {
        let entry = self.openings.get(&position.hash())?;

        Some(entry.0[0])
    }

    pub fn next(&self, position: &Position) -> Option<Move> {
        let entry = self.openings.get(&position.hash())?;
        let dist = WeightedIndex::new(&entry.1).unwrap();
        let mut rng = rand::rng();

        Some(entry.0[dist.sample(&mut rng)])
    }
}
