use std::time::Instant;

use crate::{
    gen::*, position::Position, r#move::Move, search, searchlimits::SearchLimits,
    tt::TranspositionTable,
};

pub struct Engine {
    position: Position,
    tt: TranspositionTable,
    age: u32,
}

impl Engine {
    const NAME: &'static str = "ChessMaster";
    const AUTHOR: &'static str = "csteenvo mjoosten";

    pub fn new() -> Self {
        Self {
            position: Position::new(),
            tt: TranspositionTable::new(),
            age: 0,
        }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn position_mut(&mut self) -> &mut Position {
        &mut self.position
    }

    pub fn tt(&self) -> &TranspositionTable {
        &self.tt
    }

    pub fn tt_mut(&mut self) -> &mut TranspositionTable {
        &mut self.tt
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn uci_run(&mut self) {
        let mut quit = false;

        while !quit {
            let mut buffer = String::new();

            if let Ok(bytes) = std::io::stdin().read_line(&mut buffer) {
                if bytes == 0 {
                    break;
                }
            }

            let words: Vec<_> = buffer.split_whitespace().collect();

            if let Some(command) = words.first() {
                match *command {
                    "uci" => {
                        println!("id name {}", Self::NAME);
                        println!("id author {}", Self::AUTHOR);
                        // println!("option name OwnBook");
                        println!("uciok");
                    }
                    "isready" => println!("readyok"),
                    "setoption" => todo!(),
                    "ucinewgame" => self.position = Position::new(),
                    "position" => self.uci_position(&words[1..]),
                    "go" => self.uci_go(&words[1..]),
                    "quit" => quit = true,
                    "perft" => {
                        let start = Instant::now();
                        self.uci_perft(&words[1..]);
                        eprintln!("took {:?}", start.elapsed());
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn uci_position(&mut self, words: &[&str]) {
        if let Some(pos) = words.first() {
            let fen: [&str; 6] = match *pos {
                "fen" => words
                    .iter()
                    .skip(1)
                    .copied()
                    .take_while(|&s| s != "moves")
                    .collect::<Vec<&str>>()
                    .try_into()
                    .unwrap(),
                "startpos" => Position::STARTPOS
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .try_into()
                    .unwrap(),
                _ => panic!(),
            };

            self.position = Position::parse(&fen);

            for m in words.iter().skip_while(|&s| *s != "moves").skip(1) {
                self.position.make(Move::from_str(m));
            }
        }
    }

    pub fn uci_go(&mut self, words: &[&str]) {
        if let Some(command) = words.first() {
            if *command == "perft" {
                self.uci_perft(&words[1..]);

                return;
            }
        }

        let limits = SearchLimits::parse(words);

        println!(
            "bestmove {}",
            search::search(self, limits.get_end_time(self.position.turn()))
        );

        self.age += 1;
    }

    pub fn uci_perft(&mut self, words: &[&str]) {
        let depth = words.iter().nth(1).map(|s| s.parse().unwrap()).unwrap_or(1);

        println!("Nodes searched: {}", self.perft(depth, true));
    }

    pub fn perft(&mut self, depth: usize, root: bool) -> usize {
        if depth == 0 {
            return 1;
        } else if depth == 1 && !root {
            let mut total = 0;

            generate_dyn(&mut total, &self.position);

            return total;
        }

        let mut moves = MoveVec::new();

        generate_dyn(&mut moves, &self.position);

        let mut total = 0;

        for m in moves.moves() {
            let undo = self.position.make(*m);
            let count = self.perft(depth - 1, false);

            if root {
                println!("{m}: {count}");
            }

            total += count;

            self.position.unmake(undo);
        }

        total
    }
}
