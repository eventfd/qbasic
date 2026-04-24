use crate::tokenizer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub lines: Vec<Line>,
}

impl Program {
    pub fn new(lines: Vec<Line>) -> Self {
        Self { lines }
    }
}

impl From<Vec<Line>> for Program {
    fn from(lines: Vec<Line>) -> Self {
        Self { lines }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub line_no: Option<u32>,
    pub statements: Vec<Statement>,
}

impl Line {
    pub fn new(line_no: Option<u32>, statements: Vec<Statement>) -> Self {
        Self {
            line_no,
            statements,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Print(PrintStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintStatement(ArgList);

impl PrintStatement {
    pub fn new(args: ArgList) -> Self {
        Self(args)
    }

    pub fn args(&self) -> &ArgList {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgList(Vec<Expr>);

impl ArgList {
    pub fn new(exprs: Vec<Expr>) -> Self {
        Self(exprs)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Expr> {
        self.0.iter()
    }
}

impl From<Vec<Expr>> for ArgList {
    fn from(value: Vec<Expr>) -> Self {
        Self(value)
    }
}

impl From<ArgList> for PrintStatement {
    fn from(value: ArgList) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(span: Span, kind: ExprKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Int64(i64),
}
