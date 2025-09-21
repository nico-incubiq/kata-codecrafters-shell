use crate::parser::splitting::Command;
use thiserror::Error;
use crate::parser::quoting::QuotingError;

mod quoting;
mod splitting;

#[derive(Error, Debug)]
pub(crate) enum ParsingError {
    #[error(transparent)]
    Quoting(#[from] QuotingError),

    #[error(transparent)]
    CommandSplittingError(#[from] splitting::SplittingError),
}

pub(crate) fn parse_input(input: &str) -> Result<Vec<Command>, ParsingError> {
    let values = quoting::chunk_quoted_string(input)?;

    let commands = splitting::split_commands(values)?;

    Ok(commands)
}
