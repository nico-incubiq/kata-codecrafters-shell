mod builtin;

use crate::builtin::BuiltInCommand;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    loop {
        // Print the prompt and grab the input.
        let input = match input_prompt() {
            Ok(input) => input,
            Err(e) => {
                eprintln!("{}", e);

                continue;
            }
        };

        // Split the command and arguments.
        let (command, args) = input
            .split_once(' ')
            .map(|(cmd, args)| (cmd, Some(args)))
            .unwrap_or((&input, None));

        // Interpret the command name and run it.
        let _ = match BuiltInCommand::try_from(command) {
            Ok(built_in) => built_in.run(args),
            _ => run_binary(command, args),
        }
        .map_err(|e| eprintln!("{}", e));
    }
}

fn input_prompt() -> Result<String, String> {
    // Print the prompt.
    print!("$ ");
    io::stdout()
        .flush()
        .map_err(|e| format!("Failed to print the prompt: {:?}", e))?;

    // Wait for user input.
    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read input: {:?}", e))?;

    Ok(input.trim().to_owned())
}

fn find_binary_in_path(name: &str) -> Result<Option<PathBuf>, String> {
    // Load the PATH env variable.
    let path =
        std::env::var("PATH").map_err(|e| format!("Invalid PATH environment variable: {:?}", e))?;
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    let location = directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(name)).filter(|location| location.exists()));

    Ok(location)
}

fn print_type_of_binary(args: &str) -> Result<(), String> {
    if let Some(location) = find_binary_in_path(args)? {
        println!("{} is {}", args, location.display());

        Ok(())
    } else {
        Err(format!("{}: not found", args))
    }
}

fn run_binary(cmd: &str, args: Option<&str>) -> Result<(), String> {
    if find_binary_in_path(cmd)?.is_some() {
        let mut command = Command::new(cmd);
        if let Some(args) = args {
            command.args(args.split_ascii_whitespace());
        }

        // Start the program in a thread and wait for it to finish.
        command.status().map(|_| {}).map_err(|e| format!("{:?}", e))
    } else {
        Err(format!("{}: command not found", cmd))
    }
}
