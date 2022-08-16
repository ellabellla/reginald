use std::mem::swap;

use crate::{parser::{parse_regex, ParseError, SyntaxType, AST}, lexer::{Lexer, SetSymbol}};

#[derive(Debug)]
enum StateType {
    Symbol(char),
    Any,
    Set(Vec<SetSymbol>),
    NotSet(Vec<SetSymbol>),
    Accept,
    None,
}

impl StateType {
    #[cfg(test)]
    pub fn to_string(&self) -> String  {
        match self {
            StateType::Symbol(c) => format!("'{}'", c),
            StateType::Accept => "Accept".to_string(),
            StateType::None => "None".to_string(),
            StateType::Any => "Any".to_string(),
            StateType::Set(set) => set.iter().map(|symbol| symbol.to_string()).collect::<Vec<String>>().join(", "),
            StateType::NotSet(set) => format!("not {}",set.iter().map(|symbol| symbol.to_string()).collect::<Vec<String>>().join(", ")),
        }
    }
}

struct States {
    nodes: Box<Vec<StateNode>>,
    starting_state: usize,
}

impl States {
    #[cfg(test)]
    pub fn to_string(&self) -> String{
        let mut out = vec![];
        States::push_string("```mermaid\n", &mut out);
        States::push_string("flowchart LR\n", &mut out);

        for (i, state) in self.nodes.as_ref().iter().enumerate() {
            States::push_string(&format!("\t{}({})\n", i, state.state_type.to_string()), &mut out);

            for next_state in &state.next {
                States::push_string(&format!("\t{}-->{}\n", i, next_state), &mut out);
            }
        }

        States::push_string("```\n", &mut out);
        
        out.iter().collect()
    }
    
    #[cfg(test)]
    fn push_string(string: &str, out: &mut Vec<char>) {
        for c in string.chars() {
            out.push(c);
        }
    }
}

struct StateNode { 
    state_type: StateType,
    next: Vec<usize>
}

pub struct Regex {
    states: States,
}

impl Regex {
    pub fn compile(code: &str) -> Result<Regex, ParseError> {
        let ast = parse_regex(&mut Lexer::new(&code))?;
        let mut regex = Regex{states: States { nodes: Box::new(vec![StateNode{state_type: StateType::None, next: vec![]}]), starting_state: 0 }};

        regex.init(ast);

        Ok(regex)
    }

    pub fn test(&self, string: &str) -> bool {
        return self.simulate_states(&string.chars().collect(), 0) == string.len();
    }

    pub fn matches(&self, string: &str) -> Vec<(usize, usize)> {
        let chars = &string.chars().collect();
        let mut found = vec![];
        let mut i = 0usize;


        while i < string.len() {
            let size_of_found = self.simulate_states(chars, i);
            if size_of_found != 0 {
                found.push((i, size_of_found));
                i += size_of_found
            } else {
                i += 1
            }
        }

        found
    }

    pub fn is_match(&self, string: &str) -> bool {
        let chars = &string.chars().collect();
        let mut i = 0usize;


        while i < string.len() {
            let size_of_found = self.simulate_states(chars, i);
            if size_of_found != 0 {
                return true
            } else {
                i += 1
            }
        }

        return false
    }


