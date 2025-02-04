#[allow(unused_imports)]
use std::io::{self, Write};

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
        let mut command = input
            .trim()
            .split_once(' ')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or_else(|| (input.trim(), None));

        match command {
            ("exit", Some("0")) => break,
            ("echo", args) => {
                println!("{}", args.unwrap_or_default());
            }
            ("type", Some(cmd)) => {
                if ["echo", "exit", "type"].contains(&cmd) {
                    println!("{} is a shell builtin", cmd);
                } else {
                    command_not_found(cmd);
                }
            }
            (cmd, _) => {
                command_not_found(cmd);
            }
        }
    }
}

fn command_not_found(command: &str) {
    eprintln!("{}: command not found", command);
}
