use crate::tokenizer::Keyword;
use crate::tokenizer::Span;
use crate::tokenizer::Token;
use crate::tokenizer::TokenKind;

#[derive(Debug, Clone)]
pub struct Tokenizer<'s> {
    buf: &'s [u8],
    offset: usize,
    span: Span,
}

impl<'s> Tokenizer<'s> {
    pub fn new(buf: &'s [u8]) -> Self {
        Self {
            buf,
            offset: 0,
            span: Span::default(),
        }
    }

    pub fn span_of(&self, span: Span) -> &'s [u8] {
        let offset = span.offset as usize;
        let len = span.len as usize;
        self.buf.get(offset..offset + len).unwrap_or_default()
    }

    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.offset == self.buf.len()
    }

    #[inline(always)]
    fn make_token(&mut self, kind: TokenKind) -> Token {
        Token::new(kind, self.span)
    }

    #[inline(always)]
    fn bump(&mut self) {
        if self.offset < self.buf.len() {
            self.offset += 1;
        }
    }

    #[inline]
    fn peek(&self, ahead: usize) -> Option<u8> {
        self.buf.get(self.offset + ahead).copied()
    }

    #[inline]
    fn next(&mut self) -> u8 {
        self.bump();
        self.buf[self.offset - 1]
    }

    #[inline(always)]
    fn begin_span(&mut self) {
        self.span.offset = self.offset as u32;
        self.span.len = 0;
    }

    #[inline]
    fn skip_ws(&mut self) {
        while let Some(b'\t' | b'\x0C' | b'\r' | b' ') = self.peek(0) {
            self.bump();
        }
    }

    #[inline(always)]
    fn end_span(&mut self) {
        self.span.len = (self.offset as u32).saturating_sub(self.span.offset);
    }

    fn classify_identifier(text: &[u8]) -> TokenKind {
        macro_rules! g {
            ($($l:expr => $e:expr),* $(,)?) => {
                $(
                    if text.eq_ignore_ascii_case($l) {
                        return $e;
                    } else
                )*
                { return TokenKind::Identifier; }
            };
        }
        g! {
            b"print" => TokenKind::Keyword(Keyword::Print),
            b"input" => TokenKind::Keyword(Keyword::Input),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();
        self.begin_span();
        if self.is_eof() {
            return self.make_token(TokenKind::Eof);
        }
        let ch = self.next();
        match ch {
            b'0'..=b'9' => {
                while self
                    .peek(0)
                    .map(|e| e.is_ascii_digit())
                    .unwrap_or_default()
                {
                    self.bump();
                }
                self.end_span();
                self.make_token(TokenKind::Int64)
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while let Some(b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9') =
                    self.peek(0)
                {
                    self.bump();
                }
                self.end_span();
                self.make_token(Self::classify_identifier(
                    self.span_of(self.span),
                ))
            }
            b';' => {
                self.end_span();
                self.make_token(TokenKind::Semicolon)
            }
            b',' => {
                self.end_span();
                self.make_token(TokenKind::Comma)
            }
            b':' => {
                self.end_span();
                self.make_token(TokenKind::Colon)
            }
            b'\n' => {
                self.end_span();
                self.make_token(TokenKind::Eol)
            }
            _ => {
                self.end_span();
                self.make_token(TokenKind::Error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::Keyword;
    use crate::tokenizer::Span;
    use crate::tokenizer::Token;
    use crate::tokenizer::TokenKind;
    use crate::tokenizer::Tokenizer;

    #[test]
    fn test_eof() {
        let mut tokenizer = Tokenizer::new(b"");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Eof, Span::default()));
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Eof, Span::default()));
    }

    #[test]
    fn test_int64() {
        let mut tokenizer = Tokenizer::new(b"123456");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Int64, Span::new(0, 6)));
        assert_eq!(tokenizer.span_of(token.span), b"123456");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Eof, Span::new(6, 0)));
        assert_eq!(tokenizer.span_of(token.span), b"");
    }

    #[test]
    fn test_identifiers() {
        let mut tokenizer = Tokenizer::new(b"hello_world");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Identifier, Span::new(0, 11)));
        assert_eq!(tokenizer.span_of(token.span), b"hello_world");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Eof, Span::new(11, 0)));
        assert_eq!(tokenizer.span_of(token.span), b"");
    }

    #[test]
    fn test_keywords() {
        let mut tokenizer = Tokenizer::new(b"PRINT     12345");
        let token = tokenizer.next_token();
        assert_eq!(
            token,
            Token::new(TokenKind::Keyword(Keyword::Print), Span::new(0, 5))
        );
        assert_eq!(tokenizer.span_of(token.span), b"PRINT");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Int64, Span::new(10, 5)));
        assert_eq!(tokenizer.span_of(token.span), b"12345");
        let token = tokenizer.next_token();
        assert_eq!(token, Token::new(TokenKind::Eof, Span::new(15, 0)));
        assert_eq!(tokenizer.span_of(token.span), b"");
    }
}
