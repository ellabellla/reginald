
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ZeroOrMore,
    OneOrMore,
    Or,
    OpenParenthesis,
    CloseParenthesis,
    Symbol(char),
}

const PARSE_TABLE: [(char, Token); 5] = [
    ('*', Token::ZeroOrMore),
    ('+', Token::OneOrMore),
    ('|', Token::Or),
    ('(', Token::OpenParenthesis),
    (')', Token::CloseParenthesis),
];


pub struct Lexer{
    data: Vec<char>,
    index: usize,
}

impl<'a> Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next()
    }
}

impl Lexer {
    pub fn new(data: &str) -> Lexer {
        Lexer { data: data.chars().collect(), index: 0 }
    }

    pub fn pos(&self) -> usize {
        self.index
    }

    pub fn seek(&mut self, pos: usize) {
        self.index = pos;
    }

    pub fn peek(&mut self) -> Option<Token> {
        let prev_index = self.index;

        let token = self.parse_next();

        self.index = prev_index;

        token
    }

    fn parse_next(&mut self) -> Option<Token> {
        self.consume_whitespace();

        if let Some(c) = self.data.get(self.index) {
            self.index += 1;

            for (token_char, token) in PARSE_TABLE {
                if *c == token_char {
                    return Some(token.clone())
                }
            }
            Some(Token::Symbol(*c))
        } else {
            self.index += 1;

            None
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.data.get(self.index) {
            if *c != ' ' && *c != '\t' && *c != '\r' &&
                *c != '\n' {
                    break;
            } 

            self.index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token};

    #[test] 
    fn test() {
        let mut lexer = Lexer::new("1(0)*+|a5");
        let expected_tokens = [
            Token::Symbol('1'), 
            Token::OpenParenthesis, 
            Token::Symbol('0'), 
            Token::CloseParenthesis,
            Token::ZeroOrMore,
            Token::OneOrMore,
            Token::Or,
            Token::Symbol('a'),
            Token::Symbol('5'),
        ];

        for expected in expected_tokens {
            assert_eq!(lexer.peek().unwrap(), expected);
            assert_eq!(lexer.next().unwrap(), expected);
        }
    }
}

