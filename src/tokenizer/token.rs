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
pub enum Keyword {
    Print,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier,
    Int64,
    Float64,
    String,
    Comma,
    Semicolon,
    Colon,
    Eol,
    Eof,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub offset: u32,
    pub len: u32,
}

impl Span {
    #[inline(always)]
    pub const fn new(offset: u32, len: u32) -> Self {
        Self { offset, len }
    }
}
