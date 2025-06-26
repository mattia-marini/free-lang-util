use std::cmp::max;
use std::collections::{HashMap, HashSet};

use petgraph::algo::{condensation, toposort};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Direction, Graph};

use crate::args::GrammarDecodeError;
use crate::lr0::{Lr0Item, get_parsing_automaton};
use crate::util::get_dot_from_petgraph;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct Grammar {
    pub starting_prod: Option<Production>,
    pub productions: Vec<Production>,
    pub terms: HashSet<char>,
    pub non_terms: HashSet<char>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Production {
    pub index: Option<usize>,
    pub driver: char,
    pub body: Vec<char>,
}

impl Production {
    pub fn new(driver: char, body: Vec<char>) -> Self {
        Production {
            index: None,
            driver,
            body,
        }
    }

    pub fn as_lr0_item<'a>(&'a self) -> Lr0Item<'a> {
        Lr0Item::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct FirstFollowSet {
    first: HashSet<char>,
    follow: HashSet<char>,
    nullable: bool,
}

impl FirstFollowSet {
    pub fn new() -> Self {
        FirstFollowSet {
            first: HashSet::new(),
            follow: HashSet::new(),
            nullable: false,
        }
    }
}

impl Grammar {
    pub fn new() -> Self {
        Grammar {
            starting_prod: None,
            productions: vec![],
            terms: HashSet::new(),
            non_terms: HashSet::new(),
        }
    }

    pub fn add_production(&mut self, production: Production) {
        if self.starting_prod.is_none() {
            self.starting_prod = Some(Production {
                index: None,
                driver: '@',
                body: vec![production.driver],
            });
        }

        if !self.non_terms.contains(&production.driver) {
            self.non_terms.insert(production.driver);
        }

        for symbol in &production.body {
            if symbol.is_lowercase() && !self.terms.contains(symbol) {
                self.terms.insert(*symbol);
            }
        }

        self.productions.push(production);
    }

    pub fn add_term(&mut self, term: char) {
        if !self.terms.contains(&term) {
            self.terms.insert(term);
        }
    }

    pub fn add_non_term(&mut self, non_term: char) {
        if !self.non_terms.contains(&non_term) {
            self.non_terms.insert(non_term);
        }
    }

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

    fn generate_parsing_table_latex(
        &self,
        parsing_table: &Vec<HashMap<char, Vec<Action>>>,
        sorted_terms: &Vec<char>,
        sorted_non_terms: &Vec<char>,
    ) -> String {
        let mut rv = String::new();

        rv.push_str("\\begin{table}[H]");
        rv.push_str("\\centering");
        rv.push_str(
            format!(
                "\\begin{{tabular}}{{{}}}\n",
                "c".repeat(sorted_terms.len() + sorted_non_terms.len() + 1)
            )
            .as_str(),
        );
        rv.push_str("\\toprule\n");
        let header = format!(
            "States & {} & {}\\\\\n",
            sorted_terms
                .iter()
                .map(|c| if *c == '$' {
                    "\\$".to_string()
                } else {
                    c.to_string()
                })
                .collect::<Vec<String>>()
                .join(" & "),
            sorted_non_terms
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(" & ")
        );
        rv.push_str(header.as_str());
        rv.push_str("\\midrule\n");

        for (node_index, row) in parsing_table.iter().enumerate() {
            let mut row_str = vec![];
            for term in sorted_terms.iter() {
                if let Some(actions) = row.get(term) {
                    let actions_str: Vec<String> =
                        actions.iter().map(|action| action.to_string()).collect();
                    // actions_str = actions_str.join(", ");
                    // join("/");
                    row_str.push(actions_str.join("/"));
                } else {
                    row_str.push(String::from(" "));
                }
            }

            for non_term in sorted_non_terms.iter() {
                if let Some(actions) = row.get(non_term) {
                    let actions_str: Vec<String> =
                        actions.iter().map(|action| action.to_string()).collect();
                    // actions_str = actions_str.join(", ");
                    // join("/");
                    row_str.push(actions_str.join("/"));
                } else {
                    row_str.push(String::from(" "));
                }
            }

            rv.push_str(format!("s{} & {} \\\\ \n", node_index, row_str.join(" & ")).as_str());
        }

        rv.push_str("\\bottomrule\n");
        rv.push_str("\\end{tabular}\n");
        rv.push_str("\\caption{Tabella LR(0) senza pruning}");
        rv.push_str("\\end{table}");
        rv
    }

    fn generate_first_follow_table_latex(
        &self,
        parsing_table: &Vec<HashMap<char, Vec<Action>>>,
        sorted_terms: &Vec<char>,
        sorted_non_terms: &Vec<char>,
    ) -> String {
        let mut rv = String::new();
        let first_follow_set = self.get_first_follow_table();

        rv.push_str("\\begin{table}[H]");
        rv.push_str("\\centering");
        rv.push_str("\\begin{tabular}{cccc}\n");
        rv.push_str("\\toprule\n");
        rv.push_str("Symbol & First\\-set & Follow\\-set & Nullable\\\\\n");
        rv.push_str("\\midrule\n");
        for non_term in sorted_non_terms.iter() {
            if let Some(set) = first_follow_set.get(non_term) {
                let espace_latex_chars = |e: char| {
                    if e == '$' {
                        "\\$".to_string()
                    } else {
                        e.to_string()
                    }
                };

                let mut first_set = vec![];
                for c in sorted_terms.iter() {
                    if let Some(first) = set.first.get(c) {
                        first_set.push(*first);
                    }
                }

                let mut follow_set = vec![];
                for c in sorted_terms.iter() {
                    if let Some(follow) = set.follow.get(c) {
                        follow_set.push(*follow);
                    }
                }

                let first_set_str = first_set
                    .iter()
                    .cloned()
                    .map(espace_latex_chars)
                    .collect::<Vec<String>>()
                    .join(",");
                let follow_set_str: String = follow_set
                    .iter()
                    .cloned()
                    .map(espace_latex_chars)
                    .collect::<Vec<String>>()
                    .join(",");

                let nullable_str = if set.nullable { "Yes" } else { "No" };
                rv.push_str(
                    format!(
                        "{} & {} & {} & {}\\\\\n",
                        non_term, first_set_str, follow_set_str, nullable_str
                    )
                    .as_str(),
                );
            }
        }
        rv.push_str("\\bottomrule\n");
        rv.push_str("\\end{tabular}\n");
        rv.push_str("\\end{table}");
        rv
    }

    pub fn generate_latex_string(&self) -> String {
        /* ######################### Common ######################### */
        let parsing_table = self.get_lr0_parsing_table();

        let mut sorted_terms: Vec<char> = self.terms.iter().cloned().collect();
        sorted_terms.sort();
        sorted_terms.push('$');

        let mut sorted_non_terms: Vec<char> =
            self.productions.iter().map(|prod| prod.driver).collect();
        sorted_non_terms.dedup();

        /* ######################### Parsing table ######################### */
        let parsing_table_string = Self::generate_parsing_table_latex(
            self,
            &parsing_table,
            &sorted_terms,
            &sorted_non_terms,
        );

        /* ######################### First follow table ######################### */
        let first_follow_table_string = Self::generate_first_follow_table_latex(
            self,
            &parsing_table,
            &sorted_terms,
            &sorted_non_terms,
        );

        format!(
            "%Parsing table\n{} \n\n %First-follow set\n{}",
            parsing_table_string, first_follow_table_string
        )
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
            println!("Nullable: {}", nullable);
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
        // let wheights: Vec<String> = topological_order
        //     .iter()
        //     .map(|e| condensation_graph.node_weight(*e).unwrap())
        //     .map(|e| {
        //         e.iter()
        //             .map(|c| c.to_string())
        //             .collect::<Vec<String>>()
        //             .join(",")
        //     })
        //     .collect();
        //
        // println!("Topological order: {:?}", wheights);

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

pub enum Action {
    Shift(usize),
    Reduce(usize),
    Acc,
    Goto(usize),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Shift(to) => write!(f, "s{}", to),
            Action::Reduce(prod_index) => write!(f, "r{}", prod_index),
            Action::Goto(to) => write!(f, "{}", to),
            Action::Acc => write!(f, "acc"),
        }
    }
}

impl std::fmt::Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for production in &self.productions {
            let body_str: String = production.body.iter().collect();
            let prod_index = production
                .index
                .map_or("?".to_string(), |idx| idx.to_string());
            writeln!(f, "{}\t{} -> {}", prod_index, production.driver, body_str)?;
        }
        Ok(())
    }
}

