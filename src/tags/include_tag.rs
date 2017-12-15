use error::{Result, Error};

use syntax::Context;
use syntax::LiquidOptions;
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


    Ok(Box::new(Include { partial: parse_partial(name, options)? }))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path;
    use tags;
    use filters;
    use syntax;

    fn options() -> LiquidOptions {
        let mut options = LiquidOptions::default();
        options.include_source = Box::new(syntax::FilesystemInclude::new(path::PathBuf::from("tests/fixtures/input")));
        options.tags.insert("include".to_owned(),
                            Box::new(syntax::FnTagParser::new(include_tag)));
        options.blocks.insert("comment".to_owned(),
                              Box::new(syntax::FnBlockParser::new(tags::comment_block)));
        options.blocks.insert("if".to_owned(),
                              Box::new(syntax::FnBlockParser::new(tags::if_block)));
        options
    }

    #[test]
    fn include_tag_quotes() {
        let text = "{% include 'example.txt' %}";
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options())
            .map(syntax::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.add_filter("size", Box::new(filters::size));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("5 wot wot\n".to_owned()));
    }

    #[test]
    fn include_non_string() {
        let text = "{% include example.txt %}";
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options())
            .map(syntax::Template::new)
            .unwrap();

        let mut context = Context::new();
        context.add_filter("size", Box::new(filters::size));
        let output = template.render(&mut context).unwrap();
        assert_eq!(output, Some("5 wot wot\n".to_owned()));
    }

    #[test]
    fn no_file() {
        let text = "{% include 'file_does_not_exist.liquid' %}";
        let tokens = syntax::tokenize(&text).unwrap();
        let template = syntax::parse(&tokens, &options()).map(syntax::Template::new);

        assert!(template.is_err());
        if let Err(Error::Other(val)) = template {
            assert!(val.contains("file_does_not_exist.liquid\" does not exist"));
        } else {
            panic!("output should be err::other");
        }
    }
}
