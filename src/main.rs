#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::Path;

fn main() {
    loop {
        // Print the prompt.
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input.
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        // Split the command and arguments.
        let (command, args) = input.trim().split_once(' ').unwrap_or((input.trim(), ""));

        match command {
            "exit" if args == "0" => break,
            "echo" => println!("{}", args),
            "type" if !args.is_empty() => match args {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", args),
                _ => type_command(args),
            },
            _ => {
                eprintln!("{}: command not found", command);
            }
        }
    }
}

fn type_command(args: &str) {
    // Load the PATH env variable.
    let path = std::env::var("PATH").unwrap();
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    if let Some(location) = directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(args)).filter(|location| location.exists()))
    {
        println!("{} is {}", args, location.display());
    } else {
        eprintln!("{}: not found", args);
    }
}
