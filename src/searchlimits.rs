use crate::types::Color;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct SearchLimits {
    time: [Duration; 2],
    inc: [Duration; 2],
    movetime: Option<Duration>,
    /*
    searchmoves: Vec<Move>,
    ponder: bool,
    movestogo: usize,
    depth: usize,
    nodes: usize,
    mate: usize,
    infinite: bool,
    */
}

impl SearchLimits {
    pub fn parse<'a>(words: &[&str]) -> Self {
        let mut it = words.iter();
        let mut limits = Self {
            time: [Duration::default(); 2],
            inc: [Duration::default(); 2],
            movetime: None,
        };

        while let Some(command) = it.next() {
            if let Some(time) = it.next() {
                match *command {
                    "wtime" => {
                        *Color::White.index_mut(&mut limits.time) =
                            Duration::from_millis(time.parse::<u64>().unwrap())
                    }
                    "btime" => {
                        *Color::Black.index_mut(&mut limits.time) =
                            Duration::from_millis(time.parse::<u64>().unwrap())
                    }
                    "winc" => {
                        *Color::White.index_mut(&mut limits.inc) =
                            Duration::from_millis(time.parse::<u64>().unwrap())
                    }
                    "binc" => {
                        *Color::Black.index_mut(&mut limits.inc) =
                            Duration::from_millis(time.parse::<u64>().unwrap())
                    }
                    "movetime" => {
                        limits.movetime = Some(Duration::from_millis(time.parse().unwrap()))
                    }
                    _ => (),
                }
            }
        }

        limits
    }

    pub fn get_end_time(&self, color: Color) -> Instant {
        if let Some(movetime) = self.movetime {
            return Instant::now() + movetime;
        }

        Instant::now() + std::cmp::min(*color.index(&self.time) / 2, Duration::from_millis(100)) + *color.index(&self.inc)
    }
}
