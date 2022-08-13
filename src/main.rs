use lexer::Lexer;
use parser::parse_regex;

mod lexer;
mod parser;

fn main() {
    let mut lexer = Lexer::new("a+*");
    let _ast = parse_regex(&mut lexer).unwrap();

    println!("{}", _ast.to_string());
}
