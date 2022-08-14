use std::{fmt::Display, vec};

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub enum SyntaxType {
    ZeroOrMore,
    Optional,
    OneOrMore,
    Once,
    Or,
    Symbol(char),
}

/*
L(Ø) = {}
L(ε) = {}
L(a) = {a} for all a ∈ Σ
L(R1|R2) = L(R1) ∪ L(R2)
L(R1R2) = L(R1)L(R2)
L(R*) = L(R*) = {∈} U L(R) U L(R) U L(R)...
*/

#[derive(Debug)]
pub struct ParseError {
    msg: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.msg))
    }
}

impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        ParseError { msg: msg.to_string() }
    }
}

pub struct ASTNode {
    node_type: SyntaxType,
    children: Vec<Box<ASTNode>>,
}

impl ASTNode {
    pub fn to_string(&self) -> String{
        let mut out = vec![];

        out.push('(');

        for child in &self.children {
            ASTNode::to_string_helper(child, &mut out);
        }

        ASTNode::push_token(&self.node_type, &mut out);
        out.push(')');

        out.iter().collect()
    }


    fn to_string_helper(node: &Box<ASTNode>, out: &mut Vec<char>) {
        out.push('(');


        for child in &node.children {
            ASTNode::to_string_helper(child, out);
        }

        ASTNode::push_token(&node.node_type, out);
        out.push(')');
    }

    fn push_token(token: &SyntaxType, out: &mut Vec<char>) {
        let name = format!("{:?}", token);
        for c in name.chars() {
            out.push(c);
        }
    }
}


pub fn parse_regex(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    let mut children = vec![parse_concat(lexer)?];

    while let Some(Token::Or) =  lexer.peek() {
        lexer.next();
        children.push(parse_concat(lexer)?);
    }

    if children.len() == 1 {
        Ok(children.pop().unwrap())
    } else {
        Ok(Box::new(ASTNode{node_type: SyntaxType::Or, children}))
    }
}

fn parse_concat(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    let mut children = vec![parse_value(lexer)?];

    while let Ok(child) = parse_value(lexer) {
        children.push(child);
    }

    Ok(Box::new(ASTNode { node_type: SyntaxType::Once, children }))
}

fn parse_value(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    let fallback = lexer.pos();

    let mut regex = parse_symbol(lexer)
    .or_else(|_| {
        lexer.seek(fallback);
        parse_bracketed(lexer)
    })?;


    loop {
        let next_token = lexer.peek();
        if let Some(next_token) = next_token {
            if matches!(next_token, Token::ZeroOrMore) {
                lexer.next();
                regex = Box::new(ASTNode{node_type:SyntaxType::ZeroOrMore, children: vec![regex]})
            } else if matches!(next_token, Token::OneOrMore) {
                lexer.next();
                regex = Box::new(ASTNode{node_type:SyntaxType::OneOrMore, children: vec![regex]})
            } else if matches!(next_token, Token::Optional) {
                lexer.next();
                regex = Box::new(ASTNode{node_type:SyntaxType::Optional, children: vec![regex]})
            } else {
                return Ok(regex)
            }
        } else {
            return Ok(regex)
        }
    }

}

fn parse_bracketed(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    let fallback = lexer.pos();

    if let Some(Token::OpenParenthesis) = lexer.peek() {
        lexer.next();
    } else {
        return Err(ParseError::new("expected parenthesis"))
    }

    let res = parse_regex(lexer);
    if res.is_ok() { 
        if let Some(Token::CloseParenthesis) = lexer.next() {
            res
        } else {
            lexer.seek(fallback);
            Err(ParseError::new("expected regex"))
        }
    } else {
        lexer.seek(fallback);
        Err(ParseError::new("expected closing parenthesis"))
    }
}

fn parse_symbol(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    if let Some(Token::Symbol(c)) = lexer.peek() {
        lexer.next();

        Ok(Box::new(ASTNode{node_type: SyntaxType::Symbol(c), children: vec![]}))
    } else {
        Err(ParseError::new("expected symbol"))
    }
}
#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use super::parse_regex;

    #[test]
    fn test() {
        assert_eq!(parse("abcd"), "((Symbol('a'))(Symbol('b'))(Symbol('c'))(Symbol('d'))Once)");
        assert_eq!(parse("(ab)cd"), "(((Symbol('a'))(Symbol('b'))Once)(Symbol('c'))(Symbol('d'))Once)");
        assert_eq!(parse("a*+c*d+e?"), "((((Symbol('a'))ZeroOrMore)OneOrMore)((Symbol('c'))ZeroOrMore)((Symbol('d'))OneOrMore)((Symbol('e'))Optional)Once)");
        assert_eq!(parse("(ab)*cd+"), "((((Symbol('a'))(Symbol('b'))Once)ZeroOrMore)(Symbol('c'))((Symbol('d'))OneOrMore)Once)");
        assert_eq!(parse("ab|cd"), "(((Symbol('a'))(Symbol('b'))Once)((Symbol('c'))(Symbol('d'))Once)Or)");
        assert_eq!(parse("(a)+b|c*d"), "(((((Symbol('a'))Once)OneOrMore)(Symbol('b'))Once)(((Symbol('c'))ZeroOrMore)(Symbol('d'))Once)Or)");
    }

    fn parse(string:&str) -> String {
        parse_regex(&mut Lexer::new(string)).unwrap().to_string()
    }
}


