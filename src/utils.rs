use crate::parity_game::ParityGame;

use std::collections::{HashSet, VecDeque};

pub(crate) fn attract(game: &ParityGame, excluded: &HashSet<usize>, nodes_to_attract: &[usize], player: usize) -> Vec<usize> {
    let mut attractor: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();

    for &n in nodes_to_attract {
        if !excluded.contains(&n) && !attractor.contains(&n) {
            attractor.insert(n);
            queue.push_back(n);
        }
    }

    while let Some(_) = queue.pop_front() {
        for u in 0..game.num_nodes() {
            if excluded.contains(&u) || attractor.contains(&u) {
                continue;
            }

            let succs = game.get_successors(u);

            let mut has_edge_to_attractor = false;
            for &s in succs {
                if attractor.contains(&s) {
                    has_edge_to_attractor = true;
                    break;
                }
            }

            if !has_edge_to_attractor {
                continue;
            }

            let owner = game.get_owner(u);
            if owner == player {
                attractor.insert(u);
                queue.push_back(u);
            } else {
                let mut all_in = true;
                for &s in succs {
                    if excluded.contains(&s) {
                        all_in = false;
                        break;
                    }
                    if !attractor.contains(&s) {
                        all_in = false;
                        break;
                    }
                }
                if all_in {
                    attractor.insert(u);
                    queue.push_back(u);
                }
            }
        }
    }

    let mut res: Vec<usize> = attractor.into_iter().collect();
    res.sort_unstable();
    res
}