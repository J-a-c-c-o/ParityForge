use crate::parity_game::ParityGame;

use std::collections::{HashSet, VecDeque};

pub fn run_tl(
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
    tangle_learning(game)
}

fn tangle_learning(
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
    let nodes = game.num_nodes();
    let mut in_game: Vec<bool> = vec![true; nodes];
    let mut winner: Vec<Option<usize>> = vec![None; nodes];
    let mut strat0: Vec<Option<usize>> = vec![None; nodes];
    let mut strat1: Vec<Option<usize>> = vec![None; nodes];
    let mut tangles: Vec<Tangle> = Vec::new();

    while in_game.iter().any(|&x| x) {
        let new_tangles = search(game, &tangles, &in_game);

        let mut dominions = Vec::new();
        let mut learned_new_tangle = false;

        for t in new_tangles {
            if t.escapes.is_empty() {
                dominions.push(t);
            } else {
                let exists = tangles
                    .iter()
                    .any(|existing| existing.player == t.player && existing.nodes == t.nodes);

                if !exists {
                    tangles.push(t);
                    learned_new_tangle = true;
                }
            }
        }

        if dominions.is_empty() {
            if !learned_new_tangle {
                return Err("No new tangles learned and no dominions found, but game is not solved. This should not happen.".to_string());
            }
            continue;
        }

        let escape_map0 = build_escape_map(nodes, &tangles, 0, &in_game);
        let escape_map1 = build_escape_map(nodes, &tangles, 1, &in_game);

        for player in [0usize, 1usize] {
            let dominion_nodes = collect_dominion_nodes(&dominions, player, nodes, &in_game);
            if dominion_nodes.is_empty() {
                continue;
            }

            let mut sigma = vec![None; nodes];
            for t in dominions.iter().filter(|t| t.player == player) {
                merge_strategy(&mut sigma, &t.strategy, player, game);
            }

            let escape_map = if player == 0 {
                &escape_map0
            } else {
                &escape_map1
            };
            let (z_plus, sigma_plus) = tangle_attract(
                game,
                &tangles,
                escape_map,
                &in_game,
                &dominion_nodes,
                player,
                &sigma,
            );

            if player == 0 {
                merge_strategy(&mut strat0, &sigma_plus, 0, game);
            } else {
                merge_strategy(&mut strat1, &sigma_plus, 1, game);
            }

            mark_winner(&mut winner, &mut in_game, &z_plus, player);
        }

        tangles.retain(|t| t.nodes.iter().all(|&v| in_game[v]));
    }

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    for v in 0..nodes {
        match winner[v] {
            Some(0) => w0.push(v),
            Some(1) => w1.push(v),
            _ => {}
        }
    }

    Ok((w0, w1, strat0, strat1))
}

fn search(game: &ParityGame, tangles: &[Tangle], in_game: &[bool]) -> Vec<Tangle> {
    let nodes = game.num_nodes();
    let mut decomposed = vec![false; nodes];
    let mut all_extracted_tangles = Vec::new();

    while in_game
        .iter()
        .zip(&decomposed)
        .any(|(&in_g, &dec)| in_g && !dec)
    {
        let mut max_prio = 0;
        let mut has_nodes = false;
        for v in 0..nodes {
            if in_game[v] && !decomposed[v] {
                let prio = game.get_priority(v);
                if !has_nodes || prio > max_prio {
                    max_prio = prio;
                    has_nodes = true;
                }
            }
        }
        if !has_nodes {
            break;
        }

        let alpha = max_prio % 2;

        let mut targets = Vec::new();
        for v in 0..nodes {
            if in_game[v] && !decomposed[v] && game.get_priority(v) == max_prio {
                targets.push(v);
            }
        }

        let mut in_g_prime = vec![false; nodes];
        for v in 0..nodes {
            in_g_prime[v] = in_game[v] && !decomposed[v];
        }

        let sigma_init = vec![None; nodes];
        let escape_map = build_escape_map(nodes, tangles, alpha, &in_g_prime);
        let (z, sigma) = tangle_attract(
            game,
            tangles,
            &escape_map,
            &in_g_prime,
            &targets,
            alpha,
            &sigma_init,
        );

        let extracted = extract_tangles_from_region(game, &z, &sigma, alpha, &in_g_prime, in_game);

        let mut found_dominion = false;
        for t in &extracted {
            if t.escapes.is_empty() {
                found_dominion = true;
            }
        }

        all_extracted_tangles.extend(extracted);

        if found_dominion {
            break;
        }

        for &v in &z {
            decomposed[v] = true;
        }
    }

    all_extracted_tangles
}

