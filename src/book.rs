use crate::{position::Position, r#move::Move};
use std::collections::HashMap;
use rand::distr::{weighted::WeightedIndex, Distribution};

pub struct Book {
    openings: HashMap<u64, (Vec<Move>, Vec<usize>)>,
}

impl Book {
    pub fn new() -> Self {
        let content = String::from_utf8_lossy(include_bytes!("../book"));
        let mut openings = HashMap::new();
        let mut position = Position::new();

        for line in content.lines().map(|line| line.trim()).filter(|line| !line.is_empty()) {
            let mut moves: Vec<Move> = Vec::new();
            let mut weights: Vec<usize> = Vec::new();
            let mut words = line.split(" ");

            if line.starts_with("pos") {
                position = Position::parse(&words.skip(1).collect::<Vec<_>>());
            } else {
                let m = Move::from_str(words.next().unwrap());
                let weight = words.next().unwrap().parse().unwrap();

                moves.push(m);
                weights.push(weight);
            }

            openings.insert(position.hash(), (moves, weights));
        }

        assert!(openings.len() == 22234);

        Self { openings }
    }

    pub fn next(&self, position: &Position) -> Option<Move> {
        let entry = self.openings.get(&position.hash())?;
        let dist = WeightedIndex::new(&entry.1).unwrap();
        let mut rng = rand::rng();

        Some(entry.0[dist.sample(&mut rng)])
    }
}
