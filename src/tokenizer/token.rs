use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    #[inline(always)]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKind {
    Eof,
    Error,
    // keywords
    Print,
    Input,
    // meta tags
    Identifier,
    Int64,
    Float64,
    String,
    // separators
    Comma,
    Semicolon,
    Colon,
    Eol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline(always)]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {})", self.start, self.end)
    }
}
