use crate::parity_game::ParityGame;
use std::ops;

pub fn run_si(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

pub fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    let mut strat0 = vec![None; game.num_nodes()];
    let mut strat1 = vec![None; game.num_nodes()];

    for node in 0..game.num_nodes() {
        let player = game.get_owner(node);
        let best = game.get_successors(node).iter().min_by_key(|&s| game.get_priority(*s)).unwrap();
        if player == 0 {
            strat0[node] = Some(*best);
        } else {
            strat1[node] = Some(*best);
        }
    }

    let mut halting_set: Vec<usize> = (0..game.num_nodes()).collect();

        
    loop {
        let mut sigma_or_h_changed = false;

        loop {
            let in_halting = build_halting_map(game.num_nodes(), &halting_set);
            let valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

            let mut tau_changed = false;
            for node in 0..game.num_nodes() {
                if game.get_owner(node) == 1 {
                    let current_succ = strat1[node].unwrap();
                    let mut best_succ = current_succ;
                    let mut best_val = valuations[current_succ];

                    for &succ in game.get_successors(node).iter() {
                        let succ_val = valuations[succ];
                        if succ_val < best_val {
                            best_val = succ_val;
                            best_succ = succ;
                        }
                    }

                    if best_succ != current_succ {
                        strat1[node] = Some(best_succ);
                        tau_changed = true;
                    }
                }
            }

            if !tau_changed {
                break;
            }
        }

        let in_halting = build_halting_map(game.num_nodes(), &halting_set);
        let valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

        for node in 0..game.num_nodes() {
            if game.get_owner(node) == 0 {
                let current_succ = strat0[node].unwrap();
                let mut best_succ = current_succ;
                let mut best_val = valuations[current_succ];

                for &succ in game.get_successors(node).iter() {
                    let succ_val = valuations[succ];
                    if succ_val > best_val {
                        best_val = succ_val;
                        best_succ = succ;
                    }
                }

                if best_succ != current_succ {
                    strat0[node] = Some(best_succ);
                    sigma_or_h_changed = true;
                }
            }
        }

        halting_set.retain(|&node| {
            if valuations[node].score >= 0 {
                sigma_or_h_changed = true;
                false 
            } else {
                true
            }
        });

        if !sigma_or_h_changed {
            break;
        }
    }
    

    let in_halting = build_halting_map(game.num_nodes(), &halting_set);
    let final_valuations = compute_all_valuations(game, &strat0, &strat1, &in_halting);

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    for node in 0..game.num_nodes() {
        if final_valuations[node].score >= 0 {
            w0.push(node);
        } else {
            w1.push(node);
        }
    }

    (w0, w1, strat0, strat1)
}


fn build_halting_map(node_count: usize, halting_set: &[usize]) -> Vec<bool> {
    let mut in_halting = vec![false; node_count];
    for &node in halting_set {
        if node < node_count {
            in_halting[node] = true;
        }
    }
    in_halting
}


enum Valuation {
    Infinite,
    Finite(Vec<usize>),
}

impl ops::Add for &Valuation {
    type Output = Valuation;

    fn add(self, other: &Valuation) -> Valuation {
        match (self, other) {
            (Valuation::Infinite, _) | (_, Valuation::Infinite) => Valuation::Infinite,
            (Valuation::Finite(vec1), Valuation::Finite(vec2)) => {
                let summed_vec: Vec<usize> = vec1.iter().zip(vec2.iter()).map(|(a, b)| a + b).collect();
                Valuation::Finite(summed_vec)
            }
        }
    }
}


// impl Valuation {
//     fn compare(&self, other: &Valuation) -> std::cmp::Ordering {
//         match (self, other) {
//             (Valuation::Infinite, Valuation::Infinite) => std::cmp::Ordering::Equal,
//             (Valuation::Infinite, _) => std::cmp::Ordering::Greater,
//             (_, Valuation::Infinite) => std::cmp::Ordering::Less,
//             (Valuation::Finite(vec1), Valuation::Finite(vec2)) => compare_vec(vec1, vec2),
//         }
//     }
// }


fn compare_vec(vec1: &[usize], vec2: &[usize]) -> std::cmp::Ordering {
    // TODO compare base on mod 2
    std::cmp::Ordering::Equal
}

fn compute_all_valuations(game: &ParityGame, strat0: &[Option<usize>], strat1: &[Option<usize>], in_halting: &[bool]) -> Vec<Valuation> {
    let mut valuations = vec![Valuation::Finite(vec![0, game.get_priorities().max()]); game.num_nodes()];
    let mut visited = vec![false; game.num_nodes()];
    let mut nodes_with_no_successors = Vec::new();
    for node in 0..game.num_nodes() {
        let successor = if game.get_owner(node) == 0 {
            strat0[node]
        } else {
            strat1[node]
        };
        if successor.is_none() || in_halting[node] {
            nodes_with_no_successors.push(node);
        } 
    }

    while let Some(node) = nodes_with_no_successors.pop() {
        if visited[node] {
            continue;
        }
        dfs_backwards(game, node, strat0, strat1, in_halting, &mut valuations, &mut visited);
    }

    for node in 0..game.num_nodes() {
        if !visited[node] {
            valuations[node] = Valuation::Infinite;
        }
    }


    valuations
}


fn dfs_backwards(game: &ParityGame, node: usize, strat0: &[Option<usize>], strat1: &[Option<usize>], in_halting: &[bool], valuations: &mut [Valuation], visited: &mut [bool]) {
    if visited[node] {
        return;
    }

    visited[node] = true;

    if valuations[node] == Valuation::Infinite {
        return;
    }


    for &pred in game.get_predecessors(node) {
        if in_halting[pred] || (strat0[pred] != Some(node) && strat1[pred] != Some(node)) {
            continue;
        } 

        dfs_backwards(game, pred, strat0, strat1, in_halting, valuations, visited);

        let valuation = &valuations[pred] + &valuation[node];
    }

    match valuations[node] {
        Valuation::Infinite => {},
        Valuation::Finite(ref mut vec) => {
            let priority = game.get_priority(node);
            vec[priority] += 1;
        }
    }
}