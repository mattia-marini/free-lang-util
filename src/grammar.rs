use std::collections::HashSet;

use crate::args::GrammarDecodeError;
use crate::lr0::Lr0Item;
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
    pub driver: char,
    pub body: Vec<char>,
}

impl Production {
    pub fn new(driver: char, body: Vec<char>) -> Self {
        Production { driver, body }
    }

    pub fn as_lr0_item<'a>(&'a self) -> Lr0Item<'a> {
        Lr0Item::new(self)
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
}

impl std::fmt::Display for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for production in &self.productions {
            let body_str: String = production.body.iter().collect();
            writeln!(f, "{} -> {}", production.driver, body_str)?;
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
                driver: driver_str.chars().next().unwrap(),
                body,
            };
            grammar.add_production(production);
        }
    }

    Ok(grammar)
}
