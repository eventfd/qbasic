use core::ops::Deref;
use core::ops::DerefMut;
use std::collections::BTreeMap;

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
pub enum LValue {
    Identifier(SymbolId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Print(PrintStatement),
    Assign(Assignment),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub lhs: LValue,
    pub rhs: Expr,
}

impl Assignment {
    #[inline(always)]
    pub fn new(lhs: LValue, rhs: Expr) -> Self {
        Self { lhs, rhs }
    }
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
}

impl AsRef<[Expr]> for ArgList {
    fn as_ref(&self) -> &[Expr] {
        &self.0
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
    Identifier(SymbolId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SymbolId(String);

impl SymbolId {
    #[inline(always)]
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default)]
#[repr(transparent)]
pub struct SymbolTable(BTreeMap<SymbolId, Symbol>);

impl Deref for SymbolTable {
    type Target = BTreeMap<SymbolId, Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SymbolTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
