use std::io::Write;

use lexer::{lexer::Lexer, utils::Reader};

fn main() {
    loop {
        print!("\n> ");
        let _ = std::io::stdout().flush();

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("to read line");

        let mut reader = Reader::new(line.as_bytes());
        let mut lexer = Lexer::new(&mut reader);

        loop {
            let lexeme = lexer.next_lexeme();
            println!("{lexeme:?}");

            if let Err(_) | Ok(None) = lexeme {
                break;
            }
        }
    }
}
