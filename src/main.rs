#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        // Print the prompt.
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let mut command = input.trim().split_ascii_whitespace();
        match command.next() {
            Some("exit") if command.next().is_some_and(|value| value == "0") => break,
            Some("echo") => {
                println!("{}", command.collect::<Vec<_>>().join(" "));
            }
            _ => {
                eprintln!("{}: command not found", input.trim());
            }
        }
    }
}
