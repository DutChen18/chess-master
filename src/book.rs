use std::{collections::HashMap, fs};
use crate::{r#move::Move, position::Position};

pub struct Book {
    openings: HashMap<u64, Move>,
}

impl Book {
    const PATH: &'static str = "book";

    pub fn new() -> Self {
        let mut openings = HashMap::new();
        let content = fs::read_to_string(Self::PATH).expect("Missing opening book");
        let mut position = Position::new();

        for line in content.lines().filter(|line| !line.is_empty()) {
            if line.starts_with("fen") { 
                let args: [&str; 6] = line.split_whitespace().skip(1).take(6).collect::<Vec<&str>>().try_into().unwrap();
                
                position = Position::parse(&args);
            } else {
                let m = Move::from_str(line);

                openings.insert(position.hash(), m);                
            }
        }

        Self { openings }
    }

    pub fn next(&self, position: &Position) -> Option<Move> {
        let _ret = self.openings.get(&position.hash()).copied();

        return None;
    }
}
