mod builtin;
mod path;

use crate::builtin::BuiltInCommand;
use crate::path::run_binary;
use std::io::Write;

fn main() {
    loop {
        if let Err(error) = repl() {
            eprintln!("{}", error);
        }
    }
}

fn repl() -> Result<(), String> {
    // Print the prompt and grab the input.
    let input = input_prompt()?;

    // Split the command and arguments.
    let (command, args) = parse_input(&input)?;

    // Interpret the command name and run it.
    match BuiltInCommand::try_from(command.as_ref()) {
        Ok(built_in) => built_in.run(&args),
        _ => run_binary(&command, &args),
    }?;

    Ok(())
}

fn input_prompt() -> Result<String, String> {
    // Print the prompt.
    print!("$ ");
    std::io::stdout()
        .flush()
        .map_err(|e| format!("Failed to print the prompt: {:?}", e))?;

    // Wait for user input.
    let stdin = std::io::stdin();
    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {:?}", e))?;

    Ok(input.trim().to_owned())
}

fn parse_input(input: &str) -> Result<(String, Vec<String>), String> {
    // Split command from arguments.
    let (command, args) = input
        .split_once(' ')
        .map(|(cmd, args)| (cmd.to_owned(), args))
        .unwrap_or((input.to_owned(), ""));

    // Split arguments separated by spaces, apart if they are single-quoted.
    let mut split_args = Vec::new();
    let mut current = "".to_owned();
    let mut is_within_quotes = false;
    for char in args.chars() {
        if !is_within_quotes && char.is_whitespace() && !current.is_empty() {
            split_args.push(current);
            current = "".to_owned();
        } else if char == '\'' {
            is_within_quotes = !is_within_quotes;
        } else if is_within_quotes || !char.is_whitespace() {
            current.push(char);
        }
    }

    if is_within_quotes {
        return Err("Invalid single-quoting".to_owned());
    } else if !current.is_empty() {
        split_args.push(current);
    }

    Ok((command, split_args))
}

#[cfg(test)]
mod tests {
    use crate::parse_input;

    #[test]
    fn it_splits_command_from_args() {
        // Split at spaces.
        assert_eq!(
            Ok(("hello".to_owned(), ["world".to_owned()].to_vec())),
            parse_input("hello world")
        );
        assert_eq!(
            Ok(("hello".to_owned(), ["world".to_owned()].to_vec())),
            parse_input("hello       world")
        );

        // Don't split single-quoted strings.
        assert_eq!(
            Ok((
                "hello".to_owned(),
                ["to the world", "from ", "me"].map(str::to_owned).to_vec()
            )),
            parse_input("hello 'to the world'     'from ' me")
        );

        // Skip single quotes if not separated by spaces.
        assert_eq!(
            Ok(("hello".to_owned(), ["world".to_owned()].to_vec())),
            parse_input("hello w'orl'd")
        );
        assert_eq!(
            Ok(("hello".to_owned(), ["world".to_owned()].to_vec())),
            parse_input("hello 'worl'd")
        );
        assert_eq!(
            Ok(("hello".to_owned(), ["world oh".to_owned()].to_vec())),
            parse_input("hello wo'rld 'oh")
        );
    }
}
