use crate::parity_game::{ParityGame, ParityGameBuilder};

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

        let node_id: usize = parts[0]
            .parse()
            .map_err(|_| format!("Invalid node ID: '{}'", parts[0]))?;
        let priority: usize = parts[1]
            .parse()
            .map_err(|_| format!("Invalid priority: '{}'", parts[1]))?;
        let owner: usize = parts[2]
            .parse()
            .map_err(|_| format!("Invalid owner: '{}'", parts[2]))?;
        let edges: Vec<usize> = parts[3]
            .split(',')
            .map(|s| {
                s.parse()
                    .map_err(|_| format!("Invalid edge target: '{}'", s))
            })
            .collect::<Result<_, _>>()?;
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

pub fn unparse_pg(game: &ParityGame) -> String {
    let mut output = String::new();
    output.push_str(&format!("parity {};\n", game.num_nodes()));
    for node in game.get_nodes() {
        let priority = game.get_priority(node);
        let owner = game.get_owner(node);
        let edges = game
            .get_edges(node)
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let label = game
            .get_label(node)
            .map_or(String::new(), |l| format!(" {}", l));
        output.push_str(&format!(
            "{} {} {} {}{};\n",
            node, priority, owner, edges, label
        ));
    }
    output
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

pub fn sol_to_strat(
    input: &str,
) -> Result<
    (
        Vec<usize>,
        Vec<usize>,
        Vec<Option<usize>>,
        Vec<Option<usize>>,
    ),
    String,
> {
    let mut winning_region0 = Vec::new();
    let mut winning_region1 = Vec::new();
    let mut strategy0 = Vec::new();
    let mut strategy1 = Vec::new();

    for line in input.lines().collect::<Vec<&str>>()[1..].iter() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.replace(";", "");

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(format!("Invalid line format: '{}'", line));
        }

        let node_id: usize = parts[0]
            .parse()
            .map_err(|_| format!("Invalid node ID: '{}'", parts[0]))?;
        let owner: usize = parts[1]
            .parse()
            .map_err(|_| format!("Invalid owner: '{}'", parts[1]))?;

        if strategy0.len() <= node_id {
            strategy0.resize(node_id + 1, None);
        }
        if strategy1.len() <= node_id {
            strategy1.resize(node_id + 1, None);
        }

        if owner == 0 {
            winning_region0.push(node_id);
            if parts.len() > 2 {
                let target: usize = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid strategy target: '{}'", parts[2]))?;
                strategy0[node_id] = Some(target);
            } else {
                strategy0[node_id] = None;
            }
        } else if owner == 1 {
            winning_region1.push(node_id);
            if parts.len() > 2 {
                let target: usize = parts[2]
                    .parse()
                    .map_err(|_| format!("Invalid strategy target: '{}'", parts[2]))?;
                strategy1[node_id] = Some(target);
            } else {
                strategy1[node_id] = None;
            }
        } else {
            return Err(format!("Invalid owner value: '{}'", owner));
        }
    }

    Ok((winning_region0, winning_region1, strategy0, strategy1))
}
