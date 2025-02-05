use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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
        let (command, args) = input
            .trim()
            .split_once(' ')
            .map(|(cmd, args)| (cmd, Some(args)))
            .unwrap_or((input.trim(), None));

        match command {
            "exit" if args.is_some_and(|arg| arg == "0") => break,
            "echo" => println!("{}", args.unwrap_or_default()),
            "type" if !args.is_none() => match args.unwrap() {
                "exit" | "echo" | "type" => {
                    println!("{} is a shell builtin", args.unwrap())
                }
                _ => type_command(args.unwrap()),
            },
            _ => run_command(command, args),
        }
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    // Load the PATH env variable.
    let path = std::env::var("PATH").unwrap();
    let directories = path.split(':');

    // Check whether the file exists in any of the directories.
    directories
        .into_iter()
        .find_map(|dir| Some(Path::new(dir).join(name)).filter(|location| location.exists()))
}

fn type_command(args: &str) {
    if let Some(location) = find_in_path(args) {
        println!("{} is {}", args, location.display());
    } else {
        eprintln!("{}: not found", args);
    }
}

fn run_command(cmd: &str, args: Option<&str>) {
    if let Some(location) = find_in_path(cmd) {
        let mut command = Command::new(location);
        if let Some(args) = args {
            command.args(args.split_ascii_whitespace());
        }

        // Start the program in a thread and wait for it to finish.
        command.spawn().unwrap().wait().unwrap();
    } else {
        eprintln!("{}: command not found", cmd);
    }
}
