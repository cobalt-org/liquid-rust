use Renderable;
use Value;
use std::collections::HashMap;

pub struct Text{
    text: String
}

impl Renderable for Text {
    fn render (&self, context: &HashMap<String, Value>) -> Option<String>{
        Some(self.text.to_string())
    }
}

impl Text {
    pub fn new(text: &str) -> Text {
        Text{text: text.to_string()}
    }
}

