use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::{
    Direction, Graph,
    algo::{condensation, toposort},
    graph::NodeIndex,
    visit::EdgeRef,
};

use crate::lr0::{Lr0Item, get_parsing_automaton};

use super::{
    grammar::Grammar,
    parse_structs::{Action, FirstFollowSet, Production},
};

impl Grammar {
    pub fn lr0_closure<'a>(&'a self, lr0_items: Vec<Lr0Item<'a>>) -> Vec<Lr0Item<'a>> {
        let mut queue = VecDeque::from(lr0_items.clone());
        let mut added_symbols: HashSet<char> = HashSet::new();
        let mut rv = vec![];
        // for item in queue.iter() {
        //     added_symbols.insert(item.production.driver);
        // }

        while !queue.is_empty() {
            let current_item = queue.pop_front().unwrap();
            let next_symbol = current_item.next_symbol();

            if let Some(next_symbol) = next_symbol {
                if !added_symbols.contains(&next_symbol) {
                    added_symbols.insert(next_symbol);
                    let closing_items = self
                        .productions
                        .iter()
                        .filter(|prod| prod.driver == next_symbol);

                    for production in closing_items {
                        let next_item = production.as_lr0_item();
                        queue.push_back(next_item.clone());
                        rv.push(next_item);
                    }
                }
            }
        }
        rv.sort_by(|lhs, rhs| {
            let mut p_iter = self.productions.iter();
            let lhs_pos = p_iter
                .clone()
                .position(|e| e.driver == lhs.production.driver)
                .unwrap();
            let rhs_pos = p_iter
                .clone()
                .position(|e| e.driver == rhs.production.driver)
                .unwrap();
            lhs_pos.cmp(&rhs_pos)
        });
        rv.into_iter()
            .filter(|item| !lr0_items.contains(item))
            .collect()
    }

    pub fn get_lr0_parsing_table(&self) -> Vec<HashMap<char, Vec<Action>>> {
        let automaton = get_parsing_automaton(self);
        let mut rv = vec![];
        for (node_index, node) in automaton.nodes.iter().enumerate() {
            let mut row = HashMap::new();

            for term in self.terms.iter() {
                row.insert(*term, vec![]);
            }
            row.insert('$', vec![]);
            for non_term in self.non_terms.iter() {
                row.insert(*non_term, vec![]);
            }

            // Populating shifs and gotos
            if let Some(edges) = automaton.edges.get(&node_index) {
                for (node_to, by_char) in edges {
                    if by_char.is_uppercase() {
                        row.get_mut(&by_char).unwrap().push(Action::Goto(*node_to));
                    } else {
                        row.get_mut(&by_char).unwrap().push(Action::Shift(*node_to));
                    }
                }
            }

            // Populating reduces
            for item in &node.kernel {
                if item.is_complete() {
                    match item.production.index {
                        Some(prod_index) => {
                            for term in self.terms.iter() {
                                row.get_mut(term).unwrap().push(Action::Reduce(prod_index));
                            }
                            row.get_mut(&'$').unwrap().push(Action::Reduce(prod_index));
                        }
                        None => row.get_mut(&'$').unwrap().push(Action::Acc),
                    }
                }
            }

            for item in &node.closure {
                if item.is_complete() {
                    match item.production.index {
                        Some(prod_index) => {
                            for term in self.terms.iter() {
                                row.get_mut(term).unwrap().push(Action::Reduce(prod_index));
                            }
                            row.get_mut(&'$').unwrap().push(Action::Reduce(prod_index));
                        }
                        None => row.get_mut(&'$').unwrap().push(Action::Acc),
                    }
                }
            }

            rv.push(row);
            // for terminal in sorted_terms {
            //     let v = automaton.edges.get(&node_index).unwrap();
            // }
        }

        rv
    }

