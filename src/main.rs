mod arguments;
mod builtin;
mod path;

use crate::arguments::parse_args;
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
    let (command, args_string) = input
        .split_once(' ')
        .map(|(cmd, args)| (cmd.to_owned(), args))
        .unwrap_or((input.to_owned(), ""));

    let args = parse_args(args_string)?;

    Ok((command, args))
}
