use crate::tokenizer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementList(Vec<Statement>);

impl From<Vec<Statement>> for StatementList {
    fn from(value: Vec<Statement>) -> Self {
        Self(value)
    }
}

impl StatementList {
    pub fn statements(&self) -> &[Statement] {
        &self.0
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
