
use Renderable;
use context::Context;
use token::Token;
use LiquidOptions;
use error::{Result, Error};

use std::fs::File;
use std::io::Read;
use std::path::Path;

struct Include {
    partial_path: String,
}

impl Renderable for Include {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        let path_str = self.partial_path.clone();
        let path = Path::new(&path_str);

        // check if file exists
        if !path.exists() {
            return Error::renderer(&format!("{:?} does not exist", path));
        }

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => return Error::renderer(&format!("[std::io::Error] {}", e)),
        };

        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => Ok(Some(content)),
            Err(e) => Error::renderer(&format!("[std::io::Error] {}", e)),
        }
    }
}

pub fn include_tag(_tag_name: &str,
                   arguments: &[Token],
                   _options: &LiquidOptions)
                   -> Result<Box<Renderable>> {
    let mut args = arguments.iter();

    match args.next() {
        Some(&Token::Identifier(ref path)) => Ok(Box::new(Include { partial_path: path.clone() })),
        arg => Error::parser("Token::Identifier", arg),
    }
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
