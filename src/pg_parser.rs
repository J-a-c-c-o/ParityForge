use crate::parity_game::{ParityGameBuilder, ParityGame};


pub fn parse_pg(input: &str) -> Result<ParityGame, String> {
    let mut builder = ParityGameBuilder::new();

    for line in input.lines().collect::<Vec<_>>()[1..].iter() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

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

