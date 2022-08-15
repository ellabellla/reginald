use regex::Regex;

mod lexer;
mod parser;
mod regex;

fn main() {
    let _regex = Regex::compile("a*b");
}
