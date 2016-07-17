use parser::token::Token;
use parser::errors::SyntaxError;

pub type CharPos = usize;

#[derive(Debug, PartialEq, Eq)]
pub struct LexedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Span {
    pub start: CharPos,
    pub end: CharPos,
}

pub struct StringReader<'a> {
    input: &'a str,
    pos: CharPos,
    curr_tok: Option<LexedToken>,
    curr_char: Option<char>,
}

impl<'a> StringReader<'a> {
    pub fn new(input: &str) -> StringReader {
        StringReader {
            input: input,
            pos: 0,
            curr_tok: None,
            curr_char: None,
        }
    }

    pub fn next_token(&mut self) -> Result<LexedToken, SyntaxError> {
        // FIXME this is a dummy implementation just to get the tests to run
        Ok(LexedToken {
            token: Token::Eof,
            span: Span {
                start: 0,
                end: 0,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::token::Token;
    use parser::errors::SyntaxError;

    fn expected_token(token: Token, start: CharPos, end: CharPos) -> Result<LexedToken, SyntaxError> {
        Ok(LexedToken {
            token: token,
            span: Span {
                start: start,
                end: end,
            }
        })
    }

    #[test]
    fn retuns_eof_for_empty_string() {
        assert_eq!(
            expected_token(Token::Eof, 0, 0),
            StringReader::new("").next_token()
        );
    }

    #[test]
    fn retuns_whitespace_and_eof_for_string_with_only_whitespace() {
        let mut reader = StringReader::new("   ");

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
        let mut reader = StringReader::new("-- comment");

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
