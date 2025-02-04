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

        match input.trim() {
            "exit 0" => break,
            _ => {
                eprintln!("{}: command not found", input.trim());
            }
        }
    }
}
