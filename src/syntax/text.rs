use Context;
use error::Result;

use super::Renderable;

pub struct Text {
    text: String,
}

impl Renderable for Text {
    fn render(&self, _context: &mut Context) -> Result<Option<String>> {
        Ok(Some(self.text.to_owned()))
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text { text: text.to_owned() }
    }
}