fn extract_tangles_from_region(
    game: &ParityGame,
    z: &[usize],
    sigma: &[Option<usize>],
    alpha: usize,
    in_g_prime: &[bool],
    in_game: &[bool],
) -> Vec<Tangle> {
    let nodes = game.num_nodes();
    let mut in_z = vec![false; nodes];
    for &v in z {
        in_z[v] = true;
    }

    let mut keep = in_z.clone();
    let mut queue = VecDeque::new();

    for &v in z {
        let escapes = if game.get_owner(v) == alpha {
            if let Some(succ) = sigma[v] {
                in_g_prime[succ] && !in_z[succ]
            } else {
                true
            }
        } else {
            game.get_successors(v)
                .iter()
                .any(|&succ| in_g_prime[succ] && !in_z[succ])
        };

        if escapes {
            keep[v] = false;
            queue.push_back(v);
        }
    }

    while let Some(u) = queue.pop_front() {
        for &pred in game.get_predecessors(u) {
            if keep[pred] {
                let now_escapes = if game.get_owner(pred) == alpha {
                    sigma[pred] == Some(u)
                } else {
                    true
                };

                if now_escapes {
                    keep[pred] = false;
                    queue.push_back(pred);
                }
            }
        }
    }

    let sccs = game.bottom_sccs(&keep, sigma);
    let mut tangles = Vec::new();

    for scc in sccs {
        let mut t_strat = vec![None; nodes];
        for &v in &scc {
            if game.get_owner(v) == alpha {
                t_strat[v] = sigma[v];
            }
        }

        let mut in_scc = vec![false; nodes];
        for &v in &scc {
            in_scc[v] = true;
        }

        let mut escapes = HashSet::new();
        for &u in &scc {
            if game.get_owner(u) != alpha {
                for &succ in game.get_successors(u) {
                    if in_game[succ] && !in_scc[succ] {
                        escapes.insert(succ);
                    }
                }
            }
        }

        tangles.push(Tangle {
            nodes: scc,
            player: alpha,
            strategy: t_strat,
            escapes: escapes.into_iter().collect(),
        });
    }

    tangles
}

fn tangle_attract(
    game: &ParityGame,
    tangles: &[Tangle],
    escape_map: &[Vec<usize>],
    in_game: &[bool],
    z_init: &[usize],
    player: usize,
    sigma_init: &[Option<usize>],
) -> (Vec<usize>, Vec<Option<usize>>) {
    let mut in_z = vec![false; game.num_nodes()];
    let mut queue = VecDeque::new();
    let mut sigma = sigma_init.to_vec();

    for &v in z_init {
        if in_game[v] && !in_z[v] {
            in_z[v] = true;
            queue.push_back(v);
        }
    }

    while let Some(v) = queue.pop_front() {
        for &pred in game.get_predecessors(v) {
            if !in_game[pred] {
                continue;
            }

            if !in_z[pred] {
                let can_attract = if game.get_owner(pred) == player {
                    true
                } else {
                    game.get_successors(pred)
                        .iter()
                        .filter(|&&s| in_game[s])
                        .all(|&s| in_z[s])
                };

                if can_attract {
                    in_z[pred] = true;
                    queue.push_back(pred);
                }
            }

            if in_z[pred] && game.get_owner(pred) == player && sigma[pred].is_none() {
                sigma[pred] = Some(v);
            }
        }

        for &tidx in &escape_map[v] {
            let tangle = &tangles[tidx];

            if !tangle.nodes.iter().all(|&u| in_game[u]) {
                continue;
            }

            if !tangle
                .escapes
                .iter()
                .filter(|&&e| in_game[e])
                .all(|&e| in_z[e])
            {
                continue;
            }

            for &u in &tangle.nodes {
                if in_game[u] && !in_z[u] {
                    in_z[u] = true;
                    queue.push_back(u);
                }

                if game.get_owner(u) == player {
                    if let Some(strat_succ) = tangle.strategy[u] {
                        sigma[u] = Some(strat_succ);
                    }
                }
            }
        }
    }

    let mut z = Vec::new();
    for v in 0..game.num_nodes() {
        if in_z[v] {
            z.push(v);
        }
    }

    (z, sigma)
}

fn build_escape_map(
    nodes: usize,
    tangles: &[Tangle],
    player: usize,
    in_region: &[bool],
) -> Vec<Vec<usize>> {
    let mut map = vec![Vec::new(); nodes];
    for (idx, t) in tangles.iter().enumerate() {
        if t.player != player {
            continue;
        }
        for &e in &t.escapes {
            if !in_region[e] {
                continue;
            }
            if e < nodes {
                map[e].push(idx);
            }
        }
    }
    map
}

fn mark_winner(winner: &mut [Option<usize>], in_game: &mut [bool], nodes: &[usize], player: usize) {
    for &v in nodes {
        winner[v] = Some(player);
        in_game[v] = false;
    }
}

fn merge_strategy(
    target: &mut [Option<usize>],
    source: &[Option<usize>],
    player: usize,
    game: &ParityGame,
) {
    for (idx, entry) in source.iter().enumerate() {
        if entry.is_some() && target[idx].is_none() && game.get_owner(idx) == player {
            target[idx] = *entry;
        }
    }
}

fn collect_dominion_nodes(
    dominions: &[Tangle],
    player: usize,
    nodes: usize,
    in_game: &[bool],
) -> Vec<usize> {
    let mut in_set = vec![false; nodes];
    for t in dominions.iter().filter(|t| t.player == player) {
        for &v in &t.nodes {
            in_set[v] = true;
        }
    }

    let mut result = Vec::new();
    for v in 0..nodes {
        if in_set[v] && in_game[v] {
            result.push(v);
        }
    }
    result
}

#[derive(Clone)]
struct Tangle {
    nodes: Vec<usize>,
    player: usize,
    strategy: Vec<Option<usize>>,
    escapes: Vec<usize>,
}
