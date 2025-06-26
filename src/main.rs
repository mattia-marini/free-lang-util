#![allow(unused)]

use base64::{Engine as _, engine::general_purpose};
use clap::Parser;
use lr0::{get_parsing_automaton, print_closures};
use std::fs;
use std::io::{self, Read};

mod args;
mod grammar;
mod lr0;
use args::{Args, decode_grammar};

fn main() {
    let mut args = args::Args::parse();
    args.finalize();

    println!("Parsed Arguments: {:?}", args);
    // match decode_grammar(args) {
    //     Ok(grammar) => {
    //         println!("Decoded Grammar:\n{}", grammar);
    //         // print_closures(&grammar);
    //         let automaton = get_parsing_automaton(&grammar);
    //         // println!("LR(0) Parsing Automaton:\n{}", automaton);
    //         println!("{}", automaton.generate_dot_notation_string());
    //     }
    //     Err(err) => {
    //         eprintln!("Error decoding grammar: {:?}", err);
    //     }
    // }

    // println!("{}", decoded_text);
}
