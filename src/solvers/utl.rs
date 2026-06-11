use crate::parity_game::ParityGame;

use std::collections::VecDeque;

#[derive(Clone)]
pub struct Tangle {
    pub nodes: Vec<usize>,
    pub player: usize,
    pub strategy: Vec<Option<usize>>,
    pub escapes: Vec<usize>,
}

pub fn run_utl(
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
    let mut in_game = vec![true; nodes];
    let mut remaining_nodes = nodes;

    let mut winner: Vec<Option<usize>> = vec![None; nodes];
    let mut strat0: Vec<Option<usize>> = vec![None; nodes];
    let mut strat1: Vec<Option<usize>> = vec![None; nodes];
    let mut tangles: Vec<Tangle> = Vec::new();

    while remaining_nodes > 0 {
        let new_tangles = search(game, &tangles, &in_game);

        let mut dominions = Vec::new();
        let mut learned_new_tangle = false;

        for t in new_tangles {
            if t.escapes.is_empty() {
                dominions.push(t);
            } else {
                tangles.push(t);
                learned_new_tangle = true;
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
                for &v in &t.nodes {
                    if sigma[v].is_none() && game.get_owner(v) == player {
                        sigma[v] = t.strategy[v];
                    }
                }
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
                merge_strategy_from_z(&mut strat0, &sigma_plus, &z_plus, 0, game);
            } else {
                merge_strategy_from_z(&mut strat1, &sigma_plus, &z_plus, 1, game);
            }

            remaining_nodes -= mark_winner(&mut winner, &mut in_game, &z_plus, player);
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
    let mut remaining_to_decompose = in_game.iter().filter(|&&b| b).count();
    let mut all_extracted_tangles = Vec::new();

    while remaining_to_decompose > 0 {
        let mut max_prio = None;
        for v in 0..nodes {
            if in_game[v] && !decomposed[v] {
                let prio = game.get_priority(v);
                if max_prio.is_none() || prio > max_prio.unwrap() {
                    max_prio = Some(prio);
                }
            }
        }
        let max_prio = max_prio.expect("There should be at least one node left to decompose, but none found.");

        let alpha = max_prio % 2;

        let mut targets = Vec::new();
        let mut in_g_prime = vec![false; nodes];
        for v in 0..nodes {
            let active = in_game[v] && !decomposed[v];
            in_g_prime[v] = active;
            if active && game.get_priority(v) == max_prio {
                targets.push(v);
            }
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
        let found_dominion = extracted.iter().any(|t| t.escapes.is_empty());

        all_extracted_tangles.extend(extracted);

        if found_dominion {
            break;
        }

        for &v in &z {
            if !decomposed[v] {
                decomposed[v] = true;
                remaining_to_decompose -= 1;
            }
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
    let mut z_set = vec![false; nodes];
    for &v in z { z_set[v] = true; }

    let mut reduced_z = z_set.clone();
    let mut queue = VecDeque::new();
    
    for &v in z {
        let escapes = if game.get_owner(v) == alpha {
            sigma[v].map_or(true, |succ| in_g_prime[succ] && !z_set[succ])
        } else {
            game.get_successors(v).iter().any(|&succ| in_g_prime[succ] && !z_set[succ])
        };

        if escapes {
            reduced_z[v] = false;
            queue.push_back(v);
        }
    }

    while let Some(u) = queue.pop_front() {
        for &v in game.get_predecessors(u) {
            if reduced_z[v] {
                let attracted = if game.get_owner(v) == alpha {
                    sigma[v] == Some(u)
                } else {
                    game.get_successors(v).iter().all(|&s| !in_g_prime[s] || !z_set[s] || !reduced_z[s] || s == u)
                };
                if attracted {
                    reduced_z[v] = false;
                    queue.push_back(v);
                }
            }
        }
    }

    let sccs = game.tarjan_sccs_restricted(&reduced_z, sigma, alpha);
    let mut found_tangles = Vec::new();

    for scc in sccs {
        let is_nontrivial = scc.len() > 1 || {
            let u = scc[0];
            if game.get_owner(u) == alpha {
                sigma[u] == Some(u)
            } else {
                game.get_successors(u).contains(&u)
            }
        };
        if !is_nontrivial { continue; }

        let mut t_strat = vec![None; nodes];
        let mut escapes = Vec::new();
        for &u in &scc {
            if game.get_owner(u) == alpha {
                t_strat[u] = sigma[u];
            } else {
                for &s in game.get_successors(u) {
                    if in_game[s] && !scc.contains(&s) {
                        escapes.push(s);
                    }
                }
            }
        }
        
        found_tangles.push(Tangle {
            nodes: scc,
            player: alpha,
            strategy: t_strat,
            escapes,
        });
    }

    found_tangles
}

fn tangle_attract(
    game: &ParityGame,
    tangles: &[Tangle],
    escape_map: &[Vec<usize>],
    in_game: &[bool],
    in_z: &[usize],
    player: usize,
    sigma_init: &[Option<usize>],
) -> (Vec<usize>, Vec<Option<usize>>) {
    let num_nodes = game.num_nodes();
    let mut queue = VecDeque::new();
    let mut sigma = sigma_init.to_vec();
    let mut final_in_z = vec![false; num_nodes];

    let mut opp_deg = vec![0; num_nodes];
    for v in 0..num_nodes {
        if in_game[v] && game.get_owner(v) != player {
            opp_deg[v] = game
                .get_successors(v)
                .iter()
                .filter(|&&s| in_game[s])
                .count();
        }
    }

    let mut tangle_escapes_left: Vec<usize> = tangles
        .iter()
        .map(|t| t.escapes.iter().filter(|&&e| in_game[e]).count())
        .collect();

    for &v in in_z {
        if in_game[v] {
            final_in_z[v] = true;
            queue.push_back(v);
        }
    }

    while let Some(v) = queue.pop_front() {
        for &pred in game.get_predecessors(v) {
            if !in_game[pred] {
                continue;
            }

            if !final_in_z[pred] {
                let can_attract = if game.get_owner(pred) == player {
                    true
                } else {
                    opp_deg[pred] -= 1;
                    opp_deg[pred] == 0
                };

                if can_attract {
                    final_in_z[pred] = true;
                    queue.push_back(pred);
                }
            }

            if final_in_z[pred] && game.get_owner(pred) == player && sigma[pred].is_none() {
                sigma[pred] = Some(v);
            }
        }

        for &tidx in &escape_map[v] {
            if tangle_escapes_left[tidx] == 0 {
                continue;
            }
            tangle_escapes_left[tidx] -= 1;

            if tangle_escapes_left[tidx] == 0 {
                let tangle = &tangles[tidx];
                if tangle.nodes.iter().all(|&u| in_game[u]) {
                    for &u in &tangle.nodes {
                        if in_game[u] {
                            if !final_in_z[u] {
                                final_in_z[u] = true;
                                queue.push_back(u);
                            }
                            if game.get_owner(u) == player
                                && let Some(strat_succ) = tangle.strategy[u]
                            {
                                sigma[u] = Some(strat_succ);
                            }
                        }
                    }
                }
            }
        }
    }

    let z = (0..num_nodes).filter(|&v| final_in_z[v]).collect();
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
            if e < nodes && in_region[e] {
                map[e].push(idx);
            }
        }
    }
    map
}

fn mark_winner(
    winner: &mut [Option<usize>],
    in_game: &mut [bool],
    nodes: &[usize],
    player: usize,
) -> usize {
    let mut removed = 0;
    for &v in nodes {
        if in_game[v] {
            winner[v] = Some(player);
            in_game[v] = false;
            removed += 1;
        }
    }
    removed
}

fn merge_strategy_from_z(
    target: &mut [Option<usize>],
    source: &[Option<usize>],
    z: &[usize],
    player: usize,
    game: &ParityGame,
) {
    for &idx in z {
        if let Some(entry) = source[idx]
            && target[idx].is_none()
            && game.get_owner(idx) == player
        {
            target[idx] = Some(entry);
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
