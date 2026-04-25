use core::fmt::Debug;
use core::fmt::Display;

use crate::tokenizer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new(span: Span, msg: impl Into<String>) -> Self {
        Self {
            span,
            message: msg.into(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
