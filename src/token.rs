use self::Token::*;
use self::ComparisonOperator::*;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Contains,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Pipe,
    Dot,
    Colon,
    Comma,
    OpenSquare,
    CloseSquare,
    OpenRound,
    CloseRound,
    Question,
    Dash,
    Assignment,
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f32),
    BooleanLiteral(bool),
    DotDot,
    Comparison(ComparisonOperator),
    Or,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Pipe => "|".to_owned(),
            Dot => ".".to_owned(),
            Colon => ":".to_owned(),
            Comma => ",".to_owned(),
            OpenSquare => "[".to_owned(),
            CloseSquare => "]".to_owned(),
            OpenRound => "(".to_owned(),
            CloseRound => ")".to_owned(),
            Question => "?".to_owned(),
            Dash => "-".to_owned(),
            DotDot => "..".to_owned(),
            Assignment => "=".to_owned(),
            Or => "or".to_owned(),

            Comparison(Equals) => "==".to_owned(),
            Comparison(NotEquals) => "!=".to_owned(),
            Comparison(LessThanEquals) => "<=".to_owned(),
            Comparison(GreaterThanEquals) => ">=".to_owned(),
            Comparison(LessThan) => "<".to_owned(),
            Comparison(GreaterThan) => ">".to_owned(),
            Comparison(Contains) => "contains".to_owned(),
            Identifier(ref x) |
            StringLiteral(ref x) => x.clone(),
            NumberLiteral(ref x) => x.to_string(),
            BooleanLiteral(ref x) => x.to_string(),
        };
        write!(f, "{}", out)
    }
}
