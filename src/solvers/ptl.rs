use crate::parity_game::ParityGame;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct Tangle {
    pub nodes: Vec<usize>,
    pub player: usize,
    pub strategy: Vec<Option<usize>>,
    pub escapes: Vec<usize>,
}

pub fn run_ptl(
    game: &ParityGame,
) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    let n = game.num_nodes();
    let mut in_game = vec![true; n];
    let mut remaining = n;

    let mut w0 = Vec::new();
    let mut w1 = Vec::new();
    let mut strat0 = vec![None; n];
    let mut strat1 = vec![None; n];

    let mut tangles: Vec<Tangle> = Vec::new();

    while remaining > 0 {
        let found = search(game, &tangles, &in_game);

        let mut dominions = Vec::new();
        let mut learned = false;

        for t in found {
            if t.escapes.is_empty() {
                dominions.push(t);
            } else {
                tangles.push(t);
                learned = true;
            }
        }

        if dominions.is_empty() {
            if !learned {
                return Err("No dominions or tangles learned. Game is stuck.".to_string());
            }
            continue;
        }

        for player in 0..=1 {
            let mut dom_targets = Vec::new();
            let mut dom_sigma = vec![None; n];

            for d in &dominions {
                if d.player == player {
                    for &v in &d.nodes {
                        dom_targets.push(v);
                        if game.get_owner(v) == player {
                            dom_sigma[v] = d.strategy[v];
                        }
                    }
                }
            }

            if dom_targets.is_empty() {
                continue;
            }

            let (z_set, sigma) = tangle_attract(
                game,
                &tangles,
                &in_game,      // Global game
                &dom_targets,
                &dom_sigma,
                player,
            );

            for i in 0..n {
                if z_set[i] {
                    in_game[i] = false;
                    remaining -= 1;
                    if player == 0 {
                        w0.push(i);
                        if game.get_owner(i) == 0 && strat0[i].is_none() {
                            strat0[i] = sigma[i];
                        }
                    } else {
                        w1.push(i);
                        if game.get_owner(i) == 1 && strat1[i].is_none() {
                            strat1[i] = sigma[i];
                        }
                    }
                }
            }
        }

        tangles.retain(|t| t.nodes.iter().all(|&v| in_game[v]));
    }

    Ok((w0, w1, strat0, strat1))
}

fn search(game: &ParityGame, tangles: &[Tangle], in_game: &[bool]) -> Vec<Tangle> {
    let n = game.num_nodes();
    let mut decomposed = vec![false; n];
    let mut new_tangles = Vec::new();

    loop {
        let mut max_prio = None;
        for i in 0..n {
            if in_game[i] && !decomposed[i] {
                let p = game.get_priority(i);
                if max_prio.is_none() || p > max_prio.unwrap() {
                    max_prio = Some(p);
                }
            }
        }

        let max_pr = match max_prio {
            Some(p) => p,
            None => break,
        };

        let alpha = max_pr % 2;
        let mut targets = Vec::new();
        let mut in_g_prime = vec![false; n];
        let target_sigma = vec![None; n];

        for i in 0..n {
            if in_game[i] && !decomposed[i] {
                in_g_prime[i] = true;
                if game.get_priority(i) == max_pr {
                    targets.push(i);
                }
            }
        }

        let (z_set, sigma) = tangle_attract(game, tangles, &in_g_prime, &targets, &target_sigma, alpha);
        let extracted = extract_tangles_from_region(game, &z_set, &sigma, alpha, &in_g_prime, in_game);

        let found_dominion = extracted.iter().any(|t| t.escapes.is_empty());
        new_tangles.extend(extracted);

        if found_dominion {
            break;
        }

        for i in 0..n {
            if z_set[i] {
                decomposed[i] = true;
            }
        }
    }

    new_tangles
}

