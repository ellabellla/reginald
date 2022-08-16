#[derive(Debug, PartialEq, Clone)]
pub enum SetSymbol {
    Char(char),
    Range(u32, u32),
}

impl SetSymbol {
    #[cfg(test)]
    pub fn to_string(&self) -> String {
        match self {
            SetSymbol::Char(c) => format!("'{}'", c),
            SetSymbol::Range(start, end) => format!("'{}'-'{}'", start, end),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    ZeroOrMore,
    Optional,
    OneOrMore,
    Or,
    OpenParenthesis,
    CloseParenthesis,
    From(usize),
    To(usize),
    Between(usize, usize),
    Symbol(char),
    Set(Vec<SetSymbol>),
    NotSet(Vec<SetSymbol>),
    Any,
}

trait ComplexParse {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token>;
}

const PARSE_TABLE: [(char, Token); 7] = [
    ('*', Token::ZeroOrMore),
    ('?', Token::Optional),
    ('+', Token::OneOrMore),
    ('|', Token::Or),
    ('(', Token::OpenParenthesis),
    (')', Token::CloseParenthesis),
    ('.', Token::Any),
];

const COMPLEX_PARSE_TABLE: [(char, &dyn ComplexParse); 5] = [
    ('{', &ParseFrom{}),
    ('{', &ParseTo{}),
    ('{', &ParseBetween{}),
    ('[', &ParseNotSet{}),
    ('[', &ParseSet{}),
];

struct ParseFrom {}
impl ComplexParse for ParseFrom {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token> {
        let fallback = lexer.pos();
        let num = lexer.parse_number()?;

        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != ',' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != '}' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        Some(Token::From(num))
    }
}

struct ParseTo {}
impl ComplexParse for ParseTo {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token> {
        let fallback = lexer.pos();
        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != ',' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        lexer.consume_whitespace();

        let num = lexer.parse_number()?;

        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != '}' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        Some(Token::To(num))
    }
}

struct ParseBetween {}
impl ComplexParse for ParseBetween {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token> {
        let fallback = lexer.pos();
        lexer.consume_whitespace();

        let num_a = lexer.parse_number()?;

        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != ',' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        lexer.consume_whitespace();

        let num_b = lexer.parse_number()?;

        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != '}' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        Some(Token::Between(num_a, num_b))
    }
}

struct ParseSet {}
impl ComplexParse for ParseSet {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token> {
        let fallback = lexer.pos();
        let mut data = vec![];

        lexer.consume_whitespace();

        while let Some(c) = lexer.data.get(lexer.index) {
            if *c == ']' {
                if data.len() == 0 {
                    lexer.seek(fallback);
                    return None
                } else {
                    lexer.index += 1;
                    return Some(Token::Set(data))
                }
            } else if *c == '-' {
                if data.len() == 0 || lexer.index == data.len() - 1 {
                    lexer.seek(fallback);
                    return None
                } else {
                    match data.last_mut().unwrap() {
                        SetSymbol::Char(c) => {
                            lexer.index += 1;
                            let symbol = SetSymbol::Range(*c as u32, *lexer.data.get(lexer.index).unwrap() as u32);
                            *data.last_mut().unwrap() = symbol;
                        },
                        SetSymbol::Range(_, _) => {
                            lexer.seek(fallback);
                            return None
                        },
                    }
                }
            } else {
                data.push(SetSymbol::Char(*c))
            }
            lexer.index += 1;
            lexer.consume_whitespace();
        }

        lexer.seek(fallback);
        None
    }
}

struct ParseNotSet {}
impl ComplexParse for ParseNotSet {
    fn parse(&self, lexer: &mut Lexer) -> Option<Token> {
        let fallback = lexer.pos();
        lexer.consume_whitespace();

        let next = lexer.data.get(lexer.index)?; 
        if *next != '^' {
            lexer.seek(fallback);
            return None
        } else {
            lexer.index += 1
        }

        let set = ParseSet{}.parse(lexer);
        match set {
            Some(Token::Set(set)) => return Some(Token::NotSet(set)),
            _ => {
                lexer.seek(fallback);
                return None
            }
        }
    }
}

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
            let c = c.clone();
            self.index += 1;

            for (token_char, parser) in COMPLEX_PARSE_TABLE {
                if c == token_char {
                    if let Some(token) = parser.parse(self) {
                        return Some(token)
                    }
                }
            }

            for (token_char, token) in PARSE_TABLE {
                if c == token_char {
                    return Some(token.clone())
                }
            }
            Some(Token::Symbol(c))
        } else {
            self.index += 1;

            None
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.data.get(self.index) {
            if *c != '\t' && *c != '\r' &&
                *c != '\n' {
                    break;
            } 

            self.index += 1;
        }
    }

    fn parse_number(&mut self) -> Option<usize> {
        let fallback = self.pos();
        self.consume_whitespace();
        let mut data = vec![];

        while let Some(c) = self.data.get(self.index) {
            if c.is_ascii_digit() {
                data.push(*c);

                self.index += 1;
            } else {
                break;
            }
        }

        if data.len() == 0 {
            self.seek(fallback);
            None
        } else if let Ok(number) = data.iter().collect::<String>().parse::<usize>() {
            Some(number)
        } else {
            self.seek(fallback);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token, SetSymbol};

    #[test] 
    fn test() {
        let mut lexer = Lexer::new("1(0)*+|a5{,2}{2,}{2,6}[ab-z][^ab-z].");
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
            Token::To(2),
            Token::From(2),
            Token::Between(2,6),
            Token::Set(vec![SetSymbol::Char('a'), SetSymbol::Range('b' as u32, 'z' as u32)]),
            Token::NotSet(vec![SetSymbol::Char('a'), SetSymbol::Range('b' as u32, 'z' as u32)]),
        ];

        for expected in expected_tokens {
            assert_eq!(lexer.peek().unwrap(), expected);
            assert_eq!(lexer.next().unwrap(), expected);
        }
    }
}

