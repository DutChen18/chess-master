use crate::{
    gen::*, position::Position, r#move::Move, search, searchlimits::SearchLimits, uci::Uci,
};

pub struct Engine {
    uci: Uci,
}

impl Engine {
    pub fn new() -> Self {
        Self { uci: Uci::new() }
    }

    pub fn run(&mut self) {
        self.uci.run();
    }

    pub fn perft(position: &mut Position, depth: usize, root: bool) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut moves: Vec<Move> = Vec::new();

        generate_dyn(&mut moves, position);

        let mut total = 0;

        for m in moves {
            let undo = position.make(m);
            let count = Self::perft(position, depth - 1, false);

            if root {
                println!("{m}: {count}");
            }

            total += count;

            position.unmake(undo);
        }

        total
    }

    pub fn search(position: &Position, _limits: SearchLimits) -> Move {
        let mut position = position.clone();

        search::mini_minimax(&mut position, 4).0
    }
}