    pub fn get_slr1_parsing_table(
        &self,
        parsing_table: Option<&Vec<HashMap<char, Vec<Action>>>>,
        first_follow_set: Option<&HashMap<char, FirstFollowSet>>,
    ) -> Vec<HashMap<char, Vec<Action>>> {
        let mut parsing_table = match parsing_table {
            Some(t) => t.clone(),
            None => self.get_lr0_parsing_table(),
        };

        let first_follow_owned;
        let first_follow_set = match first_follow_set {
            Some(ref s) => s,
            None => {
                first_follow_owned = self.get_first_follow_table();
                &first_follow_owned
            }
        };

        for (node_index, row) in parsing_table.iter_mut().enumerate() {
            for (by_char, actions) in row.iter_mut() {
                actions.retain(|a| match a {
                    Action::Reduce(reduce) => first_follow_set
                        .get(&self.productions[*reduce].driver)
                        .unwrap()
                        .follow
                        .contains(by_char),
                    _ => true,
                });
            }
        }

        parsing_table
    }

    /// Creates a table containing for each non terminal
    /// 1) the first set
    /// 2) the follow set
    /// 3) whether the terminal is nullable or not
    pub fn get_first_follow_table(&self) -> HashMap<char, FirstFollowSet> {
        let mut first_follow_table = HashMap::new();
        for non_term in &self.non_terms {
            first_follow_table.insert(*non_term, FirstFollowSet::new());
        }

        /* ######################### NULLABLES ######################### */
        let mut nullables = HashSet::new();
        let mut new_nullables_count = 0;

        for production in &self.productions {
            if production.body.is_empty() {
                nullables.insert(production.driver);
                new_nullables_count += 1;
            }
        }

        while new_nullables_count > 0 {
            new_nullables_count = 0;
            for production in &self.productions {
                let body_len = production.body.len();
                let nullables_in_body = production.body.iter().fold(0, |acc, el| {
                    acc + if nullables.contains(el) { 1 } else { 0 }
                });
                if nullables_in_body == body_len && !nullables.contains(&production.driver) {
                    nullables.insert(production.driver);
                    new_nullables_count += 1;
                }
            }
        }

        for nullable in &nullables {
            // println!("Nullable: {}", nullable);
            if let Some(set) = first_follow_table.get_mut(nullable) {
                set.nullable = true;
            }
        }

        /* ######################### FIRST ######################### */
        let mut productions_by_driver: HashMap<char, Vec<&Production>> = HashMap::new();
        for production in &self.productions {
            if !productions_by_driver.contains_key(&production.driver) {
                productions_by_driver.insert(production.driver, vec![]);
            }
            productions_by_driver
                .get_mut(&production.driver)
                .unwrap()
                .push(production);
        }

        for (driver, production_set) in &productions_by_driver {
            let mut curr_first_set = &mut first_follow_table.get_mut(&driver).unwrap().first;

            for prod in production_set {
                if let Some(first_terminal) = prod.body.iter().find(|symbol| symbol.is_lowercase())
                {
                    curr_first_set.insert(*first_terminal);
                }
            }
        }

        let mut first_graph = Graph::<(char, HashSet<char>), ()>::new();
        let mut first_graph_node_indices = HashMap::new();

        for non_term in &self.non_terms {
            let idx = first_graph.add_node((
                *non_term,
                first_follow_table.get(non_term).unwrap().first.clone(),
            ));
            first_graph_node_indices.insert(*non_term, idx);
        }

        for prod in &self.productions {
            for symbol in &prod.body {
                if symbol.is_uppercase() {
                    let node_from_idx = first_graph_node_indices.get(&prod.driver).unwrap();
                    let node_to_idx = first_graph_node_indices.get(&symbol).unwrap();
                    first_graph.add_edge(*node_from_idx, *node_to_idx, ());
                }

                if symbol.is_lowercase() || !nullables.contains(&symbol) {
                    break;
                }
            }
        }

        let first_condensation_graph = Self::propagate_referece_graph(&first_graph);
        for (non_terms, firsts) in first_condensation_graph.node_weights() {
            for non_term in non_terms.iter() {
                if let Some(set) = first_follow_table.get_mut(non_term) {
                    set.first.extend(firsts.iter().cloned());
                }
            }
        }

        /* ######################### FOLLOWS ######################### */
        let mut follow_graph = Graph::<(char, HashSet<char>), ()>::new();
        let mut follow_graph_node_indices = HashMap::new();

        for non_term in &self.non_terms {
            let idx = follow_graph.add_node((*non_term, HashSet::new()));
            follow_graph_node_indices.insert(*non_term, idx);
        }

        for prod in self.productions.iter() {
            if prod.body.len() == 0 {
                continue;
            }
            for i in 0..prod.body.len() {
                let l_char = prod.body[i];
                if l_char.is_lowercase() {
                    continue;
                }

                let mut new_follows: HashSet<char> = HashSet::new();

                let mut body_nullable = true;
                for j in i + 1..prod.body.len() {
                    let r_char = prod.body[j];
                    if r_char.is_uppercase() {
                        new_follows.extend(
                            first_follow_table
                                .get(&r_char)
                                .unwrap()
                                .first
                                .iter()
                                .clone(),
                        );
                    } else {
                        new_follows.insert(r_char);
                    }

                    if !nullables.contains(&r_char) {
                        body_nullable = false;
                        break;
                    }
                }

                if body_nullable {
                    let node_from = follow_graph_node_indices.get(&l_char).unwrap();
                    let node_to = follow_graph_node_indices.get(&prod.driver).unwrap();
                    follow_graph.add_edge(*node_from, *node_to, ());
                }

                follow_graph
                    .node_weight_mut(*follow_graph_node_indices.get(&l_char).unwrap())
                    .unwrap()
                    .1
                    .extend(new_follows.into_iter());
            }
        }

        follow_graph
            .node_weight_mut(
                *follow_graph_node_indices
                    .get(&self.productions[0].driver)
                    .unwrap(),
            )
            .unwrap()
            .1
            .insert('$');
        let follow_condensation_graph = Self::propagate_referece_graph(&follow_graph);
        for (symbols, follows) in follow_condensation_graph.node_weights() {
            for non_term in symbols.iter() {
                if let Some(set) = first_follow_table.get_mut(non_term) {
                    set.follow.extend(follows.iter().cloned());
                }
            }
        }

        first_follow_table
    }

