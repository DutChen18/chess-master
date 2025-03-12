use std::{collections::HashSet, time::Instant};

use crate::{
    engine::Engine,
    gen::{self, MoveVec},
    pick::Pick,
    r#move::Move,
    tt::{Bound, Entry},
};

const MAX_SCORE: i16 = i16::MAX / 2;
const MIN_SCORE: i16 = -MAX_SCORE;
const MATE_SCORE: i16 = MAX_SCORE / 2;

struct Stats {
    root_ply: u32,
    best_index_distribution: Vec<usize>,
}

// TODO: also do checks in quiescence search
fn quiesce(engine: &mut Engine, stats: &mut Stats, mut alpha: i16, beta: i16) -> i16 {
    let mut best_move = Move::null();
    let mut best_index = None;
    let mut bound = Bound::Upper;
    let mut pick = Pick::new::<false>(engine);

    let mut best_score = if let Some(entry) = pick.entry() {
        if match entry.bound() {
            Bound::Exact => true,
            Bound::Lower => entry.score() >= beta,
            Bound::Upper => entry.score() < alpha,
        } {
            return entry.score();
        }

        entry.score()
    } else {
        engine.position().evaluate()
    };

    if best_score > alpha {
        alpha = best_score;
    }

    if best_score >= beta {
        return best_score;
    }

    while let Some((i, r#move)) = pick.next(engine.position()) {
        let undo = engine.position_mut().make(r#move);
        let score = -quiesce(engine, stats, -beta, -alpha);

        engine.position_mut().unmake(undo);

        if score > best_score {
            best_score = score;
            best_move = r#move;
            best_index = Some(i);

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

    let hash = engine.position().hash();
    let age = engine.age();

    engine
        .tt_mut()
        .insert(Entry::new(hash, age, best_move, 0, best_score, bound));

    if let Some(best_index) = best_index {
        if best_index >= stats.best_index_distribution.len() {
            stats.best_index_distribution.resize(best_index + 1, 0);
        }

        stats.best_index_distribution[best_index] += 1;
    }

    return best_score;
}

fn alpha_beta(
    engine: &mut Engine,
    stats: &mut Stats,
    end: Instant,
    mut alpha: i16,
    beta: i16,
    depth: u16,
) -> Option<i16> {
    if depth == 0 {
        return Some(quiesce(engine, stats, alpha, beta));
    } else if depth >= 4 && Instant::now() >= end {
        return None;
    }

    let mut best_score = MIN_SCORE;
    let mut best_move = Move::null();
    let mut best_index = None;
    let mut bound = Bound::Upper;
    let mut pick = Pick::new::<true>(engine);

    // Checkmate or draw
    if pick.is_empty() {
        if pick.check() {
            return Some(MIN_SCORE + (engine.position().ply() - stats.root_ply) as i16 + 1);
        } else {
            return Some(0);
        }
    }

    // TT cut
    if let Some(entry) = pick.entry() {
        if match entry.bound() {
            Bound::Exact => true,
            Bound::Lower => entry.score() >= beta,
            Bound::Upper => entry.score() < alpha,
        } && entry.depth() >= depth
        {
            return Some(entry.score());
        }
    }

    // Null move pruning
    if !pick.check() && !engine.position().is_king_and_pawn(engine.position().turn()) && depth >= 3
    {
        engine.position_mut().make_null();

        let score = alpha_beta(engine, stats, end, -beta, -(beta - 1), depth - 3);

        engine.position_mut().unmake_null();

        let score = -score?;

        if score >= beta {
            return Some(score);
        }
    }

    // Search all children
    while let Some((i, r#move)) = pick.next(engine.position()) {
        let mut new_depth = depth - 1;

        // Late move reduction
        if i > 2 && new_depth > 0 {
            new_depth -= 1;
        }

        let undo = engine.position_mut().make(r#move);
        let score = alpha_beta(engine, stats, end, -beta, -alpha, new_depth);

        engine.position_mut().unmake(undo);

        let score = -score?;

        if score > best_score {
            best_score = score;
            best_move = r#move;
            best_index = Some(i);

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

    // Update TT
    let hash = engine.position().hash();
    let age = engine.age();

    engine
        .tt_mut()
        .insert(Entry::new(hash, age, best_move, depth, best_score, bound));

    // Updaet stats
    if let Some(best_index) = best_index {
        if best_index >= stats.best_index_distribution.len() {
            stats.best_index_distribution.resize(best_index + 1, 0);
        }

        stats.best_index_distribution[best_index] += 1;
    }

    Some(best_score)
}

fn get_pv(engine: &mut Engine, pv: &mut Vec<Move>, visited: &mut HashSet<u64>) {
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

pub fn search(engine: &mut Engine, end: Instant) -> Move {
    /*
    if let Some(r#move) = engine.book().next(engine.position()) {
        return r#move;
    }
    */

    let start = Instant::now();
    let mut best_move = Move::null();
    let mut min_score = MIN_SCORE;
    let mut max_score = MAX_SCORE;

    for depth in 1.. {
        let mut stats = Stats {
            root_ply: engine.position().ply(),
            best_index_distribution: Vec::new(),
        };

        let mut score;

        loop {
            let Some(s) = alpha_beta(engine, &mut stats, end, min_score, max_score, depth) else {
                return best_move;
            };

            score = s;

            if score <= min_score || score >= max_score {
                eprintln!("outside aspiration window");

                min_score = MIN_SCORE;
                max_score = MAX_SCORE;
            } else {
                eprintln!("inside aspiration window");

                break;
            }
        }

        let ms = start.elapsed().as_millis();
        let mut pv = Vec::new();
        let mut visited = HashSet::new();

        get_pv(engine, &mut pv, &mut visited);

        best_move = pv[0];
        min_score = i16::max(score, MIN_SCORE + 50) - 50;
        max_score = i16::min(score, MAX_SCORE - 50) + 50;

        print!("info depth {depth} time {ms} score ");

        if score.abs() > MATE_SCORE {
            if score > 0 {
                print!("mate {}", (MAX_SCORE - score) / 2);
            } else {
                print!("mate {}", (MIN_SCORE - score) / 2);
            }
        } else {
            print!("cp {score}");
        }

        print!(" pv");

        for r#move in pv {
            print!(" {}", r#move);
        }

        println!();

        let total: usize = stats.best_index_distribution.iter().sum();

        eprintln!(
            "{:.03?}",
            stats
                .best_index_distribution
                .iter()
                .take(10)
                .map(|count| *count as f64 / total as f64)
                .collect::<Vec<_>>()
        );
    }

    best_move
}
