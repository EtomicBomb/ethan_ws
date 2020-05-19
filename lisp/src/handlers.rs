use pest::Parser;
use pest::iterators::{Pair, Pairs};
use std::fmt;
use std::collections::HashMap;

use crate::scope::{FunctionContext, ContextVariables, MacroContext};
use crate::error::{RuntimeError, RuntimeErrorKind, ParameterCount};
use crate::span::Span;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct LispParser;

#[derive(Debug)]
pub struct Handlers {
    source: String,
    pub function_context: FunctionContext,
}

impl Handlers {
    pub fn parse(source: String) -> Result<Handlers, LispParseError> {
        let mut result: Pairs<Rule> = LispParser::parse(Rule::TOP, &source)?;

        let mut functions = HashMap::new();
        let mut macros = HashMap::new();

        for pair in result.next().unwrap().into_inner().take_while(|e| e.as_rule() != Rule::EOI) {
            let span = pair.as_span().into();
            let is_macro = pair.as_rule() == Rule::macro_definition;
            let mut iter = pair.into_inner();
            let name = iter.next().unwrap().as_str().to_string();
            let parameters = iter.next().unwrap().into_inner()
                .map(|p| p.as_str().to_string())
                .collect();
            let definition = Node::from_parse(iter.next().unwrap())?;

            if functions.contains_key(&name) || macros.contains_key(&name) {
                return Err(LispParseError::AlreadyDeclared(span))
            }

            if is_macro {
                macros.insert(name, (parameters, definition));
            } else {
                functions.insert(name, (parameters, definition));
            }
        }

        let mut function_context = FunctionContext::new(functions);
        let macro_context = MacroContext::new(macros);

        let functions_cloned = function_context.clone(); // should have the same behavior

        for n in function_context.definitions_iter_mut() {
            n.apply_macros_to_node(&macro_context, &functions_cloned, &ContextVariables::new())?;
        }

        Ok(Handlers { source, function_context  })
    }

    pub fn eval(&self, function_name: &str, arguments: Vec<Node>) -> Result<Node, RuntimeError> {
        match self.function_context.find_function(function_name) {
            Some(&(ref parameters, ref definition)) => {
                if parameters.len() != arguments.len() {
                    return Err(RuntimeError {
                        kind: RuntimeErrorKind::WrongNumberOfArguments(function_name.to_string(), arguments.len(), ParameterCount::Exactly(parameters.len())),
                        span: Span::single_byte(0)
                    })
                }

                let mut variable_context = ContextVariables::new();

                for (parameter, argument) in parameters.iter().cloned().zip(arguments) {
                    variable_context.new_variable(parameter, definition.span, argument);
                }

                definition.eval(&self.function_context, &variable_context, None)
            },
            None => panic!(),
        }
    }

    pub fn stringify_error(&self, error: RuntimeError) -> String {
        error.stringify(&self.source)
    }
}

impl fmt::Display for Handlers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.function_context)
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    span: Span,
    expression: Expression,
}

