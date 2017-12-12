use Context;
use LiquidOptions;
use error::{Result, Error};

use syntax::Renderable;
use syntax::Template;
use syntax::Token;
use syntax::parse;
use syntax::tokenize;

struct Include {
    partial: Template,
}

impl Renderable for Include {
    fn render(&self, mut context: &mut Context) -> Result<Option<String>> {
        self.partial.render(&mut context)
    }
}

fn parse_partial(name: &str, options: &LiquidOptions) -> Result<Template> {
    let content = options.include_source.include(name)?;

    let tokens = tokenize(&content)?;
    parse(&tokens, options).map(Template::new)
}

pub fn include_tag(_tag_name: &str,
                   arguments: &[Token],
                   options: &LiquidOptions)
                   -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    let name = match args.next() {
        Some(&Token::StringLiteral(ref name)) => name,
        Some(&Token::Identifier(ref s)) => s,
        arg => return Error::parser("String Literal", arg),
    };


    Ok(Box::new(Include { partial: try!(parse_partial(name, options)) }))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path;
    use FilesystemInclude;
    use super::super::super::parse;

    fn options() -> LiquidOptions {
        LiquidOptions {
            include_source: Box::new(FilesystemInclude::new(path::PathBuf::from("tests/fixtures/input"))),
            ..Default::default()
        }
    }

    #[test]
    fn include_tag() {
        let text = "{% include 'example.txt' %}";
        let template = parse(text, options()).unwrap();

        let mut context = Context::new();
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("5 wot wot\n".to_owned()));
    }

    #[test]
    fn include_non_string() {
        let text = "{% include example.txt %}";
        let template = parse(text, options()).unwrap();

        let mut context = Context::new();
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("5 wot wot\n".to_owned()));
    }

    #[test]
    fn no_file() {
        let text = "{% include 'file_does_not_exist.liquid' %}";
        let output = parse(text, options());

        assert!(output.is_err());
        if let Err(Error::Other(val)) = output {
            assert!(val.contains("file_does_not_exist.liquid\" does not exist"));
        } else {
            panic!("output should be err::other");
        }
    }
}
