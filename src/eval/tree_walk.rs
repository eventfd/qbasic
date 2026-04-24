use std::io::Write;

use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::Line;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Program;
use crate::parser::ast::Statement;

pub struct Evaluator<W> {
    writer: W,
}

impl<W: Write> Evaluator<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    fn eval_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Print(ps) => self.eval_print(ps),
        }
    }

    fn eval_line(&mut self, line: &Line) {
        for stmt in &line.statements {
            self.eval_statement(stmt);
        }
    }

    pub fn eval_program(&mut self, prog: &Program) {
        for e in &prog.lines {
            self.eval_line(e);
        }
    }

    fn eval_print(&mut self, ps: &PrintStatement) {
        let out = ps
            .args()
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Vec<_>>();
        if let Some((head, tail)) = out.split_first() {
            write!(self.writer, "{}", head).unwrap();
            for val in tail {
                write!(self.writer, " {}", val).unwrap();
            }
        }
        writeln!(self.writer).unwrap();
    }

    fn eval_expr(&self, expr: &Expr) -> String {
        match expr.kind {
            ExprKind::Int64(val) => format!("{}", val),
        }
    }
}
