use std::{collections::HashSet, time::Instant};

use crate::{
    bitboard::Bitboard,
    engine::Engine,
    gen::{self, Generator, MoveVec},
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

fn quiesce(
    engine: &mut Engine,
    stats: &mut Stats,
    mut alpha: i16,
    beta: i16,
    quiet_limit: usize,
) -> i16 {
    let mut best_move = Move::null();
    let mut best_index = None;
    let mut bound = Bound::Upper;
    let generator = Generator::new_dyn(engine.position());

    let mut pick = if generator.checkers() != Bitboard(0) {
        Pick::new::<true, true>(engine, &generator)
    } else if quiet_limit > 0 {
        Pick::new::<false, true>(engine, &generator)
    } else {
        Pick::new::<false, false>(engine, &generator)
    };

    // TODO: fix

    // let mut pick = 
    //     Pick::new::<false, false>(engine, &generator);

    let mut best_score = if let Some(entry) = pick.entry() {
        // TT cut
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

    if generator.checkers() != Bitboard(0) {
        if pick.is_empty() {
            best_score = MIN_SCORE + (engine.position().ply() - stats.root_ply) as i16 + 1;
        } else {
            best_score = MIN_SCORE;
        }
    } else {
        if best_score > alpha {
            alpha = best_score;
        }

        if best_score >= beta {
            return best_score;
        }
    }

    // Search all children
    while let Some((i, r#move)) = pick.next(engine.position()) {
        let mut new_quiet_limit = quiet_limit;

        // TODO: consider en-passant as capture
        if generator.checkers() == Bitboard(0) && !engine.position().get(r#move.to()).is_some() {
            new_quiet_limit = new_quiet_limit.saturating_sub(1);
        }

        let undo = engine.position_mut().make(r#move);
        let score = -quiesce(engine, stats, -beta, -alpha, new_quiet_limit);

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

    // Update TT
    let hash = engine.position().hash();
    let age = engine.age();

    engine
        .tt_mut()
        .insert(Entry::new(hash, age, best_move, 0, best_score, bound));

    // Update stats
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
    root: bool,
) -> Option<i16> {
    if depth == 0 {
        return Some(quiesce(engine, stats, alpha, beta, 1));
    } else if depth >= 4 && Instant::now() >= end {
        return None;
    }

    let mut best_score = MIN_SCORE;
    let mut best_move = Move::null();
    let mut best_index = None;
    let mut bound = Bound::Upper;
    let generator = Generator::new_dyn(engine.position());
    let mut pick = Pick::new::<true, true>(engine, &generator);

    // Checkmate or draw
    if !root && engine.position().is_technical_draw() {
        return Some(0);
    }

    if pick.is_empty() {
        if generator.checkers() != Bitboard(0) {
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
    if generator.checkers() == Bitboard(0)
        && !engine.position().is_king_and_pawn(engine.position().turn())
        && depth >= 3
    {
        engine.position_mut().make_null();

        let score = alpha_beta(engine, stats, end, -beta, -(beta - 1), depth - 3, false);

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
        if i >= 2 && new_depth >= 2 {
            new_depth -= 1;
        }

        let undo = engine.position_mut().make(r#move);
        let score = alpha_beta(engine, stats, end, -beta, -alpha, new_depth, false);

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
    if engine.options.ownbook {
        if let Some(r#move) = engine.book().next(engine.position()) {
            return r#move;
        }
    }

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
            let Some(s) = alpha_beta(engine, &mut stats, end, min_score, max_score, depth, true)
            else {
                return best_move;
            };

            score = s;

            if score <= min_score || score >= max_score {
                min_score = MIN_SCORE;
                max_score = MAX_SCORE;
            } else {
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
