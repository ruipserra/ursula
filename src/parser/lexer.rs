use parser::token::Token;
use parser::errors::SyntaxError;

pub type BytePos = usize;

#[derive(Debug, PartialEq, Eq)]
pub struct LexedToken {
    pub token: Token,
    pub span: Span,
}

impl LexedToken {
    fn new(token: Token, start: BytePos, end: BytePos) -> LexedToken {
        LexedToken {
            token: token,
            span: Span {
                start: start,
                end: end,
            }
        }
    }

    fn new_at(token: Token, pos: BytePos) -> LexedToken {
        LexedToken::new(token, pos, pos)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

pub struct StringReader<'a> {
    /// The input to be read.
    pub input: &'a str,

    /// The byte position of the previously read char.
    pub prev_pos: BytePos,

    /// The byte position of the current char.
    pub curr_pos: BytePos,

    /// The current char.
    pub curr_char: Option<char>,
}

impl<'a> StringReader<'a> {
    pub fn new(input: &'a str) -> StringReader {
        StringReader {
            input: input,
            prev_pos: 0,
            curr_pos: 0,
            curr_char: input.chars().next(),
        }
    }

    /// Returns true if no more input to read, false otherwise.
    pub fn is_eof(&self) -> bool {
        self.curr_char.is_none()
    }

    /// Returns true if `curr_char` is a newline character, false otherwise.
    pub fn is_eol(&self) -> bool {
        match self.curr_char {
            Some('\n') => true,
            _ => false
        }
    }

    /// Advances `prev_pos` and `curr_pos`
    pub fn advance(&mut self) {
        self.prev_pos = self.curr_pos;

        if let Some(c) = self.curr_char {
            self.curr_pos += c.len_utf8();
        }

        self.curr_char = self.char_at(self.curr_pos);
    }

    /// Advances the string reader's position while the given condition is satisfied.
    pub fn advance_while<F: Fn(char) -> bool>(&mut self, test: F) {
        while self.curr_char.is_some() && test(self.curr_char.unwrap()) {
            self.advance();
        }
    }

    /// Returns the next char to be read without advancing the string reader.
    pub fn peek_next(&self) -> Option<char> {
        if self.curr_char.is_some() {
            self.char_at(self.curr_pos + self.curr_char.unwrap().len_utf8())
        } else {
            None
        }
    }

    pub fn curr_is(&self, c: char) -> bool {
        self.curr_char.is_some() && self.curr_char.unwrap() == c
    }

    pub fn next_is(&self, c: char) -> bool {
        let next_char = self.peek_next();
        next_char.is_some() && next_char.unwrap() == c
    }

    pub fn read_line(&mut self) -> String {
        let mut s = String::new();

        while !self.is_eof() && !self.is_eol() {
            s.push(self.curr_char.unwrap());
            self.advance();
        }

        // Move to start of next line.
        self.advance();

        s
    }

    /// Returns `Some(c)` containing the char at the specified byte position if found,
    /// otherwise returns None.
    fn char_at(&self, pos: BytePos) -> Option<char> {
        if self.input.len() > pos {
            self.input[pos..].chars().next()
        } else {
            None
        }
    }
}

pub struct Lexer<'a> {
    reader: StringReader<'a>,
    curr_tok: Option<LexedToken>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            reader: StringReader::new(input),
            curr_tok: None,
        }
    }

    pub fn next_token(&mut self) -> Result<LexedToken, SyntaxError> {
        // This is really verbose. There's probably a better way to do this...
        if let Some(t) = self.scan_whitespace() {
            Ok(t)
        } else if let Some(t) = self.scan_comment() {
            Ok(t)
        } else if let Some(t) = self.scan_eof() {
            Ok(t)
        } else {
            // FIXME Dummy return value. This is just to satisfy the compiler for now.
            Ok(LexedToken::new_at(Token::Ident("foo".to_owned()), 0))
        }
    }

    fn scan_whitespace(&mut self) -> Option<LexedToken> {
        if is_whitespace(self.reader.curr_char) {
            let start = self.reader.curr_pos;
            self.consume_whitespace();
            Some(LexedToken::new(Token::Whitespace, start, self.reader.prev_pos))
        } else {
            None
        }
    }

    fn scan_comment(&mut self) -> Option<LexedToken> {
        if self.reader.curr_is('-') && self.reader.next_is('-') {
            let start = self.reader.curr_pos;

            // Move past the `--` characters.
            self.reader.advance();
            self.reader.advance();

            let comment = self.reader.read_line();
            Some(LexedToken::new(Token::Comment(comment), start, self.reader.prev_pos))
        } else {
            None
        }
    }

    fn scan_eof(&mut self) -> Option<LexedToken> {
        if self.reader.is_eof() {
            Some(LexedToken::new_at(Token::Eof, self.reader.prev_pos))
        } else {
            None
        }
    }

    fn consume_whitespace(&mut self) {
        self.reader.advance_while(char::is_whitespace);
    }
}

fn is_whitespace(c: Option<char>) -> bool {
    c.is_some() && c.unwrap().is_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::token::Token;
    use parser::errors::SyntaxError;

    fn expected_token(token: Token, start: BytePos, end: BytePos) -> Result<LexedToken, SyntaxError> {
        Ok(LexedToken::new(token, start, end))
    }

    #[test]
    fn retuns_eof_for_empty_string() {
        assert_eq!(
            expected_token(Token::Eof, 0, 0),
            Lexer::new("").next_token()
        );
    }

    #[test]
    fn retuns_whitespace_and_eof_for_string_with_only_whitespace() {
        let mut reader = Lexer::new("   ");

        assert_eq!(
            expected_token(Token::Whitespace, 0, 2),
            reader.next_token()
        );

        assert_eq!(
            expected_token(Token::Eof, 2, 2),
            reader.next_token()
        );
    }

    #[test]
    fn returns_comment_token_for_dash_dash_comment_only() {
        let mut reader = Lexer::new("-- comment");

        assert_eq!(
            expected_token(Token::Comment(" comment".to_string()), 0, 10),
            reader.next_token()
        );

        assert_eq!(
            expected_token(Token::Eof, 10, 10),
            reader.next_token()
        );
    }
}
