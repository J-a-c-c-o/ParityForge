use crate::parity_game::ParityGame;
use std::ops;

pub fn run_si(
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
    Ok(solve(game))
}

pub fn solve(
    game: &ParityGame,
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    let zeros = Valuation::Finite(vec![0; game.get_max_priority() + 1]);
    let mut strat: Vec<usize> = vec![0; game.num_nodes()];

    for node in 0..game.num_nodes() {
        let best = game
            .get_successors(node)
            .iter()
            .min_by_key(|&s| game.get_priority(*s))
            .unwrap();
        strat[node] = *best;
    }

    let mut in_halting = vec![false; game.num_nodes()];
    for node in 0..game.num_nodes() {
        if game.get_priority(node) % 2 == 1 {
            in_halting[node] = true;
        }
    }

    let mut valuations;

    loop {
        loop {
            valuations = compute_all_valuations(game, &strat, &in_halting);

            let tau_changed = switch_rule(game, &mut strat, &in_halting, &valuations, 1);

            if !tau_changed {
                break;
            }
        }

        for val in valuations.iter_mut() {
            if *val == Valuation::Infinite {
                *val = Valuation::Won;
            }
        }

        let mut h_changed = false;

        let sigma_changed = switch_rule(game, &mut strat, &in_halting, &valuations, 0);

        for node in 0..game.num_nodes() {
            if in_halting[node] && valuations[node] > zeros {
                in_halting[node] = false;
                h_changed = true;
            }
        }

        if !sigma_changed && !h_changed {
            break;
        }
    }

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut new_strat0: Vec<Option<usize>> = vec![None; game.num_nodes()];
    let mut new_strat1: Vec<Option<usize>> = vec![None; game.num_nodes()];
    for node in 0..game.num_nodes() {
        if valuations[node] == Valuation::Won {
            w0.push(node);
            new_strat0[node] = Some(strat[node]);
        } else {
            w1.push(node);
            new_strat1[node] = Some(strat[node]);
        }
    }

    (w0, w1, new_strat0, new_strat1)
}

fn switch_rule(
    game: &ParityGame,
    strat: &mut [usize],
    in_halting: &[bool],
    valuations: &[Valuation],
    player: usize,
) -> bool {
    let mut changed = false;
    let zeros = Valuation::Finite(vec![0; game.get_max_priority() + 1]);
    for node in 0..game.num_nodes() {
        if game.get_owner(node) == player && valuations[node] != Valuation::Won {
            let current_succ = strat[node];
            let mut best_succ = current_succ;
            let mut best_val = &valuations[current_succ];

            for &succ in game.get_successors(node).iter() {
                let succ_val = if in_halting[succ] {
                    &zeros
                } else {
                    &valuations[succ]
                };
                if player == 0 {
                    if succ_val > best_val {
                        best_val = succ_val;
                        best_succ = succ;
                    }
                } else {
                    if succ_val < best_val {
                        best_val = succ_val;
                        best_succ = succ;
                    }
                }
            }

            if best_succ != current_succ {
                strat[node] = best_succ;
                changed = true;
            }
        }
    }
    changed
}

#[derive(Debug, Clone)]
enum Valuation {
    Infinite,
    Won,
    Finite(Vec<usize>),
}

impl ops::Add for &Valuation {
    type Output = Valuation;

    fn add(self, other: &Valuation) -> Valuation {
        match (self, other) {
            (Valuation::Infinite, _) | (_, Valuation::Infinite) => Valuation::Infinite,
            (Valuation::Won, _) | (_, Valuation::Won) => Valuation::Won,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => {
                let summed_vec: Vec<usize> =
                    vec1.iter().zip(vec2.iter()).map(|(a, b)| a + b).collect();
                Valuation::Finite(summed_vec)
            }
        }
    }
}

impl PartialEq for Valuation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Valuation::Infinite, Valuation::Infinite) => true,
            (Valuation::Won, Valuation::Won) => true,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => vec1 == vec2,
            _ => false,
        }
    }
}

impl Eq for Valuation {}

impl PartialOrd for Valuation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Valuation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Valuation::Infinite, Valuation::Infinite) => std::cmp::Ordering::Equal,
            (Valuation::Won, Valuation::Won) => std::cmp::Ordering::Equal,
            (Valuation::Won, Valuation::Infinite) => std::cmp::Ordering::Equal,
            (Valuation::Infinite, Valuation::Won) => std::cmp::Ordering::Equal,
            (Valuation::Won, _) => std::cmp::Ordering::Greater,
            (_, Valuation::Won) => std::cmp::Ordering::Less,
            (Valuation::Infinite, _) => std::cmp::Ordering::Greater,
            (_, Valuation::Infinite) => std::cmp::Ordering::Less,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => compare_vec(vec1, vec2),
        }
    }
}

fn compare_vec(vec1: &[usize], vec2: &[usize]) -> std::cmp::Ordering {
    for (priority, (a, b)) in vec1.iter().zip(vec2.iter()).enumerate().rev() {
        if a != b {
            if priority % 2 == 0 {
                return a.cmp(b);
            } else {
                return b.cmp(a);
            }
        }
    }
    std::cmp::Ordering::Equal
}

fn compute_all_valuations(
    game: &ParityGame,
    strat: &[usize],
    in_halting: &[bool],
) -> Vec<Valuation> {
    let mut valuations =
        vec![Valuation::Finite(vec![0; game.get_max_priority() + 1]); game.num_nodes()];
    let mut visited = vec![false; game.num_nodes()];
    let mut nodes_with_no_successors = Vec::new();
    for node in 0..game.num_nodes() {
        if valuations[node] == Valuation::Won {
            continue;
        }
        let successor = strat[node];
        if in_halting[successor] {
            nodes_with_no_successors.push(node);
        }
    }

    while let Some(node) = nodes_with_no_successors.pop() {
        dfs_backwards(game, node, strat, in_halting, &mut valuations, &mut visited);
    }

    for node in 0..game.num_nodes() {
        if !visited[node] {
            valuations[node] = Valuation::Infinite;
        }
    }

    valuations
}

fn dfs_backwards(
    game: &ParityGame,
    node: usize,
    strat: &[usize],
    in_halting: &[bool],
    valuations: &mut [Valuation],
    visited: &mut [bool],
) {
    visited[node] = true;

    match valuations[node] {
        Valuation::Infinite => {}
        Valuation::Won => {}
        Valuation::Finite(ref mut vec) => {
            let priority = game.get_priority(node);
            vec[priority] += 1;
        }
    }

    if in_halting[node] {
        return;
    }

    for &pred in game.get_predecessors(node) {
        if strat[pred] != node {
            continue;
        }

        let valuation = &valuations[pred] + &valuations[node];
        valuations[pred] = valuation;

        dfs_backwards(game, pred, strat, in_halting, valuations, visited);
    }
}

