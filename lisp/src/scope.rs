use std::collections::HashMap;
use std::fmt;
use std::iter::FromIterator;

use crate::handlers::{Node};
use crate::span::Span;

#[derive(Clone, Debug)]
pub struct FunctionContext {
    functions: HashMap<String, (Vec<String>, Node)>,
}

impl FunctionContext {
    pub fn new(functions: HashMap<String, (Vec<String>, Node)>) -> FunctionContext {
        FunctionContext { functions }
    }

    pub fn find_function(&self, name: &str) -> Option<&(Vec<String>, Node)> {
        self.functions.get(name)
    }

    pub fn definitions_iter_mut(&mut self) -> impl Iterator<Item=&mut Node> {
        self.functions.iter_mut()
            .map(|(_, (_, n))| n)
    }
}

impl fmt::Display for FunctionContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (name, (parameters, node)) in self.functions.iter() {
            write!(f, "{} [", name)?;
            let maybe_space = |i| if i+1 < parameters.len() { " " } else { "" };
            for (i, param) in parameters.iter().enumerate() {
                write!(f, "{}{}", param, maybe_space(i))?;
            }
            write!(f, "]\n\t{}\n\n", node)?;
        }
        Ok(())
    }
}

pub struct MacroContext {
    macros: HashMap<String, (Vec<String>, Node)>,
}

impl MacroContext {
    pub fn new(macros: HashMap<String, (Vec<String>, Node)>) -> MacroContext {
        MacroContext { macros }
    }

    pub fn find_macro(&self, name: &str) -> Option<&(Vec<String>, Node)> {
        self.macros.get(name)
    }
}


#[derive(Debug)]
pub struct ContextVariables {
    frames: Vec<Frame>,
}

impl ContextVariables {
    pub fn new() -> ContextVariables {
        ContextVariables { frames: vec![Frame::new()] }
    }

    pub fn new_frame(&self) -> ContextVariables {
        let mut frames = self.frames.clone();
        frames.push(Frame::new());

        ContextVariables { frames }
    }

    pub fn new_variable(&mut self, name: String, scope: Span, value: Node) {
        let frame = self.frames.last_mut().unwrap();
        frame.new_binding(name, scope, value)
    }

    pub fn find_variable(&self, name: &str, at: usize) -> Option<Node> {
        self.frames.iter().rev()
            .find_map(|f| f.find_binding(name, at))
            .cloned()
    }
}

impl FromIterator<(String, Span, Node)> for ContextVariables {
    fn from_iter<I: IntoIterator<Item=(String, Span, Node)>>(iter: I) -> ContextVariables {
        ContextVariables {
            frames: vec![Frame {
                bindings: iter.into_iter().collect()
            }]
        }
    }
}

#[derive(Clone, Debug)]
struct Frame {
    bindings: Vec<(String, Span, Node)>,
}

impl Frame {
    fn new() -> Frame {
        Frame { bindings: Vec::new() }
    }

    fn new_binding(&mut self, name: String, scope: Span, value: Node) {
        self.bindings.push((name, scope, value));
    }

    fn find_binding(&self, name: &str, at: usize) -> Option<&Node> {
        // dbg!(self.bindings[0].1.slice(&std::fs::read_to_string("functions.lisp").unwrap()));
        self.bindings.iter()
            .find(|&&(ref n, s, _)| n == name && s.contains(at))
            .map(|&(_, _, ref n)| n)
    }
}
