mod builtin;
mod io_redirection;
mod path;
mod quoting;

use crate::builtin::BuiltInCommand;
use crate::io_redirection::handle_io_redirections;
use crate::path::run_binary;
use crate::quoting::split_quoted_string;
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
    let (command, mut args) = parse_input(&input)?;

    // Get the standard output / error descriptors to execute the command.
    let mut io_redirections = handle_io_redirections(&mut args)?;

    // Interpret the command name and run it.
    match BuiltInCommand::try_from(command.as_ref()) {
        Ok(built_in) => built_in.run(&args, &mut io_redirections),
        _ => run_binary(&command, &args, &mut io_redirections),
    }
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

/// Parse the input string into a command and its arguments.
fn parse_input(input: &str) -> Result<(String, Vec<String>), String> {
    let mut values = split_quoted_string(input)?;

    if values.is_empty() {
        Err("Empty input".to_owned())
    } else {
        Ok((values.remove(0), values))
    }
}
