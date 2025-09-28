use crate::parser::quoting::InputChunk;
use crate::parser::{Command, Descriptor, Redirect, RedirectTo};
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum SplittingError {
    #[error("Expected program, got: {0}")]
    ProgramExpected(String),

    #[error("Dangling pipe, the command is not terminated")]
    DanglingPipe,

    #[error("Missing redirect destination")]
    MissingRedirectDestination,
}

/// Parses the input string into a list of commands piped into each other.
pub(crate) fn split_commands(chunks: Vec<InputChunk>) -> Result<Vec<Command>, SplittingError> {
    if chunks.is_empty() {
        return Ok(vec![]);
    }

    let redirection_regex = Regex::new(r"^(?<from>\d+)?>(?<append>>)?(?<to>&\d+)?$").unwrap();

    let mut commands = vec![];

    let mut current_program: Option<String> = None;
    let mut current_args: Vec<String> = vec![];
    let mut current_redirections: Vec<Redirect> = vec![];

    let mut iter = chunks.into_iter();
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

                    let descriptor_id: u8 = groups
                        .name("from")
                        // Safe to unwrap as the regex only matches digits.
                        .map_or(1, |m| m.as_str().parse().unwrap());

                    let append = groups.name("append").is_some();

                    let destination = if let Some(descriptor) = groups.name("to") {
                        // Safe to unwrap as the regex only matches digits.
                        let descriptor_id: u8 = descriptor.as_str()[1..].parse().unwrap();
                        RedirectTo::Descriptor(Descriptor(descriptor_id))
                    } else {
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

                        RedirectTo::File(filename)
                    };

                    current_redirections.push(Redirect {
                        from: Descriptor(descriptor_id),
                        append,
                        to: destination,
                    });
                } else if current_program.is_none() {
                    current_program = Some(text);
                } else {
                    current_args.push(text);
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
    use super::{split_commands, RedirectTo, SplittingError};
    use crate::parser::quoting::InputChunk;
    use crate::parser::Descriptor;

    fn raw(text: &str) -> InputChunk {
        InputChunk::RawText(text.to_owned())
    }

    fn quoted(text: &str) -> InputChunk {
        InputChunk::QuotedText(text.to_owned())
    }

    #[test]
    fn it_parses_single_command_without_redirect() {
        let input = vec![raw("echo"), raw("hello")];

        let commands = split_commands(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!("echo", commands[0].program);
        assert_eq!(1, commands[0].arguments.len());
        assert_eq!("hello", commands[0].arguments[0]);
    }

    #[test]
    fn it_parses_piped_commands() {
        let input = vec![
            quoted("echo"),
            raw("hello\nworld"),
            raw("|"),
            quoted("grep"),
            quoted("hello"),
        ];

        let commands = split_commands(input).unwrap();

        assert_eq!(2, commands.len());
    }

    #[test]
    fn it_parses_file_redirections() {
        let input = vec![
            raw("echo"),
            raw("hello"),
            raw(">"),
            raw("out.txt"),
            raw("2>"),
            raw("err.txt"),
        ];

        let commands = split_commands(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].arguments.len());
        assert_eq!(2, commands[0].redirects.len());
        assert_eq!(Descriptor(1), commands[0].redirects[0].from);
        assert_eq!(
            RedirectTo::File("out.txt".to_owned()),
            commands[0].redirects[0].to
        );
        assert_eq!(Descriptor(2), commands[0].redirects[1].from);
        assert_eq!(
            RedirectTo::File("err.txt".to_owned()),
            commands[0].redirects[1].to
        );
    }

    #[test]
    fn it_parses_redirections_in_each_piped_command() {
        let input = vec![
            raw("echo"),
            raw("hello\nworld"),
            raw(">"),
            raw("first.txt"),
            raw("|"),
            raw("grep"),
            raw("world"),
            raw(">"),
            raw("second.txt"),
        ];

        let commands = split_commands(input).unwrap();

        assert_eq!(2, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert_eq!(1, commands[1].redirects.len());
    }

    #[test]
    fn it_parses_descriptor_redirections() {
        let input = vec![raw("echo"), raw("hello"), raw("1>&2")];

        let commands = split_commands(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert_eq!(Descriptor(1), commands[0].redirects[0].from);
        assert_eq!(
            RedirectTo::Descriptor(Descriptor(2)),
            commands[0].redirects[0].to
        );
    }

    #[test]
    fn it_parses_append_redirections() {
        let input = vec![raw("echo"), raw("hello"), raw(">>"), raw("out.txt")];

        let commands = split_commands(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(1, commands[0].redirects.len());
        assert!(commands[0].redirects[0].append);
    }

    #[test]
    fn it_ignores_quoted_pipes() {
        let input = vec![raw("echo"), raw("hello"), quoted("|"), raw("world")];

        let commands = split_commands(input).unwrap();

        assert_eq!(1, commands.len());
        assert_eq!(3, commands[0].arguments.len());
    }

    #[test]
    fn it_rejects_erroneous_inputs() {
        // Starting with a pipe.
        let input = vec![raw("|"), raw("echo"), raw("hello")];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::ProgramExpected(found) if found == "|"
        ));

        // Starting with a redirection.
        let input = vec![raw("2>"), raw("err.txt"), raw("echo"), raw("hello")];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::ProgramExpected(found) if found == "2>"
        ));

        // Ending with a pipe.
        let input = vec![raw("echo"), raw("hello"), raw("|")];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(res.err().unwrap(), SplittingError::DanglingPipe));

        // Missing redirection destination.
        let input = vec![raw("echo"), raw("hello"), raw(">")];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));

        // Missing redirection destination.
        let input = vec![
            raw("echo"),
            raw("hello"),
            raw(">"),
            raw("|"),
            raw("grep"),
            raw("world"),
        ];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));

        // Missing redirection destination.
        let input = vec![
            raw("echo"),
            raw("hello"),
            raw(">"),
            raw("2>"),
            raw("err.txt"),
        ];

        let res = split_commands(input);

        assert!(res.is_err());
        assert!(matches!(
            res.err().unwrap(),
            SplittingError::MissingRedirectDestination
        ));
    }
}
