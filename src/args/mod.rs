pub mod finalized;
pub mod error;

use std::fs;

use base64::{Engine as _, engine::general_purpose};
use clap::{ArgGroup, Parser};
use error::GrammarDecodeError;


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
    ),
    group(
        ArgGroup::new("latex-format")
            .required(false) 
            .multiple(true)
            .args(["grammophone_link", "graphviz_link", "grammar_definition", "lr0_parsing_table", "slr1_parsing_table", "first_follow_set", "all"]),
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

    /// Include Grammophone link
    #[arg(long, default_value_t = false, group="latex-format")]
    grammophone_link: bool,
    ///
    /// Include Graphviz link
    #[arg(long, default_value_t = false, group="latex-format")]
    graphviz_link: bool,

    /// Include grammar definition
    #[arg(long, default_value_t = false, group="latex-format")]
    grammar_definition: bool,

    /// Include LR(0) parsing table
    #[arg(long, default_value_t = false, group="latex-format")]
    lr0_parsing_table: bool,

    /// Include SLR(1) parsing table
    #[arg(long, default_value_t = false, group="latex-format")]
    slr1_parsing_table: bool,

    /// Include first-follow set
    #[arg(long, default_value_t = false, group="latex-format")]
    first_follow_set: bool,

    #[arg(long, default_value_t = false, group="latex-format")]
    all: bool,

}



