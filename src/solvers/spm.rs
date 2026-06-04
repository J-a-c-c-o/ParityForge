use crate::parity_game::ParityGame;

use std::cmp::Ordering;

pub fn run_spm(
    game: &ParityGame,
) -> Result<
    (
        Vec<usize>,
        Vec<usize>,
        Vec<Option<usize>>,
        Vec<Option<usize>>,
    ),
    String,
> {
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
        if measures[v] < lift {
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
            if prog == measures[v] {
                strat[v] = Some(u);
                break;
            }
        }
    }

    (w, strat)
}

fn lift(
    game: &ParityGame,
    alpha: usize,
    measures: &[Measure],
    bounds: &[usize],
    v: usize,
) -> Measure {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_game::ParityGameBuilder;

    fn example_game() -> ParityGame {
        let mut builder = ParityGameBuilder::new();
        let builder = builder
            .add_edge(0, 4)
            .add_edge(4, 0)
            .add_edge(7, 4)
            .add_edge(1, 7)
            .add_edge(4, 8)
            .add_edge(8, 6)
            .add_edge(7, 6)
            .add_edge(2, 1)
            .add_edge(2, 3)
            .add_edge(3, 2)
            .add_edge(3, 5)
            .add_edge(5, 3)
            .add_edge(5, 1)
            .add_edge(1, 5)
            .add_edge(6, 2)
            .set_owner(0, 0)
            .set_owner(1, 1)
            .set_owner(2, 0)
            .set_owner(3, 0)
            .set_owner(4, 1)
            .set_owner(5, 0)
            .set_owner(6, 0)
            .set_owner(7, 0)
            .set_owner(8, 0)
            .set_priority(0, 0)
            .set_priority(1, 1)
            .set_priority(2, 2)
            .set_priority(3, 3)
            .set_priority(4, 2)
            .set_priority(5, 5)
            .set_priority(6, 6)
            .set_priority(7, 7)
            .set_priority(8, 8);

        let game = builder.build();
        game
    }

    #[test]
    fn test_spm() {
        let game = example_game();
        let (w0, w1, strat0, strat1) = run_spm(&game).unwrap();
        println!("Winning set for player 0: {:?}", w0);
        println!("Winning set for player 1: {:?}", w1);
        println!("Strategy for player 0: {:?}", strat0);
        println!("Strategy for player 1: {:?}", strat1);

        panic!();
    }
}
