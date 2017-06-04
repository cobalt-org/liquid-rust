use std::str;
use std::io::prelude::*;
use std::fs::File;
use std::str::FromStr;

use regex::Regex;

use nom::*;

use error::*;

use Renderable;

use Token;
use token::Token::*;
use token::ComparisonOperator::*;
use output::{Output, FilterPrototype, Argument};
use value::Value;
use variable::Variable;

use error::{Error, Result};


lazy_static! {
    static ref MARKUP: Regex = {
        let t = "(?:[[:space:]]*\\{\\{-|\\{\\{).*?(?:-\\}\\}[[:space:]]*|\\}\\})";
        let e = "(?:[[:space:]]*\\{%-|\\{%).*?(?:-%\\}[[:space:]]*|%\\})";
        Regex::new(&format!("{}|{}", t, e)).unwrap()
    };
}

lazy_static! {
    static ref EXPRESSION: Regex = {
        let t = "(?:[[:space:]]*\\{\\{-|\\{\\{)(.*?)(?:-\\}\\}[[:space:]]*|\\}\\})";
        Regex::new(t).unwrap()
    };
    static ref TAG: Regex = {
        let e = "(?:[[:space:]]*\\{%-|\\{%)(.*?)(?:-%\\}[[:space:]]*|%\\})";
        Regex::new(e).unwrap()
    };
}

lazy_static! {
    static ref SPLIT: String =
        r#"'.*?'|".*?"|\s+|[\|:,\[\]\(\)\?]|\.\.|={1,2}|!=|<=|>=|[<>]"#.to_string();
}

lazy_static! {
    static ref IDENTIFIER: String = r"^[a-zA-Z_][\w-]*\??".to_string();
    static ref SINGLE_STRING_LITERAL: String = "^'[^']*'".to_string();
    static ref DOUBLE_STRING_LITERAL: String = r#"^"[^"]*""#.to_string();
    static ref NUMBER_LITERAL: String = r#"^-?\d+(\.\d+)?"#.to_string();
    static ref PIPE: String = r#"^\|"#.to_string();

    // Truncated because contains can't use a positive lookahead
    static ref COMPARISON_OPERATORS: String = r#"==|!=|<>|<=?|>=?"#.to_string();

    // Not part of Shopify/liquid
    static ref WHITESPACE: String = r#"\s"#.to_string();
    static ref BOOLEAN_LITERAL: String = "true|false".to_string();
    static ref NIL_LITERAL: String = "nil|null".to_string();
}

named!(pipe<&str, &str>, re_find!(&PIPE));

named!(string<&str, Token>,
    map!(
         alt_complete!(re_find!(&SINGLE_STRING_LITERAL) | re_find!(&DOUBLE_STRING_LITERAL)),
         |string: &str| {
            let val = string[1..string.len() - 1].to_string();
            Token::StringLiteral(val)
         }));

named!(number<&str, Token>,
    map!(
        re_find!(&NUMBER_LITERAL),
        |num: &str| {
            let val = num.parse::<f32>().unwrap();
            Token::NumberLiteral(val)
        }));

named!(boolean<&str, Token>,
    map!(
        re_find!(&BOOLEAN_LITERAL),
        |b: &str| {
            let val = b
                .parse::<bool>()
                .unwrap();
            Token::BooleanLiteral(val)
        }));

named!(nil<&str, Token>,
    map!(
        re_find!(&NIL_LITERAL),
        |n: &str| {
        Token::NilLiteral
    }));

named!(literal<&str, Token>,
    alt_complete!(string | number | boolean | nil));

named!(identifier<&str, Token>,
    map!(
        re_find!(&IDENTIFIER),
        |i: &str| {
            let val = i.to_string();
            Token::Identifier(val)
        }));

named!(value<&str, Token>, alt_complete!(literal | identifier));

