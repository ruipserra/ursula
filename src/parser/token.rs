pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Special(String),

    Whitespace,
    Comment,
    Eof,
}

pub enum Keyword {
    From,
    Select,
    Where,
}
