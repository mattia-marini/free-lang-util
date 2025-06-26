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
