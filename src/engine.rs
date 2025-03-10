use crate::{r#move::Move, searchlimits::SearchLimits, uci::Uci, types::*, position::Position};

pub struct Engine {
    uci: Uci,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            uci: Uci::new(),
        }
    }

    pub fn run(&mut self) {
        self.uci.run();
    }

    pub fn search(_position: &Position, _limits: SearchLimits) -> Move {
        let bestmove = Move::new(Square::A2, Square::A3);

        bestmove
    }
}
