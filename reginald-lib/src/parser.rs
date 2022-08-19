use std::{fmt::Display, vec};

use crate::lexer::{Lexer, Token, SetSymbol};

#[derive(Debug)]
pub enum SyntaxType {
    ZeroOrMore,
    Optional,
    OneOrMore,
    Once,
    Or,
    From(usize),
    To(usize),
    Between(usize, usize),
    Symbol(char),
    Set(Vec<SetSymbol>),
    NotSet(Vec<SetSymbol>),
    Any,
}

impl SyntaxType {
    #[cfg(test)]
    pub fn to_string(&self) -> String {
        match self {
            SyntaxType::ZeroOrMore => "ZeroOrMore".to_string(),
            SyntaxType::Optional => "Optional".to_string(),
            SyntaxType::OneOrMore => "OneOrMore".to_string(),
            SyntaxType::Once => "Once".to_string(),
            SyntaxType::Or => "Or".to_string(),
            SyntaxType::From(min) => format!("From {}", min),
            SyntaxType::To(max) => format!("To {}", max),
            SyntaxType::Between(min, max) => format!("Between {} and {}", min, max),
            SyntaxType::Symbol(char) => format!("Symbol {}", char),
            SyntaxType::Set(set) => set.iter().map(|symbol| symbol.to_string()).collect::<Vec<String>>().join(", "),
            SyntaxType::NotSet(set) => format!("not {}",set.iter().map(|symbol| symbol.to_string()).collect::<Vec<String>>().join(", ")),
            SyntaxType::Any => "Any".to_string(),
        }
    }
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

pub struct AST {
    pub nodes: Box<Vec<ASTNode>>,
    pub start_node: usize,
}

impl AST {
    #[cfg(test)]
    pub fn to_string(&self, lisp: bool) -> String{
        if lisp {
            let mut out = vec![];
            let start_node = self.nodes.get(self.start_node).unwrap();
    
            out.push('(');
    
            for child in &start_node.children {
                self.to_string_helper(self.nodes.get(*child).unwrap(), &mut out);
            }
    
            AST::push_token(&start_node.node_type, &mut out);
            out.push(')');
    
            out.iter().collect()
        } else {
            let mut out = vec![];
            AST::push_string("```mermaid\n", &mut out);
            AST::push_string("flowchart LR\n", &mut out);
    
            for (i, node) in self.nodes.as_ref().iter().enumerate() {
                AST::push_string(&format!("\t{}({})\n", i, node.node_type.to_string()), &mut out);
    
                for nex_node in &node.children {
                    AST::push_string(&format!("\t{}-->{}\n", i, nex_node), &mut out);
                }
            }
    
            AST::push_string("```\n", &mut out);
            
            out.iter().collect()
        }
    }

    #[cfg(test)]
    fn to_string_helper(&self, node: &ASTNode, out: &mut Vec<char>) {
        out.push('(');


        for child in &node.children {
            self.to_string_helper(self.nodes.get(*child).unwrap(), out);
        }

        AST::push_token(&node.node_type, out);
        out.push(')');
    }

    #[cfg(test)]
    fn push_token(token: &SyntaxType, out: &mut Vec<char>) {
        let name = format!("{:?}", token);
        for c in name.chars() {
            out.push(c);
        }
    }

