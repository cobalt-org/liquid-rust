
use Renderable;
use context::Context;
use token::Token;
use LiquidOptions;
use template::Template;
use parser;
use lexer;
use error::{Result, Error};

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

struct Include {
    partial: Template,
}

impl Renderable for Include {
    fn render(&self, mut context: &mut Context) -> Result<Option<String>> {
        self.partial.render(&mut context)
    }
}

fn parse_partial<P: AsRef<Path>>(path: P, options: &LiquidOptions) -> Result<Template> {
    let file_system = options.file_system.clone().unwrap_or_else(PathBuf::new);
    let path = file_system.join(path);

    // check if file exists
    if !path.exists() {
        return Err(Error::from(&*format!("{:?} does not exist", path)));
    }

    let mut file = try!(File::open(path));

    let mut content = String::new();
    try!(file.read_to_string(&mut content));

    let tokens = try!(lexer::tokenize(&content));
    parser::parse(&tokens, options).map(Template::new)
}

pub fn include_tag(_tag_name: &str,
                   arguments: &[Token],
                   options: &LiquidOptions)
                   -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    let path = match args.next() {
        Some(&Token::StringLiteral(ref path)) => path,
        Some(&Token::Identifier(ref s)) => s,
        arg => return Error::parser("String Literal", arg),
    };


    Ok(Box::new(Include { partial: try!(parse_partial(&path, &options)) }))
}

#[cfg(test)]
mod test {
    use context::Context;
    use Renderable;
    use parse;
    use error::Error;
    use LiquidOptions;
    use std::path::PathBuf;

    fn options() -> LiquidOptions {
        LiquidOptions {
            file_system: Some(PathBuf::from("tests/fixtures/input")),
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
            assert_eq!(val,
                       "\"tests/fixtures/input/file_does_not_exist.liquid\" does not exist"
                           .to_owned());
        } else {
            assert!(false);
        }
    }
}
