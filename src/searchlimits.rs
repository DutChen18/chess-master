use std::time::Duration;
use crate::types::Color;

pub struct SearchLimits {
    time: [Duration; 2],
    inc: [Duration; 2],
    
    /*
    searchmoves: Vec<Move>,
    ponder: bool,
    movestogo: usize,
    depth: usize,
    nodes: usize,
    mate: usize,
    movetime: usize
    infinite: bool,
    */
}

impl SearchLimits {
    pub fn parse<'a>(words: &[String]) -> Self {
        let mut it = words.iter();
        let mut limits = Self {
            time: [Duration::default(); 2],
            inc: [Duration::default(); 2],
        };

        while let Some(command) = it.next() {
            if let Some(time) = it.next() {
                match command.as_str() {
                    "wtime" => *Color::White.index_mut(&mut limits.time) = Duration::from_secs(time.parse::<u64>().unwrap()),
                    "btime" => *Color::Black.index_mut(&mut limits.time) = Duration::from_secs(time.parse::<u64>().unwrap()),
                    "winc" => *Color::White.index_mut(&mut limits.inc) = Duration::from_secs(time.parse::<u64>().unwrap()),
                    "binc" => *Color::Black.index_mut(&mut limits.inc) = Duration::from_secs(time.parse::<u64>().unwrap()),
                    _ => (),
                }
            }
        }

        limits
    }
}
