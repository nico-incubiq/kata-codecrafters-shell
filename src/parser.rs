use crate::parser::quoting::QuotingError;
use thiserror::Error;

mod quoting;
mod splitting;

#[derive(Error, Debug)]
pub(crate) enum ParsingError {
    #[error(transparent)]
    Quoting(#[from] QuotingError),

    #[error(transparent)]
    CommandSplittingError(#[from] splitting::SplittingError),
}

/// A file descriptor.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Descriptor(pub(crate) u8);

impl Descriptor {
    pub(crate) fn stdout() -> Self {
        Self(1)
    }

    pub(crate) fn stderr() -> Self {
        Self(2)
    }
}

/// A command with its arguments and redirections in the order they were specified.
pub(crate) struct Command {
    program: String,
    arguments: Vec<String>,
    redirects: Vec<Redirect>,
}

/// An IO redirection.
pub(crate) struct Redirect {
    /// The IO descriptor.
    /// 0: input (unsupported), 1: output, 2: error
    from: Descriptor,
    to: RedirectTo,
    append: bool,
}

impl Redirect {
    pub(crate) fn from(&self) -> Descriptor {
        self.from
    }

    pub(crate) fn to(&self) -> RedirectTo {
        self.to.clone()
    }

    pub(crate) fn append(&self) -> bool {
        self.append
    }
}

/// The destination of an IO redirection.
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum RedirectTo {
    Descriptor(Descriptor),
    File(String),
}

impl Command {
    fn new(program: String, arguments: Vec<String>, redirects: Vec<Redirect>) -> Self {
        Self {
            program,
            arguments,
            redirects,
        }
    }

    pub(crate) fn program(&self) -> &str {
        &self.program
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub(crate) fn redirects(&self) -> &[Redirect] {
        &self.redirects
    }
}

pub(crate) fn parse_input(input: &str) -> Result<Vec<Command>, ParsingError> {
    let values = quoting::chunk_quoted_string(input)?;

    let commands = splitting::split_commands(values)?;

    Ok(commands)
}
