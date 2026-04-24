pub(crate) mod ast;

use anyhow::Context;

use crate::parser::ast::ArgList;
use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Statement;
use crate::parser::ast::StatementList;
use crate::tokenizer::Keyword;
use crate::tokenizer::Token;
use crate::tokenizer::TokenKind;
use crate::tokenizer::Tokenizer;

#[derive(Debug, Clone)]
pub struct Parser<'s> {
    tokenizer: Tokenizer<'s>,
    curr: Option<Token>,
    peek: Option<Token>,
}

pub type ParseError = anyhow::Error;
pub type ParseResult<T> = anyhow::Result<T>;

impl<'s> Parser<'s> {
    pub fn new(tokenizer: Tokenizer<'s>) -> Self {
        let mut p = Self {
            tokenizer,
            curr: None,
            peek: None,
        };
        p.bump();
        p.bump();
        p
    }

    #[inline(always)]
    fn bump(&mut self) {
        self.curr = self.peek.replace(self.tokenizer.next_token());
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<()> {
        match &self.curr {
            Some(k) if k.kind == kind => {
                self.bump();
                Ok(())
            }
            _ => anyhow::bail!(
                "Parser::expect(kind: {:?}) failed, got: {:?}",
                kind,
                self.curr
            ),
        }
    }

    fn is_eol(&self) -> bool {
        self.curr
            .as_ref()
            .map(|t| {
                matches!(
                    t.kind,
                    TokenKind::Eof | TokenKind::Semicolon | TokenKind::Eol
                )
            })
            .unwrap_or_default()
    }

    fn peek(&self, kind: TokenKind) -> bool {
        self.curr
            .as_ref()
            .map(|t| t.kind == kind)
            .unwrap_or_default()
    }

    pub fn parse(&mut self) -> ParseResult<StatementList> {
        let mut stmts = vec![];
        while let Ok(stmt) = self.parse_statement() {
            stmts.push(stmt);
        }
        Ok(stmts.into())
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        self.expect(TokenKind::Keyword(Keyword::Print))?;
        Ok(Statement::Print(PrintStatement::new(self.parse_arglist()?)))
    }

    fn parse_arglist(&mut self) -> ParseResult<ArgList> {
        if self.is_eol() {
            return Ok(ArgList::new(vec![]));
        }
        let mut args = vec![self.parse_expr()?];
        while self.expect(TokenKind::Comma).is_ok() {
            let expr = self.parse_expr().context("Expected Expression")?;
            args.push(expr);
        }
        Ok(args.into())
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        if self.peek(TokenKind::Int64)
            && let Some(curr) = &self.curr
        {
            let span = curr.span;
            self.bump();
            let view = self.tokenizer.span_of(span);
            let text = core::str::from_utf8(view)
                .context("Parser::parse_expr failed to decode integer from bytes")?;
            return Ok(Expr::new(span, ExprKind::Int64(text.parse()?)));
        }
        anyhow::bail!("Not Implemented")
    }
}

#[cfg(test)]
mod tests {
    mod print_statement {
        use crate::eval::tree_walk::Evaluator;
        use crate::parser::Parser;
        use crate::parser::ast::ArgList;
        use crate::parser::ast::Expr;
        use crate::parser::ast::ExprKind;
        use crate::parser::ast::PrintStatement;
        use crate::parser::ast::Statement;
        use crate::tokenizer::Span;
        use crate::tokenizer::Tokenizer;

        #[test]
        fn test_empty() {
            let text = b"PRINT";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast.statements(),
                vec![Statement::Print(PrintStatement::new(vec![].into()))]
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(ast);
            assert_eq!(stdout, b"\n");
        }

        #[test]
        fn test_one_arg() {
            let text = b"PRINT 123\n";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast.statements(),
                vec![Statement::Print(PrintStatement::new(ArgList::new(vec![
                    Expr::new(Span::new(6, 3), ExprKind::Int64(123))
                ])))]
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(ast);
            assert_eq!(stdout, b"123\n");
        }

        #[test]
        fn test_many_args() {
            let text = b"PRINT 123, 4567, 89101112";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast.statements(),
                vec![Statement::Print(PrintStatement::new(ArgList::new(vec![
                    Expr::new(Span::new(6, 3), ExprKind::Int64(123)),
                    Expr::new(Span::new(11, 4), ExprKind::Int64(4567)),
                    Expr::new(Span::new(17, 8), ExprKind::Int64(89101112)),
                ])))]
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(ast);
            assert_eq!(stdout, b"123 4567 89101112\n");
        }
    }
}
