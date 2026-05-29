use crate::parity_game::ParityGame;

use std::cmp::Ordering;

pub fn run_spm(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    let (w0, strat0) = solve(game, 0);
    let (w1, strat1) = solve(game, 1);
    Ok((w0, w1, strat0, strat1))
}

fn solve(game: &ParityGame, alpha: usize) -> (Vec<usize>, Vec<Option<usize>>) {
    let num_nodes = game.num_nodes();
    let alpha_bar = 1 - alpha;
    let max_priority = game.get_max_priority();
    let mut bounds = vec![0; max_priority + 1];
    for v in 0..num_nodes {
        let p = game.get_priority(v);
        if p % 2 == alpha_bar {
            bounds[p] += 1;
        }
    }

    let mut measures: Vec<Measure> = vec![Measure::Fin(vec![0; max_priority + 1]); num_nodes];

    let mut queue: Vec<usize> = (0..num_nodes)
        .filter(|&v| game.get_priority(v) % 2 == alpha_bar)
        .collect();

    while let Some(v) = queue.pop() {
        let lift = lift(game, alpha, &measures, &bounds, v);
        if &measures[v] < &lift {
            measures[v] = lift;
            for &u in game.get_predecessors(v) {
                queue.push(u);
            }
        }
    }

    let mut w = Vec::new();
    let mut strat = vec![None; num_nodes];
    for v in 0..num_nodes {
        if measures[v] != Measure::Inf {
            w.push(v);
        }
    }

    for &v in &w {
        if game.get_owner(v) != alpha {
            continue;
        }
        let priority = game.get_priority(v);
        for &u in game.get_successors(v) {
            let prog = prog(alpha, &bounds, &measures[u], priority);
            if &prog == &measures[v] {
                strat[v] = Some(u);
                break;
            }
        }
    }


    (w, strat)
}



fn lift(game: &ParityGame, alpha: usize, measures: &[Measure], bounds: &[usize], v: usize) -> Measure {
    if game.get_owner(v) == alpha {
        let mut best: Option<Measure> = None;
        let priority = game.get_priority(v);
        for &w in game.get_successors(v) {
            let m = &measures[w];
            let prog = prog(alpha, bounds, m, priority);
            if best.is_none() || &prog < best.as_ref().unwrap() {
                best = Some(prog);
            }
        }
        best.unwrap_or(Measure::Inf)
    } else {
        let mut worst: Option<Measure> = None;
        let priority = game.get_priority(v);
        for &w in game.get_successors(v) {
            let m = &measures[w];
            let prog = prog(alpha, bounds, m, priority);
            if worst.is_none() || &prog > worst.as_ref().unwrap() {
                worst = Some(prog);
            }
        }
        worst.unwrap_or(Measure::Inf)
    }
}

fn prog(alpha: usize, bounds: &[usize], measure: &Measure, priority: usize) -> Measure {
    match measure {
        Measure::Inf => Measure::Inf,
        Measure::Fin(vec) => {
            if priority % 2 == alpha {
                let mut next = vec.clone();
                for idx in 0..priority {
                    next[idx] = 0;
                }
                Measure::Fin(next)
            } else {
                let alpha_bar = 1 - alpha;
                let mut target_idx = None;

                for idx in priority..vec.len() {
                    if idx % 2 == alpha_bar && vec[idx] < bounds[idx] {
                        target_idx = Some(idx);
                        break;
                    }
                }

                if let Some(i) = target_idx {
                    let mut next = vec.clone();
                    next[i] += 1;
                    for idx in 0..i {
                        next[idx] = 0;
                    }
                    Measure::Fin(next)
                } else {
                    Measure::Inf
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Measure {
    Fin(Vec<usize>),
    Inf,
}


impl Ord for Measure {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Measure::Inf, Measure::Inf) => Ordering::Equal,
            (Measure::Inf, _) => Ordering::Greater,
            (_, Measure::Inf) => Ordering::Less,
            (Measure::Fin(vec1), Measure::Fin(vec2)) => compare_vec(vec1, vec2),
        }
    }
}

fn compare_vec(vec1: &[usize], vec2: &[usize]) -> Ordering {
    for idx in (0..vec1.len()).rev() {
        match vec1[idx].cmp(&vec2[idx]) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}


impl PartialOrd for Measure {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl PartialEq for Measure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Measure::Inf, Measure::Inf) => true,
            (Measure::Fin(vec1), Measure::Fin(vec2)) => vec1 == vec2,
            _ => false,
        }
    }
}

impl Eq for Measure {}