use std::str::FromStr;

use parser::token::{Token, Keyword, Op};
use parser::errors::SyntaxError;

pub type BytePos = usize;

/// Encapsulates a token and the byte positions it spans.
#[derive(Debug, PartialEq, Eq)]
pub struct LexedToken {
    /// The token that was found.
    pub token: Token,
    /// Where the token was found.
    pub span: Span,
}

impl LexedToken {
    fn new(token: Token, start: BytePos, end: BytePos) -> LexedToken {
        LexedToken {
            token: token,
            span: Span::new(start, end),
        }
    }

    fn new_at(token: Token, pos: BytePos) -> LexedToken {
        LexedToken::new(token, pos, pos)
    }
}

/// Represents a byte range of a text segment.
#[derive(Debug, PartialEq, Eq)]
pub struct Span {
    /// Byte position where the text segment starts.
    pub start: BytePos,
    /// Byte position where the text segment ends.
    pub end: BytePos,
}

impl Span {
    pub fn new(start: BytePos, end: BytePos) -> Span {
        assert!(start <= end);

        Span {
            start: start,
            end: end,
        }
    }
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

    /// Advances the string reader's position by the given number of bytes.
    pub fn advance_bytes(&mut self, n_bytes: usize) {
        self.prev_pos = self.curr_pos;
        self.curr_pos += n_bytes;
        self.curr_char = self.char_at(self.curr_pos);
    }

    /// Returns the next char to be read without advancing the string reader.
    pub fn peek_next(&self) -> Option<char> {
        if self.curr_char.is_some() {
            self.char_at(self.curr_pos + self.curr_char.unwrap().len_utf8())
        } else {
            None
        }
    }

    /// Returns `true` if the current `char` is the same as the given `char`,
    /// returns `false` otherwise.
    pub fn curr_is(&self, c: char) -> bool {
        self.curr_char.is_some() && self.curr_char.unwrap() == c
    }

    /// Returns `true` if the next `char` is the same as the given `char`,
    /// returns `false` otherwise.
    pub fn next_is(&self, c: char) -> bool {
        let next_char = self.peek_next();
        next_char.is_some() && next_char.unwrap() == c
    }

    /// Reads the input into a `String` until a new line is found,
    /// advancing the current position. Returns the read input.
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

    /// Reads the input into a `String` while the given condition is met,
    /// advancing the current position. Returns the read input.
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

    /// Consumes input until a token is found, and returns that token.
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
            .or_else(|| self.scan_operator())
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

    fn scan_operator(&mut self) -> Option<LexedToken> {
        self.scan_multi_byte_operator()
            .or_else(|| self.scan_single_byte_operator())
    }

    fn scan_single_byte_operator(&mut self) -> Option<LexedToken> {
        let curr = self.reader.curr_char.unwrap();

        if !is_single_byte_op_char(curr) {
            return None;
        }

        match Op::from_str(curr.to_string().as_str()) {
            Ok(op) => {
                let pos = self.reader.curr_pos;
                self.reader.advance();
                Some(LexedToken::new_at(Token::Op(op), pos))
            }
            _ => None,
        }
    }

    fn scan_multi_byte_operator(&mut self) -> Option<LexedToken> {
        let curr = self.reader.curr_char.unwrap();
        if !is_multi_byte_op_start(curr) {
            return None;
        }

        let mut s = curr.to_string();

        let next = self.reader.peek_next().unwrap_or('\0');
        if is_multi_byte_op_cont(next) {
            s.push(next);
        }

        match Op::from_str(s.as_str()) {
            Ok(op) => {
                let start = self.reader.curr_pos;
                let end = start + s.len() - 1;

                self.reader.advance_bytes(s.len());

                Some(LexedToken::new(Token::Op(op), start, end))
            }
            _ => None,
        }
    }

    fn scan_keyword_or_unquoted_identifier(&mut self) -> Result<LexedToken, SyntaxError> {
        assert!(is_ident_start(self.reader.curr_char.unwrap()));

        let start = self.reader.curr_pos;
        let ident = self.reader
            .read_while(is_ident_cont)
            .to_lowercase(); // Keywords and unquoted identifiers are case insensitive.

        let tok = if let Ok(keyword) = Keyword::from_str(&ident) {
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

fn is_single_byte_op_char(c: char) -> bool {
    match c {
        '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' => true,
        _ => false,
    }
}

fn is_multi_byte_op_start(c: char) -> bool {
    match c {
        '!' | '<' | '>' => true,
        _ => false,
    }
}

fn is_multi_byte_op_cont(c: char) -> bool {
    match c {
        '=' | '>' => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::token::{Token, Keyword, Op};
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

    #[test]
    fn recognizes_operators_surrounded_by_whitespace() {
        let mut lexer = Lexer::new("+ - * / % = != <> <= >= < >");

        assert_eq!(expected_token(Token::Op(Op::Plus), 0, 0),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 1, 1),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Minus), 2, 2),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 3, 3),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Star), 4, 4),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 5, 5),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Slash), 6, 6),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 7, 7),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Percent), 8, 8),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 9, 9),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Eq), 10, 10),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 11, 11),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::NotEq), 12, 13),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 14, 14),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::NotEq), 15, 16),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 17, 17),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::LtEq), 18, 19),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 20, 20),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::GtEq), 21, 22),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 23, 23),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Lt), 24, 24),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Whitespace, 25, 25),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Gt), 26, 26),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Eof, 26, 26),
                   lexer.next_token());
    }

    #[test]
    fn recognizes_operators_even_when_joined_together() {
        let mut lexer = Lexer::new("+-*/%=!=<><=>=><");

        assert_eq!(expected_token(Token::Op(Op::Plus), 0, 0),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Minus), 1, 1),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Star), 2, 2),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Slash), 3, 3),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Percent), 4, 4),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Eq), 5, 5),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::NotEq), 6, 7),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::NotEq), 8, 9),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::LtEq), 10, 11),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::GtEq), 12, 13),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Gt), 14, 14),
                   lexer.next_token());

        assert_eq!(expected_token(Token::Op(Op::Lt), 15, 15),
                   lexer.next_token());
    }
}
