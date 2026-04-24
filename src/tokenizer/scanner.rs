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
        self.buf
            .get(span.offset..span.offset + span.len)
            .unwrap_or_default()
    }

    #[inline(always)]
    fn is_eof(&self) -> bool {
        self.offset == self.buf.len()
    }

    #[inline(always)]
    fn finish(&mut self, kind: TokenKind) -> Token {
        self.end_span();
        Token::new(kind, self.span)
    }

    #[inline(always)]
    fn bump(&mut self) {
        self.offset = self.buf.len().min(1 + self.offset);
    }

    #[inline]
    fn peek(&self, ahead: usize) -> u8 {
        *self.buf.get(self.offset + ahead).unwrap_or(&0)
    }

    #[inline]
    fn next(&mut self) -> u8 {
        self.bump();
        self.buf[self.offset - 1]
    }

    #[inline(always)]
    fn begin_span(&mut self) {
        self.span.offset = self.offset;
        self.span.len = 0;
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(0), b'\t' | b'\x0C' | b'\r' | b' ') {
            self.bump();
        }
    }

    #[inline(always)]
    fn end_span(&mut self) {
        self.span.len = self.offset.saturating_sub(self.span.offset);
    }

    fn classify_identifier(&self) -> TokenKind {
        let text = self.span_of(self.span).to_ascii_lowercase();
        match &*text {
            b"print" => TokenKind::Keyword(Keyword::Print),
            b"input" => TokenKind::Keyword(Keyword::Input),
            _ => TokenKind::Identifier,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();
        self.begin_span();
        if self.is_eof() {
            return self.finish(TokenKind::Eof);
        }
        let ch = self.next();
        match ch {
            b'0'..=b'9' => {
                while self.peek(0).is_ascii_digit() {
                    self.next();
                }
                self.finish(TokenKind::Int64)
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while matches!(self.peek(0), b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9') {
                    self.next();
                }
                self.end_span();
                self.finish(self.classify_identifier())
            }
            b';' => self.finish(TokenKind::Semicolon),
            b',' => self.finish(TokenKind::Comma),
            b'\n' => self.finish(TokenKind::Eol),
            _ => todo!(),
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
