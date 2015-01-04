use Renderable;
use Context;

pub struct Text{
    text: String
}

impl Renderable for Text {
    fn render (&self, _context: &Context) -> Option<String>{
        Some(self.text.to_string())
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text{text: text.to_string()}
    }
}

