use crate::parity_game::ParityGame;
use crate::utils::attract;
use crate::zielonka::zielonka_solve;

pub fn run_tl(
    game: &ParityGame,
) -> Result<(Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>), String> {
    Ok(solve(game))
}

fn solve(game: &ParityGame) -> (Vec<usize>, Vec<usize>, Vec<Option<usize>>, Vec<Option<usize>>) {
    let nodes = game.num_nodes();
    let mut excluded = vec![false; nodes];
    let mut winner: Vec<Option<usize>> = vec![None; nodes];
    let mut strat0: Vec<Option<usize>> = vec![None; nodes];
    let mut strat1: Vec<Option<usize>> = vec![None; nodes];

    while excluded.iter().any(|&x| !x) {
        let mut tangles = find_closed_tangles(game, &excluded);

        if tangles.is_empty() {
            let (rw0, rw1, rs0, rs1) = zielonka_solve(game, &excluded);
            mark_winner(&mut winner, &mut excluded, &rw0, 0);
            mark_winner(&mut winner, &mut excluded, &rw1, 1);
            apply_strategies(game, &mut strat0, &rs0, 0);
            apply_strategies(game, &mut strat1, &rs1, 1);
            break;
        }

        tangles.sort_by(|a, b| b.max_priority.cmp(&a.max_priority));
        let tangle = tangles.remove(0);

        let (attr, strat_attr) = attract(game, &excluded, &tangle.nodes, tangle.player);

        if tangle.player == 0 {
            apply_strategies(game, &mut strat0, &tangle.strategy, 0);
            apply_strategies(game, &mut strat0, &strat_attr, 0);
        } else {
            apply_strategies(game, &mut strat1, &tangle.strategy, 1);
            apply_strategies(game, &mut strat1, &strat_attr, 1);
        }

        mark_winner(&mut winner, &mut excluded, &attr, tangle.player);
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

fn mark_winner(winner: &mut [Option<usize>], excluded: &mut [bool], nodes: &[usize], player: usize) {
    for &v in nodes {
        winner[v] = Some(player);
        excluded[v] = true;
    }
}

fn apply_strategies(
    game: &ParityGame,
    strategy_map: &mut [Option<usize>],
    strategies: &[Option<usize>],
    player: usize,
) {
    for (idx, entry) in strategies.iter().enumerate() {
        if entry.is_some() && strategy_map[idx].is_none() && game.get_owner(idx) == player {
            strategy_map[idx] = *entry;
        }
    }
}

struct Tangle {
    nodes: Vec<usize>,
    player: usize,
    max_priority: usize,
    strategy: Vec<Option<usize>>,
}

fn find_closed_tangles(game: &ParityGame, excluded: &[bool]) -> Vec<Tangle> {
    let sccs = compute_sccs(game, excluded);
    let mut tangles = Vec::new();

    for scc in sccs {
        if scc.is_empty() {
            continue;
        }

        if !is_nontrivial_scc(game, &scc) {
            continue;
        }

        let max_priority = scc
            .iter()
            .map(|&v| game.get_priority(v))
            .max()
            .unwrap_or(0);
        let player = max_priority % 2;

        let mut in_scc = vec![false; game.num_nodes()];
        for &v in &scc {
            in_scc[v] = true;
        }

        if !is_closed_tangle(game, &in_scc, player) {
            continue;
        }

        let mut excluded_sub = excluded.to_vec();
        for v in 0..game.num_nodes() {
            if !in_scc[v] {
                excluded_sub[v] = true;
            }
        }

        let (w0, w1, rs0, rs1) = zielonka_solve(game, &excluded_sub);
        if player == 0 {
            if !w1.is_empty() {
                continue;
            }
        } else if !w0.is_empty() {
            continue;
        }

        let strategy = if player == 0 { rs0 } else { rs1 };

        tangles.push(Tangle {
            nodes: scc,
            player,
            max_priority,
            strategy,
        });
    }

    tangles
}

fn is_closed_tangle(game: &ParityGame, in_scc: &[bool], player: usize) -> bool {
    for v in 0..game.num_nodes() {
        if !in_scc[v] {
            continue;
        }

        if game.get_owner(v) == player {
            if !game.get_successors(v).iter().any(|&s| in_scc[s]) {
                return false;
            }
        } else {
            if game.get_successors(v).iter().any(|&s| !in_scc[s]) {
                return false;
            }
        }
    }
    true
}

fn is_nontrivial_scc(game: &ParityGame, scc: &[usize]) -> bool {
    if scc.len() > 1 {
        return true;
    }
    if let Some(&v) = scc.first() {
        return game.get_successors(v).iter().any(|&s| s == v);
    }
    false
}

fn compute_sccs(game: &ParityGame, excluded: &[bool]) -> Vec<Vec<usize>> {
    let mut visited = vec![false; game.num_nodes()];
    let mut order = Vec::new();

    for v in 0..game.num_nodes() {
        if excluded[v] || visited[v] {
            continue;
        }
        dfs_finish_order(game, excluded, v, &mut visited, &mut order);
    }

    let mut visited_rev = vec![false; game.num_nodes()];
    let mut sccs = Vec::new();

    while let Some(v) = order.pop() {
        if excluded[v] || visited_rev[v] {
            continue;
        }
        let mut component = Vec::new();
        dfs_collect_scc(game, excluded, v, &mut visited_rev, &mut component);
        sccs.push(component);
    }

    sccs
}

fn dfs_finish_order(
    game: &ParityGame,
    excluded: &[bool],
    start: usize,
    visited: &mut [bool],
    order: &mut Vec<usize>,
) {
    let mut stack = vec![(start, 0usize)];
    visited[start] = true;

    while let Some((node, idx)) = stack.pop() {
        let successors = game.get_successors(node);
        if idx < successors.len() {
            let next = successors[idx];
            stack.push((node, idx + 1));
            if !excluded[next] && !visited[next] {
                visited[next] = true;
                stack.push((next, 0));
            }
        } else {
            order.push(node);
        }
    }
}

fn dfs_collect_scc(
    game: &ParityGame,
    excluded: &[bool],
    start: usize,
    visited: &mut [bool],
    component: &mut Vec<usize>,
) {
    let mut stack = vec![start];
    visited[start] = true;

    while let Some(node) = stack.pop() {
        component.push(node);
        for &pred in game.get_predecessors(node) {
            if excluded[pred] || visited[pred] {
                continue;
            }
            visited[pred] = true;
            stack.push(pred);
        }
    }
}
