use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Op(Op),

    Whitespace,
    Comment(String),
    Eof,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Keyword {
    From,
    Select,
    Where,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    Minus,
    Plus,
    Star,
    Slash,
    Percent,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

impl FromStr for Keyword {
    type Err = TokenParseError;

    fn from_str(s: &str) -> Result<Keyword, TokenParseError> {
        match s.to_lowercase().as_str() {
            "from" => Ok(Keyword::From),
            "select" => Ok(Keyword::Select),
            "where" => Ok(Keyword::Where),
            _ => Err(TokenParseError),
        }
    }
}

impl FromStr for Op {
    type Err = TokenParseError;

    fn from_str(s: &str) -> Result<Op, TokenParseError> {
        match s {
            "+" => Ok(Op::Plus),
            "-" => Ok(Op::Minus),
            "*" => Ok(Op::Star),
            "/" => Ok(Op::Slash),
            "%" => Ok(Op::Percent),
            "=" => Ok(Op::Eq),
            "<" => Ok(Op::Lt),
            ">" => Ok(Op::Gt),
            "!=" => Ok(Op::NotEq),
            "<>" => Ok(Op::NotEq),
            "<=" => Ok(Op::LtEq),
            ">=" => Ok(Op::GtEq),
            _ => Err(TokenParseError),
        }
    }
}

pub struct TokenParseError;
