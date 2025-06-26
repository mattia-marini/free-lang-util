use std::fs;

use base64::{Engine as _, engine::general_purpose};
use clap::{ArgGroup, Parser};

use crate::grammar::{Grammar, create_grammar_from_str};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .multiple(false)
        .args(["file", "base64"]),

    ),
    group(
        ArgGroup::new("output")
            .required(false) 
            .multiple(false)
            .args(["latex", "dot"]),
    )
)]
pub struct Args {
    /// Input file path
    #[arg(short = 'f', long, group = "input")]
    pub file: Option<String>,

    /// Base64 encoded input
    #[arg(long = "base-64", group = "input")]
    pub base64: Option<String>,

    /// Flag per generare output in formato LaTeX
    #[arg(long, default_value_t = false, group = "output")]
    pub latex: bool,

    /// Flag per generare output in formato DOT
    #[arg(long, default_value_t = false,  group = "output")]
    pub dot: bool,
}


impl Args {
    pub fn finalize(&mut self) {
        // Se l'utente non ha specificato né --latex né --dot, abilito --latex di default
        if !self.latex && !self.dot {
            self.dot = true;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GrammarDecodeError {
    InvalidFormat(String),
    ParseError(String),
}

#[derive(Debug)]
pub enum ArgsError {
    ArgsParsingError(clap::Error),
    ArgsConflict,
    MissingRequiredArg,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InitError {
    GrammarDecodeError(GrammarDecodeError),
}

pub fn read_from_file(file_path: String) -> Result<String, GrammarDecodeError> {
    fs::read_to_string(&file_path).map_err(|err| {
        GrammarDecodeError::ParseError(format!("Failed to read file {}: {}", file_path, err))
    })
}

pub fn decode_base_64(base64: String) -> Result<String, GrammarDecodeError> {
    let decoded_bytes = general_purpose::STANDARD.decode(base64).map_err(|err| {
        GrammarDecodeError::ParseError(format!("Failed to decode base64 string: {}", err))
    })?;

    String::from_utf8(decoded_bytes).map_err(|err| {
        GrammarDecodeError::ParseError(format!("Failed to convert bytes to string: {}", err))
    })
}

pub fn decode_grammar(args: Args) -> Result<Grammar, GrammarDecodeError> {
    let decoded_text = match (args.file, args.base64) {
        (Some(file_path), None) => read_from_file(file_path),
        (None, Some(base64)) => decode_base_64(base64),
        (None, None) => panic!("This should not be happening, clap should have handled this"),
        (Some(_), Some(_)) => {
            panic!("This should not be happening, clap should have handled this")
        }
    }?;

    create_grammar_from_str(&decoded_text)
}