    #[cfg(test)]
    fn push_string(string: &str, out: &mut Vec<char>) {
        for c in string.chars() {
            out.push(c);
        }
    }
}

pub struct ASTNode {
    pub node_type: SyntaxType,
    pub children: Vec<usize>,
}

fn push_node(nodes: &mut Box<Vec<ASTNode>>, node: ASTNode) -> usize{
    nodes.push(node);
    nodes.len() - 1
}

fn create_fallback(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> (usize, usize) {
    (lexer.pos(), nodes.len())
}

fn use_fallback(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>, fallback: (usize, usize)) {
    let (pos, len) = fallback;
    lexer.seek(pos);
    nodes.truncate(len);
}

pub fn parse_regex(lexer: &mut Lexer) -> Result<AST, ParseError> {
    let mut nodes = Box::new(vec![]);
    
    let start_node = parse_regex_helper(lexer, &mut nodes)?;

    if let Some(_) = lexer.peek() {
        Err(ParseError::new("unknown symbol"))
    } else {
        Ok(AST{nodes, start_node})
    }
}

fn parse_regex_helper(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    if let Ok(child) = parse_or(lexer, nodes) {
        Ok(push_node(nodes, ASTNode{node_type: SyntaxType::Once, children: vec![child]}))
    } else {
        parse_concat(lexer, nodes)
    }
}

fn parse_or(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    let fallback = create_fallback(lexer, nodes);

    let mut children = vec![parse_concat(lexer, nodes)?];

    while let Some(Token::Or) =  lexer.peek() {
        lexer.next();
        children.push(parse_concat(lexer, nodes)?);
    }

    if children.len() == 1 {
        use_fallback(lexer, nodes, fallback);
        Err(ParseError::new("expected or"))
    } else {
        Ok(push_node(nodes, ASTNode{node_type: SyntaxType::Or, children}))
    }
}

fn parse_concat(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    let mut children = vec![parse_value(lexer, nodes)?];

    while let Ok(child) = parse_value(lexer, nodes) {
        children.push(child);
    }

    Ok(push_node(nodes, ASTNode { node_type: SyntaxType::Once, children }))
}

fn parse_value(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    let fallback = create_fallback(lexer, nodes);

    let mut regex = parse_symbol(lexer, nodes)
    .or_else(|_| {
        use_fallback(lexer, nodes, fallback);
        parse_bracketed(lexer, nodes)
    })?;


    let next_token = lexer.peek();
    if let Some(next_token) = next_token {
        match next_token {
            Token::ZeroOrMore => {
                lexer.next();
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::ZeroOrMore, children: vec![regex]})
            },
            Token::Optional => {
                lexer.next();
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::Optional, children: vec![regex]})
            },
            Token::OneOrMore => {
                lexer.next();
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::OneOrMore, children: vec![regex]})
            },
            Token::From(num) => {
                lexer.next();
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::From(num), children: vec![regex]})
            },
            Token::To(num) => {
                lexer.next();
                if num == 0 {
                    return Err(ParseError::new("to must be greater than 0 in range"))
                }
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::To(num), children: vec![regex]})
            },
            Token::Between(from, to) => {
                lexer.next();
                if from > to {
                    return Err(ParseError::new("from must be lower or equal to to in range"))
                } else if to == 0 {
                    return Err(ParseError::new("to must be greater than 0 in range"))
                }
                regex = push_node(nodes, ASTNode{node_type:SyntaxType::Between(from, to), children: vec![regex]})
            },
            _ => (),
        }
    }

    Ok(regex)
}

fn parse_bracketed(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    let fallback = create_fallback(lexer, nodes);

    if let Some(Token::OpenParenthesis) = lexer.peek() {
        lexer.next();
    } else {
        return Err(ParseError::new("expected parenthesis"))
    }

    let res = parse_regex_helper(lexer, nodes);
    if res.is_ok() { 
        if let Some(Token::CloseParenthesis) = lexer.next() {
            res
        } else {
            use_fallback(lexer, nodes, fallback);
            Err(ParseError::new("expected regex"))
        }
    } else {
        use_fallback(lexer, nodes, fallback);
        Err(ParseError::new("expected closing parenthesis"))
    }
}

