use std::time::Instant;
use std::{env, path::Path};

use crate::options::Options;
use crate::{
    book::Book, gen::*, position::Position, r#move::Move, search, searchlimits::SearchLimits,
    tt::TranspositionTable,
};

pub struct Engine {
    position: Position,
    tt: TranspositionTable,
    book: Book,
    age: u32,
    pub options: Options,
}

impl Engine {
    const NAME: &'static str = "ChessMaster";
    const AUTHOR: &'static str = "csteenvo mjoosten";

    pub fn new() -> Self {
        Self {
            position: Position::new(),
            tt: TranspositionTable::new(),
            book: Book::new(),
            age: 0,
            options: Options::new(),
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

    pub fn book(&self) -> &Book {
        &self.book
    }

    pub fn age(&self) -> u32 {
        self.age
    }

    pub fn run(&mut self) {
        let mut quit = false;
        let mut name = Self::NAME.to_string();

        if let Some(arg) = env::args().nth(0) {
            let path = Path::new(&arg);

            name = format!("{}", path.file_name().unwrap().to_str().unwrap());
        }

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
                        println!("id name {name}");
                        println!("id author {}", Self::AUTHOR);
                        // println!("option name OwnBook value check");
                        println!("uciok");
                    }
                    "debug" => {
                        if let Some(&arg) = words.iter().nth(1) {
                            match arg {
                                "on" => self.options.debug = true,
                                "off" => self.options.debug = false,
                                _ => (),
                            }
                        }
                    }
                    "isready" => println!("readyok"),
                    "setoption" => self.setoption(&words[1..]),
                    "ucinewgame" => self.position = Position::new(),
                    "position" => self.uci_position(&words[1..]),
                    "go" => self.go(&words[1..]),
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

    pub fn go(&mut self, words: &[&str]) {
        if let Some(command) = words.first() {
            if *command == "perft" {
                self.uci_perft(&words[1..]);

                return;
            }
        }

        let limits = SearchLimits::parse(words);

        println!(
            "bestmove {}",
            search::search(self, limits.get_end_time(self.position.turn()), &limits)
        );

        self.age += 1;
    }

    pub fn uci_perft(&mut self, words: &[&str]) {
        let depth = words.first().map(|s| s.parse().unwrap()).unwrap_or(1);

        println!("Nodes searched: {}", self.perft(depth, true));
    }

    pub fn perft(&mut self, depth: usize, root: bool) -> usize {
        if depth == 0 {
            return 1;
        } else if depth == 1 && !root {
            let mut total = 0;

            generate_dyn::<true>(&mut total, &self.position);

            return total;
        }

        let mut moves = MoveVec::new();

        generate_dyn::<true>(&mut moves, &self.position);

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

    pub fn setoption(&mut self, words: &[&str]) {
        let mut it = words.into_iter().map(|&s| s);

        if let Some(name) = Self::optionarg("name", &mut it) {
            if let Some(value) = Self::optionarg("value", &mut it) {
                match name {
                    "OwnBook" => {
                        match value {
                            "true" => self.options.ownbook = true,
                            "false" => self.options.ownbook = false,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn optionarg<'a>(s: &str, it: &mut impl Iterator<Item = &'a str>) -> Option<&'a str> {
        if let Some(what) = it.next() {
            if what == s {
                return it.next();
            }
        }

        None
    }
}
