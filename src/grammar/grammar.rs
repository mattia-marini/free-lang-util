use std::cmp::max;
use std::collections::{HashMap, HashSet};

use petgraph::algo::{condensation, toposort};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Direction, Graph};

use crate::args::error::GrammarDecodeError;
use crate::lr0::{Lr0Item, get_parsing_automaton};
use crate::util::get_dot_from_petgraph;
use std::collections::VecDeque;

use super::latex::LatexFormatOutputFormatDescriptor;
use super::parse_structs::{Action, FirstFollowSet, Production};

#[derive(Debug, Clone, PartialEq)]
pub struct Grammar {
    pub starting_prod: Option<Production>,
    pub productions: Vec<Production>,
    pub terms: HashSet<char>,
    pub non_terms: HashSet<char>,
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

    /// Returns a vector of the grammar's terminals sorted as they would be in a parsing table,
    /// i.e. in alphabetical order. Note that this does not include the end of input '$' symbol
    pub fn get_sorted_terms(&self) -> Vec<char> {
        let mut sorted_terms: Vec<char> = self.terms.iter().cloned().collect();
        sorted_terms.sort();
        sorted_terms
    }

    /// Returns a vector of the grammar's non terminals sorted as they would be in a parsing table,
    /// i.e. in the order of production drivers specified in the grammar.
    pub fn get_sorted_non_terms(&self) -> Vec<char> {
        let mut sorted_non_terms: Vec<char> =
            self.productions.iter().map(|prod| prod.driver).collect();
        sorted_non_terms.dedup();
        sorted_non_terms
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
