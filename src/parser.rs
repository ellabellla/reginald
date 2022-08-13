use std::{fmt::Display, vec};

use crate::lexer::{Lexer, Token};

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
    node_type: Token,
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

    fn push_token(token: &Token, out: &mut Vec<char>) {
        let name = format!("{:?}", token);
        for c in name.chars() {
            out.push(c);
        }
    }
}


pub fn parse_regex(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    let mut children = vec![parse_value(lexer)?];

    while let Some(Token::Or) =  lexer.peek() {
        lexer.next();
        children.push(parse_value(lexer)?);
    }

    Ok(Box::new(ASTNode{node_type: Token::Or, children}))
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
                regex = Box::new(ASTNode{node_type:Token::ZeroOrMore, children: vec![regex]})
            } else if matches!(next_token, Token::OneOrMore) {
                lexer.next();
                regex = Box::new(ASTNode{node_type:Token::OneOrMore, children: vec![regex]})
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
        return Err(ParseError::new("expected bracketed regex"))
    }

    let res = parse_regex(lexer);
    if res.is_ok() { 
        if let Some(Token::CloseParenthesis) = lexer.next() {
            res
        } else {
            lexer.seek(fallback);
            Err(ParseError::new("expected bracketed regex"))
        }
    } else {
        lexer.seek(fallback);
        Err(ParseError::new("expected regex inside brackets"))
    }
}

fn parse_symbol(lexer: &mut Lexer) -> Result<Box<ASTNode>, ParseError> {
    if let Some(Token::Symbol(c)) = lexer.peek() {
        lexer.next();

        let mut children = vec![];

        while let Ok(child) = parse_symbol(lexer) {
            children.push(child);
        }

        Ok(Box::new(ASTNode{node_type: Token::Symbol(c), children}))
    } else {
        Err(ParseError::new(""))
    }
}





