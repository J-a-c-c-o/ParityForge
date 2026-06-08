use crate::parity_game::ParityGame;

pub fn verify_solution(
    game: &ParityGame,
    w0: &[usize],
    w1: &[usize],
    strat0: &[Option<usize>],
    strat1: &[Option<usize>],
) -> Result<(), String> {
    let node_count = game.num_nodes();
    if strat0.len() != node_count || strat1.len() != node_count {
        return Err(format!(
            "Strategy vectors must have length {} (got {} and {})",
            node_count,
            strat0.len(),
            strat1.len()
        ));
    }

    let mut in_w0 = vec![false; node_count];
    let mut in_w1 = vec![false; node_count];

    for &v in w0 {
        if v >= node_count {
            return Err(format!("Winning set W0 contains out-of-bounds node {}", v));
        }
        if in_w0[v] {
            return Err(format!("Winning set W0 contains duplicate node {}", v));
        }
        in_w0[v] = true;
    }

    for &v in w1 {
        if v >= node_count {
            return Err(format!("Winning set W1 contains out-of-bounds node {}", v));
        }
        if in_w1[v] {
            return Err(format!("Winning set W1 contains duplicate node {}", v));
        }
        in_w1[v] = true;
    }

    for v in 0..node_count {
        if in_w0[v] && in_w1[v] {
            return Err(format!("Node {} cannot be in both winning sets", v));
        }
        if !in_w0[v] && !in_w1[v] {
            return Err(format!("Node {} is missing from both winning sets", v));
        }
    }

    validate_player_witness(game, &in_w0, strat0, 0)?;
    validate_player_witness(game, &in_w1, strat1, 1)?;

    let bottoms0 = game.bottom_sccs(&in_w0, strat0);
    let bottoms1 = game.bottom_sccs(&in_w1, strat1);

    for scc in bottoms0 {
        let max_prio = scc.iter().map(|&v| game.get_priority(v)).max().unwrap_or(0);
        if max_prio % 2 == 1 {
            return Err(format!(
                "Bottom SCC {:?} in W0 has odd max priority {}",
                scc, max_prio
            ));
        }
    }

    for scc in bottoms1 {
        let max_prio = scc.iter().map(|&v| game.get_priority(v)).max().unwrap_or(0);
        if max_prio % 2 == 0 {
            return Err(format!(
                "Bottom SCC {:?} in W1 has even max priority {}",
                scc, max_prio
            ));
        }
    }

    Ok(())
}

fn validate_player_witness(
    game: &ParityGame,
    in_region: &[bool],
    strategy: &[Option<usize>],
    player: usize,
) -> Result<(), String> {
    for v in 0..game.num_nodes() {
        if !in_region[v] || game.get_owner(v) != player {
            continue;
        }

        let succ = strategy[v]
            .ok_or_else(|| format!("Player {}'s strategy is not defined at node {}", player, v))?;

        if !game.get_successors(v).contains(&succ) {
            return Err(format!(
                "Player {}'s strategy at node {} leads to invalid successor {}",
                player, v, succ
            ));
        }

        if !in_region[succ] {
            return Err(format!(
                "Player {}'s strategy at node {} leaves the winning region",
                player, v
            ));
        }
    }

    for v in 0..game.num_nodes() {
        if !in_region[v] || game.get_owner(v) == player {
            continue;
        }

        for &succ in game.get_successors(v) {
            if !in_region[succ] {
                return Err(format!(
                    "Opponent vertex {} has successor {} outside winning region",
                    v, succ
                ));
            }
        }
    }

    Ok(())
}
