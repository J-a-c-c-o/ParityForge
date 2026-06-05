use crate::parity_game::ParityGame;
use std::collections::{HashSet, VecDeque};

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

pub fn zielonka_solve(
    game: &ParityGame,
    excluded: &[bool],
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    solve(game, excluded)
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

fn attract(
    game: &ParityGame,
    excluded: &[bool],
    nodes_to_attract: &[usize],
    player: usize,
    mut strategy: Vec<Option<usize>>,
) -> (Vec<usize>, Vec<Option<usize>>) {
    let mut attractor = HashSet::new();
    let mut queue = VecDeque::new();

    let mut out_degree = vec![0; game.num_nodes()];

    for node in game.get_nodes() {
        if !excluded[node] && game.get_owner(node) == 1 - player {
            let valid_edges_count = game
                .get_edges(node)
                .iter()
                .filter(|target| !excluded[**target])
                .count();
            out_degree[node] = valid_edges_count;
        }
    }

    for &node in nodes_to_attract {
        if !excluded[node] && attractor.insert(node) {
            queue.push_back(node);
        }
    }

    while let Some(current) = queue.pop_front() {
        for &predecessor in game.get_predecessors(current) {
            if excluded[predecessor] || attractor.contains(&predecessor) {
                continue;
            }

            if game.get_owner(predecessor) == player {
                if strategy[predecessor].is_none() {
                    strategy[predecessor] = Some(current);
                }
                attractor.insert(predecessor);
                queue.push_back(predecessor);
            } else {
                if out_degree[predecessor] > 0 {
                    out_degree[predecessor] -= 1;
                    if out_degree[predecessor] == 0 {
                        attractor.insert(predecessor);
                        queue.push_back(predecessor);
                    }
                }
            }
        }
    }

    (attractor.into_iter().collect(), strategy)
}

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
    fn test_zielonka() {
        let game = example_game();
        let (w0, w1, strat0, strat1) = solve(&game, &vec![false; game.num_nodes()]);
        println!("Winning set for player 0: {:?}", w0);
        println!("Winning set for player 1: {:?}", w1);
        println!("Strategy for player 0: {:?}", strat0);
        println!("Strategy for player 1: {:?}", strat1);

        panic!();
    }
}