fn truncate_after_last(s: &str, c: char) -> String {
    match s.rfind(c) {
        Some(index) => s[..=index].to_string(), // Include the character
        None => s.to_string(),
    }
}

pub fn create_grammar_from_str(grammar_str: &String) -> Result<Grammar, GrammarDecodeError> {
    println!("Creating grammar from string:\n{}", grammar_str);

    let cleaned_str = truncate_after_last(grammar_str.as_str(), '.');
    let mut grammar = Grammar::new();
    for line in cleaned_str.split('\n') {
        let mut cleaned_line = line.trim();
        if cleaned_line.is_empty() {
            continue;
        }
        cleaned_line = cleaned_line.trim_end_matches('.');

        let arrow_pos = cleaned_line
            .find("->")
            .ok_or_else(|| GrammarDecodeError::InvalidFormat(String::from("No -> arrow")))?;

        let (mut driver_str, mut prod_bodies_str) = cleaned_line.split_at(arrow_pos);

        driver_str = driver_str.trim();
        prod_bodies_str = &prod_bodies_str[2..];
        prod_bodies_str = prod_bodies_str.trim();

        if driver_str.len() != 1 {
            return Err(GrammarDecodeError::InvalidFormat(format!(
                "Grammar is not free: expected one symbol on the left side of '->', found {:?}",
                driver_str
            )));
        }

        for body_str in prod_bodies_str.split('|') {
            let body_str = body_str.trim();
            let mut body = vec![];
            for symbol in body_str.split(' ') {
                let symbol_len = symbol.len();
                if symbol_len == 1 {
                    body.push(symbol.chars().next().unwrap());
                } else if symbol_len == 0 {
                    // body.push(' ');
                } else {
                    return Err(GrammarDecodeError::InvalidFormat(format!(
                        "Each symbol should be a single character: {:?}",
                        symbol
                    )));
                }
            }
            let production = Production {
                index: Some(grammar.productions.len()),
                driver: driver_str.chars().next().unwrap(),
                body,
            };
            grammar.add_production(production);
        }
    }

    Ok(grammar)
}
