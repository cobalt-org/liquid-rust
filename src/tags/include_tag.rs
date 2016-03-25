
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
use std::path::Path;

struct Include {
    partial: Template,
}

impl Renderable for Include {
    fn render(&self, mut context: &mut Context) -> Result<Option<String>> {
        self.partial.render(&mut context)
    }
}

fn parse_partial(path: &str, options: &LiquidOptions) -> Result<Template> {
    let path = Path::new(&path);

    // check if file exists
    if !path.exists() {
        return Error::parser_msg(&format!("{:?} does not exist", path));
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return Error::parser_msg(&format!("[std::io::Error] {}", e)),
    };

    let mut content = String::new();
    if let Err(e) = file.read_to_string(&mut content) {
        return Error::parser_msg(&format!("[std::io::Error] {}", e));
    }

    let tokens = try!(lexer::tokenize(&content));
    parser::parse(&tokens, &options).map(Template::new)
}

pub fn include_tag(_tag_name: &str,
                   arguments: &[Token],
                   options: &LiquidOptions)
                   -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    let path = match args.next() {
        Some(&Token::Identifier(ref path)) => path,
        arg => return Error::parser("Identifier", arg),
    };

    Ok(Box::new(Include { partial: try!(parse_partial(&path, &options)) }))
}

#[cfg(test)]
mod test {
    use context::Context;
    use Renderable;
    use parse;

    #[test]
    fn include_tag() {
        let text = "{% include tests/fixtures/additional/default.liquid %}";
        let template = parse(text, Default::default()).unwrap();

        let mut context = Context::new();
        assert_eq!(template.render(&mut context).unwrap(),
                   Some("hello, world!\n".to_owned()));
    }
}
