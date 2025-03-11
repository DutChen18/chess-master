use std::time::Instant;

use crate::{
    engine::Engine,
    gen::{self, MoveVec},
    r#move::Move,
    tt::{Bound, Entry},
    types::Color,
};

const MIN_SCORE: i16 = i16::MIN + 1;
const MAX_SCORE: i16 = i16::MAX;

pub fn alpha_beta(search: &mut Engine, mut alpha: i16, beta: i16, depth: u16) -> i16 {
    if depth == 0 {
        return search.position().evaluate();
    }

    let mut moves = MoveVec::new();
    let mut best_score = MIN_SCORE;
    let mut best_move = Move::null();

    let hash = search.position().hash();

    gen::generate_dyn(&mut moves, search.position());

    let entry = search.tt().probe(hash);

    moves.moves_mut().sort_by_key(|r#move| {
        if let Some(entry) = entry {
            if entry.r#move() == *r#move {
                return -1;
            }
        }

        0
    });

    for r#move in moves.moves() {
        let undo = search.position_mut().make(*r#move);
        let score = -alpha_beta(search, -beta, -alpha, depth - 1);

        search.position_mut().unmake(undo);

        if score > best_score {
            best_score = score;
            best_move = *r#move;

            if score > alpha {
                alpha = score;
            }
        }

        if score >= beta {
            break;
        }
    }

    search
        .tt_mut()
        .insert(Entry::new(hash, best_move, depth, best_score, Bound::Exact));

    return best_score;
}

pub fn search(search: &mut Engine, end: Instant) -> Move {
    let start = Instant::now();
    let mut best_move = Move::null();

    for depth in 1.. {
        let mut score = alpha_beta(search, MIN_SCORE, MAX_SCORE, depth);
        let ms = start.elapsed().as_millis();
        let mut pv = Vec::new();

        if search.position().turn() == Color::Black {
            score = -score;
        }

        get_pv(search, &mut pv);

        best_move = pv[0];

        print!("info depth {depth} time {ms} score cp {score} pv");

        for r#move in pv {
            print!(" {}", r#move);
        }

        println!();

        if Instant::now() > end {
            break;
        }
    }

    best_move
}

pub fn get_pv(search: &mut Engine, pv: &mut Vec<Move>) {
    if let Some(entry) = search.tt().probe(search.position().hash()) {
        let mut moves = MoveVec::new();

        gen::generate_dyn(&mut moves, search.position());

        if moves.moves().contains(&entry.r#move()) {
            pv.push(entry.r#move());

            let undo = search.position_mut().make(entry.r#move());

            get_pv(search, pv);
            search.position_mut().unmake(undo);
        }
    }
}
