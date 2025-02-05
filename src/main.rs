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
        let (command, args) = input.trim().split_once(' ').unwrap_or((input.trim(), ""));

        match command {
            "exit" if args == "0" => break,
            "echo" => println!("{}", args),
            "type" => match args {
                "exit" | "echo" | "type" => println!("{} is a shell builtin", args),
                _ => eprintln!("{}: not found", args),
            },
            _ => {
                eprintln!("{}: command not found", command);
            }
        }
    }
}
