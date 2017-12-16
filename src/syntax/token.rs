use std::fmt;

use error::{Error, Result};
use value::Value;
use interpreter::Argument;
use interpreter::Variable;

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

    /// Translates a Token to a Value, looking it up in the context if
    /// necessary
    pub fn to_arg(&self) -> Result<Argument> {
        match *self {
            Token::NumberLiteral(f) => Ok(Argument::Val(Value::Num(f))),
            Token::StringLiteral(ref s) => Ok(Argument::Val(Value::Str(s.clone()))),
            Token::BooleanLiteral(b) => Ok(Argument::Val(Value::Bool(b))),
            Token::Identifier(ref id) => Ok(Argument::Var(Variable::new(id.as_ref()))),
            ref x => Error::parser("Argument", Some(x)),
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

#[cfg(test)]
mod test {
    use super::*;
    use interpreter::Context;

    #[test]
    fn evaluate_handles_string_literals() {
        let ctx = Context::new();
        let t = Token::StringLiteral("hello".to_owned());
        assert_eq!(t.to_arg().unwrap().evaluate(&ctx).unwrap(),
                   Value::str("hello"));
    }

    #[test]
    fn evaluate_handles_number_literals() {
        let ctx = Context::new();
        assert_eq!(Token::NumberLiteral(42f32)
                       .to_arg()
                       .unwrap()
                       .evaluate(&ctx)
                       .unwrap(),
                   Value::Num(42f32));
    }

    #[test]
    fn evaluate_handles_boolean_literals() {
        let ctx = Context::new();
        assert_eq!(Token::BooleanLiteral(true)
                       .to_arg()
                       .unwrap()
                       .evaluate(&ctx)
                       .unwrap(),
                   Value::Bool(true));

        assert_eq!(Token::BooleanLiteral(false)
                       .to_arg()
                       .unwrap()
                       .evaluate(&ctx)
                       .unwrap(),
                   Value::Bool(false));
    }

    #[test]
    fn evaluate_handles_identifiers() {
        let mut ctx = Context::new();
        ctx.set_val("var0", Value::Num(42f32));
        assert_eq!(Token::Identifier("var0".to_owned())
                       .to_arg()
                       .unwrap()
                       .evaluate(&ctx)
                       .unwrap(),
                   Value::Num(42f32));
        assert!(Token::Identifier("nope".to_owned())
                    .to_arg()
                    .unwrap()
                    .evaluate(&ctx)
                    .is_err());
    }

    #[test]
    fn evaluate_returns_none_on_invalid_token() {
        assert!(Token::DotDot.to_arg().is_err());
    }
}
