#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Special(String),

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
