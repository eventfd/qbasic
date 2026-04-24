use std::io::Write;

use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Statement;
use crate::parser::ast::StatementList;

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

    pub fn eval_program(&mut self, stmts: StatementList) {
        for e in stmts.statements() {
            self.eval_statement(e);
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