    fn simulate_states(&self, chars: &Vec<char>, offset: usize) -> usize{
        if offset >= chars.len() {
            return 0
        }

        let mut max_len =  offset;
        let mut stack = &mut vec![(offset, self.states.starting_state)];
        let mut stack_back = &mut vec![];

        while !stack.is_empty() {
            while let Some((index, state)) = stack.pop() {
                let state = self.states.nodes.get(state).unwrap();

                match &state.state_type {
                    StateType::Symbol(c) => if index+1 > chars.len() {
                        continue;
                    } else if *c == chars[index] {
                        for next_state in &state.next {
                            stack_back.push((index+1, *next_state));
                        }
                    } else {
                        continue;
                    },
                    StateType::Accept => max_len = max_len.max(index),
                    StateType::None => for next_state in &state.next {
                        stack_back.push((index, *next_state));
                    }
                    StateType::Any => if index+1 > chars.len() {
                        continue;
                    } else {
                        for next_state in &state.next {
                            stack_back.push((index+1, *next_state));
                        }
                    },
                    StateType::Set(set) => if index+1 > chars.len() {
                        continue;
                    } else {
                        if {
                            let mut found = false;
                            for symbol in set {
                                match symbol {
                                    SetSymbol::Char(c) => if *c == chars[index] {
                                        found = true;
                                        break;
                                    },
                                    SetSymbol::Range(start, end) => 
                                    if chars[index] as u32 >= *start && chars[index] as u32 <= *end {
                                        found = true;
                                        break;
                                    },
                                }
                            }
                            found
                        } {
                            for next_state in &state.next {
                                stack_back.push((index+1, *next_state));
                            }
                        } else {
                            continue;
                        }
                    },
                    StateType::NotSet(set) => if index+1 > chars.len() {
                        continue;
                    } else {
                        if {
                            let mut found = true;
                            for symbol in set {
                                match symbol {
                                    SetSymbol::Char(c) => if *c == chars[index] {
                                        found = false;
                                        break;
                                    },
                                    SetSymbol::Range(start, end) => 
                                    if chars[index] as u32 >= *start && chars[index] as u32 <= *end {
                                        found = false;
                                        break;
                                    },
                                }
                            }
                            found
                        } {
                            for next_state in &state.next {
                                stack_back.push((index+1, *next_state));
                            }
                        } else {
                            continue;
                        }
                    },
                }
            }

            swap(&mut stack, &mut stack_back);
        }

        return max_len - offset
    }


    fn init(&mut self, ast: AST) {
        let end_state = self.compile_once(self.states.starting_state, &ast, ast.start_node);

        self.states.nodes.push(StateNode{ state_type: StateType::Accept, next: vec![] });
        let state = self.states.nodes.len() - 1;

        let end_state = self.states.nodes.get_mut(end_state).unwrap();
        end_state.next.push(state);
    }

    fn compile_next(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        match ast.nodes.get(ast_node).unwrap().node_type {
            SyntaxType::ZeroOrMore => self.compile_zero_or_more(prev_state, ast, ast_node),
            SyntaxType::Optional => self.compile_optional(prev_state, ast, ast_node),
            SyntaxType::OneOrMore => self.compile_one_or_more(prev_state, ast, ast_node),
            SyntaxType::Once => self.compile_once(prev_state, ast, ast_node),
            SyntaxType::Or => self.compile_or(prev_state, ast, ast_node),
            SyntaxType::From(_) => self.compile_from(prev_state, ast, ast_node),
            SyntaxType::To(_) => self.compile_to(prev_state, ast, ast_node),
            SyntaxType::Between(_, _) => self.compile_between(prev_state, ast, ast_node),
            SyntaxType::Symbol(_) => self.compile_atomic(prev_state, ast, ast_node),
            SyntaxType::Set(_) => self.compile_atomic(prev_state, ast, ast_node),
            SyntaxType::NotSet(_) => self.compile_atomic(prev_state, ast, ast_node),
            SyntaxType::Any => self.compile_atomic(prev_state, ast, ast_node),
        }
    }

    fn compile_zero_or_more(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();
        let next_state = self.compile_next(prev_state, ast, node.children[0]);

        let next_state = self.states.nodes.get_mut(next_state).unwrap();
        next_state.next.push(prev_state);


        prev_state.clone()
    }

    fn compile_optional(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();
        let next_state = self.compile_next(prev_state, ast, node.children[0]);

        self.states.nodes.push(StateNode{ state_type: StateType::None, next: vec![] });
        let state = self.states.nodes.len() - 1;
        
        let prev_state = self.states.nodes.get_mut(prev_state).unwrap();
        prev_state.next.push(state);

        let next_state = self.states.nodes.get_mut(next_state).unwrap();
        next_state.next.push(state);

        state
    }

    fn compile_one_or_more(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();

        let next_state = self.compile_next(prev_state, ast, node.children[0]);
        let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
        next_state_node.next.push(prev_state);
 
        next_state
    }

    fn compile_once(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();

        let mut next_state = prev_state;
        
        for child in &node.children {
            next_state = self.compile_next(next_state, ast, *child);
        }

        next_state
    }