    /// Given a graph representing the how firsts-follows depend on each other, it return a
    /// condensation graph where each component contains the set of non-terminals that circularly
    /// depend on each other and such that no further propagation could be done (i.e. each non
    /// terminal has inherited every first-follow it can inherit)
    fn propagate_referece_graph(
        graph: &Graph<(char, HashSet<char>), ()>,
    ) -> Graph<(HashSet<char>, HashSet<char>), ()> {
        let mut condensation_graph = condensation(graph.clone(), true);

        let mut first_by_condensation_node: HashMap<NodeIndex, HashSet<char>> = HashMap::new();

        for condensed_node_idx in condensation_graph.node_indices() {
            let condensed_nodes = condensation_graph.node_weight(condensed_node_idx).unwrap();
            first_by_condensation_node.insert(condensed_node_idx, HashSet::new());

            for node in condensed_nodes.iter() {
                first_by_condensation_node
                    .get_mut(&condensed_node_idx)
                    .unwrap()
                    .extend(node.1.iter().cloned());
            }
        }

        // Condensation graph has now a list of the condensed nodes and a list of first
        // (condensed_nodes, firsts)
        let mut condensation_graph = condensation_graph.map(
            |idx, wheight| {
                (
                    wheight
                        .iter()
                        .cloned()
                        .map(|(c, _)| c)
                        .collect::<HashSet<char>>(),
                    first_by_condensation_node.remove(&idx).unwrap(),
                )
            },
            |idx, wheight| *wheight,
        );

        let topological_order =
            toposort(&condensation_graph, None).expect("Failed to compute topological order");

        for node_index in topological_order.iter().rev() {
            let mut new_firsts = HashSet::new();
            for edge in condensation_graph.edges_directed(*node_index, Direction::Outgoing) {
                let node_to_idx = edge.target();

                let new_firsts_from_curr_edge = condensation_graph
                    .node_weight(node_to_idx)
                    .unwrap()
                    .1
                    .clone();
                new_firsts.extend(new_firsts_from_curr_edge);
            }
            condensation_graph
                .node_weight_mut(*node_index)
                .unwrap()
                .1
                .extend(new_firsts.clone())
        }

        condensation_graph
    }
}
