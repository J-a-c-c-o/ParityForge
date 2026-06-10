use crate::parity_game::ParityGame;
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn run_zielonka(
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
    let excluded = vec![false; game.num_nodes()];
    Ok(solve(game, &excluded))
}

fn solve(
    game: &ParityGame,
    excluded: &[bool],
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    if excluded.iter().all(|&is_excluded| is_excluded) {
        let empty = vec![None; game.num_nodes()];
        return (vec![], vec![], empty.clone(), empty);
    }

    let max_priority = game
        .get_nodes()
        .into_iter()
        .filter(|v| !excluded[*v])
        .map(|v| game.get_priority(v))
        .max()
        .unwrap();

    let player = max_priority % 2;

    let max_nodes: Vec<usize> = game
        .get_nodes_with_priority(max_priority)
        .into_iter()
        .filter(|v| !excluded[*v])
        .collect();

    let (a, strat_a) = attract(
        game,
        excluded,
        &max_nodes,
        player,
        vec![None; game.num_nodes()],
    );

    let mut excluded_a = excluded.to_vec();
    for node in &a {
        excluded_a[*node] = true;
    }

    let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, &excluded_a);

    let opponent_region = if player == 0 { &w1 } else { &w0 };
    let opponent_strategy = if player == 0 { &strat_w1 } else { &strat_w0 };

    let (b, strat_b) = attract(
        game,
        excluded,
        opponent_region,
        1 - player,
        opponent_strategy.clone(),
    );

    let mut b_sorted = b.clone();
    b_sorted.sort_unstable();
    let mut opp_sorted = opponent_region.clone();
    opp_sorted.sort_unstable();

    if b_sorted == opp_sorted {
        if player == 0 {
            w0.extend(a);
            merge_strategy(&mut strat_w0, &strat_a);
            let pick_strat = pick(game, &max_nodes, &w0);
            merge_strategy(&mut strat_w0, &pick_strat);
        } else {
            w1.extend(a);
            merge_strategy(&mut strat_w1, &strat_a);
            let pick_strat = pick(game, &max_nodes, &w1);
            merge_strategy(&mut strat_w1, &pick_strat);
        }

        (w0, w1, strat_w0, strat_w1)
    } else {
        let mut excluded_b = excluded.to_vec();
        for node in &b {
            excluded_b[*node] = true;
        }

        let (mut w0, mut w1, mut strat_w0, mut strat_w1) = solve(game, &excluded_b);

        if player == 0 {
            w1.extend(b);
            merge_strategy(&mut strat_w1, &strat_b);
        } else {
            w0.extend(b);
            merge_strategy(&mut strat_w0, &strat_b);
        }

        (w0, w1, strat_w0, strat_w1)
    }
}

fn pick(game: &ParityGame, max_nodes: &[usize], winning_region: &[usize]) -> Vec<Option<usize>> {
    let mut strategies = vec![None; game.num_nodes()];
    for &node in max_nodes {
        for &successor in game.get_successors(node) {
            if winning_region.contains(&successor) {
                strategies[node] = Some(successor);
                break;
            }
        }
    }
    strategies
}

fn merge_strategy(target: &mut [Option<usize>], source: &[Option<usize>]) {
    for (idx, entry) in source.iter().enumerate() {
        if target[idx].is_none() {
            target[idx] = *entry;
        }
    }
}

pub fn attract(
    game: &ParityGame,
    excluded: &[bool],
    nodes_to_attract: &[usize],
    player: usize,
    strategy: Vec<Option<usize>>,
) -> (Vec<usize>, Vec<Option<usize>>) {
    let nodes = game.num_nodes();

    let out_degree: Vec<AtomicUsize> = (0..nodes)
        .into_par_iter()
        .map(|node| {
            if !excluded[node] && game.get_owner(node) == 1 - player {
                let valid_edges_count = game
                    .get_edges(node)
                    .iter()
                    .filter(|&&target| !excluded[target])
                    .count();
                AtomicUsize::new(valid_edges_count)
            } else {
                AtomicUsize::new(0)
            }
        })
        .collect();

    let in_attractor: Vec<AtomicBool> = (0..nodes).map(|_| AtomicBool::new(false)).collect();

    let atomic_strategy: Vec<AtomicUsize> = strategy
        .into_iter()
        .map(|opt| AtomicUsize::new(opt.unwrap_or(usize::MAX)))
        .collect();

    let mut frontier: Vec<usize> = Vec::new();
    for &node in nodes_to_attract {
        if !excluded[node] && !in_attractor[node].swap(true, Ordering::Relaxed) {
            frontier.push(node);
        }
    }

    while !frontier.is_empty() {
        let next_frontier: Vec<usize> = frontier
            .into_par_iter()
            .flat_map(|current| {
                let mut local_next = Vec::new();

                for &predecessor in game.get_predecessors(current) {
                    if excluded[predecessor] || in_attractor[predecessor].load(Ordering::Relaxed) {
                        continue;
                    }

                    if game.get_owner(predecessor) == player {
                        let _ = atomic_strategy[predecessor].compare_exchange(
                            usize::MAX,
                            current,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        );

                        if !in_attractor[predecessor].swap(true, Ordering::SeqCst) {
                            local_next.push(predecessor);
                        }
                    } else {
                        let mut current_deg = out_degree[predecessor].load(Ordering::Relaxed);
                        loop {
                            if current_deg == 0 {
                                break;
                            }
                            match out_degree[predecessor].compare_exchange_weak(
                                current_deg,
                                current_deg - 1,
                                Ordering::SeqCst,
                                Ordering::Relaxed,
                            ) {
                                Ok(_) => {
                                    if current_deg == 1 && !in_attractor[predecessor].swap(true, Ordering::SeqCst) {
                                        local_next.push(predecessor);
                                    }
                                    break;
                                }
                                Err(actual) => current_deg = actual,
                            }
                        }
                    }
                }
                local_next
            })
            .collect();

        frontier = next_frontier;
    }

    let final_attractor: Vec<usize> = in_attractor
        .into_iter()
        .enumerate()
        .filter_map(|(idx, atom)| if atom.into_inner() { Some(idx) } else { None })
        .collect();

    let final_strategy: Vec<Option<usize>> = atomic_strategy
        .into_iter()
        .map(|atom| {
            let val = atom.into_inner();
            if val == usize::MAX { None } else { Some(val) }
        })
        .collect();

    (final_attractor, final_strategy)
}
