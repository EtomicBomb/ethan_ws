use std::fmt;

use crate::span::Span;

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Span,
}

impl RuntimeError {
    pub fn stringify(&self, source: &str) -> String {
        if self.span.single_line(source) {
            let string = self.span.slice(source);
            let line = self.span.line_range(source).start;
            format!("{} on line {}: `{}`", self.kind, line, string)
        } else {
            let string = self.span.slice(source);
            let line_range = self.span.line_range(source);
            format!("{} in lines {} to {}: `{}`", self.kind, line_range.start, line_range.end, string)
        }
    }
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
    EmptyFunctionApplication,
    CannotEvaluatePair,
    ConsSecondArgMustBeList,
    FunctionMustBeSymbol(String),
    CannotMathNonNumerics,
    UndefinedFunction(String),
    VariableNameMustBeSymbol(String),
    VariableNotFound(String),
    WrongNumberOfArguments(String, usize, ParameterCount),
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RuntimeErrorKind::EmptyFunctionApplication => write!(f, "cannot evaluate the empty list as a function"),
            RuntimeErrorKind::CannotEvaluatePair => write!(f, "cannot evaluate pair"),
            RuntimeErrorKind::ConsSecondArgMustBeList => write!(f, "second argument of `cons` must be list"),
            RuntimeErrorKind::FunctionMustBeSymbol(ref s) => write!(f, "cannot call `{}` like a function", s),
            RuntimeErrorKind::CannotMathNonNumerics => write!(f, "attempt to do math to non-numeric values"),
            RuntimeErrorKind::UndefinedFunction(ref s) => write!(f, "function `{}` has no definition", s),
            RuntimeErrorKind::VariableNameMustBeSymbol(ref s) => write!(f, "cannot assign `{}` a value", s),
            RuntimeErrorKind::VariableNotFound(ref s) => write!(f, "cannot find variable `{}`", s),
            RuntimeErrorKind::WrongNumberOfArguments(ref operator, found, target) =>
                write!(f, "`{}`: wrong number of arguments, was {}, should be {}", operator, found, target),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParameterCount {
    Exactly(usize),
    GreaterThan(usize),
}

impl fmt::Display for ParameterCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParameterCount::Exactly(count) => write!(f, "{}", count),
            ParameterCount::GreaterThan(count) => write!(f, "greater than {}", count),
        }
    }
}

