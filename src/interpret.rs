use crate::{parser::{ASTNode, parse_regex, ParseError, SyntaxType}, lexer::Lexer};

struct Regex {
    code: String,
    ast: Box<ASTNode>,
}

impl Regex {
    pub fn compile(code: String) -> Result<Regex, ParseError> {
        let ast = parse_regex(&mut Lexer::new(&code))?;
        println!("{}", ast.to_string());
        Ok(Regex{code, ast})
    }

    pub fn matches(&self, string: &str) -> Vec<String> {
        let mut matches: Vec<String> = vec![];
        let mut characters: Vec<char> = string.chars().collect();

        let mut i = 0;
        while i < characters.len() {
            match Regex::interpret(&self.ast, &mut characters, i) {
                Ok(size) =>{
                    matches.push(characters[i..size].iter().collect());
                    i += size;
                },
                Err(_) => i+=1,
            }
        }

        matches
    }

    fn interpret(node: &Box<ASTNode>, string: &Vec<char>, index: usize) -> Result<usize, ()> {
        match node.node_type {
            SyntaxType::Once => Regex::once(node, string, index),
            SyntaxType::Or => Regex::or(node, string, index),
            SyntaxType::Symbol(_) => Regex::symbol(node, string, index),
            SyntaxType::ZeroOrMore => Regex::once(node, string, index),
            SyntaxType::Optional => Regex::once(node, string, index),
            SyntaxType::OneOrMore => Regex::once(node, string, index),
        }
    }

    fn optional(node: &Box<ASTNode>, string: &Vec<char>, index: usize) -> Result<usize, ()> {
        match Regex::interpret(node.children.last().unwrap(), string, index) {
            Ok(index) => Ok(index),
            Err(_) => Ok(index),
        }
    }

    fn once(node: &Box<ASTNode>, string: &Vec<char>, index: usize) -> Result<usize, ()> {
        Regex::once_helper(node, string, index, 0)
    }

    fn once_helper(node: &Box<ASTNode>, string: &Vec<char>, index: usize, start: usize) -> Result<usize, ()>  {
        let mut index = index;
        for i in start..node.children.len() {
            let child = &node.children[i];
            match node.children[i].node_type {
                SyntaxType::ZeroOrMore => {
                    let mut matches = vec![];
                    let old_index = index;

                    while let Ok(new_index) = Regex::interpret(child, string, index) {
                        index = new_index;
                        matches.push(new_index);
                    }

                    while let Some(new_index) = matches.pop() {
                        if let Ok(new_index) = Regex::once_helper(node, string, new_index, i+1) {
                            return Ok(new_index)
                        }
                    }

                    index = old_index;
                },
                SyntaxType::Optional => {
                    let new_index = Regex::optional(child, string, index).unwrap();
                    if new_index == index {
                        continue;
                    } else if i + 1 < node.children.len() {
                        return Regex::once_helper(node, string, new_index, i+1)
                        .or_else(|_| Regex::once_helper(node, string, index, i+1));
                    }
                },
                SyntaxType::OneOrMore => {
                    let mut matches = vec![Regex::interpret(child, string, index)?];
                    
                    while let Ok(new_index) = Regex::interpret(child, string, index) {
                        index = new_index;
                        matches.push(new_index);
                    }

                    while let Some(new_index) = matches.pop() {
                        if let Ok(new_index) = Regex::once_helper(node, string, new_index, i+1) {
                            return Ok(new_index)
                        }
                    }

                    return Err(())
                },
                SyntaxType::Once => index = Regex::once(child, string, index)?,
                SyntaxType::Or => index = Regex::or(child, string, index)?,
                SyntaxType::Symbol(_) => index = Regex::symbol(child, string, index)?,
            }
        }

        Ok(index)
    }

    fn or(node: &Box<ASTNode>, string: &Vec<char>, index: usize) -> Result<usize, ()> {
        for child in &node.children {
            if let Ok(index) = Regex::once(child, string, index) {
                return Ok(index)
            }
        }

        Err(())
    }

    fn symbol(node: &Box<ASTNode>, string: &Vec<char>, index: usize) -> Result<usize, ()>  {
        if let SyntaxType::Symbol(c) = node.node_type {
            if c == string[index] {
                Ok(index+1)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Regex;

    #[test]
    fn test() {
        let regex = Regex::compile("a*ab".to_string()).unwrap();

        let matches = regex.matches("aab ab");

        assert_eq!(matches[0], "aab");
        assert_eq!(matches[1], "ab");
    }
}