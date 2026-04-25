use core::fmt::Display;
use std::collections::HashMap;
use std::io::Write;

use crate::parser::ast::Assignment;
use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::LValue;
use crate::parser::ast::Line;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Program;
use crate::parser::ast::Statement;
use crate::parser::ast::SymbolId;

pub struct Evaluator<W> {
    writer: W,
    env: HashMap<SymbolId, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int64(i64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int64(v) => write!(f, "{}", *v),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EvalError {
    pub msg: String,
}

pub type EvalResult<T> = Result<T, EvalError>;

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl<W: Write> Evaluator<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            env: HashMap::new(),
        }
    }

    fn eval_statement(&mut self, stmt: &Statement) -> EvalResult<()> {
        match stmt {
            Statement::Print(ps) => self.eval_print(ps),
            Statement::Assign(assign) => self.eval_assign(assign),
        }
    }

    fn eval_assign(&mut self, assign: &Assignment) -> EvalResult<()> {
        match &assign.lhs {
            LValue::Identifier(id) => {
                self.env.insert(id.clone(), self.eval_expr(&assign.rhs)?);
            }
        }
        Ok(())
    }

    fn eval_line(&mut self, line: &Line) -> EvalResult<()> {
        for stmt in &line.statements {
            self.eval_statement(stmt)?;
        }
        Ok(())
    }

    pub fn eval_program(&mut self, prog: &Program) -> EvalResult<()> {
        for e in &prog.lines {
            self.eval_line(e)?;
        }
        Ok(())
    }

    fn eval_print(&mut self, ps: &PrintStatement) -> EvalResult<()> {
        if let Some((head, tail)) = ps.args().as_ref().split_first() {
            write!(self.writer, "{}", self.eval_expr(head)?)
                .map_err(|_| EvalError::new("I/O Error"))?;
            for e in tail {
                write!(self.writer, " ")
                    .map_err(|_| EvalError::new("I/O Error"))?;
                write!(self.writer, "{}", self.eval_expr(e)?)
                    .map_err(|_| EvalError::new("I/O Error"))?;
            }
        }
        writeln!(self.writer).map_err(|_| EvalError::new("I/O Error"))?;
        Ok(())
    }

    fn eval_expr(&self, expr: &Expr) -> EvalResult<Value> {
        match &expr.kind {
            ExprKind::Int64(val) => Ok(Value::Int64(*val)),
            ExprKind::Identifier(id) => {
                self.env.get(id).cloned().ok_or_else(|| {
                    EvalError::new(format!(
                        "Identifier {:?} not defined, at {:?}",
                        id, expr.span
                    ))
                })
            }
        }
    }
}
