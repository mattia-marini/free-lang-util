use crate::grammar::{Grammar, Production};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Lr0Item<'a> {
    pub production: &'a Production,
    dot_position: usize,
}

pub struct Lr0ItemOwned {
    pub production: Production,
    dot_position: usize,
}

impl<'a> Lr0Item<'a> {
    pub fn new(production: &'a Production) -> Self {
        Lr0Item {
            production,
            dot_position: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.dot_position >= self.production.body.len()
    }

    pub fn next_symbol(&self) -> Option<char> {
        self.production.body.get(self.dot_position).copied()
    }

    pub fn next_item(&self) -> Option<Lr0Item<'a>> {
        if !self.is_complete() {
            Some(Lr0Item {
                production: self.production,
                dot_position: self.dot_position + 1,
            })
        } else {
            None
        }
    }

    pub fn advance(&mut self) {
        if !self.is_complete() {
            self.dot_position += 1;
        }
    }
}

impl std::fmt::Display for Lr0Item<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body: String = self.production.body.iter().collect();
        let before_dot = &body[..self.dot_position];
        let after_dot = &body[self.dot_position..];
        write!(
            f,
            "{} -> {}•{}",
            self.production.driver,
            before_dot,
            if after_dot.ends_with(" ") {
                ""
            } else {
                after_dot
            }
        )
    }
}

pub struct Lr0Automaton<'a> {
    nodes: Vec<Lr0AutomatonNode<'a>>,
    edges: HashMap<usize, Vec<(usize, char)>>,
}

impl Lr0Automaton<'_> {
    pub fn generate_dot_notation_string(&self) -> String {
        let mut rv = String::new();
        rv.push_str("digraph G {\nnode[shape=record]\n\n");
        for (index, item) in self.nodes.iter().enumerate() {
            let mut label_kernel = String::new();
            let mut label_closure = String::new();
            for prod in &item.kernel {
                label_kernel.push_str(&format!("{}\\n", prod).replace("->", "→"));
            }
            for prod in &item.closure {
                label_closure.push_str(&format!("{}\\n", prod).replace("->", "→"));
            }
            label_kernel = label_kernel.trim_end().to_string();
            label_closure = label_closure.trim_end().to_string();

            if label_closure.is_empty() {
                rv.push_str(&format!(
                    "{} [label=\"{{ {} | {} }}\"]\n",
                    index, index, label_kernel
                ));
            } else {
                rv.push_str(&format!(
                    "{} [label=\"{{ {} | {} | {} }}\"]\n",
                    index, index, label_kernel, label_closure
                ));
            }
        }
        rv.push_str("\n\n//nodes\n");
        for (from, neighbours) in &self.edges {
            for neighbour in neighbours {
                let to = neighbour.0;
                let by_char = neighbour.1;
                rv.push_str(&format!("{} -> {} [label=\"{}\"]\n", from, to, by_char));
            }
        }
        rv.push_str("}\n");

        rv
    }
}

impl<'a> std::fmt::Display for Lr0Automaton<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, node) in self.nodes.iter().enumerate() {
            writeln!(f, "Node {}:", index)?;
            writeln!(f, "  Kernel:")?;
            for item in &node.kernel {
                writeln!(f, "    {}", item)?;
            }
            writeln!(f, "  Closure:")?;
            for item in &node.closure {
                writeln!(f, "    {}", item)?;
            }
        }
        writeln!(f, "Edges:")?;
        for (from, edges) in &self.edges {
            for (to, by_char) in edges {
                writeln!(f, "  {} --{}--> {}", from, by_char, to)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Lr0AutomatonNode<'a> {
    kernel: Vec<Lr0Item<'a>>,
    closure: Vec<Lr0Item<'a>>,
}

impl<'a> Lr0AutomatonNode<'a> {
    pub fn get_generated_kernel(&self, by_char: char) -> Vec<Lr0Item<'a>> {
        let mut rv = vec![];

        for item in &self.kernel {
            if let Some(next_symbol) = item.next_symbol() {
                if next_symbol == by_char {
                    let next_item = item.next_item().unwrap();
                    rv.push(next_item);
                }
            }
        }

        for item in &self.closure {
            if let Some(next_symbol) = item.next_symbol() {
                if next_symbol == by_char {
                    let next_item = item.next_item().unwrap();
                    rv.push(next_item);
                }
            }
        }

        rv
    }
}

