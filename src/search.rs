use crate::{gen, position::Position, r#move::Move, types::Square};

pub fn mini_minimax(position: &mut Position, depth: usize) -> (Move, i16) {
    let mut best_move = Move::new(Square::A1, Square::A1);

    if depth == 0 {
        return (best_move, position.evaluate());
    }

    let mut moves: Vec<Move> = Vec::new();
    let mut best_score = i16::MIN;

    gen::generate_dyn(&mut moves, position);

    for r#move in moves {
        let undo = position.make(r#move);

        let score = -mini_minimax(position, depth - 1).1;

        position.unmake(undo);

        if score > best_score {
            best_score = score;
            best_move = r#move;
        }
    }

    (best_move, best_score)
}