fn tangle_attract(
    game: &ParityGame,
    tangles: &[Tangle],
    in_subgame: &[bool],
    targets: &[usize],
    target_sigma: &[Option<usize>],
    player: usize,
) -> (Vec<bool>, Vec<Option<usize>>) {
    let n = game.num_nodes();
    let mut in_z = vec![false; n];
    let mut sigma = vec![None; n];
    let mut queue = VecDeque::new();

    let mut opp_deg = vec![0; n];
    for v in 0..n {
        if in_subgame[v] && game.get_owner(v) != player {
            opp_deg[v] = game.get_successors(v).iter().filter(|&&s| in_subgame[s]).count();
        }
    }

    let mut tangle_escapes_left = vec![usize::MAX; tangles.len()];
    let mut escape_map = vec![Vec::new(); n];

    for (i, t) in tangles.iter().enumerate() {
        if t.player == player {
            tangle_escapes_left[i] = t.escapes.iter().filter(|&&e| in_subgame[e]).count();
            for &e in &t.escapes {
                if e < n && in_subgame[e] {
                    escape_map[e].push(i);
                }
            }
        }
    }

    for &v in targets {
        if in_subgame[v] {
            in_z[v] = true;
            if game.get_owner(v) == player {
                sigma[v] = target_sigma[v];
            }
            queue.push_back(v);
        }
    }

    while let Some(u) = queue.pop_front() {
        for &v in game.get_predecessors(u) {
            if !in_subgame[v] { continue; }

            if in_z[v] {
                if game.get_owner(v) == player && sigma[v].is_none() {
                    sigma[v] = Some(u);
                }
                continue;
            }

            let owner = game.get_owner(v);
            let can_attract = if owner == player {
                true
            } else {
                opp_deg[v] -= 1;
                opp_deg[v] == 0
            };

            if can_attract {
                in_z[v] = true;
                if owner == player {
                    sigma[v] = Some(u);
                }
                queue.push_back(v);
            }
        }

        for &tidx in &escape_map[u] {
            if tangle_escapes_left[tidx] == usize::MAX || tangle_escapes_left[tidx] == 0 {
                continue;
            }
            tangle_escapes_left[tidx] -= 1;
            
            if tangle_escapes_left[tidx] == 0 {
                let t = &tangles[tidx];
                if t.nodes.iter().all(|&node| in_subgame[node]) {
                    for &node in &t.nodes {
                        if !in_z[node] {
                            in_z[node] = true;
                            if game.get_owner(node) == player {
                                sigma[node] = t.strategy[node];
                            }
                            queue.push_back(node);
                        }
                    }
                }
            }
        }
    }

    (in_z, sigma)
}

fn extract_tangles_from_region(
    game: &ParityGame,
    z_set: &[bool],
    sigma: &[Option<usize>],
    alpha: usize,
    in_g_prime: &[bool],
    in_game: &[bool],
) -> Vec<Tangle> {
    let n = game.num_nodes();
    let mut closed_z = z_set.to_vec();
    let mut queue = VecDeque::new();

    for u in 0..n {
        if !closed_z[u] { continue; }
        
        let leaks = if game.get_owner(u) == alpha {
            sigma[u].is_none() // Alpha failed to loop back
        } else {
            game.get_successors(u).iter().any(|&s| in_g_prime[s] && !closed_z[s])
        };

        if leaks {
            closed_z[u] = false;
            queue.push_back(u);
        }
    }

    while let Some(u) = queue.pop_front() {
        for &v in game.get_predecessors(u) {
            if closed_z[v] {
                let leaks = if game.get_owner(v) == alpha {
                    sigma[v].is_none() || sigma[v] == Some(u)
                } else {
                    true
                };
                if leaks {
                    closed_z[v] = false;
                    queue.push_back(v);
                }
            }
        }
    }

    let sccs = game.tarjan_sccs_restricted(&closed_z, sigma, alpha);
    let mut found = Vec::new();

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

        let mut in_scc = vec![false; n];
        for &u in &scc { in_scc[u] = true; }

        let mut escapes = Vec::new();
        let mut strat = vec![None; n];

        for &u in &scc {
            if game.get_owner(u) == alpha {
                strat[u] = sigma[u];
            } else {
                for &s in game.get_successors(u) {
                    if in_game[s] && !in_scc[s] {
                        escapes.push(s);
                    }
                }
            }
        }

        found.push(Tangle {
            nodes: scc,
            player: alpha,
            strategy: strat,
            escapes,
        });
    }

    found
}