/// Returns the lr0 parsing automaton for the given grammar.
pub fn get_parsing_automaton<'a>(grammar: &'a Grammar) -> Lr0Automaton<'a> {
    let starting_lr0_item: Lr0Item<'a> = grammar.starting_prod.as_ref().unwrap().as_lr0_item();
    let first_node = Lr0AutomatonNode {
        kernel: vec![starting_lr0_item.clone()],
        closure: grammar.lr0_closure(vec![starting_lr0_item.clone()]),
    };

    let mut automaton: Lr0Automaton<'a> = Lr0Automaton {
        nodes: vec![first_node.clone()],
        edges: HashMap::new(),
    };

    let mut nodes_to_process = VecDeque::new();
    let mut kernels: HashMap<Lr0AutomatonNode<'a>, usize> = HashMap::new();
    let mut latest_index = 1;
    kernels.insert(first_node, 0);

    nodes_to_process.push_back(0);

    while !nodes_to_process.is_empty() {
        let curr_node_index = nodes_to_process.pop_front().unwrap();

        let mut outgoing_chars = vec![];
        let mut outgoing_chars_set: HashSet<char> = HashSet::new();

        let curr_node = &automaton.nodes[curr_node_index];
        for item in curr_node.kernel.iter() {
            if let Some(next_symbol) = item.next_symbol() {
                if !outgoing_chars_set.contains(&next_symbol) {
                    outgoing_chars.push(next_symbol);
                    outgoing_chars_set.insert(next_symbol);
                }
            }
        }
        for item in curr_node.closure.iter() {
            if let Some(next_symbol) = item.next_symbol() {
                if !outgoing_chars_set.contains(&next_symbol) {
                    outgoing_chars.push(next_symbol);
                    outgoing_chars_set.insert(next_symbol);
                }
            }
        }
        // let x: Vec<Lr0Item<'a>> = curr_node.get_generated_kernel('a');
        let mut new_nodes: Vec<(usize, Lr0AutomatonNode<'a>)> = vec![];
        let mut new_edges: Vec<((usize, usize), char)> = vec![];

        for outgoing_char in outgoing_chars {
            let next_kernel: Vec<Lr0Item<'a>> = curr_node.get_generated_kernel(outgoing_char);
            if !next_kernel.is_empty() {
                let new_node = Lr0AutomatonNode {
                    kernel: next_kernel.clone(),
                    closure: grammar.lr0_closure(next_kernel),
                };
                if !kernels.contains_key(&new_node) {
                    kernels.insert(new_node.clone(), latest_index);
                    new_nodes.push((latest_index, new_node.clone()));
                    latest_index += 1;
                }
                new_edges.push((
                    (curr_node_index, *kernels.get(&new_node).unwrap()),
                    outgoing_char,
                ));
            }
        }
        automaton
            .nodes
            .append(&mut new_nodes.clone().into_iter().map(|e| e.1).collect());
        nodes_to_process.extend(new_nodes.into_iter().map(|e| e.0));

        for edge in new_edges {
            let from = edge.0.0;
            let to = edge.0.1;
            let by_char = edge.1;

            if !automaton.edges.contains_key(&from) {
                automaton.edges.insert(from, vec![]);
            }
            automaton.edges.get_mut(&from).unwrap().push((to, by_char));
        }
    }

    automaton
}

/// Compute and print closures for all productions in the grammar.
pub fn print_closures(grammar: &Grammar) {
    for production in &grammar.productions {
        let lr0_item = production.as_lr0_item();
        println!("Closure for production {}:", lr0_item);
        let closure = grammar.lr0_closure(vec![lr0_item]);
        for item in closure {
            let body: String = item.production.body.iter().collect();
            println!("  {} -> {}", item.production.driver, body);
        }
    }
}