    fn compile_or(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();

        
        let mut next_state = vec![];
        
        for child in &node.children {
            next_state.push(self.compile_next(prev_state, ast, *child));
        }

        self.states.nodes.push(StateNode{ state_type: StateType::None, next: vec![] });
        let state = self.states.nodes.len() - 1;
        for next_state in next_state {
            let next_state = self.states.nodes.get_mut(next_state).unwrap();
            next_state.next.push(state);
        }

        state
    }

    fn compile_from(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        if let SyntaxType::From(from)= ast.nodes.get(ast_node).unwrap().node_type {
            if from == 0 {
                self.compile_one_or_more(prev_state, ast, ast_node)
            } else {
                let node = ast.nodes.get(ast_node).unwrap();

                let mut next_state = prev_state;

                for _ in 0..from {
                    next_state = self.compile_next(next_state, ast, node.children[0]);
                }
                self.states.nodes.push(StateNode{ state_type: StateType::None, next: vec![] });
                let state = self.states.nodes.len() - 1;

                let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
                next_state_node.next.push(state);

                self.compile_zero_or_more(state, ast, ast_node);
        
                state
            }
        } else {
            unreachable!()
        }
    }

    fn compile_to(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        if let SyntaxType::To(to)= ast.nodes.get(ast_node).unwrap().node_type {
            let node = ast.nodes.get(ast_node).unwrap();
                self.states.nodes.push(StateNode{ state_type: StateType::None, next: vec![] });
                let state = self.states.nodes.len() - 1;

                let mut next_state = prev_state;
                let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
                next_state_node.next.push(state);

                for _ in 0..to {
                    next_state = self.compile_next(next_state, ast, node.children[0]);

                    let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
                    next_state_node.next.push(state);
                }
                state
        } else {
            unreachable!()
        }
    }

    fn compile_between(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        if let SyntaxType::Between(from, to)= ast.nodes.get(ast_node).unwrap().node_type {
            let node = ast.nodes.get(ast_node).unwrap();

                let mut next_state = prev_state;
                
                for _ in 0..from {
                    next_state = self.compile_next(next_state, ast, node.children[0]);
                }

                self.states.nodes.push(StateNode{ state_type: StateType::None, next: vec![] });
                let state = self.states.nodes.len() - 1;
                let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
                next_state_node.next.push(state);

                for _ in from..to {
                    next_state = self.compile_next(next_state, ast, node.children[0]);

                    let next_state_node = self.states.nodes.get_mut(next_state).unwrap();
                    next_state_node.next.push(state);
                }

                state
        } else {
            unreachable!()
        }
    }

    fn compile_atomic(&mut self, prev_state: usize, ast: &AST, ast_node: usize) -> usize {
        let node = ast.nodes.get(ast_node).unwrap();
        let state_type = match &node.node_type {
            SyntaxType::Symbol(c) => StateType::Symbol(c.clone()),
            SyntaxType::Set(set) => StateType::Set(set.clone()),
            SyntaxType::NotSet(set) => StateType::NotSet(set.clone()),
            SyntaxType::Any => StateType::Any,
            _ => unreachable!(),
        };
        self.states.nodes.push(StateNode{ state_type, next: vec![] });
        let state = self.states.nodes.len() - 1;

        let prev_state = self.states.nodes.get_mut(prev_state).unwrap();
        prev_state.next.push(state);

        state
    }
}


#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::fs::File;

    use super::Regex;

    #[test]
    fn output_diagram() {
        let regex = Regex::compile("a{2,}").unwrap();

        let mut file = File::create("regex-compiled.md").unwrap();
        writeln!(&mut file, "{}", &regex.states.to_string()).unwrap();
    }

    #[test]
    fn test() {
        let regex = Regex::compile("a{2,}").unwrap();

        assert!(regex.test("aaa"));
    }

    #[test]
    fn test_matches() {
        let regex = Regex::compile("a+(b|c)").unwrap();

        let found = regex.matches("aaaab ab ac aaacab");

        assert_eq!(found.len(), 5);
        assert_eq!(found[0], (0, 5));
        assert_eq!(found[1], (6, 2));
        assert_eq!(found[2], (9, 2));
        assert_eq!(found[3], (12, 4));
        assert_eq!(found[4], (16, 2));
    }

    #[test]
    fn test_is_match() {
        let regex = Regex::compile("a+(b|c)").unwrap();

        assert!(regex.is_match("yas ao cbhj bqwo aaab nme ab"))
    }
}