impl Node {
    fn eval(&self, functions: &FunctionContext, variables: &ContextVariables, macros: Option<&MacroContext>) -> Result<Node, RuntimeError> {


        match self.expression {
            Expression::Pair(_) => Err(RuntimeError { kind: RuntimeErrorKind::CannotEvaluatePair, span: self.span }),
            Expression::Atom(ref atom) => {
                match &atom {
                    Atom::Symbol(ref s) => {
                        variables.find_variable(s, self.span.start_index())
                            .ok_or_else(|| RuntimeError {
                                kind: RuntimeErrorKind::VariableNotFound(s.to_string()),
                                span: self.span
                            })
                    },
                    _ => Ok(self.clone()),
                }
            },
            Expression::List(ref children) => {
                match children.first().map(|n| &n.expression) {
                    None => Err(RuntimeError {
                        kind: RuntimeErrorKind::EmptyFunctionApplication,
                        span: self.span
                    }),
                    Some(&Expression::Atom(Atom::Symbol(ref operator))) => {
                        let args = &children[1..];

                        match operator.as_str() {
                            "+" => {
                                let mut sum = 0.0;
                                for arg in args {
                                    let result = arg.eval(functions, variables, macros)?;
                                    match result.expression {
                                        Expression::Atom(Atom::Number(result)) => sum += result,
                                        _ => return Err(RuntimeError { kind: RuntimeErrorKind::CannotMathNonNumerics, span: arg.span }),
                                    }
                                }

                                Ok(Node { expression: Expression::Atom(Atom::Number(sum)), span: self.span })
                            },
                            "*" => {
                                let mut product = 1.0;
                                for arg in args {
                                    let result = arg.eval(functions, variables, macros)?;
                                    match result.expression {
                                        Expression::Atom(Atom::Number(result)) => product *= result,
                                        _ => return Err(RuntimeError { kind: RuntimeErrorKind::CannotMathNonNumerics, span: arg.span }),
                                    }
                                }
                                Ok(Node { expression: Expression::Atom(Atom::Number(product)), span: self.span })
                            },
                            "-" => {
                                if args.len() > 1 {
                                    let mut total = match args[0].eval(functions, variables, macros)?.expression {
                                        Expression::Atom(Atom::Number(result)) => result,
                                        _ => return Err(RuntimeError { kind: RuntimeErrorKind::CannotMathNonNumerics, span: args[0].span }),
                                    };
                                    for arg in args.iter().skip(1) {
                                        match arg.eval(functions, variables, macros)?.expression {
                                            Expression::Atom(Atom::Number(result)) => total -= result,
                                            _ => return Err(RuntimeError { kind: RuntimeErrorKind::CannotMathNonNumerics, span: arg.span }),
                                        }
                                    }
                                    Ok(Node { expression: Expression::Atom(Atom::Number(total)), span: self.span, })
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("-".to_string(), args.len(), ParameterCount::GreaterThan(1)),
                                        span: self.span
                                    })
                                }
                            },
                            "if" => {
                                if args.len() == 3 {
                                    match args[0].eval(functions, variables, macros)? {
                                        Node { expression: Expression::Atom(Atom::Keyword(ref k)), .. } if k == ":false" => args[2].eval(functions, variables, macros),
                                        _ => args[1].eval(functions, variables, macros)
                                    }
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("if".to_string(), args.len(), ParameterCount::Exactly(3)),
                                        span: self.span
                                    })
                                }
                            },
                            "=" => {
                                if args.len() == 2 {
                                    let equal = args[0].eval(functions, variables, macros)?.equal_ignore_span(&args[1].eval(functions, variables, macros)?);
                                    Ok(Node {
                                        expression: Expression::Atom(Atom::Keyword(if equal { ":true" } else { ":false" }.to_string())),
                                        span: self.span
                                    })
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("=".to_string(), args.len(), ParameterCount::Exactly(2)),
                                        span: self.span
                                    })
                                }
                            },
                            "print" => {
                                for arg in args {
                                    let result = arg.eval(functions, variables, macros)?;
                                    println!("{}", result);
                                }

                                Ok(Node { expression: Expression::List(Vec::new()), span: self.span })
                            },
                            "quote" => {
                                if args.len() == 1 {
                                    Ok(args[0].clone())
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("quote".to_string(), args.len(), ParameterCount::Exactly(1)),
                                        span: self.span
                                    })
                                }
                            },
                            "let" => {
                                if args.len() == 3 {
                                    let name = match args[0].expression {
                                        Expression::Atom(Atom::Symbol(ref s)) => s.to_string(),
                                        ref e => return Err(RuntimeError { kind: RuntimeErrorKind::VariableNameMustBeSymbol(e.to_string()), span: args[1].span }),
                                    };
                                    let value = args[1].eval(functions, variables, macros)?;
                                    let body = &args[2];

                                    let mut child_variables = variables.new_frame();
                                    child_variables.new_variable(name, body.span, value);

                                    body.eval(functions, &child_variables, macros)
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("let".to_string(), args.len(), ParameterCount::Exactly(3)),
                                        span: self.span
                                    })
                                }
                            },
                            "pair" => {
                                if args.len() == 2 {
                                    Ok(Node {
                                        expression: Expression::Pair(Box::new((
                                            args[0].eval(functions, variables, macros)?,
                                            args[1].eval(functions, variables, macros)?,
                                        ))),
                                        span: self.span
                                    })
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("pair".to_string(), args.len(), ParameterCount::Exactly(2)),
                                        span: self.span
                                    })
                                }
                            },
                            "cons" => {
                                if args.len() == 2 {
                                    let mut ret_list = match args[1].eval(functions, variables, macros)? {
                                        Node { expression: Expression::List(ref l), .. } => l.clone(),
                                        _ => return Err(RuntimeError { kind: RuntimeErrorKind::ConsSecondArgMustBeList, span: args[1].span }),
                                    };

                                    ret_list.insert(0, args[0].eval(functions, variables, macros)?);

                                    Ok(Node { expression: Expression::List(ret_list), span: self.span })
                                } else {
                                    Err(RuntimeError {
                                        kind: RuntimeErrorKind::WrongNumberOfArguments("cons".to_string(), args.len(), ParameterCount::Exactly(2)),
                                        span: self.span
                                    })
                                }
                            },
                            "list" => {
                                let ret = args.iter()
                                    .map(|a| a.eval(functions, variables, macros))
                                    .collect::<Result<_, _>>()?;

                                Ok(Node { expression: Expression::List(ret), span: self.span })
                            },
                            "jsonify" => {
                                todo!()
                            },
                            _ => {
                                let to_apply = functions.find_function(operator)
                                    .or_else(|| macros.and_then(|m| m.find_macro(operator)));

                                match to_apply {
                                    Some((ref parameters, ref definition)) => {
                                        if parameters.len() != args.len() {
                                            return Err(RuntimeError {
                                                kind: RuntimeErrorKind::WrongNumberOfArguments(operator.to_string(), args.len(), ParameterCount::Exactly(parameters.len())),
                                                span: self.span
                                            })
                                        }

                                        let child_variables: ContextVariables =
                                            parameters.iter().cloned().into_iter().zip(args.iter().cloned())
                                                .map(|(parameter, argument)| Ok((parameter, definition.span, argument.eval(functions, variables, macros)?)))
                                                .collect::<Result<_, _>>()?;

                                        definition.eval(functions, &child_variables, macros)
                                    },
                                    None => Err(RuntimeError { kind: RuntimeErrorKind::UndefinedFunction(operator.to_string()), span: self.span }),
                                }
                            },
                        }
                    },
                    Some(e) => Err(RuntimeError { kind: RuntimeErrorKind::FunctionMustBeSymbol(e.to_string()), span: self.span }),
                }
            },
        }
    }

    fn equal_ignore_span(&self, other: &Node) -> bool {
        match (&self.expression, &other.expression) {
            (Expression::Atom(ref a), Expression::Atom(ref b)) => a == b,
            (Expression::List(ref a), Expression::List(ref b)) => a.len() == b.len() &&
                a.iter().zip(b).all(|(n0, n1)| n0.equal_ignore_span(n1)),
            _ => false,
        }
    }

    fn apply_macros_to_node(&mut self, macros: &MacroContext, functions: &FunctionContext, variables: &ContextVariables) -> Result<(), RuntimeError> {
        if let Expression::List(ref mut children) = self.expression {
            if children.is_empty() { return Err(RuntimeError { kind: RuntimeErrorKind::EmptyFunctionApplication, span: self.span }) };
            let (name_node, args) = children.split_first_mut().unwrap();

            for arg in args {
                arg.apply_macros_to_node(macros, functions, variables)?;
            }

            if let Node { expression: Expression::Atom(Atom::Symbol(ref macro_name)), .. } = *name_node {
                if let Some((parameters, definition)) = macros.find_macro(macro_name) {
                    // *self = self.eval(functions, variables, Some(macros))?;

                    // we now have our macro
                    let mut child_variables = ContextVariables::new();

                    for (parameter, argument) in parameters.iter().zip(&children[1..]) {
                        child_variables.new_variable(parameter.clone(), definition.span, argument.clone());
                    }

                    let old_span = self.span;
                    *self = definition.eval(functions, &child_variables, Some(macros))?;
                    self.span = old_span;
                }
            }
        }

        Ok(())
    }

    fn from_parse(pair: Pair<Rule>) -> Result<Node, LispParseError> {
        let quote_span = Span::single_byte(pair.as_span().start());
        let is_quoted = pair.as_str().starts_with('\'');

        let a = pair.into_inner().next().unwrap();
        let span = a.as_span().into();

        let ret = match a.as_rule() {
            Rule::atom => Node { expression: Expression::Atom(Atom::from_parse(a)?), span },
            Rule::list => {
                let vec = a.into_inner()
                    .map(Node::from_parse)
                    .collect::<Result<_, _>>()?;

                Node { expression: Expression::List(vec), span }
            },
            _ => unreachable!(),
        };

        Ok(if is_quoted {
            Node {
                expression: Expression::List(vec![
                    Node {
                        expression: Expression::Atom(Atom::Symbol("quote".to_string())),
                        span: quote_span,
                    },
                    ret,
                ]),
                span,
            }
        } else {
            ret
        })
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Atom(Atom),
    Pair(Box<(Node, Node)>),
    List(Vec<Node>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expression::Atom(ref atom) => write!(f, "{}", atom),
            Expression::Pair(ref pair) => {
                let (left, right) = &**pair;
                write!(f, "({} . {})", left, right)
            }
            Expression::List(ref children) => {
                let maybe_space = |i| if i+1 < children.len() { " " } else { "" };
                write!(f, "(")?;
                for (i, child) in children.iter().enumerate() {
                    write!(f, "{}{}", child, maybe_space(i))?;
                }
                write!(f, ")")
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Keyword(String),
    Number(f64),
    ByteVector(Vec<u8>), // interpreted as a string when it makes sense
    Symbol(String),
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Atom::ByteVector(ref v) => match String::from_utf8(v.clone()) {
                Ok(s) => write!(f, "\"{}\"", s),
                Err(_) => write!(f, "{:?}", v),
            },
            Atom::Keyword(ref b) => write!(f, "{}", b),
            Atom::Number(n) => write!(f, "{}", n),
            Atom::Symbol(ref s) => write!(f, "{}", s),
        }
    }
}

impl Atom {
    fn from_parse(pair: Pair<Rule>) -> Result<Atom, LispParseError> {
        let a = pair.into_inner().next().unwrap();
        match a.as_rule() {
            Rule::number => Ok(Atom::Number(a.as_str().parse().map_err(|_| LispParseError::BadInteger)?)),
            Rule::keyword => Ok(Atom::Keyword(a.as_str().to_string())),
            Rule::string => Ok(Atom::ByteVector(a.as_str().as_bytes()[1..a.as_str().len()-1].to_vec())),
            Rule::symbol => Ok(Atom::Symbol(a.as_str().to_string())),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum LispParseError {
    PestError(pest::error::Error<Rule>),
    BadInteger,
    AlreadyDeclared(Span),
    RuntimeErorr(RuntimeError),
}

impl From<pest::error::Error<Rule>> for LispParseError {
    fn from(e: pest::error::Error<Rule>) -> LispParseError {
        LispParseError::PestError(e)
    }
}

impl From<RuntimeError> for LispParseError {
    fn from(e: RuntimeError) -> LispParseError {
        LispParseError::RuntimeErorr(e)
    }
}

