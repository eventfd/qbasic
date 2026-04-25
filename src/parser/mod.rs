pub mod ast;
pub mod error;

use core::fmt::Debug;
use core::str::FromStr;

use crate::parser::ast::ArgList;
use crate::parser::ast::Assignment;
use crate::parser::ast::Expr;
use crate::parser::ast::ExprKind;
use crate::parser::ast::LValue;
use crate::parser::ast::Line;
use crate::parser::ast::PrintStatement;
use crate::parser::ast::Program;
use crate::parser::ast::Statement;
use crate::parser::ast::SymbolId;
use crate::parser::ast::SymbolTable;
use crate::parser::error::ParseError;
use crate::parser::error::ParseResult;
use crate::tokenizer::Token;
use crate::tokenizer::TokenKind;
use crate::tokenizer::Tokenizer;

#[derive(Debug)]
pub struct Parser<'s> {
    tokenizer: Tokenizer<'s>,
    curr: Token,
    peek: Token,
    errors: Vec<ParseError>,
    symtab: SymbolTable,
}

impl<'s> Parser<'s> {
    pub fn new(tokenizer: Tokenizer<'s>) -> Self {
        let mut p = Self {
            tokenizer,
            curr: Token::new(TokenKind::Eof, Default::default()),
            peek: Token::new(TokenKind::Eof, Default::default()),
            errors: Vec::new(),
            symtab: SymbolTable::default(),
        };
        p.bump();
        p.bump();
        p
    }

