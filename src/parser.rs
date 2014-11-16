use Renderable;
use variable::Variable;
use lexer::Element;

pub fn parse<'a> (tokens: Vec<Element>) -> Vec<Box<Renderable + 'a>> {
    vec![box Variable::new("wat") as Box<Renderable>]
}
