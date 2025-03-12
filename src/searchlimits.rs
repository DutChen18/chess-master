use crate::types::Color;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct SearchLimits {
    time: [Duration; 2],
    inc: [Duration; 2],
    movetime: Duration,
    depth: usize,
    
    /*
    searchmoves: Vec<Move>,
    ponder: bool,
    movestogo: usize,
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
            movetime: Duration::default(),
            depth: usize::MAX,
        };

        while let Some(command) = it.next() {
            if let Some(arg) = it.next() {
                if let Ok(time) = arg.parse::<u64>() {
                    let duration = Duration::from_millis(time);

                    match *command {
                        "wtime" => *Color::White.index_mut(&mut limits.time) = duration,
                        "btime" => *Color::Black.index_mut(&mut limits.time) = duration,
                        "winc" => *Color::White.index_mut(&mut limits.inc) = duration,
                        "binc" => *Color::Black.index_mut(&mut limits.inc) = duration,
                        "movetime" => limits.movetime = duration,
                        "depth" => limits.depth = time as usize,
                        _ => (),
                    }
                }
            }
        }

        limits
    }

    pub fn get_end_time(&self, color: Color) -> Instant {
        if self.movetime != Duration::default() {
            return Instant::now() + self.movetime.mul_f32(0.9);
        }

        Instant::now() + *color.index(&self.inc) + std::cmp::max(Duration::from_millis(100), color.index(&self.time).div_f32(50.0))
    }
}
