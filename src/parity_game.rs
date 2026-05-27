use std::cmp::max;

pub struct ParityGame {
    nodes: usize,
    successors: Vec<Vec<usize>>,
    predecessors: Vec<Vec<usize>>,
    priorities: Vec<usize>,
    owners: Vec<usize>,
    labels: Vec<Option<String>>,
}

#[allow(dead_code)]
impl ParityGame {
    pub fn new(nodes: usize) -> Self {
        ParityGame {
            nodes,
            successors: vec![Vec::new(); nodes],
            predecessors: vec![Vec::new(); nodes],
            priorities: vec![0; nodes],
            owners: vec![0; nodes],
            labels: vec![None; nodes],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.successors[from].push(to);
        self.predecessors[to].push(from);
    }

    fn set_priority(&mut self, node: usize, priority: usize) {
        self.priorities[node] = priority;
    }

    fn set_owner(&mut self, node: usize, owner: usize) {
        self.owners[node] = owner;
    }

    fn set_label(&mut self, node: usize, label: String) {
        self.labels[node] = Some(label);
    }

    pub fn get_edges(&self, node: usize) -> &[usize] {
        &self.successors[node]
    }

    pub fn get_successors(&self, node: usize) -> &[usize] {
        &self.successors[node]
    }

    pub fn get_predecessors(&self, node: usize) -> &[usize] {
        &self.predecessors[node]
    }

    pub fn get_priority(&self, node: usize) -> usize {
        self.priorities[node]
    }

    pub fn get_priorities(&self) -> &[usize] {
        &self.priorities
    }

    pub fn get_owner(&self, node: usize) -> usize {
        self.owners[node]
    }

    pub fn get_label(&self, node: usize) -> Option<&String> {
        self.labels[node].as_ref()
    }

    pub fn get_nodes_with_priority(&self, priority: usize) -> Vec<usize> {
        self.priorities.iter().enumerate().filter_map(|(node, &p)| if p == priority { Some(node) } else { None }).collect()
    }

    pub fn get_nodes(&self) -> Vec<usize> {
        (0..self.nodes).collect()
    }

    pub fn get_max_priority(&self) -> usize {
        *self.priorities.iter().max().unwrap_or(&0)
    }

    pub fn num_nodes(&self) -> usize {
        self.nodes
    }
} 

impl Clone for ParityGame {
    fn clone(&self) -> Self {
        ParityGame {
            nodes: self.nodes,
            successors: self.successors.clone(),
            priorities: self.priorities.clone(),
            owners: self.owners.clone(),
            labels: self.labels.clone(),
            predecessors: self.predecessors.clone(),
        }
    }
}


pub struct ParityGameBuilder {
    nodes: usize,
    edges: Vec<(usize, usize)>,
    priorities: Vec<(usize, usize)>,
    owners: Vec<(usize, usize)>,
    labels: Vec<(usize, String)>,
}

impl ParityGameBuilder {
    pub fn new() -> Self {
        ParityGameBuilder {
            nodes: 0,
            edges: Vec::new(),
            priorities: Vec::new(),
            owners: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) -> &mut Self {
        self.edges.push((from, to));
        self.nodes = max(self.nodes, from + 1);
        self.nodes = max(self.nodes, to + 1);
        self
    }

    pub fn set_priority(&mut self, node: usize, priority: usize) -> &mut Self {
        self.priorities.push((node, priority));
        self.nodes = max(self.nodes, node + 1);
        self
    }

    pub fn set_owner(&mut self, node: usize, owner: usize) -> &mut Self {
        self.owners.push((node, owner));
        self.nodes = max(self.nodes, node + 1);
        self
    }

    pub fn set_label(&mut self, node: usize, label: String) -> &mut Self {
        self.labels.push((node, label));
        self.nodes = max(self.nodes, node + 1);
        self
    }    

    pub fn build(self) -> ParityGame {
        let mut game = ParityGame::new(self.nodes);
        for (from, to) in self.edges {
            game.add_edge(from, to);
        }
        for (node, priority) in self.priorities {
            game.set_priority(node, priority);
        }
        for (node, label) in self.labels {
            game.set_label(node, label);
        }
        for (node, owner) in self.owners {
            game.set_owner(node, owner);
        }
        game
    }
}
