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
