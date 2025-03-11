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

pub fn alpha_beta(
    engine: &mut Engine,
    end: Instant,
    mut alpha: i16,
    beta: i16,
    depth: u16,
) -> Option<i16> {
    if depth == 0 {
        return Some(engine.position().evaluate());
    } else if depth >= 4 && Instant::now() >= end {
        return None;
    }

    let mut moves = MoveVec::new();
    let mut best_score = MIN_SCORE;
    let mut best_move = Move::null();
    let mut bound = Bound::Upper;

    let hash = engine.position().hash();
    let entry = engine.tt().probe(hash);

    gen::generate_dyn(&mut moves, engine.position());

    if let Some(entry) = entry {
        if match entry.bound() {
            Bound::Exact => true,
            Bound::Lower => entry.score() >= beta,
            Bound::Upper => entry.score() < alpha,
        } && entry.depth() >= depth
            && moves.moves().contains(&entry.r#move())
        {
            return Some(entry.score());
        }
    }

    moves.moves_mut().sort_unstable_by_key(|r#move| {
        if let Some(entry) = entry {
            if entry.r#move() == *r#move {
                return i16::MIN;
            }
        }

        0
    });

    for r#move in moves.moves() {
        let undo = engine.position_mut().make(*r#move);
        let score = -alpha_beta(engine, end, -beta, -alpha, depth - 1)?;

        engine.position_mut().unmake(undo);

        if score > best_score {
            best_score = score;
            best_move = *r#move;

            if score > alpha {
                alpha = score;
                bound = Bound::Exact;
            }
        }

        if score >= beta {
            bound = Bound::Lower;
            break;
        }
    }

    let age = engine.age();

    engine
        .tt_mut()
        .insert(Entry::new(hash, age, best_move, depth, best_score, bound));

    // TODO: checkmate score

    Some(best_score)
}

pub fn search(engine: &mut Engine, end: Instant) -> Move {
    let start = Instant::now();
    let mut best_move = Move::null();

    for depth in 1.. {
        let Some(mut score) = alpha_beta(engine, end, MIN_SCORE, MAX_SCORE, depth) else {
            break;
        };

        let ms = start.elapsed().as_millis();
        let mut pv = Vec::new();

        if engine.position().turn() == Color::Black {
            score = -score;
        }

        get_pv(engine, &mut pv);

        best_move = pv[0];

        print!("info depth {depth} time {ms} score cp {score} pv");

        for r#move in pv {
            print!(" {}", r#move);
        }

        println!();
    }

    best_move
}

pub fn get_pv(engine: &mut Engine, pv: &mut Vec<Move>) {
    if let Some(entry) = engine.tt().probe(engine.position().hash()) {
        let mut moves = MoveVec::new();

        gen::generate_dyn(&mut moves, engine.position());

        if moves.moves().contains(&entry.r#move()) {
            pv.push(entry.r#move());

            let undo = engine.position_mut().make(entry.r#move());

            get_pv(engine, pv);
            engine.position_mut().unmake(undo);
        }
    }
}
