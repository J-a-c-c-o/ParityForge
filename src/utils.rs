use crate::parity_game::ParityGame;

use std::collections::{HashSet, VecDeque};

pub(crate) fn attract(
    game: &ParityGame, 
    excluded: &[bool], 
    nodes_to_attract: &[usize], 
    player: usize
) -> (Vec<usize>, Vec<(usize, usize)>) {
    let mut attractor = HashSet::new();
    let mut queue = VecDeque::new();
    let mut strategy = Vec::new();

    let mut out_degree = vec![0; game.num_nodes()];

    for node in game.get_nodes() {
        if !excluded[node] && game.get_owner(node) == 1 - player {
            let valid_edges_count = game.get_edges(node)
                .iter()
                .filter(|target| !excluded[**target])
                .count();
            out_degree[node] = valid_edges_count;
        }
    }

    for &node in nodes_to_attract {
        if !excluded[node] {
            if attractor.insert(node) {
                queue.push_back(node);
            }
        }
    }

    while let Some(current) = queue.pop_front() {
        for &predecessor in game.get_predecessors(current) {
            if excluded[predecessor] || attractor.contains(&predecessor) {
                continue;
            }

            if game.get_owner(predecessor) == player {
                strategy.push((predecessor, current));
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