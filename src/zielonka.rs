use crate::parity_game::ParityGame;
use crate::utils::attract;

use std::collections::HashSet;

pub fn run_zielonka(game: &ParityGame) -> Result<(Vec<usize>, Vec<usize>), String> {
    Ok(solve(game, HashSet::new()))
}

fn solve(game: &ParityGame, excluded: HashSet<usize>) -> (Vec<usize>, Vec<usize>) {
    let max_priority = game.get_nodes().into_iter()
        .filter(|n| !excluded.contains(n))
        .map(|n| game.get_priority(n))
        .max()
        .unwrap_or(0);

    if max_priority == 0 {
        return (game.get_nodes().into_iter().filter(|n| !excluded.contains(n)).collect(), vec![]);
    }

    let nodes_with_max_priority = game.get_nodes_with_priority(max_priority);
    let player = max_priority % 2;

    let attractor = attract(game, &excluded, &nodes_with_max_priority, player);

    let mut new_excluded = excluded.clone();
    for node in &attractor {
        new_excluded.insert(*node);
    }

    let (winning_set_player, winning_set_opponent) = solve(game, new_excluded);

    if player == 0 && winning_set_opponent.is_empty() {
        return (game.get_nodes().into_iter().filter(|n| !excluded.contains(n)).collect(), vec![]);
    } else if player == 1 && winning_set_player.is_empty() {
        return (vec![], game.get_nodes().into_iter().filter(|n| !excluded.contains(n)).collect());
    }

    let mut attractor = vec![];

    if player == 0 {
        attractor = attract(game, &excluded, &winning_set_opponent, 1 - player);
    } else {
        attractor = attract(game, &excluded, &winning_set_player, 1 - player);
    }

    let mut new_excluded = excluded.clone();
    for node in &attractor {
        new_excluded.insert(*node);
    }

    let (mut winning_set_player, mut winning_set_opponent) = solve(game, new_excluded);

    if player == 0 {
        for node in attractor {
            winning_set_opponent.push(node);
        }
    } else {
        for node in attractor {
            winning_set_player.push(node);
        }
    }

    return (winning_set_player, winning_set_opponent);

}

    


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_game::ParityGameBuilder;
    use std::collections::HashSet;


    #[test]
    fn zielonka_self_loops() {
        let mut builder = ParityGameBuilder::new();
        builder.add_edge(0, 0)
            .add_edge(1, 1)
            .set_owner(0, 0)
            .set_owner(1, 0)
            .set_priority(0, 2)
            .set_priority(1, 1);

        let game = builder.build();
        let (p0, p1) = run_zielonka(&game).unwrap();
        let set0: HashSet<usize> = p0.into_iter().collect();
        let set1: HashSet<usize> = p1.into_iter().collect();
        assert_eq!(set0, vec![0].into_iter().collect());
        assert_eq!(set1, vec![1].into_iter().collect());
    }

    #[test]
    fn zielonka_cycle_both_odd() {
        let mut builder = ParityGameBuilder::new();
        builder.add_edge(0, 1)
            .add_edge(1, 0)
            .set_owner(0, 0)
            .set_owner(1, 0)
            .set_priority(0, 1)
            .set_priority(1, 1);

        let game = builder.build();
        let (p0, p1) = run_zielonka(&game).unwrap();
        let set0: HashSet<usize> = p0.into_iter().collect();
        let set1: HashSet<usize> = p1.into_iter().collect();
        assert!(set0.is_empty());
        assert_eq!(set1, vec![0, 1].into_iter().collect());
    }
}
    



