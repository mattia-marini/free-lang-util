use crate::args::GrammarDecodeError;

#[derive(Debug, Clone, PartialEq)]
pub struct Grammar {
    pub productions: Vec<Production>,
    pub terms: Vec<char>,
    pub non_terms: Vec<char>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Production {
    pub driver: char,
    pub body: Vec<char>,
}

impl Grammar {
    pub fn new() -> Self {
        Grammar {
            productions: vec![],
            terms: vec![],
            non_terms: vec![],
        }
    }

    pub fn add_production(&mut self, production: Production) {
        if !self.non_terms.contains(&production.driver) {
            self.non_terms.push(production.driver);
        }
        self.productions.push(production);
    }

    pub fn add_term(&mut self, term: char) {
        if !self.terms.contains(&term) {
            self.terms.push(term);
        }
    }

    pub fn add_non_term(&mut self, non_term: char) {
        if !self.non_terms.contains(&non_term) {
            self.non_terms.push(non_term);
        }
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
    println!("Creating grammar from string: {}", grammar_str);

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
                    body.push(' ');
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
