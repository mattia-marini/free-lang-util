#![allow(unused)]
pub mod args;
pub mod grammar;
pub mod lr0;
pub mod util;

use base64::{Engine as _, engine::general_purpose};
use clap::Parser;
use lr0::{get_parsing_automaton, print_closures};
use std::fs;
use std::io::{self, Read};

use args::Args;
use grammar::create_grammar::decode_grammar;

fn main() {
    let args = args::Args::parse();
    let finalized_args = args.finalize();

    match decode_grammar(&args) {
        Ok(grammar) => {
            // println!("Decoded Grammar:\n{}", grammar);
            if args.latex {
                println!(
                    "{}",
                    grammar.generate_latex_string(finalized_args.latex_format_descriptor.unwrap())
                );
            } else if args.dot {
                let automaton = get_parsing_automaton(&grammar);
                println!("{}", automaton.generate_dot_notation_string());
            }
        }
        Err(err) => {
            eprintln!("Error decoding grammar: {:?}", err);
        }
    }
}
