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

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s.to_lowercase().as_str() {
            "from" => Some(Keyword::From),
            "select" => Some(Keyword::Select),
            "where" => Some(Keyword::Where),
            _ => None,
        }
    }
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

impl Op {
    pub fn from_str(s: &str) -> Option<Op> {
        match s {
            "+" => Some(Op::Plus),
            "-" => Some(Op::Minus),
            "*" => Some(Op::Star),
            "/" => Some(Op::Slash),
            "%" => Some(Op::Percent),
            "=" => Some(Op::Eq),
            "<" => Some(Op::Lt),
            ">" => Some(Op::Gt),
            "!=" => Some(Op::NotEq),
            "<>" => Some(Op::NotEq),
            "<=" => Some(Op::LtEq),
            ">=" => Some(Op::GtEq),
            _ => None,
        }
    }
}