named!(contains_keyword<&str, &str>,
   do_parse!(
       contains: tag!("contains") >>
       peek!(not!(re_find!(&r#"[^\s]"#.to_string()))) >>
       (contains)
   )
);

named!(comparison_operator<&str, Token>,
    map!(
        alt_complete!(re_find!(&COMPARISON_OPERATORS)|contains_keyword),
        |c: &str| {
            let comp = match c {
                "==" => Equals,
                "!=" => NotEquals,
                "<>" => NotEquals,
                "<" => LessThan,
                ">" => GreaterThan,
                "<=" => LessThanEquals,
                ">=" => GreaterThanEquals,
                "contains" => Contains,
                _ => panic!("should never happen"),
            };
            Comparison(comp)
        }));

fn argument_from_token(t: Token) -> Argument {
    match t {
        x @ StringLiteral(_) |
        x @ NumberLiteral(_) |
        x @ BooleanLiteral(_) => Argument::Val(Value::from_token(&x).unwrap()),
        Identifier(ref v) => Argument::Var(Variable::new(v)),
        x => panic!("argument not supported: {:?}", x),
    }
}

named!(test_sp<&str, Vec<&str> >,
    ws!(separated_list!(
        tag!(","),
        alphanumeric
    ))
);

named!(filter_arguments< &str, Vec<Argument> >,
    separated_list!(
        complete!(tag!(",")),
        complete!(map!(value, argument_from_token))
    )
);

fn filter_from_tuple((ident, arguments): (Token, Vec<Argument>)) -> FilterPrototype {
    match ident {
        Identifier(ref x) => FilterPrototype::new(x, arguments),
        x => panic!("shouldn't happen, called with {:?}", x),
    }
}

named!(filter<&str, FilterPrototype>,
    map!(
        do_parse!(
            ident: identifier >>
            args: map!(
                opt!(
                    complete!(do_parse!(
                        char!(':') >>
                        args: filter_arguments >>
                        (args)
                    ))
                ), |args: Option<_>| args.unwrap_or(vec![])
            ) >>
            (ident, args)
        ),
        filter_from_tuple
    )
);

named!(variable<&str, Output>,
    map!(
        do_parse!(
            val: map!(value, argument_from_token) >>
            filters: many0!(
                complete!(do_parse!(
                    pipe >>
                    filter: filter >>
                    (filter)
                ))
            ) >>
            (val, filters)
        ), |(val, filters)| {
            Output::new(val, filters)
        }
    )
);

//named!(variable<Output>,
//    do_parse!(
//        ident: identifier! >>
//        filters: filter! >>
//
//    )
//);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_literals() {
        assert_eq!(literal("\"toto\""),
                   IResult::Done("", StringLiteral("toto".to_string())));
        assert_eq!(literal("'toto'"),
                   IResult::Done("", StringLiteral("toto".to_string())));
        assert!(literal("toto").is_err());

        assert_eq!(literal("1234"),
                   IResult::Done("", NumberLiteral(1234.0)));
        assert_eq!(literal("1234.123412341234"),
                   IResult::Done("", NumberLiteral(1234.1234)));
        assert_eq!(literal("1234,123412341234"),
                   IResult::Done(",123412341234", NumberLiteral(1234.0)));
        assert_eq!(literal("1234,'123412341234'"),
                   IResult::Done(",'123412341234'", NumberLiteral(1234.0)));

        assert_eq!(literal("false"),
                   IResult::Done("", BooleanLiteral(false)));
        assert_eq!(literal("true"),
                   IResult::Done("", BooleanLiteral(true)));

        assert_eq!(literal("nil"), IResult::Done("", NilLiteral));
        assert_eq!(literal("null"), IResult::Done("", NilLiteral));
    }

    #[test]
    fn parses_identifiers() {
        assert_eq!(identifier("toto"),
                   IResult::Done("", Identifier("toto".to_string())));
        assert_eq!(identifier("toto_foo-bar"),
                   IResult::Done("", Identifier("toto_foo-bar".to_string())));
    }

    #[test]
    fn parses_comparison_operators() {
        assert_eq!(comparison_operator("contains"),
                   IResult::Done("", Comparison(Contains)));
        assert_eq!(comparison_operator("contains_the_identifier"),
                   IResult::Error(Err::Position(ErrorKind::Alt, "contains_the_identifier")));

        assert_eq!(comparison_operator(">="),
                   IResult::Done("", Comparison(GreaterThanEquals)));
        assert_eq!(comparison_operator(">"),
                   IResult::Done("", Comparison(GreaterThan)));
        assert_eq!(comparison_operator("<="),
                   IResult::Done("", Comparison(LessThanEquals)));
        assert_eq!(comparison_operator("<"),
                   IResult::Done("", Comparison(LessThan)));
        assert_eq!(comparison_operator("=="),
                   IResult::Done("", Comparison(Equals)));
        assert_eq!(comparison_operator("!="),
                   IResult::Done("", Comparison(NotEquals)));
        assert_eq!(comparison_operator("<>"),
                   IResult::Done("", Comparison(NotEquals)));
    }

    #[test]
    fn parses_variables() {
        assert_eq!(filter_arguments("'1',2,'3'"), IResult::Done("", vec![
            Argument::Val(Value::str("1")),
            Argument::Val(Value::Num(2.0)),
            Argument::Val(Value::str("3")),
        ]));

        assert_eq!(filter("foo:'1',2,'3'"),
                   IResult::Done("", FilterPrototype::new("foo", vec![
                       Argument::Val(Value::str("1")),
                       Argument::Val(Value::Num(2.0)),
                       Argument::Val(Value::str("3")),
                   ])));
        assert_eq!(variable("abc|def:'1',2,'3'|blabla"),
                  IResult::Done("", Output::new(Argument::Var(Variable::new("abc")),
                              vec![FilterPrototype::new("def",
                                              vec![
                                                  Argument::Val(Value::str("1")),
                                                  Argument::Val(Value::Num(2.0)),
                                                  Argument::Val(Value::str("3")),
                                              ]),
                         FilterPrototype::new("blabla", vec![])])));
    }
}
