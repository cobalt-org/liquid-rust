use std::fmt;

use error::{Error, Result};

use super::Value;

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

impl Token {
    /// Parses a token that can possibly represent a Value
    /// to said Value. Returns an Err if the token can not
    /// be interpreted as a Value.
    pub fn to_value(&self) -> Result<Value> {
        match self {
            &Token::StringLiteral(ref x) => Ok(Value::str(x)),
            &Token::NumberLiteral(x) => Ok(Value::Num(x)),
            &Token::BooleanLiteral(x) => Ok(Value::Bool(x)),
            x => Error::parser("Value", Some(x)),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match *self {
            Token::Pipe => "|".to_owned(),
            Token::Dot => ".".to_owned(),
            Token::Colon => ":".to_owned(),
            Token::Comma => ",".to_owned(),
            Token::OpenSquare => "[".to_owned(),
            Token::CloseSquare => "]".to_owned(),
            Token::OpenRound => "(".to_owned(),
            Token::CloseRound => ")".to_owned(),
            Token::Question => "?".to_owned(),
            Token::Dash => "-".to_owned(),
            Token::DotDot => "..".to_owned(),
            Token::Assignment => "=".to_owned(),
            Token::Or => "or".to_owned(),

            Token::Comparison(ComparisonOperator::Equals) => "==".to_owned(),
            Token::Comparison(ComparisonOperator::NotEquals) => "!=".to_owned(),
            Token::Comparison(ComparisonOperator::LessThanEquals) => "<=".to_owned(),
            Token::Comparison(ComparisonOperator::GreaterThanEquals) => ">=".to_owned(),
            Token::Comparison(ComparisonOperator::LessThan) => "<".to_owned(),
            Token::Comparison(ComparisonOperator::GreaterThan) => ">".to_owned(),
            Token::Comparison(ComparisonOperator::Contains) => "contains".to_owned(),
            Token::Identifier(ref x) |
            Token::StringLiteral(ref x) => x.clone(),
            Token::NumberLiteral(ref x) => x.to_string(),
            Token::BooleanLiteral(ref x) => x.to_string(),
        };
        write!(f, "{}", out)
    }
}
