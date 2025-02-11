use crate::io_redirection::{IoRedirectionError, IoRedirections};
use crate::path::{find_in_path, PathError};
use std::env::VarError;
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum BuiltInCommandError {
    #[error("{0}: not a built-in command")]
    BuiltInCommandNotFound(String),

    #[error("{0}: not found")]
    OsCommandNotFound(String),

    #[error("Not enough arguments, found {found}, expected at least {min}")]
    NotEnoughArguments { found: usize, min: usize },

    #[error("Too many arguments, found {found}, expected at most {max}")]
    TooManyArguments { found: usize, max: usize },

    #[error("Invalid exit code '{0}': {1}")]
    InvalidExitCode(String, ParseIntError),

    #[error(transparent)]
    WriteLineFailed(#[from] IoRedirectionError),

    #[error(transparent)]
    FindInPathFailed(#[from] PathError),

    #[error("Failed to read environment variable: {0}")]
    GetEnvFailed(#[from] VarError),

    #[error("cd: {0}: {1}")]
    ChangeDirectoryFailed(String, #[source] std::io::Error),

    #[error("Failed to determine the current working directory: {0}")]
    GetCurrentDirectoryFailed(#[source] std::io::Error),
}

pub(crate) enum BuiltInCommand {
    ChangeDirectory,
    Echo,
    Exit,
    PrintWorkingDirectory,
    Type,
}

impl TryFrom<&str> for BuiltInCommand {
    type Error = BuiltInCommandError;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        match command {
            "cd" => Ok(BuiltInCommand::ChangeDirectory),
            "echo" => Ok(BuiltInCommand::Echo),
            "exit" => Ok(BuiltInCommand::Exit),
            "pwd" => Ok(BuiltInCommand::PrintWorkingDirectory),
            "type" => Ok(BuiltInCommand::Type),
            _ => Err(BuiltInCommandError::BuiltInCommandNotFound(
                command.to_owned(),
            )),
        }
    }
}

impl BuiltInCommand {
    /// The printable name of the built-in function.
    pub(crate) fn name(&self) -> String {
        match self {
            BuiltInCommand::ChangeDirectory => "cd",
            BuiltInCommand::Echo => "echo",
            BuiltInCommand::Exit => "exit",
            BuiltInCommand::PrintWorkingDirectory => "pwd",
            BuiltInCommand::Type => "type",
        }
        .to_owned()
    }

    /// Runs the built-in command.
    ///
    /// # Note
    /// The run method doesn't accept a stderr argument as it doesn't write to the standard error
    /// under regular circumstances. It any error is encountered, they are returned as error types.
    pub(crate) fn run(
        &self,
        args: &[String],
        io_redirections: &mut IoRedirections,
    ) -> Result<(), BuiltInCommandError> {
        match self {
            BuiltInCommand::ChangeDirectory => {
                let arg = get_single_argument(args)?;

                let working_dir = if arg == "~" {
                    std::env::var("HOME")?
                } else {
                    arg
                };

                std::env::set_current_dir(&working_dir)
                    .map_err(|e| BuiltInCommandError::ChangeDirectoryFailed(working_dir, e))?;
            }
            BuiltInCommand::Echo => {
                io_redirections.writeln(format_args!("{}", args.join(" ")))?;
            }
            BuiltInCommand::Exit => {
                let arg = get_single_argument(args)?;

                let exit_code = arg
                    .parse::<i32>()
                    .map_err(|e| BuiltInCommandError::InvalidExitCode(arg, e))?;

                std::process::exit(exit_code);
            }
            BuiltInCommand::PrintWorkingDirectory => {
                let cwd = std::env::current_dir()
                    .map_err(BuiltInCommandError::GetCurrentDirectoryFailed)?;

                io_redirections.writeln(format_args!("{}", &cwd.display()))?;
            }
            BuiltInCommand::Type => {
                let arg = get_single_argument(args)?;

                if let Ok(sub_command) = BuiltInCommand::try_from(arg.as_ref()) {
                    io_redirections
                        .writeln(format_args!("{} is a shell builtin", sub_command.name()))?;
                } else if let Some(location) = find_in_path(&arg)? {
                    io_redirections.writeln(format_args!("{} is {}", arg, location.display()))?;
                } else {
                    return Err(BuiltInCommandError::OsCommandNotFound(arg));
                }
            }
        };

        Ok(())
    }
}

fn get_single_argument(args: &[String]) -> Result<String, BuiltInCommandError> {
    if args.is_empty() {
        Err(BuiltInCommandError::NotEnoughArguments { min: 1, found: 0 })
    } else if 1 < args.len() {
        Err(BuiltInCommandError::TooManyArguments {
            max: 1,
            found: args.len(),
        })
    } else {
        Ok(args[0].trim().to_owned())
    }
}
