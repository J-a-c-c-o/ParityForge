use crate::parity_game::{ParityGameBuilder, ParityGame};


pub fn parse_pg(input: &str) -> Result<ParityGame, String> {
    let mut builder = ParityGameBuilder::new();

    for line in input.lines().collect::<Vec<_>>()[1..].iter() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.replace(";", "");

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(format!("Invalid line format: '{}'", line));
        }

        
        let node_id: usize = parts[0].parse().map_err(|_| format!("Invalid node ID: '{}'", parts[0]))?;
        let priority: usize = parts[1].parse().map_err(|_| format!("Invalid priority: '{}'", parts[1]))?;
        let owner: usize = parts[2].parse().map_err(|_| format!("Invalid owner: '{}'", parts[2]))?;
        let edges: Vec<usize> = parts[3].split(',').map(|s| s.parse().map_err(|_| format!("Invalid edge target: '{}'", s))).collect::<Result<_, _>>()?;
        let label = if parts.len() > 4 {
            Some(parts[4..].join(" "))
        } else {
            None
        };

        builder.set_priority(node_id, priority);
        builder.set_owner(node_id, owner);
        if let Some(label) = label {
            builder.set_label(node_id, label);
        }
        for target in edges {
            builder.add_edge(node_id, target);
        }
    }

    Ok(builder.build())
}



pub fn strat_to_sol(
    game: &ParityGame,
    strategy0: &[Option<usize>],
    strategy1: &[Option<usize>],
    winning_region0: &[usize],
    winning_region1: &[usize],
) -> String {
    let mut output = String::new();

    output.push_str(&format!("paritysol {}\n", game.get_max_priority()));
    // name of the format: node_id owner strategy_target
    for node in game.get_nodes() {
        // determine owner from winning regions
        let owner = if winning_region0.contains(&node) {
            0
        } else if winning_region1.contains(&node) {
            1
        } else {
            game.get_owner(node)
        };
        let strategy = if owner == 0 {
            strategy0.get(node).and_then(|s| *s)
        } else {
            strategy1.get(node).and_then(|s| *s)
        };
        if let Some(target) = strategy {
            output.push_str(&format!("{} {} {};\n", node, owner, target));
        } else {
            output.push_str(&format!("{} {};\n", node, owner));
        }
    }
    output
}