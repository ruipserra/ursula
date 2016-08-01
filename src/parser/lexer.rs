use parser::token::{Token, Keyword};
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
            },
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
            _ => false,
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

    pub fn read_while<F: Fn(char) -> bool>(&mut self, test: F) -> String {
        let mut s = String::new();

        while let Some(c) = self.curr_char {
            if test(c) {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }

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
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { reader: StringReader::new(input) }
    }

    pub fn next_token(&mut self) -> Result<LexedToken, SyntaxError> {
        if let Some(t) = self.next_token_opt() {
            Ok(t)
        } else {
            assert!(self.reader.curr_char.is_some());
            self.next_token_res()
        }
    }

    fn next_token_opt(&mut self) -> Option<LexedToken> {
        self.scan_eof()
            .or_else(|| self.scan_whitespace())
            .or_else(|| self.scan_comment())
    }

    fn next_token_res(&mut self) -> Result<LexedToken, SyntaxError> {
        match self.reader.curr_char {
            Some(c) if is_ident_start(c) => self.scan_keyword_or_unquoted_identifier(),
            _ => unimplemented!(),
        }
    }

    fn scan_whitespace(&mut self) -> Option<LexedToken> {
        let c = self.reader.curr_char.unwrap_or('\0');

        if c.is_whitespace() {
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

    fn scan_keyword_or_unquoted_identifier(&mut self) -> Result<LexedToken, SyntaxError> {
        assert!(is_ident_start(self.reader.curr_char.unwrap()));

        let start = self.reader.curr_pos;
        let ident = self.reader
            .read_while(is_ident_cont)
            .to_lowercase(); // Keywords and unquoted identifiers are case insensitive.

        let tok = if let Some(keyword) = Keyword::from_str(&ident) {
            Token::Keyword(keyword)
        } else {
            Token::Ident(ident)
        };

        Ok(LexedToken::new(tok, start, self.reader.prev_pos))
    }

    fn consume_whitespace(&mut self) {
        self.reader.advance_while(char::is_whitespace);
    }
}

fn is_ident_start(c: char) -> bool {
    match c {
        'a'...'z' |
        'A'...'Z' |
        '\u{80}'...'\u{FF}' |
        '_' => true,
        _ => false,
    }
}

fn is_ident_cont(c: char) -> bool {
    match c {
        'a'...'z' |
        'A'...'Z' |
        '\u{80}'...'\u{FF}' |
        '0'...'9' |
        '_' |
        '$' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::token::{Token, Keyword};
    use parser::errors::SyntaxError;

    fn expected_token(token: Token,
                      start: BytePos,
                      end: BytePos)
                      -> Result<LexedToken, SyntaxError> {
        Ok(LexedToken::new(token, start, end))
    }

    #[test]
    fn retuns_eof_for_empty_string() {
        assert_eq!(expected_token(Token::Eof, 0, 0),
                   Lexer::new("").next_token());
    }

    #[test]
    fn retuns_whitespace_and_eof_for_string_with_only_whitespace() {
        let mut lexer = Lexer::new("   ");

        assert_eq!(expected_token(Token::Whitespace, 0, 2), lexer.next_token());
        assert_eq!(expected_token(Token::Eof, 2, 2), lexer.next_token());
    }

    #[test]
    fn returns_comment_token_for_dash_dash_comment_only() {
        let mut lexer = Lexer::new("-- comment");

        assert_eq!(expected_token(Token::Comment(" comment".to_string()), 0, 10),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Eof, 10, 10), lexer.next_token());
    }

    #[test]
    fn recognizes_keywords_regardless_of_case() {
        let mut lexer = Lexer::new("select FROM WhErE");

        assert_eq!(expected_token(Token::Keyword(Keyword::Select), 0, 5),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 6, 6), lexer.next_token());

        assert_eq!(expected_token(Token::Keyword(Keyword::From), 7, 10),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 11, 11),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Keyword(Keyword::Where), 12, 16),
                   lexer.next_token());
    }

    #[test]
    fn returns_downcased_unquoted_identifiers() {
        let mut lexer = Lexer::new("_foo BaR IDENT2 ídèñt$3_");

        assert_eq!(expected_token(Token::Ident("_foo".to_string()), 0, 3),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 4, 4), lexer.next_token());

        assert_eq!(expected_token(Token::Ident("bar".to_string()), 5, 7),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 8, 8), lexer.next_token());

        assert_eq!(expected_token(Token::Ident("ident2".to_string()), 9, 14),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 15, 15),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Ident("ídèñt$3_".to_string()), 16, 26),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Eof, 26, 26), lexer.next_token());
    }
}
