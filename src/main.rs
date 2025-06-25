#![allow(unused)]

use base64::{Engine as _, engine::general_purpose};
use clap::Parser;
use std::fs;
use std::io::{self, Read};

mod args;
mod grammar;
use args::{Args, decode_grammar};

fn main() {
    let args = args::Args::parse();

    match decode_grammar(args) {
        Ok(grammar) => {
            println!("Decoded Grammar:\n{}", grammar);
        }
        Err(err) => {
            eprintln!("Error decoding grammar: {:?}", err);
        }
    }

    // println!("{}", decoded_text);
}
