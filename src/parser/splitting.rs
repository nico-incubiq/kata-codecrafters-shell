use crate::parser::quoting::{split_quoted_string, InputChunk, QuotingError};
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum SplittingError {
    #[error(transparent)]
    Quoting(#[from] QuotingError),

    #[error("Expected program, got: {0}")]
    ProgramExpected(String),

    #[error("Dangling pipe, the command is not terminated")]
    DanglingPipe,

    #[error("Missing redirect destination")]
    MissingRedirectDestination,
    // #[error("Invalid IO descriptor: {0}")]
    // InvalidIoDescriptor(String),
}

/// An IO redirection.
pub(crate) struct Redirect {
    /// The IO descriptor.
    /// 0: input (unsupported), 1: output, 2: error, custom (unsupported)
    descriptor: u8,
    overwrite: bool,
    destination: RedirectDestination,
}

/// The destination of an IO redirection.
#[derive(PartialEq, Debug)]
pub(crate) enum RedirectDestination {
    Descriptor(u8),
    File(String),
}

/// A command with its arguments and redirections in the order they were specified.
pub(crate) struct Command {
    program: String,
    arguments: Vec<String>,
    redirects: Vec<Redirect>,
}

impl Command {
    fn new(program: String, arguments: Vec<String>, redirects: Vec<Redirect>) -> Self {
        Self {
            program,
            arguments,
            redirects,
        }
    }
}

//TODO: test this in another module
// -  echo hello '|' world 2> out.txt 1>&2 : writes to out.txt
// -  echo hello '|' world 1>&2 2> out.txt : writes to stdout, because 1>&2 writes to stderr before the redirection is set up

/// Parses the input string into a list of commands piped into each other.
pub(crate) fn parse_input(input: &str) -> Result<Vec<Command>, SplittingError> {
    let values = split_quoted_string(input)?;

    if values.is_empty() {
        return Ok(vec![]);
    }

    let redirection_regex = Regex::new(r"^(?<from>\d+)?>(?<overwrite>>)?(?<to>&\d+)?$").unwrap();

    let mut commands = vec![];

    let mut current_program: Option<String> = None;
    let mut current_args: Vec<String> = vec![];
    let mut current_redirections: Vec<Redirect> = vec![];

    let mut iter = values.into_iter();
    while let Some(value) = iter.next() {
        match value {
            InputChunk::QuotedText(text) => {
                if current_program.is_none() {
                    current_program = Some(text);
                } else {
                    current_args.push(text);
                }
            }
            InputChunk::RawText(text) => {
                // End the current command and start parsing the next one.
                if text == "|" {
                    if let Some(program) = current_program {
                        commands.push(Command::new(program, current_args, current_redirections));

                        current_program = None;
                        current_args = vec![];
                        current_redirections = vec![];
                    } else {
                        return Err(SplittingError::ProgramExpected(text));
                    }
                } else if let Some(groups) = redirection_regex.captures(&text) {
                    if current_program.is_none() {
                        return Err(SplittingError::ProgramExpected(text));
                    }

                    let descriptor: u8 = match groups.name("from") {
                        // Safe to unwrap as the regex only matches digits.
                        Some(descriptor) => descriptor.as_str().parse().unwrap(),
                        // .map_err(|_| {
                        //     ParsingError::InvalidIoDescriptor(descriptor.as_str().to_string())
                        // })?,
                        None => 1,
                    };

                    let overwrite = groups.name("overwrite").is_some();

                    let destination = match groups.name("to") {
                        Some(descriptor) => {
                            // Safe to unwrap as the regex only matches digits.
                            let descriptor: u8 = descriptor.as_str()[1..].parse().unwrap();
                            // .map_err(|_| {
                            //     ParsingError::InvalidIoDescriptor(descriptor.as_str()[1..].to_string())
                            // })?;
                            RedirectDestination::Descriptor(descriptor)
                        }
                        None => {
                            let filename = match iter
                                .next()
                                .ok_or(SplittingError::MissingRedirectDestination)?
                            {
                                InputChunk::QuotedText(text) => text,
                                InputChunk::RawText(text) => {
                                    if text == "|" || redirection_regex.is_match(&text) {
                                        return Err(SplittingError::MissingRedirectDestination);
                                    }

                                    text
                                }
                            };

                            RedirectDestination::File(filename)
                        }
                    };

                    current_redirections.push(Redirect {
                        descriptor,
                        overwrite,
                        destination,
                    })
                } else {
                    if current_program.is_none() {
                        current_program = Some(text);
                    } else {
                        current_args.push(text);
                    }
                }
            }
        }
    }

    if let Some(program) = current_program {
        commands.push(Command::new(program, current_args, current_redirections));
    } else {
        return Err(SplittingError::DanglingPipe);
    }

    Ok(commands)
}

#[cfg(test)]
mod tests {
    use super::{parse_input, RedirectDestination, SplittingError};

    #[test]
    fn it_parses_single_command_without_redirect() {
        let input = "echo hello";

        let commands = parse_input(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!("echo", commands[0].program);
        assert_eq!(1, commands[0].arguments.len());
        assert_eq!("hello", commands[0].arguments[0]);
    }

    #[test]
    fn it_parses_piped_commands() {
        let input = r#"'echo' hello\nworld | 'grep' "hello""#;

        let commands = parse_input(input).unwrap();

        assert_eq!(2, commands.len());
    }

    #[test]
    fn it_parses_file_redirections() {
        let input = "echo hello > out.txt 2> err.txt";

        let commands = parse_input(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].arguments.len());
        assert_eq!(2, commands[0].redirects.len());
        assert_eq!(1, commands[0].redirects[0].descriptor);
        assert_eq!(
            RedirectDestination::File("out.txt".to_owned()),
            commands[0].redirects[0].destination
        );
        assert_eq!(2, commands[0].redirects[1].descriptor);
        assert_eq!(
            RedirectDestination::File("err.txt".to_owned()),
            commands[0].redirects[1].destination
        );
    }

    #[test]
    fn it_parses_redirections_in_each_piped_command() {
        let input = "echo hello\nworld > first.txt | grep world > second.txt";

        let commands = parse_input(input).unwrap();

        assert_eq!(2, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert_eq!(1, commands[1].redirects.len());
    }

    #[test]
    fn it_parses_descriptor_redirections() {
        let input = "echo hello 1>&2";

        let commands = parse_input(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert_eq!(1, commands[0].redirects[0].descriptor);
        assert_eq!(
            RedirectDestination::Descriptor(2),
            commands[0].redirects[0].destination
        );
    }

    #[test]
    fn it_parses_overwrite_redirections() {
        let input = "echo hello >> out.txt";

        let commands = parse_input(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert!(commands[0].redirects[0].overwrite);
    }

    #[test]
    fn it_ignores_quoted_pipes() {
        let input = "echo hello '|' world";

        let commands = parse_input(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(3, commands[0].arguments.len());
    }

    #[test]
    fn it_rejects_erroneous_inputs() {
        // Starting with a pipe.
        let input = "| echo hello";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::ProgramExpected(found) if found == "|"
        ));

        // Starting with a redirection.
        let input = "2> err.txt echo hello";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::ProgramExpected(found) if found == "2>"
        ));

        // Ending with a pipe.
        let input = "echo hello |";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), SplittingError::DanglingPipe));

        // Missing redirection destination.
        let input = "echo hello >";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));

        // Missing redirection destination.
        let input = "echo hello > | grep world";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));

        // Missing redirection destination.
        let input = "echo hello > 2> err.txt";

        let res = parse_input(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));

        // TODO: Decide if we want to prevent those.
        // // Invalid from descriptor.
        // let input = "echo hello A> out.txt";
        //
        // let res = parse_input(input);
        //
        // assert!(res.is_err());
        // assert!(matches!(
        //     res.err().unwrap(),
        //     ParsingError::InvalidIoDescriptor(a) if a == "A"
        // ));
        //
        // // Invalid destination descriptor.
        // let input = "echo hello >&A out.txt";
        //
        // let res = parse_input(input);
        //
        // assert!(res.is_err());
        // assert!(matches!(
        //     res.err().unwrap(),
        //     ParsingError::InvalidIoDescriptor(a) if a == "A"
        // ));
        //
        // // Missing destination descriptor.
        // let input = "echo hello >& out.txt";
        //
        // let res = parse_input(input);
        //
        // assert!(res.is_err());
        // assert!(matches!(
        //     res.err().unwrap(),
        //     ParsingError::InvalidIoDescriptor(a) if a == ""
        // ));
    }
}
