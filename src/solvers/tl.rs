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
    Ok(tangle_learning(game))
}

fn tangle_learning(
    game: &ParityGame,
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    let nodes = game.num_nodes();
    let mut in_game = vec![true; nodes];
    let mut winner: Vec<Option<usize>> = vec![None; nodes];
    let mut strat0 = vec![None; nodes];
    let mut strat1 = vec![None; nodes];
    let mut tangles: Vec<Tangle> = Vec::new();

    while in_game.iter().any(|&x| x) {
        let new_tangles = search(game, &tangles, &in_game);

        if new_tangles.is_empty() {
            let (rw0, rw1, rs0, rs1) = zielonka_fallback(game, &in_game);
            mark_winner(&mut winner, &mut in_game, &rw0, 0);
            mark_winner(&mut winner, &mut in_game, &rw1, 1);
            merge_strategy(&mut strat0, &rs0, 0, game);
            merge_strategy(&mut strat1, &rs1, 1, game);
            break;
        }

        let mut dominions = Vec::new();
        for t in new_tangles {
            if t.escapes.is_empty() {
                dominions.push(t);
            } else {
                tangles.push(t);
            }
        }

        if dominions.is_empty() {
            let (rw0, rw1, rs0, rs1) = zielonka_fallback(game, &in_game);
            mark_winner(&mut winner, &mut in_game, &rw0, 0);
            mark_winner(&mut winner, &mut in_game, &rw1, 1);
            merge_strategy(&mut strat0, &rs0, 0, game);
            merge_strategy(&mut strat1, &rs1, 1, game);
            break;
        }

        let escape_map0 = build_escape_map(nodes, &tangles, 0);
        let escape_map1 = build_escape_map(nodes, &tangles, 1);

        for player in [0usize, 1usize] {
            let dominion_nodes = collect_dominion_nodes(&dominions, player, nodes);
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

    (w0, w1, strat0, strat1)
}

fn search(game: &ParityGame, tangles: &[Tangle], in_game: &[bool]) -> Vec<Tangle> {
    if !in_game.iter().any(|&x| x) {
        return Vec::new();
    }

    let (p, alpha) = highest_priority(game, in_game);
    let targets = nodes_with_priority(game, in_game, p);

    let sigma_init = vec![None; game.num_nodes()];
    let escape_map = build_escape_map(game.num_nodes(), tangles, alpha);
    let (z, sigma) = tangle_attract(
        game,
        tangles,
        &escape_map,
        in_game,
        &targets,
        alpha,
        &sigma_init,
    );

    if z.is_empty() {
        return Vec::new();
    }

    let mut in_z = vec![false; game.num_nodes()];
    for &v in &z {
        in_z[v] = true;
    }

    let mut closed = true;
    for &v in &z {
        if game.get_owner(v) == alpha {
            if let Some(succ) = sigma[v] {
                if !in_z[succ] {
                    closed = false;
                    break;
                }
            } else {
                closed = false;
                break;
            }
        }
    }

    let mut next_game = in_game.to_vec();
    for &v in &z {
        next_game[v] = false;
    }

    if closed {
        let mut result = search(game, tangles, &next_game);
        let mut bottoms = bottom_sccs(game, &z, &sigma, alpha);
        result.append(&mut bottoms);
        result
    } else {
        search(game, tangles, &next_game)
    }
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
            if !in_game[pred] || in_z[pred] {
                continue;
            }

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
                if game.get_owner(pred) == player && sigma[pred].is_none() {
                    sigma[pred] = Some(v);
                }
            }
        }

        for &tidx in &escape_map[v] {
            let tangle = &tangles[tidx];
            if !tangle.nodes.iter().all(|&u| in_game[u] || in_z[u]) {
                continue;
            }
            if !tangle.escapes.iter().all(|&e| in_z[e]) {
                continue;
            }

            for &u in &tangle.nodes {
                if in_game[u] && !in_z[u] {
                    in_z[u] = true;
                    queue.push_back(u);
                }
            }

            for (idx, entry) in tangle.strategy.iter().enumerate() {
                if in_z[idx]
                    && entry.is_some()
                    && sigma[idx].is_none()
                    && game.get_owner(idx) == player
                {
                    sigma[idx] = *entry;
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

fn bottom_sccs(
    game: &ParityGame,
    region: &[usize],
    sigma: &[Option<usize>],
    player: usize,
) -> Vec<Tangle> {
    let mut in_region = vec![false; game.num_nodes()];
    for &v in region {
        in_region[v] = true;
    }

    let sccs = game.bottom_sccs(&in_region, sigma, player);
    let mut tangles = Vec::new();

    for scc in sccs {
        let mut strategy = vec![None; game.num_nodes()];
        for &v in &scc {
            if game.get_owner(v) == player {
                strategy[v] = sigma[v];
            }
        }

        let escapes = compute_escapes(game, &scc, player, &strategy);

        tangles.push(Tangle {
            nodes: scc,
            player,
            strategy,
            escapes,
        });
    }

    tangles
}

fn compute_escapes(
    game: &ParityGame,
    nodes: &[usize],
    player: usize,
    strategy: &[Option<usize>],
) -> Vec<usize> {
    let mut in_nodes = vec![false; game.num_nodes()];
    for &v in nodes {
        in_nodes[v] = true;
    }

    let mut escapes = HashSet::new();
    for &v in nodes {
        if game.get_owner(v) == player {
            if let Some(succ) = strategy[v]
                && !in_nodes[succ]
            {
                escapes.insert(succ);
            }
        } else {
            for &succ in game.get_successors(v) {
                if !in_nodes[succ] {
                    escapes.insert(succ);
                }
            }
        }
    }

    escapes.into_iter().collect()
}

fn build_escape_map(nodes: usize, tangles: &[Tangle], player: usize) -> Vec<Vec<usize>> {
    let mut map = vec![Vec::new(); nodes];
    for (idx, t) in tangles.iter().enumerate() {
        if t.player != player {
            continue;
        }
        for &e in &t.escapes {
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

fn collect_dominion_nodes(dominions: &[Tangle], player: usize, nodes: usize) -> Vec<usize> {
    let mut in_set = vec![false; nodes];
    for t in dominions.iter().filter(|t| t.player == player) {
        for &v in &t.nodes {
            in_set[v] = true;
        }
    }

    let mut result = Vec::new();
    for v in 0..nodes {
        if in_set[v] {
            result.push(v);
        }
    }
    result
}

fn highest_priority(game: &ParityGame, in_game: &[bool]) -> (usize, usize) {
    let max_prio = game
        .get_nodes()
        .into_iter()
        .filter(|&v| in_game[v])
        .map(|v| game.get_priority(v))
        .max()
        .unwrap_or(0);

    (max_prio, max_prio % 2)
}

fn nodes_with_priority(game: &ParityGame, in_game: &[bool], prio: usize) -> Vec<usize> {
    game.get_nodes_with_priority(prio)
        .into_iter()
        .filter(|&v| in_game[v])
        .collect()
}

fn zielonka_fallback(
    game: &ParityGame,
    in_game: &[bool],
) -> (
    Vec<usize>,
    Vec<usize>,
    Vec<Option<usize>>,
    Vec<Option<usize>>,
) {
    let mut excluded = vec![true; game.num_nodes()];
    for v in 0..game.num_nodes() {
        if in_game[v] {
            excluded[v] = false;
        }
    }

    let (w0, w1, s0, s1) = crate::solvers::zielonka::zielonka_solve(game, &excluded);
    (w0, w1, s0, s1)
}

#[derive(Clone)]
struct Tangle {
    nodes: Vec<usize>,
    player: usize,
    strategy: Vec<Option<usize>>,
    escapes: Vec<usize>,
}
