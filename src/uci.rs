use std::io::Write;

use crate::position::Position;
use crate::searchlimits::SearchLimits;
use crate::r#move::Move;
use crate::engine::Engine;

pub struct Uci {
    position: Position,
}

impl Uci {
    const NAME: &'static str = "ChessMaster";
    const AUTHOR: &'static str = "csteenvo mjoosten";

    pub fn new() -> Self {
        Self {
            position: Position::new(),
        }
    }

    pub fn run(&mut self) {
        let mut quit = false;

        while !quit {
            let mut buffer = String::new();

            if let Ok(bytes) = std::io::stdin().read_line(&mut buffer) {
                if bytes == 0 {
                    break;
                }
            }

            eprint!("> {buffer}");

            let words: Vec<String> = buffer.split_whitespace().map(String::from).collect();

            if let Some(command) = words.first() { 
                match command.as_str() {
                    "uci" => {
                        println!("id name {}", Self::NAME);
                        println!("id author {}", Self::AUTHOR);
                        // println!("option name OwnBook");
                        println!("uciok");
                    }
                    "isready" => println!("readyok"),
                    "setoption" => todo!(),
                    "ucinewgame" => self.position = Position::new(),
                    "position" => self.position(&words[1..]),
                    "go" => self.go(&words[1..]),
                    "quit" => quit = true,
                    "perft" => self.perft(&words[1..]),
                    _ => (),
                }
            }

            std::io::stdout().flush().unwrap();
        }
    }

    pub fn position(&mut self, words: &[String]) {
        if let Some(pos) = words.first() {
            let fen: [&str; 6] = match pos.as_str() {
                "fen" => words.iter().skip(1).map(|s| s.as_str()).take_while(|&s| s != "moves").collect::<Vec<&str>>().try_into().unwrap(),
                "startpos" => Position::STARTPOS.split_whitespace().collect::<Vec<&str>>().try_into().unwrap(),
                _ => panic!(),
            };

            self.position = Position::parse(&fen);

            for m in words.iter().skip_while(|&s| s != "moves").skip(1) {
                self.position.make(Move::from_str(m));
            }
        }
    }

    pub fn go(&mut self, words: &[String]) {
        if let Some(command) = words.first() {
            if command == "perft" {
                return self.perft(words);
            }
        }

        let limits = SearchLimits::parse(words);

        println!("bestmove {}", Engine::search(&self.position, limits));
    }

    pub fn perft(&mut self, words: &[String]) {
        if let Some(command) = words.first() {
            if command == "perft" {
                let depth = if let Some(s) = words.iter().nth(1) { s.parse().unwrap() } else { 1 };
                
                println!("Nodes searched: {}", Engine::perft(&mut self.position, depth, true));
            }
        }
    }
}

