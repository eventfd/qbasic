pub(crate) mod ast;

use anyhow::Context;

use crate::parser::ast::ArgList;
use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::Line;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Program;
use crate::parser::ast::Statement;
use crate::tokenizer::Keyword;
use crate::tokenizer::Token;
use crate::tokenizer::TokenKind;
use crate::tokenizer::Tokenizer;

#[derive(Debug, Clone)]
pub struct Parser<'s> {
    tokenizer: Tokenizer<'s>,
    curr: Token,
    peek: Token,
}

pub type ParseError = anyhow::Error;
pub type ParseResult<T> = anyhow::Result<T>;

impl<'s> Parser<'s> {
    pub fn new(tokenizer: Tokenizer<'s>) -> Self {
        let mut p = Self {
            tokenizer,
            curr: Token::new(TokenKind::Eof, Default::default()),
            peek: Token::new(TokenKind::Eof, Default::default()),
        };
        p.bump();
        p.bump();
        p
    }

    #[inline(always)]
    fn bump(&mut self) {
        self.curr = self.peek;
        self.peek = self.tokenizer.next_token();
    }

    fn expect(&mut self, expected: TokenKind) -> ParseResult<()> {
        if self.curr.kind == expected {
            self.bump();
            Ok(())
        } else {
            anyhow::bail!(
                "Parser::expect(kind: {:?}) failed - got {:?}",
                expected,
                &self.curr
            )
        }
    }

    #[inline(always)]
    fn is_eol(&self) -> bool {
        matches!(
            self.curr.kind,
            TokenKind::Eof | TokenKind::Semicolon | TokenKind::Eol
        )
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut rv = vec![];
        while let Ok(line) = self.parse_line() {
            rv.push(line);
        }
        Ok(rv.into())
    }

    pub fn parse_line(&mut self) -> ParseResult<Line> {
        let line_no = if self.curr.kind == TokenKind::Int64 {
            let val = self.parse_int()?.try_into()?;
            self.bump();
            Some(val)
        } else {
            None
        };
        let mut stmts = vec![self.parse_statement()?];
        while self.curr.kind == TokenKind::Colon {
            self.bump();
            stmts.push(self.parse_statement()?);
        }
        Ok(Line::new(line_no, stmts))
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
        while self.curr.kind == TokenKind::Comma {
            self.bump();
            let expr = self.parse_expr().context("Expected Expression")?;
            args.push(expr);
        }
        Ok(args.into())
    }

    fn parse_int(&self) -> ParseResult<i64> {
        let view = self.tokenizer.span_of(self.curr.span);
        let text = core::str::from_utf8(view).context(
            "Parser::parse_expr failed to decode integer from bytes",
        )?;
        text.parse().context("str::parse() failed")
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        if self.curr.kind == TokenKind::Int64 {
            let val =
                Expr::new(self.curr.span, ExprKind::Int64(self.parse_int()?));
            self.bump();
            return Ok(val);
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
        use crate::parser::ast::Line;
        use crate::parser::ast::PrintStatement;
        use crate::parser::ast::Program;
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
                ast,
                Program::new(vec![Line::new(
                    None,
                    vec![Statement::Print(PrintStatement::new(vec![].into()))]
                )],)
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(&ast);
            assert_eq!(stdout, b"\n");
        }

        #[test]
        fn test_one_arg() {
            let text = b"10 PRINT 123\n";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    Some(10),
                    vec![Statement::Print(PrintStatement::new(ArgList::new(
                        vec![Expr::new(Span::new(9, 3), ExprKind::Int64(123))]
                    )))]
                )])
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(&ast);
            assert_eq!(stdout, b"123\n");
        }

        #[test]
        fn test_many_args() {
            let text = b"PRINT 123, 4567, 89101112";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    None,
                    vec![Statement::Print(PrintStatement::new(ArgList::new(
                        vec![
                            Expr::new(Span::new(6, 3), ExprKind::Int64(123)),
                            Expr::new(Span::new(11, 4), ExprKind::Int64(4567)),
                            Expr::new(
                                Span::new(17, 8),
                                ExprKind::Int64(89101112)
                            ),
                        ]
                    )))]
                )])
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(&ast);
            assert_eq!(stdout, b"123 4567 89101112\n");
        }

        #[test]
        fn test_line_no() {
            let text = b"10     PRINT 123, 4567 : PRINT 89101112";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse().expect("parse() errors");
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    Some(10),
                    vec![
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![
                                Expr::new(
                                    Span::new(13, 3),
                                    ExprKind::Int64(123)
                                ),
                                Expr::new(
                                    Span::new(18, 4),
                                    ExprKind::Int64(4567)
                                ),
                            ]
                        ))),
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![Expr::new(
                                Span::new(31, 8),
                                ExprKind::Int64(89101112)
                            ),]
                        )))
                    ]
                )])
            );
            let mut stdout = Vec::<u8>::new();
            let mut eval = Evaluator::new(&mut stdout);
            eval.eval_program(&ast);
            assert_eq!(stdout, b"123 4567\n89101112\n");
        }
    }
}
