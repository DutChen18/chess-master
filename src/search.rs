use std::{collections::HashSet, time::Instant};

use crate::{
    engine::Engine,
    gen::{self, MoveVec},
    global::GlobalData,
    r#move::Move,
    tt::{Bound, Entry},
    types::Color,
};

const MIN_SCORE: i16 = i16::MIN + 1;
const MAX_SCORE: i16 = i16::MAX;

pub fn sort_moves(engine: &mut Engine, moves: &mut [Move]) {
    let piece_square = GlobalData::get().square();
    let phase = engine.position().phase();

    moves.sort_unstable_by_key(|r#move| {
        let piece = engine.position().get(r#move.from()).unwrap();

        if let Some(capture) = engine.position().get(r#move.to()) {
            const VALUE: [i32; 6] = [0, 1, 2, 3, 4, 5];

            return *piece.kind().index(&VALUE) * 6 + 5 - *capture.kind().index(&VALUE);
        }

        200 + (piece_square.get(piece, r#move.from(), phase) - piece_square.get(piece, r#move.to(), phase)) as i32
    });
}

pub fn quiesce(engine: &mut Engine, mut alpha: i16, beta: i16) -> i16 {
    let static_score = engine.position().evaluate();
    let mut best_score = static_score;

    if static_score >= beta {
        return static_score;
    } else if alpha < static_score {
        alpha = static_score;
    }

    let mut moves = MoveVec::new();

    // TODO: also do checks in quiescence search

    gen::generate_dyn::<false>(&mut moves, engine.position());

    for r#move in moves.moves() {
        let undo = engine.position_mut().make(*r#move);
        let score = -quiesce(engine, -beta, -alpha);

        engine.position_mut().unmake(undo);

        if score >= beta {
            return score;
        }

        if score > best_score {
            best_score = score;
        }

        if score > alpha {
            alpha = score;
        }
    }

    return best_score;
}

pub fn alpha_beta(
    engine: &mut Engine,
    end: Instant,
    mut alpha: i16,
    beta: i16,
    depth: u16,
) -> Option<i16> {
    if depth == 0 {
        // return Some(engine.position().evaluate());
        return Some(quiesce(engine, alpha, beta));
    } else if depth >= 4 && Instant::now() >= end {
        return None;
    }

    let mut moves = MoveVec::new();
    let mut best_score = MIN_SCORE;
    let mut best_move = Move::null();
    let mut bound = Bound::Upper;

    let hash = engine.position().hash();
    let entry = engine.tt().probe(hash);

    gen::generate_dyn::<true>(&mut moves, engine.position());

    if moves.moves().is_empty() {
        if moves.check() {
            best_score = MIN_SCORE + engine.position().ply() as i16;
        } else {
            best_score = 0;
        }
    }

    let mut tt_hit = false;

    if let Some(entry) = entry {
        if let Some(index) = moves
            .moves()
            .iter()
            .position(|r#move| *r#move == entry.r#move())
        {
            moves.moves_mut().swap(0, index);
            tt_hit = true;
        }

        if match entry.bound() {
            Bound::Exact => true,
            Bound::Lower => entry.score() >= beta,
            Bound::Upper => entry.score() < alpha,
        } && entry.depth() >= depth
            && tt_hit
        {
            return Some(entry.score());
        }
    }

    if !tt_hit {
        sort_moves(engine, moves.moves_mut());
    }

    for i in 0..moves.moves().len() {
        if tt_hit && i == 1 {
            sort_moves(engine, &mut moves.moves_mut()[1..]);
        }

        let r#move = moves.moves()[i];
        let undo = engine.position_mut().make(r#move);
        let score = -alpha_beta(engine, end, -beta, -alpha, depth - 1)?;

        engine.position_mut().unmake(undo);

        if score > best_score {
            best_score = score;
            best_move = r#move;

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
        let mut visited = HashSet::new();

        if engine.position().turn() == Color::Black {
            score = -score;
        }

        get_pv(engine, &mut pv, &mut visited);

        best_move = pv[0];

        print!("info depth {depth} time {ms} score cp {score} pv");

        for r#move in pv {
            print!(" {}", r#move);
        }

        println!();
    }

    best_move
}

pub fn get_pv(engine: &mut Engine, pv: &mut Vec<Move>, visited: &mut HashSet<u64>) {
    let hash = engine.position().hash();

    if visited.contains(&hash) {
        return;
    }

    visited.insert(hash);

    if let Some(entry) = engine.tt().probe(hash) {
        let mut moves = MoveVec::new();

        gen::generate_dyn::<true>(&mut moves, engine.position());

        if moves.moves().contains(&entry.r#move()) {
            pv.push(entry.r#move());

            let undo = engine.position_mut().make(entry.r#move());

            get_pv(engine, pv, visited);
            engine.position_mut().unmake(undo);
        }
    }
}
