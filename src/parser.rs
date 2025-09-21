use crate::parser::splitting::Command;
use thiserror::Error;

mod quoting;
mod splitting;

#[derive(Error, Debug)]
pub(crate) enum ParsingError {
    #[error(transparent)]
    SplittingError(#[from] splitting::SplittingError),
}

pub(crate) fn parse_input(input: &str) -> Result<Vec<Command>, ParsingError> {
    // TODO: Ideally quoting::split_quoted_string would be called, to segregate responsibilities.

    let commands = splitting::parse_input(input)?;

    Ok(commands)
}
