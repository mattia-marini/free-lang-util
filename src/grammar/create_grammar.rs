use std::fs;

use crate::{
    args::{Args, error::GrammarDecodeError},
    grammar::parse_structs::Production,
    util::truncate_after_last,
};
use base64::Engine as _;

use super::grammar::Grammar;

pub fn read_from_file(file_path: String) -> Result<String, GrammarDecodeError> {
    fs::read_to_string(&file_path).map_err(|err| {
        GrammarDecodeError::ParseError(format!("Failed to read file {}: {}", file_path, err))
    })
}

pub fn decode_base_64(base64: String) -> Result<String, GrammarDecodeError> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .map_err(|err| {
            GrammarDecodeError::ParseError(format!("Failed to decode base64 string: {}", err))
        })?;

    String::from_utf8(decoded_bytes).map_err(|err| {
        GrammarDecodeError::ParseError(format!("Failed to convert bytes to string: {}", err))
    })
}

pub fn decode_grammar(args: &Args) -> Result<Grammar, GrammarDecodeError> {
    let decoded_text = match (&args.file, &args.base64) {
        (Some(file_path), None) => read_from_file(file_path.clone()),
        (None, Some(base64)) => decode_base_64(base64.clone()),
        (None, None) => panic!("This should not be happening, clap should have handled this"),
        (Some(_), Some(_)) => {
            panic!("This should not be happening, clap should have handled this")
        }
    }?;

    create_grammar_from_str(&decoded_text)
}

pub fn create_grammar_from_str(grammar_str: &String) -> Result<Grammar, GrammarDecodeError> {
    // println!("Creating grammar from string:\n{}", grammar_str);

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
            let var_name = Production {
                index: Some(grammar.productions.len()),
                driver: driver_str.chars().next().unwrap(),
                body,
            };
            let production = var_name;
            grammar.add_production(production);
        }
    }

    Ok(grammar)
}