fn parse_symbol(lexer: &mut Lexer, nodes: &mut Box<Vec<ASTNode>>) -> Result<usize, ParseError> {
    if let Some(token) = lexer.peek() {
        match token {
            Token::Symbol(c) => {
                lexer.next();
                Ok(push_node(nodes, ASTNode{node_type: SyntaxType::Symbol(c), children: vec![]}))
            },
            Token::Set(set) => {
                for symbol in &set {
                    if let SetSymbol::Range(start, end) = symbol {
                        if *start >= '0' as u32 && *start <= 'z'  as u32 {
                            if *end >= '0' as u32 && *end <= 'z'  as u32  {
                                if  start > end{
                                    return Err(ParseError::new("the numeric value of start must be less than end in a range"))
                                }
                            } else {
                                return Err(ParseError::new("the start and end of a range must a alphanumeric"))
                            }
                        } else {
                            return Err(ParseError::new("the start and end of a range must a alphanumeric"))
                        }
                    }
                }
                lexer.next();
                Ok(push_node(nodes, ASTNode{node_type: SyntaxType::Set(set), children: vec![]}))
            },
            Token::NotSet(set) => {
                for symbol in &set {
                    if let SetSymbol::Range(start, end) = symbol {
                        if *start >= '0' as u32 && *start <= 'z'  as u32 {
                            if *end >= '0' as u32 && *end <= 'z'  as u32  {
                                if  start > end{
                                    return Err(ParseError::new("the numeric value of start must be less than end in a range"))
                                }
                            } else {
                                return Err(ParseError::new("the start and end of a range must a alphanumeric"))
                            }
                        } else {
                            return Err(ParseError::new("the start and end of a range must a alphanumeric"))
                        }
                    }
                }
                lexer.next();
                Ok(push_node(nodes, ASTNode{node_type: SyntaxType::NotSet(set), children: vec![]}))
            },
            Token::Any => {
                lexer.next();
                Ok(push_node(nodes, ASTNode{node_type: SyntaxType::Any, children: vec![]}))
            },
            _ => Err(ParseError::new("expected symbol"))
        }
    } else {
        Err(ParseError::new("expected symbol"))
    }
}
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;

    use crate::lexer::Lexer;
    use super::parse_regex;

    #[test]
    fn output_diagram() {
        let mut file = File::create("ast-compiled.md").unwrap();
        writeln!(&mut file, "{}", parse_regex(&mut Lexer::new("(ab)|(cd)")).unwrap().to_string(false)).unwrap();
    }


    #[test]
    fn test() {
        assert_eq!(parse("abcd"), "((Symbol('a'))(Symbol('b'))(Symbol('c'))(Symbol('d'))Once)");
        assert_eq!(parse("(ab)cd"), "(((Symbol('a'))(Symbol('b'))Once)(Symbol('c'))(Symbol('d'))Once)");
        assert_eq!(parse("a+c*d+e?"), "(((Symbol('a'))OneOrMore)((Symbol('c'))ZeroOrMore)((Symbol('d'))OneOrMore)((Symbol('e'))Optional)Once)");
        assert_eq!(parse("a{1,}c{,1}d{2,3}"), "(((Symbol('a'))From(1))((Symbol('c'))To(1))((Symbol('d'))Between(2, 3))Once)");
        assert_eq!(parse("[ab-z][^ab-z]"), "((Set([Char('a'), Range(98, 122)]))(NotSet([Char('a'), Range(98, 122)]))Once)");
        assert_eq!(parse("(ab)*cd+"), "((((Symbol('a'))(Symbol('b'))Once)ZeroOrMore)(Symbol('c'))((Symbol('d'))OneOrMore)Once)");
        assert_eq!(parse("ab|cd"), "((((Symbol('a'))(Symbol('b'))Once)((Symbol('c'))(Symbol('d'))Once)Or)Once)");
        assert_eq!(parse("(a)+b|c*d"), "((((((Symbol('a'))Once)OneOrMore)(Symbol('b'))Once)(((Symbol('c'))ZeroOrMore)(Symbol('d'))Once)Or)Once)");
    }

    fn parse(string:&str) -> String {
        parse_regex(&mut Lexer::new(string)).unwrap().to_string(true)
    }
}


