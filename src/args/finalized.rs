use std::fs;

use base64::{Engine as _, engine::general_purpose};
use clap::{ArgGroup, Parser};

use crate::grammar::latex::LatexFormatOutputFormatDescriptor;

use super::Args;

pub struct FinalizedArgs {
    pub input_type: InputType,
    pub output_type: OutputType,
    pub latex_format_descriptor: Option<LatexFormatOutputFormatDescriptor>,
}

pub enum InputType {
    File(String),
    Base64(String),
}

pub enum OutputType {
    Latex,
    Dot,
}

impl Args {
    pub fn finalize(&self) -> FinalizedArgs {
        // Se l'utente non ha specificato né --latex né --dot, abilito --latex di default

        let input_type = match (&self.file, &self.base64) {
            (None, Some(base64)) => InputType::Base64(base64.clone()),
            (Some(file), None) => InputType::File(file.clone()),
            (Some(_), Some(_)) => panic!("Error: --file and --base-64 are mutually exclusive"),
            _ => panic!("Error: you should provide either --file and --base-64"),
        };

        let output_type = match (self.latex, self.dot) {
            (true, false) => OutputType::Latex,
            (false, true) => OutputType::Dot,
            (false, false) => OutputType::Dot,
            (true, true) => panic!("Error: --latex and --dot are mutually exclusive"),
        };

        let latex_format_descriptor = if self.all {
            LatexFormatOutputFormatDescriptor::FULL
        } else if !self.grammophone_link
            && !self.grammophone_link
            && !self.grammar_definition
            && !self.lr0_parsing_table
            && !self.slr1_parsing_table
            && !self.first_follow_set
        {
            LatexFormatOutputFormatDescriptor::FULL
        } else {
            LatexFormatOutputFormatDescriptor {
                grammophone_link: self.grammophone_link,
                graphviz_link: self.grammophone_link,
                grammar_definition: self.grammar_definition,
                lr0_parsing_table: self.lr0_parsing_table,
                slr1_parsing_table: self.slr1_parsing_table,
                first_follow_set: self.first_follow_set,
            }
        };

        FinalizedArgs {
            input_type,
            output_type,
            latex_format_descriptor: if self.latex || self.dot {
                Some(latex_format_descriptor)
            } else {
                None
            },
        }
    }
}