    // restore the parser state to parse a line
    fn synchronize(&mut self) {
        loop {
            match self.curr.kind {
                TokenKind::Eof | TokenKind::Eol => break,
                _ => self.bump(),
            }
        }
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
            Err(ParseError::new(
                self.curr.span,
                format!("Expected {:?}, Got {:?}", expected, self.curr.kind),
            ))
        }
    }

    fn expect_peek(&mut self, expected: TokenKind) -> ParseResult<()> {
        if self.peek.kind == expected {
            self.bump();
            Ok(())
        } else {
            Err(ParseError::new(
                self.curr.span,
                format!(
                    "Expected {:?} after {:?}. Got {:?}",
                    expected, self.curr.kind, self.peek.kind
                ),
            ))
        }
    }

    fn expect_any(&mut self, choices: &[TokenKind]) -> ParseResult<()> {
        for e in choices {
            if &self.curr.kind == e {
                self.bump();
                return Ok(());
            }
        }
        Err(ParseError::new(
            self.curr.span,
            format!("Expected any of {:?}, Got {:?}", choices, self.curr.kind),
        ))
    }

    #[inline(always)]
    fn is_eol(&self) -> bool {
        matches!(
            self.curr.kind,
            TokenKind::Eof | TokenKind::Semicolon | TokenKind::Eol
        )
    }

    pub fn parse(&mut self) -> Program {
        let mut rv = vec![];
        while self.curr.kind != TokenKind::Eof {
            match self.parse_line() {
                Ok(line) => {
                    rv.push(line);
                }
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                    self.bump(); // skip the synchronizing token
                }
            }
        }
        rv.into()
    }

    pub fn parse_line(&mut self) -> ParseResult<Line> {
        let peek = self.peek;
        let line_no = if self.curr.kind == TokenKind::Int64 {
            let val = self.parse_literal()?;
            self.bump();
            Some(val)
        } else {
            None
        };
        let Ok(first_stmt) = self.parse_statement() else {
            return Err(ParseError::new(peek.span, "Expected Statement"));
        };
        let mut stmts = vec![first_stmt];
        while self.curr.kind == TokenKind::Colon {
            self.bump();
            stmts.push(self.parse_statement()?);
        }
        self.expect_any(&[TokenKind::Eol, TokenKind::Eof])?;
        Ok(Line::new(line_no, stmts))
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        match self.curr.kind {
            TokenKind::Print => self.parse_print(),
            _ => self.parse_assignment(),
        }
    }

    fn parse_print(&mut self) -> ParseResult<Statement> {
        self.expect(TokenKind::Print)?;
        let args = self.parse_arglist()?;
        Ok(Statement::Print(PrintStatement::new(args)))
    }

    fn parse_assignment(&mut self) -> ParseResult<Statement> {
        let lvalue = self.parse_lvalue()?;
        self.expect(TokenKind::Eq)?;
        let rvalue = self.parse_expr()?;
        Ok(Statement::Assign(Assignment::new(lvalue, rvalue)))
    }

    fn parse_lvalue(&mut self) -> ParseResult<LValue> {
        // parse identifiers for now
        let curr = self.curr;
        self.expect(TokenKind::Identifier)?;
        let text = core::str::from_utf8(self.tokenizer.span_of(curr.span))
            .map_err(|_| {
                ParseError::new(
                    curr.span,
                    format!("Expected identifier, got {:?}", curr.kind),
                )
            })?;
        Ok(LValue::Identifier(SymbolId::new(text.into())))
    }

    fn parse_arglist(&mut self) -> ParseResult<ArgList> {
        if self.is_eol() {
            return Ok(ArgList::new(vec![]));
        }
        let mut args = vec![self.parse_expr()?];
        while self.curr.kind == TokenKind::Comma {
            self.bump();
            let expr = self.parse_expr()?;
            args.push(expr);
        }
        Ok(args.into())
    }

    fn parse_literal<R>(&self) -> ParseResult<R>
    where
        R: FromStr,
        R::Err: Debug,
    {
        let view = self.tokenizer.span_of(self.curr.span);
        let text = core::str::from_utf8(view).map_err(|e| {
            ParseError::new(
                self.curr.span,
                format!("Parser::parse_expr failed to decode integer from bytes, due to {:?}", e)
            )
        })?;
        text.parse().map_err(|e| {
            ParseError::new(
                self.curr.span,
                format!("parse_literal failed due to {:?}", e),
            )
        })
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        match self.curr.kind {
            TokenKind::Int64 => {
                let val = self.parse_literal()?;
                let val = Expr::new(self.curr.span, ExprKind::Int64(val));
                self.bump();
                Ok(val)
            }
            TokenKind::Identifier => {
                let val = self.parse_literal()?;
                let val = Expr::new(
                    self.curr.span,
                    ExprKind::Identifier(SymbolId::new(val)),
                );
                self.bump();
                Ok(val)
            }
            kind => Err(ParseError::new(
                self.curr.span,
                format!("parse_expr(kind: {:?}) - not implemented", kind,),
            )),
        }
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    mod print {
        use crate::eval::tree_walk::Evaluator;
        use crate::parser::Parser;
        use crate::parser::ast::ArgList;
        use crate::parser::ast::Expr;
        use crate::parser::ast::ExprKind;
        use crate::parser::ast::Line;
        use crate::parser::ast::PrintStatement;
        use crate::parser::ast::Program;
        use crate::parser::ast::Statement;
        use crate::parser::error::ParseError;
        use crate::tokenizer::Span;
        use crate::tokenizer::Tokenizer;

        #[test]
        fn test_empty() {
            let text = b"PRINT";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(parser.errors(), []);
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    None,
                    vec![Statement::Print(PrintStatement::new(vec![].into()))]
                )],)
            );
            let mut stdout = Vec::new();
            let mut eval = Evaluator::new(&mut stdout);
            let eval_result = eval.eval_program(&ast);
            assert_eq!(eval_result, Ok(()));
            assert_eq!(stdout, b"\n");
        }

        #[test]
        fn test_one_arg() {
            let text = b"10 PRINT 123\n";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(parser.errors(), []);
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    Some(10),
                    vec![Statement::Print(PrintStatement::new(ArgList::new(
                        vec![Expr::new(Span::new(9, 12), ExprKind::Int64(123))]
                    )))]
                )])
            );
            let mut stdout = Vec::new();
            let mut eval = Evaluator::new(&mut stdout);
            let eval_result = eval.eval_program(&ast);
            assert_eq!(eval_result, Ok(()));
            assert_eq!(stdout, b"123\n");
        }

        #[test]
        fn test_many_args() {
            let text = b"PRINT 123, 4567, 89101112";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(parser.errors(), []);
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    None,
                    vec![Statement::Print(PrintStatement::new(ArgList::new(
                        vec![
                            Expr::new(Span::new(6, 9), ExprKind::Int64(123)),
                            Expr::new(Span::new(11, 15), ExprKind::Int64(4567)),
                            Expr::new(
                                Span::new(17, 25),
                                ExprKind::Int64(89101112)
                            ),
                        ]
                    )))]
                )])
            );
            let mut stdout = Vec::new();
            let mut eval = Evaluator::new(&mut stdout);
            let eval_result = eval.eval_program(&ast);
            assert_eq!(eval_result, Ok(()));
            assert_eq!(stdout, b"123 4567 89101112\n");
        }

        #[test]
        fn test_line_no() {
            let text = b"10     PRINT 123, 4567 : PRINT 89101112";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(parser.errors(), []);
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    Some(10),
                    vec![
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![
                                Expr::new(
                                    Span::new(13, 16),
                                    ExprKind::Int64(123)
                                ),
                                Expr::new(
                                    Span::new(18, 22),
                                    ExprKind::Int64(4567)
                                ),
                            ]
                        ))),
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![Expr::new(
                                Span::new(31, 39),
                                ExprKind::Int64(89101112)
                            ),]
                        )))
                    ]
                )])
            );
            let mut stdout = Vec::new();
            let mut eval = Evaluator::new(&mut stdout);
            let eval_result = eval.eval_program(&ast);
            assert_eq!(eval_result, Ok(()));
            assert_eq!(stdout, b"123 4567\n89101112\n");
        }

        #[test]
        fn test_errors() {
            let text = br"10      INPUT a, b
            20      PRINT 123, 4567 : PRINT 89101112
            30      GOTO 20";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(
                parser.errors(),
                [
                    ParseError::new(Span::new(8, 13), "Expected Statement"),
                    ParseError::new(Span::new(92, 96), "Expected Statement"),
                ]
            );
            assert_eq!(
                ast,
                Program::new(vec![Line::new(
                    Some(20),
                    vec![
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![
                                Expr::new(
                                    Span::new(45, 48),
                                    ExprKind::Int64(123)
                                ),
                                Expr::new(
                                    Span::new(50, 54),
                                    ExprKind::Int64(4567)
                                ),
                            ]
                        ))),
                        Statement::Print(PrintStatement::new(ArgList::new(
                            vec![Expr::new(
                                Span::new(63, 71),
                                ExprKind::Int64(89101112)
                            ),]
                        )))
                    ]
                )])
            );
        }
    }

    mod assignment {
        use crate::eval::tree_walk::Evaluator;
        use crate::parser::Parser;
        use crate::tokenizer::Tokenizer;

        #[test]
        fn test_assign_basic() {
            let text = b"10     X = 10 : PRINT 1234, 5678, X\n";
            let lexer = Tokenizer::new(text);
            let mut parser = Parser::new(lexer);
            let ast = parser.parse();
            assert_eq!(parser.errors(), []);
            let mut stdout = Vec::new();
            let mut eval = Evaluator::new(&mut stdout);
            let eval_result = eval.eval_program(&ast);
            assert_eq!(eval_result, Ok(()));
            assert_eq!(stdout, b"1234 5678 10\n");
        }
    }
}
